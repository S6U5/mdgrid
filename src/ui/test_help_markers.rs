//! [SR-5][SR-23] ヘルプの最後に、セルと行の印の意味を並べる(決定 specs/_decisions/2026-10-06-help-markers.md)。

use super::keymap::Mode;
use super::test_screen::*;
use mdgrid::i18n::{scoped, Lang};
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[("a.md", "---\ntitle: x\n---\n")];

/// `?` でヘルプを開き、末尾(G)の画面。
fn help_end(name: &str) -> String {
    let (_t, mut a) = make(name, NOTES);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    press(&mut a, KeyCode::Char('G'));
    screen(&a)
}

#[test]
fn test_sr_5_markers_section_at_end_ja() {
    let s = help_end("sr5marks_ja");
    for want in [
        " 印",
        "セルの中",
        "∅",
        "null(キーはあり、値が無い)",
        "列の型に合わない値",
        "読むだけ(選ぶと理由が出る)",
        "行の左",
        "印を付けた行(Space・v)",
        "直して元の位置に留めている行",
    ] {
        assert!(s.contains(want), "{want:?} が無い:\n{s}");
    }
    // 最後の行(下の帯の上)は行の印の最後。
    let body: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    let last = body[body.len().saturating_sub(2)];
    assert!(last.contains('~'), "最後は ~ の行: {last:?}\n{s}");
}

#[test]
fn test_sr_5_markers_section_at_end_en() {
    let _g = scoped(Lang::En);
    let s = help_end("sr5marks_en");
    for want in [
        " Markers",
        "In a cell",
        "null: the key is there, with no value",
        "the value does not fit the column type",
        "changed, not saved yet",
        "Left of a row",
        "changed outside mdgrid, or a sync conflict file",
    ] {
        assert!(s.contains(want), "{want:?} missing:\n{s}");
    }
}
