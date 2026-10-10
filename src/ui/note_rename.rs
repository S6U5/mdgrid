//! ノートの名前の変更(`impl App` の続き。CE-34)。選んだ行のノートのファイルの名前を、同じフォルダの中で変える。
//! 名前はすぐ変え(読み込み口の `rename`)、古い名前を指すリンク(`[[古い名前]]`・`[[古い名前|…]]`・`[[古い名前#…]]`)を
//! 新しい名前に書き換える変更を、ためる変更として(保存の前の差分で見られ、取り消せる)ためる。書き換えの対象は
//! 開いている表のノートのフロントマターの値(文字とリストの要素)。

use super::app::App;
use super::native_io::Ask;
use super::startup::READONLY;
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use std::path::{Path, PathBuf};

/// ためる書き換え1つ(行と新しい値)。
type Staged = (RowId, Option<NewValue>);

/// ファイルの名前(拡張子 `.md` を除く)。
fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// 文字の中の `[[old]]`・`[[old|`・`[[old#` を new に書き換える。変わらなければ None。
pub(crate) fn rewrite_links(s: &str, old: &str, new: &str) -> Option<String> {
    let mut out = s.to_string();
    for tail in ["]]", "|", "#"] {
        out = out.replace(&format!("[[{old}{tail}"), &format!("[[{new}{tail}"));
    }
    (out != s).then_some(out)
}

impl App {
    /// CE-34: 「ノートの名前を変える」。続けて新しい名前を打つ(今の名前から始める)。
    pub(crate) fn start_rename_note(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(row) = self.cur_row() else {
            return;
        };
        if self.changes.rows().contains(&row) {
            self.message = Some(Msg::RenameNotePending.into());
            return;
        }
        let now = stem(Path::new(&row.0));
        self.open_ask(Ask::RenameNote(row));
        if let Some(p) = self.palette.as_mut() {
            p.query = now;
        }
    }

    /// 打った名前で変える。名前が使えなければ入力のまま理由を出す。
    pub(crate) fn rename_note_to(&mut self, row: &RowId, query: &str) {
        let name = query.trim();
        let name = name.strip_suffix(".md").unwrap_or(name).trim();
        if name.is_empty() || name.contains(['/', '\\', '\n', '\r']) || name == "." || name == ".."
        {
            self.message = Some(Msg::RenameNoteBad.into());
            return;
        }
        let from = PathBuf::from(&row.0);
        let old = stem(&from);
        if name == old {
            self.close_palette();
            return;
        }
        let to = from.with_file_name(format!("{name}.md"));
        if to.exists() {
            self.message = Some(Msg::RenameNoteExists.fill(&[&format!("{name}.md")]));
            return;
        }
        let new_row = match self.src.rename(row, &to) {
            Ok(r) => r,
            Err(e) => {
                self.message = Some(Msg::RenameNoteFailed.fill(&[&e]));
                return;
            }
        };
        self.close_palette();
        self.links.invalidate();
        let n = self.stage_link_rewrites(&old, name);
        self.regrid = true;
        self.refresh();
        if let Some(i) = (0..self.slots.len()).find(|&i| match self.slots[i] {
            super::grid::Slot::Row(k) => self.rows.get(k) == Some(&new_row),
            _ => false,
        }) {
            self.row = i;
            self.scroll_into_view();
        }
        self.message = Some(if n == 0 {
            Msg::RenameNoteDone.fill(&[&format!("{name}.md")])
        } else {
            Msg::RenameNoteLinks.fill(&[&format!("{name}.md"), &n])
        });
    }

    /// 開いている表のノートの値のうち、old を指すリンクを new にする変更をためる。ためたセルの数を返す。
    fn stage_link_rewrites(&mut self, old: &str, new: &str) -> usize {
        let mut by_col: Vec<(String, Vec<Staged>)> = Vec::new();
        for col in self.src.columns() {
            let mut items = Vec::new();
            for r in self.src.rows() {
                let nv = match self.prop(&r, &col) {
                    Some(Value::Str(s)) => rewrite_links(&s, old, new).map(NewValue::Str),
                    Some(Value::List(xs)) => {
                        let strs: Option<Vec<String>> = xs
                            .iter()
                            .map(|x| match x {
                                Value::Str(s) => Some(s.clone()),
                                _ => None,
                            })
                            .collect();
                        strs.and_then(|v| {
                            let w: Vec<String> = v
                                .iter()
                                .map(|s| rewrite_links(s, old, new).unwrap_or_else(|| s.clone()))
                                .collect();
                            (w != v).then_some(NewValue::List(w))
                        })
                    }
                    _ => None,
                };
                if let Some(nv) = nv {
                    items.push((r, Some(nv)));
                }
            }
            if !items.is_empty() {
                by_col.push((col, items));
            }
        }
        let mut n = 0;
        for (col, items) in by_col {
            let skips = self.changes.set_each(self.src.as_ref(), &col, &items);
            n += items.len() - skips.len();
            self.stay.extend(items.into_iter().map(|(r, _)| r));
        }
        n
    }
}
