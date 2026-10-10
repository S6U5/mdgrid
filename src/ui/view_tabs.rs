//! ビューの設定の画面の「ビュー」の区画(`impl App` の続き。NV-26)。
//! 開いた対象のタブ(`.base` のビュー・既定の表・mdgrid のビュー)を見せる順に並べ、出す・隠す(Space)、
//! 順(K / J)、既定・名前の変更・削除(Enter の選び手)、末尾の行で切り替えの案内を変える。
//! ここの変更は今のビューの見せ方の写しではないので、反映を待たずに views.toml の対象の項目へすぐ書く
//! (核の `views::save_tab_prefs`・`save_default_view`、ビューの名前と削除は `edit_views`)。
//! 読むだけ(WB-15)では見せるだけで書かない。

use super::app::App;
use super::keymap::Mode;
use super::settings::{Pick, Sec, TextKind};
use super::startup::READONLY;
use mdgrid::i18n::Msg;
use mdgrid::views::{self, TabPrefs};

/// タブの選び手の項目(既定・名前の変更・削除)。
pub(crate) const TAB_MENU: usize = 3;

impl App {
    /// 区画に並べるタブの名前(見せる順。`.base` も mdgrid のビューも無ければ既定の表の1つ)と、
    /// `view_names` の添字。
    pub(crate) fn section_tabs(&self) -> Vec<(String, Option<usize>)> {
        let names = self.view_names();
        if names.is_empty() {
            return vec![(Msg::DefaultTabName.text().to_string(), None)];
        }
        self.tab_order()
            .into_iter()
            .map(|i| (names[i].clone(), Some(i)))
            .collect()
    }

    /// 既定のビューの名前(views.toml。無ければ None)。
    pub(crate) fn default_view_name(&self) -> Option<String> {
        views::load_default_view(self.nv.dir.as_ref()?, &self.nv.target)
    }

    /// 書けるか(読むだけ・置き場なしなら理由を出して false)。
    fn tabs_writable(&mut self) -> bool {
        if self.readonly {
            self.message = Some(READONLY.into());
            return false;
        }
        if self.nv.dir.is_none() {
            self.message = Some(Self::NO_DIR.into());
            return false;
        }
        true
    }

    /// タブの好みを書いて取り込む。書けなければ理由。
    fn save_tabs(&mut self, p: TabPrefs, done: String) {
        let Some(dir) = self.nv.dir.clone() else {
            return;
        };
        match views::save_tab_prefs(&dir, &self.nv.target, &p) {
            Ok(()) => {
                self.nv.tabs = p;
                self.message = Some(done);
            }
            Err(e) => {
                self.message = Some(Msg::CannotWriteFile.fill(&[&views::FILE_NAME, &e]));
            }
        }
    }

    /// Space・Enter: タブの行なら Space で出す・隠す、Enter で選び手。末尾の行は切り替えの案内。
    pub(crate) fn views_run(&mut self, i: usize, space: bool) {
        let tabs = self.section_tabs();
        if i >= tabs.len() {
            return self.toggle_tab_hint();
        }
        if space {
            return self.toggle_tab(i);
        }
        if let Some(d) = self.draft.as_mut() {
            d.pick = Some(Pick::TabMenu { tab: i, sel: 0 });
        }
    }

    fn toggle_tab_hint(&mut self) {
        if !self.tabs_writable() {
            return;
        }
        let mut p = self.nv.tabs.clone();
        p.hint = !p.hint;
        let done = if p.hint {
            Msg::TabHintOn
        } else {
            Msg::TabHintOff
        };
        self.save_tabs(p, done.into());
    }

    /// 区画の i 番目のタブを出す・隠す。開いているビューは隠さない。
    fn toggle_tab(&mut self, i: usize) {
        if !self.tabs_writable() {
            return;
        }
        let Some((name, at)) = self.section_tabs().get(i).cloned() else {
            return;
        };
        let mut p = self.nv.tabs.clone();
        if let Some(k) = p.hidden.iter().position(|h| *h == name) {
            p.hidden.remove(k);
            return self.save_tabs(p, Msg::TabShown.fill(&[&name]));
        }
        if at.is_none_or(|k| k == self.view_index()) {
            self.message = Some(Msg::TabCurrentHide.into());
            return;
        }
        p.hidden.push(name.clone());
        self.save_tabs(p, Msg::TabHidden.fill(&[&name]));
    }

    /// K / J: 区画の i 番目のタブを上・下へ。順は名前の並びで全部を書く。
    pub(crate) fn move_tab(&mut self, i: usize, up: bool) {
        let tabs = self.section_tabs();
        let j = if up { i.checked_sub(1) } else { Some(i + 1) };
        let Some(j) = j.filter(|j| *j < tabs.len() && i < tabs.len()) else {
            return;
        };
        if !self.tabs_writable() {
            return;
        }
        let mut order: Vec<String> = tabs.into_iter().map(|t| t.0).collect();
        order.swap(i, j);
        let name = order[j].clone();
        let mut p = self.nv.tabs.clone();
        p.order = order;
        self.save_tabs(p, Msg::TabMoved.fill(&[&name]));
        if let Some(d) = self.draft.as_mut() {
            d.sel[Sec::Views as usize] = j;
        }
    }

    /// タブの選び手の項目の文(既定にする・やめる・名前を変える・削除)。
    pub(crate) fn tab_menu_items(&self, tab: usize) -> Vec<String> {
        let name = self.section_tabs().get(tab).map(|t| t.0.clone());
        let is_default = name.is_some() && name == self.default_view_name();
        vec![
            if is_default {
                Msg::TabMenuUndefault
            } else {
                Msg::TabMenuDefault
            }
            .to_string(),
            Msg::TabMenuRename.to_string(),
            Msg::TabMenuDelete.to_string(),
        ]
    }

    /// タブの選び手で決めた: 0 既定にする(既定ならやめる)・1 名前を変える・2 削除。
    pub(crate) fn tab_menu_run(&mut self, tab: usize, sel: usize) {
        let Some((name, at)) = self.section_tabs().get(tab).cloned() else {
            return;
        };
        if let Some(d) = self.draft.as_mut() {
            d.pick = None;
        }
        if !self.tabs_writable() {
            return;
        }
        // mdgrid のビューの添字(`.base` のビューと既定の表は None)。
        let native = at.and_then(|i| i.checked_sub(self.base_tabs()));
        match sel {
            0 => self.toggle_default(&name),
            1 => match native {
                Some(_) => self.open_text(name.clone(), TextKind::RenameTab, name, None),
                None => self.message = Some(Msg::TabBaseFixed.fill(&[&name])),
            },
            _ => match native {
                Some(k) if Some(k) == self.nv.at => {
                    self.view_button(super::settings::BUTTONS.len() - 1)
                }
                Some(_) => self.delete_tab(&name),
                None => self.message = Some(Msg::TabBaseFixed.fill(&[&name])),
            },
        }
    }

    fn toggle_default(&mut self, name: &str) {
        let Some(dir) = self.nv.dir.clone() else {
            return;
        };
        let clear = self.default_view_name().as_deref() == Some(name);
        let store = (!clear).then_some(name);
        self.message = Some(
            match views::save_default_view(&dir, &self.nv.target, store) {
                Err(e) => Msg::CannotWriteFile.fill(&[&views::FILE_NAME, &e]),
                Ok(()) if clear => Msg::DefaultViewCleared.text().into(),
                Ok(()) => Msg::DefaultViewSet.fill(&[&super::width::sanitize(name)]),
            },
        );
    }

    /// 開いていない mdgrid のビューを消す(設定の画面は開いたまま)。
    fn delete_tab(&mut self, name: &str) {
        let res = self.edit_views(|_, list| {
            if let Some(j) = list.iter().position(|v| v.name == name) {
                list.remove(j);
            }
            Ok(())
        });
        let list = match res {
            Ok((list, ())) => list,
            Err(e) => return self.message = Some(e),
        };
        self.move_state(name, None);
        let keep = self.native_view().map(|v| v.name.clone());
        self.adopt_views(list, keep.as_deref());
        self.rename_in_tabs(name, None);
        self.message = Some(Msg::ViewDeleted.fill(&[&name]));
        let n = self.section_tabs().len();
        if let Some(d) = self.draft.as_mut() {
            d.tabs = n;
            let s = &mut d.sel[Sec::Views as usize];
            *s = (*s).min(n.saturating_sub(1));
        }
    }

    /// 開いていない mdgrid のビューの名前を変える(設定の画面は開いたまま)。
    pub(crate) fn rename_tab(&mut self, old: String, name: String) {
        if old == name {
            if let Some(d) = self.draft.as_mut() {
                d.text = None;
            }
            return self.set_mode(Mode::Settings);
        }
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
        let current = self.native_view().map(|v| v.name.clone());
        if current.as_deref() == Some(old.as_str()) {
            self.persist_state();
        }
        self.move_state(&old, Some(&name));
        let keep = if current.as_deref() == Some(old.as_str()) {
            Some(name.clone())
        } else {
            current
        };
        self.adopt_views(list, keep.as_deref());
        self.refresh_native_synth();
        self.rename_in_tabs(&old, Some(&name));
        if let Some(d) = self.draft.as_mut() {
            d.text = None;
        }
        self.set_mode(Mode::Settings);
        self.message = Some(Msg::ViewRenamed.fill(&[&old, &name, &""]));
    }

    /// ビューの名前が変わった・消えたとき、タブの順・隠す・既定の名前も合わせる(読むだけでは書かない)。
    pub(crate) fn rename_in_tabs(&mut self, old: &str, new: Option<&str>) {
        if self.readonly {
            return;
        }
        let Some(dir) = self.nv.dir.clone() else {
            return;
        };
        let fix = |v: &mut Vec<String>| {
            match new {
                Some(n) => v
                    .iter_mut()
                    .filter(|x| *x == old)
                    .for_each(|x| *x = n.into()),
                None => v.retain(|x| x != old),
            };
        };
        let mut p = self.nv.tabs.clone();
        fix(&mut p.order);
        fix(&mut p.hidden);
        if p != self.nv.tabs && views::save_tab_prefs(&dir, &self.nv.target, &p).is_ok() {
            self.nv.tabs = p;
        }
        if self.default_view_name().as_deref() == Some(old) {
            let _ = views::save_default_view(&dir, &self.nv.target, new);
        }
    }
}
