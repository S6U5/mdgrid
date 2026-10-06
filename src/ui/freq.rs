//! 列の値の頻度表(NV-9): 選んでいる列の値ごとの件数(リストは要素ごと、空は「(空)」)を、件数の多い順
//! (同じ数は値の順)に、全行に対する割合を添えて、表の上に重ねる窓で見せる。↑↓ で選び、Enter でその値の行
//! だけに絞り(NV-8 の `,` と同じ同じ値の絞り込み。ヘッダーに出る)、Esc で何もせず閉じる。
//! 数えるのは今の行(ビューの設定・簡易の絞り込み・同じ値の絞り込みのあと)で、Enter は今の同じ値の絞り込みに
//! 条件を重ねる(置き換えない)。だから窓の件数は、Enter のあとに出る行の数と必ず同じ。
//! 読み込みの途中は開かず、開いている間に表を組み直したら閉じる。
//!
//! 窓の形は操作の一覧(menu.rs)と同じ: 選んだセルの下(入らなければ上)に枠の窓を重ね、窓の外の表は見せたまま。
//! 窓が入らない端末では開かず、理由を出す。数えるのは開いたときの今のビューの行(絞り込みのあと)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::list::splice;
use super::menu::{anchor, FRAME, MIN_VIS};
use super::nav::{shown_value, Same};
use super::view::data_y;
use super::width::{fit, sanitize, take, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::settings::value_counts;
use mdgrid::source::Value;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// 開いている頻度表。
#[derive(Debug, Clone)]
pub(crate) struct Freq {
    /// 数えた列。
    pub col: String,
    /// (値, 件数)。値の None は空。件数の多い順、同じ数は値の順(空は後ろ)。
    pub items: Vec<(Option<String>, usize)>,
    /// 数えた行の数(割合の分母)。
    pub total: usize,
    /// 選んでいる項目。
    pub sel: usize,
}

/// 今のビューの行(絞り込みのあと)の、列 `col` の値ごとの件数(NV-9。NV-19 の値の一覧と同じ数え方)。
pub(crate) fn counts(app: &App, col: &str) -> Vec<(Option<String>, usize)> {
    let vals: Vec<_> = app.rows.iter().map(|r| app.setting_value(r, col)).collect();
    value_counts(vals.iter().map(|v| v.as_ref()))
}

/// 割合(整数の百分率。四捨五入)。
fn percent(n: usize, total: usize) -> usize {
    (n * 100 + total / 2).checked_div(total).unwrap_or(0)
}

/// 項目の値の見せ方(空は「(空)」)。
fn value_text(v: &Option<String>) -> String {
    sanitize(&shown_value(v.as_deref().unwrap_or("")))
}

impl App {
    /// 頻度表を開く(表のモードから)。列が無い・数える行が無い・窓が入らないときは開かず、理由を出す。
    pub(crate) fn open_freq(&mut self) {
        if self.mode != Mode::Table {
            return;
        }
        let Some(col) = self.cols.get(self.col).cloned() else {
            self.message = Some(Msg::FreqNoColumn.text().into());
            return;
        };
        // BV-16: 読み込みの途中は行がまだ揃わないので数えない。
        if !self.progress.done {
            self.message = Some(Msg::FreqLoading.text().into());
            return;
        }
        if self.rows.is_empty() {
            self.message = Some(Msg::FreqNoRows.text().into());
            return;
        }
        let items = counts(self, &col);
        self.freq = Some(Freq {
            col,
            items,
            total: self.rows.len(),
            sel: 0,
        });
        if window(self).is_none() {
            self.freq = None;
            self.message = Some(Msg::FreqNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::Freq);
        // 同じ値の絞り込みが効いていれば、その中で数えて、Enter はさらに絞ることを知らせる。
        if !self.same.is_empty() {
            self.message = Some(Msg::FreqStacked.text().into());
        }
    }

    /// 表を組み直すとき(外の変更の読み直しなど): 開いていれば閉じて、理由を出す。
    pub(crate) fn close_freq_on_regrid(&mut self) {
        if self.mode == Mode::Freq {
            self.close_freq();
            self.message = Some(Msg::FreqClosed.text().into());
        }
    }

    /// 開いたまま端末が変わって窓が入らなくなったら閉じ、開くときと同じ理由を出す。
    pub(crate) fn close_freq_if_no_room(&mut self) {
        if self.mode == Mode::Freq && window(self).is_none() {
            self.close_freq();
            self.message = Some(Msg::FreqNoRoom.text().into());
        }
    }

    /// 何もせず閉じて表に戻る。
    pub(crate) fn close_freq(&mut self) {
        self.freq = None;
        self.set_mode(Mode::Table);
    }

    /// 頻度表のモードの動作(↑↓ で選び、Enter で絞り、Esc で閉じる)。
    pub(crate) fn freq_action(&mut self, action: Action) {
        let Some(f) = self.freq.as_mut() else {
            return self.close_freq();
        };
        let last = f.items.len().saturating_sub(1);
        match action {
            Action::Up => f.sel = f.sel.saturating_sub(1),
            Action::Down => f.sel = (f.sel + 1).min(last),
            Action::Top => f.sel = 0,
            Action::Bottom => f.sel = last,
            Action::Run => {
                let i = f.sel;
                self.run_freq_item(i);
            }
            Action::Close => self.close_freq(),
            _ => {}
        }
    }

    /// 項目 `i` の値の行だけに絞って閉じる(NV-8 の `,` と同じ同じ値の絞り込み。リストは要素で、空は値の無い行)。
    fn run_freq_item(&mut self, i: usize) {
        let Some(f) = self.freq.take() else {
            return self.close_freq();
        };
        self.close_freq();
        let Some((key, _)) = f.items.get(i) else {
            return;
        };
        let v = key.clone().unwrap_or_default();
        let label = format!("{} = {}", self.title(&f.col), shown_value(&v));
        // リストの列か(ヘッダーを「含む」の形にする)。
        let list = self
            .rows
            .iter()
            .any(|r| matches!(self.setting_value(r, &f.col), Some(Value::List(_))));
        let cond = Same {
            col: f.col,
            value: v,
            by_keys: true,
            list,
        };
        // 今の同じ値の絞り込みに重ねる(数えた行 = 今の行なので、件数と絞ったあとの行の数が合う)。
        if !self.same.contains(&cond) {
            self.same.push(cond);
        }
        self.message = Some(Msg::SameFilterOn.fill(&[&label]));
        self.relayout();
    }

    /// 頻度表の上の左クリック: 項目ならその値で絞って閉じ、窓の外なら閉じるだけ。縁では何もしない。
    pub(crate) fn freq_click(&mut self, x: u16, y: u16) {
        self.message = None;
        match hit(self, x, y) {
            None => self.close_freq(),
            Some(Some(i)) => {
                self.run_freq_item(i);
                self.refresh_if_needed();
            }
            Some(None) => {}
        }
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    /// 値の欄の幅。
    vw: usize,
    /// 見せる最初の項目と項目の数。
    start: usize,
    vis: usize,
}

/// 件数と割合の欄の文字。
fn count_text(n: usize) -> String {
    n.to_string()
}

pub(crate) fn pct_text(n: usize, total: usize) -> String {
    // 1% 未満(0 件を除く)は小数1桁で(整数に丸めると `0%` になり、無いように見える)。
    let tenths = (n * 1000 + total / 2).checked_div(total).unwrap_or(0);
    // 小数1桁に丸めて 1.0% に届くなら、整数の形(`1%`)にする。
    if n > 0 && n * 100 < total && tenths < 10 {
        return if tenths == 0 {
            "(<0.1%)".into()
        } else {
            format!("(0.{tenths}%)")
        };
    }
    format!("({}%)", percent(n, total))
}

/// 窓の題(上の縁に出す)。
fn title(app: &App, f: &Freq) -> String {
    sanitize(&Msg::FreqTitle.fill(&[&app.title(&f.col)]))
}

/// 窓の置き場所(menu.rs の一覧と同じ決め方)。窓が入らなければ None。
fn window(app: &App) -> Option<Geom> {
    let f = app.freq.as_ref()?;
    let n = f.items.len();
    if n == 0 {
        return None;
    }
    // 描く幅は右端の1桁を除く。下の帯より上の行は、最下行と下の帯・メッセージ行を除く(mod.rs の draw)。
    let w = app.size.0.saturating_sub(1) as usize;
    let limit = (app.size.1.saturating_sub(1) as usize).saturating_sub(2);
    let (ax, ay) = anchor(app).unwrap_or((0, data_y(app)));
    let below_room = limit.saturating_sub(ay + 1);
    let above_room = ay.min(limit);
    let below = below_room >= n + FRAME || below_room >= above_room;
    let room = if below { below_room } else { above_room };
    if room < FRAME + MIN_VIS.min(n) || w < 8 {
        return None;
    }
    let vis = n.min(room - FRAME);
    let cw = f
        .items
        .iter()
        .map(|(_, c)| width(&count_text(*c)))
        .max()
        .unwrap_or(0);
    let pw = f
        .items
        .iter()
        .map(|(_, c)| width(&pct_text(*c, f.total)))
        .max()
        .unwrap_or(0);
    let lw = f
        .items
        .iter()
        .map(|(v, _)| width(&value_text(v)))
        .max()
        .unwrap_or(0);
    // 印・値・件数・割合の行の、値の欄より外の幅。
    let rest = 1 + 2 + cw + 2 + pw + 1;
    let inner = (rest + lw).max(width(&title(app, f)));
    let iw = (inner + 2).min(w);
    // 狭い端末では値の欄を縮める(件数と割合は切らない)。
    let vw = (iw - 2).saturating_sub(rest).min(lw);
    let sel = f.sel.min(n - 1);
    let start = (sel + 1).saturating_sub(vis);
    let height = vis + FRAME;
    Some(Geom {
        x: ax.min(w - iw),
        top: if below { ay + 1 } else { ay - height },
        iw,
        vw,
        start,
        vis,
    })
}

/// (x, y) にある窓の中身: 窓の外なら None、窓の中なら項目の添字(縁なら None の中身)。
fn hit(app: &App, x: u16, y: u16) -> Option<Option<usize>> {
    let g = window(app)?;
    let (x, y) = (x as usize, y as usize);
    let height = g.vis + FRAME;
    if x < g.x || x >= g.x + g.iw || y < g.top || y >= g.top + height {
        return None;
    }
    let k = y - g.top;
    if k == 0 || k > g.vis {
        return Some(None);
    }
    Some(Some(g.start + k - 1))
}

/// 下の帯より上の行(`lines`)に頻度表の窓を重ねる(操作の一覧と同じ見せ方)。窓の外の表はそのまま見せる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Freq {
        return;
    }
    let (Some(f), Some(g)) = (app.freq.as_ref(), window(app)) else {
        return;
    };
    let tw = g.iw - 2;
    let cw = f
        .items
        .iter()
        .map(|(_, c)| width(&count_text(*c)))
        .max()
        .unwrap_or(0);
    let pw = f
        .items
        .iter()
        .map(|(_, c)| width(&pct_text(*c, f.total)))
        .max()
        .unwrap_or(0);
    let sel = f.sel.min(f.items.len() - 1);
    let mut out: Vec<(String, Style)> = Vec::new();
    let t = take(&title(app, f), tw);
    out.push((
        format!("+{t}{}+", "-".repeat(tw - width(&t))),
        Style::default(),
    ));
    for (i, (v, c)) in f.items.iter().enumerate().skip(g.start).take(g.vis) {
        let st = if i == sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let text = format!(
            "{}{}  {}  {} ",
            if i == sel { ">" } else { " " },
            fit(&value_text(v), g.vw, Align::Left),
            fit(&count_text(*c), cw, Align::Right),
            fit(&pct_text(*c, f.total), pw, Align::Right),
        );
        out.push((format!("|{}|", fit(&text, tw, Align::Left)), st));
    }
    out.push((format!("+{}+", "-".repeat(tw)), Style::default()));
    for (k, (t, st)) in out.into_iter().enumerate() {
        if let Some(line) = lines.get_mut(g.top + k) {
            *line = splice(line, g.x, Span::styled(t, st), g.iw, w);
        }
    }
}
