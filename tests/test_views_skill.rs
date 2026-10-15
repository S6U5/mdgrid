//! [CLI-24] 印でビューと表を決める手引きのスキル(plugins/mdgrid/skills/mdgrid-views/SKILL.md)。
//! スキルに書いた印の例を見本のフォルダに置き、警告なしに読めて、印の表の行が出ることを確かめる
//! (古い書き方や読めない設定を AI に教えないため)。

mod pty;

use pty::TempDir;
use std::path::Path;
use std::process::{Command, Output};

const SKILL: &str = "plugins/mdgrid/skills/mdgrid-views/SKILL.md";

/// `home` を HOME と設定の置き場に、`cwd` を作業のフォルダにして起動する(標準出力はパイプ)。
fn run(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn skill_text() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(SKILL)).unwrap()
}

/// スキルの最初の ```toml の区画(印の例)。
fn marker_example() -> String {
    let s = skill_text();
    let start = s.find("```toml\n").expect("印の例の区画") + "```toml\n".len();
    let end = start + s[start..].find("```").expect("区画の終わり");
    s[start..end].to_string()
}

/// 見本のフォルダ: todo と archive/todo のノート、根の README.md、archive の表のノート。
fn sample(root: &Path) {
    for d in ["todo", "archive/todo", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(
        root.join("todo/20261015_open.md"),
        "---\ntitle: 開いている\nstatus: todo\nactor: ai\ndue: \"2026-10-20\"\npriority: p1\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join("archive/todo/20261001_done.md"),
        "---\ntitle: 終えた\nstatus: done\ndone_at: \"2026-10-02\"\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("README.md"), "---\ntitle: 説明\n---\n# 説明\n").unwrap();
    std::fs::create_dir_all(root.join(".mdgrid")).unwrap();
    std::fs::write(root.join(".mdgrid/workspace.toml"), marker_example()).unwrap();
}

#[test]
fn test_cli_24_skill_marker_example_reads_without_warnings() {
    // [CLI-24] スキルの印の例は警告なしに読める(--print-config --resolved の標準エラーに mdgrid: の行が無い)。
    let t = TempDir::new("views-skill-warn");
    let root = t.path();
    sample(root);
    let o = run(root, root, &["--print-config", "--resolved"]);
    assert_eq!(o.status.code(), Some(0));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!err.contains("mdgrid:"), "警告がある:\n{err}");
}

#[test]
fn test_cli_24_skill_marker_example_opens_its_views() {
    // [CLI-24] 印の例で開くと、Open のビューは todo の行だけ、Done のビューは done の行だけ。README.md の行は無い。
    let t = TempDir::new("views-skill-rows");
    let root = t.path();
    sample(root);
    let o = run(root, root, &["--print", "--with-path"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let s = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(s.contains("20261015_open.md"), "{s}");
    assert!(!s.contains("20261001_done.md"), "{s}");
    assert!(!s.contains("README.md"), "{s}");
    let o = run(root, root, &["--print", "--with-path", "--view", "Done"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let s = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(s.contains("20261001_done.md"), "{s}");
    assert!(!s.contains("20261015_open.md"), "{s}");
}

#[test]
fn test_cli_24_skill_names_the_checklist() {
    // [CLI-24] 手順に、型・選ぶ列の値・where・名前を隠さない・確かめ方が書いてある。
    let s = skill_text();
    for word in [
        "[types]",
        "values = [",
        "where = { status",
        "note_path = \"off\"",
        "--print-config --resolved",
        "--check",
        "untyped",
        "Quote non-ASCII keys",
    ] {
        assert!(s.contains(word), "スキルに {word} が無い");
    }
}
