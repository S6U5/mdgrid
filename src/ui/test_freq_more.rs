//! 列の値の頻度表(NV-9)の追いの試験(レビューの差し戻し 1回目)。記録: specs/_changes/2026-10-06-frequency-table.md。
//! 窓の件数と Enter のあとの行の数が合うこと、ヘッダーの同じ値の表示、読み込みの途中と組み直しのとき。

use super::keymap::Mode;
use super::test_action_menu::{menu_lines, shot};
use super::test_screen::{ch, col_named, make, press, screen};
use super::*;
use mdgrid::i18n::Msg;
use ratatui::crossterm::event::KeyCode;

/// status と tags の両方(会議: a b d、家: a c、status の done: a b c)。
const NOTES: &[(&str, &str)] = &[
    (
        "a.md",
        "---\ntitle: a\nstatus: done\ntags: [会議, 家]\n---\n",
    ),
    ("b.md", "---\ntitle: b\nstatus: done\ntags: [会議]\n---\n"),
    ("c.md", "---\ntitle: c\nstatus: done\ntags: [家]\n---\n"),
    ("d.md", "---\ntitle: d\nstatus: todo\ntags: [会議]\n---\n"),
    ("e.md", "---\ntitle: e\nstatus: todo\n---\n"),
];

fn names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// 列 `col` で `%` を押し、窓の行(枠の内側)。
fn open(a: &mut App, col: &str) -> Vec<String> {
    a.row = 0;
    col_named(a, col);
    let before = shot(a, 80, 24);
    ch(a, '%');
    let after = shot(a, 80, 24);
    assert_eq!(a.mode, Mode::Freq, "% で開かない:\n{after}");
    menu_lines(&before, &after)
}

/// 窓の中の値 `value` の行の件数(値の次の語)。
fn count_of(lines: &[String], value: &str) -> Option<usize> {
    lines.iter().find_map(|l| {
        let w: Vec<&str> = l
            .split_whitespace()
            .map(|w| w.trim_start_matches('>'))
            .collect();
        let i = w.iter().position(|x| *x == value)?;
        w.get(i + 1)?.parse().ok()
    })
}

/// 窓で `value` の項目まで ↓ で動いて Enter。
fn pick(a: &mut App, lines: &[String], value: &str) {
    let items: Vec<&String> = lines.iter().skip(1).collect();
    let i = items
        .iter()
        .position(|l| {
            l.split_whitespace()
                .any(|w| w.trim_start_matches('>') == value)
        })
        .unwrap_or_else(|| panic!("{value} が窓に無い: {lines:?}"));
    for _ in 0..i {
        press(a, KeyCode::Down);
    }
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_nv_9_count_matches_rows_after_comma_on_other_column() {
    // [NV-9] [NV-8] `,` で status の done に絞ったまま tags の `%` → 窓の件数は done の行の中の数で、
    // Enter のあとの行の数と同じ(`,` の絞り込みは残る)。
    let (_t, mut a) = make("nv9more-other", NOTES);
    col_named(&mut a, "status");
    a.row = 0;
    ch(&mut a, ',');
    assert_eq!(names(&a), ["a.md", "b.md", "c.md"], "材料: , で done だけ");
    let lines = open(&mut a, "tags");
    assert!(
        a.message.as_deref() == Some(Msg::FreqStacked.text()),
        "重ねることの知らせが無い: {:?}",
        a.message
    );
    let n = count_of(&lines, "会議").expect("会議の件数");
    assert_eq!(n, 2, "done の行の中の会議の数でない: {lines:?}");
    pick(&mut a, &lines, "会議");
    assert_eq!(
        names(&a),
        ["a.md", "b.md"],
        "`,` が消えたか、会議で絞れていない"
    );
    assert_eq!(names(&a).len(), n, "窓の件数と Enter のあとの行の数が違う");
}

#[test]
fn test_nv_9_count_matches_rows_on_same_list_column_twice() {
    // [NV-9] tags の会議で絞ったあと、もう一度 tags の `%` → 家 1(a だけ)。Enter で家 → 1行。
    let (_t, mut a) = make("nv9more-twice", NOTES);
    let lines = open(&mut a, "tags");
    pick(&mut a, &lines, "会議");
    assert_eq!(names(&a), ["a.md", "b.md", "d.md"]);
    let lines = open(&mut a, "tags");
    let n = count_of(&lines, "家").expect("家の件数");
    pick(&mut a, &lines, "家");
    assert_eq!(names(&a), ["a.md"]);
    assert_eq!(names(&a).len(), n, "窓の件数と Enter のあとの行の数が違う");
}

#[test]
fn test_nv_9_header_shows_contains_for_list_and_empty_label() {
    // [NV-9] リストの要素で絞る → ヘッダーは「含む」の形。空で絞る → 「(空)」。
    let (_t, mut a) = make("nv9more-head", NOTES);
    let lines = open(&mut a, "tags");
    pick(&mut a, &lines, "会議");
    let head = screen(&a).lines().next().unwrap_or_default().to_string();
    assert!(
        head.contains("tags が 会議 を含む"),
        "含むの形でない: {head}"
    );

    let (_t, mut a) = make("nv9more-empty", NOTES);
    let lines = open(&mut a, "tags");
    pick(&mut a, &lines, "(空)");
    assert_eq!(names(&a), ["e.md"]);
    let head = screen(&a).lines().next().unwrap_or_default().to_string();
    assert!(head.contains("tags=(空)"), "空が「(空)」でない: {head}");
}

#[test]
fn test_nv_8_comma_on_empty_cell_shows_empty_label() {
    // [NV-8] 値の無いセルで `,` → ヘッダーの値は「(空)」。
    let (_t, mut a) = make("nv8more-empty", NOTES);
    col_named(&mut a, "tags");
    a.row = names(&a).iter().position(|n| n == "e.md").unwrap();
    ch(&mut a, ',');
    assert_eq!(names(&a), ["e.md"]);
    let head = screen(&a).lines().next().unwrap_or_default().to_string();
    assert!(head.contains("tags=(空)"), "空が「(空)」でない: {head}");
}

#[test]
fn test_nv_9_not_while_loading() {
    // [NV-9] [BV-16] 読み込みの途中は開かず、理由を出す。
    let (_t, mut a) = make("nv9more-loading", NOTES);
    a.progress.done = false;
    col_named(&mut a, "tags");
    ch(&mut a, '%');
    assert_eq!(a.mode, Mode::Table, "読み込みの途中に開いた");
    assert_eq!(a.message.as_deref(), Some(Msg::FreqLoading.text()));
}

#[test]
fn test_nv_9_closes_on_regrid() {
    // [NV-9] 開いている間に表を組み直したら閉じて、理由を出す。
    let (_t, mut a) = make("nv9more-regrid", NOTES);
    open(&mut a, "tags");
    a.refresh();
    assert_eq!(a.mode, Mode::Table, "組み直しで閉じない");
    assert!(a.freq.is_none());
    assert_eq!(a.message.as_deref(), Some(Msg::FreqClosed.text()));
}
