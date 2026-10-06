//! その場の操作の一覧(SR-24)と前置きのキーの案内(SR-25)の受け入れの試験の続き(実装とレビューで見つけた形)。
//! 記録: specs/_changes/2026-10-05-action-menu.md。実装を見ずに、画面の文字と App の公開の口で確かめる。
//! 道具は test_action_menu のもの(窓の枠の内側の桁だけを一覧の行として読む)。

use super::keymap::{Action, Mode};
use super::test_action_menu::{
    band, frame, has_section, label_of, menu_lines, menu_mode, shot, sorted_by_s, Shot, NOTES,
};
use super::test_grid::{labels, open, vault};
use super::test_screen::{ch, col_named, make, press, screen};
use super::*;
use ratatui::crossterm::event::KeyCode;

/// status のセルを選んで `x` で一覧を開く(80×24)。開く前と後の画面を返す。
fn open_menu_24(a: &mut App, row: usize) -> (Shot, Shot) {
    a.row = row;
    col_named(a, "status");
    let before = shot(a, 80, 24);
    ch(a, 'x');
    let after = shot(a, 80, 24);
    (before, after)
}

// ---- SR-25: 前置きの取り消しは前置きだけ ----

#[test]
fn test_sr_25_more_esc_after_prefix_keeps_marks() {
    // [SR-25] [NV-5] 2行に印を付けて `g` のあと Esc → 前置きを取り消すだけで、印は残る
    // (表の Esc の「選択を解く」まで走らない)。帯は元に戻る。
    let (_t, mut a) = make("sr25marks", NOTES);
    col_named(&mut a, "status");
    ch(&mut a, ' ');
    ch(&mut a, ' ');
    assert_eq!(a.marked.len(), 2, "材料: Space で2行に印が付かない");
    let row = a.row;
    ch(&mut a, 'g');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.marked.len(), 2, "g のあとの Esc で印が消えた");
    assert_eq!(a.row, row, "g のあとの Esc で動いた");
    assert_eq!(a.mode, Mode::Table);
    let b = band(&screen(&a));
    assert!(b.contains("Enter 編集"), "Esc のあと帯が戻らない: {b}");
}

#[test]
fn test_sr_25_more_esc_after_prefix_in_menu_keeps_menu() {
    // [SR-25] [SR-24] 一覧を開いて `g` のあと Esc → 前置きを取り消すだけで、一覧は開いたまま。
    let (_t, mut a) = make("sr25menuesc", NOTES);
    let (before, _) = open_menu_24(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    ch(&mut a, 'g');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, menu_mode(), "g のあとの Esc で一覧が閉じた");
    // 一覧の窓(枠の内側)に節「セル」が残っている。
    let after = shot(&mut a, 80, 24);
    assert!(
        has_section(&menu_lines(&before, &after), "セル"),
        "g のあとの Esc で一覧が消えた:\n{after}"
    );
}

#[test]
fn test_sr_25_more_prefix_in_menu_band_returns() {
    // [SR-25] 一覧で `g` `g` → 続きを押したら帯は元に戻る(帯に出した続きは、続きを押せば実行されて案内が消える)。
    // 一覧が開いたままなら一覧の帯に、閉じたなら表の帯に戻っている。
    let (_t, mut a) = make("sr25menugg", NOTES);
    open_menu_24(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let top = label_of(&a, Action::Top);
    let menu_band = band(&screen(&a));
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    let b = band(&screen(&a));
    if a.mode == menu_mode() {
        assert_eq!(b, menu_band, "一覧で g g のあと帯が一覧の帯に戻らない");
    } else {
        assert_eq!(a.mode, Mode::Table);
        assert!(b.contains("Enter 編集"), "g g のあと帯が戻らない: {b}");
    }
    assert!(
        !b.contains("先頭の行") && !b.contains(&top),
        "g g のあとも続きの案内が残る: {b}"
    );
}

// ---- SR-24: 窓が入らない端末 ----

#[test]
fn test_sr_24_more_low_terminal_does_not_open() {
    // [SR-24] 端末が低くて窓が入らない(80×8)とき `x` → 一覧のモードにならず、メッセージ行に理由が出る。
    let (_t, mut a) = make("sr24low", NOTES);
    col_named(&mut a, "status");
    a.resize(80, 8);
    a.message = None;
    ch(&mut a, 'x');
    assert_eq!(a.mode, Mode::Table, "80×8 で一覧のモードになった");
    let msg = a
        .message
        .clone()
        .unwrap_or_else(|| panic!("80×8 の x で理由が出ない"));
    assert!(!msg.trim().is_empty());
    let s = shot(&mut a, 80, 8);
    let head: String = msg.trim().chars().take(4).collect();
    assert!(
        s.text.contains(&head),
        "メッセージ行に理由 {msg:?} が無い:\n{s}"
    );
}

// ---- SR-24: 一覧のクリック ----

#[test]
fn test_sr_24_more_click_item_runs_it() {
    // [SR-24] 一覧の項目(並べ替え)を左クリック → その操作が実行され(`s` と同じ並び)、一覧が閉じる。
    let want = sorted_by_s("sr24click-s");
    let (_t, mut a) = make("sr24click", NOTES);
    let sort = label_of(&a, Action::by_name("sort_column").unwrap());
    let (before, after) = open_menu_24(&mut a, 0);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let (x0, y0, _, _) = frame(&before, &after).unwrap_or_else(|| panic!("窓の枠が無い:\n{after}"));
    let i = menu_lines(&before, &after)
        .iter()
        .position(|l| l.contains(&sort) || l.contains("この列で並べ替え"))
        .unwrap_or_else(|| panic!("一覧に並べ替えが無い:\n{after}"));
    a.click(x0 + 2, y0 + i as u16);
    assert_eq!(a.mode, Mode::Table, "項目のクリックで一覧が閉じない");
    assert_eq!(labels(&a), want, "項目のクリックの並べ替えが s と違う");
}

#[test]
fn test_sr_24_more_click_outside_closes_only() {
    // [SR-24] 一覧の外(画面の左上の角)を左クリック → 一覧を閉じるだけで、何も実行しない
    // (並び・位置・ためる変更・エディタの頼みが変わらない)。
    let (_t, mut a) = make("sr24clickout", NOTES);
    let (before, after) = open_menu_24(&mut a, 1);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let (x0, y0, _, _) = frame(&before, &after).unwrap_or_else(|| panic!("窓の枠が無い:\n{after}"));
    assert!(x0 > 0 && y0 > 0, "材料: 窓が左上の角にかかる");
    let rows = labels(&a);
    let (row, col) = (a.row, a.col);
    a.click(0, 0);
    assert_eq!(a.mode, Mode::Table, "外のクリックで一覧が閉じない");
    assert_eq!(labels(&a), rows);
    assert_eq!((a.row, a.col), (row, col), "外のクリックで位置が変わった");
    assert_eq!(a.changes.count(), 0);
    assert!(!a.wants_editor(), "外のクリックでエディタを頼んだ");
}

// ---- SR-24: 読むだけの列の一括 ----

#[test]
fn test_sr_24_more_formula_column_has_no_bulk_item() {
    // [SR-24] [CE-10] 式の列(読むだけ)のセルで2行に印を付けて `x` → 「選んだ行にまとめて入れる」が無い。
    let tmp = vault("sr24formula");
    let mut a = open(
        &tmp,
        "formulas:\n  num: '1 + 2'\nviews:\n  - type: table\n    name: 式\n    order: [title, formula.num]\n",
        None,
    );
    a.row = 0;
    col_named(&mut a, "formula.num");
    ch(&mut a, ' ');
    ch(&mut a, ' ');
    assert_eq!(a.marked.len(), 2, "材料: Space で2行に印が付かない");
    col_named(&mut a, "formula.num");
    let before = shot(&mut a, 100, 50);
    ch(&mut a, 'x');
    let after = shot(&mut a, 100, 50);
    assert_eq!(a.mode, menu_mode(), "式のセルで x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(!lines.is_empty(), "一覧の窓が無い:\n{after}");
    assert!(
        !lines.iter().any(|l| l.contains("まとめて")),
        "式の列で「まとめて入れる」がある:\n{after}"
    );
}
