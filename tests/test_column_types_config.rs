//! 列の型の設定 `[types]`(CE-35)の読み込みと範囲の重ね方の受け入れ試験。
//! 記録: specs/_changes/2026-10-13-column-types.md(タスク #1)。

use mdgrid::config::parse;

#[test]
fn test_ce_35_types_read_without_warnings() {
    // [CE-35] 6つの型と "auto" を読める(警告なし)。
    let text = "[types]\na = \"text\"\nb = \"number\"\nc = \"checkbox\"\nd = \"date\"\ne = \"datetime\"\nf = \"list\"\ng = \"auto\"\n";
    let (c, w) = parse(text).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let d = format!("{:?}", c.resolved());
    assert!(d.contains("types"), "決まった値に types がある: {d}");
}

#[test]
fn test_ce_35_types_not_a_table_warns() {
    // [CE-35] `types = "date"`(表でない)→ 警告。
    let (_, w) = parse("types = \"date\"\n").expect("止めない");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("types"), "{w:?}");
}
