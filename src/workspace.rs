//! ワークスペース(specs/workspace/spec.md): 表をまとめる範囲。
//!
//! - アプリの側(WS-1): 設定のフォルダの `workspaces.toml`(`[[workspace]]` に `name` と `[[workspace.table]]`)。
//! - フォルダの印(WS-7): `.mdgrid/workspace.toml`(`name` と `[[table]]`。`path` は根からの相対)。表を書かなければ
//!   根の直下の、ノートのあるフォルダを表にする(`.base` は入れない。書けば足せる)。
//! - 検知(WS-5): Obsidian の保管庫(`.obsidian/`)か git のリポ(`.git`)の根を、名前の無いワークスペースに。
//! - 範囲の決め方(WS-6): `-w` → 印 → workspaces.toml → 検知 →(呼ぶ側で)登録した表。

use crate::config;
use crate::i18n::Msg;
use crate::places::expand_home;
use std::io;
use std::path::{Path, PathBuf};

/// アプリの側のワークスペースのファイル(views.toml・places.toml と同じ設定のフォルダ)。
pub const FILE_NAME: &str = "workspaces.toml";
/// フォルダの印のフォルダとファイル。
pub const MARKER_DIR: &str = ".mdgrid";
pub const MARKER_FILE: &str = "workspace.toml";

/// ワークスペースの表1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsTable {
    pub name: String,
    /// フォルダか `.base`(実体のパスか、書いたパス)。
    pub path: PathBuf,
    pub view: Option<String>,
}

/// ワークスペース1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
    pub tables: Vec<WsTable>,
}

/// 検知のしかた(WS-5)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detect {
    Vault,
    Git,
}

impl Detect {
    pub fn parse(s: &str) -> Option<Detect> {
        match s {
            "vault" => Some(Detect::Vault),
            "git" => Some(Detect::Git),
            _ => None,
        }
    }

    fn marker(self) -> &'static str {
        match self {
            Detect::Vault => ".obsidian",
            Detect::Git => ".git",
        }
    }
}

/// 範囲がどこから来たか(WS-6)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `-w` で選んだ。
    Chosen,
    /// フォルダの印。
    Marker,
    /// workspaces.toml。
    App,
    /// 検知。
    Detected(Detect),
}

/// 決めた範囲。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub name: String,
    pub source: Source,
    pub tables: Vec<WsTable>,
    /// 範囲を決めるときに読めなかったもの(印の誤りなど)。画面が出す。
    pub warnings: Vec<String>,
}

/// 表の並び(`table` の配列)を読む。`base` があれば相対のパスはそこから。
fn read_tables(
    v: Option<&toml::Value>,
    base: Option<&Path>,
    what: &str,
    warns: &mut Vec<String>,
) -> Vec<WsTable> {
    let mut out: Vec<WsTable> = Vec::new();
    let items = v.and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for (k, item) in items.iter().enumerate() {
        let n = k + 1;
        let Some(t) = item.as_table() else {
            warns.push(Msg::WsBadTable.fill(&[&what, &n]));
            continue;
        };
        let text = |key: &str| {
            t.get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
        };
        let Some(path) = text("path").filter(|p| !p.is_empty()) else {
            warns.push(Msg::WsBadTable.fill(&[&what, &n]));
            continue;
        };
        let mut p = expand_home(&path);
        if p.is_relative() {
            if let Some(b) = base {
                p = b.join(p);
            }
        }
        let name = text("name")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| stem_of(&p));
        out.push(WsTable {
            name,
            path: p,
            view: text("view").filter(|s| !s.is_empty()),
        });
    }
    out
}

/// パスの名前(`.base` は拡張子を除く。フォルダの `v1.2` はそのまま)。
pub fn stem_of(p: &Path) -> String {
    let s = if p.extension().is_some_and(|x| x == "base") {
        p.file_stem()
    } else {
        p.file_name()
    };
    s.map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// workspaces.toml の文を読む(WS-1)。読めない行は警告にして飛ばす。
pub fn parse(text: &str) -> (Vec<Workspace>, Vec<String>) {
    parse_in(text, None)
}

/// `parse` の本体。`base` があれば、相対の `path` はそこから(workspaces.toml なら設定のフォルダ)。
fn parse_in(text: &str, base: Option<&Path>) -> (Vec<Workspace>, Vec<String>) {
    let mut warns = Vec::new();
    let table: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            let (line, msg) = config::toml_error(text, &e);
            let at = line.map(|l| l.to_string()).unwrap_or_default();
            warns.push(Msg::WsBadToml.fill(&[&FILE_NAME, &at, &msg]));
            return (Vec::new(), warns);
        }
    };
    let mut out: Vec<Workspace> = Vec::new();
    let items = table
        .get("workspace")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for (k, item) in items.iter().enumerate() {
        let n = k + 1;
        let Some(t) = item.as_table() else {
            warns.push(Msg::WsBadEntry.fill(&[&n]));
            continue;
        };
        let Some(name) = t
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !out.iter().any(|w| &w.name == s))
        else {
            warns.push(Msg::WsBadEntry.fill(&[&n]));
            continue;
        };
        let tables = read_tables(t.get("table"), base, &name, &mut warns);
        out.push(Workspace { name, tables });
    }
    (out, warns)
}

/// 設定のフォルダの workspaces.toml を読む。無ければ空。
pub fn load(dir: &Path) -> (Vec<Workspace>, Vec<String>) {
    match std::fs::read_to_string(dir.join(FILE_NAME)) {
        Ok(t) => parse_in(&t, Some(dir)),
        Err(_) => (Vec::new(), Vec::new()),
    }
}

/// workspaces.toml の文にする。
pub fn to_toml(list: &[Workspace]) -> String {
    let arr: Vec<toml::Value> = list
        .iter()
        .map(|w| {
            let mut t = toml::Table::new();
            t.insert("name".into(), toml::Value::String(w.name.clone()));
            let tables: Vec<toml::Value> = w
                .tables
                .iter()
                .map(|x| {
                    let mut tt = toml::Table::new();
                    tt.insert("name".into(), toml::Value::String(x.name.clone()));
                    tt.insert("path".into(), toml::Value::String(home_short(&x.path)));
                    if let Some(v) = &x.view {
                        tt.insert("view".into(), toml::Value::String(v.clone()));
                    }
                    toml::Value::Table(tt)
                })
                .collect();
            t.insert("table".into(), toml::Value::Array(tables));
            toml::Value::Table(t)
        })
        .collect();
    let mut root = toml::Table::new();
    root.insert("workspace".into(), toml::Value::Array(arr));
    toml::to_string(&root).unwrap_or_default()
}

/// ホームのフォルダの下なら `~/…` で書く(手で書いた `~` を、書き直しで失わない)。
fn home_short(p: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if let Ok(rest) = p.strip_prefix(&home) {
            if !rest.as_os_str().is_empty() {
                return format!("~/{}", rest.to_string_lossy());
            }
        }
    }
    p.to_string_lossy().into_owned()
}

/// 書き直す前に読む。読めない行があれば書かない(書き直すとその行が消えるため)。
fn load_for_edit(dir: &Path) -> io::Result<Vec<Workspace>> {
    let (list, warns) = load(dir);
    match warns.first() {
        None => Ok(list),
        Some(w) => Err(io::Error::other(Msg::WsFixFirst.fill(&[&w]))),
    }
}

/// 同じ表か(実体のパスで比べる)。
pub fn same_path(a: &Path, b: &Path) -> bool {
    let ca = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

/// 表を足す(WS-2・WS-3)。ワークスペースが無ければ作る。同じ表がもうあれば名前とビューを置き換える。
pub fn add(dir: &Path, ws: &str, table: WsTable) -> io::Result<()> {
    let mut list = load_for_edit(dir)?;
    let w = match list.iter().position(|w| w.name == ws) {
        Some(i) => &mut list[i],
        None => {
            list.push(Workspace {
                name: ws.to_string(),
                tables: Vec::new(),
            });
            list.last_mut().expect("just pushed")
        }
    };
    match w
        .tables
        .iter_mut()
        .find(|t| same_path(&t.path, &table.path))
    {
        Some(t) => *t = table.clone(),
        None => w.tables.push(table.clone()),
    }
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap_or_default();
    let new = checked(edit_add(&text, dir, ws, &table), dir, &list);
    config::write_atomic(dir, FILE_NAME, new.as_bytes())
}

/// 文字のまま書き換えた結果を読み直し、狙った並び(`want`)と違えば(手で書いた形を読み違えたとき)、
/// 並びから書き直した文字にする(コメントは消えるが、ほかの項目を壊さない)。
fn checked(new: String, dir: &Path, want: &[Workspace]) -> String {
    let (got, warns) = parse_in(&new, Some(dir));
    if warns.is_empty() && got == want {
        new
    } else {
        to_toml(want)
    }
}

/// 表を外す(`path` があれば)か、ワークスペースごと消す(WS-2・WS-3)。何も消さなければ偽。
pub fn remove(dir: &Path, ws: &str, path: Option<&Path>) -> io::Result<bool> {
    let mut list = load_for_edit(dir)?;
    let before = list.clone();
    match path {
        Some(p) => {
            if let Some(w) = list.iter_mut().find(|w| w.name == ws) {
                w.tables.retain(|t| !same_path(&t.path, p));
            }
        }
        None => list.retain(|w| w.name != ws),
    }
    if list == before {
        return Ok(false);
    }
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap_or_default();
    let new = checked(edit_remove(&text, dir, ws, path), dir, &list);
    config::write_atomic(dir, FILE_NAME, new.as_bytes())?;
    Ok(true)
}

// ---- 文字のまま書き換える(手で書いたコメント・空行・`~`・並びを残す。WS-1) ----

const WS_HEAD: &str = "[[workspace]]";
const TABLE_HEAD: &str = "[[workspace.table]]";

/// 区画: 見出しの行(start)から、中身の終わり(end。末尾のコメントと空行は含めない)まで。
#[derive(Clone, Copy)]
struct Block {
    start: usize,
    end: usize,
}

/// `lines[from..to]` の中の、見出し `head` で始まる区画(config::toml_blocks)。ワークスペースの区画は、
/// その表の見出し(`[[workspace.table]]`)では終わらない。
fn blocks(lines: &[&str], from: usize, to: usize, head: &str) -> Vec<Block> {
    let inner: &[&str] = if head == WS_HEAD { &[TABLE_HEAD] } else { &[] };
    config::toml_blocks(lines, from, to, head, inner)
        .into_iter()
        .map(|(start, end)| Block { start, end })
        .collect()
}

/// ワークスペースの区画の名前(読めなければ None)。
fn block_name(lines: &[&str], b: Block) -> Option<String> {
    let first_table = (b.start..b.end)
        .find(|&i| config::toml_header(lines[i]).as_deref() == Some(TABLE_HEAD))
        .unwrap_or(b.end);
    let (list, _) = parse(&lines[b.start..first_table].concat());
    list.into_iter().next().map(|w| w.name)
}

/// 表の区画のパス(相対は設定のフォルダから)。
fn table_path(lines: &[&str], b: Block, dir: &Path) -> Option<PathBuf> {
    let text = format!(
        "[[workspace]]\nname = \"x\"\n{}",
        lines[b.start..b.end].concat()
    );
    let (list, _) = parse_in(&text, Some(dir));
    list.into_iter()
        .next()?
        .tables
        .into_iter()
        .next()
        .map(|t| t.path)
}

/// 表1つの文字。
fn table_text(t: &WsTable) -> String {
    let q = |s: &str| toml::Value::String(s.to_string()).to_string();
    let mut s = format!(
        "{TABLE_HEAD}\nname = {}\npath = {}\n",
        q(&t.name),
        q(&home_short(&t.path))
    );
    if let Some(v) = &t.view {
        s.push_str(&format!("view = {}\n", q(v)));
    }
    s
}

/// 行の並びを文字に戻す(最後の行の改行を保つ)。
fn join(lines: Vec<String>) -> String {
    lines.concat()
}

/// 表を足すか置き換える(ワークスペースが無ければ末尾に作る)。
fn edit_add(text: &str, dir: &Path, ws: &str, t: &WsTable) -> String {
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    let owned_tail;
    if lines.last().is_some_and(|l| !l.ends_with('\n')) {
        owned_tail = format!("{}\n", lines.pop().unwrap_or_default());
        lines.push(&owned_tail);
    }
    let n = lines.len();
    let found = blocks(&lines, 0, n, WS_HEAD)
        .into_iter()
        .find(|b| block_name(&lines, *b).as_deref() == Some(ws));
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match found {
        None => {
            if !out.is_empty() {
                out.push("\n".into());
            }
            let q = toml::Value::String(ws.to_string()).to_string();
            out.push(format!("{WS_HEAD}\nname = {q}\n\n"));
            out.push(table_text(t));
        }
        Some(b) => {
            let same = blocks(&lines, b.start + 1, b.end, TABLE_HEAD)
                .into_iter()
                .find(|tb| table_path(&lines, *tb, dir).is_some_and(|p| same_path(&p, &t.path)));
            match same {
                Some(tb) => {
                    out.splice(tb.start..tb.end, [table_text(t)]);
                }
                None => {
                    out.splice(b.end..b.end, ["\n".to_string(), table_text(t)]);
                }
            }
        }
    }
    join(out)
}

/// 表を外すか(`path`)、ワークスペースの区画ごと消す。
fn edit_remove(text: &str, dir: &Path, ws: &str, path: Option<&Path>) -> String {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let n = lines.len();
    let Some(b) = blocks(&lines, 0, n, WS_HEAD)
        .into_iter()
        .find(|b| block_name(&lines, *b).as_deref() == Some(ws))
    else {
        return text.to_string();
    };
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    match path {
        None => {
            out.drain(b.start..b.end);
        }
        Some(p) => {
            for tb in blocks(&lines, b.start + 1, b.end, TABLE_HEAD)
                .into_iter()
                .rev()
            {
                if table_path(&lines, tb, dir).is_some_and(|q| same_path(&q, p)) {
                    out.drain(tb.start..tb.end);
                }
            }
        }
    }
    join(out)
}

/// フォルダの直下の、ノートのあるフォルダ(自動の表。WS-5・WS-7)。名前の順。`.base` は入れない。
pub fn auto_tables(root: &Path) -> Vec<WsTable> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<WsTable> = Vec::new();
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let p = e.path();
        // シンボリックリンクはたどらない。依存やビルドの置き場は表にしない(重く、ノートの表でもない)。
        let link = e.file_type().is_ok_and(|t| t.is_symlink());
        if link || SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        // `.base` は入れない(表のフォルダが保管庫の根になり、ほかの表と重なる。要れば書いて足す)。
        if p.is_dir() && has_notes(&p) {
            out.push(WsTable {
                name,
                path: std::fs::canonicalize(&p).unwrap_or(p),
                view: None,
            });
        }
    }
    out
}

/// 自動の表にしないフォルダ(依存・ビルドの置き場)。
const SKIP_DIRS: [&str; 6] = [
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    "__pycache__",
];

/// フォルダの下に `.md` があるか(深さ 4 まで。見る項目は 5000 まで。シンボリックリンクはたどらない)。
fn has_notes(dir: &Path) -> bool {
    let mut stack = vec![(dir.to_path_buf(), 0usize)];
    let mut seen = 0usize;
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            seen += 1;
            if seen > 5000 {
                return false;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            if e.file_type().is_ok_and(|t| t.is_symlink()) {
                continue;
            }
            let p = e.path();
            if p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md")) {
                return true;
            }
            if p.is_dir() && depth < 4 {
                stack.push((p, depth + 1));
            }
        }
    }
    false
}

/// フォルダの印を読む(WS-7)。`root` は印のある根。
pub fn read_marker(root: &Path) -> (Workspace, Vec<String>) {
    let file = root.join(MARKER_DIR).join(MARKER_FILE);
    let mut warns = Vec::new();
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    let table: toml::Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            let (line, msg) = config::toml_error(&text, &e);
            let at = line.map(|l| l.to_string()).unwrap_or_default();
            warns.push(Msg::WsBadToml.fill(&[&file.display(), &at, &msg]));
            toml::Table::new()
        }
    };
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| stem_of(root));
    let mut tables = read_tables(table.get("table"), Some(root), &name, &mut warns);
    // 印はノートと一緒に配られるので、根の外を指す表は使わない(開いただけで外のフォルダを読ませない)。
    let real_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    tables.retain(|t| {
        let real = std::fs::canonicalize(&t.path).unwrap_or_else(|_| t.path.clone());
        let inside = real.starts_with(&real_root)
            && !t
                .path
                .components()
                .any(|c| c == std::path::Component::ParentDir);
        if !inside {
            warns.push(Msg::WsMarkerOutside.fill(&[&t.path.display()]));
        }
        inside
    });
    if table.get("table").is_none() {
        tables = auto_tables(root);
    }
    (Workspace { name, tables }, warns)
}

/// 印を作る(WS-7)。既にあれば理由。作ったファイルのパス。
pub fn init(dir: &Path) -> Result<PathBuf, String> {
    let file = dir.join(MARKER_DIR).join(MARKER_FILE);
    if file.exists() {
        return Err(Msg::WsInitExists.fill(&[&file.display()]));
    }
    let name = stem_of(&std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf()));
    let text = format!(
        "# The mdgrid workspace marker (WS-7). Without [[table]], the folders with notes right under\n\
         # this folder are the tables of the workspace (add a .base with [[table]] if you want one).\n\
         name = {}\n\n\
         # [[table]]\n# name = \"Tasks\"\n# path = \"tasks\"\n",
        toml::Value::String(name)
    );
    std::fs::create_dir_all(dir.join(MARKER_DIR)).map_err(|e| e.to_string())?;
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    Ok(file)
}

/// `start` から上へたどって、最初に見つかった印の根。
pub fn find_marker(start: &Path) -> Option<PathBuf> {
    let start = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    start
        .ancestors()
        .find(|a| a.join(MARKER_DIR).join(MARKER_FILE).is_file())
        .map(Path::to_path_buf)
}

/// `start` から上へたどって、最初に見つかった検知の根(同じフォルダなら `modes` の順)。
pub fn detect(start: &Path, modes: &[Detect]) -> Option<(PathBuf, Detect)> {
    let start = std::fs::canonicalize(start).unwrap_or_else(|_| start.to_path_buf());
    for a in start.ancestors() {
        for m in modes {
            if a.join(m.marker()).exists() {
                return Some((a.to_path_buf(), *m));
            }
        }
    }
    None
}

/// 開いたもの(フォルダか .base)が表に入るか。
pub fn contains(t: &WsTable, opened: &Path) -> bool {
    let tp = std::fs::canonicalize(&t.path).unwrap_or_else(|_| t.path.clone());
    let op = std::fs::canonicalize(opened).unwrap_or_else(|_| opened.to_path_buf());
    if tp == op {
        return true;
    }
    let tdir = if tp.extension().is_some_and(|x| x == "base") {
        return false;
    } else {
        tp
    };
    let odir = if op.extension().is_some_and(|x| x == "base") {
        op.parent().map(Path::to_path_buf).unwrap_or(op)
    } else {
        op
    };
    odir.starts_with(&tdir)
}

/// 範囲を決める(WS-6)。`chosen` が無い名前なら理由。どれにも当たらなければ None(呼ぶ側で登録した表)。
pub fn resolve(
    opened: &Path,
    chosen: Option<&str>,
    apps: &[Workspace],
    modes: &[Detect],
) -> Result<Option<Scope>, String> {
    if let Some(name) = chosen {
        let w = apps
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| Msg::WsUnknown.fill(&[&name]))?;
        // 選んだワークスペースに無い表を開いたら、それを言う(範囲は選んだもの + 今の表)。
        let mut warnings = Vec::new();
        if !w.tables.iter().any(|t| contains(t, opened)) {
            warnings.push(Msg::WsNotInChosen.fill(&[&stem_of(opened), &w.name]));
        }
        return Ok(Some(Scope {
            name: w.name.clone(),
            source: Source::Chosen,
            tables: w.tables.clone(),
            warnings,
        }));
    }
    let start = if opened.extension().is_some_and(|x| x == "base") {
        opened.parent().unwrap_or(opened).to_path_buf()
    } else {
        opened.to_path_buf()
    };
    if let Some(root) = find_marker(&start) {
        let (w, warnings) = read_marker(&root);
        if !w.tables.is_empty() {
            return Ok(Some(Scope {
                name: w.name,
                source: Source::Marker,
                tables: w.tables,
                warnings,
            }));
        }
    }
    if let Some(w) = apps
        .iter()
        .find(|w| w.tables.iter().any(|t| contains(t, opened)))
    {
        return Ok(Some(Scope {
            name: w.name.clone(),
            source: Source::App,
            tables: w.tables.clone(),
            warnings: Vec::new(),
        }));
    }
    if let Some((root, how)) = detect(&start, modes) {
        let tables = auto_tables(&root);
        if !tables.is_empty() {
            return Ok(Some(Scope {
                name: stem_of(&root),
                source: Source::Detected(how),
                tables,
                warnings: Vec::new(),
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
#[path = "test_workspace_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_workspace_more_unit.rs"]
mod tests_more;

#[cfg(test)]
#[path = "test_workspace_chosen_unit.rs"]
mod tests_chosen;

#[cfg(test)]
#[path = "test_workspace_keep_unit.rs"]
mod tests_keep;
