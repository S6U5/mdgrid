use super::*;

#[test]
fn test_sr_9_width_and_truncate() {
    // [SR-9] 全角は2、結合文字は前の文字とまとめて1、ESC は見える文字に置き換える。
    assert_eq!(width("あい"), 4);
    assert_eq!(width("e\u{301}"), 1);
    assert_eq!(sanitize("a\u{1b}[31m"), "a␛[31m");
    assert_eq!(width(&sanitize("a\u{1b}b")), 3);
    assert_eq!(truncate("あいう", 5), "あい…");
    assert_eq!(truncate("あいう", 4), "あ…");
    assert_eq!(fit("あいう", 4, Align::Left), "あ… ");
    assert_eq!(fit("12", 4, Align::Right), "  12");
    assert_eq!(fit("x", 3, Align::Center), " x ");
    for s in ["👨‍👩‍👧", "ｱﾞ", "日本語のテキスト", "e\u{301}e\u{301}"] {
        for w in 0..10 {
            assert_eq!(width(&fit(s, w, Align::Left)), w, "{s} {w}");
        }
    }
}
