//! 設定の画面の1行目(NV-18): 案内が入らない幅では、途中で切らずに案内を省く。

use super::test_screen::screen;
use super::test_settings_screen::{folder, open};

#[test]
fn test_nv_18_head_drops_hint_when_narrow() {
    // [NV-18] 広い画面では案内まで出る。80桁で未反映の数が出ると案内を省き、「…」で切れた文を出さない。
    let (_t, mut a) = folder("nv18head");
    a.resize(120, 24);
    open(&mut a);
    let top = screen(&a).lines().next().unwrap().to_string();
    assert!(top.contains("反映するまで表は変わらない"), "{top}");
    // 列を1つ隠して未反映の数を出す。
    a.draft.as_mut().unwrap().cols[1].1 = false;
    a.resize(64, 24);
    let top = screen(&a).lines().next().unwrap().to_string();
    assert!(top.contains("ビューの設定"), "{top}");
    assert!(!top.contains("反映するまで"), "{top}");
    assert!(!top.contains('…'), "{top}");
    assert!(top.contains("未反映 1"), "{top}");
    assert!(top.contains("[ 反映 ]"), "{top}");
}
