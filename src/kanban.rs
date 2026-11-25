//! カンバン(specs/_history/decidespec/kanban/spec.md の KB-1〜KB-6)の核。画面に依存しない: 列の名前からの推定(KB-1)と、
//! レーンの順(KB-2。列の決まり CE-36 の `values` → 決まりに無い値を現れた順 → 値の無い行)。

use crate::i18n::{Lang, Msg};

/// KB-1: 設定が無いときに状態の列とみなす名前(大文字小文字を問わない)。全部の言語の表(locales の
/// `KanbanColumnNames`、空白区切り)から集める: 訳ではなく、どの言語の画面でも同じ名前を推定に使う
/// (コードに日本語のリテラルを置かない。SR-23。ガントの `start_names` と同じ)。
pub fn column_names() -> Vec<&'static str> {
    Lang::ALL
        .iter()
        .flat_map(|l| Msg::KanbanColumnNames.get(*l).split_whitespace())
        .collect()
}

/// 印の `kanban = { column, card }`、または推定した列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KanbanSpec {
    pub column: String,
    /// kanban.card-columns: 札の名前の下に値を出す列(書いた順。`note.` を除いた名前。推定では空)。
    pub card: Vec<String>,
}

/// KB-1: 列の名前とリストの型かの組から、状態の列を推定する。リストでない列のうち、名前が `column_names()` に
/// 当たる最初の列。無ければ None。
pub fn infer(cols: &[(String, bool)]) -> Option<KanbanSpec> {
    let names = column_names();
    cols.iter()
        .filter(|(_, is_list)| !*is_list)
        .map(|(n, _)| n)
        .find(|n| {
            let lower = n.to_lowercase();
            names.iter().any(|x| x.to_lowercase() == lower)
        })
        .map(|n| KanbanSpec {
            column: n.clone(),
            card: Vec::new(),
        })
}

/// レーンの鍵: 列の値か、値の無い行。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LaneKey {
    Value(String),
    NoValue,
}

/// KB-2: レーンの順。`rules` は列の決まり(CE-36 の `values`)、`values` は行ごとの値の文字(前後の空白を除いたもの。
/// 空なら値なし)。決まりの順(重なりは1つ。行の無い値もレーンにする)→ 決まりに無い値を現れた順 → 値の無い行が
/// あれば最後に `NoValue`。
pub fn lane_keys(rules: Option<&[String]>, values: &[String]) -> Vec<LaneKey> {
    let mut out: Vec<LaneKey> = Vec::new();
    let push = |k: LaneKey, out: &mut Vec<LaneKey>| {
        if !out.contains(&k) {
            out.push(k);
        }
    };
    for v in rules.unwrap_or_default() {
        let v = v.trim();
        if !v.is_empty() {
            push(LaneKey::Value(v.to_string()), &mut out);
        }
    }
    let mut none = false;
    for v in values {
        let v = v.trim();
        if v.is_empty() {
            none = true;
        } else {
            push(LaneKey::Value(v.to_string()), &mut out);
        }
    }
    if none {
        out.push(LaneKey::NoValue);
    }
    out
}

/// 行の値の文字からその行のレーンの鍵。
pub fn key_of(value: &str) -> LaneKey {
    let v = value.trim();
    if v.is_empty() {
        LaneKey::NoValue
    } else {
        LaneKey::Value(v.to_string())
    }
}
