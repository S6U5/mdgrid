//! 設定の文書(docs/config.md・docs/config.ja.md)の各項目の「型」「既定」の行と例を、実装の項目の表
//! (`mdgrid::config::ITEMS`)と突き合わせる(CLI-12)。名前の集合の突き合わせは tests/test_config_docs.rs。
//!
//! 文書の各項目は、見出し `### \`道筋\`` の下に、決まった形の行を持つ:
//! - 英語: `- Type: \`<ty>\``・`- Default: \`<default>\``(既定の無い項目は `- Default: none`)・`- Scope: <範囲>`
//! - 日本語: `- 型: \`<ty_ja>\``・`- 既定: \`<default>\``(既定の無い項目は `- 既定: なし`)・`- 範囲: <範囲>`
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

struct Lang {
    rel: &'static str,
    ty_prefix: &'static str,
    default_prefix: &'static str,
    scope_prefix: &'static str,
    none: &'static str,
    ty: fn(&Item) -> &str,
    scope: fn(&Item) -> &str,
}

fn check(l: Lang) {
    let Lang {
        rel,
        ty_prefix,
        default_prefix,
        scope_prefix,
        none,
        ty,
        scope,
    } = l;
    let text = repo_file(rel);
    for item in ITEMS {
        let body = section(&text, item.path);
        assert_eq!(
            field(&body, scope_prefix, rel, item.path),
            scope(item),
            "{rel}: 項目 `{}` の書ける範囲が実装の表と違う",
            item.path
        );
        assert_eq!(
            field(&body, ty_prefix, rel, item.path),
            quoted(ty(item)),
            "{rel}: 項目 `{}` の型が実装の表と違う",
            item.path
        );
        let want = match item.default {
            Some(d) => quoted(d),
            None => none.to_string(),
        };
        assert_eq!(
            field(&body, default_prefix, rel, item.path),
            want,
            "{rel}: 項目 `{}` の既定が実装の表と違う",
            item.path
        );
        assert_eq!(
            toml_block(&body, rel, item.path),
            item.example,
            "{rel}: 項目 `{}` の例が実装の表と違う",
            item.path
        );
    }
}

#[test]
fn test_cli_12_english_doc_type_default_example() {
    // [CLI-12] docs/config.md の各項目の Type・Default の行と例が、実装の表と同じ。
    check(Lang {
        rel: "docs/config.md",
        ty_prefix: "- Type: ",
        default_prefix: "- Default: ",
        scope_prefix: "- Scope: ",
        none: "none",
        ty: |i| i.ty,
        scope: |i| i.scope.en(),
    });
}

#[test]
fn test_cli_12_japanese_doc_type_default_example() {
    // [CLI-12] docs/config.ja.md の各項目の 型・既定 の行と例が、実装の表と同じ。
    check(Lang {
        rel: "docs/config.ja.md",
        ty_prefix: "- 型: ",
        default_prefix: "- 既定: ",
        scope_prefix: "- 範囲: ",
        none: "なし",
        ty: |i| i.ty_ja,
        scope: |i| i.scope.ja(),
    });
}

/// 既定と同じ振る舞いか(アプリ全体の項目と、重ねた決まった値。出どころは見ない)。
fn behaves_as_default(c: &Config) -> bool {
    let d = Config::default();
    let mut r = c.resolved();
    r.origins.clear();
    c.language == d.language
        && c.editor == d.editor
        && c.poll_ms == d.poll_ms
        && c.keys == d.keys
        && c.terminal == d.terminal
        && c.workspace_detect == d.workspace_detect
        && c.templates.is_empty()
        && r == d.resolved()
}

#[test]
fn test_cli_12_table_defaults_are_real_defaults() {
    // [CLI-12] [CLI-11] 表の既定値の表記は、その1行だけを(区画の見出しの下に)読んでも警告なしで既定と同じ振る舞い
    // (表の既定値を実装と違えたら、文書を直す前にここで落ちる)。既定の無い項目は既定では None・空。
    for item in ITEMS {
        match item.default {
            Some(d) => {
                let text = match item.section() {
                    "" => format!("{} = {d}\n", item.leaf()),
                    sec => format!("[{sec}]\n{} = {d}\n", item.leaf()),
                };
                let (c, w) = parse(&text).unwrap();
                assert!(w.is_empty(), "`{}`: {w:?}", item.path);
                assert!(
                    behaves_as_default(&c),
                    "`{}` の表の既定値が実装と違う",
                    item.path
                );
            }
            None => {
                let d = Config::default();
                match item.path {
                    "editor" => assert_eq!(d.editor, None),
                    "keys" => assert!(d.keys.is_empty()),
                    "templates" => assert!(d.templates.is_empty()),
                    "use" => assert_eq!(d.profile.use_, None),
                    "look.style" => assert!(d.profile.look.style.is_empty()),
                    "look.columns" => assert!(d.profile.look.columns.is_empty()),
                    "look.colors" => assert!(d.profile.look.values.is_empty()),
                    "new_note" => assert_eq!(d.profile.new_note, None),
                    other => panic!("既定の無い項目 `{other}` の確かめ方を足す"),
                }
            }
        }
    }
}
