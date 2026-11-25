//! グラフの画面の数え方(CH-3)と棒・積み上げ棒(CH-4)の受け入れの試験。値は作ったもの。
//! 記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。補助は test_chart.rs。
//! 日付の区切りの単位は、開くときは区切りの軸の長さに収まる最も細かい単位なので、区切りを確かめる試験は印に
//! `unit` を書く。

use super::app::ColorMode;
use super::test_chart::{
    assert_ratio, boot, boot_full, md, open, status_notes, want, Chart, FILLS,
};
use super::test_screen::ch;
use std::collections::BTreeMap;

/// 棒の 名前 → 値。
fn values(c: &Chart) -> BTreeMap<String, String> {
    c.items().into_iter().collect()
}

// ---- CH-3: 分ける列の値と数え方 ----

#[test]
fn test_ch_3_list_column_counts_each_element() {
    // [utest -> req~chart.buckets-list-items~1] リストの列(tags)は要素ごと: 会議 と 家 を持つ行は、会議にも家にも1。値の無い行は最後の「(なし)」。
    // 体の1行目は行の数(4 行。項目の数の和ではない)。
    let notes = md(&[
        ("alpha", "tags: [会議, 家]"),
        ("beta", "tags: [会議]"),
        ("gamma", "tags: [買い物]"),
        ("delta", "title: d"),
    ]);
    let (_t, mut a) = boot(
        "ch3list",
        &notes,
        "[types]\ntags = \"list\"\n",
        "chart = { x = \"tags\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.x(), "tags", "{}", c.raw);
    let v = values(&c);
    assert_eq!(v.get("会議").map(String::as_str), Some("2"), "{}", c.raw);
    assert_eq!(v.get("家").map(String::as_str), Some("1"), "{}", c.raw);
    assert_eq!(v.get("買い物").map(String::as_str), Some("1"), "{}", c.raw);
    assert_eq!(v.get("(なし)").map(String::as_str), Some("1"), "{}", c.raw);
    let items = c.items();
    assert_eq!(items.len(), 4, "{}", c.raw);
    assert_eq!(items[0].0, "会議", "多い順: {}", c.raw);
    assert_eq!(items[3].0, "(なし)", "(なし) は最後: {}", c.raw);
    assert_eq!(c.counts(), (4, None), "{}", c.raw);
}

#[test]
fn test_ch_3_week_buckets_include_empty_weeks() {
    // [utest -> req~chart.buckets-date~1] due が 10/01 と 10/20 の2行で週の単位 → 10/01 の週(2026-09-27)から 10/20 の週(2026-10-18)まで
    // 4つの区切り(間の2つは 0)が小さい順に並ぶ。週は既定の日曜で区切る(chart.buckets-week-start)。
    let notes = md(&[("alpha", "due: 2026-10-01"), ("beta", "due: 2026-10-20")]);
    let (_t, mut a) = boot(
        "ch3week",
        &notes,
        "[types]\ndue = \"date\"\n",
        "chart = { x = \"due\", unit = \"week\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[
            ("2026-09-27", "1"),
            ("2026-10-04", "0"),
            ("2026-10-11", "0"),
            ("2026-10-18", "1"),
        ]),
        "{}",
        c.raw
    );
    assert_eq!(c.counts(), (2, None), "{}", c.raw);
}

#[test]
fn test_ch_3_month_buckets_and_rows_without_dates_are_excluded() {
    // [utest -> req~chart.buckets-x-skip~1] 日付の列で値の無い行(gamma)と読めない値の行(delta の someday)は数えず、外した 2 行と示す。
    // 月の単位: 2026-10 に2つ、2026-11 に1つ。(なし)の項目は作らない。
    let notes = md(&[
        ("alpha", "due: 2026-10-01"),
        ("beta", "due: 2026-10-30"),
        ("epsilon", "due: 2026-11-02"),
        ("gamma", "title: g"),
        ("delta", "due: someday"),
    ]);
    let (_t, mut a) = boot(
        "ch3nodate",
        &notes,
        "[types]\ndue = \"date\"\n",
        "chart = { x = \"due\", unit = \"month\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[("2026-10", "2"), ("2026-11", "1")]),
        "{}",
        c.raw
    );
    assert_eq!(c.counts().1, Some(2), "外した 2 行: {}", c.raw);
}

#[test]
fn test_ch_3_sum_skips_non_numbers_and_shows_excluded() {
    // [utest -> req~chart.buckets-y-skip~1] estimate の合計で値が "abc" の行(beta)→ 合計に入れず「外した 1 行」。todo は alpha の 3 だけ、done は 5。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 3"),
        ("beta", "status: todo\nestimate: abc"),
        ("gamma", "status: done\nestimate: 5"),
    ]);
    let (_t, mut a) = boot(
        "ch3sum",
        &notes,
        "[types]\nestimate = \"number\"\n",
        "chart = { x = \"status\", y = \"estimate\", agg = \"sum\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.value(), "estimate の合計", "{}", c.raw);
    let v = values(&c);
    assert_eq!(v.get("todo").map(String::as_str), Some("3"), "{}", c.raw);
    assert_eq!(v.get("done").map(String::as_str), Some("5"), "{}", c.raw);
    assert_eq!(c.counts().1, Some(1), "外した 1 行: {}", c.raw);
}

#[test]
fn test_ch_3_avg_min_max_use_number_rows_only() {
    // [utest -> req~chart.buckets-agg~1] 平均・最小・最大は数の値のある行だけで出す: todo は 2 と 4(と値の無い行)→ 平均 3・最小 2・最大 4。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 2"),
        ("beta", "status: todo\nestimate: 4"),
        ("gamma", "status: todo"),
    ]);
    for (agg, title, val) in [
        ("avg", "estimate の平均", "3"),
        ("min", "estimate の最小", "2"),
        ("max", "estimate の最大", "4"),
    ] {
        let (_t, mut a) = boot(
            &format!("ch3{agg}"),
            &notes,
            "[types]\nestimate = \"number\"\n",
            &format!("chart = {{ x = \"status\", y = \"estimate\", agg = \"{agg}\" }}"),
        );
        let c = open(&mut a);
        assert_eq!(c.value(), title, "{}", c.raw);
        assert_eq!(c.items(), want(&[("todo", val)]), "{agg}: {}", c.raw);
    }
}

#[test]
fn test_ch_3_number_column_is_split_into_bins() {
    // [utest -> req~chart.buckets-date~1, req~chart.buckets-number~1, req~chart.buckets-x-skip~1] 数の列で分ける → 等しい幅の区間(10 まで)を小さい順に、行の無い区間も 0 として並べる。
    // 整数 1・2・3・5 → 1 ずつの 5 区間(4 の区間は 0)。値の無い行は数えず「外した 1 行」。
    let notes = md(&[
        ("alpha", "estimate: 1"),
        ("beta", "estimate: 2"),
        ("gamma", "estimate: 3"),
        ("delta", "estimate: 5"),
        ("epsilon", "title: e"),
    ]);
    let (_t, mut a) = boot(
        "ch3bins",
        &notes,
        "[types]\nestimate = \"number\"\n",
        "chart = { x = \"estimate\" }",
    );
    let c = open(&mut a);
    let vals: Vec<String> = c.items().into_iter().map(|(_, v)| v).collect();
    assert_eq!(vals, ["1", "1", "1", "0", "1"], "{}", c.raw);
    assert_eq!(c.counts().1, Some(1), "{}", c.raw);
}

#[test]
fn test_ch_3_order_by_count_then_name_and_none_last() {
    // [utest -> req~chart.buckets-order~2] 決まりの無い列: 値の大きい順(done 3・todo 2)、同じなら名前の順(doing・waiting)、値の無い行は最後の
    // 「(なし)」(数が多くても最後)。
    let notes = md(&[
        ("a1", "status: todo"),
        ("a2", "status: todo"),
        ("a3", "status: waiting"),
        ("a4", "status: doing"),
        ("a5", "status: done"),
        ("a6", "status: done"),
        ("a7", "status: done"),
        ("n1", "title: x"),
        ("n2", "title: y"),
        ("n3", "title: z"),
        ("n4", "title: w"),
    ]);
    let (_t, mut a) = boot("ch3order", &notes, "", "chart = { x = \"status\" }");
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[
            ("done", "3"),
            ("todo", "2"),
            ("doing", "1"),
            ("waiting", "1"),
            ("(なし)", "4"),
        ]),
        "{}",
        c.raw
    );
}

#[test]
fn test_ch_3_rule_order_wins_over_count() {
    // [utest -> req~chart.buckets-order~2] 列の決まり(values = [todo, doing, done])があればその順(件数の順の done・todo・doing ではない)。
    let (_t, mut a) = boot(
        "ch3rules",
        &status_notes(),
        "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n",
        "chart = { x = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[("todo", "2"), ("doing", "1"), ("done", "3")]),
        "{}",
        c.raw
    );
}

// ---- CH-4: 棒と積み上げ棒 ----

#[test]
fn test_ch_4_bar_lengths_follow_values() {
    // [utest -> req~chart.bar-scale~2] status(todo 2・doing 1・done 3)の棒 → done の棒が全長、todo はその 2/3、doing は 1/3(1字の幅)。
    // 右に値、選んでいるのは最初の項目(done)だけで `>` が付き、説明は「done: 3」。
    let (_t, mut a) = boot("ch4ratio", &status_notes(), "", "");
    let c = open(&mut a);
    assert_ratio(&c, "todo", "done", 2.0, 3.0);
    assert_ratio(&c, "doing", "done", 1.0, 3.0);
    let full = c.bar("done").len;
    assert!(
        c.bars().iter().all(|b| b.len <= full),
        "最大の値の棒が最も長い: {}",
        c.raw
    );
    assert_eq!(c.selected(), ["done"], "{}", c.raw);
    assert_eq!(c.desc(), "done: 3", "{}", c.raw);
}

#[test]
fn test_ch_4_j_k_move_selection_marker() {
    // [utest -> req~chart.bar-cursor~1] 色なしでも、選んでいる項目に `>`(1つだけ)。j で次、k で前。説明も移る(SR-15)。
    let (_t, mut a) = boot("ch4jk", &status_notes(), "", "");
    open(&mut a);
    ch(&mut a, 'j');
    let c = super::test_chart::chart(&a);
    assert_eq!(c.selected(), ["todo"], "{}", c.raw);
    assert_eq!(c.desc(), "todo: 2", "{}", c.raw);
    ch(&mut a, 'j');
    let c = super::test_chart::chart(&a);
    assert_eq!(c.selected(), ["doing"], "{}", c.raw);
    ch(&mut a, 'k');
    let c = super::test_chart::chart(&a);
    assert_eq!(c.selected(), ["todo"], "{}", c.raw);
}

#[test]
fn test_ch_4_negative_value_draws_no_bar() {
    // [utest -> req~chart.bar-scale~2] 負の値は棒を描かず値だけ: gain の合計で todo が -2 → todo の棒は長さ 0 で値「-2」。done の 3 が全長。
    let notes = md(&[
        ("alpha", "status: todo\ngain: -2"),
        ("beta", "status: done\ngain: 3"),
    ]);
    let (_t, mut a) = boot(
        "ch4neg",
        &notes,
        "[types]\ngain = \"number\"\n",
        "chart = { x = \"status\", y = \"gain\", agg = \"sum\" }",
    );
    let c = open(&mut a);
    let todo = c.bar("todo");
    assert_eq!(todo.value, "-2", "{}", c.raw);
    assert_eq!(todo.len, 0.0, "負の値は棒を描かない: {}", c.raw);
    assert!(c.bar("done").len >= 6.0, "{}", c.raw);
}

#[test]
fn test_ch_4_item_without_numbers_shows_dash() {
    // [utest -> req~chart.bar-rows~1, req~chart.bar-scale~2, req~chart.bar-stacked~1, req~chart.bar-stacked-top5~2] 数の値のある行が無い項目は値なし: 平均で doing は「-」で棒なし。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 2"),
        ("beta", "status: doing"),
    ]);
    let (_t, mut a) = boot(
        "ch4dash",
        &notes,
        "[types]\nestimate = \"number\"\n",
        "chart = { x = \"status\", y = \"estimate\", agg = \"avg\" }",
    );
    let c = open(&mut a);
    let doing = c.bar("doing");
    assert_eq!(doing.value, "-", "{}", c.raw);
    assert_eq!(doing.len, 0.0, "{}", c.raw);
    assert_eq!(c.bar("todo").value, "2", "{}", c.raw);
}

/// 積み上げ棒の例: alice は todo 2・done 1、bob は todo 1・doing 1。内訳の全体の数は todo 3・done 1・doing 1。
fn stacked_notes() -> Vec<(String, String)> {
    md(&[
        ("a1", "assignee: alice\nstatus: todo"),
        ("a2", "assignee: alice\nstatus: todo"),
        ("a3", "assignee: alice\nstatus: done"),
        ("b1", "assignee: bob\nstatus: todo"),
        ("b2", "assignee: bob\nstatus: doing"),
    ])
}

#[test]
fn test_ch_4_stacked_legend_and_fills() {
    // [utest -> req~chart.bar-stacked~1, req~chart.bar-fill-glyph~1, req~cell-view.not-color-only~1] 積み上げ棒で x = assignee、by = status → 題に「内訳 status」、凡例に3つの値と、内訳ごとに違う
    // 塗りの字(色なしでも見分けられる)。一番多い todo は `█`。alice の棒に doing の字は無く、bob の棒に done の字は無い。
    let (_t, mut a) = boot(
        "ch4stack",
        &stacked_notes(),
        "",
        "chart = { kind = \"stacked\", x = \"assignee\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "積み上げ棒", "{}", c.raw);
    assert_eq!(c.by().as_deref(), Some("status"), "{}", c.raw);
    let legend = c.legend();
    for v in ["todo", "doing", "done"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    // 凡例の値ごとの塗りの字(値の左の最も近い塗りの字)。
    let fill_of = |v: &str| -> char {
        let i = legend.find(v).unwrap();
        legend[..i]
            .chars()
            .rev()
            .find(|c| FILLS.contains(c))
            .unwrap_or_else(|| panic!("凡例の {v} に塗りの字が無い: {legend:?}"))
    };
    let (todo, doing, done) = (fill_of("todo"), fill_of("doing"), fill_of("done"));
    assert_eq!(todo, '█', "全体で一番多い todo が1つ目の字: {legend:?}");
    assert!(
        todo != doing && doing != done && todo != done,
        "内訳ごとに違う字: {legend:?}"
    );
    let alice = c.bar("alice");
    let bob = c.bar("bob");
    assert!(
        alice.fills.contains(todo) && alice.fills.contains(done) && !alice.fills.contains(doing),
        "alice: {:?}\n{}",
        alice.fills,
        c.raw
    );
    assert!(
        bob.fills.contains(todo) && bob.fills.contains(doing) && !bob.fills.contains(done),
        "bob: {:?}\n{}",
        bob.fills,
        c.raw
    );
    // 塗りの長さの比: alice の todo は done の2倍(1字の幅)。
    let n = |s: &str, f: char| s.chars().filter(|c| *c == f).count() as f64;
    assert!(
        (n(&alice.fills, todo) - 2.0 * n(&alice.fills, done)).abs() <= 2.0,
        "alice の todo は done の2倍: {:?}",
        alice.fills
    );
}

#[test]
fn test_ch_4_percent_makes_every_bar_full() {
    // [utest -> req~chart.bar-percent~1] % で各棒を全長にした内訳の割合: alice(3行)と bob(2行)の棒の長さが同じになる。もう一度 % で戻る。
    let (_t, mut a) = boot(
        "ch4pct",
        &stacked_notes(),
        "",
        "chart = { kind = \"stacked\", x = \"assignee\", by = \"status\" }",
    );
    let c = open(&mut a);
    let (al, bl) = (c.bar("alice").len, c.bar("bob").len);
    assert!(al > bl + 1.0, "前提: 割合の前は alice が長い: {}", c.raw);
    ch(&mut a, '%');
    let c = super::test_chart::chart(&a);
    let (al, bl) = (c.bar("alice").len, c.bar("bob").len);
    assert!((al - bl).abs() <= 1.0, "どの棒も全長: {al} {bl}\n{}", c.raw);
    ch(&mut a, '%');
    let c = super::test_chart::chart(&a);
    assert!(
        c.bar("alice").len > c.bar("bob").len + 1.0,
        "% で戻る: {}",
        c.raw
    );
}

#[test]
fn test_ch_4_stacked_keeps_five_and_groups_rest_as_other() {
    // [utest -> req~chart.bar-stacked-top5~2] 内訳は全体で多い順に5つまで(o1〜o5)、残り(o6・o7)は「その他」(塗りの字 `:`)にまとめる。
    let mut list: Vec<(String, String)> = Vec::new();
    for (owner, n) in [
        ("o1", 6),
        ("o2", 5),
        ("o3", 4),
        ("o4", 3),
        ("o5", 2),
        ("o6", 1),
        ("o7", 1),
    ] {
        for k in 0..n {
            list.push((
                format!("{owner}-{k}"),
                format!("status: todo\nowner: {owner}"),
            ));
        }
    }
    let refs: Vec<(&str, &str)> = list.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let (_t, mut a) = boot(
        "ch4other",
        &md(&refs),
        "",
        "chart = { kind = \"stacked\", x = \"status\", by = \"owner\" }",
    );
    let c = open(&mut a);
    let legend = c.legend();
    for v in ["o1", "o2", "o3", "o4", "o5", "その他"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    for v in ["o6", "o7"] {
        assert!(!legend.contains(v), "{v} はその他に入る: {legend:?}");
    }
    let todo = c.bar("todo");
    assert!(
        todo.fills.contains(':'),
        "その他の塗りは `:`: {:?}",
        todo.fills
    );
    let kinds: std::collections::BTreeSet<char> = todo.fills.chars().collect();
    assert_eq!(kinds.len(), 6, "5つの内訳とその他: {:?}", todo.fills);
}

#[test]
fn test_ch_4_scrolls_to_keep_selected_visible() {
    // [utest -> req~chart.bar-scroll~1] 項目が高さに収まらない(40 個・高さ 30)→ 選んでいる項目が見える所までずらす。j 39 回で最後の v39 が
    // 見えて `>` が付き、最初の v00 は隠れる。
    let list: Vec<(String, String)> = (0..40)
        .map(|i| (format!("n{i:02}"), format!("cat: v{i:02}")))
        .collect();
    let refs: Vec<(&str, &str)> = list.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let (_t, mut a) = boot("ch4scroll", &md(&refs), "", "chart = { x = \"cat\" }");
    let c = open(&mut a);
    assert!(c.bars().len() < 40, "前提: 全部は入らない: {}", c.raw);
    assert_eq!(c.selected(), ["v00"], "{}", c.raw);
    for _ in 0..39 {
        ch(&mut a, 'j');
    }
    let c = super::test_chart::chart(&a);
    assert_eq!(c.selected(), ["v39"], "{}", c.raw);
    assert_eq!(c.desc(), "v39: 1", "{}", c.raw);
    assert!(
        !c.bars().iter().any(|b| b.name == "v00"),
        "最初の項目は隠れる: {}",
        c.raw
    );
}

#[test]
fn test_ch_4_no_color_narrow_terminal_still_marks_selection() {
    // [utest -> req~chart.bar-rows~1, req~chart.bar-scale~2, req~chart.bar-fill-glyph~1, req~chart.bar-cursor~1, req~cell-view.not-color-only~1] 色なしで幅 80・高さ 24 でも、棒の行と `>` と値が出る(3本、done に `>`)。
    let (_t, mut a) = boot_full(
        "ch4narrow",
        &status_notes(),
        "",
        "",
        ColorMode::None,
        (80, 24),
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        want(&[("done", "3"), ("todo", "2"), ("doing", "1")]),
        "{}",
        c.raw
    );
    assert_eq!(c.selected(), ["done"], "{}", c.raw);
}
