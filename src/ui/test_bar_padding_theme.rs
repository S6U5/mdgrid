//! ビューの帯とヘッダーの上の余白の端数(半分の塗りの字 `▄`・`▀` の行)の外側の半分が、テーマが画面の地を
//! 塗るとき(SR-26)は、ほかの行と同じテーマの画面の地の色になる(NV-23・SR-1)試験。
//! 2026-10-12-half-row-inverse で、半行はセルの地を帯の色で塗り、外側の半分を画面の地の色の字で消す描き方に
//! なったので、画面の地の色は字の色(fg)で確かめる。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #6)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::Terminal;

fn boot(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: done\n---\n");
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::Rgb);
    a.resize(80, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

/// 半分の塗りの字 `c` で埋まった行の添字(上から)。
fn half_rows(b: &Buffer, c: &str) -> Vec<u16> {
    (0..b.area.height)
        .filter(|&y| (0..40u16).all(|x| b[(x, y)].symbol() == c))
        .collect()
}

/// 表の下の空きの行(ノートの行の下で、何も描かない行)の地の色。テーマの画面の地。
fn screen_bg(b: &Buffer) -> Option<Color> {
    b[(10u16, 15u16)].bg.into()
}

fn check(name: &str, theme: &str) {
    // 2026-10-12-padding-default-one で既定は 1 になったので、半行は設定で選ぶ。
    let (_t, a) = boot(
        name,
        &format!("[look]\ntheme = \"{theme}\"\n[display]\ntop_padding = 0.5\nbar_padding = 0.5\n"),
    );
    let b = buf(&a);
    let s = text(&b);
    let bg = screen_bg(&b);
    assert!(
        bg.is_some_and(|c| c != Color::Reset),
        "テーマが画面の地を塗る: {bg:?}\n{s}"
    );
    let tops = half_rows(&b, "▄");
    let bottoms = half_rows(&b, "▀");
    assert!(
        !tops.is_empty() && !bottoms.is_empty(),
        "半分の塗りの行:\n{s}"
    );
    for y in tops.into_iter().chain(bottoms) {
        for x in [0u16, 30, 78, 79] {
            assert_eq!(
                Some(b[(x, y)].fg),
                bg,
                "{theme}: 半分の塗りの行 {y} の桁 {x} の外側の半分はテーマの画面の地の色\n{s}"
            );
        }
    }
}

#[test]
fn test_nv_23_half_theme_background_solarized_light() {
    // [NV-23][SR-1][SR-26] 明るいテーマ(solarized-light)→ 上の余白の ▄ の行と帯の上下の ▄・▀ の行の地は、
    // ほかの行と同じテーマの画面の地(端末の地の色の筋が出ない)。
    check("halfsol", "solarized-light");
}

#[test]
fn test_nv_23_half_theme_background_nord() {
    // [NV-23][SR-1][SR-26] 暗いテーマ(nord)でも同じ。
    check("halfnord", "nord");
}

#[test]
fn test_nv_23_half_theme_background_none_without_theme() {
    // [NV-23] テーマが画面の地を塗らない既定 → 2026-10-12-half-row-inverse で、端末の地の色が分からない
    // (試験には端末が無い)ときは半分の塗りの字を使わず、端数を1行の空き行に切り上げる(端末の地の色の筋も
    // 出ない)。半行の行が無く、余白の行の地も塗らない。
    let (_t, a) = boot("halfplain", "");
    let b = buf(&a);
    let s = text(&b);
    let rows: Vec<u16> = half_rows(&b, "▄")
        .into_iter()
        .chain(half_rows(&b, "▀"))
        .collect();
    assert!(rows.is_empty(), "{s}");
    for y in [0u16, 2, 4] {
        assert_eq!(b[(30u16, y)].bg, Color::Reset, "行 {y}\n{s}");
    }
}
