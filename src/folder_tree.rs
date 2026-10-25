//! 木の画面(specs/tree/spec.md。TR-2〜TR-5)の、画面に依存しない部分: 階層の列の設定、値を区切りで切る、
//! フォルダの木を組む(途中の階層も作り、下の階層までの行の数を添え、名前の順に並べる)、開いているフォルダだけを
//! 前順に並べる、フォルダの直下の行の並び(order の列)、開いてよい URL(http・https だけ)。
//!
//! NV-27 の親子の並べ方(src/tree.rs)とは別のもの。あちらは行どうしの親子、こちらは値の文字の階層。

use std::collections::{HashMap, HashSet};

/// 区切りを書かないときの区切り(TR-2)。
pub const DEFAULT_SEPARATOR: &str = " > ";

/// 印の `[[table]]` の `tree = { path, separator, order }`(TR-2)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSpec {
    /// 階層を持つ列。
    pub path: String,
    /// 階層の区切り(空でない)。
    pub separator: String,
    /// フォルダの中の並びの列(無ければノートの名前の順)。
    pub order: Option<String>,
}

/// 値を区切りで切った階層。前後の空白を除き、空の段は捨てる。区切りが空なら切らない(空の階層)。
pub fn split(value: &str, sep: &str) -> Vec<String> {
    if sep.is_empty() {
        return Vec::new();
    }
    value
        .split(sep)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// 木の1つのフォルダ。
#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    /// 見せる名前(「(なし)」のフォルダは呼ぶ側が渡した名前)。
    pub name: String,
    /// 根からの道(開閉と選びを、組み直しても保つ鍵)。「(なし)」は `NONE_KEY`。
    pub key: String,
    pub parent: Option<usize>,
    /// 子のフォルダ(名前の順)。
    pub kids: Vec<usize>,
    /// 直下の行(`build` に渡した添字。order の並び)。
    pub rows: Vec<usize>,
    /// 下の階層も含めた行の数。
    pub count: usize,
    /// 根は 0。
    pub depth: usize,
    /// 階層の列が空の行をまとめた「(なし)」か。
    pub none: bool,
}

/// 「(なし)」のフォルダの鍵(道の字に出ない字)。
pub const NONE_KEY: &str = "\u{0}";

/// フォルダの木。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tree {
    pub folders: Vec<Folder>,
    /// 根のフォルダ(名前の順。「(なし)」は最後)。
    pub roots: Vec<usize>,
}

/// 木に入れる1行: 階層・order の値・ノートの名前。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: Vec<String>,
    pub order: String,
    pub name: String,
}

/// 道の段をつないだ鍵。
fn key_of(parts: &[String]) -> String {
    parts.join("\u{1f}")
}

/// フォルダの名前の順(大文字と小文字を区別せずに比べ、同じなら元の字の順)。
fn by_name(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

/// order の値の並びの鍵: 数として読めるものが先(小さい順)、読めない値と空は後ろ。
fn order_key(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|f| f.is_finite())
}

/// 2つの行の並び(TR-4): order の小さい順、数でない・空は後ろ、同じならノートの名前の順。
pub fn row_order(a: &Entry, b: &Entry) -> std::cmp::Ordering {
    match (order_key(&a.order), order_key(&b.order)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
    .then_with(|| by_name(&a.name, &b.name))
}

/// 行の並びから木を組む(TR-3・TR-4)。途中の階層もフォルダにし、空の階層の行は根の「(なし)」(名前 `none_name`)へ。
/// 同じ名前でも親が違えば別のフォルダ。各フォルダの直下の行は `row_order` の順。
pub fn build(entries: &[Entry], none_name: &str) -> Tree {
    let mut folders: Vec<Folder> = Vec::new();
    let mut at: HashMap<String, usize> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();
    let mut none: Option<usize> = None;
    for (i, e) in entries.iter().enumerate() {
        let leaf = if e.path.is_empty() {
            *none.get_or_insert_with(|| {
                folders.push(Folder {
                    name: none_name.to_string(),
                    key: NONE_KEY.to_string(),
                    parent: None,
                    kids: Vec::new(),
                    rows: Vec::new(),
                    count: 0,
                    depth: 0,
                    none: true,
                });
                folders.len() - 1
            })
        } else {
            let mut parent: Option<usize> = None;
            for d in 0..e.path.len() {
                let key = key_of(&e.path[..=d]);
                let f = match at.get(&key) {
                    Some(&f) => f,
                    None => {
                        folders.push(Folder {
                            name: e.path[d].clone(),
                            key: key.clone(),
                            parent,
                            kids: Vec::new(),
                            rows: Vec::new(),
                            count: 0,
                            depth: d,
                            none: false,
                        });
                        let f = folders.len() - 1;
                        at.insert(key, f);
                        match parent {
                            Some(p) => folders[p].kids.push(f),
                            None => roots.push(f),
                        }
                        f
                    }
                };
                parent = Some(f);
            }
            parent.expect("a non-empty path has a folder")
        };
        folders[leaf].rows.push(i);
        let mut up = Some(leaf);
        while let Some(f) = up {
            folders[f].count += 1;
            up = folders[f].parent;
        }
    }
    let names: Vec<String> = folders.iter().map(|f| f.name.clone()).collect();
    let sort = |v: &mut Vec<usize>| v.sort_by(|&a, &b| by_name(&names[a], &names[b]));
    sort(&mut roots);
    for f in &mut folders {
        sort(&mut f.kids);
        f.rows.sort_by(|&a, &b| row_order(&entries[a], &entries[b]));
    }
    roots.extend(none);
    Tree { folders, roots }
}

impl Tree {
    /// 開いているフォルダだけを前順に(閉じたフォルダの下は出さない)。`folded` は閉じたフォルダの鍵。
    pub fn shown(&self, folded: &HashSet<String>) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self.roots.iter().rev().copied().collect();
        while let Some(f) = stack.pop() {
            out.push(f);
            if !folded.contains(&self.folders[f].key) {
                stack.extend(self.folders[f].kids.iter().rev());
            }
        }
        out
    }

    /// 鍵のフォルダ。
    pub fn find(&self, key: &str) -> Option<usize> {
        self.folders.iter().position(|f| f.key == key)
    }
}

/// 開いてよい URL か(TR-5): 前後の空白を除いて `http://`・`https://` で始まり(大文字と小文字を区別しない)、
/// その後ろに字があり、空白と制御文字を含まないもの。ほかの形(file:・javascript: など)は None。
pub fn web_url(s: &str) -> Option<&str> {
    let t = s.trim();
    let lower = t.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    if rest.is_empty() || t.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return None;
    }
    Some(t)
}

#[cfg(test)]
#[path = "test_folder_tree_unit.rs"]
mod tests;
