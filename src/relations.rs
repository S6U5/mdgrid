//! リレーション(specs/relations/spec.md): フロントマターの値に書いたリンクを、表と表をつなぐ関係として扱う
//! 純関数と、フォルダを読む関数。行を見分けるキーはノートのファイル名(パス)。
//!
//! - 読み取り(REL-1): 値まるごとが `[[行き先]]`・`[[行き先|表示]]`・`[[行き先#見出し]]`・`[表示](パス)`、
//!   または `/` を含むか `.md` で終わるただのパス。外のアドレス(`://`・`mailto:`)は読まない。
//! - 解決(REL-1): 値を持つノートのフォルダからの相対 → 表の根からのパス → 名前(`[[…]]` だけ。links::Index と同じ
//!   根に近い・パスの短いもの)。ただのパスは実際にあるノートを指すときだけリンク。
//! - 書く形(REL-3): 列で使われている形に合わせる。
//! - つながった行(REL-5)・表どうしのつながり(REL-6): 表のフォルダを読み、フロントマターの値の行き先を見る。

use crate::frontmatter::{self, Value};
use crate::links::{Index, Raw};
use std::path::{Component, Path, PathBuf};

/// 1つの表のフォルダで読むノートの上限(大きすぎるフォルダで止まらないように)。
const MAX_NOTES: usize = 20_000;
/// 読むフォルダの深さの上限。
const MAX_DEPTH: usize = 12;

/// リンクの書き方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `[[行き先]]`。
    Wiki,
    /// `[表示](パス)`。
    Md,
    /// ただのパス。`ext` は `.md` を書いているか。
    Path { ext: bool },
}

/// 値から読んだリンク。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// 行き先の文字(見出し・表示を除いたもの)。
    pub target: String,
    /// `[[…|表示]]`・`[表示](…)` の表示の文字。
    pub alias: Option<String>,
    pub form: Form,
}

/// `%xx` を戻す(Markdown のリンクのパス)。
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        // バイトで見る(`%` のあとが多バイトの文字でも、文字の途中で切らない)。
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// 16進の1桁。
fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// 外のアドレスか。
fn external(t: &str) -> bool {
    t.contains("://") || t.to_ascii_lowercase().starts_with("mailto:")
}

/// 値まるごとがリンクの形なら読む(REL-1)。ただのパスは形だけを見る(在るかは `resolve` で)。
pub fn parse(value: &str) -> Option<Link> {
    let v = value.trim();
    if v.is_empty() || v.contains(['\n', '\r']) {
        return None;
    }
    let v = v
        .strip_prefix('!')
        .filter(|r| r.starts_with("[["))
        .unwrap_or(v);
    if let Some(inner) = v.strip_prefix("[[").and_then(|r| r.strip_suffix("]]")) {
        if inner.contains("[[") || inner.contains("]]") {
            return None;
        }
        let (body, alias) = match inner.split_once('|') {
            Some((b, a)) => (b, Some(a.trim().to_string()).filter(|a| !a.is_empty())),
            None => (inner, None),
        };
        let target = body.split('#').next().unwrap_or("").trim();
        if target.is_empty() || external(target) {
            return None;
        }
        return Some(Link {
            target: target.to_string(),
            alias,
            form: Form::Wiki,
        });
    }
    if let Some(rest) = v.strip_prefix('[') {
        let (text, after) = rest.split_once("](")?;
        let path = after.strip_suffix(')')?;
        let path = path.split('#').next().unwrap_or("").trim();
        if path.is_empty() || external(path) {
            return None;
        }
        return Some(Link {
            target: percent_decode(path),
            alias: Some(text.trim().to_string()).filter(|a| !a.is_empty()),
            form: Form::Md,
        });
    }
    if external(v) {
        return None;
    }
    let ext = v.to_ascii_lowercase().ends_with(".md");
    // 1語はパスとみなさない(`done` のような普通の値を誤ってリンクにしない)。
    if !v.contains('/') && !ext {
        return None;
    }
    Some(Link {
        target: v.to_string(),
        alias: None,
        form: Form::Path { ext },
    })
}

/// 表のフォルダのノートの名前の索引(`[[名前]]` を解く)。
pub struct Notes {
    /// 表の根(実体のパス)。
    pub root: PathBuf,
    index: Index,
    /// 根からのパス(`.md` 付き)。名前の順ではなく読んだ順。
    pub rels: Vec<String>,
}

/// フォルダの下の `.md` を根からのパスで並べる(名前が `.` で始まるフォルダとファイルは見ない)。
fn md_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            if out.len() >= MAX_NOTES {
                return out;
            }
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let p = e.path();
            let Ok(ft) = e.file_type() else {
                continue;
            };
            if ft.is_dir() {
                if depth < MAX_DEPTH {
                    stack.push((p, depth + 1));
                }
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md")) {
                if let Ok(rel) = p.strip_prefix(root) {
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }
    out
}

impl Notes {
    /// フォルダを読んで作る。
    pub fn scan(root: &Path) -> Notes {
        let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let rels = md_files(&root);
        let index = Index::build(rels.iter().map(|r| (r.clone(), Vec::new())).collect());
        Notes { root, index, rels }
    }

    /// 根からのパス(`.md` 付き)の並びから作る(画面が読み込んだノートから)。
    pub fn from_rels(root: &Path, rels: Vec<String>) -> Notes {
        let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let index = Index::build(rels.iter().map(|r| (r.clone(), Vec::new())).collect());
        Notes { root, index, rels }
    }

    /// 名前(か根からのパス)で解く。
    fn by_name(&self, target: &str) -> Option<PathBuf> {
        let raw = Raw {
            target: target.to_string(),
            md: false,
        };
        let stem = self.index.resolve("", &raw)?;
        Some(self.root.join(format!("{stem}.md")))
    }

    /// 同じ名前(拡張子なし。大文字小文字を問わない)のノートの数。
    pub fn same_name(&self, name: &str) -> usize {
        let n = name.to_lowercase();
        self.rels
            .iter()
            .filter(|r| file_stem(Path::new(r)).to_lowercase() == n)
            .count()
    }
}

/// 拡張子を除いたファイル名。
pub fn file_stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `base` に `rel` を足して、在る `.md` のノートなら実体のパス(`.md` が無ければ足して見る)。
fn existing(base: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.trim_start_matches('/');
    if rel.is_empty() {
        return None;
    }
    let p = base.join(rel);
    let with_md = if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md")) {
        p
    } else {
        let mut s = p.into_os_string();
        s.push(".md");
        PathBuf::from(s)
    };
    if with_md.is_file() {
        std::fs::canonicalize(&with_md).ok()
    } else {
        None
    }
}

/// リンクの行き先を解く(REL-1)。`note` は値を持つノートの実体のパス、`notes` は探す表(先頭が値を持つノートの表)。
/// 解けなければ None。
pub fn resolve(link: &Link, note: &Path, notes: &[&Notes]) -> Option<PathBuf> {
    let dir = note.parent().unwrap_or(Path::new(""));
    if let Some(p) = existing(dir, &link.target) {
        return Some(p);
    }
    for n in notes {
        if let Some(p) = existing(&n.root, &link.target) {
            return Some(p);
        }
    }
    if link.form == Form::Wiki {
        for n in notes {
            if let Some(p) = n.by_name(&link.target) {
                if p.is_file() {
                    return std::fs::canonicalize(&p).ok();
                }
            }
        }
    }
    None
}

/// セルに見せる名前(REL-2・REL-10): `[[…|表示]]` なら表示の文字、ほかは行き先のノートの名前(拡張子なし)。
/// 行き先が無ければ書いた行き先の文字のまま(`.md` は除く)。
pub fn display(link: &Link, target: Option<&Path>) -> String {
    if link.form == Form::Wiki {
        if let Some(a) = &link.alias {
            return a.clone();
        }
    }
    match target {
        Some(t) => file_stem(t),
        None => strip_md(&link.target).to_string(),
    }
}

/// 探す表の全部で、その名前(拡張子なし)のノートの数(同じパスは1つ)。
pub fn count_name(name: &str, notes: &[&Notes]) -> usize {
    let n = name.to_lowercase();
    let mut seen: Vec<PathBuf> = Vec::new();
    for x in notes {
        for r in &x.rels {
            if file_stem(Path::new(r)).to_lowercase() == n {
                let p = x.root.join(r);
                if !seen.contains(&p) {
                    seen.push(p);
                }
            }
        }
    }
    seen.len()
}

/// `from`(フォルダ)から `to`(ファイル)への相対パス(`/` 区切り)。
pub fn relative(from: &Path, to: &Path) -> String {
    let a: Vec<Component> = from.components().collect();
    let b: Vec<Component> = to.components().collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..a.len() {
        parts.push("..".into());
    }
    for c in &b[common..] {
        parts.push(c.as_os_str().to_string_lossy().into_owned());
    }
    parts.join("/")
}

/// `.md` を除く。
fn strip_md(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() > 3 && b[b.len() - 3..].eq_ignore_ascii_case(b".md") {
        &s[..s.len() - 3]
    } else {
        s
    }
}

/// 書く値(REL-3): `form` の形で、`note`(書くノート)から `target` を指す。`unique` は、探す表の全部で
/// 行き先の名前が1つだけか。`[[…]]` は、名前が1つならその名前、重なるならノートのフォルダからの相対の
/// パス(解くときに最初に見るので、別の表の同じ名前に取り違えない)。
pub fn format(form: Form, target: &Path, note: &Path, unique: bool) -> String {
    let dir = note.parent().unwrap_or(Path::new(""));
    let name = file_stem(target);
    match form {
        Form::Wiki => {
            let inner = if unique {
                name
            } else {
                strip_md(&relative(dir, target)).to_string()
            };
            format!("[[{inner}]]")
        }
        Form::Md => format!("[{name}]({})", relative(dir, target).replace(' ', "%20")),
        Form::Path { ext } => {
            let r = relative(dir, target);
            if ext {
                r
            } else {
                strip_md(&r).to_string()
            }
        }
    }
}

/// 列の値の文字(リストは要素)。
pub fn strings(v: &Value) -> Vec<&str> {
    match v {
        Value::Str(s) => vec![s.as_str()],
        Value::List(items) => items
            .iter()
            .filter_map(|x| match x {
                Value::Str(s) => Some(s.as_str()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// リンクの列か(REL-3。空でない値の過半数がリンク)。そうなら、いちばん多い書き方と行き先のフォルダ
/// (いちばん多いもの。解けたリンクが無ければ None)。`values` は (値を持つノート, 値)。
pub fn link_column(
    values: &[(PathBuf, Value)],
    notes: &[&Notes],
) -> Option<(Form, Option<PathBuf>)> {
    let mut total = 0usize;
    let mut links = 0usize;
    let mut forms: Vec<(Form, usize)> = Vec::new();
    let mut dirs: Vec<(PathBuf, usize)> = Vec::new();
    for (note, v) in values {
        match v {
            Value::Null => continue,
            Value::Str(s) if s.trim().is_empty() => continue,
            _ => {}
        }
        total += 1;
        let mut is_link = false;
        for s in strings(v) {
            let Some(l) = parse(s) else {
                continue;
            };
            match resolve(&l, note, notes) {
                Some(t) => {
                    is_link = true;
                    bump(&mut forms, l.form);
                    if let Some(d) = t.parent() {
                        bump(&mut dirs, d.to_path_buf());
                    }
                }
                // 行き先の無い `[[…]]`・`[…](…)` もリンク(REL-10)。ただのパスは在るときだけ。
                None if matches!(l.form, Form::Wiki | Form::Md) => {
                    is_link = true;
                    bump(&mut forms, l.form);
                }
                None => {}
            }
        }
        if is_link {
            links += 1;
        }
    }
    if total == 0 || links * 2 <= total {
        return None;
    }
    let form = forms.iter().max_by_key(|(_, n)| *n).map(|(f, _)| *f)?;
    let dir = dirs.iter().max_by_key(|(_, n)| *n).map(|(d, _)| d.clone());
    Some((form, dir))
}

fn bump<T: PartialEq>(v: &mut Vec<(T, usize)>, k: T) {
    match v.iter_mut().find(|(x, _)| *x == k) {
        Some((_, n)) => *n += 1,
        None => v.push((k, 1)),
    }
}

/// フォルダ(直下)の `.md` のノート(実体のパス。名前の順)。リンクの列の候補(REL-3)。
pub fn notes_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"))
                && !p
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
        })
        .filter_map(|p| std::fs::canonicalize(p).ok())
        .collect();
    out.sort_by_key(|p| file_stem(p).to_lowercase());
    out
}

/// ワークスペースの表(名前と、ノートのフォルダ)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub dir: PathBuf,
}

impl Table {
    /// 登録した表から。`.base` は、開いたときと同じく保管庫の根(`.obsidian` のある最寄りの上のフォルダ。
    /// 無ければ `.base` のフォルダ)を表のフォルダにする(REL-1: `.base` もフォルダも同じに扱う)。
    pub fn new(name: &str, path: &Path) -> Table {
        let dir = if path.extension().is_some_and(|x| x == "base") {
            let here = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let here = std::fs::canonicalize(&here).unwrap_or(here);
            here.ancestors()
                .find(|a| a.join(".obsidian").is_dir())
                .map(Path::to_path_buf)
                .unwrap_or(here)
        } else {
            path.to_path_buf()
        };
        let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
        Table {
            name: name.to_string(),
            dir,
        }
    }
}

/// ノートを含む表(フォルダのいちばん深いもの)。
pub fn table_of<'a>(note: &Path, tables: &'a [Table]) -> Option<&'a Table> {
    tables
        .iter()
        .filter(|t| note.starts_with(&t.dir))
        .max_by_key(|t| t.dir.components().count())
}

/// フロントマターの (列, 値) を読む。読めなければ空。
fn entries(path: &Path) -> Vec<(String, Value)> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    match frontmatter::parse(&bytes) {
        Ok(fm) => fm.entries.into_iter().map(|e| (e.key, e.value)).collect(),
        Err(_) => Vec::new(),
    }
}

/// つながった行の1つ(REL-5)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backlink {
    /// 指しているノート(実体のパス)。
    pub note: PathBuf,
    /// そのノートの表の名前(どの表にも無ければ None)。
    pub table: Option<String>,
    /// 指している列。
    pub column: String,
}

/// `target` を指しているノートを、表のフォルダから探す(REL-5)。同じノート・列は1つ。
pub fn backlinks(target: &Path, tables: &[Table]) -> Vec<Backlink> {
    let target = std::fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    let scanned: Vec<Notes> = dedup_dirs(tables).iter().map(|d| Notes::scan(d)).collect();
    let all: Vec<&Notes> = scanned.iter().collect();
    let mut out: Vec<Backlink> = Vec::new();
    for n in &scanned {
        for rel in &n.rels {
            let note = n.root.join(rel);
            if note == target {
                continue;
            }
            let mut order: Vec<&Notes> = vec![n];
            order.extend(all.iter().filter(|x| !std::ptr::eq(**x, n)));
            for (col, v) in entries(&note) {
                let hit = strings(&v).iter().any(|s| {
                    parse(s).and_then(|l| resolve(&l, &note, &order)).as_deref() == Some(&target)
                });
                if hit && !out.iter().any(|b| b.note == note && b.column == col) {
                    out.push(Backlink {
                        table: table_of(&note, tables).map(|t| t.name.clone()),
                        note: note.clone(),
                        column: col,
                    });
                }
            }
        }
    }
    out
}

/// 同じフォルダ・中に含まれるフォルダを1つにする(外側を残す)。
fn dedup_dirs(tables: &[Table]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = tables.iter().map(|t| t.dir.clone()).collect();
    dirs.sort_by_key(|d| d.components().count());
    let mut out: Vec<PathBuf> = Vec::new();
    for d in dirs {
        if !out.iter().any(|o| d.starts_with(o)) {
            out.push(d);
        }
    }
    out
}

/// 表どうしのつながりの1つ(REL-6)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub column: String,
    pub to: String,
    /// リストの列(多対多)。そうでなければ多対一。
    pub many: bool,
}

/// 表のフォルダを読み、列の値の行き先が別の表(同じ表も)のノートなら、その列をつながりとみなす(REL-6)。
/// どの表にも無い行き先は数えない(リンクとしては扱う)。
pub fn edges(tables: &[Table]) -> Vec<Edge> {
    let scanned: Vec<(usize, Notes)> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| (i, Notes::scan(&t.dir)))
        .collect();
    let all: Vec<&Notes> = scanned.iter().map(|(_, n)| n).collect();
    let mut out: Vec<Edge> = Vec::new();
    for (i, n) in &scanned {
        let from = &tables[*i];
        for rel in &n.rels {
            let note = n.root.join(rel);
            // ほかの表のフォルダの中のノートは、その表で数える。
            if table_of(&note, tables).map(|t| &t.name) != Some(&from.name) {
                continue;
            }
            let mut order: Vec<&Notes> = vec![n];
            order.extend(all.iter().filter(|x| !std::ptr::eq(**x, n)));
            for (col, v) in entries(&note) {
                let many = matches!(v, Value::List(_));
                for s in strings(&v) {
                    let Some(t) = parse(s).and_then(|l| resolve(&l, &note, &order)) else {
                        continue;
                    };
                    let Some(to) = table_of(&t, tables) else {
                        continue;
                    };
                    match out
                        .iter_mut()
                        .find(|e| e.from == from.name && e.column == col && e.to == to.name)
                    {
                        Some(e) => e.many |= many,
                        None => out.push(Edge {
                            from: from.name.clone(),
                            column: col.clone(),
                            to: to.name.clone(),
                            many,
                        }),
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "test_relations_unit.rs"]
mod tests;
