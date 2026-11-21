//! 日付の言葉(cell-edit.date-words・cell-edit.date-words-weekday・cell-edit.date-words-period)の読み。
//! 期待値は要求の受け入れの例から取る。読むのは `types::parse_date_input`(セルの編集・一括・絞り込みが通る口)。

use crate::i18n::{resolve, scoped, Language};
use crate::types::{format_date, parse_date, parse_date_input, DateFormat, DateInput, WeekStart};

fn day(s: &str) -> i64 {
    parse_date(s).unwrap()
}

/// `text` を、今日 `today`・週の始まり `week`・既定の日付の形で読んだ日(`YYYY-MM-DD`)。読めなければ理由。
fn read(text: &str, today: &str, week: WeekStart) -> Result<String, String> {
    let fmt = DateFormat::iso();
    let how = DateInput { fmt: &fmt, week };
    match parse_date_input(text, how, day(today)) {
        Ok(Some(d)) => Ok(format_date(d)),
        Ok(None) => Err("空".to_string()),
        Err(e) => Err(e),
    }
}

#[test]
fn test_date_words_from_today() {
    // [utest -> req~cell-edit.date-words~1] 今日が 2026-11-21(土)、週の始まりが日曜なら、日本語と英語の言葉・単位つきの日数が要求の日になる。
    let cases = [
        ("明日", "2026-11-22"),
        ("tomorrow", "2026-11-22"),
        ("昨日", "2026-11-20"),
        ("yesterday", "2026-11-20"),
        ("明後日", "2026-11-23"),
        ("3日後", "2026-11-24"),
        ("in 3 days", "2026-11-24"),
        ("3d", "2026-11-24"),
        ("+3d", "2026-11-24"),
        ("2日前", "2026-11-19"),
        ("-2d", "2026-11-19"),
        ("2週間後", "2026-12-05"),
        ("+2w", "2026-12-05"),
        ("1か月後", "2026-12-21"),
        ("+1m", "2026-12-21"),
    ];
    for (text, want) in cases {
        assert_eq!(
            read(text, "2026-11-21", WeekStart::Sun),
            Ok(want.to_string()),
            "{text}"
        );
    }
}

#[test]
fn test_date_words_regardless_of_language() {
    // [utest -> req~cell-edit.date-words~1] `LANG=en_US.UTF-8` で文言が英語でも、`明日` は 2026-11-22 になる。
    let lang = resolve(Language::Auto, |k| {
        (k == "LANG").then(|| "en_US.UTF-8".to_string())
    });
    let _g = scoped(lang);
    assert_eq!(
        read("明日", "2026-11-21", WeekStart::Sun),
        Ok("2026-11-22".to_string())
    );
}

#[test]
fn test_date_words_weekday() {
    // [utest -> req~cell-edit.date-words-weekday~1] 曜日の言葉は、今日より後の最初のその曜日・今の週・次の週の曜日になり、週は dates.week_start で区切る。
    let cases = [
        ("金曜", "2026-11-20", WeekStart::Sun, "2026-11-27"),
        ("今週の金曜", "2026-11-20", WeekStart::Sun, "2026-11-20"),
        ("来週の金曜", "2026-11-20", WeekStart::Sun, "2026-11-27"),
        ("来週の月曜", "2026-11-20", WeekStart::Sun, "2026-11-23"),
        ("来週の月曜", "2026-11-22", WeekStart::Mon, "2026-11-23"),
        ("今週の月曜", "2026-11-22", WeekStart::Mon, "2026-11-16"),
    ];
    for (text, today, week, want) in cases {
        assert_eq!(
            read(text, today, week),
            Ok(want.to_string()),
            "{text}(今日 {today}・{week:?} 始まり)"
        );
    }
}

#[test]
fn test_date_words_weekday_spellings() {
    // [utest -> req~cell-edit.date-words-weekday~1] `金曜日`・`fri`・`friday` は `金曜` と、`this fri` は `今週の金曜` と、`next fri` は `来週の金曜` と同じ日になる。
    let today = "2026-11-20";
    for (text, want) in [
        ("金曜日", "2026-11-27"),
        ("fri", "2026-11-27"),
        ("friday", "2026-11-27"),
        ("this fri", "2026-11-20"),
        ("next fri", "2026-11-27"),
    ] {
        assert_eq!(
            read(text, today, WeekStart::Sun),
            Ok(want.to_string()),
            "{text}"
        );
    }
}

#[test]
fn test_date_words_period() {
    // [utest -> req~cell-edit.date-words-period~1] 今日が 2026-11-21・日曜始まりなら、来週は次の週の初めの日、来月は次の月の1日、月末・来月末はその月の最後の日になる。
    let cases = [
        ("来週", "2026-11-22"),
        ("next week", "2026-11-22"),
        ("来月", "2026-12-01"),
        ("next month", "2026-12-01"),
        ("月末", "2026-11-30"),
        ("eom", "2026-11-30"),
        ("来月末", "2026-12-31"),
    ];
    for (text, want) in cases {
        assert_eq!(
            read(text, "2026-11-21", WeekStart::Sun),
            Ok(want.to_string()),
            "{text}"
        );
    }
}

#[test]
fn test_date_words_month_does_not_overflow() {
    // [utest -> req~cell-edit.date-words-period~1] 今日が 2027-01-31 なら、`1か月後` は月の終わりを越えずに 2027-02-28 になる。
    assert_eq!(
        read("1か月後", "2027-01-31", WeekStart::Sun),
        Ok("2027-02-28".to_string())
    );
}

#[test]
fn test_date_words_other_spellings() {
    // [utest -> req~cell-edit.date-words~1] 要求の例と同じ意味の言い方(今日・かな・英語の長い形・年・大文字・全角)も、同じ決まりで今日から数えた日になる。
    let cases = [
        ("今日", "2026-11-21"),
        ("today", "2026-11-21"),
        ("あした", "2026-11-22"),
        ("あさって", "2026-11-23"),
        ("一昨日", "2026-11-19"),
        ("3 days ago", "2026-11-18"),
        ("in 2 weeks", "2026-12-05"),
        ("in 1 month", "2026-12-21"),
        ("1年後", "2027-11-21"),
        ("+1y", "2027-11-21"),
        ("2週後", "2026-12-05"),
        ("1ヶ月後", "2026-12-21"),
        ("Tomorrow", "2026-11-22"),
        ("３日後", "2026-11-24"),
        ("来週 の 金曜", "2026-11-27"),
        ("Next Friday", "2026-11-27"),
        ("end of next month", "2026-12-31"),
    ];
    for (text, want) in cases {
        assert_eq!(
            read(text, "2026-11-21", WeekStart::Sun),
            Ok(want.to_string()),
            "{text}"
        );
    }
}

#[test]
fn test_date_words_unreadable_and_out_of_range() {
    // [utest -> req~cell-edit.date-words~1] 言葉でない打ち込み(曜日の1字だけ・続きのある言葉)は読めない理由に、日付の範囲を出る言葉は範囲の外の理由になる。
    for text in [
        "来週の金",
        "金",
        "あさっての次",
        "soon",
        "3x",
        "in days",
        "next",
    ] {
        let e = read(text, "2026-11-21", WeekStart::Sun).unwrap_err();
        assert!(e.contains("読めない") && e.contains(text), "{text}: {e}");
    }
    for text in [
        "+99999y",
        "9999999999999999999999d",
        "-3000000d",
        "in 99999 months",
    ] {
        let e = read(text, "2026-11-21", WeekStart::Sun).unwrap_err();
        assert!(e.contains("範囲"), "{text}: {e}");
    }
    // 範囲の最後の月の「来月」は範囲の外、「月末」は範囲の最後の日。
    let e = read("来月", "9999-12-15", WeekStart::Sun).unwrap_err();
    assert!(e.contains("範囲"), "{e}");
    assert_eq!(
        read("月末", "9999-12-15", WeekStart::Sun),
        Ok("9999-12-31".to_string())
    );
}
