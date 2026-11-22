//! 開き直し(resume.*)を画面につなぐ: 画面の今の表とビューから開き直す命令と窓の題を作り、変わったときだけ
//! Herdr に伝え(resume.herdr-follow・resume.herdr-title)、起動したフォルダの最後の画面に書く
//! (resume.last-screen-save)。命令と題の形と Herdr の呼び方は `mdgrid::resume`。
//! 同じく表かビューが変わったら、最近開いた表に覚える(cli.recent-record。形は `mdgrid::recent`)。

use crate::ui::App;
use mdgrid::resume::{self, Herdr, Reopen, Reporter};
use std::path::PathBuf;

pub(crate) struct Resumer {
    /// 開いている表(ビューは毎回画面から取る)。
    base: Reopen,
    herdr: Option<Reporter>,
    /// 最後の画面を書く (見た目の状態の置き場, 起動したフォルダ)。
    last: Option<(PathBuf, PathBuf)>,
    /// 前に伝えて書いた中身と、そのときのビューの数(同じなら何もしない。ビューの数で題にビューを添えるかが変わる)。
    sent: Option<(Reopen, usize)>,
    /// cli.recent-record: 最近開いた表の置き場(見た目の状態の置き場)と、前に覚えた中身。
    recent: Option<PathBuf>,
    recorded: Option<Reopen>,
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
            recent: state_dir.clone(),
            last: state_dir.zip(folder),
            sent: None,
            recorded: None,
        }
    }

    /// 一覧などで別の表へ移った(CLI-19)。次の `tick` で伝え直す。
    pub(crate) fn switch(&mut self, base: Reopen) {
        self.base = base;
    }

    /// 画面の今の表とビューが前と違えば、Herdr に命令と題を伝えて最後の画面に書く。
    pub(crate) fn tick(&mut self, app: &App) {
        // stack.page-resume: 縦に並べた表では、選んでいる表とビュー(その場で並べた形は名前が無いので)。ページを
        // 開いていれば、ワークスペースとページの名前。
        let mut now = if app.stack.is_some() {
            self.base.retarget(&crate::ui::stack::target_paths(app))
        } else {
            self.base.clone()
        };
        now.view = app.resume_view();
        self.record_recent(app, &now);
        if let Some((page, ws)) = app.stack_page() {
            now.page = Some(page);
            now.view = None;
            if ws.is_some() {
                now.workspace = ws;
            }
        }
        let views = app.view_names().len();
        if self
            .sent
            .as_ref()
            .is_some_and(|(r, n)| *r == now && *n == views)
        {
            return;
        }
        if let Some(h) = &mut self.herdr {
            h.update(&now.argv(), &title(app, &now, views));
        }
        if self.sent.as_ref().map(|(r, _)| r) != Some(&now) {
            if let Some((dir, folder)) = &self.last {
                // 見た目の状態と同じく、書けなくても画面は止めない(消えても困らない。SR-11)。
                let _ = resume::save_last(dir, folder, &now);
            }
        }
        self.sent = Some((now, views));
    }

    /// cli.recent-record: 開いている表(1つ)とビューが前に覚えたものと違えば、最近開いた表の先頭に覚える。引数なしの
    /// 起動の一覧を開いている間は覚えない(一覧で選んだ表を開いたことにし、今のフォルダは覚えない)。
    fn record_recent(&mut self, app: &App, now: &Reopen) {
        let Some(dir) = &self.recent else {
            return;
        };
        if self.recorded.as_ref() == Some(now) || app.launch_list_open() {
            return;
        }
        if let [path] = now.paths.as_slice() {
            // 見た目の状態と同じく、書けなくても画面は止めない(消えても困らない)。
            let at = mdgrid::recent::now_stamp();
            let _ = mdgrid::recent::record(dir, path, now.view.as_deref(), &at);
        }
        self.recorded = Some(now.clone());
    }

    /// 画面を終える: 題を消して Herdr に窓を返す(resume.herdr-title-clear・resume.herdr-release)。
    pub(crate) fn finish(self) {
        if let Some(h) = self.herdr {
            h.finish();
        }
    }
}

/// resume.herdr-title-form: 窓の題。表の名前は今の範囲(ワークスペース)の表から引き、ビューのタブを出さない表
/// (ビューが1つか無い)では表の名前だけにする。ページを開いているときは(stack.page-open)ページの名前。
pub(crate) fn title(app: &App, now: &Reopen, views: usize) -> String {
    if let Some(p) = &now.page {
        return p.clone();
    }
    let tables = app.scope.as_ref().map_or(&[][..], |s| s.tables.as_slice());
    let name = resume::table_name(&now.paths, tables, now.view.as_deref());
    resume::title(&name, now.view.as_deref().filter(|_| views > 1))
}
