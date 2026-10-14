//! [CLI-23] `--check` の残りの確かめ方: `.md` を渡したらそのノートだけ、ビューの絞り込みで列の型が変わらない。
//! 記録: specs/_changes/2026-10-14-type-check.md(タスク 4)、specs/_decisions/2026-10-14-check-kinds.md。
//! 本物の実行ファイルで確かめる。作業のフォルダの作り方と動かし方は tests/test_check_cli.rs(錠あり)から写した。

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn vault(name: &str, notes: &[(&str, &str)], config: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mdgrid-check-more-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(root.join("notes/.obsidian")).unwrap();
    std::fs::create_dir_all(root.join("cfg")).unwrap();
    for (n, body) in notes {
        std::fs::write(root.join("notes").join(n), body).unwrap();
    }
    std::fs::write(root.join("check.toml"), config).unwrap();
    root
}

/// `mdgrid <target> --check --config check.toml` を root で動かす。
fn check(root: &Path, target: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .current_dir(root)
        .args([target, "--check", "--config", "check.toml"])
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
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

fn lines(o: &Output) -> Vec<String> {
    out(o).lines().map(str::to_string).collect()
}

const TYPES_AND_REQUIRED: &str = "[types]\ndue = \"date\"\n\n[rules.status]\nrequired = true\n";

#[test]
fn test_cli_23_md_arg_checks_only_that_note() {
    // [CLI-23] `.md` を渡すと(CLI-15 と同じく)そのノートの行だけを確かめる。
    // ほかのノートに問題があっても出ず、渡したノートに問題が無ければ終了コード 0。
    let ok = "---\ntitle: OK\ndue: 2026-10-02\nstatus: todo\n---\n";
    let bad_type = "---\ntitle: X\ndue: someday\nstatus: todo\n---\n";
    let no_status = "---\ntitle: Y\ndue: 2026-10-01\n---\n";
    let root = vault(
        "md",
        &[("a.md", ok), ("b.md", bad_type), ("c.md", no_status)],
        TYPES_AND_REQUIRED,
    );

    // 問題の無い a.md だけ → 出力なし、終了コード 0(b・c の問題は出ない)。
    let o = check(&root, "notes/a.md");
    assert_eq!(o.status.code(), Some(0), "{}\n{}", out(&o), err(&o));
    assert_eq!(out(&o), "", "{}", err(&o));

    // 問題のある b.md だけ → b の問題だけ(c の必須の違反は出ない)、「問題 1」、終了コード 1。
    let o = check(&root, "notes/b.md");
    assert_eq!(o.status.code(), Some(1), "{}\n{}", out(&o), err(&o));
    assert_eq!(
        lines(&o),
        vec!["notes/b.md:3: due: 型が合わない(日付ではない)".to_string()],
        "{}",
        err(&o)
    );
    assert!(err(&o).contains("問題 1"), "{}", err(&o));

    // ノートは書き換えない。
    assert_eq!(
        std::fs::read(root.join("notes/b.md")).unwrap(),
        bad_type.as_bytes()
    );
}

#[test]
fn test_cli_23_view_filter_keeps_column_kind() {
    // [CLI-23] ビューの絞り込みで確かめる行が z.md だけになっても、列の型は画面と同じく
    // 読み込んだ全部のノート(a・b・c は日付)からの推定で「日付」に決まり、z.md の `due: someday` が問題になる。
    // (z.md だけから推定すると文字の列になり、問題にならない。)設定に [types] は書かない。
    let date_a = "---\ntitle: A\ndue: 2026-10-01\n---\n";
    let date_b = "---\ntitle: B\ndue: 2026-10-02\n---\n";
    let date_c = "---\ntitle: C\ndue: 2026-10-03\n---\n";
    let z = "---\ntitle: Z\ndue: someday\n---\n";
    let root = vault(
        "view",
        &[
            ("a.md", date_a),
            ("b.md", date_b),
            ("c.md", date_c),
            ("z.md", z),
        ],
        "",
    );
    std::fs::write(
        root.join("notes/t.base"),
        "views:\n  - type: table\n    name: z だけ\n    filters:\n      and:\n        - file.name == \"z.md\"\n    order:\n      - title\n      - due\n",
    )
    .unwrap();

    // 絞り込みが効いていることの確かめ(--print で z の行だけ)。
    let p = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .current_dir(&root)
        .args(["notes/t.base", "--print", "--config", "check.toml"])
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let printed = String::from_utf8_lossy(&p.stdout).into_owned();
    assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
    assert!(
        printed.contains("Z") && !printed.contains("2026-10-01"),
        "{printed}"
    );

    let o = check(&root, "notes/t.base");
    assert_eq!(o.status.code(), Some(1), "{}\n{}", out(&o), err(&o));
    assert_eq!(
        lines(&o),
        vec!["notes/z.md:3: due: 型が合わない(日付ではない)".to_string()],
        "{}",
        err(&o)
    );
    assert!(err(&o).contains("問題 1"), "{}", err(&o));
}
