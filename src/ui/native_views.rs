//! mdgrid のビュー(`impl App` の続き。BV-17〜BV-20)。形は docs/design.md の「mdgrid のビュー」。
//! タブは `.base` のビュー(`.base` なしなら「既定の表」)のあとに mdgrid のビューを並べ、`[` `]` とクリックで
//! 切り替える。mdgrid のビューを選ぶと、その定義(列の並び・式の絞り込み)から作った `.base` で表を組む。
//! 並び・隠す列・ビューの設定は定義から当て、表で変えた分(反映した設定・動かした列)は名前ごとの見た目の
//! 状態に残して定義の上に重ねる(状態はその定義の指紋を持ち、定義が変われば重ねない)。幅と畳んだ
//! まとまりもその状態から(startup.rs の `restore_state`)。今のビューが定義と違う間はタブに `*` を付ける。
//! ビューの設定の画面のボタン(名前を付けて保存・上書き・名前の変更・削除)もここ。書くときは views.toml を
//! 読み直し、その操作の分だけ足し引きして書く(ほかで足したビューを消さない)。パレットのコマンド
//! (.base に書き出す・.base のビューを取り込む)は native_io.rs。定義の読み書きと `.base` との変換は核の
//! `mdgrid::views`。読むだけ(WB-15)では views.toml も `.base` も状態も書かない。

use super::app::App;
use super::keymap::Mode;
pub(crate) use super::native_io::{ask_lead, ask_overlay, Ask};
use super::settings::{TextKind, BUTTONS, VIEW_BUTTONS};
use super::startup::READONLY;
use mdgrid::base::{Base, Column, Grid};
use mdgrid::config;
use mdgrid::i18n::Msg;
use mdgrid::settings::{Group, Settings};
use mdgrid::source::RowId;
use mdgrid::views::{self, NativeView};
use std::path::{Path, PathBuf};

/// `.base` なしで開いたときの先頭のタブの名前(日本語の値)。mdgrid のビューの名前の検査は、これと英語の
/// `Msg::DefaultTabName.en()` の両方と比べて断る。画面に見せる名前(`view_names`)と、既定の表から
/// 書き出す `.base` のビューの名前(`as_native`)は今の言語の `Msg::DefaultTabName.text()`。
pub(crate) const DEFAULT_TAB: &str = Msg::DefaultTabName.ja();
/// mdgrid のビューの見た目の状態を置く、状態の置き場の中のフォルダ(`.base` のビューの状態と鍵がぶつからない。BV-20)。
pub(crate) const STATE_DIR: &str = "views";
/// 既定の表の、前の英語の名前(SR-34 の前)。書いてある設定(places.toml の view など)が読めるよう別名として受ける。
pub(crate) const OLD_DEFAULT_TAB: &str = "Default";

/// mdgrid のビューの状態。
#[derive(Default)]
pub(crate) struct Native {
    /// 開いた対象の mdgrid のビュー(views.toml の順)。
    pub views: Vec<NativeView>,
    /// 選んだ mdgrid のビュー(None なら `.base` のビューか既定の表)。
    pub at: Option<usize>,
    /// 選んだ mdgrid のビューから作った `.base`(作れなければ理由)。
    synth: Option<Result<Base, String>>,
    /// views.toml の置き場(None なら読まず書かない)と対象。
    pub dir: Option<PathBuf>,
    pub target: PathBuf,
    /// 書き出し・取り込みの続きの入力(native_io.rs)。
    pub ask: Option<Ask>,
    /// タブの順・隠すタブ・切り替えの案内(NV-26。views.toml の対象の項目)。
    pub tabs: views::TabPrefs,
}

/// mdgrid のビューの列の並びと式の絞り込みを `.base` にする(核の `print::synth`。`--print` と同じ組み立て)。
fn synth_of(nv: &NativeView) -> Result<Base, String> {
    mdgrid::print::synth(nv)
}

/// 定義の指紋(名前は入れない)。見た目の状態がどの定義の上に重ねたものかを見分ける。
pub(crate) fn fingerprint(nv: &NativeView) -> String {
    let v = NativeView {
        name: String::new(),
        ..nv.clone()
    };
    let text = toml::to_string(&v).unwrap_or_default();
    mdgrid::source::content_hash(text.as_bytes())[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// 落とした部分の説明をメッセージの後ろに付ける形にする。
pub(crate) fn dropped_note(dropped: &[String]) -> String {
    if dropped.is_empty() {
        String::new()
    } else {
        Msg::DroppedNote.fill(&[&dropped.join(" / ")])
    }
}

impl App {
    /// 対象の mdgrid のビューを読む(BV-20)。警告(知らない項目・壊れたファイル)を返す。
    pub(crate) fn load_native(&mut self, dir: Option<PathBuf>, target: &Path) -> Vec<String> {
        self.nv.target = target.to_path_buf();
        let Some(dir) = dir else {
            return Vec::new();
        };
        let (views, warns) = views::load_views(&dir, target);
        self.nv.views = views;
        self.nv.tabs = views::load_tab_prefs(&dir, target);
        self.nv.dir = Some(dir);
        warns
    }

    /// 選んだ mdgrid のビュー。
    pub(crate) fn native_view(&self) -> Option<&NativeView> {
        self.nv.views.get(self.nv.at?)
    }

    /// 選んだ mdgrid のビューから作った `.base`(セルの見せ方に使う)。
    pub(crate) fn native_synth(&self) -> Option<&Base> {
        self.nv.at?;
        self.nv.synth.as_ref()?.as_ref().ok()
    }

    /// 選んだ mdgrid のビューの表。式の絞り込みを評価できなければ開かずに理由(BV-7 と同じ)。
    /// 列の並びが空なら既定の表と同じ列。並びがあれば、並びに無いノートのキーを右に足す(隠した列になる。
    /// startup.rs の `add_column`)。選んでいなければ None。
    pub(crate) fn native_grid(&self) -> Option<Result<Grid, String>> {
        let nv = self.native_view()?;
        let b = match self.nv.synth.as_ref()? {
            Ok(b) => b,
            Err(e) => return Some(Err(Msg::PrintNativeView.fill(&[&nv.name, &e]))),
        };
        let prop = |r: &RowId, k: &str| self.prop(r, k);
        let built = b.build(0, self.src.as_ref(), &prop, self.today, self.now);
        Some(built.map(|mut g| {
            if nv.order.is_empty() {
                g.columns.clear();
            }
            for id in self.src.columns() {
                if !g.columns.iter().any(|c| c.id == id) {
                    g.columns.push(Column {
                        title: id.clone(),
                        id,
                    });
                }
            }
            g
        }))
    }

    /// 並びのある mdgrid のビューで、並び(`order` は状態か定義の並び)に無い列か(隠して出す)。
    pub(crate) fn native_extra(&self, order: &[String], id: &str) -> bool {
        self.native_view().is_some_and(|nv| !nv.order.is_empty())
            && !order.is_empty()
            && !order.iter().any(|c| c == id)
    }

    /// 今の mdgrid のビューが定義と違う(反映した設定か、見えている列の並び。タブの `*`)。
    pub(crate) fn native_dirty(&self) -> bool {
        let Some(nv) = self.native_view() else {
            return false;
        };
        if self.settings != nv.settings {
            return true;
        }
        let order = if nv.order.is_empty() {
            self.src.columns()
        } else {
            nv.order.clone()
        };
        let want: Vec<String> = order
            .into_iter()
            .filter(|c| !nv.hidden.contains(c))
            .collect();
        self.progress.done && self.cols != want
    }

    /// タブの前半(`.base` のビューか既定の表)の数。
    pub(crate) fn base_tabs(&self) -> usize {
        self.base.as_ref().map(|b| b.base.views.len()).unwrap_or(1)
    }

    /// タブの名前(BV-13・BV-20): `.base` のビュー(`.base` なしなら既定の表)、続けて mdgrid のビュー。
    /// `.base` も mdgrid のビューも無ければ空(タブは既定の表の1つ)。
    pub(crate) fn view_names(&self) -> Vec<String> {
        let mut out: Vec<String> = match &self.base {
            Some(b) => b.base.views.iter().map(|v| v.name.clone()).collect(),
            None if self.nv.views.is_empty() => Vec::new(),
            None => vec![Msg::DefaultTabName.text().to_string()],
        };
        out.extend(self.nv.views.iter().map(|v| v.name.clone()));
        out
    }

    /// NV-26: タブの見せる順(`view_names` の添字。隠したタブも入れる)。好みの順の名前を先に、
    /// 好みに無いタブは元の順で後ろ。
    pub(crate) fn tab_order(&self) -> Vec<usize> {
        let names = self.view_names();
        let mut out: Vec<usize> = Vec::with_capacity(names.len());
        for n in &self.nv.tabs.order {
            if let Some(i) = names.iter().position(|x| x == n) {
                if !out.contains(&i) {
                    out.push(i);
                }
            }
        }
        let rest: Vec<usize> = (0..names.len()).filter(|i| !out.contains(i)).collect();
        out.extend(rest);
        out
    }

    /// NV-26: タブの行と `[` `]` に出すタブ(見せる順。隠したタブは今のビューでなければ外す)。
    pub(crate) fn tab_list(&self) -> Vec<usize> {
        let names = self.view_names();
        let cur = self.view_index();
        self.tab_order()
            .into_iter()
            .filter(|i| *i == cur || !self.nv.tabs.hidden.contains(&names[*i]))
            .collect()
    }

    pub(crate) fn view_index(&self) -> usize {
        match self.nv.at {
            Some(i) => self.base_tabs() + i,
            None => self.base.as_ref().map(|b| b.view).unwrap_or(0),
        }
    }

    /// タブ i のビューを選ぶ(BV-13)。前のビューの見た目の状態を書いてから切り替える(SR-12)。
    pub(crate) fn select_view(&mut self, i: usize) {
        let n = self.view_names().len();
        if n == 0 {
            self.message = Some(Msg::OnlyOneView.into());
            return;
        }
        if i >= n {
            return;
        }
        // 初めて開くときは書かない。
        if self.store.as_ref().is_some_and(|s| s.shown) {
            self.persist_state();
        }
        let nb = self.base_tabs();
        if i < nb {
            if let Some(b) = &mut self.base {
                b.view = i;
            }
            self.nv.at = None;
            self.nv.synth = None;
        } else {
            let k = i - nb;
            self.nv.synth = Some(synth_of(&self.nv.views[k]));
            self.nv.at = Some(k);
        }
        self.reset_view();
    }

    /// 選んだ mdgrid のビューの定義から表の元を作り直す(名前を変えたあと)。
    pub(crate) fn refresh_native_synth(&mut self) {
        if let Some(v) = self.native_view() {
            self.nv.synth = Some(synth_of(v));
        }
    }

    /// mdgrid のビューを名前で選ぶ。
    fn select_native(&mut self, name: &str) {
        if let Some(j) = self.nv.views.iter().position(|v| v.name == name) {
            self.select_view(self.base_tabs() + j);
        }
    }

    /// `[` `]`: 前・次のビュー(端で回る)。
    pub(crate) fn switch_view(&mut self, next: bool) {
        let list = self.tab_list();
        let n = list.len();
        if n < 2 {
            self.message = Some(Msg::NoViewToSwitch.into());
            return;
        }
        let i = list
            .iter()
            .position(|k| *k == self.view_index())
            .unwrap_or(0);
        self.select_view(list[if next { (i + 1) % n } else { (i + n - 1) % n }]);
        // SR-20: タブを隠していれば、どのビューに移ったかを知らせる(ヘッダーにも名前を出す)。
        if !self.shows(mdgrid::display::Item::Tabs) && self.message.is_none() {
            let name = self.view_names()[self.view_index()].clone();
            self.message = Some(Msg::SwitchedView.fill(&[&name]));
        }
    }

    /// 今のビューを mdgrid のビューの形にする(名前を付けて保存・上書き・書き出し)。`s` と `cols` は
    /// 見せている設定と列(隠した列も、表で動かした順のまま)。列の並びは今の順をいつも入れる。ただし
    /// 元のビューが並びを持たず(既定の並び)、今の順がその既定の並びのままで、`keep_order` でなければ空にする
    /// (あとで増えたキーも出る)。`.base` のビューからは filters・sort・groupBy を取り込み、ビューの設定を
    /// 重ねる。落とした部分の説明を返す。
    pub(crate) fn as_native(
        &self,
        s: &Settings,
        cols: &[(String, bool)],
        keep_order: bool,
    ) -> (NativeView, Vec<String>) {
        let (mut nv, dropped, from_base) = match (self.native_view(), &self.base) {
            (Some(n), _) => (n.clone(), Vec::new(), false),
            (None, Some(b)) => {
                let (v, d) = views::from_base(&b.base, b.view);
                (v, d, true)
            }
            (None, None) => (
                NativeView {
                    // 書き出す .base のビューの名前は今の言語の名前(名前を付けて保存・上書きは名前を置き換える)。
                    name: Msg::DefaultTabName.text().into(),
                    ..Default::default()
                },
                Vec::new(),
                false,
            ),
        };
        let full: Vec<String> = cols.iter().map(|c| c.0.clone()).collect();
        let natural = nv.order.is_empty() && !from_base && full == self.src.columns();
        nv.order = if natural && !keep_order {
            Vec::new()
        } else {
            full
        };
        nv.hidden = cols.iter().filter(|c| !c.1).map(|c| c.0.clone()).collect();
        if from_base {
            nv.settings.filters = s.filters.clone();
            if !s.sorts.is_empty() {
                nv.settings.sorts = s.sorts.clone();
            }
            if s.group != Group::Inherit {
                nv.settings.group = s.group.clone();
            }
        } else {
            nv.settings = s.clone();
        }
        (nv, dropped)
    }

    /// views.toml を読み直し、`op` でその操作の分だけ足し引きして書く(一時ファイル → 名前の変更。
    /// 核の save_views)。ほかの mdgrid や手で足したビューは消さない。読み直したファイルが壊れていれば
    /// 書かずに理由。書いた後の対象のビューの全部と `op` の結果を返す(`adopt_views` で取り込む)。
    pub(crate) fn edit_views<T>(
        &self,
        op: impl FnOnce(&Self, &mut Vec<NativeView>) -> Result<T, String>,
    ) -> Result<(Vec<NativeView>, T), String> {
        let Some(dir) = &self.nv.dir else {
            return Err(Self::NO_DIR.into());
        };
        let (mut list, _) = views::load_views(dir, &self.nv.target);
        let out = op(self, &mut list)?;
        views::save_views(dir, &self.nv.target, &list)
            .map_err(|e| Msg::CannotWriteFile.fill(&[&views::FILE_NAME, &e]))?;
        Ok((list, out))
    }

    /// 書いた後のビューの全部を取り込む(読み直しで増えたビューもタブに出る)。`keep` は今選んでいる
    /// mdgrid のビューの名前(無くなったら選んでいないことにし、その状態は書かない)。
    pub(crate) fn adopt_views(&mut self, list: Vec<NativeView>, keep: Option<&str>) {
        let was = self.nv.at.is_some();
        self.nv.views = list;
        self.nv.at = keep.and_then(|n| self.nv.views.iter().position(|v| v.name == n));
        if was && self.nv.at.is_none() {
            self.nv.synth = None;
            if let Some(s) = &mut self.store {
                s.shown = false;
            }
        }
    }

    /// `NO_DIR.into()` で今の言語の文。
    pub(crate) const NO_DIR: Msg = Msg::ViewsNoDir;

    /// 名前が使えないなら理由(空・既定の表・`.base` のビュー・`list` のほかの mdgrid のビューと同じ名前)。
    pub(crate) fn name_problem(
        &self,
        list: &[NativeView],
        name: &str,
        except: Option<&str>,
    ) -> Option<String> {
        if name.trim().is_empty() {
            return Some(Msg::ViewNameEmpty.into());
        }
        let fixed = self
            .base
            .as_ref()
            .is_some_and(|b| b.base.views.iter().any(|v| v.name == name));
        // 既定の表の名前は英日のどちらでも断る(どちらの言語の画面でもタブが2つ並ばない)。
        let default_tab =
            name == DEFAULT_TAB || name == Msg::DefaultTabName.en() || name == OLD_DEFAULT_TAB;
        if default_tab || fixed {
            return Some(Msg::ViewNameFixed.fill(&[&name]));
        }
        let taken = list
            .iter()
            .any(|v| v.name == name && Some(v.name.as_str()) != except);
        taken.then(|| Msg::ViewNameTaken.fill(&[&name]))
    }

    /// `list` の中で使える名前にする(空なら「ビュー」、重なれば「名前 (2)」など)。
    pub(crate) fn unique_name(&self, list: &[NativeView], name: &str) -> String {
        let name = if name.trim().is_empty() {
            Msg::ViewDefaultName.text()
        } else {
            name
        };
        if self.name_problem(list, name, None).is_none() {
            return name.to_string();
        }
        (2..)
            .map(|k| format!("{name} ({k})"))
            .find(|n| self.name_problem(list, n, None).is_none())
            .unwrap_or_default()
    }

    /// mdgrid のビューの見た目の状態を、名前の変更なら新しい名前へ移し、削除なら消す(SR-12・BV-20)。
    pub(crate) fn move_state(&mut self, old: &str, new: Option<&str>) {
        if self.readonly {
            return;
        }
        let Some(st) = &self.store else {
            return;
        };
        let dir = st.dir.join(STATE_DIR);
        let mut result = Ok(());
        if let Some(new) = new {
            let s = config::load_state(&dir, &st.target, old);
            if s != Default::default() {
                result = config::save_state(&dir, &st.target, new, &s);
            }
        }
        let result = result.and_then(|_| config::remove_state(&dir, &st.target, old));
        if let Err(e) = result {
            self.message = Some(Msg::StateCannotMove.fill(&[&e]));
        }
    }

    /// ビューの設定の画面を閉じて表へ(写しは捨てる)。
    fn close_draft(&mut self) {
        self.draft = None;
        self.set_mode(Mode::Table);
    }

    /// ビューの設定の画面の mdgrid のビューのボタン(BV-18)。i は BUTTONS の添字。
    pub(crate) fn view_button(&mut self, i: usize) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.nv.dir.is_none() {
            self.message = Some(Self::NO_DIR.into());
            return;
        }
        if i > VIEW_BUTTONS && self.nv.at.is_none() {
            self.message = Some(Msg::ViewButtonFixed.fill(&[
                &Msg::DefaultTabName.text(),
                &BUTTONS[i].text(),
                &BUTTONS[VIEW_BUTTONS].text(),
            ]));
            return;
        }
        match i - VIEW_BUTTONS {
            0 => self.open_text(String::new(), TextKind::SaveAs, String::new(), None),
            1 => self.overwrite_view(),
            2 => {
                let name = self
                    .native_view()
                    .map(|v| v.name.clone())
                    .unwrap_or_default();
                self.open_text(String::new(), TextKind::Rename, name, None);
            }
            _ => {
                let name = self
                    .native_view()
                    .map(|v| v.name.clone())
                    .unwrap_or_default();
                self.ask_delete_view(name);
            }
        }
    }

    /// 名前の入力を決めた(名前を付けて保存・名前の変更)。名前が使えなければ入力のまま理由を出す。
    pub(crate) fn commit_view_name(&mut self, kind: TextKind, name: String) {
        if kind == TextKind::Rename {
            self.rename_view(name)
        } else {
            self.save_as_view(name)
        }
    }

    /// 「名前を付けて保存」: 設定の画面の写しを新しい mdgrid のビューにして、そのタブへ移る。
    fn save_as_view(&mut self, name: String) {
        let Some(d) = &self.draft else {
            return;
        };
        let (mut nv, dropped) = self.as_native(&d.s, &d.cols, false);
        nv.name = name.clone();
        let res = self.edit_views(|app, list| {
            if let Some(e) = app.name_problem(list, &nv.name, None) {
                return Err(e);
            }
            list.push(nv);
            Ok(())
        });
        let list = match res {
            Ok((list, ())) => list,
            Err(e) => return self.message = Some(e),
        };
        let keep = self.native_view().map(|v| v.name.clone());
        self.adopt_views(list, keep.as_deref());
        self.close_draft();
        self.select_native(&name);
        self.message = Some(Msg::ViewSavedAs.fill(&[&name, &dropped_note(&dropped)]));
    }

    /// 「上書き」: 選んだ mdgrid のビューを設定の画面の写しで置き換え、定義から組み直す
    /// (その名前の状態の重ねは定義が変わるので当たらなくなる。幅と畳んだまとまりは残す)。
    fn overwrite_view(&mut self) {
        let (Some(k), Some(d)) = (self.nv.at, &self.draft) else {
            return;
        };
        let (mut nv, _) = self.as_native(&d.s, &d.cols, false);
        let name = self.nv.views[k].name.clone();
        nv.name = name.clone();
        // 幅と畳んだまとまりを書いておく(上書きの後は書かずに定義から組む)。
        self.persist_state();
        let res = self.edit_views(|_, list| {
            match list.iter().position(|v| v.name == name) {
                Some(j) => list[j] = nv,
                None => return Err(Msg::ViewGoneOverwrite.fill(&[&name])),
            }
            Ok(())
        });
        let list = match res {
            Ok((list, ())) => list,
            Err(e) => return self.message = Some(e),
        };
        self.adopt_views(list, Some(&name));
        self.close_draft();
        if let Some(s) = &mut self.store {
            s.shown = false;
        }
        self.select_native(&name);
        self.message = Some(Msg::ViewOverwritten.fill(&[&name]));
    }

    /// 「名前の変更」: 見た目の状態(重ね・幅・畳んだまとまり)は新しい名前へ移す(SR-12)。
    fn rename_view(&mut self, name: String) {
        let (Some(k), Some(d)) = (self.nv.at, &self.draft) else {
            return;
        };
        let unapplied = d.s != self.settings || d.cols_changed();
        let old = self.nv.views[k].name.clone();
        let res = self.edit_views(|app, list| {
            if let Some(e) = app.name_problem(list, &name, Some(&old)) {
                return Err(e);
            }
            match list.iter_mut().find(|v| v.name == old) {
                Some(v) => v.name = name.clone(),
                None => return Err(Msg::ViewGoneRename.fill(&[&old])),
            }
            Ok(())
        });
        let list = match res {
            Ok((list, ())) => list,
            Err(e) => return self.message = Some(e),
        };
        self.persist_state();
        self.move_state(&old, Some(&name));
        self.adopt_views(list, Some(&name));
        self.rename_in_tabs(&old, Some(&name));
        if let Some(v) = self.native_view() {
            self.nv.synth = Some(synth_of(v));
        }
        self.close_draft();
        let note = if unapplied {
            Msg::RenameUnapplied.text()
        } else {
            ""
        };
        self.message = Some(Msg::ViewRenamed.fill(&[&old, &name, &note]));
    }

    /// 「削除」: 選んだ mdgrid のビューとその見た目の状態を消して、前のタブへ移る。
    pub(crate) fn delete_view(&mut self) {
        let Some(k) = self.nv.at else {
            return;
        };
        let name = self.nv.views[k].name.clone();
        let res = self.edit_views(|_, list| {
            // 同じ名前のビューが手で2つ書かれていても、消すのは1つだけ。
            if let Some(j) = list.iter().position(|v| v.name == name) {
                list.remove(j);
            }
            Ok(())
        });
        let list = match res {
            Ok((list, ())) => list,
            Err(e) => return self.message = Some(e),
        };
        self.move_state(&name, None);
        self.adopt_views(list, None);
        self.rename_in_tabs(&name, None);
        self.close_draft();
        let back = (self.base_tabs() + k).saturating_sub(1);
        let n = self.view_names().len();
        if n == 0 {
            self.reset_view();
        } else {
            self.select_view(back.min(n - 1));
        }
        self.message = Some(Msg::ViewDeleted.fill(&[&name]));
    }
}

impl App {
    /// NV-25: 今のビューを既定のビューにする(views.toml の対象の default_view)。先頭のタブ(既定の表・
    /// `.base` の先頭のビュー)を選んだら、名前を消して先頭で開くことにする。読むだけでは書かない。
    pub(crate) fn set_default_view(&mut self) {
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        let Some(dir) = self.nv.dir.clone() else {
            self.message = Some(Self::NO_DIR.into());
            return;
        };
        let i = self.view_index();
        let name = self.view_names().get(i).cloned().unwrap_or_default();
        let store = (i > 0).then_some(name.as_str());
        self.message = Some(
            match mdgrid::views::save_default_view(&dir, &self.nv.target, store) {
                Err(e) => Msg::CannotWriteFile.fill(&[&mdgrid::views::FILE_NAME, &e]),
                Ok(()) if store.is_some() => {
                    Msg::DefaultViewSet.fill(&[&super::width::sanitize(&name)])
                }
                Ok(()) => Msg::DefaultViewCleared.text().into(),
            },
        );
    }
}
