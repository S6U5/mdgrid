//! [CE-4] チェックボックスの Enter は真と偽の切り替え。空かキーの無いノートは真に。空にするのは BS・Delete だけ。
//! specs/_changes/2026-10-06-checkbox-two-state.md(決定 specs/_decisions/2026-10-06-checkbox-two-state.md)。

use super::keymap::Mode;
use super::test_screen::{col_named, ctrl, make, press, read};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ndone: false\n---\n"),
    ("b.md", "---\ndone: true\n---\n"),
    ("c.md", "---\ntitle: c\n---\n"),
    ("d.md", "---\ndone:\n---\n"),
];

fn at(a: &App, name: &str) -> Option<NewValue> {
    let i = a.rows.iter().position(|r| r.0.ends_with(name)).unwrap();
    a.changes.pending(&a.rows[i], "done").cloned()
}

fn go(a: &mut App, name: &str) {
    let i = a.rows.iter().position(|r| r.0.ends_with(name)).unwrap();
    a.row = i;
    col_named(a, "done");
}

fn toggle(a: &mut App) {
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_ce_4_two_state_false_and_true() {
    let (_t, mut a) = make("ce4two", NOTES);
    go(&mut a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), None, "元の false に戻った");
    go(&mut a, "b.md");
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), None);
}

#[test]
fn test_ce_4_two_state_keyless_and_empty() {
    let (t, mut a) = make("ce4twoe", NOTES);
    go(&mut a, "c.md");
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(false)), "空に戻らない");
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(true)));
    go(&mut a, "d.md");
    toggle(&mut a);
    assert_eq!(at(&a, "d.md"), Some(NewValue::Bool(true)));
    // 空にするのは BS。
    go(&mut a, "a.md");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Null));
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), "---\ndone:\n---\n");
    assert_eq!(read(&t, "c.md"), "---\ntitle: c\ndone: true\n---\n");
}
