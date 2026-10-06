//! 英語の画面の単数と複数(SR-23、docs/todo.md の A-11)の受け入れ試験。
//! 記録: specs/_changes/2026-10-06-docs-refresh.md。
//! 英語の画面で、数が 1 なら `1 row`、2 なら `2 rows` とヘッダーに出す。日本語は変えない。

use super::test_screen::{make, screen};
use mdgrid::i18n::{scoped, Lang};

/// 画面の1行目(ヘッダー)。
fn header(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

#[test]
fn test_sr_23_plural_header_one_row() {
    // [SR-23] 英語の画面で 1 ノートならヘッダーは `1 row`(`1 rows` でない)。
    let _g = scoped(Lang::En);
    let (_t, a) = make("sr23one", &[("a.md", "---\ntitle: one\n---\n")]);
    let h = header(&screen(&a));
    assert!(h.contains(" 1 row"), "ヘッダーに `1 row` が無い: {h:?}");
    assert!(!h.contains("1 rows"), "ヘッダーが `1 rows` のまま: {h:?}");
}

#[test]
fn test_sr_23_plural_header_two_rows() {
    // [SR-23] 英語の画面で 2 ノートならヘッダーは `2 rows`。
    let _g = scoped(Lang::En);
    let (_t, a) = make(
        "sr23two",
        &[
            ("a.md", "---\ntitle: one\n---\n"),
            ("b.md", "---\ntitle: two\n---\n"),
        ],
    );
    let h = header(&screen(&a));
    assert!(h.contains(" 2 rows"), "ヘッダーに `2 rows` が無い: {h:?}");
}

#[test]
fn test_sr_23_plural_japanese_unchanged() {
    // [SR-23] 日本語の画面は 1 ノートでも `1行` のまま。
    let _g = scoped(Lang::Ja);
    let (_t, a) = make("sr23oneja", &[("a.md", "---\ntitle: one\n---\n")]);
    let h = header(&screen(&a));
    assert!(h.contains("1行"), "ヘッダーに `1行` が無い: {h:?}");
}
