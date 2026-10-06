//! フロントマターの読み取りの受け入れテスト(CE-8・WB-5)。仕様: specs/cell-edit/spec.md、specs/write-back/spec.md。

use mdgrid::frontmatter::{parse, Entry, Frontmatter, ReadOnly, Shape, Value};

fn fixture(name: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn entry<'a>(fm: &'a Frontmatter, key: &str) -> &'a Entry {
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("key {key} not found"))
}

fn assert_read_only_entry(file: &str, key: &str, shape: Shape) {
    let fm = parse(&fixture(file)).expect("parse");
    let e = entry(&fm, key);
    assert_eq!(e.shape, shape, "{file}: {key}");
    assert_eq!(e.span, None, "{file}: {key} must have no span");
}

#[test]
fn ce_8_nested_map_is_read_only() {
    // [CE-8]
    assert_read_only_entry("nested_map.md", "meta", Shape::Nested);
    // 隣の第1階層のキーは書ける。
    let fm = parse(&fixture("nested_map.md")).unwrap();
    assert!(entry(&fm, "status").span.is_some());
}

#[test]
fn ce_8_block_scalars_are_read_only() {
    // [CE-8]
    assert_read_only_entry("block_scalar.md", "literal", Shape::BlockScalar);
    assert_read_only_entry("block_scalar.md", "folded", Shape::BlockScalar);
    let fm = parse(&fixture("block_scalar.md")).unwrap();
    assert!(entry(&fm, "status").span.is_some());
}

#[test]
fn ce_8_anchor_and_alias_are_read_only() {
    // [CE-8]
    assert_read_only_entry("anchor.md", "base", Shape::Anchor);
    assert_read_only_entry("anchor.md", "alias", Shape::Anchor);
}

#[test]
fn ce_18_block_list_span_only_when_writable() {
    // [CE-18] 書ける形のブロックのリスト(要素が1行の素のスカラーか引用符つきの値だけ)には、要素の行の範囲がある。
    let bytes = fixture("block_list.md");
    let fm = parse(&bytes).unwrap();
    let e = entry(&fm, "tags");
    assert_eq!(e.shape, Shape::BlockList);
    assert_eq!(
        e.value,
        Value::List(vec![Value::Str("one".into()), Value::Str("two".into())])
    );
    let span = e.span.clone().expect("writable block list has a span");
    let text = std::str::from_utf8(&bytes[span]).unwrap();
    assert!(text.starts_with("  - one"), "{text:?}");
    assert!(text.contains("  - two"), "{text:?}");
    assert!(
        !text.contains("status"),
        "span stays inside the list: {text:?}"
    );

    // [CE-8] 書けない形(要素の間のコメント・空行、入れ子の要素)には範囲が無い。値は読める。
    for src in [
        "---\ntags:\n  - one\n  # memo\n  - two\nstatus: draft\n---\n",
        "---\ntags:\n  - one\n\n  - two\nstatus: draft\n---\n",
    ] {
        let fm = parse(src.as_bytes()).expect("parse");
        let e = entry(&fm, "tags");
        assert_eq!(e.shape, Shape::BlockList, "{src:?}");
        assert_eq!(e.span, None, "{src:?} must have no span");
        assert_eq!(
            e.value,
            Value::List(vec![Value::Str("one".into()), Value::Str("two".into())]),
            "{src:?}"
        );
        assert!(entry(&fm, "status").span.is_some(), "{src:?}: neighbour");
    }
    for src in [
        "---\ntags:\n  - {k: v}\n---\n",
        "---\ntags:\n  - one\n  - k: v\n---\n",
        "---\ntags:\n  - - x\n---\n",
    ] {
        let fm = parse(src.as_bytes()).expect("parse");
        assert_eq!(entry(&fm, "tags").span, None, "{src:?} must have no span");
    }
}

#[test]
fn ce_8_editable_shapes_have_value_span() {
    // [CE-8]
    let bytes = fixture("quoted.md");
    let fm = parse(&bytes).unwrap();
    let dq = entry(&fm, "dq");
    assert_eq!(dq.shape, Shape::DoubleQuoted);
    assert_eq!(&bytes[dq.span.clone().unwrap()], b"\"draft\"");
    assert_eq!(dq.value, Value::Str("draft".into()));
    let sq = entry(&fm, "sq");
    assert_eq!(sq.shape, Shape::SingleQuoted);
    assert_eq!(&bytes[sq.span.clone().unwrap()], b"'draft'");

    let bytes = fixture("trailing_comment.md");
    let fm = parse(&bytes).unwrap();
    let st = entry(&fm, "status");
    assert_eq!(st.shape, Shape::Plain);
    // 値の範囲に行末のコメントは含まない。
    assert_eq!(&bytes[st.span.clone().unwrap()], b"draft");

    let bytes = fixture("flow_list.md");
    let fm = parse(&bytes).unwrap();
    let tags = entry(&fm, "tags");
    assert_eq!(tags.shape, Shape::FlowList);
    assert_eq!(&bytes[tags.span.clone().unwrap()], b"[one, two]");
}

#[test]
fn wb_5_duplicate_key_is_read_only() {
    // [WB-5]
    assert_eq!(
        parse(&fixture("duplicate_key.md")),
        Err(ReadOnly::DuplicateKey("a".into()))
    );
}

#[test]
fn wb_5_bom_is_read_only() {
    // [WB-5]
    assert_eq!(parse(&fixture("bom.md")), Err(ReadOnly::Bom));
}

#[test]
fn wb_5_invalid_yaml_is_read_only() {
    // [WB-5]
    assert_eq!(
        parse(&fixture("invalid_yaml.md")),
        Err(ReadOnly::InvalidYaml)
    );
}

#[test]
fn wb_5_unclosed_is_read_only() {
    // [WB-5]
    assert_eq!(parse(&fixture("unclosed.md")), Err(ReadOnly::Unclosed));
}

#[test]
fn wb_5_not_utf8_is_read_only() {
    // [WB-5]
    assert_eq!(parse(&fixture("not_utf8.md")), Err(ReadOnly::NotUtf8));
}

#[test]
fn wb_5_mixed_newlines_is_read_only() {
    // [WB-5]
    assert_eq!(
        parse(&fixture("mixed_newlines.md")),
        Err(ReadOnly::MixedNewlines)
    );
}

#[test]
fn wb_3_empty_frontmatter_is_detected_and_writable() {
    // [WB-3] 空のフロントマター(`---` が2行だけ)は parse で EmptyFrontmatter と見分ける(読むだけではない)。
    // 既定では書け、2つの区切りの間にキーの行を足す。
    let bytes = fixture("empty_frontmatter.md");
    assert_eq!(parse(&bytes), Err(ReadOnly::EmptyFrontmatter));
    let out = mdgrid::writeback::apply(
        &bytes,
        &[mdgrid::writeback::Edit {
            key: "status".into(),
            value: mdgrid::writeback::NewValue::Str("todo".into()),
        }],
    )
    .expect("[WB-3] empty frontmatter is writable");
    assert_eq!(out, b"---\nstatus: todo\n---\nbody\n");
}

#[test]
fn wb_3_no_frontmatter_is_detected() {
    // [WB-3]
    assert_eq!(
        parse(&fixture("no_frontmatter.md")),
        Err(ReadOnly::NoFrontmatter)
    );
}

#[test]
fn wb_2_end_points_at_first_closing_delimiter() {
    // [WB-2]
    let bytes = fixture("dash_line.md");
    let fm = parse(&bytes).unwrap();
    let head = b"---\ntitle: t\nstatus: draft\n";
    assert_eq!(fm.end, head.len());
    assert!(bytes[fm.end..].starts_with(b"---\nbody\n"));
    // 本文の中の `status:` の行はキーとして読まない。
    assert_eq!(entry(&fm, "status").value, Value::Str("draft".into()));
    assert_eq!(fm.entries.len(), 2);
}

#[test]
fn wb_1_crlf_and_japanese_key_spans() {
    // [WB-1]
    let bytes = fixture("crlf.md");
    let fm = parse(&bytes).unwrap();
    let st = entry(&fm, "status");
    // 値の範囲に \r を含まない。
    assert_eq!(&bytes[st.span.clone().unwrap()], b"draft");

    let bytes = fixture("japanese_key.md");
    let fm = parse(&bytes).unwrap();
    let e = entry(&fm, "状態");
    assert_eq!(e.value, Value::Str("進行中".into()));
    assert_eq!(&bytes[e.span.clone().unwrap()], "進行中".as_bytes());
}

#[test]
fn ce_2_quoted_number_like_values_are_strings() {
    // [CE-2]
    let fm = parse(&fixture("number_like.md")).unwrap();
    assert_eq!(entry(&fm, "num").value, Value::Str("123".into()));
    assert_eq!(entry(&fm, "no_str").value, Value::Str("no".into()));
    assert_eq!(entry(&fm, "count").value, Value::Int(5));
    assert_eq!(entry(&fm, "flag").value, Value::Bool(true));

    let fm = parse(&fixture("special_chars.md")).unwrap();
    assert_eq!(entry(&fm, "colon").value, Value::Str("a: b".into()));
    assert_eq!(entry(&fm, "hash").value, Value::Str("a #b".into()));
}
