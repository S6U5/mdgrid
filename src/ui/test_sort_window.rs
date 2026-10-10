//! 並べ替えの窓(NV-24): 検索の欄の右の「並べ替え」で開き、足す・向き・外す・順。すぐ効いて設定に残る。

use super::keymap::Mode;
use super::test_grid::labels;
use super::test_screen::{ch, make, press, screen, Tmp};
use super::*;
use mdgrid::settings::Dir;
use ratatui::crossterm::event::KeyCode;

fn boot(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("a.md", "---\ndue: 2026-10-03\nprio: 2\n---\n"),
            ("b.md", "---\ndue: 2026-10-01\nprio: 1\n---\n"),
            ("c.md", "---\ndue: 2026-10-02\nprio: 2\n---\n"),
        ],
    )
}

/// 検索の欄の右のボタンの桁と行。
fn button_at(a: &App) -> (u16, u16) {
    let y = (0..a.size.1)
        .find(|&y| (0..a.size.0).any(|x| bands::sort_button_at(a, x, y)))
        .expect("並べ替えのボタン");
    let x = (0..a.size.0)
        .find(|&x| bands::sort_button_at(a, x, y))
        .unwrap();
    (x, y)
}

/// 足す列を選ぶ間に、列 `col` を選ぶ。
fn pick(a: &mut App, col: &str) {
    let i = a
        .sorts_win
        .as_ref()
        .and_then(|w| w.picking.as_ref())
        .and_then(|(cols, _)| cols.iter().position(|c| c == col))
        .expect("列の候補");
    if let Some((_, s)) = a.sorts_win.as_mut().and_then(|w| w.picking.as_mut()) {
        *s = i;
    }
    press(a, KeyCode::Enter);
}

#[test]
fn test_nv_24_sort_window() {
    // [NV-24] ボタンで開く → 「+ 並べ替えを足す」→ due → すぐ due の昇順、ボタンに「↑ due」。
    // Enter で降順、d で外す。窓の外のクリックで閉じる。
    let (_t, mut a) = boot("nv24");
    assert!(screen(&a).contains("並べ替え"), "{}", screen(&a));
    let (x, y) = button_at(&a);
    a.click(x, y);
    assert_eq!(a.mode, Mode::Sorts, "{}", screen(&a));
    assert!(screen(&a).contains("+ 並べ替えを足す"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    pick(&mut a, "due");
    assert_eq!(a.settings.sorts, [("due".to_string(), Dir::Asc)]);
    assert_eq!(labels(&a), ["b.md", "c.md", "a.md"]);
    assert!(screen(&a).contains("↑ due"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.settings.sorts, [("due".to_string(), Dir::Desc)]);
    assert_eq!(labels(&a), ["a.md", "c.md", "b.md"]);
    ch(&mut a, 'd');
    assert!(a.settings.sorts.is_empty());
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md"]);
    a.click(0, a.size.1 - 3);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.sorts_win.is_none());
}

#[test]
fn test_nv_24_two_rules_and_order() {
    // [NV-24] 決まりを2つ(prio・due)、K で順を入れ替える。ボタンは「↑ prio +1」。S のキーでも開く。
    let (_t, mut a) = boot("nv24two");
    ch(&mut a, 'S');
    assert_eq!(a.mode, Mode::Sorts);
    press(&mut a, KeyCode::Enter);
    pick(&mut a, "prio");
    // 足す行へ動かして due も足す。
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    pick(&mut a, "due");
    assert_eq!(
        labels(&a),
        ["b.md", "c.md", "a.md"],
        "prio、同じ prio は due"
    );
    press(&mut a, KeyCode::Esc);
    assert!(screen(&a).contains("↑ prio +1"), "{}", screen(&a));
    ch(&mut a, 'S');
    ch(&mut a, 'j');
    ch(&mut a, 'K');
    assert_eq!(
        a.settings.sorts,
        [
            ("due".to_string(), Dir::Asc),
            ("prio".to_string(), Dir::Asc)
        ]
    );
    assert_eq!(labels(&a), ["b.md", "c.md", "a.md"]);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}
