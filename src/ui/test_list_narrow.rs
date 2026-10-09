//! [CE-3] 自由入力で打っている間は、打った文字を含む候補を選んでいない形で出し続け、↑↓ で選べる。
//! [SR-24] 操作の一覧が窓に入らずに流れるときは、下の縁に「選んでいる番号/項目の数」を出す。
//! specs/_changes/2026-10-06-list-narrow.md。

use super::test_action_menu::shot;
use super::test_screen::{ch, col_named, ctrl, make, press, screen, typing};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: blocked\n---\n"),
    ("b.md", "---\nstatus: doing\n---\n"),
    ("c.md", "---\nstatus: Done\n---\n"),
    ("d.md", "---\nstatus: waiting\n---\n"),
];

fn pending(a: &App) -> Option<NewValue> {
    a.changes.pending(&a.rows[0], "status").cloned()
}

fn open(name: &str) -> (super::test_screen::Tmp, App) {
    let (t, mut a) = make(name, NOTES);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    (t, a)
}

#[test]
fn test_ce_3_narrow_shows_matches_without_selection() {
    let (_t, mut a) = open("ce3n_show");
    typing(&mut a, "do");
    let s = screen(&a);
    assert!(s.contains("│  doing"), "{s}");
    assert!(s.contains("│  Done"), "大文字小文字を問わない: {s}");
    assert!(!s.contains("│  waiting"), "{s}");
    assert!(!s.contains("│>"), "選んでいない形: {s}");
    assert!(a.active_list().is_none());
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a), Some(NewValue::Str("do".into())));
}

#[test]
fn test_ce_3_narrow_down_picks_match() {
    let (_t, mut a) = open("ce3n_pick");
    typing(&mut a, "do");
    press(&mut a, KeyCode::Down);
    assert!(a.active_list().is_some());
    let s = screen(&a);
    assert!(s.contains("│> doing"), "{s}");
    assert!(!s.contains("│  waiting"), "選ぶときも絞ったまま: {s}");
    press(&mut a, KeyCode::Down);
    assert!(screen(&a).contains("│> Done"));
    press(&mut a, KeyCode::Down);
    assert!(screen(&a).contains("│> Done"), "絞った候補の端で止まる");
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a), Some(NewValue::Str("doing".into())));
}

#[test]
fn test_ce_3_narrow_no_match_hides_and_revert_restores() {
    let (_t, mut a) = open("ce3n_none");
    typing(&mut a, "zz");
    let s = screen(&a);
    assert!(!s.contains("│  doing"), "{s}");
    // ↓ で全部のリストに戻る(当たる候補が無いので絞らない)。
    press(&mut a, KeyCode::Down);
    assert!(a.active_list().is_some());
    assert!(screen(&a).contains("│  waiting"));
    // 絞ったあと Ctrl+R で全部に戻る。
    typing(&mut a, "do");
    press(&mut a, KeyCode::Down);
    ctrl(&mut a, 'r');
    let s = screen(&a);
    assert!(s.contains("│  waiting"), "{s}");
}

#[test]
fn test_sr_24_menu_position_shown_when_scrolled() {
    let (_t, mut a) = make("sr24pos", NOTES);
    col_named(&mut a, "status");
    ch(&mut a, 'x');
    let s = shot(&mut a, 80, 24);
    assert!(s.text.lines().any(|l| l.contains("╰1/")), "{s}");
    press(&mut a, KeyCode::Down);
    let s = shot(&mut a, 80, 24);
    assert!(s.text.lines().any(|l| l.contains("╰2/")), "{s}");
}

#[test]
fn test_sr_24_menu_position_hidden_when_all_fit() {
    let (_t, mut a) = make("sr24all", NOTES);
    col_named(&mut a, "status");
    ch(&mut a, 'x');
    let s = shot(&mut a, 100, 60);
    assert!(!s.text.lines().any(|l| l.contains("╰1/")), "{s}");
}

#[test]
fn test_ce_3_narrow_up_picks_last_and_typing_replaces() {
    let (_t, mut a) = open("ce3n_up");
    typing(&mut a, "do");
    assert!(screen(&a).contains("(↑↓ で当たる候補を選ぶ)"));
    press(&mut a, KeyCode::Up);
    assert!(screen(&a).contains("│> Done"));
    // 選び直したあとに打つと、打った文字から始める。
    typing(&mut a, "x");
    assert_eq!(a.input.as_ref().unwrap().text, "x");
}
