//! 説明書とサイトの「できることの一覧」のページ(SR-51)。
//! docs/manual/<言語>/features.md に、できること(機能)を1項ずつ(本物の画面の画像1枚・何ができてどう始めるかの
//! 説明・くわしい節へのリンク)並べ、サイトでは見せ方の一覧の次の章に置き、入口(scripts/pages.py の「見て選ぶ」)
//! からそのページへ行ける。画像は Pages の workflow が撮る場面(docs/manual-scenarios.toml)の id で指し、
//! README の GIF の駒(gif-*)は使わない。見る形は tests/test_views_page.rs と tests/test_views_page_more.rs
//! (どちらも錠あり)に合わせ、節の区切り・見出しの一致・錨の id・リンクの判定は同じ規則で見る。
//! 読むのはリポの実物で、fixture は作らず、行番号や字の位置は決め打ちしない。

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// SR-51 の 12 の機能の語(見出しと突き合わせる)。ja の「探す/絞る/並べる」は見出しでは「探す・絞る・並べる」。
/// `.base` の `.` は見出しの中でそのまま(手引きの見出し `.base を開く` と同じ形)。
const FEATURES: [(&str, [&str; 12]); 2] = [
    (
        "ja",
        [
            "値をその場で直す",
            "差分を見て保存する",
            "まとめて直す",
            "探す・絞る・並べる",
            "新しいノートを作る",
            "ビューを決めて保存する",
            ".base を開く",
            "リンクで表をつなぐ",
            "スターターから台帳を作る",
            "ほかのコマンドと組む",
            "読むだけで開く",
            "見た目を選ぶ",
        ],
    ),
    (
        "en",
        [
            "edit values in place",
            "review the diff and save",
            "edit many rows at once",
            "find, filter and sort",
            "create a note",
            "shape a view and save it",
            "open a .base file",
            "link tables",
            "create a ledger from a starter",
            "use with other commands",
            "open read-only",
            "choose a look",
        ],
    ),
];

/// できることの一覧のページ(無ければ、どのファイルが無いかが分かる文で落ちる)。
fn features_page(lang: &str) -> (PathBuf, String) {
    let p = root().join("docs/manual").join(lang).join("features.md");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("できることの一覧のページ {} が無い({e})", rel(&p)));
    (p, text)
}

/// 撮る場面の台本(docs/manual-scenarios.toml)の [[shot]] の一覧。
fn shots() -> Vec<toml::Table> {
    let text = std::fs::read_to_string(root().join("docs/manual-scenarios.toml")).unwrap();
    let table: toml::Table = text.parse().unwrap();
    table["shot"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_table().unwrap().clone())
        .collect()
}

fn scene_ids() -> HashSet<String> {
    shots()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_string())
        .collect()
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

/// 見出しごとの節。本文は、次の同じ深さか浅い見出しの手前まで(`## …` の節は、その下の `### …` を含む)。
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
/// `まとめて直す` は `まとめて直す`・`まとめて直す: …` に当たり、`まとめて直すには` には当たらない。大文字小文字は問わない。
fn names_feature(core: &str, word: &str) -> bool {
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

/// ページの中の全部のリンク(コードの囲みの外)。
fn all_links(text: &str) -> Vec<(bool, String)> {
    prose_lines(text).into_iter().flat_map(line_links).collect()
}

/// 行の並びの中の `![…](images/<id>.svg)` の数。
fn count_scene_images(lines: &[String]) -> usize {
    lines
        .iter()
        .flat_map(|l| line_links(l))
        .filter(|(img, t)| *img && t.starts_with("images/") && t.ends_with(".svg"))
        .count()
}

/// くわしい節へのリンクか: `http` で始まらず、`#` だけでなく、行き先が `index.md`・このページ自身(`features.md`)・
/// 他言語(`../ja/`・`../en/`)・画像(`images/`)でないもの。
fn is_detail_link(target: &str) -> bool {
    if target.starts_with("http") || target.starts_with('#') {
        return false;
    }
    let path = target.split(['#', '?']).next().unwrap_or("");
    !path.is_empty()
        && path != "index.md"
        && path != "features.md"
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

/// `## <heading>` の節(次の `## ` まで)にある `- [題](ファイル.md)` のリンク先を、書いてある順に。
/// `#` 以降の断片は落とす。
fn toc_targets(text: &str, heading: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            inside = h.trim() == heading;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(rest) = line.trim_start().strip_prefix("- [") else {
            continue;
        };
        let Some(i) = rest.find("](") else { continue };
        let after = &rest[i + 2..];
        let Some(j) = after.find(')') else { continue };
        let target = after[..j].split('#').next().unwrap_or("").trim();
        out.push(target.to_string());
    }
    out
}

/// `names` が `seq` にすべて在り、その相対の順が守られているか。守られていなければ理由。
fn check_order(seq: &[String], names: &[&str]) -> Result<(), String> {
    let mut last: Option<(usize, &str)> = None;
    for name in names {
        let Some(pos) = seq.iter().position(|s| s == name) else {
            return Err(format!("{name} が無い"));
        };
        if let Some((last_pos, last_name)) = last {
            if pos < last_pos {
                return Err(format!("{name} が {last_name} より前にある"));
            }
        }
        last = Some((pos, name));
    }
    Ok(())
}

/// `start` から、その後の最初の `end` までの切れ端(`end` が無ければ末尾まで)。
fn py_section<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let i = text
        .find(start)
        .unwrap_or_else(|| panic!("scripts/pages.py に {start:?} が無い"));
    let body = &text[i + start.len()..];
    match body.find(end) {
        Some(j) => &body[..j],
        None => body,
    }
}

/// `scripts/pages.py` の `PAGES = [...]` の中の文字列を、書いてある順に(複数行でもよい)。
fn pages_list(py: &str) -> Vec<String> {
    py_section(py, "PAGES = [", "]")
        .split(',')
        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[test]
fn test_sr_51_features_pages_images_are_scenes() {
    // [SR-51] できることの一覧(ja・en)があり、画像はどれも `images/<id>.svg` で、id は撮る場面(台本)の id
    // (README の GIF の駒 gif-* は画面の一覧に出ないので使わない)。12 の機能に1枚ずつで少なくとも 12 枚ある。
    let ids = scene_ids();
    for lang in ["ja", "en"] {
        let (p, text) = features_page(lang);
        let imgs: Vec<String> = all_links(&text)
            .into_iter()
            .filter(|(img, _)| *img)
            .map(|(_, t)| t)
            .collect();
        assert!(
            imgs.len() >= 12,
            "{} の画像が少ない: {} 枚(12 の機能に1枚ずつ要る)",
            rel(&p),
            imgs.len()
        );
        for t in &imgs {
            let name = t
                .strip_prefix("images/")
                .and_then(|n| n.strip_suffix(".svg"))
                .unwrap_or_else(|| panic!("{} の画像 {t} は images/<場面>.svg でない", rel(&p)));
            assert!(
                !name.starts_with("gif-"),
                "{} の画像 {t} は README の GIF の駒(画面の一覧に出ない)",
                rel(&p)
            );
            assert!(
                ids.contains(name),
                "{} の {t} を撮る場面が docs/manual-scenarios.toml に無い",
                rel(&p)
            );
        }
    }
}

#[test]
fn test_sr_51_features_sections_have_image_and_link() {
    // [SR-51] features.md(ja・en)を見出しで節に区切り、12 の機能の節それぞれに、場面の画像
    // (`![…](images/<id>.svg)`)が1枚以上と、くわしい節へのリンク(http でなく、# だけでなく、index.md・
    // features.md 自身・他言語・画像でない行き先)が1本以上ある。ページ全体の数では、1つの節に集まっていても
    // 通ってしまうので、節ごとに見る。
    for (lang, words) in FEATURES {
        let (p, text) = features_page(lang);
        let secs = sections(&text);
        for w in words {
            let hit: Vec<&Section> = secs
                .iter()
                .filter(|s| names_feature(&heading_core(&s.heading), w))
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
fn test_sr_51_features_headings_name_each_feature_exactly() {
    // [SR-51] 12 の語は、見出しの文(半角か全角の括弧 `(…)` の中を除いた残り)との完全一致か、語のあとに区切りが
    // 続く先頭一致でだけ当たる(tests/test_views_page_more.rs の names_view と同じ規則)。`まとめて直す` は
    // 見出し `まとめて直す`(と `まとめて直す: …`)だけで、`まとめて直すには` では通らない。en は大文字小文字を
    // 問わず、`Link tables` は `Link tables` と `Link tables (relations)` だけで、`Linked tables` では通らない。
    assert!(names_feature("まとめて直す", "まとめて直す"));
    assert!(names_feature("まとめて直す: 複数の行", "まとめて直す"));
    assert!(!names_feature("まとめて直すには", "まとめて直す"));
    assert!(names_feature("探す・絞る・並べる", "探す・絞る・並べる"));
    assert!(names_feature(".base を開く", ".base を開く"));
    assert!(names_feature("Link tables", "link tables"));
    assert!(names_feature("Link tables — relations", "link tables"));
    assert!(!names_feature("Linked tables", "link tables"));
    assert!(names_feature("Open a .base file", "open a .base file"));
    assert!(names_feature("Open read-only", "open read-only"));
    assert!(names_feature("Create a note", "create a note"));
    assert!(!names_feature("Create a notebook", "create a note"));
    assert_eq!(
        heading_core("リンクで表をつなぐ(リレーション)"),
        "リンクで表をつなぐ"
    );
    assert_eq!(heading_core("Link tables (relations)"), "Link tables");
    for (lang, words) in FEATURES {
        let (p, text) = features_page(lang);
        let cores: Vec<String> = sections(&text)
            .iter()
            .map(|s| heading_core(&s.heading))
            .collect();
        for w in words {
            assert!(
                cores.iter().any(|c| names_feature(c, w)),
                "{} に見出しが「{w}」そのもの(括弧を除いて)の節が無い: {cores:?}",
                rel(&p)
            );
        }
    }
}

#[test]
fn test_sr_51_features_links_resolve() {
    // [SR-51] くわしい節へのリンクなど、相対のリンク(http で始まらず、# だけでないもの)の行き先は、
    // `#` より前の部分が docs/manual/<lang>/ からの相対で在るファイル(../../catalog/index.html のような docs の下も可)。
    // 画像(images/)はリポに置かないので、場面の id として別の試験で見る。
    for lang in ["ja", "en"] {
        let (p, text) = features_page(lang);
        let dir = p.parent().unwrap();
        let mut seen = 0;
        for (is_image, t) in all_links(&text) {
            if is_image || t.starts_with("http") || t.starts_with('#') || t.starts_with("images/") {
                continue;
            }
            let path = t.split(['#', '?']).next().unwrap();
            assert!(
                !path.is_empty(),
                "{} のリンク {t} に行き先のファイルが無い",
                rel(&p)
            );
            let target = dir.join(path);
            assert!(
                target.exists(),
                "{} のリンク {t} の行き先 {} が無い",
                rel(&p),
                rel(&target)
            );
            seen += 1;
        }
        assert!(
            seen >= 12,
            "{} の相対のリンクが少ない: {seen} 本(12 の機能にくわしい節へ1本ずつ要る)",
            rel(&p)
        );
    }
}

#[test]
fn test_sr_51_features_anchors_point_at_headings() {
    // [SR-51] features.md の相対リンクに `#錨` が付いているとき、その錨は行き先の .md の見出しを mdBook の id
    // (小文字、空白は `-`、英数と仮名・漢字と `-`・`_` 以外の記号は落とす)に直したものの中にある。
    // 行き先の無い `#錨` だけのリンクは、このページ自身の見出しで見る。
    // id の作り方は、手引きの既存の見出し(スターターから台帳を作る・.base を開く など)と同じ形になることで確かめる。
    assert_eq!(
        heading_id("スターターから台帳を作る"),
        "スターターから台帳を作る"
    );
    assert_eq!(heading_id("探す・絞る・並べる"), "探す絞る並べる");
    assert_eq!(heading_id(".base を開く"), "base-を開く");
    assert_eq!(
        heading_id("リンクで表をつなぐ(リレーション)"),
        "リンクで表をつなぐリレーション"
    );
    assert_eq!(heading_id("Find, filter and sort"), "find-filter-and-sort");
    assert_eq!(heading_id("Open a .base file"), "open-a-base-file");
    assert_eq!(
        heading_id("Create a ledger from a starter"),
        "create-a-ledger-from-a-starter"
    );
    for lang in ["ja", "en"] {
        let (p, text) = features_page(lang);
        let dir = p.parent().unwrap();
        let mut ids: HashMap<PathBuf, Vec<String>> = HashMap::new();
        let mut checked = 0;
        for (is_image, target) in all_links(&text) {
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
        assert!(
            checked >= 1,
            "{} に `#錨` 付きのリンクが1本も無い(くわしい節へは節を指す)",
            rel(&p)
        );
    }
}

#[test]
fn test_sr_51_scenarios_have_starter_pick_scene() {
    // [SR-51] 「スターターから台帳を作る」の画像のため、台本に id = "starter-pick" の場面があり、
    // keys でコマンドパレットを開き(":")、"starter" と打って(text:starter)スターターの候補を出して撮る。
    let shots = shots();
    let shot = shots
        .iter()
        .find(|s| s["id"].as_str() == Some("starter-pick"))
        .unwrap_or_else(|| {
            panic!("docs/manual-scenarios.toml に id = \"starter-pick\" の場面が無い")
        });
    let keys: Vec<&str> = shot
        .get("keys")
        .and_then(|k| k.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&":"),
        "starter-pick の場面の keys に \":\" が無い: {keys:?}"
    );
    assert!(
        keys.contains(&"text:starter"),
        "starter-pick の場面の keys に \"text:starter\" が無い: {keys:?}"
    );
}

#[test]
fn test_sr_51_pages_py_builds_features_after_views_and_links_from_landing() {
    // [SR-51] サイトを組む scripts/pages.py は features.md を本に入れ(PAGES)、その位置は views.md の直後。
    // 入口の「見て選ぶ」の文言(TEXT の ja・en の "features" と "features_desc")とカード(landing() の
    // features.html)から一覧へ行ける。
    let py = std::fs::read_to_string(root().join("scripts/pages.py")).unwrap();
    let pages = pages_list(&py);
    let shown = pages.join(" → ");
    let views = pages
        .iter()
        .position(|s| s == "views.md")
        .unwrap_or_else(|| panic!("PAGES に views.md が無い({shown})"));
    assert_eq!(
        pages.get(views + 1).map(String::as_str),
        Some("features.md"),
        "PAGES の views.md の直後は features.md({shown})"
    );
    let text = py_section(&py, "TEXT = {", "\n}");
    let (ja, en) = text
        .split_once("\"en\": {")
        .expect("TEXT に en の文言の塊が無い");
    for (lang, block) in [("ja", ja), ("en", en)] {
        for key in ["\"features\":", "\"features_desc\":"] {
            assert!(block.contains(key), "TEXT の {lang} に {key} の文言が無い");
        }
    }
    let landing = py_section(&py, "def landing(", "\ndef ");
    assert!(
        landing.contains("features.html"),
        "landing() のカードに features.html へのリンクが無い"
    );
}

#[test]
fn test_sr_51_manual_index_links_features_page_after_views() {
    // [SR-51] 説明書の目次(ja の `## 目次`・en の `## Contents`)から features.md へ行け、その行は
    // views.md の行のあと、getting-started.md の行の前(間にほかの行があってもよい)。
    for (lang, heading) in [("ja", "目次"), ("en", "Contents")] {
        let p = root().join("docs/manual").join(lang).join("index.md");
        let text = std::fs::read_to_string(&p).unwrap();
        let toc = toc_targets(&text, heading);
        assert!(
            !toc.is_empty(),
            "{} に `## {heading}` の節のリンク(- [題](ファイル.md))が無い",
            rel(&p)
        );
        assert!(
            toc.iter().any(|t| t == "features.md"),
            "{} の目次に features.md へのリンクが無い(目次の並び: {})",
            rel(&p),
            toc.join(" → ")
        );
        if let Err(why) = check_order(&toc, &["views.md", "features.md", "getting-started.md"]) {
            panic!("{} の目次: {why}(目次の並び: {})", rel(&p), toc.join(" → "));
        }
    }
}
