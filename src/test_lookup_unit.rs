//! つながったノートの値を式で引く(ルックアップ)・集める(ロールアップ)試験。一時のルートフォルダ(根に `.obsidian/`)に
//! ノートを書いて読み終えた読み込み口で、式の列(`formula.x`)のセルの値を確かめる。期待値は要求の受け入れの例から取った。

use super::*;
use crate::source::markdown::Markdown;
use std::cell::RefCell;
use std::path::PathBuf;

struct Tmp(PathBuf);
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `files`(根からのパス, 中身)を書いて読み終えた読み込み口。
fn ledger(tag: &str, files: &[(&str, &str)]) -> (Tmp, Markdown) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!(
        "mdgrid-lookup-{tag}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(d.join(".obsidian")).unwrap();
    for (rel, text) in files {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    std::fs::write(d.join("t.base"), "views: []\n").unwrap();
    let mut md = Markdown::open_vault(&d.join("t.base")).unwrap();
    while !md.load(500).done {}
    (Tmp(d), md)
}

/// 式 `expr` だけの式の列の `.base`。
fn base_of(expr: &str) -> Base {
    parse(&format!(
        "formulas:\n  x: '{}'\nviews:\n  - type: table\n    name: v\n    order: [formula.x]\n",
        expr.replace('\'', "''")
    ))
    .unwrap()
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| panic!("{label} の行が無い"))
}

/// 行 `label` で式 `expr` を評価した値(`prop` はためた値を重ねた値の口)。
fn eval_with_prop(
    src: &dyn Source,
    prop: &dyn Fn(&RowId, &str) -> Option<Value>,
    label: &str,
    expr: &str,
) -> Val {
    let b = base_of(expr);
    match b.cell(
        src,
        prop,
        &row_of(src, label),
        "formula.x",
        20_782,
        20_782 * 86_400,
    ) {
        Shown::Computed(v) => v,
        other => panic!("{expr}: 値でない {other:?}"),
    }
}

/// 行 `label` で式 `expr` を評価した値。
fn eval_at(src: &dyn Source, label: &str, expr: &str) -> Val {
    let prop = |r: &RowId, c: &str| src.get(r, c).value;
    eval_with_prop(src, &prop, label, expr)
}

fn s(x: &str) -> Val {
    Val::Str(x.to_string())
}

fn n(x: f64) -> Val {
    Val::Num(x)
}

fn list(xs: Vec<Val>) -> Val {
    Val::List(xs)
}

/// 読書の台帳: 本と抜き書き。
const READING: &[(&str, &str)] = &[
    (
        "Library/霧の灯台守.md",
        "---\nkind: book\nauthor: 佐伯 真澄\n---\n# 霧の灯台守\n",
    ),
    ("Quotes/q.md", "---\nsource: \"[[霧の灯台守]]\"\n---\n"),
    ("Quotes/q_missing.md", "---\nsource: \"[[無い本]]\"\n---\n"),
    ("Quotes/q_plain.md", "---\nsource: 霧の灯台守\n---\n"),
];

/// 人と会議の台帳。
const PEOPLE: &[(&str, &str)] = &[
    (
        "People/安藤 真.md",
        "---\norg: \"[[ノースウィンド工房]]\"\n---\n",
    ),
    ("People/岡田 健.md", "---\nrole: dev\n---\n"),
    ("Orgs/ノースウィンド工房.md", "---\nkind: own\n---\n"),
    (
        "Meetings/m.md",
        "---\nstatus: done\nattendees:\n  - \"[[安藤 真]]\"\n  - \"[[岡田 健]]\"\n---\n",
    ),
];

/// 調べ物の台帳: 問い Q を、ニュース n1 が question で、問い q2 が related で、n3 が本文で指す。
const RESEARCH: &[(&str, &str)] = &[
    ("Questions/Q.md", "---\nstatus: researching\n---\n"),
    ("News/n1.md", "---\nquestion:\n  - \"[[Q]]\"\n---\n"),
    ("Questions/q2.md", "---\nrelated:\n  - \"[[Q]]\"\n---\n"),
    ("n3.md", "---\nmemo: x\n---\n本文から [[Q]] を指す\n"),
];

#[test]
fn test_lookup_as_file_resolves_link() {
    // [utest -> req~base-view.lookup-as-file~1] q の source が "[[霧の灯台守]]" → source.asFile().path は Library/霧の灯台守.md
    let (_t, src) = ledger("asfile", READING);
    assert_eq!(
        eval_at(&src, "Quotes/q.md", "source.asFile().path"),
        s("Library/霧の灯台守.md")
    );
}

#[test]
fn test_lookup_as_file_unresolved_is_null() {
    // [utest -> req~base-view.lookup-as-file~1] 解けないリンク "[[無い本]]" と括弧の無い名前 "霧の灯台守" は、どちらも null
    let (_t, src) = ledger("asfile_null", READING);
    assert_eq!(
        eval_at(&src, "Quotes/q_missing.md", "source.asFile()"),
        Val::Null
    );
    assert_eq!(
        eval_at(&src, "Quotes/q_plain.md", "source.asFile()"),
        Val::Null
    );
}

#[test]
fn test_lookup_as_file_on_backlinks_items() {
    // [utest -> req~base-view.lookup-as-file~1] 霧の灯台守の行で file.backlinks.map(value.asFile().name) は ["q.md"]
    let (_t, src) = ledger("asfile_back", READING);
    assert_eq!(
        eval_at(
            &src,
            "Library/霧の灯台守.md",
            "file.backlinks.map(value.asFile().name)"
        ),
        list(vec![s("q.md")])
    );
}

#[test]
fn test_lookup_properties_reads_frontmatter() {
    // [utest -> req~base-view.lookup-properties~1] q の行で .properties.author・.properties["kind"]・無いキー → 佐伯 真澄・book・null
    let (_t, src) = ledger("props", READING);
    let q = "Quotes/q.md";
    assert_eq!(
        eval_at(&src, q, "source.asFile().properties.author"),
        s("佐伯 真澄")
    );
    assert_eq!(
        eval_at(&src, q, "source.asFile().properties[\"kind\"]"),
        s("book")
    );
    assert_eq!(
        eval_at(&src, q, "source.asFile().properties.isbn"),
        Val::Null
    );
}

#[test]
fn test_lookup_properties_pending_value() {
    // [utest -> req~base-view.lookup-properties~1] 霧の灯台守の author を 佐伯 真澄2 にためた(保存の前)→ q の行の引いた値も 佐伯 真澄2
    let (_t, src) = ledger("props_pending", READING);
    let book = row_of(&src, "Library/霧の灯台守.md");
    let prop = |r: &RowId, c: &str| {
        if *r == book && c == "author" {
            Some(Value::Str("佐伯 真澄2".to_string()))
        } else {
            src.get(r, c).value
        }
    };
    assert_eq!(
        eval_with_prop(
            &src,
            &prop,
            "Quotes/q.md",
            "source.asFile().properties.author"
        ),
        s("佐伯 真澄2")
    );
}

#[test]
fn test_lookup_properties_on_link_text() {
    // [utest -> req~base-view.lookup-properties~1] asFile() を省いた source.properties.author も 佐伯 真澄
    let (_t, src) = ledger("props_text", READING);
    assert_eq!(
        eval_at(&src, "Quotes/q.md", "source.properties.author"),
        s("佐伯 真澄")
    );
}

#[test]
fn test_lookup_link_values_contains_this_note() {
    // [utest -> req~base-view.lookup-link-values~1] 安藤 真の行で file.backlinks.filter(value.properties.attendees.contains(file)).length は 1
    let (_t, src) = ledger("linkvals", PEOPLE);
    assert_eq!(
        eval_at(
            &src,
            "People/安藤 真.md",
            "file.backlinks.filter(value.properties.attendees.contains(file)).length"
        ),
        n(1.0)
    );
}

#[test]
fn test_lookup_link_values_chain() {
    // [utest -> req~base-view.lookup-link-values~1] m の行で attendees[0].asFile().properties.org.properties.kind は own
    let (_t, src) = ledger("linkchain", PEOPLE);
    assert_eq!(
        eval_at(
            &src,
            "Meetings/m.md",
            "attendees[0].asFile().properties.org.properties.kind"
        ),
        s("own")
    );
}

#[test]
fn test_lookup_link_values_compare_with_file() {
    // [utest -> req~base-view.lookup-link-values~1] 安藤 真の行で "[[安藤 真]]" == file・リストの contains(file)・"[[岡田 健]]" == file → true・true・false
    let (_t, src) = ledger("linkcmp", PEOPLE);
    let me = "People/安藤 真.md";
    assert_eq!(
        eval_at(&src, me, "\"[[安藤 真]]\" == file"),
        Val::Bool(true)
    );
    assert_eq!(
        eval_at(
            &src,
            me,
            "[\"[[岡田 健]]\", \"[[安藤 真]]\"].contains(file)"
        ),
        Val::Bool(true)
    );
    assert_eq!(
        eval_at(&src, me, "\"[[岡田 健]]\" == file"),
        Val::Bool(false)
    );
}

#[test]
fn test_lookup_file_members_on_note_values() {
    // [utest -> req~base-view.lookup-file-members~1] source.asFile() の name・folder・inFolder("Library")・inFolder("Quotes") → 霧の灯台守.md・Library・true・false
    let (_t, src) = ledger("members", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "source.asFile().name"), s("霧の灯台守.md"));
    assert_eq!(eval_at(&src, q, "source.asFile().folder"), s("Library"));
    assert_eq!(
        eval_at(&src, q, "source.asFile().inFolder(\"Library\")"),
        Val::Bool(true)
    );
    assert_eq!(
        eval_at(&src, q, "source.asFile().inFolder(\"Quotes\")"),
        Val::Bool(false)
    );
}

#[test]
fn test_lookup_file_members_bare_file() {
    // [utest -> req~base-view.lookup-file-members~1] q の行で file.name と file.asFile().name はどちらも q.md、file == source は false
    let (_t, src) = ledger("bare_file", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "file.name"), s("q.md"));
    assert_eq!(eval_at(&src, q, "file.asFile().name"), s("q.md"));
    assert_eq!(eval_at(&src, q, "file == source"), Val::Bool(false));
}

#[test]
fn test_lookup_file_members_not_a_note() {
    // [utest -> req~base-view.lookup-file-members~1] "abc".name・(3).inFolder("x")・[1].path はどれも null
    let (_t, src) = ledger("not_note", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "\"abc\".name"), Val::Null);
    assert_eq!(eval_at(&src, q, "(3).inFolder(\"x\")"), Val::Null);
    assert_eq!(eval_at(&src, q, "[1].path"), Val::Null);
}

#[test]
fn test_list_filter_map_reduce() {
    // [utest -> req~base-view.list-filter-map~1] [1,2,3,4].filter(value > 2)・[1,2,3].map(value * 10 + index)・[1,2,3].reduce(acc + value, 0) → [3, 4]・[10, 21, 32]・6
    let (_t, src) = ledger("lambda", READING);
    let q = "Quotes/q.md";
    assert_eq!(
        eval_at(&src, q, "[1, 2, 3, 4].filter(value > 2)"),
        list(vec![n(3.0), n(4.0)])
    );
    assert_eq!(
        eval_at(&src, q, "[1, 2, 3].map(value * 10 + index)"),
        list(vec![n(10.0), n(21.0), n(32.0)])
    );
    assert_eq!(eval_at(&src, q, "[1, 2, 3].reduce(acc + value, 0)"), n(6.0));
}

#[test]
fn test_list_filter_map_note_value_is_frontmatter() {
    // [utest -> req~base-view.list-filter-map~1] value: 7 の行で [1, 2].map(note.value) は [7, 7]、外の value は 7
    let files = [("v.md", "---\nvalue: 7\n---\n")];
    let (_t, src) = ledger("lambda_note", &files);
    assert_eq!(
        eval_at(&src, "v.md", "[1, 2].map(note.value)"),
        list(vec![n(7.0), n(7.0)])
    );
    assert_eq!(eval_at(&src, "v.md", "value"), n(7.0));
}

#[test]
fn test_list_filter_map_not_a_list() {
    // [utest -> req~base-view.list-filter-map~1] "abc".filter(true)・null.map(value) はどちらも null
    let (_t, src) = ledger("lambda_null", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "\"abc\".filter(true)"), Val::Null);
    assert_eq!(eval_at(&src, q, "null.map(value)"), Val::Null);
}

#[test]
fn test_list_aggregate_numbers() {
    // [utest -> req~base-view.list-aggregate~1] [3, "a", 5, null].max()・[3, 5].min()・[1, 2.5, "x"].sum() → 5・3・3.5
    let (_t, src) = ledger("agg_num", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "[3, \"a\", 5, null].max()"), n(5.0));
    assert_eq!(eval_at(&src, q, "[3, 5].min()"), n(3.0));
    assert_eq!(eval_at(&src, q, "[1, 2.5, \"x\"].sum()"), n(3.5));
}

#[test]
fn test_list_aggregate_dates() {
    // [utest -> req~base-view.list-aggregate~1] 日付の max・min は日付の 2026-11-20・2026-11-01
    let (_t, src) = ledger("agg_date", READING);
    let q = "Quotes/q.md";
    let day = |t: &str| Val::Date(crate::types::parse_date(t).unwrap());
    assert_eq!(
        eval_at(
            &src,
            q,
            "[date(\"2026-11-01\"), date(\"2026-11-20\"), \"x\"].max()"
        ),
        day("2026-11-20")
    );
    assert_eq!(
        eval_at(
            &src,
            q,
            "[date(\"2026-11-01\"), date(\"2026-11-20\")].min()"
        ),
        day("2026-11-01")
    );
}

#[test]
fn test_list_aggregate_empty() {
    // [utest -> req~base-view.list-aggregate~1] [].max()・["a"].min()・[].sum() → null・null・0
    let (_t, src) = ledger("agg_empty", READING);
    let q = "Quotes/q.md";
    assert_eq!(eval_at(&src, q, "[].max()"), Val::Null);
    assert_eq!(eval_at(&src, q, "[\"a\"].min()"), Val::Null);
    assert_eq!(eval_at(&src, q, "[].sum()"), n(0.0));
}

#[test]
fn test_linked_from_counts_by_column() {
    // [utest -> req~base-view.linked-from~1] Q の行で linkedFrom("question")・("related")・("question", "related") の数と backlinks の数 → 1・1・2・3
    let (_t, src) = ledger("linked", RESEARCH);
    let q = "Questions/Q.md";
    assert_eq!(
        eval_at(&src, q, "file.linkedFrom(\"question\").length"),
        n(1.0)
    );
    assert_eq!(
        eval_at(&src, q, "file.linkedFrom(\"related\").length"),
        n(1.0)
    );
    assert_eq!(
        eval_at(&src, q, "file.linkedFrom(\"question\", \"related\").length"),
        n(2.0)
    );
    assert_eq!(eval_at(&src, q, "file.backlinks.length"), n(3.0));
}

#[test]
fn test_linked_from_items_are_notes_in_path_order() {
    // [utest -> req~base-view.linked-from~1] Q の行で file.linkedFrom("question", "related").map(value.name) は ["n1.md", "q2.md"]
    let (_t, src) = ledger("linked_items", RESEARCH);
    assert_eq!(
        eval_at(
            &src,
            "Questions/Q.md",
            "file.linkedFrom(\"question\", \"related\").map(value.name)"
        ),
        list(vec![s("n1.md"), s("q2.md")])
    );
}

#[test]
fn test_lookup_speed_reads_only_linked_notes() {
    // [utest -> req~base-view.lookup-speed~1] p を attendees に持つ会議3つと、つながらないノート 1000 の台帳で、ロールアップの式は
    // その3つの会議の値だけを読む(ほかのノートのフロントマターを読まない)。
    let mut files: Vec<(String, String)> =
        vec![("People/p.md".into(), "---\nrole: x\n---\n".into())];
    for (i, day) in ["2026-11-09", "2026-11-24", "2026-11-30"]
        .iter()
        .enumerate()
    {
        files.push((
            format!("Meetings/m{i}.md"),
            format!("---\nstatus: done\nwhen: {day}T10:00\nattendees:\n  - \"[[p]]\"\n---\n"),
        ));
    }
    for i in 0..1000 {
        files.push((
            format!("Other/o{i}.md"),
            format!("---\nstatus: done\nwhen: 2026-12-01T10:00\nn: {i}\n---\n"),
        ));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, src) = ledger("speed", &refs);
    let read: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let prop = |r: &RowId, c: &str| {
        read.borrow_mut().push(src.label(r));
        src.get(r, c).value
    };
    let v = eval_with_prop(
        &src,
        &prop,
        "People/p.md",
        "file.linkedFrom(\"attendees\").filter(value.properties.status == \"done\").map(value.properties.when).max()",
    );
    assert!(matches!(v, Val::DateTime(_)), "{v:?}");
    let mut seen = read.borrow().clone();
    seen.sort();
    seen.dedup();
    assert_eq!(
        seen,
        vec!["Meetings/m0.md", "Meetings/m1.md", "Meetings/m2.md"],
        "読んだノート"
    );
}

#[test]
fn test_expr_step_budget_nested_lambdas() {
    // [utest -> req~base-view.expr-step-budget~1] nums が 1〜1000 の行で、x = nums.filter(nums.map(value * 2).length > 0).length は
    // 評価できない式(理由は手数の上限)になり、y = nums.length は 1000。
    let nums: String = (1..=1000).map(|i| format!("  - {i}\n")).collect();
    let note = format!("---\nnums:\n{nums}---\n");
    let (_t, src) = ledger("budget", &[("n.md", note.as_str())]);
    let b = parse(
        "formulas:\n  x: 'nums.filter(nums.map(value * 2).length > 0).length'\n  y: 'nums.length'\nviews:\n  - type: table\n    name: v\n    order: [formula.x, formula.y]\n",
    )
    .unwrap();
    let prop = |r: &RowId, c: &str| src.get(r, c).value;
    let row = row_of(&src, "n.md");
    match b.cell(&src, &prop, &row, "formula.x", 0, 0) {
        Shown::Unsupported(reason) => assert!(
            reason.contains(crate::i18n::Msg::ExprBudget.text()),
            "{reason}"
        ),
        other => panic!("x が評価できない式にならない: {other:?}"),
    }
    match b.cell(&src, &prop, &row, "formula.y", 0, 0) {
        Shown::Computed(v) => assert_eq!(v, n(1000.0)),
        other => panic!("y が値でない: {other:?}"),
    }
}

#[test]
fn test_expr_step_budget_large_rollup_fits() {
    // [utest -> req~base-view.expr-step-budget~1] p を attendees に持つ会議が 3000 ある台帳で、ロールアップの式は上限に届かず、
    // 一番遅い会議の日時(2026-10-28T10:00)が出る
    let mut files: Vec<(String, String)> =
        vec![("People/p.md".into(), "---\nrole: x\n---\n".into())];
    for i in 0..3000 {
        let day = 1 + i % 28;
        files.push((
            format!("Meetings/m{i}.md"),
            format!(
                "---\nstatus: done\nwhen: 2026-10-{day:02}T10:00\nattendees:\n  - \"[[p]]\"\n---\n"
            ),
        ));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, src) = ledger("budget_ok", &refs);
    let v = eval_at(
        &src,
        "People/p.md",
        "file.linkedFrom(\"attendees\").filter(value.inFolder(\"Meetings\") && value.properties.status == \"done\").map(value.properties.when).max()",
    );
    assert_eq!(v, Val::DateTime(20_754 * 86_400 + 10 * 3600));
}
