//! [NV-1] `/` で一致の無い語を Enter で確定しても、「見つからない: 語」がメッセージ行に残る
//! (specs/_changes/2026-10-06-english-wording.md)。

use super::keymap::Mode;
use super::test_screen::{ch, make, press, typing};
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_nv_1_not_found_stays_after_enter() {
    let (_t, mut a) = make("nv1_nf", &[("a.md", "---\nstatus: todo\n---\n")]);
    ch(&mut a, '/');
    typing(&mut a, "zzz");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("見つからない") && m.contains("zzz"),
        "確定のあとも残る: {m:?}"
    );
}

#[test]
fn test_nv_1_found_has_no_not_found() {
    let (_t, mut a) = make("nv1_f", &[("a.md", "---\nstatus: todo\n---\n")]);
    ch(&mut a, '/');
    typing(&mut a, "todo");
    press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("見つからない"), "{m:?}");
}
