//! 画面の型のメニューのボタン(REL-8)と設定のボタン(SR-42)の位置: 表と関係マップで変わらない
//! (2026-10-12-header-menu で、タブ「[表] 関係」は右の端のメニューのボタンと設定のボタンに置き換わった)。

use super::bands::{header_button_at, HeaderButton};
use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{buffer, ch};
use super::*;

/// 1行目で、そのボタン(false = 画面の型のメニュー、true = 設定)を押せる桁の範囲。
fn tab_cols(a: &App, rel: bool) -> Vec<u16> {
    let want = if rel {
        HeaderButton::Settings
    } else {
        HeaderButton::Menu
    };
    (0..a.size.0)
        .filter(|&x| header_button_at(a, x, super::bands::head_y(a)) == Some(want))
        .collect()
}

/// 1行目の、その桁に描かれた文字。
fn drawn(a: &App, x: u16) -> String {
    buffer(a)[(x, super::bands::head_y(a))].symbol().to_string()
}

#[test]
fn test_rel_8_tabs_do_not_move_between_screens() {
    // [REL-8][SR-42] 「+ 新規」が出る表から関係マップへ移っても、設定のボタンの描く位置と押せる位置は同じで、
    // 画面の型のメニューのボタンは右の端(設定のボタンの左)がそろう(ボタンの字は型の名前で変わる)。
    let tmp = workspace("tabs_fixed");
    let mut a = boot_tasks(&tmp);
    a.resize(80, 24); // buffer() の大きさ
    assert!(
        super::new_note::button_shown(&a),
        "書ける表なので「+ 新規」が出る"
    );
    let table = (tab_cols(&a, false), tab_cols(&a, true));
    assert!(!table.0.is_empty() && !table.1.is_empty());
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    let rel = (tab_cols(&a, false), tab_cols(&a, true));
    assert_eq!(table.1, rel.1);
    assert_eq!(table.0.last(), rel.0.last());
    // 描いた位置も同じ: 設定のボタンの先頭の次にキーの o、メニューのボタンの先頭の次に型の印
    // (関係マップでは ◇)。
    assert_eq!(drawn(&a, rel.1[0] + 1), "o");
    assert_eq!(drawn(&a, rel.0[0] + 1), "◇");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(drawn(&a, table.1[0] + 1), "o");
    assert_eq!(
        drawn(&a, table.0[0] + 1),
        "▦",
        "表ではメニューのボタンの先頭の次に ▦"
    );
}
