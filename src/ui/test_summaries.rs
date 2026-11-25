//! `.base` のビューの集計(summaries)の画面の試験(BV-14・BV-7・SR-23・SR-9)。

use super::test_grid::open;
use super::test_screen::{assert_fits, ch, press, screen, typing, Tmp};
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

const BASE: &str = r"summaries:
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
";

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
    // [utest -> req~base-view.summaries-builtin~2] 表の下・下の帯の上に1行。集計した列の下に「名前 値」。型の合わない値は数えない。
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
    // [utest -> req~base-view.summaries-builtin~2] 集計は列の位置にそろう(数の列は右寄せ、ほかは左寄せ)。
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
    // [utest -> req~base-view.summaries-builtin~2, req~base-view.summaries-filtered~1] 絞り込み(NV-2)で行が減ると、集計の値も変わる。解くと戻る。
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
    // [utest -> req~base-view.summaries-builtin~2, req~base-view.summaries-filtered~1] 集計の無いビューでは集計の行を出さない(表の行の数も減らない)。
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
    // [utest -> req~base-view.summaries-unsupported~2, req~base-view.unsupported-filters~1] 式で書いた集計は未対応の理由が出て、ビューは開く。組み込みの集計は出る。
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

/// まとまりの見出しの行(「件)」か「)」で終わる見出しの字のある行)のうち、`word` を含む行。
fn heading_with(s: &str, word: &str) -> String {
    s.lines()
        .find(|l| (l.contains('▾') || l.contains('▸')) && l.contains(word))
        .unwrap_or_else(|| panic!("「{word}」のまとまりの見出しが無い:\n{s}"))
        .to_string()
}

#[test]
fn test_bv_14_grouped_view_total_and_reason() {
    // [utest -> req~base-view.summaries-per-group~1] まとまりのあるビューでも未対応の理由を出さず、表の下の集計の行は表全体の値(18)のまま。
    let tmp = vault("bv14group");
    let app = open(&tmp, BASE, Some("Grouped"));
    assert!(app.view_error.is_none());
    assert!(!app.groups.is_empty());
    assert!(
        app.notes.is_empty(),
        "未対応の理由は出ない: {:?}",
        app.notes
    );
    assert!(app.message.is_none(), "{:?}", app.message);
    let s = screen(&app);
    assert!(lines(&s)[SUMMARY].contains(&t(Summary::Sum, "18")), "{s}");
}

#[test]
fn test_group_summaries_on_headings() {
    // [utest -> req~base-view.summaries-per-group~1] done でまとめたビュー: true のまとまり(3 と 5)の見出しに「合計 8」、
    // false のまとまり(10 と many)の見出しに「合計 10」。どちらも estimate の列の右の端にそろう(数の列は右寄せ)。
    let tmp = vault("bvgroupsum");
    let app = open(&tmp, BASE, Some("Grouped"));
    let s = screen(&app);
    let yes = heading_with(&s, "true");
    let no = heading_with(&s, "false");
    assert!(yes.contains(&t(Summary::Sum, "8")), "{s}");
    assert!(no.contains(&t(Summary::Sum, "10")), "{s}");
    let head = lines(&s)
        .into_iter()
        .find(|l| l.contains("estimate") && !l.contains('▾'))
        .unwrap_or_else(|| panic!("列の見出し: {s}"));
    let sum = t(Summary::Sum, "8");
    assert_eq!(
        x_of(&yes, &sum) + width::width(&sum),
        x_of(&head, "estimate") + "estimate".len(),
        "小計は列の右の端にそろう: {s}"
    );
}

#[test]
fn test_group_summaries_stay_when_folded() {
    // [utest -> req~base-view.summaries-per-group~1] 先頭のまとまりを Enter で畳んでも、見出しの行の小計は出たまま。
    let tmp = vault("bvgroupfold");
    let mut app = open(&tmp, BASE, Some("Grouped"));
    let first = app.groups[0].0.clone();
    let want = if first.contains("true") { "8" } else { "10" };
    assert!(matches!(app.slots[app.row], super::grid::Slot::Head(0)));
    press(&mut app, KeyCode::Enter);
    let s = screen(&app);
    let line = s
        .lines()
        .find(|l| l.contains('▸'))
        .unwrap_or_else(|| panic!("畳んだ見出し: {s}"));
    assert!(line.contains(&t(Summary::Sum, want)), "{s}");
}

#[test]
fn test_group_summaries_follow_filter() {
    // [utest -> req~base-view.summaries-per-group~1, req~base-view.summaries-filtered~1] 簡易の絞り込みで「会議」(3)だけにすると、
    // true のまとまり(3 と 5)の小計は 3 になり、表の下の集計も 3。
    let tmp = vault("bvgroupfilter");
    let mut app = open(&tmp, BASE, Some("Grouped"));
    assert!(heading_with(&screen(&app), "true").contains(&t(Summary::Sum, "8")));
    ch(&mut app, '\\');
    typing(&mut app, "会議");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.rows.len(), 1);
    let s = screen(&app);
    assert!(
        heading_with(&s, "true").contains(&t(Summary::Sum, "3")),
        "{s}"
    );
    assert!(lines(&s)[SUMMARY].contains(&t(Summary::Sum, "3")), "{s}");
}

#[test]
fn test_group_summaries_english() {
    // [utest -> req~base-view.summaries-per-group~1, req~base-view.summaries-names~2] 英語の画面では見出しの小計も「Sum 8」。
    let _g = scoped(Lang::En);
    let tmp = vault("bvgroupen");
    let app = open(&tmp, BASE, Some("Grouped"));
    let s = screen(&app);
    assert!(heading_with(&s, "true").contains("Sum 8"), "{s}");
}

#[test]
fn test_group_summaries_fit_narrow_terminals() {
    // [utest -> req~base-view.summaries-per-group~1, req~cell-view.grapheme-width-fit~1] まとまりの小計のある見出しの行も、80 桁と狭い端末で幅を超えない。
    let tmp = vault("bvgroupfit");
    let mut app = open(&tmp, BASE, Some("Grouped"));
    assert_fits(&mut app);
}

const NAMED: &str = r"views:
  - type: table
    name: Named
    order: [title, estimate, due]
    summaries:
      title: Filled
      estimate: Sum
      due: Earliest
    summary_labels:
      title: 件名の数
      due: 3
";

#[test]
fn test_summary_labels_rename() {
    // [utest -> req~base-view.summary-labels~1] `summary_labels` の title: 件名の数 → 集計の行に「件名の数 4」、「空でない」は出ない。
    // 名前を書かない estimate は「合計 18」のまま。文字でない名前(due: 3)は組み込みの名前のまま。
    let tmp = vault("bvlabels");
    let app = open(&tmp, NAMED, None);
    let s = screen(&app);
    let row = &lines(&s)[SUMMARY];
    assert!(row.contains("件名の数 4"), "{s}");
    assert!(!row.contains(Summary::Filled.label()), "{s}");
    assert!(row.contains(&t(Summary::Sum, "18")), "{s}");
    assert!(row.contains(&t(Summary::Earliest, "2026-10-05")), "{s}");
}

#[test]
fn test_summary_labels_any_language() {
    // [utest -> req~base-view.summary-labels~1] 英語の画面でも書いた名前「件名の数 4」。
    let _g = scoped(Lang::En);
    let tmp = vault("bvlabelsen");
    let app = open(&tmp, NAMED, None);
    let s = screen(&app);
    assert!(lines(&s)[SUMMARY].contains("件名の数 4"), "{s}");
}

#[test]
fn test_summary_labels_on_group_headings() {
    // [utest -> req~base-view.summary-labels~1, req~base-view.summaries-per-group~1] まとまりの見出しの小計も書いた名前で出る。
    let base = r"views:
  - type: table
    name: G
    groupBy:
      property: done
      direction: ASC
    order: [title, estimate]
    summaries:
      estimate: Sum
    summary_labels:
      estimate: 見積もり
";
    let tmp = vault("bvlabelsgroup");
    let app = open(&tmp, base, None);
    let s = screen(&app);
    assert!(heading_with(&s, "true").contains("見積もり 8"), "{s}");
    assert!(lines(&s)[SUMMARY].contains("見積もり 18"), "{s}");
}

#[test]
fn test_bv_14_english_names() {
    // [utest -> req~base-view.summaries-builtin~2, req~base-view.summaries-names~2, req~screen.locales-two-languages~1, req~screen.locales-auto~3] 英語の画面では見出しも英語(Summary)、集計の名前も英語。
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
    // [utest -> req~base-view.summaries-builtin~2, req~base-view.summaries-names~2, req~screen.locales-two-languages~1] 日本語の画面では、Obsidian の名前の代わりに日本語の名前で出す。
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
    // [utest -> req~base-view.summaries-builtin~2, req~base-view.summaries-filtered~1, req~base-view.summaries-type-mismatch~1, req~cell-view.grapheme-width-fit~1] 80 桁と狭い端末で、どの行も幅を超えない。
    let tmp = vault("bv14fit");
    let mut app = open(&tmp, BASE, None);
    assert_fits(&mut app);
    ch(&mut app, ']');
    ch(&mut app, ']');
    assert_fits(&mut app);
}

#[test]
fn test_summary_labels_control_chars_replaced() {
    // [utest -> req~base-view.summary-labels~1] 名前の ESC は、集計の行にもまとまりの見出しの行にも出さず、見える字 `␛` にする。
    let base = r#"views:
  - type: table
    name: G
    groupBy:
      property: done
      direction: ASC
    order: [title, estimate]
    summaries:
      estimate: Sum
    summary_labels:
      estimate: "見\e[31m積"
"#;
    let tmp = vault("bvlabelsctrl");
    let app = open(&tmp, base, None);
    let s = screen(&app);
    assert!(!s.contains('\u{1b}'), "{s:?}");
    assert!(heading_with(&s, "true").contains("見␛[31m積 8"), "{s}");
    assert!(lines(&s)[SUMMARY].contains("見␛[31m積 18"), "{s}");
}
