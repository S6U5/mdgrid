//! 窓の枠の文字(SR-32): 既定はつながった罫線、`borders = "ascii"` と ambiguous_wide(CV-6)では ASCII。

use super::startup::Startup;
use super::test_screen::{press, screen, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

fn boot(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// status の列で候補のリストを開いた画面。
fn list_screen(a: &mut App) -> String {
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(a, KeyCode::Enter);
    screen(a)
}

#[test]
fn test_sr_32_rounded_by_default() {
    // [SR-32] 既定は、角の丸いつながった罫線。
    let (_t, mut a) = boot("sr32_round", "");
    let s = list_screen(&mut a);
    assert!(s.contains('│') && s.contains('╰'), "{s}");
    assert!(!s.contains("|>") && !s.contains("+-"), "{s}");
    // 操作の一覧も同じ枠。
    press(&mut a, KeyCode::Esc);
    super::test_screen::ch(&mut a, 'x');
    let s = screen(&a);
    assert!(s.contains('╭') && s.contains('╯'), "{s}");
}

#[test]
fn test_sr_32_ascii_by_setting() {
    // [SR-32] borders = "ascii" なら + - |。
    let (_t, mut a) = boot("sr32_ascii", "[look.style]\nframes = \"ascii\"\n");
    let s = list_screen(&mut a);
    assert!(s.contains("|>") && s.contains('+'), "{s}");
    assert!(!s.contains('│') && !s.contains('╰'), "{s}");
}

#[test]
fn test_sr_32_ascii_when_ambiguous_wide() {
    // [SR-32][CV-6] あいまいな幅を2とする設定では、列がずれないように ASCII。
    let (_t, mut a) = boot("sr32_wide", "[terminal]\nambiguous_wide = true\n");
    let s = list_screen(&mut a);
    assert!(s.contains("|>") && !s.contains('│'), "{s}");
}

#[test]
fn test_sr_32_bad_value_warns() {
    // [SR-32] 知らない値は警告にして既定(罫線)。
    let (c, warnings) = mdgrid::config::parse("[look.style]\nframes = \"double\"\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(c.resolved().style.frames, mdgrid::style::Frames::Rounded);
    let _ = Config::default();
}
