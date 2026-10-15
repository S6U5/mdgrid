//! ノートの欄のフォルダの出し方 `display.note_path`(SR-29・SR-46)の画面の受け入れ試験。
//! `"common"`(既定。共通のフォルダを除く)・`"name"`(名前だけ)・`"full"`(根からのパス)・`"off"`(提案中の4つ目。
//! 欄に名前も見出しも出さず、行の印だけ)。記録 specs/_changes/2026-10-15-note-path.md(タスク #1)。
//! 実装を見ずに、設定の TOML・画面の文字・キーで確かめる(行と桁の位置は決め打ちしない)。
//! 状態と mdgrid のビューの置き場は一時フォルダ(`<一時>/state`・`<一時>/config`)。
//!
//! 仮定(実装役に渡す): ビューの設定の画面の「表示」の区画の項目の名前は「ノートのパス」、値は `common`・`name`・
//! `full`・`off` をそのまま出す。Space で common → name → full → off → common の順に切り替える。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, edit_cell, edit_outside, press, text, typing, Tmp};
use super::test_settings_screen::{apply_btn, open};
use super::*;
use mdgrid::config::{parse, Config};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

// ---- 材料 ----

/// todo と archive/todo のノート(共通のフォルダは無い。既定では `todo/alpha`・`archive/todo/beta`)。
fn ledger(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join("todo")).unwrap();
    std::fs::create_dir_all(tmp.notes().join("archive/todo")).unwrap();
    tmp.write("todo/alpha.md", "---\nstatus: open\n---\n");
    tmp.write("archive/todo/beta.md", "---\nstatus: closed\n---\n");
    tmp
}

/// 全部が Tasks の下のノート(既定では共通のフォルダを除いて `alpha`・`beta`)。
fn tasks(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join("Tasks")).unwrap();
    tmp.write("Tasks/alpha.md", "---\nstatus: open\n---\n");
    tmp.write("Tasks/beta.md", "---\nstatus: closed\n---\n");
    tmp
}

/// ビューが2つの `.base`(どちらも status の列だけ。file.name の列は無い)。
const BASE: &str = r#"views:
  - type: table
    name: 一
    order: [status]
  - type: table
    name: 二
    order: [status]
"#;

/// 設定の TOML を読む(警告が無いことも確かめる)。
fn cfg_ok(text: &str) -> Config {
    let (c, w) = parse(text).expect("設定の TOML は読める");
    assert!(w.is_empty(), "設定の警告: {w:?}\n---\n{text}");
    c
}

/// フォルダを main と同じ道筋で開く(mdgrid のビューの置き場あり)。
fn boot_folder(tmp: &Tmp, config: Config) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// `.base` を main と同じ道筋で開く。
fn boot_base(tmp: &Tmp, config: Config) -> App {
    let p = tmp.notes().join("tasks.base");
    std::fs::write(&p, BASE).unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 30);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

// ---- 画面の読み取り ----

/// 今の大きさ(`App::resize`)で描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// `word` を含む表の行(最初の1つ)。
fn row_line(a: &App, word: &str) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.contains(word))
        .unwrap_or_else(|| panic!("「{word}」の行が無い:\n{s}"))
        .to_string()
}

/// 欄が名前だけ: alpha と beta の行にフォルダが付かない。
fn assert_names_only(a: &App) {
    let s = screen(a);
    let alpha = row_line(a, "alpha");
    let beta = row_line(a, "beta");
    assert!(!alpha.contains("/alpha"), "名前だけ: {alpha:?}\n{s}");
    assert!(!beta.contains("/beta"), "名前だけ: {beta:?}\n{s}");
    assert!(
        !alpha.contains(".md") && !beta.contains(".md"),
        "`.md` は除く:\n{s}"
    );
}

/// タブの行(ビューの帯の真ん中の行)。
fn tab_line(a: &App) -> String {
    let y = super::bands::mid_row(a);
    screen(a).lines().nth(y).unwrap_or("").to_string()
}

/// `]` で名前のタブまで動かす(選んだタブは `[名前]`)。
fn goto_tab(a: &mut App, name: &str) {
    let want = format!("[{name}]");
    for _ in 0..8 {
        if tab_line(a).contains(&want) {
            return;
        }
        ch(a, ']');
    }
    panic!("タブ「{name}」に移れない:\n{}", screen(a));
}

fn settle(a: &mut App) {
    for _ in 0..4 {
        if a.mode == Mode::Table {
            return;
        }
        press(a, KeyCode::Esc);
    }
}

/// 画面の文字 `label` をクリックする(ボタン)。
fn click_label(a: &mut App, label: &str) {
    let s = screen(a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(label))
        .unwrap_or_else(|| panic!("画面に「{label}」が無い:\n{s}"));
    let x = width::width(&line[..line.find(label).unwrap()]);
    a.click(x as u16 + 1, y as u16);
}

// ---- ビューの設定の画面の「表示」の区画 ----

const ITEM: &str = "ノートのパス";

/// 「表示」の区画に移る(NV-18)。
fn display_section(a: &mut App) {
    for _ in 0..12 {
        if a.draft.as_ref().map(|d| d.sec) == Some(super::settings::Sec::Display) {
            return;
        }
        press(a, KeyCode::Tab);
    }
    panic!("「表示」の区画に移れない:\n{}", screen(a));
}

/// 「ノートのパス」の項目の行。
fn item_line(a: &App) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.contains(ITEM))
        .unwrap_or_else(|| panic!("「{ITEM}」の項目が無い:\n{s}"))
        .to_string()
}

/// 「表示」の区画で「ノートのパス」を選ぶ(選んだ項目は `>名前`)。
fn select_item(a: &mut App) {
    display_section(a);
    for _ in 0..=mdgrid::display::ITEMS.len() + 1 {
        if screen(a).contains(&format!(">{ITEM}")) {
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("「{ITEM}」を選べない:\n{}", screen(a));
}

/// 選んだ「ノートのパス」を `want` になるまで Space で切り替える(4回まで)。
fn cycle_to(a: &mut App, want: &str) {
    for _ in 0..4 {
        if item_line(a).contains(want) {
            return;
        }
        ch(a, ' ');
    }
    panic!("「{ITEM}」を {want} にできない: {}", item_line(a));
}

// ---- SR-29: 値ごとのノートの欄 ----

#[test]
fn test_sr_29_note_path_name_shows_only_names() {
    // [SR-29][SR-46] note_path = "name" で todo/alpha.md・archive/todo/beta.md → 欄は alpha・beta。見出しは残る。
    let tmp = ledger("sr29np_name");
    let a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"name\"\n"));
    assert_names_only(&a);
    let s = screen(&a);
    assert!(!s.contains("todo/"), "フォルダは出さない:\n{s}");
    assert!(s.contains("ノート"), "欄の見出しは残る:\n{s}");
}

#[test]
fn test_sr_29_note_path_full_keeps_common_folder() {
    // [SR-29][SR-46] note_path = "full" で Tasks/alpha.md・Tasks/beta.md(全部が同じフォルダ)→ 欄は
    // Tasks/alpha・Tasks/beta(共通のフォルダも除かない)。`.md` は除く。
    let tmp = tasks("sr29np_full");
    let a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"full\"\n"));
    let s = screen(&a);
    assert!(row_line(&a, "alpha").contains("Tasks/alpha"), "{s}");
    assert!(row_line(&a, "beta").contains("Tasks/beta"), "{s}");
    assert!(!s.contains(".md"), "{s}");
    // 共通のフォルダの無い表でも根からのパス(既定と同じ見え方)。
    let tmp = ledger("sr29np_full2");
    let a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"full\"\n"));
    let s = screen(&a);
    assert!(row_line(&a, "alpha").contains("todo/alpha"), "{s}");
    assert!(row_line(&a, "beta").contains("archive/todo/beta"), "{s}");
}

#[test]
fn test_sr_29_note_path_name_search_matches_folder() {
    // [SR-29][NV-2] "name" でも、`\` の検索で archive と打つと archive/todo/beta.md の行が当たる(検索は根からの
    // パスのまま)。当たった行の欄は名前だけ。
    let tmp = ledger("sr29np_search");
    let mut a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"name\"\n"));
    ch(&mut a, '\\');
    typing(&mut a, "archive");
    press(&mut a, KeyCode::Enter);
    let labels: Vec<String> = a.rows.iter().map(|r| a.src.label(r)).collect();
    assert_eq!(labels, vec!["archive/todo/beta.md".to_string()]);
    let beta = row_line(&a, "beta");
    assert!(
        !beta.contains("/beta"),
        "欄は名前だけ: {beta:?}\n{}",
        screen(&a)
    );
}

#[test]
fn test_sr_29_note_path_off_hides_names_and_heading() {
    // [SR-29][SR-46] note_path = "off" → 欄に名前も見出し「ノート」も出ない(file.name の列のときと同じ)。
    // 値の列は残る。
    let tmp = ledger("sr29np_off");
    let a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"off\"\n"));
    let s = screen(&a);
    assert!(
        !s.contains("alpha") && !s.contains("beta"),
        "名前を出さない:\n{s}"
    );
    assert!(!s.contains("ノート"), "見出しも出さない:\n{s}");
    assert!(
        s.contains("open") && s.contains("closed"),
        "値の列は残る:\n{s}"
    );
}

#[test]
fn test_sr_29_note_path_off_keeps_marks() {
    // [SR-29][WB-16] "off" でも、直してためた行が外で変わったときの `!` は欄に残る。
    let tmp = ledger("sr29np_offmark");
    let mut a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"off\"\n"));
    let i = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "todo/alpha.md")
        .unwrap();
    edit_cell(&mut a, i, "status", "ready");
    edit_outside(&tmp, "todo/alpha.md", "---\nstatus: changed\n---\n");
    a.poll();
    let s = screen(&a);
    let row = row_line(&a, "ready");
    assert!(row.contains('!'), "{s}");
    assert!(!s.contains("alpha"), "名前は出さない:\n{s}");
}

// ---- SR-46: 設定・ビューの設定の画面・保存 ----

#[test]
fn test_sr_46_config_name_applies_to_every_table() {
    // [SR-46] config.toml の `[display] note_path = "name"` → フォルダの表も `.base` のどのビューも名前だけ。
    let tmp = ledger("sr46every");
    let a = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"name\"\n"));
    assert_names_only(&a);
    let mut b = boot_base(&tmp, cfg_ok("[display]\nnote_path = \"name\"\n"));
    goto_tab(&mut b, "一");
    assert_names_only(&b);
    goto_tab(&mut b, "二");
    assert_names_only(&b);
}

#[test]
fn test_sr_46_unknown_value_keeps_default_look() {
    // [SR-46][CLI-3] note_path = "short" → `config.toml: display.note_path:` で始まる警告と既定の見た目
    // (共通のフォルダの無い表なので todo/alpha・archive/todo/beta)。
    let tmp = ledger("sr46short");
    let (c, w) = parse("[display]\nnote_path = \"short\"\n").unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].starts_with("config.toml: display.note_path: "),
        "{w:?}"
    );
    let a = boot_folder(&tmp, c);
    let s = screen(&a);
    assert!(row_line(&a, "alpha").contains("todo/alpha"), "{s}");
    assert!(row_line(&a, "beta").contains("archive/todo/beta"), "{s}");
}

#[test]
fn test_sr_46_settings_cycles_four_values() {
    // [SR-46][NV-18] ビューの設定の「表示」の区画で「ノートのパス」を common → name → full → off → common と
    // 順に切り替える。既定は common。
    let tmp = ledger("sr46cycle");
    let mut a = boot_folder(&tmp, cfg_ok(""));
    a.resize(120, 50);
    open(&mut a);
    select_item(&mut a);
    assert!(item_line(&a).contains("common"), "既定: {}", item_line(&a));
    for want in ["name", "full", "off", "common"] {
        ch(&mut a, ' ');
        assert!(item_line(&a).contains(want), "{want}: {}", item_line(&a));
    }
    // 設定で name にしていれば、画面の項目も name から始まる。
    let tmp = ledger("sr46cycle2");
    let mut b = boot_folder(&tmp, cfg_ok("[display]\nnote_path = \"name\"\n"));
    b.resize(120, 50);
    open(&mut b);
    select_item(&mut b);
    assert!(item_line(&b).contains("name"), "{}", item_line(&b));
}

#[test]
fn test_sr_46_settings_full_applies_to_that_view_only() {
    // [SR-46][NV-13] `.base` のビュー「一」の設定の画面で「ノートのパス」を full にして反映 → 「一」だけ
    // Tasks/alpha・Tasks/beta。「二」は既定(共通のフォルダを除いて alpha・beta)のまま。
    let tmp = tasks("sr46view");
    let mut a = boot_base(&tmp, cfg_ok(""));
    a.resize(120, 50);
    goto_tab(&mut a, "一");
    open(&mut a);
    select_item(&mut a);
    cycle_to(&mut a, "full");
    apply_btn(&mut a);
    assert!(
        row_line(&a, "alpha").contains("Tasks/alpha"),
        "{}",
        screen(&a)
    );
    assert!(
        row_line(&a, "beta").contains("Tasks/beta"),
        "{}",
        screen(&a)
    );
    goto_tab(&mut a, "二");
    assert!(!screen(&a).contains("Tasks/"), "{}", screen(&a));
    assert_names_only(&a);
    goto_tab(&mut a, "一");
    assert!(
        row_line(&a, "alpha").contains("Tasks/alpha"),
        "{}",
        screen(&a)
    );
}

#[test]
fn test_sr_46_saved_native_view_keeps_note_path() {
    // [SR-46][BV-17] 「ノートのパス」を full にして、mdgrid のビュー「フル」として保存 → そのビューだけ根からの
    // パス。終了して開き直しても同じで、設定の画面の項目も full。
    let tmp = tasks("sr46save");
    let mut a = boot_folder(&tmp, cfg_ok(""));
    a.resize(120, 50);
    open(&mut a);
    select_item(&mut a);
    cycle_to(&mut a, "full");
    click_label(&mut a, "名前を付けて保存");
    for _ in 0..40 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "フル");
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    goto_tab(&mut a, "フル");
    assert!(
        row_line(&a, "alpha").contains("Tasks/alpha"),
        "{}",
        screen(&a)
    );
    goto_tab(&mut a, "既定の表");
    assert!(!screen(&a).contains("Tasks/"), "{}", screen(&a));
    settle(&mut a);
    ch(&mut a, 'q');
    assert!(a.quit, "終了できる: {}", screen(&a));

    let mut b = boot_folder(&tmp, cfg_ok(""));
    b.resize(120, 50);
    goto_tab(&mut b, "フル");
    assert!(
        row_line(&b, "alpha").contains("Tasks/alpha"),
        "{}",
        screen(&b)
    );
    assert!(
        row_line(&b, "beta").contains("Tasks/beta"),
        "{}",
        screen(&b)
    );
    goto_tab(&mut b, "既定の表");
    assert!(!screen(&b).contains("Tasks/"), "{}", screen(&b));
    goto_tab(&mut b, "フル");
    open(&mut b);
    select_item(&mut b);
    assert!(item_line(&b).contains("full"), "{}", item_line(&b));
}
