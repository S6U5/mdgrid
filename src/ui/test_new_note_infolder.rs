//! [CE-25] `.base` のビューで作るノートは、絞り込みの `file.inFolder("X")` のフォルダに作り、
//! 作った行がビューに残る(specs/_changes/2026-10-06-new-note-infolder.md)。

use super::startup::Startup;
use super::test_screen::{ch, ctrl, screen, typing, Tmp};
use super::*;
use std::path::PathBuf;

const BASE: &str = "filters:\n  and:\n    - file.inFolder(\"Tasks\")\nviews:\n  - type: table\n    name: Open\n    filters:\n      and:\n        - done == false\n    order: [status, done]\n";

/// `<一時>/notes` を保管庫の根(`.obsidian` あり)にし、Tasks/ に1つ、根に Tasks.base を置いて開く。
fn boot(name: &str, config: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    let root = tmp.notes();
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::create_dir_all(root.join("Tasks")).unwrap();
    std::fs::write(
        root.join("Tasks/a.md"),
        "---\nstatus: todo\ndone: false\n---\n",
    )
    .unwrap();
    let base = root.join("Tasks.base");
    std::fs::write(&base, BASE).unwrap();
    let (config, warnings) = mdgrid::config::parse(config).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let paths: Vec<PathBuf> = vec![base];
    let t = crate::open_target(&paths, None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: crate::state_target(&paths),
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

fn create(a: &mut App, name: &str) {
    ch(a, 'a');
    typing(a, name);
    // CE-26: 窓のどの欄からでも Ctrl+S で作る。
    ctrl(a, 's');
}

#[test]
fn test_ce_25_infolder_creates_in_view_folder() {
    let (tmp, mut a) = boot("ce25_infolder", "");
    create(&mut a, "new");
    let made = tmp.notes().join("Tasks/new.md");
    assert!(made.is_file(), "Tasks/ に作る:\n{}", screen(&a));
    assert!(!tmp.notes().join("new.md").exists(), "根には作らない");
    let text = std::fs::read_to_string(&made).unwrap();
    assert!(
        text.contains("done: false"),
        "ビューの条件の値も入る: {text:?}"
    );
    // 作った行がビューに残り、選ばれる。
    let row = &a.rows[a.row];
    assert!(
        a.src.label(row).contains("new"),
        "作った行が選ばれる:\n{}",
        screen(&a)
    );
}

#[test]
fn test_ce_25_infolder_keeps_configured_subfolder() {
    let (tmp, mut a) = boot(
        "ce25_infolder_sub",
        "[new_note]\nfolder = \"Tasks/inbox\"\n",
    );
    create(&mut a, "sub");
    assert!(
        tmp.notes().join("Tasks/inbox/sub.md").is_file(),
        "設定の Tasks/inbox はビューのフォルダの中なのでそのまま:\n{}",
        screen(&a)
    );
}

#[test]
fn test_ce_25_infolder_overrides_folder_outside_view() {
    let (tmp, mut a) = boot("ce25_infolder_out", "[new_note]\nfolder = \"inbox\"\n");
    create(&mut a, "out");
    assert!(
        tmp.notes().join("Tasks/out.md").is_file(),
        "設定の inbox はビューの外なので、ビューのフォルダに作る:\n{}",
        screen(&a)
    );
    assert!(!tmp.notes().join("inbox/out.md").exists());
}
