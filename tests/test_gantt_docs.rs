//! ガントの文書・見本・場面(GA-6)。読むのはリポの実物で、fixture は作らず、行番号や字の位置は決め打ちしない。
//! 節の切り方は tests/test_views_page_more.rs と同じ(見出しの深さで区切り、コードの囲みの中は見ない)。

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
            .take_while(|l| !heading_of(l).is_some_and(|(lv, _)| lv <= level))
            .map(|l| l.to_string())
            .collect();
        out.push(Section {
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
/// 大文字小文字は問わない。`ガント` は `ガント`・`ガント: …` に当たり、`ガントの設定` には当たらない。
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

#[test]
fn test_ga_6_views_page_has_gantt_section_with_image_and_link() {
    // [GA-6] docs/manual/ja・en の views.md に「ガント」/「Gantt」の見出しの節があり、その節に場面の画像
    // images/gantt.svg と、くわしい節へのリンクがある。
    for (lang, word) in [("ja", "ガント"), ("en", "Gantt")] {
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
                links.iter().any(|(img, t)| *img && t == "images/gantt.svg"),
                "{} の節「{}」に画像 images/gantt.svg が無い: {links:?}",
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
fn test_ga_6_scenario_gantt_presses_c() {
    // [GA-6] docs/manual-scenarios.toml に id = "gantt" の場面があり、keys に "C" がある。
    let (p, text) = read("docs/manual-scenarios.toml");
    let table: toml::Table = text.parse().unwrap();
    let shots = table["shot"].as_array().unwrap();
    let gantt = shots
        .iter()
        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some("gantt"))
        .unwrap_or_else(|| panic!("{} に id = \"gantt\" の場面が無い", rel(&p)));
    let keys: Vec<&str> = gantt
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|k| k.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&"C"),
        "{} の場面 gantt の keys に \"C\" が無い: {keys:?}",
        rel(&p)
    );
    for key in ["title", "text"] {
        assert!(
            gantt.get(key).is_some(),
            "{} の場面 gantt に {key} が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_ga_6_tasks_starter_has_gantt_setting_and_start_column() {
    // [GA-6] 組み込みの tasks のスターターの印(ledger・ledger.ja)に gantt = { … } の行と start の型、
    // 見本のノート(samples・samples.ja)に start の列。
    for ledger in ["ledger", "ledger.ja"] {
        let (p, text) = read(&format!("starters/tasks/{ledger}/.mdgrid/workspace.toml"));
        let gantt = text
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("gantt = {"))
            .unwrap_or_else(|| panic!("{} に gantt = {{ … }} の行が無い", rel(&p)));
        assert!(
            gantt.contains("start") && gantt.contains("due"),
            "{}: {gantt}",
            rel(&p)
        );
        assert!(
            text.lines()
                .map(str::trim)
                .any(|l| l.starts_with("start = \"date\"")),
            "{} の [types] に start = \"date\" が無い",
            rel(&p)
        );
        let parsed: toml::Table = text
            .parse()
            .unwrap_or_else(|e| panic!("{} が TOML として読めない: {e}", rel(&p)));
        let tables = parsed["table"].as_array().unwrap();
        assert!(
            tables
                .iter()
                .any(|t| t.get("gantt").is_some_and(|g| g.is_table())),
            "{} の [[table]] に gantt の表が無い",
            rel(&p)
        );
    }
    for samples in ["samples", "samples.ja"] {
        let dir = root().join("starters/tasks").join(samples).join("Tasks");
        let mut with_start = 0;
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", rel(&dir))) {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "md") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            if text.lines().any(|l| l.starts_with("start:")) {
                with_start += 1;
            }
        }
        assert!(
            with_start >= 1,
            "{} の見本のノートに start の列が無い",
            rel(&dir)
        );
    }
}

#[test]
fn test_ga_6_keys_doc_has_gantt_section_and_c_in_table() {
    // [GA-6] docs/keys.md・keys.ja.md に `## \`gantt\`` の節があり、表の節に `C` → gantt_view の行がある。
    for file in ["docs/keys.md", "docs/keys.ja.md"] {
        let (p, text) = read(file);
        let gantt = backtick_section(&text, "gantt")
            .unwrap_or_else(|| panic!("{} に `## `gantt`` の節が無い", rel(&p)));
        let rows: Vec<&str> = gantt
            .iter()
            .filter(|l| l.starts_with('|') && l.contains('`'))
            .copied()
            .collect();
        assert!(
            rows.len() >= 8,
            "{} の gantt の節のキーの表が短い: {rows:?}",
            rel(&p)
        );
        for key in [
            "`Enter`", "`Esc`", "`j`", "`k`", "`h`", "`l`", "`+`", "`-`", "`t`",
        ] {
            assert!(
                rows.iter().any(|r| r.contains(&format!("| {key} |"))),
                "{} の gantt の節に {key} の行が無い",
                rel(&p)
            );
        }
        let table = backtick_section(&text, "table").unwrap();
        assert!(
            table
                .iter()
                .any(|l| l.contains("| `C` |") && l.contains("`gantt_view`")),
            "{} の table の節に `C` → gantt_view の行が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_ga_6_config_doc_explains_gantt() {
    // [GA-6] 設定の文書(docs/config.ja.md・config.md)に、印の [[table]] の gantt = { start, end } の説明がある。
    for file in ["docs/config.ja.md", "docs/config.md"] {
        let (p, text) = read(file);
        let lines = prose_lines(&text);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("gantt = {") && l.contains("start") && l.contains("end")),
            "{} に gantt = {{ start, end }} の説明が無い",
            rel(&p)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("gantt") && l.contains("days")),
            "{} に days の説明が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_ga_6_features_page_mentions_gantt() {
    // [GA-6] できることの一覧(docs/manual/ja・en の features.md)にガントの語がある。
    for (lang, word) in [("ja", "ガント"), ("en", "Gantt")] {
        let (p, text) = read(&format!("docs/manual/{lang}/features.md"));
        assert!(
            prose_lines(&text).iter().any(|l| l.contains(word)),
            "{} に「{word}」が無い",
            rel(&p)
        );
    }
}
