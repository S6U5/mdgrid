//! [base-view.expr-eval][base-view.unsupported] 式によく使うメソッド・フィールド・関数(specs/_history/decidespec/_changes/2026-10-06-common-methods.md)。
//! 意味は Obsidian のヘルプ(https://obsidian.md/help/bases/functions)の小さな例のとおり。
//! 型の合わない受け手・引数は Null。

use super::*;
use std::collections::HashMap;

/// 2026-10-06(1970-01-01 からの日数)。火曜日。
const D: i64 = 20732;

fn run(src: &str) -> Val {
    let mut map: HashMap<String, Value> = HashMap::new();
    map.insert("status".into(), Value::Str("active".into()));
    map.insert(
        "tags".into(),
        Value::List(vec![Value::Str("b".into()), Value::Str("a".into())]),
    );
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |_: &str| None;
    let f = FileInfo {
        name: "n.md".into(),
        basename: "n".into(),
        ext: "md".into(),
        path: "n.md".into(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    };
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today: D,
        now: D * 86_400,
    };
    eval(&parse(src).expect(src), &env)
}

fn s(x: &str) -> Val {
    Val::Str(x.to_string())
}

fn n(x: f64) -> Val {
    Val::Num(x)
}

fn list(xs: &[Val]) -> Val {
    Val::List(xs.to_vec())
}

// ---- 文字 ----

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_upper_filter_is_true() {
    assert_eq!(run("status.upper() == \"ACTIVE\""), Val::Bool(true));
    assert_eq!(run("\"Hello\".upper()"), s("HELLO"));
    assert_eq!(run("(5).upper()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_lower() {
    assert_eq!(run("\"Hello\".lower()"), s("hello"));
    assert_eq!(run("[1].lower()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_title() {
    assert_eq!(run("\"hello world\".title()"), s("Hello World"));
    assert_eq!(run("\"hELLO wORLD\".title()"), s("Hello World"));
    assert_eq!(run("(5).title()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_trim() {
    assert_eq!(run("\"  a b  \".trim()"), s("a b"));
    assert_eq!(run("(5).trim()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_starts_with_ends_with() {
    assert_eq!(run("\"hello\".startsWith(\"he\")"), Val::Bool(true));
    assert_eq!(run("\"hello\".startsWith(\"lo\")"), Val::Bool(false));
    assert_eq!(run("\"hello\".endsWith(\"lo\")"), Val::Bool(true));
    assert_eq!(run("\"hello\".endsWith(\"he\")"), Val::Bool(false));
    assert_eq!(run("\"hello\".startsWith(1)"), Val::Null);
    assert_eq!(run("(5).endsWith(\"5\")"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_slice_string() {
    assert_eq!(run("\"hello\".slice(1, 4)"), s("ell"));
    assert_eq!(run("\"hello\".slice(-3)"), s("llo"));
    assert_eq!(run("\"hello\".slice(2)"), s("llo"));
    assert_eq!(run("\"hello\".slice(4, 1)"), s(""));
    assert_eq!(run("\"hello\".slice(\"a\")"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_replace_all_string_pattern() {
    assert_eq!(run("\"a,b,c,d\".replace(\",\", \"-\")"), s("a-b-c-d"));
    assert_eq!(run("\"abc\".replace(1, \"x\")"), Val::Null);
    assert_eq!(run("(5).replace(\"5\", \"6\")"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_split() {
    assert_eq!(
        run("\"a,b,c\".split(\",\")"),
        list(&[s("a"), s("b"), s("c")])
    );
    assert_eq!(run("\"a,b,c\".split(\",\", 2)"), list(&[s("a"), s("b")]));
    assert_eq!(run("\"abc\".split(\"\")"), list(&[s("a"), s("b"), s("c")]));
    assert_eq!(run("\"abc\".split(1)"), Val::Null);
    assert_eq!(run("[1].split(\",\")"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_reverse_string() {
    assert_eq!(run("\"abc\".reverse()"), s("cba"));
}

// ---- 数 ----

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_round() {
    assert_eq!(run("(2.5).round()"), n(3.0));
    assert_eq!(run("(2.4).round()"), n(2.0));
    assert_eq!(run("(-2.5).round()"), n(-2.0));
    assert_eq!(run("(2.567).round(2)"), n(2.57));
    assert_eq!(run("(2.5).round(\"a\")"), Val::Null);
    assert_eq!(run("\"2\".round()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_floor_ceil_abs() {
    assert_eq!(run("(2.7).floor()"), n(2.0));
    assert_eq!(run("(-2.2).floor()"), n(-3.0));
    assert_eq!(run("(2.1).ceil()"), n(3.0));
    assert_eq!(run("(-3).abs()"), n(3.0));
    assert_eq!(run("\"x\".floor()"), Val::Null);
    assert_eq!(run("\"x\".ceil()"), Val::Null);
    assert_eq!(run("\"x\".abs()"), Val::Null);
}

// ---- 日付・日時 ----

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_date_fields() {
    let t = "date(\"2026-10-06T07:08:09\")";
    assert_eq!(run(&format!("{t}.year")), n(2026.0));
    assert_eq!(run(&format!("{t}.month")), n(10.0));
    assert_eq!(run(&format!("{t}.day")), n(6.0));
    assert_eq!(run(&format!("{t}.hour")), n(7.0));
    assert_eq!(run(&format!("{t}.minute")), n(8.0));
    assert_eq!(run(&format!("{t}.second")), n(9.0));
    // 日付は 0 時
    assert_eq!(run("date(\"2026-10-06\").year"), n(2026.0));
    assert_eq!(run("date(\"2026-10-06\").hour"), n(0.0));
    assert_eq!(run("today().day"), n(6.0));
    assert_eq!(run("\"2026-10-06\".year"), Val::Null);
    assert_eq!(run("(5).month"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_date_and_time() {
    assert_eq!(run("date(\"2026-10-06T07:08:09\").date()"), Val::Date(D));
    assert_eq!(run("date(\"2026-10-06\").date()"), Val::Date(D));
    assert_eq!(run("date(\"2026-10-06T07:08:09\").time()"), s("07:08:09"));
    assert_eq!(run("date(\"2026-10-06\").time()"), s("00:00:00"));
    assert_eq!(run("\"x\".date()"), Val::Null);
    assert_eq!(run("(5).time()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_format() {
    let t = "date(\"2026-10-06T07:08:09\")";
    assert_eq!(
        run(&format!("{t}.format(\"YYYY-MM-DD HH:mm:ss\")")),
        s("2026-10-06 07:08:09")
    );
    assert_eq!(
        run(&format!("{t}.format(\"ddd, MMM D, YY\")")),
        s("Tue, Oct 6, 26")
    );
    assert_eq!(
        run(&format!("{t}.format(\"dddd MMMM [YYYY] H:mm\")")),
        s("Tuesday October YYYY 7:08")
    );
    assert_eq!(run("date(\"2026-01-05\").format(\"M/D\")"), s("1/5"));
    assert_eq!(
        run("date(\"2026-01-05\").format(\"YYYY年M月D日\")"),
        s("2026年1月5日")
    );
    assert_eq!(run("(5).format(\"YYYY\")"), Val::Null);
    assert_eq!(run(&format!("{t}.format(5)")), Val::Null);
}

// ---- リスト ----

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_join() {
    assert_eq!(run("[1, \"a\", true].join(\"-\")"), s("1-a-true"));
    assert_eq!(run("tags.join(\", \")"), s("b, a"));
    assert_eq!(run("[1].join(2)"), Val::Null);
    assert_eq!(run("\"a\".join(\",\")"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_unique() {
    assert_eq!(
        run("[1, 2, 1, \"a\", \"a\"].unique()"),
        list(&[n(1.0), n(2.0), s("a")])
    );
    assert_eq!(run("\"a\".unique()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_sort() {
    assert_eq!(run("[3, 1, 2].sort()"), list(&[n(1.0), n(2.0), n(3.0)]));
    assert_eq!(run("tags.sort()"), list(&[s("a"), s("b")]));
    assert_eq!(run("(5).sort()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_reverse_list() {
    assert_eq!(run("[1, 2, 3].reverse()"), list(&[n(3.0), n(2.0), n(1.0)]));
    assert_eq!(run("(5).reverse()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_flat() {
    assert_eq!(run("[1, [2, 3]].flat()"), list(&[n(1.0), n(2.0), n(3.0)]));
    assert_eq!(run("\"a\".flat()"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_slice_list() {
    assert_eq!(run("[1, 2, 3, 4].slice(1, 3)"), list(&[n(2.0), n(3.0)]));
    assert_eq!(run("[1, 2, 3, 4].slice(-1)"), list(&[n(4.0)]));
    assert_eq!(run("[1, 2].slice(true)"), Val::Null);
}

// ---- 関数 ----

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_max_min_functions() {
    assert_eq!(run("max(1, 5, 3)"), n(5.0));
    assert_eq!(run("min(1, 5, 3)"), n(1.0));
    assert_eq!(run("max(2)"), n(2.0));
    assert_eq!(run("max(1, \"a\")"), Val::Null);
    assert_eq!(run("min(null, 1)"), Val::Null);
}

// [utest -> req~base-view.expr-eval~1]
#[test]
fn test_bv_6_method_list_function() {
    assert_eq!(run("list(\"a\")"), list(&[s("a")]));
    assert_eq!(run("list([1, 2])"), list(&[n(1.0), n(2.0)]));
    assert_eq!(run("list(status).contains(\"active\")"), Val::Bool(true));
}

// ---- 未対応のまま ----

#[test]
fn test_bv_6_method_still_unsupported_names() {
    // [utest -> req~base-view.unsupported-no-guess~1, req~base-view.expr-eval~1] 足していない名前は今までどおり未対応
    for src in ["(1).toFixed(2)", "[1].map(1)", "x.year()"] {
        assert!(
            matches!(parse(src), Err(ExprError::Unsupported(_))),
            "{src}"
        );
    }
}

// ---- relative(): 今日・今からの隔たり ----

// [utest -> req~base-view.relative-days~1]
#[test]
fn test_relative_of_a_date_counts_days_from_today() {
    // 今日(2026-10-06)から1か月に満たない日付は、日の数で言う。
    assert_eq!(run("date(\"2026-10-20\").relative()"), s("あと14日"));
    assert_eq!(run("date(\"2026-10-07\").relative()"), s("あと1日"));
    assert_eq!(run("date(\"2026-10-06\").relative()"), s("今日"));
    assert_eq!(run("date(\"2026-10-01\").relative()"), s("5日前"));
    assert_eq!(run("date(\"2026-11-05\").relative()"), s("あと30日"));
}

// [utest -> req~base-view.relative-months~1]
#[test]
fn test_relative_of_a_date_counts_calendar_months_and_years() {
    // 暦で1か月以上は月の数、12か月以上は年の数で言う(端数は切り捨てる)。
    assert_eq!(run("date(\"2026-11-06\").relative()"), s("あと1か月"));
    assert_eq!(run("date(\"2027-01-15\").relative()"), s("あと3か月"));
    assert_eq!(run("date(\"2026-09-06\").relative()"), s("1か月前"));
    assert_eq!(run("date(\"2026-09-07\").relative()"), s("29日前"));
    assert_eq!(run("date(\"2027-10-05\").relative()"), s("あと11か月"));
    assert_eq!(run("date(\"2027-10-06\").relative()"), s("あと1年"));
    assert_eq!(run("date(\"2024-03-01\").relative()"), s("2年前"));
}

// [utest -> req~base-view.relative-time~1]
#[test]
fn test_relative_of_a_datetime_under_a_day_counts_hours_and_minutes() {
    // 日時は今(2026-10-06 00:00)から測り、1日に満たなければ時間、1時間に満たなければ分、1分に満たなければ「今」。
    assert_eq!(run("date(\"2026-10-06T03:20\").relative()"), s("あと3時間"));
    assert_eq!(run("date(\"2026-10-06T00:20\").relative()"), s("あと20分"));
    assert_eq!(run("date(\"2026-10-05T23:15\").relative()"), s("45分前"));
    assert_eq!(run("date(\"2026-10-06T00:00:30\").relative()"), s("今"));
    assert_eq!(run("date(\"2026-10-08T12:00\").relative()"), s("あと2日"));
}

// [utest -> req~base-view.relative-not-a-date~1]
#[test]
fn test_relative_of_a_value_that_is_not_a_date_is_null() {
    // 日付・日時でない値の relative() は null。
    assert_eq!(run("\"a\".relative()"), Val::Null);
    assert_eq!(run("(5).relative()"), Val::Null);
    assert_eq!(run("status.relative()"), Val::Null);
}

// [utest -> req~base-view.relative-days~1, req~base-view.relative-months~1]
#[test]
fn test_relative_in_english_uses_the_singular_for_one() {
    // 英語では、数が 1 のときは単数("in 1 day")、ほかは複数("in 14 days")で言う。
    let _lang = crate::i18n::scoped(crate::i18n::Lang::En);
    assert_eq!(run("date(\"2026-10-07\").relative()"), s("in 1 day"));
    assert_eq!(run("date(\"2026-10-20\").relative()"), s("in 14 days"));
    assert_eq!(run("date(\"2026-10-05\").relative()"), s("1 day ago"));
    assert_eq!(run("date(\"2026-10-01\").relative()"), s("5 days ago"));
    assert_eq!(run("date(\"2026-10-06\").relative()"), s("today"));
    assert_eq!(run("date(\"2026-11-06\").relative()"), s("in 1 month"));
    assert_eq!(run("date(\"2027-01-15\").relative()"), s("in 3 months"));
    assert_eq!(run("date(\"2027-10-06\").relative()"), s("in 1 year"));
    assert_eq!(run("date(\"2024-03-01\").relative()"), s("2 years ago"));
    assert_eq!(run("date(\"2026-10-06T01:00\").relative()"), s("in 1 hour"));
    assert_eq!(
        run("date(\"2026-10-05T23:59\").relative()"),
        s("1 minute ago")
    );
}

// ---- nextRun(): ルーチンの決まりから、今日以後の一番近い回の日 ----

/// `props` の値を持つ行で、今日を `today`(1970-01-01 からの日数)として式を評価する。
fn run_on(src: &str, props: &[(&str, &str)], today: i64) -> Val {
    let map: HashMap<String, Value> = props
        .iter()
        .map(|(k, v)| ((*k).to_string(), Value::Str((*v).to_string())))
        .collect();
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |_: &str| None;
    let f = FileInfo {
        name: "n.md".into(),
        basename: "n".into(),
        ext: "md".into(),
        path: "n.md".into(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    };
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today,
        now: today * 86_400,
    };
    eval(&parse(src).expect(src), &env)
}

fn day(s: &str) -> Val {
    Val::Date(crate::types::parse_date(s).expect(s))
}

/// 2026-11-24(火)。
fn nov24() -> i64 {
    crate::types::parse_date("2026-11-24").unwrap()
}

// [utest -> req~routine.next-run~1]
#[test]
fn test_next_run_weekly_is_the_next_listed_weekday() {
    // 今日が 2026-11-24(火)で weekly mon → 2026-11-30。毎週 火 → 今日。
    let r = run_on(
        "recurrence.nextRun()",
        &[("recurrence", "weekly mon")],
        nov24(),
    );
    assert_eq!(r, day("2026-11-30"));
    let r = run_on(
        "recurrence.nextRun()",
        &[("recurrence", "毎週 火")],
        nov24(),
    );
    assert_eq!(r, day("2026-11-24"));
}

// [utest -> req~routine.next-run~1]
#[test]
fn test_next_run_every_counts_from_the_start_argument() {
    // start 2026-11-20 から 3日ごと → 11-20・11-23・11-26 の並びで、今日(11-24)以後は 11-26。
    let props = [("recurrence", "3日ごと"), ("start", "2026-11-20")];
    let r = run_on("recurrence.nextRun(start)", &props, nov24());
    assert_eq!(r, day("2026-11-26"));
    // 起点が無ければ空。
    assert_eq!(
        run_on(
            "recurrence.nextRun()",
            &[("recurrence", "3日ごと")],
            nov24()
        ),
        Val::Null
    );
}

// [utest -> req~routine.next-run~1]
#[test]
fn test_next_run_of_an_unreadable_rule_is_empty() {
    assert_eq!(
        run_on(
            "recurrence.nextRun()",
            &[("recurrence", "毎週かも")],
            nov24()
        ),
        Val::Null
    );
    assert_eq!(
        run_on("recurrence.nextRun()", &[("recurrence", "")], nov24()),
        Val::Null
    );
}

// [utest -> req~routine.next-run~1]
#[test]
fn test_next_run_monthly_last_and_daily() {
    // 毎月 末日 → 11-30。31 日の無い月を挟む monthly 31 → 12-31。daily → 今日。
    let at = |rule: &str| run_on("recurrence.nextRun()", &[("recurrence", rule)], nov24());
    assert_eq!(at("毎月 末日"), day("2026-11-30"));
    assert_eq!(at("monthly 31"), day("2026-12-31"));
    assert_eq!(at("daily"), day("2026-11-24"));
}
