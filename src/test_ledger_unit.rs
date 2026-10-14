//! 文言の台帳の読み方と確かめ方(SR-23)。build.rs と同じ関数で、欠け・余り・空・差し込みの番号の食い違いを
//! 誤りにする(ビルドはこの誤りで落ちる)。記録: specs/_changes/2026-10-13-locales.md(タスク #2)。

use super::*;

const EN: &str = "# 区切り\n## Hello の説明\nHello = \"Hello {0}\"\nBye = \"Bye\"\n";

fn en() -> Vec<Entry> {
    parse("en.toml", EN).unwrap()
}

#[test]
fn test_sr_23_ledger_reads_keys_docs_in_order() {
    // [SR-23] 名前・説明(`##`)・文を行の順に読む。`#` の行は区切りで説明にしない。
    let e = en();
    assert_eq!(e.len(), 2);
    assert_eq!(
        (e[0].key.as_str(), e[0].text.as_str()),
        ("Hello", "Hello {0}")
    );
    assert_eq!(e[0].doc, vec!["Hello の説明".to_string()]);
    assert_eq!(e[1].key, "Bye");
    assert!(e[1].doc.is_empty());
}

#[test]
fn test_sr_23_ledger_complete_has_no_problems() {
    // [SR-23] 同じ名前と番号の台帳は誤りなし。
    let ja = parse(
        "ja.toml",
        "Hello = \"こんにちは {0}\"\nBye = \"さようなら\"\n",
    )
    .unwrap();
    assert!(check_base("en.toml", &en()).is_empty());
    assert!(check(&en(), "ja.toml", &ja).is_empty());
}

#[test]
fn test_sr_23_ledger_missing_and_extra() {
    // [SR-23] 欠け・余り → 名前を示す誤り。
    let ja = parse("ja.toml", "Hello = \"こんにちは {0}\"\nExtra = \"余り\"\n").unwrap();
    let p = check(&en(), "ja.toml", &ja);
    assert!(
        p.iter().any(|x| x.contains("Bye") && x.contains("missing")),
        "{p:?}"
    );
    assert!(p.iter().any(|x| x.contains("Extra")), "{p:?}");
}

#[test]
fn test_sr_23_ledger_placeholder_mismatch() {
    // [SR-23] 差し込みの番号の組が英語と違う → 誤り。
    let ja = parse(
        "ja.toml",
        "Hello = \"こんにちは {1}\"\nBye = \"さようなら\"\n",
    )
    .unwrap();
    let p = check(&en(), "ja.toml", &ja);
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(
        p[0].contains("Hello") && p[0].contains("placeholders"),
        "{p:?}"
    );
}

#[test]
fn test_sr_23_ledger_empty_and_bad_lines() {
    // [SR-23] 空の文・名前の形・2つある名前・1行に1つでない書き方 → 誤り。
    let ja = parse("ja.toml", "Hello = \" \"\nBye = \"x\"\n").unwrap();
    assert!(check(&en(), "ja.toml", &ja)
        .iter()
        .any(|x| x.contains("empty")));
    let bad = parse("en.toml", "lower = \"x\"\n").unwrap();
    assert!(!check_base("en.toml", &bad).is_empty());
    assert!(
        parse("en.toml", "A = \"x\"\nA = \"y\"\n").is_err(),
        "TOML の重なりも読めない"
    );
    assert!(
        parse("en.toml", "A = \"\"\"x\ny\"\"\"\n").is_err(),
        "複数行の文は書かない"
    );
    assert!(parse("en.toml", "A = 1\n").is_err());
}

#[test]
fn test_sr_23_ledger_placeholders() {
    // [SR-23] `{数字}` だけを番号に数える。
    let p = placeholders("{0} {12} {x} {} {3");
    assert_eq!(p.into_iter().collect::<Vec<_>>(), vec![0, 12]);
}
