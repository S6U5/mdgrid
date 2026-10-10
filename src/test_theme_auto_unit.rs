//! 端末に従うテーマ(SR-39): COLORFGBG と OSC 11 の答えの読み取りと、theme_light・theme_dark の選び。

use super::*;

#[test]
fn test_sr_39_auto_theme() {
    // [SR-39] COLORFGBG=0;15(明るい地)→ theme_light、15;0 → theme_dark、分からない → theme_dark。
    let (light, dark) = (Theme::Saas, Theme::Sumi);
    assert_eq!(light_from_colorfgbg("0;15"), Some(true));
    assert_eq!(light_from_colorfgbg("15;0"), Some(false));
    assert_eq!(light_from_colorfgbg("15;default;0"), Some(false));
    assert_eq!(light_from_colorfgbg("12;7"), Some(true));
    assert_eq!(light_from_colorfgbg("default"), None);
    assert_eq!(Theme::auto(Some(true), light, dark), Theme::Saas);
    assert_eq!(Theme::auto(Some(false), light, dark), Theme::Sumi);
    assert_eq!(Theme::auto(None, light, dark), Theme::Sumi);
}

#[test]
fn test_sr_39_osc11_reply() {
    // [SR-39] 端末の答え(4桁・2桁の16進)を読む。読めない答えは分からない。
    assert_eq!(
        light_from_osc11("\x1b]11;rgb:ffff/ffff/ffff\x1b\\"),
        Some(true)
    );
    assert_eq!(
        light_from_osc11("\x1b]11;rgb:1616/1717/1b1b\x07"),
        Some(false)
    );
    assert_eq!(
        light_from_osc11("\x1b]11;rgb:f6/f7/f9\x07\x1b[?62;c"),
        Some(true)
    );
    assert_eq!(light_from_osc11("\x1b[?62;c"), None);
    assert_eq!(light_from_osc11("\x1b]11;rgb:zz/00\x07"), None);
}
