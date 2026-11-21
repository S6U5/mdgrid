//! ビューの設定の画面の左の区画の一覧をキーで動かす(`impl App` の続き)。
//! navigation.settings-sections-keys・navigation.settings-sections-digits・navigation.settings-sections-focus。
//! キーが効く側(左の一覧か右の項目か)は `Draft::nav` に持つ。`h`・← で左へ、左では `j`・`k`・↑↓ で区画を選び
//! (右はその区画の中身に替わる)、`l`・→・Enter で右の項目へ戻る。数字のキーはどちらの側からでも、その番号の
//! 区画へ移る(側は替えない)。Tab・Shift+Tab と、項目・ボタンの中の動作は settings.rs のまま。描画は settings_view.rs。

use super::app::App;
use super::keymap::Action;
use super::settings::{secs, Sec};

/// 左の一覧に並べる区画(Tab の順からボタンを除く)。数字のキーの番号はこの並びの 1 から。
pub(crate) fn nav_secs(notes: bool) -> Vec<Sec> {
    secs(notes)
        .into_iter()
        .filter(|s| *s != Sec::Buttons)
        .collect()
}

impl App {
    /// 左の一覧と数字のキーの動作。扱ったら true(ほかは区画の中の動作へ回す)。選び手の上では呼ばない。
    pub(crate) fn settings_nav_action(&mut self, action: Action) -> bool {
        let list = nav_secs(self.src.notes());
        let Some(d) = self.draft.as_mut() else {
            return false;
        };
        if let Action::GoToSection(n) = action {
            // 区画の数より大きい番号は何も変えない。
            if let Some(sec) = usize::from(n).checked_sub(1).and_then(|i| list.get(i)) {
                d.sec = *sec;
            }
            return true;
        }
        if !d.nav {
            // 右の項目から左の一覧へ(ボタンの行の ←→ はボタンの中で動く)。
            if action == Action::Left && d.sec != Sec::Buttons {
                d.nav = true;
                return true;
            }
            return false;
        }
        let k = list.iter().position(|s| *s == d.sec).unwrap_or(0);
        match action {
            Action::Up => d.sec = list[k.saturating_sub(1)],
            Action::Down => d.sec = list[(k + 1).min(list.len() - 1)],
            Action::Right | Action::Run => d.nav = false,
            // 区画を巡る・取り消す(Esc)は右の項目にいるときと同じ。
            Action::NextSection | Action::PrevSection | Action::Cancel => return false,
            // 項目に効く動作(切り替え・順・消す)は、項目を見せていない左ではしない。
            _ => {}
        }
        true
    }
}
