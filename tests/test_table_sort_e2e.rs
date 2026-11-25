#![cfg(unix)]
//! 表の既定の並べ替え(印の `[[table]]` の `sort`。base-view.table-sort ほか)と、見出しで並べたときに既定を最後の決め手に
//! すること(navigation.sort-default-tiebreak)の、本物の実行ファイルでの確かめ(端末 80×24 と `--print`)。値は作ったもの。
//! 実装の前に書いた。行の順は、画面の中でノートの名前が現れる行の上下で見る。

use crate::pty;

use pty::{TempDir, Tui};
use std::process::{Command, Stdio};

/// 根の印の [[table]](表はフォルダ t)に `sort = [{ property = "due", direction = "ASC" }]` と `line` を書く。
/// alpha(todo・2026-11-30)・bravo(done・2026-11-10)・charlie(todo・2026-11-05)・delta(done・2026-11-20)。
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
        &format!(
            "[[table]]\nname = \"T\"\npath = \"t\"\nsort = [{{ property = \"due\", direction = \"ASC\" }}]\n{line}\n"
        ),
    );
    w("t/alpha.md", "---\nstatus: todo\ndue: 2026-11-30\n---\n");
    w("t/bravo.md", "---\nstatus: done\ndue: 2026-11-10\n---\n");
    w("t/charlie.md", "---\nstatus: todo\ndue: 2026-11-05\n---\n");
    w("t/delta.md", "---\nstatus: done\ndue: 2026-11-20\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

/// 画面の中で `names` が上からこの順に並んでいるか(どれも1回は現れること)。
fn in_order(s: &str, names: &[&str]) -> bool {
    let rows: Vec<Option<usize>> = names
        .iter()
        .map(|n| s.lines().position(|l| l.contains(n)))
        .collect();
    rows.iter().all(Option::is_some) && rows.windows(2).all(|w| w[0] < w[1])
}

#[test]
fn test_table_sort_screen_and_header_tiebreak_e2e() {
    // [stest -> req~base-view.table-sort~1, req~navigation.sort-default-tiebreak~1] 印の sort = due の昇順のビューの無い表 →
    // charlie・bravo・delta・alpha(due の順)。status の列で s → bravo・delta・charlie・alpha(status の順、同じ status の中は due の順)。
    let home = root("tsort-e2e", "");
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が due の順に出る", |s| {
        s.contains("4行") && in_order(s, &["charlie", "bravo", "delta", "alpha"])
    });
    tui.send("s");
    tui.wait_until(
        "status で並べ、同じ status の中は due の順",
        |s| {
            s.contains("status で昇順に並べた")
                && in_order(s, &["bravo", "delta", "charlie", "alpha"])
        },
    );
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_table_sort_kanban_e2e() {
    // [stest -> req~base-view.table-sort-outputs~1] 同じ表に kanban = status → B のカンバンの todo のレーンは charlie が alpha の上、
    // done のレーンは bravo が delta の上。
    let home = root("tsort-kanban-e2e", "kanban = { column = \"status\" }");
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[]);
    tui.wait_until("表が出る", |s| s.contains("4行"));
    tui.send("B");
    tui.wait_until("カンバンの札が due の順", |s| {
        s.contains("カンバン")
            && in_order(s, &["charlie", "alpha"])
            && in_order(s, &["bravo", "delta"])
    });
    tui.send("q");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_table_sort_print_e2e() {
    // [stest -> req~base-view.table-sort-outputs~1] 同じ表を `--print --format csv` → データの行は charlie・bravo・delta・alpha の順。
    let home = root("tsort-print-e2e", "");
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .current_dir(home.path())
        .args(["t", "--print", "--format", "csv", "--with-path"])
        .env("XDG_CONFIG_HOME", home.path().join("cfg"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(
        in_order(&out, &["charlie", "bravo", "delta", "alpha"]),
        "{out}"
    );
}
