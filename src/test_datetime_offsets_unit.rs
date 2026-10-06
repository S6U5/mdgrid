//! [CV-2][CE-2][BV-6] Z・時差・空白の区切り・小数の秒の日時を日時として読み、時差は地域の時刻に直す。
//! specs/_changes/2026-10-06-datetime-offsets.md。

use super::{date_or_datetime, fits, infer, parse_date, parse_datetime, parse_datetime_at, Kind};
use crate::frontmatter::Value;

fn s(x: &str) -> Value {
    Value::Str(x.into())
}

#[test]
fn test_cv_2_datetime_offsets_fit() {
    for v in [
        "2026-10-05T21:44:59Z",
        "2026-10-06T10:00:00+09:00",
        "2026-10-06T10:00:00.123+02:00",
        "2026-10-06T10:00+0900",
        "2026-10-06T10:00-05",
        "2026-10-06 10:00:00",
        "2026-10-06 10:00",
    ] {
        assert!(fits(Kind::DateTime, &s(v)), "{v}");
        assert!(parse_datetime(v).is_some(), "{v}");
    }
    for v in [
        "2026-10-06T25:00",
        "2026-10-06T10:00+25:00",
        "2026-10-06T10:00Q",
        "2026-10-06 x",
    ] {
        assert!(!fits(Kind::DateTime, &s(v)), "{v}");
    }
    assert_eq!(infer([s("2026-10-05T21:44:59Z")].iter()), Kind::DateTime);
}

#[test]
fn test_cv_2_datetime_offsets_convert_to_local() {
    let jst = 9 * 3600;
    let day = parse_date("2026-10-06").unwrap() * 86_400;
    // UTC の 01:00 は +09:00 の地域で 10:00。
    assert_eq!(
        parse_datetime_at("2026-10-06T01:00:00Z", jst),
        Some(day + 10 * 3600)
    );
    assert_eq!(
        parse_datetime_at("2026-10-06T10:00:00+09:00", jst),
        Some(day + 10 * 3600)
    );
    // New York(-05:00 の値)は +09:00 の地域で翌日の 00:00。
    assert_eq!(
        parse_datetime_at("2026-10-06T10:00-05", jst),
        Some(day + 24 * 3600)
    );
    // 時差の無い値は地域の時刻のまま。
    assert_eq!(
        parse_datetime_at("2026-10-06 10:00", jst),
        Some(day + 10 * 3600)
    );
}

#[test]
fn test_cv_2_datetime_offsets_write_shape_unchanged() {
    // 書くときに囲まない形(WB-18)は今までどおり `T` の区切りで時差なしだけ。
    assert!(date_or_datetime("2026-10-06T10:00"));
    assert!(!date_or_datetime("2026-10-06T10:00Z"));
    assert!(!date_or_datetime("2026-10-06 10:00"));
}
