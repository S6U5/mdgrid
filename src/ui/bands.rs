//! 画面の帯(`view` の続きの描画の純関数): ヘッダー・ビューのタブ(SR-1・BV-13)、検索の欄(NV-23・NV-2)、
//! 設定の帯(NV-16・NV-22)、下の帯(SR-1・NV-5 の「選択 N」)、
//! メッセージ行(SR-15・BV-7・NV-12 の印の説明)と、検索・簡易の絞り込みの入力の行(NV-1・NV-2)。

use super::app::App;
use super::cell::shown;
use super::glyphs::{glyph, Glyph};
use super::grid::Slot;
use super::keymap::{self, Mode};
use super::view::{pad, HELD_MARK};
use super::width::{fit, sanitize, width, Align};
use mdgrid::display::{ButtonsMode, Item};
use mdgrid::i18n::Msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// ヘッダーの左の文字(名前・行の数・読み込み・絞り込み・隠した列・ワークスペースの名前)。
fn header_text(app: &App) -> String {
    // SR-1: `.base` を開いたらその名前、無ければフォルダの名前。
    // SR-42・WS-6: パンくず「範囲 › 表」(範囲は書いたワークスペースのとき。検知した範囲は関係マップでだけ)。
    let mut text = format!(
        " {}  {}",
        crumb_text(app, &table_name(app)),
        Msg::HeaderRows.fill(&[&app.rows.len()])
    );
    // SR-20: タブを隠していれば、今のビューの名前(定義と違えば `*`。BV-20)をヘッダーに出す。
    if !tabs_shown(app) && !single_tab_hidden(app) {
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
    // NV-30: 問題のある行だけに絞っている。
    if app.only_problems {
        text.push_str(Msg::HeaderProblems.text());
    }
    if !app.hidden.is_empty() {
        text.push_str(&Msg::HeaderHidden.fill(&[&app.hidden.len()]));
    }
    text
}

/// パンくずに出す範囲の名前(SR-42・WS-6): 書いたワークスペースの範囲のとき。検知した範囲は関係マップでだけ。
fn crumb_scope(app: &App) -> Option<String> {
    let s = app.scope.as_ref()?;
    let detected = matches!(s.source, mdgrid::workspace::Source::Detected(_));
    (on_relmap(app) || !detected).then(|| sanitize(&s.name))
}

/// パンくずのボタンの文字「キー 範囲 › 表 ▾」(範囲が無ければ表だけ。キーは引き出しのキーで、割り当てが
/// 無ければ添えない。SR-42)。
fn crumb_text(app: &App, name: &str) -> String {
    let crumb = match crumb_scope(app) {
        Some(sc) => format!("{sc} › {}", sanitize(name)),
        None => sanitize(name),
    };
    match keymap::key_for(&app.keys, Mode::Table, keymap::Action::WsDrawer) {
        Some(k) => format!("{k} {crumb} ▾"),
        None => format!("{crumb} ▾"),
    }
}

/// 今の表の名前(`.base` を開いたらその名前、無ければフォルダの名前)。
fn table_name(app: &App) -> String {
    match &app.base {
        Some(b) => b.name.clone(),
        None => app.src.name(),
    }
}

/// SR-42: (x, y) がヘッダーのパンくず(範囲と表の名前)か。押すと表の一覧の引き出し(WS-8)。
pub(crate) fn crumb_at(app: &App, x: u16, y: u16) -> bool {
    if y != head_y(app) {
        return false;
    }
    let x = x as usize;
    x >= 1 && x < 1 + width(&crumb_text(app, &table_name(app)))
}

/// 関係マップの画面か(画面の型のメニューを関係マップから開いている間も)。
pub(crate) fn on_relmap(app: &App) -> bool {
    app.mode == Mode::Relations || (app.mode == Mode::ScreenMenu && app.relmap.is_some())
}

/// TR-1: 木の画面か(画面の型のメニューとパレットを木から開いている間も)。
pub(crate) fn on_tree(app: &App) -> bool {
    app.mode == Mode::Tree
        || (matches!(app.mode, Mode::ScreenMenu | Mode::Palette) && app.folder_tree.is_some())
}

/// ボタンの見た目(NV-23。ビューの帯とヘッダーのボタンで同じ)。`on` は効いているか。
fn button_style(look: &Option<super::look::Look>, on: bool) -> Style {
    match look {
        Some(l) if on => Style::default().bg(l.soft_bg).fg(l.accent),
        Some(l) => Style::default().bg(l.soft_bg).fg(l.dim),
        None if on => Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        None => Style::default().add_modifier(Modifier::UNDERLINED),
    }
}

/// NV-23・SR-42: ボタンの頭のキーの字の見た目。名前の見た目 `base` に太字を足し、モダンな見た目では
/// テーマの強調の色にする。色を使わない表示で名前が既に太字なら、下線の有る無しで名前と見分ける。
fn key_style(look: &Option<super::look::Look>, base: Style) -> Style {
    match look {
        Some(l) => base.fg(l.accent).add_modifier(Modifier::BOLD),
        // 名前が既に太字なら、下線の有る無しを名前と逆にして見分ける。
        None if base.add_modifier.contains(Modifier::BOLD) => {
            if base.add_modifier.contains(Modifier::UNDERLINED) {
                base.remove_modifier(Modifier::UNDERLINED)
            } else {
                base.add_modifier(Modifier::UNDERLINED)
            }
        }
        None => base.add_modifier(Modifier::BOLD),
    }
}

/// ボタンの文字 `t`(頭が ` キー ` か `キー `)を、キーの字だけ別の見た目にした span の並びにする。
fn key_spans(
    t: &str,
    key: Option<&str>,
    st: Style,
    look: &Option<super::look::Look>,
) -> Vec<Span<'static>> {
    let lead = if t.starts_with(' ') { " " } else { "" };
    match key {
        Some(k) if !k.is_empty() && t[lead.len()..].starts_with(&format!("{k} ")) => {
            let rest = &t[lead.len() + k.len()..];
            let mut out = Vec::new();
            if !lead.is_empty() {
                out.push(Span::styled(lead.to_string(), st));
            }
            out.push(Span::styled(k.to_string(), key_style(look, st)));
            out.push(Span::styled(rest.to_string(), st));
            out
        }
        _ => vec![Span::styled(t.to_string(), st)],
    }
}

/// ヘッダーの右の端のボタン(SR-42)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderButton {
    /// 画面の型のメニュー(REL-8)。
    Menu,
    /// ビューの設定の画面(`o`)。
    Settings,
    /// CE-25: 「+ 新規」(ビューの帯を出さないときだけ。表の画面と関係マップ)。
    New,
}

/// 設定のボタンを出す描ける幅の下限(端末の幅 80。右端の1桁は描かない。狭いときは先に省く。設定は `o` と
/// パレットでも開ける)。
const SETTINGS_MIN_W: usize = 79;

/// 画面の型のメニューのボタンの文字(前後に空白1つ): 表の画面「▦ 表 ▾」、関係マップ「◇ 関係マップ ▾」。
fn menu_button_text(app: &App) -> String {
    if on_relmap(app) {
        format!(
            " {} {} ▾ ",
            glyph(app, Glyph::RelMap),
            Msg::ScreenRelMap.text()
        )
    } else if on_tree(app) {
        format!(" {} {} ▾ ", glyph(app, Glyph::Tree), Msg::ScreenTree.text())
    } else {
        format!(
            " {} {} ▾ ",
            glyph(app, Glyph::Table),
            Msg::TabScreenTable.text()
        )
    }
}

/// 設定のボタンの (文字, キー): 「キー ⚙ 設定」(キーはビューの設定のキー。割り当てが無ければ添えない)。
fn settings_button_text(app: &App) -> (String, Option<String>) {
    let key = keymap::key_for(&app.keys, Mode::Table, keymap::Action::ViewSettings);
    let label = format!("{} {}", glyph(app, Glyph::Gear), Msg::HeaderSettings.text());
    let t = match &key {
        Some(k) => format!(" {k} {label} "),
        None => format!(" {label} "),
    };
    (t, key)
}

/// 幅 `w` のヘッダーの右の端のボタンの置き方: (桁, 文字, キー, ボタン)。左から「+ 新規」(ビューの帯を出さない
/// ときだけ。CE-25)・メニュー・設定。幅が足りなければ設定を先に省き、次に「+ 新規」を省き、パンくずとメニューを残す。
fn header_buttons(app: &App, w: usize) -> Vec<(usize, String, Option<String>, HeaderButton)> {
    let crumb_w = width(&crumb_text(app, &table_name(app)));
    let menu = menu_button_text(app);
    let (set, set_key) = settings_button_text(app);
    let new = header_new_shown(app).then(|| new_button_text(app));
    // パンくずの後ろに少なくとも2桁の空きを残す。
    let left = 1 + crumb_w + 2;
    // 残す順に並べた組み合わせのうち、入る最初のもの。
    let mut cands: Vec<Vec<(String, Option<String>, HeaderButton)>> = Vec::new();
    let m = (menu.clone(), None, HeaderButton::Menu);
    let st = (set.clone(), set_key.clone(), HeaderButton::Settings);
    if w >= SETTINGS_MIN_W {
        match &new {
            Some((nt, nk)) => cands.push(vec![
                (nt.clone(), nk.clone(), HeaderButton::New),
                m.clone(),
                st.clone(),
            ]),
            None => cands.push(vec![m.clone(), st.clone()]),
        }
    }
    if let Some((nt, nk)) = &new {
        cands.push(vec![(nt.clone(), nk.clone(), HeaderButton::New), m.clone()]);
    }
    cands.push(vec![m]);
    for c in cands {
        let total: usize = c.iter().map(|(t, _, _)| width(t)).sum::<usize>() + c.len() - 1;
        if left + total <= w {
            let mut x = w - total;
            let mut out = Vec::new();
            for (t, k, b) in c {
                let tw = width(&t);
                out.push((x, t, k, b));
                x += tw + 1;
            }
            return out;
        }
    }
    Vec::new()
}

/// CE-25: ヘッダーに「+ 新規」を出すか: ビューの帯を出さない(SR-20・SR-34 でタブも検索の欄も出さない)とき。
/// 表の画面と関係マップで同じ(関係マップは表の画面の帯の有無に従う)。
pub(crate) fn header_new_shown(app: &App) -> bool {
    tab_rows(app) == 0 && super::new_note::button_shown(app)
}

/// SR-42・REL-8: (x, y) がヘッダーの右の端のボタンなら、そのボタン。
pub(crate) fn header_button_at(app: &App, x: u16, y: u16) -> Option<HeaderButton> {
    if y != head_y(app) {
        return None;
    }
    let (w, _) = super::popup::screen(app);
    let x = x as usize;
    header_buttons(app, w)
        .into_iter()
        .find(|(x0, t, _, _)| x >= *x0 && x < x0 + width(t))
        .map(|(_, _, _, b)| b)
}

/// REL-8: 画面の型のメニューのボタンの左の桁(窓をそのすぐ下に開く)。描かれていなければ None。
pub(crate) fn menu_anchor(app: &App) -> Option<usize> {
    let (w, _) = super::popup::screen(app);
    header_buttons(app, w)
        .into_iter()
        .find(|(_, _, _, b)| *b == HeaderButton::Menu)
        .map(|(x, _, _, _)| x)
}

pub(crate) fn header(app: &App, w: usize) -> Line<'static> {
    let text = header_text(app);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let look = super::look::look(app);
    // SR-33: 名前の色はテーマのヘッダーの色(テーマが無ければ端末の色)のまま(SR-26)。
    let name_st = bold;
    // SR-42: パンくずは引き出しのボタン。ビューの帯のボタンと同じ地の色(色なしでは下線)で、名前は太字のまま。
    let crumb_st = match &look {
        Some(l) => bold.bg(l.soft_bg),
        None => bold.add_modifier(Modifier::UNDERLINED),
    };
    let crumb_w = width(&crumb_text(app, &table_name(app)));
    let crumb_key = keymap::key_for(&app.keys, Mode::Table, keymap::Action::WsDrawer);
    let left = |avail: usize| -> Vec<Span<'static>> {
        let t = fit(&text, avail, Align::Left);
        // 先頭の空白の次からボタンの幅だけ(詰められていれば入る分だけ)をボタンの見た目にする。
        let mut head = String::new();
        let mut btn = String::new();
        let mut rest = String::new();
        let mut at = 0;
        for c in t.chars() {
            let cw = width(&c.to_string());
            if at < 1 {
                head.push(c);
            } else if at < 1 + crumb_w {
                btn.push(c);
            } else {
                rest.push(c);
            }
            at += cw;
        }
        let mut spans = vec![Span::styled(head, name_st)];
        // NV-23・SR-42: 頭の引き出しのキーの字は名前と見分ける見た目。
        spans.extend(key_spans(&btn, crumb_key.as_deref(), crumb_st, &look));
        spans.push(Span::styled(rest, name_st));
        spans
    };
    // SR-42・REL-8: 右の端に画面の型のメニューと設定のボタン(ビューの帯のボタンと同じ見た目)。
    let buttons = header_buttons(app, w);
    let Some(x0) = buttons.first().map(|(x, _, _, _)| *x) else {
        return head_bg(app, Line::from(left(w)));
    };
    let mut spans = left(x0);
    let mut used = x0;
    let st = button_style(&look, false);
    for (x, t, key, _) in buttons {
        if x > used {
            spans.push(Span::raw(" ".repeat(x - used)));
        }
        used = x + width(&t);
        spans.extend(key_spans(&t, key.as_deref(), st, &look));
    }
    head_bg(app, pad(spans, w, Style::default()))
}

/// SR-1: 上の余白に端数があるときは、ヘッダーの行に帯の地の色を敷く。
fn head_bg(app: &App, line: Line<'static>) -> Line<'static> {
    match top_pad(app).1.then(|| half_bg(app)).flatten() {
        Some(c) => lay_bg(line, c),
        None => line,
    }
}

/// ビューのタブの並び(BV-13): (始まりの桁, 見せる文字, ビューの添字)。選んだビューは `[名前]`、ほかは ` 名前 `
/// (反転の色に加えて括弧で示す。SR-15)。選んだ mdgrid のビューが定義と違う間は `[名前]*`(BV-20)。
/// `.base` も mdgrid のビューも無ければ既定の表の1つ(括弧なし)。
fn tab_spans(app: &App) -> Vec<(usize, String, usize)> {
    if no_views(app) {
        return vec![(0, Msg::TabDefault.text().to_string(), app.view_index())];
    }
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

/// `.base` も mdgrid のビューも無い(タブは既定の表の1つだけで、押しても何もしない)。
fn no_views(app: &App) -> bool {
    app.base.is_none() && app.nv.views.is_empty()
}

/// ビューの帯にタブを出すか(SR-20・SR-34。隠しても `[` `]` は効く)。
fn tabs_shown(app: &App) -> bool {
    app.shows(Item::Tabs) && !single_tab_hidden(app)
}

/// ビューの帯に検索の欄を出すか(NV-23。設定 `display.search_bar` とビューの上書き。SR-20)。
fn search_shown(app: &App) -> bool {
    app.shows(Item::SearchBar)
}

/// ビューの帯(SR-1)の行の数: タブか検索の欄を出すなら 1(3行の形(plain・box)なら 3)、両方隠せば 0。
pub(crate) fn tab_rows(app: &App) -> usize {
    if !(tabs_shown(app) || search_shown(app)) {
        0
    } else {
        1 + 2 * bar_padding(app) + wrap_rows(app)
    }
}

/// NV-31: 帯の下に足した行の数(`display.tab_wrap` で、タブが帯の行に入らないとき。タブの行と、
/// `display.tab_wrap_gap` の空き行)。
pub(crate) fn wrap_rows(app: &App) -> usize {
    let n = tab_lines(app);
    n + usize::from(n > 0 && wrap_gap(app))
}

/// NV-31: 検索の欄の行の下に並べたタブの行の数。
fn tab_lines(app: &App) -> usize {
    thread_local! {
        /// 帯を置く途中(検索の欄が描かれるかを見るときに帯の行を引く)で、もう一度数えないための印。
        static COUNTING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if !(tab_wrap(app) || has_breaks(app)) || COUNTING.with(|c| c.get()) {
        return 0;
    }
    COUNTING.with(|c| c.set(true));
    let (w, _) = super::popup::screen(app);
    let n = view_bar(app, w).wrapped.len();
    COUNTING.with(|c| c.set(false));
    n
}

/// NV-31: タブを折り返す設定か(タブを出しているときだけ)。
fn tab_wrap(app: &App) -> bool {
    tabs_shown(app) && app.settings.display.over(&app.display).tab_wrap
}

/// NV-32: ビュー i のタブの前でタブの行を改めるか(`.base` のビューの `tab_break`)。
fn tab_break(app: &App, i: usize) -> bool {
    app.base
        .as_ref()
        .and_then(|b| b.base.views.get(i))
        .is_some_and(|v| v.tab_break)
}

/// NV-32: タブの行を改めるタブ(ビューの添字)。タブの行に出すタブのうち先頭でないもので、自分か、前に見せる
/// タブとの間の隠したタブ(NV-26)に `tab_break` があるもの(まとまりの先頭を隠しても、まとまりが混ざらないように)。
fn row_starts(app: &App) -> Vec<usize> {
    if !tabs_shown(app) || no_views(app) {
        return Vec::new();
    }
    let shown = app.tab_list();
    let mut out = Vec::new();
    let mut pending = false;
    for i in app.tab_order() {
        pending |= tab_break(app, i);
        if shown.contains(&i) {
            if pending && shown.first() != Some(&i) {
                out.push(i);
            }
            pending = false;
        }
    }
    out
}

/// NV-32: タブの行を改めるタブがあるか。
fn has_breaks(app: &App) -> bool {
    !row_starts(app).is_empty()
}

/// NV-31: 検索の欄の行と、下に並べたタブの行の間に空き行を入れるか(`display.tab_wrap_gap`)。
fn wrap_gap(app: &App) -> bool {
    app.settings.display.over(&app.display).tab_wrap_gap
}

/// 帯の上下に余白を取る端末の高さの下限(これより低いと表の行を残すため余白なしの1行で描く。NV-23)。
const BOX_MIN_H: u16 = 16;

/// SR-1・NV-23: 余白の端数を半分の塗りの字(`▄`・`▀`)で描けるか。色を使わない表示と、あいまいな字を全角として
/// 数える設定(半分の塗りの字が2桁になりうる)と、画面の地の色が分からないとき(テーマが画面の地を塗らず、
/// 端末の地の色も分からない)は描かず、端数を1行に切り上げる。
fn half_ok(app: &App) -> bool {
    app.color != super::app::ColorMode::None
        && !app.ambiguous_wide
        && super::look::screen_bg(app).is_some()
}

/// NV-23: 起動時に端末の地の色を問い合わせるか: 色ありの表示で、あいまいな字を全角として数えず、テーマが画面の
/// 地を塗らず、余白(ヘッダーの上か帯)に端数があるとき。
pub(crate) fn wants_term_bg(app: &App) -> bool {
    let d = app.settings.display.over(&app.display);
    app.color != super::app::ColorMode::None
        && !app.ambiguous_wide
        && app.palette().is_none()
        && (d.top_padding.0 % 2 == 1 || d.bar_padding.0 % 2 == 1)
}

/// NV-23: 帯の上下の余白の (空き行の数, 帯に接する半分の塗りの行があるか)。設定 `display.bar_padding`
/// (0〜2 の 0.5 刻み。既定 1)。検索の欄を出さないときと、端末の高さが BOX_MIN_H に満たないときは 0。
/// box では半分の行も線の行にするので、端数は切り上げる。
fn bar_pad(app: &App) -> (usize, bool) {
    if !search_shown(app) || app.size.1 < BOX_MIN_H {
        return (0, false);
    }
    let d = app.settings.display.over(&app.display);
    let boxy = d.buttons == ButtonsMode::Box;
    d.bar_padding.rows(half_ok(app) && !boxy)
}

/// NV-23: 帯の上下それぞれの余白の行の数(半分の塗りの行を含む)。
pub(crate) fn bar_padding(app: &App) -> usize {
    let (n, half) = bar_pad(app);
    n + usize::from(half)
}

/// SR-1: ヘッダーの上の余白の (空き行の数, ヘッダーに接する半分の塗りの行があるか)。設定 `display.top_padding`
/// (0〜2 の 0.5 刻み。既定 1)。端末の高さが BOX_MIN_H に満たないときは 0。
fn top_pad(app: &App) -> (usize, bool) {
    if app.size.1 < BOX_MIN_H {
        return (0, false);
    }
    app.settings
        .display
        .over(&app.display)
        .top_padding
        .rows(half_ok(app))
}

/// SR-1: ヘッダーの上の余白の行の数(表の画面と関係マップ)。
pub(crate) fn top_rows(app: &App) -> usize {
    let (n, half) = top_pad(app);
    n + usize::from(half)
}

/// SR-1: ヘッダーの画面の行。
pub(crate) fn head_y(app: &App) -> u16 {
    top_rows(app) as u16
}

/// ビューの帯の一番上の画面の行(ヘッダーの下)。
fn bar_y(app: &App) -> u16 {
    head_y(app) + 1
}

/// SR-1・NV-23: 余白の端数を描くときに敷く帯の地の色(ヘッダーは上の余白、ビューの帯は帯の余白に端数が
/// あるとき)。色を使わない表示などでは None。
fn half_bg(app: &App) -> Option<ratatui::style::Color> {
    half_ok(app).then(|| super::look::bar_bg(app)).flatten()
}

/// 行の地を `bg` にする(自分の地の色を持つ字はそのまま)。
fn lay_bg(line: Line<'static>, bg: ratatui::style::Color) -> Line<'static> {
    let spans = line
        .spans
        .into_iter()
        .map(|sp| {
            let st = if sp.style.bg.is_none() {
                sp.style.bg(bg)
            } else {
                sp.style
            };
            Span::styled(sp.content, st)
        })
        .collect::<Vec<_>>();
    Line::from(spans).style(line.style.bg(bg))
}

/// 半分の塗りの行(`c` は帯の外側の半分の字。上の半行は `▀`、下の半行は `▄`)。セルの地は帯の地の色で塗り、
/// 外側の半分を画面の地の色の字で消す(帯の行とセルの地の色で同じに見えるように)。
fn half_line(app: &App, c: char, w: usize, bg: Option<ratatui::style::Color>) -> Line<'static> {
    match (bg, super::look::screen_bg(app)) {
        (Some(band), Some(screen)) => Line::from(Span::styled(
            c.to_string().repeat(w),
            Style::default().fg(screen).bg(band),
        ))
        .style(Style::default().bg(band)),
        _ => Line::from(" ".repeat(w)),
    }
}

/// SR-1: ヘッダーの上の余白の行(空き行と、ヘッダーに接する半行。セルの地を帯の色で塗り、上の半分を画面の地の色の `▀` で消す)。
pub(crate) fn top_lines(app: &App, w: usize) -> Vec<Line<'static>> {
    let (n, half) = top_pad(app);
    let mut out: Vec<Line<'static>> = (0..n).map(|_| Line::from(" ".repeat(w))).collect();
    if half {
        out.push(half_line(app, '▀', w, half_bg(app)));
    }
    out
}

/// 帯に上下の余白があるか(3行以上の帯)。
pub(crate) fn three_rows(app: &App) -> bool {
    bar_padding(app) > 0
}

/// NV-23: 検索の欄と絞り込み・並べ替えのボタンを線で四角く囲む(`buttons = "box"`)か。線は余白の行に
/// 描くので、余白が 0 なら囲まない。
pub(crate) fn boxed(app: &App) -> bool {
    three_rows(app) && app.settings.display.over(&app.display).buttons == ButtonsMode::Box
}

/// 囲むボタンか(絞り込み・並べ替え)。
fn boxed_button(b: ViewButton) -> bool {
    matches!(b, ViewButton::Filter | ViewButton::Sort)
}

/// NV-31: 下に並べたタブの一番上の画面の行。真ん中の行のすぐ下(囲む帯では囲みの下の線の下)に、
/// `tab_wrap_gap` なら空き行を1つ挟む。
fn wrap_y(app: &App) -> u16 {
    bar_mid(app) + 1 + u16::from(boxed(app)) + u16::from(wrap_gap(app))
}

/// ビューの帯の真ん中の行(タブ・欄の字・ボタンの字の行)。1行の帯なら帯の行そのもの。
fn bar_mid(app: &App) -> u16 {
    bar_y(app) + bar_padding(app) as u16
}

/// ビューの帯の字の行(試験から。囲むときは3行の真ん中)。
#[cfg(test)]
pub(crate) fn mid_row(app: &App) -> usize {
    bar_mid(app) as usize
}

/// 行 y が帯の行か: 囲みの中のもの(`in_box`)は囲みの3行のどこでも、ほかは真ん中の行だけ。
/// plain の上下の空きの行は、どのボタンでもない。
fn bar_row(app: &App, y: u16, in_box: bool) -> bool {
    if in_box && boxed(app) {
        let m = bar_mid(app);
        (m - 1..=m + 1).contains(&y)
    } else {
        y == bar_mid(app)
    }
}

/// SR-34: `display.tabs = "auto"` で、ビューが1つしか無いのでタブを出さないか。
fn single_tab_hidden(app: &App) -> bool {
    app.settings.display.over(&app.display).tabs == mdgrid::display::TabsMode::Auto
        && app.view_names().len() <= 1
}

/// 検索の欄だけの行の数(SR-1 で検索の欄はビューの帯の中に入ったので、いつも 0)。
pub(crate) fn bar_rows(_app: &App) -> usize {
    0
}

/// 検索の欄で打っているときの、入力の末尾の印。
const BAR_CARET: &str = "▏";
/// 検索の欄の中身の幅の下限(語が短くても押しやすいように)。
const SEARCH_MIN: usize = 14;
/// タブと検索の欄の間の空き。
const TABS_GAP: usize = 3;

/// 画面の行 y が描かれているか(下の帯とメッセージ行より上の行だけが表の側。低い端末では欄や帯が切れる)。
fn row_drawn(app: &App, y: u16) -> bool {
    let h = app.size.1.saturating_sub(1);
    y < h.saturating_sub(2)
}

/// 検索の欄が実際に描かれているか(設定で出し、端末の高さに収まる。NV-23)。
fn bar_drawn(app: &App) -> bool {
    search_shown(app) && row_drawn(app, bar_mid(app))
}

/// 検索の欄で打っているか(欄が描かれていて、検索の入力中)。
/// 欄が描かれない低い端末では、最下行で打つ(打った語とカーソルが見えるように)。
fn bar_editing(app: &App) -> bool {
    bar_drawn(app) && app.mode == Mode::Filter
}

/// 検索の欄の語: 打っている間は入力、そうでなければ効いている検索の語(NV-2)。
fn bar_text(app: &App) -> String {
    let t = if bar_editing(app) {
        app.prompt.as_ref().map(|p| p.text.as_str())
    } else {
        app.filter.as_deref()
    };
    sanitize(t.unwrap_or_default())
}

/// 検索の欄の頭(NV-23): キーと「検索」(例 `\ 検索`)。キーはキーの表から(SR-4)。
fn search_lead(app: &App) -> String {
    let name = Msg::BarName.text();
    match keymap::key_for(&app.keys, Mode::Table, keymap::Action::QuickFilter) {
        Some(k) => format!("{k} {name}"),
        None => name.to_string(),
    }
}

/// 検索の欄の中身の span(前後の空白を除く): 頭・語・打っている印・合う行の数。語が無ければ「…」。
fn search_spans(app: &App) -> Vec<(String, Style)> {
    let editing = bar_editing(app);
    let dim = Style::default().add_modifier(Modifier::DIM);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let text = bar_text(app);
    let lead_st = if editing { bold } else { Style::default() };
    let mut out = vec![(search_lead(app), lead_st)];
    if editing || app.filter.is_some() {
        out.push((": ".to_string(), lead_st));
        out.push((text, bold));
        if editing {
            out.push((BAR_CARET.to_string(), Style::default()));
        }
        if app.filter.is_some() {
            out.push((Msg::BarCount.fill(&[&app.rows.len(), &app.unfiltered]), dim));
        }
    } else {
        out.push((Msg::BarEmpty.text().to_string(), dim));
    }
    out
}

/// ビューの帯のボタン(NV-23)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewButton {
    Filter,
    Sort,
    Group,
    Columns,
}

/// ボタンの文字(前後に空白1つ)と、効いているか(色を変える)。キーのあるボタンは頭にキーを添える(NV-23)。
fn button_text(app: &App, b: ViewButton) -> (String, bool) {
    use keymap::Action as A;
    let (action, label, on) = match b {
        ViewButton::Filter => (
            Some(A::FilterMenu),
            super::filters::label(app),
            !app.settings.filters.is_empty(),
        ),
        ViewButton::Sort => (
            Some(A::SortMenu),
            super::sorts::label(app),
            !app.settings.sorts.is_empty(),
        ),
        ViewButton::Group => match &app.settings.group {
            mdgrid::settings::Group::By { col, .. } => (
                None,
                format!("{} {}", Msg::ViewGroup.text(), sanitize(&app.title(col))),
                true,
            ),
            _ => (None, Msg::ViewGroup.text().to_string(), false),
        },
        ViewButton::Columns => (
            None,
            Msg::ViewColumns.text().to_string(),
            !app.hidden.is_empty(),
        ),
    };
    let key = action.and_then(|a| keymap::key_for(&app.keys, Mode::Table, a));
    let t = match &key {
        Some(k) => format!(" {k} {label} "),
        None => format!(" {label} "),
    };
    (t, on)
}

/// ビューの帯のボタンの頭のキー(NV-23。キーの無いボタンと割り当てが無ければ None)。
fn button_key(app: &App, b: ViewButton) -> Option<String> {
    let action = match b {
        ViewButton::Filter => keymap::Action::FilterMenu,
        ViewButton::Sort => keymap::Action::SortMenu,
        ViewButton::Group | ViewButton::Columns => return None,
    };
    keymap::key_for(&app.keys, Mode::Table, action)
}

/// CE-25: ビューの帯の右の端の「+ 新規」の (文字(前後に空白1つ), キー)。キーは新規のキー(SR-4)。
pub(crate) fn new_button_text(app: &App) -> (String, Option<String>) {
    let key = keymap::key_for(&app.keys, Mode::Table, keymap::Action::NewNote);
    let label = super::new_note::button();
    let t = match &key {
        Some(k) => format!(" {k} {label} "),
        None => format!(" {label} "),
    };
    (t, key)
}

/// ビューの帯の置き方(SR-1・NV-23)。
pub(crate) struct ViewBar {
    /// タブ: (桁, 文字, ビューの添字)。
    pub tabs: Vec<(usize, String, usize)>,
    /// 検索の欄: (桁, 幅)。
    pub search: Option<(usize, usize)>,
    /// ボタン: (桁, 文字, ボタン, 効いているか)。
    pub buttons: Vec<(usize, String, ViewButton, bool)>,
    /// CE-25: 右の端の「+ 新規」: (桁, 文字)。作れないか入らなければ None。
    pub new: Option<(usize, String)>,
    /// NV-31: 帯の上に足した行のタブ(上の行から。各行は (桁, 文字, ビューの添字))。
    pub wrapped: Vec<Vec<(usize, String, usize)>>,
}

/// 幅 `w` のビューの帯の置き方。左からタブ、少し空けて検索の欄、その右にボタン(表の近くにまとめる)、
/// 右の端に「+ 新規」(CE-25。作れるとき)。入らなければ 列・分類のボタン、今のでないタブの順に省き、
/// 検索の欄を中身の幅まで詰め、それでも入らなければ 並べ替え・絞り込みのボタン、「+ 新規」の順に省く。
pub(crate) fn view_bar(app: &App, w: usize) -> ViewBar {
    let mut tabs = if tabs_shown(app) {
        tab_spans(app)
    } else {
        Vec::new()
    };
    let inner: usize = search_spans(app).iter().map(|(t, _)| width(t)).sum();
    // 囲むときは左右の線の2桁を欄の幅に含める。
    let frame_w = if boxed(app) { 2 } else { 0 };
    // タブと欄の間の空き(囲むときは線が区切りになるので1桁)。
    let gap = if boxed(app) { 1 } else { TABS_GAP };
    let mut search_w = if search_shown(app) {
        inner.max(SEARCH_MIN) + 2 + frame_w
    } else {
        0
    };
    let mut buttons: Vec<(String, ViewButton, bool)> = [
        ViewButton::Filter,
        ViewButton::Sort,
        ViewButton::Group,
        ViewButton::Columns,
    ]
    .into_iter()
    .map(|b| {
        let (t, on) = button_text(app, b);
        (t, b, on)
    })
    .collect();
    let mut new = super::new_note::button_shown(app).then(|| new_button_text(app).0);
    let tabs_w =
        |tabs: &[(usize, String, usize)]| tabs.last().map(|(x, t, _)| x + width(t)).unwrap_or(0);
    let need = |tabs: &[(usize, String, usize)],
                search_w: usize,
                buttons: &[(String, ViewButton, bool)],
                new: &Option<String>| {
        let tw = tabs_w(tabs);
        let rest = search_w + buttons.iter().map(|(t, _, _)| width(t) + 1).sum::<usize>();
        let nw = new.as_ref().map_or(0, |t| width(t) + 1);
        tw + if tw > 0 && rest > 0 { gap } else { 1 } + rest + nw
    };
    // NV-31: 折り返す設定なら、タブを全部、真ん中の行の下の行へ(左から幅いっぱいに)。
    // NV-32: 切れ目(`tab_break`)があれば、幅と設定に依らず下の行へ並べ、切れ目のタブの前で行を改める。
    let mut wrapped: Vec<Vec<(usize, String, usize)>> = Vec::new();
    let starts = row_starts(app);
    let breaks = !starts.is_empty();
    if breaks || (tab_wrap(app) && tabs.len() > 1 && need(&tabs, search_w, &buttons, &new) > w) {
        let mut row: Vec<(usize, String, usize)> = Vec::new();
        let mut x = 0;
        for (_, t, i) in tabs.drain(..) {
            let tw = width(&t);
            if x > 0 && (x + tw > w || starts.contains(&i)) {
                wrapped.push(std::mem::take(&mut row));
                x = 0;
            }
            row.push((x, t, i));
            x += tw + 1;
        }
        if !row.is_empty() {
            wrapped.push(row);
        }
    }
    // 列・分類を後ろから省く。
    while need(&tabs, search_w, &buttons, &new) > w && buttons.len() > 2 {
        buttons.pop();
    }
    // 今のでないタブを省く(今のタブを左の端に)。折り返したときは省かない(NV-31)。
    if wrapped.is_empty() && need(&tabs, search_w, &buttons, &new) > w && tabs.len() > 1 {
        let cur = app.view_index();
        tabs.retain(|(_, _, i)| *i == cur);
        if let Some(t) = tabs.first_mut() {
            t.0 = 0;
        }
    }
    // 検索の欄を中身の幅まで詰める(絞り込み・並べ替え・「+ 新規」をなるべく残す)。
    let over = need(&tabs, search_w, &buttons, &new).saturating_sub(w);
    if over > 0 && search_w > 0 {
        search_w = search_w.saturating_sub(over).max(inner + 2 + frame_w);
    }
    while need(&tabs, search_w, &buttons, &new) > w && !buttons.is_empty() {
        buttons.pop();
    }
    if need(&tabs, search_w, &buttons, &new) > w {
        new = None;
    }
    let tw = tabs_w(&tabs);
    let mut x = if tw > 0 { tw + gap } else { 1 };
    let search = (search_w > 0).then(|| {
        let at = (x, search_w);
        x += search_w + 1;
        at
    });
    let buttons = buttons
        .into_iter()
        .map(|(t, b, on)| {
            let at = x;
            x += width(&t) + 1;
            (at, t, b, on)
        })
        .collect();
    let new = new.map(|t| (w - width(&t), t));
    ViewBar {
        tabs,
        search,
        buttons,
        new,
        wrapped,
    }
}

/// ビューの帯(SR-1): タブ・検索の欄(NV-23)・ボタン。囲むときは上の線・真ん中の行・下の線の3行。
pub(crate) fn view_bar_lines(app: &App, w: usize) -> Vec<Line<'static>> {
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
    let boxed = boxed(app);
    // NV-23: box では線の中を地の色で塗らない(下線も引かない)。字の色と太字はそのまま。
    let unfill = |st: Style| -> Style {
        if boxed {
            let mut s = st;
            s.bg = None;
            s.remove_modifier(Modifier::UNDERLINED)
        } else {
            st
        }
    };
    let field = unfill(match &look {
        Some(l) => Style::default().bg(l.soft_bg),
        None => Style::default().add_modifier(Modifier::UNDERLINED),
    });
    let button_st = |on: bool| button_style(&look, on);
    let fr = super::popup::frame(app);
    // NV-23: 囲みの線は窓の線と同じ字と見た目(SR-32・SR-33)。
    let border = match &look {
        Some(l) => l.border(),
        None => Style::default(),
    };
    let side = || Span::styled(fr.v.to_string(), border);
    let bar = view_bar(app, w);
    // 囲み: (桁, 幅)。
    let mut boxes: Vec<(usize, usize)> = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut used = 0;
    let goto = |spans: &mut Vec<Span<'static>>, used: &mut usize, x: usize| {
        if x > *used {
            spans.push(Span::raw(" ".repeat(x - *used)));
            *used = x;
        }
    };
    for (x, t, i) in &bar.tabs {
        goto(&mut spans, &mut used, *x);
        let st = if *i == app.view_index() { rev } else { other };
        used += width(t);
        spans.push(Span::styled(t.clone(), st));
    }
    if let Some((x, sw)) = bar.search {
        goto(&mut spans, &mut used, x);
        // 囲むときは左右の線の内側が欄。
        let fw = if boxed {
            boxes.push((x, sw));
            spans.push(side());
            sw - 2
        } else {
            sw
        };
        let mut inner = 0;
        spans.push(Span::styled(" ".to_string(), field));
        for (t, st) in search_spans(app) {
            inner += width(&t);
            spans.push(Span::styled(t, st.patch(field)));
        }
        spans.push(Span::styled(
            " ".repeat(fw.saturating_sub(inner + 1)),
            field,
        ));
        if boxed {
            spans.push(side());
        }
        used += sw;
    }
    for (x, t, b, on) in &bar.buttons {
        goto(&mut spans, &mut used, *x);
        used += width(t);
        let key = button_key(app, *b);
        // NV-23: 頭のキーの字は名前と見分ける見た目。囲むボタンは前後の空白を線にする。
        if boxed && boxed_button(*b) && t.len() >= 2 {
            boxes.push((*x, width(t)));
            spans.push(side());
            spans.extend(key_spans(
                &t[1..t.len() - 1],
                key.as_deref(),
                unfill(button_st(*on)),
                &look,
            ));
            spans.push(side());
        } else {
            spans.extend(key_spans(t, key.as_deref(), button_st(*on), &look));
        }
    }
    // NV-26: タブの切り替えの案内(タブが2つ以上で、入るときだけ。ボタンの右に)。
    let hint = Msg::TabSwitch.text();
    let end = bar.new.as_ref().map_or(w, |(x, _)| x.saturating_sub(1));
    if tabs_shown(app) && app.nv.tabs.hint && app.tab_list().len() > 1 && used + width(hint) <= end
    {
        used += width(hint);
        spans.push(Span::styled(
            hint.to_string(),
            Style::default().add_modifier(Modifier::DIM),
        ));
    }
    // CE-25: 右の端の「+ 新規」。
    if let Some((x, t)) = &bar.new {
        goto(&mut spans, &mut used, *x);
        let key = new_button_text(app).1;
        spans.extend(key_spans(t, key.as_deref(), button_st(false), &look));
    }
    let mid = pad(spans, w, Style::default());
    // NV-31: 真ん中の行の下に並べたタブの行(真ん中の行のタブと同じ見た目)。`tab_wrap_gap` なら前に空き行。
    let mut wrapped: Vec<Line<'static>> = bar
        .wrapped
        .iter()
        .map(|row| {
            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut used = 0;
            for (x, t, i) in row {
                if *x > used {
                    spans.push(Span::raw(" ".repeat(x - used)));
                    used = *x;
                }
                let st = if *i == app.view_index() { rev } else { other };
                used += width(t);
                spans.push(Span::styled(t.clone(), st));
            }
            pad(spans, w, Style::default())
        })
        .collect();
    if !wrapped.is_empty() && wrap_gap(app) {
        wrapped.insert(0, Line::from(" ".repeat(w)));
    }
    // NV-23: 上下の余白の行。box の線は真ん中の行に接する余白の行に描き、残りは空き。
    // SR-1・NV-23: 端数は帯に接する側に半行(セルの地は帯の色、外側の半分を画面の地の色の字で消す。上 `▀`・下 `▄`)。そのときは帯の行に地の色を敷く。
    let (full, half) = bar_pad(app);
    let n = full + usize::from(half);
    if n == 0 {
        return [vec![mid], wrapped].concat();
    }
    let blank = || Line::from(" ".repeat(w));
    if !boxed {
        let bg = half.then(|| half_bg(app)).flatten();
        let on_bg = |l: Line<'static>| match bg {
            Some(c) => lay_bg(l, c),
            None => l,
        };
        let mut out: Vec<Line<'static>> = (0..full).map(|_| blank()).collect();
        if half {
            out.push(half_line(app, '▀', w, bg));
        }
        out.push(on_bg(mid));
        // NV-31: タブの行は真ん中の行のすぐ下、下の余白の上。
        out.extend(wrapped.into_iter().map(on_bg));
        if half {
            out.push(half_line(app, '▄', w, bg));
        }
        out.extend((0..full).map(|_| blank()));
        return out;
    }
    // 上と下の線(囲みの桁だけ。ほかは空白)。
    let edge = |l: char, r: char| -> Line<'static> {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut used = 0;
        for &(x, bw) in &boxes {
            if x > used {
                spans.push(Span::raw(" ".repeat(x - used)));
            }
            spans.push(Span::styled(
                format!("{l}{}{r}", fr.line(bw.saturating_sub(2))),
                border,
            ));
            used = x + bw;
        }
        pad(spans, w, Style::default())
    };
    let mut out: Vec<Line<'static>> = (1..n).map(|_| blank()).collect();
    out.push(edge(fr.tl, fr.tr));
    out.push(mid);
    out.push(edge(fr.bl, fr.br));
    // NV-31: タブの行は囲みの下の線の下。
    out.extend(wrapped);
    out.extend((1..n).map(|_| blank()));
    out
}

/// (x, y) がタブなら、そのビューの添字(BV-13 のクリック)。
pub fn tab_at(app: &App, x: u16, y: u16) -> Option<usize> {
    if tab_rows(app) == 0 || no_views(app) {
        return None;
    }
    let (w, _) = super::popup::screen(app);
    let x = x as usize;
    // NV-31: 真ん中の行の下に並べたタブ。
    let top = wrap_y(app);
    if y >= top && ((y - top) as usize) < tab_lines(app) {
        return view_bar(app, w)
            .wrapped
            .get((y - top) as usize)?
            .iter()
            .find(|(x0, t, _)| x >= *x0 && x < x0 + width(t))
            .map(|(_, _, i)| *i);
    }
    if !bar_row(app, y, false) {
        return None;
    }
    view_bar(app, w)
        .tabs
        .into_iter()
        .find(|(x0, t, _)| x >= *x0 && x < x0 + width(t))
        .map(|(_, _, i)| i)
}

/// NV-23: (x, y) がビューの帯のボタンなら、そのボタン。
pub(crate) fn view_button_at(app: &App, x: u16, y: u16) -> Option<ViewButton> {
    if tab_rows(app) == 0 || !row_drawn(app, bar_mid(app)) {
        return None;
    }
    let (w, _) = super::popup::screen(app);
    let x = x as usize;
    // NV-23: 囲むボタンは囲みの3行のどこでも。
    view_bar(app, w)
        .buttons
        .into_iter()
        .find(|(x0, t, b, _)| bar_row(app, y, boxed_button(*b)) && x >= *x0 && x < x0 + width(t))
        .map(|(_, _, b, _)| b)
}

/// NV-24: (x, y) がビューの帯の「並べ替え」のボタンか(試験から使う)。
#[cfg(test)]
pub(crate) fn sort_button_at(app: &App, x: u16, y: u16) -> bool {
    view_button_at(app, x, y) == Some(ViewButton::Sort)
}

/// NV-24・NV-29: 窓を置く所: ボタンが描かれていれば (ボタンの左の桁, ボタンの行。囲むときは囲みの下の線の行)、
/// 無ければ None(窓は右の端に寄せて、ヘッダーの下に開く)。
pub(crate) fn button_anchor(app: &App, b: ViewButton) -> Option<(usize, usize)> {
    let rows = tab_rows(app);
    if rows == 0 || !row_drawn(app, bar_mid(app)) {
        return None;
    }
    // 囲むボタンは囲みの下の線の行、線なしなら帯の一番下の行(窓は帯のすぐ下)。
    let y = if boxed_button(b) && boxed(app) {
        bar_mid(app) as usize + 1
    } else if boxed_button(b) {
        bar_y(app) as usize + rows - 1
    } else {
        bar_mid(app) as usize
    };
    let (w, _) = super::popup::screen(app);
    view_bar(app, w)
        .buttons
        .into_iter()
        .find(|(_, _, bb, _)| *bb == b)
        .map(|(x, _, _, _)| (x, y))
}

/// CE-25: (x, y) がビューの帯の「+ 新規」か。関係マップでも同じ桁(2行目の右の端)。
pub(crate) fn new_button_at(app: &App, x: u16, y: u16) -> bool {
    if !super::new_note::button_shown(app) {
        return false;
    }
    let (w, _) = super::popup::screen(app);
    let x = x as usize;
    let at = if on_relmap(app) {
        // 関係マップでは帯が無いので、表の画面の帯の真ん中の行と同じ行(窓の上の縁)。
        if y != bar_mid(app) {
            return false;
        }
        relmap_new_button(app, w)
    } else {
        if !bar_row(app, y, false) || tab_rows(app) == 0 || !row_drawn(app, bar_mid(app)) {
            return false;
        }
        view_bar(app, w).new
    };
    at.is_some_and(|(x0, t)| x >= x0 && x < x0 + width(&t))
}

/// CE-25: 関係マップの「+ 新規」: (桁, 文字)。表の画面の帯の真ん中の行と同じ行・同じ桁。入らなければ None。
fn relmap_new_button(app: &App, w: usize) -> Option<(usize, String)> {
    // ビューの帯を出さないときは、表の画面と同じくヘッダーに出す。
    if header_new_shown(app) {
        return None;
    }
    let (t, _) = new_button_text(app);
    let tw = width(&t);
    (super::new_note::button_shown(app) && w >= tw + 2).then(|| (w - tw, t))
}

/// CE-25: 関係マップの窓の上の縁(表の画面の帯の真ん中の行と同じ行)の右の端に「+ 新規」を重ねる。
pub(crate) fn relmap_new_overlay(app: &App, line: &Line<'static>, w: usize) -> Line<'static> {
    let Some((x, t)) = relmap_new_button(app, w) else {
        return line.clone();
    };
    let look = super::look::look(app);
    let key = new_button_text(app).1;
    let mut out = line.clone();
    let mut at = x;
    for sp in key_spans(&t, key.as_deref(), button_style(&look, false), &look) {
        let sw = width(&sp.content);
        out = super::list::splice(&out, at, sp, sw, w);
        at += sw;
    }
    out
}

/// (x, y) が検索の欄か(NV-23 のクリックで欄に入る)。欄が描かれていなければ false。
pub fn bar_at(app: &App, x: u16, y: u16) -> bool {
    if !bar_drawn(app) || !bar_row(app, y, true) {
        return false;
    }
    let (w, _) = super::popup::screen(app);
    let x = x as usize;
    view_bar(app, w)
        .search
        .is_some_and(|(x0, sw)| x >= x0 && x < x0 + sw)
}

/// 設定の帯(NV-16)の行の数: 効いている設定が無いか、帯を隠せば 0(帯を出さない。SR-20)、あれば 1。
pub(crate) fn chip_rows(app: &App) -> usize {
    usize::from(app.shows(Item::Chips) && !app.settings.no_conditions())
}

/// 設定の帯の画面の行(検索の欄があればその下)。
fn chips_y(app: &App) -> u16 {
    bar_y(app) + tab_rows(app) as u16
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
    // NV-30: 問題の数(1つ以上のとき。クリックで問題のある行だけに絞る)。
    if app.built.problems > 0 {
        text.push_str(&Msg::FooterProblems.fill(&[&app.built.problems]));
    }
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
            | Mode::Drawer
            | Mode::FilterWin
            | Mode::FilterText
    ) || (app.mode == Mode::ScreenMenu && app.relmap.is_none() && app.folder_tree.is_none())
    {
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

/// NV-30: (x, y) が下の帯の問題の数か(押すと問題のある行だけに絞る・外す)。
pub(crate) fn problems_at(app: &App, x: u16, y: u16) -> bool {
    // 描く高さは端末の最下行を空けた高さ(ui::draw)で、下の帯はその下から2行目。
    if app.built.problems == 0 || y as usize + 3 != app.size.1 as usize {
        return false;
    }
    let x0 = width(&Msg::FooterUnsaved.fill(&[&app.changes.count()]));
    let t = Msg::FooterProblems.fill(&[&app.built.problems]);
    let lead = t.len() - t.trim_start().len();
    let x = x as usize;
    x >= x0 + lead && x < x0 + width(&t)
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
        let (sx, _) = view_bar(app, w).search?;
        let lead = width(&search_lead(app)) + width(": ");
        let frame = usize::from(boxed(app));
        (sx + frame + 1 + lead + width(&bar_text(app)), bar_mid(app))
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
