//! [CLI-16] --print の --filter(式。何度でも、全部を満たす)と --sort(列[:asc|:desc]。何度でも、渡した順)。
//! specs/_changes/2026-10-06-print-filter-sort.md。本物の実行ファイルで確かめる。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-pfs-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    for (f, t) in [
        (
            "a.md",
            "---\nstatus: done\ndue: 2026-10-01\npriority: 1\n---\n",
        ),
        (
            "b.md",
            "---\nstatus: todo\ndue: 2026-10-09\npriority: 3\n---\n",
        ),
        (
            "c.md",
            "---\nstatus: doing\ndue: 2026-10-05\npriority: 2\n---\n",
        ),
        (
            "d.md",
            "---\nstatus: todo\ndue: 2026-10-02\npriority: 2\n---\n",
        ),
    ] {
        std::fs::write(root.join(f), t).unwrap();
    }
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .output()
        .unwrap()
}

/// 行のノートの名前の並び(--with-path の1列目の最後の部分)。
fn names(out: &Output) -> Vec<String> {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .skip(1)
        .map(|l| {
            let p = l.split(',').next().unwrap();
            p.rsplit('/').next().unwrap().to_string()
        })
        .collect()
}

#[test]
fn test_cli_16_folder_filter_and_sort() {
    let root = vault("folder");
    let r = root.to_str().unwrap();
    let out = run(
        &root,
        &[
            r,
            "--print",
            "--with-path",
            "--filter",
            "status != \"done\"",
            "--sort",
            "due",
        ],
    );
    assert_eq!(names(&out), ["d.md", "c.md", "b.md"]);
}

#[test]
fn test_cli_16_sort_desc_and_several_keys() {
    let root = vault("desc");
    let r = root.to_str().unwrap();
    let out = run(
        &root,
        &[
            r,
            "--print",
            "--with-path",
            "--sort",
            "priority:desc",
            "--sort",
            "due",
        ],
    );
    assert_eq!(names(&out), ["b.md", "d.md", "c.md", "a.md"]);
}

#[test]
fn test_cli_16_several_filters_all_hold() {
    let root = vault("and");
    let r = root.to_str().unwrap();
    let out = run(
        &root,
        &[
            r,
            "--print",
            "--with-path",
            "--filter",
            "status != \"done\"",
            "--filter",
            "priority >= 2",
            "--sort",
            "file.name",
        ],
    );
    assert_eq!(names(&out), ["b.md", "c.md", "d.md"]);
}

#[test]
fn test_cli_16_base_view_filter_is_combined() {
    let root = vault("base");
    std::fs::write(
        root.join("t.base"),
        "views:\n  - type: table\n    name: v\n    filters: 'status == \"todo\"'\n    order: [file.name, due]\n    sort:\n      - property: file.name\n        direction: ASC\n",
    )
    .unwrap();
    let base = root.join("t.base");
    let b = base.to_str().unwrap();
    let out = run(
        &root,
        &[b, "--print", "--with-path", "--filter", "priority >= 3"],
    );
    assert_eq!(names(&out), ["b.md"]);
    // --sort はビューの並べ替えの代わり。
    let out = run(&root, &[b, "--print", "--with-path", "--sort", "due"]);
    assert_eq!(names(&out), ["d.md", "b.md"]);
}

#[test]
fn test_cli_16_bad_expression_and_sort_fail() {
    let root = vault("bad");
    let r = root.to_str().unwrap();
    let out = run(&root, &[r, "--print", "--filter", "foo("]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&out.stderr).lines().count(), 1);
    let out = run(&root, &[r, "--print", "--sort", "due:sideways"]);
    assert_eq!(out.status.code(), Some(2));
    // --print が無ければ使えない。
    let out = run(&root, &[r, "--filter", "x"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn test_cli_16_without_flags_unchanged() {
    let root = vault("same");
    let r = root.to_str().unwrap();
    // 旗が無くても有っても、フォルダの列は既定の表と同じ。
    let plain = run(&root, &[r, "--print"]);
    let sorted = run(&root, &[r, "--print", "--sort", "due"]);
    assert!(plain.status.success() && sorted.status.success());
    let head = |o: &Output| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(head(&plain), head(&sorted));
}
