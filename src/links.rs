//! ノートのリンクと被リンク(BV-22)。リンクの拾い方(`extract`)と、行き先を解いた索引(`Index`)の純関数。
//!
//! 拾うもの: `[[名前]]`・`[[名前|表示]]`・`[[名前#見出し]]`・`![[名前]]`・`[文字](相対のパス)`。コードの区画(``` と ~~~)と
//! インラインのコードの中は拾わない。外のアドレス(`://` を含む・`mailto:`)と見出しだけのリンクは拾わない。
//! 行き先の解き方(大文字小文字は区別しない): `[文字](…)` はノートのフォルダからの相対を先に見る。次に根からのパス
//! (`.md` を足して在ればそれ)。無ければ名前(最後の区切りの後ろ)が同じノートのうち、書いたパスで終わるもので、
//! 根に近い・パスの短いもの。解けなければ書いた文字のまま(被リンクには出ない)。
//! 索引の値はノートの根からのパスの拡張子 `.md` を除いたもの。1つのノートの同じ行き先は1つにまとめる。

use crate::source::markdown::{fence_close, fence_open};
use std::collections::{HashMap, HashSet};

/// 拾ったリンク1つ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Raw {
    /// 行き先の文字(表示・見出しを除いたもの。`[文字](…)` は `#…` を除き、%xx を戻したもの)。
    pub target: String,
    /// `[文字](…)` の形(ノートのフォルダからの相対を先に見る)。
    pub md: bool,
}

/// 本文(かフロントマターの値の文字)からリンクを拾う(書いた順)。
pub fn extract(text: &str) -> Vec<Raw> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    for line in text.lines() {
        if let Some(f) = fence {
            if fence_close(line, f) {
                fence = None;
            }
            continue;
        }
        if let Some(f) = fence_open(line) {
            fence = Some(f);
            continue;
        }
        line_links(line, &mut out);
    }
    out
}

/// 1行の中の「次の位置」の表。右から1回で作り、探すたびに行の終わりまで走らない
/// (細工した長い行で時間が2乗に延びない。links-stem-panic)。NONE は「無い」。
const NONE: usize = usize::MAX;

struct Next {
    /// i 以降で最初の `]]` の始まり。
    ww: Vec<usize>,
    /// i 以降で最初の `]`。
    rb: Vec<usize>,
    /// i 以降で最初の `)`。
    rp: Vec<usize>,
    /// i 以降で最初の `>`。
    gt: Vec<usize>,
    /// バッククォートの並びの始まり i → 同じ長さの次の並びの始まり(無ければ NONE)。
    tick: HashMap<usize, usize>,
}

impl Next {
    fn of(b: &[u8]) -> Next {
        let n = b.len();
        let mut ww = vec![NONE; n + 1];
        let mut rb = vec![NONE; n + 1];
        let mut rp = vec![NONE; n + 1];
        let mut gt = vec![NONE; n + 1];
        for i in (0..n).rev() {
            ww[i] = if b[i] == b']' && b.get(i + 1) == Some(&b']') {
                i
            } else {
                ww[i + 1]
            };
            rb[i] = if b[i] == b']' { i } else { rb[i + 1] };
            rp[i] = if b[i] == b')' { i } else { rp[i + 1] };
            gt[i] = if b[i] == b'>' { i } else { gt[i + 1] };
        }
        // バッククォートの並び(始まり, 長さ)を左から集め、右から「同じ長さの次」を結ぶ。
        let mut runs: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < n {
            if b[i] == b'`' {
                let len = b[i..].iter().take_while(|&&c| c == b'`').count();
                runs.push((i, len));
                i += len;
            } else {
                i += 1;
            }
        }
        let mut tick = HashMap::new();
        let mut seen: HashMap<usize, usize> = HashMap::new();
        for &(at, len) in runs.iter().rev() {
            tick.insert(at, seen.get(&len).copied().unwrap_or(NONE));
            seen.insert(len, at);
        }
        Next {
            ww,
            rb,
            rp,
            gt,
            tick,
        }
    }

    fn at(v: &[usize], i: usize) -> Option<usize> {
        v.get(i).copied().filter(|&j| j != NONE)
    }
}

fn line_links(line: &str, out: &mut Vec<Raw>) {
    let b = line.as_bytes();
    let next = Next::of(b);
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'`' => {
                let n = b[i..].iter().take_while(|&&c| c == b'`').count();
                // 同じ数のバッククォートで閉じる位置(閉じの後ろ)。無ければ並びの後ろへ進む。
                i = match next.tick.get(&i).copied().filter(|&j| j != NONE) {
                    Some(j) => j + n,
                    None => i + n,
                };
            }
            b'[' if b.get(i + 1) == Some(&b'[') => match Next::at(&next.ww, i + 2) {
                Some(j) => {
                    let inner = &line[i + 2..j];
                    let t = inner.split('|').next().unwrap_or("");
                    let t = t.split('#').next().unwrap_or("").trim();
                    if !t.is_empty() {
                        out.push(Raw {
                            target: t.to_string(),
                            md: false,
                        });
                    }
                    i = j + 2;
                }
                // 閉じの `]]` がこの先に無いので、もう `[[` のリンクは無い(`[` のリンクは見る)。
                None => i += 2,
            },
            b'[' => match md_link(line, i, &next) {
                Some((t, end)) => {
                    if let Some(t) = t {
                        out.push(Raw {
                            target: t,
                            md: true,
                        });
                    }
                    i = end;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
}

/// `[文字](行き先)` を `i`(`[` の位置)から読む: (拾う行き先, 読み終えた位置)。形でなければ None。
/// 位置は `next` の表で引く(切り出す位置はどれも ASCII の文字の位置なので、文字の境目)。
fn md_link(line: &str, i: usize, next: &Next) -> Option<(Option<String>, usize)> {
    let b = line.as_bytes();
    let close = Next::at(&next.rb, i + 1)?;
    if b.get(close + 1) != Some(&b'(') {
        return None;
    }
    let start = close + 2;
    let (dest, end) = if b.get(start) == Some(&b'<') {
        let j = Next::at(&next.gt, start + 1)?;
        let k = Next::at(&next.rp, j + 1)?;
        (line[start + 1..j].to_string(), k + 1)
    } else {
        let k = Next::at(&next.rp, start)?;
        let inside = line[start..k].trim();
        let d = inside.split([' ', '\t']).next().unwrap_or("");
        (d.to_string(), k + 1)
    };
    let dest = dest.split('#').next().unwrap_or("").trim().to_string();
    let lower = dest.to_lowercase();
    if dest.is_empty() || lower.contains("://") || lower.starts_with("mailto:") {
        return Some((None, end));
    }
    Some((Some(percent_decode(&dest)), end))
}

/// `%xx` を戻す(UTF-8 として読めなければ元の文字のまま)。
fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// 根からのパスの拡張子 `.md` を除いたもの(`.md` でなければそのまま)。
fn stem(rel: &str) -> &str {
    // 末尾の3バイトを ASCII として比べる(文字の途中で切らない。`.md` なら切る位置は文字の境目)。
    let b = rel.as_bytes();
    if b.len() > 3 && b[b.len() - 3..].eq_ignore_ascii_case(b".md") {
        &rel[..rel.len() - 3]
    } else {
        rel
    }
}

fn folder_of(p: &str) -> &str {
    p.rsplit_once('/').map_or("", |(f, _)| f)
}

/// `a/./b/../c` を `a/c` にする。根より上に出るなら None。
fn normalize(p: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for s in p.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

/// 根に近い・パスの短い順(同じなら文字の順)。
fn nearer(a: &str, b: &str) -> std::cmp::Ordering {
    let depth = |s: &str| s.matches('/').count();
    depth(a)
        .cmp(&depth(b))
        .then(a.len().cmp(&b.len()))
        .then(a.cmp(b))
}

/// 保管庫のリンクの索引(読み込みのたびに作り直す)。
#[derive(Debug, Default)]
pub struct Index {
    /// 小文字の stem → stem。
    paths: HashMap<String, String>,
    /// 小文字の名前(最後の区切りの後ろ)→ その名前の stem(根に近い順)。
    names: HashMap<String, Vec<String>>,
    /// ノートの根からのパス → 行き先(解いた stem か、解けない文字)。
    links: HashMap<String, Vec<String>>,
    /// 行き先の stem → 元の stem(文字の順)。
    backlinks: HashMap<String, Vec<String>>,
}

impl Index {
    /// (ノートの根からのパス, 拾ったリンク) の全部から作る。
    pub fn build(notes: Vec<(String, Vec<Raw>)>) -> Index {
        let mut ix = Index::default();
        for (rel, _) in &notes {
            let s = stem(rel).to_string();
            let name = s.rsplit('/').next().unwrap_or(&s).to_lowercase();
            ix.names.entry(name).or_default().push(s.clone());
            ix.paths.insert(s.to_lowercase(), s);
        }
        for v in ix.names.values_mut() {
            v.sort_by(|a, b| nearer(a, b));
        }
        let mut back: HashMap<String, Vec<String>> = HashMap::new();
        for (rel, raws) in notes {
            let from = stem(&rel).to_string();
            let mut seen = HashSet::new();
            let mut out = Vec::new();
            for r in &raws {
                let (v, resolved) = match ix.resolve(folder_of(&rel), r) {
                    Some(s) => (s, true),
                    None => (r.target.clone(), false),
                };
                if !seen.insert(v.clone()) {
                    continue;
                }
                if resolved {
                    back.entry(v.clone()).or_default().push(from.clone());
                }
                out.push(v);
            }
            ix.links.insert(rel, out);
        }
        for v in back.values_mut() {
            v.sort();
            v.dedup();
        }
        ix.backlinks = back;
        ix
    }

    /// stem が在ればそのノートの stem(書いた大文字小文字ではなく、ノートのもの)。
    fn exact(&self, p: &str) -> Option<String> {
        self.paths.get(&p.to_lowercase()).cloned()
    }

    /// リンクの行き先を解く(`folder` はリンクを書いたノートのフォルダ)。解けなければ None。
    pub fn resolve(&self, folder: &str, r: &Raw) -> Option<String> {
        let t = r.target.trim().trim_start_matches('/');
        if t.is_empty() {
            return None;
        }
        let t = stem(t);
        if r.md && !folder.is_empty() {
            if let Some(s) = normalize(&format!("{folder}/{t}")).and_then(|p| self.exact(&p)) {
                return Some(s);
            }
        }
        let t = normalize(t)?;
        if let Some(s) = self.exact(&t) {
            return Some(s);
        }
        let lower = t.to_lowercase();
        let name = lower.rsplit('/').next().unwrap_or(&lower);
        let tail = format!("/{lower}");
        self.names
            .get(name)?
            .iter()
            .find(|s| !lower.contains('/') || s.to_lowercase().ends_with(&tail))
            .cloned()
    }

    /// ノート(根からのパス)の行き先。索引に無いノートは空。
    pub fn links(&self, rel: &str) -> &[String] {
        self.links.get(rel).map_or(&[], Vec::as_slice)
    }

    /// ノート(根からのパス)を行き先に持つノートの stem。
    pub fn backlinks(&self, rel: &str) -> &[String] {
        self.backlinks.get(stem(rel)).map_or(&[], Vec::as_slice)
    }

    /// ノート(根からのパス)が `x`(ノートの名前・パス、か .md でないファイルのパス)を指すか。
    /// `x` がノートに解ければその stem と比べ、解けなければ解けないリンクの文字と、パスか名前で比べる。
    pub fn has_link(&self, rel: &str, x: &str) -> bool {
        let links = self.links(rel);
        if links.is_empty() {
            return false;
        }
        let raw = Raw {
            target: x.to_string(),
            md: false,
        };
        if let Some(s) = self.resolve("", &raw) {
            return links.contains(&s);
        }
        let x = x.trim().trim_start_matches('/').to_lowercase();
        !x.is_empty() && links.iter().any(|l| names_file(l, &x))
    }

    /// ノートの行き先。ただし `file`(.base など、ノートでないファイルの根からのパス)を指す解けないリンクは、
    /// `file` そのものにする(`file.links.contains(this.file)` が `[[Projects.base]]` でも当たるように。BV-22)。
    pub fn links_toward(&self, rel: &str, file: &str) -> Vec<String> {
        let x = file.trim_start_matches('/').to_lowercase();
        self.links(rel)
            .iter()
            .map(|l| {
                if !x.is_empty() && names_file(l, &x) {
                    file.to_string()
                } else {
                    l.clone()
                }
            })
            .collect()
    }
}

/// 解けないリンクの文字 `l` が、ファイル(小文字の根からのパス `x`)をパスか名前で指すか。
fn names_file(l: &str, x: &str) -> bool {
    let l = l.trim_start_matches('/').to_lowercase();
    !l.is_empty() && (l == x || x.ends_with(&format!("/{l}")))
}

#[cfg(test)]
#[path = "test_links_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_links_panic_unit.rs"]
mod test_links_panic_unit;
