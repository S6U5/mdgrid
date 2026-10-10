#![cfg(unix)]
//! `-w` は `--print` でも名前を確かめる(WS-3): 無い名前は理由1行と 0 以外の終了コード、パスが無ければ最初の表。

mod pty;

use pty::TempDir;
use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

#[test]
fn test_ws_3_print_unknown_workspace_fails() {
    // [WS-3] 無い名前 → 0 以外と理由。ある名前でパスなし → 最初の表(tasks)の行だけ。
    let h = TempDir::new("wsprint");
    let home = h.path();
    for (dir, note) in [("tasks", "a"), ("projects", "p")] {
        std::fs::create_dir_all(home.join(dir)).unwrap();
        std::fs::write(
            home.join(dir).join(format!("{note}.md")),
            "---\nstatus: todo\n---\n",
        )
        .unwrap();
    }
    let o = run(home, &["-w", "無い", "--print"]);
    assert_ne!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert!(!String::from_utf8_lossy(&o.stderr).trim().is_empty());
    let add = run(home, &["tasks", "--add-to", "Work"]);
    assert_eq!(
        add.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let o = run(home, &["-w", "Work", "--print", "--format", "csv"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(s.contains('a') && !s.contains("\np"), "{s}");
}
