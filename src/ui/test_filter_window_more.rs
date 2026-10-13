//! 絞り込みの窓(NV-29)の続き: 低い端末で段が入らなくなったら閉じること、呼び直す前のモードの表示名。

use super::keymap::Mode;
use super::test_screen::{ch, make, press};
use super::*;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\npriority: 5\n---\n"),
    ("b.md", "---\nstatus: todo\npriority: 1\n---\n"),
];

#[test]
fn test_nv_29_low_terminal_step_closes_with_reason() {
    // [NV-29] 条件の並びは入るが、列を選ぶ段は入らない低い端末 → 段へ進むと、見えないまま動かさずに閉じて理由。
    // 高さは、窓(条件の並び)がやっと入る低さ(SR-1 で窓の上のビューの帯の位置が変わっても同じ意味になるように、
    // 低い方から試して最初に窓が開く高さを使う。前の配置では 9)。
    let (_t, mut a) = make("nv29low", NOTES);
    let h = (4..24u16)
        .find(|&h| {
            a.resize(80, h);
            a.apply(super::keymap::Action::FilterMenu);
            let open = a.mode == Mode::FilterWin;
            if open {
                press(&mut a, KeyCode::Esc);
            }
            a.message = None;
            open
        })
        .expect("窓の入る高さ");
    a.resize(80, h);
    a.apply(super::keymap::Action::FilterMenu);
    assert_eq!(a.mode, Mode::FilterWin);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.filter_win.is_none());
    assert!(a.message.as_deref().unwrap_or("").contains("絞り込みの窓"));
    assert!(a.settings.filters.is_empty());
    ch(&mut a, 'j');
    assert!(a.settings.filters.is_empty());
}

#[test]
fn test_nv_29_old_mode_names_keep_meaning() {
    // [NV-1][NV-2] 呼び直す前の表示名を書いた設定は、前のモードのまま読む(`検索`・`Search` は `/`、
    // `絞り込み`・`Filter` は上の欄)。新しい名前と設定の名前も引ける。
    assert_eq!(Mode::by_name("検索"), Some(Mode::Search));
    assert_eq!(Mode::by_name("Search"), Some(Mode::Search));
    assert_eq!(Mode::by_name("絞り込み"), Some(Mode::Filter));
    assert_eq!(Mode::by_name("Filter"), Some(Mode::Filter));
    assert_eq!(Mode::by_name("探す"), Some(Mode::Search));
    assert_eq!(Mode::by_name("Find"), Some(Mode::Search));
    assert_eq!(Mode::by_name("filter_window"), Some(Mode::FilterWin));
    assert_eq!(Mode::by_name("filter_input"), Some(Mode::FilterText));
}
