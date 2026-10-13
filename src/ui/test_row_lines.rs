//! 表の行を伸ばす設定 `display.row_extra_lines`(CV-8)と、伸ばせるときの改行のある値(CV-3)の受け入れ試験。
//! 長い値は列の幅で折り返し、改行は改行で行を分け、データの1行を最大で「1 + 設定の数」の行の高さまで伸ばす。
//! 行の高さは行ごとに変わる。0 では今の1行の見え方と1字も変わらない。
//! 記録: specs/_changes/2026-10-12-row-lines.md(タスク #1)。
//!
//! 仮定(実装役に渡す): 設定の画面の「表示」の区画の項目の名前は「行を伸ばす」、値は帯の余白と同じ「N行」の字。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, text, Tmp};
use super::test_settings_screen::open;
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 50 字(幅 20 の列で 20・20・10 の3行)。
const LONG50: &str = "abcdefghijklmnopqrstABCDEFGHIJKLMNOPQRST0123456789";
/// 61 字(幅 20 の列の3行に入りきらない)。
const LONG61: &str = "abcdefghijklmnopqrstABCDEFGHIJKLMNOPQRSTuvwxyz0123456789VWXYZ";

/// 余白なし(行の位置を数えやすく)・`desc` の列の幅 20 で起動する。`extra` は `[display]` に足す行。
fn boot(name: &str, notes: &[(&str, &str)], extra: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let toml = format!("[display]\ntop_padding = 0\nbar_padding = 0\n{extra}");
    let (config, warnings) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
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

fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .map(str::to_string)
        .collect()
}

/// `t` を含む最初の行の番号。
fn line_of(l: &[String], t: &str) -> usize {
    l.iter()
        .position(|x| x.contains(t))
        .unwrap_or_else(|| panic!("「{t}」の行が無い:\n{}", l.join("\n")))
}

fn three_notes() -> Vec<(&'static str, String)> {
    vec![
        ("a.md", format!("---\ndesc: {LONG50}\n---\n")),
        ("b.md", "---\ndesc: short\n---\n".to_string()),
        ("c.md", "---\ndesc: tiny\n---\n".to_string()),
    ]
}

fn refs<'a>(v: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    v.iter().map(|(n, t)| (*n, t.as_str())).collect()
}

#[test]
fn test_cv_8_long_value_wraps_to_three_lines() {
    // [CV-8] row_extra_lines = 2、幅 20 の列に 50 字 → その行は3行の高さで、説明が全部見える。
    let notes = three_notes();
    let (_t, a) = boot("cv8_wrap", &refs(&notes), "row_extra_lines = 2\n");
    let l = lines(&a);
    let s = l.join("\n");
    let y = line_of(&l, "abcdefghijklmnopqrst");
    assert!(l[y + 1].contains("ABCDEFGHIJKLMNOPQRST"), "2行目:\n{s}");
    assert!(l[y + 2].contains("0123456789"), "3行目:\n{s}");
    let rows = l[y..y + 3].join("\n");
    assert!(!rows.contains('…'), "全部入るので `…` は無い:\n{s}");
    // 短い値だけの行は1行のまま(b のすぐ下が c)。
    let b = line_of(&l, "short");
    assert_eq!(b, y + 3, "a の3行のすぐ下が b:\n{s}");
    assert_eq!(line_of(&l, "tiny"), b + 1, "短い行は1行:\n{s}");
}

#[test]
fn test_cv_8_overflow_ends_with_ellipsis() {
    // [CV-8] 伸ばした行に入りきらない値 → 最後の行の後ろに `…`。
    let notes = vec![("a.md", format!("---\ndesc: {LONG61}\n---\n"))];
    let (_t, a) = boot("cv8_ell", &refs(&notes), "row_extra_lines = 2\n");
    let l = lines(&a);
    let s = l.join("\n");
    let y = line_of(&l, "abcdefghijklmnopqrst");
    assert!(l[y + 1].contains("ABCDEFGHIJKLMNOPQRST"), "{s}");
    assert!(
        l[y + 2].contains("uvwxyz0123456789VWX…"),
        "3行目の終わりが `…`:\n{s}"
    );
    assert!(!l[y + 3].contains("YZ"), "4行目には伸ばさない:\n{s}");
}

#[test]
fn test_cv_8_newline_splits_lines() {
    // [CV-8][CV-3] 改行のある値 → 改行で行を分ける(`…⏎` は付けない)。
    let notes = vec![
        ("a.md", "---\ndesc: |\n  first line\n  second line\n---\n"),
        ("b.md", "---\ndesc: next\n---\n"),
    ];
    let (_t, a) = boot("cv8_nl", &notes, "row_extra_lines = 2\n");
    let l = lines(&a);
    let s = l.join("\n");
    let y = line_of(&l, "first line");
    assert!(l[y + 1].contains("second line"), "2行目:\n{s}");
    assert!(!s.contains('⏎'), "伸ばせるときは `…⏎` を付けない:\n{s}");
    assert_eq!(line_of(&l, "next"), y + 2, "2行の高さ:\n{s}");
}

#[test]
fn test_cv_3_newline_marked_without_extra_lines() {
    // [CV-3][CV-8] 0(既定)では今のまま: 1行目と `…⏎`。設定を書かないときと画面が1字も変わらない。
    let notes = vec![
        (
            "a.md",
            "---\ndesc: |\n  first line\n  second line\n---\n".to_string(),
        ),
        ("b.md", format!("---\ndesc: {LONG50}\n---\n")),
    ];
    let (_t, zero) = boot("cv3_zero", &refs(&notes), "row_extra_lines = 0\n");
    let (_u, none) = boot("cv3_none", &refs(&notes), "");
    let l = lines(&zero);
    let s = l.join("\n");
    assert!(s.contains("first line…⏎"), "{s}");
    assert_eq!(l, lines(&none), "0 と既定は同じ画面");
}

#[test]
fn test_cv_8_continuation_lines_have_no_marks() {
    // [CV-8] 2行目より下の行には、行の印 `>` と行番号を出さない。
    let notes = three_notes();
    let (_t, a) = boot(
        "cv8_marks",
        &refs(&notes),
        "row_extra_lines = 2\nrow_numbers = true\n",
    );
    let l = lines(&a);
    let s = l.join("\n");
    let y = line_of(&l, "abcdefghijklmnopqrst");
    assert!(l[y].starts_with(">1 "), "1行目に行の印と行番号:\n{s}");
    for k in [1, 2] {
        let head: String = l[y + k].chars().take(6).collect();
        assert!(head.trim().is_empty(), "{}行目の左は空:\n{s}", k + 1);
    }
    // 次の行の番号は 2(伸ばした行の分を数えない)。
    assert!(l[y + 3].starts_with(" 2 "), "{s}");
}

#[test]
fn test_cv_8_click_on_second_line_selects_row() {
    // [CV-8] 伸ばした行の2行目をクリック → その行を選ぶ。その下の行をクリック → 次の行。
    let notes = three_notes();
    let (_t, mut a) = boot("cv8_click", &refs(&notes), "row_extra_lines = 2\n");
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    let x = l[y + 1].find("ABCDE").unwrap() as u16;
    let b = line_of(&l, "short");
    a.click(x, b as u16);
    assert_eq!(a.row, 1, "b を選ぶ");
    a.click(x, (y + 1) as u16);
    assert_eq!(a.row, 0, "a の2行目で a を選ぶ");
    a.click(x, (y + 2) as u16);
    assert_eq!(a.row, 0, "a の3行目でも a");
}

#[test]
fn test_cv_8_bottom_row_fully_visible() {
    // [CV-8] 一番下の行を選ぶ → その行の高さ全部が見えるまで表が流れる。
    let mut notes: Vec<(String, String)> = (0..40)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                format!("---\ndesc: row{i:02}\n---\n"),
            )
        })
        .collect();
    notes.push(("z.md".into(), format!("---\ndesc: {LONG50}\n---\n")));
    let r: Vec<(&str, &str)> = notes
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    let (_t, mut a) = boot("cv8_bottom", &r, "row_extra_lines = 2\n");
    ch(&mut a, 'G');
    assert_eq!(a.row, a.slots.len() - 1);
    let l = lines(&a);
    let s = l.join("\n");
    let y = line_of(&l, "abcdefghijklmnopqrst");
    assert!(l[y + 2].contains("0123456789"), "3行目まで見える:\n{s}");
    // 3行目は表の中(下の帯の上)。
    assert!(y + 2 < l.len() - 2, "{s}");
}

#[test]
fn test_cv_8_page_down_counts_tall_rows() {
    // [CV-8] ページ送りは伸ばした行の高さで数える: 全部3行の行で PageDown → 選んだ行が画面の中に見える。
    let notes: Vec<(String, String)> = (0..30)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                format!("---\ndesc: r{i:02}{}\n---\n", &LONG50[3..]),
            )
        })
        .collect();
    let r: Vec<(&str, &str)> = notes
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    let (_t, mut a) = boot("cv8_page", &r, "row_extra_lines = 2\n");
    let before = a.row;
    press(&mut a, KeyCode::PageDown);
    let moved = a.row - before;
    let l = lines(&a);
    let s = l.join("\n");
    // 表の行の数(画面の行)の3分の1ほど動く(1行を1と数えると3倍動いて、見ていない行を飛ばす)。
    let data_h = a.data_height();
    assert!(
        moved >= 1 && moved <= data_h / 3 + 1,
        "動いた行 {moved}、表の高さ {data_h}:\n{s}"
    );
    let name = format!("r{:02}", a.row);
    assert!(s.contains(&name), "選んだ行が見える:\n{s}");
}

#[test]
fn test_cv_8_edit_box_on_first_line() {
    // [CV-8] 伸ばした行のセルを編集する → 入力の欄はその行の1行目。
    let notes = three_notes();
    let (_t, mut a) = boot("cv8_edit", &refs(&notes), "row_extra_lines = 2\n");
    let l = lines(&a);
    let y = line_of(&l, "abcdefghijklmnopqrst");
    a.col = a.cols.iter().position(|c| c == "desc").unwrap();
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let (_, cy) = view::cursor(&a).expect("入力の欄のカーソル");
    assert_eq!(cy as usize, y, "入力の欄は1行目:\n{}", lines(&a).join("\n"));
}

#[test]
fn test_cv_8_settings_cycles_zero_to_three() {
    // [CV-8][NV-18] ビューの設定の「表示」の区画で「行を伸ばす」を 0 → 1 → 2 → 3 → 0 と切り替える。
    let notes = three_notes();
    let (_t, mut a) = boot("cv8_settings", &refs(&notes), "");
    a.resize(120, 50);
    open(&mut a);
    for _ in 0..8 {
        if a.draft.as_ref().map(|d| d.sec) == Some(super::settings::Sec::Display) {
            break;
        }
        press(&mut a, KeyCode::Tab);
    }
    let value = |a: &App| {
        let l = lines(a);
        l.iter()
            .find(|x| x.contains("行を伸ばす"))
            .unwrap_or_else(|| panic!("「行を伸ばす」が無い:\n{}", l.join("\n")))
            .clone()
    };
    for _ in 0..14 {
        if value(&a).contains(">行を伸ばす") {
            break;
        }
        press(&mut a, KeyCode::Down);
    }
    assert!(value(&a).contains(">行を伸ばす"), "{}", value(&a));
    assert!(value(&a).contains("0行"), "既定は 0: {}", value(&a));
    for want in ["1行", "2行", "3行", "0行"] {
        ch(&mut a, ' ');
        assert!(value(&a).contains(want), "{want}: {}", value(&a));
    }
}
