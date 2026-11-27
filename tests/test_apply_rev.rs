//! [cli.print-rev][cli.apply-stale-row] 書き出した行の印 `_rev` と、書き出したあとに変わったノートを `--apply` が書かないこと。
//! 本物の実行ファイルで確かめる。

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const A: &str = "---\nstatus: todo\ndue: 2026-12-01\n---\nbody a\n";

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-rev-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("notes/.obsidian")).unwrap();
    std::fs::write(root.join("notes/a.md"), A).unwrap();
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn note(root: &Path) -> String {
    std::fs::read_to_string(root.join("notes/a.md")).unwrap()
}

fn printed(root: &Path, args: &[&str]) -> String {
    let mut all = vec!["notes", "--print", "--with-path"];
    all.extend_from_slice(args);
    let o = run(root, &all);
    assert!(o.status.success(), "{}", err(&o));
    out(&o)
}

/// CSV の見出しの行と1行目のデータの行を、欄に分けて返す(この試験の値に `,` と `"` は無い)。
fn head_and_row(csv: &str) -> (Vec<String>, Vec<String>) {
    let mut lines = csv.lines();
    let split = |l: &str| l.split(',').map(str::to_string).collect::<Vec<_>>();
    (split(lines.next().unwrap()), split(lines.next().unwrap()))
}

fn rev_of(csv: &str) -> String {
    let (head, row) = head_and_row(csv);
    assert_eq!(head.last().map(String::as_str), Some("_rev"), "{csv}");
    row.last().unwrap().clone()
}

/// 外でノートを直す(更新時刻もずらす)。
fn edit_outside(root: &Path, text: &str) {
    let p = root.join("notes/a.md");
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    std::fs::write(&p, text).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old + std::time::Duration::from_secs(10))
        .unwrap();
}

#[test]
fn test_print_rev_is_the_last_column_and_follows_values_only() {
    // [stest -> req~cli.print-rev~1] `--print --with-path` の見出しの最後が `_rev` で印は空でない。本文だけを直しても同じ印、due を直すと違う印。
    let root = vault("print");
    let first = printed(&root, &[]);
    let rev = rev_of(&first);
    assert!(!rev.is_empty(), "{first}");
    edit_outside(&root, &A.replace("body a", "body a, longer"));
    assert_eq!(rev_of(&printed(&root, &[])), rev);
    edit_outside(&root, &A.replace("2026-12-01", "2026-12-20"));
    assert_ne!(rev_of(&printed(&root, &[])), rev);
}

#[test]
fn test_print_rev_in_json_only_for_apply_formats() {
    // [stest -> req~cli.print-rev~1] json は各オブジェクトの最後の鍵が `_rev`(csv と同じ印)。md と tsv には `_rev` が無い。
    let root = vault("json");
    let rev = rev_of(&printed(&root, &[]));
    // 鍵の並びは文字で見る(serde_json の Map は鍵を並べ替える)。
    let text = printed(&root, &["--format", "json"]);
    let line = text.lines().find(|l| l.contains("\"path\"")).unwrap();
    assert!(
        line.trim_end_matches(',')
            .ends_with(&format!("\"_rev\": \"{rev}\"}}")),
        "{text}"
    );
    for f in ["md", "tsv"] {
        let s = printed(&root, &["--format", f]);
        assert!(!s.contains("_rev"), "{f}: {s}");
    }
}

#[test]
fn test_apply_refuses_row_changed_after_export() {
    // [stest -> req~cli.apply-stale-row~1] 書き出したあとに外で due を直した → 書き出した CSV の status を done に直して `--apply --yes`
    // → どれも書かず(status は todo、due は外の値)終了コード 2。理由に path と、書き出し直すことが出る。
    let root = vault("stale");
    let csv = printed(&root, &[]);
    let outside = A.replace("2026-12-01", "2026-12-20");
    edit_outside(&root, &outside);
    std::fs::write(root.join("edit.csv"), csv.replacen("todo", "done", 1)).unwrap();
    let o = run(&root, &["notes", "--apply", "edit.csv", "--yes"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(note(&root), outside);
    let e = err(&o);
    assert!(e.contains("a.md") && e.contains("export it again"), "{e}");
}

#[test]
fn test_apply_refuses_stale_export_as_is() {
    // [stest -> req~cli.apply-stale-row~1] 書き出したあとに外で due を直した → 書き出した CSV をそのまま戻すと due を古い値に
    // 戻すことになるので、どれも書かずに終了コード 2。
    let root = vault("asis");
    let csv = printed(&root, &[]);
    let outside = A.replace("2026-12-01", "2026-12-20");
    edit_outside(&root, &outside);
    std::fs::write(root.join("edit.csv"), csv).unwrap();
    let o = run(&root, &["notes", "--apply", "edit.csv", "--yes"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(note(&root), outside);
}

#[test]
fn test_apply_stale_row_without_differences_is_fine() {
    // [stest -> req~cli.apply-stale-row~1] 列が status だけのビューで書き出したあと、外でビューに無い due を直した → そのまま戻す
    // と今と違うセルが無いので、何も書かずに終了コード 0。
    let root = vault("hidden");
    std::fs::write(
        root.join("notes/v.base"),
        "views:\n  - type: table\n    name: S\n    order:\n      - status\n",
    )
    .unwrap();
    let o = run(
        &root,
        &["notes/v.base", "--view", "S", "--print", "--with-path"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let csv = out(&o);
    let outside = A.replace("2026-12-01", "2026-12-20");
    edit_outside(&root, &outside);
    std::fs::write(root.join("edit.csv"), &csv).unwrap();
    let o = run(
        &root,
        &[
            "notes/v.base",
            "--view",
            "S",
            "--apply",
            "edit.csv",
            "--yes",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}\n{csv}", err(&o));
    assert_eq!(note(&root), outside);
}

#[test]
fn test_apply_fresh_export_writes_and_adds_no_rev_key() {
    // [stest -> req~cli.apply-stale-row~1] 外で直していないノートの書き出しの status を done に直して `--apply --yes` → status が
    // done になり、ノートに `_rev` のキーは足されない。
    let root = vault("fresh");
    let csv = printed(&root, &[]);
    std::fs::write(root.join("edit.csv"), csv.replacen("todo", "done", 1)).unwrap();
    let o = run(&root, &["notes", "--apply", "edit.csv", "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(note(&root), A.replace("todo", "done"));
}

#[test]
fn test_apply_without_rev_column_works_as_before() {
    // [stest -> req~cli.apply-stale-row~1] `_rev` の列の無い手で作った CSV(`path,status`)は、外で直したノートにも今までどおり
    // 今と違うセルを書く。
    let root = vault("norev");
    edit_outside(&root, &A.replace("2026-12-01", "2026-12-20"));
    std::fs::write(root.join("edit.csv"), "path,status\nnotes/a.md,done\n").unwrap();
    let o = run(&root, &["notes", "--apply", "edit.csv", "--yes"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        note(&root),
        A.replace("todo", "done")
            .replace("2026-12-01", "2026-12-20")
    );
}
