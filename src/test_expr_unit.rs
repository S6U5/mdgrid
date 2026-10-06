//! 式の単体のテスト(止まらないこと・日付の端)。BV-6・BV-7。

use super::*;
use std::collections::HashMap;

fn fi() -> FileInfo {
    FileInfo {
        name: "n.md".to_string(),
        basename: "n".to_string(),
        ext: "md".to_string(),
        path: "n.md".to_string(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: vec!["Proj/Sub".to_string()],
    }
}

fn run(src: &str) -> Val {
    let map: HashMap<String, Value> = HashMap::new();
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |_: &str| Some(Val::Date(i64::MAX));
    let f = fi();
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today: 20727,
        now: 20727 * 86_400,
    };
    eval(&parse(src).expect(src), &env)
}

#[test]
fn test_date_range_constants() {
    // [BV-6] 扱う日付の範囲の端
    assert_eq!(parse_date("0000-01-01"), Some(MIN_DAY));
    assert_eq!(parse_date("9999-12-31"), Some(MAX_DAY));
}

#[test]
fn test_deep_nesting_is_syntax_not_panic() {
    // [BV-7] 深い入れ子・長い鎖・前置の演算子の連なりは止まらずに Syntax
    for src in [
        "(".repeat(100_000) + "1" + &")".repeat(100_000),
        "!".repeat(100_000) + "true",
        "-".repeat(100_000) + "1",
        vec!["1"; 100_000].join(" + "),
        "[".repeat(100_000) + &"]".repeat(100_000),
        "a".to_string() + &".length".repeat(100_000),
        "if(".repeat(100_000),
    ] {
        assert!(matches!(parse(&src), Err(ExprError::Syntax(_))));
    }
    // 上限の内側は読める
    let ok = "(".repeat(50) + "1" + &")".repeat(50);
    assert_eq!(run(&ok), Val::Num(1.0));
}

#[test]
fn test_numbers_do_not_panic() {
    // [BV-6] 0 での割り算・あふれは Null
    assert_eq!(run("1 / 0"), Val::Null);
    assert_eq!(run("0 / 0"), Val::Null);
    assert_eq!(run("5 % 0"), Val::Null);
    assert_eq!(run("1e308 * 10"), Val::Null);
    assert_eq!(run("1e999"), Val::Null);
    assert_eq!(run("-3"), Val::Num(-3.0));
    assert_eq!(run("today() + \"9223372036854775807d\""), Val::Null);
    assert_eq!(run("today() + \"99999999999999999999d\""), Val::Null);
    assert_eq!(run("today() + \"20000 years\""), Val::Null);
    assert_eq!(run("formula.x + \"1d\""), Val::Null);
    assert_eq!(run("formula.x - today()"), Val::Null);
    assert_eq!(run("formula.x.toString()"), Val::Str(String::new()));
    assert_eq!(run("formula.x < today()"), Val::Bool(false));
}

#[test]
fn test_months_clamp_to_month_end() {
    // [BV-6] 月を足して月の終わりを越えたら、その月の最後の日
    assert_eq!(
        run("date(\"2026-01-31\") + \"1M\""),
        Val::Date(parse_date("2026-02-28").unwrap())
    );
    assert_eq!(
        run("date(\"2024-02-29\") + \"1y\""),
        Val::Date(parse_date("2025-02-28").unwrap())
    );
    assert_eq!(
        run("date(\"2026-03-15\") - \"2 months\""),
        Val::Date(parse_date("2026-01-15").unwrap())
    );
    assert_eq!(
        run("date(\"2026-10-01\") + \"1d 12h\""),
        Val::DateTime(20728 * 86_400 + 12 * 3600)
    );
    assert_eq!(
        run("date(\"2026-10-01\") + \"5m\""),
        Val::DateTime(20727 * 86_400 + 300)
    );
}

#[test]
fn test_datetime_and_bad_strings() {
    // [BV-6] 日時の文字列、読めない日付と期間は Null
    assert_eq!(
        run("date(\"2026-10-01T01:02:03\")"),
        Val::DateTime(20727 * 86_400 + 3723)
    );
    assert_eq!(run("date(\"2026-02-30\")"), Val::Null);
    assert_eq!(run("date(\"x\")"), Val::Null);
    assert_eq!(run("duration(\"abc\")"), Val::Null);
    assert_eq!(run("today() + \"abc\""), Val::Null);
    assert_eq!(
        // 期間を左にした / は期間(ミリ秒を丸める)
        run("(today() - date(\"2026-09-30\")) / 86400000"),
        Val::Duration(1)
    );
}

#[test]
fn test_has_tag_case_and_unsupported_first_wins() {
    // [BV-6] タグは大文字小文字を区別しない。[BV-7] 文法の誤りが未対応より先
    assert_eq!(run("file.hasTag(\"#proj\")"), Val::Bool(true));
    assert_eq!(run("file.hasTag(\"proj/sub\")"), Val::Bool(true));
    assert!(matches!(parse("foo(1) +"), Err(ExprError::Syntax(_))));
    assert_eq!(
        parse("foo(1) && bar(2)").unwrap_err(),
        ExprError::Unsupported("foo()".to_string())
    );
    assert!(matches!(parse("if(true)"), Err(ExprError::Syntax(_))));
    assert!(matches!(parse("1 = 1"), Err(ExprError::Syntax(_))));
}

#[test]
fn test_duration_times_number_is_duration() {
    // [BV-6] 期間を左にした * / は期間(Obsidian のヘルプ: duration('1d') * 2)
    assert_eq!(
        run("today() + duration('1d') * 2 == today() + \"2d\""),
        Val::Bool(true)
    );
    assert_eq!(run("duration('1d') * 2"), Val::Duration(2 * 86_400_000));
    assert_eq!(run("duration('1d') / 3"), Val::Duration(28_800_000));
    assert_eq!(run("duration('1s') / 3"), Val::Duration(333));
    assert_eq!(run("duration('1d') / 0"), Val::Null);
    assert_eq!(run("duration('1d') * 1e300"), Val::Null);
    // % と数を左にした式は、期間をミリ秒の数として使う
    assert_eq!(run("duration('1s') % 300"), Val::Num(100.0));
    assert_eq!(run("2 * duration('1s')"), Val::Num(2000.0));
}

#[test]
fn test_filter_shape_mtime_within_days() {
    // [BV-6] filters の形: file.mtime > now() - duration('1d') * 7
    let map: HashMap<String, Value> = HashMap::new();
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |_: &str| None;
    let now = 20727 * 86_400;
    let mut f = fi();
    let src = parse("file.mtime > now() - duration('1d') * 7").unwrap();
    for (mtime, want) in [(now - 6 * 86_400, true), (now - 8 * 86_400, false)] {
        f.mtime = mtime;
        let env = Env {
            prop: &prop,
            file: &f,
            formula: &formula,
            today: 20727,
            now,
        };
        assert_eq!(eval(&src, &env), Val::Bool(want), "mtime {mtime}");
    }
}
