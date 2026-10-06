//! レビューの直し(specs/_changes/2026-10-06-review6-fixes.md)。本物の実行ファイルで確かめる。
//! [BV-23] 保管庫の中の隠しフォルダを直接渡すと、その中を探す。[CLI-17] 同じ列に当たる2つの見出し・
//! JSON の空の文字のリスト・大きすぎる整数。[CLI-16] `:` を含む列の名前の --sort。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-r6-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("v/.obsidian")).unwrap();
    root
}

fn write(root: &Path, p: &str, t: &str) {
    let f = root.join(p);
    std::fs::create_dir_all(f.parent().unwrap()).unwrap();
    std::fs::write(f, t).unwrap();
}

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().unwrap();
    if let Some(s) = stdin {
        child.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn rows(o: &Output) -> usize {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).lines().skip(1).count()
}

#[test]
fn test_bv_23_hidden_folder_inside_vault() {
    let root = dir("hidden");
    write(&root, "v/a.md", "---\nk: a\n---\n");
    write(&root, "v/.hidden/b.md", "---\nk: b\n---\n");
    write(&root, "v/.hidden/sub/c.md", "---\nk: c\n---\n");
    assert_eq!(rows(&run(&root, &["v/.hidden", "--print"], None)), 2);
    assert_eq!(rows(&run(&root, &["v/.hidden/sub", "--print"], None)), 1);
    // 根を渡したときは今までどおり隠しフォルダを飛ばす。
    assert_eq!(rows(&run(&root, &["v", "--print"], None)), 1);
}

#[test]
fn test_cli_17_two_headers_same_column() {
    let root = dir("twohead");
    write(&root, "v/a.md", "---\nstatus: a\n---\n");
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,status,note.status\nv/a.md,b,c\n"),
    );
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(root.join("v/a.md")).unwrap(),
        "---\nstatus: a\n---\n"
    );
}

#[test]
fn test_cli_17_json_empty_list_and_big_int() {
    let root = dir("json");
    write(&root, "v/a.md", "---\ntags: [x]\nn: 1\n---\n");
    // 空の文字はリストの列を空にする(`[""]` にしない)。
    let o = run(
        &root,
        &["v", "--apply", "-"],
        Some("[{\"path\": \"v/a.md\", \"tags\": \"\"}]"),
    );
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!s.contains("[\"\"]") && s.contains("+tags:"), "{s}");
    // i64 に入らない整数は丸めずに理由。
    let o = run(
        &root,
        &["v", "--apply", "-"],
        Some("[{\"path\": \"v/a.md\", \"n\": 18446744073709551615}]"),
    );
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn test_cli_16_sort_column_with_colon() {
    let root = dir("colon");
    write(&root, "v/a.md", "---\n\"time:start\": 2\n---\n");
    write(&root, "v/b.md", "---\n\"time:start\": 1\n---\n");
    let o = run(
        &root,
        &["v", "--print", "--with-path", "--sort", "time:start"],
        None,
    );
    let s = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let first = s.lines().nth(1).unwrap_or_default();
    assert!(first.contains("b.md"), "{s}");
    let o = run(&root, &["v", "--print", "--sort", "time:start:desc"], None);
    assert!(o.status.success());
}
