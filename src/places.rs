//! よく使う表の登録(CLI-18・CLI-19): 設定のフォルダの `places.toml` に、名前・1段の分類・パス(フォルダか
//! `.base`)・ビューの並びを持つ。画面の「この表を登録」が書き、引数なしの起動とパレットの一覧が読む。

use crate::config;
use crate::i18n::Msg;
use std::io;
use std::path::{Path, PathBuf};

/// 登録のファイルの名前(views.toml と同じ設定のフォルダに置く)。
pub const FILE_NAME: &str = "places.toml";

/// 登録した表1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub name: String,
    /// 1段の分類(空なら分類なし)。
    pub group: String,
    /// フォルダか `.base` のパス(先頭の `~` はホームのフォルダに読み替え済み)。
    pub path: PathBuf,
    /// 開くビューの名前(`.base` か mdgrid のビュー)。
    pub view: Option<String>,
}

/// 先頭の `~` をホームのフォルダにする(places.toml と、書き出しで打った名前)。
pub fn expand_home(s: &str) -> PathBuf {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    if s == "~" {
        if let Some(h) = home() {
            return h;
        }
    }
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(h) = home() {
            return h.join(rest);
        }
    }
    PathBuf::from(s)
}

/// 設定のフォルダの places.toml を読む。無ければ空。読めない行は警告にして飛ばす(CLI-18)。
pub fn load(dir: &Path) -> (Vec<Place>, Vec<String>) {
    let path = dir.join(FILE_NAME);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (Vec::new(), Vec::new());
    };
    parse(&text)
}

/// places.toml の文を読む。
pub fn parse(text: &str) -> (Vec<Place>, Vec<String>) {
    let mut warns = Vec::new();
    let table: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            let (line, msg) = config::toml_error(text, &e);
            let at = line.map(|l| l.to_string()).unwrap_or_default();
            warns.push(Msg::PlacesBadToml.fill(&[&at, &msg]));
            return (Vec::new(), warns);
        }
    };
    let mut out: Vec<Place> = Vec::new();
    let items = table
        .get("place")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for (k, item) in items.iter().enumerate() {
        let n = k + 1;
        let Some(t) = item.as_table() else {
            crate::profile::warn(
                &mut warns,
                FILE_NAME,
                &format!("place[{n}]"),
                Msg::RsnNeedNamePath.text().to_string(),
            );
            continue;
        };
        let text = |key: &str| t.get(key).and_then(|v| v.as_str()).map(str::to_string);
        let (Some(name), Some(path)) = (text("name"), text("path")) else {
            crate::profile::warn(
                &mut warns,
                FILE_NAME,
                &format!("place[{n}]"),
                Msg::RsnNeedNamePath.text().to_string(),
            );
            continue;
        };
        let name = name.trim().to_string();
        if name.is_empty() || out.iter().any(|p| p.name == name) {
            crate::profile::warn(
                &mut warns,
                FILE_NAME,
                &format!("place[{n}]"),
                Msg::RsnNeedNamePath.text().to_string(),
            );
            continue;
        }
        out.push(Place {
            name,
            group: text("group").unwrap_or_default().trim().to_string(),
            path: expand_home(path.trim()),
            view: text("view").filter(|v| !v.trim().is_empty()),
        });
    }
    (out, warns)
}

/// places.toml の文にする(並びのまま)。
pub fn to_toml(places: &[Place]) -> String {
    let arr: Vec<toml::Value> = places
        .iter()
        .map(|p| {
            let mut t = toml::Table::new();
            t.insert("name".into(), toml::Value::String(p.name.clone()));
            if !p.group.is_empty() {
                t.insert("group".into(), toml::Value::String(p.group.clone()));
            }
            t.insert(
                "path".into(),
                toml::Value::String(p.path.to_string_lossy().into_owned()),
            );
            if let Some(v) = &p.view {
                t.insert("view".into(), toml::Value::String(v.clone()));
            }
            toml::Value::Table(t)
        })
        .collect();
    let mut root = toml::Table::new();
    root.insert("place".into(), toml::Value::Array(arr));
    toml::to_string(&root).unwrap_or_default()
}

/// 1つ登録する。同じ名前があれば置き換え、無ければ後ろに足す。ファイルの文字を区画ごとに扱うので、
/// 手で書いたコメント・空行・`~` のパス・読めなかった行は文字のまま残る。
pub fn save(dir: &Path, place: Place) -> io::Result<()> {
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap_or_default();
    config::write_atomic(dir, FILE_NAME, upsert(&text, &place).as_bytes())
}

/// `[[place]]` の区画の置き換えか足し(save の本体)。書き換えた結果を読み直し、狙った並びと違えば
/// (手で書いた形を読み違えたとき)、並びから書き直す(コメントは消えるが、ほかの登録を壊さない)。
pub fn upsert(text: &str, place: &Place) -> String {
    let (mut want, _) = parse(text);
    match want.iter_mut().find(|p| p.name == place.name) {
        Some(p) => *p = place.clone(),
        None => want.push(place.clone()),
    }
    let new = upsert_text(text, place);
    let (got, _) = parse(&new);
    if got == want {
        new
    } else {
        to_toml(&want)
    }
}

fn upsert_text(text: &str, place: &Place) -> String {
    let block = to_toml(std::slice::from_ref(place));
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    for (start, body_end) in config::toml_blocks(&lines, 0, lines.len(), "[[place]]", &|_| false) {
        let body: String = lines[start..body_end].concat();
        let (found, _) = parse(&body);
        if found.first().is_some_and(|p| p.name == place.name) {
            let mut out: String = lines[..start].concat();
            out.push_str(&block);
            out.push_str(&lines[body_end..].concat());
            return out;
        }
    }
    let mut out = text.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&block);
    out
}

/// 一覧の並び(CLI-19): 分類が最初に出た順にまとめ、分類の中は places.toml の順。添字の並びを返す。
pub fn grouped(places: &[Place]) -> Vec<usize> {
    let mut groups: Vec<&str> = Vec::new();
    for p in places {
        if !groups.contains(&p.group.as_str()) {
            groups.push(&p.group);
        }
    }
    groups
        .iter()
        .flat_map(|g| {
            places
                .iter()
                .enumerate()
                .filter(move |(_, p)| p.group == *g)
                .map(|(i, _)| i)
        })
        .collect()
}

/// 分類の並び(出た順)。
pub fn groups(places: &[Place]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in places {
        if !p.group.is_empty() && !out.contains(&p.group) {
            out.push(p.group.clone());
        }
    }
    out
}

#[cfg(test)]
#[path = "test_places_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_places_keep_unit.rs"]
mod tests_keep;
