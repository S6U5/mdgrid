//! 見せ方の一覧のページ(SR-49)と読む順・文の決まり(SR-50)の受け入れを強める試験。
//! tests/test_views_page.rs(錠あり)はページ全体で画像と見出しとリンクの数を見るので、ここでは
//! 節ごと(8つの見せ方それぞれに画像とくわしい節へのリンクがあるか)、見出しの一致の厳しさ
//! (`表` に `表示` が通らない)、リンクの錨(`#…`)が行き先の見出しにあるか、台本(docs/manual-scenarios.toml)の
//! 文言の大げさな語、の4点だけを見る。読むのはリポの実物で、fixture は作らず、行番号や字の位置は決め打ちしない。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// SR-49 の8つの見せ方の語(見出しと突き合わせる)。
const VIEWS: [(&str, [&str; 8]); 2] = [
    (
        "ja",
        [
            "表",
            "ビューのタブ",
            "まとまり",
            "木",
            "関係マップ",
            "値の分布",
            "絞り込みの帯",
            "テーマ",
        ],
    ),
    (
        "en",
        [
            "table",
            "view tabs",
            "groups",
            "tree",
            "relation map",
            "value counts",
            "filter band",
            "themes",
        ],
    ),
];

/// 日本語の大げさな語(docs/writing.md)。部分一致で探す。
const JA_WORDS: [&str; 6] = ["強力", "シームレス", "直感的", "画期的", "最適", "豊富"];

/// 英語の大げさな語(docs/writing.md)。語の頭で区切り、大文字小文字は問わない。
const EN_WORDS: [&str; 9] = [
    "seamless",
    "robust",
    "powerful",
    "intuitive",
    "leverage",
    "empower",
    "delve",
    "crucial",
    "pivotal",
];

/// 見せ方の一覧のページ(無ければ、どのファイルが無いかが分かる文で落ちる)。
fn views_page(lang: &str) -> (PathBuf, String) {
    let p = root().join("docs/manual").join(lang).join("views.md");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("見せ方の一覧のページ {} が無い({e})", rel(&p)));
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

/// 見出しごとの節。本文は、次の同じ深さか浅い見出しの手前まで(`## 表` の節は、その下の `### …` を含む)。
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
/// `表` は `表`・`表: …`・`表 — …` に当たり、`表示`・`木の設定` には当たらない。大文字小文字は問わない。
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

/// 行の並びの中の `![…](images/<id>.svg)` の数。
fn count_scene_images(lines: &[String]) -> usize {
    lines
        .iter()
        .flat_map(|l| line_links(l))
        .filter(|(img, t)| *img && t.starts_with("images/") && t.ends_with(".svg"))
        .count()
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

/// 行の並びの中の、くわしい節へのリンク(画像でないもの)。
fn detail_links(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .flat_map(|l| line_links(l))
        .filter(|(img, t)| !img && is_detail_link(t))
        .map(|(_, t)| t)
        .collect()
}

/// mdBook が見出しに付ける id(utils の id_from_content と normalize_id のとおり): 英数(漢字・かな・カナも
/// alphanumeric)・`-`・`_` は残して ASCII の大文字を小文字に、空白は `-` に、ほかの記号(`・`・`:`・`.`・`—`・
/// バッククォート)は落とす。`{#id}` が書いてあればそれ。
fn heading_id(heading: &str) -> String {
    let h = heading.trim();
    if let Some(i) = h.rfind("{#") {
        if let Some(j) = h[i..].find('}') {
            return h[i + 2..i + j].trim().to_string();
        }
    }
    h.chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                Some(c.to_ascii_lowercase())
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// ページの全部の見出しの id。同じ id が重なれば、mdBook と同じく 2つ目から `-1`・`-2`… を付ける。
fn heading_ids(text: &str) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    for line in prose_lines(text) {
        let Some((_, heading)) = heading_of(line) else {
            continue;
        };
        let id = heading_id(heading);
        let n = counts.entry(id.clone()).or_insert(0);
        out.push(if *n == 0 {
            id.clone()
        } else {
            format!("{id}-{n}")
        });
        *n += 1;
    }
    out
}

/// 英語の語を、語の頭で区切って(前が英字でない)、大文字小文字を問わず探す。
/// 後ろは区切らないので、powerfully・seamlessly・leverages のような活用も拾う。
fn has_en_word(line: &str, word: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(i) = lower[from..].find(word) {
        let at = from + i;
        if at == 0 || !bytes[at - 1].is_ascii_alphabetic() {
            return true;
        }
        // word は ASCII なので at の字は 1 バイト。at + 1 は字の境。
        from = at + 1;
    }
    false
}

/// 台本の値 `title`・`text`・`intro` の文(`{ ja = …, en = … }` なら言語つき、ただの文字列なら言語なし)。
fn scenario_texts(value: &toml::Value) -> Vec<(Option<&'static str>, String)> {
    match value {
        toml::Value::String(s) => vec![(None, s.clone())],
        toml::Value::Table(t) => ["ja", "en"]
            .into_iter()
            .filter_map(|lang| {
                t.get(lang)
                    .and_then(|v| v.as_str())
                    .map(|s| (Some(lang), s.to_string()))
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// 文に大げさな語があれば、その語。言語が分からない文は日英の両方で探す。
fn hype_words(lang: Option<&str>, text: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    if lang != Some("en") {
        out.extend(JA_WORDS.iter().filter(|w| text.contains(**w)).copied());
    }
    if lang != Some("ja") {
        out.extend(EN_WORDS.iter().filter(|w| has_en_word(text, w)).copied());
    }
    out
}

#[test]
fn test_sr_49_views_sections_have_image_and_link() {
    // [SR-49] views.md(ja・en)を見出しで節に区切り、8つの見せ方の節それぞれに、場面の画像
    // (`![…](images/<id>.svg)`)が1枚以上と、くわしい節へのリンク(http でなく、# だけでなく、index.md・
    // 他言語・画像でない行き先)が1本以上ある。ページ全体の数では、1つの節に集まっていても通ってしまう。
    for (lang, words) in VIEWS {
        let (p, text) = views_page(lang);
        let secs = sections(&text);
        for w in words {
            let hit: Vec<&Section> = secs
                .iter()
                .filter(|s| names_view(&heading_core(&s.heading), w))
                .collect();
            assert!(!hit.is_empty(), "{} に「{w}」の見出しの節が無い", rel(&p));
            for s in hit {
                let imgs = count_scene_images(&s.body);
                assert!(
                    imgs >= 1,
                    "{} の節「{}」に場面の画像(images/<id>.svg)が無い",
                    rel(&p),
                    s.heading
                );
                let links = detail_links(&s.body);
                assert!(
                    !links.is_empty(),
                    "{} の節「{}」にくわしい節へのリンクが無い",
                    rel(&p),
                    s.heading
                );
            }
        }
    }
}

#[test]
fn test_sr_49_views_headings_name_each_view_exactly() {
    // [SR-49] 8つの語は、見出しの文(半角か全角の括弧 `(…)` の中を除いた残り)との完全一致か、語のあとに区切りが続く
    // 先頭一致でだけ当たる。`表` は見出し `表`(と `表: …`)だけで、`表示` や `木の設定` のような別の見出しでは
    // 通らない。en は大文字小文字を問わず、`Table` は `Table`、`Tree` は `Tree` だけ。
    assert!(names_view("表", "表") && names_view("表: ノートが行", "表"));
    assert!(!names_view("表示", "表") && !names_view("木の設定", "木"));
    assert!(names_view("Table", "table") && names_view("Tree — parent and child", "tree"));
    assert!(!names_view("Tables", "table") && !names_view("Treemap", "tree"));
    assert_eq!(heading_core("まとまり(グループ)"), "まとまり");
    assert_eq!(heading_core("Groups (grouped rows)"), "Groups");
    for (lang, words) in VIEWS {
        let (p, text) = views_page(lang);
        let cores: Vec<String> = sections(&text)
            .iter()
            .map(|s| heading_core(&s.heading))
            .collect();
        for w in words {
            assert!(
                cores.iter().any(|c| names_view(c, w)),
                "{} に見出しが「{w}」そのもの(括弧を除いて)の節が無い: {cores:?}",
                rel(&p)
            );
        }
    }
}

#[test]
fn test_sr_49_views_anchors_point_at_headings() {
    // [SR-49] views.md の相対リンクに `#錨` が付いているとき、その錨は行き先の .md の見出しを mdBook の id
    // (小文字、空白は `-`、英数と仮名・漢字と `-`・`_` 以外の記号は落とす)に直したものの中にある。
    // 行き先の無い `#錨` だけのリンクは、このページ自身の見出しで見る。
    // id の作り方は、既存のリンク(tasks.md#base-を開く など)が指す見出しと同じ形になることで確かめる。
    assert_eq!(heading_id("探す・絞る・並べる"), "探す絞る並べる");
    assert_eq!(heading_id("Find, filter and sort"), "find-filter-and-sort");
    assert_eq!(heading_id("The relation map"), "the-relation-map");
    assert_eq!(heading_id(".base を開く"), "base-を開く");
    assert_eq!(
        heading_id("ワークスペース: どの表が一緒か"),
        "ワークスペース-どの表が一緒か"
    );
    assert_eq!(
        heading_id("ワークスペース・表・ビューごとの見た目"),
        "ワークスペース表ビューごとの見た目"
    );
    assert_eq!(heading_id("Open a .base file"), "open-a-base-file");
    for lang in ["ja", "en"] {
        let (p, text) = views_page(lang);
        let dir = p.parent().unwrap();
        let mut ids: HashMap<PathBuf, Vec<String>> = HashMap::new();
        let mut checked = 0;
        for line in prose_lines(&text) {
            for (is_image, target) in line_links(line) {
                if is_image || target.starts_with("http") {
                    continue;
                }
                let Some((path, anchor)) = target.split_once('#') else {
                    continue;
                };
                let file = if path.is_empty() {
                    p.clone()
                } else {
                    dir.join(path)
                };
                if file.extension().is_none_or(|x| x != "md") {
                    continue;
                }
                let known = ids.entry(file.clone()).or_insert_with(|| {
                    let t = std::fs::read_to_string(&file).unwrap_or_else(|e| {
                        panic!(
                            "{} のリンク {target} の行き先 {} が読めない({e})",
                            rel(&p),
                            rel(&file)
                        )
                    });
                    heading_ids(&t)
                });
                assert!(
                    known.iter().any(|id| id == anchor),
                    "{} のリンク {target} の錨 #{anchor} が {} の見出しに無い: {known:?}",
                    rel(&p),
                    rel(&file)
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 1,
            "{} に `#錨` 付きのリンクが1本も無い(くわしい節へは節を指す)",
            rel(&p)
        );
    }
}

#[test]
fn test_sr_50_scenarios_no_hype_words() {
    // [SR-50] 画面の一覧の文言のもとになる台本(docs/manual-scenarios.toml)の title・text・intro(ja・en)に、
    // 大げさな語(日本語: 強力・シームレス・直感的・画期的・最適・豊富 / 英語: seamless・robust・powerful・intuitive・
    // leverage・empower・delve・crucial・pivotal。語の頭で区切り、大文字小文字は問わない)が無い。
    let p = root().join("docs/manual-scenarios.toml");
    let text = std::fs::read_to_string(&p).unwrap();
    let table: toml::Table = text.parse().unwrap();
    let mut found = Vec::new();
    let mut seen = 0;
    let mut check = |place: &str, key: &str, value: &toml::Value| {
        for (lang, s) in scenario_texts(value) {
            seen += 1;
            for w in hype_words(lang, &s) {
                found.push(format!("{place} {key}[{}]: {w}: {s}", lang.unwrap_or("-")));
            }
        }
    };
    if let Some(app) = table.get("app").and_then(|a| a.as_table()) {
        for key in ["title", "intro"] {
            if let Some(v) = app.get(key) {
                check("app", key, v);
            }
        }
    }
    for shot in table["shot"].as_array().unwrap() {
        let id = shot.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        for key in ["title", "text"] {
            if let Some(v) = shot.get(key) {
                check(&format!("shot {id}"), key, v);
            }
        }
    }
    assert!(
        seen >= 8,
        "{} の title・text・intro の文が少ない: {seen}",
        rel(&p)
    );
    assert!(
        found.is_empty(),
        "{} の文言に大げさな語が {} 件ある:\n{}",
        rel(&p),
        found.len(),
        found.join("\n")
    );
}
