//! 設定の文書(docs/config.md・docs/config.ja.md)の各項目の「型」「既定」の行と例を、実装の項目の表
//! (`mdgrid::config::ITEMS`)と突き合わせる(CLI-12)。名前の集合の突き合わせは tests/test_config_docs.rs。
//!
//! 文書の各項目は、見出し `### \`name\`` の下に、決まった形の行を持つ:
//! - 英語: `- Type: \`<ty>\`` と `- Default: \`<default>\``(既定の無い項目は `- Default: none`)
//! - 日本語: `- 型: \`<ty_ja>\`` と `- 既定: \`<default>\``(既定の無い項目は `- 既定: なし`)
//! - どちらも、項目の中の ```toml の区画が `example` と同じ。

use mdgrid::config::{parse, Config, Item, ITEMS};
use std::path::PathBuf;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

/// 文書の項目 `name` の本文(見出しの次の行から、次の `### ` の見出しか `## ` の見出しの手前まで)。
fn section<'a>(text: &'a str, name: &str) -> Vec<&'a str> {
    let heading = format!("### `{name}`");
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim_end() == heading)
        .unwrap_or_else(|| panic!("見出し {heading} が無い"));
    lines[at + 1..]
        .iter()
        .take_while(|l| !l.starts_with("## ") && !l.starts_with("### "))
        .copied()
        .collect()
}

/// 本文の中で `prefix` で始まる行(ちょうど1つ)の残り。
fn field<'a>(body: &[&'a str], prefix: &str, rel: &str, name: &str) -> &'a str {
    let found: Vec<&str> = body.iter().filter_map(|l| l.strip_prefix(prefix)).collect();
    assert_eq!(
        found.len(),
        1,
        "{rel}: 項目 `{name}` に `{prefix}` の行がちょうど1つ無い: {found:?}"
    );
    found[0].trim_end()
}

/// 本文の ```toml の区画(ちょうど1つ)の中身。
fn toml_block(body: &[&str], rel: &str, name: &str) -> String {
    let starts: Vec<usize> = body
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim_end() == "```toml")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        starts.len(),
        1,
        "{rel}: 項目 `{name}` の ```toml の区画がちょうど1つ無い"
    );
    let inner: Vec<&str> = body[starts[0] + 1..]
        .iter()
        .take_while(|l| l.trim_end() != "```")
        .copied()
        .collect();
    inner.join("\n")
}

fn quoted(v: &str) -> String {
    format!("`{v}`")
}

fn check(rel: &str, ty_prefix: &str, default_prefix: &str, none: &str, ty: fn(&Item) -> &str) {
    let text = repo_file(rel);
    for item in ITEMS {
        let body = section(&text, item.name);
        assert_eq!(
            field(&body, ty_prefix, rel, item.name),
            quoted(ty(item)),
            "{rel}: 項目 `{}` の型が実装の表と違う",
            item.name
        );
        let want = match item.default {
            Some(d) => quoted(d),
            None => none.to_string(),
        };
        assert_eq!(
            field(&body, default_prefix, rel, item.name),
            want,
            "{rel}: 項目 `{}` の既定が実装の表と違う",
            item.name
        );
        assert_eq!(
            toml_block(&body, rel, item.name),
            item.example,
            "{rel}: 項目 `{}` の例が実装の表と違う",
            item.name
        );
    }
}

#[test]
fn test_cli_12_english_doc_type_default_example() {
    // [CLI-12] docs/config.md の各項目の Type・Default の行と例が、実装の表と同じ。
    check("docs/config.md", "- Type: ", "- Default: ", "none", |i| {
        i.ty
    });
}

#[test]
fn test_cli_12_japanese_doc_type_default_example() {
    // [CLI-12] docs/config.ja.md の各項目の 型・既定 の行と例が、実装の表と同じ。
    check(
        "docs/config.ja.md",
        "- 型: ",
        "- 既定: ",
        "なし",
        |i| i.ty_ja,
    );
}

#[test]
fn test_cli_12_table_defaults_are_real_defaults() {
    // [CLI-12] [CLI-11] 表の既定値の表記は、その1行だけを読んでも警告なしで Config::default() と同じ
    // (表の既定値を実装と違えたら、文書を直す前にここで落ちる)。既定の無い項目は既定では None・空。
    for item in ITEMS {
        match item.default {
            Some(d) => {
                let (c, w) = parse(&format!("{} = {d}\n", item.name)).unwrap();
                assert!(w.is_empty(), "`{}`: {w:?}", item.name);
                assert_eq!(
                    c,
                    Config::default(),
                    "`{}` の表の既定値が実装と違う",
                    item.name
                );
            }
            None => {
                let d = Config::default();
                match item.name {
                    "editor" => assert_eq!(d.editor, None),
                    "keys" => assert!(d.keys.is_empty()),
                    other => panic!("既定の無い項目 `{other}` の確かめ方を足す"),
                }
            }
        }
    }
}
