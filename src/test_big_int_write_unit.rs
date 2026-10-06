//! [CV-2][WB-3] 整数の形の文字は、i64 に収まらなくても引用符で書く(ほかの YAML の読み手が数に読まない)。
//! specs/_changes/2026-10-06-trust-notices.md。

use super::plain_safe;

#[test]
fn test_cv_2_big_int_text_stays_quoted() {
    assert!(!plain_safe("123456789012345678901234"));
    assert!(!plain_safe("0xffffffffffffffffffff"));
    assert!(!plain_safe("42"));
    assert!(plain_safe("v42"));
}
