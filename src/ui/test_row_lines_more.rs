//! 伸ばした行(CV-8)の飾りと位置の残り: 列の区切りと一行おきの色を高さ全部に、札は1行、その場の操作の窓は
//! 行の1行目から、ビューごとの上書きが表に効く。
//! 記録: specs/_changes/2026-10-12-row-lines.md(照合の指摘3)。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

const LONG50: &str = "abcdefghijklmnopqrstABCDEFGHIJKLMNOPQRST0123456789";

fn boot(name: &str, notes: &[(&str, String)], extra: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let toml = format!("[display]\ntop_padding = 0\nbar_padding = 0\n{extra}");
    let (config, warnings) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), color);
    a.resize(100, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: color == ColorMode::None,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.widths.insert("desc".into(), 20);
    (tmp, a)
}

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

fn lines(a: &App) -> Vec<String> {
    text(&buf(a)).lines().map(str::to_string).collect()
}

fn line_of(l: &[String], t: &str) -> usize {
    l.iter()
        .position(|x| x.contains(t))
        .unwrap_or_else(|| panic!("「{t}」が無い:\n{}", l.join("\n")))
}

fn notes() -> Vec<(&'static str, String)> {
    vec![
        ("a.md", "---\ndesc: one\nk: x\n---\n".to_string()),
        ("b.md", format!("---\ndesc: {LONG50}\nk: y\n---\n")),
        ("c.md", "---\ndesc: three\nk: z\n---\n".to_string()),
    ]
}

#[test]
fn test_cv_8_column_lines_cover_whole_row() {
    // [CV-8] 列の区切り線は、伸ばした行の2行目・3行目にも1行目と同じ桁に引く。
    let (_t, a) = boot(
        "cv8m_cols",
        &notes(),
        "row_extra_lines = 2\ncolumn_lines = true\n",
        ColorMode::None,
    );
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    let bars = |s: &str| {
        s.chars()
            .enumerate()
            .filter(|(_, c)| *c == '│')
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    };
    assert!(!bars(&l[y]).is_empty(), "{}", l.join("\n"));
    for k in [1, 2] {
        assert_eq!(
            bars(&l[y + k]),
            bars(&l[y]),
            "{}行目:\n{}",
            k + 1,
            l.join("\n")
        );
    }
}

#[test]
fn test_cv_8_zebra_covers_whole_row() {
    // [CV-8] 一行おきの色は、伸ばした行(2番目のノート。色の付く行)の高さ全部に付き、次の行には付かない。
    let (_t, a) = boot(
        "cv8m_zebra",
        &[
            notes(),
            vec![("d.md", "---\ndesc: four\n---\n".to_string())],
        ]
        .concat(),
        "row_extra_lines = 2\nzebra = true\n",
        ColorMode::Indexed,
    );
    // 選んでいる行の色と混ざらないよう、4番目のノートを選ぶ。
    let mut a = a;
    a.row = 3;
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    let b = buf(&a);
    let bg = |y: usize| b[(60, y as u16)].bg;
    assert_ne!(bg(y), bg(y - 1), "2番目の行は色が付く");
    for k in [1, 2] {
        assert_eq!(bg(y + k), bg(y), "{}行目も同じ色", k + 1);
    }
    assert_eq!(bg(y + 3), bg(y - 1), "次の行(3番目)は色なし");
}

#[test]
fn test_cv_8_chips_stay_one_line() {
    // [CV-8] リストの値の札は折り返さない: 札の列が狭くても、ほかが短ければ行は1行のまま。
    let notes = vec![
        (
            "a.md",
            "---\ntags: [alpha, beta, gamma, delta, epsilon, zeta]\n---\n".to_string(),
        ),
        ("b.md", "---\ntags: [x]\n---\n".to_string()),
    ];
    let (_t, mut a) = boot(
        "cv8m_chips",
        &notes,
        "row_extra_lines = 2\n",
        ColorMode::Rgb,
    );
    a.widths.insert("tags".into(), 10);
    a.row = 1;
    let l = lines(&a);
    let y = l.iter().position(|x| x.starts_with(" a ")).unwrap();
    assert!(
        l[y + 1].starts_with(">b "),
        "札の行は1行:\n{}",
        l.join("\n")
    );
}

#[test]
fn test_cv_8_menu_window_from_first_line() {
    // [CV-8] その場の操作の窓(と候補の窓)の位置の基は、伸ばした行の1行目。
    let (_t, mut a) = boot(
        "cv8m_menu",
        &notes(),
        "row_extra_lines = 2\n",
        ColorMode::None,
    );
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    a.row = 1;
    a.col = a.cols.iter().position(|c| c == "desc").unwrap();
    a.scroll_into_view();
    let (_, ay) = super::menu::anchor(&a).expect("選んだセルの位置");
    assert_eq!(ay, y);
    // 伸ばした行の下の行の基は、伸ばした分だけ下。
    a.row = 2;
    let (_, cy) = super::menu::anchor(&a).unwrap();
    assert_eq!(cy, y + 3);
}

#[test]
fn test_cv_8_view_override_applies() {
    // [CV-8][SR-20] 設定は 0 でも、ビューの設定の上書きが 2 なら、そのビューの行は伸びる。
    let (_t, mut a) = boot("cv8m_view", &notes(), "", ColorMode::None);
    let before = lines(&a);
    assert!(
        before.iter().any(|x| x.contains("abcdefghijklmnopqrs…")),
        "{}",
        before.join("\n")
    );
    a.settings.display.row_extra_lines = Some(2);
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    assert!(
        l[y + 1].contains("ABCDEFGHIJKLMNOPQRST"),
        "{}",
        l.join("\n")
    );
}
