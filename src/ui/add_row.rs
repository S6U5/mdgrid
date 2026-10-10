//! 行を足す(`impl App` の続き。SC-17)。行がノートでない表(CSV など)の `a`: 読み込み口の末尾に空の行を1つ足して
//! すぐ書き、その行の1列目の入力を開く。ためた直しは、自分が末尾に足しただけなので基準を新しい内容に進める。

use super::app::App;
use super::grid::Slot;
use super::keymap::Action;
use super::review::save_error_text;
use super::startup::READONLY;
use mdgrid::i18n::Msg;

impl App {
    pub(crate) fn add_row(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let (row, old, new) = match self.src.append_row() {
            Ok(r) => r,
            Err(e) => {
                self.message = Some(save_error_text(&e));
                return;
            }
        };
        self.changes.rebase(&old, &new);
        let name = self.src.label(&self.src.unit(&row));
        self.regrid = true;
        self.refresh();
        if let Some(i) = (0..self.slots.len())
            .find(|&i| matches!(self.slots[i], Slot::Row(k) if self.rows.get(k) == Some(&row)))
        {
            self.row = i;
            self.col = 0;
            self.scroll_into_view();
            self.apply(Action::Edit);
        }
        self.message = Some(Msg::CsvRowAdded.fill(&[&name]));
    }
}
