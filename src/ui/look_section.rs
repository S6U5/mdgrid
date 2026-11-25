//! ビューの設定の画面の「見た目」の区画(`impl App` の続き。SR-43・SR-44)。
//! 保存先の範囲(全体・このワークスペース・この表・このビュー)・テーマ・組・丸い札の端を写し(`Draft::look`)で選び、
//! 「反映」で画面に当てて、選んだ範囲の画面が書く場所(ui.toml・workspaces.toml・views.toml・ビューの設定)に残す。
//! テーマと組の横には、今の値がどの範囲から来たかを出す(印から来た値は「印」。screen.look-section-origin)。
//! テーマの一覧の最後の「カスタム」(screen.look-section-custom)は、今のテーマを元のテーマにして、元のテーマの行と
//! 役割ごとの色(`colors::ROLES`)の行を足し、反映で元のテーマと `[look.colors]` の役割の色として残す。
//! テンプレート(config.toml と ui.toml の `[templates]`)は区画の下に並べ、選ぶと写しに当てる。テンプレートの保存・
//! 削除と「この範囲の上書きを外す」は、反映を待たずにすぐ書く。読むだけ(WB-15)では画面には当てるが書かない。

use super::app::App;
use super::keymap::Mode;
use super::settings::{Pick, Sec, TextKind};
use super::startup::READONLY;
use mdgrid::colors::{self, NAMED, ROLES};
use mdgrid::config::NerdFont;
use mdgrid::i18n::Msg;
use mdgrid::profile::{Layer, LookLayer, Origin, Place, Profile, ThemeSpec};
use mdgrid::style::Preset;
use mdgrid::theme::Theme;
use mdgrid::workspace::Source;

/// 役割の色の上書き(`colors::ROLES` の順)。
pub(crate) type Roles = [Option<[u8; 3]>; 16];

/// 区画の写しの見た目(どれも決まった値)。比べるときは `custom`(行を出すかどうか)を見ない: 「カスタム」を選んだ
/// だけで色を変えていなければ、反映で書くものが無いので未反映に数えない。
#[derive(Debug, Clone, Copy)]
pub(crate) struct LookPick {
    /// 保存先の範囲(`Ui`・`Workspace`・`TableApp`・`View`)。
    pub scope: Place,
    /// テーマ(カスタムのときは元のテーマ)。
    pub theme: ThemeSpec,
    pub preset: Preset,
    pub nerd: NerdFont,
    /// カスタム(screen.look-section-custom): 元のテーマの行と役割の行を出す。
    pub custom: bool,
    /// 役割の色の上書き。カスタムでなければ全部 None。
    pub colors: Roles,
}

impl PartialEq for LookPick {
    fn eq(&self, o: &LookPick) -> bool {
        (self.scope, self.theme, self.preset, self.nerd, self.colors)
            == (o.scope, o.theme, o.preset, o.nerd, o.colors)
    }
}

impl Eq for LookPick {}

impl LookPick {
    /// テンプレートの見た目を重ねる(テンプレートに無い項目は今のまま。テーマがあれば役割の色もテンプレートのもの)。
    fn with(self, p: &Profile) -> LookPick {
        let mut out = LookPick {
            theme: p.look.theme.unwrap_or(self.theme),
            preset: p.look.preset.unwrap_or(self.preset),
            ..self
        };
        if p.look.theme.is_some() {
            out.colors = p.look.roles;
            out.custom = has_roles(&p.look.roles);
        }
        out
    }

    /// テーマの欄の文(「dozy-pink」・「カスタム(元: dozy-pink)」。screen.look-section-custom-label)。
    pub(crate) fn theme_text(theme: ThemeSpec, custom: bool) -> String {
        if custom {
            Msg::LookCustomOf.fill(&[&theme.label()])
        } else {
            theme.label()
        }
    }

    /// 一行の見せ方(「dracula · dozy-pink」)。
    pub(crate) fn summary(p: &Profile) -> String {
        let custom = has_roles(&p.look.roles);
        let names: Vec<String> = [
            p.look.theme.map(|t| LookPick::theme_text(t, custom)),
            p.look.preset.map(|x| x.name().to_string()),
        ]
        .into_iter()
        .flatten()
        .collect();
        names.join(" · ")
    }

    /// カスタムのときに足す行の数(元のテーマと役割)。
    pub(crate) fn extra(self) -> usize {
        if self.custom {
            1 + ROLES.len()
        } else {
            0
        }
    }

    /// テンプレートの行の始まり(決まった項目と、カスタムの行のあと)。
    pub(crate) fn fields(self) -> usize {
        LOOK_FIELDS + self.extra()
    }
}

/// 役割の色の上書きがあるか(値の色はカスタムに数えない)。
fn has_roles(r: &Roles) -> bool {
    r.iter().any(Option::is_some)
}

/// 区画の項目の並び: 保存先・テーマ・組・丸い札の端・(カスタムなら)元のテーマ・役割…・テンプレート…・
/// 名前を付けて保存・この範囲の上書きを外す。
pub(crate) const LOOK_FIELDS: usize = 4;
/// カスタムの元のテーマの行。
pub(crate) const BASE_FIELD: usize = LOOK_FIELDS;

/// 役割の行の番号の役割(`colors::ROLES` の添字)。役割の行でなければ None。
pub(crate) fn role_of_field(field: usize) -> Option<usize> {
    field
        .checked_sub(BASE_FIELD + 1)
        .filter(|i| *i < ROLES.len())
}

/// 役割の見せる名前(screen.color-overrides-roles の並び)。
pub(crate) fn role_label(i: usize) -> Msg {
    const LABELS: [Msg; 16] = [
        Msg::LookRoleBackground,
        Msg::LookRoleText,
        Msg::LookRoleHeader,
        Msg::LookRoleSelection,
        Msg::LookRoleSelectionText,
        Msg::LookRoleBand,
        Msg::LookRoleBandText,
        Msg::LookRoleAccent,
        Msg::LookRoleStrong,
        Msg::LookRoleZebra,
        Msg::LookRoleZebraText,
        Msg::LookRoleMark,
        Msg::LookRoleAdded,
        Msg::LookRoleRemoved,
        Msg::LookRolePending,
        Msg::LookRoleHighlight,
    ];
    LABELS[i.min(LABELS.len() - 1)]
}

/// テーマの選び手の並び(auto のあとにテーマの並び)。テーマの行ではこのあとに「カスタム」が付く。
fn theme_choices() -> Vec<ThemeSpec> {
    std::iter::once(ThemeSpec::AUTO)
        .chain(Theme::ALL.into_iter().map(ThemeSpec::Named))
        .collect()
}

/// 役割の色の選び手の項目の数: テーマのまま・見本の色・`#rrggbb` を打つ。
const COLOR_CHOICES: usize = 1 + NAMED.len() + 1;

/// 選び手の項目の数(保存先は選べる範囲の数 `scopes`)。
pub(crate) fn pick_count(field: usize, scopes: usize) -> usize {
    match field {
        0 => scopes,
        1 => theme_choices().len() + 1,
        2 => Preset::NAMES.len(),
        3 => NerdFont::ALL.len(),
        BASE_FIELD => theme_choices().len(),
        _ => COLOR_CHOICES,
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
        if self
            .scope
            .as_ref()
            .and_then(mdgrid::workspace::Scope::label)
            .is_some()
        {
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

    /// 写しの始まり: 今の決まった値(テーマは auto・明暗の組のまま)。保存先は全体。役割の色の上書きがあればカスタム
    /// (screen.look-section-custom-label)。
    pub(crate) fn look_pick_now(&self) -> LookPick {
        let r = &self.prof.resolved;
        LookPick {
            scope: Place::Ui,
            theme: r.theme,
            preset: r.preset,
            nerd: self.prof.nerd,
            custom: has_roles(&r.colors.roles),
            colors: r.colors.roles,
        }
    }

    /// 項目の値の出どころの範囲の名前(「(この表)」。印から来た値は「(印)」。screen.look-section-origin)。
    pub(crate) fn look_origin(&self, path: &str) -> Msg {
        match self.prof.resolved.origin_of(path) {
            Some(o) if o.by_author => Msg::OriginMarker,
            Some(o) => place_label(o.place),
            None => Msg::OriginDefault,
        }
    }

    /// テーマ `theme` の役割 `i` の色(色の表の無いテーマは None)。明暗の組は問い合わせた地の明るさで選ぶ。
    pub(crate) fn theme_role(&self, theme: ThemeSpec, i: usize) -> Option<[u8; 3]> {
        let p = theme.pick(self.prof.light_bg.flatten()).palette()?;
        colors::palette_roles(&p).get(i).copied()
    }

    /// 区画の項目の数(テンプレートの数と、写しのカスタムの行で変わる)。
    pub(crate) fn look_len_of(&self, lk: LookPick) -> usize {
        lk.fields() + self.look_templates().len() + 2
    }

    /// 区画の項目の数(写しがあればその見た目で)。
    pub(crate) fn look_len(&self) -> usize {
        let lk = self
            .draft
            .as_ref()
            .map_or_else(|| self.look_pick_now(), |d| d.look);
        self.look_len_of(lk)
    }

    /// 写しの見た目が変わったら、区画の項目の数を合わせる(選んでいた行は収まる所へ)。
    fn sync_look_len(&mut self) {
        let n = self.look_len();
        if let Some(d) = self.draft.as_mut() {
            d.look_n = n;
            let s = &mut d.sel[Sec::Look as usize];
            *s = (*s).min(n.saturating_sub(1));
        }
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
        let nf = lk.fields();
        if i < nf {
            let themes = theme_choices();
            let sel = match i {
                0 => scopes.iter().position(|s| *s == lk.scope),
                1 if lk.custom => Some(themes.len()),
                1 | BASE_FIELD => themes.iter().position(|t| *t == lk.theme),
                2 => Preset::NAMES.iter().position(|n| *n == lk.preset.name()),
                3 => NerdFont::ALL.iter().position(|n| *n == lk.nerd),
                f => Some(match role_of_field(f).and_then(|r| lk.colors[r]) {
                    None => 0,
                    Some(c) => NAMED
                        .iter()
                        .position(|(_, x)| *x == c)
                        .map_or(COLOR_CHOICES - 1, |k| k + 1),
                }),
            };
            d.pick = Some(Pick::Look {
                field: i,
                sel: sel.unwrap_or(0),
            });
            return;
        }
        let k = i - nf;
        if let Some((name, p, _)) = templates.get(k) {
            d.look = lk.with(p);
            self.message = Some(Msg::LookTemplateUsed.fill(&[&name]));
            self.sync_look_len();
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
        let themes = || theme_choices().into_iter().map(ThemeSpec::label);
        match field {
            0 => self
                .look_scopes()
                .into_iter()
                .map(|p| place_label(p).to_string())
                .collect(),
            1 => themes()
                .chain(std::iter::once(Msg::LookCustom.to_string()))
                .collect(),
            2 => Preset::NAMES
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            3 => NerdFont::ALL.into_iter().map(nerd_label).collect(),
            BASE_FIELD => themes().collect(),
            f => {
                let keep = match role_of_field(f).and_then(|r| self.draft_theme_role(r)) {
                    Some(c) => format!("{}  {}", Msg::LookColorKeep, colors::hex(c)),
                    None => Msg::LookColorKeep.to_string(),
                };
                std::iter::once(keep)
                    .chain(
                        NAMED
                            .iter()
                            .map(|(n, c)| format!("{n}  {}", colors::hex(*c))),
                    )
                    .chain(std::iter::once(Msg::LookColorType.to_string()))
                    .collect()
            }
        }
    }

    /// 選び手の項目の色の見本(役割の色の選び手だけ。ほかは空)。
    pub(crate) fn look_pick_swatches(&self, field: usize) -> Vec<Option<[u8; 3]>> {
        let Some(r) = role_of_field(field) else {
            return Vec::new();
        };
        std::iter::once(self.draft_theme_role(r))
            .chain(NAMED.iter().map(|(_, c)| Some(*c)))
            .chain(std::iter::once(None))
            .collect()
    }

    /// 写しの元のテーマの役割 `r` の色。
    pub(crate) fn draft_theme_role(&self, r: usize) -> Option<[u8; 3]> {
        let theme = self.draft.as_ref()?.look.theme;
        self.theme_role(theme, r)
    }

    /// 選び手で決めた。
    pub(crate) fn look_pick_run(&mut self, field: usize, sel: usize) {
        let scopes = self.look_scopes();
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        d.pick = None;
        let themes = theme_choices();
        match field {
            0 => {
                if let Some(s) = scopes.get(sel) {
                    d.look.scope = *s;
                }
            }
            // テーマの一覧の最後の「カスタム」は今のテーマを元にする。名前のテーマはカスタムをやめる。
            1 => {
                if sel == themes.len() {
                    d.look.custom = true;
                } else if let Some(t) = themes.get(sel) {
                    d.look.theme = *t;
                    d.look.custom = false;
                    d.look.colors = [None; 16];
                }
                self.sync_look_len();
            }
            2 => {
                if let Some(p) = Preset::NAMES.get(sel).and_then(|n| Preset::parse(n)) {
                    d.look.preset = p;
                }
            }
            3 => {
                if let Some(n) = NerdFont::ALL.get(sel) {
                    d.look.nerd = *n;
                }
            }
            BASE_FIELD => {
                if let Some(t) = themes.get(sel) {
                    d.look.theme = *t;
                }
            }
            f => {
                let Some(r) = role_of_field(f) else {
                    return;
                };
                match sel {
                    0 => d.look.colors[r] = None,
                    k if k <= NAMED.len() => d.look.colors[r] = Some(NAMED[k - 1].1),
                    _ => self.open_text(
                        ROLES[r].to_string(),
                        TextKind::LookColor,
                        String::new(),
                        None,
                    ),
                }
            }
        }
    }

    /// カスタムの役割の色を打って決めた(`role` は役割の設定の名前)。読めない色は理由を出して入力を開いたままにする
    /// (screen.look-section-custom-colors)。
    pub(crate) fn put_look_color(&mut self, role: &str, text: &str) {
        let Some(rgb) = colors::parse_color(text) else {
            self.message = Some(Msg::LookColorBad.fill(&[&text]));
            return;
        };
        let Some(i) = ROLES.iter().position(|r| *r == role) else {
            return;
        };
        if let Some(d) = self.draft.as_mut() {
            d.look.colors[i] = Some(rgb);
            d.text = None;
        }
        self.set_mode(Mode::Settings);
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

    /// 写しの見た目をテンプレートとして ui.toml に保存する(同じ名前があれば置き換える)。カスタムなら役割の色も。
    pub(crate) fn save_look_template(&mut self, name: String) {
        let Some(lk) = self.draft.as_ref().map(|d| d.look) else {
            return;
        };
        let mut p = Profile::default();
        p.look.theme = Some(lk.theme);
        p.look.preset = Some(lk.preset);
        p.look.roles = lk.colors;
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
        self.sync_look_len();
    }

    /// 区画の i 番目がテンプレートなら消す(すぐ書く)。消せたら(か、消せない理由を出したら)true。
    pub(crate) fn remove_look_template(&mut self, i: usize) -> bool {
        let templates = self.look_templates();
        let nf = self.draft.as_ref().map_or(LOOK_FIELDS, |d| d.look.fields());
        let Some((name, _, mine)) = i.checked_sub(nf).and_then(|k| templates.get(k)).cloned()
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
        self.sync_look_len();
        true
    }

    /// 範囲の画面が書く層(無ければ作る)。ワークスペースの範囲が無ければ None。
    fn app_layer_mut(&mut self, place: Place) -> Option<&mut LookLayer> {
        match place {
            Place::Ui => Some(&mut self.prof.ui.profile.look),
            Place::Workspace | Place::TableApp => {
                let i =
                    if let Some(i) = self.prof.scope.iter().position(|l| l.origin.place == place) {
                        i
                    } else {
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

    /// 「この範囲の上書きを外す」: 写しの保存先の範囲に画面が書いたテーマ・組・役割の色を外す(すぐ書いて画面に当てる)。
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
                d.s.look.roles = [None; 16];
            }
        }
        if let Some(l) = self.app_layer_mut(place) {
            l.theme = None;
            l.preset = None;
            l.roles = [None; 16];
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
        self.sync_look_len();
        self.message = Some(if warns.is_empty() {
            Msg::LookReset.fill(&[&place_label(place).text()])
        } else {
            warns.join(" / ")
        });
    }

    /// 反映(SR-43): 写しの見た目を選んだ範囲の層に当てて残し(読むだけでは書かない)、画面に当てる。
    /// 層には、開いたときから変えた項目と、もともとその層にあった項目だけを書く(変えていない組を書くと、
    /// 広い範囲の部品ごとの形が効かなくなるため)。丸い札の端はどの範囲でも ui.toml。`view` はビューの設定の写し。
    /// テーマか役割の色を変えたら、テーマ(カスタムなら元のテーマ)と、写しの役割の色をそのまま書く(書いたテーマは
    /// 広い範囲の役割の色を使わないので、上書きの色もその層にそろえて持つ。screen.look-section-custom-save)。
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
        let roles = lk.colors != lk0.colors;
        if !(theme || preset || roles) || self.marker_scope(lk.scope) {
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
        if theme || roles || l.theme.is_some() {
            l.theme = Some(lk.theme);
        }
        if preset || l.preset.is_some() {
            l.preset = Some(lk.preset);
        }
        if theme || roles {
            l.roles = lk.colors;
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
