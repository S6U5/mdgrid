#![cfg(unix)]
//! 日付の言葉を打っているあいだの読みの見せ方(cell-edit.date-words-preview)を、本物の実行ファイルで確かめる
//! (疑似端末 80×24)。今日は環境変数 MDGRID_TODAY で 2026-11-21(土)に固定し、週の始まりは既定(日曜)。
//! unix だけ: 疑似端末の道具(tests/pty)が unix だけなので、Windows では飛ぶ。走るのは CI の ci.yml の test の
//! ubuntu-latest と macos-latest。

use crate::pty;

use pty::{assert_gone, TempDir, Tui};

/// due だけを持つノート1つの表 `t`。
fn root(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    std::fs::create_dir_all(h.join("t")).unwrap();
    std::fs::create_dir_all(h.join("state")).unwrap();
    std::fs::write(h.join("t/a.md"), "---\ndue: 2026-10-05\n---\n").unwrap();
    home
}

/// 画面のうち、`needle` を含む行。
fn line_with(s: &str, needle: &str) -> Option<String> {
    s.lines().find(|l| l.contains(needle)).map(str::to_string)
}

#[test]
fn test_date_words_preview_e2e() {
    // [stest -> req~cell-edit.date-words-preview~2] `来週の金` まで打つと読めないと出て、続けて `曜` で入力の欄の横に `2026-11-27(金)` が出てカレンダーが 27 日を選び、Enter で due: 2026-11-27 をためる。
    let home = root("date-words-e2e");
    let mut tui = Tui::spawn(&["t"], home.path(), &[("MDGRID_TODAY", "2026-11-21")]);
    tui.wait_until("表が出る", |s| s.contains("2026-10-05"));
    tui.send("\r");
    tui.wait_until("カレンダーが出る", |s| s.contains("2026年10月"));
    assert!(!tui.screen().contains("読めない"), "{}", tui.screen());
    tui.send("来週の金");
    tui.wait_until("読めないと出る", |s| {
        line_with(s, "来週の金").is_some_and(|l| l.contains("読めない"))
    });
    tui.send("曜");
    tui.wait_until("読んだ日が出る", |s| {
        line_with(s, "来週の金曜").is_some_and(|l| l.contains("2026-11-27(金)"))
    });
    let s = tui.screen();
    assert!(!s.contains("読めない"), "{s}");
    assert!(s.contains("2026年11月"), "カレンダーが 11 月へ動く:\n{s}");
    assert!(s.contains(">27"), "カレンダーが 27 日を選ぶ:\n{s}");
    tui.send("\r");
    tui.wait_until("ためた値が表に出る", |s| {
        s.contains("2026-11-27") && !s.contains("2026年11月")
    });
    tui.send("\x13"); // Ctrl+S: 保存の確認(ためた変更の差分)
    tui.wait_for("+due: 2026-11-27");
    assert_gone(tui.stop());
}

#[test]
fn test_date_words_preview_new_note_e2e() {
    // [stest -> req~cell-edit.date-words-preview~2] 新しいノートの窓(`a`)の due の欄に `明日` と打つと、同じ行の打った文字の右に `2026-11-22(日)` が出て、
    // その右に窓の右の縁 `│` が残る(枠の外にはみ出さない。2026-11-25 の実地の操作で、枠の右の外に出ていた)。
    let home = root("date-words-newnote-e2e");
    let mut tui = Tui::spawn(&["t"], home.path(), &[("MDGRID_TODAY", "2026-11-21")]);
    tui.wait_until("表が出る", |s| s.contains("2026-10-05"));
    tui.send("a");
    tui.wait_until("新しいノートの窓が出る", |s| {
        s.contains("新しいノート")
    });
    tui.send("x\r");
    tui.wait_until("due の欄のカレンダーが出る", |s| {
        s.contains("2026年11月")
    });
    tui.send("明日");
    tui.wait_until("読んだ日が出る", |s| {
        line_with(s, "明日").is_some_and(|l| l.contains("2026-11-22(日)"))
    });
    let s = tui.screen();
    let line = line_with(&s, "明日").unwrap();
    let typed = line.find("明日").unwrap();
    let at = line.find("2026-11-22(日)").unwrap();
    assert!(
        line.find("due").is_some_and(|d| d < typed) && typed < at,
        "欄の名前・打った文字・読んだ日の順:\n{s}"
    );
    assert!(
        line[at..].contains('│'),
        "読んだ日の右に窓の右の縁が残る:\n{s}"
    );
    tui.send("\x1b");
    assert_gone(tui.stop());
}
