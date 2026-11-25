//! 数の見せ方(`[format.<列>]`。cell-view.number-format・number-format-display-only・number-format-totals・
//! number-format-locale・number-format-invalid)の画面の試験。値は作ったもの。実装の前に書いた。
//!
//! 台帳の印(`.mdgrid/workspace.toml`)の最上位に `[types]` と `[format.<列>]`、`[[table]]` に中身(`[table.base]`)と
//! グラフを書いた台帳を、`T.base` として開く(WS-9)。セルは行の名前(ノートの名前)のある行の字で見る。

use super::startup::Startup;
use super::test_screen::{col_named, edit_cell, press, text, Tmp};
use super::*;
use mdgrid::i18n::{scoped, Lang};
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

/// 金額(amount)・道のり(distance)・式の列 rate の見せ方を決めた印の頭。
const FORMATS: &str = r#"[types]
amount = "number"
distance = "number"

[format.amount]
thousands = true
prefix = "¥"

[format.distance]
thousands = true
decimals = 1
suffix = " km"

[format."formula.rate"]
suffix = "%"
"#;

/// セルの試験のノート: alpha は 98000 と 1234.56、bravo は -1200 と 3、charlie は数でない amount。
const CELLS: &[(&str, &str)] = &[
    ("alpha.md", "---\namount: 98000\ndistance: 1234.56\n---\n"),
    ("bravo.md", "---\namount: -1200\ndistance: 3\n---\n"),
    ("charlie.md", "---\namount: abc\n---\n"),
];

/// 集計の試験のノート: alpha は food の 98000、bravo は daily の 3500。
const TOTALS: &[(&str, &str)] = &[
    ("alpha.md", "---\namount: 98000\ncategory: food\n---\n"),
    ("bravo.md", "---\namount: 3500\ncategory: daily\n---\n"),
];

/// 表 T(ビュー「全部」・「合計」・「分類ごと」・「件数」と、分類ごとの金額のグラフ)。
const TABLE: &str = r#"
[[table]]
name = "T"
chart = [{ name = "分類ごとの金額", x = "category", y = "amount" }]

[table.base.formulas]
rate = '45'

[[table.base.views]]
type = "table"
name = "全部"
order = ["amount", "distance", "formula.rate", "category"]

[[table.base.views]]
type = "table"
name = "合計"
order = ["amount", "category"]
summaries = { amount = "Sum" }

[[table.base.views]]
type = "table"
name = "分類ごと"
order = ["amount"]
groupBy = { property = "category", direction = "ASC" }
summaries = { amount = "Sum" }

[[table.base.views]]
type = "table"
name = "件数"
order = ["amount", "category"]
summaries = { amount = "Filled" }

[[table.base.views]]
type = "table"
name = "金額の順"
order = ["amount"]
sort = [{ property = "amount", direction = "ASC" }]
"#;

/// 台帳(ノート `notes`・印の頭 `top` と表 T)を、設定 `conf` で幅 120・高さ 24 に開き、ビュー `view` に移る。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, conf: &str, view: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), format!("{top}{TABLE}")).unwrap();
    let p = tmp.notes().join("T.base");
    let (config, warnings) = mdgrid::config::parse(conf).unwrap();
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), Some(view)).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 24);
    a.start(Startup {
        config,
        warnings,
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: p.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn scr(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// 名前 `name` の行の、画面の字の行。
fn line_of(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("「{name}」の行が無い:\n{s}"))
        .to_string()
}

/// 名前 `name` の行の添字(まとまりの無いビューの表の行の位置)。
fn row_index(a: &App, name: &str) -> usize {
    a.rows
        .iter()
        .position(|r| a.src.label(r).contains(name))
        .unwrap_or_else(|| panic!("{name} が無い"))
}

#[test]
fn test_number_format_cells() {
    // [utest -> req~cell-view.number-format~1] 98000 は `¥98,000`、1234.56 は `1,234.6 km`(四捨五入)、3 は `3.0 km`(0 で埋める)、
    // 式の列 rate の 45 は `45%`、-1200 は `-¥1,200`、数でない abc は `!abc`(見せ方を当てない)。
    let (_t, a) = boot("nf_cells", CELLS, FORMATS, "", "全部");
    let s = scr(&a);
    let alpha = line_of(&s, "alpha");
    assert!(alpha.contains("¥98,000"), "{s}");
    assert!(alpha.contains("1,234.6 km"), "{s}");
    assert!(alpha.contains("45%"), "{s}");
    let bravo = line_of(&s, "bravo");
    assert!(bravo.contains("-¥1,200"), "{s}");
    assert!(bravo.contains("3.0 km"), "{s}");
    let charlie = line_of(&s, "charlie");
    assert!(charlie.contains("!abc"), "{s}");
    assert!(!charlie.contains('¥'), "{s}");
}

#[test]
fn test_number_format_pending_value() {
    // [utest -> req~cell-view.number-format~1] amount を 5000 に直してためた行のセルは `*¥5,000`。
    let (_t, mut a) = boot("nf_pending", CELLS, FORMATS, "", "全部");
    let i = row_index(&a, "alpha");
    edit_cell(&mut a, i, "amount", "5000");
    assert_eq!(a.changes.count(), 1);
    let s = scr(&a);
    assert!(line_of(&s, "alpha").contains("*¥5,000"), "{s}");
}

#[test]
fn test_number_format_detail_and_edit_keep_number() {
    // [utest -> req~cell-view.number-format-display-only~1] 見せ方のあるセルで Enter → 編集の欄は元の数 `98000`(`¥` も `,` も無い)。
    let (_t, mut a) = boot("nf_edit", CELLS, FORMATS, "", "全部");
    a.row = row_index(&a, "alpha");
    col_named(&mut a, "amount");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.input.as_ref().map(|i| i.text.clone()).as_deref(),
        Some("98000")
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_number_format_sort_by_number() {
    // [utest -> req~cell-view.number-format-display-only~1] amount の昇順は数の順(500・12000・98000)。見せる字の順(`¥12,000` が
    // `¥500` より前)にならない。
    let notes = [
        ("d1.md", "---\namount: 12000\n---\n"),
        ("d2.md", "---\namount: 98000\n---\n"),
        ("d3.md", "---\namount: 500\n---\n"),
    ];
    let (_t, a) = boot("nf_sort", &notes, FORMATS, "", "金額の順");
    let names: Vec<String> = a.rows.iter().map(|r| a.src.label(r)).collect();
    assert_eq!(names, ["d3.md", "d1.md", "d2.md"]);
    let s = scr(&a);
    assert!(line_of(&s, "d3").contains("¥500"), "{s}");
}

#[test]
fn test_number_format_summary_row() {
    // [utest -> req~cell-view.number-format-totals~1] 合計の集計の行は「合計 ¥101,500」。
    let (_t, a) = boot("nf_sum", TOTALS, FORMATS, "", "合計");
    let s = scr(&a);
    assert!(s.contains("合計 ¥101,500"), "{s}");
}

#[test]
fn test_number_format_group_subtotal() {
    // [utest -> req~cell-view.number-format-totals~1, req~base-view.summaries-per-group~1] 分類でまとめたビュー: food の見出しの行に
    // 「合計 ¥98,000」、daily の見出しの行に「合計 ¥3,500」。
    let (_t, a) = boot("nf_group", TOTALS, FORMATS, "", "分類ごと");
    let s = scr(&a);
    let food = s
        .lines()
        .find(|l| l.contains('▾') && l.contains("food"))
        .unwrap_or_else(|| panic!("food の見出し:\n{s}"));
    assert!(food.contains("合計 ¥98,000"), "{s}");
    let daily = s
        .lines()
        .find(|l| l.contains('▾') && l.contains("daily"))
        .unwrap_or_else(|| panic!("daily の見出し:\n{s}"));
    assert!(daily.contains("合計 ¥3,500"), "{s}");
}

#[test]
fn test_number_format_counts_stay_plain() {
    // [utest -> req~cell-view.number-format-totals~1] 件数を数える集計(Filled)は「空でない 2」で、`¥` を付けない。
    let (_t, a) = boot("nf_count", TOTALS, FORMATS, "", "件数");
    let s = scr(&a);
    assert!(s.contains("空でない 2"), "{s}");
    assert!(!s.contains("空でない ¥"), "{s}");
}

#[test]
fn test_number_format_chart_values() {
    // [utest -> req~cell-view.number-format-totals~1] 分類ごとの金額(合計)のグラフの棒の値の字は `¥98,000` と `¥3,500`。
    let (_t, mut a) = boot("nf_chart", TOTALS, FORMATS, "", "合計");
    a.open_chart();
    assert_eq!(a.mode, keymap::Mode::Chart);
    let s = scr(&a);
    assert!(s.contains("¥98,000"), "{s}");
    assert!(s.contains("¥3,500"), "{s}");
}

#[test]
fn test_number_format_locale_separators() {
    // [utest -> req~cell-view.number-format-locale~1] 1234.5 は英語と日本語で `1,234.5`、ドイツ語で `1.234,5`、フランス語で
    // `1 234,5`(区切りは U+202F)。
    let notes = [("alpha.md", "---\ndistance: 1234.5\n---\n")];
    let top =
        "[types]\ndistance = \"number\"\n\n[format.distance]\nthousands = true\ndecimals = 1\n";
    for (lang, want) in [
        (Lang::En, "1,234.5"),
        (Lang::Ja, "1,234.5"),
        (Lang::De, "1.234,5"),
        (Lang::Fr, "1\u{202F}234,5"),
    ] {
        let _g = scoped(lang);
        let (_t, a) = boot(
            &format!("nf_locale_{}", lang.code()),
            &notes,
            top,
            "",
            "全部",
        );
        let s = scr(&a);
        assert!(line_of(&s, "alpha").contains(want), "{lang:?}:\n{s}");
    }
}

#[test]
fn test_number_format_invalid_items_warned() {
    // [utest -> req~cell-view.number-format-invalid~1] `decimals = 20` と知らない `comma` は警告して使わず、`prefix` は効く(`¥98000`)。
    let conf = "[format.amount]\nprefix = \"¥\"\ndecimals = 20\ncomma = true\n";
    let (_c, warns) = mdgrid::config::parse(conf).unwrap();
    assert!(
        warns.iter().any(|w| w.contains("format.amount.decimals")),
        "{warns:?}"
    );
    assert!(
        warns.iter().any(|w| w.contains("format.amount.comma")),
        "{warns:?}"
    );
    let top = "[types]\namount = \"number\"\n";
    let (_t, a) = boot("nf_invalid", CELLS, top, conf, "全部");
    let s = scr(&a);
    assert!(line_of(&s, "alpha").contains("¥98000"), "{s}");
}

#[test]
fn test_number_format_control_chars_warned() {
    // [utest -> req~cell-view.number-format-invalid~1] 頭の字の ESC と末尾の字の U+202E は警告して使わない。ほかの項目
    // (thousands)は効く(`98,000`)。画面の字に ESC も U+202E も出ない。
    let conf =
        "[format.amount]\nprefix = \"\\u001b[31m\"\nsuffix = \"\\u202e\"\nthousands = true\n";
    let (_c, warns) = mdgrid::config::parse(conf).unwrap();
    assert!(
        warns.iter().any(|w| w.contains("format.amount.prefix")),
        "{warns:?}"
    );
    assert!(
        warns.iter().any(|w| w.contains("format.amount.suffix")),
        "{warns:?}"
    );
    let top = "[types]\namount = \"number\"\n";
    let (_t, a) = boot("nf_ctrl", CELLS, top, conf, "全部");
    let s = scr(&a);
    assert!(line_of(&s, "alpha").contains("98,000"), "{s}");
    assert!(!s.contains('\u{1b}') && !s.contains('\u{202e}'), "{s:?}");
}

#[test]
fn test_number_format_control_chars_replaced_on_screen() {
    // [utest -> req~cell-view.number-format-invalid~1] 読み込みを通らずに頭の字が ESC の見せ方が当たっても、セル・集計の行・
    // グラフの値の字に ESC を出さず、見える字 `␛` に置き換える。
    let top = "[types]\namount = \"number\"\n";
    let (_t, mut a) = boot("nf_ctrl_draw", TOTALS, top, "", "合計");
    a.prof.resolved.format.insert(
        "amount".into(),
        mdgrid::numfmt::NumFormat {
            prefix: "\u{1b}[31m".into(),
            ..mdgrid::numfmt::NumFormat::default()
        },
    );
    a.refresh();
    let s = scr(&a);
    assert!(!s.contains('\u{1b}'), "{s:?}");
    assert!(line_of(&s, "alpha").contains("␛[31m98000"), "{s}");
    assert!(s.contains("合計 ␛[31m101500"), "{s}");
    a.open_chart();
    let s = scr(&a);
    assert!(!s.contains('\u{1b}'), "{s:?}");
    assert!(s.contains("␛[31m98000"), "{s}");
}
