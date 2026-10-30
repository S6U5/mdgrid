//! 説明書とサイトの「見せ方の一覧」のページ(SR-49)。
//! docs/manual/<言語>/views.md に、画面の型と見せ方を1項ずつ(本物の画面の画像1枚・何が見えてどう開くかの説明・
//! くわしい節へのリンク)並べ、サイトの入口(scripts/pages.py の「見て選ぶ」)からそのページへ行ける。
//! 画像は Pages の workflow が撮る場面(docs/manual-scenarios.toml)の id で指す。README の GIF の駒(gif-*)は
//! 画面の一覧に出ないので使わない。説明書の全ページの画像が場面であることは tests/test_pages.rs が見るので、
//! ここは views.md だけに絞って、8つの見せ方がそろっているかを見る。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
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

/// 見せ方の一覧のページ(無ければ、どのファイルが無いかが分かる文で落ちる)。
fn views_page(lang: &str) -> (PathBuf, String) {
    let p = root().join("docs/manual").join(lang).join("views.md");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("見せ方の一覧のページ {} が無い({e})", p.display()));
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

/// `[…](X)` の X(画像の `![…](X)` も含む。コードの囲みの中は除く)。
fn links(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in prose_lines(text) {
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            let after = &rest[i + 2..];
            let Some(j) = after.find(')') else { break };
            out.push(after[..j].to_string());
            rest = &after[j..];
        }
    }
    out
}

/// `![…](X)` の X(コードの囲みの中は除く)。
fn images(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in prose_lines(text) {
        let mut rest = line;
        while let Some(i) = rest.find("![") {
            let after = &rest[i..];
            let Some(j) = after.find("](") else { break };
            let tail = &after[j + 2..];
            let Some(k) = tail.find(')') else { break };
            out.push(tail[..k].to_string());
            rest = &tail[k..];
        }
    }
    out
}

/// 見出しの文(`## ` から下の段。コードの囲みの中は除く)。
fn headings(text: &str) -> Vec<String> {
    prose_lines(text)
        .into_iter()
        .filter(|l| l.starts_with("##"))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

/// `start` から、その後の最初の `end` までの切れ端(`end` が無ければ末尾まで)。
fn section<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let i = text
        .find(start)
        .unwrap_or_else(|| panic!("scripts/pages.py に {start:?} が無い"));
    let body = &text[i + start.len()..];
    match body.find(end) {
        Some(j) => &body[..j],
        None => body,
    }
}

#[test]
fn test_sr_49_views_pages_images_are_scenes() {
    // [SR-49] 見せ方の一覧(日英)があり、画像はどれも撮る場面の id を指し(README の GIF の駒 gif-* は使わない)、
    // 8つの見せ方に1枚ずつで少なくとも 8 枚ある。
    let ids = scene_ids();
    for lang in ["ja", "en"] {
        let (p, text) = views_page(lang);
        let imgs = images(&text);
        assert!(
            imgs.len() >= 8,
            "{} の画像が少ない: {} 枚(8つの見せ方に1枚ずつ要る)",
            p.display(),
            imgs.len()
        );
        for t in &imgs {
            let name = t
                .strip_prefix("images/")
                .and_then(|n| n.strip_suffix(".svg"))
                .unwrap_or_else(|| {
                    panic!("{} の画像 {t} は images/<場面>.svg でない", p.display())
                });
            assert!(
                !name.starts_with("gif-"),
                "{} の画像 {t} は README の GIF の駒(画面の一覧に出ない)",
                p.display()
            );
            assert!(
                ids.contains(name),
                "{} の {t} を撮る場面が無い",
                p.display()
            );
        }
    }
}

#[test]
fn test_sr_49_views_pages_headings_cover_eight_views() {
    // [SR-49] 見出しに、表・ビューのタブ・まとまり(グループ)・木・関係マップ・値の分布・絞り込みの帯・
    // テーマ(と部品の形)の8つの見せ方がそれぞれある(en は大文字小文字を問わない)。
    let want: [(&str, &[&str]); 2] = [
        (
            "ja",
            &[
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
            &[
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
    for (lang, words) in want {
        let (p, text) = views_page(lang);
        let hs: Vec<String> = headings(&text).iter().map(|h| h.to_lowercase()).collect();
        for w in words {
            assert!(
                hs.iter().any(|h| h.contains(&w.to_lowercase())),
                "{} の見出しに「{w}」の見せ方が無い: {hs:?}",
                p.display()
            );
        }
    }
}

#[test]
fn test_sr_49_views_pages_links_resolve() {
    // [SR-49] くわしい節へのリンクなど、相対のリンク(http で始まらず、# だけでないもの)の行き先は、
    // `#` より前の部分が docs/manual/<lang>/ からの相対で在るファイル(../../catalog/index.html のような docs の下も可)。
    // 画像(images/)はリポに置かないので別の試験で場面の id として見る。
    for lang in ["ja", "en"] {
        let (p, text) = views_page(lang);
        let dir = p.parent().unwrap();
        let mut seen = 0;
        for t in links(&text) {
            if t.starts_with("http") || t.starts_with('#') || t.starts_with("images/") {
                continue;
            }
            let path = t.split(['#', '?']).next().unwrap();
            assert!(
                !path.is_empty(),
                "{} のリンク {t} に行き先のファイルが無い",
                p.display()
            );
            let target = dir.join(path);
            assert!(
                target.exists(),
                "{} のリンク {t} の行き先 {} が無い",
                p.display(),
                target.display()
            );
            seen += 1;
        }
        assert!(
            seen >= 8,
            "{} のくわしい節へのリンクが少ない: {seen} 本(8つの見せ方に1本ずつ要る)",
            p.display()
        );
    }
}

#[test]
fn test_sr_49_scenarios_have_tree_scene() {
    // [SR-49] 木の見せ方の画像のため、台本に id = "tree" の場面があり、キー T で木に切り替えて撮る。
    let shots = shots();
    let tree = shots
        .iter()
        .find(|s| s["id"].as_str() == Some("tree"))
        .unwrap_or_else(|| panic!("docs/manual-scenarios.toml に id = \"tree\" の場面が無い"));
    let keys: Vec<&str> = tree
        .get("keys")
        .and_then(|k| k.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        keys.contains(&"T"),
        "tree の場面の keys に \"T\" が無い: {keys:?}"
    );
}

#[test]
fn test_sr_49_pages_py_builds_views_and_links_from_landing() {
    // [SR-49] サイトを組む scripts/pages.py は views.md を本に入れ(PAGES)、入口の「見て選ぶ」の文言(TEXT の ja・en の
    // "views")とカード(landing() の views.html)から一覧へ行ける。
    let py = std::fs::read_to_string(root().join("scripts/pages.py")).unwrap();
    let pages = section(&py, "PAGES = [", "]");
    assert!(
        pages.contains("\"views.md\""),
        "PAGES に views.md が無い: [{pages}]"
    );
    let text = section(&py, "TEXT = {", "\n}");
    let (ja, en) = text
        .split_once("\"en\": {")
        .expect("TEXT に en の文言の塊が無い");
    assert!(
        ja.contains("\"views\":"),
        "TEXT の ja に \"views\" の文言が無い"
    );
    assert!(
        en.contains("\"views\":"),
        "TEXT の en に \"views\" の文言が無い"
    );
    let landing = section(&py, "def landing(", "\ndef ");
    assert!(
        landing.contains("views.html"),
        "landing() のカードに views.html へのリンクが無い"
    );
}

#[test]
fn test_sr_49_manual_index_links_views_page() {
    // [SR-49] 説明書の目次(ja・en の index.md)から views.md へ行ける。
    for lang in ["ja", "en"] {
        let p = root().join("docs/manual").join(lang).join("index.md");
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(
            links(&text)
                .iter()
                .any(|t| t.split('#').next() == Some("views.md")),
            "{} の目次に views.md へのリンクが無い",
            p.display()
        );
    }
}
