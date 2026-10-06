//! 5回目の点検の C-8・C-9・C-10(specs/_changes/2026-10-06-cli-polish.md)。本物の実行ファイルで確かめる。
//! [CLI-16] --sort の知らない列は理由1行と終了コード 2。[CLI-17] --apply が読まない列の直しを知らせ、文言を正す。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-cp-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("v/.obsidian")).unwrap();
    std::fs::write(
        root.join("v/a.md"),
        "---\nstatus: todo\ndue: 2026-10-01\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("v/b.md"), "---\nstatus: doing\n---\n").unwrap();
    root
}

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
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

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn test_cli_16_sort_unknown_column() {
    let root = vault("sort");
    let o = run(&root, &["v", "--print", "--sort", "nosuchcol"], None);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(err(&o).contains("nosuchcol"), "{}", err(&o));
    // 知っている列は通る(ノートのキー・file.*)。
    for col in ["due", "note.status", "file.name", "file.mtime:desc"] {
        let o = run(&root, &["v", "--print", "--sort", col], None);
        assert!(o.status.success(), "{col}: {}", err(&o));
    }
    // .base の式の列。
    std::fs::write(
        root.join("v/t.base"),
        "formulas:\n  x: 'status'\nviews:\n  - type: table\n    name: v\n",
    )
    .unwrap();
    let o = run(&root, &["v/t.base", "--print", "--sort", "formula.x"], None);
    assert!(o.status.success(), "{}", err(&o));
    let o = run(&root, &["v/t.base", "--print", "--sort", "formula.y"], None);
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn test_cli_17_ignored_columns_noted() {
    let root = vault("ign");
    let csv = "path,file.name,status\nv/a.md,renamed.md,done\nv/b.md,b.md,doing\n";
    let o = run(&root, &["v", "--apply", "-"], Some(csv));
    assert!(o.status.success(), "{}", err(&o));
    let e = err(&o);
    assert!(e.contains("file.name"), "読まない列の直しを知らせる: {e}");
    // 直していない読まない列は知らせない。
    let csv = "path,file.name,status\nv/a.md,a.md,done\n";
    let o = run(&root, &["v", "--apply", "-"], Some(csv));
    assert!(!err(&o).contains("file.name"), "{}", err(&o));
}

#[test]
fn test_cli_17_wording() {
    let root = vault("word");
    let o = run(
        &root,
        &["v", "--apply", "-"],
        Some("path,status\nv/a.md,done\n"),
    );
    let e = err(&o);
    assert!(e.contains("1 file would change"), "{e}");
    let o = run(
        &root,
        &["v", "--apply", "-"],
        Some("path,status\nv/a.md,x\nv/a.md,y\nv/a.md,z\n"),
    );
    let e = err(&o);
    let four = e.lines().find(|l| l.contains("line 4")).unwrap_or_default();
    assert!(four.contains("line 2"), "最初に当てた行: {e}");
}
