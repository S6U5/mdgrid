#![cfg(unix)]
//! スターターから台帳を作る(ST-5)と、作った台帳を開いて閉じてもバイトが変わらないこと(ST-6)の、本物の実行ファイル
//! での確かめ(端末 80×24)。値は作ったもの。

mod pty;

use pty::{TempDir, Tui};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// `root` の下の全部のファイル(根からの相対)とその中身。
fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
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
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                map.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    map
}

/// 組み込みのスターター `name` から、`h` の下の `name` に見本つきで作る(画面なし。今日は 2026-10-06)。
fn make(h: &Path, name: &str) {
    let root = h.join(name);
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args([
            root.to_str().unwrap(),
            "--init-workspace",
            "--starter",
            name,
            "--with-samples",
        ])
        .current_dir(h)
        .env("HOME", h)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", h.join("config"))
        .env("XDG_STATE_HOME", h.join("state"))
        .env("MDGRID_TODAY", "2026-10-06")
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// ノートのフォルダ notes/ とその下の tasks/(1件)。
fn home(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("notes/tasks/a.md", "---\nstatus: todo\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

#[test]
fn test_st_5_palette_creates_a_ledger_from_a_starter_and_opens_it_e2e() {
    // [ST-5] `:` → 「スターターから台帳を作る」→ reading → 作る場所(既定 reading)→ y → 読書の表が開き、
    // タブに「読みたい」がある。q で終えると、作った台帳のファイルがある。
    let home = home("starter-e2e");
    let h = home.path();
    let mut tui = Tui::spawn(&["notes/tasks"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| {
        s.contains("a.md") || s.contains("status")
    });
    tui.send(":");
    tui.wait_until("パレット", |s| s.contains(": "));
    tui.send("スターターから台帳を作る");
    tui.wait_until("候補", |s| s.contains("スターターから台帳を作る"));
    tui.send("\r");
    tui.wait_until("一覧", |s| {
        s.contains("スターター: ") && s.contains("reading")
    });
    tui.send("reading");
    tui.send("\r");
    tui.wait_until("作る場所", |s| s.contains("作る場所: reading"));
    tui.send("\r");
    tui.wait_until("見本", |s| s.contains("見本のノートも入れる?"));
    tui.send("y");
    tui.send("\r");
    tui.wait_until("読書の表", |s| {
        s.contains("読みたい") && s.contains("reading")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
    let root = h.join("notes/tasks/reading");
    assert!(root.join(".mdgrid/workspace.toml").is_file());
    assert!(root.join(".mdgrid/templates/entry.md").is_file());
    assert!(root.join("Reading").is_dir(), "見本のフォルダ");
}

#[test]
fn test_st_5_tilde_folder_is_under_the_home_folder_e2e() {
    // [ST-5] 作る場所の欄の既定(スターターの名前)を消して `~/books` と打つ → HOME の下の books に作り、
    // 開いたフォルダの下に `~` という名前のフォルダはできない。n で見本は入れない。
    let home = home("starter-e2e-tilde");
    let h = home.path();
    let mut tui = Tui::spawn(&["notes/tasks"], h, &[("MDGRID_TODAY", "2026-10-06")]);
    tui.wait_until("表が出る", |s| s.contains("status"));
    tui.send(":");
    tui.wait_until("パレット", |s| s.contains(": "));
    tui.send("スターターから台帳を作る");
    tui.wait_until("候補", |s| s.contains("スターターから台帳を作る"));
    tui.send("\r");
    tui.wait_until("一覧", |s| {
        s.contains("スターター:") && s.contains("tasks")
    });
    tui.send("tasks");
    tui.send("\r");
    tui.wait_until("作る場所", |s| s.contains("作る場所: tasks"));
    for _ in 0.."tasks".len() {
        tui.send("\x7f");
    }
    tui.wait_until("欄が空", |s| {
        s.contains("作る場所:") && !s.contains("作る場所: tasks")
    });
    tui.send("~/books");
    tui.wait_until("打った場所", |s| s.contains("作る場所: ~/books"));
    tui.send("\r");
    tui.wait_until("見本", |s| s.contains("見本のノートも入れる?"));
    tui.send("n");
    tui.send("\r");
    tui.wait_until("タスクの表", |s| {
        s.contains("未完了") && s.contains("books")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
    assert!(
        h.join("books/.mdgrid/workspace.toml").is_file(),
        "HOME の下に作る"
    );
    assert!(!h.join("books/Tasks").exists(), "n なので見本は無い");
    assert!(
        !h.join("notes/tasks/~").exists(),
        "`~` という名前のフォルダは作らない"
    );
}

#[test]
fn test_st_6_opening_a_made_ledger_changes_no_bytes_e2e() {
    // [ST-6] 組み込みの4つから見本つきで作った台帳を、開いて(表が描かれるまで待って)閉じても、印・雛形・見本の
    // バイトが1つも変わらない。
    let home = home("starter-e2e-bytes");
    let h = home.path();
    for name in ["tasks", "bookmarks", "reading", "people"] {
        make(h, name);
        let root = h.join(name);
        let before = files(&root);
        assert!(before.contains_key(".mdgrid/workspace.toml"), "{name}");
        let mut tui = Tui::spawn(&[name], h, &[("MDGRID_TODAY", "2026-10-06")]);
        tui.wait_until("表が出る", |s| s.contains("▦ 表 ▾"));
        tui.send("q");
        assert_eq!(tui.wait_exit(), 0, "{name}");
        assert_eq!(
            files(&root),
            before,
            "{name}: 開いて閉じたあとに変わったファイルがある"
        );
    }
}
