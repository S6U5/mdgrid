//! [CE-28] まだどのノートにも無いキーの列を名前を打って足す。値を入れたノートにだけキーの行を足す。
//! specs/_changes/2026-10-06-add-column.md。

use super::keymap::Mode;
use super::test_screen::{ch, ctrl, make, press, read, typing};
use super::*;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\n---\nbody a\n"),
    ("b.md", "---\nstatus: done\n---\nbody b\n"),
];

fn add(a: &mut App, name: &str) {
    ch(a, 'A');
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(a, name);
    press(a, KeyCode::Enter);
}

#[test]
fn test_ce_28_add_column_and_write_one_note() {
    let (t, mut a) = make("ce28", NOTES);
    add(&mut a, "reviewer");
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.cols.last().map(String::as_str), Some("reviewer"));
    assert_eq!(a.cols[a.col], "reviewer", "足した列を選ぶ");
    // 組み直しても残る。
    a.refresh();
    assert!(a.cols.iter().any(|c| c == "reviewer"));
    // 1行目に値を入れて保存 → そのノートにだけキーの行。
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "ken");
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    let first = a.src.label(&a.rows[0]);
    let (mine, other) = if first == "a.md" {
        ("a.md", "b.md")
    } else {
        ("b.md", "a.md")
    };
    assert!(
        read(&t, mine).contains("reviewer: ken\n"),
        "{}",
        read(&t, mine)
    );
    let orig = NOTES.iter().find(|(n, _)| *n == other).unwrap().1;
    assert_eq!(read(&t, other), orig);
}

#[test]
fn test_ce_28_bad_names_are_refused() {
    let (_t, mut a) = make("ce28bad", NOTES);
    let before = a.cols.clone();
    for name in ["file.x", "formula.y", "   "] {
        add(&mut a, name);
        assert_eq!(a.cols, before, "{name}");
        assert!(a.message.is_some(), "{name}");
        a.message = None;
        if a.mode != Mode::Table {
            press(&mut a, KeyCode::Esc);
        }
    }
}

#[test]
fn test_ce_28_existing_name_moves_there() {
    let (_t, mut a) = make("ce28have", NOTES);
    let n = a.cols.len();
    add(&mut a, "note.status");
    assert_eq!(a.cols.len(), n);
    assert_eq!(a.cols[a.col], "status");
}

#[test]
fn test_ce_28_readonly_does_not_add() {
    let (_t, mut a) = make("ce28ro", NOTES);
    a.readonly = true;
    ch(&mut a, 'A');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.is_some());
}
