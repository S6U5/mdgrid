//! モダンな見た目(SR-33): 色のある表示では選びは背景の色、枠とキーはアクセントの色、説明は薄い色。
//! classic と色なしは今まで。どの見た目でも画面の文字は同じ。

use super::app::ColorMode;
use super::startup::Startup;
use super::test_screen::{press, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;

fn boot(name: &str, toml: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), color);
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

fn buffer(a: &App) -> Buffer {
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    t.backend().buffer().clone()
}

fn text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 候補の窓を開いた App。
fn with_list(name: &str, toml: &str, color: ColorMode) -> (Tmp, App) {
    let (t, mut a) = boot(name, toml, color);
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    (t, a)
}

/// 文字 `ch` が最初に出るセル。
fn find(buf: &Buffer, ch: &str) -> (u16, u16) {
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            if buf[(x, y)].symbol() == ch {
                return (x, y);
            }
        }
    }
    panic!("{ch} が無い\n{}", text(buf));
}

/// 文字 `word`(ASCII)がセルに並んで始まる位置。
fn find_word(buf: &Buffer, word: &str) -> (u16, u16) {
    let cs: Vec<String> = word.chars().map(|c| c.to_string()).collect();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width.saturating_sub(cs.len() as u16) {
            if cs
                .iter()
                .enumerate()
                .all(|(k, c)| buf[(x + k as u16, y)].symbol() == c)
            {
                return (x, y);
            }
        }
    }
    panic!("{word} が無い\n{}", text(buf));
}

#[test]
fn test_sr_33_modern_popup_uses_background() {
    // [SR-33] 既定(modern)の色のある表示: 選んだ候補は反転でなく背景の色と太字、枠はアクセントの色。
    let (_t, a) = with_list("sr33_pop", "", ColorMode::Rgb);
    let buf = buffer(&a);
    let (x, y) = find_word(&buf, ">*todo");
    let c = &buf[(x + 2, y)];
    assert!(!c.modifier.contains(Modifier::REVERSED), "反転しない");
    assert!(c.modifier.contains(Modifier::BOLD));
    assert_eq!(c.bg, Color::Rgb(58, 58, 70), "選びの背景");
    let (bx, by) = find(&buf, "╰");
    assert_eq!(buf[(bx, by)].fg, Color::Cyan, "枠はアクセントの色");
}

#[test]
fn test_sr_33_footer_keys_accent_labels_dim() {
    // [SR-33] 下の帯: 反転しない。キーはアクセントの太字、説明は薄い色。
    // SR-36: 帯のキーの形は classic の組(keys)で。
    let (_t, a) = boot(
        "sr33_band",
        "[style]\npreset = \"classic\"\n",
        ColorMode::Rgb,
    );
    let buf = buffer(&a);
    let (x, y) = find_word(&buf, "Enter ");
    let key = &buf[(x, y)];
    assert_eq!(key.fg, Color::Cyan);
    assert!(key.modifier.contains(Modifier::BOLD));
    assert!(!key.modifier.contains(Modifier::REVERSED));
    let label = &buf[(x + 6, y)];
    assert_eq!(label.fg, Color::DarkGray, "説明は薄い色");
}

#[test]
fn test_sr_33_table_selection_is_tinted() {
    // [SR-33] 表: 今の行は背景の色、今のセルは札(アクセントの背景)。反転しない。
    // SR-36: 選びの形は classic の組(fill)で。
    let (_t, a) = boot(
        "sr33_table",
        "[style]\npreset = \"classic\"\n",
        ColorMode::Rgb,
    );
    let buf = buffer(&a);
    let y = view::data_y(&a) as u16;
    let row: Vec<_> = (0..buf.area.width).map(|x| buf[(x, y)].clone()).collect();
    assert!(row.iter().all(|c| !c.modifier.contains(Modifier::REVERSED)));
    assert!(row.iter().any(|c| c.bg == Color::Cyan), "今のセルは札");
    assert!(
        row.iter().any(|c| c.bg == Color::Rgb(58, 58, 70)),
        "今の行は背景の色"
    );
}

#[test]
fn test_sr_33_classic_and_no_color_keep_reverse() {
    // [SR-33] look = "classic" と色なしは今まで(反転)。
    for (name, toml, color) in [
        ("sr33_classic", "look = \"classic\"\n", ColorMode::Rgb),
        ("sr33_nocolor", "", ColorMode::None),
    ] {
        let (_t, a) = with_list(name, toml, color);
        let buf = buffer(&a);
        let (x, y) = find_word(&buf, ">*todo");
        assert!(
            buf[(x + 2, y)].modifier.contains(Modifier::REVERSED),
            "{name}: 反転"
        );
        if color == ColorMode::None {
            let colored = (0..buf.area.height).any(|yy| {
                (0..buf.area.width).any(|xx| {
                    let c = &buf[(xx, yy)];
                    c.fg != Color::Reset || c.bg != Color::Reset
                })
            });
            assert!(!colored, "色なしは色の指定が無い");
        }
    }
}

#[test]
fn test_sr_33_text_is_the_same_in_every_look() {
    // [SR-33][SR-26] どの見た目でも画面の文字は同じ(表・候補の窓)。
    let (_t, modern) = with_list("sr33_same_m", "", ColorMode::Rgb);
    let (_u, classic) = with_list("sr33_same_c", "look = \"classic\"\n", ColorMode::Rgb);
    let (_v, plain) = with_list("sr33_same_n", "", ColorMode::None);
    let m = text(&buffer(&modern));
    assert_eq!(m, text(&buffer(&classic)));
    assert_eq!(m, text(&buffer(&plain)));
}

#[test]
fn test_sr_33_bad_value_warns() {
    // [SR-33] 知らない値は警告にして既定(modern)。
    let (c, w) = mdgrid::config::parse("[look]\nmode = \"fancy\"\n").unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(!c.resolved().classic);
}
