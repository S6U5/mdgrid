//! `.base` のビューの集計(summaries)の画面の試験(BV-14・BV-7・SR-23・SR-9)。

use super::test_grid::open;
use super::test_screen::{assert_fits, ch, press, screen, Tmp};
use super::*;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::summary::Summary;
use ratatui::crossterm::event::KeyCode;

/// 下の帯の行(高さ 24 の端末。最下行は空ける)。集計の行はその1つ上。
const BAND: usize = 21;
const SUMMARY: usize = BAND - 1;

/// estimate(数)・due(日付)・done(チェック)の4つのノート。型の合わない値を1つずつ混ぜる。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date", "estimate": "number", "done": "checkbox"}}"#,
    )
    .unwrap();
    tmp.write(
        "a.md",
        "---\ntitle: 会議\nestimate: 3\ndue: 2026-10-05\ndone: true\n---\n",
    );
    tmp.write(
        "b.md",
        "---\ntitle: 買い物\nestimate: 5\ndue: 2026-10-07\ndone: true\n---\n",
    );
    tmp.write(
        "c.md",
        "---\ntitle: 読書\nestimate: 10\ndue: someday\ndone: false\n---\n",
    );
    tmp.write(
        "d.md",
        "---\ntitle: 掃除\nestimate: many\ndone: false\n---\n",
    );
    tmp
}

const BASE: &str = r#"summaries:
  customAverage: 'values.mean()'
views:
  - type: table
    name: All
    order: [title, estimate, due, done]
    summaries:
      estimate: Sum
      note.due: Earliest
      done: Checked
  - type: table
    name: Plain
    order: [title, estimate, due, done]
  - type: table
    name: Formula
    order: [title, estimate]
    summaries:
      estimate: customAverage
      title: Filled
  - type: table
    name: Grouped
    groupBy:
      property: done
      direction: ASC
    order: [title, estimate]
    summaries:
      estimate: Sum
"#;

/// 今の言語の「集計の名前 値」。
fn t(sm: Summary, v: &str) -> String {
    format!("{} {v}", sm.label())
}

fn lines(s: &str) -> Vec<String> {
    s.lines().map(str::to_string).collect()
}

/// `line` の中の `word` の始まりの桁。
fn x_of(line: &str, word: &str) -> usize {
    let at = line
        .find(word)
        .unwrap_or_else(|| panic!("{word} が無い: {line}"));
    width::width(&line[..at])
}

#[test]
fn test_bv_14_summary_row_under_table() {
    // [BV-14] 表の下・下の帯の上に1行。集計した列の下に「名前 値」。型の合わない値は数えない。
    let tmp = vault("bv14row");
    let app = open(&tmp, BASE, None);
    assert_eq!(app.rows.len(), 4);
    let s = screen(&app);
    let ls = lines(&s);
    let row = &ls[SUMMARY];
    assert!(row.contains("集計"), "{s}");
    assert!(row.contains(&t(Summary::Sum, "18")), "{s}");
    assert!(row.contains(&t(Summary::Earliest, "2026-10-05")), "{s}");
    assert!(row.contains(&t(Summary::Checked, "2")), "{s}");
    assert_eq!(
        ls.iter()
            .filter(|l| l.contains(&t(Summary::Sum, "18")))
            .count(),
        1,
        "1行だけ: {s}"
    );
}

#[test]
fn test_bv_14_summary_aligned_to_columns() {
    // [BV-14] 集計は列の位置にそろう(数の列は右寄せ、ほかは左寄せ)。
    let tmp = vault("bv14align");
    let app = open(&tmp, BASE, None);
    let s = screen(&app);
    let ls = lines(&s);
    let head = ls
        .iter()
        .find(|l| l.contains("estimate") && l.contains("due"))
        .unwrap_or_else(|| panic!("列の見出し: {s}"));
    let row = &ls[SUMMARY];
    let sum = t(Summary::Sum, "18");
    assert_eq!(
        x_of(row, &sum) + width::width(&sum),
        x_of(head, "estimate") + "estimate".len(),
        "数の列は右の端がそろう: {s}"
    );
    assert_eq!(
        x_of(row, Summary::Earliest.label()),
        x_of(head, "due"),
        "{s}"
    );
    assert_eq!(
        x_of(row, Summary::Checked.label()),
        x_of(head, "done"),
        "{s}"
    );
}

#[test]
fn test_bv_14_filter_changes_values() {
    // [BV-14] 絞り込み(NV-2)で行が減ると、集計の値も変わる。解くと戻る。
    let tmp = vault("bv14filter");
    let mut app = open(&tmp, BASE, None);
    ch(&mut app, '\\');
    ch(&mut app, '会');
    ch(&mut app, '議');
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.rows.len(), 1);
    let s = screen(&app);
    let row = &lines(&s)[SUMMARY];
    assert!(row.contains(&t(Summary::Sum, "3")), "{s}");
    assert!(row.contains(&t(Summary::Earliest, "2026-10-05")), "{s}");
    assert!(row.contains(&t(Summary::Checked, "1")), "{s}");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.rows.len(), 4);
    assert!(lines(&screen(&app))[SUMMARY].contains(&t(Summary::Sum, "18")));
}

#[test]
fn test_bv_14_no_summaries_no_row() {
    // [BV-14] 集計の無いビューでは集計の行を出さない(表の行の数も減らない)。
    let tmp = vault("bv14none");
    let mut app = open(&tmp, BASE, None);
    let with = app.data_height();
    ch(&mut app, ']');
    assert_eq!(app.view_index(), 1);
    let s = screen(&app);
    assert!(!s.contains("集計"), "{s}");
    assert!(!s.contains(&t(Summary::Sum, "")), "{s}");
    assert_eq!(app.data_height(), with + 1, "集計の行の分だけ表が広い");
    assert!(app.notes.is_empty(), "{:?}", app.notes);
}

#[test]
fn test_bv_14_formula_summary_unsupported_view_opens() {
    // [BV-14][BV-7] 式で書いた集計は未対応の理由が出て、ビューは開く。組み込みの集計は出る。
    let tmp = vault("bv14formula");
    let app = open(&tmp, BASE, Some("Formula"));
    assert!(app.view_error.is_none(), "{:?}", app.view_error);
    assert_eq!(app.rows.len(), 4);
    assert!(
        app.notes
            .iter()
            .any(|n| n.contains("customAverage") && n.contains("values.mean()")),
        "{:?}",
        app.notes
    );
    let s = screen(&app);
    let row = &lines(&s)[SUMMARY];
    assert!(row.contains(&t(Summary::Filled, "4")), "{s}");
    assert!(!row.contains("customAverage"), "{s}");
}

#[test]
fn test_bv_14_grouped_view_total_and_reason() {
    // [BV-14] まとまりごとの集計は未対応の理由を出し、表全体の集計を1行出す。
    let tmp = vault("bv14group");
    let app = open(&tmp, BASE, Some("Grouped"));
    assert!(app.view_error.is_none());
    assert!(!app.groups.is_empty());
    assert!(!app.notes.is_empty(), "まとまりごとの集計の理由");
    let s = screen(&app);
    assert!(lines(&s)[SUMMARY].contains(&t(Summary::Sum, "18")), "{s}");
}

#[test]
fn test_bv_14_english_names() {
    // [BV-14][SR-23] 英語の画面では見出しも英語(Summary)、集計の名前も英語。
    let _g = scoped(Lang::En);
    let tmp = vault("bv14en");
    let app = open(&tmp, BASE, None);
    let s = screen(&app);
    let row = &lines(&s)[SUMMARY];
    assert!(row.contains("Summary"), "{s}");
    assert!(
        row.contains(&t(Summary::Sum, "18")) && row.contains(&t(Summary::Checked, "2")),
        "{s}"
    );
    assert!(row.is_ascii(), "日本語が無い: {row}");
}

#[test]
fn test_bv_14_japanese_names() {
    // [BV-14][SR-23] 日本語の画面では、Obsidian の名前の代わりに日本語の名前で出す。
    let _g = scoped(Lang::Ja);
    let tmp = vault("bv14ja");
    let app = open(&tmp, BASE, None);
    let s = screen(&app);
    let row = &lines(&s)[SUMMARY];
    let sum = Summary::Sum.label();
    assert_ne!(sum, "Sum");
    assert!(row.contains(&format!("{sum} 18")), "{s}");
}

#[test]
fn test_bv_14_fits_narrow_terminals() {
    // [BV-14][SR-9] 80 桁と狭い端末で、どの行も幅を超えない。
    let tmp = vault("bv14fit");
    let mut app = open(&tmp, BASE, None);
    assert_fits(&mut app);
    ch(&mut app, ']');
    ch(&mut app, ']');
    assert_fits(&mut app);
}
