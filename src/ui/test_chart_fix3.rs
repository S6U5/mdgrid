//! グラフの3回目のレビューの指摘(#16・#18・#19)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md の「レビューの指摘」。
//! 値は作ったもの。前の試験のファイルは変えず、補助は test_chart.rs のものを使う。細部3件の小さい直しなので、
//! テスト役でなくメインが書いた(記録の「実装の気づき」)。直しを一時的に打ち消した版で落ちることを確かめた。

use super::test_chart::{boot, open};

/// 数と日付の列の型。
const TYPES: &str =
    "[types]\nscore = \"number\"\nestimate = \"number\"\nactual = \"number\"\ndue = \"date\"\n";

#[test]
fn test_ch_3_narrow_number_bins_have_distinct_names() {
    // [utest -> req~chart.buckets-list-items~1, req~chart.buckets-date~1, req~chart.buckets-number~1] #16: 数の列が 0.12・0.13・0.15 の3行 → 区間の幅は 0.003 で、区間の名前(`下~上`)がどれも違う
    // (小数2桁に丸めて「0.13~0.13」が2つ並ばない)。3行はどれかの区間に入る。
    let notes = super::test_chart::md(&[
        ("alpha", "score: 0.12"),
        ("beta", "score: 0.13"),
        ("gamma", "score: 0.15"),
    ]);
    let (_t, mut a) = boot("chfx16", &notes, TYPES, "chart = { x = \"score\" }");
    let c = open(&mut a);
    let items = c.items();
    assert_eq!(items.len(), 10, "区間は 10: {}", c.raw);
    let mut names: Vec<&str> = items.iter().map(|(n, _)| n.as_str()).collect();
    names.sort();
    names.dedup();
    assert_eq!(
        names.len(),
        10,
        "区間の名前が重ならない: {items:?}\n{}",
        c.raw
    );
    let total: f64 = items
        .iter()
        .filter_map(|(_, v)| v.parse::<f64>().ok())
        .sum();
    assert_eq!(total, 3.0, "{items:?}");
}

#[test]
fn test_ch_6_scatter_with_count_agg_keeps_written_y() {
    // [utest -> req~chart.config-names~1, req~chart.scatter-points~1, req~chart.scatter-skip~1, req~chart.scatter-select~1, req~chart.scatter-enter~1] #18: 散布図は集め方を使わない。`agg = "count"` を書いても、書いた y(due)を捨てて推し量らない。
    let notes = super::test_chart::md(&[
        ("alpha", "estimate: 1\nactual: 2\ndue: 2026-10-03"),
        ("beta", "estimate: 2\nactual: 3\ndue: 2026-10-10"),
    ]);
    let (_t, mut a) = boot(
        "chfx18",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", x = \"estimate\", y = \"due\", agg = \"count\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "散布図", "{}", c.raw);
    assert_eq!(c.x(), "estimate", "{}", c.raw);
    assert_eq!(c.value(), "due", "書いた y のまま: {}", c.raw);
    assert!(!c.inferred(), "推し量っていない: {}", c.raw);
    assert_eq!(c.points('*'), 2, "{}", c.raw);
}

#[test]
fn test_ch_8_stacked_with_by_same_as_only_column_gives_reason() {
    // [utest -> req~chart.bar-scale~1, req~chart.keys-kind-unfit~1] #19: 列が status だけの表で chart = { kind = "stacked", by = "status" } → 分ける列も status に
    // なるので、1色の棒を描かずに体に理由(内訳にできる列が無い)を出す。Tab で積み上げ棒へ移る道と同じ。
    let notes = super::test_chart::md(&[("alpha", "status: todo"), ("beta", "status: done")]);
    let (_t, mut a) = boot(
        "chfx19",
        &notes,
        "",
        "chart = { kind = \"stacked\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert!(!c.raw.contains('█'), "棒は描かない: {}", c.raw);
    let need_by = mdgrid::i18n::Msg::ChartNeedBy.text();
    assert!(
        c.raw.contains(need_by),
        "理由に「{need_by}」がある: {}",
        c.raw
    );
}
