//! [cell-view.type-mismatch][write-back.add-key-and-frontmatter] 整数の形の文字は、i64 に収まらなくても引用符で書く(ほかの YAML の読み手が数に読まない)。
//! specs/_history/decidespec/_changes/2026-10-06-trust-notices.md。

use super::plain_safe;

// [utest -> req~cell-view.type-mismatch~1]
#[test]
fn test_cv_2_big_int_text_stays_quoted() {
    assert!(!plain_safe("123456789012345678901234"));
    assert!(!plain_safe("0xffffffffffffffffffff"));
    assert!(!plain_safe("42"));
    assert!(plain_safe("v42"));
}
