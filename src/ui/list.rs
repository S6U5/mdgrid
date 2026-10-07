//! 候補のリスト(`impl App` の続き。CE-3・CE-4・CE-1・CV-2)。
//! テキストの列で、読み込んだノート全部の異なる値が上限以下なら、入力ボックスの下(入らなければ上)にリストを重ねる。
//! 今の値に `*`、空は「なし」。↑↓(Ctrl+P / Ctrl+N)で選び Enter で決める。文字・BS・←→ を打つと自由入力に切り替わり、
//! 打った文字を含む候補だけを選んでいない形で出し続け、↑↓ でその候補の中から選ぶ(当たらなければ全部のリストに戻る)。チェックボックスの列で型の合わない値(CV-2)と Tab で着いたときは true・false・なし のリスト。

use super::app::App;
use super::input::{input_box, new_value_text};
use super::keymap::Mode;
use super::view::visible_layout;
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, Value};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;

/// リストの1項目。
pub(crate) struct Item {
    pub value: NewValue,
    pub text: String,
    /// 今の値(印 `*`)。
    pub current: bool,
}

/// 候補のリストの状態。
pub(crate) struct List {
    pub items: Vec<Item>,
    pub sel: usize,
    /// 開いたときの選び(今の値の項目。合わない値(CV-2)なら無し = usize::MAX)。
    pub initial_sel: usize,
    /// ↑↓ かクリックで選びを動かした。動かしていなければ確定で何も書かない(CE-10)。
    pub moved: bool,
    /// 自由入力に切り替えた(確定は入力の文字で。候補は打った文字で絞って、選んでいない形で見せる)。
    pub free: bool,
    /// 自由入力から ↑↓ で戻ったときの、打った文字(この文字を含む候補だけを見せて選ぶ。CE-3)。
    pub filter: Option<String>,
    /// 打った文字で候補を絞るか(セルの入力のリスト。新しいノートの置き場の選びは絞らない)。
    pub narrow: bool,
}

/// 「なし」(Null)の見せ方。`NONE_LABEL.into()` で今の言語の文。
pub(crate) const NONE_LABEL: Msg = Msg::NoneLabel;

/// 値を書く値に。リストや読めない値は None(候補にしない)。
pub(crate) fn to_new(v: &Value) -> Option<NewValue> {
    Some(match v {
        Value::Null => NewValue::Null,
        Value::Str(s) => NewValue::Str(s.clone()),
        Value::Bool(b) => NewValue::Bool(*b),
        Value::Int(i) => NewValue::Int(*i),
        Value::Float(f) => NewValue::Float(*f),
        Value::List(_) | Value::Other => return None,
    })
}

impl List {
    /// 候補から作る。今の値が候補に無ければ(ためた値など)足し、最後に「なし」。選ぶのは今の値の項目。
    /// `fits` が偽(今の値が列の型に合わない。CV-2)なら、今の値は足さず印も付けない(合わない値を選ばせない)。
    pub(crate) fn new(cands: &[Value], cur: Option<&NewValue>, fits: bool) -> List {
        let cur = if fits { cur } else { None };
        let mut items: Vec<Item> = Vec::new();
        for v in cands.iter().filter_map(to_new) {
            if matches!(v, NewValue::Null) || items.iter().any(|i| i.value == v) {
                continue;
            }
            items.push(Item {
                text: new_value_text(&v),
                current: cur == Some(&v),
                value: v,
            });
        }
        if let Some(c) = cur.filter(|c| !matches!(c, NewValue::Null)) {
            if !items.iter().any(|i| &i.value == c) {
                items.push(Item {
                    text: new_value_text(c),
                    current: true,
                    value: c.clone(),
                });
            }
        }
        items.push(Item {
            value: NewValue::Null,
            text: NONE_LABEL.into(),
            current: fits && matches!(cur, None | Some(NewValue::Null)),
        });
        let sel = items.iter().position(|i| i.current).unwrap_or(0);
        List {
            items,
            sel,
            initial_sel: if fits { sel } else { usize::MAX },
            moved: false,
            free: false,
            filter: None,
            narrow: true,
        }
    }

    /// `text` を含む候補(大文字小文字を問わない。「なし」は除く)の添字。`text` が空なら全部。
    pub(crate) fn matches(&self, text: &str) -> Vec<usize> {
        if text.is_empty() {
            return (0..self.items.len()).collect();
        }
        let needle = text.to_lowercase();
        (0..self.items.len())
            .filter(|&k| {
                let it = &self.items[k];
                !matches!(it.value, NewValue::Null) && it.text.to_lowercase().contains(&needle)
            })
            .collect()
    }

    /// リストで見せて選べる項目の添字(絞っていれば当たる候補だけ)。
    pub(crate) fn shown(&self) -> Vec<usize> {
        match &self.filter {
            Some(f) => self.matches(f),
            None => (0..self.items.len()).collect(),
        }
    }

    pub(crate) fn chosen(&self) -> NewValue {
        self.items
            .get(self.sel)
            .map(|i| i.value.clone())
            .unwrap_or(NewValue::Null)
    }
}

impl App {
    /// 出ているリスト(自由入力に切り替えていない)。
    pub(crate) fn active_list(&self) -> Option<&List> {
        let i = self.input.as_ref()?;
        i.list.as_ref().filter(|l| !l.free)
    }

    /// 見せている候補: (リスト, 見せる項目の添字, 選べるか)。リストのときは選べる形、自由入力のときは
    /// 打った文字を含む候補を選べない形で(当たらない・打った文字そのものの1つだけなら None。CE-3)。
    pub(crate) fn list_view(&self) -> Option<(&List, Vec<usize>, bool)> {
        let i = self.input.as_ref()?;
        let l = i.list.as_ref()?;
        if !l.free {
            return Some((l, l.shown(), true));
        }
        if !l.narrow || i.text.is_empty() {
            return None;
        }
        let m = l.matches(&i.text);
        let same = |k: &usize| l.items[*k].text == i.text;
        if m.is_empty() || (m.len() == 1 && same(&m[0])) {
            return None;
        }
        Some((l, m, false))
    }

    /// ↑↓: 候補を選ぶ。自由入力からはリストに戻る(打った文字に当たる候補があれば、その候補だけで最初を選ぶ)。
    pub(crate) fn list_move(&mut self, down: bool) {
        let Some(i) = self.input.as_mut() else {
            return;
        };
        let Some(l) = i.list.as_mut() else {
            return;
        };
        if l.free {
            l.free = false;
            l.filter = None;
            let m = if l.narrow && !i.text.is_empty() {
                l.matches(&i.text)
            } else {
                Vec::new()
            };
            // ↓ は最初の候補、↑ は最後の候補から。選び直したあとに打つと置き換える(CE-3)。
            if let Some(&k) = if down { m.first() } else { m.last() } {
                l.filter = Some(i.text.clone());
                l.sel = k;
                l.moved = true;
                i.touched = true;
                i.fresh = true;
            }
            return;
        }
        l.moved = true;
        i.touched = true;
        let shown = l.shown();
        let at = shown.iter().position(|&k| k == l.sel);
        let next = match at {
            Some(p) if down => shown.get(p + 1).or(shown.last()),
            Some(p) => shown.get(p.saturating_sub(1)),
            None => shown.first(),
        };
        if let Some(&k) = next {
            l.sel = k;
        }
    }

    /// 自由入力に切り替える(文字・BS・←→ を打ったとき)。
    pub(crate) fn list_to_free(&mut self) {
        if let Some(l) = self.input.as_mut().and_then(|i| i.list.as_mut()) {
            l.free = true;
            l.filter = None;
        }
    }

    /// クリックがリストの上なら扱って true(項目なら選んで確定。CE-1)。
    pub(crate) fn list_click(&mut self, x: u16, y: u16) -> bool {
        let (w, h) = (
            self.size.0.saturating_sub(1) as usize,
            self.size.1.saturating_sub(1) as usize,
        );
        let Some(g) = geometry(self, w, h.saturating_sub(2)) else {
            return false;
        };
        let (x, y) = (x as usize, y as usize);
        if x < g.x || x >= g.x + g.iw || y < g.top || y > g.top + g.vis {
            return false;
        }
        let k = if g.below {
            y - g.top
        } else {
            y.wrapping_sub(g.top + 1)
        };
        if k < g.vis {
            let item = self
                .list_view()
                .and_then(|(_, shown, _)| shown.get(g.start + k).copied());
            if let (Some(i), Some(item)) = (self.input.as_mut(), item) {
                i.touched = true;
                if let Some(l) = i.list.as_mut() {
                    // 絞った候補のクリックも、その候補を選んで確定する。
                    l.free = false;
                    l.sel = item;
                    l.moved = true;
                }
            }
            self.apply(super::keymap::Action::Commit);
        }
        true
    }
}

// ---- 描画(純関数) ----

/// リストの置き場所。
struct Geom {
    x: usize,
    /// 箱の一番上の行(下に出すなら最初の項目、上に出すなら縁)。
    top: usize,
    iw: usize,
    /// 見せる最初の項目と数。
    start: usize,
    vis: usize,
    below: bool,
}

/// 入力の位置(桁と行)。表なら入力ボックス、詳細の表示なら値の欄。
fn anchor(app: &App, w: usize) -> Option<(usize, usize)> {
    if app.detail.is_some() {
        return super::detail::detail_input_at(app, w).map(|(x, y, _)| (x, y));
    }
    let (lay, cols) = visible_layout(app);
    input_box(app, &lay, &cols).map(|b| (b.x, b.y))
}

/// `limit` は下の帯より上の行の数。
fn geometry(app: &App, w: usize, limit: usize) -> Option<Geom> {
    if app.mode != Mode::Edit {
        return None;
    }
    let (l, shown, _) = app.list_view()?;
    let (ax, ay) = anchor(app, w)?;
    let n = shown.len();
    let below_room = limit.saturating_sub(ay + 1);
    let above_room = ay.saturating_sub(1);
    let below = below_room > n || below_room >= above_room;
    let room = if below { below_room } else { above_room };
    let vis = n.min(room.saturating_sub(1));
    if vis == 0 {
        return None;
    }
    let text_w = shown
        .iter()
        .map(|&k| width(&sanitize(&l.items[k].text)))
        .max();
    let iw = (text_w.unwrap_or(0) + 5).max(8).min(w);
    if iw < 5 {
        return None;
    }
    let at = shown.iter().position(|&k| k == l.sel).unwrap_or(0);
    let start = (at + 1).saturating_sub(vis);
    Some(Geom {
        x: ax.min(w - iw),
        top: if below { ay + 1 } else { ay - vis - 1 },
        iw,
        start,
        vis,
        below,
    })
}

/// 下の帯より上の行(`lines`)にリストを重ねる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(g) = geometry(app, w, lines.len()) else {
        return;
    };
    let Some((l, shown, pick)) = app.list_view() else {
        return;
    };
    let tw = g.iw - 4;
    let mut rows: Vec<(String, Style)> = Vec::new();
    for &k in shown.iter().skip(g.start).take(g.vis) {
        let it = &l.items[k];
        let sel = pick && k == l.sel;
        let t = format!(
            "|{}{}{}|",
            if sel { ">" } else { " " },
            if it.current { "*" } else { " " },
            fit(&sanitize(&it.text), tw, Align::Left)
        );
        let st = if sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        rows.push((t, st));
    }
    // 選べる形は「選んでいる番号/数」、絞った候補(選んでいない)は数だけ。
    let count = match shown.iter().position(|&k| k == l.sel).filter(|_| pick) {
        Some(p) => format!("{}/{}", p + 1, shown.len()),
        None => format!("{}", shown.len()),
    };
    let edge = format!("+{}+", fit(&count, g.iw - 2, Align::Left).replace(' ', "-"));
    let edge = (edge, Style::default());
    if g.below {
        rows.push(edge);
    } else {
        rows.insert(0, edge);
    }
    super::popup::blit(lines, g.x, g.top, g.iw, w, rows);
}

/// 行の桁 [x, x + iw) を `ins` で置き換える。幅2の文字が境目にかかったら空白にする。
pub(crate) fn splice(
    line: &Line<'static>,
    x: usize,
    ins: Span<'static>,
    iw: usize,
    w: usize,
) -> Line<'static> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut col = 0usize;
    let mut placed = false;
    for span in &line.spans {
        let st = line.style.patch(span.style);
        let mut keep = String::new();
        for g in span.content.graphemes(true) {
            let gw = width(g);
            let end = col + gw;
            if end <= x || col >= x + iw {
                keep.push_str(g);
            } else {
                if col < x {
                    keep.push_str(&" ".repeat(x - col));
                }
                if !placed {
                    out.push(Span::styled(std::mem::take(&mut keep), st));
                    out.push(ins.clone());
                    placed = true;
                }
                if end > x + iw {
                    keep.push_str(&" ".repeat(end - (x + iw)));
                }
            }
            col = end;
        }
        out.push(Span::styled(keep, st));
    }
    if !placed {
        out.push(Span::raw(" ".repeat(x.saturating_sub(col))));
        out.push(ins);
    }
    super::view::pad(out, w, Style::default())
}
