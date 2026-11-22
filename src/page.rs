//! ページ(stack.page-*): 縦に並べた表の形(表・それぞれのビュー・高さの比)に名前を付けて、ワークスペースに
//! 残したもの。置き場は範囲(ワークスペース)と同じで、`workspaces.toml` のワークスペースは `[[workspace.page]]`、
//! フォルダの印(`.mdgrid/workspace.toml`)は `[[page]]`(名前は tbd~stack.page-name の仮)。
//!
//! 書き方(どちらの置き場も同じ。手で書いてもよい):
//!
//! ```toml
//! [[page]]
//! name = "朝"
//! tables = [
//!   { table = "tasks", view = "今日", height = 1 },
//!   { table = "projects", view = "進行中", height = 1 },
//! ]
//! ```
//!
//! `table` は範囲の表の `name`、`view` は開くビュー(無ければその表のいつものビュー)、`height` は高さの比
//! (1 以上の整数。無ければ 1)。読めない行は「ファイル: 道筋: 理由」(cli.config-file-warning-form)で警告して
//! 飛ばす。読むのは workspace.rs(印と workspaces.toml の文)、書くのも workspace.rs(手の行を残す書き方)。

use crate::i18n::Msg;
use crate::profile::{not, warn};

/// ページの名前を置く鍵(tbd~stack.page-name の仮)。印は `[[page]]`、`workspaces.toml` は `[[workspace.page]]`。
pub const KEY: &str = "page";
/// ページの表の並びの鍵。
pub const TABLES: &str = "tables";

/// ページの表1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageTable {
    /// 範囲の表の名前(`workspace.table` の `name`)。
    pub table: String,
    /// 開くビュー(無ければ範囲の表の `view` か、いつものビュー)。
    pub view: Option<String>,
    /// 高さの比(1 以上)。
    pub height: u32,
}

/// ページ1つ。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    pub name: String,
    pub tables: Vec<PageTable>,
}

/// ページの並び(`page` の値)を読む。`known` は範囲の表の名前で、無い名前の表は警告して飛ばす
/// (stack.page-store)。表が1つも残らないページも警告して飛ばす。
pub fn read(
    v: Option<&toml::Value>,
    known: &[String],
    (file, prefix): (&str, &str),
    warns: &mut Vec<String>,
) -> Vec<Page> {
    let mut out: Vec<Page> = Vec::new();
    let Some(v) = v else {
        return out;
    };
    let Some(items) = v.as_array() else {
        warn(warns, file, prefix, not(Msg::WantPageList));
        return out;
    };
    for (k, item) in items.iter().enumerate() {
        let at = format!("{prefix}[{}]", k + 1);
        let Some(t) = item.as_table() else {
            warn(warns, file, &at, not(Msg::WantTable));
            continue;
        };
        let Some(name) = t
            .get("name")
            .and_then(toml::Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.contains(['\n', '\r']))
            .filter(|s| !out.iter().any(|p| &p.name == s))
        else {
            warn(warns, file, &at, Msg::RsnNeedName.text().to_string());
            continue;
        };
        for key in t.keys().filter(|k| *k != "name" && *k != TABLES) {
            warn(
                warns,
                file,
                &format!("{at}.{key}"),
                Msg::RsnUnknown.text().to_string(),
            );
        }
        let tables = read_tables(
            t.get(TABLES),
            known,
            (file, &format!("{at}.{TABLES}")),
            warns,
        );
        if tables.is_empty() {
            warn(warns, file, &at, Msg::RsnPageEmpty.fill(&[&name]));
            continue;
        }
        out.push(Page { name, tables });
    }
    out
}

/// ページの表の並び(`tables`)を読む。
fn read_tables(
    v: Option<&toml::Value>,
    known: &[String],
    (file, prefix): (&str, &str),
    warns: &mut Vec<String>,
) -> Vec<PageTable> {
    let mut out = Vec::new();
    let Some(items) = v.and_then(toml::Value::as_array) else {
        warn(warns, file, prefix, not(Msg::WantPageTables));
        return out;
    };
    for (k, item) in items.iter().enumerate() {
        let at = format!("{prefix}[{}]", k + 1);
        let Some(t) = item.as_table() else {
            warn(warns, file, &at, not(Msg::WantPageTables));
            continue;
        };
        let Some(table) = t
            .get("table")
            .and_then(toml::Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            warn(warns, file, &at, Msg::RsnPageNeedTable.text().to_string());
            continue;
        };
        if !known.contains(&table) {
            warn(
                warns,
                file,
                &format!("{at}.table"),
                Msg::RsnPageNoTable.fill(&[&table, &known.join(", ")]),
            );
            continue;
        }
        let view = match t.get("view") {
            None => None,
            Some(toml::Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
            Some(_) => {
                warn(warns, file, &format!("{at}.view"), not(Msg::WantString));
                None
            }
        };
        let height = match t.get("height") {
            None => 1,
            Some(toml::Value::Integer(n)) if *n >= 1 => u32::try_from(*n).unwrap_or(u32::MAX),
            Some(_) => {
                warn(
                    warns,
                    file,
                    &format!("{at}.height"),
                    not(Msg::WantIntAtLeast1),
                );
                1
            }
        };
        for key in t
            .keys()
            .filter(|k| !matches!(k.as_str(), "table" | "view" | "height"))
        {
            warn(
                warns,
                file,
                &format!("{at}.{key}"),
                Msg::RsnUnknown.text().to_string(),
            );
        }
        out.push(PageTable {
            table,
            view,
            height,
        });
    }
    out
}

/// TOML の文字列(引用符つき)。
fn quoted(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

/// ページの区画の文(見出し `head` は `[[page]]` か `[[workspace.page]]`)。最後の行は改行で終える。
pub fn text(page: &Page, head: &str) -> String {
    let mut s = format!("{head}\nname = {}\n{TABLES} = [\n", quoted(&page.name));
    for t in &page.tables {
        s.push_str(&format!("  {{ table = {}", quoted(&t.table)));
        if let Some(v) = &t.view {
            s.push_str(&format!(", view = {}", quoted(v)));
        }
        s.push_str(&format!(", height = {} }},\n", t.height.max(1)));
    }
    s.push_str("]\n");
    s
}

/// ページを TOML の値にする(`workspaces.toml` を並びから書き直すときに、ページを落とさないため)。
pub fn to_value(page: &Page) -> toml::Value {
    let mut t = toml::Table::new();
    t.insert("name".into(), toml::Value::String(page.name.clone()));
    let tables = page
        .tables
        .iter()
        .map(|x| {
            let mut e = toml::Table::new();
            e.insert("table".into(), toml::Value::String(x.table.clone()));
            if let Some(v) = &x.view {
                e.insert("view".into(), toml::Value::String(v.clone()));
            }
            e.insert(
                "height".into(),
                toml::Value::Integer(i64::from(x.height.max(1))),
            );
            toml::Value::Table(e)
        })
        .collect();
    t.insert(TABLES.into(), toml::Value::Array(tables));
    toml::Value::Table(t)
}

/// 高さの比を小さい整数にする(全部が同じなら 1 ずつ。最大公約数で割る)。
pub fn ratio(heights: &[u32]) -> Vec<u32> {
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    let g = heights.iter().copied().fold(0, gcd).max(1);
    heights.iter().map(|h| (h / g).max(1)).collect()
}

/// ページの名前として使えるか(空でなく、改行を含まない)。
pub fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && !name.contains(['\n', '\r'])
}

#[cfg(test)]
#[path = "test_page_unit.rs"]
mod tests;
