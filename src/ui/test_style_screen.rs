//! 部品の形(SR-36)の画面: 既定の組(sumi)と、部品ごとの上書き。

use super::test_screen::{app_of, buffer, press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};

fn notes(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, status, done, tags) in [
        ("a", "todo", "false", "[ui, perf]"),
        ("b", "doing", "true", "[docs]"),
        ("c", "todo", "false", "[]"),
        ("d", "done", "true", "[ui]"),
    ] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {status}\ndone: {done}\ntags: {tags}\n---\n"),
        );
    }
    tmp
}

fn with(tmp: &Tmp, color: ColorMode, cfg: &str) -> App {
    let mut a = app_of(tmp, color);
    let (c, w) = mdgrid::config::parse(cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.refresh_if_needed();
    a
}

fn row(a: &App, name: &str) -> String {
    screen(a)
        .lines()
        .find(|l| {
            l.get(1..)
                .is_some_and(|r| r.starts_with(&format!("{name} ")))
        })
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{}", screen(a)))
        .to_string()
}

/// 画面の行 `y` で、文字 `text` の始まる桁。
fn find_in(buf: &ratatui::buffer::Buffer, y: u16, text: &str) -> Option<u16> {
    let cells: Vec<String> = (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect();
    (0..cells.len())
        .find(|&x| cells[x..].concat().starts_with(text))
        .map(|x| x as u16)
}

fn row_y(a: &App, name: &str) -> u16 {
    screen(a)
        .lines()
        .position(|l| {
            l.get(1..)
                .is_some_and(|r| r.starts_with(&format!("{name} ")))
        })
        .unwrap() as u16
}

#[test]
fn test_sr_36_default_sumi_screen() {
    // [SR-36] 既定(sumi)の色のある表示: status は点と文字(点だけに色)、tags は `·` で区切る、真偽は ✓、
    // 見出しに型の印なし、見出しの下に線(下線)、今の行は背景の色で今のセルは反転も札もしない。
    let tmp = notes("sr36_sumi");
    let mut a = with(&tmp, ColorMode::Rgb, "");
    a.col = a.cols.iter().position(|c| c == "done").unwrap();
    let b = row(&a, "b");
    assert!(
        b.contains("● doing") && b.contains("✓") && b.contains("docs"),
        "{b}"
    );
    let r = row(&a, "a");
    assert!(r.contains("ui · perf") && r.contains("·"), "{r}");
    let s = screen(&a);
    let head = s.lines().find(|l| l.contains("status")).unwrap();
    assert!(!head.contains("◉") && !head.contains("⋮"), "{head}");
    let buf = buffer(&a);
    // 点だけに色、値の文字は行の色のまま。
    let y = row_y(&a, "b");
    let x = find_in(&buf, y, "● doing").unwrap();
    assert_ne!(buf[(x, y)].fg, Color::Reset, "点に色");
    assert_eq!(buf[(x + 2, y)].bg, buf[(0, y)].bg, "文字に札の地は無い");
    // 見出しの下の線。
    let hy = s.lines().position(|l| l.contains("status")).unwrap() as u16;
    let hx = find_in(&buf, hy, "status").unwrap();
    assert!(buf[(hx, hy)].modifier.contains(Modifier::UNDERLINED));
    // 今の行。
    let sy = view::data_y(&a) as u16;
    let cells: Vec<_> = (0..buf.area.width).map(|x| buf[(x, sy)].clone()).collect();
    assert!(cells
        .iter()
        .all(|c| !c.modifier.contains(Modifier::REVERSED)));
    assert!(
        cells.iter().all(|c| c.bg != Color::Cyan),
        "今のセルを札にしない"
    );
    assert!(
        cells.iter().any(|c| c.bg == Color::Rgb(58, 58, 70)),
        "今の行は背景の色"
    );
}

#[test]
fn test_sr_36_overrides_each_part() {
    // [SR-36] 部品ごとの上書き: status = "chip" は今までの札、tags = "hash"、check = "bracket"、
    // icons = true は見出しの印。
    let tmp = notes("sr36_over");
    let a = with(
        &tmp,
        ColorMode::Rgb,
        "[look.style]\nstatus = \"chip\"\ntags = \"hash\"\ncheck = \"bracket\"\nicons = true\n",
    );
    let b = row(&a, "b");
    assert!(
        b.contains(" doing ") && b.contains("#docs") && b.contains("[x]"),
        "{b}"
    );
    let r = row(&a, "a");
    assert!(r.contains("#ui #perf") && r.contains("[ ]"), "{r}");
    let s = screen(&a);
    assert!(s.contains("◉ status"), "{s}");
}

#[test]
fn test_sr_36_preset_saas_tints_and_segments() {
    // [SR-36] preset = "saas": status は淡い地の札(点つき)、リストも淡い地の札。
    let tmp = notes("sr36_saas");
    let mut a = with(&tmp, ColorMode::Rgb, "[look]\npreset = \"saas\"\n");
    a.col = a.cols.iter().position(|c| c == "done").unwrap();
    let r = row(&a, "a");
    assert!(r.contains(" ● todo ") && r.contains(" ui   perf"), "{r}");
    let buf = buffer(&a);
    let y = row_y(&a, "a");
    let x = find_in(&buf, y, "todo").unwrap();
    assert_ne!(buf[(x, y)].bg, Color::Reset, "淡い地");
}

#[test]
fn test_sr_36_pill_needs_nerd_font() {
    // [SR-36] pill は nerd_font のときだけ丸い端。無ければ soft(▐ ▌)。
    let tmp = notes("sr36_pill");
    let a = with(&tmp, ColorMode::Rgb, "[look.style]\ntags = \"pill\"\n");
    assert!(row(&a, "b").contains("▐docs▌"), "{}", row(&a, "b"));
    let a = with(
        &tmp,
        ColorMode::Rgb,
        "[terminal]\nnerd_font = true\n\n[look.style]\ntags = \"pill\"\n",
    );
    assert!(
        row(&a, "b").contains("\u{e0b6}docs\u{e0b4}"),
        "{}",
        row(&a, "b")
    );
}

#[test]
fn test_sr_36_cross_tints_column() {
    // [SR-36] select = "cross": 今の列を、ほかの行でも淡く塗る。
    let tmp = notes("sr36_cross");
    let mut a = with(&tmp, ColorMode::Rgb, "[look.style]\nselect = \"cross\"\n");
    a.col = a.cols.iter().position(|c| c == "done").unwrap();
    let buf = buffer(&a);
    let y = row_y(&a, "d");
    let x = find_in(&buf, y, "✓").unwrap();
    assert_eq!(buf[(x, y)].bg, Color::Rgb(40, 40, 48), "今の列の淡い地");
    let other = find_in(&buf, y, "done").unwrap();
    assert_eq!(buf[(other, y)].bg, Color::Reset, "ほかの列は塗らない");
}

#[test]
fn test_sr_36_rules_and_frames() {
    // [SR-36] rules = "columns" は列の区切り線、frames = "square" は角の四角い窓。
    let tmp = notes("sr36_frames");
    let mut a = with(
        &tmp,
        ColorMode::Rgb,
        "[look.style]\nrules = \"columns\"\nframes = \"square\"\n",
    );
    let s = screen(&a);
    let head = s.lines().find(|l| l.contains("status")).unwrap();
    assert!(head.contains("│"), "{head}");
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("└") && !s.contains("╰"), "{s}");
}

#[test]
fn test_sr_36_no_color_keeps_text() {
    // [SR-36][SR-35] 色を使わない表示では部品にしない(今の見た目)。
    let tmp = notes("sr36_plain");
    let a = with(&tmp, ColorMode::None, "[look]\npreset = \"saas\"\n");
    let b = row(&a, "b");
    assert!(
        b.contains("true") && b.contains("[docs]") && b.contains("doing"),
        "{b}"
    );
    assert!(!b.contains("●"), "{b}");
}
