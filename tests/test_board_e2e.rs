#![cfg(unix)]
//! ボードの画面(BD-1・BD-2・BD-4)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。
//! 記録: specs/_changes/2026-11-13-board-view.md(タスク #2・#3)。実装を見ずに、仕様(specs/board/spec.md)と
//! 記録の「設計」の既定のキー(`B`・`L`・`h`・`k`・Ctrl+S)と画面の文言(見出し「値 数」・理由の文)だけから書いた。

mod pty;

use pty::{wait_file, TempDir, Tui};
use std::path::PathBuf;

/// 保管庫 `<home>/tasks` にノートを置く。`config` があれば既定の置き場の設定に書く。
fn home(tag: &str, notes: &[(&str, &str)], config: Option<&str>) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    for d in ["tasks", "config/mdgrid", "state"] {
        std::fs::create_dir_all(h.join(d)).unwrap();
    }
    for (name, text) in notes {
        std::fs::write(h.join("tasks").join(name), text).unwrap();
    }
    if let Some(c) = config {
        std::fs::write(h.join("config/mdgrid/config.toml"), c).unwrap();
    }
    home
}

fn note(home: &TempDir, name: &str) -> PathBuf {
    home.path().join("tasks").join(name)
}

/// 列は status だけ(カーソルは status の列): todo が2行、doing と done が1行ずつ。
const TASKS: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: todo\n---\nbody\n"),
    ("beta.md", "---\nstatus: doing\n---\nbody\n"),
    ("gamma.md", "---\nstatus: done\n---\nbody\n"),
    ("delta.md", "---\nstatus: todo\n---\nbody\n"),
];

/// 見出し「値 数」(値のあとに空白を挟んで、ちょうどその数)がある行の桁。無ければ None。
fn lane_at(s: &str, value: &str, count: &str) -> Option<usize> {
    s.lines().find_map(|l| {
        l.match_indices(value).find_map(|(i, _)| {
            let before_ok = !l[..i].ends_with(|c: char| c.is_alphanumeric());
            let rest = &l[i + value.len()..];
            let t = rest.trim_start();
            (before_ok
                && t.len() < rest.len()
                && t.starts_with(count)
                && !t[count.len()..].starts_with(|c: char| c.is_ascii_digit()))
            .then(|| l[..i].chars().count())
        })
    })
}

#[test]
fn test_bd_1_board_shows_lanes_and_back_e2e() {
    // [BD-1] status の列にカーソルを置いて B → レーンに4枚のカード。Esc → 表に戻る。
    // [BD-2] 見出しは「値 数」。決まりが無いので doing・done・todo の文字の順に左から並ぶ。
    let home = home("board-e2e", TASKS, None);
    let mut tui = Tui::spawn(&["tasks"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("delta") && s.contains("▦ 表 ▾")
    });
    tui.send("B");
    tui.wait_until("ボードの画面", |s| {
        lane_at(s, "todo", "2").is_some()
            && lane_at(s, "doing", "1").is_some()
            && lane_at(s, "done", "1").is_some()
    });
    let s = tui.screen();
    assert!(!s.contains("▦ 表 ▾"), "画面の型のボタンは表でない:\n{s}");
    for name in ["alpha", "beta", "gamma", "delta"] {
        assert!(s.contains(name), "カード {name}:\n{s}");
    }
    let xs = [
        lane_at(&s, "doing", "1").unwrap(),
        lane_at(&s, "done", "1").unwrap(),
        lane_at(&s, "todo", "2").unwrap(),
    ];
    assert!(xs[0] < xs[1] && xs[1] < xs[2], "文字の順: {xs:?}\n{s}");
    // 80 桁の端末で、どの行も幅を超えない(折り返しで行が増えない)。
    assert!(s.lines().all(|l| l.chars().count() <= 80), "{s}");
    tui.send("\x1b");
    tui.wait_until("表に戻る", |s| {
        s.contains("▦ 表 ▾") && lane_at(s, "todo", "2").is_none()
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_bd_4_move_card_then_save_writes_file_e2e() {
    // [BD-4] todo のカードで L → doing へ移って「未保存 1」、ファイルは変わらない。
    // Ctrl+S → 保存の確認に差分 → 確定でファイルの値が変わる。
    // [BD-2] 決まりの values の順(todo・doing・done)でレーンが並ぶ。
    let home = home(
        "board-e2e-move",
        TASKS,
        Some("[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n"),
    );
    let before = std::fs::read_to_string(note(&home, "alpha.md")).unwrap();
    let mut tui = Tui::spawn(&["tasks"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("delta")
    });
    tui.send("B");
    tui.wait_until("ボードの画面", |s| {
        lane_at(s, "todo", "2").is_some() && lane_at(s, "doing", "1").is_some()
    });
    let s = tui.screen();
    let xs = [
        lane_at(&s, "todo", "2").unwrap(),
        lane_at(&s, "doing", "1").unwrap(),
        lane_at(&s, "done", "1").unwrap(),
    ];
    assert!(xs[0] < xs[1] && xs[1] < xs[2], "決まりの順: {xs:?}\n{s}");
    // 左の端のレーン(todo)の先頭のカードへ(どのカードを選んで開いても同じ所に着く)。
    tui.send_settle("hhhkkk");
    tui.send("L");
    tui.wait_until("doing へ移る", |s| {
        lane_at(s, "todo", "1").is_some()
            && lane_at(s, "doing", "2").is_some()
            && s.contains("未保存 1")
    });
    // ためただけで、ファイルは変わらない。
    assert_eq!(
        std::fs::read_to_string(note(&home, "alpha.md")).unwrap(),
        before
    );
    tui.send("\x13"); // Ctrl+S
    tui.wait_until("保存の確認に差分", |s| {
        s.contains("-status: todo") && s.contains("+status: doing")
    });
    tui.send("\r");
    // todo の2枚(alpha・delta)のうち、移した1枚だけが doing になる。
    let moved = |name: &str| {
        std::fs::read_to_string(note(&home, name))
            .unwrap()
            .contains("status: doing")
    };
    wait_file(&note(&home, "alpha.md"), &tui, |_| {
        moved("alpha.md") || moved("delta.md")
    });
    assert!(moved("alpha.md") != moved("delta.md"), "1枚だけ移る");
    let changed = if moved("alpha.md") {
        "alpha.md"
    } else {
        "delta.md"
    };
    assert_eq!(
        std::fs::read_to_string(note(&home, changed)).unwrap(),
        "---\nstatus: doing\n---\nbody\n"
    );
    for (name, text) in [("beta.md", TASKS[1].1), ("gamma.md", TASKS[2].1)] {
        assert_eq!(std::fs::read_to_string(note(&home, name)).unwrap(), text);
    }
    tui.wait_until("未保存が無くなる", |s| s.contains("未保存 0"));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_bd_1_number_column_does_not_switch_e2e() {
    // [BD-1] 数の列 estimate で B → 切り替わらず、理由が出る。
    let home = home(
        "board-e2e-num",
        &[
            ("alpha.md", "---\nestimate: 3\n---\n"),
            ("beta.md", "---\nestimate: 5\n---\n"),
        ],
        None,
    );
    let mut tui = Tui::spawn(&["tasks"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("estimate") && s.contains("▦ 表 ▾")
    });
    tui.send("B");
    tui.wait_until("理由が出る", |s| {
        s.contains("この列ではボードにできない")
    });
    let s = tui.screen();
    assert!(s.contains("▦ 表 ▾"), "表のまま:\n{s}");
    assert!(s.contains("estimate"), "表のまま:\n{s}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
