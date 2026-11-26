//! ノートの名前の変更(`impl App` の続き。CE-34)。選んだ行のノートのファイルの名前を、同じフォルダの中で変える。
//! 名前はすぐ変え(読み込み口の `rename`)、そのノートに解けるリンク(`[[…]]`・`[…](…)`・ただのパスのどの形も。
//! cell-edit.rename-note-links-resolved)を新しい名前に書き換える変更を、ためる変更として(保存の前の差分で見られ、
//! 取り消せる)ためる。ためるかは設定 `edit.rename_links`(cell-edit.rename-note-links-setting)で決める。書き換えの
//! 対象は開いている表のノートのフロントマターの値(文字とリストの要素)。

use super::app::App;
use super::native_io::Ask;
use super::startup::READONLY;
use mdgrid::i18n::Msg;
use mdgrid::relations::{self, Form, RenameLinks};
use mdgrid::source::{NewValue, RowId, Value};
use std::path::{Path, PathBuf};

/// ためる書き換え1つ(行と新しい値)。
pub(crate) type Staged = (RowId, Option<NewValue>);

/// 列ごとのためる書き換え。
pub(crate) type Rewrites = Vec<(String, Vec<Staged>)>;

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
    /// CE-34: 「ノートの名前を変える」。続けて新しい名前を打つ(今の名前から始める。`display.note_name` で隠す部分が
    /// あれば、見せる名前から始める。cell-edit.rename-note-hidden-head)。
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
        let shown = super::view::note_name_of(self, &now);
        let now = if shown.trim().is_empty() { now } else { shown };
        self.open_ask(Ask::RenameNote(row));
        if let Some(p) = self.palette.as_mut() {
            p.query = now;
        }
    }

    /// cell-edit.rename-note-hidden-head: 打った名前 `typed` に、今の名前 `old` の隠した部分を付けた名前。打った名前が
    /// `display.note_name` に当たる(隠す部分を持つ)なら、打ったとおり。
    fn with_hidden_parts(&self, old: &str, typed: &str) -> String {
        let shown = super::view::note_name_of(self, old);
        if shown == old
            || shown.trim().is_empty()
            || super::view::note_name_of(self, typed) != typed
        {
            return typed.to_string();
        }
        if let Some(head) = old.strip_suffix(shown.as_str()) {
            return format!("{head}{typed}");
        }
        if let Some(tail) = old.strip_prefix(shown.as_str()) {
            return format!("{typed}{tail}");
        }
        match old.find(shown.as_str()) {
            Some(i) => format!("{}{typed}{}", &old[..i], &old[i + shown.len()..]),
            None => typed.to_string(),
        }
    }

    /// 打った名前で変える。名前が使えなければ入力のまま理由を出す。
    pub(crate) fn rename_note_to(&mut self, row: &RowId, query: &str) {
        let typed = query.trim();
        let typed = typed.strip_suffix(".md").unwrap_or(typed).trim();
        let from = PathBuf::from(&row.0);
        let old = stem(&from);
        let name = if typed.is_empty() {
            String::new()
        } else {
            self.with_hidden_parts(&old, typed)
        };
        let name = name.as_str();
        if name.is_empty() || name.contains(['/', '\\', '\n', '\r']) || name == "." || name == ".."
        {
            self.message = Some(Msg::RenameNoteBad.into());
            return;
        }
        if name == old {
            self.close_palette();
            return;
        }
        let to = from.with_file_name(format!("{name}.md"));
        if to.exists() {
            self.message = Some(Msg::RenameNoteExists.fill(&[&format!("{name}.md")]));
            return;
        }
        // 行き先を解くのは、古いファイルがあるうちに。
        let mut rewrites = self.link_rewrites(&from, &old, name, &to);
        let new_row = match self.src.rename(row, &to) {
            Ok(r) => r,
            Err(e) => {
                self.message = Some(Msg::RenameNoteFailed.fill(&[&e]));
                return;
            }
        };
        // 自分を指すリンクは、新しい名前の行に書く。
        for (_, items) in &mut rewrites {
            for (r, _) in items.iter_mut() {
                if r == row {
                    *r = new_row.clone();
                }
            }
        }
        self.close_palette();
        self.links.invalidate();
        let n: usize = rewrites.iter().map(|(_, items)| items.len()).sum();
        let file = format!("{name}.md");
        let mode = self.prof.resolved.rename_links;
        let staged = if n > 0 && mode == RenameLinks::Always {
            self.stage_rewrites(std::mem::take(&mut rewrites))
        } else {
            0
        };
        self.regrid = true;
        self.refresh();
        if let Some(i) = (0..self.slots.len()).find(|&i| match self.slots[i] {
            super::grid::Slot::Row(k) => self.rows.get(k) == Some(&new_row),
            _ => false,
        }) {
            self.row = i;
            self.scroll_into_view();
        }
        self.message = Some(match (n, mode) {
            (0, _) => Msg::RenameNoteDone.fill(&[&file]),
            (_, RenameLinks::Always) => Msg::RenameNoteLinks.fill(&[&file, &staged]),
            (_, RenameLinks::Never) => Msg::RenameNoteKeptLinks.fill(&[&file, &n]),
            (_, RenameLinks::Ask) => {
                self.open_ask(Ask::RenameLinks {
                    file,
                    items: rewrites,
                    n,
                });
                return;
            }
        });
    }

    /// cell-edit.rename-note-links-setting: 問いに `y`(`yes`)なら書き換えをため、ほかは書き換えずに知らせる。
    pub(crate) fn confirm_rename_links(
        &mut self,
        file: &str,
        items: Rewrites,
        n: usize,
        yes: bool,
    ) {
        self.message = Some(if yes {
            let staged = self.stage_rewrites(items);
            self.refresh();
            Msg::RenameNoteLinks.fill(&[&file, &staged])
        } else {
            Msg::RenameNoteKeptLinks.fill(&[&file, &n])
        });
    }

    /// cell-edit.rename-note-links-resolved: 開いている表のノートの値のうち、値を持つノートから解いた行き先が `from`
    /// (名前を変える前のファイル)のリンクを、行き先の名前の部分だけ `new`(`to` が新しいファイル)にした書き換え。
    /// `[[名前]]` の形で、新しい名前のノートがほかのフォルダにもあるときは、取り違えないように相対のパスにする。
    fn link_rewrites(&self, from: &Path, old: &str, new: &str, to: &Path) -> Rewrites {
        let Ok(real) = std::fs::canonicalize(from) else {
            return Vec::new();
        };
        let clash = self.with_links(|ctx| relations::count_name(new, &ctx.notes()) > 0);
        let rewrite = |note: &Path, s: &str| -> Option<String> {
            let link = relations::parse(s)?;
            let hit = self.with_links(|ctx| relations::resolve(&link, note, &ctx.notes()));
            if hit.as_deref() != Some(real.as_path()) {
                return None;
            }
            if clash && link.form == Form::Wiki && !link.target.contains('/') {
                let rel = relations::relative(note.parent().unwrap_or(Path::new("")), to);
                let rel = rel.strip_suffix(".md").unwrap_or(&rel).to_string();
                return relations::retarget(s, old, &rel);
            }
            relations::retarget(s, old, new)
        };
        let mut by_col: Rewrites = Vec::new();
        for col in self.src.columns() {
            let mut items = Vec::new();
            for r in self.src.rows() {
                let note = PathBuf::from(&r.0);
                let nv = match self.prop(&r, &col) {
                    Some(Value::Str(s)) => rewrite(&note, &s).map(NewValue::Str),
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
                                .map(|s| rewrite(&note, s).unwrap_or_else(|| s.clone()))
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
        by_col
    }

    /// 書き換えをためる。ためたセルの数を返す。
    fn stage_rewrites(&mut self, by_col: Rewrites) -> usize {
        let mut n = 0;
        for (col, items) in by_col {
            let skips = self.changes.set_each(self.src.as_ref(), &col, &items);
            n += items.len() - skips.len();
            self.stay.extend(items.into_iter().map(|(r, _)| r));
        }
        n
    }
}
