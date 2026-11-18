//! ごみ箱(cell-edit.delete-note-*・cell-edit.trash-restore-*)。ノートは消さずに、根のフォルダの `.trash/` の下へ動かす
//! (`.trash/` は Obsidian のごみ箱と同じ置き場で、mdgrid は中を探さない)。置き場は tbd~cell-edit.delete-note-trash-place の仮。
//!
//! 形: `<根>/.trash/mdgrid/<消した時刻(UNIX 秒)>-<通し番号>/<根からの相対パス>`。1回の削除に1つのフォルダを
//! `create_dir`(既にあれば失敗)で作るので、前からごみ箱にあるファイルを置き換えることが無い。元の場所と
//! 消した時刻はパスから読めるので、別の台帳を持たない(台帳とファイルが食い違うことが無い)。

use std::io;
use std::path::{Path, PathBuf};

/// ごみ箱のフォルダの名前(根の直下)。
pub const DIR: &str = ".trash";
/// mdgrid が動かしたノートを置く、ごみ箱の中のフォルダ。
const OWN: &str = "mdgrid";

/// ごみ箱の中のノート1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trashed {
    /// 根のフォルダ。
    pub root: PathBuf,
    /// 元の場所(根からの相対。'/' 区切り)。
    pub rel: String,
    /// ごみ箱の中の今の場所。
    pub at: PathBuf,
    /// 消した時刻(UNIX 秒)。
    pub when: i64,
}

impl Trashed {
    /// 元の場所(戻す先)。
    pub fn origin(&self) -> PathBuf {
        self.rel
            .split('/')
            .fold(self.root.clone(), |p, part| p.join(part))
    }
}

/// 根からの相対('/' 区切り)。根の外・根そのものは None。
fn rel_of(path: &Path, root: &Path) -> Option<String> {
    let r = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = r
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// 空になったフォルダを、`from` から `stop` の手前まで上へ消す(空でなければそこで止まる)。
fn prune(from: &Path, stop: &Path) {
    let mut cur = Some(from);
    while let Some(d) = cur {
        if d == stop || !d.starts_with(stop) || std::fs::remove_dir(d).is_err() {
            break;
        }
        cur = d.parent();
    }
}

/// `path`(根 `root` の下のファイル)をごみ箱へ動かす。動かせなければ Err で、ファイルは元の場所のまま。
pub fn put(root: &Path, path: &Path, now: i64) -> io::Result<Trashed> {
    let rel = rel_of(path, root)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not under the root"))?;
    let base = root.join(DIR).join(OWN);
    std::fs::create_dir_all(&base)?;
    // 1回の削除に1つのフォルダ。既にある番号は使わない(前のごみを置き換えない)。
    let mut n = 0u32;
    let slot = loop {
        let slot = base.join(format!("{now}-{n}"));
        match std::fs::create_dir(&slot) {
            Ok(()) => break slot,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && n < 100_000 => n += 1,
            Err(e) => return Err(e),
        }
    };
    let at = rel.split('/').fold(slot.clone(), |p, part| p.join(part));
    let moved = at
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::rename(path, &at));
    if let Err(e) = moved {
        if let Some(parent) = at.parent() {
            prune(parent, &base);
        }
        return Err(e);
    }
    Ok(Trashed {
        root: root.to_path_buf(),
        rel,
        at,
        when: now,
    })
}

/// ごみ箱のノートを元の場所へ戻し、戻した先のパスを返す。元の場所に何かあれば、置き換えずに
/// `AlreadyExists` の Err(ごみ箱のノートもそのまま)。
pub fn restore(item: &Trashed) -> io::Result<PathBuf> {
    let to = item.origin();
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // 置き換えない移動: ハードリンクは行き先があれば失敗する(rename は黙って置き換える)。
    match std::fs::hard_link(&item.at, &to) {
        Ok(()) => {
            if let Err(e) = std::fs::remove_file(&item.at) {
                // 2つの名前が残ると読むだけのノートになるので、戻した側を外して元のままにする。
                let _ = std::fs::remove_file(&to);
                return Err(e);
            }
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => return Err(e),
        Err(_) => {
            // ハードリンクを作れない置き場: 行き先が無いことを見てから動かす。
            if to.symlink_metadata().is_ok() {
                return Err(io::Error::from(io::ErrorKind::AlreadyExists));
            }
            std::fs::rename(&item.at, &to)?;
        }
    }
    if let Some(parent) = item.at.parent() {
        prune(parent, &item.root.join(DIR).join(OWN));
    }
    Ok(to)
}

/// 根のごみ箱の中の、mdgrid が動かしたノート(新しい順。同じ時刻は元の場所の順)。
pub fn list(root: &Path) -> Vec<Trashed> {
    let base = root.join(DIR).join(OWN);
    let mut out = Vec::new();
    let Ok(slots) = std::fs::read_dir(&base) else {
        return out;
    };
    for slot in slots.flatten() {
        let name = slot.file_name().to_string_lossy().into_owned();
        let Some(when) = name.split('-').next().and_then(|s| s.parse::<i64>().ok()) else {
            continue;
        };
        let dir = slot.path();
        let mut stack = vec![dir.clone()];
        while let Some(d) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in entries.flatten() {
                let p = e.path();
                let Ok(ft) = e.file_type() else {
                    continue;
                };
                if ft.is_dir() {
                    stack.push(p);
                } else if let Some(rel) = rel_of(&p, &dir) {
                    out.push(Trashed {
                        root: root.to_path_buf(),
                        rel,
                        at: p,
                        when,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| b.when.cmp(&a.when).then_with(|| a.rel.cmp(&b.rel)));
    out
}

#[cfg(test)]
#[path = "test_trash_unit.rs"]
mod tests;
