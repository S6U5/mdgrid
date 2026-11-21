//! グラフの画面の折れ線(CH-5)・散布図(CH-6)・ヒートマップ(CH-7)・キーとマウスと外の変化(CH-8)の受け入れの試験。
//! 値は作ったもの。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。補助は test_chart.rs。
//! 散布図・折れ線の選びの初めは最初の点・最初の区切り(小さい順)、棒の選びの初めは最初の項目。

use super::keymap::Mode;
use super::test_chart::{
    assert_chart, assert_table, boot, chart, cur, md, open, reason, row_names, screen,
    status_notes, top_rows, want, Chart, KINDS, POINTS,
};
use super::test_screen::{ch, edit_outside, press, read, Tmp};
use ratatui::crossterm::event::KeyCode;

/// 日付と数の列の型。
const TYPES: &str = "[types]\ndue = \"date\"\nestimate = \"number\"\nactual = \"number\"\n";

/// 印の workspace.toml の中身。
fn marker(t: &Tmp) -> String {
    std::fs::read_to_string(t.0.join(".mdgrid/workspace.toml")).unwrap()
}

// ---- CH-5: 折れ線 ----

/// 折れ線の例: due が 2026-10 に3件(alpha・beta・gamma)、2026-11 に1件(delta)。
fn month_notes() -> Vec<(String, String)> {
    md(&[
        ("alpha", "status: todo\ndue: 2026-10-03"),
        ("beta", "status: todo\ndue: 2026-10-15"),
        ("gamma", "status: done\ndue: 2026-10-28"),
        ("delta", "status: done\ndue: 2026-11-02"),
    ])
}

const LINE_MONTH: &str = "chart = { kind = \"line\", x = \"due\", unit = \"month\" }";

#[test]
fn test_ch_5_line_points_labels_and_selection() {
    // [utest -> req~chart.line-points~1, req~chart.line-axes~1] due で 2026-10 に3件・2026-11 に1件、単位が月 → 2点(`*`)を点字の線で結ぶ。下に最初と最後の区切りの
    // 名前。選びの初めは最初の区切り「2026-10: 3」。l で「2026-11: 1」、選んでいる区切りの下の `^` が右へ動く。
    // 最後の区切りで l、最初の区切りで h を押しても、端で止まる。
    let (_t, mut a) = boot("ch5line", &month_notes(), TYPES, LINE_MONTH);
    let c = open(&mut a);
    assert_eq!(c.kind(), "折れ線", "{}", c.raw);
    assert_eq!(c.x(), "due", "{}", c.raw);
    assert_eq!(c.points('*'), 2, "区切りごとの点: {}", c.raw);
    assert!(c.has_braille(), "点を点字の線で結ぶ: {}", c.raw);
    assert_eq!(c.desc(), "2026-10: 3", "{}", c.raw);
    let below: Vec<String> = c.below().iter().map(|r| r.text.clone()).collect();
    assert!(
        below.iter().any(|l| l.contains("2026-10")) && below.iter().any(|l| l.contains("2026-11")),
        "下に最初と最後の区切りの名前: {}",
        c.raw
    );
    let caret0 = c.caret().unwrap_or_else(|| panic!("`^` が無い: {}", c.raw));
    let pts = c.point_cols('*');
    assert!(pts[0] < pts[1], "点は左から: {}", c.raw);
    ch(&mut a, 'l');
    let c = chart(&a);
    assert_eq!(c.desc(), "2026-11: 1", "{}", c.raw);
    let caret1 = c.caret().unwrap_or_else(|| panic!("`^` が無い: {}", c.raw));
    assert!(
        caret1 > caret0,
        "`^` が右へ: {caret0} → {caret1}\n{}",
        c.raw
    );
    ch(&mut a, 'l');
    assert_eq!(chart(&a).desc(), "2026-11: 1", "右の端で止まる");
    ch(&mut a, 'h');
    ch(&mut a, 'h');
    assert_eq!(chart(&a).desc(), "2026-10: 3", "左の端で止まる");
}

#[test]
fn test_ch_5_scale_shows_max_and_min() {
    // [utest -> req~chart.line-axes~1] 左に値の目盛り: 最大は 3、最小は 1 以下(0 から始めてもよい)。
    let (_t, mut a) = boot("ch5scale", &month_notes(), TYPES, LINE_MONTH);
    let c = open(&mut a);
    let s = c.scale();
    let max = s.iter().copied().fold(f64::MIN, f64::max);
    let min = s.iter().copied().fold(f64::MAX, f64::min);
    assert_eq!(max, 3.0, "目盛りの最大: {s:?}\n{}", c.raw);
    assert!(min <= 1.0, "目盛りの最小: {s:?}\n{}", c.raw);
}

#[test]
fn test_ch_5_c_toggles_cumulative() {
    // [utest -> req~chart.line-axes~1, req~chart.line-cumulative~1] c で累積(その区切りまでの合計)→ 2026-10 は 3、2026-11 は 4。目盛りの最大も 4。もう一度 c で戻る。
    let (_t, mut a) = boot("ch5cum", &month_notes(), TYPES, LINE_MONTH);
    open(&mut a);
    ch(&mut a, 'c');
    let c = chart(&a);
    assert_eq!(c.desc(), "2026-10: 3", "{}", c.raw);
    ch(&mut a, 'l');
    let c = chart(&a);
    assert_eq!(c.desc(), "2026-11: 4", "{}", c.raw);
    let max = c.scale().iter().copied().fold(f64::MIN, f64::max);
    assert_eq!(max, 4.0, "累積の目盛りの最大: {}", c.raw);
    ch(&mut a, 'c');
    assert_eq!(chart(&a).desc(), "2026-11: 1", "累積を外す");
}

#[test]
fn test_ch_5_cumulative_setting_from_marker() {
    // [utest -> req~chart.line-cumulative~1, req~chart.config-one~1, req~chart.config-array~1, req~chart.config-names~1] 印の cumulative = true で開くと、初めから累積。
    let (_t, mut a) = boot(
        "ch5cummarker",
        &month_notes(),
        TYPES,
        "chart = { kind = \"line\", x = \"due\", unit = \"month\", cumulative = true }",
    );
    open(&mut a);
    ch(&mut a, 'l');
    assert_eq!(chart(&a).desc(), "2026-11: 4");
}

#[test]
fn test_ch_5_by_draws_one_line_per_value_with_distinct_points() {
    // [utest -> req~chart.line-series-glyph~1, req~cell-view.not-color-only~1] by = status → 内訳ごとに線(3本)。点の字(* + o)と凡例で見分けられる。説明は区切りと内訳ごとの値。
    let mut list: Vec<(String, String)> = Vec::new();
    for (month, counts) in [
        ("2026-10-05", [("todo", 3), ("doing", 2), ("done", 1)]),
        ("2026-11-05", [("todo", 1), ("doing", 3), ("done", 2)]),
    ] {
        for (st, n) in counts {
            for k in 0..n {
                list.push((
                    format!("{st}-{month}-{k}"),
                    format!("status: {st}\ndue: {month}"),
                ));
            }
        }
    }
    let refs: Vec<(&str, &str)> = list.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let (_t, mut a) = boot(
        "ch5by",
        &md(&refs),
        TYPES,
        "chart = { kind = \"line\", x = \"due\", by = \"status\", unit = \"month\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.by().as_deref(), Some("status"), "{}", c.raw);
    let legend = c.legend();
    for v in ["todo", "doing", "done"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    // 凡例の値ごとの点の字(値の左の、空白を飛ばした最も近い字)。値の語の中の o は数えない。
    let point_of = |v: &str| -> char {
        let i = legend.find(v).unwrap();
        let p = legend[..i]
            .chars()
            .rev()
            .find(|c| !c.is_whitespace())
            .unwrap_or_else(|| panic!("凡例の {v} の左に点の字が無い: {legend:?}"));
        assert!(
            POINTS.contains(&p),
            "凡例の {v} の左は点の字: {p:?} {legend:?}"
        );
        p
    };
    let used: Vec<char> = ["todo", "doing", "done"]
        .iter()
        .map(|v| point_of(v))
        .collect();
    assert!(
        used[0] != used[1] && used[1] != used[2] && used[0] != used[2],
        "内訳ごとに違う点の字: {used:?} {legend:?}"
    );
    for p in &used {
        assert!(c.points(*p) >= 1, "格子に点の字 {p}: {}", c.raw);
    }
    let d = c.desc();
    assert!(d.starts_with("2026-10"), "{d:?}");
    for part in ["todo 3", "doing 2", "done 1"] {
        assert!(d.contains(part), "説明に {part}: {d:?}");
    }
}

#[test]
fn test_ch_5_rows_without_dates_are_excluded() {
    // [utest -> req~chart.line-points~1] 日付の無い行(epsilon)は数えず「外した 1 行」。点は2つのまま。
    let mut notes = month_notes();
    notes.extend(md(&[("epsilon", "status: todo")]));
    let (_t, mut a) = boot("ch5nodate", &notes, TYPES, LINE_MONTH);
    let c = open(&mut a);
    assert_eq!(c.counts().1, Some(1), "{}", c.raw);
    assert_eq!(c.points('*'), 2, "{}", c.raw);
}

// ---- CH-6: 散布図 ----

/// 散布図の例: (estimate, actual) が alpha (1, 2)・beta (2, 3)・gamma (3, 1)。
fn scatter_notes() -> Vec<(String, String)> {
    md(&[
        ("alpha", "status: todo\nestimate: 1\nactual: 2"),
        ("beta", "status: done\nestimate: 2\nactual: 3"),
        ("gamma", "status: todo\nestimate: 3\nactual: 1"),
    ])
}

const SCATTER: &str = "chart = { kind = \"scatter\", x = \"estimate\", y = \"actual\" }";

#[test]
fn test_ch_6_scatter_points_selection_and_enter_selects_row() {
    // [utest -> req~chart.scatter-select~1, req~chart.scatter-enter~1] estimate と actual の数の列で3行 → 3点。題は「散布図 · estimate × actual」。選びの初めは横の値の最小の
    // alpha(「alpha: 1 × 2」)、j で beta、k で alpha、j → Enter で表に戻って beta の行を選ぶ(絞らない)。
    let (_t, mut a) = boot("ch6scatter", &scatter_notes(), TYPES, SCATTER);
    let c = open(&mut a);
    assert_eq!(c.kind(), "散布図", "{}", c.raw);
    assert_eq!(c.x(), "estimate", "{}", c.raw);
    assert_eq!(c.value(), "actual", "{}", c.raw);
    assert_eq!(c.points('*'), 3, "{}", c.raw);
    let d = c.desc();
    assert!(d.contains("alpha") && d.contains("1 × 2"), "{d:?}");
    ch(&mut a, 'j');
    let d = chart(&a).desc();
    assert!(d.contains("beta") && d.contains("2 × 3"), "{d:?}");
    ch(&mut a, 'k');
    let d = chart(&a).desc();
    assert!(d.contains("alpha"), "{d:?}");
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    assert_eq!(cur(&a), "beta");
    assert_eq!(a.rows.len(), 3, "散布図の Enter は絞らない");
}

#[test]
fn test_ch_6_rows_missing_a_value_are_excluded() {
    // [utest -> req~chart.scatter-skip~1] actual が空の行(delta)と estimate が読めない行(epsilon)→ 外した 2 行で、点は3つ。
    let mut notes = scatter_notes();
    notes.extend(md(&[
        ("delta", "status: todo\nestimate: 4"),
        ("epsilon", "status: todo\nestimate: abc\nactual: 2"),
    ]));
    let (_t, mut a) = boot("ch6missing", &notes, TYPES, SCATTER);
    let c = open(&mut a);
    assert_eq!(c.counts().1, Some(2), "{}", c.raw);
    assert_eq!(c.points('*'), 3, "{}", c.raw);
}

#[test]
fn test_ch_6_by_uses_point_chars() {
    // [utest -> req~chart.scatter-series-glyph~1, req~cell-view.not-color-only~1] by = status → 内訳ごとの点の字(多い todo が `*` で2点、done が `+` で1点)。凡例に値。
    let (_t, mut a) = boot(
        "ch6by",
        &scatter_notes(),
        TYPES,
        "chart = { kind = \"scatter\", x = \"estimate\", y = \"actual\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.points('*'), 2, "{}", c.raw);
    assert_eq!(c.points('+'), 1, "{}", c.raw);
    let legend = c.legend();
    assert!(
        legend.contains("todo") && legend.contains("done"),
        "{legend:?}"
    );
}

#[test]
fn test_ch_6_no_scatter_without_two_numeric_columns() {
    // [utest -> req~chart.scatter-points~1, req~chart.scatter-select~1, req~chart.keys-kind-unfit~1] 数か日付の列が1つ(estimate)しか無い → Tab で回しても散布図にならず、飛ばすときに理由1行。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 1"),
        ("beta", "status: done\nestimate: 2"),
    ]);
    let (_t, mut a) = boot("ch6one", &notes, TYPES, "");
    open(&mut a);
    let mut reasons = 0;
    for _ in 0..6 {
        a.message = None;
        press(&mut a, KeyCode::Tab);
        assert_chart(&a);
        let c = chart(&a);
        assert_ne!(c.kind(), "散布図", "{}", c.raw);
        if a.message.is_some() {
            reason(&a);
            reasons += 1;
        }
    }
    assert!(reasons >= 1, "散布図を飛ばすときに理由が出る");
}

// ---- CH-7: ヒートマップ ----

/// ヒートマップの例: todo × alice が2行(alpha・beta)、todo × bob が1行、done × bob が1行、doing × alice が1行。
/// 縦は status の多い順(todo 3、doing と done は 1 で名前の順)、横は assignee の多い順(alice 3・bob 2)。
fn heat_notes() -> Vec<(String, String)> {
    md(&[
        ("alpha", "status: todo\nassignee: alice"),
        ("beta", "status: todo\nassignee: alice"),
        ("gamma", "status: todo\nassignee: bob"),
        ("delta", "status: done\nassignee: bob"),
        ("epsilon", "status: doing\nassignee: alice"),
    ])
}

const HEAT: &str = "chart = { kind = \"heatmap\", x = \"status\", by = \"assignee\" }";

/// 左上のます目へ(端で止まる)。
fn to_top_left(a: &mut super::app::App) {
    for _ in 0..5 {
        ch(a, 'k');
        ch(a, 'h');
    }
}

#[test]
fn test_ch_7_heatmap_cells_show_values_and_density() {
    // [utest -> req~chart.heatmap-no-color~1, req~cell-view.not-color-only~1] status × assignee → todo の行に 2(alice)と 1(bob)。色なしでも最大のます目(todo × alice)に `█`。
    // 最大でないます目だけの行(doing・done)に `█` は無い。横の値の行に alice が bob より左。
    let (_t, mut a) = boot("ch7heat", &heat_notes(), "", HEAT);
    let c = open(&mut a);
    assert_eq!(c.kind(), "ヒートマップ", "{}", c.raw);
    assert_eq!(c.by().as_deref(), Some("assignee"), "{}", c.raw);
    let todo = c.heat_row("todo");
    assert_eq!(
        Chart::numbers(&todo),
        [2.0, 1.0],
        "{:?}\n{}",
        todo.text,
        c.raw
    );
    assert!(todo.text.contains('█'), "最大のます目に █: {:?}", todo.text);
    for v in ["doing", "done"] {
        let r = c.heat_row(v);
        assert!(!r.text.contains('█'), "{v}: {:?}\n{}", r.text, c.raw);
    }
    let head = c
        .below()
        .into_iter()
        .find(|r| r.text.contains("alice") && r.text.contains("bob"))
        .unwrap_or_else(|| panic!("横の値の行: {}", c.raw))
        .text
        .clone();
    assert!(head.find("alice") < head.find("bob"), "{head:?}");
}

#[test]
fn test_ch_7_hjkl_select_cells_and_describe() {
    // [utest -> req~chart.heatmap-select~1] h j k l でます目を選び、選んだ組と値を出す: 左上「todo × alice: 2」、l「todo × bob: 1」、
    // j「doing × bob: 0」、h「doing × alice: 1」。
    let (_t, mut a) = boot("ch7keys", &heat_notes(), "", HEAT);
    open(&mut a);
    to_top_left(&mut a);
    assert_eq!(chart(&a).desc(), "todo × alice: 2");
    ch(&mut a, 'l');
    assert_eq!(chart(&a).desc(), "todo × bob: 1");
    ch(&mut a, 'j');
    assert_eq!(chart(&a).desc(), "doing × bob: 0");
    ch(&mut a, 'h');
    assert_eq!(chart(&a).desc(), "doing × alice: 1");
    assert_chart(&a);
}

#[test]
fn test_ch_7_enter_filters_both_values() {
    // [utest -> req~chart.heatmap-enter~1] todo × alice で Enter → 表が status = todo かつ assignee = alice の行(alpha・beta)だけ。
    let (_t, mut a) = boot("ch7enter", &heat_notes(), "", HEAT);
    open(&mut a);
    to_top_left(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    assert_eq!(row_names(&a), ["alpha", "beta"]);
    assert_eq!(a.changes.count(), 0, "絞り込みでノートを書かない");
}

#[test]
fn test_ch_7_enter_on_date_bucket_gives_reason() {
    // [utest -> req~chart.heatmap-enter~1] 縦が日付の区切り(due の月)のヒートマップで Enter → 絞らずに理由1行(グラフのまま)。
    let notes = md(&[
        ("alpha", "due: 2026-10-03\nassignee: alice"),
        ("beta", "due: 2026-11-03\nassignee: bob"),
    ]);
    let (_t, mut a) = boot(
        "ch7date",
        &notes,
        TYPES,
        "chart = { kind = \"heatmap\", x = \"due\", by = \"assignee\", unit = \"month\" }",
    );
    open(&mut a);
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_chart(&a);
    reason(&a);
    assert_eq!(a.rows.len(), 2, "絞らない");
}

// ---- CH-8: キー ----

/// 種類を全部使える表: 文字(status・assignee)・日付(due)・数(estimate・actual)。
fn all_notes() -> Vec<(String, String)> {
    md(&[
        (
            "alpha",
            "status: todo\nassignee: alice\ndue: 2026-10-01\nestimate: 3\nactual: 4",
        ),
        (
            "beta",
            "status: todo\nassignee: bob\ndue: 2026-10-08\nestimate: 2\nactual: 2",
        ),
        (
            "gamma",
            "status: doing\nassignee: alice\ndue: 2026-10-15\nestimate: 4\nactual: 5",
        ),
        (
            "delta",
            "status: done\nassignee: bob\ndue: 2026-10-22\nestimate: 1\nactual: 1",
        ),
        (
            "epsilon",
            "status: done\nassignee: alice\ndue: 2026-10-29\nestimate: 1\nactual: 2",
        ),
        (
            "zeta",
            "status: done\nassignee: bob\ndue: 2026-11-05\nestimate: 1\nactual: 1",
        ),
    ])
}

#[test]
fn test_ch_8_tab_cycles_kinds_and_backtab_reverses() {
    // [utest -> req~chart.keys-kind-unfit~1] Tab で 棒 → 積み上げ棒 → 折れ線 → 散布図 → ヒートマップ → 棒。Shift+Tab で逆。
    let (_t, mut a) = boot("ch8tab", &all_notes(), TYPES, "");
    let c = open(&mut a);
    assert_eq!(c.kind(), "棒");
    let mut seen = vec![c.kind()];
    for _ in 0..5 {
        press(&mut a, KeyCode::Tab);
        assert_chart(&a);
        seen.push(chart(&a).kind());
    }
    assert_eq!(
        seen,
        ["棒", "積み上げ棒", "折れ線", "散布図", "ヒートマップ", "棒"],
        "Tab の順"
    );
    let mut back = vec![chart(&a).kind()];
    for _ in 0..5 {
        press(&mut a, KeyCode::BackTab);
        back.push(chart(&a).kind());
    }
    assert_eq!(
        back,
        ["棒", "ヒートマップ", "散布図", "折れ線", "積み上げ棒", "棒"],
        "Shift+Tab は逆の順"
    );
}

#[test]
fn test_ch_8_tab_skips_kinds_without_columns_with_reason() {
    // [utest -> req~chart.keys-kind-unfit~1] 日付と数の列の無い表(status・assignee)で、積み上げ棒から Tab → 折れ線・散布図にせず、理由1行で次の
    // 種類(ヒートマップ)へ。Shift+Tab でも同じく飛ばして積み上げ棒へ。
    let notes = md(&[
        ("alpha", "status: todo\nassignee: alice"),
        ("beta", "status: done\nassignee: bob"),
    ]);
    let (_t, mut a) = boot("ch8skip", &notes, "", "");
    open(&mut a);
    press(&mut a, KeyCode::Tab);
    assert_eq!(chart(&a).kind(), "積み上げ棒");
    a.message = None;
    press(&mut a, KeyCode::Tab);
    assert_chart(&a);
    assert_eq!(chart(&a).kind(), "ヒートマップ", "{}", screen(&a));
    reason(&a);
    press(&mut a, KeyCode::Tab);
    assert_eq!(chart(&a).kind(), "棒");
    press(&mut a, KeyCode::BackTab);
    assert_eq!(chart(&a).kind(), "ヒートマップ");
    a.message = None;
    press(&mut a, KeyCode::BackTab);
    assert_eq!(chart(&a).kind(), "積み上げ棒", "{}", screen(&a));
    reason(&a);
}

#[test]
fn test_ch_8_only_one_text_column_stays_bar_with_reason() {
    // [utest -> req~chart.keys-kind-unfit~1] 列が status だけ → 内訳にできる列も日付・数の列も無いので、Tab は棒のまま理由1行。
    let (_t, mut a) = boot("ch8onecol", &status_notes(), "", "");
    assert_eq!(a.src.columns(), ["status"], "前提: 列は status だけ");
    open(&mut a);
    a.message = None;
    press(&mut a, KeyCode::Tab);
    assert_chart(&a);
    assert_eq!(chart(&a).kind(), "棒");
    reason(&a);
}

#[test]
fn test_ch_8_x_cycles_split_column() {
    // [utest -> req~chart.keys-x~1] x で分ける列が次の列に変わり、押し続けると status に戻る。
    let (_t, mut a) = boot("ch8x", &all_notes(), TYPES, "");
    let c = open(&mut a);
    assert_eq!(c.x(), "status");
    ch(&mut a, 'x');
    let c = chart(&a);
    assert_ne!(c.x(), "status", "{}", c.raw);
    assert!(
        a.src.columns().contains(&c.x()),
        "表の列: {}\n{}",
        c.x(),
        c.raw
    );
    let mut back = false;
    for _ in 0..6 {
        ch(&mut a, 'x');
        if chart(&a).x() == "status" {
            back = true;
            break;
        }
    }
    assert!(back, "x を押し続けると status に戻る");
}

#[test]
fn test_ch_8_y_and_a_change_value_and_aggregation() {
    // [utest -> req~chart.keys-y~1, req~chart.keys-agg~1, req~chart.keys-charts~1] y で値が 件数 → estimate の合計(数の列)。a で 合計 → 平均 → 最小 → 最大 → 合計。y を押し続けると件数に戻る。
    // todo は alpha 3・beta 2、doing は gamma 4、done は 1・1・1。
    let (_t, mut a) = boot("ch8ya", &all_notes(), TYPES, "");
    let c = open(&mut a);
    assert_eq!(c.value(), "件数");
    ch(&mut a, 'y');
    let c = chart(&a);
    assert_eq!(c.value(), "estimate の合計", "{}", c.raw);
    let map = |c: &Chart| -> std::collections::BTreeMap<String, String> {
        c.items().into_iter().collect()
    };
    let m = map(&c);
    assert_eq!(
        (m["todo"].as_str(), m["doing"].as_str(), m["done"].as_str()),
        ("5", "4", "3"),
        "{}",
        c.raw
    );
    for (title, todo) in [
        ("estimate の平均", "2.5"),
        ("estimate の最小", "2"),
        ("estimate の最大", "3"),
        ("estimate の合計", "5"),
    ] {
        ch(&mut a, 'a');
        let c = chart(&a);
        assert_eq!(c.value(), title, "{}", c.raw);
        assert_eq!(map(&c)["todo"], todo, "{title}: {}", c.raw);
    }
    let mut back = false;
    for _ in 0..4 {
        ch(&mut a, 'y');
        if chart(&a).value() == "件数" {
            back = true;
            break;
        }
    }
    assert!(back, "y を押し続けると件数に戻る");
}

#[test]
fn test_ch_8_b_changes_breakdown_column() {
    // [utest -> req~chart.keys-kind-unfit~1] Tab → 積み上げ棒で、内訳は分ける列(status)と別の最初の内訳にできる列(assignee)。b で次の列(priority)。
    let notes = md(&[
        ("alpha", "status: todo\nassignee: alice\npriority: p1"),
        ("beta", "status: done\nassignee: bob\npriority: p2"),
        ("gamma", "status: done\nassignee: alice\npriority: p1"),
    ]);
    let (_t, mut a) = boot("ch8b", &notes, "", "");
    open(&mut a);
    press(&mut a, KeyCode::Tab);
    let c = chart(&a);
    assert_eq!(c.kind(), "積み上げ棒");
    assert_eq!(c.by().as_deref(), Some("assignee"), "{}", c.raw);
    ch(&mut a, 'b');
    let c = chart(&a);
    assert_eq!(c.by().as_deref(), Some("priority"), "{}", c.raw);
    assert!(
        c.legend().contains("p1") && c.legend().contains("p2"),
        "{}",
        c.raw
    );
}

#[test]
fn test_ch_8_plus_minus_change_date_unit() {
    // [utest -> req~chart.keys-unit~1, req~chart.keys-charts~1, req~chart.keys-kind-unfit~1, req~chart.keys-enter~1] 日付の列の棒で、+ で単位を細かく(月 → 週 → 日)、- で粗く。月より粗くはならない。
    let notes = md(&[
        ("alpha", "due: 2026-10-01"),
        ("beta", "due: 2026-10-02"),
        ("gamma", "due: 2026-11-03"),
    ]);
    let (_t, mut a) = boot(
        "ch8unit",
        &notes,
        TYPES,
        "chart = { x = \"due\", unit = \"month\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[("2026-10", "2"), ("2026-11", "1")]),
        "{}",
        c.raw
    );
    ch(&mut a, '+');
    let c = chart(&a);
    assert_eq!(
        c.items(),
        want(&[
            ("2026-09-28", "2"),
            ("2026-10-05", "0"),
            ("2026-10-12", "0"),
            ("2026-10-19", "0"),
            ("2026-10-26", "0"),
            ("2026-11-02", "1"),
        ]),
        "週: {}",
        c.raw
    );
    ch(&mut a, '+');
    let c = chart(&a);
    assert_eq!(c.desc(), "2026-10-01: 1", "日: {}", c.raw);
    assert_eq!(c.items()[1], ("2026-10-02".to_string(), "1".to_string()));
    ch(&mut a, '-');
    ch(&mut a, '-');
    ch(&mut a, '-');
    let c = chart(&a);
    assert_eq!(
        c.items(),
        want(&[("2026-10", "2"), ("2026-11", "1")]),
        "月より粗くならない: {}",
        c.raw
    );
}

#[test]
fn test_ch_8_switching_does_not_rewrite_marker() {
    // [utest -> req~chart.keys-charts~1, req~chart.keys-no-write~1] 画面の中の切り替え(Tab・x・y・a・b・+・-・[・]・c・%)は印を書き換えない。
    let (t, mut a) = boot(
        "ch8keep",
        &all_notes(),
        TYPES,
        "chart = [{ name = \"状態\", x = \"status\" }, { name = \"月ごと\", kind = \"line\", x = \"due\" }]",
    );
    let before = marker(&t);
    open(&mut a);
    for k in ['x', 'y', 'a', 'b', '+', '-', ']', 'c', '[', '%'] {
        ch(&mut a, k);
        assert_chart(&a);
    }
    press(&mut a, KeyCode::Tab);
    press(&mut a, KeyCode::BackTab);
    press(&mut a, KeyCode::Esc);
    assert_eq!(marker(&t), before, "印は書き換えない");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ch_8_enter_filters_like_frequency_table() {
    // [utest -> req~chart.keys-enter~1] done の棒(最初の項目)で Enter → 表が status = done の行だけになり、ヘッダーの帯に絞り込み(done)。
    let (_t, mut a) = boot("ch8enter", &status_notes(), "", "");
    assert!(!top_rows(&a).contains("done"), "前提: 帯に条件が無い");
    let c = open(&mut a);
    assert_eq!(c.selected(), ["done"]);
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    assert_eq!(row_names(&a), ["delta", "epsilon", "zeta"]);
    assert!(top_rows(&a).contains("done"), "帯に条件:\n{}", screen(&a));
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ch_8_enter_on_none_and_list_items() {
    // [utest -> req~chart.keys-kind-unfit~1, req~chart.keys-enter~1, req~chart.keys-click-again~1] 「(なし)」で Enter → 値の無い行だけ。リストの列の項目(会議)で Enter → 会議を含む行だけ。
    let notes = md(&[
        ("alpha", "status: todo\ntags: [会議, 家]"),
        ("beta", "status: todo\ntags: [会議]"),
        ("gamma", "title: g\ntags: [家]"),
    ]);
    let (_t, mut a) = boot(
        "ch8none",
        &notes,
        "[types]\ntags = \"list\"\n",
        "chart = { x = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[("todo", "2"), ("(なし)", "1")]),
        "{}",
        c.raw
    );
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    assert_eq!(row_names(&a), ["gamma"]);
    // リストの列。
    let (_t2, mut b) = boot(
        "ch8list",
        &notes,
        "[types]\ntags = \"list\"\n",
        "chart = { x = \"tags\" }",
    );
    let c = open(&mut b);
    assert_eq!(c.selected(), ["会議"], "{}", c.raw);
    press(&mut b, KeyCode::Enter);
    assert_table(&b);
    assert_eq!(row_names(&b), ["alpha", "beta"]);
}

#[test]
fn test_ch_8_enter_on_date_item_gives_reason() {
    // [utest -> req~chart.keys-enter~1] 日付の区切りの項目で Enter → 絞らずに理由1行(グラフのまま)。
    let notes = md(&[("alpha", "due: 2026-10-01"), ("beta", "due: 2026-11-02")]);
    let (_t, mut a) = boot(
        "ch8datenter",
        &notes,
        TYPES,
        "chart = { x = \"due\", unit = \"month\" }",
    );
    open(&mut a);
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_chart(&a);
    reason(&a);
    assert_eq!(a.rows.len(), 2);
}

#[test]
fn test_ch_8_esc_q_p_return_to_table() {
    // [utest -> req~chart.keys-no-write~1, req~chart.keys-enter~1, req~chart.keys-back~1] Esc・q・P で表に戻る(q で終わらない)。
    let (_t, mut a) = boot("ch8back", &status_notes(), "", "");
    open(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_table(&a);
    open(&mut a);
    ch(&mut a, 'q');
    assert_table(&a);
    assert!(!a.quit);
    open(&mut a);
    ch(&mut a, 'P');
    assert_table(&a);
    assert!(!a.quit);
}

#[test]
fn test_ch_8_arrow_keys_move_like_jk() {
    // [utest -> req~chart.keys-item~1, req~chart.keys-enter~1, req~chart.keys-click~1, req~chart.keys-click-again~1] ↓ ↑ は j k と同じ(棒の項目を選ぶ)。
    let (_t, mut a) = boot("ch8arrows", &status_notes(), "", "");
    open(&mut a);
    press(&mut a, KeyCode::Down);
    assert_eq!(chart(&a).selected(), ["todo"]);
    press(&mut a, KeyCode::Up);
    assert_eq!(chart(&a).selected(), ["done"]);
}

// ---- CH-8: マウス ----

#[test]
fn test_ch_8_click_selects_then_second_click_filters() {
    // [utest -> req~chart.keys-click-again~1] 項目(todo の棒)のクリックで選ぶ(グラフのまま)。選んでいる項目をもう一度クリック → Enter と同じく
    // todo の行だけの表。
    let (_t, mut a) = boot("ch8click", &status_notes(), "", "");
    let c = open(&mut a);
    let (x, y) = c.bar("todo").at;
    a.click(x, y);
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(c.selected(), ["todo"], "{}", c.raw);
    assert_eq!(c.desc(), "todo: 2");
    let (x, y) = c.bar("todo").at;
    a.click(x + 1, y);
    assert_table(&a);
    assert_eq!(row_names(&a), ["alpha", "beta"]);
}

#[test]
fn test_ch_8_wheel_moves_selection() {
    // [utest -> req~chart.keys-wheel~1] ホイールで選びを1つ動かす。
    let (_t, mut a) = boot("ch8wheel", &status_notes(), "", "");
    open(&mut a);
    a.wheel(true);
    assert_eq!(chart(&a).selected(), ["todo"]);
    a.wheel(true);
    assert_eq!(chart(&a).selected(), ["doing"]);
    a.wheel(false);
    assert_eq!(chart(&a).selected(), ["todo"]);
    assert_chart(&a);
}

// ---- CH-8: 外の変化と読むだけ ----

#[test]
fn test_ch_8_external_change_recounts() {
    // [utest -> req~chart.keys-recount~1] 開いたまま外で alpha の status を todo から done に書き換える → 数え直して done 4・todo 1・doing 1。
    let (t, mut a) = boot("ch8ext", &status_notes(), "", "");
    open(&mut a);
    edit_outside(&t, "alpha.md", "---\nstatus: done\n---\n");
    a.poll();
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(
        c.items(),
        want(&[("done", "4"), ("doing", "1"), ("todo", "1")]),
        "{}",
        c.raw
    );
    assert_eq!(c.counts(), (6, None));
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ch_8_values_cannot_be_edited_in_chart() {
    // [utest -> req~chart.keys-read-only~1] グラフの画面では値を直せない: 直すキー(e・i・L・H・d・Space)を押しても、ためた変更は 0 でノートは同じ。
    let (t, mut a) = boot("ch8ro", &status_notes(), "", "");
    open(&mut a);
    for k in ['e', 'i', 'L', 'H', 'd', ' '] {
        ch(&mut a, k);
        if a.mode != Mode::Chart {
            press(&mut a, KeyCode::Esc);
            if a.mode == Mode::Table {
                open(&mut a);
            }
        }
    }
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    assert_eq!(read(&t, "alpha.md"), "---\nstatus: todo\n---\n");
}

#[test]
fn test_ch_8_kind_names_cover_all_kinds() {
    // [utest -> req~chart.keys-kind-unfit~1] 題の種類の名前は5つ(棒・積み上げ棒・折れ線・散布図・ヒートマップ)で、Tab で全部に行ける(前提の確かめ)。
    let (_t, mut a) = boot("ch8names", &all_notes(), TYPES, "");
    open(&mut a);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..5 {
        seen.insert(chart(&a).kind());
        press(&mut a, KeyCode::Tab);
    }
    let all: std::collections::BTreeSet<String> =
        KINDS.iter().map(std::string::ToString::to_string).collect();
    assert_eq!(seen, all);
}
