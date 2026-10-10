//! 設定の画面の「見た目」の区画で選んだ見た目(SR-43)。設定の置き場の `look.toml` に、テーマ・組・丸い札の端と、
//! 名前を付けたテンプレートを持つ。config.toml は人が書いた注釈を持つので書き換えず、起動では config.toml を
//! 読んだあとにこれを重ねる(`apply`)。壊れた `look.toml` は警告して使わない。書くときは置き場の中の一時ファイル →
//! 名前の変更(config::write_atomic)。画面に依存しない。

use crate::config::{self, Config};
use crate::i18n::Msg;
use crate::style::{Preset, Style};
use crate::theme::Theme;
use std::io;
use std::path::Path;

/// 置き場の中のファイルの名前。
pub const FILE_NAME: &str = "look.toml";

/// 丸い札の端(`nerd_font`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nerd {
    Auto,
    On,
    Off,
}

impl Nerd {
    pub const ALL: [Nerd; 3] = [Nerd::Auto, Nerd::On, Nerd::Off];

    pub fn name(self) -> &'static str {
        match self {
            Nerd::Auto => "auto",
            Nerd::On => "true",
            Nerd::Off => "false",
        }
    }

    fn of(v: &toml::Value) -> Option<Nerd> {
        match (v.as_bool(), v.as_str()) {
            (Some(true), _) => Some(Nerd::On),
            (Some(false), _) => Some(Nerd::Off),
            (_, Some("auto")) => Some(Nerd::Auto),
            _ => None,
        }
    }

    fn value(self) -> toml::Value {
        match self {
            Nerd::Auto => toml::Value::String("auto".into()),
            Nerd::On => toml::Value::Boolean(true),
            Nerd::Off => toml::Value::Boolean(false),
        }
    }
}

/// 見た目の組み合わせ。None の項目は config.toml のまま。`theme` は `auto` か Theme の名前。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Look {
    pub theme: Option<ThemeChoice>,
    pub preset: Option<Preset>,
    pub nerd: Option<Nerd>,
}

/// テーマの選び: `auto`(SR-39)か名前のテーマ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Auto,
    Named(Theme),
}

impl ThemeChoice {
    pub fn name(self) -> &'static str {
        match self {
            ThemeChoice::Auto => "auto",
            ThemeChoice::Named(t) => t.name(),
        }
    }

    pub fn parse(s: &str) -> Option<ThemeChoice> {
        if s == "auto" {
            Some(ThemeChoice::Auto)
        } else {
            Theme::parse(s).map(ThemeChoice::Named)
        }
    }

    /// 選べる並び(auto のあとにテーマの並び)。
    pub fn all() -> Vec<ThemeChoice> {
        std::iter::once(ThemeChoice::Auto)
            .chain(Theme::ALL.into_iter().map(ThemeChoice::Named))
            .collect()
    }
}

impl Look {
    pub fn is_empty(&self) -> bool {
        *self == Look::default()
    }
}

/// `look.toml` の中身: 今の見た目と、名前を付けたテンプレート(書いた順)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookFile {
    pub look: Look,
    pub templates: Vec<(String, Look)>,
}

const KEYS: &[&str] = &["theme", "preset", "nerd_font", "template"];
const TEMPLATE_KEYS: &[&str] = &["name", "theme", "preset", "nerd_font"];

/// 表から見た目を読む(読めない値は警告して外す)。
fn read_look(t: &toml::Table, at: &str, warns: &mut Vec<String>) -> Look {
    let mut l = Look::default();
    if let Some(v) = t.get("theme") {
        l.theme = v.as_str().and_then(ThemeChoice::parse);
        if l.theme.is_none() {
            warns.push(Msg::LookBadValue.fill(&[&FILE_NAME, &format!("{at}theme"), &v]));
        }
    }
    if let Some(v) = t.get("preset") {
        l.preset = v.as_str().and_then(Preset::parse);
        if l.preset.is_none() {
            warns.push(Msg::LookBadValue.fill(&[&FILE_NAME, &format!("{at}preset"), &v]));
        }
    }
    if let Some(v) = t.get("nerd_font") {
        l.nerd = Nerd::of(v);
        if l.nerd.is_none() {
            warns.push(Msg::LookBadValue.fill(&[&FILE_NAME, &format!("{at}nerd_font"), &v]));
        }
    }
    l
}

/// `look.toml` を読む。無ければ空。壊れていれば警告して空。知らない項目は警告して読む。
pub fn load(dir: &Path) -> (LookFile, Vec<String>) {
    let mut warns = Vec::new();
    let text = match std::fs::read_to_string(dir.join(FILE_NAME)) {
        Ok(t) => t,
        Err(_) => return (LookFile::default(), warns),
    };
    let table = match text.parse::<toml::Table>() {
        Ok(t) => t,
        Err(e) => {
            let msg = config::toml_error(&text, &e).1;
            warns.push(Msg::LookBroken.fill(&[&FILE_NAME, &msg]));
            return (LookFile::default(), warns);
        }
    };
    for k in table.keys() {
        if !KEYS.contains(&k.as_str()) {
            warns.push(Msg::LookUnknownKey.fill(&[&FILE_NAME, &k]));
        }
    }
    let look = read_look(&table, "", &mut warns);
    let mut templates = Vec::new();
    for t in table
        .get("template")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(t) = t.as_table() else {
            continue;
        };
        let Some(name) = t.get("name").and_then(|v| v.as_str()) else {
            warns.push(Msg::LookTemplateNoName.fill(&[&FILE_NAME]));
            continue;
        };
        for k in t.keys() {
            if !TEMPLATE_KEYS.contains(&k.as_str()) {
                warns.push(Msg::LookUnknownKey.fill(&[&FILE_NAME, &format!("template.{k}")]));
            }
        }
        let l = read_look(t, "template.", &mut warns);
        templates.push((name.to_string(), l));
    }
    (LookFile { look, templates }, warns)
}

fn look_table(l: &Look, t: &mut toml::Table) {
    if let Some(v) = l.theme {
        t.insert("theme".into(), toml::Value::String(v.name().into()));
    }
    if let Some(p) = l.preset {
        t.insert("preset".into(), toml::Value::String(p.name().into()));
    }
    if let Some(n) = l.nerd {
        t.insert("nerd_font".into(), n.value());
    }
}

/// `look.toml` を書く(中身が空ならファイルを消す)。置き場が無ければ作る。
pub fn save(dir: &Path, f: &LookFile) -> io::Result<()> {
    if f.look.is_empty() && f.templates.is_empty() {
        return match std::fs::remove_file(dir.join(FILE_NAME)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    let mut table = toml::Table::new();
    look_table(&f.look, &mut table);
    if !f.templates.is_empty() {
        let arr = f
            .templates
            .iter()
            .map(|(name, l)| {
                let mut t = toml::Table::new();
                t.insert("name".into(), toml::Value::String(name.clone()));
                look_table(l, &mut t);
                toml::Value::Table(t)
            })
            .collect();
        table.insert("template".into(), toml::Value::Array(arr));
    }
    let text = toml::to_string(&table).map_err(|e| io::Error::other(e.to_string()))?;
    std::fs::create_dir_all(dir)?;
    config::write_atomic(dir, FILE_NAME, text.as_bytes())
}

/// 見た目を設定に重ねる(起動のとき、nerd_font と theme の auto を決める前に)。組を選んでいれば、
/// config.toml の `[style]` の部品ごとの形は重ねず、その組の形にする。
pub fn apply(c: &mut Config, l: &Look) {
    match l.theme {
        Some(ThemeChoice::Auto) => c.theme_auto = true,
        Some(ThemeChoice::Named(t)) => {
            c.theme = t;
            c.theme_auto = false;
        }
        None => {}
    }
    if let Some(p) = l.preset {
        c.style = Style::of(p);
    }
    match l.nerd {
        Some(Nerd::Auto) => c.nerd_font_auto = true,
        Some(Nerd::On) => {
            c.nerd_font = true;
            c.nerd_font_auto = false;
        }
        Some(Nerd::Off) => {
            c.nerd_font = false;
            c.nerd_font_auto = false;
        }
        None => {}
    }
}
