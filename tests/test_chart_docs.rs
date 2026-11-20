//! グラフの文書・スターター・場面・録画の台本(CH-9)。読むのはリポの実物で、fixture は作らず、行番号や字の位置は
//! 決め打ちしない。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。
//! 節の切り方は tests/test_kanban_docs.rs と同じ(見出しの深さで区切り、コードの囲みの中は見ない)。

use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

fn read(relpath: &str) -> (PathBuf, String) {
    let p = root().join(relpath);
    let text =
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} が読めない({e})", rel(&p)));
    (p, text)
}

/// コードの囲み(```)の外の行。
fn prose_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
            continue;
        }
        if !fence {
            out.push(line);
        }
    }
    out
}

/// 行が見出し(`#` の並びと空白で始まる)なら、(段の深さ, 見出しの文)。
fn heading_of(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &line[level..];
    if !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }
    Some((level, rest.trim()))
}

/// 見出しごとの節。本文は、次の同じ深さか浅い見出しの手前まで。
struct Section {
    level: usize,
    heading: String,
    body: Vec<String>,
}

fn sections(text: &str) -> Vec<Section> {
    let lines = prose_lines(text);
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some((level, heading)) = heading_of(line) else {
            continue;
        };
        let body: Vec<String> = lines[i + 1..]
            .iter()
            .take_while(|l| heading_of(l).is_none_or(|(lv, _)| lv > level))
            .map(std::string::ToString::to_string)
            .collect();
        out.push(Section {
            level,
            heading: heading.to_string(),
            body,
        });
    }
    out
}

/// 見出しの文から、半角と全角の括弧 `(…)` の中と、mdBook の `{#id}` を除いた残り。
fn heading_core(heading: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in heading.chars() {
        match c {
            '(' | '\u{FF08}' => depth += 1,
            ')' | '\u{FF09}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    if let Some(i) = out.find("{#") {
        out.truncate(i);
    }
    out.trim().to_string()
}

/// 見出しの文(括弧を除いた残り)が語そのものか、語のあとに区切り(英数でも仮名・漢字でもない字)が続く形か。
/// 大文字小文字は問わない。`グラフ` は `グラフ`・`グラフ: …` に当たり、`グラフの設定` には当たらない。
fn names_view(core: &str, word: &str) -> bool {
    let core = core.to_lowercase();
    let word = word.to_lowercase();
    match core.strip_prefix(word.as_str()) {
        None => false,
        Some(rest) => rest.chars().next().is_none_or(|c| !c.is_alphanumeric()),
    }
}

/// 1行の中の `[…](X)` の X を、画像(`![…](X)`)かどうかと一緒に。`X "題"` の題は落とす。
fn line_links(line: &str) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(i) = rest.find("](") {
        let before = &rest[..i];
        let after = &rest[i + 2..];
        let Some(j) = after.find(')') else { break };
        let is_image = before
            .rfind('[')
            .is_some_and(|k| before[..k].ends_with('!'));
        let target = after[..j].split_whitespace().next().unwrap_or("");
        out.push((is_image, target.to_string()));
        rest = &after[j..];
    }
    out
}

/// くわしい節へのリンクか: `http` で始まらず、`#` だけでなく、行き先が `index.md`・他言語(`../ja/`・`../en/`)・
/// 画像(`images/`)でないもの。
fn is_detail_link(target: &str) -> bool {
    if target.starts_with("http") || target.starts_with('#') {
        return false;
    }
    let path = target.split(['#', '?']).next().unwrap_or("");
    !path.is_empty()
        && path != "index.md"
        && !path.starts_with("../ja/")
        && !path.starts_with("../en/")
        && !path.starts_with("images/")
}

/// `## \`name\`` の節(次の `##` の手前まで)の行。
fn backtick_section<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let want = format!("## `{name}`");
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| l.trim_end() == want)?;
    Some(
        lines[start + 1..]
            .iter()
            .take_while(|l| !l.starts_with("## "))
            .copied()
            .collect(),
    )
}

/// キーの表の行(`| \`キー\` | \`動作\` | 説明 |`)を (キー, 動作) に。
fn key_rows<'a>(lines: &[&'a str]) -> Vec<(&'a str, &'a str)> {
    lines
        .iter()
        .filter(|l| l.starts_with('|'))
        .filter_map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            let key = cells.get(1)?.strip_prefix('`')?.strip_suffix('`')?;
            let action = cells.get(2)?.strip_prefix('`')?.strip_suffix('`')?;
            Some((key, action))
        })
        .collect()
}

#[test]
fn test_ch_9_views_page_has_chart_section_with_image_and_link() {
    // [utest -> req~chart.docs~1] 見せ方の一覧(SR-49): docs/manual/ja・en の views.md に「グラフ」/「Chart」の見出しの節があり、その節に
    // 場面の画像 images/chart.svg と、くわしい節へのリンクがある。
    for (lang, word) in [("ja", "グラフ"), ("en", "Chart")] {
        let (p, text) = read(&format!("docs/manual/{lang}/views.md"));
        let secs = sections(&text);
        let hit: Vec<&Section> = secs
            .iter()
            .filter(|s| names_view(&heading_core(&s.heading), word))
            .collect();
        assert!(
            !hit.is_empty(),
            "{} に「{word}」の見出しの節が無い: {:?}",
            rel(&p),
            secs.iter().map(|s| s.heading.as_str()).collect::<Vec<_>>()
        );
        for s in hit {
            let links: Vec<(bool, String)> = s.body.iter().flat_map(|l| line_links(l)).collect();
            assert!(
                links.iter().any(|(img, t)| *img && t == "images/chart.svg"),
                "{} の節「{}」に画像 images/chart.svg が無い: {links:?}",
                rel(&p),
                s.heading
            );
            assert!(
                links.iter().any(|(img, t)| !img && is_detail_link(t)),
                "{} の節「{}」にくわしい節へのリンクが無い: {links:?}",
                rel(&p),
                s.heading
            );
        }
    }
}

#[test]
fn test_ch_9_features_page_has_chart_item() {
    // [utest -> req~chart.docs~1] できることの一覧(SR-51): docs/manual/ja・en の features.md に、見出しがグラフ(en は chart)を名指す
    // `##` の項があり、場面の画像(images/*.svg)とくわしい節へのリンクを持つ。
    for (lang, word) in [("ja", "グラフ"), ("en", "chart")] {
        let (p, text) = read(&format!("docs/manual/{lang}/features.md"));
        let secs = sections(&text);
        let hit: Vec<&Section> = secs
            .iter()
            .filter(|s| s.level == 2 && s.heading.to_lowercase().contains(word))
            .collect();
        assert!(
            !hit.is_empty(),
            "{} にグラフの項が無い: {:?}",
            rel(&p),
            secs.iter().map(|s| s.heading.as_str()).collect::<Vec<_>>()
        );
        let ok = hit.iter().any(|s| {
            let links: Vec<(bool, String)> = s.body.iter().flat_map(|l| line_links(l)).collect();
            links.iter().any(|(img, t)| {
                *img && t.starts_with("images/")
                    && std::path::Path::new(t)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
            }) && links.iter().any(|(img, t)| !img && is_detail_link(t))
        });
        assert!(
            ok,
            "{} のグラフの項({:?})に、場面の画像とくわしい節へのリンクが無い",
            rel(&p),
            hit.iter().map(|s| s.heading.as_str()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_ch_9_keys_doc_has_chart_section_and_p_in_table() {
    // [utest -> req~chart.docs~1] キーの文書(SR-22): docs/keys.md・keys.ja.md に `## \`chart\`` の節があり、種類・列・値・集め方・内訳・
    // 単位・印のグラフ・累積・割合を切り替えるキーと、選ぶキー・Enter・表に戻るキーの行がある。表の節に
    // `P` → chart_view、画面の型のメニューの節に `P`。
    for file in ["docs/keys.md", "docs/keys.ja.md"] {
        let (p, text) = read(file);
        let chart = backtick_section(&text, "chart")
            .unwrap_or_else(|| panic!("{} に `## `chart`` の節が無い", rel(&p)));
        let rows = key_rows(&chart);
        for key in [
            "Enter",
            "Esc",
            "q",
            "P",
            "j",
            "k",
            "h",
            "l",
            "Tab",
            "Shift+Tab",
            "x",
            "y",
            "a",
            "b",
            "+",
            "-",
            "[",
            "]",
            "c",
            "%",
        ] {
            assert!(
                rows.iter().any(|(k, _)| *k == key),
                "{} の chart の節に `{key}` の行が無い: {rows:?}",
                rel(&p)
            );
        }
        for (key, action) in [
            ("Tab", "chart_kind_next"),
            ("Shift+Tab", "chart_kind_prev"),
            ("x", "chart_x"),
            ("y", "chart_y"),
            ("a", "chart_agg"),
            ("b", "chart_by"),
            ("+", "chart_finer"),
            ("-", "chart_coarser"),
            ("[", "chart_prev_saved"),
            ("]", "chart_next_saved"),
            ("c", "chart_cumulative"),
            ("%", "chart_percent"),
        ] {
            assert!(
                rows.contains(&(key, action)),
                "{} の chart の節に `{key}` → {action} の行が無い: {rows:?}",
                rel(&p)
            );
        }
        let table = backtick_section(&text, "table").unwrap();
        assert!(
            key_rows(&table).contains(&("P", "chart_view")),
            "{} の table の節に `P` → chart_view の行が無い",
            rel(&p)
        );
        let menu: String = backtick_section(&text, "screen_menu")
            .unwrap_or_else(|| panic!("{} に screen_menu の節が無い", rel(&p)))
            .iter()
            .filter(|l| !l.starts_with('|'))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            menu.contains("`P`"),
            "{} の screen_menu の節に `P` が無い:\n{menu}",
            rel(&p)
        );
    }
}

#[test]
fn test_ch_9_config_doc_explains_chart_keys_and_values() {
    // [utest -> req~chart.docs~1] 設定の文書(docs/config.ja.md・config.md)に、印の [[table]] の chart = { … } の書き方(1つと配列)と、
    // 鍵(kind・x・y・agg・by・unit・cumulative・name)と、種類・集め方・単位の値の名前がある。
    for file in ["docs/config.ja.md", "docs/config.md"] {
        let (p, text) = read(file);
        let lines = prose_lines(&text);
        assert!(
            lines.iter().any(|l| l.contains("chart = {")),
            "{} に chart = {{ … }} の説明が無い",
            rel(&p)
        );
        assert!(
            lines.iter().any(|l| l.contains("chart = [")),
            "{} に複数のグラフ(chart = [ … ])の説明が無い",
            rel(&p)
        );
        let about: String = lines
            .iter()
            .filter(|l| l.contains("chart"))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        for word in [
            "kind",
            "agg",
            "by",
            "unit",
            "cumulative",
            "name",
            "bar",
            "stacked",
            "line",
            "scatter",
            "heatmap",
            "count",
            "sum",
            "avg",
            "min",
            "max",
            "day",
            "week",
            "month",
        ] {
            assert!(
                about.contains(word),
                "{} の chart の説明に {word} が無い",
                rel(&p)
            );
        }
    }
}

/// [[table]] の chart を、1つでも配列でもグラフの並びにする。
fn charts_of(t: &toml::Value) -> Vec<toml::Table> {
    match t.get("chart") {
        Some(toml::Value::Table(c)) => vec![c.clone()],
        Some(toml::Value::Array(a)) => a.iter().filter_map(|v| v.as_table().cloned()).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn test_ch_9_tasks_starters_and_example_have_chart_setting() {
    // [utest -> req~chart.docs~1] 組み込みの tasks のスターターの印(ledger・ledger.ja)と examples/tasks の印の [[table]] に chart がある:
    // 状態(status)ごとの棒と、期限(due)の折れ線。書いた鍵と値は知っているものだけ(印の読みで外れない)。
    let keys = ["name", "kind", "x", "y", "agg", "by", "unit", "cumulative"];
    for file in [
        "starters/tasks/ledger/.mdgrid/workspace.toml",
        "starters/tasks/ledger.ja/.mdgrid/workspace.toml",
        "examples/tasks/.mdgrid/workspace.toml",
    ] {
        let (p, text) = read(file);
        let parsed: toml::Table = text
            .parse()
            .unwrap_or_else(|e| panic!("{} が TOML として読めない: {e}", rel(&p)));
        let tables = parsed["table"].as_array().unwrap();
        let charts: Vec<toml::Table> = tables.iter().flat_map(charts_of).collect();
        assert!(
            !charts.is_empty(),
            "{} の [[table]] に chart が無い",
            rel(&p)
        );
        for c in &charts {
            for k in c.keys() {
                assert!(keys.contains(&k.as_str()), "{}: 知らない鍵 {k}", rel(&p));
            }
            if let Some(k) = c.get("kind").and_then(|v| v.as_str()) {
                assert!(
                    mdgrid::chart::ChartKind::parse(k).is_some(),
                    "{}: 知らない種類 {k}",
                    rel(&p)
                );
            }
            if let Some(a) = c.get("agg").and_then(|v| v.as_str()) {
                assert!(
                    mdgrid::chart::Agg::parse(a).is_some(),
                    "{}: 知らない集め方 {a}",
                    rel(&p)
                );
            }
        }
        let s = |c: &toml::Table, k: &str| c.get(k).and_then(|v| v.as_str()).map(str::to_string);
        assert!(
            charts.iter().any(|c| s(c, "x").as_deref() == Some("status")
                && matches!(s(c, "kind").as_deref(), None | Some("bar"))),
            "{} に状態ごとの棒(x = \"status\")が無い",
            rel(&p)
        );
        assert!(
            charts
                .iter()
                .any(|c| s(c, "x").as_deref() == Some("due")
                    && s(c, "kind").as_deref() == Some("line")),
            "{} に期限の折れ線(kind = \"line\", x = \"due\")が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_ch_9_scenario_chart_presses_p() {
    // [utest -> req~chart.docs~1] docs/manual-scenarios.toml に id = "chart" の場面があり、keys に "P" があり、題と説明がある。
    let (p, text) = read("docs/manual-scenarios.toml");
    let table: toml::Table = text.parse().unwrap();
    let shots = table["shot"].as_array().unwrap();
    let chart = shots
        .iter()
        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some("chart"))
        .unwrap_or_else(|| panic!("{} に id = \"chart\" の場面が無い", rel(&p)));
    let keys: Vec<&str> = chart
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|k| k.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&"P"),
        "{} の場面 chart の keys に \"P\" が無い: {keys:?}",
        rel(&p)
    );
    for key in ["title", "text"] {
        assert!(
            chart.get(key).is_some(),
            "{} の場面 chart に {key} が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_ch_9_demo_tape_records_chart() {
    // [utest -> req~chart.docs~1] 録画の台本 demos/chart.tape があり、demos/out/demo-chart.gif に出し、共通の設定(setup.tape)を読み、
    // P でグラフを開く。録画の一覧(demos/README.md)とサイトの録画の一覧(scripts/pages.py)にも載る。
    let (p, tape) = read("demos/chart.tape");
    let lines: Vec<&str> = tape.lines().map(str::trim).collect();
    assert!(
        lines.contains(&"Output demos/out/demo-chart.gif"),
        "{} の Output が demos/out/demo-chart.gif でない",
        rel(&p)
    );
    assert!(
        lines.contains(&"Source demos/setup.tape"),
        "{} が demos/setup.tape を読まない",
        rel(&p)
    );
    assert!(
        lines.iter().any(|l| l.starts_with("Type \"P\"")),
        "{} に P(グラフを開く)を打つ行が無い",
        rel(&p)
    );
    let (p, readme) = read("demos/README.md");
    assert!(
        readme
            .lines()
            .any(|l| l.contains("`chart.tape`") && l.contains("demo-chart.gif")),
        "{} の一覧に chart.tape → demo-chart.gif の行が無い",
        rel(&p)
    );
    let (p, pages) = read("scripts/pages.py");
    assert!(
        pages.contains("\"demo-chart.gif\""),
        "{} の録画の一覧に demo-chart.gif が無い",
        rel(&p)
    );
}
