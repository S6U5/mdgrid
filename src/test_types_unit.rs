use super::*;

fn st(v: &str) -> Value {
    Value::Str(v.to_string())
}

#[test]
fn non_ascii_after_time_does_not_panic() {
    // 「byte index 5 is not a char boundary」の入力(レビューの指摘)。
    for s in [
        "2026-10-01T10:0é",
        "2026-10-01T1é:00",
        "2026-10-01Té",
        "2026-10-01T10:00:0é",
        "2026-10-01T10:00:00.é",
        "2026-10-0é",
        "é026-10-01",
        "2026-1é-01",
        "2026-10-01é",
    ] {
        assert_eq!(infer([st(s)].iter()), Kind::Text, "{s}");
        for k in [Kind::Date, Kind::DateTime] {
            assert!(!fits(k, &st(s)), "{k:?} {s}");
        }
        assert_eq!(parse_date(s), None, "{s}");
    }
}

#[test]
fn datetime_shapes() {
    assert_eq!(
        infer([st("2026-10-01T10:00:00.123")].iter()),
        Kind::DateTime
    );
    assert!(fits(Kind::DateTime, &st("2026-10-01T23:59:59")));
    assert!(fits(Kind::DateTime, &st("2026-10-01")));
    assert!(!fits(Kind::DateTime, &st("2026-10-01T24:00")));
    assert!(!fits(Kind::DateTime, &st("2026-02-30T10:00")));
    assert!(!fits(Kind::DateTime, &st("2026-10-01T10:00:00.")));
    assert!(!fits(Kind::DateTime, &st("2026-10-01T10:00:")));
}

#[test]
fn duplicate_types_key_takes_the_later() {
    let m =
        read_types_json(br#"{"types": {"a": "date"}, "types": {"a": "number", "a": "checkbox"}}"#);
    assert_eq!(m.get("a"), Some(&Kind::Checkbox));
    assert_eq!(m.len(), 1);
}

#[test]
fn json_rejects_bad_numbers_and_escapes() {
    assert!(read_types_json(br#"{"types": {"a": "date"}, "x": 01}"#).is_empty());
    assert!(read_types_json(br#"{"types": {"a": "date"}, "x": "\q"}"#).is_empty());
    assert!(read_types_json(br#"{"types": {"a": "date"},}"#).is_empty());
    let m = read_types_json(br#"{"x": [1, -2.5e3, true, null], "types": {"a": "date"}}"#);
    assert_eq!(m.get("a"), Some(&Kind::Date));
}

fn day(s: &str) -> i64 {
    parse_date(s).unwrap()
}

#[test]
fn date_format_rejects_ambiguous_and_duplicate_parts() {
    // [CE-22] M・D を数の部品と区切り無しで並べる形、部品の重ね、知らない英字は Err。
    for p in [
        "MD",
        "YYYYM/D",
        "MM/DD/DD",
        "YYYY YY-MM-DD",
        "MM/DD ddd ddd",
        "YYY-MM-DD",
        "dd/MM",
    ] {
        assert!(DateFormat::parse(p).is_err(), "{p}");
    }
    // 0で埋める部品どうしは区切り無しでよい。
    let f = DateFormat::parse("YYYYMMDD").unwrap();
    assert_eq!(f.format(day("2026-01-05")), "20260105");
    assert_eq!(f.read("20260105", 0), Some(Ok(day("2026-01-05"))));
    assert_eq!(f.pattern(), "YYYYMMDD");
}

#[test]
fn date_input_weekday_mismatch_and_century() {
    // [CE-22] 曜日の食い違いは Err。YY は today の世紀。
    let f = DateFormat::parse("YYYY/MM/DD (ddd)").unwrap();
    let today = day("2026-10-02");
    assert!(parse_date_input("2026/11/03 (水)", &f, today).is_err());
    let f = DateFormat::parse("YY-MM-DD").unwrap();
    assert_eq!(
        parse_date_input("99-01-02", &f, day("1999-05-05")),
        Ok(Some(day("1999-01-02")))
    );
    // 範囲の外の今日からの日数は Err。
    assert!(parse_date_input("+99999999", &DateFormat::iso(), today).is_err());
    // 理由は設定の形を案内する。
    let e = parse_date_input("x", &DateFormat::parse("M/D").unwrap(), today).unwrap_err();
    assert!(e.contains("M/D") && e.contains("YYYY-MM-DD"), "{e}");
}

#[test]
fn month_grid_and_add_months_edges() {
    // [CE-21] 知らない月は空。範囲の端を越える月の足し引きは端で止まる。
    assert!(month_grid(2026, 0, WeekStart::Sun).is_empty());
    assert!(month_grid(2026, 13, WeekStart::Mon).is_empty());
    // 2026-02 は日曜始まりでちょうど4週。
    assert_eq!(month_grid(2026, 2, WeekStart::Sun).len(), 4);
    assert_eq!(add_months(LAST_DAY, 1), LAST_DAY);
    assert_eq!(add_months(FIRST_DAY, -1), FIRST_DAY);
    assert_eq!(add_months(day("2026-10-05"), i32::MAX), LAST_DAY);
    assert_eq!(add_months(LAST_DAY + 10, 1), LAST_DAY + 10);
    // 遠い日数の曜日・見せ方でも落ちない。
    let _ = DateFormat::parse("YY/M/D ddd").unwrap().format(i64::MIN);
}

#[test]
fn date_range_bounds() {
    assert_eq!(parse_date("0000-01-01"), Some(FIRST_DAY));
    assert_eq!(parse_date("9999-12-31"), Some(LAST_DAY));
    assert!(date_in_range(FIRST_DAY) && date_in_range(LAST_DAY) && date_in_range(0));
    assert!(!date_in_range(FIRST_DAY - 1) && !date_in_range(LAST_DAY + 1));
    assert_eq!(format_date(FIRST_DAY), "0000-01-01");
    assert_eq!(format_date(LAST_DAY), "9999-12-31");
    assert_eq!(format_date(LAST_DAY + 1), "10000-01-01");
    assert_eq!(parse_date(&format_date(LAST_DAY + 1)), None);
    // 端の値でもオーバーフローしない。
    for d in [i64::MAX, i64::MIN, i64::MAX - 1, i64::MIN + 1] {
        let s = format_date(d);
        assert_eq!(parse_date(&s), None, "{s}");
    }
}
