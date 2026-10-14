//! 0.5 を設定で選んでも、色を使わない表示と、あいまいな字を全角として数える設定では、端末の地の色が分かって
//! いても半分の塗りの字を使わず、端数を1行の空き行に切り上げる(NV-23・SR-1)試験。既定が 1 になった
//! (2026-10-12-padding-default-one)ので、0.5 は設定で明示する。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #8)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const HALF: &str = "[display]\ntop_padding = 0.5\nbar_padding = 0.5\n";

fn boot(name: &str, toml: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: done\n---\n");
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), color);
    a.resize(80, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: color == ColorMode::None,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    // 地の色は分かっている(切り上げの理由が、色なし・あいまいな字の設定のほうであることを見る)。
    a.term_bg = Some([12, 12, 12]);
    (tmp, a)
}

fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .map(str::to_string)
        .collect()
}

fn assert_rounded_up(l: &[String]) {
    let s = l.join("\n");
    assert!(
        !s.contains('▀') && !s.contains('▄'),
        "半分の塗りの字は無い:\n{s}"
    );
    for y in [0, 2, 4] {
        assert!(l[y].trim().is_empty(), "{}行目は空き行:\n{s}", y + 1);
    }
    assert!(l[1].contains("notes ▾"), "2行目がヘッダー:\n{s}");
    assert!(l[3].contains("\\ 検索…"), "4行目がビューの帯:\n{s}");
    assert!(l[5].contains("ノート"), "6行目が見出し:\n{s}");
}

#[test]
fn test_sr_1_half_explicit_no_color_rounds_up() {
    // [SR-1][NV-23] 色なしで `top_padding = 0.5`・`bar_padding = 0.5` → 上下に1行ずつ空く。
    let (_t, a) = boot("halfx_nocolor", HALF, ColorMode::None);
    assert_rounded_up(&lines(&a));
}

#[test]
fn test_nv_23_half_explicit_ambiguous_wide_rounds_up() {
    // [NV-23][SR-1] あいまいな字を全角として数える設定で 0.5 → 半分の塗りの字を使わず、1行ずつ空く。
    let toml = format!("{HALF}[terminal]\nambiguous_wide = true\n");
    let (_t, a) = boot("halfx_amb", &toml, ColorMode::Rgb);
    assert_rounded_up(&lines(&a));
}

#[test]
fn test_nv_23_half_explicit_known_background_draws_half() {
    // [NV-23] 対照: 同じ 0.5 で、色あり・地の色が分かる → 半分の塗りの字で描く(上の2本の切り上げが、色なしと
    // あいまいな字の設定によることを示す)。
    let (_t, a) = boot("halfx_rgb", HALF, ColorMode::Rgb);
    let s = lines(&a).join("\n");
    assert!(s.contains('▀') && s.contains('▄'), "{s}");
}
