//! 画面のテーマ(SR-26・SR-27)の塗り分けの続き(レビューの指摘: zebra の行の選択・印・太字と、列の見出しの色の範囲)。
//! 記録: specs/_changes/2026-10-06-themes.md。

use super::test_screen::{app_of, buffer, col_named, press, text, Tmp};
use super::*;
use mdgrid::config;
use mdgrid::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: alpha\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: beta\nstatus: done\n---\n"),
    ("c.md", "---\ntitle: gamma\nstatus: todo\n---\n"),
    ("d.md", "---\ntitle: delta\nstatus: done\n---\n"),
];

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    tmp
}

fn configured(tmp: &Tmp, text: &str) -> App {
    let mut a = app_of(tmp, ColorMode::Rgb);
    // テーマの塗り替え(SR-26)は今までの見た目(look = "classic")の上で確かめる。
    a.configure(
        &config::parse(&format!("look = \"classic\"\n{text}"))
            .unwrap()
            .0,
    );
    a
}

fn rgb(c: [u8; 3]) -> Color {
    Color::Rgb(c[0], c[1], c[2])
}

fn nord() -> mdgrid::theme::Palette {
    Theme::Nord.palette().unwrap()
}

/// テーマなしの画面で、行 y の太字+下線で前景の無いセルの x。
fn bold_underlined(buf: &Buffer, y: u16) -> Vec<u16> {
    (0..buf.area.width)
        .filter(|&x| {
            let c = &buf[(x, y)];
            c.fg == Color::Reset
                && !c.modifier.contains(Modifier::REVERSED)
                && c.modifier.contains(Modifier::BOLD | Modifier::UNDERLINED)
        })
        .collect()
}

#[test]
fn test_sr_26_zebra_row_selection_mark_and_bold() {
    // [SR-26][SR-20] zebra = true で偶数番目の行を選ぶ → 選んでいるセルは選択の色(反転は外れる)、
    // 左端の `>` は mark、名前の太字は strong、ほかの文字は zebra_fg、地は zebra_bg。
    let tmp = vault("thmzebsel");
    let zebra = "[display]\nzebra = true\n";
    let mut base_app = configured(&tmp, zebra);
    base_app.row = 1;
    let mut a = configured(&tmp, &format!("theme = \"nord\"\n\n{zebra}"));
    a.row = 1;
    let base = buffer(&base_app);
    let buf = buffer(&a);
    let s = text(&buf);
    let p = nord();
    let y = (view::data_y(&a) + 1) as u16;

    let sel: Vec<u16> = (0..80u16)
        .filter(|&x| base[(x, y)].modifier.contains(Modifier::REVERSED))
        .collect();
    assert!(
        !sel.is_empty(),
        "テーマなしで反転のセルが無い\n{}",
        text(&base)
    );
    for &x in &sel {
        let c = &buf[(x, y)];
        assert_eq!(c.bg, rgb(p.sel_bg), "選んでいるセル ({x},{y}) の背景\n{s}");
        assert_eq!(c.fg, rgb(p.sel_fg), "選んでいるセル ({x},{y}) の前景\n{s}");
        assert!(!c.modifier.contains(Modifier::REVERSED), "({x},{y}) の反転");
    }

    assert_eq!(buf[(0u16, y)].symbol(), ">", "{s}");
    assert_eq!(buf[(0u16, y)].fg, rgb(p.mark), "`>` の前景\n{s}");
    assert_eq!(buf[(0u16, y)].bg, rgb(p.zebra_bg), "`>` の背景\n{s}");

    let bold: Vec<u16> = (0..80u16)
        .filter(|&x| {
            let c = &base[(x, y)];
            c.modifier.contains(Modifier::BOLD) && !c.modifier.contains(Modifier::REVERSED)
        })
        .collect();
    assert!(!bold.is_empty(), "名前の太字が無い\n{}", text(&base));
    for x in bold {
        assert_eq!(buf[(x, y)].fg, rgb(p.strong), "太字 ({x},{y})\n{s}");
    }

    // 太字でも反転でもなく、前景が zebra の値のセルは zebra_fg。
    let plain_x = (1..79u16).find(|&x| {
        let c = &base[(x, y)];
        c.modifier.is_empty() && c.fg == Color::Rgb(208, 208, 208)
    });
    let x = plain_x.expect("zebra の前景のセル");
    assert_eq!(buf[(x, y)].fg, rgb(p.zebra_fg), "({x},{y})\n{s}");
    assert_eq!(buf[(x, y)].bg, rgb(p.zebra_bg), "({x},{y})\n{s}");
}

#[test]
fn test_sr_26_colhead_only_on_the_column_heading_row() {
    // [SR-26] 列の見出しの行の太字+下線は colhead。
    let tmp = vault("thmcolhead");
    let base = buffer(&configured(&tmp, ""));
    let a = configured(&tmp, "theme = \"nord\"\n");
    let buf = buffer(&a);
    let s = text(&buf);
    let p = nord();
    let hy = (view::data_y(&a) - 1) as u16;
    let xs = bold_underlined(&base, hy);
    assert!(!xs.is_empty(), "列の見出しが無い\n{}", text(&base));
    for x in xs {
        assert_eq!(
            buf[(x, hy)].fg,
            rgb(p.colhead),
            "列の見出し ({x},{hy})\n{s}"
        );
    }
}

#[test]
fn test_sr_26_input_box_is_strong_not_colhead() {
    // [SR-26] 編集の入力ボックス(太字+下線)は列の見出しの色でなく strong。
    let tmp = vault("thminput");
    let cfg = "candidates = 0\n";
    let mut b = configured(&tmp, cfg);
    let mut a = configured(&tmp, &format!("theme = \"nord\"\n{cfg}"));
    for app in [&mut b, &mut a] {
        col_named(app, "title");
        press(app, KeyCode::Enter);
        assert_eq!(app.mode, keymap::Mode::Edit);
    }
    let base = buffer(&b);
    let buf = buffer(&a);
    let s = text(&buf);
    let p = nord();
    let y = view::data_y(&a) as u16;
    let xs = bold_underlined(&base, y);
    assert!(!xs.is_empty(), "入力ボックスが無い\n{}", text(&base));
    for x in xs {
        assert_eq!(buf[(x, y)].fg, rgb(p.strong), "入力ボックス ({x},{y})\n{s}");
    }
}

#[test]
fn test_sr_26_search_hit_name_is_strong_not_colhead() {
    // [SR-26] 検索に一致した今の行の名前(太字+下線)も strong。
    let tmp = vault("thmsearch");
    let mut b = configured(&tmp, "");
    let mut a = configured(&tmp, "theme = \"nord\"\n");
    for app in [&mut b, &mut a] {
        app.search = Some("a".into());
    }
    let base = buffer(&b);
    let buf = buffer(&a);
    let s = text(&buf);
    let p = nord();
    let y = view::data_y(&a) as u16;
    let xs = bold_underlined(&base, y);
    assert!(!xs.is_empty(), "一致した名前が無い\n{}", text(&base));
    for x in xs {
        assert_eq!(buf[(x, y)].fg, rgb(p.strong), "一致した名前 ({x},{y})\n{s}");
    }
}
