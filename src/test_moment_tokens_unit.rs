//! [BV-6] 日付の .format() の Moment の書き方(英語の既定のロケール)。specs/_changes/2026-10-06-moment-tokens.md。

use super::format_when;
use crate::types::parse_date;

fn f(date: &str, t: i64, fmt: &str) -> String {
    format_when(parse_date(date).unwrap(), t, fmt).unwrap()
}

const T1504: i64 = 15 * 3600 + 4 * 60;

#[test]
fn test_bv_6_moment_tokens_ordinal_and_12h() {
    assert_eq!(f("2026-10-05", T1504, "Do MMMM"), "5th October");
    for (d, want) in [
        ("01", "1st"),
        ("02", "2nd"),
        ("03", "3rd"),
        ("11", "11th"),
        ("12", "12th"),
        ("13", "13th"),
        ("21", "21st"),
        ("22", "22nd"),
        ("23", "23rd"),
        ("31", "31st"),
    ] {
        assert_eq!(f(&format!("2026-10-{d}"), 0, "Do"), want);
    }
    assert_eq!(f("2026-10-05", T1504, "hh:mm A"), "03:04 PM");
    assert_eq!(f("2026-10-05", T1504, "h:mm a"), "3:04 pm");
    assert_eq!(f("2026-10-05", 0, "hh A"), "12 AM");
    assert_eq!(f("2026-10-05", 12 * 3600, "h A"), "12 PM");
}

#[test]
fn test_bv_6_moment_tokens_weeks() {
    // 2026-10-05 は月曜日。ISO の第41週、日曜始まりの週も第41週。
    assert_eq!(f("2026-10-05", 0, "YYYY-[W]WW"), "2026-W41");
    assert_eq!(f("2026-10-05", 0, "gggg-[W]ww"), "2026-W41");
    // 2027-01-01 は金曜日: ISO では 2026 年の第53週、日曜始まりでは 2027 年の第1週。
    assert_eq!(f("2027-01-01", 0, "GGGG-[W]WW"), "2026-W53");
    assert_eq!(f("2027-01-01", 0, "gggg-[W]ww"), "2027-W01");
    // 2025-12-29(月曜日)は ISO では 2026 年の第1週。
    assert_eq!(f("2025-12-29", 0, "GGGG W"), "2026 1");
}

#[test]
fn test_bv_6_moment_tokens_misc() {
    assert_eq!(f("2026-10-05", 0, "Q"), "4");
    assert_eq!(f("2026-10-05", 0, "dd d E e"), "Mo 1 1 1");
    assert_eq!(f("2026-10-04", 0, "dd d E e"), "Su 0 7 0");
    assert_eq!(f("2026-02-01", 0, "DDDD DDD"), "032 32");
    assert_eq!(f("1970-01-02", 0, "X x SSS"), "86400 86400000 000");
    // 知らない文字と [...] はそのまま。
    assert_eq!(
        f("2026-10-05", T1504, "YYYY-MM-DDTHH:mm [Do]"),
        "2026-10-05T15:04 Do"
    );
}
