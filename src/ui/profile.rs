//! 範囲ごとの表のプロファイル(SR-44。`impl App` の続き)。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! 層は 全体(config.toml → ui.toml)→ ワークスペース → 表(手で書いたもの → views.toml)→ ビュー(ビューの設定)。
//! 起動(`configure`・`start`)とビューの切り替え・反映のたびに `apply_profile` で重ね直し、画面が使う値
//! (部品の形・色・セルの部品・表示・日付・編集・新しいノート・テーマ)を当てる。見せ方のビューの上書き
//! (`settings.display`)は今までどおり display.rs の `shows` が重ねる。

use super::app::{App, ColorMode};
use mdgrid::config::{Config, NerdFont};
use mdgrid::profile::{self, Layer, Origin, Place, Resolved, Templates};
use mdgrid::uifile::UiFile;
use std::path::Path;

/// 範囲ごとの層と、重ねた値。
#[derive(Default)]
pub(crate) struct Prof {
    /// config.toml。
    pub config: Config,
    /// ui.toml(画面で選んだ全体の設定。main が起動の前に渡す)。
    pub ui: UiFile,
    /// ワークスペース・表(手で書いたもの)・表(views.toml)の層。範囲を決めたあとに `start` が集める。
    pub scope: Vec<Layer>,
    /// 今のビューまで重ねた値。
    pub resolved: Resolved,
    /// 丸い札の端の設定(ui.toml があればそれ、無ければ config.toml)。
    pub nerd: NerdFont,
    /// 端末の名前(`TERM_PROGRAM`。nerd_font = "auto" の決め方。main が渡す)。
    pub term_program: Option<String>,
    /// 端末の地の明るさを問い合わせる(SR-39。main が渡す。試験では無い)。
    pub probe: Option<Box<dyn FnMut() -> Option<bool>>>,
    /// 問い合わせた明るさ(None はまだ問い合わせていない)。
    pub light_bg: Option<Option<bool>>,
}

impl Prof {
    /// 全体の層(config.toml と ui.toml)と範囲の層。
    pub fn layers(&self) -> Vec<Layer> {
        let mut out = vec![self.config.layer()];
        if !self.ui.profile.is_empty() {
            out.push(self.ui.layer());
        }
        out.extend(self.scope.iter().cloned());
        out
    }

    /// テンプレート(config.toml のあとに ui.toml。同じ名前は ui.toml が先)。
    pub fn templates(&self) -> Templates {
        let mut t = self.config.templates.clone();
        t.extend(self.ui.templates.iter().cloned());
        t
    }
}

impl App {
    /// 範囲の層を集める(WS-6 で範囲を決めたあと)。`target` は開いた対象、`config_dir` は views.toml の置き場。
    /// views.toml の表のプロファイルの警告を返す。
    pub(crate) fn collect_scope_layers(
        &mut self,
        target: &Path,
        config_dir: Option<&Path>,
    ) -> Vec<String> {
        let mut layers = Vec::new();
        let first = target
            .to_string_lossy()
            .lines()
            .next()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| target.to_path_buf());
        if let Some(sc) = &self.scope {
            layers.extend(sc.layer());
            layers.extend(sc.table_layer(&first));
        }
        let mut warnings = Vec::new();
        if let Some(dir) = config_dir {
            let (p, w) = mdgrid::views::load_table_profile(dir, target);
            warnings.extend(w);
            if !p.is_empty() {
                let name = mdgrid::workspace::stem_of(&first);
                layers.push(Layer {
                    origin: Origin::new(
                        Place::TableApp,
                        format!("{} ({name})", mdgrid::views::FILE_NAME),
                    ),
                    profile: p,
                });
            }
        }
        self.prof.scope = layers;
        warnings
    }

    /// 層を重ね直して、画面が使う値を当てる(SR-44)。無いテンプレートなどの警告を返す。
    pub(crate) fn apply_profile(&mut self) -> Vec<String> {
        let mut layers = self.prof.layers();
        let view = self.settings.profile();
        if !view.is_empty() {
            layers.push(Layer {
                origin: Origin::new(Place::View, mdgrid::i18n::Msg::OriginView.text()),
                profile: view,
            });
        }
        let mut w = Vec::new();
        let r = profile::resolve(&layers, &self.prof.templates(), &mut w);
        self.style = r.style;
        self.colors = r.colors.clone();
        self.cells = r.cells.clone();
        self.display = r.display;
        self.candidates = r.candidates;
        self.date_format = r.date_format.clone();
        self.week_start = r.week_start;
        self.look_classic = r.classic;
        self.note.rule = r.new_note.clone();
        let bg = if r.theme.is_pair() {
            self.light_bg()
        } else {
            None
        };
        self.theme = r.theme.pick(bg);
        self.prof.resolved = r;
        w
    }

    /// 端末の地の明るさ(SR-39)。色を使わない表示では問い合わせない。問い合わせは1回だけ。
    fn light_bg(&mut self) -> Option<bool> {
        if self.color == ColorMode::None {
            return None;
        }
        if let Some(v) = self.prof.light_bg {
            return v;
        }
        let v = self.prof.probe.as_mut().and_then(|p| p());
        self.prof.light_bg = Some(v);
        v
    }
}
