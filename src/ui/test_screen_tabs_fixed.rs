//! 画面の型のタブ(REL-8)の位置: 表と関係マップで変わらない(「+ 新規」の幅を関係マップでも空けておく)。

use super::bands::screen_tab_at;
use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{buffer, ch};
use super::*;

/// 1行目で、そのタブ(false = 表、true = 関係)を押せる桁の範囲。
fn tab_cols(a: &App, rel: bool) -> Vec<u16> {
    (0..a.size.0)
        .filter(|&x| screen_tab_at(a, x, 0) == Some(rel))
        .collect()
}

/// 1行目の、その桁に描かれた文字。
fn drawn(a: &App, x: u16) -> String {
    buffer(a)[(x, 0)].symbol().to_string()
}

#[test]
fn test_rel_8_tabs_do_not_move_between_screens() {
    // [REL-8] 「+ 新規」が出る表から関係マップへ移っても、タブの描く位置と押せる位置は同じ。
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
    assert_eq!(table, rel);
    // 描いた位置も同じ: 関係マップでは右のタブの先頭に `[`(選んでいる印)。
    assert_eq!(drawn(&a, rel.1[0]), "[");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(drawn(&a, table.0[0]), "[", "表では左のタブの先頭に `[`");
}
