//! 上の余白とビューの帯の余白の既定は 1(SR-1・NV-23)。画面の地の色が分かる環境でも、既定では半行を描かず
//! 1行の空き行にし、0.5 は設定で選んだときだけ半行にする試験。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #8)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

/// 端末の地の色(試験で与える。分かる環境でも既定は半行にならないことを見る)。
const TERM_BG: [u8; 3] = [12, 12, 12];

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
    a.term_bg = Some(TERM_BG);
    (tmp, a)
}

fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    let b: Buffer = term.backend().buffer().clone();
    text(&b).lines().map(str::to_string).collect()
}

/// 既定の配置: 1行目が空き行、2行目がヘッダー、3行目が空き行、4行目がビューの帯、5行目が空き行、6行目が見出し。
fn assert_default_rows(l: &[String]) {
    let s = l.join("\n");
    assert!(!s.contains('▀') && !s.contains('▄'), "半行の字は無い:\n{s}");
    assert!(l[0].trim().is_empty(), "上の余白は1行の空き行:\n{s}");
    assert!(l[1].contains("notes ▾"), "ヘッダー:\n{s}");
    assert!(l[2].trim().is_empty(), "帯の上の余白は1行の空き行:\n{s}");
    assert!(
        l[3].contains("\\ 検索…") && l[3].contains("絞り込み"),
        "帯:\n{s}"
    );
    assert!(l[4].trim().is_empty(), "帯の下の余白は1行の空き行:\n{s}");
    assert!(l[5].contains("ノート"), "見出し:\n{s}");
}

#[test]
fn test_sr_1_default_one_with_known_terminal_background() {
    // [SR-1][NV-23] 端末の地の色が分かっても、既定(top_padding・bar_padding とも 1)では半行を描かず、
    // ヘッダーの上と帯の上下に1行ずつ空き行を置く(環境で見え方がぶれないように)。
    let (_t, a) = boot("pd1_term", "");
    assert_default_rows(&lines(&a));
}

#[test]
fn test_nv_23_default_one_with_painting_theme() {
    // [NV-23][SR-1] 画面の地を塗るテーマでも、既定は同じ1行の空き行(テーマを見比べても配置が変わらない)。
    let (_t, a) = boot("pd1_theme", "[look]\ntheme = \"nord\"\n");
    assert_default_rows(&lines(&a));
}

#[test]
fn test_nv_23_default_one_half_still_selectable() {
    // [NV-23][SR-1] 0.5 は設定で選べば、地の色が分かるときは前のとおり半行で描く。
    let (_t, a) = boot(
        "pd1_half",
        "[display]\ntop_padding = 0.5\nbar_padding = 0.5\n",
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[0].contains('▀'), "上の余白は半行:\n{s}");
    assert!(l[1].contains("notes ▾"), "ヘッダー:\n{s}");
    assert!(
        l[2].contains('▀') && l[4].contains('▄'),
        "帯の上下は半行:\n{s}"
    );
}
