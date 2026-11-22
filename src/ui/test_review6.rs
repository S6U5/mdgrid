//! [navigation.frequency][cell-edit.calendar-keys] 1% のすぐ下の割合、空白の区切りの日時のカレンダー(specs/_history/decidespec/_changes/2026-10-06-review6-fixes.md)。

use super::test_screen::{col_named, press, Tmp};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

// [utest -> req~navigation.frequency-open~1, req~navigation.frequency-order~1, req~navigation.frequency-enter~1, req~navigation.frequency-esc~1, req~navigation.frequency-scope~1]
#[test]
fn test_nv_9_percent_just_under_one() {
    assert_eq!(super::freq::pct_text(1, 101), "(1%)");
    assert_eq!(super::freq::pct_text(1, 200), "(0.5%)");
}

// [utest -> req~cell-edit.calendar-keys-move~2, req~cell-edit.calendar-keys-keep-time~1]
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
