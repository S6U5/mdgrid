//! 詳細の画面(NV-6)のモダンな見た目(SR-33): 選んでいる項目の行は背景の色。classic は太字のまま。

use super::keymap::Mode;
use super::test_screen::{app_of, buffer, ch, Tmp};
use super::*;

#[test]
fn test_sr_33_detail_selected_row_tinted() {
    // [SR-33] K で詳細 → 選んでいる項目(先頭の status)の行の地は、選びの背景の色。
    let tmp = Tmp::new("sr33_detail");
    tmp.write("a.md", "---\nstatus: todo\ndue: 2026-10-01\n---\nbody\n");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    let sel_bg = super::look::look(&a).unwrap().sel_bg;
    let b = buffer(&a);
    let y = (0..b.area.height)
        .find(|&y| {
            let row: String = (0..12).map(|x| b[(x, y)].symbol().to_string()).collect();
            row.contains("status")
        })
        .expect("status の行");
    assert_eq!(b[(2, y)].bg, sel_bg);
    let y2 = (0..b.area.height)
        .find(|&y| {
            let row: String = (0..12).map(|x| b[(x, y)].symbol().to_string()).collect();
            row.contains("due")
        })
        .expect("due の行");
    assert_ne!(b[(2, y2)].bg, sel_bg, "選んでいない行は地の色なし");
}
