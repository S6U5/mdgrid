//! 値を選ぶ一覧(CE-3)でも、札の列の候補は表と同じ札の色(SR-35)。文字は同じ。

use super::test_screen::{app_of, buffer, press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_sr_35_value_list_uses_chip_colors() {
    // [SR-35][CE-3] status(札の列)のセルで Enter → 選んでいない候補 doing の文字に、表と同じ札の地の色。
    let tmp = Tmp::new("sr35_list");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo"), ("d", "done")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    // SR-36: 札の色を確かめるので、今までの組(classic)で。
    a.style = mdgrid::style::Style::of(mdgrid::style::Preset::Classic);
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let buf = buffer(&a);
    // 窓の中の doing(縁の3つ右から候補の文字)。
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
    let want = super::chips::style(&a, "doing").and_then(|st| st.bg);
    assert_eq!(Some(buf[(x, y)].bg), want, "{s}");
}
