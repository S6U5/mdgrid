//! ビューの設定の画面の「見た目」の区画(`impl App` の続き。SR-43)。
//! テーマ・組・丸い札の端を写し(`Draft::look`)で選び、「反映」で画面に当てて look.toml に残す。
//! テンプレート(名前を付けた組み合わせ)は区画の下に並べ、選ぶと写しに当てる。テンプレートの保存・削除と
//! 「config.toml に戻す」はビューの一覧の管理と同じく、反映を待たずにすぐ look.toml に書く。
//! 読むだけ(WB-15)では画面には当てるが書かない。読み書きは核の `mdgrid::look`。

use super::app::App;
use super::keymap::Mode;
use super::settings::{Pick, Sec, TextKind};
use super::startup::READONLY;
use mdgrid::i18n::Msg;
use mdgrid::look::{self, Look, LookFile, Nerd, ThemeChoice};
use mdgrid::style::{Preset, Style};

/// 区画の写しの見た目(3つとも決まった値)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LookPick {
    pub theme: ThemeChoice,
    pub preset: Preset,
    pub nerd: Nerd,
}

impl LookPick {
    fn look(self) -> Look {
        Look {
            theme: Some(self.theme),
            preset: Some(self.preset),
            nerd: Some(self.nerd),
        }
    }

    /// テンプレートの見た目を重ねる(テンプレートに無い項目は今のまま)。
    fn with(self, l: &Look) -> LookPick {
        LookPick {
            theme: l.theme.unwrap_or(self.theme),
            preset: l.preset.unwrap_or(self.preset),
            nerd: l.nerd.unwrap_or(self.nerd),
        }
    }

    /// 一行の見せ方(「dracula · dozy-pink · true」)。
    pub(crate) fn summary(l: &Look) -> String {
        let names: Vec<&str> = [
            l.theme.map(|t| t.name()),
            l.preset.map(|p| p.name()),
            l.nerd.map(|n| n.name()),
        ]
        .into_iter()
        .flatten()
        .collect();
        names.join(" · ")
    }
}

/// 区画の項目の並び: テーマ・組・丸い札の端・テンプレート…・名前を付けて保存・config.toml に戻す。
pub(crate) const LOOK_FIELDS: usize = 3;

impl App {
    /// 設定の置き場の look.toml(無ければ・置き場が無ければ空)。
    pub(crate) fn look_file(&self) -> LookFile {
        self.nv
            .dir
            .as_ref()
            .map(|d| look::load(d).0)
            .unwrap_or_default()
    }

    /// 写しの始まり: look.toml にあればその値、無ければ今の画面の見た目。
    pub(crate) fn look_pick_now(&self) -> LookPick {
        let now = LookPick {
            theme: ThemeChoice::Named(self.theme),
            preset: self.style.preset,
            nerd: if self.nerd_font { Nerd::On } else { Nerd::Off },
        };
        now.with(&self.look_file().look)
    }

    /// 区画の項目の数(テンプレートの数で変わる)。
    pub(crate) fn look_len(&self) -> usize {
        LOOK_FIELDS + self.look_file().templates.len() + 2
    }

    /// Enter・Space: 項目ごとの操作。
    pub(crate) fn look_run(&mut self, i: usize) {
        let templates = self.look_file().templates;
        let nt = templates.len();
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let lk = d.look;
        if i < LOOK_FIELDS {
            let sel = match i {
                0 => ThemeChoice::all().iter().position(|t| *t == lk.theme),
                1 => Preset::NAMES.iter().position(|n| *n == lk.preset.name()),
                _ => Nerd::ALL.iter().position(|n| *n == lk.nerd),
            };
            d.pick = Some(Pick::Look {
                field: i,
                sel: sel.unwrap_or(0),
            });
            return;
        }
        let k = i - LOOK_FIELDS;
        if let Some((name, l)) = templates.get(k) {
            d.look = lk.with(l);
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
    pub(crate) fn look_pick_items(field: usize) -> Vec<String> {
        match field {
            0 => ThemeChoice::all()
                .into_iter()
                .map(|t| t.name().to_string())
                .collect(),
            1 => Preset::NAMES.iter().map(|s| s.to_string()).collect(),
            _ => Nerd::ALL
                .into_iter()
                .map(|n| match n {
                    Nerd::Auto => Msg::LookNerdAuto.to_string(),
                    Nerd::On => Msg::LookNerdOn.to_string(),
                    Nerd::Off => Msg::LookNerdOff.to_string(),
                })
                .collect(),
        }
    }

    /// 選び手で決めた。
    pub(crate) fn look_pick_run(&mut self, field: usize, sel: usize) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        d.pick = None;
        match field {
            0 => {
                if let Some(t) = ThemeChoice::all().get(sel) {
                    d.look.theme = *t;
                }
            }
            1 => {
                if let Some(p) = Preset::NAMES.get(sel).and_then(|n| Preset::parse(n)) {
                    d.look.preset = p;
                }
            }
            _ => {
                if let Some(n) = Nerd::ALL.get(sel) {
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

    fn save_look_file(&mut self, f: &LookFile) -> bool {
        let Some(dir) = self.nv.dir.clone() else {
            return false;
        };
        match look::save(&dir, f) {
            Ok(()) => true,
            Err(e) => {
                self.message = Some(Msg::CannotWriteFile.fill(&[&look::FILE_NAME, &e]));
                false
            }
        }
    }

    /// 写しの見た目をテンプレートとして保存する(同じ名前があれば置き換える)。
    pub(crate) fn save_look_template(&mut self, name: String) {
        let Some(lk) = self.draft.as_ref().map(|d| d.look) else {
            return;
        };
        let mut f = self.look_file();
        match f.templates.iter_mut().find(|t| t.0 == name) {
            Some(t) => t.1 = lk.look(),
            None => f.templates.push((name.clone(), lk.look())),
        }
        if let Some(d) = self.draft.as_mut() {
            d.text = None;
        }
        self.set_mode(Mode::Settings);
        if self.save_look_file(&f) {
            self.message = Some(Msg::LookTemplateSaved.fill(&[&name]));
        }
        let n = self.look_len();
        if let Some(d) = self.draft.as_mut() {
            d.look_n = n;
        }
    }

    /// 区画の i 番目がテンプレートなら消す(すぐ書く)。消せたら true。
    pub(crate) fn remove_look_template(&mut self, i: usize) -> bool {
        let mut f = self.look_file();
        let Some(k) = i
            .checked_sub(LOOK_FIELDS)
            .filter(|k| *k < f.templates.len())
        else {
            return false;
        };
        if !self.look_writable() {
            return true;
        }
        let (name, _) = f.templates.remove(k);
        if self.save_look_file(&f) {
            self.message = Some(Msg::LookTemplateDeleted.fill(&[&name]));
        }
        let n = self.look_len();
        if let Some(d) = self.draft.as_mut() {
            d.look_n = n;
            d.sel[Sec::Look as usize] = i.min(n - 1);
        }
        true
    }

    /// 「config.toml に戻す」: look.toml の見た目を外す(テンプレートは残す)。次の起動から config.toml のとおり。
    fn look_reset(&mut self) {
        if !self.look_writable() {
            return;
        }
        let mut f = self.look_file();
        f.look = Look::default();
        if self.save_look_file(&f) {
            self.message = Some(Msg::LookReset.into());
        }
    }

    /// 反映(SR-43): 写しの見た目を画面に当て、look.toml に残す(読むだけでは書かない)。
    pub(crate) fn apply_look(&mut self, lk: LookPick) {
        match lk.theme {
            ThemeChoice::Named(t) => self.theme = t,
            // auto は起動のときに端末の地で決める。今は今のテーマのまま。
            ThemeChoice::Auto => {}
        }
        self.style = Style::of(lk.preset);
        self.nerd_font = match lk.nerd {
            Nerd::On => true,
            Nerd::Off => false,
            Nerd::Auto => mdgrid::style::nerd_auto(std::env::var("TERM_PROGRAM").ok().as_deref()),
        };
        if self.readonly || self.nv.dir.is_none() {
            return;
        }
        let mut f = self.look_file();
        f.look = lk.look();
        self.save_look_file(&f);
    }
}
