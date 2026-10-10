//! 部品の形の設定(SR-36)と、新しいテーマ(SR-37)・端末に従うテーマの設定(SR-39)の読み取り。

use super::*;

fn parse(text: &str) -> (crate::config::Config, Vec<String>) {
    crate::config::parse(text).unwrap()
}

#[test]
fn test_sr_36_style_presets_and_overrides() {
    // [SR-36] 既定は sumi。preset で組をまとめて選び、ほかの項目はその上に重ねる。
    let (c, w) = parse("");
    assert!(w.is_empty());
    assert_eq!(c.style, Style::of(Preset::Sumi));
    assert_eq!(
        (c.style.status, c.style.tags, c.style.select),
        (Status::Dot, Tags::Dots, Select::Bar)
    );
    let (c, w) = parse("[style]\nselect = \"cross\"\npreset = \"saas\"\n");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(c.style.status, Status::Tint, "組の部品");
    assert_eq!(
        c.style.select,
        Select::Cross,
        "項目は組の上に(書いた順によらない)"
    );
    let (c, _) = parse("[style]\npreset = \"classic\"\n");
    assert_eq!(
        (c.style.status, c.style.select, c.style.tabs),
        (Status::Chip, Select::Fill, Tabs::Pill)
    );
}

#[test]
fn test_sr_36_style_bad_values_warn() {
    // [SR-36] 知らない値・知らない項目・型の違いは警告して既定のまま。
    for text in [
        "[style]\nstatus = \"sparkle\"\n",
        "[style]\npreset = \"neon\"\n",
        "[style]\nwobble = 1\n",
        "[style]\nicons = \"yes\"\n",
        "style = \"sumi\"\n",
    ] {
        let (c, w) = parse(text);
        assert_eq!(w.len(), 1, "{text}: {w:?}");
        assert_eq!(c.style, Style::default(), "{text}");
    }
}

#[test]
fn test_sr_37_new_themes_and_sr_39_auto() {
    // [SR-37] 新しいテーマの名前を受ける。[SR-39] theme = "auto" と theme_light・theme_dark。
    use crate::theme::Theme;
    for name in ["sumi", "slate", "saas", "saas-dark", "paper"] {
        let (c, w) = parse(&format!("theme = \"{name}\"\n"));
        assert!(w.is_empty(), "{name}: {w:?}");
        assert_eq!(c.theme.name(), name);
    }
    assert!(Theme::Saas.is_light() && Theme::Paper.is_light() && !Theme::Sumi.is_light());
    let (c, w) = parse(
        "theme = \"auto\"\ntheme_light = \"paper\"\ntheme_dark = \"saas-dark\"\nnerd_font = true\n",
    );
    assert!(w.is_empty(), "{w:?}");
    assert!(c.theme_auto && c.nerd_font);
    assert_eq!(
        (c.theme_light, c.theme_dark),
        (Theme::Paper, Theme::SaasDark)
    );
}

#[test]
fn test_sr_38_names_are_listed() {
    // [SR-38] カタログと突き合わせる名前の並び(設定の項目と、各部品の値)。
    assert_eq!(KEYS.len(), 10);
    assert!(Status::NAMES.contains(&"dot") && Tags::NAMES.contains(&"dots"));
    assert!(Preset::NAMES.contains(&"classic"));
}
