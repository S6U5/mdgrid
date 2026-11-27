//! AI の案(ai.proposal-file): 台帳の根(印 `.mdgrid/workspace.toml` のあるフォルダ。無ければ開いたフォルダ)の
//! `.mdgrid/proposals/` に、1つの案を1つの JSON のファイルにして置く。同期の道具で何台の機械から置いても重ならない
//! ように、ファイルの名前は作った日時と乱数にし、既にあるファイルを置き換えない(`create_new`)。
//!
//! 形: `{"note": 理由, "created": "YYYY-MM-DDTHH:MM", "changes": [{"path": 根からのパス, "set": {列: 値}, "was": {列: 値}}]}`。
//! 値は `--print --format json` と同じ形(数・真偽・null・リストは JSON の型、日付と日時は文字列)。

use crate::workspace::{find_marker, MARKER_DIR};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 案の置き場の、印のフォルダの下の名前。
pub const DIR: &str = "proposals";

/// 1つのノート(行)への直し。
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// 台帳の根からのパス(CSV の台帳の行は `台帳.csv#2` のように行の位置の印つき)。
    pub path: String,
    /// 列 → 入れる値。
    pub set: serde_json::Map<String, serde_json::Value>,
    /// 列 → 案を作ったときの値(入れるときに今の値と比べる。ai.proposal-take)。
    pub was: serde_json::Map<String, serde_json::Value>,
}

/// 読んだ案。
#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    pub file: PathBuf,
    pub note: String,
    pub created: String,
    pub changes: Vec<Change>,
}

impl Proposal {
    /// 直すセルの数。
    pub fn cells(&self) -> usize {
        self.changes.iter().map(|c| c.set.len()).sum()
    }
}

/// 開いたもの(フォルダ・`.base`・ノート)から台帳の根: 印のある上のフォルダ、無ければ開いたフォルダ。
pub fn root_of(opened: &Path) -> PathBuf {
    let dir = if opened.is_file() {
        opened.parent().unwrap_or(opened).to_path_buf()
    } else {
        opened.to_path_buf()
    };
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    find_marker(&dir).unwrap_or(dir)
}

/// 案の置き場。
pub fn dir(root: &Path) -> PathBuf {
    root.join(MARKER_DIR).join(DIR)
}

/// 案を1つ書く。書いたファイルのパス。`created` は `YYYY-MM-DDTHH:MM`。
pub fn write(
    root: &Path,
    note: &str,
    created: &str,
    changes: &[Change],
) -> std::io::Result<PathBuf> {
    let d = dir(root);
    std::fs::create_dir_all(&d)?;
    let items: Vec<serde_json::Value> = changes
        .iter()
        .map(|c| {
            serde_json::json!({
                "path": c.path,
                "set": serde_json::Value::Object(c.set.clone()),
                "was": serde_json::Value::Object(c.was.clone()),
            })
        })
        .collect();
    let body = serde_json::json!({
        "note": note,
        "created": created,
        "changes": items,
    });
    let text = serde_json::to_string_pretty(&body).map_err(std::io::Error::other)? + "\n";
    let stamp: String = created.chars().filter(char::is_ascii_digit).collect();
    // 同じ分に重なっても、乱数を変えて新しい名前で作り直す(既にあるファイルは置き換えない)。
    for _ in 0..8 {
        let mut b = [0u8; 4];
        let tail = if getrandom::fill(&mut b).is_ok() {
            b.iter().map(|x| format!("{x:02x}")).collect::<String>()
        } else {
            format!("{:08x}", std::process::id())
        };
        let path = d.join(format!("{stamp}-{tail}.json"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(text.as_bytes())?;
                f.sync_all()?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "proposal name",
    ))
}

/// 置き場の案を全部読む(作った日時、次にファイルの名前の順)。読めないファイルは飛ばして名前を返す。
pub fn read_all(root: &Path) -> (Vec<Proposal>, Vec<PathBuf>) {
    let mut out = Vec::new();
    let mut bad = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir(root)) else {
        return (out, bad);
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json") && p.is_file())
        .collect();
    files.sort();
    for f in files {
        match read_one(&f) {
            Some(p) => out.push(p),
            None => bad.push(f),
        }
    }
    out.sort_by(|a, b| a.created.cmp(&b.created).then(a.file.cmp(&b.file)));
    (out, bad)
}

fn read_one(file: &Path) -> Option<Proposal> {
    let text = crate::textfile::read_text(file).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let obj = v.as_object()?;
    let changes = obj
        .get("changes")?
        .as_array()?
        .iter()
        .map(|c| {
            let c = c.as_object()?;
            Some(Change {
                path: c.get("path")?.as_str()?.to_string(),
                set: c.get("set")?.as_object()?.clone(),
                was: c
                    .get("was")
                    .and_then(serde_json::Value::as_object)
                    .cloned()
                    .unwrap_or_default(),
            })
        })
        .collect::<Option<Vec<Change>>>()?;
    Some(Proposal {
        file: file.to_path_buf(),
        note: obj
            .get("note")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        created: obj
            .get("created")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        changes,
    })
}

/// 案のファイルを消す(受けたか捨てたとき。write-back.frontmatter-only-no-body の例外)。
pub fn remove(p: &Proposal) -> std::io::Result<()> {
    std::fs::remove_file(&p.file)
}

/// 置き場が最後に変わった時刻(見回りで読み直すかを決める。無ければ None)。
pub fn stamp(root: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(dir(root)).and_then(|m| m.modified()).ok()
}
