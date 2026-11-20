//! グラフのレビューの指摘(#1〜#9)の直しの試験。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md の「レビューの指摘」。
//! 値は作ったもの。前の試験のファイル(test_chart*.rs)は変えず、補助は test_chart.rs のものを使う。
//!
//! 直す前の版で落ち、直した後の版で通る期待にする。キーとマウスを送り、画面の文字と今ある口(`mode`・`message`・
//! `rows`・`click`)で見る。題の日付の単位は末尾の ` · 単位 <単位>`、理由の文は決め打ちせず、指摘が名指す文
//! (「その他」・ChartNeedAxis)だけを見る。

use super::app::ColorMode;
use super::keymap::Mode;
use super::test_chart::{
    assert_chart, assert_table, boot, boot_full, chart, md, menu_item, menu_text, open, reason,
    row_names, screen, Chart, POINTS,
};
use super::test_screen::{ch, press};
use super::width::width;
use mdgrid::i18n::Msg;
use ratatui::crossterm::event::KeyCode;

/// 日付と数の列の型。
const TYPES: &str =
    "[types]\nstart = \"date\"\ndue = \"date\"\nestimate = \"number\"\nn = \"number\"\n";

/// 名前と中身の並びを、`md` に渡せる形に。
fn refs(list: &[(String, String)]) -> Vec<(&str, &str)> {
    list.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

/// Tab を最大 6 回押して、種類が `kind` になったら true。
fn tab_until(a: &mut super::app::App, kind: &str) -> bool {
    for _ in 0..6 {
        press(a, KeyCode::Tab);
        assert_chart(a);
        if chart(a).kind() == kind {
            return true;
        }
    }
    false
}

/// 題の日付の単位(` · 単位 <単位>` の <単位>)。
fn title_unit(c: &Chart) -> Option<String> {
    let i = c.title.find("単位 ")?;
    let rest = &c.title[i + "単位 ".len()..];
    Some(
        rest.split([' ', '·', '('])
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
    )
}

/// グラフの枠の行(題のある上の辺から下の辺まで。メッセージ行を除く)。
fn frame_lines(s: &str) -> Vec<String> {
    let lines: Vec<&str> = s.lines().collect();
    let body = &lines[..lines.len().saturating_sub(1)];
    let Some(top) = body.iter().position(|l| {
        l.match_indices("グラフ")
            .any(|(i, w)| l[i + w.len()..].trim_start().starts_with([':', ':']))
    }) else {
        panic!("グラフの枠が無い:\n{s}");
    };
    let mut out = Vec::new();
    for l in &body[top..] {
        out.push(l.to_string());
        if out.len() > 1 && l.trim_start().starts_with(['╰', '└', '┗', '╯', '┘', '┛']) {
            break;
        }
    }
    out
}

/// 枠のどの行も、右の縁が上の辺の右の角と同じ桁にある(行の右の端の空白を除いた幅が同じ)。
fn assert_inside_frame(a: &super::app::App, what: &str) {
    let s = screen(a);
    let lines = frame_lines(&s);
    let w0 = width(lines[0].trim_end());
    for l in &lines {
        let t = l.trim_end();
        assert_eq!(
            width(t),
            w0,
            "{what}: 枠の右の縁の桁がそろわない(行が枠を越えるか短い): {t:?}\n{s}"
        );
        let last = t.chars().last().unwrap_or(' ');
        assert!(
            ['│', '┃', '|', '╮', '┐', '┓', '╯', '┘', '┛'].contains(&last),
            "{what}: 行の右の端が枠の縁でない: {t:?}\n{s}"
        );
    }
}

// ---- #1: 散布図の縦に日付の列 ----

#[test]
fn test_ch_6_scatter_with_two_date_columns_via_tab() {
    // [utest -> req~chart.scatter-points~1, req~chart.scatter-series-glyph~1, req~chart.scatter-select~1, req~chart.keys-kind-unfit~1] #1: 日付の列 start・due だけの表 → Tab を回すと散布図に入り、題は2つの列(start と due)。
    let notes = md(&[
        ("alpha", "start: 2026-10-01\ndue: 2026-10-05"),
        ("beta", "start: 2026-10-03\ndue: 2026-10-10"),
        ("gamma", "start: 2026-10-06\ndue: 2026-10-08"),
    ]);
    let (_t, mut a) = boot("chfx1dates", &notes, TYPES, "");
    open(&mut a);
    assert!(
        tab_until(&mut a, "散布図"),
        "日付の列が2つあれば散布図にできる: {:?}\n{}",
        a.message,
        screen(&a)
    );
    let c = chart(&a);
    let (x, y) = (c.x(), c.value());
    assert_ne!(x, y, "{}", c.raw);
    for col in [&x, &y] {
        assert!(col == "start" || col == "due", "{col}: {}", c.raw);
    }
    assert_eq!(c.points('*'), 3, "3行 → 3点: {}", c.raw);
}

#[test]
fn test_ch_6_scatter_marker_with_date_y_draws_points() {
    // [utest -> req~chart.scatter-points~1, req~chart.scatter-select~1] #1: 印の kind = "scatter", x = "estimate", y = "due" → 3点を打つ。選んだ点(横の最小の alpha)の説明の
    // 値は日付の形(2026-10-03)。
    let notes = md(&[
        ("alpha", "estimate: 1\ndue: 2026-10-03"),
        ("beta", "estimate: 2\ndue: 2026-10-10"),
        ("gamma", "estimate: 3\ndue: 2026-10-01"),
    ]);
    let (_t, mut a) = boot(
        "chfx1marker",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", x = \"estimate\", y = \"due\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "散布図", "{}", c.raw);
    assert_eq!(c.points('*'), 3, "{}", c.raw);
    let d = c.desc();
    assert!(
        d.contains("alpha") && d.contains("2026-10-03"),
        "{d:?}\n{}",
        c.raw
    );
}

#[test]
fn test_ch_8_line_on_number_tabs_into_scatter_with_date_y() {
    // [utest -> req~chart.keys-charts~1, req~chart.keys-kind-unfit~1, req~chart.scatter-points~1, req~chart.scatter-skip~1] #1: 数の列 estimate と日付の列 due だけの表で、x = estimate の折れ線から Tab → 散布図に入る
    // (飛ばさない。縦は due)。
    let notes = md(&[
        ("alpha", "estimate: 1\ndue: 2026-10-03"),
        ("beta", "estimate: 2\ndue: 2026-10-10"),
    ]);
    let (_t, mut a) = boot(
        "chfx1line",
        &notes,
        TYPES,
        "chart = { kind = \"line\", x = \"estimate\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "折れ線", "{}", c.raw);
    a.message = None;
    press(&mut a, KeyCode::Tab);
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(c.kind(), "散布図", "飛ばさない: {:?}\n{}", a.message, c.raw);
    assert_eq!(c.points('*'), 2, "{}", c.raw);
}

#[test]
fn test_ch_6_single_date_column_still_skips_scatter() {
    // [utest -> req~chart.scatter-points~1, req~chart.keys-kind-unfit~1] #1 の境界: 数か日付の列が1つ(due)しか無い表では、今までどおり散布図を飛ばして理由。
    let notes = md(&[
        ("alpha", "status: todo\ndue: 2026-10-03"),
        ("beta", "status: done\ndue: 2026-10-10"),
    ]);
    let (_t, mut a) = boot("chfx1one", &notes, TYPES, "");
    open(&mut a);
    let mut reasons = 0;
    for _ in 0..6 {
        a.message = None;
        press(&mut a, KeyCode::Tab);
        assert_chart(&a);
        assert_ne!(chart(&a).kind(), "散布図", "{}", screen(&a));
        if a.message.is_some() {
            reason(&a);
            reasons += 1;
        }
    }
    assert!(reasons >= 1, "散布図を飛ばすときに理由");
}

// ---- #2: 内訳は件数の多い順に選ぶ ----

#[test]
fn test_ch_4_stacked_picks_top_five_by_count_not_rule_order() {
    // [utest -> req~chart.bar-stacked-top5~1] #2: [rules.owner] values = o1〜o6 で o1〜o5 が1行ずつ、o6 が 10 行 → 内訳は件数の多い順に5つ
    // (o6 と、同じ件数の o1〜o4)。凡例に o6 が出て、「その他」には名前の順で後ろの o5 が入る。
    let mut list: Vec<(String, String)> = Vec::new();
    for o in ["o1", "o2", "o3", "o4", "o5"] {
        list.push((format!("n-{o}"), format!("status: todo\nowner: {o}")));
    }
    for k in 0..10 {
        list.push((format!("m-{k:02}"), "status: todo\nowner: o6".to_string()));
    }
    let (_t, mut a) = boot(
        "chfx2stack",
        &md(&refs(&list)),
        "[rules.owner]\nvalues = [\"o1\", \"o2\", \"o3\", \"o4\", \"o5\", \"o6\"]\n",
        "chart = { kind = \"stacked\", x = \"status\", by = \"owner\" }",
    );
    let c = open(&mut a);
    let legend = c.legend();
    for v in ["o6", "o1", "o2", "o3", "o4", "その他"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    assert!(
        !legend.contains("o5"),
        "o5 は「その他」に入る: {legend:?}\n{}",
        c.raw
    );
}

#[test]
fn test_ch_7_heatmap_picks_top_columns_by_count_not_rule_order() {
    // [utest -> req~chart.heatmap-cells~1, req~chart.heatmap-enter~1, req~chart.bar-stacked-top5~1] #2: ヒートマップの横(12 まで)も件数の多い順に選ぶ: [rules.tag] values = h01〜h13 で h13 が 10 行・
    // ほかが1行ずつ → h13 は「その他」でなく自分の列になり、ます目の説明に「todo × h13: 10」が出る。
    let mut list: Vec<(String, String)> = Vec::new();
    for i in 1..=12 {
        list.push((format!("n-{i:02}"), format!("status: todo\ntag: h{i:02}")));
    }
    for k in 0..10 {
        list.push((format!("m-{k:02}"), "status: todo\ntag: h13".to_string()));
    }
    let values: Vec<String> = (1..=13).map(|i| format!("\"h{i:02}\"")).collect();
    let (_t, mut a) = boot(
        "chfx2heat",
        &md(&refs(&list)),
        &format!("[rules.tag]\nvalues = [{}]\n", values.join(", ")),
        "chart = { kind = \"heatmap\", x = \"status\", by = \"tag\" }",
    );
    open(&mut a);
    for _ in 0..15 {
        ch(&mut a, 'h');
    }
    let mut seen = vec![chart(&a).desc()];
    for _ in 0..15 {
        ch(&mut a, 'l');
        seen.push(chart(&a).desc());
    }
    assert!(
        seen.iter().any(|d| d == "todo × h13: 10"),
        "h13 の列がある: {seen:?}\n{}",
        screen(&a)
    );
}

// ---- #3: 列の無い表では印のグラフがあっても開かない ----

#[test]
fn test_ch_2_no_columns_with_saved_chart_menu_hides_and_p_gives_reason() {
    // [utest -> req~chart.open-refuse~1] #3: 前付けの無いノートだけの表(列が1つも無い)で印に chart = { kind = "bar" } → メニューに「グラフ」を
    // 出さず、P は理由1行で開かない。
    let notes = vec![
        ("alpha.md".to_string(), "# alpha\n本文\n".to_string()),
        ("beta.md".to_string(), "# beta\n".to_string()),
    ];
    let (_t, mut a) = boot("chfx3", &notes, "", "chart = { kind = \"bar\" }");
    assert!(a.src.columns().is_empty(), "前提: 列が無い");
    assert_eq!(a.rows.len(), 2, "前提: 行はある");
    let menu = menu_text(&mut a);
    assert!(
        menu_item(&menu, "グラフ", "P").is_none(),
        "メニューに「グラフ」を出さない:\n{menu}"
    );
    a.message = None;
    ch(&mut a, 'P');
    assert_table(&a);
    reason(&a);
}

// ---- #4: 収まらないときのクリックで表示が跳ねない ----

#[test]
fn test_ch_8_click_on_scrolled_item_does_not_jump() {
    // [utest -> req~chart.bar-scroll~1, req~chart.keys-enter~1, req~chart.keys-click~1, req~chart.keys-click-again~1] #4: 項目 20 個が高さに収まらない(高さ 16)。j で最後まで下りてずらしたあと、見えている一番上の
    // 項目をクリック → その項目が選ばれ、同じ画面の行に残る。同じ所をもう一度クリック → Enter と同じく、その値の
    // 行だけの表。
    let list: Vec<(String, String)> = (0..20)
        .map(|i| (format!("n{i:02}"), format!("cat: v{i:02}")))
        .collect();
    let (_t, mut a) = boot_full(
        "chfx4",
        &md(&refs(&list)),
        "",
        "chart = { x = \"cat\" }",
        ColorMode::None,
        (120, 16),
    );
    let c = open(&mut a);
    assert!(c.bars().len() < 20, "前提: 全部は入らない: {}", c.raw);
    for _ in 0..19 {
        ch(&mut a, 'j');
    }
    let c = chart(&a);
    assert_eq!(c.selected(), ["v19"], "前提: 最後まで下りた: {}", c.raw);
    let top = c.bars()[0].clone();
    assert_ne!(top.name, "v00", "前提: ずらしている: {}", c.raw);
    let (x, y) = top.at;
    a.click(x, y);
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(
        c.selected(),
        std::slice::from_ref(&top.name),
        "クリックで選ぶ: {}",
        c.raw
    );
    let now = c.bar(&top.name);
    assert_eq!(
        now.at.1, y,
        "選んだ項目 {} は同じ画面の行に残る(表示が跳ねない): {}",
        top.name, c.raw
    );
    a.click(x, y);
    assert_table(&a);
    assert_eq!(
        row_names(&a),
        [format!("n{}", &top.name[1..])],
        "同じ所のもう一度のクリックは Enter と同じ"
    );
}

// ---- #5: 単位の知らせと極端な日付の範囲 ----

#[test]
fn test_ch_8_plus_message_matches_actual_unit() {
    // [utest -> req~chart.keys-unit~1, req~chart.keys-kind-unfit~1, req~chart.keys-enter~1] #5: 50 年に広がる日付(1950-01-01 と 2000-01-01)の棒 → 月の単位。+ で細かくしても区切りが多すぎて数える
    // 側で月に戻るとき、知らせ(メッセージ行)が「単位: 週」のように実際と違う単位を言わない。知らせが単位を言うなら、
    // 題の単位と同じ。
    let notes = md(&[("alpha", "due: 1950-01-01"), ("beta", "due: 2000-01-01")]);
    let (_t, mut a) = boot("chfx5unit", &notes, TYPES, "chart = { x = \"due\" }");
    let c = open(&mut a);
    let before = title_unit(&c).unwrap_or_else(|| panic!("題に単位: {}", c.raw));
    a.message = None;
    ch(&mut a, '+');
    assert_chart(&a);
    let c = chart(&a);
    let after = title_unit(&c).unwrap_or_else(|| panic!("題に単位: {}", c.raw));
    let m = reason(&a);
    match m.strip_prefix("単位: ") {
        Some(u) => assert_eq!(
            u.trim(),
            after,
            "知らせの単位と題の単位(実際の単位)が合う: {m:?}\n{}",
            c.raw
        ),
        None => assert_eq!(
            after, before,
            "細かくできない理由なら単位は変わらない: {m:?}"
        ),
    }
}

#[test]
fn test_ch_3_extreme_date_range_shows_reason_in_body() {
    // [utest -> req~chart.buckets-date~1, req~chart.buckets-x-skip~1, req~chart.buckets-agg~1, req~chart.keys-kind-unfit~1, req~chart.keys-enter~1] #5: 0001-01-01 と 9999-12-31 の2行の日付の列 → グラフにせず、体に理由1行(棒は描かない)。開いても
    // パニックせず、Tab で種類を回しても画面が描ける。
    let notes = md(&[
        ("alpha", "status: todo\ndue: 0001-01-01"),
        ("beta", "status: done\ndue: 9999-12-31"),
    ]);
    let (_t, mut a) = boot("chfx5wide", &notes, TYPES, "chart = { x = \"due\" }");
    let c = open(&mut a);
    assert!(c.bars().is_empty(), "棒は描かない: {}", c.raw);
    let filled: Vec<String> = c
        .rows
        .iter()
        .map(|r| r.text.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    assert!(
        !filled.is_empty() && filled.len() <= 2,
        "体は理由1行(と行の数の行まで): {filled:?}\n{}",
        c.raw
    );
    for _ in 0..5 {
        press(&mut a, KeyCode::Tab);
        assert_chart(&a);
        let _ = screen(&a);
    }
}

// ---- #6: 理由の文 ----

#[test]
fn test_ch_7_enter_on_other_column_gives_other_reason() {
    // [utest -> req~chart.heatmap-enter~1] #6: ヒートマップの「その他」の列(横が 12 を超える)で Enter → 表に戻らず、「その他」と書かれた理由。
    let mut list: Vec<(String, String)> = Vec::new();
    for i in 1..=14 {
        list.push((format!("n-{i:02}"), format!("status: todo\ntag: t{i:02}")));
    }
    let (_t, mut a) = boot(
        "chfx6other",
        &md(&refs(&list)),
        "",
        "chart = { kind = \"heatmap\", x = \"status\", by = \"tag\" }",
    );
    open(&mut a);
    for _ in 0..20 {
        ch(&mut a, 'l');
    }
    let d = chart(&a).desc();
    assert!(
        d.contains("× その他"),
        "前提: 「その他」の列を選んだ: {d:?}"
    );
    a.message = None;
    press(&mut a, KeyCode::Enter);
    assert_chart(&a);
    let m = reason(&a);
    assert!(m.contains("その他"), "理由は「その他」を言う: {m:?}");
}

#[test]
fn test_ch_5_line_marker_without_axis_column_says_need_axis() {
    // [utest -> req~chart.line-points~1] #6: 印の kind = "line" で日付と数の列が無い表 → 体の理由は「日付か数の列が無い」(ChartNeedAxis)。
    let notes = md(&[("alpha", "status: todo"), ("beta", "status: done")]);
    let (_t, mut a) = boot("chfx6axis", &notes, "", "chart = { kind = \"line\" }");
    ch(&mut a, 'P');
    assert_eq!(a.mode, Mode::Chart, "{:?}", a.message);
    let s = screen(&a);
    let lines: Vec<&str> = s.lines().collect();
    let above = lines[..lines.len() - 1].join("\n");
    assert!(
        above.contains(Msg::ChartNeedAxis.text()),
        "体の理由に「{}」:\n{s}",
        Msg::ChartNeedAxis.text()
    );
}

// ---- #7: とても大きい数の区間 ----

#[test]
fn test_ch_3_huge_numbers_every_row_falls_in_a_bin() {
    // [utest -> req~chart.buckets-date~1, req~chart.buckets-number~1, req~chart.buckets-x-skip~1, req~chart.buckets-y-skip~1] #7: 数の列に -1e308・0・1e308 の行 → どの行もどれかの区間に入り、区間の棒の件数の合計が数えた行の数(3)。
    let notes = md(&[
        ("alpha", "n: -1e308"),
        ("beta", "n: 0"),
        ("gamma", "n: 1e308"),
    ]);
    let (_t, mut a) = boot("chfx7", &notes, TYPES, "chart = { x = \"n\" }");
    let c = open(&mut a);
    let (n, skipped) = c.counts();
    assert_eq!((n, skipped), (3, None), "前提: 3行を数える: {}", c.raw);
    let bars = c.bars();
    assert!(!bars.is_empty() && bars.len() <= 10, "{}", c.raw);
    let sum: f64 = bars
        .iter()
        .map(|b| {
            b.value
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("件数が数でない: {:?}\n{}", b.value, c.raw))
        })
        .sum();
    assert_eq!(sum, 3.0, "区間の件数の合計が行の数: {}", c.raw);
}

// ---- #8: 長い値の文字でも枠に収まる ----

#[test]
fn test_ch_4_long_values_stay_inside_frame_every_kind() {
    // [utest -> req~chart.bar-percent~1] #8: 合計が 50 桁のようにとても長い値(数の書き方で 50 字ほど)→ 幅 40 で、棒・積み上げ棒・折れ線・散布図・
    // ヒートマップのどれでも、枠のどの行も右の縁が同じ桁(枠を越えない)。
    let notes = md(&[
        (
            "alpha",
            "status: todo\nassignee: alice\ndue: 2026-10-01\nn: 12345678901234567890123456789012345678901234567890",
        ),
        ("beta", "status: done\nassignee: bob\ndue: 2026-10-20\nn: 1"),
        (
            "gamma",
            "status: todo\nassignee: bob\ndue: 2026-11-02\nn: 98765432109876543210987654321098765432109876543210",
        ),
    ]);
    let (_t, mut a) = boot_full(
        "chfx8",
        &notes,
        TYPES,
        "chart = { x = \"status\", y = \"n\", agg = \"sum\" }",
        ColorMode::None,
        (40, 24),
    );
    open(&mut a);
    for _ in 0..5 {
        let k = chart(&a).kind();
        assert_inside_frame(&a, &k);
        press(&mut a, KeyCode::Tab);
        assert_chart(&a);
    }
}

// ---- #9: 種類を切り替えて推し量ったら推定 ----

#[test]
fn test_ch_2_tab_inferring_breakdown_marks_title_inferred() {
    // [utest -> req~chart.open-infer~1, req~chart.open-title-inferred~1] #9: 全部の鍵を書いた印のグラフ(kind = "bar", x = "status")は題に推定が無い。Tab で積み上げ棒へ移り、
    // 内訳(assignee)を推し量ったら、題に「推定」。
    let notes = md(&[
        ("alpha", "status: todo\nassignee: alice"),
        ("beta", "status: done\nassignee: bob"),
    ]);
    let (_t, mut a) = boot(
        "chfx9",
        &notes,
        "",
        "chart = { kind = \"bar\", x = \"status\" }",
    );
    let c = open(&mut a);
    assert!(
        !c.inferred(),
        "前提: 全部の鍵を書いたので推定でない: {}",
        c.raw
    );
    press(&mut a, KeyCode::Tab);
    let c = chart(&a);
    assert_eq!(c.kind(), "積み上げ棒", "{}", c.raw);
    assert_eq!(c.by().as_deref(), Some("assignee"), "{}", c.raw);
    assert!(c.inferred(), "内訳を推し量ったので題に推定: {}", c.raw);
}

#[test]
fn test_ch_6_scatter_point_chars_still_distinct_with_date_y() {
    // [utest -> req~chart.scatter-series-glyph~1, req~cell-view.not-color-only~1] #1 の続き: 縦が日付の散布図でも、内訳(status)ごとの点の字で見分けられる(todo 2点・done 1点)。
    let notes = md(&[
        ("alpha", "status: todo\nestimate: 1\ndue: 2026-10-03"),
        ("beta", "status: done\nestimate: 2\ndue: 2026-10-10"),
        ("gamma", "status: todo\nestimate: 3\ndue: 2026-10-01"),
    ]);
    let (_t, mut a) = boot(
        "chfx1by",
        &notes,
        TYPES,
        "chart = { kind = \"scatter\", x = \"estimate\", y = \"due\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert_eq!(c.points(POINTS[0]), 2, "{}", c.raw);
    assert_eq!(c.points(POINTS[1]), 1, "{}", c.raw);
}
