//! [SR-28] テーマでは、ためる変更のセルの文字・差分の足した行と消した行の文字・一致の強調の地も
//! テーマの色(地で読める色)。テーマなしは今までと同じ色。specs/_changes/2026-10-06-theme-meaning-colors.md。

use super::test_screen::{app_of, buffer, col_named, ctrl, press, typing, Tmp};
use super::*;
use mdgrid::config;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

const SOL_PENDING: Color = Color::Rgb(0xb5, 0x89, 0x00);
const SOL_ADD: Color = Color::Rgb(0x85, 0x99, 0x00);
const SOL_DEL: Color = Color::Rgb(0xdc, 0x32, 0x2f);

fn app(name: &str, theme: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    // 意味の色(SR-28)は今までの見た目(look = "classic")の上で確かめる。
    a.configure(
        &config::parse(&format!(
            "[look]\nmode = \"classic\"\ntheme = \"{theme}\"\n"
        ))
        .unwrap()
        .0,
    );
    (tmp, a)
}

/// `word` が始まるセルの前景(画面のどこかで最初に見つかったもの)。
fn fg_of(buf: &Buffer, word: &str) -> Option<Color> {
    let a = buf.area;
    for y in 0..a.height {
        let line: String = (0..a.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        if let Some(at) = line.find(word) {
            let x = line[..at].chars().count() as u16;
            return Some(buf[(x, y)].fg);
        }
    }
    None
}

/// 1行目の status を `wip` にためる。
fn set_wip(a: &mut App) {
    col_named(a, "status");
    press(a, KeyCode::Enter);
    typing(a, "wip");
    press(a, KeyCode::Enter);
}

#[test]
fn test_sr_28_pending_cell_uses_theme_color() {
    let (_t, mut a) = app("sr28p", "solarized-light");
    set_wip(&mut a);
    assert_eq!(fg_of(&buffer(&a), "wip"), Some(SOL_PENDING));
}

#[test]
fn test_sr_28_diff_lines_use_theme_colors() {
    let (_t, mut a) = app("sr28d", "solarized-light");
    set_wip(&mut a);
    ctrl(&mut a, 's');
    assert!(a.review.is_some());
    let buf = buffer(&a);
    assert_eq!(fg_of(&buf, "+status: wip"), Some(SOL_ADD));
    assert_eq!(fg_of(&buf, "-status: todo"), Some(SOL_DEL));
}

#[test]
fn test_sr_28_highlight_background_uses_theme_color() {
    let (_t, mut a) = app("sr28h", "solarized-light");
    let p = mdgrid::theme::Theme::SolarizedLight.palette().unwrap();
    col_named(&mut a, "status");
    typing(&mut a, "/todo");
    press(&mut a, KeyCode::Enter);
    let buf = buffer(&a);
    let a2 = buf.area;
    let want = Color::Rgb(p.hl_bg[0], p.hl_bg[1], p.hl_bg[2]);
    let found = (0..a2.height).any(|y| (0..a2.width).any(|x| buf[(x, y)].bg == want));
    assert!(found, "強調の地がテーマの色");
    // 地で読める: 文字との明るさの差が大きい。
    let lum = |c: [u8; 3]| 0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64;
    assert!((lum(p.hl_bg) - lum(p.fg)).abs() > 80.0);
}

#[test]
fn test_sr_28_default_theme_keeps_colors() {
    let (_t, mut a) = app("sr28def", "default");
    set_wip(&mut a);
    assert_eq!(fg_of(&buffer(&a), "wip"), Some(Color::Rgb(255, 175, 0)));
}

#[test]
fn test_sr_28_every_theme_has_readable_meaning_colors() {
    // 意味の色は地と明るさがはっきり違う(地に溶けない)。
    let lum = |c: [u8; 3]| 0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64;
    for t in mdgrid::theme::Theme::ALL {
        let Some(p) = t.palette() else { continue };
        for (what, c) in [("add", p.add), ("del", p.del), ("pending", p.pending)] {
            let d = (lum(c) - lum(p.bg)).abs();
            assert!(d > 60.0, "{} の {what} が地に近い: {d:.0}", t.name());
        }
    }
}
