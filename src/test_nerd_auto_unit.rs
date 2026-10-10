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
    use crate::config::NerdFont;
    let nerd = |text: &str| {
        let (c, w) = crate::config::parse(text).unwrap();
        (c.terminal.nerd_font, w)
    };
    let (n, w) = nerd("");
    assert!(w.is_empty() && n == NerdFont::Auto && !n.resolve(None));
    assert_eq!(nerd("[terminal]\nnerd_font = true\n").0, NerdFont::On);
    assert_eq!(nerd("[terminal]\nnerd_font = false\n").0, NerdFont::Off);
    let (n, w) = nerd("[terminal]\nnerd_font = \"auto\"\n");
    assert!(w.is_empty() && n == NerdFont::Auto && n.resolve(Some("ghostty")));
    for bad in [
        "[terminal]\nnerd_font = \"yes\"\n",
        "[terminal]\nnerd_font = 1\n",
        "[terminal]\nnerd_font = \"AUTO\"\n",
    ] {
        let (n, w) = nerd(bad);
        assert_eq!(w.len(), 1, "{bad:?}");
        assert_eq!(n, NerdFont::Auto, "{bad:?}");
    }
}
