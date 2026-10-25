//! [SR-48] 計算の列の見出しの頭に ƒ を付けて、ノートのキーの列と見分ける(画面だけ。--print は変えない)。
#![cfg(unix)]

mod pty;

use pty::{TempDir, Tui};
use std::process::Command;

fn sample(root: &std::path::Path) {
    for d in ["todo", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(
        root.join("todo/a.md"),
        "---\ntitle: 一つめ\nactor: ai\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".mdgrid/workspace.toml"),
        "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\n\n[table.base]\nfilters = { or = ['file.inFolder(\"todo\")'] }\n\n[table.base.formulas]\n\"担当\" = 'if(actor == \"ai\", \"AI\", \"人\")'\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"title\", \"actor\", \"formula.担当\"]\n",
    )
    .unwrap();
}

#[test]
fn test_sr_48_formula_header_has_mark() {
    // [SR-48] 画面の見出し: 計算の列は「ƒ 担当」、ノートのキーの列(title・actor)には ƒ が付かない。
    let t = TempDir::new("fmark");
    let root = t.path();
    sample(root);
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("表", |s| s.contains("一つめ"));
    let s = tui.screen();
    let head = s
        .lines()
        .find(|l| l.contains("担当") && l.contains("actor"))
        .unwrap_or_default()
        .to_string();
    assert!(head.contains("ƒ 担当"), "{s}");
    assert_eq!(head.matches('ƒ').count(), 1, "ƒ は計算の列だけ: {head}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_sr_48_formula_mark_without_color() {
    // [SR-48] 色を使わない表示(--no-color)でも ƒ が付く。
    let t = TempDir::new("fmark-nocolor");
    let root = t.path();
    sample(root);
    let mut tui = Tui::spawn(&["--no-color"], root, &[]);
    tui.wait_until("表", |s| s.contains("一つめ"));
    let s = tui.screen();
    assert!(s.contains("ƒ 担当"), "{s}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_sr_48_print_header_unchanged() {
    // [SR-48] --print の見出しは変えない(担当のまま)。
    let t = TempDir::new("fmark-print");
    let root = t.path();
    sample(root);
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg("--print")
        .current_dir(root)
        .env("HOME", root)
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_STATE_HOME", root.join("state"))
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.lines().next().unwrap_or("").contains("担当"), "{out}");
    assert!(!out.contains('ƒ'), "{out}");
}
