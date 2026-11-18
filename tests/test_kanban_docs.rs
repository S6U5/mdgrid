//! カンバンの文書・スターター・場面(KB-6)。読むのはリポの実物で、fixture は作らず、行番号や字の位置は決め打ちしない。
//! 記録: specs/_history/decidespec/_changes/2026-11-02-kanban.md(タスク 5)。実装の前に書いた。
//! 節の切り方は tests/test_gantt_docs.rs と同じ(見出しの深さで区切り、コードの囲みの中は見ない)。

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
            .take_while(|l| !heading_of(l).is_some_and(|(lv, _)| lv <= level))
            .map(|l| l.to_string())
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
/// 大文字小文字は問わない。`カンバン` は `カンバン`・`カンバン: …` に当たり、`カンバンの設定` には当たらない。
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
fn test_kb_6_views_page_has_kanban_section_with_image_and_link() {
    // [utest -> req~kanban.docs~1] docs/manual/ja・en の views.md に「カンバン」/「Kanban」の見出しの節があり、その節に場面の画像
    // images/kanban.svg と、くわしい節へのリンクがある(SR-49)。
    for (lang, word) in [("ja", "カンバン"), ("en", "Kanban")] {
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
                links
                    .iter()
                    .any(|(img, t)| *img && t == "images/kanban.svg"),
                "{} の節「{}」に画像 images/kanban.svg が無い: {links:?}",
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
fn test_kb_6_features_page_has_card_moving_item() {
    // [utest -> req~kanban.docs~1] できることの一覧(docs/manual/ja・en の features.md)に、札を動かして状態を直す項がある: `##` の節で、
    // 見出しがカンバン(en は kanban)を名指すか、札(en は card)を名指して本文にカンバンの語があり、
    // 場面の画像(images/*.svg)とくわしい節へのリンクを持つ(SR-51)。
    for (lang, word, card) in [("ja", "カンバン", "札"), ("en", "kanban", "card")] {
        let (p, text) = read(&format!("docs/manual/{lang}/features.md"));
        let secs = sections(&text);
        let hit: Vec<&Section> = secs
            .iter()
            .filter(|s| s.level == 2)
            .filter(|s| {
                let h = s.heading.to_lowercase();
                let body = s.body.join("\n").to_lowercase();
                h.contains(word) || (h.contains(card) && body.contains(word))
            })
            .collect();
        assert!(
            !hit.is_empty(),
            "{} にカンバン(札を動かして状態を直す)の項が無い: {:?}",
            rel(&p),
            secs.iter().map(|s| s.heading.as_str()).collect::<Vec<_>>()
        );
        let ok = hit.iter().any(|s| {
            let links: Vec<(bool, String)> = s.body.iter().flat_map(|l| line_links(l)).collect();
            links
                .iter()
                .any(|(img, t)| *img && t.starts_with("images/") && t.ends_with(".svg"))
                && links.iter().any(|(img, t)| !img && is_detail_link(t))
        });
        assert!(
            ok,
            "{} のカンバンの項({:?})に、場面の画像とくわしい節へのリンクが無い",
            rel(&p),
            hit.iter().map(|s| s.heading.as_str()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_kb_6_keys_doc_has_kanban_section_and_b_in_table() {
    // [utest -> req~kanban.docs~1] docs/keys.md・keys.ja.md に `## \`kanban\`` の節があり、札・レーンを選ぶキー、札を動かすキー
    // (H → move_card_left、L → move_card_right)、表に戻るキー、取り消し・やり直し・保存のキーの行がある。
    // 表の節に `B` → kanban_view の行がある(SR-22)。
    for file in ["docs/keys.md", "docs/keys.ja.md"] {
        let (p, text) = read(file);
        let kanban = backtick_section(&text, "kanban")
            .unwrap_or_else(|| panic!("{} に `## `kanban`` の節が無い", rel(&p)));
        let rows = key_rows(&kanban);
        for key in [
            "Enter", "Esc", "q", "B", "j", "k", "h", "l", "H", "L", "u", "U", "Ctrl+s",
        ] {
            assert!(
                rows.iter().any(|(k, _)| *k == key),
                "{} の kanban の節に `{key}` の行が無い: {rows:?}",
                rel(&p)
            );
        }
        for (key, action) in [("H", "move_card_left"), ("L", "move_card_right")] {
            assert!(
                rows.contains(&(key, action)),
                "{} の kanban の節に `{key}` → {action} の行が無い: {rows:?}",
                rel(&p)
            );
        }
        let table = backtick_section(&text, "table").unwrap();
        assert!(
            key_rows(&table).contains(&("B", "kanban_view")),
            "{} の table の節に `B` → kanban_view の行が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_kb_6_config_doc_explains_kanban_and_inferred_names() {
    // [utest -> req~kanban.docs~1] 設定の文書(docs/config.ja.md・config.md)に、印の [[table]] の kanban = { column = "<列>" } の説明と、
    // 設定が無いときに推定に使う列の名前(status・state・stage・状態・ステータス・段階)がある。
    for file in ["docs/config.ja.md", "docs/config.md"] {
        let (p, text) = read(file);
        let lines = prose_lines(&text);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("kanban = {") && l.contains("column")),
            "{} に kanban = {{ column = … }} の説明が無い",
            rel(&p)
        );
        let names = ["status", "state", "stage", "状態", "ステータス", "段階"];
        assert!(
            lines
                .iter()
                .any(|l| l.contains("kanban") && names.iter().all(|n| l.contains(n))),
            "{} に、kanban の推定に使う列の名前({names:?})を並べた説明が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_kb_6_tasks_starters_and_example_have_kanban_setting() {
    // [utest -> req~kanban.docs~1] 組み込みの tasks のスターターの印(ledger・ledger.ja)と examples/tasks の印の [[table]] に
    // kanban = { column = "status" } がある。
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
        assert!(
            tables.iter().any(|t| t
                .get("kanban")
                .and_then(|k| k.as_table())
                .and_then(|k| k.get("column"))
                .and_then(|c| c.as_str())
                == Some("status")),
            "{} の [[table]] に kanban = {{ column = \"status\" }} が無い",
            rel(&p)
        );
    }
}

#[test]
fn test_kb_6_scenario_kanban_presses_b() {
    // [utest -> req~kanban.docs~1] docs/manual-scenarios.toml に id = "kanban" の場面があり、keys に "B" があり、題と説明がある。
    let (p, text) = read("docs/manual-scenarios.toml");
    let table: toml::Table = text.parse().unwrap();
    let shots = table["shot"].as_array().unwrap();
    let kanban = shots
        .iter()
        .find(|s| s.get("id").and_then(|v| v.as_str()) == Some("kanban"))
        .unwrap_or_else(|| panic!("{} に id = \"kanban\" の場面が無い", rel(&p)));
    let keys: Vec<&str> = kanban
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|k| k.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&"B"),
        "{} の場面 kanban の keys に \"B\" が無い: {keys:?}",
        rel(&p)
    );
    for key in ["title", "text"] {
        assert!(
            kanban.get(key).is_some(),
            "{} の場面 kanban に {key} が無い",
            rel(&p)
        );
    }
}
