//! [BV-23] 隠しフォルダ(`.` で始まる)と node_modules の下のノートは探さない。渡したフォルダそのものは探す。
//! specs/_changes/2026-10-06-skip-hidden-dirs.md。本物の実行ファイルの --print で確かめる。

use std::path::{Path, PathBuf};
use std::process::Command;

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-sh-{name}-{}-{nanos}", std::process::id()));
    for (p, t) in [
        ("a.md", "---\nk: a\n---\n"),
        (".hidden/b.md", "---\nk: b\n---\n"),
        ("node_modules/p/README.md", "---\nk: p\n---\n"),
        ("sub/c.md", "---\nk: c\n---\n"),
        ("sub/.cache/d.md", "---\nk: d\n---\n"),
    ] {
        let f = root.join(p);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, t).unwrap();
    }
    root
}

fn print(root: &Path, target: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(target)
        .args(["--print", "--with-path"])
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_bv_23_skips_hidden_and_node_modules() {
    let root = dir("skip");
    let out = print(&root, &root);
    let rows: Vec<&str> = out.lines().skip(1).collect();
    assert_eq!(rows.len(), 2, "{out}");
    assert!(out.contains("a.md") && out.contains("sub/c.md"), "{out}");
    assert!(
        !out.contains("README") && !out.contains("b.md") && !out.contains("d.md"),
        "{out}"
    );
}

#[test]
fn test_bv_23_given_hidden_folder_is_searched() {
    let root = dir("given");
    let out = print(&root, &root.join(".hidden"));
    assert_eq!(out.lines().skip(1).count(), 1, "{out}");
    assert!(out.contains("b.md"), "{out}");
}
