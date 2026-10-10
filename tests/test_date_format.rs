//! 日付の形・打ち込み・月の格子・月の足し引き・設定の受け入れテスト(date-picker のタスク 1)。
//! CE-5・CE-21・CE-22・CLI-3。
//! 仕様: specs/cell-edit/spec.md(CE-5・CE-21・CE-22)、specs/cli/spec.md(CLI-3)。
//! 形: docs/design.md の「日付のカレンダーと日付の形(CE-20〜CE-22)」の核。
//! 実装を見ずに書いた。日数は 1970-01-01 からの日数(types::parse_date・format_date と同じ)。
//!
//! 推測した点:
//! - `DateFormat`・`parse_date_input`・`WeekStart`・`month_grid`・`add_months` は `mdgrid::types` にある。
//! - `config::Config` の欄は `date_format: DateFormat` と `week_start: WeekStart`。
//! - 設定の week_start の値は `"sun"`・`"mon"`。
//! - `DateFormat`・`WeekStart` に Debug・PartialEq があるとは仮定しない(見せた結果と `matches!` で確かめる)。

use mdgrid::config::parse as parse_config;
use mdgrid::types::{
    add_months, format_date, month_grid, parse_date, parse_date_input, DateFormat, WeekStart,
};

// ---- 道具 ----

fn d(s: &str) -> i64 {
    parse_date(s).unwrap_or_else(|| panic!("{s} は日付として読めるはず"))
}

fn fmt(p: &str) -> DateFormat {
    match DateFormat::parse(p) {
        Ok(f) => f,
        Err(e) => panic!("形 {p:?} は読めるはず: {e}"),
    }
}

/// 1970-01-01(木)からの日数の曜日。0 = 日 … 6 = 土。
fn weekday_sun0(days: i64) -> i64 {
    (days + 4).rem_euclid(7)
}

fn input_ok(s: &str, f: &DateFormat, today: i64) -> Option<i64> {
    match parse_date_input(s, f, today) {
        Ok(v) => v,
        Err(e) => panic!("打ち込み {s:?} は読めるはず: {e}"),
    }
}

fn input_err(s: &str, f: &DateFormat, today: i64) {
    match parse_date_input(s, f, today) {
        Err(e) => assert!(!e.trim().is_empty(), "{s:?} の Err の理由が空"),
        Ok(v) => panic!("打ち込み {s:?} は Err のはずが Ok({v:?})"),
    }
}

// ---- DateFormat::parse・format・iso(CE-22) ----

#[test]
fn test_ce_22_date_format_parse_accepts_common_patterns() {
    // [CE-22] よくある形が読め、その形で見せられる。2026-10-05 は月曜。
    let day = d("2026-10-05");
    let cases = [
        ("YYYY-MM-DD", "2026-10-05"),
        ("YYYY/MM/DD", "2026/10/05"),
        ("YYYY.MM.DD", "2026.10.05"),
        ("YYYY年M月D日", "2026年10月5日"),
        ("MM/DD", "10/05"),
        ("M/D", "10/5"),
        ("YY-MM-DD", "26-10-05"),
        ("YYYY/MM/DD (ddd)", "2026/10/05 (月)"),
    ];
    for (pattern, want) in cases {
        assert_eq!(fmt(pattern).format(day), want, "形 {pattern:?}");
    }
}

#[test]
fn test_ce_22_date_format_single_digit_parts() {
    // [CE-22] M・D は0で埋めず、MM・DD は0で埋める。
    let day = d("2026-01-05");
    assert_eq!(fmt("YYYY年M月D日").format(day), "2026年1月5日");
    assert_eq!(fmt("M/D").format(day), "1/5");
    assert_eq!(fmt("MM/DD").format(day), "01/05");
    assert_eq!(fmt("YYYY/MM/DD").format(day), "2026/01/05");
}

#[test]
fn test_ce_22_date_format_weekday_names() {
    // [CE-22] ddd は曜日の短い名前。2026-10-04(日)〜2026-10-10(土)。
    let f = fmt("YYYY/MM/DD (ddd)");
    let names = ["日", "月", "火", "水", "木", "金", "土"];
    for (i, name) in names.iter().enumerate() {
        let day = d("2026-10-04") + i as i64;
        let s = f.format(day);
        assert!(
            s.contains(&format!("({name})")),
            "{} の曜日: {s}",
            format_date(day)
        );
    }
}

#[test]
fn test_ce_22_date_format_iso_is_default_shape() {
    // [CE-22] iso() は YYYY-MM-DD(ノートに書く形と同じ)。
    let iso = DateFormat::iso();
    assert_eq!(iso.format(d("2026-10-05")), "2026-10-05");
    assert_eq!(iso.format(d("1970-01-01")), "1970-01-01");
    assert_eq!(iso.format(d("2024-02-29")), "2024-02-29");
    // YYYY-MM-DD を読んだ形も同じに見せる。
    assert_eq!(fmt("YYYY-MM-DD").format(d("2026-10-05")), "2026-10-05");
}

#[test]
fn test_ce_22_date_format_parse_rejects_bad_patterns() {
    // [CE-22] 読めない形は Err(理由1行で空でない)。空・部品の無い形・知らない部品。
    for p in ["", "   ", "-/.", "年月日", "QQ", "YYYY-QQ-DD"] {
        match DateFormat::parse(p) {
            Err(e) => {
                assert!(!e.trim().is_empty(), "{p:?} の理由が空");
                assert!(
                    !e.trim_end().contains('\n'),
                    "{p:?} の理由が1行でない: {e:?}"
                );
            }
            Ok(f) => panic!(
                "形 {p:?} は Err のはずが Ok(2026-10-05 → {:?})",
                f.format(d("2026-10-05"))
            ),
        }
    }
}

// ---- parse_date_input(CE-5・CE-22) ----

#[test]
fn test_ce_22_input_iso_accepted_under_any_format() {
    // [CE-22][CE-5] YYYY-MM-DD はどの設定の形でも読める。
    let today = d("2026-10-02");
    let want = Some(d("2026-11-03"));
    for p in [
        "YYYY-MM-DD",
        "YYYY/MM/DD",
        "YYYY.MM.DD",
        "YYYY年M月D日",
        "MM/DD",
        "M/D",
        "YY-MM-DD",
        "YYYY/MM/DD (ddd)",
    ] {
        assert_eq!(
            input_ok("2026-11-03", &fmt(p), today),
            want,
            "設定の形 {p:?}"
        );
    }
    assert_eq!(input_ok("2026-11-03", &DateFormat::iso(), today), want);
}

#[test]
fn test_ce_22_input_configured_format_accepted() {
    // [CE-22] 設定の形で打っても読める。
    let today = d("2026-10-02");
    let want = Some(d("2026-11-03"));
    assert_eq!(input_ok("2026/11/03", &fmt("YYYY/MM/DD"), today), want);
    assert_eq!(input_ok("2026.11.03", &fmt("YYYY.MM.DD"), today), want);
    assert_eq!(input_ok("2026年11月3日", &fmt("YYYY年M月D日"), today), want);
    assert_eq!(input_ok("26-11-03", &fmt("YY-MM-DD"), today), want);
}

#[test]
fn test_ce_22_input_yearless_format_uses_todays_year() {
    // [CE-22] 年の無い形は今年(today の年)。
    let f = fmt("M/D");
    assert_eq!(input_ok("11/3", &f, d("2026-10-02")), Some(d("2026-11-03")));
    assert_eq!(input_ok("11/3", &f, d("2027-05-20")), Some(d("2027-11-03")));
    let f = fmt("MM/DD");
    assert_eq!(
        input_ok("11/03", &f, d("2026-10-02")),
        Some(d("2026-11-03"))
    );
}

#[test]
fn test_ce_22_input_roundtrips_formatted_text() {
    // [CE-22] 設定の形で見せた文字をそのまま打てば、同じ日に戻る(カレンダーで動かした入力ボックスの文字を確定する場合)。
    let today = d("2026-10-02");
    for p in [
        "YYYY-MM-DD",
        "YYYY/MM/DD",
        "YYYY.MM.DD",
        "YYYY年M月D日",
        "MM/DD",
        "M/D",
        "YY-MM-DD",
        "YYYY/MM/DD (ddd)",
    ] {
        let f = fmt(p);
        for s in ["2026-01-05", "2026-10-30", "2026-12-31", "2026-02-28"] {
            let day = d(s);
            let shown = f.format(day);
            assert_eq!(
                input_ok(&shown, &f, today),
                Some(day),
                "形 {p:?} で見せた {shown:?} を打つ"
            );
        }
    }
}

#[test]
fn test_ce_5_input_relative_days_from_today() {
    // [CE-5] +N・-N は today からの日数。設定の形によらない。
    let today = d("2026-10-02");
    for f in [DateFormat::iso(), fmt("YYYY/MM/DD"), fmt("M/D")] {
        assert_eq!(input_ok("+3", &f, today), Some(d("2026-10-05")));
        assert_eq!(input_ok("-2", &f, today), Some(d("2026-09-30")));
        assert_eq!(input_ok("+30", &f, today), Some(d("2026-11-01")));
    }
}

#[test]
fn test_ce_5_input_empty_is_none() {
    // [CE-5] 空は Ok(None)(null にする。CE-9)。空白だけも空とみなす(前後の空白は許す)。
    let today = d("2026-10-02");
    for f in [DateFormat::iso(), fmt("YYYY/MM/DD")] {
        assert_eq!(input_ok("", &f, today), None);
        assert_eq!(input_ok("   ", &f, today), None);
    }
}

#[test]
fn test_ce_5_input_rejects_invalid_dates() {
    // [CE-5][CE-22] 実在しない日付・読めない打ち込みは Err(理由が空でない)。
    let today = d("2026-10-02");
    let iso = DateFormat::iso();
    let slash = fmt("YYYY/MM/DD");
    input_err("2026-02-30", &iso, today);
    input_err("2026-02-30", &slash, today);
    input_err("2026/13/01", &slash, today);
    input_err("2026/02/30", &slash, today);
    input_err("2025-02-29", &iso, today);
    input_err("abc", &iso, today);
    input_err("abc", &slash, today);
    input_err("13/1", &fmt("M/D"), today);
}

#[test]
fn test_ce_5_input_allows_surrounding_whitespace() {
    // [CE-5][CE-22] 前後の空白は許す。
    let today = d("2026-10-02");
    let slash = fmt("YYYY/MM/DD");
    assert_eq!(
        input_ok("  2026-11-03 ", &slash, today),
        Some(d("2026-11-03"))
    );
    assert_eq!(
        input_ok(" 2026/11/03\t", &slash, today),
        Some(d("2026-11-03"))
    );
    assert_eq!(input_ok(" +3 ", &slash, today), Some(d("2026-10-05")));
    assert_eq!(
        input_ok(" 11/3 ", &fmt("M/D"), today),
        Some(d("2026-11-03"))
    );
}

#[test]
fn test_ce_22_input_with_weekday_format() {
    // [CE-22] 曜日つきの形: 正しい曜日つきで打てば読め、YYYY-MM-DD も読める。
    // 曜日が違うときの扱い(日付で決めるか Err か)は確かめない。
    let today = d("2026-10-02");
    let f = fmt("YYYY/MM/DD (ddd)");
    // 2026-11-03 は火曜。
    assert_eq!(
        input_ok("2026/11/03 (火)", &f, today),
        Some(d("2026-11-03"))
    );
    assert_eq!(input_ok("2026-11-03", &f, today), Some(d("2026-11-03")));
    // 曜日が違っても、Ok なら日付で決まっている。
    if let Ok(v) = parse_date_input("2026/11/03 (水)", &f, today) {
        assert_eq!(
            v,
            Some(d("2026-11-03")),
            "曜日が違うときに Ok なら日付で決める"
        );
    }
}

// ---- month_grid(CE-21) ----

fn check_grid(year: i32, month: u32, start: WeekStart, sun_start: bool, ndays: u32) {
    let grid = month_grid(year, month, start);
    assert!(
        (4..=6).contains(&grid.len()),
        "{year}-{month:02} の週の数: {}",
        grid.len()
    );
    // 全部の日がちょうど1回ずつ、順に並ぶ。
    let days: Vec<u32> = grid
        .iter()
        .flat_map(|w| w.iter().flatten().copied())
        .collect();
    let want: Vec<u32> = (1..=ndays).collect();
    assert_eq!(days, want, "{year}-{month:02} の日の並び");
    // 列の曜日が合う。
    for week in &grid {
        for (col, cell) in week.iter().enumerate() {
            if let Some(day) = cell {
                let s = format!("{year:04}-{month:02}-{day:02}");
                let wd = weekday_sun0(d(&s));
                let want_col = if sun_start { wd } else { (wd + 6) % 7 };
                assert_eq!(col as i64, want_col, "{s} の列(日曜始まり={sun_start})");
            }
        }
    }
}

#[test]
fn test_ce_21_month_grid_october_2026_sunday_start() {
    // [CE-21] 2026-10-01 は木曜。日曜始まりで1週目は [None×4, 1, 2, 3]。
    let grid = month_grid(2026, 10, WeekStart::Sun);
    assert_eq!(grid[0], [None, None, None, None, Some(1), Some(2), Some(3)]);
    assert!(grid.len() == 5 || grid.len() == 6, "週の数: {}", grid.len());
    check_grid(2026, 10, WeekStart::Sun, true, 31);
}

#[test]
fn test_ce_21_month_grid_october_2026_monday_start() {
    // [CE-21] 設定で月曜始まり → 1列目が月。1週目は [None×3, 1, 2, 3, 4]。
    let grid = month_grid(2026, 10, WeekStart::Mon);
    assert_eq!(
        grid[0],
        [None, None, None, Some(1), Some(2), Some(3), Some(4)]
    );
    assert!(grid.len() == 5 || grid.len() == 6, "週の数: {}", grid.len());
    check_grid(2026, 10, WeekStart::Mon, false, 31);
}

#[test]
fn test_ce_21_month_grid_leap_february() {
    // [CE-21] 閏年の2月は29日、平年は28日。
    check_grid(2024, 2, WeekStart::Sun, true, 29);
    check_grid(2024, 2, WeekStart::Mon, false, 29);
    check_grid(2026, 2, WeekStart::Sun, true, 28);
    check_grid(2026, 2, WeekStart::Mon, false, 28);
    check_grid(2000, 2, WeekStart::Sun, true, 29);
    check_grid(2100, 2, WeekStart::Sun, true, 28);
}

#[test]
fn test_ce_21_month_grid_various_months() {
    // [CE-21] いろいろな月で、日がちょうど1回ずつ、曜日の列が合う。
    let lens = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, n) in lens.iter().enumerate() {
        let m = i as u32 + 1;
        check_grid(2026, m, WeekStart::Sun, true, *n);
        check_grid(2026, m, WeekStart::Mon, false, *n);
    }
    // 1日が日曜の月(2026-03)と土曜の月(2026-08)。
    assert_eq!(month_grid(2026, 3, WeekStart::Sun)[0][0], Some(1));
    assert_eq!(month_grid(2026, 8, WeekStart::Sun)[0][6], Some(1));
    assert_eq!(month_grid(2026, 3, WeekStart::Mon)[0][6], Some(1));
}

// ---- add_months(CE-21) ----

#[test]
fn test_ce_21_add_months_same_day_or_month_end() {
    // [CE-21] PageDown・PageUp: 翌月・前月の同じ日、無ければ月末。
    assert_eq!(add_months(d("2026-10-30"), 1), d("2026-11-30"));
    assert_eq!(add_months(d("2026-10-31"), 1), d("2026-11-30"));
    assert_eq!(add_months(d("2026-01-31"), 1), d("2026-02-28"));
    assert_eq!(add_months(d("2024-01-31"), 1), d("2024-02-29"));
    assert_eq!(add_months(d("2026-10-15"), -1), d("2026-09-15"));
    assert_eq!(add_months(d("2026-03-31"), -1), d("2026-02-28"));
    assert_eq!(add_months(d("2026-10-05"), 0), d("2026-10-05"));
}

#[test]
fn test_ce_21_add_months_crosses_years() {
    // [CE-21] 年をまたぐ。
    assert_eq!(add_months(d("2026-12-15"), 1), d("2027-01-15"));
    assert_eq!(add_months(d("2026-01-15"), -1), d("2025-12-15"));
    assert_eq!(add_months(d("2026-10-05"), 12), d("2027-10-05"));
    assert_eq!(add_months(d("2026-10-05"), -13), d("2025-09-05"));
    assert_eq!(add_months(d("2024-02-29"), 12), d("2025-02-28"));
}

// ---- 設定(CLI-3・CE-22・CE-21) ----

#[test]
fn test_cli_3_config_reads_date_format_and_week_start() {
    // [CLI-3][CE-22][CE-21] date_format と week_start を設定ファイルから読む。
    let text = "[dates]\nformat = \"YYYY/MM/DD\"\nweek_start = \"mon\"\n";
    let (c, warnings) = parse_config(text).expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(
        c.resolved().date_format.format(d("2026-10-05")),
        "2026/10/05"
    );
    assert!(
        matches!(c.resolved().week_start, WeekStart::Mon),
        "week_start が月曜でない"
    );

    let (c, warnings) = parse_config("[dates]\nweek_start = \"sun\"\n").expect("読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert!(
        matches!(c.resolved().week_start, WeekStart::Sun),
        "week_start が日曜でない"
    );
}

#[test]
fn test_cli_3_config_date_defaults() {
    // [CLI-3][CE-22][CE-21] 無ければ既定(YYYY-MM-DD・日曜始まり)。今までどおり 2026-10-05 と見せる。
    let (c, warnings) = parse_config("").expect("空の設定は読める");
    assert!(warnings.is_empty(), "警告: {:?}", warnings);
    assert_eq!(
        c.resolved().date_format.format(d("2026-10-05")),
        "2026-10-05"
    );
    assert!(
        matches!(c.resolved().week_start, WeekStart::Sun),
        "既定の週の始まりが日曜でない"
    );
}

#[test]
fn test_cli_3_config_bad_date_format_warns_and_uses_default() {
    // [CLI-3][CE-22] 読めない date_format → Ok で警告し、既定の形を使う。
    for bad in ["QQ", "", "-/."] {
        let text = format!("[dates]\nformat = \"{bad}\"\n");
        let (c, warnings) = parse_config(&text).expect("読めない date_format でも Ok");
        assert!(
            warnings.iter().any(|w| w.contains("dates.format")),
            "{bad:?} の警告に dates.format が無い: {:?}",
            warnings
        );
        assert_eq!(
            c.resolved().date_format.format(d("2026-10-05")),
            "2026-10-05"
        );
    }
}

#[test]
fn test_cli_3_config_unknown_week_start_warns() {
    // [CLI-3][CE-21] 知らない week_start → Ok で警告し、既定(日曜)。
    let (c, warnings) =
        parse_config("[dates]\nweek_start = \"tue\"\n").expect("知らない week_start でも Ok");
    assert!(
        warnings.iter().any(|w| w.contains("week_start")),
        "警告に week_start が無い: {:?}",
        warnings
    );
    assert!(
        matches!(c.resolved().week_start, WeekStart::Sun),
        "既定の日曜でない"
    );
}
