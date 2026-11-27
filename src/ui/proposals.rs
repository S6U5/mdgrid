//! AI の案を画面で受ける(ai.proposal-notice・ai.proposal-take・ai.proposal-discard)。案は台帳の根の
//! `.mdgrid/proposals/` の JSON(mdgrid::proposals)。開いた表の行を指す案だけを数え、並べ、入れる(ためる変更にする)か
//! 捨てる。入れても捨てても案のファイルを消す。

use super::app::App;
use super::native_io::Ask;
use mdgrid::i18n::Msg;
use mdgrid::proposals::{self, Proposal};
use mdgrid::source::{RowId, Source};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 画面が持つ案の状態。
#[derive(Debug, Default)]
pub(crate) struct AiProposals {
    /// 根を決めたときの開いたフォルダ(開き直したら決め直す)。
    pub target: PathBuf,
    /// 台帳の根(開いたフォルダから決める)。
    pub root: Option<PathBuf>,
    /// 案の置き場の最後の更新時刻(見回りで読み直すかを決める)。
    pub stamp: Option<std::time::SystemTime>,
    /// 開いた表の行を指す案(作った日時の順)。
    pub list: Vec<Proposal>,
}

/// 行の場所(実体のパス + 行の位置の印)→ 行。
fn row_map(src: &dyn Source) -> HashMap<String, RowId> {
    src.rows()
        .into_iter()
        .map(|r| {
            let (file, at) = src.locate(&r);
            let file = std::fs::canonicalize(&file).unwrap_or(file);
            (file.to_string_lossy().into_owned() + &at, r)
        })
        .collect()
}

/// 案の `path`(根からのパス。後ろに `#行` があれば行の位置の印)を行の場所の鍵にする。
fn key_of(root: &Path, path: &str) -> String {
    let (file, at) = match path.rfind('#') {
        Some(k) if root.join(&path[..k]).exists() && !root.join(path).exists() => {
            (&path[..k], &path[k..])
        }
        _ => (path, ""),
    };
    let p = root.join(file);
    let p = std::fs::canonicalize(&p).unwrap_or(p);
    p.to_string_lossy().into_owned() + at
}

/// 下の帯や一覧に出す日時(`YYYY-MM-DDTHH:MM` → `MM-DD HH:MM`)。
fn short_time(created: &str) -> String {
    match (created.get(5..10), created.get(11..16)) {
        (Some(d), Some(t)) => format!("{d} {t}"),
        _ => created.to_string(),
    }
}

impl App {
    /// 案を読み直す(`force` でなければ、置き場が変わっていないときは読まない)。
    pub(crate) fn refresh_proposals(&mut self, force: bool) {
        // 開いたフォルダが無ければ(試験の組み立てなど)案を読まない(今のフォルダからたどらない)。
        if self.nv.target.as_os_str().is_empty() {
            self.ai = AiProposals::default();
            return;
        }
        if self.ai.root.is_none() || self.ai.target != self.nv.target {
            self.ai.target.clone_from(&self.nv.target);
            self.ai.root = Some(proposals::root_of(&self.nv.target));
            self.ai.stamp = None;
        }
        let Some(root) = self.ai.root.clone() else {
            return;
        };
        let stamp = proposals::stamp(&root);
        if !force && stamp == self.ai.stamp && (stamp.is_some() || self.ai.list.is_empty()) {
            return;
        }
        self.ai.stamp = stamp;
        let rows = row_map(self.src.as_ref());
        let (all, _bad) = proposals::read_all(&root);
        self.ai.list = all
            .into_iter()
            .filter(|p| {
                p.changes
                    .iter()
                    .any(|c| rows.contains_key(&key_of(&root, &c.path)))
            })
            .collect();
    }

    /// 開いた表の行を指す案の数(下の帯)。
    pub(crate) fn proposal_count(&self) -> usize {
        self.ai.list.len()
    }

    /// 「AI の案を見る」: 案を並べて選ばせる。
    pub(crate) fn start_ai_proposals(&mut self) {
        self.refresh_proposals(true);
        if self.ai.list.is_empty() {
            self.message = Some(Msg::ProposalNone.into());
            return;
        }
        let items = self
            .ai
            .list
            .iter()
            .map(|p| {
                Msg::ProposalItem.fill(&[
                    &short_time(&p.created),
                    &super::width::sanitize(&p.note),
                    &p.cells(),
                ])
            })
            .collect();
        let files = self.ai.list.iter().map(|p| p.file.clone()).collect();
        self.open_ask(Ask::Proposals { items, files });
    }

    /// 選んだ案で、入れるか捨てるかを選ばせる。
    pub(crate) fn choose_proposal(&mut self, file: PathBuf) {
        let items = vec![
            Msg::ProposalTake.text().to_string(),
            Msg::ProposalDiscard.text().to_string(),
        ];
        self.open_ask(Ask::ProposalAction { items, file });
    }

    /// 入れる: 案の値をためる変更にし(案を作ったときの値と今の値が違うセル・表に無い行のセルは入れない)、案のファイルを消す。
    pub(crate) fn take_proposal(&mut self, file: &Path) {
        let Some(root) = self.ai.root.clone() else {
            return;
        };
        let Some(p) = proposals::read_all(&root)
            .0
            .into_iter()
            .find(|p| p.file == file)
        else {
            self.message = Some(Msg::ProposalGone.into());
            self.refresh_proposals(true);
            return;
        };
        let rows = row_map(self.src.as_ref());
        let mut items = Vec::new();
        let mut skipped = 0usize;
        for c in &p.changes {
            let Some(row) = rows.get(&key_of(&root, &c.path)) else {
                skipped += c.set.len();
                continue;
            };
            for (col, v) in &c.set {
                let cur = self.src.get(row, col).value;
                let now = cur.as_ref().map_or(serde_json::Value::Null, |v| {
                    serde_json::from_str(&mdgrid::print::value_json(v))
                        .unwrap_or(serde_json::Value::Null)
                });
                if c.was.get(col).is_some_and(|w| *w != now) {
                    skipped += 1;
                    continue;
                }
                let kind = self.src.kind(col).kind;
                match crate::apply::to_value(
                    &crate::apply::Cell::Json(v.clone()),
                    kind,
                    cur.as_ref(),
                    self.today,
                    &self.prof.resolved.date_format,
                ) {
                    Ok(nv) => items.push((row.clone(), col.clone(), nv)),
                    Err(_) => skipped += 1,
                }
            }
        }
        let skips = self.changes.set_cells(self.src.as_ref(), &items);
        skipped += skips.len();
        let taken = items.len() - skips.len();
        let _ = proposals::remove(&p);
        self.close_palette();
        self.refresh();
        self.refresh_proposals(true);
        let mut msg = Msg::ProposalTaken.fill(&[&taken]);
        if skipped > 0 {
            msg.push_str(&Msg::ProposalSkipped.fill(&[&skipped]));
        }
        self.message = Some(msg);
    }

    /// 捨てる: 案のファイルを消す(表とノートは変えない)。
    pub(crate) fn discard_proposal(&mut self, file: &Path) {
        let note = self
            .ai
            .list
            .iter()
            .find(|p| p.file == file)
            .map(|p| p.note.clone())
            .unwrap_or_default();
        let res = std::fs::remove_file(file);
        self.close_palette();
        self.refresh_proposals(true);
        self.message = Some(match res {
            Ok(()) => Msg::ProposalDiscarded.fill(&[&super::width::sanitize(&note)]),
            Err(e) => Msg::ProposalRemoveFailed.fill(&[&e]),
        });
    }
}
