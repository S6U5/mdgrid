//! 表のプロファイル(CLI-3・SR-44)と、範囲ごとに重ねた決まった値。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! プロファイルは `[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]` と `use`。どの範囲(全体・ワークスペース・表・
//! ビュー)でも同じ形で書け、`Profile::read` が同じ読み方をする。`resolve` が既定から狭い範囲へ順に重ねる。
//! 画面に依存しない。

use crate::cells::{Cells, ColStyle};
use crate::colors::{self, Colors};
use crate::display::{Display, DisplayOverride, TabsMode};
use crate::i18n::Msg;
use crate::newnote::{self, NewNote};
use crate::schema;
use crate::style::{Band, Check, Frames, Links, Preset, Rules, Select, Status, Style, Tabs, Tags};
use crate::theme::Theme;
use crate::types::{DateFormat, WeekStart};
use std::collections::BTreeMap;

// ---- 警告 ----

/// 設定の警告(CLI-3): 「ファイルの名前: 項目の道筋: 理由」。
pub fn warn(out: &mut Vec<String>, file: &str, path: &str, reason: String) {
    out.push(Msg::SettingWarn.fill(&[&file, &path, &reason]));
}

/// 理由「〜でないので無視した」。
pub fn not(want: Msg) -> String {
    Msg::RsnNot.fill(&[&want.text()])
}

/// 理由「〜のどれかでないので無視した」。
pub fn not_one_of(names: &[&str]) -> String {
    let want = names
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ");
    Msg::RsnNot.fill(&[&Msg::WantOneOf.fill(&[&want])])
}

fn join(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

// ---- テーマの選び ----

/// テーマ(`look.theme`): 名前か、端末の地の明るさで選ぶ明暗の組(SR-39)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSpec {
    Named(Theme),
    Pair { light: Theme, dark: Theme },
}

impl Default for ThemeSpec {
    fn default() -> Self {
        ThemeSpec::Named(Theme::Default)
    }
}

impl ThemeSpec {
    /// `"auto"` と同じ組。
    pub const AUTO: ThemeSpec = ThemeSpec::Pair {
        light: Theme::Saas,
        dark: Theme::Sumi,
    };

    /// 設定の値を読む(`"auto"`・名前・`{ light, dark }` の表。表の片方を省けば AUTO のその側)。
    pub fn parse(v: &toml::Value) -> Option<ThemeSpec> {
        if let Some(s) = v.as_str() {
            return if s == "auto" {
                Some(ThemeSpec::AUTO)
            } else {
                Theme::parse(s).map(ThemeSpec::Named)
            };
        }
        let t = v.as_table()?;
        let mut light = Theme::Saas;
        let mut dark = Theme::Sumi;
        for (k, x) in t {
            let th = Theme::parse(x.as_str()?)?;
            match k.as_str() {
                "light" => light = th,
                "dark" => dark = th,
                _ => return None,
            }
        }
        Some(ThemeSpec::Pair { light, dark })
    }

    /// 設定の値にする(AUTO は `"auto"`)。
    pub fn to_value(self) -> toml::Value {
        match self {
            ThemeSpec::Named(t) => toml::Value::String(t.name().into()),
            s if s == ThemeSpec::AUTO => toml::Value::String("auto".into()),
            ThemeSpec::Pair { light, dark } => {
                let mut t = toml::Table::new();
                t.insert("light".into(), toml::Value::String(light.name().into()));
                t.insert("dark".into(), toml::Value::String(dark.name().into()));
                toml::Value::Table(t)
            }
        }
    }

    /// 画面に出す名前(`auto`・テーマの名前・`light/dark`)。
    pub fn label(self) -> String {
        match self {
            ThemeSpec::Named(t) => t.name().to_string(),
            s if s == ThemeSpec::AUTO => "auto".to_string(),
            ThemeSpec::Pair { light, dark } => format!("{}/{}", light.name(), dark.name()),
        }
    }

    /// 明暗の組か(端末に問い合わせる)。
    pub fn is_pair(self) -> bool {
        matches!(self, ThemeSpec::Pair { .. })
    }

    /// 使うテーマ。組なら地の明るさ(分からなければ暗い地)で選ぶ。
    pub fn pick(self, light_bg: Option<bool>) -> Theme {
        match self {
            ThemeSpec::Named(t) => t,
            ThemeSpec::Pair { light, dark } => Theme::auto(light_bg, light, dark),
        }
    }
}

// ---- 見た目の層 ----

/// `[look.style]` の部品ごとの形(書いた部品だけ Some)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StyleLayer {
    pub status: Option<Status>,
    pub tags: Option<Tags>,
    pub check: Option<Check>,
    pub select: Option<Select>,
    pub rules: Option<Rules>,
    pub tabs: Option<Tabs>,
    pub frames: Option<Frames>,
    pub band: Option<Band>,
    pub links: Option<Links>,
    pub icons: Option<bool>,
}

macro_rules! style_parts {
    ($m:ident) => {
        $m!(status, Status);
        $m!(tags, Tags);
        $m!(check, Check);
        $m!(select, Select);
        $m!(rules, Rules);
        $m!(tabs, Tabs);
        $m!(frames, Frames);
        $m!(band, Band);
        $m!(links, Links);
    };
}

impl StyleLayer {
    pub fn is_empty(&self) -> bool {
        *self == StyleLayer::default()
    }

    /// 書いた部品を形に当てる。
    pub fn apply(&self, s: &mut Style) {
        macro_rules! put {
            ($f:ident, $ty:ident) => {
                if let Some(v) = self.$f {
                    s.$f = v;
                }
            };
        }
        style_parts!(put);
        if let Some(b) = self.icons {
            s.icons = b;
        }
    }

    /// 書いた部品の名前(決まった値の出どころに使う)。
    pub fn written(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        macro_rules! name {
            ($f:ident, $ty:ident) => {
                if self.$f.is_some() {
                    out.push(concat!("look.style.", stringify!($f)));
                }
            };
        }
        style_parts!(name);
        if self.icons.is_some() {
            out.push("look.style.icons");
        }
        out
    }

    fn read(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantTable));
        };
        for (k, v) in t {
            let p = join(path, k);
            macro_rules! set {
                ($f:ident, $ty:ident) => {
                    if k == stringify!($f) {
                        match v.as_str().and_then($ty::parse) {
                            Some(x) => self.$f = Some(x),
                            None => warn(w, file, &p, not_one_of($ty::NAMES)),
                        }
                        continue;
                    }
                };
            }
            style_parts!(set);
            if k == "icons" {
                match v.as_bool() {
                    Some(b) => self.icons = Some(b),
                    None => warn(w, file, &p, not(Msg::WantBool)),
                }
                continue;
            }
            warn(w, file, &p, Msg::RsnUnknown.text().to_string());
        }
    }

    fn write(&self, t: &mut toml::Table) {
        macro_rules! out {
            ($f:ident, $ty:ident) => {
                if let Some(v) = self.$f {
                    t.insert(stringify!($f).into(), toml::Value::String(v.name().into()));
                }
            };
        }
        style_parts!(out);
        if let Some(b) = self.icons {
            t.insert("icons".into(), toml::Value::Boolean(b));
        }
    }
}

/// `[look]` の層(書いた項目だけ Some)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookLayer {
    pub theme: Option<ThemeSpec>,
    pub preset: Option<Preset>,
    /// `mode = "classic"`(SR-33)。
    pub classic: Option<bool>,
    /// `cells = "rich"`(SR-35)。
    pub rich: Option<bool>,
    pub style: StyleLayer,
    pub columns: BTreeMap<String, ColStyle>,
    /// `[look.colors]` の役割(Palette の欄の順)。
    pub roles: [Option<[u8; 3]>; 16],
    /// `[look.colors.values]`(値の文字は前後の空白を除いて小文字)。
    pub values: Vec<(String, [u8; 3])>,
}

impl LookLayer {
    pub fn is_empty(&self) -> bool {
        *self == LookLayer::default()
    }

    fn read(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantTable));
        };
        for (k, v) in t {
            let p = join(path, k);
            match k.as_str() {
                "theme" => match ThemeSpec::parse(v) {
                    Some(s) => self.theme = Some(s),
                    None => warn(w, file, &p, not(Msg::WantTheme)),
                },
                "preset" => match v.as_str().and_then(Preset::parse) {
                    Some(x) => self.preset = Some(x),
                    None => warn(w, file, &p, not_one_of(Preset::NAMES)),
                },
                "mode" => match v.as_str() {
                    Some("modern") => self.classic = Some(false),
                    Some("classic") => self.classic = Some(true),
                    _ => warn(w, file, &p, not(Msg::WantLook)),
                },
                "cells" => match v.as_str() {
                    Some("rich") => self.rich = Some(true),
                    Some("plain") => self.rich = Some(false),
                    _ => warn(w, file, &p, not(Msg::WantCells)),
                },
                "style" => self.style.read(v, file, &p, w),
                "columns" => self.read_columns(v, file, &p, w),
                "colors" => self.read_colors(v, file, &p, w),
                _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
            }
        }
    }

    fn read_columns(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantCellsColumns));
        };
        for (col, s) in t {
            match s.as_str().and_then(ColStyle::parse) {
                Some(st) => {
                    self.columns.insert(col.clone(), st);
                }
                None => warn(w, file, &join(path, col), not(Msg::WantColStyle)),
            }
        }
    }

    fn read_colors(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantTable));
        };
        for (k, v) in t {
            let p = join(path, k);
            if k == "values" {
                let Some(vals) = v.as_table() else {
                    warn(w, file, &p, not(Msg::WantTable));
                    continue;
                };
                for (item, c) in vals {
                    let pp = join(&p, item);
                    match c.as_str().and_then(colors::parse_color) {
                        Some(rgb) => {
                            let key = colors::value_key(item);
                            if self.values.iter().any(|(x, _)| *x == key) {
                                warn(w, file, &pp, Msg::RsnDuplicateValue.text().to_string());
                            }
                            self.values.retain(|(x, _)| *x != key);
                            self.values.push((key, rgb));
                        }
                        None => warn(w, file, &pp, not(Msg::WantColor)),
                    }
                }
                continue;
            }
            let Some(i) = colors::ROLES.iter().position(|r| r == k) else {
                warn(w, file, &p, Msg::RsnUnknown.text().to_string());
                continue;
            };
            match v.as_str().and_then(colors::parse_color) {
                Some(rgb) => self.roles[i] = Some(rgb),
                None => warn(w, file, &p, not(Msg::WantColor)),
            }
        }
    }

    fn write(&self, t: &mut toml::Table) {
        if let Some(s) = self.theme {
            t.insert("theme".into(), s.to_value());
        }
        if let Some(p) = self.preset {
            t.insert("preset".into(), toml::Value::String(p.name().into()));
        }
        if let Some(c) = self.classic {
            let m = if c { "classic" } else { "modern" };
            t.insert("mode".into(), toml::Value::String(m.into()));
        }
        if let Some(r) = self.rich {
            let m = if r { "rich" } else { "plain" };
            t.insert("cells".into(), toml::Value::String(m.into()));
        }
        if !self.style.is_empty() {
            let mut s = toml::Table::new();
            self.style.write(&mut s);
            t.insert("style".into(), toml::Value::Table(s));
        }
        if !self.columns.is_empty() {
            let c = self
                .columns
                .iter()
                .map(|(k, v)| (k.clone(), toml::Value::String(v.name().into())))
                .collect();
            t.insert("columns".into(), toml::Value::Table(c));
        }
        if self.roles.iter().any(Option::is_some) || !self.values.is_empty() {
            let mut c = toml::Table::new();
            for (name, rgb) in colors::ROLES.iter().zip(self.roles.iter()) {
                if let Some(rgb) = rgb {
                    c.insert((*name).into(), toml::Value::String(colors::hex(*rgb)));
                }
            }
            if !self.values.is_empty() {
                let v = self
                    .values
                    .iter()
                    .map(|(k, rgb)| (k.clone(), toml::Value::String(colors::hex(*rgb))))
                    .collect();
                c.insert("values".into(), toml::Value::Table(v));
            }
            t.insert("colors".into(), toml::Value::Table(c));
        }
    }
}

/// `[dates]` の層。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DatesLayer {
    pub format: Option<DateFormat>,
    pub week_start: Option<WeekStart>,
}

/// `[edit]` の層。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditLayer {
    pub candidates: Option<usize>,
    pub add_frontmatter: Option<bool>,
}

impl DatesLayer {
    pub fn is_empty(&self) -> bool {
        *self == DatesLayer::default()
    }

    fn read(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantTable));
        };
        for (k, v) in t {
            let p = join(path, k);
            match k.as_str() {
                "format" => match v.as_str() {
                    Some(s) => match DateFormat::parse(s) {
                        Ok(f) => self.format = Some(f),
                        Err(e) => warn(
                            w,
                            file,
                            &p,
                            Msg::RsnBadDateFormat.fill(&[&crate::config::squash_ws(&e)]),
                        ),
                    },
                    None => warn(w, file, &p, not(Msg::WantDateFormat)),
                },
                "week_start" => match v.as_str() {
                    Some("sun") => self.week_start = Some(WeekStart::Sun),
                    Some("mon") => self.week_start = Some(WeekStart::Mon),
                    _ => warn(w, file, &p, not(Msg::WantWeekStart)),
                },
                _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
            }
        }
    }

    fn to_table(&self) -> toml::Table {
        let mut d = toml::Table::new();
        if let Some(f) = &self.format {
            d.insert("format".into(), toml::Value::String(f.pattern().into()));
        }
        if let Some(ws) = self.week_start {
            let s = if ws == WeekStart::Mon { "mon" } else { "sun" };
            d.insert("week_start".into(), toml::Value::String(s.into()));
        }
        d
    }
}

impl EditLayer {
    pub fn is_empty(&self) -> bool {
        *self == EditLayer::default()
    }

    fn read(&mut self, v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) {
        let Some(t) = v.as_table() else {
            return warn(w, file, path, not(Msg::WantTable));
        };
        for (k, v) in t {
            let p = join(path, k);
            match k.as_str() {
                "candidates" => match v.as_integer().and_then(|n| usize::try_from(n).ok()) {
                    Some(n) => self.candidates = Some(n),
                    None => warn(w, file, &p, not(Msg::WantIntAtLeast0)),
                },
                "add_frontmatter" => match v.as_bool() {
                    Some(b) => self.add_frontmatter = Some(b),
                    None => warn(w, file, &p, not(Msg::WantBool)),
                },
                _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
            }
        }
    }

    fn to_table(&self) -> toml::Table {
        let mut e = toml::Table::new();
        if let Some(n) = self.candidates {
            e.insert("candidates".into(), toml::Value::Integer(n as i64));
        }
        if let Some(b) = self.add_frontmatter {
            e.insert("add_frontmatter".into(), toml::Value::Boolean(b));
        }
        e
    }
}

/// ビューの設定(views.toml・見た目の状態)に持つ層の読み書き。読めない所は捨てる(状態は消えても困らない。
/// views.toml の知らない項目の警告は views の検査が出す)。
macro_rules! layer_serde {
    ($ty:ident, $read:expr, $write:expr) => {
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let t: toml::Table = $write(self);
                serde::Serialize::serialize(&t, s)
            }
        }
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let t = <toml::Table as serde::Deserialize>::deserialize(d)?;
                let mut out = $ty::default();
                $read(&mut out, &toml::Value::Table(t));
                Ok(out)
            }
        }
    };
}

layer_serde!(
    LookLayer,
    |l: &mut LookLayer, v: &toml::Value| l.read(v, "", "", &mut Vec::new()),
    |l: &LookLayer| {
        let mut t = toml::Table::new();
        l.write(&mut t);
        t
    }
);
layer_serde!(
    DatesLayer,
    |l: &mut DatesLayer, v: &toml::Value| l.read(v, "", "", &mut Vec::new()),
    |l: &DatesLayer| l.to_table()
);
layer_serde!(
    EditLayer,
    |l: &mut EditLayer, v: &toml::Value| l.read(v, "", "", &mut Vec::new()),
    |l: &EditLayer| l.to_table()
);

// ---- プロファイル ----

/// 表のプロファイル(書いた項目だけを持つ部分の値)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Profile {
    /// `use`: 下に敷くテンプレートの名前。
    pub use_: Option<String>,
    pub look: LookLayer,
    pub display: DisplayOverride,
    pub dates: DatesLayer,
    pub edit: EditLayer,
    /// `[new_note]`(1つの項目。狭い範囲に書けば丸ごと代わる)。
    pub new_note: Option<NewNote>,
}

impl Profile {
    pub fn is_empty(&self) -> bool {
        *self == Profile::default()
    }

    /// プロファイルの最上位の項目 `key` を読む。プロファイルの項目でなければ false(呼ぶ側が判定する)。
    pub fn read_key(
        &mut self,
        key: &str,
        v: &toml::Value,
        file: &str,
        prefix: &str,
        w: &mut Vec<String>,
    ) -> bool {
        let p = join(prefix, key);
        match key {
            "use" => match v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                Some(s) => self.use_ = Some(s.to_string()),
                None => warn(w, file, &p, not(Msg::WantString)),
            },
            "look" => self.look.read(v, file, &p, w),
            "display" => read_display(&mut self.display, v, file, &p, w),
            "dates" => self.dates.read(v, file, &p, w),
            "edit" => self.edit.read(v, file, &p, w),
            "new_note" => self.new_note = Some(read_new_note(v, file, &p, w)),
            _ => return false,
        }
        true
    }

    /// 範囲のファイルの表を読む(ワークスペース・表・ビューの範囲)。アプリ全体の項目は「全体にだけ」、
    /// `skip` に挙げた名前(その場所の自分の項目)は読まずに飛ばし、ほかは知らない項目として警告する。
    pub fn read_scoped(
        t: &toml::Table,
        file: &str,
        prefix: &str,
        skip: &[&str],
        w: &mut Vec<String>,
    ) -> Profile {
        let mut p = Profile::default();
        for (k, v) in t {
            if skip.contains(&k.as_str()) || p.read_key(k, v, file, prefix, w) {
                continue;
            }
            let path = join(prefix, k);
            if schema::GLOBAL_KEYS.contains(&k.as_str())
                || schema::LEGACY.iter().any(|(old, _)| old == k)
            {
                warn(w, file, &path, Msg::RsnGlobalOnly.text().to_string());
            } else {
                warn(w, file, &path, Msg::RsnUnknown.text().to_string());
            }
        }
        p
    }

    /// 書いた項目を表に書く(画面の書くファイル。ui.toml・workspaces.toml・views.toml)。
    pub fn write(&self, t: &mut toml::Table) {
        if let Some(u) = &self.use_ {
            t.insert("use".into(), toml::Value::String(u.clone()));
        }
        if !self.look.is_empty() {
            let mut l = toml::Table::new();
            self.look.write(&mut l);
            t.insert("look".into(), toml::Value::Table(l));
        }
        if !self.display.is_empty() {
            if let Ok(toml::Value::Table(d)) = toml::Value::try_from(self.display) {
                t.insert("display".into(), toml::Value::Table(d));
            }
        }
        if !self.dates.is_empty() {
            t.insert("dates".into(), toml::Value::Table(self.dates.to_table()));
        }
        if !self.edit.is_empty() {
            t.insert("edit".into(), toml::Value::Table(self.edit.to_table()));
        }
        if let Some(n) = &self.new_note {
            if let Ok(v) = toml::Value::try_from(n) {
                t.insert("new_note".into(), v);
            }
        }
    }

    /// 書いた項目の表。
    pub fn to_table(&self) -> toml::Table {
        let mut t = toml::Table::new();
        self.write(&mut t);
        t
    }
}

/// `[display]` を読む(SR-21)。`tabs` の真偽は旧い書き方(CLI-20)として受ける。
fn read_display(
    d: &mut DisplayOverride,
    v: &toml::Value,
    file: &str,
    path: &str,
    w: &mut Vec<String>,
) {
    let Some(t) = v.as_table() else {
        return warn(w, file, path, not(Msg::WantDisplayTable));
    };
    for (k, v) in t {
        let p = join(path, k);
        if k == "tabs" {
            match (v.as_str().and_then(TabsMode::parse), v.as_bool()) {
                (Some(m), _) => d.tabs = Some(m),
                (None, Some(b)) => {
                    let m = if b { TabsMode::Always } else { TabsMode::Never };
                    d.tabs = Some(m);
                    warn(
                        w,
                        file,
                        &p,
                        Msg::RsnMoved.fill(&[&format!("{p} = \"{}\"", m.name())]),
                    );
                }
                _ => warn(w, file, &p, not(Msg::WantTabs)),
            }
            continue;
        }
        match d.bool_mut(k) {
            None => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
            Some(slot) => match v.as_bool() {
                Some(b) => *slot = Some(b),
                None => warn(w, file, &p, not(Msg::WantBool)),
            },
        }
    }
}

/// `[new_note]`(CE-27)を読む。型の違う値・知らない項目は警告にして飛ばす。
fn read_new_note(v: &toml::Value, file: &str, path: &str, w: &mut Vec<String>) -> NewNote {
    let mut n = NewNote::default();
    let Some(t) = v.as_table() else {
        warn(w, file, path, not(Msg::WantNewNoteTable));
        return n;
    };
    let cols = |v: &toml::Value| {
        v.as_array().and_then(|a| {
            a.iter()
                .map(|x| x.as_str().map(str::to_string))
                .collect::<Option<Vec<String>>>()
        })
    };
    for (k, v) in t {
        let p = join(path, k);
        match k.as_str() {
            "folder" | "name" | "body" => match v.as_str() {
                Some(s) => {
                    let slot = match k.as_str() {
                        "folder" => &mut n.folder,
                        "name" => &mut n.name,
                        _ => &mut n.body,
                    };
                    *slot = s.to_string();
                }
                None => warn(w, file, &p, not(Msg::WantString)),
            },
            "mode" => match v.as_str() {
                Some(s @ ("form" | "editor")) => n.mode = s.to_string(),
                Some(s) => warn(w, file, &p, Msg::RsnBadNoteMode.fill(&[&s])),
                None => warn(w, file, &p, not(Msg::WantString)),
            },
            "ask" | "required" | "hidden" => match cols(v) {
                Some(cs) => {
                    let out = match k.as_str() {
                        "ask" => &mut n.ask,
                        "required" => &mut n.required,
                        _ => &mut n.hidden,
                    };
                    for c in cs {
                        if newnote::not_a_key(&c) || out.contains(&c) {
                            warn(w, file, &p, Msg::RsnBadColumn.fill(&[&c]));
                        } else {
                            out.push(c);
                        }
                    }
                }
                None => warn(w, file, &p, not(Msg::WantColumnNames)),
            },
            "set" => {
                let Some(set) = v.as_table() else {
                    warn(w, file, &p, not(Msg::WantColumnValueTable));
                    continue;
                };
                for (col, x) in set {
                    let pp = join(&p, col);
                    if newnote::not_a_key(col) {
                        warn(w, file, &pp, Msg::RsnBadColumn.fill(&[col]));
                        continue;
                    }
                    match newnote::value_from_toml(x) {
                        Some(nv) => n.set.push((col.clone(), nv)),
                        None => warn(w, file, &pp, not(Msg::WantSetValue)),
                    }
                }
            }
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
    n
}

// ---- 範囲と重ね ----

/// 値の出どころの範囲(SR-44)。並びは広い → 狭い。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Place {
    Default,
    Config,
    Ui,
    Workspace,
    TableHand,
    TableApp,
    View,
}

/// 値の出どころ(範囲と、ファイル・名前の説明。CLI-21 のコメントと SR-43 の「(この表)」)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub place: Place,
    /// 説明(`config.toml`・`views.toml (notes/tasks)`・`workspace Product (.mdgrid/workspace.toml)` など)。
    pub label: String,
    /// テンプレートから来たならその名前。
    pub template: Option<String>,
}

impl Origin {
    pub fn new(place: Place, label: impl Into<String>) -> Origin {
        Origin {
            place,
            label: label.into(),
            template: None,
        }
    }

    /// コメントの文(`views.toml (notes/tasks), template night`)。
    pub fn describe(&self) -> String {
        match &self.template {
            Some(t) => format!("{} (template {t})", self.label),
            None => self.label.clone(),
        }
    }
}

/// 重ねる層の1つ。
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub origin: Origin,
    pub profile: Profile,
}

/// 名前を付けたテンプレートの並び(同じ名前はあとのもの。config.toml → ui.toml の順で足す)。
pub type Templates = Vec<(String, Profile)>;

/// テンプレートの名前で引く(あとに足したものが先)。
pub fn template<'a>(ts: &'a Templates, name: &str) -> Option<&'a Profile> {
    ts.iter().rev().find(|(n, _)| n == name).map(|(_, p)| p)
}

/// 重ねて決まった値(画面が使う型)と、項目ごとの出どころ。
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub theme: ThemeSpec,
    pub preset: Preset,
    pub style: Style,
    pub classic: bool,
    pub cells: Cells,
    pub colors: Colors,
    pub display: Display,
    pub date_format: DateFormat,
    pub week_start: WeekStart,
    pub candidates: usize,
    pub add_frontmatter: bool,
    pub new_note: NewNote,
    /// 項目の道筋 → 出どころ(既定の項目は無い)。
    pub origins: BTreeMap<String, Origin>,
}

impl Default for Resolved {
    fn default() -> Self {
        Resolved {
            theme: ThemeSpec::default(),
            preset: Preset::Sumi,
            style: Style::default(),
            classic: false,
            cells: Cells::default(),
            colors: Colors::default(),
            display: Display::default(),
            date_format: DateFormat::iso(),
            week_start: WeekStart::Sun,
            candidates: 20,
            add_frontmatter: true,
            new_note: NewNote::default(),
            origins: BTreeMap::new(),
        }
    }
}

impl Resolved {
    /// 道筋の出どころ(無ければ既定)。
    pub fn origin(&self, path: &str) -> Place {
        self.origins.get(path).map_or(Place::Default, |o| o.place)
    }

    fn mark(&mut self, path: &str, o: &Origin) {
        self.origins.insert(path.to_string(), o.clone());
    }

    /// 1つの層の値を重ねる。
    fn apply(&mut self, p: &Profile, o: &Origin) {
        let l = &p.look;
        if let Some(pr) = l.preset {
            // 組を書いた層より広い層の部品の形は使わない(SR-44)。
            self.preset = pr;
            self.style = Style::of(pr);
            self.origins.retain(|k, _| !k.starts_with("look.style."));
            self.mark("look.preset", o);
        }
        l.style.apply(&mut self.style);
        for path in l.style.written() {
            self.mark(path, o);
        }
        if let Some(t) = l.theme {
            // テーマを書いた層より広い層の役割の色は使わない(値の色は使う)。
            self.theme = t;
            self.colors.roles = [None; 16];
            self.origins.retain(|k, _| {
                !k.starts_with("look.colors.") || k.starts_with("look.colors.values")
            });
            self.mark("look.theme", o);
        }
        for (i, c) in l.roles.iter().enumerate() {
            if let Some(c) = c {
                self.colors.roles[i] = Some(*c);
                self.mark(&format!("look.colors.{}", colors::ROLES[i]), o);
            }
        }
        for (k, c) in &l.values {
            self.colors.values.retain(|(x, _)| x != k);
            self.colors.values.push((k.clone(), *c));
            self.mark(&format!("look.colors.values.{k}"), o);
        }
        if let Some(c) = l.classic {
            self.classic = c;
            self.mark("look.mode", o);
        }
        if let Some(r) = l.rich {
            self.cells.rich = r;
            self.mark("look.cells", o);
        }
        for (col, st) in &l.columns {
            self.cells.columns.insert(col.clone(), *st);
            self.mark(&format!("look.columns.{col}"), o);
        }
        for name in p.display.apply_to(&mut self.display) {
            self.mark(&format!("display.{name}"), o);
        }
        if let Some(f) = &p.dates.format {
            self.date_format = f.clone();
            self.mark("dates.format", o);
        }
        if let Some(ws) = p.dates.week_start {
            self.week_start = ws;
            self.mark("dates.week_start", o);
        }
        if let Some(n) = p.edit.candidates {
            self.candidates = n;
            self.mark("edit.candidates", o);
        }
        if let Some(b) = p.edit.add_frontmatter {
            self.add_frontmatter = b;
            self.mark("edit.add_frontmatter", o);
        }
        if let Some(n) = &p.new_note {
            self.new_note = n.clone();
            self.mark("new_note", o);
        }
    }

    /// 決まった値をプロファイルの形にする(CLI-21 の出力。既定の値も全部書く)。
    pub fn to_profile(&self) -> Profile {
        let mut p = Profile::default();
        p.look.theme = Some(self.theme);
        p.look.preset = Some(self.preset);
        p.look.classic = Some(self.classic);
        p.look.rich = Some(self.cells.rich);
        let base = Style::of(self.preset);
        let s = &self.style;
        macro_rules! diff {
            ($f:ident, $ty:ident) => {
                if s.$f != base.$f {
                    p.look.style.$f = Some(s.$f);
                }
            };
        }
        style_parts!(diff);
        if s.icons != base.icons {
            p.look.style.icons = Some(s.icons);
        }
        p.look.columns = self.cells.columns.clone();
        p.look.roles = self.colors.roles;
        p.look.values = self.colors.values.clone();
        p.display = DisplayOverride::all(&self.display);
        p.dates.format = Some(self.date_format.clone());
        p.dates.week_start = Some(self.week_start);
        p.edit.candidates = Some(self.candidates);
        p.edit.add_frontmatter = Some(self.add_frontmatter);
        if self.new_note != NewNote::default() {
            p.new_note = Some(self.new_note.clone());
        }
        p
    }
}

/// 層を既定から狭い範囲へ順に重ねる(SR-44)。各層は先に `use` のテンプレートを敷く。
/// 無いテンプレートと、テンプレートの中の `use` は警告にする。
pub fn resolve(layers: &[Layer], templates: &Templates, w: &mut Vec<String>) -> Resolved {
    let mut r = Resolved::default();
    for layer in layers {
        if let Some(name) = &layer.profile.use_ {
            match template(templates, name) {
                Some(t) => {
                    if t.use_.is_some() {
                        warn(
                            w,
                            &layer.origin.label,
                            &format!("templates.{name}.use"),
                            Msg::RsnNestedUse.text().to_string(),
                        );
                    }
                    let mut o = layer.origin.clone();
                    o.template = Some(name.clone());
                    r.apply(t, &o);
                }
                None => warn(
                    w,
                    &layer.origin.label,
                    "use",
                    Msg::RsnNoTemplate.fill(&[name]),
                ),
            }
        }
        r.apply(&layer.profile, &layer.origin);
    }
    r
}

/// `キー = 値` の1行(表の値は1行の表 `{ light = …, dark = … }` で書く)。
pub fn key_line(k: &str, v: &toml::Value) -> String {
    let mut one = toml::Table::new();
    one.insert(k.to_string(), toml::Value::Boolean(true));
    let key = toml::to_string(&one).unwrap_or_default();
    let key = key.trim_end().trim_end_matches("true").trim_end();
    format!("{key} {v}\n")
}

/// 決まった値を `--print-config --resolved` の TOML にする(CLI-21): 区画ごとに、各項目の上に出どころのコメント。
pub fn resolved_toml(r: &Resolved) -> String {
    let t = r.to_profile().to_table();
    let mut out = String::new();
    let origin = |path: &str| -> String {
        r.origins
            .iter()
            .filter(|(k, _)| *k == path || k.starts_with(&format!("{path}.")))
            .map(|(_, o)| o.describe())
            .next()
            .unwrap_or_else(|| "default".to_string())
    };
    fn scalar_lines(
        out: &mut String,
        t: &toml::Table,
        path: &str,
        origin: &dyn Fn(&str) -> String,
    ) -> Vec<(String, toml::Table)> {
        let mut subs = Vec::new();
        for (k, v) in t {
            let p = join(path, k);
            match v {
                toml::Value::Table(s) if !(path == "look" && k == "theme") => {
                    subs.push((p, s.clone()))
                }
                _ => {
                    out.push_str(&format!("# {}\n", origin(&p)));
                    out.push_str(&key_line(k, v));
                }
            }
        }
        subs
    }
    let mut queue: Vec<(String, toml::Table)> = Vec::new();
    for (k, v) in &t {
        if let toml::Value::Table(s) = v {
            queue.push((k.clone(), s.clone()));
        }
    }
    while let Some((path, table)) = (!queue.is_empty()).then(|| queue.remove(0)) {
        if path == "new_note" {
            out.push_str(&format!("\n# {}\n", origin("new_note")));
            let mut one = toml::Table::new();
            one.insert("new_note".into(), toml::Value::Table(table));
            out.push_str(&toml::to_string(&one).unwrap_or_default());
            continue;
        }
        out.push_str(&format!("\n[{path}]\n"));
        let subs = scalar_lines(&mut out, &table, &path, &origin);
        for s in subs.into_iter().rev() {
            queue.insert(0, s);
        }
    }
    out
}
