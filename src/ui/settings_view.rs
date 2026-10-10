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
    /// 区画の説明(薄く)。
    desc: bool,
}

impl Row {
    fn head(text: impl Into<String>, focus: bool) -> Row {
        Row {
            text: text.into(),
            hit: None,
            sel: focus,
            head: true,
            desc: false,
        }
    }

    fn item(text: impl Into<String>, hit: Hit, sel: bool) -> Row {
        Row {
            text: text.into(),
            hit: Some(hit),
            sel,
            head: false,
            desc: false,
        }
    }

    fn desc(text: impl Into<String>) -> Row {
        Row {
            text: text.into(),
            hit: None,
            sel: false,
            head: false,
            desc: true,
        }
    }

    fn blank() -> Row {
        Row {
            text: String::new(),
            hit: None,
            sel: false,
            head: false,
            desc: false,
        }
    }
}

/// 区画の一覧と中身の始まりの行(上に見出しと反映の行、空行)。
const TOP: usize = 2;

/// 左の区画の一覧の幅。
fn nav_w(w: usize) -> usize {
    (w / 4).clamp(14, 24).min(w)
}

/// 欄の高さ(上の2行と、下の空行・ボタンの行・下の帯・メッセージ行を除く)。
fn pane_h(h: usize) -> usize {
    h.saturating_sub(TOP + 4)
}

/// 下のボタンの行(既定に戻す・mdgrid のビューのボタン。反映と取り消しは上の右)。
fn buttons_y(h: usize) -> usize {
    h.saturating_sub(3)
}

/// `focus` の行が見えるように、高さ `h` の分だけ切り出す。
fn window(rows: Vec<Row>, focus: usize, h: usize) -> Vec<Row> {
    let start = if focus >= h { focus + 1 - h } else { 0 };
    rows.into_iter().skip(start).take(h).collect()
}

/// オン・オフのスイッチ(NV-18。色に頼らず、塗った ● と空の ○ と言葉で分かる)。
fn switch(on: bool, yes: Msg, no: Msg) -> String {
    if on {
        format!("● {yes}")
    } else {
        format!("○ {no}")
    }
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
        "◉"
    } else {
        "○"
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

/// 左の一覧に並べる区画(Tab の順。ボタンは上と下に置くので一覧には出さない)。
const NAV: [Sec; 8] = [
    Sec::Columns,
    Sec::Filters,
    Sec::Sorts,
    Sec::Group,
    Sec::Tree,
    Sec::Display,
    Sec::Views,
    Sec::Look,
];

/// 右に中身を見せる区画(ボタンを選んでいる間は、その前の「表示」)。
fn shown_sec(d: &Draft) -> Sec {
    if d.sec == Sec::Buttons {
        Sec::Display
    } else {
        d.sec
    }
}

fn sec_name(sec: Sec) -> Msg {
    match sec {
        Sec::Columns => Msg::NavColumns,
        Sec::Filters => Msg::NavFilters,
        Sec::Sorts => Msg::NavSorts,
        Sec::Group => Msg::NavGroup,
        Sec::Tree => Msg::NavTree,
        Sec::Display | Sec::Buttons => Msg::NavDisplay,
        Sec::Views => Msg::NavViews,
        Sec::Look => Msg::NavLook,
    }
}

/// 区画の今の状態(一覧に添える。例「5/6」「2」「status」。何も効いていなければ「—」)。
fn badge(app: &App, d: &Draft, sec: Sec) -> String {
    let none = || "—".to_string();
    match sec {
        Sec::Columns => format!("{}/{}", d.cols.iter().filter(|c| c.1).count(), d.cols.len()),
        Sec::Filters if d.s.filters.is_empty() => none(),
        Sec::Filters => d.s.filters.len().to_string(),
        Sec::Sorts if d.s.sorts.is_empty() => none(),
        Sec::Sorts => d.s.sorts.len().to_string(),
        Sec::Group => match &d.s.group {
            Group::By { col, .. } => sanitize(&app.title(col)),
            Group::Off => Msg::NavGroupOff.to_string(),
            Group::Inherit => none(),
        },
        Sec::Tree => match &d.s.tree {
            Some(k) => sanitize(&app.title(k)),
            None => none(),
        },
        Sec::Display | Sec::Buttons => {
            let on = mdgrid::display::ITEMS
                .iter()
                .filter(|(item, _)| app.draft_shows(*item))
                .count();
            format!("{on}/{}", mdgrid::display::ITEMS.len())
        }
        // SR-43: 写しの組の名前。
        Sec::Look => d.look.preset.name().to_string(),
        // NV-26: タブの行に出している数/全部。
        Sec::Views => {
            let tabs = app.section_tabs();
            let shown = tabs
                .iter()
                .filter(|(n, _)| !app.nv.tabs.hidden.contains(n))
                .count();
            format!("{shown}/{}", tabs.len())
        }
    }
}

/// 反映していない変更の数(区画ごとに数える。上の「反映」に添える)。
fn pending(app: &App, d: &Draft) -> usize {
    let s = &app.settings;
    [
        d.cols_changed(),
        d.s.filters != s.filters,
        d.s.sorts != s.sorts,
        d.s.group != s.group,
        d.s.tree != s.tree,
        d.s.wbs != s.wbs,
        d.s.display != s.display,
        d.look != d.look0,
    ]
    .iter()
    .filter(|b| **b)
    .count()
}

/// 左の区画の一覧(1行おき)。
fn nav_rows(app: &App, d: &Draft, nw: usize) -> Vec<(String, Option<Sec>, bool)> {
    let cur = shown_sec(d);
    let mut out = Vec::new();
    for sec in NAV {
        let on = sec == cur;
        let name = sec_name(sec).to_string();
        let b = badge(app, d, sec);
        let room = nw.saturating_sub(3 + width(&b) + 1);
        out.push((
            format!(
                "{}{} {}",
                if on { "> " } else { "  " },
                fit(&name, room, Align::Left),
                b
            ),
            Some(sec),
            on,
        ));
        out.push((String::new(), None, false));
    }
    out
}

/// 区画の説明(右の中身の2行目)。
fn sec_desc(sec: Sec) -> Msg {
    match sec {
        Sec::Columns => Msg::DescColumns,
        Sec::Filters => Msg::DescFilters,
        Sec::Sorts => Msg::DescSorts,
        Sec::Group => Msg::DescGroup,
        Sec::Tree => Msg::DescTree,
        Sec::Display | Sec::Buttons => Msg::DescDisplay,
        Sec::Views => Msg::DescViews,
        Sec::Look => Msg::DescLook,
    }
}

/// 右の欄: 選んだ区画の見出し・説明・項目か、開いた選び手。
fn right_rows(app: &App, d: &Draft, ph: usize) -> Vec<Row> {
    if let Some(p) = &d.pick {
        return pick_rows(app, d, p, ph);
    }
    let sec = shown_sec(d);
    let focus = d.sec == sec;
    let sel = |i: usize| focus && d.at(sec) == i;
    let mut head = vec![
        Row::head(sec_name(sec), focus),
        Row::desc(sec_desc(sec)),
        Row::blank(),
    ];
    let items: Vec<Row> = match sec {
        Sec::Columns => d
            .cols
            .iter()
            .enumerate()
            .map(|(i, (c, shown))| {
                let name = fit(&sanitize(&app.title(c)), 20, Align::Left);
                Row::item(
                    format!("⠿ {name} {}", switch(*shown, Msg::SwShown, Msg::SwHidden)),
                    Hit::Item(Sec::Columns, i),
                    sel(i),
                )
            })
            .collect(),
        Sec::Filters => {
            let n = d.s.filters.len();
            let mut v: Vec<Row> =
                d.s.filters
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        Row::item(sanitize(&cond_label(c)), Hit::Item(Sec::Filters, i), sel(i))
                    })
                    .collect();
            v.push(Row::item(
                Msg::SetAddFilter,
                Hit::Item(Sec::Filters, n),
                sel(n),
            ));
            v
        }
        Sec::Sorts => {
            let n = d.s.sorts.len();
            let mut v: Vec<Row> =
                d.s.sorts
                    .iter()
                    .enumerate()
                    .map(|(i, (c, dir))| {
                        Row::item(
                            format!("{}. {} {}", i + 1, sanitize(&app.title(c)), arrow(*dir)),
                            Hit::Item(Sec::Sorts, i),
                            sel(i),
                        )
                    })
                    .collect();
            v.push(Row::item(Msg::SetAddSort, Hit::Item(Sec::Sorts, n), sel(n)));
            v
        }
        Sec::Group => {
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
                format!(
                    "{}  {}",
                    fit(Msg::NavHideEmpty.text(), 20, Align::Left),
                    switch(hide, Msg::SwOn, Msg::SwOff)
                ),
                Msg::SetGroupOrder.fill(&[&arrow(dir.unwrap_or(mdgrid::settings::Dir::Asc))]),
            ];
            items
                .into_iter()
                .enumerate()
                .map(|(i, t)| Row::item(t, Hit::Item(Sec::Group, i), sel(i)))
                .collect()
        }
        Sec::Display | Sec::Buttons => mdgrid::display::ITEMS
            .iter()
            .enumerate()
            .map(|(i, (item, _))| {
                let on = app.draft_shows(*item);
                // SR-34: タブの行は always・auto・never の3つ。
                let value = match (*item, app.draft_tabs()) {
                    (mdgrid::display::Item::Tabs, mdgrid::display::TabsMode::Auto) => {
                        format!("◐ {}", Msg::SwAuto)
                    }
                    _ => switch(on, Msg::SwOn, Msg::SwOff),
                };
                Row::item(
                    format!("{}  {}", fit(item.label(), 20, Align::Left), value),
                    Hit::Item(Sec::Display, i),
                    sel(i),
                )
            })
            .collect(),
        Sec::Views => {
            let default = app.default_view_name();
            let mut v: Vec<Row> = app
                .section_tabs()
                .into_iter()
                .enumerate()
                .map(|(i, (name, _))| {
                    let shown = !app.nv.tabs.hidden.contains(&name);
                    let mark = if default.as_deref() == Some(name.as_str()) {
                        format!("  ★ {}", Msg::TabDefaultMark)
                    } else {
                        String::new()
                    };
                    Row::item(
                        format!(
                            "⠿ {} {}{mark}",
                            fit(&sanitize(&name), 20, Align::Left),
                            switch(shown, Msg::SwShown, Msg::SwHidden)
                        ),
                        Hit::Item(Sec::Views, i),
                        sel(i),
                    )
                })
                .collect();
            let n = v.len();
            v.push(Row::blank());
            v.push(Row::item(
                format!(
                    "{}  {}",
                    fit(Msg::TabHintRow.text(), 21, Align::Left),
                    switch(app.nv.tabs.hint, Msg::SwOn, Msg::SwOff)
                ),
                Hit::Item(Sec::Views, n),
                sel(n),
            ));
            v
        }
        Sec::Tree => {
            let mut v = vec![
                Row::item(
                    format!(
                        "{}  {}",
                        fit(Msg::TreeOnRow.text(), 20, Align::Left),
                        switch(d.s.tree.is_some(), Msg::SwOn, Msg::SwOff)
                    ),
                    Hit::Item(Sec::Tree, 0),
                    sel(0),
                ),
                Row::item(
                    format!(
                        "{}  ▾ {}",
                        fit(Msg::TreeKeyRow.text(), 20, Align::Left),
                        sanitize(&d.tree_key)
                    ),
                    Hit::Item(Sec::Tree, 1),
                    sel(1),
                ),
            ];
            // NV-28: WBS のスイッチと進み具合のキー、オンなら値ごとの割合とラベル。
            v.push(Row::item(
                format!(
                    "{}  {}",
                    fit(Msg::TreeWbsRow.text(), 20, Align::Left),
                    switch(d.s.wbs.is_some(), Msg::SwOn, Msg::SwOff)
                ),
                Hit::Item(Sec::Tree, 2),
                sel(2),
            ));
            v.push(Row::item(
                format!(
                    "{}  ▾ {}",
                    fit(Msg::TreeWbsKeyRow.text(), 20, Align::Left),
                    sanitize(&d.wbs.key)
                ),
                Hit::Item(Sec::Tree, 3),
                sel(3),
            ));
            if d.s.wbs.is_some() {
                for (k, (val, n)) in d.wbs_vals.iter().enumerate() {
                    let to = match d.wbs.of(val) {
                        Some(m) => format!("{}% {}", m.percent, sanitize(&m.label)),
                        None => "—".to_string(),
                    };
                    v.push(Row::item(
                        format!(
                            "  {}  → {to}",
                            fit(
                                &Msg::WbsValueRow.fill(&[&sanitize(val), n]),
                                18,
                                Align::Left
                            )
                        ),
                        Hit::Item(Sec::Tree, 4 + k),
                        sel(4 + k),
                    ));
                }
            }
            v
        }
        Sec::Look => {
            let lk = d.look;
            let from = |path: &str| {
                format!(
                    "  ({})",
                    super::look_section::place_label(app.look_origin(path))
                )
            };
            let fields = [
                (
                    Msg::LookScope,
                    super::look_section::place_label(lk.scope).to_string(),
                ),
                (
                    Msg::LookTheme,
                    format!("{}{}", lk.theme.label(), from("look.theme")),
                ),
                (
                    Msg::LookPreset,
                    format!("{}{}", lk.preset.name(), from("look.preset")),
                ),
                (Msg::LookNerd, super::look_section::nerd_text(lk.nerd)),
            ];
            let mut v: Vec<Row> = fields
                .into_iter()
                .enumerate()
                .map(|(i, (label, value))| {
                    Row::item(
                        format!("{}  ▾ {value}", fit(label.text(), 20, Align::Left)),
                        Hit::Item(Sec::Look, i),
                        sel(i),
                    )
                })
                .collect();
            let templates = app.look_templates();
            let base = super::look_section::LOOK_FIELDS;
            v.push(Row::blank());
            v.push(Row::desc(Msg::LookTemplates));
            for (k, (name, l, _)) in templates.iter().enumerate() {
                v.push(Row::item(
                    format!(
                        "◆ {} {}",
                        fit(&sanitize(name), 18, Align::Left),
                        super::look_section::LookPick::summary(l)
                    ),
                    Hit::Item(Sec::Look, base + k),
                    sel(base + k),
                ));
            }
            let n = base + templates.len();
            v.push(Row::item(
                Msg::LookSaveTemplate,
                Hit::Item(Sec::Look, n),
                sel(n),
            ));
            v.push(Row::item(
                Msg::LookResetRow,
                Hit::Item(Sec::Look, n + 1),
                sel(n + 1),
            ));
            v
        }
    };
    let at = items.iter().position(|r| r.sel).unwrap_or(0);
    let room = ph.saturating_sub(head.len());
    head.extend(window(items, at, room));
    head
}

/// 上の右のボタン(反映・取り消し): (始まりの桁, 文字, 添字)。
fn top_buttons(app: &App, d: &Draft, w: usize) -> Vec<(usize, String, usize)> {
    let focus = d.sec == Sec::Buttons && d.pick.is_none();
    let label = |i: usize| {
        let b = BUTTONS[i];
        if focus && d.at(Sec::Buttons) == i {
            format!("[>{b}<]")
        } else {
            format!("[ {b} ]")
        }
    };
    let (a, c) = (label(0), label(1));
    let n = pending(app, d);
    let note = if n > 0 {
        Msg::SetPending.fill(&[&n])
    } else {
        String::new()
    };
    let total = width(&note) + if n > 0 { 2 } else { 0 } + width(&a) + 1 + width(&c);
    let x0 = w.saturating_sub(total + 1);
    let xa = x0 + width(&note) + if n > 0 { 2 } else { 0 };
    vec![(xa, a.clone(), 0), (xa + width(&a) + 1, c, 1)]
}

/// 下のボタン(既定に戻す・mdgrid のビューのボタン): (始まりの桁, 文字, 添字)。
fn bottom_buttons(d: &Draft, x0: usize) -> Vec<(usize, String, usize)> {
    let focus = d.sec == Sec::Buttons && d.pick.is_none();
    let mut x = x0;
    let mut out = Vec::new();
    // 既定に戻すと、mdgrid のビューのボタン(読むだけでは出さない。WB-15)。添字の順(←→ と同じ並び)。
    for (i, b) in BUTTONS.iter().enumerate().take(d.len(Sec::Buttons)).skip(2) {
        let t = if focus && d.at(Sec::Buttons) == i {
            format!("[>{b}<]")
        } else {
            format!("[ {b} ]")
        };
        let tw = width(&t);
        out.push((x, t, i));
        x += tw + 1;
    }
    out
}

/// 選び手の行(右の欄)。
fn pick_rows(app: &App, d: &Draft, p: &Pick, ph: usize) -> Vec<Row> {
    let (head, items): (String, Vec<String>) = match p {
        Pick::Column { purpose, .. } => (
            Msg::PickColumnFor.fill(&[&purpose.label()]),
            d.keys.iter().map(|c| col_text(app, c)).collect(),
        ),
        Pick::Look { field, .. } => {
            let what = [
                Msg::LookScope,
                Msg::LookTheme,
                Msg::LookPreset,
                Msg::LookNerd,
            ][(*field).min(3)];
            (
                Msg::LookPickOf.fill(&[&what.text()]),
                app.look_pick_items(*field),
            )
        }
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
        Pick::TabMenu { tab, .. } => {
            let name = app
                .section_tabs()
                .get(*tab)
                .map(|t| sanitize(&t.0))
                .unwrap_or_default();
            (Msg::TabMenuOf.fill(&[&name]), app.tab_menu_items(*tab))
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

/// 欄の1行を幅 `w` の span にする(見出しは太字、説明は薄く、選んだ項目は `>` と反転)。
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
    if r.desc {
        st = st.add_modifier(Modifier::DIM);
    }
    if r.sel {
        st = st.add_modifier(Modifier::REVERSED);
    }
    let lead = if r.desc { " " } else { lead };
    Span::styled(fit(&format!("{lead}{}", r.text), w, Align::Left), st)
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
        TextKind::Rename | TextKind::RenameTab => Msg::TextRename.into(),
        TextKind::LookTemplate => Msg::TextLookTemplate.into(),
        TextKind::WbsValue => Msg::TextWbsValue.fill(&[&sanitize(col)]),
        TextKind::DeleteView => Msg::TextDeleteView.fill(&[&sanitize(col)]),
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
        Sec::Tree => Msg::SetHintTree.fill(&[&en]),
        Sec::Display => Msg::SetHintDisplay.fill(&[&sp]),
        Sec::Views => Msg::SetHintViews.fill(&[&sp, &up, &down, &en]),
        Sec::Look => Msg::SetHintLook.fill(&[&en, &rm]),
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

/// ビューの設定の画面(NV-18)。上に見出しと「反映」「取り消し」、左に区画の一覧、右に選んだ区画の中身、
/// 下に既定に戻すと mdgrid のビューのボタン。各行の幅はちょうど `w`。
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
    // 1行目: 見出しと、右に「未反映 N」「反映」「取り消し」。
    let tops = top_buttons(app, d, w);
    let x0 = tops.first().map_or(w, |(x, _, _)| *x);
    let n = pending(app, d);
    let note = if n > 0 {
        Msg::SetPending.fill(&[&n])
    } else {
        String::new()
    };
    let note_x = x0.saturating_sub(width(&note) + if n > 0 { 2 } else { 0 });
    // 案内まで入らなければ、案内を省いて見出しと名前だけにする(途中で切らない)。
    let head = if width(&head) + 1 > note_x {
        Msg::SetHeadShort.fill(&[&sanitize(&name)])
    } else {
        head
    };
    let mut spans = vec![Span::styled(fit(&head, note_x, Align::Left), bold)];
    if n > 0 {
        spans.push(Span::styled(note, bold));
        spans.push(Span::raw("  "));
    }
    let focus = d.sec == Sec::Buttons && d.pick.is_none();
    let mut used = x0;
    for (x, t, i) in &tops {
        if *x > used {
            spans.push(Span::raw(" ".repeat(x - used)));
        }
        let mut st = Style::default().add_modifier(Modifier::BOLD);
        if focus && d.at(Sec::Buttons) == *i {
            st = st.add_modifier(Modifier::REVERSED);
        }
        // 変更があれば「反映」を目立たせる(下線。色のテーマではアクセントの色の太字)。
        if *i == 0 && n > 0 {
            st = st.add_modifier(Modifier::UNDERLINED);
        }
        used = x + width(t);
        spans.push(Span::styled(t.clone(), st));
    }
    let mut lines = vec![pad(spans, w, Style::default()), Line::from(" ".repeat(w))];
    let ph = pane_h(h);
    let nw = nav_w(w);
    let rw = w.saturating_sub(nw + 3);
    let nav = nav_rows(app, d, nw);
    let right = right_rows(app, d, ph);
    for k in 0..ph {
        let left = match nav.get(k) {
            Some((t, _, on)) => {
                let mut st = Style::default();
                if *on {
                    st = st.add_modifier(Modifier::BOLD | Modifier::REVERSED);
                }
                Span::styled(fit(t, nw, Align::Left), st)
            }
            None => Span::raw(" ".repeat(nw)),
        };
        let spans = vec![left, Span::raw(" │ "), row_span(right.get(k), rw)];
        lines.push(pad(spans, w, Style::default()));
    }
    lines.push(Line::from(" ".repeat(w)));
    // 下のボタン(左の端から。狭い端末でも入るように)。
    let mut spans = vec![Span::raw("  ")];
    let mut used = 2;
    for (x, t, i) in bottom_buttons(d, 2) {
        if x > used {
            spans.push(Span::raw(" ".repeat(x - used)));
        }
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
    lines.push(pad(spans, w, Style::default()));
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
    /// 設定の画面のクリック(NV-18): 左の一覧は区画を選ぶ、右の項目は選んで決める(値の一覧はチェックの
    /// 付け外し)、上と下のボタンは押す。
    pub(crate) fn settings_click(&mut self, x: u16, y: u16) {
        let w = self.size.0.saturating_sub(1) as usize;
        let h = self.size.1.saturating_sub(1) as usize;
        let (x, y) = (x as usize, y as usize);
        let Some(d) = &self.draft else {
            return;
        };
        let nw = nav_w(w);
        let buttons = if y == 0 {
            top_buttons(self, d, w)
        } else if y == buttons_y(h) {
            bottom_buttons(d, 2)
        } else {
            Vec::new()
        };
        if y == 0 || y == buttons_y(h) {
            let Some(i) = buttons
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
        if x < nw {
            // 左の一覧: その区画へ(選び手は閉じる)。
            if let Some((_, Some(sec), _)) = nav_rows(self, d, nw).get(y - TOP) {
                let sec = *sec;
                if let Some(d) = self.draft.as_mut() {
                    d.pick = None;
                    d.sec = sec;
                }
            }
            return;
        }
        if x < nw + 3 {
            return;
        }
        let rows = right_rows(self, d, ph);
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
                // ビューの区画: タブの行は選び手(Enter)、末尾の案内の行は切り替え。
                let views_toggle = sec == Sec::Views
                    && i + 1 == self.draft.as_ref().map_or(0, |d| d.len(Sec::Views));
                if matches!(sec, Sec::Columns | Sec::Display) || views_toggle {
                    Action::Toggle
                } else {
                    Action::Run
                }
            }
            Hit::Pick(i) => {
                if let Some(p) = self.draft.as_mut().and_then(|d| d.pick.as_mut()) {
                    *p.sel_mut() = i;
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
