//! 日時の列のカレンダーの時刻の欄(CE-30・CE-31)。specs/_changes/2026-10-07-time-picker.md。
//! 今日は 2026-10-02 に固定する。

use super::test_screen::{col_named, ctrl, press, screen, Tmp};
use super::*;
use mdgrid::source::{NewValue, RowId};
use mdgrid::types;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// types.json(at は datetime)のある保管庫。
fn vault(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"at": "datetime"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    a.today = types::parse_date("2026-10-02").unwrap();
    (tmp, a)
}

fn shift(a: &mut App, code: KeyCode) {
    a.key(KeyEvent::new(code, KeyModifiers::SHIFT));
}

fn text(a: &App) -> String {
    a.input.as_ref().unwrap().text.clone()
}

fn pending(a: &App, row: usize) -> Option<NewValue> {
    let r: &RowId = &a.rows[row];
    a.changes.pending(r, "at").cloned()
}

fn date(s: &str) -> Option<NewValue> {
    Some(NewValue::Date(s.into()))
}

/// 行 `row` の at のセルを開く。
fn open(a: &mut App, row: usize) {
    a.row = row;
    col_named(a, "at");
    press(a, KeyCode::Enter);
    assert!(calendar::shown(a), "{}", screen(a));
}

#[test]
fn test_ce_30_time_picker_moves_and_commits() {
    let (_t, mut a) = vault(
        "ce30move",
        &[
            ("a.md", "---\nat: 2026-10-30T09:00\n---\n"),
            ("b.md", "---\nat: 2026-10-30\n---\n"),
        ],
    );
    open(&mut a, 0);
    // 時刻の欄が見え、今の時刻が出る。
    let s = screen(&a);
    assert!(s.contains("09:00"), "{s}");
    // 時刻に切り替えて ↑ で15分、Shift+↑ で1時間。
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T09:15");
    shift(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T10:15");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    assert_eq!(text(&a), "2026-10-30T09:45");
    // 日に戻すと ←→ は日を動かし、時刻はそのまま。
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-31T09:45");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0), date("2026-10-31T09:45"));
    // 時刻の無い値: 欄は空で始まり、動かさずに確定すれば日だけ。
    open(&mut a, 1);
    assert!(screen(&a).contains("--:--"), "{}", screen(&a));
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1), None, "元と同じ値なので書かない");
    // 打ち込みでも今までどおり入れられる。
    open(&mut a, 1);
    press(&mut a, KeyCode::End);
    super::test_screen::typing(&mut a, "T18:30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1), date("2026-10-30T18:30"));
}

#[test]
fn test_ce_31_time_picker_details() {
    let (_t, mut a) = vault(
        "ce31details",
        &[
            ("a.md", "---\nat: 2026-10-30\n---\n"),
            ("b.md", "---\nat: 2026-10-30T09:07\n---\n"),
            ("c.md", "---\nat: 2026-10-30T23:45\n---\n"),
            ("d.md", "---\nat: 2026-10-30T09:00:30+09:00\n---\n"),
        ],
    );
    // 時刻の無い値で最初に ↑ → 09:00 から15分で 09:15。
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T09:15");
    press(&mut a, KeyCode::Esc);
    // 刻みにそろえる: 09:07 で ↑ → 09:15、↓ → 09:00。
    open(&mut a, 1);
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T09:15");
    press(&mut a, KeyCode::Esc);
    open(&mut a, 1);
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Down);
    assert_eq!(text(&a), "2026-10-30T09:00");
    press(&mut a, KeyCode::Esc);
    // 0時をまたいでも日は変えない。
    open(&mut a, 2);
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T00:00");
    press(&mut a, KeyCode::Esc);
    // 秒とタイムゾーンは保つ。
    open(&mut a, 3);
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2026-10-30T09:15:30+09:00");
    press(&mut a, KeyCode::Esc);
    // 下の縁に切り替えのキー。
    open(&mut a, 0);
    assert!(screen(&a).contains("^O 時刻"), "{}", screen(&a));
}

#[test]
fn test_ce_31_bulk_time_goes_to_every_row() {
    // CE-10: 一括の編集で時刻を動かしたら、全部の行にその時刻。
    let (_t, mut a) = vault(
        "ce31bulk",
        &[
            ("a.md", "---\nat: 2026-10-30T09:00\n---\n"),
            ("b.md", "---\nat: 2026-10-30T18:30\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    open(&mut a, 0);
    assert!(a.input.as_ref().unwrap().bulk.is_some());
    ctrl(&mut a, 'o');
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0), date("2026-10-30T09:15"));
    assert_eq!(pending(&a, 1), date("2026-10-30T09:15"));
}
