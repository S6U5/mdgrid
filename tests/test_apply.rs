//! [CLI-17] `--apply <ファイル>`: `--print --with-path` の形の CSV・JSON を読み、今と違うセルだけを変える。
//! 既定は差分だけ(書かない)、`--yes` で書く。理由が1つでもあればどれも書かず終了コード 2。
//! specs/_changes/2026-10-06-apply.md。本物の実行ファイルで確かめる。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const A: &str = "---\nstatus: todo\npriority: 1\ntags: [a, b]\n---\nbody a\n";
const B: &str = "---\nstatus: doing\npriority: 2\n---\nbody b\n";

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mdgrid-apply-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(root.join("notes/.obsidian")).unwrap();
    std::fs::write(root.join("notes/a.md"), A).unwrap();
    std::fs::write(root.join("notes/b.md"), B).unwrap();
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

fn read(root: &Path, n: &str) -> String {
    std::fs::read_to_string(root.join("notes").join(n)).unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// `--print --with-path --format <f>` の出力。
fn printed(root: &Path, format: &str) -> String {
    let o = run(
        root,
        &["notes", "--print", "--with-path", "--format", format],
        None,
    );
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

#[test]
fn test_cli_17_json_dry_run_shows_diff_and_writes_nothing() {
    let root = vault("dry");
    let json = printed(&root, "json").replacen("\"todo\"", "\"done\"", 1);
    std::fs::write(root.join("edit.json"), &json).unwrap();
    let o = run(&root, &["notes", "--apply", "edit.json"], None);
    assert!(o.status.success(), "{}", err(&o));
    let s = out(&o);
    assert!(
        s.contains("-status: todo") && s.contains("+status: done"),
        "{s}"
    );
    assert!(!s.contains("doing"), "変えないノートは出さない: {s}");
    assert_eq!(read(&root, "a.md"), A);
    assert!(err(&o).contains("--yes"), "{}", err(&o));
}

#[test]
fn test_cli_17_yes_writes_only_changed_cells() {
    let root = vault("yes");
    let json = printed(&root, "json").replacen("\"todo\"", "\"done\"", 1);
    std::fs::write(root.join("edit.json"), &json).unwrap();
    let o = run(&root, &["notes", "--apply", "edit.json", "--yes"], None);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        read(&root, "a.md"),
        A.replace("status: todo", "status: done")
    );
    assert_eq!(read(&root, "b.md"), B);
}

#[test]
fn test_cli_17_csv_round_trip_from_stdin() {
    let root = vault("csv");
    let csv = printed(&root, "csv");
    // 変えずに戻す → 何も変わらない。
    let o = run(&root, &["notes", "--apply", "-", "--yes"], Some(&csv));
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(read(&root, "a.md"), A);
    assert_eq!(read(&root, "b.md"), B);
    // 数とリストを直す。
    let edited = csv.replace(",2,", ",5,").replace("\"a, b\"", "\"a, b, c\"");
    assert_ne!(edited, csv, "材料の CSV: {csv}");
    let o = run(&root, &["notes", "--apply", "-", "--yes"], Some(&edited));
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        read(&root, "b.md").contains("priority: 5\n"),
        "{}",
        read(&root, "b.md")
    );
    assert!(
        read(&root, "a.md").contains("tags: [a, b, c]"),
        "{}",
        read(&root, "a.md")
    );
}

#[test]
fn test_cli_17_empty_csv_cell_does_not_add_key() {
    let root = vault("empty");
    // b.md に tags は無い(CSV では空)。そのまま戻しても tags を足さない。
    let csv = printed(&root, "csv");
    std::fs::write(root.join("e.csv"), &csv).unwrap();
    let o = run(&root, &["notes", "--apply", "e.csv", "--yes"], None);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(read(&root, "b.md"), B);
}

#[test]
fn test_cli_17_any_problem_writes_nothing() {
    let root = vault("bad");
    // a.md は正しく直し、知らない path と数でない値を混ぜる → どれも書かない。
    let json = "[{\"path\": \"notes/a.md\", \"status\": \"done\"}, {\"path\": \"notes/none.md\", \"status\": \"x\"}, {\"path\": \"notes/b.md\", \"priority\": \"many\"}]";
    std::fs::write(root.join("bad.json"), json).unwrap();
    let o = run(&root, &["notes", "--apply", "bad.json", "--yes"], None);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    let e = err(&o);
    assert!(e.contains("none.md"), "{e}");
    assert!(e.contains("many"), "{e}");
    assert_eq!(read(&root, "a.md"), A);
    assert_eq!(read(&root, "b.md"), B);
}

#[test]
fn test_cli_17_locked_cell_is_a_problem() {
    let root = vault("lock");
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(b"---\nstatus: todo\n---\n");
    std::fs::write(root.join("notes/c.md"), &bom).unwrap();
    let json = "[{\"path\": \"notes/c.md\", \"status\": \"done\"}]";
    let o = run(&root, &["notes", "--apply", "-", "--yes"], Some(json));
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(std::fs::read(root.join("notes/c.md")).unwrap(), bom);
}

#[test]
fn test_cli_17_flag_combinations() {
    let root = vault("flags");
    let o = run(&root, &["notes", "--yes"], None);
    assert_eq!(o.status.code(), Some(2));
    let o = run(&root, &["notes", "--apply", "x.json", "--print"], None);
    assert_eq!(o.status.code(), Some(2));
    let o = run(&root, &["notes", "--apply", "missing.json"], None);
    assert_eq!(o.status.code(), Some(2));
}
