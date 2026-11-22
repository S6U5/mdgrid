#![cfg(unix)]
//! ビューの設定の画面の左の区画の一覧をキーで動かす(navigation.settings-sections-keys)を、本物の実行ファイルで
//! 確かめる(疑似端末 80×24)。画面の1行は「左の一覧 │ 右の欄」で、選びの印は左では行の頭の `> `、右の項目では
//! `│` のあとの `  >`。

use crate::pty;

use pty::{TempDir, Tui};

fn home(tag: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "vault"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    let v = h.path().join("vault");
    std::fs::write(v.join("a.md"), "---\ntitle: x\nstatus: todo\n---\n").unwrap();
    std::fs::write(v.join("b.md"), "---\ntitle: y\nstatus: done\n---\n").unwrap();
    h
}

/// 左の一覧で選びの印の付いた行の、印のあと(区画の番号と名前)が `name` で始まるか。
fn left_marked(s: &str, name: &str) -> bool {
    s.lines()
        .any(|l| l.strip_prefix("> ").is_some_and(|r| r.starts_with(name)))
}

/// 左の一覧に選びの印が無いか。
fn no_left_mark(s: &str) -> bool {
    !s.lines().any(|l| l.starts_with("> "))
}

/// 右の欄で選びの印の付いた項目の行の文字。
fn right_marked(s: &str) -> Vec<String> {
    s.lines()
        .filter_map(|l| l.split_once('│'))
        .filter_map(|(_, r)| r.strip_prefix("   >").map(|t| t.trim_end().to_string()))
        .collect()
}

/// 右の欄の見出し(キーが右に効いていれば `>` が付く)。
fn right_head(s: &str, head: &str) -> bool {
    s.lines()
        .filter_map(|l| l.split_once('│'))
        .any(|(_, r)| r.trim_end() == format!(" {head}"))
}

#[test]
fn test_settings_sections_keys_e2e() {
    // [stest -> req~navigation.settings-sections-keys~2] `o` で列の区画の項目にいる → `h` → 左の「列」が選ばれ、選びの印が左に移る。
    // `j` を2回 → 左は「並べ替え」で、右が並べ替えの中身。`l` → 右の並べ替えの1つ目の項目。Tab と Shift+Tab はこれまでどおり区画を巡る。
    let h = home("settings-sections");
    let mut t = Tui::spawn(&["vault"], h.path(), &[]);
    t.wait_for("1/2行");
    t.send("o");
    t.wait_for("ビューの設定");
    // 開いたときはフィルターの区画。Shift+Tab で列の区画の項目へ。
    t.send("\x1b[Z");
    t.wait_until("列の区画の項目", |s| {
        right_head(s, ">列") && right_marked(s).iter().any(|r| r.contains("title"))
    });
    t.send("h");
    t.wait_until("左の「列」に選びの印", |s| {
        left_marked(s, "1 列") && right_marked(s).is_empty() && right_head(s, " 列")
    });
    t.send("j");
    t.send("j");
    t.wait_until(
        "左は「並べ替え」で、右が並べ替えの中身",
        |s| {
            left_marked(s, "3 並べ替え")
                && right_head(s, " 並べ替え")
                && s.contains("+ 並べ替えを足す")
                && right_marked(s).is_empty()
        },
    );
    t.send("l");
    t.wait_until(
        "右の並べ替えの1つ目の項目に選びの印",
        |s| {
            no_left_mark(s) && right_marked(s) == ["+ 並べ替えを足す"] && right_head(s, ">並べ替え")
        },
    );
    t.send("\t");
    t.wait_until("Tab で次の区画(グループ)", |s| {
        right_head(s, ">グループ")
    });
    t.send("\x1b[Z");
    t.wait_until("Shift+Tab で前の区画(並べ替え)", |s| {
        right_head(s, ">並べ替え") && right_marked(s) == ["+ 並べ替えを足す"]
    });
    t.send("\x1b");
    t.wait_for("1/2行");
    t.send("q");
    pty::assert_gone(t.stop());
}
