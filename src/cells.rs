//! セルの部品の設定(SR-35): `cells` は文字列(`"rich"`・`"plain"`)か表(`style`・部品の種類ごとの真偽・
//! `[cells.columns]` の列ごとの見せ方)。列ごとの設定は種類ごとの設定より優先する。色を使わない表示で
//! 部品にしないのは画面の側(この設定は色の有無を知らない)。

use crate::i18n::Msg;
use std::collections::BTreeMap;

/// 部品の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// 真偽を `☑`・`☐`。
    Checkbox,
    /// リストの要素を札。
    Chips,
    /// 種類の少ない短い文字の列の値を札。
    Select,
    /// リンクをアクセントの色。
    Links,
    /// 列の見出しに型の印。
    Icons,
}

/// 列ごとの見せ方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColStyle {
    /// 部品(種類ごとの設定によらず全部)。
    Rich,
    /// 文字のまま。
    Plain,
    /// 自動で札にならない列も、値を札に。
    Chip,
}

/// `cells` の設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    pub rich: bool,
    pub checkbox: bool,
    pub chips: bool,
    pub select: bool,
    pub links: bool,
    pub icons: bool,
    pub columns: BTreeMap<String, ColStyle>,
}

impl Default for Cells {
    fn default() -> Self {
        Cells {
            rich: true,
            checkbox: true,
            chips: true,
            select: true,
            links: true,
            icons: true,
            columns: BTreeMap::new(),
        }
    }
}

impl Cells {
    /// 列 `col` で部品 `part` を使うか(色の有無は見ない)。
    pub fn on(&self, col: &str, part: Part) -> bool {
        match self.columns.get(col) {
            Some(ColStyle::Plain) => false,
            Some(ColStyle::Rich) => true,
            // 札を強いる列: 値(文字・数・リストの要素)を札にし、印も出す。真偽とリンクは種類ごとの設定のまま。
            Some(ColStyle::Chip) => match part {
                Part::Select | Part::Chips | Part::Icons => true,
                _ => self.rich && self.kind_on(part),
            },
            None => self.rich && self.kind_on(part),
        }
    }

    /// 列 `col` の値を、自動の判定によらず札にするか。
    pub fn forced_chip(&self, col: &str) -> bool {
        self.columns.get(col) == Some(&ColStyle::Chip)
    }

    fn kind_on(&self, part: Part) -> bool {
        match part {
            Part::Checkbox => self.checkbox,
            Part::Chips => self.chips,
            Part::Select => self.select,
            Part::Links => self.links,
            Part::Icons => self.icons,
        }
    }
}

/// 設定の `cells` を読む。読めない所は警告にして既定のまま。
pub fn read(value: &toml::Value, c: &mut Cells, warnings: &mut Vec<String>) {
    let style = |v: &toml::Value, c: &mut Cells, w: &mut Vec<String>, name: &str| match v.as_str() {
        Some("rich") => c.rich = true,
        Some("plain") => c.rich = false,
        _ => w.push(Msg::ConfigWrongType.fill(&[&name, &Msg::WantCells.text()])),
    };
    let Some(t) = value.as_table() else {
        return style(value, c, warnings, "cells");
    };
    for (k, v) in t {
        let name = format!("cells.{k}");
        let flag = match k.as_str() {
            "style" => {
                style(v, c, warnings, &name);
                continue;
            }
            "checkbox" => &mut c.checkbox,
            "chips" => &mut c.chips,
            "select" => &mut c.select,
            "links" => &mut c.links,
            "icons" => &mut c.icons,
            "columns" => {
                read_columns(v, c, warnings);
                continue;
            }
            _ => {
                warnings.push(Msg::ConfigUnknownItem.fill(&[&name]));
                continue;
            }
        };
        match v.as_bool() {
            Some(b) => *flag = b,
            None => warnings.push(Msg::ConfigWrongType.fill(&[&name, &Msg::WantBool.text()])),
        }
    }
}

fn read_columns(v: &toml::Value, c: &mut Cells, warnings: &mut Vec<String>) {
    let Some(t) = v.as_table() else {
        warnings
            .push(Msg::ConfigWrongType.fill(&[&"cells.columns", &Msg::WantCellsColumns.text()]));
        return;
    };
    for (col, s) in t {
        let st = match s.as_str() {
            Some("rich") => ColStyle::Rich,
            Some("plain") => ColStyle::Plain,
            Some("chip") => ColStyle::Chip,
            _ => {
                let name = format!("cells.columns.{col}");
                warnings.push(Msg::ConfigWrongType.fill(&[&name, &Msg::WantColStyle.text()]));
                continue;
            }
        };
        c.columns.insert(col.clone(), st);
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
