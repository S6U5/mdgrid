//! 表の行と列の組み立て(`impl App` の続き)。`.base` があれば `Base::build`、無ければ `default_grid`(BV-1・BV-4・BV-5)。
//! その上にビューの設定(NV-20。`settings::apply`)を重ね、
//! ためた値を重ねて計算し直し、簡易の絞り込み(NV-2)・同じ値の絞り込み(NV-8)・一時的な並べ替え(NV-3)を
//! 重ねる(`overlay`)。値を直した行は、保存か移動の操作まで元の位置に留める(NV-12)。
//! 列の順は表の側が持ち、新しい列は右に足す。ビューの切り替えの後の組み直し(BV-13。タブと切り替えは
//! native_views.rs)、グループの見出しの開閉(SR-2)、
//! 列の幅の変更(SR-3・NV-4 の `<` `>` とドラッグ)もここ。列を隠す・戻す・動かす・固定する(NV-4)は columns.rs。
//! 核には Source・Changes・Base・default_grid・expr::Val だけで触れる(SC-14)。

use super::app::App;
use mdgrid::base::{self, Base, Grid, Shown};
use mdgrid::expr::{to_val, Val};
use mdgrid::i18n::Msg;
use mdgrid::print::{self, val_value};
use mdgrid::settings::{self, Group};
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::summary::{self, Summary};
use mdgrid::types::Kind;
use std::collections::{HashMap, HashSet};

/// 開いた `.base`(BV-13)。
pub(crate) struct BaseFile {
    pub base: Base,
    /// ヘッダーに出す名前(`.base` のファイル名。SR-1)。
    pub name: String,
    /// 選んだビューの添字。
    pub view: usize,
}

/// 表の1行: グループの見出し(`App.groups` の添字)か、ノートの行(`App.rows` の添字)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    Head(usize),
    Row(usize),
}

/// 組み立て直しの前後で同じ行を探す鍵(BV-10)。
#[derive(PartialEq, Eq)]
enum SlotKey {
    Head(String),
    Row(RowId),
}

/// 列のドラッグ(SR-3): 動かしている列と、その列の左端の桁。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Drag {
    pub col: usize,
    pub x0: usize,
}

/// ためた値をフロントマターの値の形にする(NV-12)。
pub(crate) fn value_of(nv: &NewValue) -> Value {
    match nv {
        NewValue::Null => Value::Null,
        NewValue::Str(s) | NewValue::Date(s) => Value::Str(s.clone()),
        NewValue::Bool(b) => Value::Bool(*b),
        NewValue::Int(i) => Value::Int(*i),
        NewValue::Float(f) => Value::Float(*f),
        NewValue::List(items) => Value::List(items.iter().map(|s| Value::Str(s.clone())).collect()),
        // CE-29: 名前を変える・消すキーは、保存まで印の文字として並べ・絞り込む。
        NewValue::RenameKey(to) => Value::Str(format!("→ {to}")),
        NewValue::DeleteKey => Value::Null,
    }
}

impl App {
    /// ノートのキーの値に、ためた値を重ねたもの(NV-12)。
    pub(crate) fn prop(&self, row: &RowId, key: &str) -> Option<Value> {
        match self.changes.pending(row, key) {
            Some(nv) => Some(value_of(nv)),
            None => self.src.get(row, key).value,
        }
    }

    /// ビューの設定(NV-14・NV-19)で比べる値: ノートのキーはためた値を重ねた値、計算の列(`file.*`・
    /// `formula.*`)は画面の値(`Shown::Computed`)を `Value` に写したもの。評価できない式は値なし。
    pub(crate) fn setting_value(&self, row: &RowId, col: &str) -> Option<Value> {
        match self.cell(row, col) {
            Shown::Computed(v) => Some(val_value(&v)),
            Shown::Unsupported(_) => None,
            Shown::Prop(_) => self.prop(row, col),
        }
    }

    /// 重ね順の2段目(NV-20): `.base` の結果に、ビューの設定を重ねる(絞る・並べる・まとめる)。
    fn apply_settings(
        &self,
        rows: Vec<RowId>,
        groups: Vec<(String, std::ops::Range<usize>)>,
    ) -> (Vec<RowId>, Vec<(String, std::ops::Range<usize>)>) {
        if self.settings.no_conditions() {
            return (rows, groups);
        }
        let get = |r: &RowId, c: &str| self.setting_value(r, c);
        let kind = |c: &str| self.column_kind(c);
        settings::apply(rows, groups, &self.settings, &get, &kind)
    }

    pub(crate) fn build_grid(&self) -> Result<Grid, String> {
        // CE-28: 足した列は、まだ無ければ右に足す(ノートに値が入れば、どの組み立てでも普通の列になる)。
        self.build_view_grid().map(|mut g| {
            for c in &self.extra_cols {
                if !g.columns.iter().any(|x| &x.id == c) {
                    g.columns.push(base::Column {
                        id: c.clone(),
                        title: c.clone(),
                    });
                }
            }
            g
        })
    }

    fn build_view_grid(&self) -> Result<Grid, String> {
        // BV-17: mdgrid のビューを選んでいれば、その定義で組む(native_views.rs)。
        if let Some(g) = self.native_grid() {
            return g;
        }
        let Some(b) = &self.base else {
            return Ok(base::default_grid(self.src.as_ref()));
        };
        if b.base.views.is_empty() {
            return Err(Msg::PrintNoViews.text().into());
        }
        let prop = |r: &RowId, k: &str| self.prop(r, k);
        b.base
            .build(b.view, self.src.as_ref(), &prop, self.today, self.now)
    }

    /// セル1つの見せ方の元(BV-7)。`.base` なしならノートのキーのセル。
    pub(crate) fn cell(&self, row: &RowId, col: &str) -> Shown {
        if let Some(b) = self.native_synth() {
            let prop = |r: &RowId, k: &str| self.prop(r, k);
            return b.cell(self.src.as_ref(), &prop, row, col, self.today, self.now);
        }
        match &self.base {
            Some(b) => {
                let prop = |r: &RowId, k: &str| self.prop(r, k);
                b.base
                    .cell(self.src.as_ref(), &prop, row, col, self.today, self.now)
            }
            None => Shown::Prop(self.src.get(row, col)),
        }
    }

    /// 列の型(CV-4・CV-2・NV-19)。組み立て直しのたびに `.base` の結果の行の全部から一度だけ決めた型
    /// (`col_kinds`)を引く。表の寄せとビューの設定の絞り込み・並べ替え・グループは同じ型を使う。
    /// 決めていない列(ビューに無いノートのキー)はその場で決める。
    pub(crate) fn column_kind(&self, col: &str) -> Kind {
        match self.col_kinds.get(col) {
            Some(k) => *k,
            None => self.kind_in(col, &self.rows),
        }
    }

    /// 列の型を `rows` から決める: ノートのキーは `Source::kind`、`file.size` は数、formula の列は
    /// `rows` の順で最初の空でない値(数なら数、真偽ならチェック)、ほかの計算の列はテキスト。
    /// 決め方は核の `print::kind_in`(`--print` と同じ)。
    fn kind_in(&self, col: &str, rows: &[RowId]) -> Kind {
        print::kind_in(self.src.as_ref(), col, rows, &|r, c| self.cell(r, c))
    }

    /// 列の型を名前で引く。
    pub(crate) fn kind_of(&self, col: &str) -> Kind {
        self.cols
            .iter()
            .position(|c| c == col)
            .and_then(|j| self.kinds.get(j).copied())
            .unwrap_or(Kind::Text)
    }

    /// 列の見出し(displayName か id)。
    pub(crate) fn title(&self, col: &str) -> String {
        self.titles
            .get(col)
            .cloned()
            .unwrap_or_else(|| base::default_title(col))
    }

    fn slot_key(&self, i: usize) -> Option<SlotKey> {
        Some(match self.slots.get(i)? {
            Slot::Head(g) => SlotKey::Head(self.groups.get(*g)?.0.clone()),
            Slot::Row(r) => SlotKey::Row(self.rows.get(*r)?.clone()),
        })
    }

    fn build_slots(&mut self) {
        self.slots.clear();
        if self.groups.is_empty() {
            self.slots.extend((0..self.rows.len()).map(Slot::Row));
            return;
        }
        for (g, (h, range)) in self.groups.iter().enumerate() {
            self.slots.push(Slot::Head(g));
            if !self.folded.contains(h) {
                self.slots.extend(range.clone().map(Slot::Row));
            }
        }
    }

    /// 行と列を組み立て直す。選んだ行は鍵で保ち(BV-10)、列の順は保って新しい列を右に足す。
    /// 評価できない filters のビューは開かず、行も列も空にして理由を持つ(BV-7)。
    pub(crate) fn refresh(&mut self) {
        // NV-9: 頻度表は開いたときの行で数えたので、組み直したら閉じる(件数が今の行と合わなくなる)。
        self.close_freq_on_regrid();
        let old_key = self.slot_key(self.row);
        let old_row = self.row;
        let selected_col = self.cols.get(self.col).cloned();
        let grid = match self.build_grid() {
            Ok(g) => {
                self.view_error = None;
                g
            }
            Err(e) => {
                self.view_error = Some(e);
                Grid {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    groups: Vec::new(),
                    notes: Vec::new(),
                    marked: 0,
                    summaries: Vec::new(),
                }
            }
        };
        let ids: HashSet<&String> = grid.columns.iter().map(|c| &c.id).collect();
        self.cols.retain(|c| ids.contains(c));
        let mut have: HashSet<String> = self.cols.iter().cloned().collect();
        // 隠した列(NV-4)は足さない。
        have.extend(self.hidden.iter().map(|(c, _, _)| c.clone()));
        for c in &grid.columns {
            if !have.contains(&c.id) {
                // SR-12: 見た目の状態の並び・隠す列を当て、状態に無い列は右に足す。
                self.add_column(&c.id);
            }
        }
        self.keep_one_column();
        self.titles = grid
            .columns
            .iter()
            .map(|c| (c.id.clone(), c.title.clone()))
            .collect::<HashMap<_, _>>();
        if let Some(c) = selected_col {
            if let Some(i) = self.cols.iter().position(|x| *x == c) {
                self.col = i;
            }
        }
        self.col = self.col.min(self.cols.len().saturating_sub(1));
        // 列の型は、組み立て直しの前に `.base` の結果の行の全部から一度だけ決める(表の寄せとビューの設定で
        // 同じ型。前の表や見えている行によらない)。
        let mut kind_cols: Vec<&str> = grid.columns.iter().map(|c| c.id.as_str()).collect();
        kind_cols.extend(self.settings.filters.iter().map(|c| c.col.as_str()));
        kind_cols.extend(self.settings.sorts.iter().map(|(c, _)| c.as_str()));
        if let Group::By { col, .. } = &self.settings.group {
            kind_cols.push(col);
        }
        let col_kinds: HashMap<String, Kind> = kind_cols
            .into_iter()
            .map(|c| (c.to_string(), self.kind_in(c, &grid.rows)))
            .collect();
        self.col_kinds = col_kinds;
        // NV-20: `.base` → ビューの設定 → 簡易の絞り込み・同じ値 → 一時的な並べ替え → 直した行の留め。
        let (rows, groups) = self.apply_settings(grid.rows, grid.groups);
        let (rows, groups) = self.overlay(rows, groups);
        self.rows = rows;
        self.groups = groups;
        self.label_prefix = common_folder(self.src.rows().iter().map(|r| self.src.label(r)));
        // BV-14・BV-7: 画面に印の無い未対応の理由(集計・並べ替え)は、開いたとき(理由が変わったとき)に
        // 下の行へ一度出す(セルを選んでいると案内に隠れるため)。列の理由はセルの `?` と案内で見える。
        // ほかの知らせが出ていれば上書きしない。
        let quiet = &grid.notes[grid.marked.min(grid.notes.len())..];
        if grid.notes != self.notes && !quiet.is_empty() && self.message.is_none() {
            self.message = Some(Msg::HintUnsupported.fill(&[&quiet.join(" / ")]));
        }
        self.notes = grid.notes;
        self.summaries = self.compute_summaries(grid.summaries);
        self.build_slots();
        let found = old_key
            .and_then(|k| (0..self.slots.len()).find(|&i| self.slot_key(i).as_ref() == Some(&k)));
        self.row = found.unwrap_or_else(|| old_row.min(self.slots.len().saturating_sub(1)));
        self.kinds = self.cols.iter().map(|c| self.column_kind(c)).collect();
        self.scroll_into_view();
        self.guard_input();
    }

    /// BV-14: 集計を今の行(ビューの設定・簡易の絞り込みのあと)で計算する。ノートのキーはためた値を重ねた値、
    /// 計算の列は画面の値、評価できない式は空。列ごとに行を1回だけ走査する。
    fn compute_summaries(&self, list: Vec<(String, Summary)>) -> Vec<(String, Summary, Val)> {
        list.into_iter()
            .map(|(col, s)| {
                let values = self.rows.iter().map(|r| match self.cell(r, &col) {
                    Shown::Computed(v) => v,
                    Shown::Unsupported(_) => Val::Null,
                    Shown::Prop(_) => self.prop(r, &col).map_or(Val::Null, |v| to_val(&v)),
                });
                let v = summary::compute(s, values);
                (col, s, v)
            })
            .collect()
    }

    /// 組み立てた行に、簡易の絞り込み・同じ値の絞り込み(NV-2・NV-8)と一時的な並べ替え(NV-3)を重ね、
    /// 値を直した行(`stay`)を元の位置(前の表のまとまりと、その中の位置)に留める(NV-12)。
    /// 留めた行のうち本来の位置と違う行を `held` に入れる(印を付ける)。
    fn overlay(
        &mut self,
        rows: Vec<RowId>,
        groups: Vec<(String, std::ops::Range<usize>)>,
    ) -> (Vec<RowId>, Vec<(String, std::ops::Range<usize>)>) {
        let grouped = !groups.is_empty();
        let mut segs: Vec<(String, Vec<RowId>)> = if grouped {
            groups
                .into_iter()
                .map(|(h, r)| (h, rows[r].to_vec()))
                .collect()
        } else {
            vec![(String::new(), rows)]
        };
        let narrowed = self.filter.is_some() || !self.same.is_empty();
        // NV-23: 検索の欄の「N/M行」の M(絞る前の行の数)。
        self.unfiltered = segs.iter().map(|(_, rs)| rs.len()).sum();
        for (_, rs) in &mut segs {
            if narrowed {
                rs.retain(|r| self.passes(r));
            }
            if let Some((col, desc)) = &self.sort {
                let mut keyed: Vec<(super::columns::SortKey, RowId)> =
                    rs.drain(..).map(|r| (self.sort_key(&r, col), r)).collect();
                keyed.sort_by(|a, b| super::columns::sort_cmp(&a.0, &b.0, *desc));
                rs.extend(keyed.into_iter().map(|(_, r)| r));
            }
        }
        self.held.clear();
        if !self.stay.is_empty() {
            self.keep_stay(&mut segs);
        }
        segs.retain(|(_, rs)| !rs.is_empty() || !grouped);
        let mut out_rows = Vec::new();
        let mut out_groups = Vec::new();
        for (h, rs) in segs {
            let start = out_rows.len();
            out_rows.extend(rs);
            if grouped {
                out_groups.push((h, start..out_rows.len()));
            }
        }
        (out_rows, out_groups)
    }

    /// NV-12: `stay` の行を、前の表(`self.rows`・`self.groups`)での位置に戻す。
    fn keep_stay(&mut self, segs: &mut Vec<(String, Vec<RowId>)>) {
        // 前の表での位置(まとまりの添字・見出し・中の位置)。
        let mut old: Vec<(usize, String, usize, RowId)> = Vec::new();
        let old_groups: Vec<(String, std::ops::Range<usize>)> = if self.groups.is_empty() {
            vec![(String::new(), 0..self.rows.len())]
        } else {
            self.groups.clone()
        };
        for (g, (h, range)) in old_groups.iter().enumerate() {
            for (k, i) in range.clone().enumerate() {
                if let Some(r) = self.rows.get(i).filter(|r| self.stay.contains(*r)) {
                    old.push((g, h.clone(), k, r.clone()));
                }
            }
        }
        if old.is_empty() {
            return;
        }
        // 本来の位置(組み立てたままの位置)を覚えてから抜く。消えたノートは留めない。
        let mut natural: HashMap<RowId, (String, usize)> = HashMap::new();
        let mut gone: Vec<&RowId> = Vec::new();
        for (_, _, _, r) in &old {
            let at = segs
                .iter()
                .find_map(|(sh, rs)| rs.iter().position(|x| x == r).map(|k| (sh.clone(), k)));
            match at {
                Some(p) => {
                    natural.insert(r.clone(), p);
                }
                None => gone.push(r),
            }
        }
        let alive: HashSet<RowId> = if gone.is_empty() {
            HashSet::new()
        } else {
            self.src.rows().into_iter().collect()
        };
        for (_, rs) in segs.iter_mut() {
            rs.retain(|r| !self.stay.contains(r));
        }
        for (g, h, k, r) in old {
            if !natural.contains_key(&r) && !alive.contains(&r) {
                continue;
            }
            let si = match segs.iter().position(|(sh, _)| *sh == h) {
                Some(i) => i,
                None => {
                    let at = g.min(segs.len());
                    segs.insert(at, (h.clone(), Vec::new()));
                    at
                }
            };
            let rs = &mut segs[si].1;
            let at = k.min(rs.len());
            rs.insert(at, r.clone());
            if natural.get(&r) != Some(&(h, at)) {
                self.held.insert(r);
            }
        }
    }

    /// 組み立て直しのあと、入力ボックスの行が選んだ行でなくなっていたら(絞り込みから外れた・畳んだ
    /// まとまりに入った・消えた)、入力を閉じて理由を出す。見えていない行に書かないため。
    pub(crate) fn guard_input(&mut self) {
        let Some(i) = &self.input else {
            return;
        };
        // 新しいノートの入力(CE-25)は書き先の行を持たない。
        if self.note.flow.is_some() || self.cur_row().as_ref() == Some(&i.row) {
            return;
        }
        // 詳細の表示から開いた入力は、詳細の表示の行に確定する(表の選択とは別)。
        if self.detail.as_ref().is_some_and(|d| d.row == i.row) {
            return;
        }
        let label = self.src.label(&i.row);
        self.input = None;
        self.close_input();
        self.message = Some(Msg::InputClosedRowGone.fill(&[&label]));
    }
    /// 選んだ行のノート。見出しの行なら None。
    pub(crate) fn cur_row(&self) -> Option<RowId> {
        match self.slots.get(self.row)? {
            Slot::Row(r) => self.rows.get(*r).cloned(),
            Slot::Head(_) => None,
        }
    }

    /// 選んだ行が見出しなら、そのグループの添字。
    pub(crate) fn cur_head(&self) -> Option<usize> {
        match self.slots.get(self.row)? {
            Slot::Head(g) => Some(*g),
            Slot::Row(_) => None,
        }
    }

    /// 見出しの行のグループを開閉する(SR-2)。見出しの行は選んだまま。
    pub(crate) fn toggle_group(&mut self, g: usize) {
        let Some((h, _)) = self.groups.get(g) else {
            return;
        };
        let h = h.clone();
        if !self.folded.remove(&h) {
            self.folded.insert(h);
        }
        self.build_slots();
        if let Some(i) = self.slots.iter().position(|s| *s == Slot::Head(g)) {
            self.row = i;
        }
        self.scroll_into_view();
    }

    /// groupBy の列の見出し(見出しの行に出す)。ビューの設定でまとめ直したら、その列(NV-15)。
    pub(crate) fn group_title(&self) -> Option<String> {
        match &self.settings.group {
            Group::Off => return None,
            Group::By { col, .. } => return Some(self.title(col)),
            Group::Inherit => {}
        }
        // mdgrid のビューは groupBy を持たない(まとめるのはビューの設定)。
        if self.nv.at.is_some() {
            return None;
        }
        let b = self.base.as_ref()?;
        let (id, _) = b.base.views.get(b.view)?.group_by.as_ref()?;
        Some(self.title(id))
    }

    /// `.base` を開く(CLI-1・CLI-2)。`view` は先頭(BV-13)か `--view` で選んだビュー。
    pub fn set_base(&mut self, base: Base, name: String, view: usize) {
        self.base = Some(BaseFile { base, name, view });
        if let Some(s) = &mut self.store {
            s.shown = false;
        }
        self.select_view(view);
    }

    /// 選んだビューで表を初めから組む(BV-13・SR-12)。列の順・幅・畳んだグループは選んだビューの
    /// 見た目の状態から(無ければ初めから)、選択と一時的な並べ替えは初めから。どのビューかは呼ぶ側
    /// (native_views.rs の `select_view`)が先に決める。
    pub(crate) fn reset_view(&mut self) {
        self.cols.clear();
        self.widths.clear();
        self.folded.clear();
        // 列と行の一時的な状態もビューごと(NV-3・NV-4・NV-5・NV-8・NV-12)。簡易の絞り込みの語は残す。
        self.hidden.clear();
        self.frozen = 0;
        self.sort = None;
        self.same.clear();
        self.same_mark = None;
        self.marked.clear();
        self.anchor = None;
        self.stay.clear();
        self.row = 0;
        self.col = 0;
        self.top = 0;
        self.left = 0;
        self.slots.clear();
        self.restore_state();
        self.refresh();
    }

    /// `<` `>`: 選んだ列の幅を1桁変える(SR-3・NV-4)。最小は1、最大は値の列に使える幅。
    pub(crate) fn resize_column(&mut self, delta: isize) {
        let Some(col) = self.cols.get(self.col).cloned() else {
            return;
        };
        let lay = super::view::layout(self);
        let now = lay.widths.get(self.col).copied().unwrap_or(1);
        let w = (now as isize + delta).max(1) as usize;
        self.set_width(&col, w);
    }

    /// 列の幅を決める(手で決めた幅。CV-5 の上限は当てない)。
    pub(crate) fn set_width(&mut self, col: &str, w: usize) {
        let max = super::view::max_column_width(self);
        self.widths.insert(col.to_string(), w.clamp(1, max.max(1)));
        self.scroll_into_view();
    }

    /// ドラッグの途中(SR-3): 境界がマウスの位置に来るよう幅を変える。
    pub fn drag(&mut self, x: u16) {
        let Some(d) = self.drag else {
            return;
        };
        let Some(col) = self.cols.get(d.col).cloned() else {
            return;
        };
        let w = (x as usize).saturating_sub(d.x0).max(1);
        self.set_width(&col, w);
    }

    /// ドラッグの終わり。
    pub fn release(&mut self) {
        self.drag = None;
    }
}

/// SR-29: パスの並びに共通のフォルダ(フォルダの区切りの単位で、末尾の `/` まで)。無ければ空。
pub(crate) fn common_folder(labels: impl Iterator<Item = String>) -> String {
    let mut common: Option<String> = None;
    for l in labels {
        let dir = l.rfind('/').map_or("", |i| &l[..=i]);
        let next = match &common {
            None => dir.to_string(),
            Some(c) => {
                let end = c
                    .match_indices('/')
                    .map(|(i, _)| i + 1)
                    .take_while(|&e| dir.as_bytes().get(..e) == Some(&c.as_bytes()[..e]))
                    .last()
                    .unwrap_or(0);
                c[..end].to_string()
            }
        };
        let done = next.is_empty();
        common = Some(next);
        if done {
            break;
        }
    }
    common.unwrap_or_default()
}
