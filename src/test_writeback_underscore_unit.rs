//! `_` を除くと数に読める文字の値は、囲んで書く(WB-7)。
use super::*;

#[test]
fn test_wb_7_underscore_numbers_are_quoted() {
    // [WB-7] `_0`・`1_0`・`_1.5` は囲む。`_a`・`snake_case` はそのまま。
    for s in ["_0", "1_0", "_1.5", "0_"] {
        assert!(!plain_safe(s), "{s}");
    }
    for s in ["_a", "snake_case", "a_1b"] {
        assert!(plain_safe(s), "{s}");
    }
}
