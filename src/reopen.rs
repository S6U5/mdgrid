//! 開き直し(resume.*)を画面につなぐ: 画面の今の表とビューから開き直す命令を作り、変わったときだけ
//! Herdr に伝え(resume.herdr-follow)、起動したフォルダの最後の画面に書く(resume.last-screen-save)。
//! 命令の形と Herdr の呼び方は `mdgrid::resume`。

use crate::ui::App;
use mdgrid::resume::{self, Herdr, Reopen, Reporter};
use std::path::PathBuf;

pub(crate) struct Resumer {
    /// 開いている表(ビューは毎回画面から取る)。
    base: Reopen,
    herdr: Option<Reporter>,
    /// 最後の画面を書く (見た目の状態の置き場, 起動したフォルダ)。
    last: Option<(PathBuf, PathBuf)>,
    /// 前に伝えて書いた中身(同じなら何もしない)。
    sent: Option<Reopen>,
}

impl Resumer {
    /// 画面を出す起動のときだけ作る(`--pick` では作らない。resume.herdr-skip-pick)。Herdr を呼ぶのは、Herdr の窓で
    /// `terminal.herdr` が真のときだけ(resume.herdr-config-off)。
    pub(crate) fn new(
        base: Reopen,
        herdr_on: bool,
        state_dir: Option<PathBuf>,
        folder: Option<PathBuf>,
    ) -> Resumer {
        let herdr = herdr_on
            .then(|| Herdr::from_env(|k| std::env::var(k).ok()))
            .flatten()
            .map(Reporter::start);
        Resumer {
            base,
            herdr,
            last: state_dir.zip(folder),
            sent: None,
        }
    }

    /// 一覧などで別の表へ移った(CLI-19)。次の `tick` で伝え直す。
    pub(crate) fn switch(&mut self, base: Reopen) {
        self.base = base;
    }

    /// 画面の今の表とビューが前と違えば、Herdr に伝えて最後の画面に書く。
    pub(crate) fn tick(&mut self, app: &App) {
        let mut now = self.base.clone();
        now.view = app.resume_view();
        if self.sent.as_ref() == Some(&now) {
            return;
        }
        if let Some(h) = &mut self.herdr {
            h.update(&now.argv());
        }
        if let Some((dir, folder)) = &self.last {
            // 見た目の状態と同じく、書けなくても画面は止めない(消えても困らない。SR-11)。
            let _ = resume::save_last(dir, folder, &now);
        }
        self.sent = Some(now);
    }

    /// 画面を終える: Herdr に窓を返す(resume.herdr-release)。
    pub(crate) fn finish(self) {
        if let Some(h) = self.herdr {
            h.finish();
        }
    }
}
