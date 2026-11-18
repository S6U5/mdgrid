//! [cell-edit.then] 列をある値に直したとき、同じ行に一緒に入れる値(`[rules.<列>.then]`)。
//! 置き場と効かせる範囲は仮決め(tbd~then-scope)。

use super::app::App;
use super::kanban::value_for;
use mdgrid::changes::Follow;
use mdgrid::newnote::{expand_ids, Ids, Vars};
use mdgrid::writeback::NewValue;

impl App {
    /// `col` の決まりの `then` を、ためる値にする(値の文字 → (列, 値))。入れる文字は雛形の変数(今日・今の時刻・
    /// `{uuid}`・`{uid}`)を埋め、入れる先の列の型に合わせて読む。空の文字はその列を空にする。型に合わない値は入れない。
    pub(crate) fn follow(&self, col: &str) -> Follow {
        let Some(then) = self.prof.resolved.then.get(col) else {
            return Follow::new();
        };
        let vars = Vars {
            today: self.today,
            minutes: self.now.rem_euclid(86_400) / 60,
            ..Vars::default()
        };
        let ids = Ids::new();
        then.iter()
            .map(|(value, sets)| {
                let cells = sets
                    .iter()
                    .filter_map(|(c, text)| {
                        let text = expand_ids(text, &vars, &ids);
                        let v = if text.is_empty() {
                            NewValue::Null
                        } else {
                            value_for(self, c, &text).ok()?
                        };
                        Some((c.clone(), v))
                    })
                    .collect();
                (value.clone(), cells)
            })
            .collect()
    }
}
