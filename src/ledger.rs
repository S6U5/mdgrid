//! 画面の文言の台帳(`locales/<言語の符号>.toml`)を読み、確かめる純関数(SR-23)。
//!
//! build.rs が `#[path]` でこのファイルを使い、台帳から文言の表(`Lang`・`Language`・`Msg`)を作る。台帳に誤りが
//! あればビルドを落とす。lib からも `mdgrid::ledger` として呼べる(確かめ方そのものを試験する)。依存は `toml` だけで、
//! crate のほかの模組を使わない(build.rs から使うため)。
//!
//! 台帳は1行に1つ `名前 = "文"`(TOML の基本の文字列)。`##` で始まる行は次の文言の説明、`#` で始まる行は区切り。
//! 英語の台帳(`en.toml`)が文言の名前と並びの正本。

use std::collections::{BTreeSet, HashSet};

/// 台帳の1つの文言。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// 文言の名前(`Msg` の値の名前)。
    pub key: String,
    /// 説明の行(`##` の後ろ。英語の台帳のものがコードの文書になる)。
    pub doc: Vec<String>,
    /// 文。
    pub text: String,
}

/// 台帳を読む。並びは行の順。`name` は誤りの文に出す台帳の名前。
pub fn parse(name: &str, text: &str) -> Result<Vec<Entry>, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("{name}: not valid TOML: {e}"))?;
    let mut out = Vec::new();
    let mut doc = Vec::new();
    for line in text.lines() {
        if let Some(d) = line.strip_prefix("##") {
            doc.push(d.strip_prefix(' ').unwrap_or(d).to_string());
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            doc.clear();
            continue;
        }
        let Some((k, _)) = line.split_once('=') else {
            return Err(format!(
                "{name}: write one `Key = \"text\"` per line: {line}"
            ));
        };
        let k = k.trim();
        let Some(v) = table.get(k) else {
            return Err(format!(
                "{name}: write one `Key = \"text\"` per line: {line}"
            ));
        };
        let Some(t) = v.as_str() else {
            return Err(format!("{name}: {k} is not a string"));
        };
        out.push(Entry {
            key: k.to_string(),
            doc: std::mem::take(&mut doc),
            text: t.to_string(),
        });
    }
    if out.len() != table.len() {
        return Err(format!(
            "{name}: write one `Key = \"text\"` per line ({} read by lines, {} in the table)",
            out.len(),
            table.len()
        ));
    }
    Ok(out)
}

/// 文の差し込みの番号(`{0}` など)の組。
pub fn placeholders(s: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();
    let mut rest = s;
    while let Some(at) = rest.find('{') {
        let after = &rest[at + 1..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && after.as_bytes().get(digits) == Some(&b'}') {
            if let Ok(n) = after[..digits].parse() {
                out.insert(n);
            }
        }
        rest = after;
    }
    out
}

/// 文言の名前として使えるか(大文字で始まる英数字。`Msg` の値の名前になる)。
fn good_key(k: &str) -> bool {
    k.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && k.chars().all(|c| c.is_ascii_alphanumeric())
}

/// 正本の台帳(英語)そのものの誤り: 名前の形・重なり・空の文。
pub fn check_base(name: &str, base: &[Entry]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for e in base {
        if !good_key(&e.key) {
            out.push(format!(
                "{name}: {} is not a valid key (letters and digits, starting with a capital)",
                e.key
            ));
        }
        if !seen.insert(e.key.as_str()) {
            out.push(format!("{name}: {} appears twice", e.key));
        }
        if e.text.trim().is_empty() {
            out.push(format!("{name}: {} is empty", e.key));
        }
    }
    out
}

/// ほかの言語の台帳を正本と比べた誤り: 欠け・余り・空の文・差し込みの番号の食い違い。
pub fn check(base: &[Entry], name: &str, other: &[Entry]) -> Vec<String> {
    let mut out = Vec::new();
    let keys: HashSet<&str> = other.iter().map(|e| e.key.as_str()).collect();
    for e in base {
        if !keys.contains(e.key.as_str()) {
            out.push(format!("{name}: {} is missing", e.key));
        }
    }
    let base_keys: HashSet<&str> = base.iter().map(|e| e.key.as_str()).collect();
    let mut seen = HashSet::new();
    for e in other {
        if !base_keys.contains(e.key.as_str()) {
            out.push(format!("{name}: {} is not in the English ledger", e.key));
            continue;
        }
        if !seen.insert(e.key.as_str()) {
            out.push(format!("{name}: {} appears twice", e.key));
        }
        if e.text.trim().is_empty() {
            out.push(format!("{name}: {} is empty", e.key));
        }
        if let Some(b) = base.iter().find(|b| b.key == e.key) {
            let (want, got) = (placeholders(&b.text), placeholders(&e.text));
            if want != got {
                out.push(format!(
                    "{name}: {} uses placeholders {got:?}, but English uses {want:?}",
                    e.key
                ));
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "test_ledger_unit.rs"]
mod tests;
