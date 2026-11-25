#![cfg(unix)]
//! グラフの実地の操作(2026-11-25)で見つかったことの、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 値の名前とアイコン(chart.value-labels)と、タブを移った直後に開いた画面のキーがその画面に届くこと(台帳の写しで
//! 「`]` → `P` → 2秒後に `]` で表に戻る」と見えた件。カンバン・ガント・木・関係マップも同じ形で見る)。

use crate::pty;

use pty::{TempDir, Tui};
use std::time::Duration;

/// 中身を印に書いた表「台帳」(タブ「全部」と「まとめ」)。status に値の名前とアイコン、カンバン・木・ガント・グラフ
/// (状態ごと・担当と状態)の設定。alpha・beta は todo、gamma は doing、delta は done。
fn root(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    let h = t.path();
    for d in ["t", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(h.join(d)).unwrap();
    }
    let notes = [
        (
            "alpha",
            "todo",
            "alice",
            "a > b",
            "2026-11-02",
            "2026-11-05",
        ),
        ("beta", "todo", "bob", "a > c", "2026-11-03", "2026-11-09"),
        (
            "gamma",
            "doing",
            "alice",
            "a > b",
            "2026-11-04",
            "2026-11-12",
        ),
        ("delta", "done", "bob", "d", "2026-11-06", "2026-11-08"),
    ];
    for (name, status, owner, folder, start, due) in notes {
        std::fs::write(
            h.join(format!("t/{name}.md")),
            format!(
                "---\nstatus: {status}\nowner: {owner}\nfolder: {folder}\nstart: {start}\ndue: {due}\n---\n"
            ),
        )
        .unwrap();
    }
    std::fs::write(
        h.join(".mdgrid/workspace.toml"),
        r##"open = "台帳"

[types]
start = "date"
due = "date"

[rules.status]
values = ["todo", "doing", "done"]
icons = { todo = "#" }
labels = { todo = "未着手", doing = "進行中" }

[[table]]
name = "台帳"
kanban = { column = "status" }
tree = { path = "folder" }
gantt = { start = "start", end = "due" }
chart = [{ name = "状態ごと", x = "status" }, { name = "担当と状態", kind = "heatmap", x = "owner", by = "status" }]

[table.base]
filters = { or = ['file.inFolder("t")'] }

[[table.base.views]]
type = "table"
name = "全部"
order = ["status", "owner"]

[[table.base.views]]
type = "table"
name = "まとめ"
order = ["status", "owner", "folder"]
groupBy = { property = "folder", direction = "ASC" }
"##,
    )
    .unwrap();
    t
}

/// 表の画面(ヘッダーの画面の型のボタンが「表」)。
fn is_table(s: &str) -> bool {
    s.contains("表 ▾") && !s.contains("グラフ:")
}

#[test]
fn test_chart_value_labels_shown_and_enter_filters_by_value_e2e() {
    // [stest -> req~chart.value-labels~1] P → 棒の項目は「# 未着手」・「進行中」・「done」(status の値の todo・doing は出ない)。
    // 最初の「# 未着手」で Enter → 表は todo の alpha・beta だけ。
    let home = root("chart-labels-e2e");
    let mut tui = Tui::spawn(&[], home.path(), &[]);
    tui.wait_until("表が出る", |s| s.contains("alpha") && is_table(s));
    tui.send("P");
    tui.wait_until("値の名前の棒", |s| {
        s.contains("グラフ:")
            && s.contains("# 未着手")
            && s.contains("進行中")
            && s.contains("done")
    });
    let s = tui.screen();
    let body: String = s
        .lines()
        .skip_while(|l| !l.contains("グラフ:"))
        .collect::<Vec<_>>()
        .join("\n");
    for v in ["todo", "doing"] {
        assert!(!body.contains(v), "グラフに値の {v} が残る:\n{s}");
    }
    tui.send("\r");
    tui.wait_until("表に戻って todo の2行", |s| {
        is_table(s)
            && s.contains("alpha")
            && s.contains("beta")
            && !s.contains("gamma")
            && !s.contains("delta")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_chart_next_saved_right_after_tab_switch_e2e() {
    // [stest -> req~chart.keys-charts~1, req~chart.open-first~1] 表で `]`(次のタブ「まとめ」)→ `P`(最初のグラフ「状態ごと」)→ すぐ `]` →
    // 2つ目の「担当と状態」に進み、2秒たってもグラフのまま(表に戻らない)。
    let home = root("chart-next-after-tab-e2e");
    let mut tui = Tui::spawn(&[], home.path(), &[]);
    tui.wait_until("表が出る", |s| s.contains("alpha") && is_table(s));
    tui.send("]");
    tui.wait_until("タブ「まとめ」(folder のまとまり)", |s| {
        is_table(s) && s.contains("件)")
    });
    tui.send("P");
    tui.wait_until("最初のグラフ", |s| {
        s.contains("グラフ:") && s.contains("状態ごと")
    });
    tui.send("]");
    tui.wait_until("2つ目のグラフ", |s| {
        s.contains("グラフ:") && s.contains("担当と状態")
    });
    std::thread::sleep(Duration::from_secs(2));
    let s = tui.screen();
    assert!(
        s.contains("グラフ:") && s.contains("担当と状態") && !is_table(&s),
        "2秒後もグラフのまま:\n{s}"
    );
    tui.send("q");
    tui.wait_until("表に戻る", is_table);
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_screens_keep_keys_right_after_tab_switch_e2e() {
    // [stest -> req~kanban.open-lanes~1, req~gantt.open-bars~1, req~tree.screen-type~1, req~relations.screen-menu-direct-key~1] 表で `]` で
    // タブを移ってすぐ、B(カンバン)・C(ガント)・T(木)・R(関係マップ)で開き、すぐ `j` を押す → 2秒たってもその画面の
    // まま(ヘッダーの画面の型のボタンがその型で、表に戻らない)。Esc で表に戻ってから次の型を見る。
    let home = root("screens-after-tab-e2e");
    let mut tui = Tui::spawn(&[], home.path(), &[]);
    tui.wait_until("表が出る", |s| s.contains("alpha") && is_table(s));
    for (key, name) in [
        ("B", "カンバン"),
        ("C", "ガント"),
        ("T", "木"),
        ("R", "関係マップ"),
    ] {
        tui.send("]");
        std::thread::sleep(Duration::from_millis(200));
        tui.send(key);
        let button = format!("{name} ▾");
        tui.wait_until(&format!("{key} で{name}"), |s| s.contains(&button));
        tui.send("j");
        std::thread::sleep(Duration::from_secs(2));
        let s = tui.screen();
        assert!(
            s.contains(&button) && !is_table(&s),
            "{key}: 2秒後も{name}のまま:\n{s}"
        );
        tui.send("\x1b");
        tui.wait_until(&format!("{name}から表に戻る"), is_table);
    }
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
