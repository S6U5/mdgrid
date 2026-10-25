//! スターターから台帳を作る画面の流れ(ST-5): パレットの「スターターから台帳を作る」→ 一覧から選ぶ → 作る場所 →
//! 見本のノートを入れるか(y/n)→ 作って、その台帳の印の `open` の表(無ければ根)を開く。作るのは核の
//! `starter::create`(ST-4 と同じ)で、開き直しは登録した表を開くときと同じ道(`switch_to`・`begin_quit`)。

use super::app::App;
use super::help::PaletteState;
use super::native_io::Ask;
use super::startup::READONLY;
use mdgrid::i18n::{self, Lang, Msg};
use mdgrid::places::{self, Place};
use mdgrid::starter::{self, Origin, Starter};
use mdgrid::{clock, workspace};
use std::path::{Path, PathBuf};

/// 打った作る場所を決める: `~` と `~/…` はホーム、絶対はそのまま、ほか(`~foo` も)は `base` からの相対。
pub(crate) fn resolve_dir(base: &Path, q: &str) -> PathBuf {
    if q == "~" || q.starts_with("~/") {
        places::expand_home(q)
    } else if Path::new(q).is_absolute() {
        PathBuf::from(q)
    } else {
        base.join(q)
    }
}

/// 一覧の1行: 名前、今の言語の名前、自分のものの印、説明。
fn label(s: &Starter, lang: Lang) -> String {
    let (title, desc) = starter::describe(s, lang);
    let mut t = format!("{}  {title}", s.name);
    if matches!(s.origin, Origin::User(_)) {
        t.push(' ');
        t.push_str(Msg::StarterUser.text());
    }
    if !desc.is_empty() {
        t.push_str(" · ");
        t.push_str(&desc);
    }
    t
}

impl App {
    /// 作る場所の基準(開いたフォルダ)。フォルダを2つ以上開いているとき(`nv.target` が実体のパスを改行で連ねたもの)は
    /// None(登録 `start_register_place` と同じ守り。`export_dir` はその形を見分けないため)。
    fn starter_base(&self) -> Option<PathBuf> {
        if self.nv.target.to_string_lossy().contains('\n') {
            return None;
        }
        self.export_dir()
    }

    /// ST-5: パレットから。スターターの一覧(ユーザー → 組み込み)を出す。読めないスターターの警告は下の帯に1つ。
    pub(crate) fn start_starter_new(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.starter_base().is_none() {
            self.message = Some(Msg::StarterMulti.into());
            return;
        }
        let (list, warns) = starter::list(self.nv.dir.as_deref());
        if let Some(w) = warns.first() {
            self.message = Some(w.clone());
        }
        let lang = i18n::current();
        let items = list.iter().map(|s| label(s, lang)).collect();
        self.open_ask(Ask::StarterPick {
            items,
            starters: list,
        });
    }

    /// 一覧で選んだ: 作る場所の欄に、スターターの名前を入れて始める(既定は開いたフォルダの下)。
    pub(crate) fn starter_picked(&mut self, s: Starter) {
        self.palette = Some(PaletteState {
            query: s.name.clone(),
            sel: 0,
        });
        self.nv.ask = Some(Ask::StarterDir(Box::new(s)));
    }

    /// 作る場所を決めた: 相対は開いたフォルダから、先頭の `~` はホーム、絶対はそのまま。次に見本を入れるかを聞く。
    pub(crate) fn starter_dir(&mut self, s: Starter, query: &str) {
        let q = query.trim();
        if q.is_empty() {
            self.message = Some(Msg::StarterDirEmpty.into());
            return;
        }
        let Some(base) = self.starter_base() else {
            self.message = Some(Msg::StarterMulti.into());
            return;
        };
        let root = resolve_dir(&base, q);
        self.palette = Some(PaletteState::default());
        self.nv.ask = Some(Ask::StarterSamples {
            starter: Box::new(s),
            root,
        });
    }

    /// 作って、その台帳の印の `open` の表(無ければ根)を開き直す。作れなければ理由を下の帯に。
    pub(crate) fn starter_make(&mut self, s: Starter, root: PathBuf, with_samples: bool) {
        let today_now = clock::today_now(|k| std::env::var(k).ok());
        match starter::create(&root, &s, i18n::current(), with_samples, today_now) {
            Err(e) => self.message = Some(e),
            Ok(made) => {
                let real = std::fs::canonicalize(&root).unwrap_or(root);
                let (open, _) = workspace::marker_open(&real);
                let place = match open {
                    Some(t) => Place {
                        name: t.name,
                        group: String::new(),
                        path: t.path,
                        view: t.view,
                    },
                    None => Place {
                        name: s.name.clone(),
                        group: String::new(),
                        path: real.clone(),
                        view: None,
                    },
                };
                self.message =
                    Some(Msg::StarterMade.fill(&[&real.display(), &s.name, &made.len()]));
                self.switch_to = Some(place);
                self.switch_select = None;
                self.switch_leave_workspace = true;
                self.begin_quit();
            }
        }
    }
}
