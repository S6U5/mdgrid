//! [BV-6] リストの型の列で1つの値で書かれたセルは、式では1つの要素のリスト(Obsidian と同じ)。
//! specs/_changes/2026-10-06-scalar-list-expr.md。本物の実行ファイルの --print で確かめる。

use std::path::{Path, PathBuf};
use std::process::Command;

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-sl-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::write(
        root.join(".obsidian/types.json"),
        r#"{"types":{"tags":"tags"}}"#,
    )
    .unwrap();
    std::fs::write(root.join("a.md"), "---\ntags: project\n---\n").unwrap();
    std::fs::write(root.join("b.md"), "---\ntags: [project, home]\n---\n").unwrap();
    std::fs::write(root.join("c.md"), "---\ntags: other\n---\n").unwrap();
    root
}

fn print(root: &Path, filter: &str, formula: &str) -> String {
    let base = format!(
        "formulas:\n  n: {formula}\nviews:\n  - type: table\n    name: v\n    filters:\n      and:\n        - {filter}\n    order: [file.name, formula.n]\n"
    );
    std::fs::write(root.join("t.base"), base).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(root.join("t.base"))
        .arg("--print")
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn test_bv_6_scalar_list_contains_is_element_match() {
    let root = vault("contains");
    // 文字の一部("proj")では一致しない。要素の "project" では a と b。
    let part = print(&root, "tags.contains(\"proj\")", "tags.length");
    assert_eq!(part.lines().count(), 1, "見出しだけ:\n{part}");
    let whole = print(&root, "tags.contains(\"project\")", "tags.length");
    let rows: Vec<&str> = whole.lines().skip(1).collect();
    assert_eq!(rows.len(), 2, "{whole}");
    assert!(
        rows.iter().any(|l| l.starts_with("a.md,1")),
        "a は1つの要素: {whole}"
    );
    assert!(rows.iter().any(|l| l.starts_with("b.md,2")), "{whole}");
}
