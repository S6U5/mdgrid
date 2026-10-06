//! [CE-19] リストの列で1つの文字列で書かれた値(`tags: x`)は1つの要素のリストとして開き、
//! 確定したらフローのリストに書き換える。specs/_changes/2026-10-07-scalar-list-edit.md。

use mdgrid::writeback::{apply, Edit, EditError, NewValue};

fn put(src: &str, items: &[&str]) -> Result<String, EditError> {
    apply(
        src.as_bytes(),
        &[Edit {
            key: "tags".into(),
            value: NewValue::List(items.iter().map(|s| s.to_string()).collect()),
        }],
    )
    .map(|b| String::from_utf8(b).unwrap())
}

#[test]
fn test_ce_19_scalar_list_is_rewritten_as_flow_list() {
    // 足す: 値だけが `[..]` になり、行のほかのバイト(行末のコメント・CRLF)はそのまま。
    assert_eq!(
        put("---\ntags: x\nb: 1\n---\nbody\n", &["x", "w"]).unwrap(),
        "---\ntags: [x, w]\nb: 1\n---\nbody\n"
    );
    assert_eq!(
        put("---\r\ntags: x # memo\r\n---\r\n", &["x", "w"]).unwrap(),
        "---\r\ntags: [x, w] # memo\r\n---\r\n"
    );
    // 元の引用符の書き方は残す(WB-7)。新しい要素の書き方は WB-7 のとおり。
    assert_eq!(
        put("---\ntags: 'a b'\n---\n", &["a b", "no"]).unwrap(),
        "---\ntags: ['a b', \"no\"]\n---\n"
    );
    // 外して別のものに替える・全部外す(CE-9 と同じく `key:` を残す)。
    assert_eq!(
        put("---\ntags: x\n---\n", &["z"]).unwrap(),
        "---\ntags: [z]\n---\n"
    );
    assert_eq!(
        put("---\ntags: x\n---\n", &[]).unwrap(),
        "---\ntags:\n---\n"
    );
    // フローで書けない文字の元の要素は引用符で囲む。
    assert_eq!(
        put("---\ntags: a,b\n---\n", &["a,b", "c"]).unwrap(),
        "---\ntags: [\"a,b\", c]\n---\n"
    );
}

#[test]
fn test_ce_19_scalar_list_non_string_stays_read_only() {
    // 文字列でない1つの値とブロックの文字は書かない(書くと型か書き方が変わる)。
    for src in [
        "---\ntags: 2024\n---\n",
        "---\ntags: true\n---\n",
        "---\ntags: |\n  x\n---\n",
    ] {
        assert!(
            matches!(put(src, &["x", "w"]), Err(EditError::NotEditable(_))),
            "{src:?}"
        );
    }
}
