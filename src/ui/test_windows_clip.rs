//! Windows の OS のクリップボード(OUT-1。specs/_changes/2026-10-15-windows-target.md)。

use super::*;

#[test]
fn test_out_1_windows_uses_clip() {
    // [OUT-1] Windows は clip に Windows の文字の形で渡す。macOS・ほかの Unix は今までどおり。
    assert_eq!(
        clip_commands("windows"),
        &[("clip", &[] as &[&str], ClipInput::WindowsText)]
    );
    assert_eq!(clip_commands("macos")[0].0, "pbcopy");
    if cfg!(unix) {
        let linux: Vec<&str> = clip_commands("linux").iter().map(|c| c.0).collect();
        assert_eq!(linux, vec!["wl-copy", "xclip"]);
    }
    assert!(clip_commands("linux")
        .iter()
        .all(|c| c.2 == ClipInput::Utf8));
}

#[test]
fn test_out_1_windows_text_is_utf16le_bom_crlf() {
    // [OUT-1] clip には BOM つきの UTF-16LE。改行は CRLF にし、元の CRLF は二重にしない。日本語も1字ずつ。
    let got = ClipInput::WindowsText.encode("a\tあ\nb\r\n");
    let mut want = vec![0xFF, 0xFE];
    for u in "a\tあ\r\nb\r\n".encode_utf16() {
        want.extend(u.to_le_bytes());
    }
    assert_eq!(got, want);
    assert_eq!(&got[..2], &[0xFF, 0xFE]);
    assert_eq!(&got[4..6], &[0x09, 0x00]);
    assert_eq!(&got[6..8], &[0x42, 0x30]);
    // UTF-8 の形はそのまま。
    assert_eq!(ClipInput::Utf8.encode("a\nあ"), "a\nあ".as_bytes());
}
