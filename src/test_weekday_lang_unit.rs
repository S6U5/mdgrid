//! 日付の形の曜日(ddd)は画面の言語の名前で見せ、打ち込みはどちらの名前も受ける(SR-23・CE-22)。

use crate::i18n::{scoped, Lang};
use crate::types::{parse_date, parse_date_input, DateFormat};

#[test]
fn test_sr_23_weekday_follows_language() {
    // [SR-23] 2026-10-01 は木曜: 英語の画面は Thu、日本語の画面は 木。打ち込みは言語によらず両方を受ける。
    let f = DateFormat::parse("DD/MM/YYYY ddd").unwrap();
    let day = parse_date("2026-10-01").unwrap();
    {
        let _g = scoped(Lang::En);
        assert_eq!(f.format(day), "01/10/2026 Thu");
        assert_eq!(
            parse_date_input("01/10/2026 木", &f, day).unwrap(),
            Some(day)
        );
        assert_eq!(
            parse_date_input("01/10/2026 Thu", &f, day).unwrap(),
            Some(day)
        );
        // 曜日の食い違いは英語の名前で知らせる。
        let e = parse_date_input("01/10/2026 Fri", &f, day).unwrap_err();
        assert!(e.contains("Thu"), "{e}");
    }
    {
        let _g = scoped(Lang::Ja);
        assert_eq!(f.format(day), "01/10/2026 木");
        assert_eq!(
            parse_date_input("01/10/2026 Thu", &f, day).unwrap(),
            Some(day)
        );
    }
}
