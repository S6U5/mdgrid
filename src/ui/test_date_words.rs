//! 日付の言葉を、セルの編集・一括の設定・絞り込みの比べる値で入れる(cell-edit.date-words の組)。
//! 今日は 2026-11-21(土)に固定する(`App::today`。起動では `MDGRID_TODAY`)。期待値は要求の受け入れの例から取る。
//! 言葉の読みそのもの(言葉の一覧と日の決め方)は src/test_datewords_unit.rs で確かめる。
//! 読みの見せ方(cell-edit.date-words-preview)の確かめは stest(tests/test_date_words_e2e.rs)で、ここの見せ方の
//! 試験は要求の名前だけを書く(要求の 確かめ: は stest だけなので、utest の印にしない)。

use super::keymap::Mode;
use super::test_screen::{ch, col_named, ctrl, press, screen, text, typing, Tmp};
use super::*;
use mdgrid::settings::{CmpOp, Cond, Op};
use mdgrid::source::NewValue;
use mdgrid::types;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

const TODAY: &str = "2026-11-21";

/// types.json(due は date、at は datetime)のある保管庫。今日は `today`。
fn vault(name: &str, notes: &[(&str, &str)], today: &str) -> (Tmp, App) {
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
    a.today = types::parse_date(today).unwrap();
    (tmp, a)
}

/// 名前が `label` の行に移る。
fn go_row(a: &mut App, label: &str) {
    a.row = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == label)
        .unwrap_or_else(|| panic!("行 {label} が無い"));
    a.scroll_into_view();
}

/// 名前が `label` の行の、列 `col` のためた値。
fn pending(a: &App, label: &str, col: &str) -> Option<NewValue> {
    let r = a
        .rows
        .iter()
        .find(|r| a.src.label(r) == label)
        .unwrap_or_else(|| panic!("行 {label} が無い"));
    a.changes.pending(r, col).cloned()
}

fn day(s: &str) -> NewValue {
    NewValue::Date(s.into())
}

/// 入力の文字を全部消す。
fn wipe(a: &mut App) {
    press(a, KeyCode::End);
    let n = a.input.as_ref().unwrap().text.chars().count();
    for _ in 0..n {
        press(a, KeyCode::Backspace);
    }
}

/// 行 `label` の列 `col` のセルを開き、`words` を打つ(確定はしない)。
fn type_into(a: &mut App, label: &str, col: &str, words: &str) {
    go_row(a, label);
    col_named(a, col);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    wipe(a);
    typing(a, words);
}

/// 入力ボックスの行の文字(端末のカーソルの行)。
fn input_line(a: &App) -> String {
    let (_, y) = view::cursor(a).expect("入力ボックスのカーソル");
    screen(a)
        .lines()
        .nth(y as usize)
        .unwrap_or_default()
        .to_string()
}

#[test]
fn test_date_words_cell_edit() {
    // [utest -> req~cell-edit.date-words~1] 今日が 2026-11-21 なら、due のセルに `明日` と打って Enter → due: 2026-11-22 をためる。
    let (_t, mut a) = vault("dw-cell", &[("a.md", "---\ndue: 2026-10-05\n---\n")], TODAY);
    type_into(&mut a, "a.md", "due", "明日");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, "a.md", "due"), Some(day("2026-11-22")));
}

#[test]
fn test_date_words_bulk_set() {
    // [utest -> req~cell-edit.date-words~1] 一括の設定でも、全部の行を選んで due に `tomorrow` → どの行も due: 2026-11-22 をためる。
    let (_t, mut a) = vault(
        "dw-bulk",
        &[
            ("a.md", "---\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ndue: 2026-09-01\n---\n"),
        ],
        TODAY,
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_some(), "一括の入力");
    wipe(&mut a);
    typing(&mut a, "tomorrow");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, "a.md", "due"), Some(day("2026-11-22")));
    assert_eq!(pending(&a, "b.md", "due"), Some(day("2026-11-22")));
}

#[test]
fn test_date_words_week_start_setting() {
    // [utest -> req~cell-edit.date-words-weekday~2] 設定の週の始まりが月曜で今日が 2026-11-22(日)なら、セルで `今週の月曜` は 2026-11-16、`来週の月曜` は 2026-11-23 になる。
    let (_t, mut a) = vault(
        "dw-week",
        &[
            ("a.md", "---\ndue: 2026-10-05\n---\n"),
            ("b.md", "---\ndue: 2026-09-01\n---\n"),
        ],
        "2026-11-22",
    );
    let (c, warnings) = mdgrid::config::parse("[dates]\nweek_start = \"mon\"\n").unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    a.configure(&c);
    type_into(&mut a, "a.md", "due", "今週の月曜");
    press(&mut a, KeyCode::Enter);
    type_into(&mut a, "b.md", "due", "来週の月曜");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, "a.md", "due"), Some(day("2026-11-16")));
    assert_eq!(pending(&a, "b.md", "due"), Some(day("2026-11-23")));
}

#[test]
fn test_date_words_preview_beside_input() {
    // [cell-edit.date-words-preview] `来週の金` まで打つと入力の欄の行に「読めない」、続けて `曜` で同じ行に `2026-11-27(金)` が出てカレンダーが 27 日を選び、Enter で due: 2026-11-27 をためる。
    let (_t, mut a) = vault(
        "dw-preview",
        &[("a.md", "---\ndue: 2026-10-05\n---\n")],
        TODAY,
    );
    type_into(&mut a, "a.md", "due", "来週の金");
    let line = input_line(&a);
    assert!(line.contains("読めない"), "{line:?}\n{}", screen(&a));
    typing(&mut a, "曜");
    let line = input_line(&a);
    assert!(line.contains("2026-11-27(金)"), "{line:?}\n{}", screen(&a));
    assert!(!line.contains("読めない"), "{line:?}");
    let cal = a.input.as_ref().unwrap().cal.as_ref().unwrap();
    assert_eq!(types::format_date(cal.sel), "2026-11-27");
    assert!(screen(&a).contains(">27"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, "a.md", "due"), Some(day("2026-11-27")));
}

#[test]
fn test_date_words_invalid_keeps_input() {
    // [utest -> req~cell-edit.date-words-invalid~1] `あさっての次` で Enter → 欄は閉じず、不正を出し、ためた変更は増えない。
    let (_t, mut a) = vault(
        "dw-invalid",
        &[("a.md", "---\ndue: 2026-10-05\n---\n")],
        TODAY,
    );
    type_into(&mut a, "a.md", "due", "あさっての次");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "閉じない");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("あさっての次") && m.contains("読めない"),
        "{m:?}"
    );
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_date_words_datetime_keeps_time() {
    // [utest -> req~cell-edit.date-words-datetime~1] 日時の列で `2026-11-21T15:00` のセルに `明日` → `2026-11-22T15:00`。元に時刻が無いセルは `2026-11-22`。
    let (_t, mut a) = vault(
        "dw-datetime",
        &[
            ("a.md", "---\nat: 2026-11-21T15:00\n---\n"),
            ("b.md", "---\nat: 2026-11-21\n---\n"),
        ],
        TODAY,
    );
    type_into(&mut a, "a.md", "at", "明日");
    press(&mut a, KeyCode::Enter);
    type_into(&mut a, "b.md", "at", "明日");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, "a.md", "at"), Some(day("2026-11-22T15:00")));
    assert_eq!(pending(&a, "b.md", "at"), Some(day("2026-11-22")));
}

// ---- 絞り込みの比べる値(絞り込みの窓。test_filter_window.rs と同じ操作) ----

/// 今の端末の大きさで描いた画面の文字。
fn scr(a: &App) -> String {
    let mut term = Terminal::new(TestBackend::new(a.size.0, a.size.1)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// ビューの帯の「絞り込み」のボタンを押す。
fn click_filter_button(a: &mut App) {
    let s = scr(a);
    let (y, line) = s
        .lines()
        .enumerate()
        .take(view::data_y(a))
        .find(|(_, l)| l.contains("検索"))
        .unwrap_or_else(|| panic!("検索の欄が無い:\n{s}"));
    let at = line.find("絞り込み").expect("絞り込みのボタン");
    let x = width::width(&line[..at]) as u16 + 2;
    a.click(x, y as u16);
}

/// 窓の中の行を、先頭に戻ってから j で下りて `target` の行(頭に「> 」)を選ぶ。
fn select(a: &mut App, target: &str) {
    for _ in 0..40 {
        ch(a, 'k');
    }
    for _ in 0..40 {
        let hit = scr(a).lines().any(|l| {
            l.match_indices("> ")
                .any(|(i, _)| l[i + 2..].find(target).is_some_and(|p| p <= 8))
        });
        if hit {
            return;
        }
        ch(a, 'j');
    }
    panic!("窓に「{target}」の行が無い:\n{}", scr(a));
}

#[test]
fn test_date_words_filter_value() {
    // [utest -> req~cell-edit.date-words~1] 絞り込みの比べる値でも、due の「< より前」に `明日` → 条件の値は 2026-11-22 になり、それより前の行だけが残る。
    let (_t, mut a) = vault(
        "dw-filter",
        &[
            ("a.md", "---\ndue: 2026-11-20\n---\n"),
            ("b.md", "---\ndue: 2026-11-22\n---\n"),
            ("c.md", "---\ndue: 2026-11-25\n---\n"),
        ],
        TODAY,
    );
    click_filter_button(&mut a);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    select(&mut a, "+ 絞り込みを足す");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "比べる");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "<  より小さい");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText);
    typing(&mut a, "明日");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.settings.filters,
        [Cond {
            col: "due".to_string(),
            op: Op::Cmp(CmpOp::Lt, "2026-11-22".to_string()),
        }]
    );
    let names: Vec<String> = a.rows.iter().map(|r| a.src.label(r)).collect();
    assert_eq!(names, ["a.md"]);
}

#[test]
fn test_date_words_filter_preview() {
    // [cell-edit.date-words-preview] 絞り込みの比べる値を打っているあいだも、値の行に読んだ日 `2026-11-22(日)` を出し、読めない言葉では「読めない」を出す。
    let (_t, mut a) = vault(
        "dw-filter-pv",
        &[("a.md", "---\ndue: 2026-11-20\n---\n")],
        TODAY,
    );
    click_filter_button(&mut a);
    select(&mut a, "+ 絞り込みを足す");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "比べる");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "<  より小さい");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "明日");
    let s = scr(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("明日") && l.contains("2026-11-22(日)")),
        "{s}"
    );
    typing(&mut a, "の次");
    let s = scr(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("明日の次") && l.contains("読めない")),
        "{s}"
    );
}

/// 行 `line` の中で `pat` が始まる桁(表示の幅で)。
fn col_of(line: &str, pat: &str) -> Option<usize> {
    line.find(pat).map(|i| width::width(&line[..i]))
}

#[test]
fn test_date_words_preview_inside_new_note_window() {
    // [cell-edit.date-words-preview] 新しいノートの窓(`a`)の due の欄に `明日` と打つと、読んだ日 `2026-11-22(日)` は欄と同じ行の、窓の枠の中
    // (左の縁 `╭` と右の縁 `╮` の桁のあいだ)に出て、欄の名前 due と打った文字を隠さない(2026-11-25 の実地の操作で、枠の右の外に
    // はみ出していた)。幅 80 と 160 の端末で。
    for wd in [80u16, 160] {
        let (_t, mut a) = vault(
            &format!("dw-newnote-{wd}"),
            &[("a.md", "---\ndue: 2026-10-05\n---\n")],
            TODAY,
        );
        a.resize(wd, 24);
        ch(&mut a, 'a');
        assert!(a.note.flow.is_some(), "新しいノートの窓:\n{}", scr(&a));
        typing(&mut a, "x");
        press(&mut a, KeyCode::Enter);
        typing(&mut a, "明日");
        let s = scr(&a);
        let top = s
            .lines()
            .find(|l| l.contains("新しいノート"))
            .unwrap_or_else(|| panic!("窓の題が無い:\n{s}"));
        let left = col_of(top, "╭").expect("窓の左上");
        let right = col_of(top, "╮").expect("窓の右上");
        let line = s
            .lines()
            .find(|l| l.contains("明日"))
            .unwrap_or_else(|| panic!("打った文字の行が無い:\n{s}"));
        let at = col_of(line, "2026-11-22(日)")
            .unwrap_or_else(|| panic!("幅 {wd}: 読んだ日が無い:\n{s}"));
        let end = at + width::width("2026-11-22(日)");
        assert!(
            left < at && end <= right,
            "幅 {wd}: 読んだ日({at}〜{end})が窓の枠({left}〜{right})の外:\n{s}"
        );
        let label =
            col_of(line, "due").unwrap_or_else(|| panic!("幅 {wd}: 欄の名前が隠れる:\n{s}"));
        let typed = col_of(line, "明日").unwrap();
        assert!(
            label < typed && typed < at,
            "幅 {wd}: 欄の名前・打った文字・読んだ日の順:\n{s}"
        );
    }
}
