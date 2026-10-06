//! [BV-6][CE-21] 今日と今は地域の時刻で決める(specs/_changes/2026-10-06-local-today.md)。
//! 時計と時差を渡せる純関数 `print::today_now_at` で確かめる(試験を回す端末の時差に依らない)。

use super::today_now_at;
use crate::types::parse_date;

fn day(s: &str) -> i64 {
    parse_date(s).unwrap()
}

/// 2026-10-05T18:21:00Z の UNIX 秒。
const OCT5_1821_UTC: i64 = 1_791_224_460;

#[test]
fn test_bv_6_today_uses_local_offset_east() {
    // UTC+9(東京)では、UTC の 10月5日 18:21 は地域の 10月6日 03:21。今日は 10月6日。
    let (today, now) = today_now_at(OCT5_1821_UTC, 9 * 3600, |_| None);
    assert_eq!(today, day("2026-10-06"));
    assert_eq!(now, OCT5_1821_UTC + 9 * 3600, "今は地域の時計の秒");
}

#[test]
fn test_bv_6_today_uses_local_offset_west() {
    // UTC−7 では、UTC の 10月6日 03:00 は地域の 10月5日 20:00。今日は 10月5日。
    let utc = OCT5_1821_UTC + 8 * 3600 + 39 * 60; // 2026-10-06T03:00:00Z
    let (today, _) = today_now_at(utc, -7 * 3600, |_| None);
    assert_eq!(today, day("2026-10-05"));
}

#[test]
fn test_bv_6_today_utc_without_offset() {
    let (today, now) = today_now_at(OCT5_1821_UTC, 0, |_| None);
    assert_eq!(today, day("2026-10-05"));
    assert_eq!(now, OCT5_1821_UTC);
}

#[test]
fn test_bv_6_mdgrid_today_wins_over_clock() {
    let env = |k: &str| (k == "MDGRID_TODAY").then(|| "2026-12-24".to_string());
    let (today, now) = today_now_at(OCT5_1821_UTC, 9 * 3600, env);
    assert_eq!(today, day("2026-12-24"));
    assert_eq!(now, today * 86_400);
}
