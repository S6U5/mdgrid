//! 候補の出し方(cell-edit.candidates-sort・cell-edit.candidates-where・cell-edit.candidates-table-rows)。
//! 値の候補は、どの列も今の表のノートから集める(cell-edit.value-list-open・cell-edit.list-items-candidates)。決まり
//! `[rules.<列>.candidates]` を書いた列は `where` で絞って `sort` で並べ、書かない列は今までの並び(値の列は表の行に
//! 出てきた順で、くり返しの無い列には出さない。リストの列は多い順)。リンクの列は行き先のノートの名前の順。

use super::App;
use mdgrid::relations;
use mdgrid::rules::{CandSort, Cands};
use mdgrid::source::{RowId, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// 1件ごとに違う値の列と見なす、値のある行の数の下限(cell-edit.value-list-open)。
const UNIQUE_MIN: usize = 5;

/// 値の比べる文字(where の値と、候補の名前)。文字・数・真偽だけ。
fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.clone()),
        Value::Int(n) => Some(n.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// リンクの候補の名前の順の鍵(link_choices と同じ。ファイルの名前の小文字)。
fn name_key(p: &std::path::Path) -> String {
    relations::file_stem(p).to_lowercase()
}

/// 1つの候補の値の集計: 値・持つノートの数・一番新しい更新の日時(UNIX 秒)。
struct Stat {
    value: Value,
    key: String,
    count: usize,
    newest: i64,
}

impl App {
    /// 列の候補の出し方(書いていなければ None)。
    pub(crate) fn cand_spec(&self, col: &str) -> Option<&Cands> {
        self.prof.resolved.cands.get(col)
    }

    /// 行 `row` が候補の出し方の `where` に当たるか(列ごとに値のどれか。リストの列は要素のどれか。列どうしは全部)。
    fn cand_where_ok(&self, row: &RowId, spec: &Cands) -> bool {
        mdgrid::rules::where_matches(&spec.where_, |k| self.prop(row, k))
    }

    /// ノートのパスの行(読み込み口の行の書き方に合わせる。合わなければ実体のパスで探す)。
    fn row_of_note(&self, p: &std::path::Path) -> Option<RowId> {
        let direct = RowId(p.to_string_lossy().into_owned());
        if self.src.file(&direct).is_some() {
            return Some(direct);
        }
        let real = std::fs::canonicalize(p).ok()?;
        self.src
            .rows()
            .into_iter()
            .find(|r| std::fs::canonicalize(&r.0).ok().as_deref() == Some(real.as_path()))
    }

    /// cell-edit.candidates-sort・-where: リンクの候補のノート `notes` を、列の候補の出し方で絞って並べる。`keep` は
    /// 絞りに当たらなくても残すノート(今の値)。出し方を書いていない列はそのまま。
    pub(crate) fn arrange_links(
        &self,
        col: &str,
        notes: Vec<PathBuf>,
        keep: &[PathBuf],
    ) -> Vec<PathBuf> {
        let Some(spec) = self.cand_spec(col) else {
            return notes;
        };
        let mut notes: Vec<PathBuf> = notes
            .into_iter()
            .filter(|p| {
                spec.where_.is_empty()
                    || keep.contains(p)
                    || self
                        .row_of_note(p)
                        .is_some_and(|r| self.cand_where_ok(&r, spec))
            })
            .collect();
        notes.sort_by_key(|p| name_key(p));
        match spec.sort.unwrap_or(CandSort::Name) {
            CandSort::Name => {}
            CandSort::Count => {
                let counts = self.link_counts(col);
                let n = |p: &PathBuf| counts.get(p).copied().unwrap_or(0);
                notes.sort_by_key(|p| std::cmp::Reverse(n(p)));
            }
            CandSort::Recent => {
                let t = |p: &PathBuf| {
                    self.row_of_note(p)
                        .and_then(|r| self.src.file(&r))
                        .map_or(i64::MIN, |f| f.mtime)
                };
                notes.sort_by_key(|p| std::cmp::Reverse(t(p)));
            }
        }
        notes
    }

    /// 今の表のノートのうち、列 `col` で各ノートを指しているノートの数(1つのノートが同じノートを2回指しても1件)。
    fn link_counts(&self, col: &str) -> HashMap<PathBuf, usize> {
        let mut out: HashMap<PathBuf, usize> = HashMap::new();
        for r in self.table_rows() {
            let vals: Vec<String> = match self.prop(&r, col) {
                Some(Value::List(items)) => items.iter().filter_map(scalar_text).collect(),
                Some(v) => scalar_text(&v).into_iter().collect(),
                None => Vec::new(),
            };
            let mut seen = HashSet::new();
            for v in vals {
                if let Some((_, Some(t))) = self.link_target(&r, &v) {
                    if seen.insert(t.clone()) {
                        *out.entry(t).or_default() += 1;
                    }
                }
            }
        }
        out
    }

    /// 値の列の候補の集計(cell-edit.candidates-table-rows): 今の表のノートのうち `where` に当たるものから、値
    /// (`list` ならリストの要素。tags の頭の `#` は外す)ごとに数と一番新しい更新の日時を集め、`sort` で並べる。
    fn value_stats(&self, col: &str, spec: &Cands, list: bool) -> Vec<Stat> {
        let mut stats: Vec<Stat> = Vec::new();
        let mut at: HashMap<String, usize> = HashMap::new();
        for r in self.table_rows() {
            if !self.cand_where_ok(&r, spec) {
                continue;
            }
            let items: Vec<Value> = match self.prop(&r, col) {
                Some(Value::List(items)) if list => items,
                Some(v @ Value::Str(_)) if list => vec![v],
                Some(Value::List(_)) | None => continue,
                Some(v) => vec![v],
            };
            let newest = self.src.file(&r).map_or(i64::MIN, |f| f.mtime);
            let mut seen = HashSet::new();
            for v in items {
                let Some(mut key) = scalar_text(&v) else {
                    continue;
                };
                if list && col == "tags" {
                    key = key.strip_prefix('#').unwrap_or(&key).to_string();
                }
                if key.is_empty() || !seen.insert(key.clone()) {
                    continue;
                }
                let k = *at.entry(key.clone()).or_insert_with(|| {
                    stats.push(Stat {
                        value: v.clone(),
                        key: key.clone(),
                        count: 0,
                        newest: i64::MIN,
                    });
                    stats.len() - 1
                });
                stats[k].count += 1;
                stats[k].newest = stats[k].newest.max(newest);
            }
        }
        stats.sort_by(|a, b| a.key.cmp(&b.key));
        match spec.sort.unwrap_or(CandSort::Count) {
            CandSort::Name => {}
            CandSort::Count => stats.sort_by_key(|s| std::cmp::Reverse(s.count)),
            CandSort::Recent => stats.sort_by_key(|s| std::cmp::Reverse(s.newest)),
        }
        stats
    }

    /// 1つの値の列の候補: 出し方を書いた列は今の表のノートから集めて並べる。書かない列も今の表のノートから、表の行に
    /// 出てきた順に集め、異なる値が上限を超える列と、値のある行が5件以上でどの値も1件だけの列(URL・メモのように
    /// 1件ごとに違う値)は None(cell-edit.value-list-open)。
    pub(crate) fn value_cands(&self, col: &str) -> Option<Vec<Value>> {
        let Some(spec) = self.cand_spec(col) else {
            let mut out: Vec<Value> = Vec::new();
            let mut repeated = false;
            for r in self.table_rows() {
                let Some(v) = self.prop(&r, col) else {
                    continue;
                };
                // 空の値(Null・空の文字列・空のリスト)は候補にしない(読み込み口の候補と同じ)。
                if matches!(&v, Value::Null)
                    || matches!(&v, Value::Str(x) if x.is_empty())
                    || matches!(&v, Value::List(l) if l.is_empty())
                {
                    continue;
                }
                if out.contains(&v) {
                    repeated = true;
                    continue;
                }
                if out.len() == self.candidates {
                    return None;
                }
                out.push(v);
            }
            // 行の少ない表は、くり返しを見ない(始めたばかりの表の状態の列にも候補を出す)。
            return (repeated || out.len() < UNIQUE_MIN).then_some(out);
        };
        let stats = self.value_stats(col, spec, false);
        if stats.len() > self.candidates {
            return None;
        }
        Some(stats.into_iter().map(|s| s.value).collect())
    }

    /// リストの列の候補と、その要素を持つノートの数: 今の表のノートから集め、出し方を書いた列はその並べ方、書かない列は
    /// 多い順(同じ数は名前の順。cell-edit.list-items-candidates)。
    pub(crate) fn list_cands(&self, col: &str) -> Vec<(String, usize)> {
        let all = Cands::default();
        let spec = self.cand_spec(col).unwrap_or(&all);
        self.value_stats(col, spec, true)
            .into_iter()
            .map(|s| (s.key, s.count))
            .collect()
    }
}
