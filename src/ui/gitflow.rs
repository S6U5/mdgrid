//! 保存のあとの git(write-back.git-*): 書いた・作ったノートをためておき、設定の `[git]` の段まで git に渡す。
//! `when = "save"` なら保存(と画面からのノートの作成)のたび、`when = "quit"` なら終了のときに1回。
//! 終了のときに残っているノートは、どちらでもそこで流す。`message = "ask"` は保存のあとに1行の欄を出す。

use super::native_io::Ask;
use super::App;
use mdgrid::gitsync::{self, Level, Message, When};
use mdgrid::i18n::Msg;
use std::path::PathBuf;

impl App {
    /// git に渡すノートを1つ覚える(設定が off・読むだけの起動なら覚えない)。
    pub(crate) fn git_note(&mut self, path: PathBuf) {
        if self.prof.config.git.auto == Level::Off || self.readonly {
            return;
        }
        if !self.git_pending.contains(&path) {
            self.git_pending.push(path);
        }
    }

    /// 保存か作成のあと: `when = "save"` なら、文を聞く(`message = "ask"`)か、すぐ流す。
    pub(crate) fn git_after_write(&mut self) {
        let g = self.prof.config.git;
        if g.when != When::Save || self.git_pending.is_empty() || self.quit {
            return;
        }
        // リポの中のノートが1つも無ければ、欄を出さない。
        if !self
            .git_pending
            .iter()
            .any(|p| gitsync::repo_root(p).is_some())
        {
            self.git_pending.clear();
            return;
        }
        match g.message {
            Message::Ask if g.auto >= Level::Commit => self.open_ask(Ask::GitMessage),
            _ => self.git_flush(None),
        }
    }

    /// ためたノートを git に渡し、結果を1行でメッセージに足す。`first` はコミットの文の1行目(無ければ自動の文)。
    /// 失敗したら理由を出して、ノートはためたまま(次の機会にもう一度渡す)。
    pub(crate) fn git_flush(&mut self, first: Option<&str>) {
        if let Some(note) = self.git_run(first) {
            self.message = Some(match self.message.take() {
                Some(m) => format!("{m}{}{note}", Msg::SentenceSep.text()),
                None => note,
            });
        }
    }

    /// 終了のとき(main が呼ぶ): 残りを自動の文で流す。失敗の理由があれば返す(端末を戻してから出す)。
    pub fn git_flush_at_quit(&mut self) -> Option<String> {
        let before = self.git_pending.len();
        let note = self.git_run(None);
        // 失敗して残ったときだけ知らせる。
        (before > 0 && !self.git_pending.is_empty())
            .then_some(note)
            .flatten()
    }

    /// git に渡して、知らせの文を返す(渡すものが無ければ None)。
    fn git_run(&mut self, first: Option<&str>) -> Option<String> {
        let level = self.prof.config.git.auto;
        if level == Level::Off || self.git_pending.is_empty() {
            return None;
        }
        let in_repo: Vec<PathBuf> = self
            .git_pending
            .iter()
            .filter(|p| gitsync::repo_root(p).is_some())
            .cloned()
            .collect();
        if in_repo.is_empty() {
            self.git_pending.clear();
            return None;
        }
        let auto = match in_repo.len() {
            1 => Msg::GitAutoMessageOne.text().to_string(),
            n => Msg::GitAutoMessage.fill(&[&n]),
        };
        let first = first.map(str::trim).filter(|s| !s.is_empty());
        match gitsync::run(level, &in_repo, first.unwrap_or(&auto)) {
            Ok(done) => {
                self.git_pending.clear();
                Some(if done.pushed > 0 {
                    Msg::GitPushed.fill(&[&done.added])
                } else if level >= Level::Commit {
                    Msg::GitCommitted.fill(&[&done.added])
                } else {
                    Msg::GitAdded.fill(&[&done.added])
                })
            }
            Err(e) => Some(Msg::GitFailed.fill(&[&e])),
        }
    }
}
