//! ノートの集まりを読む(タスク 4)。根の決め方(BV-2)、重なりとリンクの重複の除去(BV-11)、
//! 少しずつの読み込みと中止(BV-16)、更新時刻と大きさのポーリング(BV-9)、読み直しのあとの選択(BV-10)、
//! 同期の道具の競合ファイルの印(WB-13)。

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// 探さない下のフォルダか(BV-1・BV-23): 名前が `.` で始まる(`.obsidian`・`.git`・`.trash` を含む隠しフォルダ。
/// Obsidian も保管庫に入れない)か `node_modules`(コードのリポの依存のパッケージ)。渡したフォルダそのものは探す。
fn skipped_dir(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules"
}

/// 渡したフォルダから保管庫の根を決める(BV-2): `.obsidian/` を持つ最寄りの上のフォルダ、無ければ渡したフォルダ。
/// 実体のパス(canonicalize)にし、同じ根・ほかの根の中に入る根は1つにまとめる。読めないパスは Err。
pub fn roots(folders: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for folder in folders {
        let real = folder.canonicalize()?;
        if !real.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                crate::i18n::Msg::NotAFolder.fill(&[&folder.display()]),
            ));
        }
        fs::read_dir(&real)?;
        let root = real
            .ancestors()
            .find(|a| a.join(".obsidian").is_dir())
            .map(Path::to_path_buf)
            .unwrap_or(real);
        found.push(root);
    }
    found.sort();
    found.dedup();
    let mut out: Vec<PathBuf> = Vec::new();
    for r in found {
        // 並べた順では、親は子より先に来る
        if !out.iter().any(|o| r.starts_with(o)) {
            out.push(r);
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub mtime: SystemTime,
    pub len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// 実体のパス(canonicalize 済み。BV-11 の重複の除去の鍵)
    pub path: PathBuf,
    /// 表示用: 根からの相対パス('/' 区切り)
    pub rel: String,
    /// 最後に読んだ中身
    pub bytes: Vec<u8>,
    /// 最後に読んだときの更新時刻と大きさ
    pub stamp: Stamp,
    /// 同期の道具の競合ファイル(名前に `.sync-conflict-` か `(conflicted copy)`)。WB-13
    pub conflict: bool,
    /// 見つけたときの道筋のパス(リンクを辿る前。根の下)。行の絞り込み(BV-1)はこれで見る
    pub found: Vec<PathBuf>,
    /// ハードリンクの数(nlink)。2以上なら読むだけ(WB-5)
    pub links: u64,
    /// 利用者が書けるか(`writeback::can_write`)。偽なら読むだけ(WB-5)
    pub writable: bool,
    /// 最後に見たときの権限の印(mode・持ち主・グループ)。chmod・chown は更新時刻を変えないので、poll はこれで見て
    /// writable を見直す
    pub perm: (u32, u32, u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub loaded: usize,
    pub total: Option<usize>,
    pub done: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Poll {
    pub changed: Vec<PathBuf>,
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

/// 探して見つけた、まだ読んでいないノート。
struct Pending {
    real: PathBuf,
    rel: String,
    /// 見つけた道筋のパス
    found: Vec<PathBuf>,
}

/// フォルダの中を1回に見る項目の数の上限(大きなフォルダ1つで止まらないように)。
const DIR_CHUNK: usize = 64;

/// 開いて読んでいる途中のフォルダ。
struct OpenDir {
    entries: fs::ReadDir,
    /// フォルダの実体のパス
    dir: PathBuf,
    root: PathBuf,
    /// 根からの相対パス(見つけた道筋)
    rel_dir: String,
}

/// 根の下の探索の状態(少しずつ進める)。
struct Walk {
    /// (フォルダの実体のパス, 根, 根からの相対パス)
    dirs: VecDeque<(PathBuf, PathBuf, String)>,
    current: Option<OpenDir>,
    visited_dirs: HashSet<PathBuf>,
    seen_files: HashSet<PathBuf>,
    /// 2度目以降に見つけた道筋(実体のパス, 道筋のパス)。Scan が Pending かノートに移す
    again: Vec<(PathBuf, PathBuf)>,
    /// 隠しフォルダでも探すフォルダ(起動の引数に渡したフォルダの実体のパス。BV-23)。
    keep: Vec<PathBuf>,
}

impl Walk {
    fn new(roots: &[PathBuf], keep: &[PathBuf]) -> Walk {
        let mut w = Walk {
            dirs: VecDeque::new(),
            current: None,
            visited_dirs: HashSet::new(),
            seen_files: HashSet::new(),
            again: Vec::new(),
            keep: keep.to_vec(),
        };
        for r in roots {
            if w.visited_dirs.insert(r.clone()) {
                w.dirs.push_back((r.clone(), r.clone(), String::new()));
            }
        }
        w
    }

    fn finished(&self) -> bool {
        self.current.is_none() && self.dirs.is_empty()
    }

    /// フォルダの中の項目を最大 `DIR_CHUNK` 個見て、見つけたノートを `out` に足す。
    fn step(&mut self, out: &mut VecDeque<Pending>) {
        let mut cur = match self.current.take() {
            Some(c) => c,
            None => {
                let Some((dir, root, rel_dir)) = self.dirs.pop_front() else {
                    return;
                };
                let Ok(entries) = fs::read_dir(&dir) else {
                    return;
                };
                OpenDir {
                    entries,
                    dir,
                    root,
                    rel_dir,
                }
            }
        };
        for _ in 0..DIR_CHUNK {
            let Some(entry) = cur.entries.next() else {
                return; // 読み終えた(current は空のまま)
            };
            let Ok(entry) = entry else {
                continue;
            };
            self.visit(&cur, &entry, out);
        }
        self.current = Some(cur);
    }

    fn visit(&mut self, cur: &OpenDir, entry: &fs::DirEntry, out: &mut VecDeque<Pending>) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().into_owned();
        let walked = cur.dir.join(&name);
        let Ok(ft) = entry.file_type() else {
            return;
        };
        // シンボリックリンクは辿る(BV-11)。辿れないリンクは無視する。
        // リンクでなければ、親が実体のパスなので walked がそのまま実体のパス
        let (is_dir, is_file) = if ft.is_symlink() {
            match fs::metadata(&walked) {
                Ok(m) => (m.is_dir(), m.is_file()),
                Err(_) => return,
            }
        } else {
            (ft.is_dir(), ft.is_file())
        };
        let real_of = |w: &Path| {
            if ft.is_symlink() {
                w.canonicalize().ok()
            } else {
                Some(w.to_path_buf())
            }
        };
        let rel = if cur.rel_dir.is_empty() {
            name_str.clone()
        } else {
            format!("{}/{}", cur.rel_dir, name_str)
        };
        if is_dir {
            let Some(real) = real_of(&walked) else {
                return;
            };
            // 渡したフォルダとその上(根から渡したフォルダまでの道)は、隠しフォルダでも探す(BV-23)。
            if skipped_dir(&name_str) && !self.keep.iter().any(|k| k.starts_with(&real)) {
                return;
            }
            // 同じ実体のフォルダは1度だけ(リンクの輪もここで止まる)
            if self.visited_dirs.insert(real.clone()) {
                self.dirs.push_back((real, cur.root.clone(), rel));
            }
        } else if is_file && is_markdown(&walked) {
            let Some(real) = real_of(&walked) else {
                return;
            };
            let route = cur.root.join(&rel);
            if self.seen_files.insert(real.clone()) {
                let rel = rel_of(&real, &cur.root).unwrap_or(rel);
                out.push_back(Pending {
                    real,
                    rel,
                    found: vec![route],
                });
            } else {
                self.again.push((real, route));
            }
        }
    }
}

fn is_markdown(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md"))
}

fn rel_of(real: &Path, root: &Path) -> Option<String> {
    let r = real.strip_prefix(root).ok()?;
    let parts: Vec<String> = r
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// 同期の道具の競合ファイル(WB-13)。Syncthing は `.sync-conflict-`、Dropbox は
/// `note (名前's conflicted copy 2026-09-30).md` の形なので、大文字小文字を区別せずに含むかで見る。
fn is_conflict(p: &Path) -> bool {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    name.contains(".sync-conflict-") || name.contains("conflicted copy")
}

fn stamp_of(meta: &fs::Metadata) -> Stamp {
    Stamp {
        mtime: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        len: meta.len(),
    }
}

#[cfg(unix)]
fn links_of(meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.nlink()
}

#[cfg(not(unix))]
fn links_of(_meta: &fs::Metadata) -> u64 {
    1
}

#[cfg(unix)]
fn perm_of(meta: &fs::Metadata) -> (u32, u32, u32) {
    use std::os::unix::fs::MetadataExt;
    (meta.mode(), meta.uid(), meta.gid())
}

#[cfg(not(unix))]
fn perm_of(meta: &fs::Metadata) -> (u32, u32, u32) {
    (meta.permissions().readonly() as u32, 0, 0)
}

/// 道筋を重ねずに足す。
fn add_route(found: &mut Vec<PathBuf>, route: PathBuf) {
    if !found.contains(&route) {
        found.push(route);
    }
}

/// 更新時刻を先に取ってから読む。読む間に変わっても、次の poll で時刻のずれとして見つかる。
fn read_note(real: PathBuf, rel: String, found: Vec<PathBuf>) -> io::Result<Note> {
    let meta = fs::metadata(&real)?;
    let bytes = fs::read(&real)?;
    let stamp = Stamp {
        mtime: stamp_of(&meta).mtime,
        len: bytes.len() as u64,
    };
    let conflict = is_conflict(&real);
    let writable = crate::writeback::can_write(&real);
    Ok(Note {
        path: real,
        rel,
        bytes,
        stamp,
        conflict,
        found,
        links: links_of(&meta),
        writable,
        perm: perm_of(&meta),
    })
}

/// poll 1回で、追加と消えたリンクを探すのに使う時間の上限。超えたら次の poll で続きから進める。
const POLL_SCAN_TIME: Duration = Duration::from_millis(15);
/// 時間を確かめる間に進める件数(フォルダの項目 `DIR_CHUNK` 個・ノート1つを1件と数える)。
const POLL_SCAN_STEP: usize = 4;

/// 探索と、見つけてまだ読んでいないノート。読み込み(load)と poll の探し直しの両方で使う。
struct Scan {
    walk: Walk,
    pending: VecDeque<Pending>,
    /// 見つけたノートの数
    found: usize,
    /// 見つけたが読めなかったノートの数
    failed: usize,
}

impl Scan {
    fn new(roots: &[PathBuf], keep: &[PathBuf]) -> Scan {
        Scan {
            walk: Walk::new(roots, keep),
            pending: VecDeque::new(),
            found: 0,
            failed: 0,
        }
    }

    fn finished(&self) -> bool {
        self.walk.finished() && self.pending.is_empty()
    }

    /// 最大 `budget` 件(フォルダの項目を `DIR_CHUNK` 個まで見る1回、ノートを読む1回をそれぞれ1件)進め、読めたノートを返す。
    /// `skip` が真のノート(既に持っているもの)は読まずに飛ばし、件数に数えない。
    /// 2つ目の戻り値は、既に読んだノートを別の道筋でも見つけたもの(実体のパス, 道筋のパス)。
    fn advance(
        &mut self,
        budget: usize,
        skip: impl Fn(&Path) -> bool,
    ) -> (Vec<Note>, Vec<(PathBuf, PathBuf)>) {
        let mut out: Vec<Note> = Vec::new();
        let mut late = Vec::new();
        let mut used = 0;
        while used < budget {
            if let Some(p) = self.pending.pop_front() {
                if skip(&p.real) {
                    continue;
                }
                used += 1;
                match read_note(p.real, p.rel, p.found) {
                    Ok(note) => out.push(note),
                    Err(_) => self.failed += 1,
                }
            } else if !self.walk.finished() {
                used += 1;
                let before = self.pending.len();
                self.walk.step(&mut self.pending);
                self.found += self.pending.len() - before;
                for (real, route) in std::mem::take(&mut self.walk.again) {
                    if let Some(p) = self.pending.iter_mut().find(|p| p.real == real) {
                        add_route(&mut p.found, route);
                    } else if let Some(n) = out.iter_mut().find(|n| n.path == real) {
                        add_route(&mut n.found, route);
                    } else {
                        late.push((real, route));
                    }
                }
            } else {
                break;
            }
        }
        (out, late)
    }
}

pub struct Vault {
    roots: Vec<PathBuf>,
    /// 隠しフォルダでも探すフォルダ(BV-23)。
    keep: Vec<PathBuf>,
    /// path の昇順
    notes: Vec<Note>,
    /// 最初の読み込み。終わるか中止したら None
    load: Option<Scan>,
    /// 最初の読み込みで読めたノートの数の見込み(Progress の total)
    total: Option<usize>,
    cancelled: bool,
    paused: bool,
    /// poll での探し直し(追加と、根の外を指すリンクが消えたことを見つける)
    rescan: Option<Scan>,
    /// 大きな保管庫で、次の poll が更新時刻を見始めるノートの位置(BV-9 の順に回す見回り)。
    cursor: usize,
}

/// 1回の poll で更新時刻を見るノートの数の上限(BV-9)。これより多い保管庫は順に回して全部を見る。
pub const STAT_BUDGET: usize = 2000;

impl Vault {
    /// まだ何も読まない(BV-16: 画面を先に出す)。`.md` を再帰で探す。隠しフォルダと `node_modules/` の下は探さない(BV-23)。
    /// シンボリックリンクは辿り、実体のパスで1つにする(BV-11)。リンクの輪は無視する。
    pub fn open(roots: Vec<PathBuf>) -> Vault {
        Vault::open_keeping(roots, Vec::new())
    }

    /// `open` と同じ。`keep`(起動の引数に渡したフォルダの実体のパス)とその上のフォルダは、隠しフォルダでも
    /// 探す(BV-23: 保管庫の根から探すので、渡した隠しフォルダを飛ばさないように)。
    pub fn open_keeping(roots: Vec<PathBuf>, keep: Vec<PathBuf>) -> Vault {
        let load = Scan::new(&roots, &keep);
        Vault {
            keep,
            roots,
            notes: Vec::new(),
            load: Some(load),
            total: None,
            cancelled: false,
            paused: false,
            rescan: None,
            cursor: 0,
        }
    }

    fn progress(&self) -> Progress {
        let total = match &self.load {
            Some(s) if !s.walk.finished() => None,
            Some(s) => Some(s.found - s.failed),
            None => self.total,
        };
        Progress {
            loaded: self.notes.len(),
            total,
            done: self.load.is_none(),
        }
    }

    /// 最大 `budget` 件を読む。フォルダの項目を `DIR_CHUNK` 個まで見る1回も1件と数える。読むたびに `notes()` が増える(BV-16)。
    pub fn load(&mut self, budget: usize) -> Progress {
        if let Some(mut scan) = self.load.take() {
            let (notes, late) = scan.advance(budget, |_| false);
            for n in notes {
                insert(&mut self.notes, n);
            }
            self.add_routes(late);
            if scan.finished() {
                self.total = Some(scan.found - scan.failed);
            } else {
                self.load = Some(scan);
            }
        }
        self.progress()
    }

    /// 読み込みを止める。読んだ分は残り、以後 `load` は何もしない(BV-16)。
    pub fn cancel(&mut self) {
        if self.load.take().is_some() {
            self.cancelled = true;
            self.total = Some(self.notes.len());
        }
    }

    /// 根(実体のパス)。
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    /// path の昇順
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn note(&self, path: &Path) -> Option<&Note> {
        find(&self.notes, path).ok().map(|i| &self.notes[i])
    }

    /// 更新時刻と大きさを見て、変わったノートだけ読み直す(BV-9)。止めている間(`pause(true)`)は何もせず空を返す。
    ///
    /// 追加と、根の外を指すリンクが消えたことは、探し直しで見つける。探し直しは1回の poll で
    /// `POLL_SCAN_TIME` までしか進めず、次の poll で続きから進める。一巡したら、見つからなかった
    /// ノートを removed に入れ、次の poll で新しく一巡を始める。読み込みの途中と中止のあとは探し直さない
    /// (まだ読んでいない・読むのを止めたノートを足さない)。
    pub fn poll(&mut self) -> Poll {
        let mut out = Poll::default();
        if self.paused {
            return out;
        }

        // 既に持つノートの変化(stat)。多ければ STAT_BUDGET 個ずつ順に回す(何もしていない間の CPU を抑える)。
        let n = self.notes.len();
        let (start, count) = if n > STAT_BUDGET {
            (self.cursor % n, STAT_BUDGET)
        } else {
            (0, n)
        };
        self.cursor = if n == 0 { 0 } else { (start + count) % n };
        let mut gone = Vec::new();
        for i in (0..count).map(|k| (start + k) % n.max(1)) {
            let note = &mut self.notes[i];
            match fs::metadata(&note.path) {
                Ok(meta) if meta.is_file() => {
                    if stamp_of(&meta) != note.stamp {
                        match read_note(note.path.clone(), note.rel.clone(), note.found.clone()) {
                            Ok(n) => {
                                *note = n;
                                out.changed.push(note.path.clone());
                            }
                            Err(_) => gone.push(i),
                        }
                    } else if perm_of(&meta) != note.perm {
                        // 権限だけが変わった(WB-5)。中身は同じなので changed には入れない(入れると、ためた変更の
                        // ある行が「外で変更」になる)。セルの lock は writable から毎回作るので、次の描画で出る。
                        note.perm = perm_of(&meta);
                        note.writable = crate::writeback::can_write(&note.path);
                    }
                }
                _ => gone.push(i),
            }
        }
        gone.sort_unstable();
        for i in gone.into_iter().rev() {
            out.removed.push(self.notes.remove(i).path);
        }

        // 追加と消えたリンク(少しずつの探し直し)
        if !self.cancelled && self.load.is_none() {
            let mut scan = self
                .rescan
                .take()
                .unwrap_or_else(|| Scan::new(&self.roots, &self.keep));
            let started = Instant::now();
            while !scan.finished() && started.elapsed() < POLL_SCAN_TIME {
                let notes = &self.notes;
                let (added, late) = scan.advance(POLL_SCAN_STEP, |p| find(notes, p).is_ok());
                for n in added {
                    out.added.push(n.path.clone());
                    insert(&mut self.notes, n);
                }
                self.add_routes(late);
            }
            if scan.finished() {
                let seen = &scan.walk.seen_files;
                let mut i = 0;
                while i < self.notes.len() {
                    if seen.contains(&self.notes[i].path) {
                        i += 1;
                    } else {
                        out.removed.push(self.notes.remove(i).path);
                    }
                }
                self.total = Some(scan.found - scan.failed);
            } else {
                self.rescan = Some(scan);
            }
            out.added.sort();
        }
        out.removed.sort();
        out.changed.sort();
        out
    }

    pub fn pause(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// 1つのノートを今すぐ読み直す(SR-8・WB-16 の「外の変更の上に書く」)。
    pub fn reload(&mut self, path: &Path) -> io::Result<()> {
        let real = path.canonicalize()?;
        let (rel, found) = match find(&self.notes, &real) {
            Ok(i) => (self.notes[i].rel.clone(), self.notes[i].found.clone()),
            Err(_) => {
                let under_root = self.roots.iter().find_map(|r| rel_of(&real, r));
                let found = if under_root.is_some() {
                    vec![real.clone()]
                } else {
                    Vec::new()
                };
                let rel = under_root.unwrap_or_else(|| real.to_string_lossy().into_owned());
                (rel, found)
            }
        };
        let note = read_note(real, rel, found)?;
        insert(&mut self.notes, note);
        Ok(())
    }

    /// 自分で書いたあと、書いたバイトと基準でノートを置き換える(読み直さない。WB-12。その後の外の変更は次の poll で見つかる)。
    /// 持っていないノートなら何もしない。
    pub fn replace(&mut self, path: &Path, bytes: Vec<u8>, stamp: Stamp) {
        if let Ok(i) = find(&self.notes, path) {
            let note = &mut self.notes[i];
            note.bytes = bytes;
            note.stamp = stamp;
        }
    }

    /// 既に読んだノートを別の道筋でも見つけた分を足す。
    fn add_routes(&mut self, late: Vec<(PathBuf, PathBuf)>) {
        let mut by_real: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        for (real, route) in late {
            by_real.entry(real).or_default().push(route);
        }
        for (real, routes) in by_real {
            if let Ok(i) = find(&self.notes, &real) {
                for r in routes {
                    add_route(&mut self.notes[i].found, r);
                }
            }
        }
    }
}

fn find(notes: &[Note], path: &Path) -> Result<usize, usize> {
    notes.binary_search_by(|n| n.path.as_path().cmp(path))
}

fn insert(notes: &mut Vec<Note>, note: Note) {
    match find(notes, &note.path) {
        Ok(i) => notes[i] = note,
        Err(i) => notes.insert(i, note),
    }
}

/// 読み直しのあとの選択(BV-10): 同じ行が残ればその位置、消えていれば元の位置に最も近い行(同じ添字、はみ出せば末尾)。行が0なら None。
pub fn reselect(old: &[PathBuf], selected: usize, new: &[PathBuf]) -> Option<usize> {
    if new.is_empty() {
        return None;
    }
    if let Some(p) = old.get(selected) {
        if let Some(i) = new.iter().position(|n| n == p) {
            return Some(i);
        }
    }
    Some(selected.min(new.len() - 1))
}
