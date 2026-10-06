//! [NV-9][CE-21] 1% のすぐ下の割合、空白の区切りの日時のカレンダー(specs/_changes/2026-10-06-review6-fixes.md)。

use super::test_screen::{col_named, press, Tmp};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_nv_9_percent_just_under_one() {
    assert_eq!(super::freq::pct_text(1, 101), "(1%)");
    assert_eq!(super::freq::pct_text(1, 200), "(0.5%)");
}

#[test]
fn test_ce_21_space_datetime_day_pick_writes_t() {
    let tmp = Tmp::new("r6cal");
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"at": "datetime"}}"#,
    )
    .unwrap();
    tmp.write("a.md", "---\nat: 2026-10-06 10:00\n---\n");
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.changes.pending(&a.rows[0], "at").cloned(),
        Some(NewValue::Date("2026-10-07T10:00".into())),
        "{:?}",
        a.message
    );
}
