//! 名前を付けたセッション(cli.session-*): 表の表示(開く表・ビュー・画面で掛けた絞り込み・並び・列の並びと
//! 隠す列・読むだけ)に名前を付けたもの。設定の置き場の `sessions.toml` に `[sessions.<名前>]` で置き、ノートの
//! フォルダと `.base` には書かない(cli.session-store)。値は持たず、起動のたびにノートから作り直す
//! (cli.session-content-fresh-values)。画面に依存しない(当てるのは src/ui/sessions.rs)。
//!
//! 形(手で書いてもよい。知らない項目は警告にとどめる):
//!
//! ```toml
//! [sessions."進行中"]
//! folders = ["~/notes/tasks"]
//! base = "~/notes/tasks.base"   # 任意(あればこちらを開く)
//! view = "表"                    # 任意
//! columns = ["title", "status", "due"]
//! hidden = ["priority"]
//! readonly = false
//!
//! [[sessions."進行中".filters]]
//! kind = "value"                 # text・value・expr・cond・show
//! key = "status"
//! value = "進行中"
//!
//! [[sessions."進行中".sort]]
//! key = "due"
//! dir = "asc"
//! ```

use crate::config;
use crate::i18n::Msg;
use crate::profile::{not, not_one_of, warn};
use crate::settings::Dir;
use std::io;
use std::path::{Path, PathBuf};

/// セッションのファイルの名前(config.toml・views.toml と同じ設定のフォルダに置く)。
pub const FILE_NAME: &str = "sessions.toml";

/// 画面で掛けた絞り込みの1つ(cli.session-content-fields)。掛けた種類のまま持つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// navigation.search: `\` で打った語(`kind = "text"`)。
    Text(String),
    /// navigation.same-value と頻度表: 列の値の行だけ(`kind = "value"`)。`by_keys` は頻度表の数え方で比べる、
    /// `list` はリストの列の要素で絞った。
    Value {
        key: String,
        value: String,
        by_keys: bool,
        list: bool,
    },
    /// navigation.expr-filter: 式(`kind = "expr"`)。
    Expr(String),
    /// condition.digit-keys: 数字のキーで入れた名前の条件(`kind = "cond"`)。
    Cond(String),
    /// condition.hide-toggle: いつも隠す条件を止めたもの(`kind = "show"`)。
    Show(String),
}

/// 名前を付けたセッション1つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    pub name: String,
    /// 開くフォルダ(1つ以上。`.base` で開いたときはそのフォルダ。先頭の `~` はホームに読み替え済み)。
    pub folders: Vec<PathBuf>,
    /// 開く `.base`(あればフォルダの代わりにこれを開く)。
    pub base: Option<PathBuf>,
    /// 開くビューの名前(`.base` のビューか mdgrid のビュー)。
    pub view: Option<String>,
    /// 見せる列(並びのまま)と隠す列。
    pub columns: Vec<String>,
    pub hidden: Vec<String>,
    /// 並び(見出しからの並べ替えと並べ替えの窓の決まり。先のものが先に効く)。
    pub sort: Vec<(String, Dir)>,
    pub filters: Vec<Filter>,
    /// 読むだけで開く(`--readonly`)。
    pub readonly: bool,
}

impl Session {
    /// 開くパス(`.base` があればそれ1つ、無ければフォルダの並び)。
    pub fn open_paths(&self) -> Vec<PathBuf> {
        match &self.base {
            Some(b) => vec![b.clone()],
            None => self.folders.clone(),
        }
    }
}

/// 設定のフォルダの sessions.toml を読む。無ければ空。壊れていれば理由1行(cli.session-errors)。
/// 知らない項目と型の違う値は警告にして飛ばす(cli.session-store)。並びは名前の順。
pub fn load(dir: &Path) -> Result<(Vec<Session>, Vec<String>), String> {
    let path = dir.join(FILE_NAME);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_named(&text, &path.display().to_string()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok((Vec::new(), Vec::new())),
        Err(e) => Err(Msg::SessionsUnreadable.fill(&[&path.display(), &e])),
    }
}

/// sessions.toml の文を読む(壊れていれば、ファイルの名前を添えた理由)。
pub fn parse(text: &str) -> Result<(Vec<Session>, Vec<String>), String> {
    parse_named(text, FILE_NAME)
}

fn parse_named(text: &str, shown: &str) -> Result<(Vec<Session>, Vec<String>), String> {
    let table: toml::Table = text.parse().map_err(|e| {
        let (line, msg) = config::toml_error(text, &e);
        let at = line.map_or_else(|| "?".to_string(), |l| l.to_string());
        Msg::SessionsBadToml.fill(&[&shown, &at, &msg])
    })?;
    let mut warns = Vec::new();
    let mut out = Vec::new();
    for (k, v) in &table {
        if k != "sessions" {
            warn(&mut warns, FILE_NAME, k, Msg::RsnUnknown.text().to_string());
            continue;
        }
        let Some(all) = v.as_table() else {
            warn(&mut warns, FILE_NAME, k, not(Msg::WantTable));
            continue;
        };
        for (name, item) in all {
            let at = format!("sessions.{name}");
            let Some(t) = item.as_table() else {
                warn(&mut warns, FILE_NAME, &at, not(Msg::WantTable));
                continue;
            };
            out.push(read_one(name, t, &at, &mut warns));
        }
    }
    Ok((out, warns))
}

/// 文字列の並び(違えば警告して空)。
fn strings(v: &toml::Value, at: &str, warns: &mut Vec<String>) -> Vec<String> {
    let list: Option<Vec<String>> = v.as_array().and_then(|a| {
        a.iter()
            .map(|x| x.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
    });
    list.unwrap_or_else(|| {
        warn(warns, FILE_NAME, at, not(Msg::WantStringList));
        Vec::new()
    })
}

/// 文字列(違えば警告して None)。
fn string(v: &toml::Value, at: &str, warns: &mut Vec<String>) -> Option<String> {
    let s = v.as_str().map(str::to_string);
    if s.is_none() {
        warn(warns, FILE_NAME, at, not(Msg::WantString));
    }
    s
}

fn read_one(name: &str, t: &toml::Table, at: &str, warns: &mut Vec<String>) -> Session {
    let mut s = Session {
        name: name.to_string(),
        ..Session::default()
    };
    for (k, v) in t {
        let here = format!("{at}.{k}");
        match k.as_str() {
            "folders" => {
                s.folders = strings(v, &here, warns)
                    .iter()
                    .map(|p| crate::places::expand_home(p.trim()))
                    .collect();
            }
            "base" => {
                s.base = string(v, &here, warns)
                    .filter(|p| !p.trim().is_empty())
                    .map(|p| crate::places::expand_home(p.trim()));
            }
            "view" => s.view = string(v, &here, warns).filter(|v| !v.trim().is_empty()),
            "columns" => s.columns = strings(v, &here, warns),
            "hidden" => s.hidden = strings(v, &here, warns),
            "readonly" => match v.as_bool() {
                Some(b) => s.readonly = b,
                None => warn(warns, FILE_NAME, &here, not(Msg::WantBool)),
            },
            "sort" => s.sort = read_sort(v, &here, warns),
            "filters" => s.filters = read_filters(v, &here, warns),
            _ => warn(warns, FILE_NAME, &here, Msg::RsnUnknown.text().to_string()),
        }
    }
    s
}

/// 表の並び(`[[...]]`)の各表と、その道筋(`sort[1]` の形。1 始まり)。並びでなければ警告して空。
fn tables<'a>(
    v: &'a toml::Value,
    at: &str,
    warns: &mut Vec<String>,
) -> Vec<(String, &'a toml::Table)> {
    let Some(items) = v.as_array() else {
        warn(warns, FILE_NAME, at, not(Msg::WantTable));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let here = format!("{at}[{}]", i + 1);
        match item.as_table() {
            Some(t) => out.push((here, t)),
            None => warn(warns, FILE_NAME, &here, not(Msg::WantTable)),
        }
    }
    out
}

fn read_sort(v: &toml::Value, at: &str, warns: &mut Vec<String>) -> Vec<(String, Dir)> {
    let mut out = Vec::new();
    for (here, t) in tables(v, at, warns) {
        let mut key = None;
        let mut dir = Some(Dir::Asc);
        for (k, v) in t {
            let item = format!("{here}.{k}");
            match k.as_str() {
                "key" => key = string(v, &item, warns),
                "dir" => {
                    dir = match v.as_str() {
                        Some("asc") => Some(Dir::Asc),
                        Some("desc") => Some(Dir::Desc),
                        _ => {
                            warn(warns, FILE_NAME, &item, not_one_of(&["asc", "desc"]));
                            None
                        }
                    }
                }
                _ => warn(warns, FILE_NAME, &item, Msg::RsnUnknown.text().to_string()),
            }
        }
        match (key.filter(|k| !k.trim().is_empty()), dir) {
            (Some(k), Some(d)) => out.push((k, d)),
            (None, _) => warn(
                warns,
                FILE_NAME,
                &here,
                Msg::RsnSessionNeeds.fill(&[&"key"]),
            ),
            _ => {}
        }
    }
    out
}

/// 絞り込みの種類の名前(`kind`)。
const KINDS: [&str; 5] = ["text", "value", "expr", "cond", "show"];

fn read_filters(v: &toml::Value, at: &str, warns: &mut Vec<String>) -> Vec<Filter> {
    let mut out = Vec::new();
    for (here, t) in tables(v, at, warns) {
        let kind = t.get("kind").and_then(toml::Value::as_str).unwrap_or("");
        if !KINDS.contains(&kind) {
            warn(
                warns,
                FILE_NAME,
                &format!("{here}.kind"),
                not_one_of(&KINDS),
            );
            continue;
        }
        // 種類ごとに書ける項目(ほかは知らない項目)。
        let allowed: &[&str] = match kind {
            "text" => &["kind", "value"],
            "value" => &["kind", "key", "value", "by_keys", "list"],
            "expr" => &["kind", "expr"],
            _ => &["kind", "name"],
        };
        let mut text = |k: &str| -> Option<String> {
            let v = t.get(k)?;
            string(v, &format!("{here}.{k}"), warns)
        };
        let key = text("key");
        let value = text("value");
        let expr = text("expr");
        let name = text("name");
        let mut flag = |k: &str| -> bool {
            match t.get(k) {
                None => false,
                Some(v) => v.as_bool().unwrap_or_else(|| {
                    warn(warns, FILE_NAME, &format!("{here}.{k}"), not(Msg::WantBool));
                    false
                }),
            }
        };
        let by_keys = flag("by_keys");
        let list = flag("list");
        for k in t.keys() {
            if !allowed.contains(&k.as_str()) {
                warn(
                    warns,
                    FILE_NAME,
                    &format!("{here}.{k}"),
                    Msg::RsnUnknown.text().to_string(),
                );
            }
        }
        let need = |what: &str, warns: &mut Vec<String>| {
            warn(warns, FILE_NAME, &here, Msg::RsnSessionNeeds.fill(&[&what]));
        };
        let f = match kind {
            "text" => value.map(Filter::Text).or_else(|| {
                need("value", warns);
                None
            }),
            "value" => {
                if let (Some(key), Some(value)) = (key, value) {
                    Some(Filter::Value {
                        key,
                        value,
                        by_keys,
                        list,
                    })
                } else {
                    need("key, value", warns);
                    None
                }
            }
            "expr" => expr.map(Filter::Expr).or_else(|| {
                need("expr", warns);
                None
            }),
            "cond" => name.map(Filter::Cond).or_else(|| {
                need("name", warns);
                None
            }),
            _ => name.map(Filter::Show).or_else(|| {
                need("name", warns);
                None
            }),
        };
        out.extend(f);
    }
    out
}

/// TOML の鍵の書き方(裸で書けなければ引用符で囲む。`profile::key_line` と同じく TOML の道具に任せる)。
fn key_text(k: &str) -> String {
    let mut one = toml::Table::new();
    one.insert(k.to_string(), toml::Value::Boolean(true));
    let line = toml::to_string(&one).unwrap_or_default();
    line.trim_end()
        .trim_end_matches("true")
        .trim_end()
        .trim_end_matches('=')
        .trim_end()
        .to_string()
}

fn value_text(v: toml::Value) -> String {
    v.to_string()
}

fn str_text(s: &str) -> String {
    value_text(toml::Value::String(s.to_string()))
}

fn strs_text(list: &[String]) -> String {
    value_text(toml::Value::Array(
        list.iter()
            .map(|s| toml::Value::String(s.clone()))
            .collect(),
    ))
}

/// ホームのフォルダの下なら `~/…` で書く(dotfiles で別のマシンに持っていけるように)。
pub fn home_short(p: &Path) -> String {
    if let Some(home) = config::home_dir() {
        if let Ok(rest) = p.strip_prefix(&home) {
            if !rest.as_os_str().is_empty() {
                return format!("~/{}", rest.to_string_lossy());
            }
        }
    }
    p.to_string_lossy().into_owned()
}

/// セッション1つの区画の文(`[sessions.<名前>]` と、その絞り込み・並びの `[[...]]`)。
pub fn block(s: &Session) -> String {
    let head = format!("sessions.{}", key_text(&s.name));
    let mut out = format!("[{head}]\n");
    let paths: Vec<String> = s.folders.iter().map(|p| home_short(p)).collect();
    out.push_str(&format!("folders = {}\n", strs_text(&paths)));
    if let Some(b) = &s.base {
        out.push_str(&format!("base = {}\n", str_text(&home_short(b))));
    }
    if let Some(v) = &s.view {
        out.push_str(&format!("view = {}\n", str_text(v)));
    }
    out.push_str(&format!("columns = {}\n", strs_text(&s.columns)));
    out.push_str(&format!("hidden = {}\n", strs_text(&s.hidden)));
    out.push_str(&format!("readonly = {}\n", s.readonly));
    for f in &s.filters {
        out.push_str(&format!("\n[[{head}.filters]]\n"));
        let line = |k: &str, v: &str| format!("{k} = {}\n", str_text(v));
        match f {
            Filter::Text(v) => {
                out.push_str(&line("kind", "text"));
                out.push_str(&line("value", v));
            }
            Filter::Value {
                key,
                value,
                by_keys,
                list,
            } => {
                out.push_str(&line("kind", "value"));
                out.push_str(&line("key", key));
                out.push_str(&line("value", value));
                if *by_keys {
                    out.push_str("by_keys = true\n");
                }
                if *list {
                    out.push_str("list = true\n");
                }
            }
            Filter::Expr(e) => {
                out.push_str(&line("kind", "expr"));
                out.push_str(&line("expr", e));
            }
            Filter::Cond(n) => {
                out.push_str(&line("kind", "cond"));
                out.push_str(&line("name", n));
            }
            Filter::Show(n) => {
                out.push_str(&line("kind", "show"));
                out.push_str(&line("name", n));
            }
        }
    }
    for (k, d) in &s.sort {
        out.push_str(&format!("\n[[{head}.sort]]\n"));
        out.push_str(&format!("key = {}\n", str_text(k)));
        let dir = match d {
            Dir::Asc => "asc",
            Dir::Desc => "desc",
        };
        out.push_str(&format!("dir = {}\n", str_text(dir)));
    }
    out
}

/// 並びのセッションを sessions.toml の文にする(並びのまま)。
pub fn to_toml(list: &[Session]) -> String {
    list.iter().map(block).collect::<Vec<_>>().join("\n")
}

/// 名前のセッションを置く(同じ名前があれば置き換え、無ければ後ろに足す)。ファイルの文字を区画ごとに扱うので、
/// 手で書いたコメント・空行・ほかのセッションは文字のまま残る。今のファイルが壊れていれば書かない(理由)。
pub fn save(dir: &Path, s: &Session) -> io::Result<()> {
    let text = match std::fs::read_to_string(dir.join(FILE_NAME)) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let new = upsert(&text, s).map_err(io::Error::other)?;
    config::write_atomic(dir, FILE_NAME, new.as_bytes())
}

/// save の本体: 置き換えた文。書き換えた結果を読み直して狙った中身と違えば(手で書いた形を読み違えたとき)、
/// 中身から書き直す(コメントは消えるが、ほかのセッションを壊さない)。
pub fn upsert(text: &str, s: &Session) -> Result<String, String> {
    let (mut want, _) = parse(text)?;
    match want.iter_mut().find(|x| x.name == s.name) {
        Some(x) => *x = s.clone(),
        None => want.push(s.clone()),
    }
    want.sort_by(|a, b| a.name.cmp(&b.name));
    let new = upsert_text(text, s);
    match parse(&new) {
        Ok((got, _)) if got == want => Ok(new),
        _ => Ok(to_toml(&want)),
    }
}

/// 見出しの行で区切った区画(始まり, 終わり)。末尾の空行とコメントは次の区画の前置きにする。
fn sections(lines: &[&str]) -> Vec<(usize, usize)> {
    let heads: Vec<usize> = (0..lines.len())
        .filter(|&i| config::toml_header(lines[i]).is_some())
        .collect();
    let mut out = Vec::new();
    for (k, &start) in heads.iter().enumerate() {
        let mut end = heads.get(k + 1).copied().unwrap_or(lines.len());
        while end > start + 1 {
            let t = lines[end - 1].trim();
            if t.is_empty() || t.starts_with('#') {
                end -= 1;
            } else {
                break;
            }
        }
        out.push((start, end));
    }
    out
}

/// その区画がセッション `name` のものか(区画だけを読んで `sessions.<name>` があるか)。
fn belongs(body: &str, name: &str) -> bool {
    body.parse::<toml::Table>().is_ok_and(|t| {
        t.get("sessions")
            .and_then(toml::Value::as_table)
            .is_some_and(|s| s.contains_key(name))
    })
}

fn upsert_text(text: &str, s: &Session) -> String {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mine: Vec<(usize, usize)> = sections(&lines)
        .into_iter()
        .filter(|(a, b)| belongs(&lines[*a..*b].concat(), &s.name))
        .collect();
    let new = block(s);
    if mine.is_empty() {
        let mut out = text.to_string();
        if !out.is_empty() {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }
        out.push_str(&new);
        return out;
    }
    let mut out = String::new();
    let mut at = 0;
    for (k, (a, b)) in mine.iter().enumerate() {
        out.push_str(&lines[at..*a].concat());
        if k == 0 {
            out.push_str(&new);
        }
        at = *b;
    }
    let rest = lines[at..].concat();
    if !rest.is_empty() && !out.ends_with("\n\n") && !rest.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(&rest);
    out
}

/// `mdgrid --sessions` の出力(cli.session-screen-list): 1つ1行で「名前<TAB>開くフォルダ」(フォルダが2つ以上なら
/// タブでつなぐ。フォルダが無ければ `.base` のパス)。ホームの下は `~/…`。
pub fn list_text(list: &[Session]) -> String {
    let mut out = String::new();
    for s in list {
        let paths: Vec<String> = if s.folders.is_empty() {
            s.base.iter().map(|b| home_short(b)).collect()
        } else {
            s.folders.iter().map(|p| home_short(p)).collect()
        };
        out.push_str(&s.name);
        for p in paths {
            out.push('\t');
            out.push_str(&p);
        }
        out.push('\n');
    }
    out
}

/// 画面から保存するセッションの名前を確かめる(空・タブや改行などの制御の文字は理由)。
pub fn check_name(name: &str) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(Msg::SessionNameEmpty.text().to_string());
    }
    if n.chars().any(char::is_control) {
        return Err(Msg::SessionNameBad.text().to_string());
    }
    Ok(n.to_string())
}

#[cfg(test)]
#[path = "test_session_unit.rs"]
mod tests;
