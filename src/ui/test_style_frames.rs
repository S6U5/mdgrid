//! 窓の枠の形(SR-36 の frames)でも、モダンな見た目の枠はアクセントの色(SR-33)。

use super::test_screen::{app_of, buffer, press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

#[test]
fn test_sr_33_frame_accent_in_every_frame_shape() {
    // [SR-33][SR-36] frames = square・heavy でも、候補の窓の角と縦の線はアクセントの色。
    let tmp = Tmp::new("sr33_frames");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    for (frames, corner, side) in [("square", "└", "│"), ("heavy", "┗", "┃")] {
        let mut a = app_of(&tmp, ColorMode::Rgb);
        let (c, w) = mdgrid::config::parse(&format!("[style]\nframes = \"{frames}\"\n")).unwrap();
        assert!(w.is_empty(), "{w:?}");
        a.configure(&c);
        a.col = a.cols.iter().position(|c| c == "status").unwrap();
        press(&mut a, KeyCode::Enter);
        let s = screen(&a);
        let buf = buffer(&a);
        let find = |sym: &str| {
            (0..buf.area.height)
                .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
                .find(|&(x, y)| buf[(x, y)].symbol() == sym)
                .unwrap_or_else(|| panic!("{frames}: {sym} が無い\n{s}"))
        };
        let (x, y) = find(corner);
        assert_eq!(buf[(x, y)].fg, Color::Cyan, "{frames}: 角");
        let (x, y) = find(side);
        assert_eq!(buf[(x, y)].fg, Color::Cyan, "{frames}: 縦の線");
    }
}
