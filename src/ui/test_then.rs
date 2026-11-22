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
