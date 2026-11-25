//! グラフの2回目のレビューの指摘(#10〜#15)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md の「レビューの指摘」。
//! 値は作ったもの。前の試験のファイル(test_chart*.rs・test_chart_fix.rs)は変えず、補助は test_chart.rs のものを使う。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする(#12 は数の書き方だけ先に直してあるので、直す前の版でも通りうる)。
//! キーを送り、画面の文字と今ある口(`mode`・`message`)で見る。知らせの文は、指摘が名指す文(ChartNoUnit・
//! ChartNoOtherY・ChartNoNumber)だけを `Msg` で見る。

use super::test_chart::{assert_chart, boot, chart, md, open, reason, screen, Chart, SEPS};
use super::test_screen::ch;
use super::width::width;
use mdgrid::i18n::Msg;

/// 日付と数の列の型。
const TYPES: &str =
    "[types]\nstart = \"date\"\ndue = \"date\"\nestimate = \"number\"\nn = \"number\"\nm = \"number\"\n";

/// 格子の点の字 `c` の (格子の行の番号, 軸の線から右の桁) と、格子の (行の数, 幅)。
fn point_cells(c: &Chart, p: char) -> (Vec<(usize, usize)>, (usize, usize)) {
    let plot = c.plot();
    let mut cells = Vec::new();
    let mut pw = 0;
    for (k, r) in plot.iter().enumerate() {
        let i = r.text.find(SEPS).unwrap();
        let sep = r.text[i..].chars().next().unwrap();
        let start = i + sep.len_utf8();
        let after = &r.text[start..];
        pw = pw.max(width(after));
        for (j, x) in after.char_indices() {
            if x == p {
                cells.push((k, width(&after[..j])));
            }
        }
    }
    (cells, (plot.len(), pw))
}

// ---- #10: 散布図で y だけを書いたときの x の推し量り ----

#[test]
fn test_ch_6_scatter_with_only_y_infers_x_from_other_column() {
    // [utest -> req~chart.config-names~1, req~chart.scatter-points~1, req~chart.scatter-select~1] #10: estimate(数)と due(日付)の表で chart = { kind = "scatter", y = "due" } → x は書いた y と別の
    // 列(estimate)から推し量り、点を打つ。題は「散布図 · estimate × due」で推定。
    let notes = md(&[
        ("alpha", "estimate: 1\ndue: 2026-10-03"),
        ("beta", "estimate: 2\ndue: 2026-10-10"),
        ("gamma", "estimate: 3\ndue: 2026-10-01"),
    ]);
    let (_t, mut a) = boot(
        "chfx10",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", y = \"due\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "散布図", "{}", c.raw);
    assert_eq!(c.x(), "estimate", "x は y と別の列: {}", c.raw);
    assert_eq!(c.value(), "due", "{}", c.raw);
    assert!(c.inferred(), "x を推し量ったので推定: {}", c.raw);
    assert_eq!(c.points('*'), 3, "{}", c.raw);
}

// ---- #11: 積み上げ棒・ヒートマップで by だけを書いたときの x の推し量り ----

/// status と assignee の表: alice は todo と done、bob は todo。
fn by_notes() -> Vec<(String, String)> {
    md(&[
        ("alpha", "status: todo\nassignee: alice"),
        ("beta", "status: done\nassignee: alice"),
        ("gamma", "status: todo\nassignee: bob"),
    ])
}

#[test]
fn test_ch_4_stacked_with_only_by_infers_x_from_other_column() {
    // [utest -> req~chart.config-names~1, req~chart.bar-stacked~1, req~chart.bar-stacked-top5~2] #11: chart = { kind = "stacked", by = "status" } → 分ける列は status でなく assignee、内訳は status。
    // alice の棒は2つの内訳(todo・done)で塗り分ける(1色でない)。
    let (_t, mut a) = boot(
        "chfx11stack",
        &by_notes(),
        "",
        "chart = { kind = \"stacked\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "積み上げ棒", "{}", c.raw);
    assert_eq!(c.x(), "assignee", "x は by と別の列: {}", c.raw);
    assert_eq!(c.by().as_deref(), Some("status"), "{}", c.raw);
    let fills: std::collections::BTreeSet<char> = c.bar("alice").fills.chars().collect();
    assert_eq!(fills.len(), 2, "alice は2色: {fills:?}\n{}", c.raw);
}

#[test]
fn test_ch_7_heatmap_with_only_by_infers_x_from_other_column() {
    // [utest -> req~chart.config-names~1, req~chart.heatmap-cells~1] #11: chart = { kind = "heatmap", by = "status" } → 縦(分ける列)は status でなく assignee、横は status。
    let (_t, mut a) = boot(
        "chfx11heat",
        &by_notes(),
        "",
        "chart = { kind = \"heatmap\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "ヒートマップ", "{}", c.raw);
    assert_eq!(c.x(), "assignee", "x は by と別の列: {}", c.raw);
    assert_eq!(c.by().as_deref(), Some("status"), "{}", c.raw);
}

// ---- #12: とても大きい数の書き方 ----

#[test]
fn test_ch_3_huge_number_bins_have_distinct_names_without_inf() {
    // [utest -> req~chart.buckets-x-skip~1] #12: -1e308・0・1e308 の3行の数の列で分けた棒 → 区間の名前と値に inf が出ず、区間の名前は全部違う。
    let notes = md(&[
        ("alpha", "n: -1e308"),
        ("beta", "n: 0"),
        ("gamma", "n: 1e308"),
    ]);
    let (_t, mut a) = boot("chfx12", &notes, TYPES, "chart = { x = \"n\" }");
    let c = open(&mut a);
    let bars = c.bars();
    assert!(bars.len() >= 2, "{}", c.raw);
    for b in &bars {
        assert!(
            !b.name.to_lowercase().contains("inf") && !b.value.to_lowercase().contains("inf"),
            "inf が出ない: {:?} {:?}\n{}",
            b.name,
            b.value,
            c.raw
        );
    }
    let names: std::collections::BTreeSet<&str> = bars.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(
        names.len(),
        bars.len(),
        "区間の名前が見分けられる: {}",
        c.raw
    );
    assert!(
        !c.desc().to_lowercase().contains("inf"),
        "説明にも inf が出ない: {}",
        c.raw
    );
}

// ---- #13: 値の範囲の幅があふれるときの点の位置 ----

#[test]
fn test_ch_6_scatter_overflowing_range_places_points_at_right_edges() {
    // [utest -> req~chart.line-axes~1, req~chart.line-series-glyph~1, req~chart.scatter-skip~1, req~chart.scatter-select~1] #13: (n, m) が alpha (-1e308, 1e308)・beta (0, 0)・gamma (1e308, -1e308) の散布図 → 3点が別々の
    // ます目に出る(左上の隅に寄らない)。x が最大の点(gamma)は右の半分、y が最小の点(gamma)は下の半分で、
    // 一番右の点が一番下の点。
    let notes = md(&[
        ("alpha", "n: -1e308\nm: 1e308"),
        ("beta", "n: 0\nm: 0"),
        ("gamma", "n: 1e308\nm: -1e308"),
    ]);
    let (_t, mut a) = boot(
        "chfx13scatter",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", x = \"n\", y = \"m\" }",
    );
    let c = open(&mut a);
    let (cells, (rows, pw)) = point_cells(&c, '*');
    assert_eq!(cells.len(), 3, "3点が別々のます目: {cells:?}\n{}", c.raw);
    let right = cells.iter().max_by_key(|(_, x)| *x).unwrap();
    let bottom = cells.iter().max_by_key(|(y, _)| *y).unwrap();
    assert!(
        right.1 * 2 > pw,
        "x が最大の点は右の半分: {right:?} 幅 {pw}\n{}",
        c.raw
    );
    assert!(
        bottom.0 * 2 >= rows,
        "y が最小の点は下の半分: {bottom:?} 行 {rows}\n{}",
        c.raw
    );
    assert_eq!(right, bottom, "一番右の点(gamma)が一番下: {}", c.raw);
}

#[test]
fn test_ch_5_line_overflowing_range_places_points_apart() {
    // [utest -> req~chart.line-points~1] #13: 折れ線で区切りの値が 1e308(2026-10)と -1e308(2026-11)→ 2点が別の高さに出る(上の端に寄らない)。
    // 値の大きい 2026-10 の点が上。
    let notes = md(&[
        ("alpha", "due: 2026-10-03\nn: 1e308"),
        ("beta", "due: 2026-11-03\nn: -1e308"),
    ]);
    let (_t, mut a) = boot(
        "chfx13line",
        &notes,
        TYPES,
        "chart = { kind = \"line\", x = \"due\", y = \"n\", agg = \"sum\", unit = \"month\" }",
    );
    let c = open(&mut a);
    let (cells, (rows, _)) = point_cells(&c, '*');
    assert_eq!(cells.len(), 2, "2点: {cells:?}\n{}", c.raw);
    let left = cells.iter().min_by_key(|(_, x)| *x).unwrap();
    let right = cells.iter().max_by_key(|(_, x)| *x).unwrap();
    assert!(
        left.0 < right.0,
        "2026-10(1e308)の点が 2026-11(-1e308)の点より上: {cells:?}\n{}",
        c.raw
    );
    assert!(
        right.0 * 2 >= rows,
        "-1e308 の点は下の半分: {right:?} 行 {rows}\n{}",
        c.raw
    );
}

// ---- #14: 広すぎる日付の範囲での単位の知らせ ----

#[test]
fn test_ch_8_unit_keys_on_too_wide_dates_do_not_say_not_split_by_date() {
    // [utest -> req~chart.keys-unit~1, req~chart.keys-kind-unfit~1, req~chart.keys-enter~2] #14: 0001-01-01 と 9999-12-31 の日付の列で分けたグラフ(理由を出している)→ `-` と `+` の知らせが
    // 「日付の列で分けていないので、単位は変えない」(ChartNoUnit)にならない。グラフのまま。
    let notes = md(&[
        ("alpha", "status: todo\ndue: 0001-01-01"),
        ("beta", "status: done\ndue: 9999-12-31"),
    ]);
    let (_t, mut a) = boot("chfx14", &notes, TYPES, "chart = { x = \"due\" }");
    open(&mut a);
    for k in ['-', '+'] {
        a.message = None;
        ch(&mut a, k);
        assert_chart(&a);
        let m = a.message.clone().unwrap_or_default();
        assert!(
            !m.contains(Msg::ChartNoUnit.text()),
            "{k}: 日付の列で分けているので ChartNoUnit の文でない: {m:?}\n{}",
            screen(&a)
        );
    }
}

// ---- #15: 散布図の y の知らせ ----

#[test]
fn test_ch_8_y_in_scatter_without_other_vertical_column_says_no_other_y() {
    // [utest -> req~chart.keys-kind-unfit~1] #15: 数か日付の列がちょうど2つ(estimate・due)の散布図で y → ほかに縦にできる列が無いことを言う知らせ
    // (ChartNoOtherY)。「値にできる数の列が無い」(ChartNoNumber)ではない。縦は due のまま。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 1\ndue: 2026-10-03"),
        ("beta", "status: done\nestimate: 2\ndue: 2026-10-10"),
    ]);
    let (_t, mut a) = boot(
        "chfx15",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", x = \"estimate\", y = \"due\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.value(), "due", "前提: 縦は due: {}", c.raw);
    a.message = None;
    ch(&mut a, 'y');
    assert_chart(&a);
    let m = reason(&a);
    assert!(
        !m.contains(Msg::ChartNoNumber.text()),
        "「値にできる数の列が無い」ではない: {m:?}"
    );
    assert!(
        m.contains(Msg::ChartNoOtherY.text()),
        "縦にできる列が無いことを言う({}): {m:?}",
        Msg::ChartNoOtherY.text()
    );
    assert_eq!(chart(&a).value(), "due", "縦は変わらない");
}
