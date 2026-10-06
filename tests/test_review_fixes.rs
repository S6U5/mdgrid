//! 書き戻しのレビュー(2026-09-30)で見つかった形の受け入れテスト(WB-5・WB-7・CE-18)。仕様: specs/write-back/spec.md。

use mdgrid::frontmatter::{parse, ReadOnly};
use mdgrid::writeback::{apply, Edit, EditError, NewValue};

fn edit(key: &str, v: &str) -> Edit {
    Edit {
        key: key.to_string(),
        value: NewValue::Str(v.to_string()),
    }
}

// [WB-5] タブで字下げした続きの行は YAML として不正なので、読むだけにする。
#[test]
fn wb_5_tab_indented_list_is_invalid_yaml() {
    let src = b"---\nstatus: todo\ntags:\n\t- x\n---\nbody\n";
    assert_eq!(parse(src), Err(ReadOnly::InvalidYaml));
    assert!(matches!(
        apply(src, &[edit("status", "done")]),
        Err(EditError::ReadOnly(ReadOnly::InvalidYaml))
    ));
}

// [WB-5] タブで字下げした入れ子の対応も同じ。
#[test]
fn wb_5_tab_indented_mapping_is_invalid_yaml() {
    let src = b"---\nstatus: todo\nm:\n\tk: v\n---\n";
    assert_eq!(parse(src), Err(ReadOnly::InvalidYaml));
}

// [CE-18] フロー形式のリストに文字列を当てると書き方が変わるので、書かない(書き方を保つ)。
#[test]
fn ce_18_string_onto_flow_list_is_not_editable() {
    let src = b"---\ntags: [a, b]\n---\n";
    assert!(matches!(
        apply(src, &[edit("tags", "x")]),
        Err(EditError::NotEditable(_))
    ));
}
