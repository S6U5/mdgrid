//! 行が1つも無い表の、画面の型の設定の列の例外(GA-1・TR-2・KB-1 の「ただし、行が1つも無い表…」)の試験。
//! 記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。決定: specs/_history/decidespec/_decisions/2026-11-08-empty-table-screen-columns.md。
//! 値は作ったもの。
//!
//! 設定の列が表に無い名前(`start = "none"`・`path = "none"`・`column = "none"`)でも、行が1つも無い表では
//! 列はノートのキーから作るので、列があるかを決めない。読み終えても警告せず、画面の型のメニューに出し、
//! 開くキー(`C`・`T`・`B`)では開かずに「出す行が無い」の理由を出す。
//! 今ある口(`mode`・`message`・`rows`・`open_screen_menu`・`close_screen_menu`)と描いた画面の文字で確かめる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

/// 印の [[table]] に `line` を書き、`notes` のフォルダを表として開いて読み終える(色なし・幅 120)。
fn boot_notes(tag: &str, notes: &[(&str, &str)], line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// ノートの無いフォルダを表として開く。
fn boot_empty(tag: &str, line: &str) -> (Tmp, App) {
    boot_notes(tag, &[], line)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の型のメニューを開いて描いた文字(閉じて戻す)。
fn menu_text(a: &mut App) -> String {
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu, "{:?}", a.message);
    let s = screen(a);
    a.close_screen_menu();
    s
}

/// メニューの窓に、名前 `name` と切り替えのキー `key` の項目がある(行の右の端の枠を除くと、キーで終わり、
/// その手前が名前で終わる)。
fn has_item(menu: &str, name: &str, key: &str) -> bool {
    menu.lines().skip(1).any(|l| {
        let t = l.trim_end().trim_end_matches(['│', '┃', '|']).trim_end();
        t.strip_suffix(key)
            .is_some_and(|rest| rest.trim_end().ends_with(name) && rest.ends_with(' '))
    })
}

/// 行の無い表で、警告が無い・メニューに出る・開くキーで開かずに理由が出る、を確かめる。
fn check(tag: &str, line: &str, at: &str, name: &str, key: char, reason: &str) {
    let (_t, mut a) = boot_empty(tag, line);
    assert!(a.rows.is_empty(), "前提: 0件");
    assert!(a.loaded(), "前提: 読み終えた");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        !m.contains(at) && !m.contains("none"),
        "{line}: 行の無い表では列が無いと警告しない: {m:?}"
    );
    let menu = menu_text(&mut a);
    assert!(
        has_item(&menu, name, &key.to_string()),
        "{line}: 画面の型のメニューに「{name}」(キー {key})が出る:\n{menu}"
    );
    assert_eq!(a.mode, Mode::Table, "前提: メニューを閉じて表");
    a.message = None;
    ch(&mut a, key);
    assert_eq!(
        a.mode,
        Mode::Table,
        "{line}: {key} で開かない: {:?}",
        a.message
    );
    let m = a
        .message
        .clone()
        .unwrap_or_else(|| panic!("{line}: {key} で理由が出ない\n{}", screen(&a)));
    assert!(!m.contains('\n'), "{line}: 理由は1行: {m:?}");
    assert!(
        m.contains(reason),
        "{line}: 理由は「{reason}」(列が無いでなく、出す行が無い): {m:?}"
    );
}

#[test]
fn test_ga_1_empty_table_missing_start_column_no_warning_menu_and_reason() {
    // [utest -> req~gantt.config-empty-table~1] 行が1つも無い表で gantt = { start = "none", end = "due" } → 読み終えても警告しない。
    // 画面の型のメニューに「ガント」(キー C)が出て、C では開かずに理由「ガントに出す行が無い」。
    check(
        "emptyga",
        "gantt = { start = \"none\", end = \"due\" }",
        "table[1].gantt",
        "ガント",
        'C',
        "ガントに出す行が無い",
    );
}

#[test]
fn test_tr_2_empty_table_missing_path_column_no_warning_menu_and_reason() {
    // [utest -> req~tree.config-empty-table~1] 行が1つも無い表で tree = { path = "none" } → 読み終えても警告しない。
    // 画面の型のメニューに「木」(キー T)が出て、T では開かずに理由「木に出す行が無い」。
    check(
        "emptytr",
        "tree = { path = \"none\" }",
        "table[1].tree",
        "木",
        'T',
        "木に出す行が無い",
    );
}

#[test]
fn test_kb_1_empty_table_missing_status_column_no_warning_menu_and_reason() {
    // [utest -> req~kanban.config-empty-table~2] 行が1つも無い表で kanban = { column = "none" } → 読み終えても警告しない。
    // 画面の型のメニューに「カンバン」(キー B)が出て、B では開かずに理由「カンバンに出す行が無い」。
    check(
        "emptykb",
        "kanban = { column = \"none\" }",
        "table[1].kanban",
        "カンバン",
        'B',
        "カンバンに出す行が無い",
    );
}

#[test]
fn test_ga_1_empty_table_with_rows_elsewhere_still_warns() {
    // [utest -> req~gantt.config-empty-table~1] 境界: 例外は行が1つも無い表だけ。行のある表で start = "none" → 読み終えて警告1行
    // (CLI-3 の形。場所 table[1].gantt と列の名前 none)で、メニューに「ガント」は出ない。
    let (_t, mut a) = boot_notes(
        "emptyga-rows",
        &[("a.md", "---\ndue: 2026-10-03\n---\n")],
        "gantt = { start = \"none\", end = \"due\" }",
    );
    assert_eq!(a.rows.len(), 1, "前提: 1件");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].gantt") && m.contains("none"),
        "行のある表では警告する: {m:?}"
    );
    let menu = menu_text(&mut a);
    assert!(!has_item(&menu, "ガント", "C"), "メニューに出ない:\n{menu}");
}
