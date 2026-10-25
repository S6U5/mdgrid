//! スターターから台帳を作る画面の流れ(ST-5): パレット → 一覧 → 作る場所 → 見本を入れるか → 作って開き直す。
//! 見本は test_relations.rs の notes/(tasks を開く)。値は作ったもの。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_relations::workspace;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::Path;

/// tasks のフォルダを開く(読むだけかを選べる)。
fn boot(tmp: &Tmp, readonly: bool) -> App {
    let cfg = tmp.0.join("config");
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

/// パスの部品を `/` でつなぐ(区切りは OS に依らず `/`。名前の中の `\` は残る)。
fn slash(p: &Path) -> String {
    p.iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// `root` の下のファイル(根からの相対。揃える。区切りは OS に依らず `/`)。
fn files_under(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(slash(p.strip_prefix(root).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn test_st_5_palette_lists_starters_then_folder_then_samples_then_opens() {
    // [ST-5] パレットの「スターターから台帳を作る」→ 一覧に組み込みの4つ(名前と説明)→ reading を選ぶ →
    // 「作る場所」の欄に reading が入っている → Enter → 「見本のノートも入れる?」→ y → 開いたフォルダの下の
    // reading/ に印・雛形・見本ができ、その台帳の open の表へ開き直す。
    let tmp = workspace("st_flow");
    let mut a = boot(&tmp, false);
    palette(&mut a, "スターターから台帳を作る");
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    let s = screen(&a);
    for name in ["tasks", "bookmarks", "reading", "people"] {
        assert!(s.contains(name), "一覧に {name}:\n{s}");
    }
    assert!(s.contains("スターター:"), "{s}");
    assert!(s.contains("タスク"), "今の言語の名前:\n{s}");
    typing(&mut a, "reading");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    let s = screen(&a);
    assert!(
        s.contains("作る場所: reading"),
        "既定はスターターの名前:\n{s}"
    );
    assert!(s.contains("基準のフォルダ:"), "基準のフォルダ:\n{s}");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("見本のノートも入れる?"), "{s}");
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    let root = tmp.notes().join("tasks/reading");
    let files = files_under(&root);
    assert!(
        files.iter().any(|f| f == ".mdgrid/workspace.toml"),
        "印: {files:?}"
    );
    assert!(
        files
            .iter()
            .any(|f| f.starts_with(".mdgrid/templates/") && f.ends_with(".md")),
        "雛形: {files:?}"
    );
    assert!(
        files
            .iter()
            .any(|f| f.starts_with("Reading/") && f.ends_with(".md")),
        "見本: {files:?}"
    );
    let to = a.switch_to.as_ref().expect("開き直す表");
    let real = std::fs::canonicalize(&root).unwrap();
    assert!(
        to.path.starts_with(&real),
        "open の表は作った台帳の中: {}",
        to.path.display()
    );
    assert!(a.switch_leave_workspace);
    assert!(
        a.message.as_deref().unwrap_or("").contains("reading"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_st_5_n_makes_the_ledger_without_samples() {
    // [ST-5] 見本を入れるかに n → 印と雛形だけで、見本のノートは書かない。
    let tmp = workspace("st_no_samples");
    let mut a = boot(&tmp, false);
    palette(&mut a, "スターターから台帳を作る");
    typing(&mut a, "tasks");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    let files = files_under(&tmp.notes().join("tasks/tasks"));
    assert!(
        files.iter().any(|f| f == ".mdgrid/workspace.toml"),
        "{files:?}"
    );
    assert!(
        files.iter().all(|f| f.starts_with(".mdgrid/")),
        "見本は書かない: {files:?}"
    );
    assert!(a.switch_to.is_some());
}

#[test]
fn test_st_5_esc_at_any_step_cancels_and_writes_nothing() {
    // [ST-5] Esc はどの段でも取りやめで、何も作らない。見本を入れるかに y/n 以外も取りやめ。
    let tmp = workspace("st_esc");
    let mut a = boot(&tmp, false);
    let before = files_under(&tmp.notes());
    // 一覧で Esc。
    palette(&mut a, "スターターから台帳を作る");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    // 作る場所で Esc。
    palette(&mut a, "スターターから台帳を作る");
    typing(&mut a, "reading");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    // 見本を入れるかで Esc、そして y/n 以外。
    palette(&mut a, "スターターから台帳を作る");
    typing(&mut a, "reading");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("見本のノートも入れる?"));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    palette(&mut a, "スターターから台帳を作る");
    typing(&mut a, "reading");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "x");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("やめた"),
        "{:?}",
        a.message
    );
    assert_eq!(files_under(&tmp.notes()), before, "何も作らない");
    assert!(a.switch_to.is_none());
}

#[test]
fn test_st_5_empty_folder_is_refused_and_absolute_path_is_used_as_is() {
    // [ST-5] 作る場所が空なら理由を出して欄に留まる。絶対のパスはそのまま作る先になる。
    let tmp = workspace("st_dir");
    let mut a = boot(&tmp, false);
    palette(&mut a, "スターターから台帳を作る");
    typing(&mut a, "bookmarks");
    press(&mut a, KeyCode::Enter);
    // 既定の名前を消して空で Enter。
    for _ in 0.."bookmarks".len() {
        press(&mut a, KeyCode::Backspace);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette, "欄に留まる");
    assert!(
        a.message.as_deref().unwrap_or("").contains("作る場所"),
        "{:?}",
        a.message
    );
    let abs = tmp.0.join("elsewhere/marks");
    typing(&mut a, &abs.to_string_lossy());
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert!(
        abs.join(".mdgrid/workspace.toml").is_file(),
        "絶対のパスに作る"
    );
    assert!(!tmp.notes().join("tasks/bookmarks").exists());
}

#[test]
fn test_st_5_readonly_has_no_command_and_refuses() {
    // [ST-5] 読むだけの起動では、パレットに「スターターから台帳を作る」が無く、動作を呼んでも理由だけ。
    let tmp = workspace("st_readonly");
    let mut a = boot(&tmp, true);
    ch(&mut a, ':');
    typing(&mut a, "スターター");
    assert!(
        !screen(&a).contains("スターターから台帳を作る"),
        "{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Esc);
    a.start_starter_new();
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.is_some(), "理由を出す");
    assert!(!tmp.notes().join("tasks/tasks").exists());
}
