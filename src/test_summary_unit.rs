//! 集計の純関数の試験(BV-14)。14 種のそれぞれの小さな例・型の合わない値は数えない・値が無ければ空。

use super::*;
use crate::i18n::{scoped, Lang};

fn n(x: f64) -> Val {
    Val::Num(x)
}

fn s(x: &str) -> Val {
    Val::Str(x.to_string())
}

/// 2026-10-05 などの日付(1970-01-01 からの日)。
fn d(text: &str) -> Val {
    Val::Date(crate::types::parse_date(text).unwrap())
}

/// 数・文字・日付・真偽・空の混ざった列。
fn mixed() -> Vec<Val> {
    vec![
        n(3.0),
        s("abc"),
        n(5.0),
        Val::Bool(true),
        Val::Null,
        n(10.0),
        d("2026-10-05"),
        s(""),
    ]
}

#[test]
fn test_bv_14_names_parse_and_key() {
    // [BV-14] 組み込みの 14 種は Obsidian の名前で読める。知らない名前・式は None。
    assert_eq!(Summary::ALL.len(), 14);
    for sm in Summary::ALL {
        assert_eq!(Summary::parse(sm.key()), Some(sm));
    }
    assert_eq!(Summary::parse("Sum"), Some(Summary::Sum));
    assert_eq!(Summary::parse("Earliest"), Some(Summary::Earliest));
    assert_eq!(Summary::parse("customAverage"), None);
    assert_eq!(Summary::parse("values.mean()"), None);
    assert_eq!(Summary::parse(""), None);
}

#[test]
fn test_bv_14_labels_english_and_japanese() {
    // [BV-14][SR-23] 見せ方の名前は英語と日本語。英語は Obsidian の名前と同じ。
    {
        let _g = scoped(Lang::En);
        for sm in Summary::ALL {
            assert_eq!(sm.label(), sm.key());
        }
    }
    let _g = scoped(Lang::Ja);
    for sm in Summary::ALL {
        assert!(
            !sm.label().is_ascii(),
            "日本語の名前: {sm:?} {}",
            sm.label()
        );
    }
}

#[test]
fn test_bv_14_sum_average_min_max_skip_non_numbers() {
    // [BV-14] 数の集計は数だけを数える(文字・真偽・日付・空は数えない)。
    assert_eq!(compute(Summary::Sum, mixed()), n(18.0));
    assert_eq!(compute(Summary::Average, mixed()), n(6.0));
    assert_eq!(compute(Summary::Min, mixed()), n(3.0));
    assert_eq!(compute(Summary::Max, mixed()), n(10.0));
    assert_eq!(compute(Summary::Range, mixed()), n(7.0));
}

#[test]
fn test_bv_14_median_odd_and_even() {
    // [BV-14] 中央値: 奇数なら真ん中、偶数なら真ん中の2つの平均。
    assert_eq!(compute(Summary::Median, mixed()), n(5.0));
    assert_eq!(
        compute(Summary::Median, vec![n(4.0), n(1.0), n(3.0), n(2.0)]),
        n(2.5)
    );
}

#[test]
fn test_bv_14_stddev_population() {
    // [BV-14] 標準偏差は母集団の(n で割る)。2,4,4,4,5,5,7,9 → 2。
    let xs = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0].map(n);
    assert_eq!(compute(Summary::Stddev, xs), n(2.0));
    assert_eq!(compute(Summary::Stddev, vec![n(5.0)]), n(0.0));
}

#[test]
fn test_bv_14_earliest_latest_dates_only() {
    // [BV-14] 日付の集計は日付と日時だけを数える(数・文字は数えない)。
    let xs = vec![
        d("2026-10-07"),
        n(1.0),
        d("2026-10-05"),
        s("someday"),
        Val::DateTime(crate::types::parse_date("2026-10-09").unwrap() * 86_400 + 3600),
        Val::Null,
    ];
    assert_eq!(compute(Summary::Earliest, xs.clone()), d("2026-10-05"));
    assert_eq!(
        compute(Summary::Latest, xs),
        Val::DateTime(crate::types::parse_date("2026-10-09").unwrap() * 86_400 + 3600)
    );
}

#[test]
fn test_bv_14_range_of_dates_is_duration() {
    // [BV-14] 日付だけの列の Range は Latest − Earliest の期間。
    let xs = vec![d("2026-10-01"), s("x"), d("2026-10-11"), d("2026-10-05")];
    assert_eq!(
        compute(Summary::Range, xs),
        Val::Duration(10 * 86_400 * 1000)
    );
}

#[test]
fn test_bv_14_checked_unchecked_count_booleans() {
    // [BV-14] 真偽の true・false の数。文字の "true" は数えない。
    let xs = vec![
        Val::Bool(true),
        Val::Bool(false),
        Val::Bool(true),
        s("true"),
        Val::Null,
        n(1.0),
    ];
    assert_eq!(compute(Summary::Checked, xs.clone()), n(2.0));
    assert_eq!(compute(Summary::Unchecked, xs), n(1.0));
}

#[test]
fn test_bv_14_empty_filled_unique() {
    // [BV-14] 空は null・空の文字・空のリスト。種類は空でない値の種類の数。
    let xs = vec![
        s("a"),
        Val::Null,
        s(""),
        Val::List(vec![]),
        s("a"),
        s("b"),
        n(1.0),
        s("1"),
        Val::List(vec![s("x")]),
    ];
    assert_eq!(compute(Summary::Empty, xs.clone()), n(3.0));
    assert_eq!(compute(Summary::Filled, xs.clone()), n(6.0));
    assert_eq!(
        compute(Summary::Unique, xs),
        n(5.0),
        "a・b・1(数)・\"1\"(文字)・[x]"
    );
}

#[test]
fn test_bv_14_no_values_is_empty() {
    // [BV-14] 数える値が無ければ空(Null)。数えるだけの集計は 0。
    let none: Vec<Val> = Vec::new();
    for sm in [
        Summary::Average,
        Summary::Min,
        Summary::Max,
        Summary::Sum,
        Summary::Range,
        Summary::Median,
        Summary::Stddev,
        Summary::Earliest,
        Summary::Latest,
    ] {
        assert_eq!(compute(sm, none.clone()), Val::Null, "{sm:?}");
        assert_eq!(
            compute(sm, vec![s("text"), Val::Null]),
            Val::Null,
            "型の合う値が無い: {sm:?}"
        );
    }
    for sm in [
        Summary::Checked,
        Summary::Unchecked,
        Summary::Empty,
        Summary::Filled,
        Summary::Unique,
    ] {
        assert_eq!(compute(sm, none.clone()), n(0.0), "{sm:?}");
    }
}

#[test]
fn test_bv_14_many_values_one_pass() {
    // [BV-14] 値が多くても計算できる(合計があふれたら空)。
    let xs = (1..=100_000).map(|i| n(i as f64));
    assert_eq!(compute(Summary::Sum, xs), n(5_000_050_000.0));
    let big = vec![n(f64::MAX), n(f64::MAX)];
    assert_eq!(compute(Summary::Sum, big), Val::Null, "有限でない数は空");
    let uniq = (0..50_000).map(|i| n((i % 1000) as f64));
    assert_eq!(compute(Summary::Unique, uniq), n(1000.0));
}
