//! 画面の帯(`view` の続きの描画の純関数): ヘッダー・ビューのタブ(SR-1・BV-13)、検索の欄(NV-23・NV-2)、
//! 設定の帯(NV-16・NV-22)、下の帯(SR-1・NV-5 の「選択 N」)、
//! メッセージ行(SR-15・BV-7・NV-12 の印の説明)と、検索・簡易の絞り込みの入力の行(NV-1・NV-2)。

use super::app::App;
use super::cell::shown;
use super::grid::Slot;
use super::keymap::{self, Mode};
use super::view::{pad, HELD_MARK};
use super::width::{fit, sanitize, width, Align};
use mdgrid::display::Item;
use mdgrid::i18n::Msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// ヘッダーの左の文字(名前・行の数・読み込み・絞り込み・隠した列・ワークスペースの名前)。
fn header_text(app: &App) -> String {
    // SR-1: `.base` を開いたらその名前、無ければフォルダの名前。
    let name = match &app.base {
        Some(b) => b.name.clone(),
        None => app.src.name(),
    };
    let mut text = format!(
        " {}  {}",
        sanitize(&name),
        Msg::HeaderRows.fill(&[&app.rows.len()])
    );
    // SR-20: タブを隠していれば、今のビューの名前(定義と違えば `*`。BV-20)をヘッダーに出す。
    if tab_rows(app) == 0 && !single_tab_hidden(app) {
        if let Some(v) = app.view_names().get(app.view_index()) {
            let dirty = if app.native_dirty() { "*" } else { "" };
            text.push_str(&Msg::HeaderView.fill(&[&sanitize(v), &dirty]));
        }
    }
    if !app.progress.done {
        let total = app
            .progress
            .total
            .map(|t| t.to_string())
            .unwrap_or_else(|| "?".into());
        text.push_str(&Msg::HeaderLoading.fill(&[&app.progress.loaded, &total]));
        if let Some(k) = keymap::key_for(&app.keys, app.mode, keymap::Action::CancelLoad) {
            text.push_str(&Msg::HeaderCancelKey.fill(&[&k]));
        }
    } else if app.cancelled {
        text.push_str(Msg::HeaderCancelled.text());
    }
    // 画面の中だけで効いている絞り込みと隠した列(NV-2・NV-8・NV-4)。簡易の絞り込みの語は、
    // 検索の欄(NV-23)があればそこに出すので、欄が描かれないとき(設定か低い端末)だけヘッダーに出す。
    if let Some(f) = app.filter.as_ref().filter(|_| !bar_drawn(app)) {
        text.push_str(&Msg::HeaderFilter.fill(&[&sanitize(f)]));
    }
    // NV-8・NV-9: 空の値は「(空)」、リストの要素で絞った条件は「含む」の形。
    for s in &app.same {
        let msg = if s.list && !s.value.is_empty() {
            Msg::HeaderSameHas
        } else {
            Msg::HeaderSame
        };
        text.push_str(&msg.fill(&[
            &sanitize(&app.title(&s.col)),
            &sanitize(&super::nav::shown_value(&s.value)),
        ]));
    }
    if !app.hidden.is_empty() {
        text.push_str(&Msg::HeaderHidden.fill(&[&app.hidden.len()]));
    }
    // WS-6: 範囲のワークスペースの名前(書いた範囲のとき。検知の範囲は関係マップの画面だけ)。
    if let Some(s) = &app.scope {
        let detected = matches!(s.source, mdgrid::workspace::Source::Detected(_));
        if app.mode == Mode::Relations || !detected {
            text.push_str(&Msg::HeaderWorkspace.fill(&[&sanitize(&s.name)]));
        }
    }
    text
}

pub(crate) fn header(app: &App, w: usize) -> Line<'static> {
    let text = header_text(app);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    // SR-42: タブの左に、ワークスペースと設定のボタン(幅が足りなければ出さない)。
    let buttons = header_buttons(app, w);
    let buttons_w: usize = buttons.iter().map(|(_, t, _)| width(t) + 1).sum();
    // REL-8: 登録した表があれば、右に画面の型のタブ(表・関係)。
    let look = super::look::look(app);
    let tabs = screen_tabs(app);
    let tabs_w: usize = tabs.iter().map(|(t, _)| width(t)).sum();
    let tab_spans: Vec<Span<'static>> = tabs
        .into_iter()
        .map(|(t, on)| {
            let st = match (&look, on) {
                (Some(l), on) => l.tab(app.style.tabs, on),
                (None, true) => bold.add_modifier(Modifier::REVERSED),
                (None, false) => Style::default(),
            };
            Span::styled(t, st)
        })
        .collect();
    // CE-25: 右の端に「+ 新規」(クリックで新しいノート。読むだけでは出さない)。
    let button = super::new_note::button();
    let bw = width(button);
    // SR-33: モダンな見た目では「+ 新規」を札にする(文字は同じ)。名前の色はテーマのヘッダーの色
    // (テーマが無ければ端末の色)のまま(SR-26)。
    let (name_st, button_st) = match &look {
        Some(l) => (bold, l.pill()),
        None => (bold, bold.add_modifier(Modifier::REVERSED)),
    };
    // CE-25: 「+ 新規」は関係マップでも同じ位置に出す(関係マップでは選んでいる表に作る)。
    let with_button = super::new_note::button_shown(app);
    let right = tabs_w + if with_button { bw } else { 0 } + buttons_w;
    if right > 0 && w >= right + 2 {
        let mut spans = vec![Span::styled(fit(&text, w - right, Align::Left), name_st)];
        let shortcut_st = |l: &Option<super::look::Look>| match l {
            Some(l) => Style::default().bg(l.soft_bg).fg(l.accent),
            None => Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        };
        for (_, t, _) in &buttons {
            spans.push(Span::styled(t.clone(), shortcut_st(&look)));
            spans.push(Span::raw(" "));
        }
        spans.extend(tab_spans);
        if with_button {
            spans.push(Span::styled(button, button_st));
        }
        return Line::from(spans);
    }
    Line::from(Span::styled(fit(&text, w, Align::Left), name_st))
}

/// 画面の型のタブ(REL-8): 登録した表があるとき(と関係マップの画面)だけ。(文字, 選んでいるか)。
/// 選んでいるものは `[表]`、ほかは ` 関係 `(ビューのタブと同じ形。色なしでも分かる)。
fn screen_tabs(app: &App) -> Vec<(String, bool)> {
    if app.scope_places().is_empty() && app.mode != Mode::Relations {
        return Vec::new();
    }
    let rel = app.mode == Mode::Relations;
    let tab = |name: &str, on: bool| {
        if on {
            format!("[{name}]")
        } else {
            format!(" {name} ")
        }
    };
    vec![
        (tab(Msg::TabScreenTable.text(), !rel), !rel),
        (tab(Msg::TabScreenRelations.text(), rel), rel),
        (" ".to_string(), false),
    ]
}

/// (x, y) が画面の型のタブなら、関係マップか(REL-8 のクリック)。
pub(crate) fn screen_tab_at(app: &App, x: u16, y: u16) -> Option<bool> {
    if y != 0 {
        return None;
    }
    let tabs = screen_tabs(app);
    if tabs.is_empty() {
        return None;
    }
    let (w, _) = super::popup::screen(app);
    let button = super::new_note::button();
    let with_button = super::new_note::button_shown(app);
    let tabs_w: usize = tabs.iter().map(|(t, _)| width(t)).sum();
    let right = tabs_w + if with_button { width(button) } else { 0 };
    let mut at = w.checked_sub(right)?;
    let x = x as usize;
    for (k, (t, _)) in tabs.iter().enumerate().take(2) {
        let tw = width(t);
        if x >= at && x < at + tw {
            return Some(k == 1);
        }
        at += tw;
    }
    None
}

/// ビューのタブの並び(BV-13): (始まりの桁, 見せる文字, ビューの添字)。選んだビューは `[名前]`、ほかは ` 名前 `
/// (反転の色に加えて括弧で示す。SR-15)。選んだ mdgrid のビューが定義と違う間は `[名前]*`(BV-20)。
fn tab_spans(app: &App) -> Vec<(usize, String, usize)> {
    let mut out = Vec::new();
    let mut x = 0;
    let dirty = app.native_dirty();
    let names = app.view_names();
    // NV-26: 好みの順で、隠したタブ(今のビューでなければ)を外して並べる。
    for i in app.tab_list() {
        let name = sanitize(&names[i]);
        let t = if i == app.view_index() && dirty {
            format!("[{name}]*")
        } else if i == app.view_index() {
            format!("[{name}]")
        } else {
            format!(" {name} ")
        };
        let tw = width(&t);
        out.push((x, t, i));
        x += tw + 1;
    }
    out
}

/// ビューのタブ。`.base` のビュー(`.base` なしなら「既定の表」)のあとに mdgrid のビュー(BV-20)。
/// `.base` も mdgrid のビューも無ければ既定の表の1つ(括弧なし)。
pub(crate) fn tabs(app: &App, w: usize) -> Line<'static> {
    // SR-33: モダンな見た目では、選んだタブを札、ほかを薄い色(文字は同じ)。
    let look = super::look::look(app);
    let rev = match &look {
        Some(l) => l.tab(app.style.tabs, true),
        None => Style::default().add_modifier(Modifier::REVERSED),
    };
    let other = match &look {
        Some(l) => l.tab(app.style.tabs, false),
        None => Style::default(),
    };
    if app.base.is_none() && app.nv.views.is_empty() {
        let label = Msg::TabDefault.text();
        let tab = fit(label, w.min(width(label)), Align::Left);
        return pad(vec![Span::styled(tab, rev)], w, Style::default());
    }
    let mut spans = Vec::new();
    let mut used = 0;
    for (x, t, i) in tab_spans(app) {
        if x > used {
            spans.push(Span::raw(" ".repeat(x - used)));
        }
        let st = if i == app.view_index() { rev } else { other };
        used = x + width(&t);
        spans.push(Span::styled(t, st));
    }
    let hint = Msg::TabSwitch.text();
    if app.nv.tabs.hint && app.tab_list().len() > 1 && used + width(hint) <= w {
        spans.push(Span::styled(
            hint.to_string(),
            Style::default().add_modifier(Modifier::DIM),
        ));
    }
    pad(spans, w, Style::default())
}

/// (x, y) がタブなら、そのビューの添字(BV-13 のクリック)。
pub fn tab_at(app: &App, x: u16, y: u16) -> Option<usize> {
    if y != 1 || tab_rows(app) == 0 || (app.base.is_none() && app.nv.views.is_empty()) {
        return None;
    }
    let x = x as usize;
    tab_spans(app)
        .into_iter()
        .find(|(x0, t, _)| x >= *x0 && x < x0 + width(t))
        .map(|(_, _, i)| i)
}

/// ビューのタブの行の数: 出すなら 1、隠すなら 0(SR-20。隠しても `[` `]` は効く)。
pub(crate) fn tab_rows(app: &App) -> usize {
    usize::from(app.shows(Item::Tabs) && !single_tab_hidden(app))
}

/// SR-34: `display.tabs = "auto"` で、ビューが1つしか無いのでタブの行を出さないか。
fn single_tab_hidden(app: &App) -> bool {
    app.settings.display.over(&app.display).tabs == mdgrid::display::TabsMode::Auto
        && app.view_names().len() <= 1
}

/// 検索の欄(NV-23)の行の数: 出すなら 1、出さないなら 0(設定 `display.search_bar` とビューの上書き。SR-20)。
pub(crate) fn bar_rows(app: &App) -> usize {
    usize::from(app.shows(Item::SearchBar))
}

/// 検索の欄の画面の行(ビューのタブの下。タブを隠せばヘッダーの下)。
fn bar_y(app: &App) -> u16 {
    1 + tab_rows(app) as u16
}
/// 検索の欄の前置き(今の言語。カーソルの桁もこの幅から)。
fn bar_lead() -> &'static str {
    Msg::BarLead.text()
}
/// 検索の欄で打っているときの、入力の末尾の印。
const BAR_CARET: &str = "▏";

/// 画面の行 y が描かれているか(下の帯とメッセージ行より上の行だけが表の側。低い端末では欄や帯が切れる)。
fn row_drawn(app: &App, y: u16) -> bool {
    let h = app.size.1.saturating_sub(1);
    y < h.saturating_sub(2)
}

/// 検索の欄が実際に描かれているか(設定で出し、端末の高さに収まる。NV-23)。
fn bar_drawn(app: &App) -> bool {
    bar_rows(app) > 0 && row_drawn(app, bar_y(app))
}

/// 検索の欄で打っているか(欄が描かれていて、簡易の絞り込みの入力中)。
/// 欄が描かれない低い端末では、最下行で打つ(打った語とカーソルが見えるように)。
fn bar_editing(app: &App) -> bool {
    bar_drawn(app) && app.mode == Mode::Filter
}

/// 検索の欄の語: 打っている間は入力、そうでなければ効いている簡易の絞り込みの語(NV-2)。
fn bar_text(app: &App) -> String {
    let t = if bar_editing(app) {
        app.prompt.as_ref().map(|p| p.text.as_str())
    } else {
        app.filter.as_deref()
    };
    sanitize(t.unwrap_or_default())
}

/// 検索の欄(NV-23): ` 検索: 会議▏  3/16行`。語が無ければ ` 検索: (\ で絞る)` を薄く。
/// `▏` は打っている間だけ。キーはキーの表から(SR-4)。
pub(crate) fn search_bar(app: &App, w: usize) -> Line<'static> {
    let editing = bar_editing(app);
    let dim = Style::default().add_modifier(Modifier::DIM);
    let text = bar_text(app);
    let lead_st = if editing {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut spans = vec![Span::styled(bar_lead().to_string(), lead_st)];
    if !text.is_empty() {
        spans.push(Span::styled(
            text,
            Style::default().add_modifier(Modifier::BOLD),
        ));
    }
    if editing {
        spans.push(Span::raw(BAR_CARET));
    }
    if app.filter.is_some() {
        spans.push(Span::styled(
            Msg::BarCount.fill(&[&app.rows.len(), &app.unfiltered]),
            dim,
        ));
    } else if !editing {
        let hint = match keymap::key_for(&app.keys, Mode::Table, keymap::Action::QuickFilter) {
            Some(k) => Msg::BarKey.fill(&[&k]),
            None => Msg::BarClick.text().to_string(),
        };
        spans.push(Span::styled(hint, dim));
    }
    // NV-24: 右の端に「並べ替え」のボタン(入らなければ出さない)。
    let label = super::sorts::button(app);
    let bw = width(&label);
    let used: usize = spans.iter().map(|s| width(&s.content)).sum();
    if w > bw + used {
        let mut line = pad(spans, w - bw, Style::default());
        let look = super::look::look(app);
        let st = match &look {
            Some(l) if !app.settings.sorts.is_empty() => {
                Style::default().bg(l.soft_bg).fg(l.accent)
            }
            Some(l) => Style::default().bg(l.soft_bg).fg(l.dim),
            None => Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        };
        line.spans.push(Span::styled(label, st));
        return line;
    }
    pad(spans, w, Style::default())
}

/// NV-24: (x, y) が検索の欄の右の「並べ替え」のボタンか。描いたときだけ(狭い端末や長い語で
/// ボタンを省いたら、そこは検索の欄のまま)。
pub(crate) fn sort_button_at(app: &App, x: u16, y: u16) -> bool {
    if !bar_drawn(app) || y != bar_y(app) {
        return false;
    }
    let (w, _) = super::popup::screen(app);
    let label = super::sorts::button(app);
    let drawn = search_bar(app, w)
        .spans
        .last()
        .is_some_and(|s| s.content == label.as_str());
    let bw = width(&label);
    drawn && (x as usize) >= w.saturating_sub(bw) && (x as usize) < w
}

/// NV-24: 並べ替えの窓を置く行(この行の下に開く): 検索の欄の行、無ければヘッダーの行。
pub(crate) fn sort_anchor_y(app: &App) -> usize {
    if bar_drawn(app) {
        bar_y(app) as usize
    } else {
        0
    }
}

/// (x, y) が検索の欄か(NV-23 のクリックで欄に入る)。欄が描かれていなければ false。
pub fn bar_at(app: &App, _x: u16, y: u16) -> bool {
    bar_drawn(app) && y == bar_y(app)
}

/// 設定の帯(NV-16)の行の数: 効いている設定が無いか、帯を隠せば 0(帯を出さない。SR-20)、あれば 1。
pub(crate) fn chip_rows(app: &App) -> usize {
    usize::from(app.shows(Item::Chips) && !app.settings.no_conditions())
}

/// 設定の帯の画面の行(検索の欄があればその下)。
fn chips_y(app: &App) -> u16 {
    bar_y(app) + bar_rows(app) as u16
}

/// 設定の帯の前置き(今の言語。項目の桁もこの幅から)。
fn chips_lead() -> &'static str {
    Msg::ChipsLead.text()
}

/// 設定の帯の項目の並び(NV-16): (始まりの桁, 見せる文字, 項目の添字)。項目は `[文字]`。
/// 帯を選んでいるとき(NV-22)の選んだ項目は、前の空白の代わりに `>` を付ける(反転の色に加えて。SR-15)。
fn chip_spans(app: &App) -> Vec<(usize, String, usize)> {
    let mut out = Vec::new();
    let mut x = width(chips_lead()) + 1;
    for (i, c) in app.settings.chips().iter().enumerate() {
        let t = format!("[{}]", sanitize(c));
        let tw = width(&t);
        out.push((x, t, i));
        x += tw + 1;
    }
    out
}

/// 設定の帯(NV-16): ビューのタブの下に、効いているフィルター・並べ替え・グループを1項目ずつ。
pub(crate) fn chips(app: &App, w: usize) -> Line<'static> {
    let focused = app.mode == Mode::Chips;
    let dim = Style::default().add_modifier(Modifier::DIM);
    let mut spans = vec![Span::styled(chips_lead().to_string(), dim)];
    let mut used = width(chips_lead());
    for (x, t, i) in chip_spans(app) {
        let sel = focused && i == app.chip;
        if x > used {
            let gap = x - used;
            let lead = if sel { ">" } else { " " };
            spans.push(Span::raw(format!("{}{lead}", " ".repeat(gap - 1))));
        }
        let st = if sel {
            Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default()
        };
        used = x + width(&t);
        spans.push(Span::styled(t, st));
    }
    // 外し方の案内(キーは表から。SR-4)。収まるときだけ。
    let hint = if focused {
        keymap::key_for(&app.keys, Mode::Chips, keymap::Action::RemoveItem)
            .map(|k| Msg::ChipsRemoveKey.fill(&[&k]))
    } else {
        keymap::key_for(&app.keys, Mode::Table, keymap::Action::FocusChips)
            .map(|k| Msg::ChipsSelectKey.fill(&[&k]))
    };
    if let Some(h) = hint {
        if used + width(&h) <= w {
            spans.push(Span::styled(h, dim));
        }
    }
    pad(spans, w, Style::default())
}

/// (x, y) が設定の帯の項目なら、その添字(NV-22 のクリックで外す)。
pub fn chip_at(app: &App, x: u16, y: u16) -> Option<usize> {
    if chip_rows(app) == 0 || y != chips_y(app) || !row_drawn(app, y) {
        return None;
    }
    let x = x as usize;
    chip_spans(app)
        .into_iter()
        .find(|(x0, t, _)| x >= *x0 && x < x0 + width(t))
        .map(|(_, _, i)| i)
}

pub(crate) fn footer(app: &App, w: usize) -> Line<'static> {
    // 未保存の数は狭い端末でも切れないよう先頭に置く(SR-1)。
    let mut text = Msg::FooterUnsaved.fill(&[&app.changes.count()]);
    // NV-5: 選んだ行の数。
    let n = app.selection().len();
    if n > 0 {
        text.push_str(&Msg::FooterSelected.fill(&[&n]));
    }
    text.push_str(&format!("  {}", app.mode.label()));
    if matches!(
        app.mode,
        Mode::Table
            | Mode::Edit
            | Mode::Search
            | Mode::Filter
            | Mode::ListPick
            | Mode::Menu
            | Mode::Freq
            | Mode::Sorts
    ) {
        let pos = match app.slots.get(app.row) {
            None | Some(Slot::Gap) => Msg::FooterRow.fill(&[&0, &0]),
            Some(Slot::Row(r)) => Msg::FooterRow.fill(&[&(r + 1), &app.rows.len()]),
            Some(Slot::Head(g)) => Msg::FooterHead.fill(&[&(g + 1), &app.groups.len()]),
        };
        let col = match app.cols.get(app.col) {
            Some(_) => Msg::FooterCol.fill(&[&(app.col + 1), &app.cols.len()]),
            None => String::new(),
        };
        text.push_str(&format!(" {pos}{col}"));
        // BV-7: 評価できない式のセルを選んだら、下の帯にも出す(薄い表示が出ない端末でも見分けられる)。
        if let Some((row, col)) = app.selected() {
            if shown(app, &row, &col).unsupported.is_some() {
                text.push_str(Msg::FooterUnsupported.text());
            }
        }
    }
    // 今押せるキーは順位の順に、収まる分だけ(途中で切らない)。
    // CE-25: 新しいノートの入力では、その入力のキー(次へ・前の項目・やめる)。
    // SR-25: 前置きのキーを待っている間は「`前置き` → 続きのキー 表示名 …」(続きか Esc で元に戻る)。
    let hints = match &app.prefix {
        Some(p) => prefix_hints(app, p),
        None => super::new_note::hints(app).unwrap_or_else(|| keymap::hints(&app.keys, app.mode)),
    };
    let head = text.clone();
    let mut shown: Vec<String> = Vec::new();
    for h in hints {
        if width(&text) + 2 + width(&h) > w {
            break;
        }
        text.push_str("  ");
        text.push_str(&h);
        shown.push(h);
    }
    // SR-33: モダンな見た目では反転の帯をやめ、キーをアクセントの太字、説明を薄い色にする(文字は同じ)。
    if let Some(l) = super::look::look(app) {
        if width(&text) <= w {
            // テーマの色の表があれば、帯の地と文字はテーマの帯の色のまま(SR-26)。
            let base = match l.band {
                Some((bg, fg)) => Style::default().bg(bg).fg(fg),
                None => Style::default(),
            };
            let mut spans = vec![Span::styled(head, base.add_modifier(Modifier::BOLD))];
            for h in shown {
                spans.push(Span::styled("  ", base));
                let (k, label) = h.split_once(' ').unwrap_or((h.as_str(), ""));
                spans.push(Span::styled(
                    k.to_string(),
                    base.patch(l.band_key(app.style.band)),
                ));
                if !label.is_empty() {
                    spans.push(Span::styled(
                        format!(" {label}"),
                        base.patch(Style::default().fg(l.band_dim)),
                    ));
                }
            }
            return pad(spans, w, base);
        }
    }
    Line::from(Span::styled(
        fit(&text, w, Align::Left),
        Style::default().add_modifier(Modifier::REVERSED),
    ))
}

/// 前置きのキーの続きの案内(SR-25): 先頭に `前置き →`、続けて押せるキーと表示名。
/// 操作の一覧(SR-24)の上では、一覧のモードの続きと、表のモードの続きのうち一覧の項目の動作だけ
/// (一覧の上のほかのキーは表のモードのキーとして読むので)。
fn prefix_hints(app: &App, prefix: &str) -> Vec<String> {
    let mut out = vec![format!("{} →", keymap::display(prefix))];
    let own = keymap::continuations(&app.keys, app.mode, prefix);
    out.extend(own.into_iter().map(|(_, s)| s));
    if app.mode == Mode::Menu {
        let items = super::menu::item_actions(app);
        out.extend(
            keymap::continuations(&app.keys, Mode::Table, prefix)
                .into_iter()
                .filter(|(a, _)| items.contains(a))
                .map(|(_, s)| s),
        );
    }
    out
}

/// メッセージ行: 直前の操作の知らせ、無ければ選んだセル・行の案内(読むだけの理由など。SR-15)。
pub(crate) fn message(app: &App, w: usize) -> Line<'static> {
    // 検索・簡易の絞り込みの入力(NV-1・NV-2)は、メッセージ行に出す。
    if let Some(mut t) = prompt_text(app) {
        if let Some(m) = &app.message {
            t.push_str(&format!("  ({})", sanitize(m)));
        }
        return Line::from(fit(&t, w, Align::Left));
    }
    let text = app
        .message
        .clone()
        // CE-25: 新しいノートの入力では、その入り方だけ(セルの案内は出さない)。
        .or_else(|| super::new_note::hint(app).unwrap_or_else(|| super::input::hint(app)))
        .or_else(|| super::listpick::hint(app))
        .or_else(|| {
            if app.note.flow.is_some() {
                return None;
            }
            if let Some(r) = &app.review {
                let item = r.items.get(r.sel)?;
                return Some(if item.external {
                    Msg::HintReviewExternal.text().into()
                } else {
                    Msg::HintReview.text().into()
                });
            }
            if app.view_error.is_some() {
                return Some(Msg::HintViewError.text().into());
            }
            let notes = || {
                (!app.notes.is_empty())
                    .then(|| Msg::HintUnsupported.fill(&[&app.notes.join(" / ")]))
            };
            if let Some(g) = app.cur_head() {
                let n = app.groups.get(g).map_or(0, |(_, r)| r.len());
                return Some(Msg::HintHeadRow.fill(&[&n])).or_else(notes);
            }
            let Some((row, col)) = app.selected() else {
                return notes();
            };
            if app.changes.external(&row) {
                return Some(Msg::HintExternalNote.text().into());
            }
            if app.held.contains(&row) {
                return Some(Msg::HintHeld.fill(&[&HELD_MARK]));
            }
            if let Some(m) = app.src.mark(&row) {
                return Some(m);
            }
            let s = shown(app, &row, &col);
            if let Some(r) = s.unsupported {
                return Some(Msg::HintUnsupportedFormula.fill(&[&r]));
            }
            s.locked
                .map(|r| Msg::HintReadOnly.fill(&[&r]))
                .or_else(notes)
        });
    Line::from(fit(&sanitize(&text.unwrap_or_default()), w, Align::Left))
}

/// 最下行の検索・簡易の絞り込みの入力の文字(`/語`・`\語`)。最下行で入力中でなければ None
/// (検索の欄があれば、簡易の絞り込みは欄で打つ。NV-23)。
fn prompt_text(app: &App) -> Option<String> {
    let p = app.prompt.as_ref()?;
    let lead = match app.mode {
        Mode::Search => "/",
        Mode::Filter if !bar_drawn(app) => "\\",
        _ => return None,
    };
    Some(format!("{lead}{}", sanitize(&p.text)))
}

/// 入力のカーソル(SR-17: 変換の窓がそこに出る)。検索の欄で打っていれば欄の語の末尾、
/// 最下行で打っていればメッセージ行の末尾。
pub(crate) fn prompt_cursor(app: &App) -> Option<(u16, u16)> {
    let w = app.size.0.saturating_sub(1) as usize;
    let (x, y) = if bar_editing(app) {
        (width(bar_lead()) + width(&bar_text(app)), bar_y(app))
    } else {
        (width(&prompt_text(app)?), app.size.1.saturating_sub(2))
    };
    (x < w).then_some((x as u16, y))
}

/// 終了の確認(WB-11)。表の中ほどに帯で重ねる。キーはキーの表から(SR-4)。
pub(crate) fn quit_overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let keys = keymap::hints(&app.keys, Mode::Quit).join("   ");
    let band_lines = [
        String::new(),
        Msg::QuitBand.fill(&[&app.changes.count()]),
        format!("   {keys}"),
        String::new(),
    ];
    // lines は表の部分(ヘッダーから表の末尾まで)。表の中ほどに重ねる。
    let start = (lines.len() / 2)
        .saturating_sub(band_lines.len() / 2)
        .max(super::view::data_y(app));
    for (k, t) in band_lines.iter().enumerate() {
        if let Some(l) = lines.get_mut(start + k) {
            *l = band(t, w);
        }
    }
}

pub(crate) fn band(text: &str, w: usize) -> Line<'static> {
    Line::from(Span::styled(
        fit(&sanitize(text), w, Align::Left),
        Style::default().add_modifier(Modifier::REVERSED),
    ))
}

/// SR-42: ヘッダーの近道のボタン。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderButton {
    Workspace,
    Settings,
}

/// SR-42: ヘッダーのボタン(タブの左の端からの桁, 文字, ボタン)。幅 `w` に、名前の欄を少し残して収まらなければ空。
pub(crate) fn header_buttons(app: &App, w: usize) -> Vec<(usize, String, HeaderButton)> {
    let ws = match &app.scope {
        Some(s) if !matches!(s.source, mdgrid::workspace::Source::Detected(_)) => {
            Msg::HeaderWsNamed.fill(&[&sanitize(&s.name)])
        }
        _ => Msg::HeaderWsButton.text().to_string(),
    };
    // ワークスペースが無くても出す(押すと、作り方を知らせる。WS-6)。
    let items = vec![
        (ws, HeaderButton::Workspace),
        (
            Msg::HeaderSettingsButton.text().to_string(),
            HeaderButton::Settings,
        ),
    ];
    let need: usize = items.iter().map(|(t, _)| width(t) + 1).sum();
    let tabs_w: usize = screen_tabs(app).iter().map(|(t, _)| width(t)).sum();
    let bw = if super::new_note::button_shown(app) {
        width(super::new_note::button())
    } else {
        0
    };
    // 左の名前と行の数などが切れずに収まるときだけ(ボタンは近道なので、狭ければ隠す)。
    if w < need + tabs_w + bw + width(&header_text(app)) + 2 {
        return Vec::new();
    }
    let mut x = 0;
    items
        .into_iter()
        .map(|(t, b)| {
            let at = x;
            x += width(&t) + 1;
            (at, t, b)
        })
        .collect()
}

/// SR-42: (x, y) がヘッダーのボタンなら、そのボタン。
pub(crate) fn header_button_at(app: &App, x: u16, y: u16) -> Option<HeaderButton> {
    if y != 0 {
        return None;
    }
    let (w, _) = super::popup::screen(app);
    let buttons = header_buttons(app, w);
    let buttons_w: usize = buttons.iter().map(|(_, t, _)| width(t) + 1).sum();
    let tabs_w: usize = screen_tabs(app).iter().map(|(t, _)| width(t)).sum();
    let bw = if super::new_note::button_shown(app) {
        width(super::new_note::button())
    } else {
        0
    };
    let start = w.checked_sub(buttons_w + tabs_w + bw)?;
    let x = x as usize;
    buttons
        .into_iter()
        .find(|(at, t, _)| x >= start + at && x < start + at + width(t))
        .map(|(_, _, b)| b)
}
