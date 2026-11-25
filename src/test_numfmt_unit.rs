//! 数の見せ方の純関数の試験(cell-view.number-format・number-format-locale)。

use super::*;
use crate::i18n::{scoped, Lang};

fn yen() -> NumFormat {
    NumFormat {
        thousands: true,
        prefix: "¥".into(),
        ..NumFormat::default()
    }
}

#[test]
fn test_numfmt_int_groups_and_prefix() {
    // [utest -> req~cell-view.number-format~1] 整数は3桁ごとに区切り、頭の字を付ける。負の数は `-` を頭の字の前に。
    let _g = scoped(Lang::Ja);
    let f = yen();
    assert_eq!(f.int(98000), "¥98,000");
    assert_eq!(f.int(-1200), "-¥1,200");
    assert_eq!(f.int(0), "¥0");
    assert_eq!(f.int(999), "¥999");
    assert_eq!(f.int(1_234_567), "¥1,234,567");
    // 大きな整数も桁を落とさない(f64 にしない)。
    assert_eq!(f.int(9_007_199_254_740_993), "¥9,007,199,254,740,993");
}

#[test]
fn test_numfmt_decimals_round_half_away() {
    // [utest -> req~cell-view.number-format~1] decimals は足りない桁を 0 で埋め、超える桁は四捨五入(0.5 は 0 から遠い方へ)。
    let _g = scoped(Lang::Ja);
    let f = NumFormat {
        thousands: true,
        decimals: Some(1),
        suffix: " km".into(),
        ..NumFormat::default()
    };
    assert_eq!(f.float(1234.56), "1,234.6 km");
    assert_eq!(f.float(3.0), "3.0 km");
    assert_eq!(f.int(3), "3.0 km");
    assert_eq!(f.float(0.25), "0.3 km");
    assert_eq!(f.float(-0.25), "-0.3 km");
    // 丸めて 0 になった負の数に `-` を付けない。
    assert_eq!(f.float(-0.04), "0.0 km");
    let none = NumFormat {
        decimals: Some(0),
        ..NumFormat::default()
    };
    assert_eq!(none.float(2.5), "3");
    assert_eq!(none.float(-2.5), "-3");
}

#[test]
fn test_numfmt_keeps_digits_without_decimals() {
    // [utest -> req~cell-view.number-format~1] decimals を書かなければ値の桁のまま(整数の数は小数点を付けない)。
    let _g = scoped(Lang::Ja);
    let f = NumFormat {
        suffix: "%".into(),
        ..NumFormat::default()
    };
    assert_eq!(f.float(45.0), "45%");
    assert_eq!(f.float(12.75), "12.75%");
    assert_eq!(f.float(f64::NAN), "NaN%");
}

#[test]
fn test_numfmt_locale_separators() {
    // [utest -> req~cell-view.number-format-locale~1] 区切りと小数点は言語の台帳の見本から: 英語・日本語・中国語・韓国語は
    // `1,234.5`、ドイツ語・スペイン語・ポルトガル語は `1.234,5`、フランス語は狭い空き U+202F と `,`。
    let f = NumFormat {
        thousands: true,
        decimals: Some(1),
        ..NumFormat::default()
    };
    for (lang, want) in [
        (Lang::En, "1,234.5"),
        (Lang::Ja, "1,234.5"),
        (Lang::ZhCn, "1,234.5"),
        (Lang::ZhTw, "1,234.5"),
        (Lang::Ko, "1,234.5"),
        (Lang::De, "1.234,5"),
        (Lang::Es, "1.234,5"),
        (Lang::PtBr, "1.234,5"),
        (Lang::Fr, "1\u{202F}234,5"),
    ] {
        let _g = scoped(lang);
        assert_eq!(f.float(1234.5), want, "{lang:?}");
    }
}

#[test]
fn test_numfmt_sample_parse() {
    // [utest -> req~cell-view.number-format-locale~1] 見本の読み方: 試しの言語の頭の字 `⟦` は飛ばす。読めない見本は None。
    assert_eq!(
        parse_sample("⟦1,234.5"),
        Some((",".to_string(), ".".to_string()))
    );
    assert_eq!(parse_sample("12345"), None);
    assert_eq!(parse_sample("abc"), None);
}
