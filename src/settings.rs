//! ビューの設定(NV-13〜NV-22)の核。条件の判定と、行を絞る・並べる・まとめる処理。
//! 画面に依存しない。形は docs/design.md の「ビューの設定(NV-13〜NV-22)」。
//!
//! 比べ方: 数は数として(f64)、日付は `types::parse_date`、日時は日数と秒。Text などのほかの型は
//! 表示の文字列の順。型の合わない値と空は比較(Cmp)で残らず、並べ替えでは向きによらず後ろ。

use crate::display::DisplayOverride;
use crate::i18n::Msg;
use crate::source::{RowId, Value};
use crate::types::{self, Kind};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;

/// 空のまとまりの見出し(`.base` の groupBy と同じ。日本語の文)。見出しと帯には今の言語の
/// `Msg::EmptyHeading.text()` を使う(SR-23)。
pub const EMPTY_HEADING: &str = Msg::EmptyHeading.ja();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    /// 値の一覧のチェック: 残す値(None は「(空)」)。リストの値は要素のどれかが入れば残す。
    Keep(#[serde(with = "keys_serde")] Vec<Option<String>>),
    /// 隠す値。リストの値は要素のどれかが入れば隠す。
    Drop(#[serde(with = "keys_serde")] Vec<Option<String>>),
    /// テキストを含む(大文字小文字を区別しない)。
    Contains(String),
    NotContains(String),
    /// 数と日付の比較。列の型(Kind)で読む。型の合わない値・空は残らない。
    Cmp(CmpOp, String),
    Empty,
    NotEmpty,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cond {
    pub col: String,
    pub op: Op,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Group {
    /// `.base` の groupBy のまま。
    #[default]
    Inherit,
    /// まとめない。
    Off,
    /// 列の値でまとめ直す。
    By {
        col: String,
        dir: Dir,
        hide_empty: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub filters: Vec<Cond>,
    pub sorts: Vec<(String, Dir)>,
    pub group: Group,
    /// SR-20・SR-21: 表の見せ方のビューごとの上書き(設定と違う項目だけ)。
    #[serde(skip_serializing_if = "DisplayOverride::is_empty")]
    pub display: DisplayOverride,
}

impl Settings {
    pub fn is_default(&self) -> bool {
        *self == Settings::default()
    }

    /// 行に効く条件(フィルター・並べ替え・グループ)が無いか(設定の帯の項目が無い。NV-16)。
    /// 見せ方の上書き(`display`)は問わない。
    pub fn no_conditions(&self) -> bool {
        self.filters.is_empty() && self.sorts.is_empty() && self.group == Group::Inherit
    }

    /// 表の上の帯の項目(NV-16)。1条件 = 1項目。
    pub fn chips(&self) -> Vec<String> {
        let mut out: Vec<String> = self.filters.iter().map(cond_text).collect();
        for (col, dir) in &self.sorts {
            out.push(Msg::ChipSort.fill(&[col, &arrow(*dir)]));
        }
        match &self.group {
            Group::Inherit => {}
            Group::Off => out.push(Msg::ChipGroupOff.text().to_string()),
            Group::By {
                col,
                dir,
                hide_empty,
            } => {
                let mut s = Msg::ChipGroup.fill(&[col, &arrow(*dir)]);
                if *hide_empty {
                    s.push_str(Msg::ChipHideEmpty.text());
                }
                out.push(s);
            }
        }
        out
    }
}

fn arrow(d: Dir) -> &'static str {
    match d {
        Dir::Asc => "↑",
        Dir::Desc => "↓",
    }
}

fn key_label(k: &Option<String>) -> &str {
    k.as_deref().unwrap_or(Msg::EmptyHeading.text())
}

fn keys_label(keys: &[Option<String>]) -> String {
    keys.iter().map(key_label).collect::<Vec<_>>().join(", ")
}

fn cmp_symbol(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "=",
        CmpOp::Ne => "≠",
        CmpOp::Lt => "<",
        CmpOp::Le => "≤",
        CmpOp::Gt => ">",
        CmpOp::Ge => "≥",
    }
}

fn cond_text(c: &Cond) -> String {
    let body = match &c.op {
        Op::Keep(k) => Msg::CondOnly.fill(&[&keys_label(k)]),
        Op::Drop(k) => Msg::CondExcept.fill(&[&keys_label(k)]),
        Op::Contains(s) => Msg::CondContains.fill(&[s]),
        Op::NotContains(s) => Msg::CondNotContains.fill(&[s]),
        Op::Cmp(op, s) => format!("{} {}", cmp_symbol(*op), s),
        Op::Empty => Msg::CondEmpty.text().to_string(),
        Op::NotEmpty => Msg::CondNotEmpty.text().to_string(),
    };
    format!("{}: {}", c.col, body)
}

/// スカラーの値の表示の文字列。空(Null・空の文字列)は None。
fn scalar_key(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::Bool(b) => Some(b.to_string()),
        Value::Int(n) => Some(n.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Str(s) if s.is_empty() => None,
        Value::Str(s) => Some(s.clone()),
        Value::List(_) => None,
        Value::Other => Some("{…}".to_string()),
    }
}

/// 値の鍵(表示の文字列)。空(Null・空の文字列・空のリスト・キーなし)は [None]、リストは要素ごと。
pub fn value_keys(v: Option<&Value>) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = match v {
        None => Vec::new(),
        Some(Value::List(items)) => items
            .iter()
            .filter_map(|i| match i {
                Value::List(_) => Some(Some("{…}".to_string())),
                other => scalar_key(other).map(Some),
            })
            .collect(),
        Some(other) => scalar_key(other).map(Some).into_iter().collect(),
    };
    if out.is_empty() {
        out.push(None);
    }
    out
}

/// 値の一覧(件数つき。空は None。件数の多い順、同数は文字の順で空は後ろ)。リストは要素ごとに数える(NV-19)。
pub fn value_counts<'a>(
    values: impl Iterator<Item = Option<&'a Value>>,
) -> Vec<(Option<String>, usize)> {
    let mut counts: HashMap<Option<String>, usize> = HashMap::new();
    for v in values {
        let mut keys = value_keys(v);
        keys.sort();
        keys.dedup();
        for k in keys {
            *counts.entry(k).or_default() += 1;
        }
    }
    let mut out: Vec<(Option<String>, usize)> = counts.into_iter().collect();
    out.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| match (&a.0, &b.0) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
    });
    out
}

fn is_empty(v: Option<&Value>) -> bool {
    value_keys(v) == [None]
}

/// 比べるための鍵。型の合わない値は None。
#[derive(Clone, Debug, PartialEq)]
enum Cmpable {
    Num(f64),
    Date(i64),
    DateTime(i64, i64),
    Text(String),
}

impl Cmpable {
    fn cmp(&self, other: &Cmpable) -> Option<Ordering> {
        match (self, other) {
            (Cmpable::Num(a), Cmpable::Num(b)) => a.partial_cmp(b),
            (Cmpable::Date(a), Cmpable::Date(b)) => Some(a.cmp(b)),
            (Cmpable::DateTime(a, x), Cmpable::DateTime(b, y)) => Some((a, x).cmp(&(b, y))),
            (Cmpable::Text(a), Cmpable::Text(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

/// 日時(`YYYY-MM-DD` か `YYYY-MM-DDTHH:MM[:SS[.frac]]`)を (日数, 秒) にする。
fn parse_datetime(s: &str) -> Option<(i64, i64)> {
    if let Some(d) = types::parse_date(s) {
        return Some((d, 0));
    }
    // 時差つき・空白の区切りも(地域の時刻に直して比べる。C-2)。
    if let Some(t) = types::parse_datetime(s) {
        return Some((t.div_euclid(86_400), t.rem_euclid(86_400)));
    }
    if !types::fits(Kind::DateTime, &Value::Str(s.to_string())) {
        return None;
    }
    // fits を通った日時は ASCII で、`YYYY-MM-DDTHH:MM` で始まる。
    let b = s.as_bytes();
    let d = types::parse_date(s.get(..10)?)?;
    let num = |r: Range<usize>| -> Option<i64> { s.get(r)?.parse().ok() };
    let h = num(11..13)?;
    let m = num(14..16)?;
    let sec = if b.get(16) == Some(&b':') {
        num(17..19)?
    } else {
        0
    };
    Some((d, h * 3600 + m * 60 + sec))
}

/// スカラーの値を列の型で読む。空と型の合わない値は None。
fn scalar_cmpable(v: &Value, kind: Kind) -> Option<Cmpable> {
    match kind {
        Kind::Number => match v {
            Value::Int(n) => Some(Cmpable::Num(*n as f64)),
            Value::Float(f) if !f.is_nan() => Some(Cmpable::Num(*f)),
            _ => None,
        },
        Kind::Date => match v {
            Value::Str(s) => types::parse_date(s).map(Cmpable::Date),
            _ => None,
        },
        Kind::DateTime => match v {
            Value::Str(s) => parse_datetime(s).map(|(d, t)| Cmpable::DateTime(d, t)),
            _ => None,
        },
        _ => match v {
            Value::List(_) => None,
            other => scalar_key(other).map(Cmpable::Text),
        },
    }
}

/// 打たれた比較の値を列の型で読む。
fn operand(s: &str, kind: Kind) -> Option<Cmpable> {
    let s = s.trim();
    match kind {
        Kind::Number => s
            .parse::<f64>()
            .ok()
            .filter(|f| !f.is_nan())
            .map(Cmpable::Num),
        Kind::Date => types::parse_date(s).map(Cmpable::Date),
        Kind::DateTime => parse_datetime(s).map(|(d, t)| Cmpable::DateTime(d, t)),
        _ => Some(Cmpable::Text(s.to_string())),
    }
}

/// 値の要素(リストは要素ごと、スカラーは1つ)。
fn elements(v: &Value) -> Vec<&Value> {
    match v {
        Value::List(items) => items.iter().collect(),
        other => vec![other],
    }
}

/// 1つの値に条件を当てる(NV-14・NV-19)。
pub fn matches(cond: &Cond, v: Option<&Value>, kind: Kind) -> bool {
    match &cond.op {
        Op::Keep(keys) => value_keys(v).iter().any(|k| keys.contains(k)),
        Op::Drop(keys) => !value_keys(v).iter().any(|k| keys.contains(k)),
        Op::Contains(s) => contains(v, s),
        Op::NotContains(s) => !contains(v, s),
        Op::Cmp(op, s) => {
            let Some(rhs) = operand(s, kind) else {
                return false;
            };
            let Some(v) = v else {
                return false;
            };
            // 列の型で読めた要素ごとの比較の結果。型の合わない要素と空の要素は入らない。
            let ords: Vec<Ordering> = elements(v)
                .into_iter()
                .filter_map(|e| scalar_cmpable(e, kind)?.cmp(&rhs))
                .collect();
            match op {
                // ≠ だけは、読めた要素がどれも等しくない(リストに等しい要素が1つでもあれば残さない)。
                CmpOp::Ne => !ords.is_empty() && ords.iter().all(|o| *o != Ordering::Equal),
                CmpOp::Eq => ords.contains(&Ordering::Equal),
                CmpOp::Lt => ords.contains(&Ordering::Less),
                CmpOp::Le => ords.iter().any(|o| *o != Ordering::Greater),
                CmpOp::Gt => ords.contains(&Ordering::Greater),
                CmpOp::Ge => ords.iter().any(|o| *o != Ordering::Less),
            }
        }
        Op::Empty => is_empty(v),
        Op::NotEmpty => !is_empty(v),
    }
}

fn contains(v: Option<&Value>, needle: &str) -> bool {
    let needle = needle.to_lowercase();
    value_keys(v)
        .into_iter()
        .flatten()
        .any(|k| k.to_lowercase().contains(&needle))
}

/// 並べ替えの鍵。型の合わない値と空は None(向きによらず後ろ)。リストは空でない最初の要素で比べる。
fn sort_key(v: Option<&Value>, kind: Kind) -> Option<Cmpable> {
    let v = v?;
    let first = match v {
        Value::List(items) => items
            .iter()
            .find(|e| !matches!(e, Value::Null) && !matches!(e, Value::Str(s) if s.is_empty()))?,
        other => other,
    };
    match (kind, v) {
        // リストの列は表示の文字列(要素をつないだもの)で比べる。
        (Kind::List, Value::List(_)) => {
            let keys = value_keys(Some(v));
            if keys == [None] {
                None
            } else {
                Some(Cmpable::Text(
                    keys.into_iter().flatten().collect::<Vec<_>>().join(", "),
                ))
            }
        }
        _ => scalar_cmpable(first, kind),
    }
}

/// 鍵を向きで比べる。None は向きによらず後ろ。
fn key_cmp(a: &Option<Cmpable>, b: &Option<Cmpable>, dir: Dir) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => {
            let o = x.cmp(y).unwrap_or(Ordering::Equal);
            match dir {
                Dir::Asc => o,
                Dir::Desc => o.reverse(),
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// 行を sorts で並べる(安定)。
fn sort_rows(
    rows: &mut [RowId],
    sorts: &[(String, Dir)],
    get: &dyn Fn(&RowId, &str) -> Option<Value>,
    kind: &dyn Fn(&str) -> Kind,
) {
    if sorts.is_empty() {
        return;
    }
    let kinds: Vec<Kind> = sorts.iter().map(|(c, _)| kind(c)).collect();
    let mut keyed: Vec<(Vec<Option<Cmpable>>, RowId)> = rows
        .iter()
        .map(|r| {
            let keys = sorts
                .iter()
                .zip(&kinds)
                .map(|((c, _), k)| sort_key(get(r, c).as_ref(), *k))
                .collect();
            (keys, r.clone())
        })
        .collect();
    keyed.sort_by(|(a, _), (b, _)| {
        a.iter()
            .zip(b)
            .zip(sorts)
            .map(|((x, y), (_, d))| key_cmp(x, y, *d))
            .find(|o| *o != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    });
    for (slot, (_, r)) in rows.iter_mut().zip(keyed) {
        *slot = r;
    }
}

/// まとまりの見出しの鍵(表示の文字列)。空は None。リストは要素をつないだもの。
fn group_label(v: Option<&Value>) -> Option<String> {
    let keys = value_keys(v);
    if keys == [None] {
        return None;
    }
    Some(keys.into_iter().flatten().collect::<Vec<_>>().join(", "))
}

/// 行を絞り(全部の条件を満たす行だけ。NV-14・NV-19)、並べ(sorts があれば並べ直す。安定。型の合わない値と空は後ろ)、
/// まとめる(Inherit は渡された groups のまま、Off はまとめない、By は列の値でまとめ直す。NV-15・NV-21)。
pub fn apply(
    rows: Vec<RowId>,
    groups: Vec<(String, Range<usize>)>,
    s: &Settings,
    get: &dyn Fn(&RowId, &str) -> Option<Value>,
    kind: &dyn Fn(&str) -> Kind,
) -> (Vec<RowId>, Vec<(String, Range<usize>)>) {
    let kinds: Vec<Kind> = s.filters.iter().map(|c| kind(&c.col)).collect();
    let keep = |r: &RowId| -> bool {
        s.filters
            .iter()
            .zip(&kinds)
            .all(|(c, k)| matches(c, get(r, &c.col).as_ref(), *k))
    };

    match &s.group {
        Group::Inherit if !groups.is_empty() => {
            // 渡されたまとまりごとに絞って並べる。空になったまとまりは出さない。
            let mut out_rows = Vec::with_capacity(rows.len());
            let mut out_groups = Vec::with_capacity(groups.len());
            let mut covered = 0;
            let mut push = |h: String, range: Range<usize>, out_rows: &mut Vec<RowId>| {
                let mut part: Vec<RowId> =
                    rows[range].iter().filter(|r| keep(r)).cloned().collect();
                sort_rows(&mut part, &s.sorts, get, kind);
                if !part.is_empty() {
                    let start = out_rows.len();
                    out_rows.extend(part);
                    out_groups.push((h, start..out_rows.len()));
                }
            };
            for (h, range) in groups {
                let start = range.start.clamp(covered, rows.len());
                let end = range.end.clamp(start, rows.len());
                // まとまりの間の隙間の行も落とさない(前のまとまりに入れずに、見出しなしのまま)。
                if start > covered {
                    let mut gap: Vec<RowId> = rows[covered..start]
                        .iter()
                        .filter(|r| keep(r))
                        .cloned()
                        .collect();
                    sort_rows(&mut gap, &s.sorts, get, kind);
                    out_rows.extend(gap);
                }
                push(h, start..end, &mut out_rows);
                covered = end;
            }
            if covered < rows.len() {
                let mut rest: Vec<RowId> = rows[covered..]
                    .iter()
                    .filter(|r| keep(r))
                    .cloned()
                    .collect();
                sort_rows(&mut rest, &s.sorts, get, kind);
                out_rows.extend(rest);
            }
            (out_rows, out_groups)
        }
        Group::Inherit | Group::Off => {
            let mut out: Vec<RowId> = rows.into_iter().filter(|r| keep(r)).collect();
            sort_rows(&mut out, &s.sorts, get, kind);
            (out, Vec::new())
        }
        Group::By {
            col,
            dir,
            hide_empty,
        } => {
            let gkind = kind(col);
            let mut out: Vec<RowId> = rows.into_iter().filter(|r| keep(r)).collect();
            sort_rows(&mut out, &s.sorts, get, kind);
            // 見出しごとに、最初に現れた順で行を集める(中は sorts の順のまま)。
            let mut order: Vec<(Option<String>, Option<Cmpable>, Vec<RowId>)> = Vec::new();
            let mut index: HashMap<Option<String>, usize> = HashMap::new();
            for r in out {
                let v = get(&r, col);
                let label = group_label(v.as_ref());
                if label.is_none() && *hide_empty {
                    continue;
                }
                match index.get(&label) {
                    Some(&i) => order[i].2.push(r),
                    None => {
                        index.insert(label.clone(), order.len());
                        let key = sort_key(v.as_ref(), gkind);
                        order.push((label, key, vec![r]));
                    }
                }
            }
            // まとまりの並び: 値の順(向きつき)。型の合わない値はその後ろ(文字の順)、空は最後。
            order.sort_by(|a, b| match (&a.0, &b.0) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(x), Some(y)) => match (&a.1, &b.1) {
                    (None, None) => {
                        let o = x.cmp(y);
                        match dir {
                            Dir::Asc => o,
                            Dir::Desc => o.reverse(),
                        }
                    }
                    _ => key_cmp(&a.1, &b.1, *dir),
                },
            });
            let mut rows_out = Vec::new();
            let mut groups_out = Vec::new();
            for (label, _, part) in order {
                let start = rows_out.len();
                rows_out.extend(part);
                groups_out.push((
                    label.unwrap_or_else(|| Msg::EmptyHeading.text().to_string()),
                    start..rows_out.len(),
                ));
            }
            (rows_out, groups_out)
        }
    }
}

/// Keep・Drop の値(`Vec<Option<String>>`)の保存の形。TOML は None を書けないので、
/// 1つの値を表 `{ value = "..." }`、「(空)」を `{ empty = true }` にした表の配列で持つ。
mod keys_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Key {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        empty: bool,
    }

    pub fn serialize<S: Serializer>(keys: &[Option<String>], s: S) -> Result<S::Ok, S::Error> {
        let v: Vec<Key> = keys
            .iter()
            .map(|k| match k {
                Some(v) => Key {
                    value: Some(v.clone()),
                    empty: false,
                },
                None => Key {
                    value: None,
                    empty: true,
                },
            })
            .collect();
        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Option<String>>, D::Error> {
        let v = Vec::<Key>::deserialize(d)?;
        Ok(v.into_iter()
            .map(|k| if k.empty { None } else { k.value })
            .collect())
    }
}

#[cfg(test)]
#[path = "test_settings_unit.rs"]
mod tests;
