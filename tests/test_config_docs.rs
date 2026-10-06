//! タスク 2(oss-config)の受け入れ: `--print-config`(CLI-11・CLI-2)と、設定の文書・実装・書き出しの
//! 突き合わせ(CLI-12・CLI-3)。実装を見ずに書いた。環境変数は書き換えない。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/config.rs: 実装が読む設定の最上位の項目の名前の一覧(項目の表から作ってよい)。
//! // `keys`([keys.<モード>] の表)と `editor` も含む。
//! pub const KEYS: &[&str];
//! ```
//!
//! 仮定した文書の形: docs/config.md(英語)と docs/config.ja.md(日本語)で、項目ごとに
//! 見出し `### \`name\`` を1つ置き、その下に型・既定値・説明を書く。

use mdgrid::config::{parse, Config, KEYS};
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

/// 書き出しの行が項目 `name` のもの(コメントアウトされた例も含む)か。
fn is_item_line(line: &str, name: &str) -> bool {
    let t = line.trim_start().trim_start_matches('#').trim_start();
    let after = if let Some(r) = t.strip_prefix(name) {
        r
    } else if let Some(r) = t.strip_prefix('[').and_then(|r| r.strip_prefix(name)) {
        r
    } else {
        return false;
    };
    let after = after.trim_start();
    after.starts_with('=') || after.starts_with(']') || after.starts_with('.')
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
    assert_eq!(
        c,
        Config::default(),
        "読み直した設定が既定と違う\n---\n{text}"
    );
}

#[test]
fn test_cli_11_output_has_every_item_with_comment() {
    // [CLI-11] 出力に全項目の名前と、説明のコメント(`#`)がある。
    let text = print_config();
    let lines: Vec<&str> = text.lines().collect();
    for name in KEYS {
        let Some(at) = lines.iter().position(|l| is_item_line(l, name)) else {
            panic!("--print-config に項目 `{name}` が無い\n---\n{text}");
        };
        // その行がコメントか、すぐ上(空行や別の項目の行に当たるまで)に説明のコメントがある。
        let mut commented = lines[at].contains('#');
        for l in lines[..at].iter().rev() {
            let t = l.trim();
            if t.is_empty() || KEYS.iter().any(|k| is_item_line(l, k)) {
                break;
            }
            if t.starts_with('#') {
                commented = true;
                break;
            }
        }
        assert!(
            commented,
            "項目 `{name}` に説明のコメントが無い\n---\n{text}"
        );
    }
}

#[test]
fn test_cli_11_output_values_are_defaults() {
    // [CLI-11] 書き出しの値の行は既定値。項目を1つずつ既定の設定に足しても、既定から変わらない。
    let text = print_config();
    let table: toml::Table = text.parse().expect("TOML として読める");
    for (name, value) in &table {
        assert!(
            KEYS.contains(&name.as_str()),
            "書き出しに KEYS に無い項目 `{name}` がある"
        );
        let mut one = toml::Table::new();
        one.insert(name.clone(), value.clone());
        let (c, warnings) = parse(&one.to_string()).expect("読める");
        assert!(warnings.is_empty(), "`{name}` で警告: {:?}", warnings);
        assert_eq!(c, Config::default(), "`{name}` の書き出しの値が既定でない");
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
    // [CLI-12] 実装の項目の一覧に、今ある設定の項目(CLI-3・SR-13・SR-8 ほか)が入っている。
    let set = keys_set();
    for name in [
        "keys",
        "color",
        "candidates",
        "poll_ms",
        "ambiguous_wide",
        "search_bar",
        "date_format",
        "week_start",
        "add_frontmatter",
        "editor",
    ] {
        assert!(set.contains(name), "KEYS に `{name}` が無い: {:?}", set);
    }
}

#[test]
fn test_cli_12_parse_knows_every_key() {
    // [CLI-12] [CLI-3] KEYS の項目はどれも設定の読み取りが知っている(知らない項目の警告にならない)。
    // 知らない項目の警告の文を、本当に知らない名前で作り、名前を差し替えて比べる(文言に依らない)。
    let probe = "zz_not_a_config_item";
    let (_, w) = parse(&format!("{probe} = 0\n")).expect("読める");
    assert_eq!(w.len(), 1, "知らない項目の警告が1つ出る: {:?}", w);
    let unknown_template = &w[0];
    assert!(unknown_template.contains(probe));
    for name in KEYS {
        let unknown = unknown_template.replace(probe, name);
        let (_, warnings) = parse(&format!("{name} = 0\n")).expect("読める");
        assert!(
            !warnings.contains(&unknown),
            "実装が `{name}` を知らない項目として扱った: {:?}",
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
    // [CLI-12] --print-config の項目(コメントアウトの例も含む)と実装の項目が同じ集合。
    let text = print_config();
    let printed: BTreeSet<String> = KEYS
        .iter()
        .filter(|k| text.lines().any(|l| is_item_line(l, k)))
        .map(|k| k.to_string())
        .collect();
    assert_eq!(
        printed,
        keys_set(),
        "--print-config に出ない項目がある\n---\n{text}"
    );
    // 書き出しの値の行の項目は、どれも実装の項目。
    let table: toml::Table = text.parse().expect("TOML として読める");
    for name in table.keys() {
        assert!(
            KEYS.contains(&name.as_str()),
            "書き出しに KEYS に無い項目 `{name}`"
        );
    }
}
