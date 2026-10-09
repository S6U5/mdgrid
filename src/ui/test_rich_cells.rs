//! セルの部品(SR-35): 色を使う表示で、真偽は ☑ ☐、リストと札の列は色の付いた札、見出しに型の印。
//! 色なしと cells = "plain" は今の見た目。部品の種類ごと・列ごとに設定で選べる。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;

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
            &format!("---\nstatus: {status}\ndone: {done}\ntags: {tags}\ndue: 2026-10-0{}\nmemo: note {n}\n---\n", n.len() + 1),
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

fn head(a: &App) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.contains("status"))
        .unwrap_or_default()
        .to_string()
}

fn row(a: &App, name: &str) -> String {
    screen(a)
        .lines()
        .find(|l| l.starts_with(&format!(" {name} ")) || l.starts_with(&format!(">{name} ")))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{}", screen(a)))
        .to_string()
}

#[test]
fn test_sr_35_rich_cells() {
    // [SR-35] 色のある表示: 真偽は ☑ ☐、リストの要素と status の値は札、見出しに型の印。値の文字は見せる。
    let tmp = notes("sr35_rich");
    let a = with(&tmp, ColorMode::Rgb, "");
    let h = head(&a);
    assert!(
        h.contains("◉ status") && h.contains("☑ done") && h.contains("⋮ tags"),
        "{h}"
    );
    assert!(h.contains("◷ due") && !h.contains("◉ memo"), "{h}");
    let b = row(&a, "b");
    assert!(b.contains("☑") && !b.contains("true"), "{b}");
    assert!(b.contains(" doing ") && b.contains(" docs "), "{b}");
    let r = row(&a, "a");
    assert!(r.contains("☐") && r.contains(" ui   perf "), "{r}");
}

#[test]
fn test_sr_35_chip_colors_stable() {
    // [SR-35] 札は値ごとに決まった色の地(同じ値は同じ色)。地の色が付く。
    let tmp = notes("sr35_color");
    let mut a = with(&tmp, ColorMode::Rgb, "");
    a.resize(80, 24);
    a.col = a.cols.iter().position(|c| c == "memo").unwrap(); // 選んでいるセルは札にしないので外す
    let buf = buffer(&a);
    let s = screen(&a);
    let find = |name: &str, text: &str| -> ratatui::style::Color {
        let y = s
            .lines()
            .position(|l| {
                l.get(1..)
                    .is_some_and(|r| r.starts_with(&format!("{name} ")))
            })
            .unwrap() as u16;
        let cells: Vec<String> = (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        let x = (0..cells.len())
            .find(|&x| cells[x..].concat().starts_with(text))
            .unwrap();
        buf[(x as u16, y)].bg
    };
    let todo_a = find("a", "todo");
    let todo_c = find("c", "todo");
    assert_eq!(todo_a, todo_c, "同じ値は同じ色");
    assert_ne!(todo_a, ratatui::style::Color::Reset, "札には地の色");
}

#[test]
fn test_sr_35_plain_without_color_or_setting() {
    // [SR-35] 色なしと cells = "plain" では今の見た目(true・[a, b]・印なし)。
    let tmp = notes("sr35_plain");
    for a in [
        with(&tmp, ColorMode::None, ""),
        with(&tmp, ColorMode::Rgb, "cells = \"plain\"\n"),
    ] {
        let b = row(&a, "b");
        assert!(
            b.contains("true") && b.contains("[docs]") && !b.contains("☑"),
            "{b}"
        );
        let h = head(&a);
        assert!(!h.contains("◉") && !h.contains("◷"), "{h}");
    }
}

#[test]
fn test_sr_35_per_part_and_column() {
    // [SR-35] 部品の種類ごと(checkbox = false)と列ごと(status は文字、memo は札)。列が種類より優先。
    let tmp = notes("sr35_cols");
    let a = with(
        &tmp,
        ColorMode::Rgb,
        "[cells]\ncheckbox = false\n[cells.columns]\nstatus = \"plain\"\nmemo = \"chip\"\n",
    );
    let b = row(&a, "b");
    assert!(
        b.contains("true") && !b.contains("☑"),
        "真偽は文字のまま: {b}"
    );
    assert!(b.contains(" docs "), "リストは札のまま: {b}");
    assert!(b.contains("doing"), "{b}");
    let h = head(&a);
    assert!(h.contains("◉ memo") && !h.contains("◉ status"), "{h}");
    let a2 = with(
        &tmp,
        ColorMode::Rgb,
        "[cells]\nstyle = \"plain\"\n[cells.columns]\ndone = \"rich\"\n",
    );
    let b = row(&a2, "b");
    assert!(
        b.contains("☑") && b.contains("[docs]"),
        "rich の列だけ部品: {b}"
    );
}

#[test]
fn test_sr_35_search_matches_value_text() {
    // [SR-35] 検索は見せる文字(☑)でなく値の文字(true)でも一致する。
    let tmp = notes("sr35_search");
    let mut a = with(&tmp, ColorMode::Rgb, "");
    a.search = Some("true".into());
    let rows = a.rows.clone();
    let done_hits = rows
        .iter()
        .filter(|r| a.search_hit(r, Some("done")))
        .count();
    assert_eq!(done_hits, 2);
}
