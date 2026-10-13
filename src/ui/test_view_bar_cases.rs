//! ビューの帯(SR-1・SR-20・SR-34)で、タブと検索の欄の片方だけを出す場合の試験。照合で、どの試験にも
//! 出てこないと分かった2つの場合を足す。記録: specs/_changes/2026-10-11-view-bar.md(タスク #7)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\n---\n"),
    ("b.md", "---\nstatus: todo\n---\n"),
];

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

/// 画面の行(2026-10-12-bar-padding-half のヘッダーの上の余白の行を除いて、ヘッダーを先頭に数える)。
fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .skip(super::bands::top_rows(a))
        .map(str::to_string)
        .collect()
}

#[test]
fn test_sr_34_auto_single_view_keeps_search_bar() {
    // [SR-34][SR-1] auto でビューが1つ → タブは出さず、ヘッダーにもビューの名前を足さない。検索の欄を
    // 出しているので、ビューの帯の行は残り(欄とボタン)、見出しは3行目。
    let (_t, a) = boot_config("sr34auto", "[display]\ntabs = \"auto\"\n");
    let l = lines(&a);
    let s = l.join("\n");
    // 2026-10-12-search-box・view-bar-plain: 3行の帯では、字は真ん中の行、見出しは帯の下。
    let m = super::bands::mid_row(&a) - super::bands::top_rows(&a);
    let head = 1 + super::bands::tab_rows(&a);
    assert!(!l[0].contains("既定の表"), "ヘッダーにビューの名前を足さない:\n{s}");
    assert!(!l[m].contains("既定の表"), "タブは出さない:\n{s}");
    for t in ["\\ 検索", "絞り込み", "並べ替え"] {
        assert!(l[m].contains(t), "帯の行に「{t}」:\n{s}");
    }
    assert!(l[head].contains("ノート"), "見出しは帯の下:\n{s}");
}

#[test]
fn test_sr_20_search_hidden_keeps_tabs_and_buttons() {
    // [SR-20][SR-1] タブを出し、検索の欄だけ隠す → ビューの帯の行は残り、タブとボタンが並ぶ。欄は無い。
    let (_t, a) = boot_config(
        "sr20nosearch",
        "[display]\ntabs = \"always\"\nsearch_bar = false\n",
    );
    let l = lines(&a);
    let s = l.join("\n");
    assert!(l[1].contains("既定の表"), "タブは出す:\n{s}");
    for t in ["絞り込み", "並べ替え"] {
        assert!(l[1].contains(t), "2行目に「{t}」:\n{s}");
    }
    assert!(!l[1].contains("\\ 検索"), "検索の欄は出さない:\n{s}");
    assert!(l[2].contains("ノート"), "見出しは3行目:\n{s}");
}
