//! 移動・検索・簡易の絞り込み・選択・同じ値(`impl App` の続き)。
//! NV-1(`/`・`n` `N`・smartcase・一致の強調)、NV-2(`\` の簡易の絞り込み)、NV-5(Space・`v`・Shift+矢印・
//! Ctrl+A・Esc)、NV-7(`0` `$`・Ctrl+D / Ctrl+U)、NV-8(`*` の強調・`,` の絞り込み)、SR-18(Esc の二段)。
//! 絞り込みと並べ替えを表に重ねるのは grid.rs(`overlay`)。

use super::app::App;
use super::cell::{shown, val_text};
use super::external::{plain, plain_new};
use super::grid::{value_of, Slot};
use super::keymap::{Action, Mode};
use mdgrid::base::Shown;
use mdgrid::i18n::Msg;
use mdgrid::print::{self, PrintCell};
use mdgrid::source::RowId;
use std::collections::HashSet;

/// `--pick` で選んでいる間にエディタで開こうとしたときの文(OUT-3・WB-15)。
pub(crate) const PICK_NO_EDITOR: Msg = Msg::PickNoEditor;

/// 下の行の入力(検索 NV-1・簡易の絞り込み NV-2)。
pub(crate) struct Prompt {
    pub text: String,
    /// 開いたときの選択(検索の取り消しで戻る)と、検索の語。
    origin: (usize, usize, usize, Option<String>),
}

/// 同じ値の行だけに絞る条件の1つ(NV-8 の `,`・NV-9 の頻度表)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Same {
    pub col: String,
    /// 値(空の文字は値の無い行)。
    pub value: String,
    /// 値の鍵(NV-19 の数え方。リストは要素ごと、空は値の無い行)で比べる(頻度表から)。false は素の値の文字で比べる(`,`)。
    pub by_keys: bool,
    /// リストの列の要素で絞った(ヘッダーは「含む」の形で出す)。
    pub list: bool,
}

/// smartcase の一致(NV-1): 語に大文字があれば区別し、無ければ区別しない。空の語は一致しない。
pub(crate) fn matches(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    if query.chars().any(char::is_uppercase) {
        text.contains(query)
    } else {
        text.to_lowercase().contains(query)
    }
}

impl App {
    /// セルの素の値(ためた値を重ねる。null とキーの無いセルは空。計算の列は値の文字)。
    /// 簡易の絞り込み(NV-2)と同じ値(NV-8)で比べる。
    pub(crate) fn plain(&self, row: &RowId, col: &str) -> String {
        if let Some(nv) = self.changes.pending(row, col) {
            return plain_new(nv);
        }
        match self.cell(row, col) {
            Shown::Prop(c) => c.value.as_ref().map(plain).unwrap_or_default(),
            Shown::Computed(v) => val_text(&v),
            Shown::Unsupported(_) => String::new(),
        }
    }

    /// 簡易の絞り込み(NV-2)と同じ値の絞り込み(NV-8)を通るか。
    pub(crate) fn passes(&self, row: &RowId) -> bool {
        for s in &self.same {
            let hit = if s.by_keys {
                // NV-9: 頻度表の値(リストは要素ごと、空は値の無い行)。
                let key = (!s.value.is_empty()).then(|| s.value.clone());
                mdgrid::settings::value_keys(self.setting_value(row, &s.col).as_ref())
                    .contains(&key)
            } else {
                self.plain(row, &s.col) == s.value
            };
            if !hit {
                return false;
            }
        }
        match &self.filter {
            None => true,
            Some(q) => {
                matches(q, &self.src.label(row))
                    || self.cols.iter().any(|c| matches(q, &self.plain(row, c)))
            }
        }
    }

    /// 検索の語に一致するか(見せている文字で。NV-1)。`col` が None ならノートの名前。
    pub(crate) fn search_hit(&self, row: &RowId, col: Option<&str>) -> bool {
        let Some(q) = &self.search else {
            return false;
        };
        match col {
            None => matches(q, &self.src.label(row)),
            Some(c) => matches(q, &shown(self, row, c).text),
        }
    }

    /// 表のモードの、移動・検索・選択の動作。扱ったら true。
    pub(crate) fn nav_action(&mut self, action: Action) -> bool {
        let last_row = self.slots.len().saturating_sub(1);
        let half = (self.data_height() / 2).max(1);
        match action {
            Action::HalfPageDown => self.row = (self.row + half).min(last_row),
            Action::HalfPageUp => self.row = self.row.saturating_sub(half),
            Action::Search => self.open_prompt(Mode::Search),
            Action::QuickFilter => self.open_prompt(Mode::Filter),
            Action::SearchNext => self.search_step(true),
            Action::SearchPrev => self.search_step(false),
            Action::Mark => {
                if let Some(r) = self.cur_row() {
                    if !self.marked.remove(&r) {
                        self.marked.insert(r);
                    }
                    self.row = (self.row + 1).min(last_row);
                }
            }
            Action::Visual => {
                if self.anchor.is_some() {
                    // 範囲を印にして決める。
                    let range = self.anchor_range_of(self.row);
                    self.anchor = None;
                    self.marked.extend(range);
                } else if let Some(r) = self.cur_row() {
                    self.anchor = Some(r);
                    self.message = Some(Msg::VisualHint.into());
                }
            }
            Action::ExtendUp | Action::ExtendDown => {
                if self.anchor.is_none() {
                    self.anchor = self.cur_row();
                }
                self.row = if action == Action::ExtendUp {
                    self.row.saturating_sub(1)
                } else {
                    (self.row + 1).min(last_row)
                };
            }
            Action::SelectAll => {
                self.marked.extend(self.rows.iter().cloned());
                self.anchor = None;
                self.message = Some(Msg::SelectedAll.fill(&[&self.rows.len()]));
            }
            Action::Escape => self.escape(),
            Action::HighlightSame => self.highlight_same(),
            Action::FilterSame => self.filter_same(),
            _ => return false,
        }
        true
    }

    /// Esc(SR-18): 選択があれば選択を、無ければ簡易の絞り込み(と同じ値の絞り込み)を、
    /// それも無ければ検索と同じ値の強調を解く。
    fn escape(&mut self) {
        if self.anchor.is_some() || !self.marked.is_empty() {
            self.anchor = None;
            self.marked.clear();
            self.message = Some(Msg::SelectionCleared.into());
        } else if self.filter.is_some() || !self.same.is_empty() {
            self.filter = None;
            self.same.clear();
            self.relayout();
            self.message = Some(Msg::FilterCleared.into());
        } else if self.search.is_some() || self.same_mark.is_some() {
            self.search = None;
            self.same_mark = None;
        }
    }

    /// 絞り込み・並べ替えを変えたら、直した行を留めるのをやめて組み立て直す(NV-12)。
    pub(crate) fn relayout(&mut self) {
        self.stay.clear();
        self.regrid = true;
    }

    /// v の範囲(錨の行から `to` までのノートの行)。錨が表に無ければ空。
    fn anchor_range_of(&self, to: usize) -> Vec<RowId> {
        let Some((a, b)) = self.anchor_span(to) else {
            return Vec::new();
        };
        (a..=b)
            .filter_map(|i| match self.slots.get(i)? {
                Slot::Row(r) => self.rows.get(*r).cloned(),
                Slot::Head(_) | Slot::Gap => None,
            })
            .collect()
    }

    /// 錨の行と `to` の画面の行の範囲(小さい順)。
    pub(crate) fn anchor_span(&self, to: usize) -> Option<(usize, usize)> {
        let a = self.anchor.as_ref()?;
        let i = self.slots.iter().position(|s| match s {
            Slot::Row(r) => self.rows.get(*r) == Some(a),
            Slot::Head(_) | Slot::Gap => false,
        })?;
        Some((i.min(to), i.max(to)))
    }

    /// `--pick` で選び始める(OUT-3)。読むだけ(WB-15)は起動の側で当てる。表の Enter の案内を「選ぶ」にする。
    pub fn start_choosing(&mut self) {
        self.choosing = true;
        for b in &mut self.keys {
            if b.mode == Mode::Table && b.action == Action::Edit {
                b.msg = mdgrid::i18n::Msg::KeyPickFinish;
                b.label = b.msg.ja();
                b.rank = 0;
            }
        }
    }

    /// `--pick` で選んだ行(表の並び)。取りやめたか、まだ選んでいなければ None。
    pub fn chosen(&self) -> Option<&[RowId]> {
        self.chosen.as_deref()
    }

    /// `--pick` の表の動作(OUT-3)。Enter はノートの行なら印を付けた行(無ければ選んでいる行)を決めて終わる
    /// (見出しの行は開閉のまま)。Esc は印か範囲があれば解き(今どおり)、無ければ取りやめ。扱ったら true。
    pub(crate) fn choose_action(&mut self, action: Action) -> bool {
        match action {
            Action::Edit if self.cur_head().is_none() => {
                let mut rows = self.selection();
                if rows.is_empty() {
                    rows.extend(self.cur_row());
                }
                if rows.is_empty() {
                    self.message = Some(Msg::NoRowToSelect.into());
                } else {
                    self.chosen = Some(rows);
                    self.quit = true;
                }
                true
            }
            // SR-18 の Esc の順(選択 → 絞り込み → 検索・強調)は保ち、解くものが何も無いときだけ取りやめ。
            Action::Escape
                if self.anchor.is_none()
                    && self.marked.is_empty()
                    && self.filter.is_none()
                    && self.same.is_empty()
                    && self.search.is_none()
                    && self.same_mark.is_none() =>
            {
                self.quit = true;
                true
            }
            // WB-15: エディタはノートを書き換えられ、その画面が標準出力の結果に流れるので開かない。
            Action::OpenEditor => {
                self.message = Some(PICK_NO_EDITOR.into());
                true
            }
            _ => false,
        }
    }

    /// `--pick` のコピー(OUT-1)の OSC 52 を、画面と同じ端末(`/dev/tty`)へ送る(標準出力の結果に混ぜない)。
    pub fn copy_to_tty(&mut self) {
        self.clipboard = Box::new(super::external::TtyClipboard);
    }

    /// `--pick <列>` の列の名前を表の列の id にする(id か見出しで。隠した列も含む)。無ければ None。
    pub fn find_column(&self, name: &str) -> Option<String> {
        let all = || {
            self.cols
                .iter()
                .chain(self.hidden.iter().map(|(c, _, _)| c))
        };
        all()
            .find(|c| *c == name)
            .or_else(|| all().find(|c| self.title(c) == name))
            .cloned()
    }

    /// 表の列の名前(見せている列と隠した列。`--pick` の誤りの案内)。
    pub fn column_names(&self) -> Vec<String> {
        self.cols
            .iter()
            .chain(self.hidden.iter().map(|(c, _, _)| c))
            .map(|c| self.title(c))
            .collect()
    }

    /// `--pick <列>` の1行(OUT-3): `--print` と同じ素の文字(印なし・リストは `, `・日付は YYYY-MM-DD)で、
    /// 改行は空白。ためた値があれば重ねる。
    pub fn pick_text(&self, row: &RowId, col: &str) -> String {
        let cell = match self.cell(row, col) {
            Shown::Prop(c) => PrintCell::Prop(match self.changes.pending(row, col) {
                Some(nv) => Some(value_of(nv)),
                None => c.value,
            }),
            Shown::Computed(v) => PrintCell::Computed(v),
            Shown::Unsupported(_) => PrintCell::Unsupported,
        };
        print::one_line(&print::plain(&cell))
    }

    /// 選んだ行(印と v の範囲。表の並びで。NV-5)。
    pub(crate) fn selection(&self) -> Vec<RowId> {
        let range: HashSet<RowId> = self.anchor_range_of(self.row).into_iter().collect();
        if self.marked.is_empty() && range.is_empty() {
            return Vec::new();
        }
        self.rows
            .iter()
            .filter(|r| self.marked.contains(*r) || range.contains(*r))
            .cloned()
            .collect()
    }

    /// `*`: 選んだセルと同じ値の行を強調する(NV-8)。同じセルでもう一度押すと解く。
    fn highlight_same(&mut self) {
        let Some((row, col)) = self.selected() else {
            self.message = Some(Msg::SelectValueCell.into());
            return;
        };
        let v = self.plain(&row, &col);
        if self.same_mark.as_ref() == Some(&(col.clone(), v.clone())) {
            self.same_mark = None;
            return;
        }
        let n = self
            .rows
            .iter()
            .filter(|r| self.plain(r, &col) == v)
            .count();
        self.message = Some(Msg::HighlightSame.fill(&[&self.title(&col), &shown_value(&v), &n]));
        self.same_mark = Some((col, v));
    }

    /// `,`: 選んだセルと同じ値の行だけに絞る(NV-8)。同じ条件でもう一度押すと解く。
    fn filter_same(&mut self) {
        let Some((row, col)) = self.selected() else {
            self.message = Some(Msg::SelectValueCell.into());
            return;
        };
        let v = self.plain(&row, &col);
        let label = format!("{} = {}", self.title(&col), shown_value(&v));
        if matches!(self.same.as_slice(), [s] if s.col == col && s.value == v) {
            self.same.clear();
            self.message = Some(Msg::SameFilterCleared.fill(&[&label]));
        } else {
            self.same = vec![Same {
                col,
                value: v,
                by_keys: false,
                list: false,
            }];
            self.message = Some(Msg::SameFilterOn.fill(&[&label]));
        }
        self.relayout();
    }

    /// `/`・`\`: 下の行の入力を開く。
    fn open_prompt(&mut self, mode: Mode) {
        let text = match mode {
            Mode::Filter => self.filter.clone().unwrap_or_default(),
            _ => String::new(),
        };
        self.prompt = Some(Prompt {
            text,
            origin: (self.row, self.col, self.top, self.search.clone()),
        });
        self.set_mode(mode);
    }

    /// 入力に文字を足す(打つたびに検索・絞り込みを当てる)。
    pub(crate) fn prompt_insert(&mut self, c: char) {
        self.message = None;
        if let Some(p) = &mut self.prompt {
            p.text.push(c);
        }
        self.prompt_changed();
        self.refresh_if_needed();
    }

    fn prompt_changed(&mut self) {
        let Some(p) = &self.prompt else {
            return;
        };
        let text = p.text.clone();
        let (row, col, top, _) = p.origin.clone();
        if self.mode == Mode::Search {
            self.search = (!text.is_empty()).then_some(text);
            self.row = row;
            self.col = col;
            self.top = top;
            self.search_at = None;
            if self.search.is_some() && !self.search_from(true, true) {
                self.message = Some(Msg::NotFound.into());
            }
        } else {
            self.filter = (!text.is_empty()).then_some(text);
            self.relayout();
        }
    }

    /// 検索・絞り込みの入力の動作。
    pub(crate) fn prompt_action(&mut self, action: Action) {
        match action {
            Action::Commit => {
                // NV-1: 一致の無い語で確定しても「見つからない」を残す(表へ戻るとメッセージが消えるため)。
                let missed = (self.mode == Mode::Search && self.search_at.is_none())
                    .then(|| self.search.clone())
                    .flatten();
                self.prompt = None;
                self.set_mode(Mode::Table);
                if let Some(term) = missed {
                    self.message = Some(Msg::NotFoundTerm.fill(&[&term]));
                }
            }
            Action::Cancel => {
                let p = self.prompt.take();
                if self.mode == Mode::Search {
                    // 取り消し: 開く前の選択と語に戻る。
                    if let Some(p) = p {
                        let (row, col, top, before) = p.origin;
                        self.row = row;
                        self.col = col;
                        self.top = top;
                        self.search = before;
                    }
                } else {
                    // NV-2: Esc で簡易の絞り込みを解く。
                    self.filter = None;
                    self.relayout();
                }
                self.set_mode(Mode::Table);
            }
            Action::DeleteBack => {
                if let Some(p) = &mut self.prompt {
                    p.text.pop();
                }
                self.prompt_changed();
            }
            _ => {}
        }
    }

    /// `n` `N`: 次・前の一致へ(端で回る。NV-1)。
    fn search_step(&mut self, forward: bool) {
        if self.search.is_none() {
            self.message = Some(Msg::NoSearchTerm.into());
            return;
        }
        if !self.search_from(forward, false) {
            self.message =
                Some(Msg::NotFoundTerm.fill(&[&self.search.clone().unwrap_or_default()]));
        }
    }

    /// 今の位置から一致を探して移る。`inclusive` なら今の位置も含める。見つかれば true。
    /// 位置は (画面の行, k)。k = 0 はノートの名前、k = j + 1 は列 j。
    fn search_from(&mut self, forward: bool, inclusive: bool) -> bool {
        let n = self.slots.len();
        let m = self.cols.len() + 1;
        if n == 0 {
            return false;
        }
        let here = match self.search_at {
            Some((i, k)) if i == self.row => k,
            _ => self.col + 1,
        };
        let total = n * m;
        let start = self.row * m + here;
        let first = if inclusive { 0 } else { 1 };
        for step in first..=total {
            let p = if forward {
                (start + step) % total
            } else {
                (start + total * 2 - step) % total
            };
            let (i, k) = (p / m, p % m);
            let Some(Slot::Row(r)) = self.slots.get(i) else {
                continue;
            };
            let Some(row) = self.rows.get(*r) else {
                continue;
            };
            let col = (k > 0).then(|| self.cols[k - 1].as_str());
            if self.search_hit(row, col) {
                self.row = i;
                if k > 0 {
                    self.col = k - 1;
                }
                self.search_at = Some((i, k));
                if forward && p < start && !inclusive {
                    self.message = Some(Msg::WrappedToTop.into());
                } else if !forward && p > start {
                    self.message = Some(Msg::WrappedToBottom.into());
                }
                return true;
            }
        }
        false
    }
}

/// 同じ値の知らせの値(空は `(空)`)。
pub(crate) fn shown_value(v: &str) -> String {
    if v.is_empty() {
        Msg::EmptyHeading.into()
    } else {
        v.to_string()
    }
}
