//! セルの部品(SR-35)の照合で見つけた所: 札を強いた列のリスト、詳細の値の文字、検索の印、点を含む列の名前。

use super::test_screen::{app_of, buffer, screen, Tmp};
use super::*;
use ratatui::style::Modifier;

/// 設定の文 `cfg` の `[look]` の区画に1行を足す(区画が無ければ作る。同じ見出しを2回書かない)。
fn with_look(cfg: &str, line: &str) -> String {
    match cfg.find("[look]\n") {
        Some(i) => format!("{}{line}\n{}", &cfg[..i + 7], &cfg[i + 7..]),
        None => format!("{cfg}\n[look]\n{line}\n"),
    }
}

fn notes(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, done, tags, ver) in [
        ("a", "true", "[ui, perf]", "v1"),
        ("b", "false", "[docs]", "v1"),
        ("c", "true", "[ui]", "v2"),
    ] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\ndone: {done}\ntags: {tags}\nrel.ver: {ver}\n---\n"),
        );
    }
    tmp
}

fn with(tmp: &Tmp, cfg: &str) -> App {
    let mut a = app_of(tmp, ColorMode::Rgb);
    // SR-36: 今までの形(classic の組)の部品を確かめる。
    let cfg = with_look(cfg, "preset = \"classic\"");
    let (c, w) = mdgrid::config::parse(&cfg).unwrap();
    assert!(w.is_empty(), "{w:?}");
    a.configure(&c);
    a.refresh_if_needed();
    a
}

fn row_text(a: &App, name: &str) -> String {
    screen(a)
        .lines()
        .find(|l| {
            l.get(1..)
                .is_some_and(|r| r.starts_with(&format!("{name} ")))
        })
        .unwrap()
        .to_string()
}

#[test]
fn test_sr_35_chip_column_wins_over_parts() {
    // [SR-35] 列の "chip" は、全体が plain でもリストを札にする(列の設定が優先)。
    let tmp = notes("sr35m_chip");
    let a = with(
        &tmp,
        "[look]\ncells = \"plain\"\n\n[look.columns]\ntags = \"chip\"\n",
    );
    let b = row_text(&a, "b");
    assert!(b.contains(" docs ") && !b.contains("[docs]"), "{b}");
    assert!(b.contains("false"), "ほかの列は文字のまま: {b}");
}

#[test]
fn test_sr_35_detail_shows_value_text() {
    // [SR-35] 部品は表の中だけ。詳細は値の文字(true・[ui, perf])。
    let tmp = notes("sr35m_detail");
    let a = with(&tmp, "");
    let row = a
        .rows
        .iter()
        .find(|r| r.0.ends_with("a.md"))
        .unwrap()
        .clone();
    assert_eq!(a.detail_text(&row, "done"), "true");
    assert_eq!(a.detail_text(&row, "tags"), "[ui, perf]");
}

#[test]
fn test_sr_35_search_marks_rich_cells() {
    // [SR-35][NV-1] 検索の語が値の文字に一致すれば、☑ のセルにも検索の印(下線)。
    let tmp = notes("sr35m_search");
    let mut a = with(&tmp, "");
    a.col = a.cols.iter().position(|c| c == "tags").unwrap();
    a.search = Some("true".into());
    let s = screen(&a);
    let y = s
        .lines()
        .position(|l| l.get(1..).is_some_and(|r| r.starts_with("c ")))
        .unwrap() as u16;
    let buf = buffer(&a);
    let x = (0..buf.area.width)
        .find(|&x| buf[(x, y)].symbol() == "☑")
        .expect("☑ のセル");
    assert!(buf[(x, y)].modifier.contains(Modifier::UNDERLINED));
}

#[test]
fn test_sr_35_dotted_key_can_be_chips() {
    // [SR-35] 名前に点を含むふつうのキー(rel.ver)も、くり返しのある短い値なら札(計算の列だけ除く)。
    let tmp = notes("sr35m_dot");
    let a = with(&tmp, "");
    assert!(a.is_select("rel.ver"));
    let s = screen(&a);
    assert!(s.contains("◉ rel.ver"), "{s}");
}
