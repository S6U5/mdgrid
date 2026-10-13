//! ビューの帯の既定を線なし・上下に余白の3行にし、box は中を塗らず、1行は compact にする(SR-1・NV-23)
//! 受け入れの試験。記録: specs/_changes/2026-10-12-search-box.md(タスク #5)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\n---\n"),
    ("b.md", "---\nstatus: todo\n---\n"),
];

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

/// 画面の行(2026-10-12-bar-padding-half のヘッダーの上の余白の行を除いて、ヘッダーを先頭に数える)。
fn lines(a: &App) -> Vec<String> {
    text(&buf(a))
        .lines()
        .skip(top(a) as usize)
        .map(str::to_string)
        .collect()
}

/// ヘッダーの上の余白の行の数(画面の行は、この分だけ下にずれる)。
fn top(a: &App) -> u16 {
    super::bands::top_rows(a) as u16
}

fn boot(name: &str, toml: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
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

fn col(l: &str, t: &str) -> Option<usize> {
    l.find(t).map(|i| width::width(&l[..i]))
}

#[test]
fn test_nv_23_plain_default_blank_rows_around() {
    // [NV-23][SR-1] 既定 → 2行目と4行目は空き(線も無い)、3行目にタブ・欄・ボタン、5行目が見出し。
    let (_t, a) = boot("plaindef", "", ColorMode::None);
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[1].trim().is_empty(), "2行目は空き:\n{s}");
    assert!(l[3].trim().is_empty(), "4行目は空き:\n{s}");
    for t in [
        "既定の表",
        "\\ 検索…",
        "| 絞り込み",
        "S 並べ替え",
        "分類",
        "列",
    ] {
        assert!(l[2].contains(t), "3行目に「{t}」:\n{s}");
    }
    assert!(!l[2].contains('│'), "線は無い:\n{s}");
    assert!(l[4].contains("ノート"), "見出しは5行目:\n{s}");
}

#[test]
fn test_nv_23_plain_window_right_below_bar() {
    // [NV-23][NV-24] 既定で並べ替えを押す → 窓の上の縁は4行目(帯の下の余白の行)か5行目。帯の真ん中の行は隠さない。
    let (_t, mut a) = boot("plainwin", "", ColorMode::None);
    let l = lines(&a);
    let x = col(&l[2], "並べ替え").unwrap();
    a.click(x as u16, 2 + top(&a));
    assert_eq!(a.mode, Mode::Sorts);
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[2].contains("S 並べ替え"), "帯の行はそのまま:\n{s}");
    let top = (3..6)
        .find(|&y| l[y].contains("並べ替え") && (l[y].contains('╭') || l[y].contains('┌')))
        .unwrap_or_else(|| panic!("窓の上の縁が帯のすぐ下に無い:\n{s}"));
    assert!(top <= 4, "{s}");
}

#[test]
fn test_nv_23_box_inside_not_filled() {
    // [NV-23] `buttons = "box"` → 線で囲み、線の中(欄の字の後ろの空き)は地の色で塗らない。囲まない既定では
    // 欄の空きに地の色(か下線)がある。
    let (_t, a) = boot(
        "boxnofill",
        "[display]\nbuttons = \"box\"\n",
        ColorMode::Rgb,
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[1].contains('╭'), "box は線で囲む:\n{s}");
    let b = buf(&a);
    let x = col(&l[2], "検索…").unwrap() as u16 + 4;
    let st = b[(x, 2 + top(&a))].style();
    assert!(
        st.bg.is_none() || st.bg == Some(ratatui::style::Color::Reset),
        "線の中は塗らない: {st:?}\n{s}"
    );
    let (_t, a) = boot("plainfill", "", ColorMode::Rgb);
    let l = lines(&a);
    let b = buf(&a);
    let x = col(&l[2], "検索…").unwrap() as u16 + 4;
    let st = b[(x, 2 + top(&a))].style();
    assert!(
        st.bg.is_some_and(|c| c != ratatui::style::Color::Reset)
            || st
                .add_modifier
                .contains(ratatui::style::Modifier::UNDERLINED),
        "既定では欄を地の色か下線で示す: {st:?}"
    );
}

#[test]
fn test_nv_23_compact_one_row() {
    // [NV-23] `bar_padding = 0`(2026-10-12-bar-padding で `buttons = "compact"` から替えた)→ 余白なしの1行、
    // 3行目が見出し。
    let (_t, a) = boot("compact", "[display]\nbar_padding = 0\n", ColorMode::None);
    let l = lines(&a);
    let s = l.join("\n");
    assert!(
        l[1].contains("\\ 検索…") && l[1].contains("絞り込み"),
        "{s}"
    );
    assert!(l[2].contains("ノート"), "{s}");
}
