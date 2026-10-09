//! 詳細の表示(`impl App` の続き。NV-6・SR-16): `K` で選んだ行の全プロパティを縦に並べ、長い値は折り返して
//! 全文を見せる。Enter で選んだプロパティの編集(入力ボックスはその値の欄に出す)、Esc と `K` で閉じる。
//! プロパティの下に、本文(フロントマターのあと)の先頭の最大 20 行を読むだけで見せる(折り返さず幅で切る。選べない)。

use super::app::App;
use super::cell::shown;
use super::input::new_value_text;
use super::keymap::{Action, Mode};
use super::view::{footer, message};
use super::width::{fit, sanitize, take, width, Align};
use mdgrid::base::Shown;
use mdgrid::i18n::Msg;
use mdgrid::source::{RowId, Value};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;

/// 詳細の表示の状態。
pub(crate) struct Detail {
    pub row: RowId,
    /// 選んだプロパティの添字。
    pub sel: usize,
    /// 見えている先頭の行(本文の行)。
    pub top: usize,
}

/// 本文の1行: どのプロパティか、そのプロパティの最初の行か、見せる文字(印・キーの欄・値の欄)。
/// ノートの本文の節の行は prop が `NOT_PROP`(選べない)。
struct BodyLine {
    prop: usize,
    first: bool,
    /// ノートの本文の節の見出し(太字)。
    head: bool,
    text: String,
}

/// ノートの本文の節の行の prop(どのプロパティでもない)。
const NOT_PROP: usize = usize::MAX;

/// ノートの本文の節に見せる行の数の上限(NV-6)。超えたら「…」の行を足す。
const NOTE_BODY_MAX: usize = 20;

/// ノートの本文の節の行(見せる文字。制御文字は置き換える)。先頭と末尾の空行は除く。本文が無ければ空。
fn note_body_lines(body: &str) -> Vec<String> {
    let lines: Vec<&str> = body.split('\n').map(|l| l.trim_end_matches('\r')).collect();
    let blank = |l: &&str| l.trim().is_empty();
    let Some(start) = lines.iter().position(|l| !blank(l)) else {
        return Vec::new();
    };
    let end = lines
        .iter()
        .rposition(|l| !blank(l))
        .map_or(start, |e| e + 1);
    let lines = &lines[start..end];
    let mut out: Vec<String> = lines
        .iter()
        .take(NOTE_BODY_MAX)
        .map(|l| sanitize(l))
        .collect();
    if lines.len() > NOTE_BODY_MAX {
        out.push("…".into());
    }
    out
}

/// 値を幅 `w` で折り返す(改行でも分ける)。空なら1行の空。
fn wrap(s: &str, w: usize) -> Vec<String> {
    let w = w.max(1);
    let mut out = Vec::new();
    for para in s.split('\n') {
        let mut rest = sanitize(para.trim_end_matches('\r'));
        if rest.is_empty() {
            out.push(String::new());
            continue;
        }
        while !rest.is_empty() {
            let mut head = take(&rest, w);
            if head.is_empty() {
                head = rest.graphemes(true).next().unwrap_or_default().to_string();
            }
            rest.drain(..head.len());
            out.push(head);
        }
    }
    out
}

impl App {
    /// `K`: 選んだ行の詳細の表示を開く。
    pub(crate) fn open_detail(&mut self) {
        let Some(row) = self.cur_row() else {
            self.message = Some(Msg::DetailNoRow.text().into());
            return;
        };
        let props = self.detail_props(&row);
        let sel = self
            .cols
            .get(self.col)
            .and_then(|c| props.iter().position(|p| p == c))
            .unwrap_or(0);
        self.detail = Some(Detail { row, sel, top: 0 });
        self.set_mode(Mode::Detail);
        self.detail_scroll();
    }

    /// 詳細に出すプロパティ: 表の列(隠した列も)のあとに、表に無いこのノートのキー。
    pub(crate) fn detail_props(&self, row: &RowId) -> Vec<String> {
        let mut out: Vec<String> = self.cols.clone();
        for (c, _, _) in &self.hidden {
            if !out.contains(c) {
                out.push(c.clone());
            }
        }
        for c in self.src.columns() {
            if !out.contains(&c) && self.src.get(row, &c).value.is_some() {
                out.push(c);
            }
        }
        out
    }

    /// 詳細に出す値の全文(改行も保つ)。ためた値は `*`、読むだけは `#` を先頭に。
    pub(crate) fn detail_text(&self, row: &RowId, col: &str) -> String {
        if let Some(nv) = self.changes.pending(row, col) {
            return format!("*{}", new_value_text(nv));
        }
        let s = shown(self, row, col);
        if let Shown::Prop(c) = self.cell(row, col) {
            if let Some(Value::Str(v)) = &c.value {
                if !v.is_empty() {
                    let mark = if s.locked.is_some() { "#" } else { "" };
                    return format!("{mark}{v}");
                }
            }
        }
        // SR-35: 部品(☑・札)は表の中だけ。詳細は値の文字で見せる。
        let rich = s.part != super::cell::CellPart::None
            || s.text == super::cell::CHECK_ON
            || s.text == super::cell::CHECK_OFF;
        if let (true, Shown::Prop(c)) = (rich, self.cell(row, col)) {
            if let Some(v) = &c.value {
                return super::cell::value_plain(v);
            }
        }
        s.text
    }

    /// キーの欄の幅(長いキーは切る)。
    fn detail_key_w(&self, props: &[String], w: usize) -> usize {
        props
            .iter()
            .map(|p| width(&sanitize(&self.title(p))))
            .max()
            .unwrap_or(1)
            .clamp(4, (w / 3).max(4))
    }

    /// 本文の行(プロパティごとに、キーと折り返した値)。
    fn detail_body(&self, d: &Detail, w: usize) -> Vec<BodyLine> {
        let props = self.detail_props(&d.row);
        let kw = self.detail_key_w(&props, w);
        let vw = w.saturating_sub(kw + 3).max(1);
        let mut out = Vec::new();
        for (i, p) in props.iter().enumerate() {
            let editing = self.mode == Mode::Edit && i == d.sel;
            let vals = if editing {
                vec![String::new()]
            } else {
                wrap(&self.detail_text(&d.row, p), vw)
            };
            for (k, v) in vals.into_iter().enumerate() {
                let key = if k == 0 {
                    fit(&sanitize(&self.title(p)), kw, Align::Left)
                } else {
                    " ".repeat(kw)
                };
                let mark = if k == 0 && i == d.sel { ">" } else { " " };
                out.push(BodyLine {
                    prop: i,
                    first: k == 0,
                    head: false,
                    text: format!("{mark}{key}  {v}"),
                });
            }
        }
        let note = note_body_lines(&self.src.body(&d.row).unwrap_or_default());
        if !note.is_empty() {
            let line = |head: bool, text: String| BodyLine {
                prop: NOT_PROP,
                first: false,
                head,
                text,
            };
            out.push(line(false, String::new()));
            out.push(line(true, format!(" {}", Msg::DetailBody.text())));
            out.extend(note.into_iter().map(|t| line(false, format!(" {t}"))));
        }
        out
    }

    /// 本文の高さ(見出し・下の帯・メッセージ行を除く)。
    fn detail_height(&self) -> usize {
        (self.size.1.saturating_sub(1) as usize)
            .saturating_sub(3)
            .max(1)
    }

    /// 選んだプロパティの最初の行が見えるように流す。
    fn detail_scroll(&mut self) {
        let w = self.size.0.saturating_sub(1) as usize;
        let h = self.detail_height();
        let Some(d) = &self.detail else {
            return;
        };
        let body = self.detail_body(d, w);
        let first = body
            .iter()
            .position(|l| l.prop == d.sel && l.first)
            .unwrap_or(0);
        let last = body.iter().rposition(|l| l.prop == d.sel).unwrap_or(first);
        let mut top = d.top;
        if first < top {
            top = first;
        } else if last >= top + h {
            top = (last + 1 - h).min(first);
        }
        if let Some(d) = &mut self.detail {
            d.top = top;
        }
    }

    /// 詳細の表示の動作(SR-16)。
    pub(crate) fn detail_action(&mut self, action: Action) {
        let Some(d) = &self.detail else {
            return self.set_mode(Mode::Table);
        };
        let n = self.detail_props(&d.row).len();
        let page = self.detail_height();
        let Some(d) = &mut self.detail else {
            return;
        };
        match action {
            Action::Close => {
                self.detail = None;
                self.set_mode(Mode::Table);
                return;
            }
            Action::Down
            | Action::Up
            | Action::PageDown
            | Action::PageUp
            | Action::Top
            | Action::Bottom => {
                d.sel = super::popup::step_sel(d.sel, n, action, page).unwrap_or(d.sel);
            }
            Action::Edit if self.readonly => {
                self.message = Some(super::startup::READONLY.into());
            }
            Action::Edit => {
                let (row, sel) = (d.row.clone(), d.sel);
                if let Some(col) = self.detail_props(&row).get(sel).cloned() {
                    self.open_edit_at(row, col);
                }
            }
            Action::Help => self.open_help(),
            _ => {}
        }
        self.detail_scroll();
    }

    /// 詳細の表示から開いた入力の Tab / Shift+Tab: 確定して、下・上のプロパティを選ぶ。扱ったら true。
    pub(crate) fn detail_input_action(&mut self, action: Action) -> bool {
        if self.detail.is_none() || !matches!(action, Action::CommitNext | Action::CommitPrev) {
            return false;
        }
        self.input_action(Action::Commit);
        if self.mode == Mode::Detail {
            let a = if action == Action::CommitNext {
                Action::Down
            } else {
                Action::Up
            };
            self.detail_action(a);
        }
        true
    }
}

/// 入力ボックスの位置(詳細の表示から編集しているとき): 桁・行・幅。
pub(crate) fn detail_input_at(app: &App, w: usize) -> Option<(usize, usize, usize)> {
    if app.mode != Mode::Edit {
        return None;
    }
    detail_anchor(app, w)
}

/// 選んだプロパティの値の欄の位置(桁・行・幅)。見えていなければ None(リストの選択の窓の置き場。CE-16)。
pub(crate) fn detail_anchor(app: &App, w: usize) -> Option<(usize, usize, usize)> {
    let d = app.detail.as_ref()?;
    let body = app.detail_body(d, w);
    let i = body.iter().position(|l| l.prop == d.sel && l.first)?;
    if i < d.top || i >= d.top + app.detail_height() {
        return None;
    }
    let props = app.detail_props(&d.row);
    let x = 1 + app.detail_key_w(&props, w) + 2;
    Some((x, 1 + i - d.top, w.saturating_sub(x).max(1)))
}

/// 入力の見せる文字とカーソルの桁(カーソルが見えるように左を落とす)。新しいノートの欄(new_note.rs)も使う。
pub(crate) fn input_view(app: &App, room: usize) -> Option<(String, usize)> {
    let input = app.input.as_ref()?;
    let mut before = sanitize(&input.text[..input.cursor]);
    while width(&before) + 1 > room && !before.is_empty() {
        let first = before.graphemes(true).next().map(str::len).unwrap_or(0);
        before.drain(..first);
    }
    let after = sanitize(&input.text[input.cursor..]);
    let cx = width(&before).min(room.saturating_sub(1));
    Some((take(&format!("{before}{after}"), room), cx))
}

/// (x, y) が詳細の表示の入力ボックスの中か(クリックで確定しない。CE-1)。
pub(crate) fn in_detail_input(app: &App, x: u16, y: u16) -> bool {
    let w = app.size.0.saturating_sub(1) as usize;
    detail_input_at(app, w).is_some_and(|(ix, iy, room)| {
        y as usize == iy && (x as usize) >= ix && (x as usize) < ix + room
    })
}

/// 端末のカーソルの位置(詳細の表示から編集しているとき。SR-17)。
pub(crate) fn detail_cursor(app: &App) -> Option<(u16, u16)> {
    let w = app.size.0.saturating_sub(1) as usize;
    let (x, y, room) = detail_input_at(app, w)?;
    let (_, cx) = input_view(app, room)?;
    Some(((x + cx) as u16, y as u16))
}

/// 詳細の表示の画面: 見出し・本文・下の帯・メッセージ行。
pub(crate) fn render_detail(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(d) = &app.detail else {
        return Vec::new();
    };
    let body = app.detail_body(d, w);
    let props = app.detail_props(&d.row);
    let body_h = h.saturating_sub(3);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let head = Msg::DetailHead.fill(&[
        &sanitize(&app.src.label(&d.row)),
        &(d.sel + 1).min(props.len()),
        &props.len(),
    ]);
    let mut lines = vec![Line::from(Span::styled(fit(&head, w, Align::Left), bold))];
    let input = detail_input_at(app, w);
    for (k, l) in body.iter().enumerate().skip(d.top).take(body_h) {
        let y = 1 + k - d.top;
        if let Some((x, _, room)) = input.filter(|(_, iy, _)| *iy == y) {
            // 狭い端末では値の欄が幅の外に出る。行は幅で切る(SR-9)。
            let x = x.min(w);
            let room = room.min(w - x);
            let (t, _) = input_view(app, room).unwrap_or_default();
            let st = Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
            lines.push(Line::from(vec![
                Span::raw(fit(&l.text, x, Align::Left)),
                Span::styled(fit(&t, room, Align::Left), st),
            ]));
            continue;
        }
        // SR-33: モダンな見た目では、選んでいる項目の行は背景の色(表と同じ)、見出し(本文)はアクセントの色。
        let st = match (super::look::look(app), l.head, l.prop == d.sel) {
            (Some(lk), true, _) => lk.key(),
            (Some(lk), false, true) => lk.selected(),
            (None, true, _) | (None, _, true) => bold,
            _ => Style::default(),
        };
        lines.push(Line::from(Span::styled(fit(&l.text, w, Align::Left), st)));
    }
    while lines.len() < h.saturating_sub(2) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(h.saturating_sub(2));
    super::list::overlay(app, &mut lines, w);
    super::listpick::overlay(app, &mut lines, w);
    lines.push(footer(app, w));
    lines.push(message(app, w));
    lines.truncate(h);
    lines
}
