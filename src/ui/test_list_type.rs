//! [CE-3] リストを開いた直後(選び直したあとも)に文字を打つと、今の値を消して打った文字から自由入力を始める
//! (決定 specs/_decisions/2026-10-06-list-type-replaces.md)。

use super::test_screen::{ctrl, make, press, typing, Tmp};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

fn three() -> (Tmp, App) {
    make(
        "ce3_type",
        &[
            ("a.md", "---\nstatus: doing\n---\n"),
            ("b.md", "---\nstatus: todo\n---\n"),
            ("c.md", "---\nstatus: done\n---\n"),
        ],
    )
}

/// 最初の行の status のセルで Enter(最初に選ばれている列は status)。
fn open(a: &mut App) {
    press(a, KeyCode::Enter);
    assert!(a.active_list().is_some(), "リストが出る");
}

fn pending(a: &App) -> Option<NewValue> {
    let r = &a.rows[0];
    a.changes.pending(r, "status").cloned()
}

#[test]
fn test_ce_3_type_replaces_current_value() {
    let (_t, mut a) = three();
    open(&mut a);
    typing(&mut a, "blocked");
    assert_eq!(a.input.as_ref().unwrap().text, "blocked");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a), Some(NewValue::Str("blocked".into())));
}

#[test]
fn test_ce_3_type_replaces_after_moving_in_list() {
    let (_t, mut a) = three();
    open(&mut a);
    press(&mut a, KeyCode::Down);
    typing(&mut a, "x");
    assert_eq!(a.input.as_ref().unwrap().text, "x");
}

#[test]
fn test_ce_3_type_continues_after_cursor_key() {
    let (_t, mut a) = three();
    open(&mut a);
    press(&mut a, KeyCode::Left);
    press(&mut a, KeyCode::End);
    typing(&mut a, "!");
    assert_eq!(a.input.as_ref().unwrap().text, "doing!");
}

#[test]
fn test_ce_3_type_continues_after_revert() {
    let (_t, mut a) = three();
    open(&mut a);
    typing(&mut a, "zz");
    ctrl(&mut a, 'r');
    assert_eq!(a.input.as_ref().unwrap().text, "doing");
    typing(&mut a, "-x");
    assert_eq!(a.input.as_ref().unwrap().text, "doing-x");
}
