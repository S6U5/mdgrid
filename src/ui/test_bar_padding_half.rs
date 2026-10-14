//! ビューの帯とヘッダーの上の余白を 0.5 刻みにし、既定を 0.5 にして、端数を半分の塗りの字で描く
//! (SR-1・NV-23)受け入れの試験。記録: specs/_changes/2026-10-12-search-box.md(タスク #6)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::Terminal;

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

fn lines(a: &App) -> Vec<String> {
    text(&buf(a)).lines().map(str::to_string).collect()
}

#[test]
fn test_sr_1_half_default_layout() {
    // [SR-1][NV-23] 色ありの既定 → 1行目は上の余白、2行目がヘッダー、3行目が帯の上の余白、4行目がビューの帯、
    // 5行目が帯の下の余白、6行目が列の見出し。
    // 2026-10-12-half-row-inverse: テーマが画面の地を塗らず、端末の地の色も分からない(試験には端末が無い)
    // ので、0.5 の余白は1行の空き行に切り上げ、地の色も敷かない。端末の地の色が分かるときの半行の描き方は
    // test_half_row_inverse_termbg.rs で確かめる。
    let (_t, a) = boot("halfdef", "", ColorMode::Rgb);
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[0].trim().is_empty(), "上の余白:\n{s}");
    assert!(l[1].contains("notes ▾"), "ヘッダー:\n{s}");
    assert!(l[2].trim().is_empty(), "帯の上の余白:\n{s}");
    assert!(
        l[3].contains("\\ 検索…") && l[3].contains("絞り込み"),
        "帯:\n{s}"
    );
    assert!(l[4].trim().is_empty(), "帯の下の余白:\n{s}");
    assert!(l[5].contains("ノート"), "見出し:\n{s}");
    assert!(!s.contains('▄') && !s.contains('▀'), "{s}");
    // 帯の地の色は敷かない(ヘッダーの空いた所と、帯の右の端の地は塗らない。ボタンの地は今までどおり)。
    let b = buf(&a);
    let plain = |x: u16, y: u16| matches!(b[(x, y)].style().bg, None | Some(Color::Reset));
    assert!(plain(40, 1), "ヘッダーに地の色を敷かない");
    assert!(plain(79, 3), "帯に地の色を敷かない");
}

#[test]
fn test_sr_1_half_no_color_rounds_up() {
    // [SR-1][NV-23] 色なしの既定 → 0.5 は1行に切り上げ: 1・3・5行目が空き、2行目がヘッダー、4行目が帯、
    // 6行目が見出し。半分の塗りの字は使わない。
    let (_t, a) = boot("halfnocolor", "", ColorMode::None);
    let l = lines(&a);
    let s = l.join("\n");
    for y in [0, 2, 4] {
        assert!(l[y].trim().is_empty(), "{}行目は空き:\n{s}", y + 1);
    }
    assert!(l[1].contains("notes ▾") && l[3].contains("\\ 検索…"), "{s}");
    assert!(l[5].contains("ノート"), "{s}");
    assert!(!s.contains('▄') && !s.contains('▀'), "{s}");
}

#[test]
fn test_sr_1_half_zero_paddings() {
    // [SR-1][NV-23] `top_padding = 0`・`bar_padding = 0` → 1行目がヘッダー、2行目が帯、3行目が見出し。
    let (_t, a) = boot(
        "halfzero",
        "[display]\ntop_padding = 0\nbar_padding = 0\n",
        ColorMode::Rgb,
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[0].contains("notes ▾"), "{s}");
    assert!(l[1].contains("\\ 検索…"), "{s}");
    assert!(l[2].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_half_one_and_a_half() {
    // [NV-23] `top_padding = 0`・`bar_padding = 1.5` → 1行目がヘッダー、2・3行目が空き、4行目が帯、
    // 5・6行目が空き、7行目が見出し(2026-10-12-half-row-inverse: 端末の地の色が分からないので、端数の半行は
    // 1行の空き行に切り上げる。半行で描くときは test_half_row_inverse_termbg.rs)。
    let (_t, a) = boot(
        "half15",
        "[display]\ntop_padding = 0\nbar_padding = 1.5\n",
        ColorMode::Rgb,
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[0].contains("notes ▾"), "{s}");
    assert!(l[1].trim().is_empty(), "{s}");
    assert!(l[2].trim().is_empty(), "{s}");
    assert!(l[3].contains("\\ 検索…"), "{s}");
    assert!(l[4].trim().is_empty(), "{s}");
    assert!(l[5].trim().is_empty(), "{s}");
    assert!(l[6].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_half_ambiguous_wide_rounds_up() {
    // [NV-23] あいまいな字を全角として数える設定 → 半分の塗りの字を使わず、端数は1行の空きに切り上げる。
    let (_t, a) = boot(
        "halfamb",
        "[terminal]\nambiguous_wide = true\n",
        ColorMode::Rgb,
    );
    let s = lines(&a).join("\n");
    assert!(!s.contains('▄') && !s.contains('▀'), "{s}");
    let l = lines(&a);
    assert!(l[1].contains("notes ▾") && l[3].contains("\\ 検索…"), "{s}");
}
