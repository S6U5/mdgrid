//! ためる変更(タスク 7。WB-4・WB-9・WB-10・WB-14・WB-16・CE-9・CE-10・SR-18)。
//! 核なので `Source` だけを通す(SC-14)。形は docs/design.md。

use crate::i18n::Msg;
use crate::source::{Edit, EditError, NewValue, RowId, SaveError, Source, Stamp, Value};
use std::collections::{BTreeMap, BTreeSet};

/// 直せなかった行と理由(表示用の短い日本語)。
#[derive(Debug, Clone, PartialEq)]
pub struct Skip {
    pub row: RowId,
    pub reason: String,
}

/// 保存の前の差分(WB-9)。1つの書く単位(`Source::unit`)ごと。external は「外で変更」の印(WB-16)。
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// 書く単位(Markdown では行そのもの)。
    pub row: RowId,
    /// 単位の中の、ためた変更のある行。
    pub rows: Vec<RowId>,
    pub before: Vec<u8>,
    pub after: Vec<u8>,
    pub external: bool,
}

/// 書く単位ごとの保存の結果。
#[derive(Debug)]
pub enum Outcome {
    Saved,
    Failed(SaveError),
    /// 外で変わって止めた(WB-4・WB-16)。
    Changed,
}

/// 行の状態(基準と「外で変更」の印)。ためる変更が無い行は (None, false)。
#[derive(Debug, Clone, Copy, PartialEq)]
struct RowState {
    base: Option<Stamp>,
    external: bool,
}

const EMPTY: RowState = RowState {
    base: None,
    external: false,
};

/// 1手の中の1つのセルの前と後(None = ためていない)。
#[derive(Debug, Clone)]
struct CellChange {
    row: RowId,
    col: String,
    before: Option<NewValue>,
    after: Option<NewValue>,
}

/// 1手の中の1つの行の状態の前と後。
#[derive(Debug, Clone)]
struct RowChange {
    row: RowId,
    before: RowState,
    after: RowState,
}

/// 取り消し・やり直しの1手(WB-10)。set・set_many・discard を同じに扱う。
#[derive(Debug, Clone, Default)]
struct Step {
    cells: Vec<CellChange>,
    rows: Vec<RowChange>,
}

impl Step {
    fn is_empty(&self) -> bool {
        self.cells.is_empty() && self.rows.is_empty()
    }

    fn drop_row(&mut self, row: &RowId) {
        self.cells.retain(|c| &c.row != row);
        self.rows.retain(|r| &r.row != row);
    }
}

/// ためる変更: (行, 列) → 新しい値、行ごとの基準、取り消しとやり直しの積み、外で変更の印。
#[derive(Debug, Default)]
pub struct Changes {
    cells: BTreeMap<RowId, BTreeMap<String, NewValue>>,
    states: BTreeMap<RowId, RowState>,
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl Changes {
    pub fn new() -> Changes {
        Changes::default()
    }

    /// 1手としてためる。読むだけのセル(Cell.lock)は Err(Skip)。その行に初めてためるとき、src.stamp を基準に取る(WB-4)。
    /// NewValue::Null で、キーの無いセルなら何もしない(SR-18)。
    pub fn set(
        &mut self,
        src: &dyn Source,
        row: &RowId,
        col: &str,
        v: NewValue,
    ) -> Result<(), Skip> {
        let mut bases = BTreeMap::new();
        let change = self.plan(src, row, col, Some(&v), &mut bases)?;
        self.commit(change.into_iter().collect(), bases, true);
        Ok(())
    }

    /// 複数の行に同じ値を1手でためる。直せない行は飛ばして返す(CE-10)。
    pub fn set_many(
        &mut self,
        src: &dyn Source,
        rows: &[RowId],
        col: &str,
        v: NewValue,
    ) -> Vec<Skip> {
        let items: Vec<(RowId, Option<NewValue>)> =
            rows.iter().map(|r| (r.clone(), Some(v.clone()))).collect();
        self.set_each(src, col, &items)
    }

    /// 行ごとに違う値を1手で当てる(CE-4 の一括の切り替え・CE-10)。Some(v) はためる(set と同じ決まり)、
    /// None はその行の `col` のためた値を外す(元に戻す。WB-17)。直せない行は飛ばして返す。重ねて渡した行は最初だけ。
    pub fn set_each(
        &mut self,
        src: &dyn Source,
        col: &str,
        items: &[(RowId, Option<NewValue>)],
    ) -> Vec<Skip> {
        let mut bases = BTreeMap::new();
        let mut cells = Vec::new();
        let mut skips = Vec::new();
        let mut seen = BTreeSet::new();
        for (row, v) in items {
            if !seen.insert(row.clone()) {
                continue;
            }
            match self.plan(src, row, col, v.as_ref(), &mut bases) {
                Ok(Some(c)) => cells.push(c),
                Ok(None) => {}
                Err(s) => skips.push(s),
            }
        }
        self.commit(cells, bases, true);
        skips
    }

    pub fn undo(&mut self) -> bool {
        let Some(mut step) = self.undo.pop() else {
            return false;
        };
        self.apply(&mut step, false);
        self.redo.push(step);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(mut step) = self.redo.pop() else {
            return false;
        };
        self.apply(&mut step, true);
        self.undo.push(step);
        true
    }

    pub fn pending(&self, row: &RowId, col: &str) -> Option<&NewValue> {
        self.cells.get(row).and_then(|m| m.get(col))
    }

    /// 未保存の数 = ためた (行, 列) の数。
    pub fn count(&self) -> usize {
        self.cells.values().map(|m| m.len()).sum()
    }

    pub fn rows(&self) -> Vec<RowId> {
        self.cells.keys().cloned().collect()
    }

    /// 外で変わった行を受け取り、ためた変更のある行に「外で変更」の印を付ける(WB-16)。
    pub fn note_external(&mut self, rows: &[RowId]) {
        for row in rows {
            if let Some(st) = self.states.get_mut(row) {
                st.external = true;
            }
            // 今はためていないが、取り消し・やり直しで戻りうる行の記録にも印を付ける(印は付いている側に倒す)。
            for step in self.undo.iter_mut().chain(self.redo.iter_mut()) {
                for r in step.rows.iter_mut().filter(|r| &r.row == row) {
                    for st in [&mut r.before, &mut r.after] {
                        if st.base.is_some() {
                            st.external = true;
                        }
                    }
                }
            }
        }
    }

    /// 読み直した行のためたセルを、今の読んだ値と比べ、同じセルを1手として外す(WB-17)。
    /// 行のためる変更が無くなれば、その行の基準と印も外れる(undo で戻る)。外すものが無ければ手を積まない。
    /// `note_external` は読んだ値を見られないので、外の変化を受けたら続けてこれを呼ぶ。
    /// 外の変化で積む手なので、やり直しの積みは消さない(利用者の操作ではない)。
    pub fn drop_same(&mut self, src: &dyn Source, rows: &[RowId]) {
        let mut seen = BTreeSet::new();
        let mut cells = Vec::new();
        for row in rows {
            if !seen.insert(row.clone()) {
                continue;
            }
            for col in self.same_cols(src, row) {
                let before = self.pending(row, &col).cloned();
                cells.push(CellChange {
                    row: row.clone(),
                    col,
                    before,
                    after: None,
                });
            }
        }
        self.commit(cells, BTreeMap::new(), false);
    }

    pub fn external(&self, row: &RowId) -> bool {
        self.states.get(row).is_some_and(|s| s.external)
    }

    /// ためた変更のある行を、書く単位(`Source::unit`)ごとにまとめる(単位は最初に現れた順、行は昇順)。
    pub fn units(&self, src: &dyn Source) -> Vec<(RowId, Vec<RowId>)> {
        let mut out: Vec<(RowId, Vec<RowId>)> = Vec::new();
        for row in self.cells.keys() {
            let u = src.unit(row);
            match out.iter_mut().find(|(k, _)| *k == u) {
                Some((_, rows)) => rows.push(row.clone()),
                None => out.push((u, vec![row.clone()])),
            }
        }
        out
    }

    /// その書く単位の、ためた変更のある行。
    pub fn rows_in(&self, src: &dyn Source, unit: &RowId) -> Vec<RowId> {
        self.units(src)
            .into_iter()
            .find(|(u, _)| u == unit)
            .map(|(_, rows)| rows)
            .unwrap_or_default()
    }

    /// 書く単位のどれかの行に「外で変更」の印があるか(WB-16)。
    pub fn external_unit(&self, src: &dyn Source, unit: &RowId) -> bool {
        self.rows_in(src, unit).iter().any(|r| self.external(r))
    }

    /// 単位の中の行ごとの、書く直し(WB-17: 読んだ値と同じセルは除く。全部同じ行は除く)。
    fn unit_edits(&self, src: &dyn Source, rows: &[RowId]) -> Vec<(RowId, Vec<Edit>)> {
        rows.iter()
            .map(|r| (r.clone(), self.edits(src, r)))
            .filter(|(_, e)| !e.is_empty())
            .collect()
    }

    pub fn previews(&self, src: &dyn Source) -> Vec<Result<Preview, (RowId, EditError)>> {
        self.units(src)
            .into_iter()
            .filter_map(|(unit, rows)| {
                // WB-17: 読んだ値と同じセルは差分に出さない(全部同じ単位は書かないので出さない)。
                let edits = self.unit_edits(src, &rows);
                if edits.is_empty() {
                    return None;
                }
                let external = rows.iter().any(|r| self.external(r));
                Some(match src.preview_unit(&edits) {
                    Ok((before, after)) => Ok(Preview {
                        row: unit,
                        rows: edits.into_iter().map(|(r, _)| r).collect(),
                        before,
                        after,
                        external,
                    }),
                    Err(e) => Err((unit, e)),
                })
            })
            .collect()
    }

    /// 書く単位ごとに保存する。Saved の単位の変更は消え、基準は新しくなる。それ以外は残す(WB-14)。
    pub fn save(&mut self, src: &mut dyn Source) -> Vec<(RowId, Outcome)> {
        let rows = self.rows();
        self.save_rows(src, &rows)
    }

    /// 指定の行の書く単位だけを保存する(保存の確認の画面で1ファイルを選んで書くとき)。単位の中のためた変更は全部書く。
    /// 結果は単位ごと(形は save と同じ)。ためた変更の無い行と、重ねて渡した単位は飛ばす。
    pub fn save_rows(&mut self, src: &mut dyn Source, rows: &[RowId]) -> Vec<(RowId, Outcome)> {
        let units = self.units(src);
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for row in rows {
            if !self.cells.contains_key(row) && !units.iter().any(|(u, _)| u == row) {
                continue;
            }
            let unit = src.unit(row);
            if !seen.insert(unit.clone()) {
                continue;
            }
            let Some((_, unit_rows)) = units.iter().find(|(u, _)| *u == unit) else {
                continue;
            };
            // 基準(WB-4)は単位の中で1つ。違う基準の行が混ざれば、書かずに止める(基準の無いためる変更は作らないが、念のため)。
            let bases: BTreeSet<[u8; 32]> = unit_rows
                .iter()
                .filter_map(|r| self.states.get(r).and_then(|s| s.base))
                .map(|b| b.hash)
                .collect();
            let base = unit_rows
                .iter()
                .find_map(|r| self.states.get(r).and_then(|s| s.base));
            let (Some(base), 1) = (base, bases.len()) else {
                self.mark_external(unit_rows);
                out.push((unit, Outcome::Changed));
                continue;
            };
            // WB-17 の守り: 読んだ値と同じセルは書かない(クオートや書式だけが変わらないように)。
            // 全部が同じ行は書かずに、ためる変更から外す(外の変化と同じ扱いの手。結果には出さない)。
            let edits = self.unit_edits(src, unit_rows);
            let same: Vec<RowId> = unit_rows
                .iter()
                .filter(|r| !edits.iter().any(|(e, _)| e == *r))
                .cloned()
                .collect();
            self.drop_same(src, &same);
            if edits.is_empty() {
                continue;
            }
            match src.save_unit(&base, &edits) {
                Ok(_) => {
                    // 書いた行の変更は消え、積みからも除く(書いたものは取り消せない)。
                    for (r, _) in &edits {
                        self.cells.remove(r);
                        self.states.remove(r);
                        for s in self.undo.iter_mut().chain(self.redo.iter_mut()) {
                            s.drop_row(r);
                        }
                    }
                    self.undo.retain(|s| !s.is_empty());
                    self.redo.retain(|s| !s.is_empty());
                    out.push((unit, Outcome::Saved));
                }
                Err(SaveError::Changed) => {
                    self.mark_external(unit_rows);
                    out.push((unit, Outcome::Changed));
                }
                Err(e) => out.push((unit, Outcome::Failed(e))),
            }
        }
        out
    }

    /// 読み込み口が自分で書いた(ためた値に触れない書き込み。SC-17 の行を足す)あと、`old` を基準にしていた行の基準を
    /// `new` に進める。外で変わった印のある行は進めない(WB-16)。
    pub fn rebase(&mut self, old: &Stamp, new: &Stamp) {
        for st in self.states.values_mut() {
            if !st.external && st.base.is_some_and(|b| b.hash == old.hash) {
                st.base = Some(*new);
            }
        }
    }

    fn mark_external(&mut self, rows: &[RowId]) {
        for r in rows {
            if let Some(st) = self.states.get_mut(r) {
                st.external = true;
            }
        }
    }

    /// WB-16 の「外の変更の上に書く」: 今のファイルを読み直して基準にし、印を消す。
    pub fn overwrite(&mut self, src: &mut dyn Source, row: &RowId) -> std::io::Result<()> {
        self.overwrite_if(src, row, None).map(|_| ())
    }

    /// 画面からの「外の変更の上に書く」: 読み直した内容のハッシュが `seen`(利用者に見せた差分の元の内容)と同じときだけ
    /// 基準を進めて印を消し、Ok(true)。違えば基準も印も変えず Ok(false)(見せていない内容の上に書かない。WB-9・WB-16)。
    pub fn overwrite_seen(
        &mut self,
        src: &mut dyn Source,
        row: &RowId,
        seen: [u8; 32],
    ) -> std::io::Result<bool> {
        self.overwrite_if(src, row, Some(seen))
    }

    /// 読み直し、(seen があれば一致するときだけ)今の内容を基準にして印を消す。基準を進めたら true。
    fn overwrite_if(
        &mut self,
        src: &mut dyn Source,
        row: &RowId,
        seen: Option<[u8; 32]>,
    ) -> std::io::Result<bool> {
        let had = self.cells.contains_key(row);
        src.reload(row)?;
        // WB-17: 読み直した値と同じになったセルは書かない。
        self.drop_same(src, std::slice::from_ref(row));
        if !had {
            return Ok(false);
        }
        let stamp = src.stamp(row).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, Msg::NoBaseAfterReload.text())
        })?;
        if seen.is_some_and(|h| h != stamp.hash) {
            return Ok(false);
        }
        if !self.cells.contains_key(row) {
            // 全部が読み直した値と同じで外れた。書くものは無く、基準と印も外れている。
            return Ok(true);
        }
        self.states.insert(
            row.clone(),
            RowState {
                base: Some(stamp),
                external: false,
            },
        );
        Ok(true)
    }

    /// WB-16 の「ためた変更を捨てる」。その行の変更を1手として消す。
    pub fn discard(&mut self, row: &RowId) {
        self.discard_rows(std::slice::from_ref(row));
    }

    /// 複数の行(1つの書く単位など)の変更を1手として消す。
    pub fn discard_rows(&mut self, rows: &[RowId]) {
        let cells = rows
            .iter()
            .filter_map(|row| self.cells.get(row).map(|m| (row, m)))
            .flat_map(|(row, m)| {
                m.iter().map(move |(col, v)| CellChange {
                    row: row.clone(),
                    col: col.clone(),
                    before: Some(v.clone()),
                    after: None,
                })
            })
            .collect();
        self.commit(cells, BTreeMap::new(), true);
    }

    // ---- 内側 ----

    /// その行のためたセルのうち、今の読んだ値と同じ列(WB-17)。
    fn same_cols(&self, src: &dyn Source, row: &RowId) -> Vec<String> {
        self.cells
            .get(row)
            .map(|m| {
                m.iter()
                    .filter(|(col, v)| same_as_read(v, &src.get(row, col).value))
                    .map(|(col, _)| col.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 1つのセルの変更を決める。何もしないなら Ok(None)。基準が要る行は bases に取る。
    /// v が None なら、ためた値を外す(読むだけかを見ない。読むだけのセルにためた値は無い)。
    fn plan(
        &self,
        src: &dyn Source,
        row: &RowId,
        col: &str,
        v: Option<&NewValue>,
        bases: &mut BTreeMap<RowId, Stamp>,
    ) -> Result<Option<CellChange>, Skip> {
        let before = self.pending(row, col).cloned();
        let after = match v {
            None => None,
            Some(v) => {
                let cell = src.get(row, col);
                if let Some(reason) = cell.lock {
                    return Err(Skip {
                        row: row.clone(),
                        reason,
                    });
                }
                // CE-16・CE-18: リストのセル(値がリストか、列の型がリスト)に、リストでない値(空にする Null は除く)は当てない。
                // 当てると `tags: foo` のように書き方が変わる(書き戻しでも NotEditable)。
                let list_cell = matches!(cell.value, Some(Value::List(_)))
                    || src.kind(col).kind == crate::types::Kind::List;
                // CE-29: キーの名前の変更・削除は値の書き方を変えないので、リストのセルにも当てる。
                if list_cell
                    && !matches!(
                        v,
                        NewValue::List(_)
                            | NewValue::Null
                            | NewValue::RenameKey(_)
                            | NewValue::DeleteKey
                    )
                {
                    return Err(Skip {
                        row: row.clone(),
                        reason: Msg::SkipListCell.text().to_string(),
                    });
                }
                // SR-18: キーの無いセルを空にしてもキーを足さない(ためた値があれば、それを消すだけ)。
                // WB-17: 最後に読んだ値と同じ値は、ためない(ためていればそれを外す)。
                // WB-18: 型の決まらない列の日付・日時の形だけの文字は、日付として(囲まずに)書く。
                let v = crate::source::date_if_untyped(src, col, v.clone());
                if same_as_read(&v, &cell.value) {
                    None
                } else {
                    Some(v)
                }
            }
        };
        if before == after {
            return Ok(None);
        }
        if after.is_some() && !self.cells.contains_key(row) && !bases.contains_key(row) {
            let stamp = src.stamp(row).ok_or_else(|| Skip {
                row: row.clone(),
                reason: Msg::SkipNotLoaded.text().to_string(),
            })?;
            bases.insert(row.clone(), stamp);
        }
        Ok(Some(CellChange {
            row: row.clone(),
            col: col.to_string(),
            before,
            after,
        }))
    }

    /// セルの変更を当て、行の状態を合わせ、1手として積む。`clear_redo` が偽なのは外の変化で積む手(drop_same)。
    fn commit(&mut self, cells: Vec<CellChange>, bases: BTreeMap<RowId, Stamp>, clear_redo: bool) {
        if cells.is_empty() {
            return;
        }
        let touched: BTreeSet<RowId> = cells.iter().map(|c| c.row.clone()).collect();
        let befores: Vec<(RowId, RowState)> = touched
            .iter()
            .map(|r| (r.clone(), self.states.get(r).copied().unwrap_or(EMPTY)))
            .collect();
        for c in &cells {
            self.put(&c.row, &c.col, c.after.clone());
        }
        let mut rows = Vec::new();
        for (row, before) in befores {
            let after = if self.cells.contains_key(&row) {
                RowState {
                    base: before.base.or_else(|| bases.get(&row).copied()),
                    external: before.external,
                }
            } else {
                EMPTY
            };
            self.put_state(&row, after);
            rows.push(RowChange { row, before, after });
        }
        self.undo.push(Step { cells, rows });
        if clear_redo {
            self.redo.clear();
        }
    }

    /// 1手を当てる(forward = やり直し)。行の状態を手の記録から戻すのは「ためた変更なし → あり」の行だけ。
    /// 前後どちらでもためた変更がある行は、今の状態(note_external・save の印、overwrite の基準)を保つ。
    /// 「あり → なし」の行は、今の状態を手の記録に書き戻してから空にする(戻したときに新しいほうが使われる)。
    fn apply(&mut self, step: &mut Step, forward: bool) {
        let had: Vec<bool> = step
            .rows
            .iter()
            .map(|r| self.cells.contains_key(&r.row))
            .collect();
        for c in &step.cells {
            let v = if forward { &c.after } else { &c.before };
            self.put(&c.row, &c.col, v.clone());
        }
        for (r, had) in step.rows.iter_mut().zip(had) {
            let has = self.cells.contains_key(&r.row);
            let current = self.states.get(&r.row).copied().unwrap_or(EMPTY);
            let (target, other) = if forward {
                (&mut r.after, &mut r.before)
            } else {
                (&mut r.before, &mut r.after)
            };
            match (had, has) {
                (true, true) => {}
                (true, false) => {
                    *other = current;
                    self.put_state(&r.row, EMPTY);
                }
                (false, true) => {
                    let st = *target;
                    self.put_state(&r.row, st);
                }
                (false, false) => self.put_state(&r.row, EMPTY),
            }
        }
    }

    fn put(&mut self, row: &RowId, col: &str, v: Option<NewValue>) {
        match v {
            Some(v) => {
                self.cells
                    .entry(row.clone())
                    .or_default()
                    .insert(col.to_string(), v);
            }
            None => {
                if let Some(m) = self.cells.get_mut(row) {
                    m.remove(col);
                    if m.is_empty() {
                        self.cells.remove(row);
                    }
                }
            }
        }
    }

    fn put_state(&mut self, row: &RowId, st: RowState) {
        if st == EMPTY {
            self.states.remove(row);
        } else {
            self.states.insert(row.clone(), st);
        }
    }

    /// 書く変更。読んだ値と同じセルは除く(WB-17。undo で戻ったセルなどを書いて書式だけが変わらないように)。
    fn edits(&self, src: &dyn Source, row: &RowId) -> Vec<Edit> {
        self.cells
            .get(row)
            .map(|m| {
                m.iter()
                    .filter(|(k, v)| !same_as_read(v, &src.get(row, k).value))
                    .map(|(k, v)| Edit {
                        key: k.clone(),
                        value: v.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// ためる値と読んだ値が同じか(WB-17)。型を見て比べる: Null と Null、Str・Date と Str は完全一致、
/// Int・Float は数として同じ、Bool と Bool。それ以外(数と文字、Null と空の文字列など)は違う。
/// List は要素の文字列の並びで比べ、空の並びは Null と同じ(書くと `key:` になる。CE-19)。
fn same_value(new: &NewValue, old: &Value) -> bool {
    match (new, old) {
        (NewValue::Null, Value::Null) => true,
        (NewValue::List(a), Value::Null) => a.is_empty(),
        // `key: ""` に空の並び: 外す要素が無いので書かない(書くと `key:` に変わる)。
        (NewValue::List(a), Value::Str(s)) if s.is_empty() => a.is_empty(),
        (NewValue::List(a), Value::List(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| matches!(y, Value::Str(y) if x == y))
        }
        // WB-18: 日付は読んだ文字列と比べる(素の日付も Str で読まれる)。
        (NewValue::Str(a) | NewValue::Date(a), Value::Str(b)) => a == b,
        (NewValue::Bool(a), Value::Bool(b)) => a == b,
        (NewValue::Int(a), Value::Int(b)) => a == b,
        (NewValue::Float(a), Value::Float(b)) => a == b,
        (NewValue::Int(a), Value::Float(b)) | (NewValue::Float(b), Value::Int(a)) => {
            int_eq_float(*a, *b)
        }
        _ => false,
    }
}

/// 整数と小数が数として同じか。f64 に丸めて比べると 2^53 を超える整数で誤るので、
/// 小数が有限・整数値・i64 の範囲の中のときだけ i64 にして比べる。
fn int_eq_float(a: i64, b: f64) -> bool {
    // i64::MIN は -2^63 でちょうど表せる。上は 2^63 未満(2^63 は i64 に入らない)。
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    b.is_finite() && b.fract() == 0.0 && (-LIMIT..LIMIT).contains(&b) && b as i64 == a
}

/// ためる値が、読んだセル(None = キーが無い)と同じか。キーの無いセルに Null は同じとみる(SR-18)。
fn same_as_read(v: &NewValue, read: &Option<Value>) -> bool {
    match read {
        // 空の並びも、キーの無いセルには何も書かない。
        None => matches!(v, NewValue::Null) || matches!(v, NewValue::List(a) if a.is_empty()),
        Some(old) => same_value(v, old),
    }
}

#[cfg(test)]
#[path = "test_changes_unit.rs"]
mod tests;
