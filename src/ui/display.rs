//! 表の見せ方の切り替え(SR-20・SR-21。`impl App` の続きと描画の材料)。
//! 決まった値は 範囲を重ねた設定(`App.display`。SR-44)→ ビューの上書き(`settings.display`)の順。
//! 描画(view.rs・bands.rs)はここで値を引き、ビューの設定の画面の「表示」の節(settings.rs・settings_view.rs)は
//! 写しの上書きをここで切り替える。

use super::app::{App, ColorMode};
use super::grid::Slot;
use super::view::Layout;
use super::width::{fit, Align};
use mdgrid::display::{Item, ITEMS};
use mdgrid::i18n::Msg;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

impl App {
    /// 見せ方 `item` を今のビューで出すか(設定 → ビューの上書き)。
    pub(crate) fn shows(&self, item: Item) -> bool {
        self.settings.display.resolve(item, &self.display)
    }

    /// ビューの設定の画面の写しで、見せ方 `item` が入っているか(設定 → 写しの上書き)。
    pub(crate) fn draft_shows(&self, item: Item) -> bool {
        match &self.draft {
            Some(d) => d.s.display.resolve(item, &self.display),
            None => self.shows(item),
        }
    }

    /// ビューの設定の画面の写しの、タブの行の値(SR-34)。
    pub(crate) fn draft_tabs(&self) -> mdgrid::display::TabsMode {
        match &self.draft {
            Some(d) => d.s.display.over(&self.display).tabs,
            None => self.settings.display.over(&self.display).tabs,
        }
    }

    /// ビューの設定の画面の写しの、帯のボタンの描き方(NV-23)。
    pub(crate) fn draft_buttons(&self) -> mdgrid::display::ButtonsMode {
        match &self.draft {
            Some(d) => d.s.display.over(&self.display).buttons,
            None => self.settings.display.over(&self.display).buttons,
        }
    }

    /// ビューの設定の画面の写しの、帯の上下の余白の行の数(NV-23)。
    pub(crate) fn draft_bar_padding(&self) -> mdgrid::display::Pad {
        match &self.draft {
            Some(d) => d.s.display.over(&self.display).bar_padding,
            None => self.settings.display.over(&self.display).bar_padding,
        }
    }

    /// ビューの設定の画面の写しの、ヘッダーの上の余白(SR-1)。
    pub(crate) fn draft_top_padding(&self) -> mdgrid::display::Pad {
        match &self.draft {
            Some(d) => d.s.display.over(&self.display).top_padding,
            None => self.settings.display.over(&self.display).top_padding,
        }
    }

    /// ビューの設定の画面の写しの、行を伸ばす数(CV-8)。
    pub(crate) fn draft_row_extra_lines(&self) -> u8 {
        match &self.draft {
            Some(d) => d.s.display.over(&self.display).row_extra_lines,
            None => self.settings.display.over(&self.display).row_extra_lines,
        }
    }

    /// 「表示」の節の i 番目の項目を切り替える(写しだけ。反映で表に効く)。設定と同じ値に戻したら上書きを外す。
    /// タブの行は always → auto → never の順に変える(SR-34)。帯のボタンは plain ↔ box、帯の余白と上の余白は 0〜2 を 0.5 刻み(NV-23・SR-1)。
    /// 行を伸ばす数は 0 → 1 → 2 → 3 → 0(CV-8)。
    pub(crate) fn toggle_display(&mut self, i: usize) {
        let Some(&(item, _)) = ITEMS.get(i) else {
            return;
        };
        let base = self.display;
        if item == Item::Tabs {
            let next = self.draft_tabs().next();
            if let Some(d) = self.draft.as_mut() {
                d.s.display.set_tabs(next, &base);
            }
            return;
        }
        // NV-23・SR-1: 帯の余白と上の余白は 0 → 0.5 → 1 → 1.5 → 2 → 0 の順に変える。
        if item == Item::BarPadding {
            let next = self.draft_bar_padding().next();
            if let Some(d) = self.draft.as_mut() {
                d.s.display.set_bar_padding(next, &base);
            }
            return;
        }
        if item == Item::RowExtraLines {
            let next = mdgrid::display::next_row_extra_lines(self.draft_row_extra_lines());
            if let Some(d) = self.draft.as_mut() {
                d.s.display.set_row_extra_lines(next, &base);
            }
            return;
        }
        if item == Item::TopPadding {
            let next = self.draft_top_padding().next();
            if let Some(d) = self.draft.as_mut() {
                d.s.display.set_top_padding(next, &base);
            }
            return;
        }
        // NV-23: 帯のボタンの描き方は plain ↔ box。
        if item == Item::Buttons {
            let next = self.draft_buttons().next();
            if let Some(d) = self.draft.as_mut() {
                d.s.display.set_buttons(next, &base);
            }
            return;
        }
        let on = !self.draft_shows(item);
        if let Some(d) = self.draft.as_mut() {
            d.s.display.set(item, on, &base);
        }
    }
}

/// 行番号の欄の幅(数の桁と後ろの空白1つ)。出さなければ 0。
pub(crate) fn num_width(app: &App) -> usize {
    if !app.shows(Item::RowNumbers) {
        return 0;
    }
    app.rows.len().max(1).to_string().len() + 1
}

/// 列の間の区切り(列の区切り線を入れれば `│`、無ければ空白)。いつも幅1(列の位置・クリック・入力の
/// 位置は区切りを1桁として数える)。`│` は East Asian Ambiguous で、`ambiguous_wide`(CV-6)では幅2に
/// なるので、そのときは幅1の `|` を引く。
pub(crate) fn col_sep(app: &App) -> &'static str {
    use mdgrid::style::Rules;
    let lines = matches!(super::look::rules(app), Rules::Columns | Rules::Grid);
    if !app.shows(Item::ColumnLines) && !lines {
        " "
    } else if app.ambiguous_wide || app.style.frames == mdgrid::style::Frames::Ascii {
        "|"
    } else {
        "│"
    }
}

/// 表の行 i(`slots` の添字)の行番号の欄: 今の表示の並びの1始まりの数を右に寄せ、後ろに空白1つ。
/// 見出しの行は空白だけ。行番号を出さなければ None。
pub(crate) fn num_span(app: &App, lay: &Layout, i: usize) -> Option<Span<'static>> {
    if lay.num_w == 0 {
        return None;
    }
    let n = match app.slots.get(i) {
        Some(Slot::Row(r)) => (r + 1).to_string(),
        _ => String::new(),
    };
    Some(Span::styled(
        format!("{} ", fit(&n, lay.num_w - 1, Align::Right)),
        Style::default().add_modifier(Modifier::DIM),
    ))
}

/// 一行おきの色: 表の行 i(`slots` の添字)がノートの偶数番目(行番号 2・4・6…。グループの見出しは数えない)なら、
/// 背景と前景を付けた行。色を使わない表示では付けない(SR-15)。
/// 前景も決めるので、端末の背景が明るくても暗くても読め、薄い(DIM)セルも背景との差が残る。
/// セルが自分の前景・背景(ためた値の色・一致の強調)を持てば、それが勝つ。
pub(crate) fn zebra(app: &App, i: usize, line: Line<'static>) -> Line<'static> {
    let even = matches!(app.slots.get(i), Some(Slot::Row(r)) if r % 2 == 1);
    if !app.shows(Item::Zebra) || !even {
        return line;
    }
    let (bg, fg) = match app.color {
        ColorMode::None => return line,
        ColorMode::Indexed => (Color::Indexed(237), Color::Indexed(252)),
        ColorMode::Rgb => (Color::Rgb(58, 58, 58), Color::Rgb(208, 208, 208)),
    };
    line.style(Style::default().bg(bg).fg(fg))
}

impl App {
    /// SR-20・NV-22: 設定の帯を隠しているときの帯の操作(`f`・←→)では、選んだ条件をメッセージ行に出す
    /// (外す前に何を外すか見える)。帯を出していれば何もしない。
    /// 帯の条件を外したあとの知らせ。帯を隠していて条件が残っていれば、次に選ばれた条件も見せる
    /// (見えない条件を続けて外さないため。SR-15)。
    pub(crate) fn removed_chip_message(&mut self, text: &str) {
        let done = Msg::ChipRemoved.fill(&[&text]);
        self.message = None;
        self.hidden_chip_hint();
        self.message = Some(match self.message.take() {
            Some(next) => Msg::TwoMessages.fill(&[&done, &next]),
            None => done,
        });
    }

    pub(crate) fn hidden_chip_hint(&mut self) {
        if self.shows(Item::Chips) || self.mode != super::keymap::Mode::Chips {
            return;
        }
        let chips = self.settings.chips();
        let Some(c) = chips.get(self.chip) else {
            return;
        };
        let rm = self.key_of(
            super::keymap::Mode::Chips,
            super::keymap::Action::RemoveItem,
        );
        self.message = Some(Msg::HiddenChip.fill(&[&(self.chip + 1), &chips.len(), &c, &rm]));
    }
}
