#![cfg(unix)]
//! タスク 1(open-md-file)の受け入れ: 起動の引数に `.md` のファイルを渡す(CLI-15)。
//! 仕様: specs/cli/spec.md の CLI-15(関係: CLI-1・CLI-4、specs/output/spec.md の OUT-3)、
//! specs/_changes/2026-10-06-open-md-file.md の「不明点と仮定」。
//! 実装(src/main.rs・src/ui)を見ずに、要件だけで書いた。本物の実行ファイルを動かす。
//!
//! - `--print` と失敗は `Command::output`(標準出力はパイプ。設定と状態の置き場は一時フォルダ)。
//! - 画面は疑似端末の道具(tests/pty/mod.rs)。選んでいる行は、行の左の `>` で見分ける。
//!
//! 材料の保管庫(`<home>/notes`。起動の引数は相対、子の作業フォルダは `<home>`):
//!
//! | note | title |
//! |------|-------|
//! | a.md | A     |
//! | b.md | B     |
//! | c.md | C     |
//!
//! 表の並び(`.base` の無いフォルダの既定)は a.md・b.md・c.md と仮定した。

mod pty;

use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;

use pty::{assert_gone, TempDir, Tui};

// ---------------------------------------------------------------- 場

/// `<home>/notes` に a.md・b.md・c.md を置き、設定と状態の置き場を作る。
fn make_home(tag: &str) -> TempDir {
    let home = TempDir::new(&format!("open-md-{tag}"));
    for d in ["notes", "config", "state"] {
        std::fs::create_dir_all(home.path().join(d)).unwrap();
    }
    for (name, title) in [("a.md", "A"), ("b.md", "B"), ("c.md", "C")] {
        std::fs::write(
            home.path().join("notes").join(name),
            format!("---\ntitle: {title}\n---\n本文 {title}\n"),
        )
        .unwrap();
    }
    home
}

/// 作業フォルダを `home` にし、設定と状態の置き場を一時フォルダの下に向けて動かす。標準出力はパイプ。
fn run_in(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env_remove("MDGRID_CONFIG")
        .output()
        .expect("mdgrid を起動できる")
}

fn stderr_lines(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// 画面の行のうち、左の `>` で選ばれている行(表の行)。
fn selected_lines(screen: &str) -> Vec<String> {
    screen
        .lines()
        .filter(|l| l.trim_start().starts_with('>'))
        .map(|l| l.to_string())
        .collect()
}

/// 疑似端末で起動し、表の3行が出るまで待つ。
fn open(home: &Path, args: &[&str]) -> Tui {
    let t = Tui::spawn(args, home, &[]);
    // SR-29: 左のノートの欄は `.md` を除いた名前(a・b・c)。
    t.wait_for("3行");
    std::thread::sleep(Duration::from_millis(300));
    t
}

/// 選ばれている行がちょうど1つで、それが `note` の行であることを待って確かめる。
fn assert_selected(t: &Tui, note: &str) {
    // SR-29: 欄の名前は `.md` を除いたもの。行は `>名前 ` で始まる。
    let row = |n: &str| format!(">{} ", n.trim_end_matches(".md"));
    t.wait_until(&format!("{note} の行が選ばれている"), |s| {
        let sel = selected_lines(s);
        sel.len() == 1 && sel[0].trim_start().starts_with(&row(note))
    });
    let s = t.screen();
    for other in ["a.md", "b.md", "c.md"].iter().filter(|n| **n != note) {
        assert!(
            !selected_lines(&s)
                .iter()
                .any(|l| l.trim_start().starts_with(&row(other))),
            "{other} の行は選ばれていない。画面:\n{s}"
        );
    }
}

// ---------------------------------------------------------------- --print

#[test]
fn test_cli_15_print_md_file_prints_its_row() {
    // [CLI-15] `notes/b.md --print` → `notes --print` と同じ列で、b の1行だけ、終了コード 0。
    let home = make_home("print");
    let folder = run_in(home.path(), &["notes", "--print"]);
    assert_eq!(folder.status.code(), Some(0));
    let folder_out = String::from_utf8(folder.stdout.clone()).unwrap();
    let file = run_in(home.path(), &["notes/b.md", "--print"]);
    assert_eq!(
        file.status.code(),
        Some(0),
        "`notes/b.md --print` は終了コード 0。stderr: {}",
        String::from_utf8_lossy(&file.stderr)
    );
    let out = String::from_utf8(file.stdout.clone()).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "見出しと b の1行: {out:?}");
    assert_eq!(
        lines[0],
        folder_out.lines().next().unwrap(),
        "列はフォルダの表と同じ"
    );
    assert!(lines[1].contains('B'), "{out:?}");
}

#[test]
fn test_cli_15_print_two_md_files() {
    // [CLI-15] `notes/a.md notes/b.md --print` → a と b の2行。
    let home = make_home("print-two");
    let file = run_in(home.path(), &["notes/a.md", "notes/b.md", "--print"]);
    assert_eq!(file.status.code(), Some(0));
    let out = String::from_utf8(file.stdout).unwrap();
    let rows: Vec<&str> = out.lines().skip(1).collect();
    assert_eq!(rows.len(), 2, "{out:?}");
    assert!(rows.iter().any(|l| l.contains('A')) && rows.iter().any(|l| l.contains('B')));
}

#[test]
fn test_cli_15_print_md_file_extension_is_case_insensitive() {
    // [CLI-15] 拡張子の大文字小文字を問わない(change record の仮定)。
    let home = make_home("print-upper");
    std::fs::rename(
        home.path().join("notes/b.md"),
        home.path().join("notes/B.MD"),
    )
    .unwrap();
    let file = run_in(home.path(), &["notes/B.MD", "--print"]);
    assert_eq!(
        file.status.code(),
        Some(0),
        "`notes/B.MD --print` は終了コード 0。stderr: {}",
        String::from_utf8_lossy(&file.stderr)
    );
    let out = String::from_utf8(file.stdout).unwrap();
    assert_eq!(out.lines().count(), 2, "{out:?}");
    assert!(out.contains('B'), "{out:?}");
}

// ---------------------------------------------------------------- 失敗

#[test]
fn test_cli_15_missing_md_file_fails_with_one_line_and_code_2() {
    // [CLI-15][CLI-4] 在らない `.md` → 理由1行、終了コード 2、標準出力は空。
    let home = make_home("missing");
    for extra in [&[][..], &["--print"][..]] {
        let mut args = vec!["notes/無い.md"];
        args.extend_from_slice(extra);
        let out = run_in(home.path(), &args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{args:?}: 終了コード 2。stdout: {} stderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let lines = stderr_lines(&out);
        assert_eq!(lines.len(), 1, "{args:?}: 標準エラーは理由1行: {lines:?}");
        assert!(
            lines[0].contains("無い.md"),
            "{args:?}: 理由は渡したファイルを名指す: {:?}",
            lines[0]
        );
        assert!(
            out.stdout.is_empty(),
            "{args:?}: 標準出力は空: {:?}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
}

// ---------------------------------------------------------------- 画面

#[test]
fn test_cli_15_folder_selects_first_row_for_comparison() {
    // [CLI-1] 比べ: フォルダを開くと最初の行(a.md)が選ばれている。
    let home = make_home("tui-folder");
    let t = open(home.path(), &["notes"]);
    assert_selected(&t, "a.md");
    assert_gone(t.stop());
}

#[test]
fn test_cli_15_md_file_opens_folder_and_selects_its_row() {
    // [CLI-15] `notes/c.md` → notes の表が開き、c.md の行が選ばれている。
    let home = make_home("tui-file");
    let t = open(home.path(), &["notes/c.md"]);
    assert!(
        t.screen().lines().any(|l| l.starts_with(" b ")),
        "notes の表の全ての行が出る"
    );
    assert_selected(&t, "c.md");
    assert_gone(t.stop());
}

#[test]
fn test_cli_15_readonly_md_file_selects_its_row() {
    // [CLI-15] `--readonly notes/b.md` でも b.md の行が選ばれている。
    let home = make_home("tui-readonly");
    let t = open(home.path(), &["--readonly", "notes/b.md"]);
    assert_selected(&t, "b.md");
    assert_gone(t.stop());
}
