//! CSV・TSV の読み書きの芯(SC-15・SC-16)。

use crate::csvfile::{append, encode, parse, set};

#[test]
fn test_sc_15_parse_quotes_newlines_bom_crlf() {
    // [SC-15] BOM・CRLF・引用符の中の区切りと改行と `""` を読む。空の行は行にしない。
    let src = "\u{feff}id,name,note\r\n1,\"東京, 本社\",\"1行目\r\n2行目\"\r\n\r\n2,大阪,\"言った \"\"はい\"\"\"\r\n";
    let f = parse(src.as_bytes(), b',').unwrap();
    assert_eq!(f.header, ["id", "name", "note"]);
    assert_eq!(f.records.len(), 2);
    assert_eq!(f.records[0].fields[1].value, "東京, 本社");
    assert_eq!(f.records[0].fields[2].value, "1行目\r\n2行目");
    assert_eq!(f.records[1].fields[2].value, "言った \"はい\"");
    assert_eq!(f.records[1].line, 5);
    assert_eq!(f.newline, "\r\n");
    assert!(f.ends_with_newline);
    // TSV。
    let t = parse(b"a\tb\n1\t2", b'\t').unwrap();
    assert_eq!(t.records[0].fields[1].value, "2");
    assert!(!t.ends_with_newline);
}

#[test]
fn test_sc_15_parse_errors() {
    // [SC-15] UTF-8 でない・空・閉じない引用符は理由。
    assert!(parse(&[0x83, 0x41, b','], b',').is_err());
    assert!(parse(b"", b',').is_err());
    assert!(parse(b"\n\n", b',').is_err());
    let e = parse(b"a,b\n1,\"open\n", b',').unwrap_err();
    assert!(e.contains('2'), "{e}");
}

#[test]
fn test_sc_16_set_changes_only_that_value() {
    // [SC-16] 1つの値を変えると、その値のバイトだけが変わる(CRLF・ほかの引用符・最後の改行の無さはそのまま)。
    let src = b"id,name,amount\r\n1,\"A\",100\r\n2,B,200";
    let f = parse(src, b',').unwrap();
    let out = set(src, &f, 1, 2, "300").unwrap();
    assert_eq!(out, b"id,name,amount\r\n1,\"A\",100\r\n2,B,300");
    // 区切りを含む値は引用符で囲む。引用符の付いた値を、引用符の要らない値にすると外れる。
    let out = set(src, &f, 0, 1, "a,b").unwrap();
    assert_eq!(out, b"id,name,amount\r\n1,\"a,b\",100\r\n2,B,200");
    let out = set(src, &f, 0, 1, "Z").unwrap();
    assert_eq!(out, b"id,name,amount\r\n1,Z,100\r\n2,B,200");
    // 無い列(列の数が足りない行)は書かない。
    let g = parse(b"a,b\n1\n", b',').unwrap();
    assert!(set(b"a,b\n1\n", &g, 0, 1, "x").is_none());
}

#[test]
fn test_sc_16_append_and_encode() {
    // [SC-16] 末尾に1行足す(改行で終わらないファイルは先に改行)。値の囲み。
    let src = b"id,name\r\n1,A";
    let f = parse(src, b',').unwrap();
    assert_eq!(
        append(src, &f, &["2".into(), "B, C".into()]),
        b"id,name\r\n1,A\r\n2,\"B, C\"\r\n"
    );
    assert_eq!(encode("a\"b", b','), "\"a\"\"b\"");
    assert_eq!(encode("plain", b','), "plain");
    assert_eq!(encode("a\tb", b'\t'), "\"a\tb\"");
}
