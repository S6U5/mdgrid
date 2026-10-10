//! nerd_font = "auto"(SR-36): 丸い端を自分で描く端末(Ghostty・WezTerm)だけ。

use super::*;

#[test]
fn test_sr_36_nerd_auto() {
    // [SR-36] TERM_PROGRAM が ghostty・WezTerm(大文字・小文字によらない)なら描ける。ほか・無し・tmux は描けない。
    for (t, want) in [
        (Some("ghostty"), true),
        (Some("WezTerm"), true),
        (Some("wezterm"), true),
        (Some(" ghostty "), true),
        (Some("Apple_Terminal"), false),
        (Some("iTerm.app"), false),
        (Some("tmux"), false),
        (Some("vscode"), false),
        (Some(""), false),
        (None, false),
    ] {
        assert_eq!(nerd_auto(t), want, "{t:?}");
    }
}

#[test]
fn test_sr_36_nerd_font_setting() {
    // [SR-36] 既定は "auto"。true・false は auto を外す。ほかの値は警告して既定(auto)のまま。
    let (c, w) = crate::config::parse("").unwrap();
    assert!(w.is_empty() && c.nerd_font_auto && !c.nerd_font);
    let (c, _) = crate::config::parse("nerd_font = true\n").unwrap();
    assert!(!c.nerd_font_auto && c.nerd_font);
    let (c, _) = crate::config::parse("nerd_font = false\n").unwrap();
    assert!(!c.nerd_font_auto && !c.nerd_font);
    let (c, w) = crate::config::parse("nerd_font = \"auto\"\n").unwrap();
    assert!(w.is_empty() && c.nerd_font_auto);
    for bad in [
        "nerd_font = \"yes\"\n",
        "nerd_font = 1\n",
        "nerd_font = \"AUTO\"\n",
    ] {
        let (c, w) = crate::config::parse(bad).unwrap();
        assert_eq!(w.len(), 1, "{bad:?}");
        assert!(c.nerd_font_auto, "{bad:?}");
    }
}
