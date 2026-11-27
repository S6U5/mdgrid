//! 設定(CLI-3・SR-13)と見た目の状態(SR-11・SR-12)。形は docs/design.md の「設定の形と範囲ごとの上書き」。
//!
//! 設定はユーザーが書く TOML(`$XDG_CONFIG_HOME/mdgrid/config.toml`。Windows は `%APPDATA%\mdgrid\config.toml`)。項目はアプリ全体の項目(`language`・
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
    /// SR-45: アイコン用のフォント(Nerd Font)の記号を使う(既定 false)。
    pub nerd_icons: bool,
    /// resume.herdr-config-off: Herdr の窓で、開き直す命令を伝える(既定 true。tbd~resume.herdr-config-name)。
    pub herdr: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Terminal {
            color: true,
            ambiguous_wide: false,
            nerd_font: NerdFont::Auto,
            nerd_icons: false,
            herdr: true,
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
    /// screen.vim-setting: vim モード(`[keys]` の `vim`。tbd~screen.vim-setting-name。既定は偽)。
    pub vim: bool,
    pub terminal: Terminal,
    /// WS-5: ワークスペースの検知(`workspace.detect`。既定は `.obsidian` だけ)。
    pub workspace_detect: Vec<crate::workspace::Detect>,
    /// write-back.git-*: 保存のあとの git(`[git]`。既定は何もしない)。
    pub git: crate::gitsync::Git,
    /// 全体の表のプロファイル(config.toml に書いたもの)。
    pub profile: Profile,
    /// `[templates.<名前>]`(書いた順)。
    pub templates: Templates,
    /// condition.define: `[conditions.<名前>]`(使える条件だけ。書いた順)。
    pub conditions: std::sync::Arc<crate::condition::Conditions>,
    /// notice.opt-in: `[[notices]]`(書いた順)。連絡の項目のパスは読んだまま(main が設定のフォルダで直す)。
    pub notices: Vec<crate::notice::Item>,
    /// notify.opt-in: `[[notify]]`(書いた順。使える項目だけ)。
    pub notify: Vec<crate::notify::Item>,
    /// navigation.across-option: `[search]` の `across`(パレットの「ワークスペースを横断して探す」。既定は偽)。
    pub search_across: bool,
    /// ai.ask-command: `[ask]` の `command`(AI に頼むコマンドと引数。全体の設定にだけ書ける。既定は空)。
    pub ask_command: Vec<String>,
    /// condition.marker・notice.marker・notify.marker: 開く表の印から重ねたもの(`for_table`)。印が無ければ None。
    pub ledger: Option<Ledger>,
}

/// 印(ワークスペースの印)から設定に重ねたもの(condition.marker・notice.marker・notify.marker)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ledger {
    /// 出どころの名前(`.mdgrid/workspace.toml (<ワークスペースの名前>)`。cli.resolved-config-ledger)。
    pub origin: String,
    /// 印の根。
    pub root: PathBuf,
    /// 全体の設定の条件(ほかの表を数えるとき、その表の印と重ね直す。`table_conditions`)。
    pub global: std::sync::Arc<crate::condition::Conditions>,
    /// `notices` の頭の、印から来た項目の数(tbd~notice.marker-order: 印の項目が先)。
    pub notices: usize,
    /// `notify` の頭の、印から来た項目の数。
    pub notify: usize,
}

/// 印の警告の場所の名前。
fn marker_label() -> String {
    format!(
        "{}/{}",
        crate::workspace::MARKER_DIR,
        crate::workspace::MARKER_FILE
    )
}

/// condition.marker: 印の表 `table`(文 `text`)の `[conditions]`(書いた順)。
fn marker_conditions(
    table: &toml::Table,
    text: &str,
    w: &mut Vec<String>,
) -> Vec<crate::condition::Condition> {
    let label = marker_label();
    table.get("conditions").map_or_else(Vec::new, |v| {
        crate::condition::in_written_order(crate::condition::read(v, &label, w), text)
    })
}

/// condition.marker-layer: 開く表 `opened` の印の条件を、全体の設定の条件 `global` に重ねた条件(印が無ければ
/// `global` のまま)。印の警告は捨てる(開く表の警告は `for_table` が出す)。ほかの表を数えるときに使う。
pub fn table_conditions(
    opened: &Path,
    global: &std::sync::Arc<crate::condition::Conditions>,
) -> std::sync::Arc<crate::condition::Conditions> {
    let Some((_, _, text, table)) = crate::workspace::marker_table(opened) else {
        return global.clone();
    };
    let mut w = Vec::new();
    let marker = marker_conditions(&table, &text, &mut w);
    if marker.is_empty() {
        return global.clone();
    }
    std::sync::Arc::new(crate::condition::Conditions::layered(
        marker,
        &marker_label(),
        global,
        &mut w,
    ))
}

/// notice.marker-inside: 印に書いた相対のパス `rel` が、台帳の根 `root` の中を指すか。絶対のパス・`~` で始まる
/// パス・`..` を含むパスは外とみなし、在るファイルはリンクをたどった先が根の中かも見る(まだ無いファイルは、
/// 根からの相対なので中)。
fn inside_root(root: &Path, rel: &Path) -> bool {
    use std::path::Component;
    if rel.to_string_lossy().starts_with('~')
        || rel.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return false;
    }
    match (
        std::fs::canonicalize(root.join(rel)),
        std::fs::canonicalize(root),
    ) {
        (Ok(full), Ok(r)) => full.starts_with(r),
        _ => true,
    }
}

/// condition.marker・notice.marker・notify.marker: 開く表 `opened`(フォルダか .base。改行でつないだ複数のパスは
/// 最初の1つ)で効く設定。上へたどって最初に見つかった印の `[conditions]`・`[[notices]]`・`[[notify]]` を、全体の
/// 設定のものに重ねる: 条件は印のものを先に並べて同じ名前は印のもの(condition.marker-layer)、通知の欄は印の項目を
/// 先に(tbd~notice.marker-order)、知らせは印の項目を先にし、印と同じ `cond` の全体の項目は使わない(notify.marker)。
/// 連絡の項目の相対の `note` は印の根から、`~` は `home` から。印の警告(`.mdgrid/workspace.toml: …`)も返す。
/// 印が無い・どれも書いていなければ、設定をそのまま返す。
pub fn for_table(c: &Config, opened: &Path, home: Option<&Path>) -> (Config, Vec<String>) {
    let mut w = Vec::new();
    let Some((root, origin, text, table)) = crate::workspace::marker_table(opened) else {
        return (c.clone(), w);
    };
    if !["conditions", "notices", "notify"]
        .iter()
        .any(|k| table.contains_key(*k))
    {
        return (c.clone(), w);
    }
    let label = marker_label();
    let marker = marker_conditions(&table, &text, &mut w);
    let conds = std::sync::Arc::new(crate::condition::Conditions::layered(
        marker,
        &label,
        &c.conditions,
        &mut w,
    ));
    // notice.marker: 書いていない条件の件数の項目は、警告して使わない(全体の設定と同じ。notice.count-item)。
    let mut notices: Vec<crate::notice::Item> = Vec::new();
    if let Some(v) = table.get("notices") {
        for (i, item) in crate::notice::read(v, &label, &mut w)
            .into_iter()
            .enumerate()
        {
            if let crate::notice::Item::Count { cond, .. } = &item {
                if conds.get(cond).is_none() {
                    warn(
                        &mut w,
                        &label,
                        &format!("notices[{}].cond", i + 1),
                        Msg::RsnNoticeNoCond.fill(&[cond]),
                    );
                    continue;
                }
            }
            // notice.marker-inside: 印は人から受け取ったフォルダにも入っているので、連絡の項目のノートは台帳の根の中だけ。
            if let crate::notice::Item::Note { path, .. } = &item {
                if !inside_root(&root, path) {
                    warn(
                        &mut w,
                        &label,
                        &format!("notices[{}].note", i + 1),
                        Msg::RsnNoticeNoteOutside.fill(&[&path.display()]),
                    );
                    continue;
                }
            }
            notices.push(item);
        }
    }
    crate::notice::resolve_notes(&mut notices, Some(&root), home);
    // notify.marker: 書いていない条件と、印の前の項目と同じ条件の項目は、警告して使わない(notify.item)。
    let mut notify: Vec<crate::notify::Item> = Vec::new();
    if let Some(v) = table.get("notify") {
        for (i, mut item) in crate::notify::read(v, &label, &mut w)
            .into_iter()
            .enumerate()
        {
            // notify.marker-no-command: 印は人から受け取ったフォルダにも入っているので、コマンドの出し先は外す
            // (開いただけ・--notify だけで、ほかの人が書いたコマンドを動かさない)。出し先が残らなければ使わない。
            if item.to.contains(&crate::notify::To::Command) || !item.command.is_empty() {
                warn(
                    &mut w,
                    &label,
                    &format!("notify[{}].command", i + 1),
                    Msg::RsnNotifyCommandInMarker.text().to_string(),
                );
                item.to.retain(|t| *t != crate::notify::To::Command);
                item.command.clear();
                if item.to.is_empty() {
                    continue;
                }
            }
            let at = format!("notify[{}].cond", i + 1);
            if conds.get(&item.cond).is_none() {
                warn(
                    &mut w,
                    &label,
                    &at,
                    Msg::RsnNoticeNoCond.fill(&[&item.cond]),
                );
            } else if notify.iter().any(|k| k.cond == item.cond) {
                warn(
                    &mut w,
                    &label,
                    &at,
                    Msg::RsnNotifyDupCond.fill(&[&item.cond]),
                );
            } else {
                notify.push(item);
            }
        }
    }
    let ledger = Ledger {
        origin,
        root,
        global: c.conditions.clone(),
        notices: notices.len(),
        notify: notify.len(),
    };
    let mut out = c.clone();
    out.conditions = conds;
    notices.extend(c.notices.iter().cloned());
    out.notices = notices;
    // 印と同じ条件を見張る全体の項目は、印の項目に置き換わる(警告しない)。
    let own: Vec<String> = notify.iter().map(|i| i.cond.clone()).collect();
    notify.extend(c.notify.iter().filter(|i| !own.contains(&i.cond)).cloned());
    out.notify = notify;
    out.ledger = Some(ledger);
    (out, w)
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: Language::Auto,
            editor: None,
            poll_ms: 1000,
            keys: Vec::new(),
            vim: false,
            terminal: Terminal::default(),
            workspace_detect: vec![crate::workspace::Detect::Vault],
            git: crate::gitsync::Git::default(),
            profile: Profile::default(),
            templates: Vec::new(),
            conditions: std::sync::Arc::default(),
            notices: Vec::new(),
            notify: Vec::new(),
            search_across: false,
            ask_command: Vec::new(),
            ledger: None,
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
            "language" => {
                if let Some(l) = value.as_str().and_then(Language::parse) {
                    c.language = l;
                } else {
                    // screen.locales-setting: 使える値は auto と台帳のある言語の符号。
                    let names: Vec<String> = Language::ALL
                        .iter()
                        .map(|l| format!("\"{}\"", l.name()))
                        .collect();
                    let want = Msg::WantLanguage.fill(&[&names.join(", ")]);
                    warn(&mut w, file, name, Msg::RsnNot.fill(&[&want]));
                }
            }
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
            "git" => read_git(value, &mut c.git, file, &mut w),
            "keys" => read_keys(value, &mut c.keys, &mut c.vim, file, &mut w),
            "templates" => c.templates = read_templates(value, file, &mut w),
            "notices" => c.notices = crate::notice::read(value, file, &mut w),
            "notify" => c.notify = crate::notify::read(value, file, &mut w),
            "search" => read_search(value, &mut c.search_across, file, &mut w),
            "ask" => read_ask(value, &mut c.ask_command, file, &mut w),
            "conditions" => {
                let specs = crate::condition::in_written_order(
                    crate::condition::read(value, file, &mut w),
                    text,
                );
                c.conditions =
                    std::sync::Arc::new(crate::condition::Conditions::build(specs, file, &mut w));
            }
            _ => warn(&mut w, file, name, Msg::RsnUnknown.text().to_string()),
        }
    }
    // notice.count-item: 書いていない条件の件数の項目は、警告して使わない。
    let conds = c.conditions.clone();
    let mut kept = Vec::new();
    for (i, item) in std::mem::take(&mut c.notices).into_iter().enumerate() {
        if let crate::notice::Item::Count { cond, .. } = &item {
            if conds.get(cond).is_none() {
                warn(
                    &mut w,
                    file,
                    &format!("notices[{}].cond", i + 1),
                    Msg::RsnNoticeNoCond.fill(&[cond]),
                );
                continue;
            }
        }
        kept.push(item);
    }
    c.notices = kept;
    // notify.item: 書いていない条件と、前の項目と同じ条件の項目は、警告して使わない。
    let mut kept: Vec<crate::notify::Item> = Vec::new();
    for (i, item) in std::mem::take(&mut c.notify).into_iter().enumerate() {
        let at = format!("notify[{}].cond", i + 1);
        if conds.get(&item.cond).is_none() {
            warn(&mut w, file, &at, Msg::RsnNoticeNoCond.fill(&[&item.cond]));
        } else if kept.iter().any(|k| k.cond == item.cond) {
            warn(&mut w, file, &at, Msg::RsnNotifyDupCond.fill(&[&item.cond]));
        } else {
            kept.push(item);
        }
    }
    c.notify = kept;
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
            "nerd_icons" => match v.as_bool() {
                Some(b) => t.nerd_icons = b,
                None => warn(w, file, &p, not(Msg::WantBool)),
            },
            "herdr" => match v.as_bool() {
                Some(b) => t.herdr = b,
                None => warn(w, file, &p, not(Msg::WantBool)),
            },
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
}

/// `[search]`(navigation.across-option): `across`(真偽)。知らない項目と型の違う値は警告して既定のまま。
/// `[ask]`(ai.ask-command): `command`(文字列の並び。空の並びは無いのと同じ)。
fn read_ask(v: &toml::Value, command: &mut Vec<String>, file: &str, w: &mut Vec<String>) {
    let Some(tab) = v.as_table() else {
        return warn(w, file, "ask", not(Msg::WantTable));
    };
    for (k, v) in tab {
        let p = format!("ask.{k}");
        match k.as_str() {
            "command" => match v.as_array().and_then(|a| {
                a.iter()
                    .map(|x| x.as_str().map(str::to_string))
                    .collect::<Option<Vec<String>>>()
            }) {
                Some(c) => *command = c,
                None => warn(w, file, &p, not(Msg::WantStringList)),
            },
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
}

fn read_search(v: &toml::Value, across: &mut bool, file: &str, w: &mut Vec<String>) {
    let Some(tab) = v.as_table() else {
        return warn(w, file, "search", not(Msg::WantTable));
    };
    for (k, v) in tab {
        let p = format!("search.{k}");
        match k.as_str() {
            "across" => match v.as_bool() {
                Some(b) => *across = b,
                None => warn(w, file, &p, not(Msg::WantBool)),
            },
            _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
        }
    }
}

/// `[git]`(write-back.git-*): `auto`・`when`・`message`。知らない値は警告して既定のまま。
fn read_git(v: &toml::Value, g: &mut crate::gitsync::Git, file: &str, w: &mut Vec<String>) {
    use crate::gitsync::{Level, Message, When};
    let Some(tab) = v.as_table() else {
        return warn(w, file, "git", not(Msg::WantTable));
    };
    for (k, v) in tab {
        let p = format!("git.{k}");
        let s = v.as_str();
        match k.as_str() {
            "auto" => match s.and_then(Level::parse) {
                Some(x) => g.auto = x,
                None => warn(w, file, &p, not(Msg::WantGitAuto)),
            },
            "when" => match s.and_then(When::parse) {
                Some(x) => g.when = x,
                None => warn(w, file, &p, not(Msg::WantGitWhen)),
            },
            "message" => match s.and_then(Message::parse) {
                Some(x) => g.message = x,
                None => warn(w, file, &p, not(Msg::WantGitMessage)),
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
                // WS-5: 値は目印のフォルダの名前(旧い値は legacy::lift が写して警告する)。
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
    vim: &mut bool,
    file: &str,
    w: &mut Vec<String>,
) {
    let Some(modes) = value.as_table() else {
        return warn(w, file, "keys", not(Msg::WantKeysTable));
    };
    for (mode, binds) in modes {
        // screen.vim-setting: `[keys]` の `vim`(真偽)は、モードの表でなく vim モードの入り切り。
        if mode == "vim" {
            match binds.as_bool() {
                Some(b) => *vim = b,
                None => warn(w, file, "keys.vim", not(Msg::WantBool)),
            }
            continue;
        }
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

/// SR-8: 設定・`$VISUAL`・`$EDITOR` のどれも無いときのエディタ。
pub const DEFAULT_EDITOR: &str = default_editor_for(cfg!(windows));

/// SR-8: OS ごとの既定のエディタ。Windows には vi がふつう無いので notepad。
pub const fn default_editor_for(windows: bool) -> &'static str {
    if windows {
        "notepad"
    } else {
        "vi"
    }
}

/// SR-8: ノートを開くエディタ。設定の `editor`・`$VISUAL`・`$EDITOR` の順で空でない最初のもの
/// (空白だけの値も空とみなす)、どれも無ければ `DEFAULT_EDITOR`。値は単語に分けずそのまま返す(分けるのは起動の側の `editor_argv`)。
/// 環境変数は呼ぶ側(main)が読んで渡す。
pub fn resolve_editor(config: Option<&str>, visual: Option<&str>, editor: Option<&str>) -> String {
    [config, visual, editor]
        .into_iter()
        .flatten()
        .find(|v| !v.trim().is_empty())
        .unwrap_or(DEFAULT_EDITOR)
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
    // screen.locales-all-in-ledger: 頭の説明・項目の説明・範囲・例の見出しは台帳から今の言語で出す。
    let mut out = String::new();
    for line in Msg::CfgPrintHead.text().lines() {
        out.push_str("# ");
        out.push_str(line);
        out.push('\n');
    }
    let mut section = "";
    for item in ITEMS {
        let sec = item.section();
        if !item.is_table() && sec != section {
            out.push_str(&format!("\n[{sec}]\n"));
            section = sec;
        }
        out.push('\n');
        for line in item.msg.text().lines() {
            out.push_str("# ");
            out.push_str(line.trim());
            out.push('\n');
        }
        out.push_str(&format!(
            "# {}\n",
            Msg::CfgScopeLine.fill(&[&item.scope.text()])
        ));
        match item.default {
            // 表の項目(`types` の既定 `{}` など)は、既定があっても書き方の例を出す(区画の見出しにする)。
            Some(v) if !item.is_table() => out.push_str(&format!("{} = {}\n", item.leaf(), v)),
            // 区画の下の項目の例は、区画の見出しの行を除いて出す(今の区画の中)。
            None if !item.is_table() && !sec.is_empty() => {
                out.push_str(&format!("# {}\n", Msg::CfgExampleLine.text()));
                for line in item.example.lines().skip(1) {
                    out.push_str("# ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            _ => {
                out.push_str(&format!("# {}\n", Msg::CfgExampleLine.text()));
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
    out.push_str(src(c.terminal.nerd_icons == d.terminal.nerd_icons));
    out.push_str(&line(
        "nerd_icons",
        toml::Value::Boolean(c.terminal.nerd_icons),
    ));
    out.push_str(src(c.terminal.herdr == d.terminal.herdr));
    out.push_str(&line("herdr", toml::Value::Boolean(c.terminal.herdr)));
    out.push_str("\n[workspace]\n");
    out.push_str(src(c.workspace_detect == d.workspace_detect));
    let detect = c
        .workspace_detect
        .iter()
        .map(|m| toml::Value::String(m.name().into()))
        .collect();
    out.push_str(&line("detect", toml::Value::Array(detect)));
    out.push_str("\n[git]\n");
    out.push_str(src(c.git.auto == d.git.auto));
    out.push_str(&line("auto", toml::Value::String(c.git.auto.name().into())));
    out.push_str(src(c.git.when == d.git.when));
    out.push_str(&line("when", toml::Value::String(c.git.when.name().into())));
    out.push_str(src(c.git.message == d.git.message));
    out.push_str(&line(
        "message",
        toml::Value::String(c.git.message.name().into()),
    ));
    out.push_str("\n[search]\n");
    out.push_str(src(c.search_across == d.search_across));
    out.push_str(&line("across", toml::Value::Boolean(c.search_across)));
    out.push_str("\n[ask]\n");
    out.push_str(src(c.ask_command == d.ask_command));
    let command = c
        .ask_command
        .iter()
        .map(|s| toml::Value::String(s.clone()))
        .collect();
    out.push_str(&line("command", toml::Value::Array(command)));
    out.push_str("\n[keys]\n");
    out.push_str(src(c.vim == d.vim));
    out.push_str(&line("vim", toml::Value::Boolean(c.vim)));
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

/// cli.resolved-config-ledger: その表で効く名前の条件・通知の欄・知らせの項目(`for_table` で印を重ねた設定)を、
/// 項目ごとに出どころ(`# config.toml` か `# .mdgrid/workspace.toml (<名前>)`)のコメントを添えた TOML にする。
/// 設定のファイルとしてそのまま読める形(cli.resolved-config-reads-back)。
pub fn ledger_toml(c: &Config) -> String {
    let marker = c.ledger.as_ref().map(|l| l.origin.as_str());
    let origin = |from_marker: bool| match (from_marker, marker) {
        (true, Some(m)) => m,
        _ => CONFIG_FILE,
    };
    let s = |v: &str| toml::Value::String(v.to_string());
    let scope_value = |sc: &crate::notice::Scope| match sc {
        crate::notice::Scope::Table => s("table"),
        crate::notice::Scope::Workspace => s("workspace"),
        crate::notice::Scope::Tables(names) => {
            toml::Value::Array(names.iter().map(|n| s(n)).collect())
        }
    };
    let block = |key: &str, t: toml::Table, array: bool| {
        let mut outer = toml::Table::new();
        if array {
            outer.insert(key.into(), toml::Value::Array(vec![toml::Value::Table(t)]));
        } else {
            outer.insert(key.into(), toml::Value::Table(t));
        }
        toml::to_string(&outer).unwrap_or_default()
    };
    let mut out = String::new();
    for cond in c.conditions.list() {
        let mut t = toml::Table::new();
        t.insert("expr".into(), s(&cond.src));
        if let Some(l) = &cond.label {
            t.insert("label".into(), s(l));
        }
        if let Some(k) = cond.key {
            t.insert("key".into(), toml::Value::Integer(i64::from(k)));
        }
        if cond.hide {
            t.insert("hide".into(), toml::Value::Boolean(true));
        }
        let mut named = toml::Table::new();
        named.insert(cond.name.clone(), toml::Value::Table(t));
        out.push_str(&format!(
            "\n# {}\n",
            origin(c.conditions.from_marker(&cond.name))
        ));
        out.push_str(&block("conditions", named, false));
    }
    let own_notices = c.ledger.as_ref().map_or(0, |l| l.notices);
    for (i, item) in c.notices.iter().enumerate() {
        let mut t = toml::Table::new();
        match item {
            crate::notice::Item::Count {
                cond,
                text,
                scope,
                show_zero,
            } => {
                t.insert("cond".into(), s(cond));
                if let Some(x) = text {
                    t.insert("text".into(), s(x));
                }
                t.insert("scope".into(), scope_value(scope));
                if *show_zero {
                    t.insert("show_zero".into(), toml::Value::Boolean(true));
                }
            }
            crate::notice::Item::Note { path, lines } => {
                t.insert("note".into(), s(&path.to_string_lossy()));
                let n = i64::try_from(*lines).unwrap_or(1);
                t.insert("lines".into(), toml::Value::Integer(n));
            }
        }
        out.push_str(&format!("\n# {}\n", origin(i < own_notices)));
        out.push_str(&block("notices", t, true));
    }
    let own_notify = c.ledger.as_ref().map_or(0, |l| l.notify);
    for (i, item) in c.notify.iter().enumerate() {
        let mut t = toml::Table::new();
        t.insert("cond".into(), s(&item.cond));
        t.insert("scope".into(), scope_value(&item.scope));
        if let Some(x) = &item.title {
            t.insert("title".into(), s(x));
        }
        if let Some(x) = &item.text {
            t.insert("text".into(), s(x));
        }
        let to = item.to.iter().map(|x| s(x.name())).collect();
        t.insert("to".into(), toml::Value::Array(to));
        let n = i64::try_from(item.group_over).unwrap_or(i64::MAX);
        t.insert("group_over".into(), toml::Value::Integer(n));
        if let Some(x) = &item.group_text {
            t.insert("group_text".into(), s(x));
        }
        if !item.command.is_empty() {
            let argv = item.command.iter().map(|x| s(x)).collect();
            t.insert("command".into(), toml::Value::Array(argv));
        }
        out.push_str(&format!("\n# {}\n", origin(i < own_notify)));
        out.push_str(&block("notify", t, true));
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

/// $XDG_CONFIG_HOME/mdgrid/config.toml、無ければ ~/.config/mdgrid/config.toml(Windows では
/// %APPDATA%\mdgrid\config.toml。CLI-3)。どの環境変数も無い(空の)ときは None。
pub fn config_path() -> Option<PathBuf> {
    config_path_in(&|k| std::env::var_os(k), cfg!(windows))
}

/// 見た目の状態の既定の置き場: $XDG_STATE_HOME/mdgrid/、無ければ ~/.local/state/mdgrid/
/// (Windows では %LOCALAPPDATA%\mdgrid\)。
pub fn state_dir() -> Option<PathBuf> {
    state_dir_in(&|k| std::env::var_os(k), cfg!(windows))
}

/// ホームのフォルダ: $HOME、無ければ Windows では %USERPROFILE%(先頭の `~` の読み替えと短い見せ方)。
/// ホームのフォルダの下なら `~/…` で書く。`~/` の後ろの区切りはどの OS でも `/`(Windows でも `~/notes/tasks`。
/// 書いた設定を別のマシンへ持っていけるように。読むときは `~/` の後ろをホームにつなぐので、Windows でも `/` のまま読める)。
pub fn home_short(p: &Path) -> String {
    if let Some(home) = home_dir() {
        if let Ok(rest) = p.strip_prefix(&home) {
            if !rest.as_os_str().is_empty() {
                let parts: Vec<String> = rest
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                return format!("~/{}", parts.join("/"));
            }
        }
    }
    p.to_string_lossy().into_owned()
}

pub fn home_dir() -> Option<PathBuf> {
    home_dir_in(&|k| std::env::var_os(k), cfg!(windows))
}

/// 環境変数を読む関数(試験では作った値を渡す)。
pub(crate) type Env<'a> = &'a dyn Fn(&str) -> Option<std::ffi::OsString>;

pub(crate) fn config_path_in(env: Env<'_>, windows: bool) -> Option<PathBuf> {
    base_dir_in(env, windows, "XDG_CONFIG_HOME", ".config", "APPDATA")
        .map(|d| d.join("mdgrid").join("config.toml"))
}

pub(crate) fn state_dir_in(env: Env<'_>, windows: bool) -> Option<PathBuf> {
    base_dir_in(
        env,
        windows,
        "XDG_STATE_HOME",
        ".local/state",
        "LOCALAPPDATA",
    )
    .map(|d| d.join("mdgrid"))
}

pub(crate) fn home_dir_in(env: Env<'_>, windows: bool) -> Option<PathBuf> {
    let var = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    var("HOME").or_else(|| if windows { var("USERPROFILE") } else { None })
}

/// `XDG_*`(絶対パスだけ)、無ければ Windows は `win`(`APPDATA` など)、ほかは `$HOME/<home_rel>`。
fn base_dir_in(
    env: Env<'_>,
    windows: bool,
    xdg: &str,
    home_rel: &str,
    win: &str,
) -> Option<PathBuf> {
    // XDG の仕様どおり、絶対パスでない値は無視する。
    if let Some(v) = env(xdg) {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return Some(p);
        }
    }
    if windows {
        return env(win).filter(|v| !v.is_empty()).map(PathBuf::from);
    }
    let home = env("HOME").filter(|h| !h.is_empty())?;
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

/// screen.state-store-follow-order: 覚えた列の並び `saved`(覚えたときの印の `order` が `before`)を、今の印の
/// `order` の `now` に合わせた並び。`now` から始め、人が動かした列(`before` から `saved` への最長の共通の並びに
/// 入らない列)のうち、印も動かしていない列だけを、`saved` で直前にあった列の後ろ(無ければ先頭)へ移す。直前の
/// 列を印が動かしていれば、食い違うので印のままにする。印が消した列は戻さない。
pub fn follow_order(before: &[String], saved: &[String], now: &[String]) -> Vec<String> {
    let kept_by_user = common_order(before, saved);
    let kept_by_marker = common_order(before, now);
    let marker_moved = |c: &String| now.contains(c) && !kept_by_marker.contains(c);
    let mut out: Vec<String> = now.to_vec();
    for (i, c) in saved.iter().enumerate() {
        if kept_by_user.contains(c) || marker_moved(c) {
            continue;
        }
        let anchor = saved[..i].iter().rev().find(|p| out.contains(p));
        if anchor.is_some_and(marker_moved) {
            continue;
        }
        match out.iter().position(|x| x == c) {
            Some(at) => {
                out.remove(at);
            }
            // 印が消した列は戻さない(印に無かった列は、あとで現れたときのために位置を残す)。
            None if before.contains(c) => continue,
            None => {}
        }
        let at = anchor
            .and_then(|p| out.iter().position(|x| x == p))
            .map_or(0, |k| k + 1);
        out.insert(at, c.clone());
    }
    out
}

/// 2つの並びの最長の共通の並び(並びの中の列は重ならない)。同じ長さなら前の並び `a` の前の方を残す。
fn common_order(a: &[String], b: &[String]) -> Vec<String> {
    let (n, m) = (a.len(), b.len());
    let mut t = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a[i] == b[j] {
                t[i + 1][j + 1] + 1
            } else {
                t[i + 1][j].max(t[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(a[i].clone());
            i += 1;
            j += 1;
        } else if t[i][j + 1] >= t[i + 1][j] {
            j += 1;
        } else {
            i += 1;
        }
    }
    out
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
    /// screen.state-store-follow-order: 列の並びを覚えたときの印(`.base`)のビューの `order` の控え。印の `order` が
    /// 無いビューと、並びを覚えていない状態では書かない。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base_order: Option<Vec<String>>,
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
        name.push_str(&format!("{b:02x}"));
    }
    name.push_str(".toml");
    name
}

/// 状態を読む。無い・読めない・壊れている・別の対象のもの → 既定(空)。
pub fn load_state(dir: &Path, target: &Path, view: &str) -> ViewState {
    load_state_with_base(dir, target, view).0
}

/// 状態と、列の並びを覚えたときの印の `order` の控え(screen.state-store-follow-order。無ければ None)を読む。
pub fn load_state_with_base(
    dir: &Path,
    target: &Path,
    view: &str,
) -> (ViewState, Option<Vec<String>>) {
    let key = target_key(target);
    let path = dir.join(state_file_name(&key, view));
    let Ok(bytes) = crate::textfile::read_bytes(&path) else {
        return (ViewState::default(), None);
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return (ViewState::default(), None);
    };
    // 設定(NV-17)は別に読む。設定の表が壊れていても、ほかの見た目の状態は捨てない。
    let Ok(mut table) = toml::from_str::<toml::Table>(text) else {
        return (ViewState::default(), None);
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
            (s, f.base_order)
        }
        _ => (ViewState::default(), None),
    }
}

/// 設定の表を読む。読めなければ既定(SR-11: 状態は消えても困らない)。
fn read_settings(v: toml::Value) -> crate::settings::Settings {
    crate::settings::Settings::deserialize(v).unwrap_or_default()
}

/// 状態を書く。dir の中に一時ファイルを書いてから名前を変えて置き換える。
pub fn save_state(dir: &Path, target: &Path, view: &str, s: &ViewState) -> io::Result<()> {
    save_state_with_base(dir, target, view, s, None)
}

/// 状態を、列の並びを覚えたときの印の `order` の控え `base`(screen.state-store-follow-order)と一緒に書く。
pub fn save_state_with_base(
    dir: &Path,
    target: &Path,
    view: &str,
    s: &ViewState,
    base: Option<&[String]>,
) -> io::Result<()> {
    let key = target_key(target);
    let name = state_file_name(&key, view);
    let mut state = s.clone();
    let settings = std::mem::take(&mut state.settings);
    let file = StateFile {
        target: key,
        view_name: view.to_string(),
        state,
        settings,
        base_order: base.map(<[String]>::to_vec),
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
    let prefix = format!(".{name}.tmp.");
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

#[cfg(test)]
#[path = "test_windows_unit.rs"]
mod tests_windows;
