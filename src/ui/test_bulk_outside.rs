//! [CE-10] 印を付けた行の外で編集を始めたら、この行だけを直すことと、入らない印の行の数を知らせる
//! (specs/_changes/2026-10-06-bulk-outside-note.md)。印の行の上で始めたときは今までどおり一括。

use super::keymap::Mode;
use super::test_screen::{make, press, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;

fn four() -> (Tmp, App) {
    make(
        "ce10_outside",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: todo\n---\n"),
            ("c.md", "---\nstatus: todo\n---\n"),
            ("d.md", "---\nstatus: doing\n---\n"),
        ],
    )
}

/// 行1と行3に印を付け、行4に立つ(Space は印を付けて下へ動く)。
fn mark_1_and_3(a: &mut App) {
    press(a, KeyCode::Char(' '));
    press(a, KeyCode::Down);
    press(a, KeyCode::Char(' '));
    assert_eq!(a.row, 3, "行4に立つ");
}

#[test]
fn test_ce_10_outside_selection_tells_marked_rows_are_left_out() {
    let (_t, mut a) = four();
    mark_1_and_3(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("この行だけ") && m.contains('2'), "知らせ: {m:?}");
}

#[test]
fn test_ce_10_inside_selection_has_no_outside_note() {
    let (_t, mut a) = four();
    mark_1_and_3(&mut a);
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Up);
    assert_eq!(a.row, 1);
    // 行3(印の行)へ。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("この行だけ"), "印の行の上では知らせない: {m:?}");
}

#[test]
fn test_ce_10_no_marks_no_outside_note() {
    let (_t, mut a) = four();
    press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("この行だけ"), "印が無ければ知らせない: {m:?}");
}
