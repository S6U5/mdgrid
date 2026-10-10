//! 部品の形の設定(SR-36)と、新しいテーマ(SR-37)・端末に従うテーマの設定(SR-39)の読み取り。

use super::*;
use crate::profile::{Resolved, ThemeSpec};

fn parse(text: &str) -> (Resolved, crate::config::Config, Vec<String>) {
    let (c, w) = crate::config::parse(text).unwrap();
    (c.resolved(), c, w)
}

#[test]
fn test_sr_36_style_presets_and_overrides() {
    // [SR-36] 既定は sumi。look.preset で組をまとめて選び、[look.style] の項目はその上に重ねる。
    let (r, _, w) = parse("");
    assert!(w.is_empty());
    assert_eq!(r.style, Style::of(Preset::Sumi));
    assert_eq!(
        (r.style.status, r.style.tags, r.style.select),
        (Status::Dot, Tags::Dots, Select::Bar)
    );
    let (r, _, w) = parse("[look.style]\nselect = \"cross\"\n[look]\npreset = \"saas\"\n");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(r.style.status, Status::Tint, "組の部品");
    assert_eq!(
        r.style.select,
        Select::Cross,
        "項目は組の上に(書いた順によらない)"
    );
    let (r, _, _) = parse("[look]\npreset = \"classic\"\n");
    assert_eq!(
        (r.style.status, r.style.select, r.style.tabs),
        (Status::Chip, Select::Fill, Tabs::Pill)
    );
}

#[test]
fn test_sr_36_style_bad_values_warn() {
    // [SR-36] 知らない値・知らない項目・型の違いは警告して既定のまま。
    for text in [
        "[look.style]\nstatus = \"sparkle\"\n",
        "[look]\npreset = \"neon\"\n",
        "[look.style]\nwobble = 1\n",
        "[look.style]\nicons = \"yes\"\n",
        "[look]\nstyle = \"sumi\"\n",
    ] {
        let (r, _, w) = parse(text);
        assert_eq!(w.len(), 1, "{text}: {w:?}");
        assert_eq!(r.style, Style::default(), "{text}");
    }
}

#[test]
fn test_sr_37_new_themes_and_sr_39_auto() {
    // [SR-37] 新しいテーマの名前を受ける。[SR-39] look.theme の "auto" と明暗の表。
    use crate::theme::Theme;
    for name in ["sumi", "slate", "saas", "saas-dark", "paper"] {
        let (r, _, w) = parse(&format!("[look]\ntheme = \"{name}\"\n"));
        assert!(w.is_empty(), "{name}: {w:?}");
        assert_eq!(r.theme.label(), name);
    }
    assert!(Theme::Saas.is_light() && Theme::Paper.is_light() && !Theme::Sumi.is_light());
    let (r, c, w) = parse(
        "[look]\ntheme = { light = \"paper\", dark = \"saas-dark\" }\n\n[terminal]\nnerd_font = true\n",
    );
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(c.terminal.nerd_font, crate::config::NerdFont::On);
    assert_eq!(
        r.theme,
        ThemeSpec::Pair {
            light: Theme::Paper,
            dark: Theme::SaasDark
        }
    );
    let (r, _, _) = parse("[look]\ntheme = \"auto\"\n");
    assert_eq!(r.theme, ThemeSpec::AUTO);
    let (r, _, _) = parse("[look]\ntheme = { light = \"paper\" }\n");
    assert_eq!(r.theme.pick(Some(true)), Theme::Paper);
    assert_eq!(r.theme.pick(Some(false)), Theme::Sumi, "省いた側は既定");
}

#[test]
fn test_sr_38_names_are_listed() {
    // [SR-38] カタログと突き合わせる名前の並び(設定の項目と、各部品の値)。
    assert_eq!(KEYS.len(), 10);
    assert!(Status::NAMES.contains(&"dot") && Tags::NAMES.contains(&"dots"));
    assert!(Preset::NAMES.contains(&"classic") && Links::NAMES.contains(&"accent"));
}
