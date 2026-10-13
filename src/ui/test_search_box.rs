//! 検索の欄と絞り込み・並べ替えのボタンを線で四角く囲む3行のビューの帯(SR-1・NV-23)の受け入れの試験。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #1)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\n---\n"),
    ("b.md", "---\nstatus: todo\n---\n"),
];

/// 画面の行(2026-10-12-bar-padding-half のヘッダーの上の余白の行を除いて、ヘッダーを先頭に数える)。
fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .skip(top(a) as usize)
        .map(str::to_string)
        .collect()
}

/// ヘッダーの上の余白の行の数(画面の行は、この分だけ下にずれる)。
fn top(a: &App) -> u16 {
    super::bands::top_rows(a) as u16
}

fn boot_config(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
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

/// 行 `y` で文字 `t` の始まる桁(全角は2桁で数える)。
fn col(l: &str, t: &str) -> Option<usize> {
    l.find(t).map(|i| width::width(&l[..i]))
}

#[test]
fn test_nv_23_box_default_three_rows() {
    // [NV-23][SR-1] 80×24 で `buttons = "box"`(2026-10-12-view-bar-plain で既定は plain になった)→ 2〜4行目がビューの帯。2行目と4行目に囲みの線の角が3つずつ(検索の欄・
    // 絞り込み・並べ替え)、3行目に「既定の表」・「\ 検索…」・「| 絞り込み」・「S 並べ替え」・分類・列、
    // 5行目が列の見出し。
    let (_t, a) = boot_config("boxdef", "[display]\nbuttons = \"box\"\n");
    let l = lines(&a);
    let s = l.join("\n");
    assert_eq!(l[1].matches('╭').count(), 3, "上の線:\n{s}");
    assert_eq!(l[3].matches('╰').count(), 3, "下の線:\n{s}");
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
    assert!(l[2].contains('│'), "囲みの縦の線:\n{s}");
    assert!(l[4].contains("ノート"), "見出しは5行目:\n{s}");
}

#[test]
fn test_nv_23_box_surrounds_search_field() {
    // [NV-23] 検索の欄の囲み: 上の線の角と下の線の角が、真ん中の行の縦の線と同じ桁で、欄の字はその内側。
    let (_t, a) = boot_config("boxfield", "[display]\nbuttons = \"box\"\n");
    let l = lines(&a);
    let s = l.join("\n");
    let left = col(&l[1], "╭").unwrap_or_else(|| panic!("{s}"));
    assert_eq!(col(&l[3], "╰"), Some(left), "下の角も同じ桁:\n{s}");
    let mid: Vec<char> = l[2].chars().collect();
    let mut x = 0;
    let mut at = None;
    for c in &mid {
        if x == left {
            at = Some(*c);
            break;
        }
        x += width::width(&c.to_string());
    }
    assert_eq!(at, Some('│'), "縦の線が同じ桁:\n{s}");
    let field = col(&l[2], "\\ 検索").unwrap();
    assert!(field > left, "欄の字は囲みの内側:\n{s}");
}

#[test]
fn test_nv_23_box_typing_shows_word_inside() {
    // [NV-23][NV-2] `\` で「a」と打つ → 3行目の囲みの中に語、5行目は見出しのまま。
    let (_t, mut a) = boot_config("boxtype", "[display]\nbuttons = \"box\"\n");
    ch(&mut a, '\\');
    typing(&mut a, "do");
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[2].contains("do"), "{s}");
    assert!(l[4].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_box_window_below_bottom_line() {
    // [NV-23][NV-24] 並べ替えのボタンを押す → 窓の上の縁は囲みの下の線(4行目)のすぐ下の5行目。
    let (_t, mut a) = boot_config("boxwin", "[display]\nbuttons = \"box\"\n");
    let l = lines(&a);
    let x = col(&l[2], "並べ替え").unwrap();
    a.click(x as u16, 2 + top(&a));
    assert_eq!(a.mode, Mode::Sorts, "{}", lines(&a).join("\n"));
    let l = lines(&a);
    assert!(
        l[4].contains("並べ替え") && (l[4].contains('╭') || l[4].contains('┌')),
        "窓の上の縁が5行目:\n{}",
        l.join("\n")
    );
}

#[test]
fn test_nv_23_box_click_any_row_of_box() {
    // [NV-23] 囲みの上の線を押しても、そのボタン(絞り込み)の窓が開く。
    let (_t, mut a) = boot_config("boxclick", "[display]\nbuttons = \"box\"\n");
    let l = lines(&a);
    let x = col(&l[2], "絞り込み").unwrap();
    a.click(x as u16, 1 + top(&a));
    assert_eq!(a.mode, Mode::FilterWin, "{}", lines(&a).join("\n"));
}

#[test]
fn test_nv_23_flat_setting_one_row() {
    // [NV-23] `display.bar_padding = 0`(2026-10-12-bar-padding で `buttons = "flat"`・"compact" から替えた)→ 囲みが無く、ビューの帯は2行目だけ、3行目が見出し。
    let (_t, a) = boot_config("boxflat", "[display]\nbar_padding = 0\n");
    let l = lines(&a);
    let s = l.join("\n");
    assert!(!l[1].contains('╭') && !l[1].contains('│'), "{s}");
    assert!(
        l[1].contains("\\ 検索…") && l[1].contains("絞り込み"),
        "{s}"
    );
    assert!(l[2].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_box_low_terminal_one_row() {
    // [NV-23] 高さ 14(16行に満たない)→ box でも囲まず1行。
    let (_t, mut a) = boot_config("boxlow", "[display]\nbuttons = \"box\"\n");
    a.resize(80, 14);
    let l = lines(&a);
    let s = l.join("\n");
    assert!(!l[1].contains('╭'), "{s}");
    assert!(l[1].contains("\\ 検索…"), "{s}");
    assert!(l[2].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_box_off_when_search_hidden() {
    // [NV-23][SR-20] 検索の欄を出さない → 囲みも出さず、ビューの帯は1行(タブとボタン)。
    let (_t, a) = boot_config(
        "boxnosearch",
        "[display]\ntabs = \"always\"\nsearch_bar = false\nbuttons = \"box\"\n",
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(!l[1].contains('╭'), "{s}");
    assert!(
        l[1].contains("既定の表") && l[1].contains("絞り込み"),
        "{s}"
    );
    assert!(l[2].contains("ノート"), "{s}");
}
