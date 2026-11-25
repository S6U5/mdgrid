//! 最近開いた表(cli.recent-*): 画面で開いた表(フォルダか `.base` と、そのとき見ていたビュー)を、新しい順に
//! 20 件まで覚える。置き場は見た目の状態の置き場(screen.state-store)の `recent.toml` で、ノートのフォルダには
//! 書かない。画面の側(src/reopen.rs)が表かビューを移るたびに書き、パレットの「最近開いた表」・引数なしの起動の
//! 一覧・`mdgrid --recent` が読む。パスが無くなった表は、読むときに覚えた並びから外す(cli.recent-missing)。

use crate::workspace;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// 覚えるファイルの名前(見た目の状態の置き場の直下)。
pub const FILE_NAME: &str = "recent.toml";
/// 覚える数の上限(cli.recent-record)。
pub const LIMIT: usize = 20;
/// 引数なしの起動の一覧に並べる数(cli.recent-in-launch-list)。
pub const LAUNCH_LIMIT: usize = 5;

/// 覚えた表1つ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recent {
    /// 表のパス(フォルダか `.base` か `.csv`・`.tsv`。実体の絶対パス)。
    pub path: PathBuf,
    /// そのとき見ていたビューの名前(既定の表だけなら無し)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// 最後に開いた日時(地域の時刻の `YYYY-MM-DD HH:MM`)。
    #[serde(default)]
    pub at: String,
}

impl Recent {
    /// `mdgrid --recent` の1行(cli.recent-cli): 「パス(タブ)ビューの名前(タブ)最後に開いた日時」。
    pub fn line(&self) -> String {
        format!(
            "{}\t{}\t{}",
            self.path.display(),
            self.view.as_deref().unwrap_or_default(),
            self.at
        )
    }

    /// 一覧に見せる表の名前: 表のある所の印の `[[table]]` で名前を付けた表ならその `name`(引き出しと同じ。同じパスの
    /// 表がいくつもあればビューの合うもの)、無ければフォルダか `.base` の名前。
    pub fn name(&self) -> String {
        let start = if self.path.is_dir() {
            Some(self.path.as_path())
        } else {
            self.path.parent()
        };
        let tables = start
            .and_then(workspace::find_marker)
            .map(|root| workspace::read_marker(&root).0.tables)
            .unwrap_or_default();
        crate::resume::table_name(
            std::slice::from_ref(&self.path),
            &tables,
            self.view.as_deref(),
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct File {
    #[serde(default)]
    table: Vec<Recent>,
}

/// 覚えた並び(新しい順)。無い・読めなければ空(見た目の状態と同じく、消えても困らない)。
pub fn load(state_dir: &Path) -> Vec<Recent> {
    crate::textfile::read_text(state_dir.join(FILE_NAME))
        .ok()
        .and_then(|t| toml::from_str::<File>(&t).ok())
        .map(|f| f.table)
        .unwrap_or_default()
}

fn save(state_dir: &Path, list: &[Recent]) -> io::Result<()> {
    let text = toml::to_string(&File {
        table: list.to_vec(),
    })
    .map_err(|e| io::Error::other(e.to_string()))?;
    crate::config::write_atomic(state_dir, FILE_NAME, text.as_bytes())
}

/// 実体の絶対パス(印に中身を書いた表のように実体が無ければ、絶対パスのまま)。
fn real(p: &Path) -> PathBuf {
    std::fs::canonicalize(p)
        .or_else(|_| std::path::absolute(p))
        .unwrap_or_else(|_| p.to_path_buf())
}

/// 表を開いた(cli.recent-record): 同じ表を外してから先頭に置き、上限を超えた古いものを捨てる。
pub fn record(state_dir: &Path, path: &Path, view: Option<&str>, at: &str) -> io::Result<()> {
    let mut list = load(state_dir);
    list.retain(|r| !workspace::same_path(&r.path, path));
    list.insert(
        0,
        Recent {
            path: real(path),
            view: view.map(str::to_string),
            at: at.to_string(),
        },
    );
    list.truncate(LIMIT);
    save(state_dir, &list)
}

/// 一覧と `--recent` に出す並び(cli.recent-missing): パスがまだ在る表だけ。無くなった表があれば、覚えた並びから
/// 外して書き直す(書けなくても出す並びは同じ)。
pub fn present(state_dir: &Path) -> Vec<Recent> {
    let list = load(state_dir);
    let n = list.len();
    let keep: Vec<Recent> = list
        .into_iter()
        .filter(|r| workspace::openable(&r.path))
        .collect();
    if keep.len() != n {
        let _ = save(state_dir, &keep);
    }
    keep
}

/// 日時の文字(`YYYY-MM-DD HH:MM`)。`now_utc` は UNIX 秒、`offset` は地域の時差(秒)。
pub fn stamp(now_utc: i64, offset: i64) -> String {
    let local = now_utc + offset;
    let secs = local.rem_euclid(86_400);
    format!(
        "{} {:02}:{:02}",
        crate::types::format_date(local.div_euclid(86_400)),
        secs / 3600,
        secs % 3600 / 60
    )
}

/// 今の地域の時刻の日時の文字(時差は起動の最初に決めたもの。clock::set_local_offset)。
pub fn now_stamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));
    stamp(now, crate::clock::local_offset())
}

#[cfg(test)]
#[path = "test_recent_unit.rs"]
mod tests;
