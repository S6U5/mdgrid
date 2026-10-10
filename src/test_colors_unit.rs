//! 色の上書き(SR-40・SR-41)の読み取り。

use super::*;

#[test]
fn test_sr_40_colors() {
    // [SR-40] #rrggbb・#rgb・名前を読み、役割を上書き。知らない役割・読めない色は警告して無視。
    assert_eq!(parse_color("#e0a458"), Some([0xe0, 0xa4, 0x58]));
    assert_eq!(parse_color("#fa0"), Some([0xff, 0xaa, 0x00]));
    assert_eq!(parse_color("Orange"), Some([234, 140, 52]));
    assert_eq!(parse_color("#zz"), None);
    assert_eq!(parse_color("chartreuse"), None);
    let (c, w) = crate::config::parse(
        "[look.colors]\naccent = \"#e0a458\"\nbackground = \"black\"\nfoo = \"red\"\nselection = \"#zz\"\n",
    )
    .unwrap();
    assert_eq!(w.len(), 2, "{w:?}");
    let c = c.resolved();
    assert_eq!(c.colors.role("accent"), Some([0xe0, 0xa4, 0x58]));
    assert_eq!(c.colors.role("background"), Some([17, 17, 17]));
    assert_eq!(c.colors.role("selection"), None);
    let p = c.colors.apply(crate::theme::Theme::Sumi.palette().unwrap());
    assert_eq!(p.colhead, [0xe0, 0xa4, 0x58]);
    assert_eq!(p.bg, [17, 17, 17]);
    assert_eq!(
        p.fg,
        crate::theme::Theme::Sumi.palette().unwrap().fg,
        "書かなかった役割はテーマのまま"
    );
}

#[test]
fn test_sr_41_value_colors_read() {
    // [SR-41] 値の色は大文字・小文字と前後の空白によらず引ける。日本語の値も。読めない色は警告。
    let (c, w) = crate::config::parse(
        "[look.colors.values]\n\"進行中\" = \"#d6a85c\"\nDone = \"green\"\nbad = \"nope\"\n",
    )
    .unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    let c = c.resolved();
    assert_eq!(c.colors.value("進行中"), Some([0xd6, 0xa8, 0x5c]));
    assert_eq!(c.colors.value(" done "), Some([70, 168, 98]));
    assert_eq!(c.colors.value("todo"), None);
}
