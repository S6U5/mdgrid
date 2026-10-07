//! 最初の読み込み口: フロントマター付き Markdown のフォルダ(SC-14)。
//!
//! 行は BV-1 の既定の表: 渡したフォルダの下のノートだけ(根が渡したフォルダより上でも、行は渡したフォルダの下に限る)。
//! 下かどうかは、見つけた道筋のパス(リンクを辿る前)で見る。重複の除去は実体のパスのまま(vault)。
//! RowId は実体のパスの文字列。UTF-8 でないパスのノートは行に入れない。
//! Cell.lock: ReadOnly の各理由(フロントマターが無い・空のフロントマターを除く)とハードリンク・書き込めない権限(WB-5)・span の無い値(CE-8)・
//! 文字列でない要素や入れ子の要素を含むリスト(CE-8)・`file.*` と `formula.*` の列(CE-8)・
//! 根ごとの types.json で型が食い違う列(BV-12)。書ける形のリストは lock なし(CE-16)。
//! フロントマターのあるノートでキーが無いセルと、フロントマターの無いノート(書くと先頭に足す)・空のフロントマターの
//! ノート(区切りの間に足す)のセル(WB-3)は lock なし。ただし設定 add_frontmatter = false ではこの2つを読むだけにする。
//! 列の型(CE-2)と候補(CE-3)は、行に限らず根の下の読んだノート全部(BV-2 の範囲)から、パスの昇順で見る。

use super::{Cell, ColumnKind, Edit, EditError, FileInfo, RowId, SaveError, Source, Stamp};
use crate::frontmatter::{self, Entry, Frontmatter, ListIssue, ReadOnly, Shape, Value};
use crate::i18n::Msg;
use crate::links;
use crate::mdtext::{fence_close, fence_open, indent_of};
use crate::types::{self, Declared, Kind};
use crate::vault::{self, Note, Progress, Vault};
use crate::writeback::{self, Baseline};
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

impl Parsed {
    /// 値を見せるフロントマター: 読めればそれ、BOM つきで BOM を除けば読めるならそれ(BV-1・WB-5)。
    fn readable(&self) -> Option<&Frontmatter> {
        self.fm.as_ref().ok().or(self.shown.as_ref())
    }
}

/// types.json の宣言を大文字小文字を外した名前で引く表。同じ名前に違う型が集まれば Conflict。
fn lower_declared(declared: &HashMap<String, Declared>) -> HashMap<String, Declared> {
    let mut out: HashMap<String, Declared> = HashMap::new();
    for (k, d) in declared {
        out.entry(k.to_lowercase())
            .and_modify(|x| {
                if x != d {
                    *x = Declared::Conflict;
                }
            })
            .or_insert(*d);
    }
    out
}

/// 読んだノート1つを解析した結果。
struct Parsed {
    fm: Result<Frontmatter, ReadOnly>,
    /// BOM で始まるノート(読むだけ。WB-5)を、BOM を除いて読めたもの。見せる値と列にだけ使う(書かない)。
    shown: Option<Frontmatter>,
    /// 行に入るノートか(BV-1)。列は行のノートだけから作る。
    row: bool,
    /// 作成時刻(UNIX 秒)。取れなければ更新時刻。
    ctime: i64,
    /// フロントマターの tags と本文の `#tag`。
    tags: Vec<String>,
    /// 本文とフロントマターの値から拾ったリンク(BV-22)。
    links: Vec<links::Raw>,
}

pub struct Markdown {
    vault: Vault,
    /// 渡したフォルダ(実体のパス)。行はこの下のノートだけ。
    folders: Vec<PathBuf>,
    /// 最後に読んだ内容を解析した結果(実体のパス → 解析)。根の下の読んだノート全部。
    parsed: HashMap<PathBuf, Parsed>,
    /// types.json に無い列の推定した型(CE-2)。セルを読むたびに全ノートから推定し直すと、大きな保管庫で
    /// 2乗に遅くなるので覚える。ノートを解析し直したら忘れる。
    inferred: std::cell::RefCell<HashMap<String, Kind>>,
    /// リンクの索引(BV-22)。inferred と同じく、セルごとに全ノートを見直さないよう覚え、ノートを解析し直したら忘れる。
    link_index: std::cell::RefCell<Option<Rc<links::Index>>>,
    /// 根ごとの `.obsidian/types.json` を合わせたもの(開いたときに読む)。
    declared: HashMap<String, Declared>,
    /// declared を大文字小文字を外した名前で引く表(CE-2。開くときに1回作る。大文字小文字だけ違う宣言の型が
    /// 食い違えば Conflict)。
    declared_lower: HashMap<String, Declared>,
    /// 読んだ行のキーの和(最初に現れた順)。
    columns: Vec<String>,
    seen_columns: HashSet<String>,
    /// 最初の読み込みを終えた(以後の load は何もしない)。
    done: Option<Progress>,
    /// 設定の add_frontmatter(WB-3・CLI-3。既定 true)。false ならフロントマターの無いノートと空のフロントマターの
    /// ノートを読むだけにする。
    add_frontmatter: bool,
}

impl Markdown {
    /// まだ何も読まない(BV-16)。読めないフォルダは Err。
    pub fn open(folders: &[PathBuf]) -> io::Result<Markdown> {
        let mut real = Vec::new();
        for f in folders {
            real.push(f.canonicalize()?);
        }
        let roots = vault::roots(folders)?;
        let per_root: Vec<HashMap<String, Kind>> = roots
            .iter()
            .map(|r| {
                std::fs::read(r.join(".obsidian").join("types.json"))
                    .map(|b| types::read_types_json(&b))
                    .unwrap_or_default()
            })
            .collect();
        Ok(Markdown {
            // BV-23: 渡したフォルダ(とその上)は、隠しフォルダでも探す。
            vault: Vault::open_keeping(roots, real.clone()),
            folders: real,
            parsed: HashMap::new(),
            inferred: Default::default(),
            link_index: Default::default(),
            declared_lower: lower_declared(&types::merge(&per_root)),
            declared: types::merge(&per_root),
            columns: Vec::new(),
            seen_columns: HashSet::new(),
            done: None,
            add_frontmatter: true,
        })
    }

    /// 設定の add_frontmatter を渡す(WB-3・CLI-3)。開いた直後、読み込み(load)の前に呼ぶ。
    /// false なら、フロントマターの無いノートと空のフロントマターのノートのセルを読むだけにし、書かない。
    pub fn set_add_frontmatter(&mut self, on: bool) {
        self.add_frontmatter = on;
    }

    /// 設定(add_frontmatter = false)で書かないノートの理由。書けるなら None。
    fn setting_lock(&self, fm: &Result<Frontmatter, ReadOnly>) -> Option<&'static str> {
        if self.add_frontmatter {
            return None;
        }
        match fm {
            Err(ReadOnly::NoFrontmatter) => Some(Msg::LockNoFrontmatterSetting.text()),
            Err(ReadOnly::EmptyFrontmatter) => Some(Msg::LockEmptyFrontmatterSetting.text()),
            _ => None,
        }
    }

    /// `.base` のファイルから開く(BV-2): 根は `.base` のあるフォルダから決め(`.obsidian/` を持つ最寄りの上、
    /// 無ければそのフォルダ)、行は根の下の全ノート。`.base` は読まない(BV-3)。無い・読めないパスは Err。
    pub fn open_vault(base_file: &Path) -> io::Result<Markdown> {
        std::fs::metadata(base_file)?;
        let dir = match base_file.parent() {
            Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let roots = vault::roots(&[dir])?;
        Markdown::open(&roots)
    }

    /// 保管庫の根(BV-2)。
    pub fn roots(&self) -> &[PathBuf] {
        self.vault.roots()
    }

    /// 根の下のファイル(`.base` など。BV-22 の `this`)の属性。根の下に無い・読めないファイルは None。
    pub fn this_file(&self, path: &Path) -> Option<FileInfo> {
        let real = path.canonicalize().ok()?;
        let meta = std::fs::metadata(&real).ok()?;
        let rel = self.vault.roots().iter().find_map(|r| {
            let parts: Vec<String> = real
                .strip_prefix(r)
                .ok()?
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            (!parts.is_empty()).then(|| parts.join("/"))
        })?;
        let (folder, name) = match rel.rsplit_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => (String::new(), rel.clone()),
        };
        let (basename, ext) = match name.rsplit_once('.') {
            Some((b, e)) if !b.is_empty() => (b.to_string(), e.to_string()),
            _ => (name.clone(), String::new()),
        };
        let mtime = meta.modified().map(unix_secs).unwrap_or(0);
        Some(FileInfo {
            name,
            basename,
            ext,
            path: rel,
            folder,
            size: meta.len(),
            mtime,
            ctime: meta.created().map(unix_secs).unwrap_or(mtime),
            tags: Vec::new(),
        })
    }

    /// 行に入るノートか(BV-1): 見つけた道筋のどれかが渡したフォルダの下で、実体のパスが UTF-8。
    fn in_scope(&self, note: &Note) -> bool {
        if note.path.to_str().is_none() {
            return false;
        }
        let under = |p: &Path| self.folders.iter().any(|f| p.starts_with(f));
        if note.found.is_empty() {
            under(&note.path)
        } else {
            note.found.iter().any(|p| under(p))
        }
    }

    fn notes(&self) -> impl Iterator<Item = &Note> {
        self.vault.notes().iter().filter(|n| self.in_scope(n))
    }

    fn note(&self, row: &RowId) -> Option<&Note> {
        self.vault
            .note(Path::new(&row.0))
            .filter(|n| self.in_scope(n))
    }

    fn add_columns(&mut self, fm: &Frontmatter) {
        for e in &fm.entries {
            if self.seen_columns.insert(e.key.clone()) {
                self.columns.push(e.key.clone());
            }
        }
    }

    /// 増えたノートだけ解析し、行のノートの新しいキーを列の末尾に足す。
    fn parse_new(&mut self) {
        self.inferred.borrow_mut().clear();
        self.link_index.borrow_mut().take();
        let mut new = Vec::new();
        let mut now_rows = Vec::new();
        for n in self.vault.notes() {
            match self.parsed.get(&n.path) {
                None => new.push((n.path.clone(), parse_note(n, self.in_scope(n)))),
                // poll の探し直しで道筋が増え、行に入るようになったノート。
                Some(p) if !p.row && self.in_scope(n) => now_rows.push(n.path.clone()),
                Some(_) => {}
            }
        }
        for path in now_rows {
            if let Some(mut p) = self.parsed.remove(&path) {
                p.row = true;
                if let Some(fm) = p.readable() {
                    self.add_columns(fm);
                }
                self.parsed.insert(path, p);
            }
        }
        for (path, p) in new {
            if let (true, Some(fm)) = (p.row, p.readable()) {
                self.add_columns(fm);
            }
            self.parsed.insert(path, p);
        }
    }

    /// 根の下の読んだノート全部の `col` の値(パスの昇順)。
    fn values<'a>(&'a self, col: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
        self.vault.notes().iter().filter_map(move |n| {
            let fm = self.parsed.get(&n.path)?.readable()?;
            fm.entries.iter().find(|e| e.key == col).map(|e| &e.value)
        })
    }

    /// 変わった・消えたノートのあと: `stale` を解析し直し、列を行の順に作り直す。
    fn rebuild(&mut self, stale: &[PathBuf]) {
        self.inferred.borrow_mut().clear();
        self.link_index.borrow_mut().take();
        for p in stale {
            self.parsed.remove(p);
        }
        let keep: HashSet<PathBuf> = self.vault.notes().iter().map(|n| n.path.clone()).collect();
        self.parsed.retain(|p, _| keep.contains(p));
        self.parse_new();
        self.columns.clear();
        self.seen_columns.clear();
        let order: Vec<PathBuf> = self.notes().map(|n| n.path.clone()).collect();
        let parsed = std::mem::take(&mut self.parsed);
        for p in &order {
            if let Some(fm) = parsed.get(p).and_then(Parsed::readable) {
                self.add_columns(fm);
            }
        }
        self.parsed = parsed;
    }

    /// types.json の列の型(CE-2): 同じ名前、無ければ大文字小文字を問わない名前(Obsidian のプロパティの名前と同じ)。
    fn declared_of(&self, col: &str) -> Option<&Declared> {
        self.declared
            .get(col)
            .or_else(|| self.declared_lower.get(&col.to_lowercase()))
    }

    /// 根ごとの types.json が食い違う列の理由(BV-12)。
    fn declared_lock(&self, col: &str) -> Option<String> {
        matches!(self.declared_of(col), Some(Declared::Conflict))
            .then(|| Msg::LockTypesConflict.text().to_string())
    }

    /// リストの列(CE-16)なのに、空でない1つの値で書かれたセル(`tags: x`)。リストとして書くと書き方が
    /// 変わるので読むだけ(書き戻しは NotEditable)。
    /// CE-19: リストの列の1つの値。文字列は1つの要素のリストとして書けるので lock なし(ブロックの文字は
    /// value_lock が先に「複数行の値」で止める)。文字列でない値は書くと型が変わるので読むだけ。
    fn scalar_list_lock(&self, col: &str, e: &Entry) -> Option<String> {
        let scalar = matches!(e.value, Value::Bool(_) | Value::Int(_) | Value::Float(_));
        (scalar && self.kind(col).kind == Kind::List).then(|| Msg::ListNonString.text().to_string())
    }

    fn row_of(note: &Note) -> Option<RowId> {
        note.path.to_str().map(|s| RowId(s.to_string()))
    }
}

fn unix_secs(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

fn parse_note(note: &Note, row: bool) -> Parsed {
    let fm = frontmatter::parse(&note.bytes);
    let ctime = std::fs::metadata(&note.path)
        .and_then(|m| m.created())
        .map(unix_secs)
        .unwrap_or_else(|_| unix_secs(note.stamp.mtime));
    let shown = match &fm {
        Err(ReadOnly::Bom) => frontmatter::parse(&note.bytes[3..]).ok(),
        // C-4: JSON・フローの形は YAML の読み手で値だけを読む(書かない)。
        Err(ReadOnly::InvalidYaml) => frontmatter::flow_frontmatter(&note.bytes),
        _ => None,
    };
    // タグとリンクは、ファイルのバイトの位置が合うフロントマター(読めたものか、フローの形)から。
    // BOM の見せる値は3バイトずれるので使わない。
    let placed = match &fm {
        Ok(f) => Some(f),
        Err(ReadOnly::InvalidYaml) => shown.as_ref(),
        Err(_) => None,
    };
    let tags = tags_of(&note.bytes, placed);
    let links = links_of(&note.bytes, placed);
    Parsed {
        fm,
        shown,
        row,
        ctime,
        tags,
        links,
    }
}

/// フロントマターの tags(リストか、カンマ・空白で区切った文字列)と本文の `#tag`。先頭の # を除き、重複は除く。
/// フロントマターが読めない(ReadOnly の)ノートは、値を推測しないので tags を読まず、本文だけ見る。
fn tags_of(bytes: &[u8], fm: Option<&Frontmatter>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |t: &str| {
        let t = t.trim().trim_start_matches('#');
        if !t.is_empty() && !out.iter().any(|x| x == t) {
            out.push(t.to_string());
        }
    };
    // Obsidian と同じく、タグのキーは大文字小文字を問わず `tags` と `tag`(C-5)。
    let keys = fm
        .into_iter()
        .flat_map(|fm| fm.entries.iter())
        .filter(|e| e.key.eq_ignore_ascii_case("tags") || e.key.eq_ignore_ascii_case("tag"));
    for e in keys {
        match &e.value {
            Value::List(items) => {
                for v in items {
                    if let Value::Str(t) = v {
                        push(t);
                    }
                }
            }
            Value::Str(s) => s.split([',', ' ']).for_each(&mut push),
            _ => {}
        }
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return out;
    };
    for t in body_tags(&text[body_start(text, fm)..]) {
        push(t);
    }
    out
}

/// 本文のリンクと、フロントマターの値(文字と、リストの文字の要素)の中のリンク(BV-22)。
fn links_of(bytes: &[u8], fm: Option<&Frontmatter>) -> Vec<links::Raw> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    let mut out = links::extract(&text[body_start(text, fm)..]);
    for e in fm.map_or(&[][..], |fm| fm.entries.as_slice()) {
        let items = match &e.value {
            Value::List(items) => items.as_slice(),
            v => std::slice::from_ref(v),
        };
        for v in items {
            if let Value::Str(s) = v {
                out.extend(links::extract(s));
            }
        }
    }
    out
}

/// 本文の始まり(バイト位置): フロントマターの閉じの行の次。フロントマターが読めないとき(ReadOnly)は、
/// BOM を除いた先頭の行が `---` で、`---` の行で閉じていればその次、ほかは(BOM の後の)先頭。
fn body_start(text: &str, fm: Option<&Frontmatter>) -> usize {
    let after_line = |i: usize| text[i..].find('\n').map_or(text.len(), |j| i + j + 1);
    if let Some(fm) = fm {
        return after_line(fm.end);
    }
    let bom = if text.starts_with('\u{feff}') {
        '\u{feff}'.len_utf8()
    } else {
        0
    };
    let is_delim = |line: &str| line.strip_suffix('\r').unwrap_or(line) == "---";
    let mut lines = text[bom..].split_inclusive('\n');
    let Some(first) = lines.next() else {
        return bom;
    };
    if !is_delim(first.strip_suffix('\n').unwrap_or(first)) {
        return bom;
    }
    let mut at = bom + first.len();
    for line in lines {
        at += line.len();
        if is_delim(line.strip_suffix('\n').unwrap_or(line)) {
            return at;
        }
    }
    bom
}

/// リストの項目の行(`- `・`* `・`+ `・`1. `・`1) `)か。
fn list_item(rest: &str) -> bool {
    let b = rest.as_bytes();
    if matches!(b.first(), Some(b'-' | b'*' | b'+')) {
        return matches!(b.get(1), None | Some(b' ' | b'\t'));
    }
    let n = b.iter().take_while(|c| c.is_ascii_digit()).count();
    (1..=9).contains(&n)
        && matches!(b.get(n), Some(b'.' | b')'))
        && matches!(b.get(n + 1), None | Some(b' ' | b'\t'))
}

/// 本文の `#tag`(Obsidian の形): 行頭か空白の後の `#` に、文字・数字・`_`・`-`・`/` が続き、数字だけではないもの。
/// コードのフェンスの中(開きと同じ記号で、開き以上の長さで閉じる)、字下げ4つ以上のコードブロック(リストの中を除く)、
/// インラインのコード(同じ数のバッククォートで対にする)は見ない。
fn body_tags(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut prev_blank = true;
    let mut indented_code = false;
    let mut in_list = false;
    for line in body.lines() {
        if let Some(f) = fence {
            if fence_close(line, f) {
                fence = None;
            }
            continue;
        }
        if line.trim().is_empty() {
            prev_blank = true;
            continue;
        }
        let (w, rest) = indent_of(line);
        let blank_before = std::mem::replace(&mut prev_blank, false);
        if w >= 4 && !in_list && (blank_before || indented_code) {
            indented_code = true;
            continue;
        }
        indented_code = false;
        if let Some(f) = fence_open(line) {
            fence = Some(f);
            continue;
        }
        if list_item(rest) {
            in_list = true;
        } else if w == 0 && blank_before {
            in_list = false;
        }
        line_tags(line, &mut out);
    }
    out
}

/// 1行の中の `#tag`(インラインのコードを除く)。
fn line_tags<'a>(line: &'a str, out: &mut Vec<&'a str>) {
    let b = line.as_bytes();
    let mut i = 0;
    let mut prev_space = true;
    while i < b.len() {
        if b[i] == b'`' {
            let n = b[i..].iter().take_while(|&&c| c == b'`').count();
            // 同じ数のバッククォートの並び(前後が ` でない)を探す。無ければ字のまま。
            let mut j = i + n;
            let mut close = None;
            while j < b.len() {
                if b[j] == b'`' {
                    let m = b[j..].iter().take_while(|&&c| c == b'`').count();
                    if m == n {
                        close = Some(j + m);
                        break;
                    }
                    j += m;
                } else {
                    j += 1;
                }
            }
            i = close.unwrap_or(i + n);
            prev_space = false;
            continue;
        }
        if b[i] == b'#' && prev_space {
            let rest = &line[i + 1..];
            let len = rest
                .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/')))
                .unwrap_or(rest.len());
            let tag = rest[..len].trim_end_matches('/');
            if !tag.is_empty() && !tag.chars().all(|c| c.is_ascii_digit()) {
                out.push(tag);
            }
        }
        // i は文字の先頭(` と # は ASCII なので、ほかの文字は丸ごと進む)。
        let c = line[i..].chars().next().unwrap_or(' ');
        prev_space = c.is_whitespace();
        i += c.len_utf8();
    }
}

/// フロントマターごと読むだけにする理由。YAML として読めないときは、行と誤りの文を添える(`bytes` から)。
fn read_only_reason(r: &ReadOnly, bytes: &[u8]) -> String {
    match r {
        ReadOnly::NoFrontmatter => Msg::LockNoFrontmatter.text().into(),
        ReadOnly::EmptyFrontmatter => Msg::LockEmptyFrontmatter.text().into(),
        ReadOnly::Bom => Msg::LockBom.text().into(),
        ReadOnly::NotUtf8 => Msg::LockNotUtf8.text().into(),
        ReadOnly::MixedNewlines => Msg::LockMixedNewlines.text().into(),
        ReadOnly::DuplicateKey(k) => Msg::LockDuplicateKey.fill(&[k]),
        ReadOnly::Unclosed => Msg::LockUnclosed.text().into(),
        ReadOnly::HardLink => Msg::LockHardLink.text().into(),
        ReadOnly::InvalidYaml => match crate::frontmatter::yaml_error(bytes) {
            Some((line, why)) => Msg::LockInvalidYamlAt.fill(&[&line, &why]),
            None => Msg::LockInvalidYaml.text().into(),
        },
    }
}

/// 列の名前だけで読むだけになる列(CE-8)。
fn column_lock(col: &str) -> Option<&'static str> {
    if col.starts_with("file.") {
        Some(Msg::LockFileAttr.text())
    } else if col.starts_with("formula.") {
        Some(Msg::LockFormulaColumn.text())
    } else {
        None
    }
}

/// キーの行の値の部分と、続きの行(字下げ・空行・コメント・行頭の `- `)の数を、元のバイトから探す。
fn source_of<'a>(bytes: &'a [u8], fm: &Frontmatter, key: &str) -> Option<(&'a str, usize)> {
    let text = std::str::from_utf8(bytes.get(..fm.end)?).ok()?;
    let lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let candidates = [key.to_string(), format!("\"{key}\""), format!("'{key}'")];
    for (i, line) in lines.iter().enumerate().skip(1) {
        let Some(rest) = candidates
            .iter()
            .find_map(|c| line.strip_prefix(c.as_str()))
        else {
            continue;
        };
        let Some(value) = rest.trim_start_matches([' ', '\t']).strip_prefix(':') else {
            continue;
        };
        let more = lines[i + 1..]
            .iter()
            .take_while(|l| l.starts_with([' ', '\t', '-']) || l.trim().is_empty())
            .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
            .count();
        return Some((value.trim_matches([' ', '\t']), more));
    }
    None
}

/// 書けない形のリストの理由(CE-8・CE-19)。形ごとに分ける。
fn list_issue_reason(issue: ListIssue) -> &'static str {
    match issue {
        ListIssue::BlankBetween => Msg::ListBlankBetween,
        ListIssue::CommentBetween => Msg::ListCommentBetween,
        ListIssue::TrailingComment => Msg::ListTrailingComment,
        ListIssue::Indent => Msg::ListIndent,
        ListIssue::TabAfterDash => Msg::ListTabAfterDash,
        ListIssue::MultiLine => Msg::ListMultiLine,
        ListIssue::Nested => Msg::ListNested,
        ListIssue::NonString => Msg::ListNonString,
    }
    .text()
}

/// 値ごとに読むだけにする理由(CE-8)。書ける値なら None。
/// リストの値(CE-16)は、span があり要素が全部文字列なら書ける(writeback::apply の NewValue::List と同じ条件)。
fn value_lock(bytes: &[u8], fm: &Frontmatter, e: &Entry) -> Option<String> {
    let non_str = |v: &Value| match v {
        Value::List(items) => items.iter().any(|i| !matches!(i, Value::Str(_))),
        _ => false,
    };
    let nested = |v: &Value| match v {
        Value::List(items) => items
            .iter()
            .any(|i| matches!(i, Value::Other | Value::List(_))),
        _ => false,
    };
    if e.span.is_some() {
        if matches!(e.shape, Shape::FlowList | Shape::BlockList) && non_str(&e.value) {
            return Some(if nested(&e.value) {
                list_issue_reason(ListIssue::Nested).into()
            } else {
                list_issue_reason(ListIssue::NonString).into()
            });
        }
        return None;
    }
    let reason = match e.shape {
        Shape::BlockList => {
            list_issue_reason(frontmatter::list_issue(bytes, &e.key).unwrap_or(ListIssue::Nested))
        }
        Shape::Nested => Msg::ValueNested.text(),
        Shape::BlockScalar => Msg::ValueMultiLine.text(),
        Shape::Anchor => Msg::ValueAnchor.text(),
        _ => match source_of(bytes, fm, &e.key) {
            Some((v, _)) if v.starts_with('!') => Msg::ValueTagged.text(),
            Some((v, _)) if v.starts_with(['&', '*']) => Msg::ValueAnchor.text(),
            Some((_, more)) if more > 0 => Msg::ValueMultiLine.text(),
            _ => Msg::ValueUnwritable.text(),
        },
    };
    Some(reason.into())
}

fn stamp_of(note: &Note) -> Stamp {
    Stamp {
        mtime: note.stamp.mtime,
        len: note.bytes.len() as u64,
        hash: writeback::sha256(&note.bytes),
    }
}

fn not_loaded() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, Msg::RowNotLoaded.text())
}

impl Source for Markdown {
    fn name(&self) -> String {
        self.folders
            .iter()
            .map(|f| {
                f.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| f.to_string_lossy().into_owned())
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn load(&mut self, budget: usize) -> Progress {
        if let Some(p) = self.done {
            return p;
        }
        let p = self.vault.load(budget);
        self.parse_new();
        if p.done {
            self.done = Some(p);
        }
        p
    }

    fn cancel(&mut self) {
        self.vault.cancel();
        self.done = Some(self.vault.load(0));
    }

    fn rows(&self) -> Vec<RowId> {
        self.notes().filter_map(Self::row_of).collect()
    }

    fn label(&self, row: &RowId) -> String {
        self.note(row)
            .map(|n| n.rel.clone())
            .unwrap_or_else(|| row.0.clone())
    }

    fn mark(&self, row: &RowId) -> Option<String> {
        self.note(row)
            .filter(|n| n.conflict)
            .map(|_| Msg::SyncConflictFile.text().to_string())
    }

    fn columns(&self) -> Vec<String> {
        self.columns.clone()
    }

    fn get(&self, row: &RowId, col: &str) -> Cell {
        let Some(note) = self.note(row) else {
            return Cell {
                value: None,
                lock: None,
            };
        };
        let (value, lock) = match self.parsed.get(&note.path).map(|p| &p.fm) {
            None => (None, None),
            // WB-3: フロントマターの無いノート(先頭にフロントマターを足す)と空のフロントマターのノート(区切りの間に
            // 足す)は、既定で lock なし(値は無し)。設定 add_frontmatter = false なら読むだけ。
            // WB-20: TOML のフロントマターは「無い」と読めるが、足さずに読むだけ。
            Some(Err(ReadOnly::NoFrontmatter)) if crate::frontmatter::is_toml(&note.bytes) => {
                (None, Some(Msg::LockToml.text().to_string()))
            }
            Some(fm @ Err(ReadOnly::NoFrontmatter | ReadOnly::EmptyFrontmatter)) => {
                (None, self.setting_lock(fm).map(String::from))
            }
            // WB-5: BOM つきは読むだけのまま、BOM を除いて読めた値を見せる(BV-1)。
            Some(Err(r)) => {
                let shown = self.parsed.get(&note.path).and_then(|p| p.shown.as_ref());
                let value = shown
                    .and_then(|fm| fm.entries.iter().find(|e| e.key == col))
                    .map(|e| e.value.clone());
                (value, Some(read_only_reason(r, &note.bytes)))
            }
            Some(Ok(fm)) => match fm.entries.iter().find(|e| e.key == col) {
                None => (None, None),
                Some(e) => {
                    let lock =
                        value_lock(&note.bytes, fm, e).or_else(|| self.scalar_list_lock(col, e));
                    (Some(e.value.clone()), lock)
                }
            },
        };
        // 行全体・列全体の理由が先(WB-5・CE-8)。
        let lock = if note.links > 1 {
            Some(Msg::LockHardLink.text().to_string())
        } else if !note.writable {
            // WB-5: 権限はハードリンクと同じくファイルの性質なので、読み取りの層(ReadOnly)ではなくここで持つ。
            Some(Msg::NoPermission.text().to_string())
        } else {
            column_lock(col)
                .map(String::from)
                .or_else(|| self.declared_lock(col))
                .or(lock)
        };
        Cell { value, lock }
    }

    fn stamp(&self, row: &RowId) -> Option<Stamp> {
        self.note(row).map(stamp_of)
    }

    fn reload(&mut self, row: &RowId) -> io::Result<()> {
        let path = PathBuf::from(&row.0);
        self.vault.reload(&path)?;
        self.rebuild(&[path]);
        Ok(())
    }

    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError> {
        let note = self
            .note(row)
            .ok_or_else(|| EditError::NotEditable(Msg::RowNotLoaded.text().into()))?;
        // 設定 add_frontmatter はここでは見ない(書かない。外で変わった行の差分を外の変更として見せるため。
        // 書くかの判断は save が基準の検査のあとにする)。
        // 今のディスクのバイト。読んだ内容(vault)は変えない。
        let before = std::fs::read(&note.path)
            .map_err(|e| EditError::NotEditable(Msg::CannotRead.fill(&[&e])))?;
        let after = writeback::apply(&before, edits)?;
        Ok((before, after))
    }

    fn save(&mut self, row: &RowId, base: &Stamp, edits: &[Edit]) -> Result<Stamp, SaveError> {
        let note = self.note(row).ok_or_else(not_loaded)?;
        let path = note.path.clone();
        // WB-3: 設定で書かないノートは、画面の lock を通らずに来ても書かない。外の変更の判定(WB-4・WB-16)が先で、
        // 基準から変わっていれば Changed。判断は基準の内容(= 書く元のバイト)で見る。
        if !self.add_frontmatter {
            let base_line = Baseline {
                mtime: base.mtime,
                len: base.len,
                hash: base.hash,
            };
            if writeback::baseline(&path)? != base_line {
                return Err(SaveError::Changed);
            }
            let bytes = if stamp_of(note).hash == base.hash {
                note.bytes.clone()
            } else {
                std::fs::read(&path)?
            };
            if writeback::sha256(&bytes) != base.hash {
                return Err(SaveError::Changed);
            }
            if let Some(reason) = self.setting_lock(&frontmatter::parse(&bytes)) {
                return Err(SaveError::Edit(EditError::NotEditable(reason.into())));
            }
        }
        // 書くバイトを先に作っておく(基準が読んだ内容と同じときだけ使える)。
        let written = if stamp_of(note).hash == base.hash {
            writeback::apply(&note.bytes, edits).ok()
        } else {
            None
        };
        let next = writeback::save(
            &path,
            &Baseline {
                mtime: base.mtime,
                len: base.len,
                hash: base.hash,
            },
            edits,
        )?;
        match written {
            // 書いたバイトと返った基準で置き換える。読み直さないので、書いたあとの外の変更は次の changed で見つかる(WB-16)。
            Some(bytes) if writeback::sha256(&bytes) == next.hash => {
                let stamp = vault::Stamp {
                    mtime: next.mtime,
                    len: next.len,
                };
                self.vault.replace(&path, bytes, stamp);
            }
            // 読んだ内容と違う基準で書けた(まれ)。書いた内容は分からないので読み直す。
            // 失敗しても書いたことは変わらず、古いままの中身は次の changed で読み直される。
            _ => {
                let _ = self.vault.reload(&path);
            }
        }
        self.rebuild(std::slice::from_ref(&path));
        Ok(Stamp {
            mtime: next.mtime,
            len: next.len,
            hash: next.hash,
        })
    }

    fn changed(&mut self) -> Vec<RowId> {
        let poll = self.vault.poll();
        if poll.changed.is_empty() && poll.added.is_empty() && poll.removed.is_empty() {
            return Vec::new();
        }
        // 消えたノートが行だったかは、解析の結果(行のノートだけを持つ)で見る。
        let mut touched: Vec<PathBuf> = poll
            .removed
            .iter()
            .filter(|p| self.parsed.get(*p).is_some_and(|x| x.row))
            .cloned()
            .collect();
        touched.extend(
            poll.changed
                .iter()
                .chain(&poll.added)
                .filter(|p| self.vault.note(p).is_some_and(|n| self.in_scope(n)))
                .cloned(),
        );
        if poll.changed.is_empty() && poll.removed.is_empty() {
            self.parse_new();
        } else {
            self.rebuild(&poll.changed);
        }
        touched.sort();
        touched.dedup();
        touched
            .into_iter()
            .filter_map(|p| p.to_str().map(|s| RowId(s.to_string())))
            .collect()
    }

    fn pause(&mut self, paused: bool) {
        self.vault.pause(paused);
    }

    fn kind(&self, col: &str) -> ColumnKind {
        match self.declared_of(col) {
            Some(Declared::One(k)) => ColumnKind {
                kind: *k,
                lock: None,
            },
            Some(Declared::Conflict) => ColumnKind {
                kind: Kind::Text,
                lock: self.declared_lock(col),
            },
            None => {
                let cached = self.inferred.borrow().get(col).copied();
                let kind = cached.unwrap_or_else(|| {
                    let k = types::infer(self.values(col));
                    self.inferred.borrow_mut().insert(col.to_string(), k);
                    k
                });
                ColumnKind { kind, lock: None }
            }
        }
    }

    fn typed(&self, col: &str) -> bool {
        self.declared_of(col).is_some() || self.values(col).any(|v| !types::is_empty(v))
    }

    fn folders(&self) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = Vec::new();
        for f in &self.folders {
            if !out.contains(f) {
                out.push(f.clone());
            }
        }
        out
    }

    fn body(&self, row: &RowId) -> Option<String> {
        let note = self.note(row)?;
        let text = std::str::from_utf8(&note.bytes).ok()?;
        let fm = self.parsed.get(&note.path).and_then(|p| p.fm.as_ref().ok());
        Some(text[body_start(text, fm)..].to_string())
    }

    fn file(&self, row: &RowId) -> Option<FileInfo> {
        let note = self.note(row)?;
        let p = self.parsed.get(&note.path);
        let path = note.rel.clone();
        let (folder, name) = match path.rsplit_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => (String::new(), path.clone()),
        };
        let (basename, ext) = match name.rsplit_once('.') {
            Some((b, e)) if !b.is_empty() => (b.to_string(), e.to_string()),
            _ => (name.clone(), String::new()),
        };
        let mtime = unix_secs(note.stamp.mtime);
        Some(FileInfo {
            name,
            basename,
            ext,
            path,
            folder,
            size: note.bytes.len() as u64,
            mtime,
            ctime: p.map_or(mtime, |p| p.ctime),
            tags: p.map(|p| p.tags.clone()).unwrap_or_default(),
        })
    }

    fn candidates(&self, col: &str, max: usize) -> Option<Vec<Value>> {
        let mut out: Vec<Value> = Vec::new();
        for v in self.values(col) {
            if types::is_empty(v) || out.contains(v) {
                continue;
            }
            if out.len() == max {
                return None;
            }
            out.push(v.clone());
        }
        Some(out)
    }

    fn link_index(&self) -> Option<Rc<links::Index>> {
        if let Some(ix) = self.link_index.borrow().as_ref() {
            return Some(Rc::clone(ix));
        }
        let notes = self
            .vault
            .notes()
            .iter()
            .map(|n| {
                let raws = self
                    .parsed
                    .get(&n.path)
                    .map(|p| p.links.clone())
                    .unwrap_or_default();
                (n.rel.clone(), raws)
            })
            .collect();
        let ix = Rc::new(links::Index::build(notes));
        *self.link_index.borrow_mut() = Some(Rc::clone(&ix));
        Some(ix)
    }

    fn list_candidates(&self, col: &str) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for v in self.values(col) {
            // 1つの値で書かれたもの(`tags: x`)も、1つの要素として数える。
            let items = match v {
                Value::List(items) => items.as_slice(),
                Value::Str(_) => std::slice::from_ref(v),
                _ => continue,
            };
            // 件数はその要素を持つノートの数(1つのノートで同じ要素が2つあっても1件)。
            let mut seen = HashSet::new();
            for item in items {
                let Value::Str(s) = item else {
                    continue;
                };
                let s = if col == "tags" {
                    s.strip_prefix('#').unwrap_or(s)
                } else {
                    s.as_str()
                };
                if !s.is_empty() && seen.insert(s) {
                    *counts.entry(s.to_string()).or_default() += 1;
                }
            }
        }
        let mut out: Vec<(String, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
}

#[cfg(test)]
#[path = "test_markdown_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_types_case_unit.rs"]
mod test_types_case_unit;

#[cfg(test)]
#[path = "test_review6_unit.rs"]
mod test_review6_unit;
