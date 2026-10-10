//! 画面で選んだ全体の設定(CLI-3・SR-43): 設定の置き場の `ui.toml`。config.toml と同じ形で、表のプロファイル・
//! `[terminal] nerd_font`・`[templates.<名前>]` を持ち、config.toml の上に重ねる(config.toml は書き換えない)。
//! 前の版の `look.toml`(`theme`・`preset`・`nerd_font`・`[[template]]`)は、ui.toml が無いときに読み、次に書くときに
//! ui.toml へ移して消す(CLI-20)。壊れたファイルは警告して使わない。書くときは置き場の中の一時ファイル → 名前の変更
//! (config::write_atomic)。画面に依存しない。

use crate::config::{self, NerdFont};
use crate::i18n::Msg;
use crate::profile::{self, not, warn, Layer, Origin, Place, Profile, Templates, ThemeSpec};
use crate::style::Preset;
use std::io;
use std::path::Path;

/// 置き場の中のファイルの名前。
pub const FILE_NAME: &str = "ui.toml";
/// 前の版のファイル(CLI-20)。
pub const OLD_FILE_NAME: &str = "look.toml";

/// `ui.toml` の中身。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiFile {
    /// 全体の表のプロファイル(画面で選んだもの)。
    pub profile: Profile,
    /// 丸い札の端(端末の性質。SR-43 の区画で選ぶ)。
    pub nerd_font: Option<NerdFont>,
    /// 名前を付けたテンプレート(書いた順)。
    pub templates: Templates,
}

impl UiFile {
    pub fn is_empty(&self) -> bool {
        *self == UiFile::default()
    }

    /// 全体の画面の層(SR-44)。
    pub fn layer(&self) -> Layer {
        Layer {
            origin: Origin::new(Place::Ui, FILE_NAME),
            profile: self.profile.clone(),
        }
    }
}

/// ui.toml(無ければ look.toml)を読む。無ければ空。壊れていれば警告して空。
pub fn load(dir: &Path) -> (UiFile, Vec<String>) {
    let mut w = Vec::new();
    if let Ok(text) = std::fs::read_to_string(dir.join(FILE_NAME)) {
        return match text.parse::<toml::Table>() {
            Ok(t) => (read(&t, &mut w), w),
            Err(e) => {
                let msg = config::toml_error(&text, &e).1;
                w.push(Msg::LookBroken.fill(&[&FILE_NAME, &msg]));
                (UiFile::default(), w)
            }
        };
    }
    let Ok(text) = std::fs::read_to_string(dir.join(OLD_FILE_NAME)) else {
        return (UiFile::default(), w);
    };
    match text.parse::<toml::Table>() {
        Ok(t) => (read_old(&t, &mut w), w),
        Err(e) => {
            let msg = config::toml_error(&text, &e).1;
            w.push(Msg::LookBroken.fill(&[&OLD_FILE_NAME, &msg]));
            (UiFile::default(), w)
        }
    }
}

/// ui.toml の表を読む。
fn read(t: &toml::Table, w: &mut Vec<String>) -> UiFile {
    let mut f = UiFile::default();
    for (k, v) in t {
        if f.profile.read_key(k, v, FILE_NAME, "", w) {
            continue;
        }
        match k.as_str() {
            "terminal" => {
                let Some(tab) = v.as_table() else {
                    warn(w, FILE_NAME, k, not(Msg::WantTable));
                    continue;
                };
                for (kk, vv) in tab {
                    let p = format!("terminal.{kk}");
                    if kk != "nerd_font" {
                        warn(w, FILE_NAME, &p, Msg::RsnUnknown.text().to_string());
                        continue;
                    }
                    match NerdFont::parse(vv) {
                        Some(n) => f.nerd_font = Some(n),
                        None => warn(w, FILE_NAME, &p, not(Msg::WantNerdFont)),
                    }
                }
            }
            "templates" => f.templates = config::read_templates(v, FILE_NAME, w),
            _ => warn(w, FILE_NAME, k, Msg::RsnUnknown.text().to_string()),
        }
    }
    f
}

/// 前の版の look.toml を読む(`theme`・`preset`・`nerd_font` と `[[template]]`)。
fn read_old(t: &toml::Table, w: &mut Vec<String>) -> UiFile {
    let look = |t: &toml::Table, at: &str, w: &mut Vec<String>| -> (Profile, Option<NerdFont>) {
        let mut p = Profile::default();
        let mut nerd = None;
        for (k, v) in t {
            let path = format!("{at}{k}");
            match k.as_str() {
                "theme" => match ThemeSpec::parse(v) {
                    Some(s) => p.look.theme = Some(s),
                    None => warn(w, OLD_FILE_NAME, &path, not(Msg::WantTheme)),
                },
                "preset" => match v.as_str().and_then(Preset::parse) {
                    Some(x) => p.look.preset = Some(x),
                    None => warn(w, OLD_FILE_NAME, &path, profile::not_one_of(Preset::NAMES)),
                },
                "nerd_font" => match NerdFont::parse(v) {
                    Some(n) => nerd = Some(n),
                    None => warn(w, OLD_FILE_NAME, &path, not(Msg::WantNerdFont)),
                },
                "name" | "template" => {}
                _ => warn(w, OLD_FILE_NAME, &path, Msg::RsnUnknown.text().to_string()),
            }
        }
        (p, nerd)
    };
    let (profile, nerd_font) = look(t, "", w);
    let mut templates = Vec::new();
    for item in t
        .get("template")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(tt) = item.as_table() else {
            continue;
        };
        let Some(name) = tt.get("name").and_then(|v| v.as_str()) else {
            w.push(Msg::LookTemplateNoName.fill(&[&OLD_FILE_NAME]));
            continue;
        };
        templates.push((name.to_string(), look(tt, "template.", w).0));
    }
    UiFile {
        profile,
        nerd_font,
        templates,
    }
}

/// ui.toml の文にする。
pub fn to_toml(f: &UiFile) -> String {
    let mut t = f.profile.to_table();
    if let Some(n) = f.nerd_font {
        let mut term = toml::Table::new();
        term.insert("nerd_font".into(), n.to_value());
        t.insert("terminal".into(), toml::Value::Table(term));
    }
    if !f.templates.is_empty() {
        let ts = f
            .templates
            .iter()
            .map(|(n, p)| (n.clone(), toml::Value::Table(p.to_table())))
            .collect();
        t.insert("templates".into(), toml::Value::Table(ts));
    }
    toml::to_string(&t).unwrap_or_default()
}

/// ui.toml を書く(中身が空ならファイルを消す)。置き場が無ければ作る。前の版の look.toml は消す。
pub fn save(dir: &Path, f: &UiFile) -> io::Result<()> {
    let old = dir.join(OLD_FILE_NAME);
    if old.exists() {
        std::fs::remove_file(old)?;
    }
    if f.is_empty() {
        return match std::fs::remove_file(dir.join(FILE_NAME)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    std::fs::create_dir_all(dir)?;
    config::write_atomic(dir, FILE_NAME, to_toml(f).as_bytes())
}
