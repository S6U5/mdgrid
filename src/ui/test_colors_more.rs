//! 値の色(SR-41)はどの札の形でも効く。色の上書き(SR-40)は色を使わない表示では効かない。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;
use ratatui::style::Color;

fn boot(name: &str, color: ColorMode, cfg: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, s) in [("a", "done"), ("b", "todo"), ("c", "done")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, color);
    let (c, w) = mdgrid::config::parse(cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.refresh_if_needed();
    a.col = a.cols.iter().position(|c| c == "x").unwrap();
    (tmp, a)
}

#[test]
fn test_sr_41_value_color_in_every_chip_shape() {
    // [SR-41] chip・soft・pill の札も、決めた値の色を地にする。
    for status in ["chip", "soft", "pill"] {
        let cfg = format!(
            "nerd_font = true\n[style]\nstatus = \"{status}\"\n[colors.values]\nDone = \"#2f9e66\"\n"
        );
        let (_t, a) = boot(&format!("sr41_{status}"), ColorMode::Rgb, &cfg);
        let y = view::data_y(&a) as u16;
        let buf = buffer(&a);
        let x = (0..buf.area.width.saturating_sub(4))
            .find(|&x| (x..x + 4).map(|i| buf[(i, y)].symbol()).collect::<String>() == "done")
            .unwrap_or_else(|| panic!("{status}: done が無い\n{}", screen(&a)));
        assert_eq!(buf[(x, y)].bg, Color::Rgb(0x2f, 0x9e, 0x66), "{status}");
    }
}

#[test]
fn test_sr_40_no_color_ignores_colors() {
    // [SR-40] --no-color(色なし)では [colors] も [colors.values] も効かない(色の指定が無い)。
    let (_t, a) = boot(
        "sr40_nocolor",
        ColorMode::None,
        "theme = \"sumi\"\n[colors]\naccent = \"red\"\n[colors.values]\ndone = \"green\"\n",
    );
    let buf = buffer(&a);
    let colored = (0..buf.area.height).any(|y| {
        (0..buf.area.width)
            .any(|x| buf[(x, y)].fg != Color::Reset || buf[(x, y)].bg != Color::Reset)
    });
    assert!(!colored);
}
