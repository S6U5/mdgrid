//! グラフ(CH-1〜CH-3)の、画面に依存しない部分(`src/chart.rs` と印の読み `workspace::marker_chart`)の試験。
//! 値は作ったもの。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。
//! 日付は 1970-01-01 からの日数(`types::parse_date`)。2026-10-01 は木曜、2026-10-05 は月曜。
//! 区間(`bins`)は、境界の細かい形を決め打ちせず、区間の数・最初の下の端・最後の上の端・各値の入る区間で見る。

use crate::chart::{
    bin_of, bins, bucket_label, fmt_num, infer_x, Acc, Agg, ChartKind, ChartSpec, ColInfo,
};
use crate::gantt::Unit;
use crate::types::Kind;

fn d(s: &str) -> i64 {
    crate::types::parse_date(s).unwrap()
}

/// `infer_x` に渡す列(名前, 型, 決まりの values があるか)。
fn cols(list: &[(&str, Kind, bool)]) -> Vec<ColInfo> {
    list.iter()
        .map(|(n, k, v)| ColInfo {
            name: n.to_string(),
            kind: *k,
            has_values: *v,
        })
        .collect()
}

fn x_of(list: &[(&str, Kind, bool)]) -> Option<String> {
    infer_x(&cols(list))
}

// ---- CH-1: 種類・集め方の名前と、設定の形 ----

#[test]
fn test_ch_1_kind_names_round_trip_in_cycle_order() {
    // [utest -> req~chart.config-names~1, req~chart.config-invalid~1] 種類は bar・stacked・line・scatter・heatmap で、ALL はこの順(Tab で切り替える順。CH-8)。
    // 名前と parse は往復する。知らない名前(pie・空)は None。
    assert_eq!(
        ChartKind::ALL,
        [
            ChartKind::Bar,
            ChartKind::Stacked,
            ChartKind::Line,
            ChartKind::Scatter,
            ChartKind::Heatmap
        ]
    );
    let names: Vec<&str> = ChartKind::ALL.iter().map(|k| k.name()).collect();
    assert_eq!(names, ["bar", "stacked", "line", "scatter", "heatmap"]);
    for k in ChartKind::ALL {
        assert_eq!(ChartKind::parse(k.name()), Some(k), "{k:?}");
    }
    for bad in ["pie", "", "bars", "area"] {
        assert_eq!(ChartKind::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn test_ch_1_agg_names_round_trip() {
    // [utest -> req~chart.config-names~1] 集め方は count・sum・avg・min・max。名前と parse は往復し、知らない名前(median・空)は None。
    assert_eq!(
        Agg::ALL,
        [Agg::Count, Agg::Sum, Agg::Avg, Agg::Min, Agg::Max]
    );
    let names: Vec<&str> = Agg::ALL.iter().map(|a| a.name()).collect();
    assert_eq!(names, ["count", "sum", "avg", "min", "max"]);
    for a in Agg::ALL {
        assert_eq!(Agg::parse(a.name()), Some(a), "{a:?}");
    }
    for bad in ["median", "", "average", "total"] {
        assert_eq!(Agg::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn test_ch_1_spec_default_is_all_omitted() {
    // [utest -> req~chart.config-one~1, req~chart.config-missing-column~1] 鍵はどれも省けるので、既定の設定は全部が無い(cumulative は false)。
    let s = ChartSpec::default();
    assert_eq!(s.name, None);
    assert_eq!(s.kind, None);
    assert_eq!(s.x, None);
    assert_eq!(s.y, None);
    assert_eq!(s.agg, None);
    assert_eq!(s.by, None);
    assert_eq!(s.unit, None);
    assert!(!s.cumulative);
}

// ---- CH-1: 印の読み ----

/// 印のある一時フォルダ(根)と、その下の表のフォルダ `notes`。
struct Root(std::path::PathBuf);

impl Root {
    fn new(tag: &str, marker: &str) -> Root {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-chart-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(dir.join("notes")).unwrap();
        std::fs::create_dir_all(dir.join(".mdgrid")).unwrap();
        std::fs::write(dir.join(".mdgrid/workspace.toml"), marker).unwrap();
        Root(dir)
    }

    fn notes(&self) -> std::path::PathBuf {
        self.0.join("notes")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const MARKER_HEAD: &str = "[[table]]\nname = \"T\"\npath = \"notes\"\n";

/// 表 notes の [[table]] に `line` を書いた印を読む。
fn read_chart(tag: &str, line: &str) -> (Vec<(ChartSpec, String)>, Vec<String>) {
    let r = Root::new(tag, &format!("{MARKER_HEAD}{line}\n"));
    crate::workspace::marker_chart(&r.notes())
}

fn spec_x(kind: Option<ChartKind>, x: &str) -> ChartSpec {
    ChartSpec {
        kind,
        x: Some(x.to_string()),
        ..ChartSpec::default()
    }
}

#[test]
fn test_ch_1_marker_reads_one_chart() {
    // [utest -> req~chart.config-invalid~1, req~chart.config-missing-column~1] chart = { kind = "bar", x = "status" } → グラフ1つ、場所は table[1].chart、警告なし。
    // 範囲の読み(read_marker)でも chart は知らない項目にならない。
    let r = Root::new(
        "one",
        &format!("{MARKER_HEAD}chart = {{ kind = \"bar\", x = \"status\" }}\n"),
    );
    let (got, warns) = crate::workspace::marker_chart(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(
        got,
        vec![(
            spec_x(Some(ChartKind::Bar), "status"),
            "table[1].chart".to_string()
        )]
    );
    let (_, warns) = crate::workspace::read_marker(&r.0);
    assert!(
        warns.is_empty(),
        "chart は [[table]] の知っている鍵: {warns:?}"
    );
    // 別の表(chart の無い表)にはグラフが無い。
    std::fs::create_dir_all(r.0.join("other")).unwrap();
    let (got, warns) = crate::workspace::marker_chart(&r.0.join("other"));
    assert!(got.is_empty(), "{got:?}");
    assert!(warns.is_empty(), "{warns:?}");
}

#[test]
fn test_ch_1_marker_reads_all_keys() {
    // [utest -> req~chart.config-one~1, req~chart.config-array~1, req~chart.config-names~1, req~chart.config-defaults~1] 鍵を全部書いた形: kind・x・y・agg・by・unit・cumulative・name を読む。
    let (got, warns) = read_chart(
        "all",
        "chart = { name = \"月ごとの見積もり\", kind = \"line\", x = \"due\", y = \"estimate\", agg = \"sum\", by = \"status\", unit = \"month\", cumulative = true }",
    );
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got.len(), 1, "{got:?}");
    let (s, at) = &got[0];
    assert_eq!(at, "table[1].chart");
    assert_eq!(
        *s,
        ChartSpec {
            name: Some("月ごとの見積もり".into()),
            kind: Some(ChartKind::Line),
            x: Some("due".into()),
            y: Some("estimate".into()),
            agg: Some(Agg::Sum),
            by: Some("status".into()),
            unit: Some(Unit::Month),
            cumulative: true,
        }
    );
}

#[test]
fn test_ch_1_marker_reads_each_kind_agg_and_unit() {
    // [utest -> req~chart.config-names~1] 種類(5つ)・集め方(5つ)・単位(day・week・month)のどの名前も読める。
    for k in ChartKind::ALL {
        let (got, warns) = read_chart("kind", &format!("chart = {{ kind = \"{}\" }}", k.name()));
        assert!(warns.is_empty(), "{k:?}: {warns:?}");
        assert_eq!(got.len(), 1, "{k:?}");
        assert_eq!(got[0].0.kind, Some(k));
    }
    for a in Agg::ALL {
        let (got, warns) = read_chart(
            "agg",
            &format!("chart = {{ y = \"estimate\", agg = \"{}\" }}", a.name()),
        );
        assert!(warns.is_empty(), "{a:?}: {warns:?}");
        assert_eq!(got[0].0.agg, Some(a));
    }
    for (name, unit) in [
        ("day", Unit::Day),
        ("week", Unit::Week),
        ("month", Unit::Month),
    ] {
        let (got, warns) = read_chart("unit", &format!("chart = {{ unit = \"{name}\" }}"));
        assert!(warns.is_empty(), "{name}: {warns:?}");
        assert_eq!(got[0].0.unit, Some(unit));
    }
}

#[test]
fn test_ch_1_marker_empty_table_is_all_inferred() {
    // [utest -> req~chart.config-defaults~1] 境界: chart = {} は書き方の違いでなく、鍵を全部省いたグラフ(CH-2 の推定で埋める)。警告なし。
    let (got, warns) = read_chart("emptyspec", "chart = {}");
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(
        got,
        vec![(ChartSpec::default(), "table[1].chart".to_string())]
    );
}

#[test]
fn test_ch_1_marker_reads_array_in_order() {
    // [utest -> req~chart.config-one~1, req~chart.config-array~1] chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }] → 2つを書いた順に。
    // 場所は table[1].chart[1]・table[1].chart[2]。
    let (got, warns) = read_chart(
        "array",
        "chart = [{ name = \"状態\", x = \"status\" }, { name = \"月ごと\", kind = \"line\", x = \"due\" }]",
    );
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got.len(), 2, "{got:?}");
    assert_eq!(got[0].0.name.as_deref(), Some("状態"));
    assert_eq!(got[0].0.x.as_deref(), Some("status"));
    assert_eq!(got[0].0.kind, None);
    assert_eq!(got[0].1, "table[1].chart[1]");
    assert_eq!(got[1].0.name.as_deref(), Some("月ごと"));
    assert_eq!(got[1].0.kind, Some(ChartKind::Line));
    assert_eq!(got[1].0.x.as_deref(), Some("due"));
    assert_eq!(got[1].1, "table[1].chart[2]");
}

/// 警告が CLI-3 の形(`.mdgrid/workspace.toml: <場所>: <理由>`)の1行で、場所 `at` を含む。
fn assert_warn(warns: &[String], at: &str, what: &str) {
    assert_eq!(warns.len(), 1, "{what}: 警告は1つ: {warns:?}");
    let w = &warns[0];
    assert!(
        w.starts_with(".mdgrid/workspace.toml: ") && w.contains(at),
        "{what}: 場所 {at} の警告: {w:?}"
    );
    assert!(!w.contains('\n'), "{what}: 1行: {w:?}");
}

#[test]
fn test_ch_1_marker_bad_values_warn_and_drop_that_chart() {
    // [utest -> req~chart.config-invalid~1] 書き方の違い(知らない種類・集め方・単位、文字でない列の名前、真偽でない cumulative、知らない鍵)→
    // CLI-3 の形の警告1行(場所は table[1].chart.<鍵>)で、そのグラフを外す。
    for (line, at) in [
        (
            "chart = { kind = \"pie\", x = \"status\" }",
            "table[1].chart.kind",
        ),
        (
            "chart = { y = \"estimate\", agg = \"median\" }",
            "table[1].chart.agg",
        ),
        (
            "chart = { x = \"due\", unit = \"year\" }",
            "table[1].chart.unit",
        ),
        ("chart = { x = 3 }", "table[1].chart.x"),
        ("chart = { x = \"status\", y = true }", "table[1].chart.y"),
        (
            "chart = { x = \"status\", by = [\"a\"] }",
            "table[1].chart.by",
        ),
        (
            "chart = { x = \"due\", cumulative = \"yes\" }",
            "table[1].chart.cumulative",
        ),
        ("chart = { x = \"status\", foo = 1 }", "table[1].chart.foo"),
    ] {
        let (got, warns) = read_chart("bad", line);
        assert!(got.is_empty(), "{line}: そのグラフを外す: {got:?}");
        assert_warn(&warns, at, line);
    }
}

#[test]
fn test_ch_1_marker_not_a_table_warns() {
    // [utest -> req~chart.config-invalid~1, req~chart.config-missing-column~1] chart が表でも表の配列でもない(文字・数)→ 警告1行(場所 table[1].chart)で、グラフは無い。
    for line in ["chart = \"bar\"", "chart = 3"] {
        let (got, warns) = read_chart("nottable", line);
        assert!(got.is_empty(), "{line}: {got:?}");
        assert_warn(&warns, "table[1].chart", line);
    }
}

#[test]
fn test_ch_1_marker_array_drops_only_bad_one() {
    // [utest -> req~chart.config-array~1, req~chart.config-invalid~1] 配列の中の1つだけが違う → そのグラフだけを外し(場所 table[1].chart[1] など)、ほかは残す。
    // 配列の要素が表でない → 場所 table[1].chart[1] で警告して、残りを読む。
    let (got, warns) = read_chart(
        "arraybad",
        "chart = [{ name = \"円\", kind = \"pie\" }, { name = \"状態\", x = \"status\" }]",
    );
    assert_warn(&warns, "table[1].chart[1].kind", "配列の1つ目の kind");
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].0.name.as_deref(), Some("状態"));
    assert_eq!(
        got[0].1, "table[1].chart[2]",
        "残ったグラフの場所は書いた位置のまま"
    );
    let (got, warns) = read_chart("arraynottable", "chart = [1, { x = \"status\" }]");
    assert_warn(&warns, "table[1].chart[1]", "表でない要素");
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].0.x.as_deref(), Some("status"));
    assert_eq!(got[0].1, "table[1].chart[2]");
}

#[test]
fn test_ch_1_marker_second_table_location() {
    // [utest -> req~chart.config-one~1, req~chart.config-array~1, req~chart.config-invalid~1, req~chart.config-missing-column~1] 2つ目の表のグラフの場所は table[2].chart。1つ目の表のグラフは2つ目の表に漏れない。
    let r = Root::new(
        "second",
        &format!(
            "[[table]]\nname = \"A\"\npath = \"other\"\nchart = {{ x = \"a\" }}\n\n{MARKER_HEAD}chart = {{ x = \"status\" }}\n"
        ),
    );
    std::fs::create_dir_all(r.0.join("other")).unwrap();
    let (got, warns) = crate::workspace::marker_chart(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].0.x.as_deref(), Some("status"));
    assert_eq!(got[0].1, "table[2].chart");
}

#[test]
fn test_ch_1_marker_does_not_check_columns() {
    // [utest -> req~chart.config-missing-column~1] 印の読みは列が表にあるかを決めない(列が出そろってから画面の側で警告する)。x = "none" も読む。
    let (got, warns) = read_chart("nonecol", "chart = { x = \"none\" }");
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].0.x.as_deref(), Some("none"));
}

// ---- CH-2: 分ける列の推定 ----

#[test]
fn test_ch_2_infer_status_name_first() {
    // [utest -> req~chart.open-infer~1] 状態の列の名前(KB-1。大文字小文字を問わない)が最優先: 決まりのある列や日付の列より先。
    assert_eq!(
        x_of(&[
            ("title", Kind::Text, false),
            ("priority", Kind::Text, true),
            ("due", Kind::Date, false),
            ("status", Kind::Text, false),
        ]),
        Some("status".into())
    );
    for name in ["Status", "state", "stage", "状態", "ステータス", "段階"] {
        assert_eq!(
            x_of(&[
                ("title", Kind::Text, false),
                ("done", Kind::Checkbox, false),
                (name, Kind::Text, false),
            ]),
            Some(name.into()),
            "{name}"
        );
    }
}

#[test]
fn test_ch_2_infer_skips_list_status_then_rule_values() {
    // [utest -> req~chart.open-infer~1] リストの型の状態の列は選ばない。次は決まり(CE-36 の values)のある列。
    assert_eq!(
        x_of(&[
            ("status", Kind::List, false),
            ("title", Kind::Text, false),
            ("phase", Kind::Text, true),
        ]),
        Some("phase".into())
    );
    // 決まりのある列はチェックボックスの列より先。
    assert_eq!(
        x_of(&[("done", Kind::Checkbox, false), ("phase", Kind::Text, true),]),
        Some("phase".into())
    );
}

#[test]
fn test_ch_2_infer_checkbox_then_list_then_date_then_first() {
    // [utest -> req~chart.open-infer~1] チェックボックス → リスト → 日付(日付・日時)→ 最初の列の順。
    assert_eq!(
        x_of(&[
            ("tags", Kind::List, false),
            ("due", Kind::Date, false),
            ("done", Kind::Checkbox, false),
        ]),
        Some("done".into())
    );
    assert_eq!(
        x_of(&[
            ("title", Kind::Text, false),
            ("due", Kind::Date, false),
            ("tags", Kind::List, false),
        ]),
        Some("tags".into())
    );
    assert_eq!(
        x_of(&[
            ("title", Kind::Text, false),
            ("n", Kind::Number, false),
            ("due", Kind::Date, false),
        ]),
        Some("due".into())
    );
    assert_eq!(
        x_of(&[("title", Kind::Text, false), ("at", Kind::DateTime, false)]),
        Some("at".into())
    );
    assert_eq!(
        x_of(&[("title", Kind::Text, false), ("n", Kind::Number, false)]),
        Some("title".into())
    );
}

#[test]
fn test_ch_2_infer_no_columns_is_none() {
    // [utest -> req~chart.open-refuse~1] 境界: 列が1つも無い → None(画面は理由1行で開かない)。
    assert_eq!(infer_x(&[]), None);
}

// ---- CH-3: 数え方・集め方 ----

#[test]
fn test_ch_3_acc_count_sum_avg_min_max() {
    // [utest -> req~chart.buckets-agg~1] 件数は行の数(数の無い行も数える)。合計・平均・最小・最大は数のある行だけ。
    let mut a = Acc::default();
    a.add(Some(3.0));
    a.add(None);
    a.add(Some(5.0));
    assert_eq!(a.value(Agg::Count), Some(3.0));
    assert_eq!(a.value(Agg::Sum), Some(8.0));
    assert_eq!(a.value(Agg::Avg), Some(4.0), "平均は数のある2行で割る");
    assert_eq!(a.value(Agg::Min), Some(3.0));
    assert_eq!(a.value(Agg::Max), Some(5.0));
}

#[test]
fn test_ch_3_acc_negative_values() {
    // [utest -> req~chart.buckets-agg~1] 負の値も数える: -2 と 1 → 合計 -1、平均 -0.5、最小 -2、最大 1。
    let mut a = Acc::default();
    a.add(Some(-2.0));
    a.add(Some(1.0));
    assert_eq!(a.value(Agg::Sum), Some(-1.0));
    assert_eq!(a.value(Agg::Avg), Some(-0.5));
    assert_eq!(a.value(Agg::Min), Some(-2.0));
    assert_eq!(a.value(Agg::Max), Some(1.0));
}

#[test]
fn test_ch_3_acc_without_numbers_has_no_value() {
    // [utest -> req~chart.buckets-agg~1] 境界: 数のある行が1つも無い項目は、合計・平均・最小・最大が値なし(None)。件数は行の数。
    let mut a = Acc::default();
    a.add(None);
    a.add(None);
    assert_eq!(a.value(Agg::Count), Some(2.0));
    for agg in [Agg::Sum, Agg::Avg, Agg::Min, Agg::Max] {
        assert_eq!(a.value(agg), None, "{agg:?}");
    }
    // 何も足していない項目の件数は 0。
    assert_eq!(Acc::default().value(Agg::Count), Some(0.0));
    assert_eq!(Acc::default().value(Agg::Sum), None);
}

#[test]
fn test_ch_3_bins_integers_by_one() {
    // [utest -> req~chart.buckets-date~1, req~chart.buckets-number~1] 整数だけで幅(max - min + 1)が 10 以下 → 1 ずつの区間: 1..=5 は 5 区間で、各値は自分の区間に入る。
    let b = bins(1.0, 5.0, true);
    assert_eq!(b.len(), 5, "{b:?}");
    assert_eq!(b[0].0, 1.0, "{b:?}");
    for (i, (lo, hi)) in b.iter().enumerate() {
        assert_eq!(hi - lo, 1.0, "区間 {i} の幅は 1: {b:?}");
    }
    for (v, want) in [(1.0, 0), (2.0, 1), (3.0, 2), (4.0, 3), (5.0, 4)] {
        assert_eq!(bin_of(&b, v), Some(want), "{v}: {b:?}");
    }
    // 幅 10(1..=10)も1ずつ。幅 11(0..=10)は1ずつにしない(10 区間まで)。
    assert_eq!(bins(1.0, 10.0, true).len(), 10);
    assert!(bins(0.0, 10.0, true).len() <= 10);
}

#[test]
fn test_ch_3_bins_ten_equal_widths() {
    // [utest -> req~chart.buckets-number~1] 小数のある値 → 等しい幅の 10 区間。最初の下の端は最小、最後の上の端は最大で、最大の値は最後の区間に入る。
    let b = bins(0.0, 10.0, false);
    assert_eq!(b.len(), 10, "{b:?}");
    assert_eq!(b[0].0, 0.0);
    assert_eq!(b[9].1, 10.0);
    let w = b[0].1 - b[0].0;
    for (lo, hi) in &b {
        assert!(((hi - lo) - w).abs() < 1e-9, "等しい幅: {b:?}");
    }
    assert_eq!(bin_of(&b, 0.0), Some(0));
    assert_eq!(bin_of(&b, 0.5), Some(0));
    assert_eq!(bin_of(&b, 9.5), Some(9));
    assert_eq!(bin_of(&b, 10.0), Some(9), "最後の区間は上の端を含む");
    // 範囲の外は入らない。
    assert_eq!(bin_of(&b, -0.1), None);
    assert_eq!(bin_of(&b, 10.1), None);
    // 整数でも幅が広ければ 10 区間(0..=100)。
    let b = bins(0.0, 100.0, true);
    assert_eq!(b.len(), 10, "{b:?}");
    assert_eq!(b[0].0, 0.0);
    assert_eq!(b[9].1, 100.0);
    assert_eq!(bin_of(&b, 100.0), Some(9));
    assert_eq!(bin_of(&b, 0.0), Some(0));
}

#[test]
fn test_ch_3_bins_single_value() {
    // [utest -> req~chart.buckets-number~1, req~chart.buckets-agg~1] 境界: 最小と最大が同じ → 区間は1つで、その値が入る。
    for all_int in [true, false] {
        let b = bins(3.0, 3.0, all_int);
        assert_eq!(b.len(), 1, "{all_int}: {b:?}");
        assert_eq!(bin_of(&b, 3.0), Some(0), "{all_int}: {b:?}");
    }
}

#[test]
fn test_ch_3_fmt_num() {
    // [utest -> req~chart.buckets-date~1, req~chart.buckets-number~1, req~chart.buckets-order~1] 数の書き方: 整数ならそのまま、ほかは小数2桁までで末尾の 0 を除く。
    assert_eq!(fmt_num(3.0), "3");
    assert_eq!(fmt_num(2.5), "2.5");
    assert_eq!(fmt_num(10.0 / 3.0), "3.33");
    assert_eq!(fmt_num(-1.0), "-1");
    assert_eq!(fmt_num(0.0), "0");
    assert_eq!(fmt_num(100.0), "100", "整数の 0 は除かない");
    assert_eq!(fmt_num(20.0), "20");
    assert_eq!(fmt_num(1.10), "1.1");
    assert_eq!(fmt_num(2.0 / 3.0), "0.67");
    assert_eq!(fmt_num(0.1 + 0.2), "0.3");
    assert_eq!(fmt_num(-0.5), "-0.5");
}

#[test]
fn test_ch_3_bucket_label_day_week_month() {
    // [utest -> req~chart.buckets-date~1] 日付の区切りの名前: 日は日付、週は週の始まりの日(月曜)、月は年と月。
    assert_eq!(bucket_label(d("2026-10-01"), Unit::Day), "2026-10-01");
    assert_eq!(bucket_label(d("2026-10-05"), Unit::Week), "2026-10-05");
    assert_eq!(bucket_label(d("2026-09-28"), Unit::Week), "2026-09-28");
    assert_eq!(bucket_label(d("2026-10-01"), Unit::Month), "2026-10");
    assert_eq!(bucket_label(d("2026-12-01"), Unit::Month), "2026-12");
    assert_eq!(bucket_label(d("2027-01-01"), Unit::Month), "2027-01");
}
