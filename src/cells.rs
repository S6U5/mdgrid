//! セルの部品(SR-35): `look.cells`(`"rich"`・`"plain"`)と `[look.columns]` の列ごとの見せ方。部品の種類ごとに
//! 使うかは部品の形(SR-36。`plain`・`text`・印なしなら文字のまま)。列ごとの設定は部品の形より優先する。
//! 色を使わない表示で部品にしないのは画面の側(この設定は色の有無を知らない)。読み取りは profile。

use crate::style::{Check, Links, Status, Style, Tags};
use std::collections::BTreeMap;

/// 部品の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// 真偽を `☑`・`☐` など(SR-36 の `check`)。
    Checkbox,
    /// リストの要素を札(`tags`)。
    Chips,
    /// 種類の少ない短い文字の列の値を札(`status`)。
    Select,
    /// リンクをアクセントの色(`links`)。
    Links,
    /// 列の見出しに型の印(`icons`)。
    Icons,
}

/// 列ごとの見せ方(`[look.columns]`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColStyle {
    /// 部品(部品の形によらず全部)。
    Rich,
    /// 文字のまま。
    Plain,
    /// 自動で札にならない列も、値を札に。
    Chip,
}

impl ColStyle {
    pub fn parse(s: &str) -> Option<ColStyle> {
        match s {
            "rich" => Some(ColStyle::Rich),
            "plain" => Some(ColStyle::Plain),
            "chip" => Some(ColStyle::Chip),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ColStyle::Rich => "rich",
            ColStyle::Plain => "plain",
            ColStyle::Chip => "chip",
        }
    }
}

/// セルの部品の決まった値(`look.cells` と `[look.columns]`)。部品の種類ごとに使うかは部品の形(`Style`)で決める。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    pub rich: bool,
    pub columns: BTreeMap<String, ColStyle>,
}

impl Default for Cells {
    fn default() -> Self {
        Cells {
            rich: true,
            columns: BTreeMap::new(),
        }
    }
}

impl Cells {
    /// 列 `col` で部品 `part` を使うか(色の有無は見ない)。部品の形が文字のまま(`plain`・`text`・印なし)なら使わない。
    pub fn on(&self, col: &str, part: Part, style: &Style) -> bool {
        match self.columns.get(col) {
            Some(ColStyle::Plain) => false,
            Some(ColStyle::Rich) => true,
            // 札を強いる列: 値(文字・数・リストの要素)を札にし、印も出す。真偽とリンクは部品の形のまま。
            Some(ColStyle::Chip) => match part {
                Part::Select | Part::Chips | Part::Icons => true,
                _ => self.rich && kind_on(part, style),
            },
            None => self.rich && kind_on(part, style),
        }
    }

    /// 列 `col` の値を、自動の判定によらず札にするか。
    pub fn forced_chip(&self, col: &str) -> bool {
        self.columns.get(col) == Some(&ColStyle::Chip)
    }
}

/// 部品の形が、その部品を使う形か。
fn kind_on(part: Part, s: &Style) -> bool {
    match part {
        Part::Checkbox => s.check != Check::Text,
        Part::Chips => s.tags != Tags::Plain,
        Part::Select => s.status != Status::Plain,
        Part::Links => s.links != Links::Plain,
        Part::Icons => s.icons,
    }
}

/// 種類の少ない短い文字の列か(自動の札。SR-35)。`values` は値のある行の文字(リスト・数などは渡さない)。
/// 値のある行が2つ以上、違う値が 12 以下で、くり返しがあり(違う値が行の数より少ない)、どの値も 20 字以下で改行なし。
pub fn looks_like_select<S: AsRef<str>>(values: impl Iterator<Item = S>) -> bool {
    let mut seen: Vec<String> = Vec::new();
    let mut n = 0usize;
    for v in values {
        let v = v.as_ref();
        if v.is_empty() {
            continue;
        }
        if v.chars().count() > 20 || v.contains(['\n', '\r']) {
            return false;
        }
        n += 1;
        if !seen.iter().any(|s| s == v) {
            if seen.len() == 12 {
                return false;
            }
            seen.push(v.to_string());
        }
    }
    n >= 2 && seen.len() < n
}

#[cfg(test)]
#[path = "test_cells_unit.rs"]
mod tests;
