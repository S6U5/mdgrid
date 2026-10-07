//! mdgrid のビューの書き出しと取り込み(`impl App` の続き。BV-19)。パレットのコマンド
//! 「.base に書き出す」(続けてファイルの名前)と「.base のビューを取り込む」(続けて `.base` → ビュー)の
//! 続きの入力は、パレットの入力と候補の場所をそのまま使う(`Ask`)。書き出しは開いたフォルダの下の
//! 新しい名前のファイルだけに書き(既存の `.base` は書き換えない。BV-3)、落とした部分を知らせる。
//! 変換は核の `mdgrid::views`、タブとビューの保存は native_views.rs。読むだけ(WB-15)では書かない。

use super::app::App;
use super::help::{score, PaletteState};
use super::keymap::{Action, Mode};
use super::native_views::dropped_note;
use super::startup::READONLY;
use super::view;
use super::width::{fit, sanitize, Align};
use mdgrid::base::{self, Base};
use mdgrid::i18n::Msg;
use mdgrid::source::NewValue;
use mdgrid::views;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 取り込みで探す `.base` の深さと数の上限。
const MAX_DEPTH: usize = 8;
const MAX_FILES: usize = 1000;
/// 候補を一度に見せる数。
const ASK_ROWS: usize = 8;

/// パレットの続きの入力(BV-19)。
pub(crate) enum Ask {
    /// 書き出す `.base` のファイルの名前。
    ExportName,
    /// 足す列の名前(CE-28)。
    NewColumn,
    /// 画面の表を書き出すファイルの名前(OUT-2)。
    ExportTable,
    /// 既にあるファイルへの上書きの確かめ(OUT-5)。続けて `y`。
    ExportOverwrite(PathBuf),
    /// 名前を変えるキー(CE-29)。続けて新しい名前。
    RenameKey(String),
    /// 消すキーとノートの数(CE-29)。続けて `y`。
    DeleteKey(String, usize),
    /// 取り込む `.base`(開いたフォルダからの相対パス)。
    ImportFile(Vec<String>),
    /// 取り込むビュー。
    ImportView {
        file: String,
        base: Box<Base>,
        names: Vec<String>,
    },
}

/// 候補を打った語で絞る(パレットと同じあいまいな一致の点の順)。返すのは items の添字。
fn filtered(items: &[String], q: &str) -> Vec<usize> {
    let mut s: Vec<(i32, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, t)| score(q.trim(), t).map(|p| (p, i)))
        .collect();
    s.sort_by_key(|&(p, i)| (-p, i));
    s.into_iter().map(|(_, i)| i).collect()
}

/// フォルダの下の `.base`(相対パス。名前が `.` で始まるフォルダとファイルは見ない)。
fn base_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            if out.len() >= MAX_FILES {
                break;
            }
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let p = e.path();
            let Ok(ft) = e.file_type() else {
                continue;
            };
            if ft.is_dir() {
                if depth < MAX_DEPTH {
                    stack.push((p, depth + 1));
                }
            } else if p.extension().is_some_and(|x| x == "base") {
                if let Ok(rel) = p.strip_prefix(root) {
                    out.push(rel.to_string_lossy().into_owned());
                }
            }
        }
    }
    out.sort();
    out
}

/// 対象(フォルダか `.base`)から、書き出しと取り込みの置き場を決める。対象が空なら None。
pub(crate) fn export_dir_of(t: &Path) -> Option<PathBuf> {
    if t.as_os_str().is_empty() {
        return None;
    }
    if t.is_dir() {
        return Some(t.to_path_buf());
    }
    match t.parent() {
        Some(p) if !p.as_os_str().is_empty() => Some(p.to_path_buf()),
        _ => Some(PathBuf::from(".")),
    }
}

/// パレットの入力の前置き(続きの入力のときだけ)。
pub(crate) fn ask_lead(app: &App) -> String {
    match &app.nv.ask {
        None => String::new(),
        Some(Ask::ExportName) => Msg::AskExportName.into(),
        Some(Ask::NewColumn) => Msg::AskNewColumn.into(),
        Some(Ask::ExportTable) => Msg::AskExportTable.into(),
        Some(Ask::ExportOverwrite(p)) => {
            Msg::AskExportOverwrite.fill(&[&sanitize(&p.display().to_string())])
        }
        Some(Ask::RenameKey(k)) => Msg::AskRenameKey.fill(&[&sanitize(k)]),
        Some(Ask::DeleteKey(k, n)) => Msg::AskDeleteKey.fill(&[&sanitize(k), n]),
        Some(Ask::ImportFile(_)) => Msg::AskImportFile.into(),
        Some(Ask::ImportView { file, .. }) => Msg::AskImportView.fill(&[&sanitize(file)]),
    }
}

/// 続きの入力の候補(全部)と、打った語で絞った添字。
fn ask_items<'a>(app: &'a App, query: &str) -> (&'a [String], Vec<usize>) {
    let items: &[String] = match &app.nv.ask {
        Some(Ask::ImportFile(files)) => files,
        Some(Ask::ImportView { names, .. }) => names,
        _ => &[],
    };
    (items, filtered(items, query))
}

/// 続きの入力を表の上に重ねる(パレットと同じ場所: 2行目に入力、その下に候補)。
pub(crate) fn ask_overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let (Some(p), Some(ask)) = (&app.palette, &app.nv.ask) else {
        return;
    };
    let mut out = vec![view::band(
        &format!(": {}{}", ask_lead(app), sanitize(&p.query)),
        w,
    )];
    let room = lines.len().saturating_sub(3).clamp(1, ASK_ROWS);
    if let Ask::ExportName = ask {
        let dir = app
            .export_dir()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let t = Msg::AskExportWhere.fill(&[&dir]);
        out.push(Line::from(fit(&sanitize(&t), w, Align::Left)));
    } else {
        let (items, idx) = ask_items(app, &p.query);
        if idx.is_empty() {
            out.push(Line::from(fit(Msg::AskNoMatch.text(), w, Align::Left)));
        }
        let start = p.sel.saturating_sub(room - 1);
        for (k, &i) in idx.iter().enumerate().skip(start).take(room) {
            let sel = k == p.sel;
            let t = format!("{} {}", if sel { ">" } else { " " }, sanitize(&items[i]));
            let st = if sel {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
        }
    }
    out.push(Line::from(" ".repeat(w)));
    for (k, l) in out.into_iter().enumerate() {
        if let Some(slot) = lines.get_mut(1 + k) {
            *slot = l;
        }
    }
}

impl App {
    /// 書き出しと取り込みの `.base` の置き場: 開いたフォルダ(`.base` で開いたならその `.base` のフォルダ。
    /// `mdgrid tasks.base` のように相対パスで開いてフォルダの部分が空なら今のフォルダ)。
    fn export_dir(&self) -> Option<PathBuf> {
        export_dir_of(&self.nv.target)
    }

    pub(crate) fn open_ask(&mut self, ask: Ask) {
        self.palette = Some(PaletteState::default());
        self.nv.ask = Some(ask);
        self.set_mode(Mode::Palette);
    }

    /// パレットの「.base に書き出す」: 続けてファイルの名前を打つ。
    pub(crate) fn start_export(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.export_dir().is_none() {
            self.message = Some(Msg::ExportNoDir.into());
            return;
        }
        self.open_ask(Ask::ExportName);
    }

    /// CE-28: どのノートにも無いキーの列を足す。続けて名前を打つ。読むだけの起動では足さない。
    pub(crate) fn start_add_column(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        self.open_ask(Ask::NewColumn);
    }

    /// CE-28: 打った名前の列を表の右に足して選ぶ。もうある列ならそこへ移る。書けない名前は理由を出して入力を残す。
    fn add_extra_column(&mut self, query: &str) {
        let name = query.trim();
        let name = name.strip_prefix("note.").unwrap_or(name).trim();
        if name.is_empty()
            || name.contains(['\n', '\r'])
            || name.starts_with("file.")
            || name.starts_with("formula.")
        {
            self.message = Some(Msg::AddColumnBad.into());
            return;
        }
        self.close_palette();
        if let Some(j) = self.cols.iter().position(|c| c == name) {
            self.col = j;
            self.scroll_into_view();
            return;
        }
        if !self.built.extra_cols.iter().any(|c| c == name) {
            self.built.extra_cols.push(name.to_string());
        }
        self.hidden.retain(|(c, _, _)| c != name);
        self.refresh();
        if let Some(j) = self.cols.iter().position(|c| c == name) {
            self.col = j;
            self.scroll_into_view();
        }
        self.message = Some(Msg::AddColumnAdded.fill(&[&name]));
    }

    /// CE-29: 今の列のキー(ノートのキーの列でなければ理由を出して None)。読むだけの起動でも None。
    fn key_column(&mut self) -> Option<String> {
        if self.readonly {
            self.message = Some(READONLY.into());
            return None;
        }
        let col = self.cols.get(self.col)?.clone();
        if col.starts_with("file.") || col.starts_with("formula.") {
            self.message = Some(Msg::KeyOpComputed.into());
            return None;
        }
        Some(col)
    }

    /// CE-29: そのキーを持つ全部のノート(読み込んだ全部。ビューの絞り込みの前)。
    fn rows_with_key(&self, key: &str) -> Vec<mdgrid::source::RowId> {
        self.src
            .rows()
            .into_iter()
            .filter(|r| self.src.get(r, key).value.is_some())
            .collect()
    }

    /// CE-29: 今の列のキーの名前を変える。続けて新しい名前を打つ。
    pub(crate) fn start_rename_key(&mut self) {
        let Some(col) = self.key_column() else {
            return;
        };
        if self.rows_with_key(&col).is_empty() {
            self.message = Some(Msg::KeyOpNoNotes.fill(&[&col]));
            return;
        }
        self.open_ask(Ask::RenameKey(col));
    }

    /// CE-29: 今の列のキーを消す。数を示して `y` で確かめる。
    pub(crate) fn start_delete_key(&mut self) {
        let Some(col) = self.key_column() else {
            return;
        };
        let n = self.rows_with_key(&col).len();
        if n == 0 {
            self.message = Some(Msg::KeyOpNoNotes.fill(&[&col]));
            return;
        }
        self.open_ask(Ask::DeleteKey(col, n));
    }

    fn rename_key_to(&mut self, from: &str, query: &str) {
        let to = query.trim();
        let to = to.strip_prefix("note.").unwrap_or(to).trim();
        if to.is_empty()
            || to.contains(['\n', '\r'])
            || to.starts_with("file.")
            || to.starts_with("formula.")
        {
            self.message = Some(Msg::KeyOpBadName.into());
            return;
        }
        self.close_palette();
        if to == from {
            return;
        }
        self.stage_key_op(from, NewValue::RenameKey(to.to_string()));
    }

    /// CE-29: そのキーを持つ全部のノートに、名前の変更か削除をためる(1手)。新しい名前のキーが既にあるノートと、
    /// 書けないノート・値は飛ばして数と理由を出す。
    fn stage_key_op(&mut self, key: &str, op: NewValue) {
        let mut reasons = Vec::new();
        let mut items = Vec::new();
        for r in self.rows_with_key(key) {
            // 同じセルに直した値があれば、名前の変更・削除で黙って捨てないように飛ばす。
            if let Some(p) = self.changes.pending(&r, key) {
                if !matches!(p, NewValue::RenameKey(_) | NewValue::DeleteKey) {
                    reasons.push(Msg::KeyOpPending.text().to_string());
                    continue;
                }
            }
            if let NewValue::RenameKey(to) = &op {
                // 大文字小文字だけ違う名前も同じキーとみなす(Obsidian と同じ)。ためた値の列も見る。
                let lower = to.to_lowercase();
                let clash = self
                    .src
                    .columns()
                    .iter()
                    .filter(|c| c.as_str() != key && c.to_lowercase() == lower)
                    .any(|c| {
                        self.src.get(&r, c).value.is_some() || self.changes.pending(&r, c).is_some()
                    })
                    || self.changes.pending(&r, to).is_some();
                if clash {
                    reasons.push(Msg::KeyOpExists.fill(&[to]));
                    continue;
                }
            }
            items.push((r, Some(op.clone())));
        }
        let skips = self.changes.set_each(self.src.as_ref(), key, &items);
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let mut msg = match &op {
            NewValue::RenameKey(to) => Msg::KeyRenamed.fill(&[&key, to, &done]),
            _ => Msg::KeyDeleted.fill(&[&key, &done]),
        };
        super::input::bulk_tail(&mut msg, 0, reasons);
        self.message = Some(msg);
    }

    /// パレットの「.base のビューを取り込む」: 続けて `.base` を選び、ビューを選ぶ。
    pub(crate) fn start_import(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.nv.dir.is_none() {
            self.message = Some(Self::NO_DIR.into());
            return;
        }
        let Some(dir) = self.export_dir() else {
            self.message = Some(Msg::ImportNoDir.into());
            return;
        };
        let files = base_files(&dir);
        if files.is_empty() {
            self.message = Some(Msg::ImportNoFiles.fill(&[&dir.display()]));
            return;
        }
        self.open_ask(Ask::ImportFile(files));
    }

    /// 続きの入力の中の動作(パレットのキー): 決める・やめる・候補の選択・1字消す。
    pub(crate) fn ask_action(&mut self, action: Action) {
        let query = self
            .palette
            .as_ref()
            .map(|p| p.query.clone())
            .unwrap_or_default();
        let n = ask_items(self, &query).1.len();
        let Some(p) = &mut self.palette else {
            return self.close_palette();
        };
        match action {
            Action::Close => {
                self.close_palette();
                self.message = Some(Msg::AskCancelled.into());
            }
            Action::Down => p.sel = (p.sel + 1).min(n.saturating_sub(1)),
            Action::Up => p.sel = p.sel.saturating_sub(1),
            Action::DeleteBack => {
                p.query.pop();
                p.sel = 0;
            }
            Action::Run => self.ask_run(&query),
            _ => {}
        }
    }

    fn ask_run(&mut self, query: &str) {
        let sel = self.palette.as_ref().map(|p| p.sel).unwrap_or(0);
        let picked = ask_items(self, query).1.get(sel).copied();
        match &self.nv.ask {
            Some(Ask::ExportName) => self.export_to(query),
            Some(Ask::NewColumn) => self.add_extra_column(query),
            Some(Ask::ExportTable) => self.export_table_to(query),
            Some(Ask::ExportOverwrite(path)) => {
                let path = path.clone();
                self.close_palette();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.export_table_write(&path);
                } else {
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            Some(Ask::RenameKey(from)) => {
                let from = from.clone();
                self.rename_key_to(&from, query)
            }
            Some(Ask::DeleteKey(key, _)) => {
                let key = key.clone();
                self.close_palette();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.stage_key_op(&key, NewValue::DeleteKey);
                } else {
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            Some(Ask::ImportFile(files)) => {
                let Some(rel) = picked.map(|i| files[i].clone()) else {
                    self.message = Some(Msg::ImportNoMatchFile.into());
                    return;
                };
                let Some(dir) = self.export_dir() else {
                    return;
                };
                let parsed = std::fs::read_to_string(dir.join(&rel))
                    .map_err(|e| e.to_string())
                    .and_then(|t| base::parse(&t));
                match parsed {
                    Err(e) => self.message = Some(Msg::ImportUnreadable.fill(&[&rel, &e])),
                    Ok(b) if b.views.is_empty() => {
                        self.message = Some(Msg::ImportNoViews.fill(&[&rel]))
                    }
                    Ok(b) => {
                        // 名前の無いビューも選べるように、候補には番号を出す。
                        let names = b
                            .views
                            .iter()
                            .enumerate()
                            .map(|(k, v)| match v.name.trim() {
                                "" => Msg::ImportUnnamed.fill(&[&(k + 1)]),
                                _ => v.name.clone(),
                            })
                            .collect();
                        self.nv.ask = Some(Ask::ImportView {
                            file: rel,
                            base: Box::new(b),
                            names,
                        });
                        self.palette = Some(PaletteState::default());
                    }
                }
            }
            Some(Ask::ImportView { file, base, names }) => {
                let Some(i) = picked else {
                    self.message = Some(Msg::ImportNoMatchView.into());
                    return;
                };
                let (file, from) = (file.clone(), names[i].clone());
                let (mut nv, dropped) = views::from_base(base, i);
                // 名前の無いビューは「.base のファイル名 番号」。
                if nv.name.trim().is_empty() {
                    let stem = Path::new(&file)
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    nv.name = format!("{stem} {}", i + 1);
                }
                let res = self.edit_views(|app, list| {
                    nv.name = app.unique_name(list, &nv.name);
                    let name = nv.name.clone();
                    list.push(nv);
                    Ok(name)
                });
                let (list, name) = match res {
                    Ok(r) => r,
                    Err(e) => return self.message = Some(e),
                };
                let keep = self.native_view().map(|v| v.name.clone());
                self.adopt_views(list, keep.as_deref());
                self.close_palette();
                if let Some(j) = self.nv.views.iter().position(|v| v.name == name) {
                    self.select_view(self.base_tabs() + j);
                }
                self.message =
                    Some(Msg::Imported.fill(&[&file, &from, &name, &dropped_note(&dropped)]));
            }
            None => self.close_palette(),
        }
    }

    /// 今のビューを新しい `.base` に書き出す(BV-19)。既存のファイルの名前なら書かずに理由(BV-3)。
    fn export_to(&mut self, query: &str) {
        let name = query.trim();
        if name.is_empty() {
            self.message = Some(Msg::ExportNameEmpty.into());
            return;
        }
        if name.contains(['/', '\\']) || name.starts_with('.') {
            self.message = Some(Msg::ExportNameOnly.into());
            return;
        }
        let Some(dir) = self.export_dir() else {
            return;
        };
        let file = if name.ends_with(".base") {
            name.to_string()
        } else {
            format!("{name}.base")
        };
        let path = dir.join(&file);
        if path.exists() {
            self.message = Some(Msg::ExportExists.fill(&[&file]));
            return;
        }
        let cols = self.column_list();
        let (nv, mut dropped) = self.as_native(&self.settings, &cols, true);
        let kind = |c: &str| Some(self.column_kind(c));
        let (yaml, more) = views::to_base_typed(&nv, &kind);
        dropped.extend(more);
        let written = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut f| f.write_all(yaml.as_bytes()));
        if let Err(e) = written {
            self.message = Some(Msg::CannotWriteFile.fill(&[&file, &e]));
            return;
        }
        self.close_palette();
        self.message = Some(Msg::Exported.fill(&[&nv.name, &file, &dropped_note(&dropped)]));
    }
}
