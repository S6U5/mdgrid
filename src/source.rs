//! 読み込み口(タスク 18。SC-14)。核(changes・ui)が形式に触れる唯一の口。形は docs/design.md。

pub mod markdown;

use crate::types;
use crate::vault;
use std::time::SystemTime;

pub use crate::frontmatter::Value;
pub use crate::writeback::{Edit, EditError, NewValue, SaveError};

/// Stamp.hash と同じ方式(SHA-256)のハッシュ。画面が「見せた差分の元の内容」を覚えるのに使う。
pub fn content_hash(bytes: &[u8]) -> [u8; 32] {
    crate::writeback::sha256(bytes)
}

/// WB-18: 型の決まらない列(`Source::typed` が偽)に書く、日付か日時の形だけの文字は、日付の値
/// (`NewValue::Date`。囲まずに書く)にする。ほかの値と、型の決まった列はそのまま(WB-7)。
pub fn date_if_untyped(src: &dyn Source, col: &str, v: NewValue) -> NewValue {
    match v {
        NewValue::Str(s) if types::date_or_datetime(&s) && !src.typed(col) => NewValue::Date(s),
        v => v,
    }
}

/// 行の鍵。Markdown では実体のパスの文字列。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowId(pub String);

/// 1つのセル。
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// None = キーが無い。
    pub value: Option<Value>,
    /// 読むだけの理由(表示用の短い日本語)。None なら書ける。
    pub lock: Option<String>,
}

/// 保存の基準(WB-4)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub mtime: SystemTime,
    pub len: u64,
    pub hash: [u8; 32],
}

/// 列の型(CE-2)。根ごとの types.json が食い違う列(BV-12)は Text で lock あり。
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnKind {
    pub kind: types::Kind,
    /// 列ごと読むだけにする理由。None なら書ける。
    pub lock: Option<String>,
}

/// ファイルの属性(`file.*`。BV-6)。
#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub name: String,
    pub basename: String,
    pub ext: String,
    /// 根からの相対('/' 区切り)。
    pub path: String,
    /// path の親(根の直下は "")。
    pub folder: String,
    pub size: u64,
    /// UNIX 秒。
    pub mtime: i64,
    /// UNIX 秒(作成時刻が取れなければ mtime)。
    pub ctime: i64,
    /// フロントマターの tags と本文の `#tag`(先頭の # を除く。入れ子 `a/b` はそのまま)。
    pub tags: Vec<String>,
}

pub trait Source {
    /// ヘッダーに出す名前(SR-1)。
    fn name(&self) -> String;
    fn load(&mut self, budget: usize) -> vault::Progress;
    fn cancel(&mut self);
    /// 読んだ行(Markdown では実体のパスの文字列、昇順)。
    fn rows(&self) -> Vec<RowId>;
    /// 行の表示名(Markdown では根からの相対パス)。
    fn label(&self, row: &RowId) -> String;
    /// 行の印と案内(WB-13 の競合ファイルなど)。無ければ None。
    fn mark(&self, row: &RowId) -> Option<String>;
    /// 読んだ行のキーの和(最初に現れた順)。
    fn columns(&self) -> Vec<String>;
    fn get(&self, row: &RowId, col: &str) -> Cell;
    /// 最後に読んだ内容の基準。
    fn stamp(&self, row: &RowId) -> Option<Stamp>;
    fn reload(&mut self, row: &RowId) -> std::io::Result<()>;
    /// CE-34: 行のノートのファイルの名前を `to`(同じフォルダの中のパス)に変え、新しい行を返す。読むだけのノートと、
    /// 名前を変えられない読み込み口は Err。
    fn rename(&mut self, row: &RowId, to: &std::path::Path) -> std::io::Result<RowId> {
        let _ = (row, to);
        Err(std::io::Error::other("rename is not supported"))
    }
    /// 保存の前の差分用: (今のディスクのバイト, それに edits を当てたバイト)。
    /// ファイルも読んだ内容も変えない(変えると changed が外の変更を見逃す)。
    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError>;
    /// base から変わっていれば SaveError::Changed。書けたら読み直して新しい基準を返す(WB-12)。
    fn save(&mut self, row: &RowId, base: &Stamp, edits: &[Edit]) -> Result<Stamp, SaveError>;
    /// 外での変化(BV-9 のポーリング)。読み直しも済ませる。
    fn changed(&mut self) -> Vec<RowId>;
    fn pause(&mut self, paused: bool);
    /// 列の型(CE-2): types.json があればそれ、無ければ読んだ全行(BV-2 の範囲)の値から推定。
    fn kind(&self, col: &str) -> ColumnKind;
    /// 列の型が決まっているか(WB-18): 型の設定(CE-2)があるか、読んだどれかの行に空でない値がある。
    /// 偽なら `kind` は推定の既定(Text)なだけ。既定は真(今までの振る舞い: 日付に見える文字も囲む)。
    fn typed(&self, col: &str) -> bool {
        let _ = col;
        true
    }
    /// WB-3: フロントマターの無いノートと空のフロントマターのノートに書くか(表のプロファイルの
    /// `edit.add_frontmatter`。SR-44)。読み込みの前に画面が渡す。書けない読み込み口は何もしない(既定)。
    fn set_add_frontmatter(&mut self, on: bool) {
        let _ = on;
    }
    /// 開いたフォルダ(CE-25 の作る場所の候補): 開くときに渡したフォルダの実体のパスを、渡した順で、
    /// 同じフォルダは最初の1つにまとめて返す。フォルダを持たない読み込み口は空(既定)。
    fn folders(&self) -> Vec<std::path::PathBuf> {
        Vec::new()
    }
    /// NV-6: 行の本文(フロントマターのあと。フロントマターの無いノートは全体)。読んだ内容から取り、
    /// ファイルは読み直さない。本文を持たない読み込み口と読んでいない行は None(既定)。
    fn body(&self, row: &RowId) -> Option<String> {
        let _ = row;
        None
    }
    /// 行のファイルの属性。読んでいない行は None。
    fn file(&self, row: &RowId) -> Option<FileInfo>;
    /// CE-3: 読んだ全行(BV-2 の範囲)の異なる値(Null と空は除く)。max を超えたら None。
    fn candidates(&self, col: &str, max: usize) -> Option<Vec<Value>>;
    /// CE-16・CE-19: 読んだ全行(BV-2 の範囲)のその列のリストの要素(文字列)と、それを持つノートの数。
    /// 件数の多い順、同じ件数は文字の順。tags の列は先頭の `#` を除く。
    fn list_candidates(&self, col: &str) -> Vec<(String, usize)>;
    /// BV-22: 読んだノート全部(BV-2 の範囲)のリンクの索引(`file.links`・`file.backlinks`・`file.hasLink`)。
    /// 組み立てのたびに作り直さないよう、読み込み口が覚えておく。リンクを持たない読み込み口は None(既定)。
    fn link_index(&self) -> Option<std::rc::Rc<crate::links::Index>> {
        None
    }
}

/// 式に渡す値(BV-6): 列の型がリスト(CE-2)なのに1つの値(空でない文字・数・真偽)で書かれたセルは、
/// Obsidian と同じく1つの要素のリストにする(`tags: project` の `tags.contains("proj")` を文字の一部の一致にしない)。
/// 画面の見せ方と書き戻しは元の値のまま(この関数は式の評価にだけ使う)。
pub fn expr_value(src: &dyn Source, col: &str, v: Option<Value>) -> Option<Value> {
    match v {
        Some(x @ (Value::Str(_) | Value::Int(_) | Value::Float(_) | Value::Bool(_)))
            if !matches!(&x, Value::Str(s) if s.is_empty())
                && src.kind(col).kind == types::Kind::List =>
        {
            Some(Value::List(vec![x]))
        }
        other => other,
    }
}
