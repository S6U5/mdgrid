//! 境界の値の試験(SR-36・SR-39・SR-40・SR-41): 色の書き方、[look.colors]・[look.style] の型の誤り、端末の明るさの読み取り。
//! どれも落ちず(パニックせず)、読めないものは None か警告になる。

use crate::colors::{parse_color, Colors};
use crate::style::{Preset, Style};
use crate::theme::{light_from_colorfgbg, light_from_osc11, Theme};
use proptest::prelude::*;

#[test]
fn test_sr_40_color_boundaries() {
    // [SR-40] 桁の数の境目(2・3・4・5・6・7 桁)、空、# だけ、大文字の16進、前後の空白、名前の大文字。
    for bad in [
        "", "#", "#1", "#12", "#1234", "#12345", "#1234567", "#ggg", "# fff", "fff", "#-12", "red2",
    ] {
        assert_eq!(parse_color(bad), None, "{bad:?}");
    }
    assert_eq!(parse_color("#000"), Some([0, 0, 0]));
    assert_eq!(parse_color("#FFF"), Some([255, 255, 255]));
    assert_eq!(parse_color("#ffffff"), Some([255, 255, 255]));
    assert_eq!(parse_color("#000000"), Some([0, 0, 0]));
    assert_eq!(parse_color("  #A1b2C3  "), Some([0xa1, 0xb2, 0xc3]));
    assert_eq!(parse_color("RED"), parse_color("red"));
    assert!(parse_color("red").is_some());
}

#[test]
fn test_sr_40_colors_wrong_types_warn() {
    // [SR-40][SR-41] 表でない [colors]・表でない values・文字でない色・空の表 → 警告して無視(落ちない)。
    for (cfg, warns) in [
        ("[look]\ncolors = \"red\"\n", 1),
        ("[look]\ncolors = 3\n", 1),
        ("[look.colors]\nvalues = \"x\"\n", 1),
        ("[look.colors]\naccent = 3\n", 1),
        ("[look.colors]\naccent = true\n", 1),
        ("[look.colors.values]\ndone = 1\n", 1),
        ("[look.colors]\n", 0),
        ("[look.colors.values]\n", 0),
    ] {
        let (c, w) = crate::config::parse(cfg).unwrap();
        assert_eq!(w.len(), warns, "{cfg:?}: {w:?}");
        assert_eq!(c.resolved().colors, Colors::default(), "{cfg:?}");
    }
    // 同じ値(大文字・小文字・空白違い)を2回 → 警告して1つ(TOML の表は名前の順に読むので、書いた順は残らない)。
    let (c, w) =
        crate::config::parse("[look.colors.values]\ndone = \"red\"\n\" DONE \" = \"blue\"\n")
            .unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(c.resolved().colors.values.len(), 1);
    // 空の値のキーも読める(照合では空の値にだけ当たる)。
    let (c, _) = crate::config::parse("[look.colors.values]\n\"\" = \"red\"\n").unwrap();
    let c = c.resolved();
    assert_eq!(c.colors.value(""), parse_color("red"));
    assert_eq!(c.colors.value("x"), None);
}

#[test]
fn test_sr_36_style_wrong_types_warn() {
    // [SR-36] 型の誤り(preset が数・icons が文字・[style] が文字・空の文字・大文字の名前)→ 警告して既定のまま。
    for cfg in [
        "[look]\nstyle = \"saas\"\n",
        "[look]\npreset = 1\n",
        "[look]\npreset = \"\"\n",
        "[look]\npreset = \"SAAS\"\n",
        "[look.style]\nicons = \"yes\"\n",
        "[look.style]\nstatus = \"Dot\"\n",
        "[look.style]\nselect = [\"bar\"]\n",
    ] {
        let (c, w) = crate::config::parse(cfg).unwrap();
        assert_eq!(w.len(), 1, "{cfg:?}: {w:?}");
        assert_eq!(c.resolved().style, Style::default(), "{cfg:?}");
    }
    // preset は書いた順によらず先に当たる(上書きが後の preset に消されない)。
    let (c, w) =
        crate::config::parse("[look.style]\nselect = \"cross\"\n[look]\npreset = \"paper\"\n")
            .unwrap();
    assert!(w.is_empty());
    let s = c.resolved().style;
    assert_eq!(s.select.name(), "cross");
    assert_eq!(s.rules, Style::of(Preset::Paper).rules);
}

#[test]
fn test_sr_39_colorfgbg_boundaries() {
    // [SR-39] 地の番号の境目: 0〜6・8 は暗い、7・9〜15 は明るい。16 以上・負・空・数でない → 分からない。
    for (v, want) in [
        ("0;0", Some(false)),
        ("0;6", Some(false)),
        ("0;7", Some(true)),
        ("0;8", Some(false)),
        ("0;9", Some(true)),
        ("0;15", Some(true)),
        ("0;16", Some(false)),
        ("15", Some(true)),
        ("0;255", Some(false)),
        ("0;256", None),
        ("0;-1", None),
        ("", None),
        (";", None),
        ("15;", None),
        ("default;default", None),
        (" 0 ; 15 ", Some(true)),
    ] {
        assert_eq!(light_from_colorfgbg(v), want, "{v:?}");
    }
}

#[test]
fn test_sr_39_osc11_boundaries() {
    // [SR-39] 1〜4 桁の16進、5 桁、足りない成分、rgba、真ん中の灰の境目。
    assert_eq!(light_from_osc11("rgb:f/f/f"), Some(true));
    assert_eq!(light_from_osc11("rgb:0/0/0"), Some(false));
    assert_eq!(light_from_osc11("rgb:fff/fff/fff"), Some(true));
    assert_eq!(light_from_osc11("rgb:fffff/0/0"), None);
    assert_eq!(light_from_osc11("rgb:ff/ff"), None);
    assert_eq!(light_from_osc11("rgb://"), None);
    assert_eq!(light_from_osc11(""), None);
    // 輝度 0.5 の前後(灰 7f は暗い、80 より明るい灰は明るい)。
    assert_eq!(light_from_osc11("rgb:7f/7f/7f"), Some(false));
    assert_eq!(light_from_osc11("rgb:81/81/81"), Some(true));
    assert_eq!(
        Theme::auto(
            light_from_osc11("rgb:ffff/ffff/ffff"),
            Theme::Paper,
            Theme::Sumi
        ),
        Theme::Paper
    );
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn test_sr_40_parse_color_never_panics(s in ".{0,12}") {
        // [SR-40] どんな文字でも落ちない。
        let _ = parse_color(&s);
    }

    #[test]
    fn test_sr_40_hex_round_trip(r in any::<u8>(), g in any::<u8>(), b in any::<u8>()) {
        // [SR-40] #rrggbb は書いたとおりの色に読める。
        prop_assert_eq!(parse_color(&format!("#{r:02x}{g:02x}{b:02x}")), Some([r, g, b]));
        prop_assert_eq!(parse_color(&format!("#{r:02X}{g:02X}{b:02X}")), Some([r, g, b]));
    }

    #[test]
    fn test_sr_39_terminal_answers_never_panic(s in ".{0,40}") {
        // [SR-39] 端末の答えと COLORFGBG にどんな文字が来ても落ちない。
        let _ = light_from_osc11(&s);
        let _ = light_from_colorfgbg(&s);
    }
}
