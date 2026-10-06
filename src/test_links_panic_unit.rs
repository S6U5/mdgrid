//! [BV-22][BV-7] 外から来るリンクの文字で止まらず、長い行でも時間が延びない
//! (specs/_changes/2026-10-06-links-stem-panic.md)。

use super::*;

#[test]
fn test_bv_22_no_panic_multibyte_tail() {
    // 末尾が多バイトの文字の行き先(拡張子 `.md` の判定で文字の途中を切らない)。
    let raws = extract("[[aあ]] [[日本語]] [x](ノート.md) [[é]] \\あ[[い]]");
    let notes = vec![
        ("aあ.md".to_string(), raws.clone()),
        ("日本語.md".to_string(), Vec::new()),
    ];
    let ix = Index::build(notes);
    let _ = ix.links("aあ.md");
    assert!(raws.iter().any(|r| r.target == "aあ"));
    assert!(raws.iter().any(|r| r.target == "い"));
    // 拡張子を外す関数に、末尾が多バイトの文字を直に渡す。
    assert_eq!(stem("aあ"), "aあ");
    // 末尾の3バイトが4バイトの文字の途中に当たる(直す前はここで止まった)。
    assert_eq!(stem("a😀"), "a😀");
    assert_eq!(stem("あé"), "あé");
    assert_eq!(stem("ノート.md"), "ノート");
    assert_eq!(stem("x.MD"), "x");
}

#[test]
fn test_bv_22_no_panic_long_hostile_lines_are_fast() {
    let n = 200_000;
    let lines = [
        "[[".repeat(n),
        "[".repeat(n) + "]",
        "`".repeat(3) + &"x`".repeat(n),
        "[a](".repeat(n),
    ];
    let started = std::time::Instant::now();
    for l in &lines {
        let _ = extract(l);
    }
    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "長い行で {:?}",
        started.elapsed()
    );
}
