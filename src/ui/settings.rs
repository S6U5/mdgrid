//! ビューの設定の画面(`impl App` の続き。NV-13〜NV-22)と、表の上の設定の帯の操作(NV-22)。
//! 開くと今の設定と列の表示・順の写し(`Draft`)を編集し、「反映」で App に当てて組み立て直し、
//! 見た目の状態に書く(読むだけ(WB-15)では書かない)。「取り消し」・Esc は写しを捨てる(NV-13)。
//! 区画は 列・フィルター・並べ替え・グループ・表示(SR-20。display.rs)・ボタン(NV-18)。フィルターは 列 → 種類 → 値の一覧
//! (件数つき・「(空)」あり)か値の入力(NV-19)。条件の判定・絞る・並べる・まとめるは核の
//! `mdgrid::settings`、選び手と値の入力は settings_pick.rs、描画とクリックは settings_view.rs。キーはすべてキーの表(SR-4・SR-16)。

use super::app::App;
use super::keymap::{self, Action, Mode};
use mdgrid::i18n::Msg;
use mdgrid::settings::{CmpOp, Cond, Dir, Group, Op, Settings};

/// 区画(NV-18)。Tab の順。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sec {
    Columns,
    Filters,
    Sorts,
    Group,
    Buttons,
    /// 表示(SR-20。行番号・一行おきの色・列の区切り線・タブ・検索の欄・設定の帯)。左の欄の下。
    Display,
    /// ビューのタブ(NV-26。view_tabs.rs)。変えるとすぐ保存する。
    Views,
    /// 見た目(SR-43。look_section.rs)。テーマ・組・丸い札の端とテンプレート。
    Look,
    /// 親子(NV-27)。親子で並べるかと親のキー。
    Tree,
}

pub(crate) const SECS: [Sec; 9] = [
    Sec::Columns,
    Sec::Filters,
    Sec::Sorts,
    Sec::Group,
    Sec::Tree,
    Sec::Display,
    Sec::Views,
    Sec::Look,
    Sec::Buttons,
];

/// 下のボタン(NV-13・NV-22)と、mdgrid のビューのボタン(BV-18。`VIEW_BUTTONS` から後ろ)。
pub(crate) const BUTTONS: [Msg; 7] = [
    Msg::BtnApply,
    Msg::BtnCancel,
    Msg::BtnReset,
    Msg::BtnSaveAs,
    Msg::BtnOverwrite,
    Msg::BtnRename,
    Msg::BtnDelete,
];
/// mdgrid のビューのボタンの始まり(画面ではほかのボタンの上の行)。
pub(crate) const VIEW_BUTTONS: usize = 3;
/// グループの区画の項目の数(.base のまま・しない・列で分ける・空を隠す・並び。NV-15・NV-21)。
pub(crate) const GROUP_ITEMS: usize = 5;
/// フィルターの条件の種類(NV-19)。
pub(crate) const KINDS: [Msg; 6] = [
    Msg::KindValues,
    Msg::KindContains,
    Msg::KindNotContains,
    Msg::KindCompare,
    Msg::KindEmpty,
    Msg::KindNotEmpty,
];
/// 比べ方(NV-19)。
pub(crate) const CMPS: [(CmpOp, Msg); 6] = [
    (CmpOp::Eq, Msg::CmpEq),
    (CmpOp::Ne, Msg::CmpNe),
    (CmpOp::Lt, Msg::CmpLt),
    (CmpOp::Le, Msg::CmpLe),
    (CmpOp::Gt, Msg::CmpGt),
    (CmpOp::Ge, Msg::CmpGe),
];

/// 列を選ぶ目的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Purpose {
    Filter,
    Sort,
    Group,
    Tree,
    WbsKey,
}

impl Purpose {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Purpose::Filter => Msg::PurposeFilter.text(),
            Purpose::Sort => Msg::PurposeSort.text(),
            Purpose::Group => Msg::PurposeGroup.text(),
            Purpose::Tree => Msg::PurposeTree.text(),
            Purpose::WbsKey => Msg::PurposeWbsKey.text(),
        }
    }
}

/// 区画の上に開く選び手(右の欄に出す)。`edit` は直しているフィルターの添字(None は足す)。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Pick {
    Column {
        purpose: Purpose,
        sel: usize,
    },
    Kind {
        col: String,
        sel: usize,
        edit: Option<usize>,
    },
    Cmp {
        col: String,
        sel: usize,
        edit: Option<usize>,
    },
    /// 値の一覧(NV-19)。項目 0 は方式(残す・隠す)の切り替え、1 から値。
    Values {
        col: String,
        keep: bool,
        checked: Vec<Option<String>>,
        counts: Vec<(Option<String>, usize)>,
        sel: usize,
        edit: Option<usize>,
    },
    /// ビューの区画のタブの選び手(NV-26): 既定にする・名前を変える・削除。`tab` は区画の行。
    TabMenu {
        tab: usize,
        sel: usize,
    },
    /// 見た目の区画の選び手(SR-43): `field` は 0 テーマ・1 組・2 丸い札の端。
    Look {
        field: usize,
        sel: usize,
    },
}

impl Pick {
    pub(crate) fn sel(&self) -> usize {
        match self {
            Pick::Column { sel, .. }
            | Pick::Kind { sel, .. }
            | Pick::Cmp { sel, .. }
            | Pick::Values { sel, .. }
            | Pick::TabMenu { sel, .. }
            | Pick::Look { sel, .. } => *sel,
        }
    }

    pub(crate) fn sel_mut(&mut self) -> &mut usize {
        match self {
            Pick::Column { sel, .. }
            | Pick::Kind { sel, .. }
            | Pick::Cmp { sel, .. }
            | Pick::Values { sel, .. }
            | Pick::TabMenu { sel, .. }
            | Pick::Look { sel, .. } => sel,
        }
    }
}

/// 値を打つ条件の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextKind {
    Contains,
    NotContains,
    Cmp(CmpOp),
    /// mdgrid のビューの名前(BV-18): 名前を付けて保存・名前の変更。
    SaveAs,
    Rename,
    /// ビューの区画から、開いていないビューも含めて名前を変える(NV-26。`col` は元の名前)。
    RenameTab,
    /// 見た目のテンプレートの名前(SR-43)。
    LookTemplate,
    /// WBS の値の割合とラベル(NV-28。`col` は値)。
    WbsValue,
    /// ビューの削除の確かめ(BV-18。`col` はビューの名前)。`y` で消す。
    DeleteView,
}

/// 値の入力(含む・含まない・比べる。NV-19)。
#[derive(Debug, Clone)]
pub(crate) struct TextEntry {
    pub col: String,
    pub kind: TextKind,
    pub text: String,
    pub edit: Option<usize>,
}

/// 設定の画面で編集している写し(NV-13)。
#[derive(Debug, Clone)]
pub(crate) struct Draft {
    pub s: Settings,
    /// 列の並び(隠した列も入れる)と、表示するか(NV-18 の列の区画)。
    pub cols: Vec<(String, bool)>,
    /// 開いたときの列(変えていなければ、反映で列に触らない)。
    cols0: Vec<(String, bool)>,
    /// 列の選び手に出す列: 表の列(隠した列も)と、ビューに無いノートのキー(NV-14: どの列でも)。
    pub keys: Vec<String>,
    pub sec: Sec,
    /// 区画ごとに選んだ項目。
    pub sel: [usize; 9],
    pub pick: Option<Pick>,
    pub text: Option<TextEntry>,
    /// mdgrid のビューのボタンを出すか(読むだけ(WB-15)では出さない。BV-18)。
    pub view_buttons: bool,
    /// ビューの区画のタブの数(NV-26。区画の項目の数に使う。タブが変われば直す)。
    pub tabs: usize,
    /// 見た目の写しと、開いたときの見た目(SR-43)。違えば反映で画面に当てる。
    pub look: super::look_section::LookPick,
    pub look0: super::look_section::LookPick,
    /// 見た目の区画の項目の数(テンプレートの数で変わる)。
    pub look_n: usize,
    /// 保存先に選べる範囲の数(SR-43)。
    pub look_scopes: usize,
    /// 親子の区画の親のキー(オフの間も覚えておく。NV-27)。
    pub tree_key: String,
    /// WBS の写し(オフの間も覚えておく)と、進み具合のキーの値と件数(NV-28)。
    pub wbs: mdgrid::settings::Wbs,
    pub wbs_vals: Vec<(String, usize)>,
}

impl Draft {
    /// WBS の写しを、オンなら設定にも当てる。
    pub(crate) fn sync_wbs(&mut self) {
        if self.s.wbs.is_some() {
            self.s.wbs = Some(self.wbs.clone());
        }
    }
}

impl Draft {
    pub(crate) fn at(&self, sec: Sec) -> usize {
        self.sel[sec as usize]
    }

    /// 区画の項目の数(フィルターと並べ替えは末尾に「足す」)。
    pub(crate) fn len(&self, sec: Sec) -> usize {
        match sec {
            Sec::Columns => self.cols.len(),
            Sec::Filters => self.s.filters.len() + 1,
            Sec::Sorts => self.s.sorts.len() + 1,
            Sec::Group => GROUP_ITEMS,
            Sec::Display => mdgrid::display::ITEMS.len(),
            // タブの行と、末尾の切り替えの案内。
            Sec::Views => self.tabs + 1,
            Sec::Look => self.look_n,
            // NV-28: WBS がオンなら値の行も。
            Sec::Tree if self.s.wbs.is_some() => 4 + self.wbs_vals.len(),
            Sec::Tree => 4,
            Sec::Buttons if self.view_buttons => BUTTONS.len(),
            Sec::Buttons => VIEW_BUTTONS,
        }
    }

    /// ボタン i と同じ行のボタンの添字の範囲(画面の並び。←→ はこの中で動く)。
    pub(crate) fn button_row(&self, i: usize) -> std::ops::Range<usize> {
        // 上の右は反映・取り消し、下は mdgrid のビューのボタンと既定に戻す(NV-18)。
        if i < 2 {
            0..2
        } else {
            2..self.len(Sec::Buttons)
        }
    }

    pub(crate) fn pick_len(&self) -> usize {
        match &self.pick {
            None => 0,
            Some(Pick::Column { .. }) => self.keys.len(),
            Some(Pick::Kind { .. }) => KINDS.len(),
            Some(Pick::Cmp { .. }) => CMPS.len(),
            Some(Pick::Values { counts, .. }) => counts.len() + 1,
            Some(Pick::TabMenu { .. }) => super::view_tabs::TAB_MENU,
            Some(Pick::Look { field, .. }) => {
                super::look_section::pick_count(*field, self.look_scopes)
            }
        }
    }

    pub(crate) fn select(&mut self, sec: Sec, i: usize) {
        self.sec = sec;
        self.sel[sec as usize] = i;
    }

    /// 列の表示・順を開いたときから変えたか。
    pub(crate) fn cols_changed(&self) -> bool {
        self.cols != self.cols0
    }
}

/// 1つの条件の見せ方(帯の項目と同じ文。NV-16)。
pub(crate) fn cond_label(c: &Cond) -> String {
    let s = Settings {
        filters: vec![c.clone()],
        ..Default::default()
    };
    s.chips().into_iter().next().unwrap_or_default()
}

pub(crate) fn arrow(d: Dir) -> &'static str {
    match d {
        Dir::Asc => Msg::DirAsc.text(),
        Dir::Desc => Msg::DirDesc.text(),
    }
}

fn flip(d: Dir) -> Dir {
    match d {
        Dir::Asc => Dir::Desc,
        Dir::Desc => Dir::Asc,
    }
}

impl App {
    pub(crate) fn key_of(&self, mode: Mode, a: Action) -> String {
        keymap::key_for(&self.keys, mode, a).unwrap_or_else(|| a.name().to_string())
    }

    /// 列の並び(隠した列は隠したときの位置に入れる)と、表示しているか。
    pub(crate) fn column_list(&self) -> Vec<(String, bool)> {
        let mut order: Vec<(String, bool)> = self.cols.iter().map(|c| (c.clone(), true)).collect();
        for (c, j, _) in self.hidden.iter().rev() {
            order.insert((*j).min(order.len()), (c.clone(), false));
        }
        order
    }

    /// `o`(とパレットの `view_settings`): 今の設定の写しで、ビューの設定の画面を開く(NV-13)。
    /// NV-28: 進み具合のキーの値と件数(表の全部のノートから。多い順、同じなら名前の順)。対応表にある値は
    /// 件数が 0 でも並べる。
    pub(crate) fn wbs_values(&self, w: &mdgrid::settings::Wbs) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for r in self.src.rows() {
            if let Some(mdgrid::source::Value::Str(s)) = self.prop(&r, &w.key) {
                let s = s.trim();
                if !s.is_empty() {
                    *counts.entry(s.to_string()).or_insert(0) += 1;
                }
            }
        }
        for m in &w.map {
            counts.entry(m.value.clone()).or_insert(0);
        }
        let mut v: Vec<(String, usize)> = counts.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    pub(crate) fn open_settings(&mut self) {
        let wbs = self
            .settings
            .wbs
            .clone()
            .unwrap_or_else(|| mdgrid::settings::Wbs {
                key: "status".into(),
                map: Vec::new(),
            });
        let cols = self.column_list();
        let mut keys: Vec<String> = cols.iter().map(|c| c.0.clone()).collect();
        for k in self.src.columns() {
            if !keys.contains(&k) {
                keys.push(k);
            }
        }
        self.draft = Some(Draft {
            s: self.settings.clone(),
            cols0: cols.clone(),
            cols,
            keys,
            sec: Sec::Filters,
            sel: [0; 9],
            pick: None,
            text: None,
            view_buttons: !self.readonly,
            tabs: self.section_tabs().len(),
            look: self.look_pick_now(),
            look0: self.look_pick_now(),
            look_n: self.look_len(),
            look_scopes: self.look_scopes().len(),
            tree_key: self
                .settings
                .tree
                .clone()
                .unwrap_or_else(|| "parent".into()),
            wbs: wbs.clone(),
            wbs_vals: self.wbs_values(&wbs),
        });
        self.set_mode(Mode::Settings);
    }

    /// ビューの設定の画面の動作。
    pub(crate) fn settings_action(&mut self, action: Action) {
        if self.draft.is_none() {
            return self.set_mode(Mode::Table);
        }
        if self.mode == Mode::SettingsText {
            return self.text_action(action);
        }
        if action == Action::Help {
            return self.open_help();
        }
        if self.draft.as_ref().is_some_and(|d| d.pick.is_some()) {
            return self.pick_action(action);
        }
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let (sec, i, len) = (d.sec, d.at(d.sec), d.len(d.sec));
        match action {
            Action::Cancel => self.cancel_settings(),
            Action::NextSection | Action::PrevSection => {
                let k = SECS.iter().position(|s| *s == sec).unwrap_or(0);
                let n = SECS.len();
                d.sec = SECS[if action == Action::NextSection {
                    (k + 1) % n
                } else {
                    (k + n - 1) % n
                }];
            }
            Action::Up => d.sel[sec as usize] = i.saturating_sub(1),
            Action::Down => d.sel[sec as usize] = (i + 1).min(len.saturating_sub(1)),
            // ←→ は画面の同じ行のボタンの中で動く。
            Action::Left if sec == Sec::Buttons => {
                d.sel[sec as usize] = i.saturating_sub(1).max(d.button_row(i).start)
            }
            Action::Right if sec == Sec::Buttons => {
                d.sel[sec as usize] = (i + 1).min(d.button_row(i).end.saturating_sub(1))
            }
            Action::Run => self.settings_run(false),
            Action::Toggle => self.settings_run(true),
            Action::MoveItemUp => self.move_item(true),
            Action::MoveItemDown => self.move_item(false),
            Action::RemoveItem => self.remove_item(),
            _ => {}
        }
    }

    /// Enter(`space` は Space)で選んだ項目を決める・切り替える。
    fn settings_run(&mut self, space: bool) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let (sec, i) = (d.sec, d.at(d.sec));
        match sec {
            Sec::Columns => self.toggle_column(i),
            Sec::Filters => {
                if i >= d.s.filters.len() {
                    d.pick = Some(Pick::Column {
                        purpose: Purpose::Filter,
                        sel: 0,
                    });
                    return;
                }
                // Space は値の一覧の条件の 残す ↔ 隠す(NV-14)。
                let op = &mut d.s.filters[i].op;
                if space && matches!(op, Op::Keep(_) | Op::Drop(_)) {
                    *op = match std::mem::replace(op, Op::Empty) {
                        Op::Keep(k) => Op::Drop(k),
                        Op::Drop(k) => Op::Keep(k),
                        o => o,
                    };
                } else {
                    self.edit_cond(i);
                }
            }
            Sec::Sorts => match d.s.sorts.get_mut(i) {
                Some((_, dir)) => *dir = flip(*dir),
                None => {
                    d.pick = Some(Pick::Column {
                        purpose: Purpose::Sort,
                        sel: 0,
                    })
                }
            },
            Sec::Group => match i {
                0 => d.s.group = Group::Inherit,
                1 => d.s.group = Group::Off,
                2 => {
                    let at = match &d.s.group {
                        Group::By { col, .. } => d.keys.iter().position(|c| c == col),
                        _ => None,
                    };
                    d.pick = Some(Pick::Column {
                        purpose: Purpose::Group,
                        sel: at.unwrap_or(0),
                    });
                }
                _ => match &mut d.s.group {
                    Group::By {
                        dir, hide_empty, ..
                    } => {
                        if i == 3 {
                            *hide_empty = !*hide_empty;
                        } else {
                            *dir = flip(*dir);
                        }
                    }
                    _ => self.message = Some(Msg::GroupOnlyByColumn.into()),
                },
            },
            Sec::Display => self.toggle_display(i),
            Sec::Views => self.views_run(i, space),
            Sec::Look => self.look_run(i),
            // NV-27: 0 は親子で並べるかの切り替え、1 は親のキーを選ぶ。
            Sec::Tree => {
                if i == 0 {
                    d.s.tree = match d.s.tree {
                        Some(_) => None,
                        None => Some(d.tree_key.clone()),
                    };
                } else if i == 1 {
                    let at = d.keys.iter().position(|c| *c == d.tree_key);
                    d.pick = Some(Pick::Column {
                        purpose: Purpose::Tree,
                        sel: at.unwrap_or(0),
                    });
                } else if i == 2 {
                    // NV-28: WBS のオン・オフ。
                    d.s.wbs = match d.s.wbs {
                        Some(_) => None,
                        None => Some(d.wbs.clone()),
                    };
                } else if i == 3 {
                    let at = d.keys.iter().position(|c| *c == d.wbs.key);
                    d.pick = Some(Pick::Column {
                        purpose: Purpose::WbsKey,
                        sel: at.unwrap_or(0),
                    });
                } else if let Some((v, _)) = d.wbs_vals.get(i - 4).cloned() {
                    let now = d
                        .wbs
                        .of(&v)
                        .map(|m| format!("{} {}", m.percent, m.label).trim_end().to_string())
                        .unwrap_or_default();
                    self.open_text(v, TextKind::WbsValue, now, None);
                }
            }
            Sec::Buttons => match i {
                0 => self.apply_draft(),
                1 => self.cancel_settings(),
                2 => self.reset_draft(),
                // BV-18: mdgrid のビューのボタン(native_views.rs)。
                _ => self.view_button(i),
            },
        }
    }

    /// 列の表示・非表示(NV-18)。最後の1列は隠さない(NV-4)。
    fn toggle_column(&mut self, i: usize) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let shown = d.cols.iter().filter(|c| c.1).count();
        let Some(c) = d.cols.get_mut(i) else {
            return;
        };
        if c.1 && shown <= 1 {
            self.message = Some(Msg::LastColumnHide.into());
            return;
        }
        c.1 = !c.1;
    }

    /// 列・並べ替え・フィルターの順を動かす(K / J)。
    fn move_item(&mut self, up: bool) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let (sec, i) = (d.sec, d.at(d.sec));
        if sec == Sec::Views {
            return self.move_tab(i, up);
        }
        let n = match sec {
            Sec::Columns => d.cols.len(),
            Sec::Filters => d.s.filters.len(),
            Sec::Sorts => d.s.sorts.len(),
            _ => return,
        };
        let j = if up { i.checked_sub(1) } else { Some(i + 1) };
        let Some(j) = j.filter(|j| *j < n && i < n) else {
            return;
        };
        match sec {
            Sec::Columns => d.cols.swap(i, j),
            Sec::Filters => d.s.filters.swap(i, j),
            _ => d.s.sorts.swap(i, j),
        }
        d.sel[sec as usize] = j;
    }

    /// 選んだフィルター・並べ替えを消す。グループの区画では「.base のまま」に戻す。
    fn remove_item(&mut self) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let (sec, i) = (d.sec, d.at(d.sec));
        match sec {
            Sec::Filters if i < d.s.filters.len() => {
                d.s.filters.remove(i);
            }
            Sec::Sorts if i < d.s.sorts.len() => {
                d.s.sorts.remove(i);
            }
            Sec::Group => d.s.group = Group::Inherit,
            Sec::Look => {
                if !self.remove_look_template(i) {
                    self.message = Some(Msg::NotRemovable.into());
                }
                return;
            }
            _ => {
                self.message = Some(Msg::NotRemovable.into());
                return;
            }
        }
        let len = d.len(sec);
        d.sel[sec as usize] = i.min(len.saturating_sub(1));
    }

    /// 「取り消し」・Esc: 写しを捨てて表へ(表は開く前のまま。NV-13)。
    fn cancel_settings(&mut self) {
        // 写しを変えていなければ「閉じた」(ビューの区画の変更はもう保存してある。NV-26)。
        let changed = self
            .draft
            .take()
            .is_some_and(|d| d.s != self.settings || d.cols_changed());
        self.set_mode(Mode::Table);
        self.message = Some(if changed {
            Msg::SettingsCancelled.into()
        } else {
            Msg::SettingsClosed.into()
        });
    }

    /// 「既定に戻す」(NV-22): 写しを `.base` のビューの既定(`.base` なしなら何もしない状態)と
    /// `.base` の列の順(全部を表示)にする。反映で表に効く。
    fn reset_draft(&mut self) {
        let order: Vec<String> = match self.build_grid() {
            Ok(g) if !g.columns.is_empty() => g.columns.into_iter().map(|c| c.id).collect(),
            _ => self.column_list().into_iter().map(|c| c.0).collect(),
        };
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        d.s = Settings::default();
        d.cols = order.into_iter().map(|c| (c, true)).collect();
        d.select(Sec::Buttons, 0);
        self.message = Some(Msg::SettingsReset.into());
    }

    /// 「反映」(NV-13): 写しを App に当てて組み立て直し、見た目の状態に書く(WB-15 では書かない)。
    fn apply_draft(&mut self) {
        let Some(mut d) = self.draft.take() else {
            return;
        };
        self.set_mode(Mode::Table);
        // SR-43: 見た目を変えていれば、選んだ範囲の層に当てて残す(ビューの範囲は写しの設定に)。書けなかった理由と
        // 重ねの警告は、反映した旨のあとに出す。
        let mut notes = Vec::new();
        if d.look != d.look0 {
            self.message = None;
            self.apply_look(&mut d.s, d.look, d.look0);
            notes.extend(self.message.take());
        }
        self.settings = d.s;
        notes.extend(self.apply_profile());
        if d.cols != d.cols0 {
            // 選んでいた列は名前で引き直す。隠したなら、写しの並びで最寄りの表示する列(右を先に)。
            let picked = self.cols.get(self.col).cloned();
            let at = picked.and_then(|p| d.cols.iter().position(|(c, _)| *c == p));
            let target = at.and_then(|p| {
                let right = d.cols[p..].iter().find(|c| c.1);
                let left = || d.cols[..p].iter().rev().find(|c| c.1);
                right.or_else(left).map(|c| c.0.clone())
            });
            let mut cols = Vec::new();
            let mut hidden = Vec::new();
            for (c, shown) in d.cols {
                if shown {
                    cols.push(c);
                } else {
                    hidden.push((c, cols.len(), false));
                }
            }
            self.cols = cols;
            self.hidden = hidden;
            self.kinds = self.cols.iter().map(|c| self.column_kind(c)).collect();
            self.frozen = self.frozen.min(self.cols.len());
            self.col = target
                .and_then(|t| self.cols.iter().position(|c| *c == t))
                .unwrap_or(self.col)
                .min(self.cols.len().saturating_sub(1));
            self.left = 0;
        }
        // 並びが変わるので、直した行の留め(NV-12)はやめて本来の位置へ。
        self.stay.clear();
        self.regrid = true;
        self.persist_state();
        let done: String = if self.readonly {
            Msg::SettingsAppliedReadOnly.into()
        } else {
            Msg::SettingsApplied.into()
        };
        notes.insert(0, done);
        self.message = Some(notes.join(" / "));
    }

    /// NV-3・NV-24: 並べ替えの決まりを今のビューの設定に当て、組み直して見た目の状態に残す。
    pub(crate) fn set_sorts(&mut self, sorts: Vec<(String, mdgrid::settings::Dir)>) {
        self.settings.sorts = sorts;
        self.sort = None;
        // 並びが変わるので、直した行の留め(NV-12)はやめて本来の位置へ。
        self.stay.clear();
        self.regrid = true;
        self.refresh_if_needed();
        self.persist_state();
    }

    // ---- 表の上の設定の帯(NV-16・NV-22) ----

    /// `f`: 設定の帯の項目を選ぶ。
    pub(crate) fn focus_chips(&mut self) {
        if self.settings.no_conditions() {
            let o = self.key_of(Mode::Table, Action::ViewSettings);
            self.message = Some(Msg::ChipsEmpty.fill(&[&o]));
            return;
        }
        self.chip = 0;
        self.set_mode(Mode::Chips);
        self.hidden_chip_hint();
    }

    /// 設定の帯の中の動作。
    pub(crate) fn chips_action(&mut self, action: Action) {
        let n = self.settings.chips().len();
        match action {
            Action::Left => {
                self.chip = self.chip.saturating_sub(1);
                self.hidden_chip_hint();
            }
            Action::Right => {
                self.chip = (self.chip + 1).min(n.saturating_sub(1));
                self.hidden_chip_hint();
            }
            Action::RemoveItem => self.remove_chip(self.chip),
            Action::Close => self.set_mode(Mode::Table),
            Action::ViewSettings => {
                self.set_mode(Mode::Table);
                self.open_settings();
            }
            Action::Help => self.open_help(),
            _ => {}
        }
        if self.mode == Mode::Chips && self.settings.no_conditions() {
            self.set_mode(Mode::Table);
        }
    }

    /// 帯の i 番目の項目の条件を外す(NV-22)。項目の順は フィルター → 並べ替え → グループ(`chips()`)。
    pub(crate) fn remove_chip(&mut self, i: usize) {
        let Some(text) = self.settings.chips().get(i).cloned() else {
            return;
        };
        let nf = self.settings.filters.len();
        let ns = self.settings.sorts.len();
        if i < nf {
            self.settings.filters.remove(i);
        } else if i < nf + ns {
            self.settings.sorts.remove(i - nf);
        } else {
            self.settings.group = Group::Inherit;
        }
        self.chip = self.chip.min(self.settings.chips().len().saturating_sub(1));
        self.stay.clear();
        self.regrid = true;
        self.persist_state();
        if self.settings.no_conditions() && self.mode == Mode::Chips {
            self.set_mode(Mode::Table);
        }
        self.removed_chip_message(&text);
    }
}
