//! screen.locales-auto の環境変数の値から台帳を選ぶ規則(`i18n::locale_index`)の試験。台帳の符号の並びは
//! screen.locales-languages の9つを渡して確かめる(台帳のファイルの有る無しによらない純関数)。

use crate::i18n::locale_index;

const CODES: [&str; 9] = [
    "en", "de", "es", "fr", "ja", "ko", "pt-BR", "zh-CN", "zh-TW",
];

fn pick(v: &str) -> Option<&'static str> {
    locale_index(v, &CODES).map(|i| CODES[i])
}

#[test]
fn test_locale_pick_language_and_region() {
    // [utest -> req~screen.locales-auto~3] 言語と地域が合う台帳、無ければ言語が合う台帳、どれも合わなければ無し(英語)。
    assert_eq!(pick("en_US.UTF-8"), Some("en"));
    assert_eq!(pick("ja_JP.UTF-8"), Some("ja"));
    assert_eq!(pick("zh_TW.UTF-8"), Some("zh-TW"));
    assert_eq!(pick("zh_HK.UTF-8"), Some("zh-TW"));
    assert_eq!(pick("zh_CN.UTF-8"), Some("zh-CN"));
    assert_eq!(pick("zh"), Some("zh-CN"));
    assert_eq!(pick("pt_PT.UTF-8"), Some("pt-BR"));
    assert_eq!(pick("de_AT.UTF-8"), Some("de"));
    assert_eq!(pick("ko_KR.UTF-8"), Some("ko"));
    assert_eq!(pick("fr_CA@euro"), Some("fr"));
    assert_eq!(pick("nl_NL.UTF-8"), None);
    assert_eq!(pick("C"), None);
    assert_eq!(pick("POSIX"), None);
}
