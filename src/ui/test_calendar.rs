//! 日付のカレンダーと日付の形の試験(CE-20・CE-21・CE-22。関係: CE-5・CE-10・CE-11・CV-2・SR-9・SR-15)。
//! 今日は 2026-10-02(金)に固定する(`App::today`。起動では `MDGRID_TODAY`)。

use super::input::Entry;
use super::keymap::Mode;
use super::test_screen::{assert_fits, col_named, ctrl, golden, press, read, screen, typing, Tmp};
use super::*;
use mdgrid::source::{NewValue, RowId};
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

const TODAY: &str = "2026-10-02";

/// types.json(due は date、at は datetime)のある保管庫。
fn vault(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date", "at": "datetime"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    a.today = types::parse_date(TODAY).unwrap();
    (tmp, a)
}

/// 設定ファイルの文を当てる(CLI-3)。
fn configure(a: &mut App, toml: &str) {
    let (c, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    a.configure(&c);
}

fn pending(a: &App, row: usize, col: &str) -> Option<NewValue> {
    let r: &RowId = &a.rows[row];
    a.changes.pending(r, col).cloned()
}

fn text(a: &App) -> String {
    a.input.as_ref().unwrap().text.clone()
}

fn sel(a: &App) -> String {
    types::format_date(a.input.as_ref().unwrap().cal.as_ref().unwrap().sel)
}

/// 入力の文字を全部消す。
fn wipe(a: &mut App) {
    press(a, KeyCode::End);
    let n = text(a).chars().count();
    for _ in 0..n {
        press(a, KeyCode::Backspace);
    }
}

/// 日付の列にためる値。WB-18 で日付の列は Date として書く(囲まずに書くため)。
fn day(s: &str) -> NewValue {
    NewValue::Date(s.into())
}

#[test]
fn test_ce_20_calendar_select_and_type() {
    // [CE-20] due のセルで Enter → 入力ボックスの下に今の値の月のカレンダー、今の値の日が選ばれている。
    // → で次の日 → Enter → その日がためる変更。
    let (_t, mut a) = vault(
        "ce20",
        &[
            ("a.md", "---\ntitle: 会議\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ntitle: 買い物\ndue: 2026-09-01\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.input.as_ref().unwrap().entry, Entry::Date);
    assert!(calendar::shown(&a));
    assert_eq!(sel(&a), "2026-10-05");
    let s = screen(&a);
    let lines: Vec<&str> = s.lines().collect();
    let (_, cy) = view::cursor(&a).unwrap();
    // 入力ボックスのすぐ下の行が見出し(年と月)、次が曜日(日曜始まり)。
    assert!(lines[cy as usize + 1].contains(" 2026年10月 "), "{s}");
    assert!(
        lines[cy as usize + 2].contains("| 日  月  火  水  木  金  土 |"),
        "{s}"
    );
    // 選んでいる日は `>`、元の値は `*`(SR-15)。
    assert!(s.contains("> 5*"), "{s}");
    golden("ce_20", &s);
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-06");
    assert!(screen(&a).contains("> 6 "));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-10-06")));

    // `2026-11-03` と打って Enter → 打った日。打っている間はカレンダーが打った日の月に動く。
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-11-0");
    assert_eq!(sel(&a), "2026-10-06", "読めない間は動かない");
    typing(&mut a, "3");
    assert_eq!(sel(&a), "2026-11-03");
    assert!(screen(&a).contains(" 2026年11月 "));
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-11-03")));
    assert_fits(&mut a);
}

#[test]
fn test_ce_20_invalid_typing_keeps_input_and_esc_cancels() {
    // [CE-20] 打ち込みは今までどおり(CE-5): 実在しない日付は閉じずに理由。Esc で取り消し、Ctrl+R で編集前(CE-11)。
    let (_t, mut a) = vault("ce20bad", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-02-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.message.as_deref().unwrap().contains("実在しない日付"));
    ctrl(&mut a, 'r');
    assert_eq!(text(&a), "2026-10-05");
    assert_eq!(sel(&a), "2026-10-05");
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // `+3` も読め、カレンダーは今日から3日へ動く。
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "+3");
    assert_eq!(sel(&a), "2026-10-05");
    // Tab は確定して右へ(CE-11)。右にセルが無ければ表に戻る。
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.changes.count(), 0, "今の値と同じなので書かない");
}

#[test]
fn test_ce_20_click_a_day() {
    // [CE-20] カレンダーの日のクリックはその日を選ぶ(確定のクリックにしない)。
    let (_t, mut a) = vault("ce20click", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let (ty, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("2026年10月"))
        .unwrap();
    let gx = width::width(&line[..line.find("+-").unwrap()]);
    // 2026-10 は木曜始まり。2週目(4〜10日)の水曜(4列目)が 7 日。
    a.click((gx + 1 + 3 * 4 + 1) as u16, (ty + 2 + 1) as u16);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(text(&a), "2026-10-07");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-10-07")));
}

#[test]
fn test_ce_21_moves_today_and_clear() {
    // [CE-21] 2026-10-30 で ↓ → 2026-11-06。↑ で戻る。← で1日戻る。
    let (_t, mut a) = vault(
        "ce21",
        &[
            ("a.md", "---\ndue: 2026-10-30\n---\n"),
            ("b.md", "---\ndue: 2026-10-31\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    assert_eq!(text(&a), "2026-11-06");
    assert!(screen(&a).contains(" 2026年11月 "));
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Left);
    assert_eq!(text(&a), "2026-10-29");
    // PageDown → 翌月の同じ日。PageUp で戻る。
    press(&mut a, KeyCode::PageDown);
    assert_eq!(text(&a), "2026-11-29");
    press(&mut a, KeyCode::PageUp);
    assert_eq!(text(&a), "2026-10-29");
    // 今日に戻す(Ctrl+T)→ 今日の日。今日は下線(SR-15)。
    ctrl(&mut a, 't');
    assert_eq!(text(&a), TODAY);
    let buf = super::test_screen::buffer(&a);
    let (_, cy) = view::cursor(&a).unwrap();
    let underlined = (2..80u16).any(|x| {
        (cy + 1..24).any(|y| {
            buf[(x, y)].symbol() == "2"
                && buf[(x, y)]
                    .modifier
                    .contains(ratatui::style::Modifier::UNDERLINED)
                && buf[(x - 2, y)].symbol() == ">"
        })
    });
    assert!(underlined, "今日の日に下線");
    // 空にする(Ctrl+D)→ null のためる変更(CE-5・CE-9)。
    ctrl(&mut a, 'd');
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 0, "due"), Some(NewValue::Null));
    // 2026-10-31 で PageDown → 翌月に同じ日が無いので月末。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::PageDown);
    assert_eq!(text(&a), "2026-11-30");
    press(&mut a, KeyCode::Esc);
}

#[test]
fn test_ce_21_monday_start() {
    // [CE-21] 設定で月曜始まり → 1列目が月。
    let (_t, mut a) = vault(
        "ce21mon",
        &[("a.md", "---\ntitle: 会議\ndue: 2026-10-05\n---\n")],
    );
    configure(&mut a, "week_start = \"mon\"\n");
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("| 月  火  水  木  金  土  日 |"), "{s}");
    golden("ce_21_mon", &s);
}

#[test]
fn test_ce_21_mismatched_or_empty_shows_this_month() {
    // [CE-21] `someday`(CV-2)のセル・空のセル → 今日の月が出て、今日が選ばれ、元の値の `*` は無い。
    let (_t, mut a) = vault(
        "ce21cv2",
        &[
            ("a.md", "---\ndue: someday\n---\n"),
            ("b.md", "---\ndue:\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    for r in 0..2 {
        a.row = r;
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Edit);
        assert_eq!(sel(&a), TODAY);
        let s = screen(&a);
        assert!(s.contains(" 2026年10月 ") && s.contains("> 2 "), "{s}");
        assert!(a
            .input
            .as_ref()
            .unwrap()
            .cal
            .as_ref()
            .unwrap()
            .orig
            .is_none());
        press(&mut a, KeyCode::Esc);
    }
    // 型の合わない値の文字は打ち込みのまま、→ で日を選べば日付になる。
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(text(&a), "someday");
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-03");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-10-03")));
}

#[test]
fn test_ce_21_datetime_keeps_time() {
    // [CE-21] `2026-10-30T09:00` の日時のセルで → → `2026-10-31T09:00`(時刻を保つ)。
    // 元に時刻が無ければ日付だけになり、時刻は打ち込みで入れる。
    let (_t, mut a) = vault(
        "ce21dt",
        &[
            ("a.md", "---\nat: 2026-10-30T09:00\n---\n"),
            ("b.md", "---\nat: 2026-10-30\n---\n"),
        ],
    );
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().entry, Entry::DateTime);
    assert!(calendar::shown(&a));
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-31T09:00");
    press(&mut a, KeyCode::PageDown);
    assert_eq!(text(&a), "2026-11-30T09:00");
    press(&mut a, KeyCode::PageUp);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0, "元と同じ値なので書かない");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-31T09:00")));
    // 時刻の無い値。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-31");
    typing(&mut a, "T18:30");
    assert_eq!(sel(&a), "2026-10-31");
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-11-01T18:30", "打った時刻も保つ");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "at"), Some(day("2026-11-01T18:30")));
}

#[test]
fn test_ce_22_short_formats_keep_year() {
    // [CE-22][CE-20] 設定の形に4桁の年が無くても、違う日を書かない。カレンダーの日は元の値の日数から作り、
    // 矢印で選んだ日は日数から書く。入力ボックスは `YYYY-MM-DD`(表は設定の形のまま)。
    let (_t, mut a) = vault(
        "ce22short",
        &[
            ("a.md", "---\ndue: 2026-12-31\n---\n"),
            ("b.md", "---\ndue: 2020-06-10\n---\n"),
            ("c.md", "---\ndue: 1999-12-30\n---\n"),
        ],
    );
    configure(&mut a, "date_format = \"MM/DD\"\n");
    assert!(screen(&a).contains("12/31"));
    col_named(&mut a, "due");
    // 年をまたぐ: 2026-12-31 → → 2027-01-01。
    press(&mut a, KeyCode::Enter);
    assert_eq!(text(&a), "2026-12-31");
    assert_eq!(sel(&a), "2026-12-31");
    press(&mut a, KeyCode::Right);
    assert_eq!(sel(&a), "2027-01-01");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2027-01-01")));
    // 今年でない年: 開いて何もせず Enter → 何もためない。→ で 2020-06-11。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(sel(&a), "2020-06-10");
    assert!(screen(&a).contains(" 2020年6月 "));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 1);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "due"), Some(day("2020-06-11")));
    // 前の世紀(`YY/MM/DD`): 1999-12-30 は 2099 にならない。
    configure(&mut a, "date_format = \"YY/MM/DD\"\n");
    assert!(screen(&a).contains("99/12/30"));
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(text(&a), "1999-12-30");
    assert_eq!(sel(&a), "1999-12-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 2, "何もせず Enter は書かない");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 2, "due"), Some(day("1999-12-31")));
}

#[test]
fn test_ce_21_datetime_typed_without_time_keeps_time() {
    // [CE-21] 日時の列で `+3` を打つ(時刻なし)→ ←→ でも Enter でも元の時刻を保つ。
    let (_t, mut a) = vault("ce21dtrel", &[("a.md", "---\nat: 2026-10-30T09:00\n---\n")]);
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "+3");
    assert_eq!(sel(&a), "2026-10-05");
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-06T09:00");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-06T09:00")));
    // 打ち込みのまま Enter でも時刻を保つ。
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "+3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-05T09:00")));
}

#[test]
fn test_cv_2_datetime_mismatch_time_not_carried() {
    // [CV-2][CE-21] 日時の列の型に合わない値(`T25:00`・` 午後`)の後ろの部分は時刻として持ち越さない。
    // 合わない値なので今日の月を出し、元の日の `*` は付けない。
    let (_t, mut a) = vault(
        "cv2dt",
        &[
            ("a.md", "---\nat: 2026-10-30T25:00\n---\n"),
            ("b.md", "---\nat: 2026-10-30 午後\n---\n"),
        ],
    );
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().mismatch);
    assert_eq!(sel(&a), TODAY);
    assert!(a
        .input
        .as_ref()
        .unwrap()
        .cal
        .as_ref()
        .unwrap()
        .orig
        .is_none());
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-03");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-03")));
    // 日付だけを打つ → 時刻なしで書く(` 午後` を付けない)。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-11-03");
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-11-04");
    press(&mut a, KeyCode::Left);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "at"), Some(day("2026-11-03")));
    // 正しくない時刻を打つ → 時刻として覚えず、確定は閉じずに理由。
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-12-01T25:00");
    assert_eq!(a.input.as_ref().unwrap().cal.as_ref().unwrap().time, "");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.message.as_deref().unwrap().contains("読めない"));
}

#[test]
fn test_ce_10_bulk_datetime_keeps_each_rows_time() {
    // [CE-10][CE-21] 日時の列の一括: 日だけを変え、時刻は行ごとの元の値(元に時刻が無い行は時刻なし)。
    // 打ち込みで時刻を書いたときだけ、その時刻を全部の行に入れる。
    let (_t, mut a) = vault(
        "ce10dt",
        &[
            ("a.md", "---\nat: 2026-10-30T09:00\n---\n"),
            ("b.md", "---\nat: 2026-10-30T18:30\n---\n"),
            ("c.md", "---\nat: 2026-10-30\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "at");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_some());
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-31T09:00")));
    assert_eq!(pending(&a, 1, "at"), Some(day("2026-10-31T18:30")));
    assert_eq!(pending(&a, 2, "at"), Some(day("2026-10-31")));
    // 打ち込みで日だけ(`+3`)→ 行ごとの時刻を保つ。
    a.row = 0;
    ctrl(&mut a, 'a');
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "+3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "at"), Some(day("2026-10-05T09:00")));
    assert_eq!(pending(&a, 1, "at"), Some(day("2026-10-05T18:30")));
    assert_eq!(pending(&a, 2, "at"), Some(day("2026-10-05")));
    // 時刻を打つ → 全部の行にその時刻。カレンダーで動かしても打った時刻のまま。
    a.row = 0;
    ctrl(&mut a, 'a');
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-11-01T07:00");
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    for r in 0..3 {
        assert_eq!(pending(&a, r, "at"), Some(day("2026-11-02T07:00")));
    }
}

#[test]
fn test_ce_21_narrow_terminal_types_only() {
    // [CE-21][SR-9] 幅 30 の端末 → カレンダーが出ず、←→ は文字のカーソル、打ち込みはできる。
    let (_t, mut a) = vault("ce21w30", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    a.resize(30, 24);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(!calendar::shown(&a));
    let lines = view::render(&a, 29, 23);
    let all: String = lines
        .iter()
        .flat_map(|l| l.spans.iter().map(|s| s.content.to_string()))
        .collect();
    assert!(!all.contains('年'), "{all}");
    press(&mut a, KeyCode::Left);
    assert_eq!(text(&a), "2026-10-05");
    assert_eq!(a.input.as_ref().unwrap().cursor, 9);
    // PageDown・今日に戻すは出ていなくても効く。
    press(&mut a, KeyCode::PageDown);
    assert_eq!(text(&a), "2026-11-05");
    wipe(&mut a);
    typing(&mut a, "+1");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-10-03")));
    // 低い端末でも出さない(入力ボックスの上下に9行が無い)。
    a.resize(80, 12);
    press(&mut a, KeyCode::Enter);
    assert!(!calendar::shown(&a));
    press(&mut a, KeyCode::Esc);
}

#[test]
fn test_ce_21_calendar_above_when_no_room_below() {
    // [CE-20] 下に入らなければ入力ボックスの上に出す。
    let notes: Vec<(String, String)> = (0..16)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                "---\ndue: 2026-10-05\n---\n".to_string(),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, mut a) = vault("ce21above", &refs);
    col_named(&mut a, "due");
    a.row = 15;
    a.scroll_into_view();
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    let (_, cy) = view::cursor(&a).unwrap();
    let s = screen(&a);
    let title = s.lines().position(|l| l.contains("2026年10月")).unwrap();
    assert_eq!(title + 10, cy as usize, "{s}");
    assert_fits(&mut a);
}

#[test]
fn test_ce_21_keys_only_for_dates() {
    // [CE-21][SR-4] 今日に戻す・空にするのキーは日付・日時の入力だけ。テキストの入力では理由を出して何もしない。
    let (_t, mut a) = vault(
        "ce21text",
        &[("a.md", "---\ntitle: 会議\ndue: 2026-10-05\n---\n")],
    );
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 'd');
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.message.as_deref().unwrap().contains("日付"));
    assert_eq!(a.changes.count(), 0);
    press(&mut a, KeyCode::Esc);
    // キーの表(ヘルプ・下の帯の元)にある。
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Edit, "Ctrl+t"),
        Some(keymap::Action::Today)
    );
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Edit, "Ctrl+d"),
        Some(keymap::Action::Clear)
    );
}

#[test]
fn test_ce_10_bulk_date_with_calendar() {
    // [CE-10][CE-20] 一括で日付の列に入れるときもカレンダーと同じ入力。
    let (_t, mut a) = vault(
        "ce10cal",
        &[
            ("a.md", "---\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ndue: 2026-09-01\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_some());
    assert!(calendar::shown(&a));
    press(&mut a, KeyCode::Right);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-10-06")));
    assert_eq!(pending(&a, 1, "due"), Some(day("2026-10-06")));
}

#[test]
fn test_ce_22_date_format_in_table_and_input() {
    // [CE-22] `date_format = "YYYY/MM/DD"` → 表の due が `2026/10/05`。`2026/11/03` と打っても
    // `2026-11-03` と打っても、ファイルには `due: 2026-11-03`。型の合わない値はそのまま `!`(CV-2)。
    let (t, mut a) = vault(
        "ce22",
        &[
            ("a.md", "---\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ndue: 2026-10-06\n---\n"),
            ("c.md", "---\ndue: someday\n---\n"),
        ],
    );
    configure(&mut a, "date_format = \"YYYY/MM/DD\"\n");
    let s = screen(&a);
    assert!(s.contains("2026/10/05") && s.contains("!someday"), "{s}");
    assert!(!s.contains("2026-10-05"), "{s}");
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(text(&a), "2026/10/05");
    assert!(screen(&a).contains("YYYY/MM/DD"), "案内に設定の形");
    press(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026/10/06", "カレンダーで動かすと設定の形");
    wipe(&mut a);
    typing(&mut a, "2026/11/03");
    assert_eq!(sel(&a), "2026-11-03");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-11-03")));
    assert!(screen(&a).contains("*2026/11/03"));
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "2026-11-03");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "due"), Some(day("2026-11-03")));
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    // [WB-18] ファイルには表の見せ方の形ではなく `YYYY-MM-DD` で、元のとおり囲まずに書く。
    for n in ["a.md", "b.md"] {
        let body = read(&t, n);
        assert_eq!(body, "---\ndue: 2026-11-03\n---\n", "{n}");
    }
    // 設定が無い → 今までどおり `2026-10-05`。
    let (_t2, b) = vault("ce22def", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    assert!(screen(&b).contains("2026-10-05"));
}

/// Shift を付けたキーを送る(CE-23・CE-24)。
fn shift(a: &mut App, code: KeyCode) {
    a.key(ratatui::crossterm::event::KeyEvent::new(
        code,
        ratatui::crossterm::event::KeyModifiers::SHIFT,
    ));
}

/// カレンダーの下の縁の行(見出しの行より下で、最初に `+-` で始まる縁)。
fn bottom_edge(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let title = lines
        .iter()
        .position(|l| l.contains("2026年10月"))
        .unwrap_or_else(|| panic!("カレンダーが出ていない\n{s}"));
    lines[title + 1..]
        .iter()
        .find(|l| l.trim_start().starts_with("+-") || l.contains(" +-"))
        .unwrap_or_else(|| panic!("下の縁が無い\n{s}"))
        .to_string()
}

/// `line` の中で `key` のあとに `word` が出る(キーの表記と語が並ぶ)。
fn key_then_word(line: &str, key: &str, word: &str) -> bool {
    line.find(key)
        .map(|i| line[i + key.len()..].contains(word))
        .unwrap_or(false)
}

#[test]
fn test_ce_23_shift_left_right_moves_month() {
    // [CE-23] 2026-10-30 のセルでカレンダーを出し Shift+→ → 2026-11-30。そこから Shift+← を3回 → 2026-08-30。
    let (_t, mut a) = vault("ce23", &[("a.md", "---\ndue: 2026-10-30\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    shift(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-11-30");
    assert_eq!(sel(&a), "2026-11-30");
    assert!(screen(&a).contains(" 2026年11月 "));
    // 11-30 から3回戻す(10-30 から2回戻した 08-30。仕様の例は 10-30 からの数え方)。
    shift(&mut a, KeyCode::Left);
    shift(&mut a, KeyCode::Left);
    shift(&mut a, KeyCode::Left);
    assert_eq!(text(&a), "2026-08-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-08-30")));
    // [CE-21] PageDown はこれまでどおり1か月。
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::PageDown);
    assert_eq!(text(&a), "2026-09-30");
    shift(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-30");
    press(&mut a, KeyCode::Esc);
}

#[test]
fn test_ce_23_shift_left_clamps_to_month_end() {
    // [CE-23] 2026-03-31 で Shift+← → 2026-02-28(無ければ月末)。
    let (_t, mut a) = vault("ce23end", &[("a.md", "---\ndue: 2026-03-31\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    shift(&mut a, KeyCode::Left);
    assert_eq!(text(&a), "2026-02-28");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2026-02-28")));
}

#[test]
fn test_ce_23_ce_24_keys_on_bottom_edge() {
    // [CE-23][CE-24] カレンダーの下の縁に `Shift+←→` と「月」、`Shift+↑↓` と「年」が並んで出る。
    let (_t, mut a) = vault("ce23edge", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    let s = screen(&a);
    let edge = bottom_edge(&s);
    assert!(key_then_word(&edge, "Shift+←→", "月"), "{s}");
    assert!(key_then_word(&edge, "Shift+↑↓", "年"), "{s}");
    assert_fits(&mut a);
}

#[test]
fn test_ce_23_no_calendar_shift_right_keeps_date() {
    // [CE-23][SR-9] カレンダーが出ていない(幅 30)ときは Shift+→ で日付が変わらない(文字のカーソルが動くだけ)。
    let (_t, mut a) = vault("ce23w30", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    a.resize(30, 24);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(!calendar::shown(&a));
    press(&mut a, KeyCode::Home);
    shift(&mut a, KeyCode::Right);
    assert_eq!(text(&a), "2026-10-05");
    assert_eq!(a.input.as_ref().unwrap().cursor, 1, "文字のカーソルが動く");
    shift(&mut a, KeyCode::Left);
    assert_eq!(text(&a), "2026-10-05");
    assert_eq!(a.input.as_ref().unwrap().cursor, 0);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_24_shift_up_down_moves_year() {
    // [CE-24] 2026-10-30 で Shift+↓ → 2027-10-30。Shift+↑ → 1年前。
    let (_t, mut a) = vault("ce24", &[("a.md", "---\ndue: 2026-10-30\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    shift(&mut a, KeyCode::Down);
    assert_eq!(text(&a), "2027-10-30");
    assert!(screen(&a).contains(" 2027年10月 "));
    shift(&mut a, KeyCode::Up);
    shift(&mut a, KeyCode::Up);
    assert_eq!(text(&a), "2025-10-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2025-10-30")));
}

#[test]
fn test_ce_24_leap_day_clamps() {
    // [CE-24] 2028-02-29 で Shift+↓ → 2029-02-28。
    let (_t, mut a) = vault("ce24leap", &[("a.md", "---\ndue: 2028-02-29\n---\n")]);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    shift(&mut a, KeyCode::Down);
    assert_eq!(text(&a), "2029-02-28");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "due"), Some(day("2029-02-28")));
}

#[test]
fn test_ce_24_ambiguous_wide_keeps_year_key() {
    // [CE-24][CV-6][SR-9] 矢印を幅2と数える(ambiguous_wide)と1行に入らないので、縁の行を分けて「年」のキーも残す。
    let (_t, mut a) = vault("ce24wide", &[("a.md", "---\ndue: 2026-10-05\n---\n")]);
    configure(&mut a, "ambiguous_wide = true");
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(calendar::shown(&a));
    let lines: Vec<String> = view::render(&a, 79, 23)
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect();
    let all = lines.join("\n");
    for (key, word) in [("Shift+←→", "月"), ("Shift+↑↓", "年"), ("^T", "今日")] {
        assert!(
            lines.iter().any(|l| key_then_word(l, key, word)),
            "{key} {word}\n{all}"
        );
    }
    for l in &lines {
        assert!(width::width(l) <= 79, "{l}");
    }
    assert_fits(&mut a);
}
