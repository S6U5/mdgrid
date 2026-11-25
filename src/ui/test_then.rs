//! [cell-edit.then] `[rules.<列>.then]`: 列をある値に直したら、同じ行の別の列にも値をためる。

use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{col_named, edit_cell, press, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use mdgrid::writeback::NewValue;
use ratatui::crossterm::event::KeyCode;

const THEN: &str = "[rules.status.then]\ndone = { done_at = \"{date}\" }\n";

/// 設定 `toml` で開いた表(a.md と b.md は `status: todo`、c.md は `status: done` と `done_at: 2026-10-01`)。
/// 今日は 2026-10-06。
fn app_with(tag: &str, toml: &str) -> (Tmp, App, Vec<String>) {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\nstatus: todo\ndone_at:\n---\n");
    tmp.write("b.md", "---\nstatus: todo\ndone_at:\n---\n");
    tmp.write("c.md", "---\nstatus: done\ndone_at: 2026-10-01\n---\n");
    let (cfg, warnings) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.today = types::parse_date("2026-10-06").unwrap();
    (tmp, a, warnings)
}

/// ためた変更を(ノートの名前の順に)「列=値」で並べる。
fn pending(a: &App) -> Vec<String> {
    let mut out = Vec::new();
    for row in a.changes.rows() {
        for col in ["status", "done_at"] {
            match a.changes.pending(&row, col) {
                Some(NewValue::Str(s) | NewValue::Date(s)) => out.push(format!("{col}={s}")),
                Some(v) => out.push(format!("{col}={v:?}")),
                None => {}
            }
        }
    }
    out
}

// [utest -> req~cell-edit.then-set~2]
#[test]
fn test_then_sets_the_other_column_when_the_value_matches() {
    // status を done に直すと done_at に今日がたまり、doing に直した行には何もたまらない。
    let (_t, mut a, w) = app_with("then-set", THEN);
    assert!(w.is_empty(), "{w:?}");
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 1, "status", "doing");
    assert_eq!(
        pending(&a),
        ["status=done", "done_at=2026-10-06", "status=doing"]
    );
}

// [utest -> req~cell-edit.then-undo~1]
#[test]
fn test_then_is_undone_in_one_step_with_the_edit() {
    // 一緒に入れた値は、元の直しと同じ1手で取り消せる。
    let (_t, mut a, _) = app_with("then-undo", THEN);
    edit_cell(&mut a, 0, "status", "done");
    assert_eq!(pending(&a), ["status=done", "done_at=2026-10-06"]);
    a.apply(Action::Undo);
    assert_eq!(pending(&a), Vec::<String>::new());
}

// [utest -> req~cell-edit.then-unset~2]
#[test]
fn test_then_value_is_dropped_when_the_edit_moves_to_another_value() {
    // done に直してから doing に直すと、done_at にためた値は外れる。元の値に戻したときも外れる。
    let (_t, mut a, _) = app_with("then-unset", THEN);
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 0, "status", "doing");
    assert_eq!(pending(&a), ["status=doing"]);
    edit_cell(&mut a, 1, "status", "done");
    edit_cell(&mut a, 1, "status", "todo");
    assert_eq!(pending(&a), ["status=doing"]);
}

// [utest -> req~cell-edit.then-bulk~1]
#[test]
fn test_then_applies_to_every_row_of_a_bulk_edit() {
    // 印を付けた2行をまとめて done に直すと、2行とも done_at に今日がたまる。
    let (_t, mut a, _) = app_with("then-bulk", THEN);
    a.row = 0;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Char(' '));
    press(&mut a, KeyCode::Char(' '));
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    a.input.as_mut().unwrap().text.clear();
    a.input.as_mut().unwrap().cursor = 0;
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a),
        [
            "status=done",
            "done_at=2026-10-06",
            "status=done",
            "done_at=2026-10-06"
        ],
        "{:?}",
        a.message
    );
}

// [utest -> req~cell-edit.then-clear~1]
#[test]
fn test_then_empty_value_clears_the_other_column() {
    // 空の文字の値は、その列を空にする。
    let tmp_toml = "[rules.status.then]\ntodo = { done_at = \"\" }\n";
    let (_t, mut a, _) = app_with("then-clear", tmp_toml);
    edit_cell(&mut a, 2, "status", "todo");
    assert_eq!(pending(&a), ["status=todo", "done_at=Null"]);
}

// [utest -> req~cell-edit.then-invalid~1]
#[test]
fn test_then_of_a_wrong_shape_warns_and_is_not_used() {
    // 表でない then は、場所を示す警告1行で、値を連れて入れない。
    let (_t, mut a, w) = app_with("then-bad", "[rules.status]\nthen = \"done\"\n");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].starts_with("config.toml: rules.status.then:"), "{w:?}");
    edit_cell(&mut a, 0, "status", "done");
    assert_eq!(pending(&a), ["status=done"]);
}

// ---- cell-edit.then-if-empty: 入れる先の列が空のときだけ入れる ----

/// started が空のときだけ今日を入れる設定。
const IF_EMPTY: &str =
    "[rules.status.then]\nreading = { started = { value = \"{date}\", if_empty = true } }\n";

/// 設定 `toml` で、p.md(`status: paused`・`started: 2026-09-01`)と t.md(`status: todo`。started の無い行)を開く。
/// 行は p・t の順。今日は 2026-10-06。
fn reading_app(tag: &str, toml: &str) -> (Tmp, App, Vec<String>) {
    let tmp = Tmp::new(tag);
    tmp.write("p.md", "---\nstatus: paused\nstarted: 2026-09-01\n---\n");
    tmp.write("t.md", "---\nstatus: todo\n---\n");
    let (cfg, warnings) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.today = types::parse_date("2026-10-06").unwrap();
    (tmp, a, warnings)
}

/// ためた変更を、行の名前ごとに「名前: 列=値」で並べる(列は `cols` の順)。
fn pending_in(a: &App, cols: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for row in a.changes.rows() {
        let label = a.src.label(&row);
        let name = label.trim_end_matches(".md");
        for col in cols {
            match a.changes.pending(&row, col) {
                Some(NewValue::Str(s) | NewValue::Date(s)) => {
                    out.push(format!("{name}: {col}={s}"));
                }
                Some(v) => out.push(format!("{name}: {col}={v:?}")),
                None => {}
            }
        }
    }
    out
}

/// 行 `row` の名前(行の並びの前提を確かめる)。
fn row_name(a: &App, row: usize) -> String {
    a.src
        .label(&a.rows[row])
        .trim_end_matches(".md")
        .to_string()
}

// [utest -> req~cell-edit.then-if-empty~1]
#[test]
fn test_then_if_empty_fills_only_an_empty_column() {
    // status を reading に直すと、started のある p は status だけ、started の無い t は started に今日もたまる。
    let (_t, mut a, w) = reading_app("then-ifempty-set", IF_EMPTY);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!((row_name(&a, 0), row_name(&a, 1)), ("p".into(), "t".into()));
    edit_cell(&mut a, 0, "status", "reading");
    edit_cell(&mut a, 1, "status", "reading");
    assert_eq!(
        pending_in(&a, &["status", "started"]),
        [
            "p: status=reading",
            "t: status=reading",
            "t: started=2026-10-06"
        ]
    );
}

// [utest -> req~cell-edit.then-if-empty~1]
#[test]
fn test_then_if_empty_applies_per_row_in_a_bulk_edit() {
    // 2行に印を付けて status をまとめて reading に直すと、空の t にだけ started がたまる。
    let (_t, mut a, _) = reading_app("then-ifempty-bulk", IF_EMPTY);
    a.row = 0;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Char(' '));
    press(&mut a, KeyCode::Char(' '));
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    a.input.as_mut().unwrap().text.clear();
    a.input.as_mut().unwrap().cursor = 0;
    typing(&mut a, "reading");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending_in(&a, &["status", "started"]),
        [
            "p: status=reading",
            "t: status=reading",
            "t: started=2026-10-06"
        ],
        "{:?}",
        a.message
    );
}

// [utest -> req~cell-edit.then-if-empty~1]
#[test]
fn test_then_if_empty_keeps_a_value_typed_by_hand() {
    // t の started に手で 2026-10-01 を入れてから、status を reading → paused に直しても、手で入れた started は残る。
    let (_t, mut a, _) = reading_app("then-ifempty-hand", IF_EMPTY);
    edit_cell(&mut a, 1, "started", "2026-10-01");
    edit_cell(&mut a, 1, "status", "reading");
    assert_eq!(
        pending_in(&a, &["status", "started"]),
        ["t: status=reading", "t: started=2026-10-01"]
    );
    edit_cell(&mut a, 1, "status", "paused");
    assert_eq!(
        pending_in(&a, &["status", "started"]),
        ["t: status=paused", "t: started=2026-10-01"]
    );
}

// [utest -> req~cell-edit.then-if-empty~1]
#[test]
fn test_then_if_empty_value_is_dropped_when_the_edit_moves_on() {
    // started の無い t を reading にしてから paused に直すと、一緒にためた started は外れる(cell-edit.then-unset と同じ)。
    let (_t, mut a, _) = reading_app("then-ifempty-unset", IF_EMPTY);
    edit_cell(&mut a, 1, "status", "reading");
    assert_eq!(
        pending_in(&a, &["status", "started"]),
        ["t: status=reading", "t: started=2026-10-06"]
    );
    edit_cell(&mut a, 1, "status", "paused");
    assert_eq!(pending_in(&a, &["status", "started"]), ["t: status=paused"]);
}

// [utest -> req~cell-edit.then-if-empty-invalid~1]
#[test]
fn test_then_table_with_a_misspelled_item_warns_and_skips_that_column() {
    // if_empty の書き誤り(if_emtpy)は、その道筋の警告1行で、その列には入れない(ほかの列には入れる)。
    let toml = "[rules.status.then]\ndone = { done_at = { value = \"{date}\", if_emtpy = true }, note = \"済\" }\n";
    let (_t, mut a, w) = app_with("then-ifempty-typo", toml);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].starts_with("config.toml: rules.status.then.done.done_at.if_emtpy:"),
        "{w:?}"
    );
    edit_cell(&mut a, 0, "status", "done");
    let row = a.changes.rows().into_iter().next().unwrap();
    assert_eq!(a.changes.pending(&row, "done_at"), None);
    assert_eq!(
        a.changes.pending(&row, "note"),
        Some(&NewValue::Str("済".into()))
    );
    assert_eq!(pending(&a), ["status=done"]);
}

// [utest -> req~cell-edit.then-if-empty-invalid~1]
#[test]
fn test_then_table_without_value_or_with_a_wrong_flag_warns_and_skips() {
    // value の無い表と、if_empty が真偽でない表は、その道筋の警告1行で、done_at に入れない。
    let cases = [
        (
            "then-ifempty-novalue",
            "[rules.status.then]\ndone = { done_at = { if_empty = true } }\n",
            "config.toml: rules.status.then.done.done_at:",
        ),
        (
            "then-ifempty-flag",
            "[rules.status.then]\ndone = { done_at = { value = \"{date}\", if_empty = \"yes\" } }\n",
            "config.toml: rules.status.then.done.done_at.if_empty:",
        ),
    ];
    for (tag, toml, head) in cases {
        let (_t, mut a, w) = app_with(tag, toml);
        assert_eq!(w.len(), 1, "{tag}: {w:?}");
        assert!(w[0].starts_with(head), "{tag}: {w:?}");
        edit_cell(&mut a, 0, "status", "done");
        assert_eq!(pending(&a), ["status=done"], "{tag}");
    }
}
