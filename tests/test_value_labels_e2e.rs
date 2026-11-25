#![cfg(unix)]
//! 値の名前(`[rules.<列>] labels`)をセルと詳細の表示で見せ、書き出しは値のまま(navigation.value-labels-cells)を、
//! 本物の実行ファイルで確かめる。値は作ったもの。

use crate::pty;

use pty::{TempDir, Tui};
use std::process::Command;

/// `vault/.mdgrid/workspace.toml` に status の名前を書いた台帳(todo の行 alpha と doing の行 beta)。
fn home(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let v = home.path().join("vault");
    std::fs::create_dir_all(v.join(".mdgrid")).unwrap();
    std::fs::create_dir_all(v.join("notes")).unwrap();
    std::fs::create_dir_all(home.path().join("config")).unwrap();
    std::fs::create_dir_all(home.path().join("state")).unwrap();
    std::fs::write(
        v.join(".mdgrid/workspace.toml"),
        "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\", doing = \"進行中\" }\n\n[[table]]\nname = \"T\"\npath = \"notes\"\n",
    )
    .unwrap();
    std::fs::write(v.join("notes/alpha.md"), "---\nstatus: todo\n---\n").unwrap();
    std::fs::write(v.join("notes/beta.md"), "---\nstatus: doing\n---\n").unwrap();
    home
}

#[test]
fn test_value_labels_cells_screen_and_detail() {
    // [stest -> req~navigation.value-labels-cells~1] 本物の画面で、status のセルは「未着手」「進行中」(値の todo・doing は出ない)。
    // 詳細の表示(K)でも status の値は「未着手」。
    let h = home("vlc-screen");
    let mut t = Tui::spawn(&["vault"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("進行中");
    let s = t.screen();
    assert!(s.contains("未着手"), "{s}");
    assert!(!s.contains("todo") && !s.contains("doing"), "{s}");
    t.send("K");
    t.wait_until("詳細の表示の status の行", |s| {
        s.lines()
            .any(|l| l.contains("status") && l.contains("未着手"))
    });
    t.send("q");
    pty::assert_gone(t.stop());
}

#[test]
fn test_value_labels_cells_print_keeps_values() {
    // [stest -> req~navigation.value-labels-cells~1] `--print --format csv` の status は値のまま(todo・doing)。
    let h = home("vlc-print");
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(["vault", "--print", "--format", "csv"])
        .current_dir(h.path())
        .env("HOME", h.path())
        .env("XDG_CONFIG_HOME", h.path().join("config"))
        .env("XDG_STATE_HOME", h.path().join("state"))
        .env("LANG", "ja_JP.UTF-8")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let csv = String::from_utf8(out.stdout).unwrap();
    assert!(csv.contains("todo") && csv.contains("doing"), "{csv}");
    assert!(!csv.contains("未着手") && !csv.contains("進行中"), "{csv}");
}
