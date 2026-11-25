//! 読むだけの理由に前置き「読むだけ: 」を付けて出す所で、理由の文に同じ語を重ねない(screen.read-only-reason-once)試験。
//!
//! 期待値は要求の受け入れの例から取った: 式の列のセルの案内は「読むだけ: 式の列」、値が3つで列が2つの CSV の行は
//! 「読むだけ: この行は値が 3 個で、列は 2 個」。文言の全部の言語で、理由の文が前置きの語を含まないことも確かめる。

use super::app::App;
use super::test_screen::{screen, Tmp};
use super::ColorMode;
use mdgrid::i18n::{self, Lang, Msg};
use mdgrid::source::csv::Csv;
use mdgrid::source::markdown::Markdown;

/// 読むだけの理由の文言(ノートと CSV の行の理由)。
fn reasons() -> Vec<Msg> {
    Msg::ALL
        .iter()
        .copied()
        .filter(|m| {
            let k = format!("{m:?}");
            k.starts_with("Lock") || k == "CsvColumnCount"
        })
        .collect()
}

#[test]
fn test_read_only_reasons_do_not_repeat_the_lead_in_any_language() {
    // [utest -> req~screen.read-only-reason-once~1] 読むだけの理由の文言(全部の言語)は、どれもその言語の前置きの語
    // (日本語は「読むだけ」、英語は "read-only")を含まない。
    let list = reasons();
    assert!(list.len() > 10, "理由の文言が少ない: {list:?}");
    let mut bad = Vec::new();
    for &lang in Lang::ALL {
        let lead = Msg::ReadOnlyLead.get(lang);
        let word = lead.trim().trim_end_matches([':', '：']).trim();
        assert!(!word.is_empty(), "{lang:?} の前置きが空");
        for m in &list {
            let text = m.get(lang);
            if text.contains(word) {
                bad.push(format!("{lang:?} {m:?}: {text}"));
            }
        }
    }
    assert!(bad.is_empty(), "前置きの語を重ねる理由: {bad:#?}");
}

/// 画面の中で `what` を含む最初の行。
fn line_with(s: &str, what: &str) -> String {
    s.lines()
        .find(|l| l.contains(what))
        .unwrap_or_else(|| panic!("「{what}」の行が無い:\n{s}"))
        .to_string()
}

#[test]
fn test_read_only_formula_cell_hint_says_it_once() {
    // [utest -> req~screen.read-only-reason-once~1] `.base` の式の列のセルを選ぶと、案内は「読むだけ: 式の列」。
    let _ja = i18n::scoped(Lang::Ja);
    let t = Tmp::new("ro_formula");
    t.write("a.md", "---\nqty: 2\n---\n");
    let yaml = "formulas:\n  twice: 'qty * 2'\nviews:\n  - type: table\n    name: V\n    order:\n      - formula.twice\n      - qty\n";
    t.write("T.base", yaml);
    let path = t.notes().join("T.base");
    let base = mdgrid::base::parse(yaml).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    a.start(super::startup::Startup {
        config: mdgrid::config::Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(t.0.join("state")),
        config_dir: Some(t.0.join("config")),
        target: path,
        base: Some((base, "T.base".into(), None)),
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.col = a.cols.iter().position(|c| c == "formula.twice").unwrap();
    let s = screen(&a);
    let line = line_with(&s, "読むだけ: ");
    assert!(line.contains("読むだけ: 式の列"), "{line}");
    assert_eq!(line.matches("読むだけ").count(), 1, "{line}");
}

#[test]
fn test_read_only_ragged_csv_row_hint_says_it_once() {
    // [utest -> req~screen.read-only-reason-once~1] 値が3つで列が2つの CSV の行のセルを選ぶと、案内は
    // 「読むだけ: この行は値が 3 個で、列は 2 個」。
    let _ja = i18n::scoped(Lang::Ja);
    let t = Tmp::new("ro_csv");
    t.write("t.csv", "name,qty\nりんご,3\nbad,1,9\n");
    let src = Csv::open(&t.notes().join("t.csv")).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    let at = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "bad")
        .expect("bad の行");
    a.row = a
        .slots
        .iter()
        .position(|s| *s == super::grid::Slot::Row(at))
        .unwrap();
    a.col = 0;
    let s = screen(&a);
    let line = line_with(&s, "読むだけ: ");
    assert!(
        line.contains("読むだけ: この行は値が 3 個で、列は 2 個"),
        "{line}"
    );
    assert_eq!(line.matches("読むだけ").count(), 1, "{line}");
}
