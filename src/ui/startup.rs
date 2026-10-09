//! 起動(`impl App` の続き。タスク 8): 設定(CLI-3・SR-13)、読むだけ(WB-15)、見た目の状態(SR-11・SR-12)。
//! 見た目の状態は、対象(`.base` のパス、`.base` なしならフォルダのパス)とビューの名前ごとに、
//! 呼ぶ側が渡す置き場(既定は `$XDG_STATE_HOME/mdgrid/`)に置く。ノートのフォルダには書かない(SR-11)。
//! 起動とビューの切り替えで読み、ビューの切り替えと終了で書く。一時的な並べ替え(NV-3)と固定は残さない。
//! 起動で開くビューは先頭(`--view` があればそれ。BV-13)で、前に選んだビューは起動に使わない。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use mdgrid::base::Base;
use mdgrid::config::{self, Config, ViewState};
use mdgrid::i18n::Msg;
use std::path::{Path, PathBuf};

/// 読むだけで開いているときに、編集・保存のキーで出す文(WB-15)。`READONLY.into()` で今の言語の文。
pub(crate) const READONLY: Msg = Msg::OpenedReadOnly;

/// 見た目の状態の置き場と対象(SR-11・SR-12)。
pub(crate) struct Store {
    pub dir: PathBuf,
    pub target: PathBuf,
    /// 今のビューの状態を読んで当てた(書いてよい)。
    pub shown: bool,
}

/// 起動のときに App に当てるもの(main が引数と設定から作る)。
pub struct Startup {
    pub config: Config,
    /// 設定を読んだときの警告(CLI-3)。キーの警告と合わせてメッセージ行に出す。
    pub warnings: Vec<String>,
    /// `--readonly`(WB-15)。
    pub readonly: bool,
    /// `--no-color`(CLI-2・SR-10)。
    pub no_color: bool,
    /// 見た目の状態の置き場。None なら読まず書かない。
    pub state_dir: Option<PathBuf>,
    /// mdgrid のビュー(`views.toml`)の置き場(BV-17・BV-20)。None なら読まず書かない。
    pub config_dir: Option<PathBuf>,
    /// 見た目の状態と mdgrid のビューの対象(`.base` かフォルダのパス)。
    pub target: PathBuf,
    /// `.base`・ヘッダーの名前・`--view` で選んだビュー(None なら先頭。BV-13)。
    pub base: Option<(Base, String, Option<usize>)>,
}

impl App {
    /// 起動の設定を当てて、表を開く。
    pub fn start(&mut self, s: Startup) {
        let mut warnings = s.warnings;
        warnings.extend(self.configure(&s.config));
        // CE-26・CE-27: 新しいノートの決まり(ビューに new_note が無いとき)。
        self.note.rule = s.config.new_note.clone();
        // CE-25: 作る場所の一覧(起動の引数の順。`.base` はそのフォルダ。同じフォルダは1つ)。
        self.note.places = super::new_note::places(self.src.as_ref(), &s.target, s.base.is_some());
        if s.no_color {
            self.color = ColorMode::None;
        }
        if s.readonly {
            self.set_readonly();
        }
        // CLI-18: 登録した表は views.toml と同じ設定のフォルダ。WS-1: ワークスペースも。
        if let Some(dir) = &s.config_dir {
            let (places, warns) = mdgrid::places::load(dir);
            self.registered = places;
            warnings.extend(warns);
            let (ws, warns) = mdgrid::workspace::load(dir);
            self.workspaces = ws;
            warnings.extend(warns);
        }
        // WS-6: 範囲を決める(-w → 印 → workspaces.toml → 検知 → 登録した表)。
        if let Some(e) = self.resolve_scope(&s.target, &s.config.workspace_detect) {
            warnings.push(e);
        }
        // BV-20: mdgrid のビューは開く前に読む(壊れていれば警告してタブなし)。
        warnings.extend(self.load_native(s.config_dir, &s.target));
        self.store = s.state_dir.map(|dir| Store {
            dir,
            target: s.target,
            shown: false,
        });
        match s.base {
            Some((base, name, view)) => {
                // BV-13: 起動は先頭のビュー(`--view` があればそれ)。
                self.set_base(base, name, view.unwrap_or(0));
            }
            None => {
                self.cols.clear();
                self.hidden.clear();
                self.restore_state();
                self.refresh();
            }
        }
        if !warnings.is_empty() {
            self.message = Some(warnings.join(" / "));
        }
    }

    /// WB-15: 空にする・取り消し・やり直し・保存・新しいノートと、詳細の表示の編集のキーを表から外し
    /// (下の帯・ヘルプ・パレットに出さない)、押したら「読むだけ」と出す。表の Enter は見出しの行の開閉に残す。
    fn set_readonly(&mut self) {
        self.readonly = true;
        let mut locked = Vec::new();
        self.keys.retain(|b| {
            let off = matches!(
                b.action,
                Action::Clear | Action::Save | Action::Undo | Action::Redo | Action::NewNote
            ) || (b.mode == Mode::Detail && b.action == Action::Edit);
            if off {
                locked.push((b.mode, b.key));
            }
            !off
        });
        for b in &mut self.keys {
            if b.mode == Mode::Table && b.action == Action::Edit {
                b.msg = mdgrid::i18n::Msg::KeyToggleGroup;
                b.label = b.msg.ja();
                b.rank = 0;
            }
        }
        self.locked = locked;
    }

    /// 今のビューの状態の場所(置き場の中のフォルダとビューの名前)。`.base` なしの既定の表なら名前は空。
    /// mdgrid のビューは置き場の中の別のフォルダ(`.base` のビューと同じ名前でもぶつからない。BV-20)。
    fn state_slot(&self, dir: &Path) -> (PathBuf, String) {
        if let Some(nv) = self.native_view() {
            return (dir.join(super::native_views::STATE_DIR), nv.name.clone());
        }
        let name = self
            .base
            .as_ref()
            .and_then(|b| b.base.views.get(b.view))
            .map(|v| v.name.clone())
            .unwrap_or_default();
        (dir.to_path_buf(), name)
    }

    /// 今のビューの見た目の状態を読み、幅と畳んだまとまりを当てる(SR-12)。並びと隠す列は、
    /// 列が表に現れたときに `add_column` で当てる。無い・壊れている → 既定。
    /// mdgrid のビューは、状態が今の定義の指紋を持てば(表で変えた分の重ね)それを、持たなければ定義の
    /// 並び・隠す列・ビューの設定を当てる。幅と畳んだまとまりはどちらも状態から(BV-20)。
    pub(crate) fn restore_state(&mut self) {
        let slot = self.store.as_ref().map(|st| self.state_slot(&st.dir));
        match (&mut self.store, slot) {
            (Some(st), Some((dir, view))) => {
                self.saved = config::load_state(&dir, &st.target, &view);
                st.shown = true;
                // NV-17: ビューの設定もビューごとの状態から。
                self.settings = self.saved.settings.clone();
                for (c, w) in &self.saved.widths {
                    self.widths.insert(c.clone(), (*w).max(1) as usize);
                }
                self.folded.extend(self.saved.folded.iter().cloned());
            }
            _ => {
                self.saved = ViewState::default();
                self.settings = Default::default();
            }
        }
        if let Some(nv) = self.native_view().cloned() {
            let fp = super::native_views::fingerprint(&nv);
            if self.saved.view.as_deref() == Some(fp.as_str()) {
                return;
            }
            self.saved.order = nv.order;
            self.saved.hidden = nv.hidden;
            self.saved.settings = nv.settings.clone();
            self.settings = nv.settings;
        }
    }

    /// 表に新しく現れた列を置く(SR-12): 状態の並びにある列はその順の位置に、状態で隠した列は隠し、
    /// 状態に無い列は右に足す。読み込みの途中で列が少しずつ増えても、読み終わりは `apply_state` と同じ並び。
    pub(crate) fn add_column(&mut self, id: &str) {
        let order = &self.saved.order;
        let idx = |c: &str| order.iter().position(|x| x == c);
        let pos = match idx(id) {
            Some(si) => self
                .cols
                .iter()
                .position(|c| idx(c).is_none_or(|k| k > si))
                .unwrap_or(self.cols.len()),
            None => self.cols.len(),
        };
        // 並びのある mdgrid のビューでは、並びに無い列は隠した列にする(`+` と設定の画面で出せる)。
        if self.saved.hidden.iter().any(|h| h == id) || self.native_extra(order, id) {
            self.hidden.push((id.to_string(), pos, false));
        } else {
            self.cols.insert(pos, id.to_string());
        }
    }

    /// 読み終わって見える列が無ければ(状態で全部の列を隠した)、最後に隠した列を戻す
    /// (最後の列は隠せない決まり。NV-4)。
    pub(crate) fn keep_one_column(&mut self) {
        if self.progress.done && self.cols.is_empty() {
            if let Some((c, _, _)) = self.hidden.pop() {
                self.cols.push(c);
            }
        }
    }

    /// 今の見た目の状態(一時的な並べ替え・固定・選択は入れない。SR-12)。
    pub(crate) fn view_state(&self) -> ViewState {
        // 隠した列は、戻したときの位置に入れた並びにする(`+` の逆の順に戻す)。
        let mut order = self.cols.clone();
        for (c, j, _) in self.hidden.iter().rev() {
            order.insert((*j).min(order.len()), c.clone());
        }
        let mut hidden: Vec<String> = self.hidden.iter().map(|(c, _, _)| c.clone()).collect();
        // 読み込みの途中で終えたときは、まだ現れていない列の状態を捨てない。
        if !self.progress.done {
            for c in &self.saved.order {
                if !order.contains(c) {
                    order.push(c.clone());
                }
            }
            for c in &self.saved.hidden {
                if !hidden.contains(c) && !self.cols.contains(c) {
                    hidden.push(c.clone());
                }
            }
        }
        let mut widths: Vec<(String, u16)> = self
            .widths
            .iter()
            .map(|(c, w)| (c.clone(), (*w).min(u16::MAX as usize) as u16))
            .collect();
        widths.sort();
        let mut folded: Vec<String> = self.folded.iter().cloned().collect();
        folded.sort();
        ViewState {
            order,
            hidden,
            widths,
            folded,
            view: None,
            // NV-17: App の設定を入れる(入れないと保存のたびに消える)。
            settings: self.settings.clone(),
        }
    }

    /// 見た目の状態を書く(SR-12。終了とビューの切り替え)。読むだけ(WB-15)では書かない。
    pub(crate) fn persist_state(&mut self) {
        if self.readonly {
            return;
        }
        let Some(st) = &self.store else {
            return;
        };
        if !st.shown {
            return;
        }
        let (dir, view) = self.state_slot(&st.dir);
        let mut state = self.view_state();
        // mdgrid のビューの状態は、どの定義の上に重ねたかの指紋を持つ(BV-20)。
        state.view = self.native_view().map(super::native_views::fingerprint);
        if let Err(e) = config::save_state(&dir, &st.target, &view, &state) {
            self.message = Some(Msg::StateUnwritable.fill(&[&e]));
        }
    }
}
