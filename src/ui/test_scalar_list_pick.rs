//! [CE-19] リストの列の1つの文字列(`tags: x`)は、1つの要素のリストとして開き、確定でリストになる。
//! specs/_changes/2026-10-07-scalar-list-edit.md。

use super::super::keymap::Mode;
use super::super::test_screen::{app_of, col_named, press, typing, Tmp};
use super::super::ColorMode;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_ce_19_scalar_list_opens_as_one_item() {
    let tmp = Tmp::new("ce19scalar");
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"tags": "tags"}}"#,
    )
    .unwrap();
    tmp.write("a.md", "---\ntags: proj\n---\n");
    tmp.write("b.md", "---\ntags: [proj, 会議]\n---\n");
    let mut a = app_of(&tmp, ColorMode::None);
    a.row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "a.md")
        .unwrap();
    col_named(&mut a, "tags");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    let r = a.rows[a.row].clone();
    assert_eq!(
        a.changes.pending(&r, "tags").cloned(),
        Some(NewValue::List(vec!["proj".into(), "会議".into()]))
    );
}
