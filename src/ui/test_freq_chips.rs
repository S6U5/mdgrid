//! 値の件数の窓(NV-9)でも、札で見せる列の値は表と同じ札の色(SR-35)。文字は同じ。

use super::test_screen::{app_of, buffer, ch, screen, Tmp};
use super::*;

#[test]
fn test_sr_35_freq_values_use_chip_colors() {
    // [SR-35][NV-9] status(札の列)で % → 選んでいない項目の値に、表と同じ札の地の色。色なしでは付けない。
    let tmp = Tmp::new("sr35_freq");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo"), ("d", "done")] {
        tmp.write(&format!("{n}.md"), &format!("---\nstatus: {s}\n---\n"));
    }
    for (color, colored) in [(ColorMode::Rgb, true), (ColorMode::None, false)] {
        let mut a = app_of(&tmp, color);
        // SR-36: 札の色を確かめるので、今までの組(classic)で。
        a.style = mdgrid::style::Style::of(mdgrid::style::Preset::Classic);
        a.col = a.cols.iter().position(|c| c == "status").unwrap();
        ch(&mut a, '%');
        let s = screen(&a);
        // 件数の窓の「doing」の行(件数の多い順なので todo が選ばれ、doing は選んでいない)。
        let y = s
            .lines()
            .position(|l| l.contains("doing") && l.contains('%'))
            .unwrap_or_else(|| panic!("{s}")) as u16;
        let buf = buffer(&a);
        let line: Vec<String> = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        let x = (0..line.len())
            .find(|&x| line[x..].concat().starts_with("doing"))
            .unwrap() as u16;
        let want = super::chips::style(&a, "doing").and_then(|st| st.bg);
        assert_eq!(want.is_some(), colored);
        if colored {
            assert_eq!(Some(buf[(x, y)].bg), want, "{s}");
        } else {
            assert_eq!(buf[(x, y)].bg, ratatui::style::Color::Reset);
        }
    }
}
