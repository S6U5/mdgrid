//! お気に入りのワークスペース(workspace.favorite-*): 設定のフォルダの `workspaces.toml` の最上位の `favorites` に、
//! 入れた順で持つ。`workspaces.toml` のワークスペースは名前、印のワークスペース(workspace.marker)は根のパス
//! (ホームの下なら `~/…`)。人ごとの好みなので、ノートのフォルダと印には書かない(workspace.favorite-store)。
//!
//! 書き換えは `workspaces.toml` の文字のまま、最上位の `favorites` の行だけを置き換える(手で書いたコメント・
//! ワークスペースの区画は残す。workspace.rs の書き換えと同じ考え)。画面の切り替えの一覧(ui/switch.rs)と
//! 引き出しの見出しの印(ui/drawer.rs)が読む。

use crate::config;
use crate::i18n::Msg;
use crate::places::expand_home;
use crate::workspace::{self, Workspace, FILE_NAME, MARKER_DIR, MARKER_FILE};
use std::io;
use std::path::{Path, PathBuf};

/// 最上位の鍵の名前。
const KEY: &str = "favorites";

/// お気に入りのワークスペース1つ(読み解いたもの)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Favorite {
    /// `workspaces.toml` のワークスペース(名前)。
    App(String),
    /// 印のワークスペース: 根のフォルダ(実体のパス)と、印の名前(無ければフォルダの名前)。
    Marker { root: PathBuf, name: String },
}

impl Favorite {
    /// 一覧に見せる名前。
    pub fn name(&self) -> &str {
        match self {
            Favorite::App(n) | Favorite::Marker { name: n, .. } => n,
        }
    }
}

/// `workspaces.toml` の文から、最上位の `favorites` の並び(文字のまま)と、読めない項目の警告。
fn parse(text: &str) -> (Vec<String>, Vec<String>) {
    let mut warns = Vec::new();
    let Ok(table) = text.parse::<toml::Table>() else {
        // 壊れた TOML はワークスペースを読むところ(workspace::load)が知らせる。
        return (Vec::new(), warns);
    };
    let Some(v) = table.get(KEY) else {
        return (Vec::new(), warns);
    };
    let Some(items) = v.as_array() else {
        crate::profile::warn(
            &mut warns,
            FILE_NAME,
            KEY,
            crate::profile::not(Msg::WantStringList),
        );
        return (Vec::new(), warns);
    };
    let mut out = Vec::new();
    for (k, item) in items.iter().enumerate() {
        match item.as_str().map(str::trim) {
            Some(s) if !s.is_empty() => out.push(s.to_string()),
            _ => crate::profile::warn(
                &mut warns,
                FILE_NAME,
                &format!("{KEY}[{}]", k + 1),
                crate::profile::not(Msg::WantString),
            ),
        }
    }
    (out, warns)
}

/// 書いてある並び(文字のまま。読めない項目は飛ばす)。ファイルが無ければ空。
pub fn entries(dir: &Path) -> Vec<String> {
    std::fs::read_to_string(dir.join(FILE_NAME))
        .map(|t| parse(&t).0)
        .unwrap_or_default()
}

/// 印の根(ワークスペースの印 `.mdgrid/workspace.toml` を持つフォルダ)か。
fn is_marker_root(p: &Path) -> bool {
    p.join(MARKER_DIR).join(MARKER_FILE).is_file()
}

/// 書いてある並びを、入れた順のワークスペースに読み解く(workspace.favorite-store)。`apps` は `workspaces.toml` の
/// ワークスペース。名前が `apps` に無く、印の根でもない項目(消したワークスペース・無くなった根)は、
/// 一覧から外して警告にする(cli.config-file-warning-form)。
pub fn resolve(dir: &Path, apps: &[Workspace]) -> (Vec<Favorite>, Vec<String>) {
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap_or_default();
    let (list, mut warns) = parse(&text);
    let mut out: Vec<Favorite> = Vec::new();
    for (k, e) in list.iter().enumerate() {
        let fav = if apps.iter().any(|w| w.name == *e) {
            Some(Favorite::App(e.clone()))
        } else {
            as_path(e).filter(|p| is_marker_root(p)).map(|p| {
                let root = std::fs::canonicalize(&p).unwrap_or(p);
                let name = workspace::read_marker(&root).0.name;
                Favorite::Marker { root, name }
            })
        };
        match fav {
            Some(f) if !out.contains(&f) => out.push(f),
            Some(_) => {}
            None => crate::profile::warn(
                &mut warns,
                FILE_NAME,
                &format!("{KEY}[{}]", k + 1),
                Msg::RsnFavoriteGone.fill(&[e]),
            ),
        }
    }
    (out, warns)
}

/// 印のワークスペースを書く形(根のパス。ホームの下なら `~/…`)。
pub fn marker_entry(root: &Path) -> String {
    if let Some(home) = config::home_dir() {
        let home = std::fs::canonicalize(&home).unwrap_or(home);
        if let Ok(rest) = root.strip_prefix(&home) {
            if !rest.as_os_str().is_empty() {
                return format!("~/{}", rest.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    root.to_string_lossy().into_owned()
}

/// 項目がパス(印の根)の形なら、そのパス(`~` で始まるか絶対パス)。ほかは名前。
fn as_path(e: &str) -> Option<PathBuf> {
    (e.starts_with('~') || Path::new(e).is_absolute()).then(|| expand_home(e))
}

/// 2つの項目が同じワークスペースを指すか(同じ文字か、同じ実体のフォルダ)。
pub fn same(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    match (as_path(a), as_path(b)) {
        (Some(pa), Some(pb)) => pa.exists() && pb.exists() && workspace::same_path(&pa, &pb),
        _ => false,
    }
}

/// 項目 `entry`(名前か `marker_entry` の形)が入っているか。
pub fn contains(list: &[String], entry: &str) -> bool {
    list.iter().any(|e| same(e, entry))
}

/// お気に入りに入れる(`on`。末尾に足す。もう入っていれば何もしない)か外す(workspace.favorite-mark)。
/// `workspaces.toml` の最上位の `favorites` の行だけを書き換える。ファイルが読めない TOML なら書かない。
pub fn set(dir: &Path, entry: &str, on: bool) -> io::Result<()> {
    let path = dir.join(FILE_NAME);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if let Err(e) = text.parse::<toml::Table>() {
        let (line, msg) = config::toml_error(&text, &e);
        let at = line.map(|l| l.to_string()).unwrap_or_default();
        let why = Msg::WsBadToml.fill(&[&FILE_NAME, &at, &msg]);
        return Err(io::Error::other(Msg::WsFixFirst.fill(&[&why])));
    }
    let mut list = parse(&text).0;
    let had = contains(&list, entry);
    match (on, had) {
        (true, false) => list.push(entry.to_string()),
        (false, true) => list.retain(|e| !same(e, entry)),
        _ => return Ok(()),
    }
    let new = edited(&text, &list);
    config::write_atomic(dir, FILE_NAME, new.as_bytes())
}

/// `favorites = [...]` の1行(並びが空なら None)。
fn line(list: &[String]) -> Option<String> {
    if list.is_empty() {
        return None;
    }
    let items: Vec<String> = list
        .iter()
        .map(|s| toml::Value::String(s.clone()).to_string())
        .collect();
    Some(format!("{KEY} = [{}]\n", items.join(", ")))
}

/// 文の最上位の `favorites` の行(無ければ None)。workspace.rs が書き直しの文に引き継ぐのに使う。
pub(crate) fn line_of(text: &str) -> Option<String> {
    line(&parse(text).0)
}

/// 行が最上位の鍵 `key` の始まりか(`key =`・`"key" =`)。
fn starts_key(l: &str, key: &str) -> bool {
    let t = l.trim_start();
    let rest = t
        .strip_prefix(key)
        .or_else(|| t.strip_prefix(&format!("\"{key}\"")));
    rest.is_some_and(|r| r.trim_start().starts_with('='))
}

/// 最上位の `favorites` を `list` にした文。区画の見出しより前の `favorites` の行(複数行の配列も)を置き換え、
/// 無ければ先頭に足す。結果を読み直して、狙った並びと、ワークスペースの並びが前と同じでなければ(手で書いた形を
/// 読み違えたとき)、並びから書き直す(コメントは消えるが、ほかの項目を壊さない)。
fn edited(text: &str, list: &[String]) -> String {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let first_head = lines
        .iter()
        .position(|l| config::toml_header(l).is_some())
        .unwrap_or(lines.len());
    let new_line = line(list);
    let mut out: Vec<String> = Vec::new();
    if let Some(i) = (0..first_head).find(|&i| starts_key(lines[i], KEY)) {
        // 配列が閉じる行まで(その範囲だけで TOML として読める最初の行)。
        let end = (i..first_head)
            .find(|&j| {
                lines[i..=j]
                    .concat()
                    .parse::<toml::Table>()
                    .is_ok_and(|t| t.contains_key(KEY))
            })
            .unwrap_or(i);
        out.extend(lines[..i].iter().map(|s| (*s).to_string()));
        out.extend(new_line.clone());
        out.extend(lines[end + 1..].iter().map(|s| (*s).to_string()));
    } else {
        if let Some(l) = &new_line {
            out.push(l.clone());
            if !text.trim().is_empty() {
                out.push("\n".into());
            }
        }
        out.extend(lines.iter().map(|s| (*s).to_string()));
        if let Some(last) = out.last_mut() {
            if !last.ends_with('\n') && !last.is_empty() {
                last.push('\n');
            }
        }
    }
    let new = out.concat();
    let (before, _) = workspace::parse(text);
    let (after, _) = workspace::parse(&new);
    if parse(&new).0 == list && before == after {
        new
    } else {
        let mut s = new_line.map(|l| l + "\n").unwrap_or_default();
        s.push_str(&workspace::to_toml(&before));
        s
    }
}

#[cfg(test)]
#[path = "test_favorites_unit.rs"]
mod tests;
