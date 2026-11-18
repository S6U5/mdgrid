//! ノートの削除と、ごみ箱から戻すこと(`impl App` の続き。cell-edit.delete-note-*・cell-edit.new-note-undo・
//! cell-edit.new-note-redo・cell-edit.trash-restore-*)。ノートは消さずにごみ箱(`mdgrid::trash`)へ動かす。
//! ノートの作成と削除は、ためる変更の手(`Changes`)と同じ順で取り消し・やり直しができる: 手を積んだときの
//! 「ためる変更の手の数」を覚えておき、それより新しいためる変更の手が残っている間は、そちらを先に戻す。

use super::app::App;
use super::grid::Slot;
use super::native_io::Ask;
use super::startup::READONLY;
use super::width::sanitize;
use mdgrid::i18n::Msg;
use mdgrid::source::{RowId, Value};
use mdgrid::trash::Trashed;
use std::path::Path;

/// 手の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoteOpKind {
    /// 新しいノートを作った。
    Created,
    /// ノートをごみ箱へ動かした。
    Deleted,
}

/// ノートの今の居場所。
#[derive(Debug, Clone)]
pub(crate) enum NoteAt {
    /// 元の場所にある(表の行)。
    Live(RowId),
    /// ごみ箱にある。
    Gone(Trashed),
}

/// ノートの作成・削除の1手。
#[derive(Debug, Clone)]
pub(crate) struct NoteOp {
    pub(crate) kind: NoteOpKind,
    pub(crate) at: NoteAt,
    /// 積んだときの、ためる変更の手の数(取り消しの積みなら取り消せる手、やり直しの積みならやり直せる手)。
    mark: usize,
    /// やり直しの積みに置いたときの `Changes::redo_epoch`(違えば、新しい手で古くなっている)。
    epoch: u64,
}

/// 消した時刻(UNIX 秒)を、地域の時計の `YYYY-MM-DD HH:MM` にする。
fn when_text(when: i64) -> String {
    let local = when + mdgrid::clock::local_offset();
    let t = local.rem_euclid(86_400);
    format!(
        "{} {:02}:{:02}",
        mdgrid::types::format_date(local.div_euclid(86_400)),
        t / 3600,
        t / 60 % 60
    )
}

/// ファイルの名前(拡張子を除く)。
fn stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl App {
    /// cell-edit.new-note-undo: 作ったノートを、取り消しの1手として積む(ノートの表だけ)。
    pub(crate) fn note_created(&mut self, row: RowId) {
        if self.src.notes() {
            self.push_note_op(NoteOpKind::Created, NoteAt::Live(row));
        }
    }

    /// 新しい手を積む(やり直しの積みは、ためる変更の新しい手と同じく消える)。
    fn push_note_op(&mut self, kind: NoteOpKind, at: NoteAt) {
        self.note_redo.clear();
        self.changes.clear_redo();
        self.note_undo.push(NoteOp {
            kind,
            at,
            mark: self.changes.undo_depth(),
            epoch: 0,
        });
    }

    /// cell-edit.delete-note-menu: 「ノートを消す」。続けて、名前を示して確かめる。
    pub(crate) fn start_delete_note(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(row) = self.cur_row() else {
            return;
        };
        // cell-edit.delete-note-refuse-pending: ためた変更を、ノートごと黙って捨てない。
        if self.changes.rows().contains(&row) {
            self.message = Some(Msg::DeleteNotePending.into());
            return;
        }
        let links = self.links_to(&stem(Path::new(&row.0)), &row);
        self.open_ask(Ask::DeleteNote { row, links });
    }

    /// cell-edit.delete-note-links: リンクは書き換えず、指しているノートの数を確かめの問いに添える。
    /// 開いている表のノートのうち、フロントマターの値に `[[name]]`・`[[name|…]]`・`[[name#…]]` を持つものの数。
    fn links_to(&self, name: &str, except: &RowId) -> usize {
        let hit = |s: &str| super::note_rename::rewrite_links(s, name, "\u{0}").is_some();
        let cols = self.src.columns();
        self.src
            .rows()
            .iter()
            .filter(|r| *r != except)
            .filter(|r| {
                cols.iter().any(|c| match self.prop(r, c) {
                    Some(Value::Str(s)) => hit(&s),
                    Some(Value::List(xs)) => {
                        xs.iter().any(|x| matches!(x, Value::Str(s) if hit(s)))
                    }
                    _ => false,
                })
            })
            .count()
    }

    /// cell-edit.delete-note-confirm: 確かめの答え。`y` のときだけごみ箱へ動かす。
    pub(crate) fn confirm_delete_note(&mut self, row: RowId, yes: bool) {
        self.close_palette();
        if !yes {
            self.message = Some(Msg::AskCancelled.into());
            return;
        }
        // 確かめの間にためた変更ができることは無いが、動かす直前にもう一度見る。
        if self.changes.rows().contains(&row) {
            self.message = Some(Msg::DeleteNotePending.into());
            return;
        }
        let label = sanitize(&self.src.label(&row));
        match self.src.trash(&row) {
            Ok(item) => {
                self.push_note_op(NoteOpKind::Deleted, NoteAt::Gone(item));
                self.after_note_gone();
                self.message = Some(Msg::DeleteNoteDone.fill(&[&label]));
            }
            // cell-edit.delete-note-trash-only: 動かせなければ、ノートは元の場所のまま。
            Err(e) => self.message = Some(Msg::DeleteNoteFailed.fill(&[&e])),
        }
    }

    /// 行が1つ無くなったあと: 表を作り直し、同じ位置(= 下の行。最後の行だったら上の行)を選ぶ。
    fn after_note_gone(&mut self) {
        let at = self.row;
        self.data_gen += 1;
        self.links.invalidate();
        self.regrid = true;
        self.refresh();
        self.row = at.min(self.slots.len().saturating_sub(1));
        self.scroll_into_view();
    }

    /// 行が戻った・できたあと: 表を作り直してその行を選ぶ。見えなければ(絞り込み・畳んだまとまり)偽。
    fn after_note_back(&mut self, row: &RowId) -> bool {
        self.data_gen += 1;
        self.links.invalidate();
        self.stay.clear();
        self.regrid = true;
        self.refresh();
        let at = self
            .rows
            .iter()
            .position(|r| r == row)
            .and_then(|r| self.slots.iter().position(|s| *s == Slot::Row(r)));
        if let Some(i) = at {
            self.row = i;
            self.scroll_into_view();
        }
        at.is_some()
    }

    /// 元の場所のノートをごみ箱へ(手の取り消し・やり直し)。動かせたらごみ箱の中の場所。
    fn op_to_trash(&mut self, row: &RowId) -> Option<Trashed> {
        let label = sanitize(&self.src.label(row));
        match self.src.trash(row) {
            Ok(item) => {
                self.after_note_gone();
                self.message = Some(Msg::DeleteNoteDone.fill(&[&label]));
                Some(item)
            }
            Err(e) => {
                self.message = Some(Msg::DeleteNoteFailed.fill(&[&e]));
                None
            }
        }
    }

    /// ごみ箱のノートを元の場所へ。戻せたらその行。元の場所にファイルがあれば置き換えずに断る
    /// (cell-edit.trash-restore-keep-existing)。
    fn op_restore(&mut self, item: &Trashed) -> Option<RowId> {
        match self.src.restore(item) {
            Ok(row) => {
                let shown = self.after_note_back(&row);
                let label = sanitize(&self.src.label(&row));
                self.message = Some(if shown {
                    Msg::RestoreNoteDone.fill(&[&label])
                } else {
                    Msg::RestoreNoteHidden.fill(&[&label])
                });
                Some(row)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                self.message = Some(Msg::RestoreNoteExists.fill(&[&sanitize(&item.rel)]));
                None
            }
            Err(e) => {
                self.message = Some(Msg::RestoreNoteFailed.fill(&[&e]));
                None
            }
        }
    }

    /// 手を当てる(`forward` = やり直し)。当てられたら、逆向きに使える手を返す。
    fn run_note_op(&mut self, op: NoteOp, forward: bool) -> Option<NoteOp> {
        let at = match &op.at {
            NoteAt::Live(row) => NoteAt::Gone(self.op_to_trash(row)?),
            NoteAt::Gone(item) => NoteAt::Live(self.op_restore(item)?),
        };
        // 作成の取り消しと削除のやり直しはごみ箱へ、作成のやり直しと削除の取り消しは元の場所へ。
        debug_assert_eq!(
            matches!(at, NoteAt::Gone(_)),
            (op.kind == NoteOpKind::Created) != forward
        );
        Some(NoteOp { at, ..op })
    }

    /// `u`: 一番新しい手がノートの作成・削除なら、それを取り消して true(取り消せず理由を出したときも true)。
    /// それより新しいためる変更の手があれば false(呼び手がそちらを戻す)。
    pub(crate) fn undo_note_op(&mut self) -> bool {
        let Some(top) = self.note_undo.last() else {
            return false;
        };
        if self.changes.undo_depth() > top.mark {
            return false;
        }
        // そのノートにためた変更が残っていれば(保存で手の数が減ったあとなど)、先にそちらを戻す。
        if let NoteAt::Live(row) = &top.at {
            if self.changes.rows().contains(row) {
                return false;
            }
        }
        let Some(op) = self.note_undo.pop() else {
            return false;
        };
        // 当てられなかった手(外で消えた・戻す先にファイルがある)は積みに戻さない(理由は出してある)。
        if let Some(mut done) = self.run_note_op(op, false) {
            done.mark = self.changes.redo_depth();
            done.epoch = self.changes.redo_epoch();
            self.note_redo.push(done);
        }
        true
    }

    /// Ctrl+R: 次にやり直す手がノートの作成・削除なら、それをやり直して true。
    pub(crate) fn redo_note_op(&mut self) -> bool {
        // 取り消したあとに新しい手があれば、やり直しは古い(ためる変更のやり直しと同じく消える)。
        let epoch = self.changes.redo_epoch();
        self.note_redo.retain(|o| o.epoch == epoch);
        let Some(top) = self.note_redo.last() else {
            return false;
        };
        if self.changes.redo_depth() > top.mark {
            return false;
        }
        let Some(op) = self.note_redo.pop() else {
            return false;
        };
        if let Some(mut done) = self.run_note_op(op, true) {
            done.mark = self.changes.undo_depth();
            self.note_undo.push(done);
        }
        true
    }

    /// cell-edit.trash-restore-list: 「ごみ箱から戻す」。ごみ箱のノートの一覧(名前・元の場所・消した日時)を出す。
    pub(crate) fn start_restore_note(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let notes = self.src.trashed();
        if notes.is_empty() {
            self.message = Some(Msg::TrashEmpty.into());
            return;
        }
        let items = notes
            .iter()
            .map(|t| {
                format!(
                    "{}  {}  {}",
                    sanitize(&stem(Path::new(&t.rel))),
                    sanitize(&t.rel),
                    when_text(t.when)
                )
            })
            .collect();
        self.open_ask(Ask::RestoreNote { items, notes });
    }

    /// 一覧で選んだノートを元の場所へ戻す。
    pub(crate) fn restore_note(&mut self, item: Trashed) {
        self.close_palette();
        if self.op_restore(&item).is_none() {
            return;
        }
        // 戻したノートを指していた手は、もう当てられない。
        let stale = |o: &NoteOp| matches!(&o.at, NoteAt::Gone(t) if t.at == item.at);
        self.note_undo.retain(|o| !stale(o));
        self.note_redo.retain(|o| !stale(o));
    }
}
