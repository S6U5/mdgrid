//! チェックを付け外しする一覧の窓のキー(screen.checklist-keys)と、一覧の窓の下の縁のキーの案内
//! (screen.window-key-hints)の受け入れ試験。リストの値の付け外しの窓と、絞り込みの窓の値の一覧で確かめる。
//! 窓の中の状態には頼らず、キー・クリック・画面の文字・`mode`・ためる変更・`settings.filters` で確かめる。

use super::super::keymap::{Action, Mode};
use super::super::test_screen::{col_named, make, press, text, typing};
use super::super::{width, App};
use mdgrid::settings::{Cond, Op};
use mdgrid::source::NewValue;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// tags の候補は 仕事・開発・家(どれも1件)。一 は 仕事 が付いている。
const TAGS: &[(&str, &str)] = &[
    ("n1.md", "---\ntitle: 一\ntags: [仕事]\n---\n"),
    ("n2.md", "---\ntitle: 二\ntags: [開発]\n---\n"),
    ("n3.md", "---\ntitle: 三\ntags: [家]\n---\n"),
];

/// status は done が2つ・todo が1つ。
const STATUS: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\n---\n"),
    ("b.md", "---\nstatus: done\n---\n"),
    ("c.md", "---\nstatus: todo\n---\n"),
];

/// App の大きさで描いた画面の文字。
fn scr(a: &App) -> String {
    let mut term = Terminal::new(TestBackend::new(a.size.0, a.size.1)).unwrap();
    term.draw(|f| super::super::draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// 行の名前(ファイル名)の行を選ぶ。
fn row_of(a: &mut App, name: &str) {
    a.row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == name)
        .unwrap_or_else(|| panic!("行が無い: {name}"));
}

/// 一 の tags のセルで付け外しの窓を開く。
fn open_tags(a: &mut App) {
    row_of(a, "n1.md");
    col_named(a, "tags");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
}

/// 窓の中(縦の線の右)で、頭に「>」の付いた(選んでいる)行が `target` を含むか。表の選んだ行の頭の「>」は見ない。
fn selected_has(a: &App, target: &str) -> bool {
    scr(a).lines().any(|l| {
        let Some(at) = l.find('│') else {
            return false;
        };
        let l = &l[at..];
        l.match_indices('>')
            .any(|(i, _)| l[i + 1..].find(target).is_some_and(|p| p <= 8))
    })
}

/// 窓の中の行を、先頭に戻ってから ↓ で下りて `target` の行を選ぶ。
fn select(a: &mut App, target: &str) {
    for _ in 0..20 {
        press(a, KeyCode::Up);
    }
    for _ in 0..20 {
        if selected_has(a, target) {
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("窓に「{target}」の行が無い:\n{}", scr(a));
}

fn pending_tags(a: &App) -> Option<NewValue> {
    let r = a
        .rows
        .iter()
        .find(|r| a.src.label(r) == "n1.md")
        .unwrap()
        .clone();
    a.changes.pending(&r, "tags").cloned()
}

fn list(items: &[&str]) -> Option<NewValue> {
    Some(NewValue::List(
        items.iter().map(std::string::ToString::to_string).collect(),
    ))
}

/// 付け外しの窓を開き、開発 を Enter で付けた所。
fn checked_dev(name: &str) -> (super::super::test_screen::Tmp, App) {
    let (t, mut a) = make(name, TAGS);
    open_tags(&mut a);
    select(&mut a, "開発");
    press(&mut a, KeyCode::Enter);
    (t, a)
}

/// 画面の中で `needle` の左の端の桁と行(幅2の字は2桁で数える)。
fn spot(a: &App, needle: &str) -> Option<(u16, u16)> {
    scr(a).lines().enumerate().find_map(|(y, l)| {
        l.find(needle)
            .map(|at| (width::width(&l[..at]) as u16, y as u16))
    })
}

/// 窓の下の縁(左下の角と右下の角の間)に `needle` があるか。
fn on_edge(l: &str, needle: &str) -> bool {
    match (l.find('╰'), l.find(needle), l.rfind('╯')) {
        (Some(a), Some(n), Some(b)) => a < n && n < b,
        _ => false,
    }
}

/// 案内 `needle` を含む窓の下の縁の行(下の帯の同じ文字ではなく)。
fn edge_with(a: &App, needle: &str) -> String {
    let s = scr(a);
    s.lines()
        .find(|l| on_edge(l, needle))
        .unwrap_or_else(|| panic!("下の縁に「{needle}」が無い:\n{s}"))
        .to_string()
}

/// 窓の下の縁の案内 `needle` の左の端の桁と行。
fn edge_spot(a: &App, needle: &str) -> (u16, u16) {
    let s = scr(a);
    s.lines()
        .enumerate()
        .find_map(|(y, l)| {
            on_edge(l, needle).then(|| {
                let at = l.find(needle).unwrap();
                (width::width(&l[..at]) as u16, y as u16)
            })
        })
        .unwrap_or_else(|| panic!("下の縁に「{needle}」が無い:\n{s}"))
}

// ---- screen.checklist-keys ----

#[test]
fn test_checklist_enter_checks_and_keeps_window() {
    // [utest -> req~screen.checklist-keys~1] tags の候補に 仕事・開発・家、仕事 が付いたセル → 付け外しの窓で 開発 を選んで Enter
    // → 開発 にチェックが付き、窓は開いたまま(ためる変更はまだ無い)。
    let (_t, a) = checked_dev("ckenter");
    assert_eq!(a.mode, Mode::ListPick, "窓は開いたまま:\n{}", scr(&a));
    let s = scr(&a);
    assert!(s.contains("[x] 開発"), "開発 にチェック:\n{s}");
    assert!(s.contains("[x] 仕事"), "仕事 はそのまま:\n{s}");
    assert!(s.contains("[ ] 家"), "{s}");
    assert_eq!(a.changes.count(), 0, "まだ決めていない");
}

#[test]
fn test_checklist_tab_decides() {
    // [utest -> req~screen.checklist-keys~1] 開発 にチェックを付けた → Tab → 窓が閉じ、tags は「仕事・開発」のためる変更。
    let (_t, mut a) = checked_dev("cktab");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert_eq!(pending_tags(&a), list(&["仕事", "開発"]));
}

#[test]
fn test_checklist_decide_row_enter_and_click() {
    // [utest -> req~screen.checklist-keys~1] 開発 にチェックを付けた → 窓の一番下の「[ 決める ]」の行で Enter → Tab と同じく決める。
    let (_t, mut a) = checked_dev("ckdecide");
    let s = scr(&a);
    let rows: Vec<&str> = s.lines().filter(|l| l.contains('│')).collect();
    let last = rows.last().copied().unwrap_or_default();
    assert!(
        last.contains("[ 決める ]"),
        "「[ 決める ]」は窓の一番下の行:\n{s}"
    );
    select(&mut a, "[ 決める ]");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert_eq!(pending_tags(&a), list(&["仕事", "開発"]));
    // クリックでも決める。
    let (_t2, mut b) = checked_dev("ckdecideclick");
    let (x, y) = spot(&b, "[ 決める ]").expect("「[ 決める ]」の行");
    b.click(x + 2, y);
    assert_eq!(b.mode, Mode::Table, "{}", scr(&b));
    assert_eq!(pending_tags(&b), list(&["仕事", "開発"]));
}

#[test]
fn test_checklist_esc_cancels() {
    // [utest -> req~screen.checklist-keys~1] 開発 にチェックを付けた → Esc → 窓が閉じ、何もためない。
    let (_t, mut a) = checked_dev("ckesc");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_checklist_space_toggles_but_types_while_narrowing() {
    // [utest -> req~screen.checklist-keys~1] Space でも付け外しする。打った字で絞る欄に打っているあいだの Space は空白のまま。
    let (_t, mut a) = make("ckspace", TAGS);
    open_tags(&mut a);
    select(&mut a, "家");
    press(&mut a, KeyCode::Char(' '));
    assert!(scr(&a).contains("[x] 家"), "{}", scr(&a));
    press(&mut a, KeyCode::Char(' '));
    assert!(scr(&a).contains("[ ] 家"), "{}", scr(&a));
    typing(&mut a, "開");
    press(&mut a, KeyCode::Char(' '));
    assert_eq!(a.pick.as_ref().map(|p| p.query.as_str()), Some("開 "));
    assert_eq!(a.mode, Mode::ListPick);
}

/// 絞り込みの窓を開き、status の値の一覧まで進む。
fn status_values(a: &mut App) {
    a.apply(Action::FilterMenu);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(a));
    select(a, "+ 絞り込みを足す");
    press(a, KeyCode::Enter);
    select(a, "status");
    press(a, KeyCode::Enter);
    select(a, "値の一覧");
    press(a, KeyCode::Enter);
    assert!(scr(a).contains("done"), "値の一覧:\n{}", scr(a));
}

#[test]
fn test_checklist_filter_values_enter_then_tab() {
    // [utest -> req~screen.checklist-keys~1] 絞り込みの窓の status の値の一覧 → 「done」で Enter、続けて Tab → done のチェックが入り、
    // 「status が done を除く」(一覧の先頭の切り替えのとおり)の条件ができる。
    let (_t, mut a) = make("ckfilter", STATUS);
    status_values(&mut a);
    let hide = scr(&a).contains("チェックした値の行を隠す");
    select(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert!(a.settings.filters.is_empty(), "Enter では決めない");
    assert!(
        scr(&a)
            .lines()
            .any(|l| l.contains("done") && l.contains("[x]")),
        "done にチェック:\n{}",
        scr(&a)
    );
    press(&mut a, KeyCode::Tab);
    let keys = vec![Some("done".to_string())];
    let op = if hide { Op::Drop(keys) } else { Op::Keep(keys) };
    assert_eq!(
        a.settings.filters,
        [Cond {
            col: "status".to_string(),
            op,
        }]
    );
    assert_eq!(a.mode, Mode::FilterWin, "条件の並びに戻る");
}

#[test]
fn test_checklist_filter_values_decide_row() {
    // [utest -> req~screen.checklist-keys~1] 絞り込みの窓の値の一覧も、一番下の「[ 決める ]」の行で Enter かクリックで決める。
    let (_t, mut a) = make("ckfilterdecide", STATUS);
    status_values(&mut a);
    select(&mut a, "todo");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "[ 決める ]");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.settings.filters.len(), 1, "{}", scr(&a));
    assert_eq!(a.settings.filters[0].col, "status");
    let (_t2, mut b) = make("ckfilterclick", STATUS);
    status_values(&mut b);
    select(&mut b, "todo");
    press(&mut b, KeyCode::Char(' '));
    let (x, y) = spot(&b, "[ 決める ]").expect("「[ 決める ]」の行");
    b.click(x + 2, y);
    assert_eq!(b.settings.filters.len(), 1, "{}", scr(&b));
}

// ---- screen.window-key-hints ----

#[test]
fn test_window_hints_on_toggle_window_bottom_edge() {
    // [utest -> req~screen.window-key-hints~1] リストの値の付け外しの窓を描く → 下の縁に「Enter 入れる・外す」「Tab 決める」「Esc やめる」が並ぶ。
    let (_t, mut a) = make("hintpick", TAGS);
    open_tags(&mut a);
    let line = edge_with(&a, "Enter 入れる・外す");
    let e = line.find("Enter 入れる・外す").unwrap();
    let t = line.find("Tab 決める").expect("Tab 決める");
    let x = line.find("Esc やめる").expect("Esc やめる");
    assert!(e < t && t < x, "この順に並ぶ: {line:?}");
    let l = line.rfind('╰').expect("下の縁の左下の角");
    let r = line.rfind('╯').expect("下の縁の右下の角");
    assert!(l < e && x < r, "縁の上に並ぶ: {line:?}");
}

#[test]
fn test_window_hints_click_tab_decides() {
    // [utest -> req~screen.window-key-hints~1] 同じ窓で、下の縁の「Tab 決める」をクリック → Tab と同じく決める。
    let (_t, mut a) = checked_dev("hintclick");
    let (x, y) = edge_spot(&a, "Tab 決める");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert_eq!(pending_tags(&a), list(&["仕事", "開発"]));
    // 「Esc やめる」のクリックは Esc と同じ(何もためない)。
    let (_t2, mut b) = checked_dev("hintclickesc");
    let (x, y) = edge_spot(&b, "Esc やめる");
    b.click(x + 1, y);
    assert_eq!(b.mode, Mode::Table, "{}", scr(&b));
    assert_eq!(b.changes.count(), 0);
}

#[test]
fn test_window_hints_narrow_drop_from_the_end() {
    // [utest -> req~screen.window-key-hints~1] 幅の狭い窓を描く → 入らないキーは後ろから省き、縁の線と字が重ならない。
    let (_t, mut a) = make("hintnarrow", TAGS);
    a.resize(36, 24);
    open_tags(&mut a);
    let s = scr(&a);
    let line = edge_with(&a, "Enter 入れる・外す");
    assert!(!line.contains("Esc"), "後ろから省く: {line:?}\n{s}");
    assert!(!line.contains("Tab"), "後ろから省く: {line:?}\n{s}");
    let l = line.rfind('╰').expect("左下の角");
    let r = line.rfind('╯').expect("右下の角");
    let edge = &line[l..r + '╯'.len_utf8()];
    assert!(edge.starts_with("╰─ "), "左の角のあとは線と空白: {edge:?}");
    assert!(
        edge.ends_with(" ─╯") || edge.ends_with("──╯"),
        "右の角の前は線: {edge:?}"
    );
    // もっと狭くて1つも入らなければ、線だけ。
    let (_t2, mut b) = make("hintnarrower", TAGS);
    b.resize(22, 24);
    open_tags(&mut b);
    let s = scr(&b);
    let bottom = s
        .lines()
        .find(|l| l.contains('╰') && l.contains('╯'))
        .unwrap_or_else(|| panic!("下の縁:\n{s}"));
    let l = bottom.rfind('╰').unwrap();
    let r = bottom.rfind('╯').unwrap();
    assert!(
        bottom[l + '╰'.len_utf8()..r].chars().all(|c| c == '─'),
        "線だけ: {bottom:?}"
    );
}

#[test]
fn test_window_hints_filter_values_and_list() {
    // [utest -> req~screen.window-key-hints~1] 絞り込みの窓の下の縁にも、その段で効くキー。値の一覧は「Enter 入れる・外す」「Tab 決める」「Esc やめる」。
    let (_t, mut a) = make("hintfilter", STATUS);
    a.apply(Action::FilterMenu);
    let line = edge_with(&a, "Esc 閉じる");
    assert!(line.contains("Enter "), "{line:?}");
    status_values(&mut a);
    let line = edge_with(&a, "Enter 入れる・外す");
    assert!(
        line.contains("Tab 決める") && line.contains("Esc やめる"),
        "{line:?}"
    );
    // 縁の「Tab 決める」のクリックは Tab と同じ。
    select(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    let (x, y) = edge_spot(&a, "Tab 決める");
    a.click(x + 1, y);
    assert_eq!(a.settings.filters.len(), 1, "{}", scr(&a));
}

#[test]
fn test_window_hints_sort_window() {
    // [utest -> req~screen.window-key-hints~1] 並べ替えの窓の下の縁に Enter と Esc の案内。縁の「Esc 閉じる」のクリックで閉じる。
    let (_t, mut a) = make("hintsort", STATUS);
    a.apply(Action::SortMenu);
    assert_eq!(a.mode, Mode::Sorts);
    let line = edge_with(&a, "Esc 閉じる");
    assert!(line.contains("Enter "), "{line:?}");
    let (x, y) = edge_spot(&a, "Esc 閉じる");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
}

#[test]
fn test_window_hints_action_menu() {
    // [utest -> req~screen.window-key-hints~1] 操作の一覧の下の縁に「Enter 実行」「Esc 閉じる」。縁の「Esc 閉じる」のクリックで閉じる。
    let (_t, mut a) = make("hintmenu", STATUS);
    a.apply(Action::Menu);
    assert_eq!(a.mode, Mode::Menu);
    let line = edge_with(&a, "Esc 閉じる");
    assert!(line.contains("Enter 実行"), "{line:?}");
    let (x, y) = edge_spot(&a, "Esc 閉じる");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
}

#[test]
fn test_window_hints_screen_menu() {
    // [utest -> req~screen.window-key-hints~1] 画面の型のメニューの下の縁に Enter と Esc の案内。
    let (_t, mut a) = make("hintscreen", STATUS);
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu);
    let line = edge_with(&a, "Esc 閉じる");
    assert!(line.contains("Enter "), "{line:?}");
    let (x, y) = edge_spot(&a, "Esc 閉じる");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
}

#[test]
fn test_window_hints_value_candidates() {
    // [utest -> req~screen.window-key-hints~1] 値の候補の窓の下の縁に「Enter 確定」「Esc 取り消し」。縁の「Esc 取り消し」のクリックは Esc と同じ。
    let (_t, mut a) = make("hintcands", STATUS);
    row_of(&mut a, "c.md");
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.active_list().is_some(), "候補の窓:\n{}", scr(&a));
    let line = edge_with(&a, "Esc 取り消し");
    assert!(line.contains("Enter 確定"), "{line:?}");
    press(&mut a, KeyCode::Up);
    let (x, y) = edge_spot(&a, "Esc 取り消し");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert_eq!(a.changes.count(), 0);
}
