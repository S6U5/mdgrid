//! 画面の文言を言語ごとの台帳(locales/*.toml)に置く(SR-23)の受け入れ試験。
//! 台帳の文言の名前・並び・文が、実行ファイルの文言(`Msg`)と同じで、`src/i18n.rs` に文言の文が残っていない。
//! 記録: specs/_changes/2026-10-13-locales.md(タスク #1)。

use mdgrid::i18n::Msg;
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// 台帳の (名前, 文) を並びのとおりに。
fn ledger(code: &str) -> Vec<(String, String)> {
    let path = root().join("locales").join(format!("{code}.toml"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("台帳 {} が読めない: {e}", path.display()));
    let table: toml::Table = text.parse().expect("台帳は TOML");
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((k, _)) = line.split_once(" = ") else {
            continue;
        };
        if k.starts_with('#') || k.trim() != k {
            continue;
        }
        let v = table[k].as_str().expect("文は文字列").to_string();
        out.push((k.to_string(), v));
    }
    assert_eq!(out.len(), table.len(), "{code}: 1行に1つの文言");
    out
}

#[test]
fn test_sr_23_locales_en_matches_msgs() {
    // [SR-23] 英語の台帳の名前と並びと文が、Msg の全部と同じ。
    let en = ledger("en");
    assert_eq!(en.len(), Msg::ALL.len());
    for ((k, v), m) in en.iter().zip(Msg::ALL.iter()) {
        assert_eq!(k, &format!("{m:?}"), "並び");
        assert_eq!(v, m.en(), "{k} の英語");
    }
}

#[test]
fn test_sr_23_locales_ja_matches_msgs() {
    // [SR-23] 日本語の台帳も同じ名前・並びで、文が Msg の日本語と同じ。
    let ja = ledger("ja");
    assert_eq!(ja.len(), Msg::ALL.len());
    for ((k, v), m) in ja.iter().zip(Msg::ALL.iter()) {
        assert_eq!(k, &format!("{m:?}"), "並び");
        assert_eq!(v, m.ja(), "{k} の日本語");
    }
}

#[test]
fn test_sr_23_locales_not_in_source() {
    // [SR-23] src/i18n.rs に文言の表(`msgs!`)が残っていない。
    let src = std::fs::read_to_string(root().join("src/i18n.rs")).unwrap();
    assert!(!src.contains("msgs! {"), "文言の表は台帳へ");
    assert!(
        !src.contains("\"Usage: mdgrid"),
        "文言の文は src/i18n.rs に無い"
    );
}
