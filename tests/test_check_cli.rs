//! [CLI-23] `mdgrid <フォルダ> --check`: 画面を出さずに、型(CE-2・CE-35)と決まり(CE-36)に合わない値を
//! 1つ1行 `<パス>:<行>: <列>: <理由>`(キーが無いときは `<パス>: <列>: <理由>`)で標準出力に出し、
//! 問題の数を標準エラーに出す。終了コードは 問題なし 0・あり 1・起動できない 2。ノートは書き換えない。
//! 記録: specs/_changes/2026-10-14-type-check.md(タスク #4)。本物の実行ファイルで確かめる。
//!
//! 子の作業のフォルダを一時フォルダにし、引数を `notes` で渡すので、パスは `notes/a.md` の形(OUT-3 と同じ)。
//! 言語は .cargo/config.toml の [env] の日本語のまま。設定は `--config` の一時のファイルで渡し、
//! XDG_CONFIG_HOME も一時のフォルダにして利用者の設定を読まない。

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// `due: someday`(3行目)の、日付の列に合わないノート。
const A_BAD: &str = "---\ntitle: A\ndue: someday\nstatus: todo\n---\nbody a\n";
/// status の無いノート。
const B_BAD: &str = "---\ntitle: B\ndue: 2026-10-01\n---\nbody b\n";

const A_OK: &str = "---\ntitle: A\ndue: 2026-10-02\nstatus: todo\n---\nbody a\n";
const B_OK: &str = "---\ntitle: B\ndue: 2026-10-01\nstatus: done\n---\nbody b\n";

const TYPES_AND_REQUIRED: &str = "[types]\ndue = \"date\"\n\n[rules.status]\nrequired = true\n";

fn vault(name: &str, notes: &[(&str, &str)], config: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mdgrid-check-{name}-{}-{nanos}",
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

#[test]
fn test_cli_23_reports_type_and_required_problems() {
    // [CLI-23] 型の違反(行あり)と必須のキーの無いノート(行なし)を、パスの順に1行ずつ出し、
    // 標準エラーに「問題 2」、終了コード 1。ノートのバイトは変わらない。
    let root = vault(
        "report",
        &[("a.md", A_BAD), ("b.md", B_BAD)],
        TYPES_AND_REQUIRED,
    );
    let o = check(&root, "notes");
    assert_eq!(o.status.code(), Some(1), "{}\n{}", out(&o), err(&o));
    assert_eq!(
        lines(&o),
        vec![
            "notes/a.md:3: due: 型が合わない(日付ではない)".to_string(),
            "notes/b.md: status: 必須のキーが無い".to_string(),
        ],
        "{}",
        err(&o)
    );
    let e = err(&o);
    assert!(e.contains("問題 2"), "{e}");
    assert_eq!(e.trim_end().lines().count(), 1, "標準エラーは1行: {e}");
    assert_eq!(
        std::fs::read(root.join("notes/a.md")).unwrap(),
        A_BAD.as_bytes()
    );
    assert_eq!(
        std::fs::read(root.join("notes/b.md")).unwrap(),
        B_BAD.as_bytes()
    );
}

#[test]
fn test_cli_23_no_problems_prints_nothing_and_exits_0() {
    // [CLI-23] 直したあと → 標準出力に何も出さず、終了コード 0。
    let root = vault(
        "clean",
        &[("a.md", A_OK), ("b.md", B_OK)],
        TYPES_AND_REQUIRED,
    );
    let o = check(&root, "notes");
    assert_eq!(o.status.code(), Some(0), "{}\n{}", out(&o), err(&o));
    assert_eq!(out(&o), "", "{}", err(&o));
}

#[test]
fn test_cli_23_missing_path_exits_2() {
    // [CLI-23] 無いパス → 理由を1行出して終了コード 2(CLI-4 と同じ)。
    let root = vault("missing", &[("a.md", A_OK)], TYPES_AND_REQUIRED);
    let o = check(&root, "nothere");
    assert_eq!(o.status.code(), Some(2), "{}\n{}", out(&o), err(&o));
    assert_eq!(out(&o), "");
    let e = err(&o);
    assert_eq!(e.trim_end().lines().count(), 1, "理由は1行: {e}");
    assert!(!e.trim().is_empty());
}

#[test]
fn test_cli_23_values_violation_lines_in_line_order() {
    // [CLI-23] values に無い値は「選べる値ではない」。1つのノートの2つの問題は行の順。
    // values に合う値(done)は出ない。
    let cfg = "[types]\ndue = \"date\"\n\n[rules.status]\nvalues = [\"todo\", \"done\"]\n";
    let c = "---\nstatus: wip\ndue: later\n---\n";
    let d = "---\nstatus: done\ndue: 2026-10-03\n---\n";
    let root = vault("values", &[("c.md", c), ("d.md", d)], cfg);
    let o = check(&root, "notes");
    assert_eq!(o.status.code(), Some(1), "{}\n{}", out(&o), err(&o));
    assert_eq!(
        lines(&o),
        vec![
            "notes/c.md:2: status: 選べる値ではない".to_string(),
            "notes/c.md:3: due: 型が合わない(日付ではない)".to_string(),
        ],
        "{}",
        err(&o)
    );
    assert!(err(&o).contains("問題 2"), "{}", err(&o));
    assert_eq!(
        std::fs::read(root.join("notes/c.md")).unwrap(),
        c.as_bytes()
    );
}
