#![cfg(unix)]
//! [SC-15][SC-16][SC-17] 本物の実行ファイルで CSV・TSV を開いて直す(疑似端末 80×24)。
//! specs/_changes/2026-10-10-csv-source.md のタスク 5。

mod pty;

use pty::{wait_file, TempDir, Tui};
use std::process::Command;

const LEDGER: &str = "name,qty,note\r\nりんご,3,\"赤い, 甘い\"\r\nみかん,12,\r\n";

fn home(tag: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config", "state"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    std::fs::write(h.path().join("台帳.csv"), LEDGER).unwrap();
    h
}

#[test]
fn test_sc_16_e2e_edit_and_save_csv() {
    // [SC-15][SC-16] `mdgrid 台帳.csv` → 表。qty を直して保存 → その値だけが変わり、CRLF と引用符はそのまま。
    let h = home("csv-edit");
    let mut t = Tui::spawn(&["台帳.csv"], h.path(), &[]);
    t.wait_for("りんご");
    t.wait_for("赤い, 甘い");
    t.send_settle("l");
    t.send_settle("\r");
    t.send_settle("5");
    t.send_settle("\r");
    t.send_settle("\x13");
    t.wait_for("台帳.csv");
    t.send("\r");
    let got = wait_file(&h.path().join("台帳.csv"), &t, |s| s.contains("りんご,5"));
    assert_eq!(
        got,
        "name,qty,note\r\nりんご,5,\"赤い, 甘い\"\r\nみかん,12,\r\n"
    );
}

#[test]
fn test_sc_17_e2e_editor_refused_and_add_row() {
    // [SC-17] `e`(エディタで開く)は理由を出して断る。`a` → 末尾に1行足して1列目を打てる。
    let h = home("csv-add");
    let mut t = Tui::spawn(&["台帳.csv"], h.path(), &[]);
    t.wait_for("りんご");
    t.send_settle("e");
    t.wait_for("CSV・TSV の表では使えない");
    t.send_settle("a");
    t.send_settle("ぶどう");
    t.send_settle("\r");
    t.send_settle("\x13");
    t.send("\r");
    let got = wait_file(&h.path().join("台帳.csv"), &t, |s| s.contains("ぶどう"));
    assert_eq!(
        got,
        "name,qty,note\r\nりんご,3,\"赤い, 甘い\"\r\nみかん,12,\r\nぶどう,,\r\n"
    );
}

#[test]
fn test_sc_15_e2e_print_and_unreadable() {
    // [SC-15] `--print` は CSV の行を出す。UTF-8 でないファイルは理由1行と 0 以外の終了コード。
    let h = home("csv-print");
    std::fs::write(h.path().join("sjis.csv"), b"name\n\x82\xa0\n").unwrap();
    let bin = env!("CARGO_BIN_EXE_mdgrid");
    let out = Command::new(bin)
        .args(["--print", "--format", "json", "台帳.csv"])
        .current_dir(h.path())
        .env("LANG", "ja_JP.UTF-8")
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("\"qty\": 12") && s.contains("赤い, 甘い"), "{s}");
    let out = Command::new(bin)
        .args(["--print", "sjis.csv"])
        .current_dir(h.path())
        .env("LANG", "ja_JP.UTF-8")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(e.contains("UTF-8") && e.lines().count() == 1, "{e}");
}
