//! 画面の表の書き出し(OUT-2・OUT-5)。`impl App` の続き。
//!
//! パレットの「表をファイルに書き出す」でファイルの名前を聞き、拡張子で形(csv・tsv・json・md)を決め、
//! 今画面に出している行(ビューの設定・簡易の絞り込み・一時的な並べ替えのあと)と列(隠した列を除いた並び)を、
//! `--print` と同じ書き方(print.rs)で書く。先頭の列は各行のノートのパス(`--print --with-path` と同じ。
//! CSV と JSON は `--apply` で戻せる)。セルの値は画面と同じく、ためた変更があればその値。ノートは書かない。

use super::app::App;
use super::native_io::Ask;
use mdgrid::base::Column;
use mdgrid::base::Shown;
use mdgrid::frontmatter::Value;
use mdgrid::i18n::Msg;
use mdgrid::print::{self, Format, PrintCell, Table};
use mdgrid::source::{NewValue, RowId};
use std::path::{Path, PathBuf};

/// ためた値をノートの値の形にする(書き出しのセル)。キーの名前の変更・削除は値を変えないので None。
fn pending_value(v: &NewValue) -> Option<Value> {
    Some(match v {
        NewValue::Null => Value::Null,
        NewValue::Str(s) | NewValue::Date(s) => Value::Str(s.clone()),
        NewValue::Bool(b) => Value::Bool(*b),
        NewValue::Int(i) => Value::Int(*i),
        NewValue::Float(f) => Value::Float(*f),
        NewValue::List(items) => Value::List(items.iter().map(|s| Value::Str(s.clone())).collect()),
        NewValue::RenameKey(_) | NewValue::DeleteKey => return None,
    })
}

/// OUT-5: 打った名前のパス。先頭の `~` はホームのフォルダ、相対は起動したフォルダから。
fn resolve(name: &str) -> PathBuf {
    mdgrid::places::expand_home(name)
}

impl App {
    /// パレットの「表をファイルに書き出す」: 続けてファイルの名前を打つ(読むだけの起動でも書ける。ノートは書かない)。
    pub(crate) fn start_export_table(&mut self) {
        self.open_ask(Ask::ExportTable);
    }

    /// 打った名前に書く。形の分からない名前は理由を出して入力を残す。既にあれば `y` で確かめる。
    pub(crate) fn export_table_to(&mut self, query: &str) {
        let name = query.trim();
        let path = resolve(name);
        if name.is_empty() || Format::from_extension(&path).is_none() {
            self.message = Some(Msg::ExportTableBadExt.into());
            return;
        }
        self.close_palette();
        if path.exists() {
            self.open_ask(Ask::ExportOverwrite(path));
            return;
        }
        self.export_table_write(&path);
    }

    /// 今の表を書く。
    pub(crate) fn export_table_write(&mut self, path: &Path) {
        let Some(format) = Format::from_extension(path) else {
            self.message = Some(Msg::ExportTableBadExt.into());
            return;
        };
        let table = self.shown_table();
        let n = table.rows.len();
        let shown = path.display().to_string();
        self.message = Some(match std::fs::write(path, print::render(&table, format)) {
            Ok(()) => Msg::ExportTableDone.fill(&[&n, &shown]),
            Err(e) => Msg::ExportTableFailed.fill(&[&shown, &e]),
        });
    }

    /// 画面に出している表(OUT-2): 先頭に path の列(OUT-5)、続けて見えている列の見出しと値。
    pub(crate) fn shown_table(&self) -> Table {
        let mut columns = vec![Column {
            id: "path".into(),
            title: "path".into(),
        }];
        columns.extend(self.cols.iter().map(|c| Column {
            id: c.clone(),
            title: self.title(c),
        }));
        let args = [self.nv.target.clone()];
        let rows = self
            .rows
            .iter()
            .map(|r| {
                let path = crate::row_path(&args, self.src.as_ref(), r);
                std::iter::once(PrintCell::Prop(Some(Value::Str(path))))
                    .chain(self.cols.iter().map(|c| self.export_cell(r, c)))
                    .collect()
            })
            .collect();
        Table {
            columns,
            rows,
            row_ids: self.rows.clone(),
            warnings: Vec::new(),
        }
    }

    fn export_cell(&self, row: &RowId, col: &str) -> PrintCell {
        if let Some(v) = self.changes.pending(row, col).and_then(pending_value) {
            return PrintCell::Prop(Some(v));
        }
        match self.cell(row, col) {
            Shown::Prop(c) => PrintCell::Prop(c.value),
            Shown::Computed(v) => PrintCell::Computed(v),
            Shown::Unsupported(_) => PrintCell::Unsupported,
        }
    }
}
