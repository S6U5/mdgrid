//! 設定(CLI-3・SR-13)と見た目の状態(SR-11・SR-12)。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! 設定はユーザーが書く TOML(`$XDG_CONFIG_HOME/mdgrid/config.toml`)。項目はアプリ全体の項目(`language`・
//! `editor`・`poll_ms`・`[terminal]`・`[workspace]`・`[keys]`)と、表のプロファイル(`profile::Profile`)と、
//! テンプレート(`[templates.<名前>]`)。項目の表は `schema`、旧い名前の写しは `legacy`。
//! 見た目の状態は消えても困らないもので、ノートのフォルダには書かず、呼ぶ側が渡す状態のフォルダ
//! (`$XDG_STATE_HOME/mdgrid/` など)に、対象のパスとビューの名前ごとに1つの TOML として置く。

pub use crate::display::Display;
pub use crate::i18n::Language;
use crate::i18n::Msg;
use crate::profile::{self, not, warn, Layer, Origin, Place, Profile, Resolved, Templates};
pub use crate::schema::{Item, ITEMS, KEYS};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 丸い札の端(`terminal.nerd_font`。SR-36)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NerdFont {
    #[default]
    Auto,
    On,
    Off,
}

impl NerdFont {
    pub const ALL: [NerdFont; 3] = [NerdFont::Auto, NerdFont::On, NerdFont::Off];

    pub fn name(self) -> &'static str {
        match self {
            NerdFont::Auto => "auto",
            NerdFont::On => "true",
            NerdFont::Off => "false",
        }
    }

    pub fn parse(v: &toml::Value) -> Option<NerdFont> {
        match (v.as_bool(), v.as_str()) {
            (Some(true), _) => Some(NerdFont::On),
            (Some(false), _) => Some(NerdFont::Off),
            (_, Some("auto")) => Some(NerdFont::Auto),
            _ => None,
        }
    }

    pub fn to_value(self) -> toml::Value {
        match self {
            NerdFont::Auto => toml::Value::String("auto".into()),
            NerdFont::On => toml::Value::Boolean(true),
            NerdFont::Off => toml::Value::Boolean(false),
        }
    }

    /// 描いてよいか。`auto` は端末の名前で決める(`style::nerd_auto`)。
    pub fn resolve(self, term_program: Option<&str>) -> bool {
        match self {
            NerdFont::Auto => crate::style::nerd_auto(term_program),
            NerdFont::On => true,
            NerdFont::Off => false,
        }
    }
}

/// 端末の性質(`[terminal]`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terminal {
    /// false で色なし(NO_COLOR と同じ)。
    pub color: bool,
    /// CV-6: East Asian Ambiguous を幅2にする。
    pub ambiguous_wide: bool,
    /// SR-36: 丸い札の端。
    pub nerd_font: NerdFont,
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            color: true,
            ambiguous_wide: false,
            nerd_font: NerdFont::Auto,
        }
    }
}

/// 設定(CLI-3)。`parse("")` が既定。
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// SR-23: 画面と起動の文言の言語(既定 `Auto` は環境変数に従う)。
    pub language: Language,
    /// SR-8: ノートを開くエディタ(引数つきでよい)。None(既定。空の文字列も同じ)なら $VISUAL・$EDITOR・vi。
    pub editor: Option<String>,
    /// BV-9 の読み直しの間隔(ミリ秒)。
    pub poll_ms: u64,
    /// (モード, キーの表記 "j"・"ctrl+s"・"shift+tab", 動作の名前か "none")。SR-13。
    pub keys: Vec<(String, String, String)>,
    pub terminal: Terminal,
    /// WS-5: ワークスペースの検知(`workspace.detect`。既定は保管庫だけ)。
    pub workspace_detect: Vec<crate::workspace::Detect>,
    /// 全体の表のプロファイル(config.toml に書いたもの)。
    pub profile: Profile,
    /// `[templates.<名前>]`(書いた順)。
    pub templates: Templates,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: Language::Auto,
            editor: None,
            poll_ms: 1000,
            keys: Vec::new(),
            terminal: Terminal::default(),
            workspace_detect: vec![crate::workspace::Detect::Vault],
            profile: Profile::default(),
            templates: Vec::new(),
        }
    }
}

impl Config {
    /// config.toml の層(SR-44)。
    pub fn layer(&self) -> Layer {
        Layer {
            origin: Origin::new(Place::Config, CONFIG_FILE),
            profile: self.profile.clone(),
        }
    }

    /// config.toml だけを重ねた決まった値(範囲の無いときと試験)。
    pub fn resolved(&self) -> Resolved {
        profile::resolve(&[self.layer()], &self.templates, &mut Vec::new())
    }
}

/// 設定のファイルの名前(警告と出どころの説明)。
pub const CONFIG_FILE: &str = "config.toml";

/// TOML を読む(config.toml)。知らない項目・型の違う項目は警告の文にして返し、止めない(CLI-3)。
/// 壊れた TOML は Err(理由1行)。
pub fn parse(text: &str) -> Result<(Config, Vec<String>), String> {
    parse_named(text, CONFIG_FILE)
}

/// `file` の名前で警告を出して読む(`--config` で渡したファイルなど)。
pub fn parse_named(text: &str, file: &str) -> Result<(Config, Vec<String>), String> {
    let table = parse_table(text)?;
    let mut w = Vec::new();
    let table = crate::legacy::lift(table, file, &mut w);
    let mut c = Config::default();
    for (name, value) in &table {
        if c.profile.read_key(name, value, file, "", &mut w) {
            continue;
        }
        match name.as_str() {
            "language" => match value.as_str().and_then(Language::parse) {
                Some(l) => c.language = l,
                None => warn(&mut w, file, name, not(Msg::WantLanguage)),
            },
            "editor" => match value.as_str() {
                // 空(空白だけも)は無いのと同じ(SR-8)。
                Some(e) => c.editor = (!e.trim().is_empty()).then(|| e.to_string()),
                None => warn(&mut w, file, name, not(Msg::WantString)),
            },
            "poll_ms" => match value.as_integer().and_then(|n| u64::try_from(n).ok()) {
                Some(n) if n > 0 => c.poll_ms = n,
                _ => warn(&mut w, file, name, not(Msg::WantIntAtLeast1)),
            },
            "terminal" => read_terminal(value, &mut c.terminal, file, &mut w),
            "workspace" => read_workspace(value, &mut c, file, &mut w),
            "keys" => read_keys(value, &mut c.keys, file, &mut w),
            "templates" => c.templates = read_templates(value, file, &mut w),
            _ => warn(&mut w, file, name, Msg::RsnUnknown.text().to_string()),
        }
    }
    Ok((c, w))
}

/// 壊れた TOML の理由1行。
fn parse_table(text: &str) -> Result<toml::Table, String> {
    text.parse()
        .map_err(|e: toml::de::Error| match toml_error(text, &e) {
            (Some(line), msg) => Msg::ConfigTomlLine.fill(&[&line, &msg]),
            (None, msg) => Msg::ConfigToml.fill(&[&msg]),
        })
}

fn read_terminal(v: &toml::Value, t: &mut Terminal, file: &str, w: &mut Vec<String>) {
    let Some(tab) = v.as_table() else {
        return warn(w, file, "terminal", not(Msg::WantTable));
    };
    for (k, v) in tab {
        let p = format!("terminal.{k}");
        match k.as_str() {
            "color" | "ambiguous_wide" => match v.as_bool() {
                Some(b) if k == "color" => t.color = b,
                Some(b) => t.ambiguous_wide = b,
                None => warn(w, file, &p, not(Msg::WantBool)),
            },
            "nerd_font" => match NerdFont::parse(v) {
                Some(n) => t.nerd_font = n,
                None => warn(w, file, &p, not(Msg::WantNerdFont)),
            },
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
}

fn read_workspace(v: &toml::Value, c: &mut Config, file: &str, w: &mut Vec<String>) {
    let Some(tab) = v.as_table() else {
        return warn(w, file, "workspace", not(Msg::WantTable));
    };
    for (k, v) in tab {
        let p = format!("workspace.{k}");
        match k.as_str() {
            "detect" => {
                let parsed: Option<Vec<_>> = v.as_array().and_then(|items| {
                    items
                        .iter()
                        .map(|v| v.as_str().and_then(crate::workspace::Detect::parse))
                        .collect()
                });
                match parsed {
                    Some(d) => c.workspace_detect = d,
                    None => warn(w, file, &p, not(Msg::WantDetect)),
                }
            }
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
}

fn read_keys(
    value: &toml::Value,
    keys: &mut Vec<(String, String, String)>,
    file: &str,
    w: &mut Vec<String>,
) {
    let Some(modes) = value.as_table() else {
        return warn(w, file, "keys", not(Msg::WantKeysTable));
    };
    for (mode, binds) in modes {
        let Some(binds) = binds.as_table() else {
            warn(
                w,
                file,
                &format!("keys.{mode}"),
                not(Msg::WantKeyActionTable),
            );
            continue;
        };
        for (key, action) in binds {
            match action.as_str() {
                Some(a) => keys.push((mode.clone(), key.clone(), a.to_string())),
                None => warn(
                    w,
                    file,
                    &format!("keys.{mode}.{key}"),
                    not(Msg::WantActionName),
                ),
            }
        }
    }
}

/// `[templates.<名前>]` を読む(どれもプロファイルの断片。アプリ全体の項目は書けない)。
pub(crate) fn read_templates(v: &toml::Value, file: &str, w: &mut Vec<String>) -> Templates {
    let Some(tab) = v.as_table() else {
        warn(w, file, "templates", not(Msg::WantTable));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, t) in tab {
        let path = format!("templates.{name}");
        let Some(t) = t.as_table() else {
            warn(w, file, &path, not(Msg::WantTable));
            continue;
        };
        out.push((name.clone(), Profile::read_scoped(t, file, &path, &[], w)));
    }
    out
}

/// SR-8: ノートを開くエディタ。設定の `editor`・`$VISUAL`・`$EDITOR` の順で空でない最初のもの
/// (空白だけの値も空とみなす)、どれも無ければ `vi`。値は単語に分けずそのまま返す(分けるのは起動の側の `editor_argv`)。
/// 環境変数は呼ぶ側(main)が読んで渡す。
pub fn resolve_editor(config: Option<&str>, visual: Option<&str>, editor: Option<&str>) -> String {
    [config, visual, editor]
        .into_iter()
        .flatten()
        .find(|v| !v.trim().is_empty())
        .unwrap_or("vi")
        .to_string()
}

/// SR-23: 設定の TOML から `language` だけを読む(起動の理由を出す前に言語を決めるため)。
/// 壊れた TOML・無い・読めない値は `Auto`(誤りと警告は `parse` が出す)。
pub fn peek_language(text: &str) -> Language {
    text.parse::<toml::Table>()
        .ok()
        .and_then(|t| t.get("language")?.as_str().and_then(Language::parse))
        .unwrap_or_default()
}

/// `--print-config` の出力(CLI-11): 全項目を区画ごとに、既定値と英語の説明と書ける範囲のコメント付きで並べた TOML。
/// 既定で書かない項目(`editor`・`use`・表の項目)は書き方の例をコメントで出す。読み直すと警告なしで既定と同じ。
pub fn default_toml() -> String {
    let mut out = String::from(
        "# mdgrid configuration with every item at its default value.\n\
         # Location: $XDG_CONFIG_HOME/mdgrid/config.toml (or ~/.config/mdgrid/config.toml).\n\
         # Items marked \"global only\" belong to the whole app; the others form the table profile and can\n\
         # also be written per workspace, table and view.\n\
         # Options: mdgrid --help. Full reference: docs/config.md in the mdgrid repository.\n",
    );
    let mut section = "";
    for item in ITEMS {
        let sec = item.section();
        if !item.is_table() && sec != section {
            out.push_str(&format!("\n[{sec}]\n"));
            section = sec;
        }
        out.push('\n');
        for line in item.en.lines() {
            out.push_str("# ");
            out.push_str(line.trim());
            out.push('\n');
        }
        out.push_str(&format!("# Scope: {}.\n", item.scope.en()));
        match item.default {
            Some(v) => out.push_str(&format!("{} = {}\n", item.leaf(), v)),
            // 区画の下の項目の例は、区画の見出しの行を除いて出す(今の区画の中)。
            None if !item.is_table() && !sec.is_empty() => {
                out.push_str("# Example:\n");
                for line in item.example.lines().skip(1) {
                    out.push_str("# ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            None => {
                out.push_str("# Example:\n");
                for line in item.example.lines() {
                    out.push_str("# ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
    }
    out
}

/// `--print-config --resolved` のアプリ全体の項目(CLI-21): 各項目の上に出どころ(`config.toml` か `default`)。
pub fn app_toml(c: &Config) -> String {
    let d = Config::default();
    let src = |same: bool| {
        if same {
            "# default\n"
        } else {
            "# config.toml\n"
        }
    };
    let mut out = String::new();
    let line = |k: &str, v: toml::Value| {
        let mut t = toml::Table::new();
        t.insert(k.into(), v);
        toml::to_string(&t).unwrap_or_default()
    };
    out.push_str(src(c.language == d.language));
    out.push_str(&line(
        "language",
        toml::Value::String(c.language.name().into()),
    ));
    if let Some(e) = &c.editor {
        out.push_str(src(false));
        out.push_str(&line("editor", toml::Value::String(e.clone())));
    }
    out.push_str(src(c.poll_ms == d.poll_ms));
    out.push_str(&line("poll_ms", toml::Value::Integer(c.poll_ms as i64)));
    out.push_str("\n[terminal]\n");
    out.push_str(src(c.terminal.color == d.terminal.color));
    out.push_str(&line("color", toml::Value::Boolean(c.terminal.color)));
    out.push_str(src(c.terminal.ambiguous_wide == d.terminal.ambiguous_wide));
    out.push_str(&line(
        "ambiguous_wide",
        toml::Value::Boolean(c.terminal.ambiguous_wide),
    ));
    out.push_str(src(c.terminal.nerd_font == d.terminal.nerd_font));
    out.push_str(&line("nerd_font", c.terminal.nerd_font.to_value()));
    out.push_str("\n[workspace]\n");
    out.push_str(src(c.workspace_detect == d.workspace_detect));
    let detect = c
        .workspace_detect
        .iter()
        .map(|m| toml::Value::String(m.name().into()))
        .collect();
    out.push_str(&line("detect", toml::Value::Array(detect)));
    if !c.keys.is_empty() {
        let mut modes = toml::Table::new();
        for (mode, key, action) in &c.keys {
            let m = modes
                .entry(mode.clone())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            if let toml::Value::Table(m) = m {
                m.insert(key.clone(), toml::Value::String(action.clone()));
            }
        }
        let mut t = toml::Table::new();
        t.insert("keys".into(), toml::Value::Table(modes));
        out.push_str("\n# config.toml\n");
        out.push_str(&toml::to_string(&t).unwrap_or_default());
    }
    out
}

/// `--migrate-config` の出力(CLI-20): 旧い書き方を新しい形に写した TOML。注釈は移らない。
pub fn migrate(text: &str) -> Result<String, String> {
    let table = parse_table(text)?;
    let mut lifted = crate::legacy::lift(table, CONFIG_FILE, &mut Vec::new());
    // 明暗の組のテーマは `[look]` の下の1行の表で書く(`[look.theme]` の区画にしない)。
    let theme = lifted
        .get_mut("look")
        .and_then(|l| l.as_table_mut())
        .and_then(|l| match l.get("theme") {
            Some(toml::Value::Table(_)) => l.remove("theme"),
            _ => None,
        });
    let mut body = toml::to_string(&lifted).map_err(|e| squash_ws(&e.to_string()))?;
    if let Some(th) = theme {
        let line = crate::profile::key_line("theme", &th);
        body = match body.find("[look]\n") {
            Some(i) => format!("{}{line}{}", &body[..i + 7], &body[i + 7..]),
            None => format!("{body}\n[look]\n{line}"),
        };
    }
    Ok(format!(
        "# mdgrid configuration moved to the current layout by mdgrid --migrate-config.\n\
         # Comments of the original file are not carried over.\n\n{body}"
    ))
}

/// TOML の見出しの行なら、括弧の中の空白と行末のコメントを除いた形(`[[workspace]]`・`[meta]`)。
/// 区画を文字のまま書き換えるときに使う(places.toml・workspaces.toml)。
pub(crate) fn toml_header(line: &str) -> Option<String> {
    let t = line.trim();
    let (open, close) = if t.starts_with("[[") {
        ("[[", "]]")
    } else if t.starts_with('[') {
        ("[", "]")
    } else {
        return None;
    };
    let end = t.find(close)?;
    let inner: String = t[open.len()..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let rest = t[end + close.len()..].trim();
    (rest.is_empty() || rest.starts_with('#')).then(|| format!("{open}{inner}{close}"))
}

/// `lines[from..to]` の中の、見出し `head` で始まる区画 (始まり, 中身の終わり)。区画は次の見出し
/// (`inner` が真のものを除く)か `to` の前まで。末尾のコメントと空行は中身に含めない(次の区画の前置き)。
pub(crate) fn toml_blocks(
    lines: &[&str],
    from: usize,
    to: usize,
    head: &str,
    inner: &dyn Fn(&str) -> bool,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = from;
    while i < to {
        if toml_header(lines[i]).as_deref() != Some(head) {
            i += 1;
            continue;
        }
        let start = i;
        let mut end = start + 1;
        while end < to {
            match toml_header(lines[end]) {
                Some(h) if !inner(&h) => break,
                _ => end += 1,
            }
        }
        let next = end;
        while end > start + 1 {
            let t = lines[end - 1].trim();
            if t.is_empty() || t.starts_with('#') {
                end -= 1;
            } else {
                break;
            }
        }
        out.push((start, end));
        i = next;
    }
    out
}

pub(crate) fn squash_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// $XDG_CONFIG_HOME/mdgrid/config.toml、無ければ ~/.config/mdgrid/config.toml(CLI-3)。
/// どちらの環境変数も無い(空の)ときは None。
pub fn config_path() -> Option<PathBuf> {
    base_dir("XDG_CONFIG_HOME", ".config").map(|d| d.join("mdgrid").join("config.toml"))
}

/// 見た目の状態の既定の置き場: $XDG_STATE_HOME/mdgrid/、無ければ ~/.local/state/mdgrid/。
pub fn state_dir() -> Option<PathBuf> {
    base_dir("XDG_STATE_HOME", ".local/state").map(|d| d.join("mdgrid"))
}

fn base_dir(xdg: &str, home_rel: &str) -> Option<PathBuf> {
    // XDG の仕様どおり、絶対パスでない値は無視する。
    if let Some(v) = std::env::var_os(xdg) {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return Some(p);
        }
    }
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join(home_rel))
}

/// 見た目の状態(SR-11・SR-12)。Default は空(既定)。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewState {
    pub order: Vec<String>,
    pub hidden: Vec<String>,
    pub widths: Vec<(String, u16)>,
    pub folded: Vec<String>,
    pub view: Option<String>,
    /// ビューの設定(NV-17)。無い古い状態のファイルは既定。状態のファイルでは最上位の `settings` に書く。
    #[serde(default, skip_serializing_if = "crate::settings::Settings::is_default")]
    pub settings: crate::settings::Settings,
}

/// 状態のファイルの中身。取り違え(ハッシュの衝突)を避けるため、対象とビューの名前も持つ。
#[derive(Serialize, Deserialize)]
struct StateFile {
    target: String,
    view_name: String,
    state: ViewState,
    /// ビューの設定(NV-17)。`state` の中ではなく最上位に置く。無い古い形は既定。
    #[serde(default, skip_serializing_if = "crate::settings::Settings::is_default")]
    settings: crate::settings::Settings,
}

/// 対象の実体のパス(解決できなければ渡されたまま)の文字列。
pub(crate) fn target_key(target: &Path) -> String {
    let real = std::fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    real.to_string_lossy().into_owned()
}

/// 対象とビューの名前から作る、固定の長さのファイル名(SHA-256 の16進 + `.toml`)。
fn state_file_name(target: &str, view: &str) -> String {
    let mut bytes = Vec::with_capacity(target.len() + view.len() + 1);
    bytes.extend_from_slice(target.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(view.as_bytes());
    let hash = crate::source::content_hash(&bytes);
    let mut name = String::with_capacity(64 + 5);
    for b in hash {
        name.push_str(&format!("{:02x}", b));
    }
    name.push_str(".toml");
    name
}

/// 状態を読む。無い・読めない・壊れている・別の対象のもの → 既定(空)。
pub fn load_state(dir: &Path, target: &Path, view: &str) -> ViewState {
    let key = target_key(target);
    let path = dir.join(state_file_name(&key, view));
    let Ok(bytes) = std::fs::read(&path) else {
        return ViewState::default();
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return ViewState::default();
    };
    // 設定(NV-17)は別に読む。設定の表が壊れていても、ほかの見た目の状態は捨てない。
    let Ok(mut table) = toml::from_str::<toml::Table>(text) else {
        return ViewState::default();
    };
    let top = table.remove("settings");
    let inner = table
        .get_mut("state")
        .and_then(|s| s.as_table_mut())
        .and_then(|s| s.remove("settings"));
    match StateFile::deserialize(toml::Value::Table(table)) {
        Ok(f) if f.target == key && f.view_name == view => {
            let mut s = f.state;
            s.settings = top.or(inner).map(read_settings).unwrap_or_default();
            s
        }
        _ => ViewState::default(),
    }
}

/// 設定の表を読む。読めなければ既定(SR-11: 状態は消えても困らない)。
fn read_settings(v: toml::Value) -> crate::settings::Settings {
    crate::settings::Settings::deserialize(v).unwrap_or_default()
}

/// 状態を書く。dir の中に一時ファイルを書いてから名前を変えて置き換える。
pub fn save_state(dir: &Path, target: &Path, view: &str, s: &ViewState) -> io::Result<()> {
    let key = target_key(target);
    let name = state_file_name(&key, view);
    let mut state = s.clone();
    let settings = std::mem::take(&mut state.settings);
    let file = StateFile {
        target: key,
        view_name: view.to_string(),
        state,
        settings,
    };
    let text = toml::to_string(&file).map_err(|e| io::Error::other(squash_ws(&e.to_string())))?;
    write_atomic(dir, &name, text.as_bytes())
}

/// 設定のフォルダの `name` を置き換える: 同じフォルダの一時ファイルに書いて fsync し、名前を変えて置き換える
/// (途中で止まっても前のファイルが残る)。前に残った一時ファイルは先に消す。失敗したら一時ファイルを消す。
pub(crate) fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    remove_stale_tmps(dir, name);
    let tmp = dir.join(format!(".{}.tmp.{}", name, std::process::id()));
    let result = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, dir.join(name))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// 壊れた TOML の(行の番号、1行にした理由)。行が分からなければ None。
pub(crate) fn toml_error(text: &str, e: &toml::de::Error) -> (Option<usize>, String) {
    let msg = squash_ws(e.message());
    let line = e
        .span()
        .map(|span| text[..span.start.min(text.len())].matches('\n').count() + 1);
    (line, msg)
}

/// 状態を消す(無ければ何もしない)。mdgrid のビューの名前の変更・削除で使う(BV-20)。
pub fn remove_state(dir: &Path, target: &Path, view: &str) -> io::Result<()> {
    let name = state_file_name(&target_key(target), view);
    match std::fs::remove_file(dir.join(name)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        r => r,
    }
}

/// 前の save_state が create と rename の間で止まって残した、同じ name の一時ファイル
/// (`.{name}.tmp.*`)を消す。pid が違うので次の起動では名前で探す。消せなくても止めない。
pub(crate) fn remove_stale_tmps(dir: &Path, name: &str) {
    let prefix = format!(".{}.tmp.", name);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        if e.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// SR-12: 状態の並びと隠す列は今ある列にだけ当て、状態に無い列は今の列の順で右に足す。
/// (並び, 隠す列)を返す。
pub fn apply_state(columns: &[String], s: &ViewState) -> (Vec<String>, Vec<String>) {
    let mut order: Vec<String> = Vec::with_capacity(columns.len());
    for c in &s.order {
        if columns.contains(c) && !order.contains(c) {
            order.push(c.clone());
        }
    }
    for c in columns {
        if !order.contains(c) {
            order.push(c.clone());
        }
    }
    let mut hidden: Vec<String> = Vec::new();
    for c in &s.hidden {
        if columns.contains(c) && !hidden.contains(c) {
            hidden.push(c.clone());
        }
    }
    (order, hidden)
}

#[cfg(test)]
#[path = "test_config_unit.rs"]
mod tests;
