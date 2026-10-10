//! タスク 2(oss-config)の受け入れ: `--print-config`(CLI-11・CLI-2)と、設定の文書・実装・書き出しの
//! 突き合わせ(CLI-12・CLI-3)。config-v2(specs/_changes/2026-10-10-config-v2.md)で、項目を区画ごとの道筋
//! (`look.theme` など)にした。環境変数は書き換えない。
//!
//! 文書の形: docs/config.md(英語)と docs/config.ja.md(日本語)で、項目ごとに見出し `### \`道筋\`` を1つ置き、
//! その下に型・既定値・書ける範囲・説明を書く。旧い名前の対応は表の行 `| \`旧い名前\` | \`新しい道筋\` |`。

use mdgrid::config::{parse, Config, ITEMS, KEYS};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    // 利用者の設定を読まないよう、子の環境だけで設定の置き場を存在しない所へ向ける。
    let nowhere =
        std::env::temp_dir().join(format!("mdgrid-config-docs-none-{}", std::process::id()));
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .env("XDG_CONFIG_HOME", &nowhere)
        .env("XDG_STATE_HOME", &nowhere)
        .output()
        .expect("mdgrid を起動できる")
}

fn print_config() -> String {
    let out = run(&["--print-config"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--print-config の終了コード。stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("出力は UTF-8")
}

fn keys_set() -> BTreeSet<String> {
    let set: BTreeSet<String> = KEYS.iter().map(|k| k.to_string()).collect();
    assert_eq!(set.len(), KEYS.len(), "KEYS に重複がある: {:?}", KEYS);
    set
}

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

/// 文書の項目の見出し `### \`name\`` の名前の一覧(並べた集合)。各項目の本文が空でないことも確かめる。
fn doc_items(rel: &str) -> BTreeSet<String> {
    let text = repo_file(rel);
    let lines: Vec<&str> = text.lines().collect();
    let mut names = BTreeSet::new();
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim_end();
        let Some(rest) = t.strip_prefix("### `") else {
            continue;
        };
        let Some(name) = rest.strip_suffix('`') else {
            continue;
        };
        assert!(
            names.insert(name.to_string()),
            "{rel}: 項目 `{name}` の見出しが2つある"
        );
        // 次の見出しまでに説明の本文がある。
        let body: String = lines[i + 1..]
            .iter()
            .take_while(|l| !l.starts_with('#'))
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("");
        assert!(!body.is_empty(), "{rel}: 項目 `{name}` に説明が無い");
    }
    names
}

/// 書き出しの文で、項目 `path` の行(コメントアウトされた例も含む)の位置。区画の見出しをたどって、その区画の中の
/// `名前 =` の行か、表の項目なら `[道筋` で始まる見出し(コメントも)を探す。
fn item_line(text: &str, path: &str) -> Option<usize> {
    let item = ITEMS.iter().find(|i| i.path == path)?;
    let mut section = String::new();
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start().trim_start_matches('#').trim_start();
        if item.is_table() {
            let header = t.starts_with(&format!("[{path}]")) || t.starts_with(&format!("[{path}."));
            if header && t.ends_with(']') {
                return Some(i);
            }
            continue;
        }
        if !line.starts_with('#') && t.starts_with('[') {
            section = t.trim_matches(|c| c == '[' || c == ']').to_string();
            continue;
        }
        if section == item.section() {
            if let Some(rest) = t.strip_prefix(item.leaf()) {
                if rest.trim_start().starts_with('=') {
                    return Some(i);
                }
            }
        }
    }
    None
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

// ---- CLI-11: --print-config ----

#[test]
fn test_cli_11_print_config_exits_zero() {
    // [CLI-11] 標準出力に TOML を出し、終了コード 0。
    let text = print_config();
    assert!(!text.trim().is_empty(), "--print-config の出力が空");
}

#[test]
fn test_cli_11_output_reads_back_without_warnings_as_default() {
    // [CLI-11] 出したものはそのまま設定として警告なしで読め、既定と同じ。
    let text = print_config();
    let (c, warnings) = parse(&text).expect("--print-config の出力は設定として読める");
    assert!(
        warnings.is_empty(),
        "読み直しで警告: {:?}\n---\n{text}",
        warnings
    );
    assert!(
        behaves_as_default(&c),
        "読み直した設定が既定と違う\n---\n{text}"
    );
}

#[test]
fn test_cli_11_output_has_every_item_with_comment() {
    // [CLI-11] 出力に全項目と、説明のコメント(`#`)と、書ける範囲(`# Scope:`)がある。
    let text = print_config();
    let lines: Vec<&str> = text.lines().collect();
    for item in ITEMS {
        let Some(at) = item_line(&text, item.path) else {
            panic!("--print-config に項目 `{}` が無い\n---\n{text}", item.path);
        };
        // すぐ上(空行に当たるまで)に、説明と書ける範囲のコメントがある。
        let above: Vec<&str> = lines[..at]
            .iter()
            .rev()
            .take_while(|l| !l.trim().is_empty())
            .copied()
            .collect();
        assert!(
            above.iter().any(|l| l.starts_with("# Scope: ")),
            "項目 `{}` に書ける範囲のコメントが無い\n---\n{text}",
            item.path
        );
        assert!(
            above.len() >= 2,
            "項目 `{}` に説明のコメントが無い\n---\n{text}",
            item.path
        );
    }
}

#[test]
fn test_cli_11_output_values_are_defaults() {
    // [CLI-11] 書き出しの値の行は既定値。区画ごとに1つずつ読んでも、既定と同じ振る舞い。
    let text = print_config();
    let table: toml::Table = text.parse().expect("TOML として読める");
    for (name, value) in &table {
        let mut one = toml::Table::new();
        one.insert(name.clone(), value.clone());
        let (c, warnings) = parse(&one.to_string()).expect("読める");
        assert!(warnings.is_empty(), "`{name}` で警告: {:?}", warnings);
        assert!(
            behaves_as_default(&c),
            "`{name}` の書き出しの値が既定でない"
        );
    }
}

// ---- CLI-2: --print-config は起動の引数のひとつ ----

#[test]
fn test_cli_2_help_mentions_print_config() {
    // [CLI-2] [CLI-11] --help に --print-config が出る。
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(
        help.contains("--print-config"),
        "--help に --print-config が無い:\n{help}"
    );
}

// ---- CLI-12: 文書・実装・書き出しの突き合わせ ----

#[test]
fn test_cli_12_keys_cover_known_items() {
    // [CLI-12] 実装の項目の一覧に、今ある設定の項目(CLI-3・SR-13・SR-8 ほか)が区画の道筋で入っている。
    let set = keys_set();
    for name in [
        "keys",
        "terminal.color",
        "edit.candidates",
        "poll_ms",
        "terminal.ambiguous_wide",
        "display.search_bar",
        "dates.format",
        "dates.week_start",
        "edit.add_frontmatter",
        "editor",
        "look.theme",
        "templates",
    ] {
        assert!(set.contains(name), "KEYS に `{name}` が無い: {:?}", set);
    }
}

#[test]
fn test_cli_12_parse_knows_every_key() {
    // [CLI-12] [CLI-3] KEYS の項目はどれも設定の読み取りが知っている(知らない項目の警告にならない)。
    let (_, w) = parse("zz_not_a_config_item = 0\n").expect("読める");
    assert_eq!(w.len(), 1, "知らない項目の警告が1つ出る: {:?}", w);
    let unknown = w[0].rsplit(": ").next().unwrap().to_string();
    for item in ITEMS {
        let text = match item.section() {
            "" => format!("{} = 0\n", item.leaf()),
            sec => format!("[{sec}]\n{} = 0\n", item.leaf()),
        };
        let (_, warnings) = parse(&text).expect("読める");
        assert!(
            !warnings.iter().any(|w| w.ends_with(&unknown)),
            "実装が `{}` を知らない項目として扱った: {:?}",
            item.path,
            warnings
        );
    }
}

#[test]
fn test_cli_12_english_doc_matches_keys() {
    // [CLI-12] docs/config.md の項目と実装の項目が同じ集合。
    let doc = doc_items("docs/config.md");
    let keys = keys_set();
    assert_eq!(
        doc,
        keys,
        "docs/config.md と実装の項目が違う。文書だけ: {:?} 実装だけ: {:?}",
        doc.difference(&keys).collect::<Vec<_>>(),
        keys.difference(&doc).collect::<Vec<_>>()
    );
}

#[test]
fn test_cli_12_japanese_doc_matches_keys() {
    // [CLI-12] docs/config.ja.md の項目と実装の項目が同じ集合。
    let doc = doc_items("docs/config.ja.md");
    let keys = keys_set();
    assert_eq!(
        doc,
        keys,
        "docs/config.ja.md と実装の項目が違う。文書だけ: {:?} 実装だけ: {:?}",
        doc.difference(&keys).collect::<Vec<_>>(),
        keys.difference(&doc).collect::<Vec<_>>()
    );
}

#[test]
fn test_cli_12_print_config_matches_keys() {
    // [CLI-12] --print-config の項目(コメントアウトの例も含む)と実装の項目が同じ集合。値の行の区画は、どれも実装の区画。
    let text = print_config();
    let printed: BTreeSet<String> = KEYS
        .iter()
        .filter(|k| item_line(&text, k).is_some())
        .map(|k| k.to_string())
        .collect();
    assert_eq!(
        printed,
        keys_set(),
        "--print-config に出ない項目がある\n---\n{text}"
    );
    let table: toml::Table = text.parse().expect("TOML として読める");
    for (name, v) in &table {
        match v.as_table() {
            Some(t) => {
                for k in t.keys() {
                    let path = format!("{name}.{k}");
                    assert!(
                        KEYS.contains(&path.as_str()),
                        "書き出しに KEYS に無い項目 `{path}`"
                    );
                }
            }
            None => assert!(
                KEYS.contains(&name.as_str()),
                "書き出しに KEYS に無い項目 `{name}`"
            ),
        }
    }
}

#[test]
fn test_cli_12_docs_list_old_names() {
    // [CLI-12][CLI-20] 文書に、旧い名前と移った先の対応の表がある(実装の表 LEGACY と同じ行)。
    for rel in ["docs/config.md", "docs/config.ja.md"] {
        let text = repo_file(rel);
        for (old, new) in mdgrid::schema::LEGACY {
            let row = format!("| `{old}` | `{new}` |");
            assert!(text.contains(&row), "{rel} に旧い名前の行 {row} が無い");
        }
    }
}
