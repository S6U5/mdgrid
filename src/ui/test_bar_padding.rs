//! ビューの帯の上下の余白を行の数の設定 `display.bar_padding` にする(NV-23)受け入れの試験。
//! 記録: specs/_changes/2026-10-12-search-box.md(タスク #5)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn lines_with(name: &str, toml: &str) -> Vec<String> {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: done\n---\n");
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
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| draw(f, &a)).unwrap();
    // 2026-10-12-bar-padding-half: ヘッダーの上の余白の行を除いて、ヘッダーを先頭に数える。
    text(term.backend().buffer())
        .lines()
        .skip(super::bands::top_rows(&a))
        .map(str::to_string)
        .collect()
}

#[test]
fn test_nv_23_bar_padding_zero_is_one_row() {
    // [NV-23] `bar_padding = 0` → 余白なしの1行(2行目に欄、3行目が見出し)。
    let l = lines_with("pad0", "[display]\nbar_padding = 0\n");
    let s = l.join("\n");
    assert!(l[1].contains("\\ 検索…"), "{s}");
    assert!(l[2].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_bar_padding_two_rows_each() {
    // [NV-23] `bar_padding = 2` → 2・3行目と5・6行目が空き、4行目に欄、7行目が見出し。
    let l = lines_with("pad2", "[display]\nbar_padding = 2\n");
    let s = l.join("\n");
    for y in [1, 2, 4, 5] {
        assert!(l[y].trim().is_empty(), "{}行目は空き:\n{s}", y + 1);
    }
    assert!(l[3].contains("\\ 検索…"), "{s}");
    assert!(l[6].contains("ノート"), "{s}");
}

#[test]
fn test_nv_23_box_needs_padding() {
    // [NV-23] `buttons = "box"` で `bar_padding = 0` → 線を描かず1行。
    let l = lines_with("box0", "[display]\nbuttons = \"box\"\nbar_padding = 0\n");
    let s = l.join("\n");
    assert!(!l[1].contains('│') && !l[1].contains('╭'), "{s}");
    assert!(l[2].contains("ノート"), "{s}");
}
