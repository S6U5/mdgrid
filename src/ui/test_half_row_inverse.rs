//! 半行の余白を、帯の色はセルの地の色で塗り、外側を画面の地の色の半分の塗りの字で消す描き方(NV-23)の
//! 受け入れの試験。記録: specs/_changes/2026-10-12-search-box.md(タスク #7)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
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

fn lines(b: &Buffer) -> Vec<String> {
    text(b).lines().map(str::to_string).collect()
}

/// 行 y が、その字だけで埋まっているか(空白は除く)。
fn filled(l: &str, c: char) -> bool {
    let t: Vec<char> = l.chars().filter(|x| *x != ' ').collect();
    t.len() >= 40 && t.iter().all(|x| *x == c)
}

#[test]
fn test_nv_23_half_inverse_with_painted_background() {
    // [NV-23] 画面の地を塗るテーマ(Nord)→ ビューの帯の上の半行は `▀`、下の半行は `▄` で、どちらもセルの地は
    // 帯の行と同じ色、字の色は画面の地の色(表の下の空いた所の地と同じ)。
    let (_t, a) = boot(
        "invnord",
        // 2026-10-12-padding-default-one で既定は 1 になったので、半行は設定で選ぶ。
        "[look]\ntheme = \"nord\"\n[display]\ntop_padding = 0.5\nbar_padding = 0.5\n",
    );
    let b = buf(&a);
    let l = lines(&b);
    let s = l.join("\n");
    let bar = (0..10)
        .find(|&y| l[y].contains("\\ 検索…"))
        .unwrap_or_else(|| panic!("帯が無い:\n{s}"));
    assert!(filled(&l[bar - 1], '▀'), "帯の上の半行は ▀:\n{s}");
    assert!(filled(&l[bar + 1], '▄'), "帯の下の半行は ▄:\n{s}");
    let screen_bg = b[(79, 20)].style().bg;
    let band_bg = b[(79, bar as u16)].style().bg;
    assert_ne!(screen_bg, band_bg, "帯には地の色");
    for y in [bar - 1, bar + 1] {
        let c = &b[(40, y as u16)];
        assert_eq!(c.style().bg, band_bg, "{}行目のセルの地は帯の色", y + 1);
        assert_eq!(
            c.style().fg,
            screen_bg,
            "{}行目の字の色は画面の地の色",
            y + 1
        );
    }
}

#[test]
fn test_nv_23_half_inverse_top_padding_with_painted_background() {
    // [NV-23][SR-1] 同じテーマで、ヘッダーの上の半行(1行目)も `▄` でなく、セルの地はヘッダーの行と同じ色、
    // 外側の上半分を画面の地の色の `▀` で消す。
    let (_t, a) = boot(
        "invtop",
        // 2026-10-12-padding-default-one で既定は 1 になったので、半行は設定で選ぶ。
        "[look]\ntheme = \"nord\"\n[display]\ntop_padding = 0.5\nbar_padding = 0.5\n",
    );
    let b = buf(&a);
    let l = lines(&b);
    let s = l.join("\n");
    assert!(filled(&l[0], '▀'), "上の半行は ▀:\n{s}");
    assert!(l[1].contains("notes ▾"), "{s}");
    let screen_bg = b[(79, 20)].style().bg;
    let head_bg = b[(79, 1)].style().bg;
    assert_eq!(b[(40, 0)].style().bg, head_bg, "セルの地はヘッダーの色");
    assert_eq!(b[(40, 0)].style().fg, screen_bg, "字の色は画面の地の色");
}

#[test]
fn test_nv_23_half_inverse_unknown_background_rounds_up() {
    // [NV-23] テーマが画面の地を塗らず、端末の地の色も分からない(試験には端末が無い)→ 半分の塗りの字を
    // 使わず、端数を1行の空き行に切り上げる。
    let (_t, a) = boot("invunknown", "");
    let b = buf(&a);
    let s = lines(&b).join("\n");
    assert!(!s.contains('▄') && !s.contains('▀'), "{s}");
}
