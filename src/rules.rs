//! 列の決まりと問題の判定(CE-36・CV-9・NV-30・CLI-23)。形は docs/design.md の「型と決まりの問題」。
//!
//! 「問題」は、列の型に合わない値(CV-2)と、決まり(`[rules.<列>]` の `required`・`values`)に合わない値。
//! 判定はここ1か所にあり、画面(セルの印・跳ぶ・数・絞る)と `--check` が同じ判定を使う。

use crate::frontmatter::Value;
use crate::i18n::Msg;
use crate::source::{RowId, Source};
use crate::types::{self, Kind};
use std::collections::BTreeMap;

/// 1つの列の決まり(範囲を重ねたあと)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rule {
    /// 真なら、キーが無い・null・空の文字列・空のリストを違反とする。
    pub required: bool,
    /// 選べる値。値の文字(リストなら各要素の文字)がこのどれかでなければ違反。None は決まりなし。
    pub values: Option<Vec<String>>,
}

/// 1つの範囲に書いた決まり(書いた項目だけ。狭い範囲が項目ごとに勝つ。SR-44)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuleLayer {
    pub required: Option<bool>,
    pub values: Option<Vec<String>>,
}

impl RuleLayer {
    /// 広い範囲の決まり `r` に重ねる。
    pub fn apply(&self, r: &mut Rule) {
        if let Some(b) = self.required {
            r.required = b;
        }
        if let Some(v) = &self.values {
            r.values = Some(v.clone());
        }
    }
}

/// 問題の理由(CV-9)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// 列の型に合わない(CV-2)。
    Type(Kind),
    /// 必須のキーが無い(空)。
    Required,
    /// 選べる値ではない。
    NotAllowed,
}

impl Reason {
    /// 今の言語の理由の文。
    pub fn text(&self) -> String {
        match self {
            Reason::Type(k) => Msg::ProblemType.fill(&[&kind_label(*k)]),
            Reason::Required => Msg::ProblemRequired.text().to_string(),
            Reason::NotAllowed => Msg::ProblemNotAllowed.text().to_string(),
        }
    }
}

/// 型の名前(列の型の窓と同じ文言)。
pub fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Text => Msg::TypeText.text(),
        Kind::Number => Msg::TypeNumber.text(),
        Kind::Checkbox => Msg::TypeCheckbox.text(),
        Kind::Date => Msg::TypeDate.text(),
        Kind::DateTime => Msg::TypeDateTime.text(),
        Kind::List => Msg::TypeList.text(),
    }
}

/// 決まりを書けない列(計算の列)。
pub fn computed(col: &str) -> bool {
    col.starts_with("file.") || col.starts_with("formula.")
}

/// 選べる値と比べる値の文字。リストの要素と入れ子は Other として扱わない。
fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.clone()),
        Value::Int(i) => Some(i.to_string()),
        Value::Float(f) => Some(format!("{f:?}")),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null | Value::List(_) | Value::Other => None,
    }
}

/// 値 `v`(None はキーが無い)の問題。型 → 必須 → 選べる値の順で、最初の1つ。
pub fn problem(kind: Kind, rule: Option<&Rule>, v: Option<&Value>) -> Option<Reason> {
    if let Some(v) = v {
        if !types::fits(kind, v) {
            return Some(Reason::Type(kind));
        }
    }
    let rule = rule?;
    let empty = v.is_none_or(types::is_empty);
    if empty {
        return rule.required.then_some(Reason::Required);
    }
    let allowed = rule.values.as_ref()?;
    let ok = |x: &Value| scalar_text(x).is_some_and(|t| allowed.contains(&t));
    let fine = match v {
        Some(Value::List(items)) => items.iter().filter(|x| !types::is_empty(x)).all(ok),
        Some(x) => ok(x),
        None => true,
    };
    (!fine).then_some(Reason::NotAllowed)
}

/// `--check` の1つの問題(CLI-23)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub row: RowId,
    pub col: String,
    /// キーの行(1始まり)。キーが無い・行の分からない読み込み口は None。
    pub line: Option<usize>,
    pub reason: Reason,
}

/// 行 `rows` の各ノートの全部のキーと、決まりのある列を確かめる(CLI-23)。`kind` は列の型(CE-2 の順)。
/// 結果は行の順、同じ行では行の番号の順(キーの無い問題は後ろ)。
pub fn check(
    src: &dyn Source,
    rows: &[RowId],
    kind: &dyn Fn(&str) -> Kind,
    rules: &BTreeMap<String, Rule>,
) -> Vec<Found> {
    let mut cols: Vec<String> = src.columns().into_iter().filter(|c| !computed(c)).collect();
    for c in rules.keys() {
        if !computed(c) && !cols.contains(c) {
            cols.push(c.clone());
        }
    }
    let kinds: Vec<Kind> = cols.iter().map(|c| kind(c)).collect();
    let mut out = Vec::new();
    for row in rows {
        let lines = if src.notes() {
            key_line_numbers(src, row)
        } else {
            BTreeMap::new()
        };
        let mut found: Vec<Found> = Vec::new();
        for (c, k) in cols.iter().zip(&kinds) {
            let v = src.get(row, c).value;
            if let Some(reason) = problem(*k, rules.get(c), v.as_ref()) {
                found.push(Found {
                    row: row.clone(),
                    col: c.clone(),
                    line: v.as_ref().and(lines.get(c).copied()),
                    reason,
                });
            }
        }
        found.sort_by_key(|f| f.line.unwrap_or(usize::MAX));
        out.extend(found);
    }
    out
}

/// ノートのファイルのキーの行の番号(1始まり)。読めなければ空。
fn key_line_numbers(src: &dyn Source, row: &RowId) -> BTreeMap<String, usize> {
    let (path, _) = src.locate(row);
    let Ok(bytes) = std::fs::read(path) else {
        return BTreeMap::new();
    };
    let Some(keys) = crate::frontmatter::key_lines(&bytes) else {
        return BTreeMap::new();
    };
    keys.into_iter()
        .map(|k| {
            let line = bytes[..k.token.start]
                .iter()
                .filter(|b| **b == b'\n')
                .count()
                + 1;
            (k.key, line)
        })
        .collect()
}
