//! アイコン用のフォントの記号を設定で使う(SR-45)の受け入れ試験。既定は今の字の記号、`terminal.nerd_icons = true`
//! では設定のボタン・画面の型のボタンと窓・列の見出しの型の印・カレンダーの題に Nerd Font の記号。
//! 記録: specs/_changes/2026-10-13-nerd-icons.md(タスク #1)。

use super::startup::Startup;
use super::test_screen::{press, text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

const GEAR: char = '\u{f013}';
const TABLE: char = '\u{f0ce}';
const CAL: char = '\u{f073}';

fn boot(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ndue: 2026-10-01\n---\n");
    tmp.write("b.md", "---\ndue: 2026-10-05\n---\n");
    let toml = format!("[look.style]\nicons = true\n{toml}");
    let (config, warnings) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::Rgb);
    a.resize(100, 30);
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
    a.col = a.cols.iter().position(|c| c == "due").unwrap();
    (tmp, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn pua(s: &str) -> bool {
    s.chars().any(|c| ('\u{e000}'..='\u{f8ff}').contains(&c))
}

#[test]
fn test_sr_45_default_plain_glyphs() {
    // [SR-45] 既定は今の字の記号(⚙・▦・◷)で、私用の領域の字は出さない。
    let (_t, a) = boot("sr45_plain", "");
    let s = screen(&a);
    assert!(s.contains("⚙ 設定") && s.contains("▦ 表 ▾"), "{s}");
    assert!(s.contains("◷ due"), "{s}");
    assert!(!pua(&s), "{s}");
}

#[test]
fn test_sr_45_nerd_icons_in_header_and_marks() {
    // [SR-45] nerd_icons = true → 設定のボタン・表のボタン・日付の列の見出しに Nerd Font の記号。字の記号は出ない。
    let (_t, a) = boot("sr45_nerd", "[terminal]\nnerd_icons = true\n");
    let s = screen(&a);
    assert!(s.contains(&format!("{GEAR} 設定")), "{s}");
    assert!(s.contains(&format!("{TABLE} 表 ▾")), "{s}");
    assert!(s.contains(&format!("{CAL} due")), "{s}");
    for plain in ["⚙", "▦", "◷"] {
        assert!(!s.contains(plain), "{plain}:\n{s}");
    }
    // SR-9: どの行も画面の幅を超えない。
    for l in s.lines() {
        assert!(width::width(l) <= a.size.0 as usize, "{l}");
    }
}

#[test]
fn test_sr_45_nerd_icons_calendar_title() {
    // [SR-45] 日付のセルで Enter → カレンダーの題の頭にカレンダーの記号。既定では付けない。
    let (_t, mut a) = boot("sr45_cal", "[terminal]\nnerd_icons = true\n");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let title = s
        .lines()
        .find(|l| l.contains("2026") && l.contains('╭'))
        .unwrap_or_else(|| panic!("カレンダーの題が無い:\n{s}"));
    assert!(title.contains(CAL), "{title}");
    let (_u, mut b) = boot("sr45_cal_plain", "");
    press(&mut b, KeyCode::Enter);
    assert!(!pua(&screen(&b)), "{}", screen(&b));
}

#[test]
fn test_sr_45_nerd_icons_in_screen_menu() {
    // [SR-45] 画面の型の窓の項目にも記号(表)。
    let (_t, mut a) = boot("sr45_menu", "[terminal]\nnerd_icons = true\n");
    a.open_screen_menu();
    let s = screen(&a);
    assert!(
        s.lines().filter(|l| l.contains(TABLE)).count() >= 2,
        "ボタンと窓の項目の両方:\n{s}"
    );
}

#[test]
fn test_sr_45_nerd_icons_every_type_mark() {
    // [SR-45] 数・リスト・チェックの見出しの印も Nerd Font の記号(字の記号 # ⋮ ✓ は出ない)。
    let tmp = Tmp::new("sr45_kinds");
    tmp.write("a.md", "---\nn: 3\ntags: [x, y]\ndone: true\n---\n");
    let (config, w) =
        mdgrid::config::parse("[look.style]\nicons = true\n[terminal]\nnerd_icons = true\n")
            .unwrap();
    assert!(w.is_empty(), "{w:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::Rgb);
    a.resize(100, 24);
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
    a.row = 0;
    a.col = 0;
    let s = screen(&a);
    for g in ['\u{f292}', '\u{f0ca}', '\u{f046}'] {
        assert!(s.contains(g), "{g:?}:\n{s}");
    }
    let head = s.lines().find(|l| l.contains("tags")).unwrap();
    for plain in ["# n", "⋮ tags", "✓ done"] {
        assert!(!head.contains(plain), "{plain}:\n{s}");
    }
}

#[test]
fn test_sr_45_nerd_icons_relmap_button() {
    // [SR-45] 関係マップの画面の型のボタンにも記号(U+F0E8)。
    use super::test_relations::{boot_tasks, workspace};
    use super::test_screen::ch;
    let tmp = workspace("sr45_rel");
    let mut a = boot_tasks(&tmp);
    a.nerd_icons = true;
    a.resize(120, 30);
    ch(&mut a, 'R');
    let s = screen(&a);
    assert!(s.contains("\u{f0e8} 関係マップ ▾"), "{s}");
}

#[test]
fn test_sr_45_nerd_icons_ambiguous_wide_click() {
    // [SR-45][CV-6][CE-35] あいまいな字を全角として数える設定でも、行が画面の幅を超えず、見出しの印のクリックは
    // 型の窓、設定のボタンのクリックは設定の画面に当たる。
    let (_t, mut a) = boot(
        "sr45_amb",
        "[terminal]\nnerd_icons = true\nambiguous_wide = true\n",
    );
    let s = screen(&a);
    for l in s.lines() {
        assert!(width::width(l) <= a.size.0 as usize, "{l}");
    }
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(&format!("{CAL} due")))
        .unwrap_or_else(|| panic!("{s}"));
    let x = width::width(&line[..line.find(CAL).unwrap()]);
    a.click(x as u16, y as u16);
    assert_eq!(format!("{:?}", a.mode), "ColumnType", "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(&format!("{GEAR} 設定")))
        .unwrap();
    let x = width::width(&line[..line.find(GEAR).unwrap()]);
    a.click(x as u16, y as u16);
    assert_eq!(format!("{:?}", a.mode), "Settings", "{}", screen(&a));
}
