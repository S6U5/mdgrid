//! 色の上書き(SR-40)・値の色(SR-41)・透けない札(SR-36 の solid)の画面。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;
use ratatui::style::Color;

fn boot(name: &str, cfg: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, s) in [
        ("a", "進行中"),
        ("b", "done"),
        ("c", "進行中"),
        ("d", "done"),
    ] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nx: 1\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let (c, w) = mdgrid::config::parse(cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.refresh_if_needed();
    a.col = a.cols.iter().position(|c| c == "x").unwrap(); // status のセルを選ばない
    (tmp, a)
}

/// 画面で文字 `t` の始まる (桁, 行)(見出しより下の行から探す)。
fn find(a: &App, t: &str, from_y: u16) -> (u16, u16) {
    let buf = buffer(a);
    for y in from_y..buf.area.height {
        let cells: Vec<String> = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        if let Some(x) = (0..cells.len()).find(|&x| cells[x..].concat().starts_with(t)) {
            return (x as u16, y);
        }
    }
    panic!("{t} が無い:\n{}", screen(a));
}

#[test]
fn test_sr_40_accent_override() {
    // [SR-40] sumi に accent を上書き → 列の見出しがその色。地は sumi のまま。default テーマでも accent は効く。
    let (_t, a) = boot(
        "sr40_accent",
        "[look]\ntheme = \"sumi\"\n\n[look.colors]\naccent = \"#e0a458\"\n",
    );
    let hy = view::data_y(&a) as u16 - 1;
    let (x, y) = find(&a, "status", hy);
    let buf = buffer(&a);
    assert_eq!(buf[(x, y)].fg, Color::Rgb(0xe0, 0xa4, 0x58));
    let (bx, by) = find(&a, "done", hy + 1);
    assert_eq!(
        buf[(bx - 3, by)].bg,
        Color::Rgb(0x16, 0x17, 0x1b),
        "地は sumi のまま"
    );
    let (_u, d) = boot("sr40_default", "[look.colors]\naccent = \"orange\"\n");
    let (x, y) = find(&d, "status", hy);
    assert_eq!(buffer(&d)[(x, y)].fg, Color::Rgb(234, 140, 52));
}

#[test]
fn test_sr_41_value_colors() {
    // [SR-41] [colors.values] の日本語の値 進行中 → 点がその色。決めていない done は今までの意味の色。
    let (_t, a) = boot(
        "sr41_values",
        "[look.colors.values]\n\"進行中\" = \"#d6a85c\"\n",
    );
    let hy = view::data_y(&a) as u16;
    // 1行目は a(進行中)、2行目は b(done)。全角の字の後ろの桁は空白なので、点の「●」で探す。
    let (x, y) = find(&a, "●", hy);
    let buf = buffer(&a);
    assert_eq!(buf[(x, y)].fg, Color::Rgb(0xd6, 0xa8, 0x5c));
    let (x, y) = find(&a, "● done", hy + 1);
    assert_ne!(buf[(x, y)].fg, Color::Rgb(0xd6, 0xa8, 0x5c));
}

#[test]
fn test_sr_36_solid() {
    // [SR-36] status = "solid" → 値の色をそのまま地に塗った札(淡くない)。nerd_font なら両端が丸い端の字。
    let (_t, a) = boot(
        "sr36_solid",
        "[look.style]\nstatus = \"solid\"\n\n[look.colors.values]\ndone = \"#2f9e66\"\n",
    );
    let hy = view::data_y(&a) as u16;
    let (x, y) = find(&a, "done", hy);
    assert_eq!(
        buffer(&a)[(x, y)].bg,
        Color::Rgb(0x2f, 0x9e, 0x66),
        "値の色そのまま"
    );
    let (_u, b) = boot(
        "sr36_solid_nerd",
        "[terminal]\nnerd_font = true\n\n[look.style]\nstatus = \"solid\"\n",
    );
    assert!(
        screen(&b).contains("\u{e0b6}done\u{e0b4}"),
        "{}",
        screen(&b)
    );
}
