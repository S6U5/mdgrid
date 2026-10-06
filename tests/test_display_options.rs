//! display-options(specs/_changes/2026-10-02-display-options.md)の受け入れ: 設定の `[display]`(SR-21・CLI-3)と
//! `--print-config`(CLI-11・CLI-12)。実装を見ずに書いた。環境変数は書き換えない(子の環境だけ)。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/config.rs
//! pub struct Config {
//!     // …今の項目…
//!     /// SR-20・SR-21: `[display]`。既定 row_numbers = false・zebra = false・column_lines = false・
//!     /// tabs = true・chips = true。検索の欄は最上位の `search_bar` のまま。
//!     pub display: Display,
//! }
//! pub struct Display { pub row_numbers: bool, pub zebra: bool, pub column_lines: bool, pub tabs: bool, pub chips: bool }
//! ```
//!
//! 試験は `c.display.<名前>` の読み取りだけを使う(構造体の名前には頼らない)。

use mdgrid::config::{parse, Config, KEYS};
use std::process::{Command, Output};

const NAMES: [&str; 5] = ["row_numbers", "zebra", "column_lines", "tabs", "chips"];

/// (名前, 既定)。
const DEFAULTS: [(&str, bool); 5] = [
    ("row_numbers", false),
    ("zebra", false),
    ("column_lines", false),
    ("tabs", true),
    ("chips", true),
];

fn display_values(c: &Config) -> [(&'static str, bool); 5] {
    [
        ("row_numbers", c.display.row_numbers),
        ("zebra", c.display.zebra),
        ("column_lines", c.display.column_lines),
        ("tabs", c.display.tabs),
        ("chips", c.display.chips),
    ]
}

fn read_ok(text: &str) -> Config {
    let (c, warnings) = parse(text).expect("読める");
    assert!(warnings.is_empty(), "警告: {warnings:?}\n---\n{text}");
    c
}

// ---- SR-21: [display] を読む ----

#[test]
fn test_sr_21_defaults() {
    // [SR-21] 設定が空なら row_numbers=false・zebra=false・column_lines=false・tabs=true・chips=true。
    let c = read_ok("");
    assert_eq!(display_values(&c), DEFAULTS);
    assert_eq!(display_values(&Config::default()), DEFAULTS);
    // 空の [display] も既定のまま、警告なし。
    let c = read_ok("[display]\n");
    assert_eq!(display_values(&c), DEFAULTS);
}

#[test]
fn test_sr_21_reads_each_item() {
    // [SR-21] 5項目を全部既定と逆にして読む。ほかの項目はそのまま。
    let text = "search_bar = false\n\n[display]\nrow_numbers = true\nzebra = true\n\
                column_lines = true\ntabs = false\nchips = false\n";
    let c = read_ok(text);
    assert_eq!(
        display_values(&c),
        [
            ("row_numbers", true),
            ("zebra", true),
            ("column_lines", true),
            ("tabs", false),
            ("chips", false),
        ]
    );
    assert!(!c.search_bar, "検索の欄は最上位の search_bar");
    // 1つだけ書けば、ほかは既定。
    let c = read_ok("[display]\nrow_numbers = true\n");
    assert!(c.display.row_numbers);
    assert!(!c.display.zebra && !c.display.column_lines && c.display.tabs && c.display.chips);
}

#[test]
fn test_sr_21_display_is_a_known_item() {
    // [SR-21][CLI-12] `display` は設定の項目の表にある(知らない項目の警告にならない)。
    assert!(
        KEYS.contains(&"display"),
        "KEYS に display が無い: {KEYS:?}"
    );
    let (_, warnings) = parse("[display]\nzebra = true\n").unwrap();
    assert!(
        !warnings
            .iter()
            .any(|w| w.contains("知らない項目 `display`")),
        "{warnings:?}"
    );
}

#[test]
fn test_sr_21_wrong_type_warns_and_keeps_default() {
    // [SR-21][CLI-3] 型の違う値は警告して既定のまま。止めずに、ほかの項目は読む。
    for (name, default) in DEFAULTS {
        let text = format!("candidates = 7\n\n[display]\n{name} = \"yes\"\n");
        let (c, warnings) = parse(&text).expect("型違いでも止めない");
        assert_eq!(warnings.len(), 1, "{name}: {warnings:?}");
        assert!(warnings[0].contains(name), "警告に項目の名前: {warnings:?}");
        let got = display_values(&c)
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        assert_eq!(got, default, "{name} は既定のまま");
        assert_eq!(c.candidates, 7, "ほかの項目は読む");
    }
    // 数も型違い。
    let (c, warnings) = parse("[display]\nrow_numbers = 1\ntabs = false\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("row_numbers"), "{warnings:?}");
    assert!(!c.display.row_numbers);
    assert!(!c.display.tabs, "同じ表のほかの項目は読む");
    // `display` が表でない。
    let (c, warnings) = parse("display = true\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("display"), "{warnings:?}");
    assert_eq!(display_values(&c), DEFAULTS);
}

#[test]
fn test_sr_21_unknown_item_warns() {
    // [SR-21][CLI-3] [display] の知らない項目は警告して無視、ほかは読む。
    let (c, warnings) = parse("[display]\nfoo = true\nzebra = true\n").expect("止めない");
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("foo"), "{warnings:?}");
    assert!(c.display.zebra);
    // 検索の欄は [display] に足さない(最上位の search_bar のまま)→ [display] の search_bar は知らない項目。
    let (c, warnings) = parse("[display]\nsearch_bar = false\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("search_bar"), "{warnings:?}");
    assert!(c.search_bar, "[display] の search_bar は効かない");
}

// ---- SR-21・CLI-11: --print-config ----

fn run(args: &[&str]) -> Output {
    // 利用者の設定を読まないよう、子の環境だけで設定と状態の置き場を一時フォルダの存在しない所へ向ける。
    let nowhere = std::env::temp_dir().join(format!(
        "mdgrid-display-options-none-{}",
        std::process::id()
    ));
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

#[test]
fn test_sr_21_print_config_has_display() {
    // [SR-21][CLI-11] --print-config に [display] の5項目が既定値で出る(TOML として読んで判定)。
    let text = print_config();
    let table: toml::Table = text.parse().expect("--print-config の出力は TOML");
    let display = table
        .get("display")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| panic!("--print-config に [display] の表が無い\n---\n{text}"));
    for (name, default) in DEFAULTS {
        assert_eq!(
            display.get(name).and_then(|v| v.as_bool()),
            Some(default),
            "[display] の {name} が既定値 {default} で無い\n---\n{text}"
        );
    }
    assert_eq!(
        display.len(),
        NAMES.len(),
        "[display] は5項目だけ: {display:?}"
    );
    assert!(
        !display.contains_key("search_bar"),
        "検索の欄は最上位の search_bar"
    );
    // ほかの項目が [display] の表に紛れ込まない(読み直して警告なし・既定と同じ)。
    let (c, warnings) = parse(&text).expect("読める");
    assert!(warnings.is_empty(), "警告: {warnings:?}\n---\n{text}");
    assert_eq!(c, Config::default());
    assert!(table.contains_key("search_bar"), "search_bar は最上位");
}
