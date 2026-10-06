//! [CV-2] 64ビットに入らない整数は、元の文字のまま読む(在る値をマップの印・null にしない)。
//! specs/_changes/2026-10-06-trust-notices.md。

use super::{resolve_plain, Value};

#[test]
fn test_cv_2_big_int_kept_as_text() {
    for s in [
        "123456789012345678901234",
        "-99999999999999999999",
        "+18446744073709551616",
        "0xffffffffffffffffffff",
        "0o7777777777777777777777777",
    ] {
        assert_eq!(resolve_plain(s), Value::Str(s.to_string()), "{s}");
    }
}

#[test]
fn test_cv_2_big_int_boundary_still_int() {
    assert_eq!(resolve_plain("9223372036854775807"), Value::Int(i64::MAX));
    assert_eq!(resolve_plain("-9223372036854775808"), Value::Int(i64::MIN));
    assert_eq!(
        resolve_plain("9223372036854775808"),
        Value::Str("9223372036854775808".into())
    );
}
