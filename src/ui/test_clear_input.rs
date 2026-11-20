//! [cell-edit.inline][screen.key-table] 編集のモードの Ctrl+U で入力の文字を全部消す(specs/_history/decidespec/_changes/2026-10-06-clear-input.md)。

use super::keymap::Action;
use super::test_screen::{ctrl, make, press, typing};
use ratatui::crossterm::event::KeyCode;

// [utest -> req~cell-edit.inline-enter-esc~1, req~cell-edit.inline-click-invalid~1]
#[test]
fn test_ce_1_clear_input_empties_the_box() {
    let (_t, mut a) = make("clear_in", &[("a.md", "---\ntitle: a long title\n---\n")]);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::End);
    ctrl(&mut a, 'u');
    let i = a.input.as_ref().expect("入力が開いたまま");
    assert_eq!(i.text, "");
    assert_eq!(i.cursor, 0);
    typing(&mut a, "new");
    assert_eq!(a.input.as_ref().unwrap().text, "new");
    // Ctrl+R で編集前に戻せる。
    ctrl(&mut a, 'r');
    assert_eq!(a.input.as_ref().unwrap().text, "a long title");
}

// [utest -> req~cell-edit.inline-input~1, req~cell-edit.inline-enter-esc~1, req~cell-edit.inline-pending~1, req~cell-edit.inline-click-commit~1, req~cell-edit.inline-click-invalid~1]
#[test]
fn test_ce_1_clear_input_has_a_name() {
    assert_eq!(Action::by_name("clear_input"), Some(Action::ClearInput));
}
