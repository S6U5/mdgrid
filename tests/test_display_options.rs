//! display-options(specs/_changes/2026-10-02-display-options.md)の受け入れ: 設定の `[display]`(SR-21・CLI-3)と
//! `--print-config`(CLI-11・CLI-12)。config-v2(specs/_changes/2026-10-10-config-v2.md)で、検索の欄
//! `search_bar` と、タブの行の3つの値 `tabs`(always・auto・never)を `[display]` に入れた。
//! 環境変数は書き換えない(子の環境だけ)。

use mdgrid::config::{parse, Config, KEYS};
use mdgrid::display::TabsMode;
use std::process::{Command, Output};

/// 真偽の項目(名前, 既定)。`tabs` は文字列で別に確かめる。
const DEFAULTS: [(&str, bool); 6] = [
    ("row_numbers", false),
    ("zebra", false),
    ("column_lines", false),
    ("group_gap", false),
    ("search_bar", true),
    ("chips", true),
];

fn display_values(c: &Config) -> [(&'static str, bool); 6] {
    let d = c.resolved().display;
    [
        ("row_numbers", d.row_numbers),
        ("zebra", d.zebra),
        ("column_lines", d.column_lines),
        ("group_gap", d.group_gap),
        ("search_bar", d.search_bar),
        ("chips", d.chips),
    ]
}

fn tabs(c: &Config) -> TabsMode {
    c.resolved().display.tabs
}

fn read_ok(text: &str) -> Config {
    let (c, warnings) = parse(text).expect("読める");
    assert!(warnings.is_empty(), "警告: {warnings:?}\n---\n{text}");
    c
}

// ---- SR-21: [display] を読む ----

#[test]
fn test_sr_21_defaults() {
    // [SR-21] 設定が空なら row_numbers=false・zebra=false・column_lines=false・group_gap=false・tabs="always"・
    // search_bar=true・chips=true。
    let c = read_ok("");
    assert_eq!(display_values(&c), DEFAULTS);
    assert_eq!(tabs(&c), TabsMode::Always);
    assert_eq!(display_values(&Config::default()), DEFAULTS);
    // 空の [display] も既定のまま、警告なし。
    let c = read_ok("[display]\n");
    assert_eq!(display_values(&c), DEFAULTS);
}

#[test]
fn test_sr_21_reads_each_item() {
    // [SR-21] 7項目を全部既定と逆にして読む。
    let text =
        "[display]\nrow_numbers = true\nzebra = true\ncolumn_lines = true\ngroup_gap = true\n\
                tabs = \"never\"\nsearch_bar = false\nchips = false\n";
    let c = read_ok(text);
    assert_eq!(
        display_values(&c),
        [
            ("row_numbers", true),
            ("zebra", true),
            ("column_lines", true),
            ("group_gap", true),
            ("search_bar", false),
            ("chips", false),
        ]
    );
    assert_eq!(tabs(&c), TabsMode::Never);
    assert_eq!(
        tabs(&read_ok("[display]\ntabs = \"auto\"\n")),
        TabsMode::Auto
    );
    // 1つだけ書けば、ほかは既定。
    let d = read_ok("[display]\nrow_numbers = true\n")
        .resolved()
        .display;
    assert!(d.row_numbers);
    assert!(!d.zebra && !d.column_lines && d.tabs == TabsMode::Always && d.chips && d.search_bar);
}

#[test]
fn test_sr_21_display_is_a_known_item() {
    // [SR-21][CLI-12] `display.*` は設定の項目の表にある(知らない項目の警告にならない)。
    for name in [
        "row_numbers",
        "zebra",
        "column_lines",
        "group_gap",
        "tabs",
        "search_bar",
        "chips",
    ] {
        let path = format!("display.{name}");
        assert!(
            KEYS.contains(&path.as_str()),
            "KEYS に {path} が無い: {KEYS:?}"
        );
    }
    let (_, warnings) = parse("[display]\nzebra = true\n").unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn test_sr_21_wrong_type_warns_and_keeps_default() {
    // [SR-21][CLI-3] 型の違う値は警告して既定のまま。止めずに、ほかの項目は読む。
    for (name, default) in DEFAULTS {
        let text = format!("[edit]\ncandidates = 7\n\n[display]\n{name} = \"yes\"\n");
        let (c, warnings) = parse(&text).expect("型違いでも止めない");
        assert_eq!(warnings.len(), 1, "{name}: {warnings:?}");
        assert!(
            warnings[0].contains(&format!("display.{name}")),
            "警告に項目の道筋: {warnings:?}"
        );
        let got = display_values(&c)
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        assert_eq!(got, default, "{name} は既定のまま");
        assert_eq!(c.resolved().candidates, 7, "ほかの項目は読む");
    }
    // 数も型違い。tabs の知らない値も。
    let (c, warnings) = parse("[display]\nrow_numbers = 1\ntabs = \"never\"\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("display.row_numbers"), "{warnings:?}");
    assert!(!c.resolved().display.row_numbers);
    assert_eq!(tabs(&c), TabsMode::Never, "同じ表のほかの項目は読む");
    let (c, warnings) = parse("[display]\ntabs = \"sometimes\"\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(tabs(&c), TabsMode::Always);
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
    assert!(warnings[0].contains("display.foo"), "{warnings:?}");
    assert!(c.resolved().display.zebra);
}

#[test]
fn test_cli_20_old_display_names_are_moved() {
    // [CLI-20] 最上位の search_bar・view_tabs と、[display] の tabs の真偽は、警告して新しい道筋として読む。
    let (c, warnings) = parse("search_bar = false\nview_tabs = \"auto\"\n").unwrap();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(!c.resolved().display.search_bar);
    assert_eq!(tabs(&c), TabsMode::Auto);
    let (c, warnings) = parse("[display]\ntabs = false\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(tabs(&c), TabsMode::Never);
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
    // [SR-21][CLI-11] --print-config に [display] の7項目が既定値で出る(TOML として読んで判定)。
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
        display.get("tabs").and_then(|v| v.as_str()),
        Some("always"),
        "[display] の tabs\n---\n{text}"
    );
    assert_eq!(display.len(), 7, "[display] は7項目: {display:?}");
    assert!(
        !table.contains_key("search_bar"),
        "検索の欄は [display] の中"
    );
    // ほかの項目が [display] の表に紛れ込まない(読み直して警告なし・既定と同じ)。
    let (c, warnings) = parse(&text).expect("読める");
    assert!(warnings.is_empty(), "警告: {warnings:?}\n---\n{text}");
    assert_eq!(c, Config::default());
}
