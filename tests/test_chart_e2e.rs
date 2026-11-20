#![cfg(unix)]
//! グラフ(CH-2・CH-8)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。
//! 行番号や字の位置は決め打ちせず、題「グラフ:」・種類の名前・項目の名前・下の帯の位置(「1/6行」)で見る。

use crate::pty;

use pty::{assert_gone, TempDir, Tui};

/// [[table]] に `line` を書いた根と、その下の表 `t`。status は todo 2件・doing 1件・done 3件。
/// 行は名前の順で alpha・beta・delta・epsilon・gamma・zeta。
fn root(tag: &str, line: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        &format!("[[table]]\nname = \"T\"\npath = \"t\"\n{line}\n"),
    );
    w("t/alpha.md", "---\nstatus: todo\n---\n");
    w("t/beta.md", "---\nstatus: todo\n---\n");
    w("t/gamma.md", "---\nstatus: doing\n---\n");
    w("t/delta.md", "---\nstatus: done\n---\n");
    w("t/epsilon.md", "---\nstatus: done\n---\n");
    w("t/zeta.md", "---\nstatus: done\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

/// グラフの画面: 題「グラフ:」と棒の種類の名前と、3つの項目(done・todo・doing)。
fn is_chart(s: &str) -> bool {
    s.contains("グラフ:")
        && s.contains("棒")
        && ["done", "todo", "doing"].iter().all(|v| s.contains(v))
}

/// 表の画面(ヘッダーの「▦ 表 ▾」があり、グラフの題が無い)。
fn is_table(s: &str) -> bool {
    s.contains("▦ 表 ▾") && !s.contains("グラフ:")
}

#[test]
fn test_ch_2_p_opens_chart_and_esc_returns_e2e() {
    // [stest -> req~chart.open-title~1, req~chart.open-title-inferred~1, req~chart.keys-no-write~1, req~chart.keys-enter~1, req~chart.keys-back~1, req~chart.keys-read-only~1] P → グラフの題と棒の項目(設定なしで status を推定)。Esc で表に戻る。q・P でも表に戻り
    // (q で終わらない)、表で q を押すと終わる(終了コード 0)。
    let home = root("chart-e2e", "");
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾") && s.contains("1/6行")
    });
    for back in ["\x1b", "q", "P"] {
        tui.send("P");
        tui.wait_until("グラフの画面", is_chart);
        tui.send(back);
        tui.wait_until(&format!("{back:?} で表に戻る"), is_table);
        assert_eq!(tui.try_exit(), None, "{back:?} で終わらない");
    }
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_ch_8_enter_filters_to_selected_item_e2e() {
    // [stest -> req~chart.keys-enter~1] chart = { kind = "bar", x = "status" } の表で P → 最初の項目(done。3件)で Enter → 表が done の
    // 3行だけになる(下の帯の「1/3行」)。
    let home = root(
        "chart-e2e-enter",
        "chart = { kind = \"bar\", x = \"status\" }",
    );
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("1/6行")
    });
    tui.send("P");
    tui.wait_until("グラフの画面", is_chart);
    tui.send("\r");
    tui.wait_until("表に戻って done の3行", |s| {
        is_table(s) && s.contains("1/3行") && s.contains("delta") && !s.contains("alpha")
    });
    assert_gone(tui.stop());
}
