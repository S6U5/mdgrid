//! 設定(CLI-3・SR-13)と見た目の状態(SR-11・SR-12)。形は docs/design.md の `src/config.rs`。
//!
//! 設定はユーザーが書く TOML(`$XDG_CONFIG_HOME/mdgrid/config.toml`)。見た目の状態は
//! 消えても困らないもので、ノートのフォルダには書かず、呼ぶ側が渡す状態のフォルダ
//! (`$XDG_STATE_HOME/mdgrid/` など)に、対象のパスとビューの名前ごとに1つの TOML として置く。

pub use crate::display::Display;
pub use crate::i18n::Language;
use crate::i18n::Msg;
use crate::newnote::{self, NewNote};
use crate::theme::Theme;
use crate::types::{DateFormat, WeekStart};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 設定(CLI-3)。`parse("")` が既定。
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// (モード, キーの表記 "j"・"ctrl+s"・"shift+tab", 動作の名前か "none")。SR-13。
    pub keys: Vec<(String, String, String)>,
    /// false で色なし(NO_COLOR と同じ)。
    pub color: bool,
    /// CE-3 の候補の上限。
    pub candidates: usize,
    /// BV-9 の読み直しの間隔(ミリ秒)。
    pub poll_ms: u64,
    /// CV-6: East Asian Ambiguous を幅2にする。
    pub ambiguous_wide: bool,
    /// SR-32: 窓の枠を ASCII(`+ - |`)で描く(既定は角の丸い罫線)。
    pub borders_ascii: bool,
    /// SR-33: 色を使うときの今までの見た目(反転)。既定は lazygit のようなモダンな見た目。
    pub look_classic: bool,
    /// SR-34: ビューが1つならタブの行を出さない(`view_tabs = "auto"`)。既定は "always"(いつも出す)。
    pub view_tabs_auto: bool,
    /// SR-35: セルの部品(`cells`)。
    pub cells: crate::cells::Cells,
    /// WS-5: ワークスペースの検知(既定は保管庫だけ)。
    pub workspace_detect: Vec<crate::workspace::Detect>,
    /// NV-23: 表の上に検索の欄を出す(既定 true)。false なら出さず、簡易の絞り込み(NV-2)は最下行で打つ。
    pub search_bar: bool,
    /// CE-22: 表の日付の見せ方と打ち込みの形(既定 `YYYY-MM-DD`)。ノートに書く形は変えない。
    pub date_format: DateFormat,
    /// CE-21: カレンダーの週の始まり(既定 日曜。設定の値は `"sun"`・`"mon"`)。
    pub week_start: WeekStart,
    /// WB-3: フロントマターの無いノートと空のフロントマターのノートに書く(既定 true)。false ならこの2つを読むだけ。
    pub add_frontmatter: bool,
    /// SR-8: ノートを開くエディタ(引数つきでよい)。None(既定。空の文字列も同じ)なら $VISUAL・$EDITOR・vi。
    pub editor: Option<String>,
    /// SR-23: 画面と起動の文言の言語(既定 `Auto` は環境変数に従う)。
    pub language: Language,
    /// SR-26・SR-27: 画面のテーマ(既定 `Default` は今の見た目)。
    pub theme: Theme,
    /// CE-26・CE-27: 新しいノートの決まり(`[new_note]`)。既定は空(開いたフォルダ・雛形なし・聞かない・入れない)。
    pub new_note: NewNote,
    /// SR-20・SR-21: 表の見せ方(`[display]`)。検索の欄は最上位の `search_bar` のまま。
    pub display: Display,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            keys: Vec::new(),
            color: true,
            candidates: 20,
            poll_ms: 1000,
            ambiguous_wide: false,
            borders_ascii: false,
            look_classic: false,
            view_tabs_auto: false,
            cells: crate::cells::Cells::default(),
            workspace_detect: vec![crate::workspace::Detect::Vault],
            search_bar: true,
            date_format: DateFormat::iso(),
            week_start: WeekStart::Sun,
            add_frontmatter: true,
            editor: None,
            language: Language::Auto,
            theme: Theme::Default,
            new_note: NewNote::default(),
            display: Display::default(),
        }
    }
}

/// TOML を読む。知らない項目・型の違う項目は警告の文にして返し、止めない(CLI-3)。
/// 壊れた TOML は Err(理由1行)。
pub fn parse(text: &str) -> Result<(Config, Vec<String>), String> {
    let table: toml::Table =
        text.parse()
            .map_err(|e: toml::de::Error| match toml_error(text, &e) {
                (Some(line), msg) => Msg::ConfigTomlLine.fill(&[&line, &msg]),
                (None, msg) => Msg::ConfigToml.fill(&[&msg]),
            })?;

    let mut c = Config::default();
    let mut warnings = Vec::new();
    for (name, value) in &table {
        // 知る項目かどうかは項目の表(ITEMS)で決める(CLI-12)。
        if !KEYS.contains(&name.as_str()) {
            warnings.push(Msg::ConfigUnknownItem.fill(&[name]));
            continue;
        }
        match name.as_str() {
            "candidates" => match value.as_integer().and_then(|n| usize::try_from(n).ok()) {
                Some(n) => c.candidates = n,
                None => warnings.push(type_warning(name, Msg::WantIntAtLeast0)),
            },
            "poll_ms" => match value.as_integer().and_then(|n| u64::try_from(n).ok()) {
                Some(n) if n > 0 => c.poll_ms = n,
                _ => warnings.push(type_warning(name, Msg::WantIntAtLeast1)),
            },
            "ambiguous_wide" => match value.as_bool() {
                Some(b) => c.ambiguous_wide = b,
                None => warnings.push(type_warning(name, Msg::WantBool)),
            },
            "workspace_detect" => match value.as_array() {
                Some(items) => {
                    let parsed: Option<Vec<_>> = items
                        .iter()
                        .map(|v| v.as_str().and_then(crate::workspace::Detect::parse))
                        .collect();
                    match parsed {
                        Some(d) => c.workspace_detect = d,
                        None => warnings.push(type_warning(name, Msg::WantDetect)),
                    }
                }
                None => warnings.push(type_warning(name, Msg::WantDetect)),
            },
            "look" => match value.as_str() {
                Some("modern") => c.look_classic = false,
                Some("classic") => c.look_classic = true,
                _ => warnings.push(type_warning(name, Msg::WantLook)),
            },
            "view_tabs" => match value.as_str() {
                Some("always") => c.view_tabs_auto = false,
                Some("auto") => c.view_tabs_auto = true,
                _ => warnings.push(type_warning(name, Msg::WantViewTabs)),
            },
            "borders" => match value.as_str() {
                Some("rounded") => c.borders_ascii = false,
                Some("ascii") => c.borders_ascii = true,
                _ => warnings.push(type_warning(name, Msg::WantBorders)),
            },
            "color" => match value.as_bool() {
                Some(b) => c.color = b,
                None => warnings.push(type_warning(name, Msg::WantBool)),
            },
            "search_bar" => match value.as_bool() {
                Some(b) => c.search_bar = b,
                None => warnings.push(type_warning(name, Msg::WantBool)),
            },
            "add_frontmatter" => match value.as_bool() {
                Some(b) => c.add_frontmatter = b,
                None => warnings.push(type_warning(name, Msg::WantBool)),
            },
            "date_format" => match value.as_str() {
                Some(p) => match DateFormat::parse(p) {
                    Ok(f) => c.date_format = f,
                    Err(e) => warnings.push(Msg::ConfigBadDateFormat.fill(&[&squash_ws(&e)])),
                },
                None => warnings.push(type_warning(name, Msg::WantDateFormat)),
            },
            "week_start" => match value.as_str() {
                Some("sun") => c.week_start = WeekStart::Sun,
                Some("mon") => c.week_start = WeekStart::Mon,
                _ => warnings.push(type_warning(name, Msg::WantWeekStart)),
            },
            "editor" => match value.as_str() {
                // 空(空白だけも)は無いのと同じ(SR-8)。
                Some(e) => c.editor = (!e.trim().is_empty()).then(|| e.to_string()),
                None => warnings.push(type_warning(name, Msg::WantString)),
            },
            "language" => match value.as_str().and_then(Language::parse) {
                Some(l) => c.language = l,
                None => warnings.push(type_warning(name, Msg::WantLanguage)),
            },
            "theme" => match value.as_str().and_then(Theme::parse) {
                Some(t) => c.theme = t,
                None => warnings.push(type_warning(name, Msg::WantTheme)),
            },
            "keys" => read_keys(value, &mut c.keys, &mut warnings),
            "new_note" => c.new_note = read_new_note(value, &mut warnings),
            "display" => read_display(value, &mut c.display, &mut warnings),
            "cells" => crate::cells::read(value, &mut c.cells, &mut warnings),
            // 表にあって読み取りの無い項目は単体の試験で落とす(test_config_unit)。
            _ => {}
        }
    }
    Ok((c, warnings))
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

#[path = "config_items.rs"]
mod items;
pub use items::{Item, ITEMS, KEYS};

/// `--print-config` の出力(CLI-11): 全項目を既定値と英語の説明のコメント付きで並べた TOML。
/// 既定で書かない項目(`editor`・`keys`)は書き方の例をコメントで出す。読み直すと警告なしで既定と同じ。
pub fn default_toml() -> String {
    // 文書の案内は、公開してリポの URL が決まったら URL に替える(今は公開前。SC-10)。
    let mut out = String::from(
        "# mdgrid configuration with every item at its default value.\n\
         # Location: $XDG_CONFIG_HOME/mdgrid/config.toml (or ~/.config/mdgrid/config.toml).\n\
         # Options: mdgrid --help. Full reference: docs/config.md in the mdgrid repository.\n",
    );
    for item in ITEMS {
        out.push('\n');
        for line in item.en.lines() {
            out.push_str("# ");
            out.push_str(line.trim());
            out.push('\n');
        }
        match item.default {
            Some(v) => out.push_str(&format!("{} = {}\n", item.name, v)),
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

fn read_keys(
    value: &toml::Value,
    keys: &mut Vec<(String, String, String)>,
    warnings: &mut Vec<String>,
) {
    let Some(modes) = value.as_table() else {
        warnings.push(type_warning("keys", Msg::WantKeysTable));
        return;
    };
    for (mode, binds) in modes {
        let Some(binds) = binds.as_table() else {
            warnings.push(type_warning(
                &format!("keys.{}", mode),
                Msg::WantKeyActionTable,
            ));
            continue;
        };
        for (key, action) in binds {
            match action.as_str() {
                Some(a) => keys.push((mode.clone(), key.clone(), a.to_string())),
                None => warnings.push(type_warning(
                    &format!("keys.{}.{}", mode, key),
                    Msg::WantActionName,
                )),
            }
        }
    }
}

/// `[new_note]`(CE-27)を読む。型の違う値・知らない項目は警告にして飛ばす。
fn read_new_note(value: &toml::Value, warnings: &mut Vec<String>) -> NewNote {
    let mut n = NewNote::default();
    let Some(t) = value.as_table() else {
        warnings.push(type_warning("new_note", Msg::WantNewNoteTable));
        return n;
    };
    for (k, v) in t {
        let name = format!("new_note.{k}");
        match k.as_str() {
            "folder" => match v.as_str() {
                Some(s) => n.folder = s.to_string(),
                None => warnings.push(type_warning(&name, Msg::WantString)),
            },
            "name" => match v.as_str() {
                Some(s) => n.name = s.to_string(),
                None => warnings.push(type_warning(&name, Msg::WantString)),
            },
            "mode" => match v.as_str() {
                Some(s @ ("form" | "editor")) => n.mode = s.to_string(),
                Some(s) => warnings.push(Msg::ConfigBadNoteMode.fill(&[&s])),
                None => warnings.push(type_warning(&name, Msg::WantString)),
            },
            "body" => match v.as_str() {
                Some(s) => n.body = s.to_string(),
                None => warnings.push(type_warning(&name, Msg::WantString)),
            },
            "required" | "hidden" => match v.as_array().and_then(|a| {
                a.iter()
                    .map(|x| x.as_str().map(str::to_string))
                    .collect::<Option<Vec<String>>>()
            }) {
                Some(cols) => {
                    let out = if k == "required" {
                        &mut n.required
                    } else {
                        &mut n.hidden
                    };
                    for c in cols {
                        if newnote::not_a_key(&c) || out.contains(&c) {
                            warnings.push(Msg::ConfigBadAsk.fill(&[&c]));
                        } else {
                            out.push(c);
                        }
                    }
                }
                None => warnings.push(type_warning(&name, Msg::WantColumnNames)),
            },
            "ask" => match v.as_array().and_then(|a| {
                a.iter()
                    .map(|x| x.as_str().map(str::to_string))
                    .collect::<Option<Vec<String>>>()
            }) {
                Some(cols) => {
                    for c in cols {
                        if newnote::not_a_key(&c) || n.ask.contains(&c) {
                            warnings.push(Msg::ConfigBadAsk.fill(&[&c]));
                        } else {
                            n.ask.push(c);
                        }
                    }
                }
                None => warnings.push(type_warning(&name, Msg::WantColumnNames)),
            },
            "set" => {
                let Some(set) = v.as_table() else {
                    warnings.push(type_warning(&name, Msg::WantColumnValueTable));
                    continue;
                };
                for (col, x) in set {
                    if newnote::not_a_key(col) {
                        warnings.push(Msg::ConfigBadSet.fill(&[col]));
                        continue;
                    }
                    match newnote::value_from_toml(x) {
                        Some(nv) => n.set.push((col.clone(), nv)),
                        None => warnings.push(type_warning(
                            &format!("new_note.set.{col}"),
                            Msg::WantSetValue,
                        )),
                    }
                }
            }
            _ => warnings.push(Msg::ConfigUnknownItem.fill(&[&name])),
        }
    }
    n
}

/// `[display]`(SR-21)を読む。表でない・型の違う値・知らない項目は警告にして既定のまま。
fn read_display(value: &toml::Value, d: &mut Display, warnings: &mut Vec<String>) {
    let Some(t) = value.as_table() else {
        warnings.push(type_warning("display", Msg::WantDisplayTable));
        return;
    };
    for (k, v) in t {
        let name = format!("display.{k}");
        match d.field_mut(k) {
            None => warnings.push(Msg::ConfigUnknownItem.fill(&[&name])),
            Some(slot) => match v.as_bool() {
                Some(b) => *slot = b,
                None => warnings.push(type_warning(&name, Msg::WantBool)),
            },
        }
    }
}

/// 型の違う項目の警告。`want` は求める型(`Msg::Want*`)。
fn type_warning(name: &str, want: Msg) -> String {
    Msg::ConfigWrongType.fill(&[&name, &want.text()])
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
/// (`inner` に挙げたものを除く)か `to` の前まで。末尾のコメントと空行は中身に含めない(次の区画の前置き)。
pub(crate) fn toml_blocks(
    lines: &[&str],
    from: usize,
    to: usize,
    head: &str,
    inner: &[&str],
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
                Some(h) if !inner.contains(&h.as_str()) => break,
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
