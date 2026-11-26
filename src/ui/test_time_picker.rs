//! 日時の列のカレンダーの時刻の欄(CE-30・CE-31)。specs/_history/decidespec/_changes/2026-10-07-time-picker.md。
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

// [utest -> req~cell-edit.datetime-calendar-time-field~2, req~cell-edit.datetime-calendar-keep-date-only~1, req~cell-edit.datetime-calendar-typing~1]
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

// [utest -> req~cell-edit.time-keys-midnight~1]
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

// [utest -> req~cell-edit.time-keys-bulk~1]
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

// ---- 時刻を数字で打つ・ボタンで動かす・案内(cell-edit.datetime-calendar-time-digits・-buttons・-hint) ----

fn nine(tag: &str) -> (Tmp, App) {
    vault(tag, &[("a.md", "---\nat: 2026-10-30T09:00\n---\n")])
}

/// カレンダーの時刻の欄に出ている `HH:MM`(選んでいれば `>` の後ろ)。
fn time_field(a: &App) -> String {
    let s = screen(a);
    let line = s
        .lines()
        .find(|l| l.contains(mdgrid::i18n::Msg::CalTime.text()) && l.contains(':'))
        .unwrap_or_else(|| panic!("時刻の欄が無い:\n{s}"));
    let at = line.find(':').unwrap();
    line[at - 2..at + 3].to_string()
}

/// 画面の最初の `label` をクリックする。
fn click_label(a: &mut App, label: &str) {
    let s = screen(a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(label))
        .unwrap_or_else(|| panic!("画面に「{label}」が無い:\n{s}"));
    let x = super::width::width(&line[..line.find(label).unwrap()]);
    a.click(x as u16 + 1, y as u16);
}

// [utest -> req~cell-edit.datetime-calendar-time-digits~1]
/// 時刻を選んで `1430` と打って Enter → 14:30 のためる変更。
#[test]
fn test_time_digits_four_commit() {
    let (_t, mut a) = nine("ctdig1");
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    super::test_screen::typing(&mut a, "1430");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0), date("2026-10-30T14:30"));
}

// [utest -> req~cell-edit.datetime-calendar-time-digits~1]
/// 時刻を選んで `9:30` と打つ → 時刻の欄が 09:30、入力の文字も 09:30。
#[test]
fn test_time_digits_with_colon() {
    let (_t, mut a) = nine("ctdig2");
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    super::test_screen::typing(&mut a, "9:30");
    assert_eq!(time_field(&a), "09:30", "{}", screen(&a));
    assert_eq!(text(&a), "2026-10-30T09:30");
}

// [utest -> req~cell-edit.datetime-calendar-time-digits~1]
/// `14` と打って Backspace を2回 → 打つ前の 09:00 に戻る。
#[test]
fn test_time_digits_backspace_restores() {
    let (_t, mut a) = nine("ctdig3");
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    super::test_screen::typing(&mut a, "14");
    assert_eq!(text(&a), "2026-10-30T14:00");
    press(&mut a, KeyCode::Backspace);
    press(&mut a, KeyCode::Backspace);
    assert_eq!(time_field(&a), "09:00", "{}", screen(&a));
    assert_eq!(text(&a), "2026-10-30T09:00");
}

// [utest -> req~cell-edit.datetime-calendar-time-digits~1]
/// `25` と打って Enter → 確定せず、読めない理由が出る。
#[test]
fn test_time_digits_unreadable_stays() {
    let (_t, mut a) = nine("ctdig4");
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    super::test_screen::typing(&mut a, "25");
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert!(a.input.is_some(), "入力は開いたまま:\n{}", screen(&a));
    assert_eq!(pending(&a, 0), None);
    assert!(a.message.is_some(), "理由が出る");
}

// [utest -> req~cell-edit.datetime-calendar-time-buttons~1]
/// 時刻の欄の `[+]` のクリックで 09:15、`[-]` のクリックで 08:45(09:00 から)。
#[test]
fn test_time_buttons_step() {
    let (_t, mut a) = nine("ctbtn1");
    open(&mut a, 0);
    click_label(&mut a, "[+]");
    assert_eq!(time_field(&a), "09:15", "{}", screen(&a));
    assert_eq!(text(&a), "2026-10-30T09:15");
    press(&mut a, KeyCode::Esc);
    open(&mut a, 0);
    click_label(&mut a, "[-]");
    assert_eq!(time_field(&a), "08:45", "{}", screen(&a));
}

// [utest -> req~cell-edit.datetime-calendar-time-hint~1]
/// 時刻に切り替えると、メッセージ行に時刻の操作の案内が出る。
#[test]
fn test_time_hint_in_cell() {
    let (_t, mut a) = nine("cthint1");
    open(&mut a, 0);
    ctrl(&mut a, 'o');
    let s = screen(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("15分") && l.contains("数字") && l.contains("^O")),
        "{s}"
    );
}

/// 新しいノートの窓で ask = [at] の台帳(at は日時の列)。
fn form(tag: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\nat: 2026-10-30T09:00\n---\n");
    let toml = "[types]\nat = \"datetime\"\n[new_note]\nask = [\"at\"]\n";
    let (cfg, warnings): (mdgrid::config::Config, _) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = mdgrid::source::markdown::Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
    app.start(super::startup::Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = types::parse_date("2026-10-02").unwrap();
    super::test_screen::ch(&mut app, 'a');
    super::test_screen::typing(&mut app, "会議");
    press(&mut app, KeyCode::Enter);
    assert!(
        calendar::shown(&app),
        "at の欄のカレンダー:\n{}",
        screen(&app)
    );
    (tmp, app)
}

// [utest -> req~cell-edit.datetime-calendar-time-digits~1]
/// 新しいノートの窓の日時の欄でも、時刻を選んで `1430` と打てば今日の 14:30 で作る。
#[test]
fn test_time_digits_in_new_note_form() {
    let (tmp, mut a) = form("ctdig5");
    ctrl(&mut a, 'o');
    super::test_screen::typing(&mut a, "1430");
    ctrl(&mut a, 's');
    let made = std::fs::read_to_string(tmp.notes().join("会議.md")).unwrap_or_default();
    assert!(made.contains("at: 2026-10-02T14:30"), "{made}");
}

// [utest -> req~cell-edit.datetime-calendar-time-hint~1]
/// 新しいノートの窓でも、時刻に切り替えるとメッセージ行に時刻の操作の案内が出る。
#[test]
fn test_time_hint_in_new_note_form() {
    let (_tmp, mut a) = form("cthint2");
    ctrl(&mut a, 'o');
    let s = screen(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("15分") && l.contains("数字") && l.contains("^O")),
        "{s}"
    );
}
