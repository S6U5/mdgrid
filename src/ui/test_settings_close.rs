//! 設定の画面を閉じたときの言葉(NV-13): 写しを変えていなければ「閉じた」、変えていれば「取り消した」。

use super::test_screen::press;
use super::test_settings_screen::{folder, open};
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_nv_13_close_without_changes_says_closed() {
    // [NV-13] 何も変えずに Esc → 「閉じた」。列を1つ隠してから Esc → 「取り消した」で表は元のまま。
    let (_t, mut a) = folder("nv13close");
    open(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.message.as_deref(), Some("ビューの設定を閉じた"));
    let cols = a.cols.clone();
    open(&mut a);
    a.draft.as_mut().unwrap().cols[1].1 = false;
    press(&mut a, KeyCode::Esc);
    assert_eq!(
        a.message.as_deref(),
        Some("ビューの設定を取り消した(表は開く前のまま)")
    );
    assert_eq!(a.cols, cols);
}
