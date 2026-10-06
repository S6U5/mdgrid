//! 列の操作(`impl App` の続き): 一時的な並べ替え(NV-3)、列を隠す・戻す・並びを変える・左に固定する(NV-4)。
//! どれも `.base` は変えず、画面の中だけで効く。並べ替えを表に重ねるのは grid.rs の `overlay`。
//! 列の並び・隠す列は見た目の状態として残す(SR-11・SR-12。startup.rs)。並べ替えと固定は残さない。

use super::app::App;
use super::keymap::Action;
use mdgrid::base::Shown;
use mdgrid::expr::Val;
use mdgrid::i18n::Msg;
use mdgrid::source::{RowId, Value};
use std::cmp::Ordering;

/// 並べ替えの鍵(NV-3)。数(日付・期間も数)→ 真偽 → 文字 の順で、空は向きによらず末尾。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortKey {
    Num(f64),
    Bool(bool),
    Str(String),
    Empty,
}

fn rank(k: &SortKey) -> u8 {
    match k {
        SortKey::Num(_) => 0,
        SortKey::Bool(_) => 1,
        SortKey::Str(_) => 2,
        SortKey::Empty => 3,
    }
}

/// 鍵を比べる。降順は空でない値の間だけ逆にする(空は末尾のまま)。
pub(crate) fn sort_cmp(a: &SortKey, b: &SortKey, desc: bool) -> Ordering {
    let o = match (a, b) {
        (SortKey::Empty, SortKey::Empty) => return Ordering::Equal,
        (SortKey::Empty, _) => return Ordering::Greater,
        (_, SortKey::Empty) => return Ordering::Less,
        (SortKey::Num(x), SortKey::Num(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (SortKey::Bool(x), SortKey::Bool(y)) => x.cmp(y),
        (SortKey::Str(x), SortKey::Str(y)) => x.cmp(y),
        _ => rank(a).cmp(&rank(b)),
    };
    if desc {
        o.reverse()
    } else {
        o
    }
}

fn value_key(v: &Value) -> SortKey {
    match v {
        Value::Null | Value::Other => SortKey::Empty,
        Value::Str(s) if s.is_empty() => SortKey::Empty,
        Value::Str(s) => SortKey::Str(s.clone()),
        Value::Bool(b) => SortKey::Bool(*b),
        Value::Int(i) => SortKey::Num(*i as f64),
        Value::Float(f) => SortKey::Num(*f),
        Value::List(_) => SortKey::Str(super::external::plain(v)),
    }
}

fn val_key(v: &Val) -> SortKey {
    match v {
        Val::Null => SortKey::Empty,
        Val::Bool(b) => SortKey::Bool(*b),
        Val::Num(n) => SortKey::Num(*n),
        Val::Date(d) => SortKey::Num(*d as f64),
        Val::DateTime(t) => SortKey::Num(*t as f64),
        Val::Duration(ms) => SortKey::Num(*ms as f64),
        Val::Str(s) if s.is_empty() => SortKey::Empty,
        _ => SortKey::Str(super::cell::val_text(v)),
    }
}

impl App {
    /// 行の並べ替えの鍵(ためた値を重ねる。NV-12)。
    pub(crate) fn sort_key(&self, row: &RowId, col: &str) -> SortKey {
        if let Some(nv) = self.changes.pending(row, col) {
            return value_key(&super::grid::value_of(nv));
        }
        match self.cell(row, col) {
            Shown::Prop(c) => c.value.as_ref().map(value_key).unwrap_or(SortKey::Empty),
            Shown::Computed(v) => val_key(&v),
            Shown::Unsupported(_) => SortKey::Empty,
        }
    }

    /// 列の見出しの文字。一時的に並べている列には向きの印(`↑` 昇順・`↓` 降順。NV-3)。
    pub(crate) fn head_title(&self, col: &str) -> String {
        let t = self.title(col);
        match &self.sort {
            Some((c, desc)) if c == col => format!("{t}{}", if *desc { "↓" } else { "↑" }),
            _ => t,
        }
    }

    /// 列の操作の動作。扱ったら true。
    pub(crate) fn column_action(&mut self, action: Action) -> bool {
        match action {
            Action::Sort => self.cycle_sort(),
            Action::HideColumn => self.hide_column(),
            Action::ShowColumn => self.show_column(),
            Action::AddColumn => self.start_add_column(),
            Action::MoveColumnLeft => self.move_column(false),
            Action::MoveColumnRight => self.move_column(true),
            Action::Freeze => self.freeze(),
            _ => return false,
        }
        true
    }

    /// 選んだ列で、昇順 → 降順 → 解除(`.base` の並び)を回す(NV-3)。別の列なら昇順から。
    pub(crate) fn cycle_sort(&mut self) {
        let Some(col) = self.cols.get(self.col).cloned() else {
            return;
        };
        let t = self.title(&col);
        self.sort = match self.sort.take() {
            Some((c, false)) if c == col => Some((c, true)),
            Some((c, true)) if c == col => None,
            _ => Some((col, false)),
        };
        self.message = Some(match &self.sort {
            Some((_, false)) => Msg::SortedAsc.fill(&[&t]),
            Some((_, true)) => Msg::SortedDesc.fill(&[&t]),
            None => Msg::SortCleared.text().into(),
        });
        self.relayout();
    }

    /// 選んだ列を隠す(NV-4)。最後の1列は隠さない。
    fn hide_column(&mut self) {
        if self.cols.len() <= 1 {
            self.message = Some(Msg::LastColumnHide.text().into());
            return;
        }
        let j = self.col;
        let col = self.cols.remove(j);
        self.kinds.remove(j);
        let was_frozen = j < self.frozen;
        if was_frozen {
            self.frozen -= 1;
        }
        self.message = Some(Msg::ColumnHidden.fill(&[&self.title(&col)]));
        self.hidden.push((col, j, was_frozen));
        self.col = j.min(self.cols.len() - 1);
        self.regrid = true;
    }

    /// 最後に隠した列を、隠したときの位置に戻す(NV-4)。
    fn show_column(&mut self) {
        let Some((col, j, was_frozen)) = self.hidden.pop() else {
            self.message = Some(Msg::NoHiddenColumns.text().into());
            return;
        };
        let at = j.min(self.cols.len());
        // 固定した列の間(か、固定していた列の位置)に戻すなら、固定の数も戻す。
        if at < self.frozen || (at == self.frozen && was_frozen) {
            self.frozen += 1;
        }
        self.message = Some(Msg::ColumnShown.fill(&[&self.title(&col)]));
        self.cols.insert(at, col);
        self.col = at;
        self.regrid = true;
    }

    /// 選んだ列を左・右の列と入れ替える(NV-4)。
    fn move_column(&mut self, right: bool) {
        let j = self.col;
        let k = if right {
            j + 1
        } else {
            match j.checked_sub(1) {
                Some(k) => k,
                None => return,
            }
        };
        if k >= self.cols.len() {
            return;
        }
        self.cols.swap(j, k);
        self.kinds.swap(j, k);
        self.col = k;
    }

    /// 選んだ列までを左に固定する(NV-4。ノートの列はいつも固定)。同じ列でもう一度押すと解く。
    fn freeze(&mut self) {
        if self.cols.is_empty() {
            return;
        }
        if self.frozen == self.col + 1 {
            self.frozen = 0;
            self.message = Some(Msg::FreezeCleared.text().into());
        } else if !super::view::freeze_fits(self, self.col) {
            // 固定した列だけで画面を超えると、ほかの列を選べなくなる(SR-3)。
            self.message = Some(Msg::FreezeTooWide.fill(&[&(self.col + 1)]));
        } else {
            self.frozen = self.col + 1;
            self.message = Some(Msg::Frozen.fill(&[&self.frozen]));
        }
        self.left = self.frozen.min(self.cols.len().saturating_sub(1));
    }
}
