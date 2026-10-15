//! 横のホイールと shift+ホイールの試験(変更 2026-10-15-horizontal-wheel。SR-6・SR-3)。
use super::keymap::Mode;
use super::test_screen::*;
use super::*;

/// 30字の値の列を5つ(a1 a2 a3 b4 b5)持つノート。80桁には収まらない。
fn wide(name: &str) -> (Tmp, App) {
    let v = "x".repeat(30);
    let note = format!("---\na1: {v}\na2: {v}\na3: {v}\nb4: {v}\nb5: {v}\n---\n");
    make(name, &[("a.md", note.as_str())])
}

#[test]
fn test_sr_6_hwheel_moves_columns_in_table() {
    // [SR-6] 表で横のホイールを右 → 右の列、左 → 左の列、一番左で左 → そのまま。
    let (_t, mut a) = wide("hwheel_table");
    let first = a.col;
    a.hwheel(true);
    assert_eq!(a.col, first + 1);
    a.hwheel(false);
    assert_eq!(a.col, first);
    a.hwheel(false);
    assert_eq!(a.col, first, "一番左で左はそのまま");
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_6_hwheel_keeps_selected_visible() {
    // [SR-6] [SR-3] 列が画面に収まらない表で右へ流し続ける → 選んだセルが見える位置まで表が流れる。
    let (_t, mut a) = wide("hwheel_scroll");
    for _ in 0..10 {
        a.hwheel(true);
        assert!(view::selected_drawn(&a), "列 {}", a.cols[a.col]);
    }
    assert_eq!(a.cols[a.col], "b5");
    assert!(a.left > 0, "表が右へ流れた");
}

#[test]
fn test_sr_6_hwheel_shift_wheel_in_table() {
    // [SR-6] 表で shift+ホイールの下 → 右の列、上 → 左の列。行は動かない。
    let (_t, mut a) = make(
        "hwheel_shift",
        &[
            ("a.md", "---\ntitle: a\nstatus: x\n---\n"),
            ("b.md", "---\ntitle: b\nstatus: y\n---\n"),
        ],
    );
    let (row, col) = (a.row, a.col);
    a.shift_wheel(true);
    assert_eq!((a.row, a.col), (row, col + 1));
    a.shift_wheel(false);
    assert_eq!((a.row, a.col), (row, col));
}

#[test]
fn test_sr_6_hwheel_ignored_outside_table() {
    // [SR-6] ヘルプで横のホイール → 何も変わらない。shift+ホイールの下 → ヘルプが下へ流れる(縦のまま)。
    let (_t, mut a) = make("hwheel_help", &[("a.md", "---\ntitle: a\n---\n")]);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let col = a.col;
    a.hwheel(true);
    a.hwheel(false);
    assert_eq!(a.mode, Mode::Help);
    assert_eq!(a.col, col);
    assert_eq!(a.help.as_ref().unwrap().top, 0);
    a.shift_wheel(true);
    assert_eq!(a.mode, Mode::Help);
    assert_eq!(a.help.as_ref().unwrap().top, 1);
    assert_eq!(a.col, col);
}
