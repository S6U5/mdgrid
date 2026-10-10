//! CSV・TSV の読み込み口(SC-15〜SC-17)。1つのファイルの行を表の行にする。
//! 行の鍵は「実体のパス#データの何行目か(1から)」。値は文字として読み、数・真偽に見えるものはその型にする
//! (先頭に 0 のある数(`007`)は文字のまま。見せ方が変わらないように)。列の型は値から推し量る。
//! 書き戻しは `csvfile::set` で直した値のバイトだけを置き換える。全部の行が同じファイルなので、書く単位(`unit`)は
//! ファイル1つ: 保存の確認と保存は、直した行をまとめて1回で書く(基準が今のファイルと違えば書かない。WB-4)。

use crate::csvfile::{self, CsvFile};
use crate::frontmatter::Value;
use crate::i18n::Msg;
use crate::source::NewValue;
use crate::source::{Cell, ColumnKind, FileInfo, RowId, Source, Stamp};
use crate::types;
use crate::vault::Progress;
use crate::writeback::{self, Edit, EditError, SaveError};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Csv {
    path: PathBuf,
    bytes: Vec<u8>,
    file: CsvFile,
    stamp: Stamp,
    /// データの行ごとの、変わらない行の番号(行の鍵の `#` の後ろ)。行の位置でなく番号で引くので、外で行が
    /// 足されたり消えたりしても、直しを別の行に当てない(外で変わったときは行の中身で引き継ぐ。`carried`)。
    ids: Vec<u64>,
    /// 次に配る行の番号。
    next: u64,
    paused: bool,
}

fn stamp_of(path: &Path, bytes: &[u8]) -> Stamp {
    let mtime = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(UNIX_EPOCH);
    Stamp {
        mtime,
        len: bytes.len() as u64,
        hash: writeback::sha256(bytes),
    }
}

/// セルの文字を値にする。空は Null、数・真偽に見えるものはその型(先頭が 0 の数は文字のまま)。
/// 列の id(見出しの名前)。空の名前は `#列の番号`、重なる名前は2つ目から `名前#列の番号`(1から)にして、どの列も
/// 1つの id で引けるようにする(同じ名前の列の値を取り違えて書かないため)。
fn column_ids(header: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(header.len());
    for (i, h) in header.iter().enumerate() {
        let id = if h.is_empty() {
            format!("#{}", i + 1)
        } else if out.contains(h) {
            format!("{h}#{}", i + 1)
        } else {
            h.clone()
        };
        out.push(id);
    }
    out
}

/// 列の数が見出しと合わない行の理由(読むだけ)。
fn width_lock(file: &CsvFile, i: usize) -> Option<String> {
    let n = file.records.get(i)?.fields.len();
    let cols = file.header.len();
    (n != cols).then(|| Msg::CsvColumnCount.fill(&[&n, &cols]))
}

fn value_of(s: &str) -> Value {
    if s.is_empty() {
        return Value::Null;
    }
    match s {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    let lead_zero = s.len() > 1 && s.starts_with('0') && !s.starts_with("0.");
    if !lead_zero && !s.starts_with('+') && s.trim() == s {
        if let Ok(i) = s.parse::<i64>() {
            return Value::Int(i);
        }
        if s.contains('.') {
            if let Ok(f) = s.parse::<f64>() {
                if f.is_finite() {
                    return Value::Float(f);
                }
            }
        }
    }
    Value::Str(s.to_string())
}

/// 書く文字。リストは `, ` でつなぐ。名前の変更・削除のような列の操作は CSV では受けない(SC-17)。
fn text_of(v: &NewValue) -> Result<String, EditError> {
    Ok(match v {
        NewValue::Null => String::new(),
        NewValue::Str(s) | NewValue::Date(s) => s.clone(),
        NewValue::Bool(b) => b.to_string(),
        NewValue::Int(i) => i.to_string(),
        // 小数の値は小数の形で書く(`2.0` を `2` にして整数に見せない)。
        NewValue::Float(f) => {
            let s = f.to_string();
            if s.contains(['.', 'e', 'E']) || !f.is_finite() {
                s
            } else {
                format!("{s}.0")
            }
        }
        NewValue::List(items) => items.join(", "),
        NewValue::RenameKey(_) | NewValue::DeleteKey => {
            return Err(EditError::NotEditable(Msg::CsvNoKeyOps.text().into()))
        }
    })
}

impl Csv {
    /// この読み込み口で開くファイルか(`.csv`・`.tsv`。拡張子の大文字小文字を問わない)。
    pub fn handles(path: &Path) -> bool {
        path.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("csv") || e.eq_ignore_ascii_case("tsv"))
    }

    /// ファイルを読んで開く。読めなければ理由。
    pub fn open(path: &Path) -> Result<Csv, String> {
        let path = path
            .canonicalize()
            .map_err(|e| Msg::CannotRead.fill(&[&e]))?;
        let bytes = std::fs::read(&path).map_err(|e| Msg::CannotRead.fill(&[&e]))?;
        let file = csvfile::parse(&bytes, csvfile::delim_of(&path))?;
        let stamp = stamp_of(&path, &bytes);
        let n = file.records.len() as u64;
        Ok(Csv {
            path,
            bytes,
            file,
            stamp,
            ids: (1..=n).collect(),
            next: n + 1,
            paused: false,
        })
    }

    /// 行の鍵の行の番号。
    fn id_of(&self, row: &RowId) -> Option<u64> {
        let (p, n) = row.0.rsplit_once('#')?;
        if Path::new(p) != self.path {
            return None;
        }
        n.parse::<u64>().ok()
    }

    /// 行の鍵からデータの行の添字。
    fn index(&self, row: &RowId) -> Option<usize> {
        let id = self.id_of(row)?;
        self.ids.iter().position(|&k| k == id)
    }

    fn row_id(&self, i: usize) -> RowId {
        RowId(format!("{}#{}", self.path.display(), self.ids[i]))
    }

    /// 外で変わった内容(`bytes` を読んだ `file`)の行ごとに、今の行の番号を引き継ぐ: 1文字も変わっていない行
    /// (行のバイトが同じ)だけが同じ番号で、ほかは None(中身の変わった行の直しは、どの行にも当てない)。
    /// 同じ中身の行が並ぶときは、前から順に引き継ぐ。
    fn carried(&self, bytes: &[u8], file: &CsvFile) -> Vec<Option<u64>> {
        let mut old: HashMap<&[u8], std::collections::VecDeque<u64>> = HashMap::new();
        for (r, &id) in self.file.records.iter().zip(&self.ids) {
            old.entry(&self.bytes[r.start..r.end])
                .or_default()
                .push_back(id);
        }
        file.records
            .iter()
            .map(|r| {
                old.get_mut(&bytes[r.start..r.end])
                    .and_then(|q| q.pop_front())
            })
            .collect()
    }

    fn col_index(&self, col: &str) -> Option<usize> {
        column_ids(&self.file.header).iter().position(|h| h == col)
    }

    /// 列の数が合わない行の理由。
    fn row_lock(&self, i: usize) -> Option<String> {
        width_lock(&self.file, i)
    }

    fn values(&self, col: &str) -> impl Iterator<Item = Value> + '_ {
        let c = self.col_index(col);
        self.file
            .records
            .iter()
            .filter_map(move |r| c.and_then(|c| r.fields.get(c)).map(|f| value_of(&f.value)))
    }

    /// 行ごとの edits を `bytes` に全部当てたバイト。行と列は `bytes` を1度読んだものから引き(外で変わった内容の
    /// 差分を見せるときも、見せた差分のとおりに書く。WB-16)、全部を1度に置き換える(前の置き換えで後ろの位置がずれない)。
    fn applied(&self, bytes: &[u8], edits: &[(RowId, Vec<Edit>)]) -> Result<Vec<u8>, EditError> {
        let file = csvfile::parse(bytes, self.file.delim).map_err(EditError::NotEditable)?;
        let ids = column_ids(&file.header);
        // 行は番号で引く。読んだ内容と違うバイト(外で変わった)なら、行の中身で番号を引き継いだ位置。
        let rows: Vec<Option<u64>> = if bytes == self.bytes.as_slice() {
            self.ids.iter().copied().map(Some).collect()
        } else {
            self.carried(bytes, &file)
        };
        let mut sets = Vec::new();
        for (row, es) in edits {
            let id = self
                .id_of(row)
                .ok_or_else(|| EditError::NotEditable(Msg::RowNotLoaded.text().into()))?;
            let i = rows
                .iter()
                .position(|&k| k == Some(id))
                .ok_or_else(|| EditError::NotEditable(Msg::CsvRowChanged.text().into()))?;
            if let Some(reason) = width_lock(&file, i) {
                return Err(EditError::NotEditable(reason));
            }
            for e in es {
                let c = ids
                    .iter()
                    .position(|h| *h == e.key)
                    .ok_or_else(|| EditError::NotEditable(Msg::CsvNoSuchColumn.fill(&[&e.key])))?;
                sets.push((i, c, text_of(&e.value)?));
            }
        }
        csvfile::set_many(bytes, &file, &sets)
            .ok_or_else(|| EditError::NotEditable(Msg::RowNotLoaded.text().into()))
    }

    /// 今のファイルが基準と同じときだけ、バイトを書いて読み直す。
    fn write(
        &mut self,
        base: &Stamp,
        f: impl FnOnce(&Csv, &[u8]) -> Result<Vec<u8>, EditError>,
    ) -> Result<Stamp, SaveError> {
        let disk = self.disk()?;
        if writeback::sha256(&disk) != base.hash || base.hash != self.stamp.hash {
            return Err(SaveError::Changed);
        }
        let out = f(self, &disk).map_err(SaveError::Edit)?;
        let dir = self.path.parent().unwrap_or(Path::new("."));
        let name = self.name();
        crate::config::write_atomic(dir, &name, &out)?;
        self.adopt(out, false)
            .map_err(|e| SaveError::Edit(EditError::NotEditable(e)))?;
        Ok(self.stamp)
    }

    /// 今のディスクのバイト(この読み込み口の知らない基準なら、ほかで変わった)。
    fn disk(&self) -> io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }

    /// 新しい内容を読んだものにする。`outside` なら外で変わった内容で、行の番号は行の中身で引き継ぎ(`carried`)、
    /// 引き継げない行には新しい番号を配る。自分で書いた内容(値の置き換えと末尾に足す)は、行の位置のまま。
    fn adopt(&mut self, bytes: Vec<u8>, outside: bool) -> Result<(), String> {
        let file = csvfile::parse(&bytes, self.file.delim)?;
        let carried: Vec<Option<u64>> = if outside {
            self.carried(&bytes, &file)
        } else {
            (0..file.records.len())
                .map(|i| self.ids.get(i).copied())
                .collect()
        };
        self.ids = carried
            .into_iter()
            .map(|k| {
                k.unwrap_or_else(|| {
                    self.next += 1;
                    self.next - 1
                })
            })
            .collect();
        self.stamp = stamp_of(&self.path, &bytes);
        self.bytes = bytes;
        self.file = file;
        Ok(())
    }
}

impl Source for Csv {
    fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    fn load(&mut self, _budget: usize) -> Progress {
        let n = self.file.records.len();
        Progress {
            loaded: n,
            total: Some(n),
            done: true,
        }
    }

    fn cancel(&mut self) {}

    fn rows(&self) -> Vec<RowId> {
        (0..self.file.records.len())
            .map(|i| self.row_id(i))
            .collect()
    }

    /// 行の名前は1列目の値(空なら「#行の番号」)。書く単位(ファイル)はファイルの名前。
    fn label(&self, row: &RowId) -> String {
        let Some(i) = self.index(row) else {
            return if *row == self.unit(row) {
                self.name()
            } else {
                row.0.clone()
            };
        };
        match self.file.records.get(i).and_then(|r| r.fields.first()) {
            Some(f) if !f.value.trim().is_empty() => f.value.replace(['\n', '\r'], " "),
            _ => format!("#{}", i + 1),
        }
    }

    fn mark(&self, row: &RowId) -> Option<String> {
        self.index(row).and_then(|i| self.row_lock(i))
    }

    fn columns(&self) -> Vec<String> {
        column_ids(&self.file.header)
    }

    fn get(&self, row: &RowId, col: &str) -> Cell {
        let i = self.index(row);
        let value = i
            .and_then(|i| self.file.records.get(i))
            .zip(self.col_index(col))
            .and_then(|(r, c)| r.fields.get(c))
            .map(|f| value_of(&f.value));
        Cell {
            value,
            lock: i.and_then(|i| self.row_lock(i)),
        }
    }

    fn stamp(&self, row: &RowId) -> Option<Stamp> {
        self.index(row).map(|_| self.stamp)
    }

    fn reload(&mut self, _row: &RowId) -> io::Result<()> {
        let disk = self.disk()?;
        self.adopt(disk, true).map_err(io::Error::other)
    }

    fn notes(&self) -> bool {
        false
    }

    /// SC-17: 末尾に空の行を1つ足す(すぐ書く。ほかで変わったファイルには書かない)。
    fn append_row(&mut self) -> Result<(RowId, Stamp, Stamp), SaveError> {
        let base = self.stamp;
        let next = self.write(&base, |me, disk| Ok(csvfile::append(disk, &me.file, &[])))?;
        Ok((self.row_id(self.file.records.len() - 1), base, next))
    }

    /// 行のある場所は、ファイルと `#データの何行目か`。
    fn locate(&self, row: &RowId) -> (PathBuf, String) {
        match self.index(row) {
            Some(i) => (self.path.clone(), format!("#{}", i + 1)),
            None => (PathBuf::from(&row.0), String::new()),
        }
    }

    /// 全部の行が同じファイルに書かれる(書く単位はファイル1つ)。
    fn unit(&self, _row: &RowId) -> RowId {
        RowId(self.path.display().to_string())
    }

    fn preview_unit(&self, edits: &[(RowId, Vec<Edit>)]) -> Result<(Vec<u8>, Vec<u8>), EditError> {
        let before = self
            .disk()
            .map_err(|e| EditError::NotEditable(Msg::CannotRead.fill(&[&e])))?;
        let after = self.applied(&before, edits)?;
        Ok((before, after))
    }

    fn save_unit(
        &mut self,
        base: &Stamp,
        edits: &[(RowId, Vec<Edit>)],
    ) -> Result<Stamp, SaveError> {
        self.write(base, |me, disk| me.applied(disk, edits))
    }

    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError> {
        self.preview_unit(&[(row.clone(), edits.to_vec())])
    }

    fn save(&mut self, row: &RowId, base: &Stamp, edits: &[Edit]) -> Result<Stamp, SaveError> {
        self.save_unit(base, &[(row.clone(), edits.to_vec())])
    }

    /// 外での変化: ファイルの中身が知らない基準になったら読み直し、全部の行を変わった行として返す。
    fn changed(&mut self) -> Vec<RowId> {
        if self.paused {
            return Vec::new();
        }
        let Ok(meta) = std::fs::metadata(&self.path) else {
            return Vec::new();
        };
        let mtime = meta.modified().unwrap_or(UNIX_EPOCH);
        if mtime == self.stamp.mtime && meta.len() == self.stamp.len {
            return Vec::new();
        }
        let Ok(disk) = self.disk() else {
            return Vec::new();
        };
        if writeback::sha256(&disk) == self.stamp.hash {
            self.stamp.mtime = mtime;
            return Vec::new();
        }
        let before = self.rows();
        if self.adopt(disk, true).is_err() {
            return Vec::new();
        }
        let mut out = before;
        out.extend(self.rows());
        out.sort();
        out.dedup();
        out
    }

    fn pause(&mut self, paused: bool) {
        self.paused = paused;
    }

    fn kind(&self, col: &str) -> ColumnKind {
        let values: Vec<Value> = self.values(col).collect();
        ColumnKind {
            kind: types::infer(values.iter()),
            lock: None,
        }
    }

    fn typed(&self, col: &str) -> bool {
        self.values(col).any(|v| !types::is_empty(&v))
    }

    fn folders(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    fn file(&self, row: &RowId) -> Option<FileInfo> {
        self.index(row)?;
        let name = self.name();
        let (basename, ext) = match name.rsplit_once('.') {
            Some((b, e)) => (b.to_string(), e.to_string()),
            None => (name.clone(), String::new()),
        };
        let secs = |t: SystemTime| {
            t.duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64)
        };
        Some(FileInfo {
            name: name.clone(),
            basename,
            ext,
            path: name,
            folder: String::new(),
            size: self.bytes.len() as u64,
            mtime: secs(self.stamp.mtime),
            ctime: secs(self.stamp.mtime),
            tags: Vec::new(),
        })
    }

    fn candidates(&self, col: &str, max: usize) -> Option<Vec<Value>> {
        let mut out: Vec<Value> = Vec::new();
        for v in self.values(col) {
            if types::is_empty(&v) || out.contains(&v) {
                continue;
            }
            if out.len() == max {
                return None;
            }
            out.push(v);
        }
        Some(out)
    }

    fn list_candidates(&self, col: &str) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for v in self.values(col) {
            if let Value::Str(s) = v {
                *counts.entry(s).or_insert(0) += 1;
            }
        }
        let mut out: Vec<(String, usize)> = counts.into_iter().collect();
        out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        out
    }
}
