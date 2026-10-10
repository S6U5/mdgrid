//! ビューの設定の画面の「見た目」の区画(`impl App` の続き。SR-43・SR-44)。
//! 保存先の範囲(全体・このワークスペース・この表・このビュー)・テーマ・組・丸い札の端を写し(`Draft::look`)で選び、
//! 「反映」で画面に当てて、選んだ範囲の画面が書く場所(ui.toml・workspaces.toml・views.toml・ビューの設定)に残す。
//! テーマと組の横には、今の値がどの範囲から来たかを出す。テンプレート(config.toml と ui.toml の `[templates]`)は
//! 区画の下に並べ、選ぶと写しに当てる。テンプレートの保存・削除と「この範囲の上書きを外す」は、反映を待たずに
//! すぐ書く。読むだけ(WB-15)では画面には当てるが書かない。

use super::app::App;
use super::keymap::Mode;
use super::settings::{Pick, Sec, TextKind};
use super::startup::READONLY;
use mdgrid::config::NerdFont;
use mdgrid::i18n::Msg;
use mdgrid::profile::{Layer, LookLayer, Origin, Place, Profile, ThemeSpec};
use mdgrid::style::Preset;
use mdgrid::theme::Theme;
use mdgrid::workspace::Source;

/// 区画の写しの見た目(どれも決まった値)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LookPick {
    /// 保存先の範囲(`Ui`・`Workspace`・`TableApp`・`View`)。
    pub scope: Place,
    pub theme: ThemeSpec,
    pub preset: Preset,
    pub nerd: NerdFont,
}

impl LookPick {
    /// テンプレートの見た目を重ねる(テンプレートに無い項目は今のまま)。
    fn with(self, p: &Profile) -> LookPick {
        LookPick {
            theme: p.look.theme.unwrap_or(self.theme),
            preset: p.look.preset.unwrap_or(self.preset),
            ..self
        }
    }

    /// 一行の見せ方(「dracula · dozy-pink」)。
    pub(crate) fn summary(p: &Profile) -> String {
        let names: Vec<String> = [
            p.look.theme.map(|t| t.label()),
            p.look.preset.map(|x| x.name().to_string()),
        ]
        .into_iter()
        .flatten()
        .collect();
        names.join(" · ")
    }
}

/// 区画の項目の並び: 保存先・テーマ・組・丸い札の端・テンプレート…・名前を付けて保存・この範囲の上書きを外す。
pub(crate) const LOOK_FIELDS: usize = 4;

/// テーマの選び手の並び(auto のあとにテーマの並び)。
fn theme_choices() -> Vec<ThemeSpec> {
    std::iter::once(ThemeSpec::AUTO)
        .chain(Theme::ALL.into_iter().map(ThemeSpec::Named))
        .collect()
}

/// 選び手の項目の数(保存先は選べる範囲の数 `scopes`)。
pub(crate) fn pick_count(field: usize, scopes: usize) -> usize {
    match field {
        0 => scopes,
        1 => theme_choices().len(),
        2 => Preset::NAMES.len(),
        _ => NerdFont::ALL.len(),
    }
}

/// 範囲の名前(区画の保存先と、値の出どころ)。
pub(crate) fn place_label(p: Place) -> Msg {
    match p {
        Place::Default => Msg::OriginDefault,
        Place::Config => Msg::OriginConfig,
        Place::Ui => Msg::OriginGlobal,
        Place::Workspace => Msg::OriginWorkspace,
        Place::TableHand | Place::TableApp => Msg::OriginTable,
        Place::View => Msg::OriginThisView,
    }
}

impl App {
    /// 保存先に選べる範囲(SR-43): 全体・(範囲があれば)このワークスペース・(置き場があれば)この表・このビュー。
    pub(crate) fn look_scopes(&self) -> Vec<Place> {
        let mut v = vec![Place::Ui];
        if self.scope.as_ref().and_then(|s| s.label()).is_some() {
            v.push(Place::Workspace);
        }
        if self.nv.dir.is_some() {
            v.push(Place::TableApp);
        }
        v.push(Place::View);
        v
    }

    /// テンプレート(名前ごとに1つ。同じ名前は ui.toml のもの)と、ui.toml のものか。
    pub(crate) fn look_templates(&self) -> Vec<(String, Profile, bool)> {
        let mut out: Vec<(String, Profile, bool)> = Vec::new();
        for (n, p) in &self.prof.config.templates {
            out.push((n.clone(), p.clone(), false));
        }
        for (n, p) in &self.prof.ui.templates {
            match out.iter_mut().find(|t| t.0 == *n) {
                Some(t) => *t = (n.clone(), p.clone(), true),
                None => out.push((n.clone(), p.clone(), true)),
            }
        }
        out
    }

    /// 写しの始まり: 今の決まった値(テーマは auto・明暗の組のまま)。保存先は全体。
    pub(crate) fn look_pick_now(&self) -> LookPick {
        let r = &self.prof.resolved;
        LookPick {
            scope: Place::Ui,
            theme: r.theme,
            preset: r.preset,
            nerd: self.prof.nerd,
        }
    }

    /// 項目の値の出どころの範囲(「(この表)」)。
    pub(crate) fn look_origin(&self, path: &str) -> Place {
        self.prof.resolved.origin(path)
    }

    /// 区画の項目の数(テンプレートの数で変わる)。
    pub(crate) fn look_len(&self) -> usize {
        LOOK_FIELDS + self.look_templates().len() + 2
    }

    /// Enter・Space: 項目ごとの操作。
    pub(crate) fn look_run(&mut self, i: usize) {
        let templates = self.look_templates();
        let nt = templates.len();
        let scopes = self.look_scopes();
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let lk = d.look;
        if i < LOOK_FIELDS {
            let sel = match i {
                0 => scopes.iter().position(|s| *s == lk.scope),
                1 => theme_choices().iter().position(|t| *t == lk.theme),
                2 => Preset::NAMES.iter().position(|n| *n == lk.preset.name()),
                _ => NerdFont::ALL.iter().position(|n| *n == lk.nerd),
            };
            d.pick = Some(Pick::Look {
                field: i,
                sel: sel.unwrap_or(0),
            });
            return;
        }
        let k = i - LOOK_FIELDS;
        if let Some((name, p, _)) = templates.get(k) {
            d.look = lk.with(p);
            self.message = Some(Msg::LookTemplateUsed.fill(&[&name]));
        } else if k == nt {
            if self.look_writable() {
                self.open_text(String::new(), TextKind::LookTemplate, String::new(), None);
            }
        } else {
            self.look_reset();
        }
    }

    /// 選び手の項目の文。
    pub(crate) fn look_pick_items(&self, field: usize) -> Vec<String> {
        match field {
            0 => self
                .look_scopes()
                .into_iter()
                .map(|p| place_label(p).to_string())
                .collect(),
            1 => theme_choices().into_iter().map(|t| t.label()).collect(),
            2 => Preset::NAMES.iter().map(|s| s.to_string()).collect(),
            _ => NerdFont::ALL.into_iter().map(nerd_label).collect(),
        }
    }

    /// 選び手で決めた。
    pub(crate) fn look_pick_run(&mut self, field: usize, sel: usize) {
        let scopes = self.look_scopes();
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        d.pick = None;
        match field {
            0 => {
                if let Some(s) = scopes.get(sel) {
                    d.look.scope = *s;
                }
            }
            1 => {
                if let Some(t) = theme_choices().get(sel) {
                    d.look.theme = *t;
                }
            }
            2 => {
                if let Some(p) = Preset::NAMES.get(sel).and_then(|n| Preset::parse(n)) {
                    d.look.preset = p;
                }
            }
            _ => {
                if let Some(n) = NerdFont::ALL.get(sel) {
                    d.look.nerd = *n;
                }
            }
        }
    }

    fn look_writable(&mut self) -> bool {
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

    /// ui.toml を書く(置き場が無い・読むだけなら書かない)。書けなければ理由を出す。
    fn save_ui(&mut self) -> bool {
        let Some(dir) = self.nv.dir.clone() else {
            return false;
        };
        if self.readonly {
            return false;
        }
        match mdgrid::uifile::save(&dir, &self.prof.ui) {
            Ok(()) => true,
            Err(e) => {
                self.message = Some(Msg::CannotWriteFile.fill(&[&mdgrid::uifile::FILE_NAME, &e]));
                false
            }
        }
    }

    /// 写しの見た目をテンプレートとして ui.toml に保存する(同じ名前があれば置き換える)。
    pub(crate) fn save_look_template(&mut self, name: String) {
        let Some(lk) = self.draft.as_ref().map(|d| d.look) else {
            return;
        };
        let mut p = Profile::default();
        p.look.theme = Some(lk.theme);
        p.look.preset = Some(lk.preset);
        match self.prof.ui.templates.iter_mut().find(|t| t.0 == name) {
            Some(t) => t.1 = p,
            None => self.prof.ui.templates.push((name.clone(), p)),
        }
        if let Some(d) = self.draft.as_mut() {
            d.text = None;
        }
        self.set_mode(Mode::Settings);
        if self.save_ui() {
            self.message = Some(Msg::LookTemplateSaved.fill(&[&name]));
        }
        let n = self.look_len();
        if let Some(d) = self.draft.as_mut() {
            d.look_n = n;
        }
    }

    /// 区画の i 番目がテンプレートなら消す(すぐ書く)。消せたら(か、消せない理由を出したら)true。
    pub(crate) fn remove_look_template(&mut self, i: usize) -> bool {
        let templates = self.look_templates();
        let Some((name, _, mine)) = i
            .checked_sub(LOOK_FIELDS)
            .and_then(|k| templates.get(k))
            .cloned()
        else {
            return false;
        };
        if !mine {
            self.message = Some(Msg::LookTemplateInConfig.fill(&[&name]));
            return true;
        }
        if !self.look_writable() {
            return true;
        }
        self.prof.ui.templates.retain(|t| t.0 != name);
        if self.save_ui() {
            self.message = Some(Msg::LookTemplateDeleted.fill(&[&name]));
        }
        let n = self.look_len();
        if let Some(d) = self.draft.as_mut() {
            d.look_n = n;
            d.sel[Sec::Look as usize] = i.min(n - 1);
        }
        true
    }

    /// 範囲の画面が書く層(無ければ作る)。ワークスペースの範囲が無ければ None。
    fn app_layer_mut(&mut self, place: Place) -> Option<&mut LookLayer> {
        match place {
            Place::Ui => Some(&mut self.prof.ui.profile.look),
            Place::Workspace | Place::TableApp => {
                let i = match self.prof.scope.iter().position(|l| l.origin.place == place) {
                    Some(i) => i,
                    None => {
                        let label = match place {
                            Place::Workspace => self.scope.as_ref()?.label()?,
                            _ => format!(
                                "{} ({})",
                                mdgrid::views::FILE_NAME,
                                mdgrid::workspace::stem_of(&self.nv.target)
                            ),
                        };
                        // 範囲の層は 広い → 狭い の順に並べる。
                        let at = self
                            .prof
                            .scope
                            .iter()
                            .position(|l| l.origin.place > place)
                            .unwrap_or(self.prof.scope.len());
                        self.prof.scope.insert(
                            at,
                            Layer {
                                origin: Origin::new(place, label),
                                profile: Profile::default(),
                            },
                        );
                        at
                    }
                };
                Some(&mut self.prof.scope[i].profile.look)
            }
            _ => Some(&mut self.settings.look),
        }
    }

    /// 範囲の画面が書く層をファイルに書く(ビューの範囲はビューの設定と一緒に残るので書かない)。
    fn save_layer(&mut self, place: Place) {
        if self.readonly {
            return;
        }
        let Some(dir) = self.nv.dir.clone() else {
            return;
        };
        let profile = |app: &App| {
            app.prof
                .scope
                .iter()
                .find(|l| l.origin.place == place)
                .map(|l| l.profile.clone())
                .unwrap_or_default()
        };
        let result = match place {
            Place::Ui => {
                self.save_ui();
                Ok(())
            }
            Place::Workspace => {
                let p = profile(self);
                let name = self.scope.as_ref().map(|s| s.name.clone());
                if let Some(sc) = self.scope.as_mut() {
                    sc.profile = p.clone();
                }
                match name {
                    Some(n) => mdgrid::workspace::save_profile(&dir, &n, &p)
                        .map_err(|e| (mdgrid::workspace::FILE_NAME, e)),
                    None => Ok(()),
                }
            }
            Place::TableApp => {
                let p = profile(self);
                mdgrid::views::save_table_profile(&dir, &self.nv.target, &p)
                    .map_err(|e| (mdgrid::views::FILE_NAME, e))
            }
            _ => Ok(()),
        };
        if let Err((file, e)) = result {
            self.message = Some(Msg::CannotWriteFile.fill(&[&file, &e]));
        }
    }

    /// 保存先が印のワークスペースか(印はノートのフォルダの中なので画面からは書かない。SR-43)。
    fn marker_scope(&mut self, place: Place) -> bool {
        let marker = place == Place::Workspace
            && self
                .scope
                .as_ref()
                .is_some_and(|s| s.source == Source::Marker);
        if marker {
            self.message = Some(Msg::LookMarkerReadOnly.into());
        }
        marker
    }

    /// 「この範囲の上書きを外す」: 写しの保存先の範囲に画面が書いたテーマと組を外す(すぐ書いて画面に当てる)。
    fn look_reset(&mut self) {
        let Some(place) = self.draft.as_ref().map(|d| d.look.scope) else {
            return;
        };
        if self.marker_scope(place) {
            return;
        }
        if place == Place::View {
            if let Some(d) = self.draft.as_mut() {
                d.s.look.theme = None;
                d.s.look.preset = None;
            }
        }
        if let Some(l) = self.app_layer_mut(place) {
            l.theme = None;
            l.preset = None;
        }
        if place == Place::Ui {
            self.prof.ui.nerd_font = None;
        }
        self.save_layer(place);
        let warns = self.apply_profile();
        let now = self.look_pick_now();
        if let Some(d) = self.draft.as_mut() {
            d.look = LookPick {
                scope: place,
                ..now
            };
            d.look0 = d.look;
        }
        self.message = Some(if warns.is_empty() {
            Msg::LookReset.fill(&[&place_label(place).text()])
        } else {
            warns.join(" / ")
        });
    }

    /// 反映(SR-43): 写しの見た目を選んだ範囲の層に当てて残し(読むだけでは書かない)、画面に当てる。
    /// 層には、開いたときから変えた項目と、もともとその層にあった項目だけを書く(変えていない組を書くと、
    /// 広い範囲の部品ごとの形が効かなくなるため)。丸い札の端はどの範囲でも ui.toml。`view` はビューの設定の写し。
    pub(crate) fn apply_look(
        &mut self,
        view: &mut mdgrid::settings::Settings,
        lk: LookPick,
        lk0: LookPick,
    ) {
        if lk.nerd != lk0.nerd {
            self.prof.nerd = lk.nerd;
            self.prof.ui.nerd_font = Some(lk.nerd);
            self.nerd_font = lk.nerd.resolve(self.prof.term_program.as_deref());
            self.save_ui();
        }
        let (theme, preset) = (lk.theme != lk0.theme, lk.preset != lk0.preset);
        if !(theme || preset) || self.marker_scope(lk.scope) {
            return;
        }
        let target = if lk.scope == Place::View {
            Some(&mut view.look)
        } else {
            self.app_layer_mut(lk.scope)
        };
        let Some(l) = target else {
            return;
        };
        if theme || l.theme.is_some() {
            l.theme = Some(lk.theme);
        }
        if preset || l.preset.is_some() {
            l.preset = Some(lk.preset);
        }
        self.save_layer(lk.scope);
    }
}

/// 丸い札の端の選び手の文。
fn nerd_label(n: NerdFont) -> String {
    match n {
        NerdFont::Auto => Msg::LookNerdAuto.to_string(),
        NerdFont::On => Msg::LookNerdOn.to_string(),
        NerdFont::Off => Msg::LookNerdOff.to_string(),
    }
}

/// 写しの丸い札の端の文(区画の行)。
pub(crate) fn nerd_text(n: NerdFont) -> String {
    nerd_label(n)
}
