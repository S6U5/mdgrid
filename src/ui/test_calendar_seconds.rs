//! [CE-21] 日時のカレンダーで日を選んでも、秒つきの時刻は元の値のまま(落とさない)。
//! specs/_changes/2026-10-06-calendar-seconds.md。

use super::test_screen::{col_named, press, Tmp};
use super::*;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

fn vault(name: &str, value: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"at": "datetime"}}"#,
    )
    .unwrap();
    tmp.write("a.md", &format!("---\nat: {value}\n---\n"));
    let a = super::test_screen::app_of(&tmp, ColorMode::None);
    (tmp, a)
}

fn pick_next_day(value: &str, name: &str) -> (String, Option<NewValue>) {
    let (_t, mut a) = vault(name, value);
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    let text = a.input.as_ref().unwrap().text.clone();
    press(&mut a, KeyCode::Enter);
    let v = a.changes.pending(&a.rows[0], "at").cloned();
    (text, v)
}

#[test]
fn test_ce_21_calendar_keeps_seconds() {
    let (text, v) = pick_next_day("2026-10-30T09:00:15", "ce21sec");
    assert_eq!(text, "2026-10-31T09:00:15");
    assert_eq!(v, Some(NewValue::Date("2026-10-31T09:00:15".into())));
}
