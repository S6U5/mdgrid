//! 窓(候補の一覧)の値も、部品の形(SR-36)の色に合わせる。文字は同じ。

use super::test_screen::{app_of, buffer, press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

#[test]
fn test_sr_36_value_list_follows_style() {
    // [SR-36] 既定(sumi・status = "dot")で status のセルで Enter → 候補 doing は値の色の文字で、札の地は無い。
    let tmp = Tmp::new("sr36_list");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo"), ("d", "done")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let buf = buffer(&a);
    let hit = (0..buf.area.height).find_map(|y| {
        (3..buf.area.width).find_map(|x| {
            let side = buf[(x - 3, y)].symbol();
            let word: String = (x..(x + 5).min(buf.area.width))
                .map(|i| buf[(i, y)].symbol().to_string())
                .collect();
            ((side == "│" || side == "|") && word == "doing").then_some((x, y))
        })
    });
    let (x, y) = hit.unwrap_or_else(|| panic!("窓の doing が無い:\n{s}"));
    let chip = super::chips::style(&a, "doing").and_then(|st| st.bg);
    assert_ne!(Some(buf[(x, y)].bg), chip, "札の地は無い: {s}");
    assert_ne!(buf[(x, y)].fg, Color::Reset, "値の色の文字: {s}");
}
