//! モダンな見た目(SR-33)を、ビューの設定とヘルプにも行き渡らせる: 反転を残さず、ヘルプのキーはアクセントの色。

use super::keymap::Mode;
use super::test_screen::{app_of, buffer, ch, Tmp};
use super::*;
use ratatui::style::Modifier;

fn notes(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    tmp
}

fn reversed_cells(a: &App) -> usize {
    let b = buffer(a);
    (0..b.area.height)
        .flat_map(|y| (0..b.area.width).map(move |x| (x, y)))
        .filter(|&(x, y)| b[(x, y)].modifier.contains(Modifier::REVERSED))
        .count()
}

#[test]
fn test_sr_33_no_reverse_left_in_modern() {
    // [SR-33] モダンな見た目では、ビューの設定の画面にも反転が無い。classic では反転のまま(選びの印)。
    let tmp = notes("sr33_rest");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    ch(&mut a, 'o');
    assert_eq!(a.mode, Mode::Settings);
    assert_eq!(reversed_cells(&a), 0);
    let mut c = app_of(&tmp, ColorMode::Rgb);
    let (cfg, _) = mdgrid::config::parse("[look]\nmode = \"classic\"\n").unwrap();
    c.configure(&cfg);
    ch(&mut c, 'o');
    assert!(reversed_cells(&c) > 0, "classic は反転");
}

#[test]
fn test_sr_33_help_keys_accent() {
    // [SR-33] モダンな見た目のヘルプでは、キーの欄(Enter)をアクセントの色にする。文字は同じ。
    let tmp = notes("sr33_help");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let accent = super::look::look(&a).unwrap().accent;
    let b = buffer(&a);
    let hit = (0..b.area.height).find_map(|y| {
        (0..b.area.width.saturating_sub(5)).find_map(|x| {
            let w: String = (x..x + 5).map(|i| b[(i, y)].symbol().to_string()).collect();
            (w == "Enter").then_some((x, y))
        })
    });
    let (x, y) = hit.expect("ヘルプに Enter");
    assert_eq!(b[(x, y)].fg, accent);
}
