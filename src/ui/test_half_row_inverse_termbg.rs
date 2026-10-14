//! テーマが画面の地を塗らない既定でも、端末の地の色が分かれば(起動時の OSC 11 の答え。試験では
//! `App::term_bg` で与える)、余白の端数を半行で描く(NV-23・SR-1)試験。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #7)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::Terminal;

/// 端末の地の色(試験で与える)。
const TERM_BG: [u8; 3] = [12, 12, 12];

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
    a.term_bg = Some(TERM_BG);
    (tmp, a)
}

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

fn lines(b: &Buffer) -> Vec<String> {
    text(b).lines().map(str::to_string).collect()
}

/// 行がその字だけで埋まっているか(空白は除く)。
fn filled(l: &str, c: char) -> bool {
    let t: Vec<char> = l.chars().filter(|x| *x != ' ').collect();
    t.len() >= 40 && t.iter().all(|x| *x == c)
}

const SCREEN: Color = Color::Rgb(TERM_BG[0], TERM_BG[1], TERM_BG[2]);

#[test]
fn test_nv_23_half_termbg_default_layout() {
    // [SR-1][NV-23] 色ありの既定で端末の地の色が分かる → 1行目は上の余白の半行(`▀`)、2行目がヘッダー、3行目が
    // 帯の上の半行(`▀`)、4行目がビューの帯、5行目が帯の下の半行(`▄`)、6行目が列の見出し。半行のセルの地は
    // 帯(ヘッダー)の行と同じ帯の地の色、字の色は端末の地の色。
    let (_t, a) = boot(
        "tbgdef",
        // 2026-10-12-padding-default-one で既定は 1 になったので、半行は設定で選ぶ。
        "[display]\ntop_padding = 0.5\nbar_padding = 0.5\n",
        ColorMode::Rgb,
    );
    let b = buf(&a);
    let l = lines(&b);
    let s = l.join("\n");
    assert!(filled(&l[0], '▀'), "上の余白:\n{s}");
    assert!(l[1].contains("notes ▾"), "ヘッダー:\n{s}");
    assert!(filled(&l[2], '▀'), "帯の上の余白:\n{s}");
    assert!(
        l[3].contains("\\ 検索…") && l[3].contains("絞り込み"),
        "帯:\n{s}"
    );
    assert!(filled(&l[4], '▄'), "帯の下の余白:\n{s}");
    assert!(l[5].contains("ノート"), "見出し:\n{s}");
    let head = b[(79u16, 1u16)].bg;
    let bar = b[(79u16, 3u16)].bg;
    assert!(
        head != Color::Reset && head == bar,
        "ヘッダーと帯に同じ地の色: {head:?} {bar:?}"
    );
    for (y, band) in [(0u16, head), (2, bar), (4, bar)] {
        for x in [0u16, 40, 79] {
            assert_eq!(
                b[(x, y)].bg,
                band,
                "{}行目の桁 {x} のセルの地は帯の色\n{s}",
                y + 1
            );
            assert_eq!(
                b[(x, y)].fg,
                SCREEN,
                "{}行目の桁 {x} の字の色は端末の地の色\n{s}",
                y + 1
            );
        }
    }
}

#[test]
fn test_nv_23_half_termbg_one_and_a_half() {
    // [NV-23] `top_padding = 0`・`bar_padding = 1.5` → 1行目がヘッダー、2行目が空き、3行目が `▀`、4行目が帯、
    // 5行目が `▄`、6行目が空き、7行目が見出し。
    let (_t, a) = boot(
        "tbg15",
        "[display]\ntop_padding = 0\nbar_padding = 1.5\n",
        ColorMode::Rgb,
    );
    let b = buf(&a);
    let l = lines(&b);
    let s = l.join("\n");
    assert!(l[0].contains("notes ▾"), "{s}");
    assert!(l[1].trim().is_empty(), "{s}");
    assert!(filled(&l[2], '▀'), "{s}");
    assert!(l[3].contains("\\ 検索…"), "{s}");
    assert!(filled(&l[4], '▄'), "{s}");
    assert!(l[5].trim().is_empty(), "{s}");
    assert!(l[6].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_half_termbg_no_color_still_rounds_up() {
    // [NV-23] 端末の地の色が分かっても、色を使わない表示では今のとおり端数を1行の空き行に切り上げる。
    let (_t, a) = boot("tbgnocolor", "", ColorMode::None);
    let s = lines(&buf(&a)).join("\n");
    assert!(!s.contains('▄') && !s.contains('▀'), "{s}");
}

#[test]
fn test_nv_23_half_termbg_rgb_from_osc11() {
    // [NV-23] 端末の答え(OSC 11)から地の色を読む: 4桁と2桁の16進、読めない答え。
    use super::termbg::rgb_from_osc11;
    assert_eq!(
        rgb_from_osc11("\x1b]11;rgb:0c0c/1e1e/ffff\x1b\\"),
        Some([12, 30, 255])
    );
    assert_eq!(
        rgb_from_osc11("\x1b]11;rgb:0c/1e/ff\x07"),
        Some([12, 30, 255])
    );
    assert_eq!(rgb_from_osc11("\x1b[?62c"), None);
}
