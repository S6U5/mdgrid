//! 設定に書いた列(`[types]`・`[rules]`)は、まだどのノートにもキーが無くても表にある列と数えることの画面の試験
//! (見本なしのスターターから作った台帳で、最初のノートを作ったあとに設定の列が無いと警告していた)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, screen, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use std::path::PathBuf;

const TOP: &str = "[types]\nstart = \"date\"\ndue = \"date\"\nfolder = \"text\"\n\n\
[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n";

const NOTE: &[(&str, &str)] = &[("x.md", "---\ntitle: メモ\n---\n")];

/// 印(根 = 一時フォルダ)の先頭に TOP、[[table]] に `line` を書いて、title だけのノート1件の `notes` を表として開く。
fn boot(tag: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in NOTE {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{TOP}\n[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

#[test]
fn test_gantt_columns_declared_in_types_do_not_warn_when_no_note_has_them() {
    // [utest -> req~gantt.config-declared-column~2] [types] に start・due を書いた表で、start も due も持たないノートが1件 → 警告は無く、C でガントの画面が開く。
    let (_t, mut a) = boot("dc-gantt", "gantt = { start = \"start\", end = \"due\" }");
    assert_eq!(a.message, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
}

#[test]
fn test_gantt_column_in_neither_notes_nor_config_still_warns() {
    // [utest -> req~gantt.config-declared-column~2] 境界: [types] にも [rules] にもノートにも無い列 begin → 今までどおり警告1行で、ガントは開かない。
    let (_t, mut a) = boot(
        "dc-gantt-miss",
        "gantt = { start = \"begin\", end = \"due\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("begin"), "{m}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_kanban_column_declared_in_rules_does_not_warn_when_no_note_has_it() {
    // [utest -> req~kanban.config-declared-column~2] [rules.status] のある表で、status を持たないノートが1件 → 警告は無く、B でカンバンが開き、決まりの値のレーン todo・doing・done が並ぶ。
    let (_t, mut a) = boot("dc-kanban", "kanban = { column = \"status\" }");
    assert_eq!(a.message, None);
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let s = screen(&a);
    for lane in ["todo", "doing", "done"] {
        assert!(s.contains(lane), "レーン {lane}:\n{s}");
    }
}

#[test]
fn test_chart_column_declared_in_rules_does_not_warn_when_no_note_has_it() {
    // [utest -> req~chart.config-declared-column~2] [rules.status] のある表で、status を持たないノートが1件、印のグラフ x = "status" → 警告は無い。x = "none" は今までどおり警告1行。
    let (_t, a) = boot("dc-chart", "chart = [{ name = \"S\", x = \"status\" }]");
    assert_eq!(a.message, None);
    let (_t, a) = boot("dc-chart-miss", "chart = [{ name = \"S\", x = \"none\" }]");
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("none"), "{m}");
}

#[test]
fn test_tree_path_declared_in_types_does_not_warn_when_no_note_has_it() {
    // [utest -> req~tree.config-declared-column~2] [types] に folder を書いた表で、folder を持たないノートが1件、tree = { path = "folder" } → 警告は無い。path = "none" は今までどおり警告1行。
    let (_t, a) = boot("dc-tree", "tree = { path = \"folder\" }");
    assert_eq!(a.message, None);
    let (_t, a) = boot("dc-tree-miss", "tree = { path = \"none\" }");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("table[1].tree.path: ") && m.contains("none"),
        "{m}"
    );
}

#[test]
fn test_kanban_opens_with_empty_lanes_when_there_are_no_rows() {
    // [utest -> req~kanban.open-empty-lanes~2] ノートが1件も無い表で [rules.status] values = todo・doing・done、kanban = { column = "status" } → B でカンバンが開き、札 0 枚のレーン todo・doing・done がこの順で並ぶ。
    let tmp = Tmp::new("dc-kanban-empty");
    std::fs::create_dir_all(tmp.notes()).unwrap();
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{TOP}\n[[table]]\nname = \"T\"\npath = \"notes\"\nkanban = {{ column = \"status\" }}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    assert!(a.rows.is_empty(), "前提: 0件");
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban, "{:?}", a.message);
    let s = screen(&a);
    let at = |w: &str| s.find(w).unwrap_or_else(|| panic!("レーン {w}:\n{s}"));
    assert!(
        at("todo (0)") < at("doing (0)") && at("doing (0)") < at("done (0)"),
        "{s}"
    );
    // レーンの中を動いても、札が無いので何もためない。
    for k in ['j', 'l', 'L', 'H', 'h'] {
        ch(&mut a, k);
    }
    assert_eq!(a.mode, Mode::Kanban);
    assert_eq!(a.changes.count(), 0);
}
