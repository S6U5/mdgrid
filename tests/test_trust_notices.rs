//! 在る値と探した範囲を黙らせない(specs/_changes/2026-10-06-trust-notices.md)。
//! [CV-2] 64ビットに入らない整数は --print でも元の文字。[BV-2] `.obsidian/` の無い場所の `.base` が
//! 0行のときは、探した根を stderr に出す。本物の実行ファイルの --print で確かめる。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-tn-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn print(root: &Path, target: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(target)
        .arg("--print")
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .output()
        .unwrap()
}

#[test]
fn test_cv_2_big_int_prints_original_text() {
    let root = dir("bigint");
    std::fs::write(
        root.join("a.md"),
        "---\nid: 123456789012345678901234\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("b.md"), "---\nid: 7\n---\n").unwrap();
    let out = print(&root, &root);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("123456789012345678901234"), "{text}");
    assert!(!text.contains("null"), "{text}");
}

const BASE: &str =
    "filters:\n  and:\n    - file.inFolder(\"Projects\")\nviews:\n  - type: table\n    name: v\n";

#[test]
fn test_bv_2_vault_root_named_when_empty_without_obsidian() {
    // Bases/Tasks.base と Projects/a.md。.obsidian が無いので根は Bases になり、0行。
    let root = dir("noroot");
    std::fs::create_dir_all(root.join("Bases")).unwrap();
    std::fs::create_dir_all(root.join("Projects")).unwrap();
    std::fs::write(root.join("Projects/a.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::write(root.join("Bases/Tasks.base"), BASE).unwrap();
    let out = print(&root, &root.join("Bases/Tasks.base"));
    assert!(out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains(".obsidian"), "{err}");
    assert!(err.contains("Bases"), "探した根の場所: {err}");
}

#[test]
fn test_bv_2_vault_root_silent_with_obsidian_or_rows() {
    // .obsidian があれば根は上になり行が出る。知らせは出さない。
    let root = dir("withroot");
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::create_dir_all(root.join("Bases")).unwrap();
    std::fs::create_dir_all(root.join("Projects")).unwrap();
    std::fs::write(root.join("Projects/a.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::write(root.join("Bases/Tasks.base"), BASE).unwrap();
    let out = print(&root, &root.join("Bases/Tasks.base"));
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 2);
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(!err.contains(".obsidian"), "{err}");
    // .obsidian が無くても、行があれば黙る(フォルダの中の .base はふつう)。
    let plain = dir("rows");
    std::fs::write(plain.join("a.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::write(
        plain.join("t.base"),
        "views:\n  - type: table\n    name: v\n",
    )
    .unwrap();
    let out = print(&plain, &plain.join("t.base"));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(!err.contains(".obsidian"), "{err}");
}
