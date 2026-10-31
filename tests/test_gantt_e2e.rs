#![cfg(unix)]
//! ガント(GA-1・GA-2・GA-4)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 今日は環境変数 MDGRID_TODAY で 2026-10-06 に固定する。行番号や字の位置は決め打ちしない。

mod pty;

use pty::{TempDir, Tui};

/// 印の [[table]] に `gantt_line` を書いた根と、その下の表 `t`。
/// alpha は 10/01〜10/07 の7日、beta は due だけ(10/03)、gamma は日付なし。
fn root(tag: &str, gantt_line: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        &format!("[[table]]\nname = \"T\"\npath = \"t\"\n{gantt_line}\n"),
    );
    w(
        "t/alpha.md",
        "---\nstart: 2026-10-01\ndue: 2026-10-07\nstatus: todo\n---\n",
    );
    w("t/beta.md", "---\ndue: 2026-10-03\nstatus: todo\n---\n");
    w("t/gamma.md", "---\nstatus: done\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

const TODAY: (&str, &str) = ("MDGRID_TODAY", "2026-10-06");

#[test]
fn test_ga_1_c_opens_gantt_and_enter_returns_e2e() {
    // [GA-1] gantt = { start, end } の表で C → ガントの画面(ヘッダーに「ガント ▾」)。
    // [GA-2] alpha の行は `█` が7つ、beta の行は `◆`、今日の列に `┊`。
    // [GA-4] Enter で表に戻る。q で終わる(終了コード 0)。
    let home = root("gantt-e2e", "gantt = { start = \"start\", end = \"due\" }");
    let mut tui = Tui::spawn(&["t"], home.path(), &[TODAY]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾")
    });
    tui.send("C");
    tui.wait_until("ガントの画面", |s| {
        s.contains("ガント ▾") && s.contains('█')
    });
    let s = tui.screen();
    let alpha = s
        .lines()
        .find(|l| l.contains("alpha"))
        .unwrap_or_else(|| panic!("alpha の行:\n{s}"));
    assert_eq!(alpha.chars().filter(|c| *c == '█').count(), 7, "{s}");
    assert!(
        s.lines().any(|l| l.contains("beta") && l.contains('◆')),
        "{s}"
    );
    assert!(s.contains('┊'), "今日の列の印:\n{s}");
    tui.send("\r");
    tui.wait_until("表に戻る", |s| s.contains("▦ 表 ▾"));
    assert!(!tui.screen().contains("ガント ▾"));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_ga_1_missing_column_warns_e2e() {
    // [GA-1] start の列が表に無い → 警告1行(CLI-3 の形)で、C はガントを開かない。
    let home = root(
        "gantt-e2e-col",
        "gantt = { start = \"none\", end = \"due\" }",
    );
    let mut tui = Tui::spawn(&["t"], home.path(), &[TODAY]);
    tui.wait_until("警告", |s| {
        s.contains("table[1].gantt") && s.contains("none")
    });
    tui.send_settle("C");
    let s = tui.screen();
    assert!(s.contains("▦ 表 ▾") && !s.contains("ガント ▾"), "{s}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
