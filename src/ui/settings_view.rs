//! ビューの設定の画面の描画(純関数)とクリック(`impl App` の続き。NV-18)。
//! 上から: 見出し・空行・2つの欄(左は列の区画、右はフィルター・並べ替え・グループの区画か、開いた選び手)・
//! 空行・mdgrid のビューのボタン(名前を付けて保存・上書き・名前の変更・削除。BV-18)・
//! ボタン(反映・取り消し・既定に戻す)・下の帯・メッセージ行。選んだ項目は反転に加えて `>`(SR-15)。
//! 文字は width.rs で幅に合わせてから渡す(SR-9)。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::settings::{
    arrow, cond_label, Draft, Pick, Sec, TextKind, BUTTONS, CMPS, KINDS, VIEW_BUTTONS,
};
use super::view::{self, pad};
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::settings::Group;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// 欄の項目のクリックの行き先。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Item(Sec, usize),
    Pick(usize),
}

/// 欄の1行。
struct Row {
    text: String,
    hit: Option<Hit>,
    sel: bool,
    head: bool,
}

impl Row {
    fn head(text: impl Into<String>, focus: bool) -> Row {
        Row {
            text: text.into(),
            hit: None,
            sel: focus,
            head: true,
        }
    }

    fn item(text: impl Into<String>, hit: Hit, sel: bool) -> Row {
        Row {
            text: text.into(),
            hit: Some(hit),
            sel,
            head: false,
        }
    }

    fn blank() -> Row {
        Row {
            text: String::new(),
            hit: None,
            sel: false,
            head: false,
        }
    }
}

/// 欄の始まりの行。
const TOP: usize = 2;

/// 左の欄(列の区画)の幅。
fn left_w(w: usize) -> usize {
    (w * 3 / 10).clamp(12, 30).min(w)
}

/// 欄の高さ(見出し・空行と、下の空行・ボタンの2行・下の帯・メッセージ行を除く)。
fn pane_h(h: usize) -> usize {
    h.saturating_sub(TOP + 5)
}

/// ボタンの行(反映・取り消し・既定に戻す)。mdgrid のビューのボタン(BV-18)はその上の行。
fn buttons_y(h: usize) -> usize {
    h.saturating_sub(3)
}

/// ボタン i の行(`buttons_y` からいくつ上か)。
fn button_row(i: usize) -> usize {
    usize::from(i >= VIEW_BUTTONS)
}

/// `focus` の行が見えるように、高さ `h` の分だけ切り出す。
fn window(rows: Vec<Row>, focus: usize, h: usize) -> Vec<Row> {
    let start = if focus >= h { focus + 1 - h } else { 0 };
    rows.into_iter().skip(start).take(h).collect()
}

fn check(on: bool) -> &'static str {
    if on {
        "[x]"
    } else {
        "[ ]"
    }
}

fn radio(on: bool) -> &'static str {
    if on {
        "(*)"
    } else {
        "( )"
    }
}

/// 列の見せ方: 見出し(displayName)が id と違えば id も添える。
fn col_text(app: &App, id: &str) -> String {
    let t = app.title(id);
    if t == id {
        sanitize(id)
    } else {
        format!("{} ({})", sanitize(&t), sanitize(id))
    }
}

/// 左の欄の下の「表示」の節(SR-20)の行の数(空行・見出し・項目)。
const DISPLAY_ROWS: usize = 2 + mdgrid::display::ITEMS.len();

/// 左の欄: 列の区画(表示・非表示と順)と、その下の「表示」の節(SR-20)。
fn left_rows(app: &App, d: &Draft, ph: usize) -> Vec<Row> {
    let focus = d.sec == Sec::Columns && d.pick.is_none();
    let at = d.at(Sec::Columns);
    let mut out = vec![Row::head(Msg::SetColumns, focus)];
    let items: Vec<Row> = d
        .cols
        .iter()
        .enumerate()
        .map(|(i, (c, shown))| {
            Row::item(
                format!("{} {}", check(*shown), sanitize(&app.title(c))),
                Hit::Item(Sec::Columns, i),
                focus && i == at,
            )
        })
        .collect();
    // 列が多ければ列の区画を流し、「表示」の節はいつも欄の下に見せる(低い画面では列の区画を1行は残す)。
    let room = ph.saturating_sub(1 + DISPLAY_ROWS).max(1);
    out.extend(window(items, at, room));
    out.push(Row::blank());
    let focus = d.sec == Sec::Display && d.pick.is_none();
    let at = d.at(Sec::Display);
    let head = out.len();
    out.push(Row::head(Msg::SetDisplay, focus));
    for (i, (item, _)) in mdgrid::display::ITEMS.iter().enumerate() {
        let on = app.draft_shows(*item);
        out.push(Row::item(
            format!("{} {}", check(on), item.label()),
            Hit::Item(Sec::Display, i),
            focus && i == at,
        ));
    }
    // 低い画面で欄に入りきらなければ、選んだ項目(と、入るならその節の見出し)が見えるように流す
    // (選んだ印 `>` の見えない項目を切り替えない)。
    let sel = out.iter().position(|r| r.sel && !r.head).unwrap_or(0);
    let from = if focus && sel < head + ph { head } else { 0 };
    let start = if sel >= from + ph { sel + 1 - ph } else { from };
    let start = start.min(out.len().saturating_sub(ph));
    out.into_iter().skip(start).take(ph).collect()
}

/// 右の欄: フィルター・並べ替え・グループの区画か、開いた選び手。
fn right_rows(app: &App, d: &Draft, ph: usize) -> Vec<Row> {
    if let Some(p) = &d.pick {
        return pick_rows(app, d, p, ph);
    }
    let focus = |s: Sec| d.sec == s;
    let sel = |s: Sec, i: usize| focus(s) && d.at(s) == i;
    let mut rows = Vec::new();
    let mut focus_row = 0;
    let mut push = |rows: &mut Vec<Row>, r: Row| {
        if r.sel && !r.head {
            focus_row = rows.len();
        }
        rows.push(r);
    };
    // フィルター(NV-14・NV-19)
    push(&mut rows, Row::head(Msg::SetFilters, focus(Sec::Filters)));
    let nf = d.s.filters.len();
    for (i, c) in d.s.filters.iter().enumerate() {
        let r = Row::item(
            sanitize(&cond_label(c)),
            Hit::Item(Sec::Filters, i),
            sel(Sec::Filters, i),
        );
        push(&mut rows, r);
    }
    let r = Row::item(
        Msg::SetAddFilter,
        Hit::Item(Sec::Filters, nf),
        sel(Sec::Filters, nf),
    );
    push(&mut rows, r);
    push(&mut rows, Row::blank());
    // 並べ替え(NV-18。上の列が先)
    push(&mut rows, Row::head(Msg::SetSorts, focus(Sec::Sorts)));
    let ns = d.s.sorts.len();
    for (i, (c, dir)) in d.s.sorts.iter().enumerate() {
        let r = Row::item(
            format!("{} {}", sanitize(&app.title(c)), arrow(*dir)),
            Hit::Item(Sec::Sorts, i),
            sel(Sec::Sorts, i),
        );
        push(&mut rows, r);
    }
    let r = Row::item(
        Msg::SetAddSort,
        Hit::Item(Sec::Sorts, ns),
        sel(Sec::Sorts, ns),
    );
    push(&mut rows, r);
    push(&mut rows, Row::blank());
    // グループ(NV-15・NV-21)
    push(&mut rows, Row::head(Msg::SetGroup, focus(Sec::Group)));
    let inherit = if app.base.is_some() && app.nv.at.is_none() {
        Msg::SetGroupInheritBase
    } else {
        Msg::SetGroupInheritDefault
    };
    let (by, dir, hide) = match &d.s.group {
        Group::By {
            col,
            dir,
            hide_empty,
        } => (Some(col.as_str()), Some(*dir), *hide_empty),
        _ => (None, None, false),
    };
    let items = [
        format!("{} {inherit}", radio(d.s.group == Group::Inherit)),
        Msg::SetGroupOff.fill(&[&radio(d.s.group == Group::Off)]),
        Msg::SetGroupBy.fill(&[
            &radio(by.is_some()),
            &by.map(|c| sanitize(&app.title(c)))
                .unwrap_or_else(|| Msg::SetGroupPickColumn.into()),
        ]),
        Msg::SetGroupHideEmpty.fill(&[&check(hide)]),
        Msg::SetGroupOrder.fill(&[&arrow(dir.unwrap_or(mdgrid::settings::Dir::Asc))]),
    ];
    for (i, t) in items.into_iter().enumerate() {
        let r = Row::item(t, Hit::Item(Sec::Group, i), sel(Sec::Group, i));
        push(&mut rows, r);
    }
    window(rows, focus_row, ph)
}

/// 選び手の行(右の欄)。
fn pick_rows(app: &App, d: &Draft, p: &Pick, ph: usize) -> Vec<Row> {
    let (head, items): (String, Vec<String>) = match p {
        Pick::Column { purpose, .. } => (
            Msg::PickColumnFor.fill(&[&purpose.label()]),
            d.keys.iter().map(|c| col_text(app, c)).collect(),
        ),
        Pick::Kind { col, .. } => (
            Msg::PickKindOf.fill(&[&sanitize(&app.title(col))]),
            KINDS.iter().map(|k| k.to_string()).collect(),
        ),
        Pick::Cmp { col, .. } => (
            Msg::PickCmpOf.fill(&[&sanitize(&app.title(col))]),
            CMPS.iter().map(|(_, t)| t.to_string()).collect(),
        ),
        Pick::Values {
            col,
            keep,
            checked,
            counts,
            ..
        } => {
            let mode = if *keep {
                Msg::PickModeKeep
            } else {
                Msg::PickModeHide
            };
            let mut items = vec![mode.to_string()];
            items.extend(counts.iter().map(|(k, n)| {
                let label = k
                    .as_deref()
                    .map(sanitize)
                    .unwrap_or_else(|| Msg::EmptyHeading.into());
                Msg::PickValueRow.fill(&[&check(checked.contains(k)), &label, n])
            }));
            (Msg::PickValuesOf.fill(&[&sanitize(&app.title(col))]), items)
        }
    };
    let at = p.sel();
    let rows: Vec<Row> = items
        .into_iter()
        .enumerate()
        .map(|(i, t)| Row::item(t, Hit::Pick(i), i == at))
        .collect();
    let mut out = vec![Row::head(head, true)];
    out.extend(window(rows, at, ph.saturating_sub(1)));
    out
}

/// 欄の1行を幅 `w` の span にする。
fn row_span(r: Option<&Row>, w: usize) -> Span<'static> {
    let Some(r) = r else {
        return Span::raw(" ".repeat(w));
    };
    let lead = match (r.head, r.sel) {
        (true, true) => ">",
        (true, false) => " ",
        (false, true) => "  >",
        (false, false) => "   ",
    };
    let mut st = Style::default();
    if r.head {
        st = st.add_modifier(Modifier::BOLD);
    }
    if r.sel {
        st = st.add_modifier(Modifier::REVERSED);
    }
    Span::styled(fit(&format!("{lead}{}", r.text), w, Align::Left), st)
}

/// 行 `row`(0 は反映などの行、1 はその上の mdgrid のビューの行)のボタンの並び:
/// (始まりの桁, 見せる文字, 添字)。選んだボタンは `[>反映<]`、ほかは `[ 反映 ]`。
fn button_spans(d: &Draft, row: usize) -> Vec<(usize, String, usize)> {
    let focus = d.sec == Sec::Buttons && d.pick.is_none();
    let mut x = 2;
    let mut out = Vec::new();
    for (i, b) in BUTTONS.iter().enumerate() {
        // 読むだけでは mdgrid のビューのボタンを出さない(WB-15)。
        if button_row(i) != row || i >= d.len(Sec::Buttons) {
            continue;
        }
        let t = if focus && d.at(Sec::Buttons) == i {
            format!("[>{b}<]")
        } else {
            format!("[ {b} ]")
        };
        let tw = width(&t);
        out.push((x, t, i));
        x += tw + 2;
    }
    out
}

fn buttons_line(app: &App, d: &Draft, row: usize, w: usize) -> Line<'static> {
    let focus = d.sec == Sec::Buttons && d.pick.is_none();
    let mut spans = Vec::new();
    let mut used = 0;
    for (x, t, i) in button_spans(d, row) {
        spans.push(Span::raw(" ".repeat(x - used)));
        let mut st = Style::default().add_modifier(Modifier::BOLD);
        if focus && d.at(Sec::Buttons) == i {
            st = st.add_modifier(Modifier::REVERSED);
        }
        // 上書き・名前の変更・削除は mdgrid のビューを選んでいるときだけ(既定の表・.base のビューは薄く)。
        if i > VIEW_BUTTONS && app.nv.at.is_none() {
            st = st.add_modifier(Modifier::DIM);
        }
        used = x + width(&t);
        spans.push(Span::styled(t, st));
    }
    pad(spans, w, Style::default())
}

/// 値の入力の行の前置き(「status を含む」など)。
fn text_prompt(app: &App, col: &str, kind: TextKind) -> String {
    let c = sanitize(&app.title(col));
    match kind {
        TextKind::Contains => Msg::TextContains.fill(&[&c]),
        TextKind::NotContains => Msg::TextNotContains.fill(&[&c]),
        TextKind::Cmp(op) => {
            let sym = CMPS
                .iter()
                .find(|(o, _)| *o == op)
                .and_then(|(_, t)| t.text().split_whitespace().next())
                .unwrap_or("=");
            format!("{c} {sym}")
        }
        TextKind::SaveAs => Msg::TextSaveAs.into(),
        TextKind::Rename => Msg::TextRename.into(),
    }
}

fn text_line(app: &App, d: &Draft) -> Option<String> {
    let t = d.text.as_ref()?;
    Some(format!(
        "{}: {}",
        text_prompt(app, &t.col, t.kind),
        sanitize(&t.text)
    ))
}

/// 選んだ区画・選び手の案内(キーはキーの表から。SR-4)。
fn hint(app: &App, d: &Draft) -> String {
    let k = |a: Action| keymap::key_for(&app.keys, Mode::Settings, a).unwrap_or_default();
    let (en, sp, esc) = (k(Action::Run), k(Action::Toggle), k(Action::Cancel));
    if let Some(p) = &d.pick {
        return match p {
            Pick::Values { .. } => Msg::SetHintValues.fill(&[&sp, &en, &esc]),
            _ => Msg::SetHintPick.fill(&[&en, &esc]),
        };
    }
    let (up, down, rm) = (
        k(Action::MoveItemUp),
        k(Action::MoveItemDown),
        k(Action::RemoveItem),
    );
    match d.sec {
        Sec::Columns => Msg::SetHintColumns.fill(&[&sp, &up, &down]),
        Sec::Filters => Msg::SetHintFilters.fill(&[&en, &sp, &rm]),
        Sec::Sorts => Msg::SetHintSorts.fill(&[&en, &up, &down, &rm]),
        Sec::Group => Msg::SetHintGroup.fill(&[&en]),
        Sec::Display => Msg::SetHintDisplay.fill(&[&sp]),
        Sec::Buttons => Msg::SetHintButtons.fill(&[&en]),
    }
}

fn message_line(app: &App, d: &Draft, w: usize) -> Line<'static> {
    let text = match text_line(app, d) {
        Some(mut t) => {
            if let Some(m) = &app.message {
                t.push_str(&format!("  ({})", sanitize(m)));
            }
            t
        }
        None => sanitize(&app.message.clone().unwrap_or_else(|| hint(app, d))),
    };
    Line::from(fit(&text, w, Align::Left))
}

/// ビューの設定の画面(NV-18)。各行の幅はちょうど `w`。
pub(crate) fn render_settings(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(d) = &app.draft else {
        return Vec::new();
    };
    let name = match &app.base {
        _ if app.nv.at.is_some() => app
            .native_view()
            .map(|v| Msg::SetNativeName.fill(&[&v.name]))
            .unwrap_or_default(),
        Some(b) => {
            let v = b.base.views.get(b.view).map(|v| v.name.clone());
            format!("{} / {}", b.name, v.unwrap_or_default())
        }
        None => app.src.name(),
    };
    let head = Msg::SetHead.fill(&[&sanitize(&name)]);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let mut lines = vec![
        Line::from(Span::styled(fit(&head, w, Align::Left), bold)),
        Line::from(" ".repeat(w)),
    ];
    let ph = pane_h(h);
    let lw = left_w(w);
    let rw = w.saturating_sub(lw + 2);
    let left = left_rows(app, d, ph);
    let right = right_rows(app, d, ph);
    for k in 0..ph {
        let spans = vec![
            row_span(left.get(k), lw),
            Span::raw("  "),
            row_span(right.get(k), rw),
        ];
        lines.push(pad(spans, w, Style::default()));
    }
    lines.push(Line::from(" ".repeat(w)));
    lines.push(buttons_line(app, d, 1, w));
    lines.push(buttons_line(app, d, 0, w));
    while lines.len() < h.saturating_sub(2) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(h.saturating_sub(2));
    lines.push(view::footer(app, w));
    lines.push(message_line(app, d, w));
    lines.truncate(h);
    lines
}

/// 値の入力の位置(SR-17: 変換の窓がそこに出る)。メッセージ行の入力の末尾。
pub(crate) fn text_cursor(app: &App) -> Option<(u16, u16)> {
    let d = app.draft.as_ref()?;
    let t = text_line(app, d)?;
    let w = app.size.0.saturating_sub(1) as usize;
    let y = app.size.1.saturating_sub(2);
    let x = width(&t);
    (x < w).then_some((x as u16, y))
}

impl App {
    /// 設定の画面のクリック(NV-18): 項目は選んで決める(値の一覧はチェックの付け外し)、ボタンは押す。
    pub(crate) fn settings_click(&mut self, x: u16, y: u16) {
        let w = self.size.0.saturating_sub(1) as usize;
        let h = self.size.1.saturating_sub(1) as usize;
        let (x, y) = (x as usize, y as usize);
        let Some(d) = &self.draft else {
            return;
        };
        if y == buttons_y(h) || y + 1 == buttons_y(h) {
            let Some(i) = button_spans(d, buttons_y(h) - y)
                .into_iter()
                .find(|(x0, t, _)| x >= *x0 && x < x0 + width(t))
                .map(|(_, _, i)| i)
            else {
                return;
            };
            if let Some(d) = self.draft.as_mut() {
                d.pick = None;
                d.sec = Sec::Buttons;
                d.sel[Sec::Buttons as usize] = i;
            }
            return self.apply(Action::Run);
        }
        let ph = pane_h(h);
        if y < TOP || y >= TOP + ph {
            return;
        }
        let lw = left_w(w);
        let rows = if x < lw {
            left_rows(self, d, ph)
        } else if x >= lw + 2 {
            right_rows(self, d, ph)
        } else {
            return;
        };
        let Some(hit) = rows.get(y - TOP).and_then(|r| r.hit) else {
            return;
        };
        let picking = d.pick.is_some();
        let values = matches!(d.pick, Some(Pick::Values { .. }));
        let action = match hit {
            Hit::Item(_, _) if picking => return,
            Hit::Item(sec, i) => {
                if let Some(d) = self.draft.as_mut() {
                    d.sec = sec;
                    d.sel[sec as usize] = i;
                }
                Action::Run
            }
            Hit::Pick(i) => {
                if let Some(p) = self.draft.as_mut().and_then(|d| d.pick.as_mut()) {
                    match p {
                        Pick::Column { sel, .. }
                        | Pick::Kind { sel, .. }
                        | Pick::Cmp { sel, .. }
                        | Pick::Values { sel, .. } => *sel = i,
                    }
                }
                if values {
                    Action::Toggle
                } else {
                    Action::Run
                }
            }
        };
        self.apply(action);
    }
}
