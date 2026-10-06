//! [WB-1][WB-2] キーの名前の変更はキーの文字だけ、キーの削除はキーの行と値の続きの行だけを変える(CE-29)。
//! specs/_changes/2026-10-06-key-rename-delete.md。

use super::{apply, Edit, EditError};
use crate::source::NewValue;

fn rename(src: &str, from: &str, to: &str) -> Result<String, EditError> {
    apply(
        src.as_bytes(),
        &[Edit {
            key: from.into(),
            value: NewValue::RenameKey(to.into()),
        }],
    )
    .map(|b| String::from_utf8(b).unwrap())
}

fn delete(src: &str, key: &str) -> Result<String, EditError> {
    apply(
        src.as_bytes(),
        &[Edit {
            key: key.into(),
            value: NewValue::DeleteKey,
        }],
    )
    .map(|b| String::from_utf8(b).unwrap())
}

#[test]
fn test_wb_1_key_ops_rename_keeps_other_bytes() {
    let src = "---\na: 1\nb: [x, y]\nc0: z\n---\nbody\n";
    assert_eq!(
        rename(src, "b", "c").unwrap(),
        "---\na: 1\nc: [x, y]\nc0: z\n---\nbody\n"
    );
    // ブロックのリスト・行末のコメント・CRLF もそのまま。
    let src = "---\r\ntags: # t\r\n  - a\r\n  - b\r\nx: 1 # c\r\n---\r\n";
    assert_eq!(
        rename(src, "tags", "labels").unwrap(),
        "---\r\nlabels: # t\r\n  - a\r\n  - b\r\nx: 1 # c\r\n---\r\n"
    );
    // 引用符で囲んだキー。
    let src = "---\n\"my key\": 1\n---\n";
    assert_eq!(rename(src, "my key", "k").unwrap(), "---\nk: 1\n---\n");
    // 書き方の要る新しい名前は囲む。
    let out = rename("---\na: 1\n---\n", "a", "b: c").unwrap();
    assert!(
        out.starts_with("---\n\"b: c\": 1\n") || out.starts_with("---\n'b: c': 1\n"),
        "{out}"
    );
}

#[test]
fn test_wb_1_key_ops_rename_refuses_existing_name() {
    assert!(rename("---\na: 1\nb: 2\n---\n", "a", "b").is_err());
    assert!(rename("---\na: 1\n---\n", "a", "").is_err());
    assert!(rename("---\na: 1\n---\n", "a", "x\ny").is_err());
}

#[test]
fn test_wb_1_key_ops_delete_block_list_keeps_next_comment() {
    let src = "---\ntitle: t\ntags:\n  - x\n  - y\n\n# about b\nb: 2\n---\nbody\n";
    assert_eq!(
        delete(src, "tags").unwrap(),
        "---\ntitle: t\n\n# about b\nb: 2\n---\nbody\n"
    );
}

#[test]
fn test_wb_1_key_ops_delete_multiline_and_last() {
    let src = "---\nnote: |\n  l1\n  l2\nb: 1 # c\n---\n";
    assert_eq!(delete(src, "note").unwrap(), "---\nb: 1 # c\n---\n");
    // 最後のキーを消すと空のフロントマター。
    assert_eq!(
        delete("---\nb: 1\n---\nbody\n", "b").unwrap(),
        "---\n---\nbody\n"
    );
    // CRLF。
    assert_eq!(
        delete("---\r\na: 1\r\nb: 2\r\n---\r\n", "a").unwrap(),
        "---\r\nb: 2\r\n---\r\n"
    );
    // 無いキーは何も変えない。
    assert_eq!(
        delete("---\na: 1\n---\n", "zz").unwrap(),
        "---\na: 1\n---\n"
    );
}

#[test]
fn test_wb_1_key_ops_review_cases() {
    // アンカーを使われているキーは消さない(読めない YAML にしない)。
    assert!(delete("---\nbase: &b 1\nref: *b\n---\n", "base").is_err());
    // 大文字小文字だけ違う名前にも重ねない。
    assert!(rename("---\nTags: 1\ntags: 2\n---\n", "Tags", "TAGS").is_err());
    // ブロックの文字の `#` で始まる行も値の中身として消す。
    assert_eq!(
        delete("---\nd: |\n  l1\n  # text\nx: 1\n---\n", "d").unwrap(),
        "---\nx: 1\n---\n"
    );
    // 全部を消す削除と、ほかの値の直しが一緒でも書ける(値の直しを先に当てる)。
    let out = apply(
        b"---\na: 1\n---\n",
        &[
            Edit {
                key: "a".into(),
                value: NewValue::DeleteKey,
            },
            Edit {
                key: "b".into(),
                value: NewValue::Str("x".into()),
            },
        ],
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "---\nb: x\n---\n");
}
