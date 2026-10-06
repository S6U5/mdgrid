//! [CE-29][WB-1] 列のキーの名前の変更と削除を、全部のノートにためる変更としてまとめて行う。
//! specs/_changes/2026-10-06-key-rename-delete.md。

use super::keymap::{Action, Mode};
use super::test_screen::{ch, col_named, ctrl, make, press, read, typing};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\ntitle: a\n---\nbody a\n"),
    ("b.md", "---\ntitle: b\nstatus: doing # now\n---\n"),
    ("c.md", "---\nstatus: x\nstate: y\n---\n"),
    ("d.md", "---\ntitle: d\n---\n"),
];

fn at(a: &App, name: &str, col: &str) -> Option<NewValue> {
    let i = a.rows.iter().position(|r| r.0.ends_with(name)).unwrap();
    a.changes.pending(&a.rows[i], col).cloned()
}

#[test]
fn test_ce_29_rename_key_in_all_notes() {
    let (t, mut a) = make("ce29ren", NOTES);
    col_named(&mut a, "status");
    a.apply(Action::RenameKey);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    typing(&mut a, "state");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let rename = Some(NewValue::RenameKey("state".into()));
    assert_eq!(at(&a, "a.md", "status"), rename);
    assert_eq!(at(&a, "b.md", "status"), rename);
    assert_eq!(
        at(&a, "c.md", "status"),
        None,
        "state が既にあるノートは飛ばす"
    );
    assert_eq!(at(&a, "d.md", "status"), None);
    let m = a.message.clone().unwrap();
    assert!(m.contains('2') && m.contains("state"), "{m}");
    // 1手で戻せる。
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    ctrl(&mut a, 'r');
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        read(&t, "a.md"),
        "---\nstate: todo\ntitle: a\n---\nbody a\n"
    );
    assert_eq!(read(&t, "b.md"), "---\ntitle: b\nstate: doing # now\n---\n");
    assert_eq!(read(&t, "c.md"), NOTES[2].1);
    assert_eq!(read(&t, "d.md"), NOTES[3].1);
}

#[test]
fn test_ce_29_delete_key_after_y() {
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntags:\n  - x\n  - y\n# keep\ntitle: a\n---\n"),
        ("b.md", "---\ntitle: b\ntags: [z]\n---\n"),
        ("c.md", "---\ntitle: c\n---\n"),
    ];
    let (t, mut a) = make("ce29del", notes);
    col_named(&mut a, "tags");
    // y 以外はやめる。
    a.apply(Action::DeleteKey);
    assert_eq!(a.mode, Mode::Palette);
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0);
    a.apply(Action::DeleteKey);
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert_eq!(at(&a, "a.md", "tags"), Some(NewValue::DeleteKey));
    assert_eq!(at(&a, "b.md", "tags"), Some(NewValue::DeleteKey));
    assert_eq!(a.changes.count(), 2);
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), "---\n# keep\ntitle: a\n---\n");
    assert_eq!(read(&t, "b.md"), "---\ntitle: b\n---\n");
    assert_eq!(read(&t, "c.md"), notes[2].1);
}

#[test]
fn test_ce_29_not_in_readonly_or_bad_names() {
    let (_t, mut a) = make("ce29ro", NOTES);
    col_named(&mut a, "status");
    a.readonly = true;
    a.apply(Action::RenameKey);
    assert_eq!(a.mode, Mode::Table);
    a.readonly = false;
    a.apply(Action::RenameKey);
    typing(&mut a, "file.x");
    press(&mut a, KeyCode::Enter);
    assert!(a.message.is_some());
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_29_pending_edit_is_not_dropped() {
    let (_t, mut a) = make("ce29pend", NOTES);
    col_named(&mut a, "status");
    a.row = a.rows.iter().position(|r| r.0.ends_with("a.md")).unwrap();
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    a.apply(Action::RenameKey);
    typing(&mut a, "state");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        at(&a, "a.md", "status"),
        Some(NewValue::Str("done".into())),
        "直した値を残す"
    );
    assert_eq!(
        at(&a, "b.md", "status"),
        Some(NewValue::RenameKey("state".into()))
    );
    assert!(a.message.clone().unwrap().contains("保存していない"));
}
