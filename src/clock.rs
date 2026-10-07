//! 時計: 地域の時刻の UTC からの差と、今日と今(BV-6)。式(expr)・型(types)・出力(print)・画面が使う。
//! 出力のモジュールに置いていたものを、型と式が出力に頼らないように分けた。

use crate::types;

/// 地域の時刻の UTC からの差(秒)。起動の最初に `set_local_offset` で1回だけ決める(決めなければ UTC)。
static LOCAL_OFFSET: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// 決めた地域の時差(秒。決めていなければ 0)。
pub fn local_offset() -> i64 {
    LOCAL_OFFSET.load(std::sync::atomic::Ordering::Relaxed)
}

/// 地域の時差を決める(main の先頭で、`time` の `UtcOffset::current_local_offset` から)。
pub fn set_local_offset(secs: i64) {
    LOCAL_OFFSET.store(secs, std::sync::atomic::Ordering::Relaxed);
}

/// 今日(1970-01-01 からの日数)と今(地域の時計の秒)。`MDGRID_TODAY`(`YYYY-MM-DD`)があれば今日はそれ、
/// 今はその日の 0 時(試験で式の結果を固定する)。無ければ時計と地域の時差(`set_local_offset`)。
pub fn today_now(env: impl Fn(&str) -> Option<String>) -> (i64, i64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    today_now_at(
        now,
        LOCAL_OFFSET.load(std::sync::atomic::Ordering::Relaxed),
        env,
    )
}

/// `today_now` の中身(時計 `now_utc`(UNIX 秒)と時差 `offset`(秒)を渡せる)。時刻の付いた値は
/// Obsidian と同じく地域の時刻として読むので、今も地域の時計の秒(UNIX 秒 + 時差)で返す。
pub fn today_now_at(now_utc: i64, offset: i64, env: impl Fn(&str) -> Option<String>) -> (i64, i64) {
    if let Some(d) = env("MDGRID_TODAY").and_then(|s| types::parse_date(s.trim())) {
        return (d, d * 86_400);
    }
    let local = now_utc + offset;
    (local.div_euclid(86_400), local)
}
