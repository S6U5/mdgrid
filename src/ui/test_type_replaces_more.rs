//! [CE-5][CE-7] 日付と数の入力でも、開いた直後に打つと今の値を置き換える
//! (決定 specs/_decisions/2026-10-06-type-replaces-date-number.md)。

use super::test_screen::{col_named, make, press, typing};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

fn pending(a: &App, col: &str) -> Option<NewValue> {
    a.changes.pending(&a.rows[0], col).cloned()
}

#[test]
fn test_ce_5_type_replaces_plus_days() {
    let (_t, mut a) = make(
        "ce5_rep",
        &[
            ("a.md", "---\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ndue: 2026-10-01\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "+3");
    assert_eq!(a.input.as_ref().unwrap().text, "+3");
    press(&mut a, KeyCode::Enter);
    assert!(
        matches!(pending(&a, "due"), Some(NewValue::Date(_))),
        "{:?} {:?}",
        pending(&a, "due"),
        a.message
    );
}

#[test]
fn test_ce_5_type_replaces_continues_after_backspace() {
    let (_t, mut a) = make("ce5_left", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::End);
    press(&mut a, KeyCode::Backspace);
    typing(&mut a, "6");
    assert_eq!(a.input.as_ref().unwrap().text, "2026-10-06");
}

#[test]
fn test_ce_7_type_replaces_number() {
    let (_t, mut a) = make(
        "ce7_rep",
        &[
            ("a.md", "---\npriority: 1\n---\n"),
            ("b.md", "---\npriority: 2\n---\n"),
        ],
    );
    col_named(&mut a, "priority");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "5");
    assert_eq!(a.input.as_ref().unwrap().text, "5");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, "priority"), Some(NewValue::Int(5)));
}
