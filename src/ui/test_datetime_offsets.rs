//! [CE-21][CV-2] 時差つきの日時のセルでカレンダーを開くと、元の日から始め、時刻(時差を含む)を保つ。
//! specs/_changes/2026-10-06-datetime-offsets.md。

use super::test_screen::{col_named, press, Tmp};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_ce_21_offset_time_kept() {
    let tmp = Tmp::new("ce21off");
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"updated": "datetime"}}"#,
    )
    .unwrap();
    tmp.write("a.md", "---\nupdated: 2026-10-05T21:44:59Z\n---\n");
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    col_named(&mut a, "updated");
    assert!(
        !super::cell::shown(&a, &a.rows[0].clone(), "updated")
            .text
            .contains(super::cell::MISFIT_MARK),
        "型の合う値(! が付かない)"
    );
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    assert_eq!(a.input.as_ref().unwrap().text, "2026-10-06T21:44:59Z");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.changes.pending(&a.rows[0], "updated").cloned(),
        Some(NewValue::Date("2026-10-06T21:44:59Z".into()))
    );
}

#[test]
fn test_ce_21_space_separated_written_with_t() {
    use super::entry::parse;
    use super::input::Entry;
    let fmt = mdgrid::types::DateFormat::default();
    assert_eq!(
        parse(Entry::DateTime, "2026-10-06 10:00", 0, &fmt),
        Ok(NewValue::Date("2026-10-06T10:00".into()))
    );
    assert_eq!(
        parse(Entry::DateTime, "2026-10-06T10:00+09:00", 0, &fmt),
        Ok(NewValue::Date("2026-10-06T10:00+09:00".into()))
    );
}
