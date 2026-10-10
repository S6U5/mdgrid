//! mdgrid 独自のビュー(BV-17〜BV-20)の核。画面に依存しない。
//! 形は docs/design.md の「mdgrid のビュー(BV-17〜BV-20)」。
//!
//! 定義は設定の置き場の `views.toml` に、開いた対象の実体のパスごとに置く(ノートのフォルダには書かない)。
//! 読むだけではフォルダもファイルも作らず、壊れた `views.toml` は警告にして書き換えない。
//! 書くときは置き場の中の一時ファイル → 名前の変更で置き換え、ほかの対象のビューはそのまま保つ。
//!
//! `.base` との変換(BV-19): 書き出し(to_base)は文字列を返すだけでファイルを書かない。設定の条件
//! (settings::Cond)は、expr で評価して settings::apply と同じ行が残る式の文字列にする(cond_expr)。
//! 式で同じにできない場合(日時の値は式では秒まで補った形になる、リストの要素をつないだ文字で探す、
//! マップと入れ子のリストは区別できない、列の型が分からない比較)は、近似の注意を落とした説明に出す。
//! 列の型は to_base_typed で渡せる。to_base は比較(Cmp)の値の形を列の型とみなす(数 → 数、日付 → 日付)。
//! グループの「空を隠す」(NV-21)は `.base` の groupBy で表せないので落とす。

use crate::base::{self, Base};
use crate::config;
use crate::expr::{self, Env, Val};
use crate::i18n::Msg;
use crate::newnote::{self, NewNote};
use crate::settings::{CmpOp, Cond, Dir, Group, Op, Settings};
use crate::source::{FileInfo, Value};
use crate::types::{self, Kind};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// 置き場の中のファイルの名前。
pub const FILE_NAME: &str = "views.toml";

/// mdgrid のビュー1つ。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct NativeView {
    pub name: String,
    /// 列の並び(空なら既定の並び)。
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default)]
    pub hidden: Vec<String>,
    /// `.base` と同じ書き方の式の絞り込み(全部を満たす行だけ。`.base` から取り込んだ filters もここ)。
    #[serde(default)]
    pub filters_expr: Vec<String>,
    /// NV-13〜NV-22 のフィルター・並べ替え・グループ。省くと既定。
    #[serde(default, skip_serializing_if = "Settings::is_default")]
    pub settings: Settings,
    /// CE-26・CE-27: このビューで新しいノートを作るときの決まり(`[target.view.new_note]`)。
    /// 無ければ設定の `[new_note]`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_note: Option<NewNote>,
}

const TARGET_KEYS: &[&str] = &[
    "path",
    "view",
    "default_view",
    "tab_order",
    "hidden_tabs",
    "tab_hint",
];
const VIEW_KEYS: &[&str] = &[
    "name",
    "order",
    "hidden",
    "filters_expr",
    "settings",
    "new_note",
];
const NEW_NOTE_KEYS: &[&str] = NewNote::KEYS;
const SETTINGS_KEYS: &[&str] = &["filters", "sorts", "group", "display", "tree", "wbs"];
const COND_KEYS: &[&str] = &["col", "op"];

// ---- 読み書き ----

/// TOML を表として読む。壊れていれば理由1行。
fn parse_table(text: &str) -> Result<toml::Table, String> {
    text.parse::<toml::Table>()
        .map_err(|e: toml::de::Error| match config::toml_error(text, &e) {
            (Some(line), msg) => Msg::ViewsTomlLine.fill(&[&FILE_NAME, &line, &msg]),
            (None, msg) => Msg::ViewsToml.fill(&[&FILE_NAME, &msg]),
        })
}

/// 書かれた対象のパスが、開いた対象(実体のパス)と同じか。
fn same_target(stored: &str, key: &str) -> bool {
    stored == key || config::target_key(Path::new(stored)) == key
}

fn unknown(path: &str, warns: &mut Vec<String>) {
    warns.push(Msg::ViewsUnknownItem.fill(&[&FILE_NAME, &path]));
}

fn check_keys(t: &toml::Table, allowed: &[&str], prefix: &str, warns: &mut Vec<String>) {
    for k in t.keys() {
        if !allowed.contains(&k.as_str()) {
            unknown(&format!("{prefix}.{k}"), warns);
        }
    }
}

/// ビューの表の知らない項目を警告にする(settings と、その filters の条件まで見る)。
fn check_view(v: &toml::Table, warns: &mut Vec<String>) {
    check_keys(v, VIEW_KEYS, "target.view", warns);
    if let Some(n) = v.get("new_note").and_then(|n| n.as_table()) {
        check_keys(n, NEW_NOTE_KEYS, "target.view.new_note", warns);
        let mut seen: Vec<&str> = Vec::new();
        for c in n
            .get("ask")
            .and_then(|a| a.as_array())
            .into_iter()
            .flatten()
            .filter_map(|c| c.as_str())
        {
            if newnote::not_a_key(c) || seen.contains(&c) {
                warns.push(Msg::ViewsBadAsk.fill(&[&FILE_NAME, &c]));
            } else {
                seen.push(c);
            }
        }
        for c in n
            .get("set")
            .and_then(|s| s.as_table())
            .into_iter()
            .flat_map(|s| s.keys())
        {
            if newnote::not_a_key(c) {
                warns.push(Msg::ViewsBadSet.fill(&[&FILE_NAME, c]));
            }
        }
    }
    let Some(s) = v.get("settings").and_then(|s| s.as_table()) else {
        return;
    };
    check_keys(s, SETTINGS_KEYS, "target.view.settings", warns);
    if let Some(d) = s.get("display").and_then(|d| d.as_table()) {
        check_keys(
            d,
            crate::display::OVERRIDE_KEYS,
            "target.view.settings.display",
            warns,
        );
    }
    if let Some(fs) = s.get("filters").and_then(|f| f.as_array()) {
        for c in fs.iter().filter_map(|c| c.as_table()) {
            check_keys(c, COND_KEYS, "target.view.settings.filters", warns);
        }
    }
}

/// views.toml を読む。対象(実体のパス)の分だけ返す。知らない項目・壊れたファイルは警告の文にして返し、
/// 止めない(BV-20・CLI-3)。ファイルが無ければ空で警告なし。読むだけでは何も作らない。
pub fn load_views(dir: &Path, target: &Path) -> (Vec<NativeView>, Vec<String>) {
    let path = dir.join(FILE_NAME);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return (Vec::new(), Vec::new()),
        Err(e) => {
            return (
                Vec::new(),
                vec![Msg::ViewsToml.fill(&[&path.display(), &config::squash_ws(&e.to_string())])],
            )
        }
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return (Vec::new(), vec![Msg::ViewsNotUtf8.fill(&[&FILE_NAME])]);
    };
    let table = match parse_table(&text) {
        Ok(t) => t,
        Err(msg) => return (Vec::new(), vec![msg]),
    };

    let key = config::target_key(target);
    let mut warns = Vec::new();
    let mut views = Vec::new();
    for k in table.keys() {
        if k != "target" {
            unknown(k, &mut warns);
        }
    }
    let targets: &[toml::Value] = match table.get("target") {
        None => &[],
        Some(toml::Value::Array(a)) => a,
        Some(_) => {
            warns.push(Msg::ViewsTargetNotArray.fill(&[&FILE_NAME]));
            &[]
        }
    };
    for (i, t) in targets.iter().enumerate() {
        let Some(t) = t.as_table() else {
            warns.push(Msg::ViewsTargetNotTable.fill(&[&FILE_NAME, &(i + 1)]));
            continue;
        };
        check_keys(t, TARGET_KEYS, "target", &mut warns);
        let Some(p) = t.get("path").and_then(|p| p.as_str()) else {
            warns.push(Msg::ViewsTargetNoPath.fill(&[&FILE_NAME, &(i + 1)]));
            continue;
        };
        let mine = same_target(p, &key);
        let vs: &[toml::Value] = match t.get("view") {
            None => &[],
            Some(toml::Value::Array(a)) => a,
            Some(_) => {
                warns.push(Msg::ViewsViewNotArray.fill(&[&FILE_NAME, &p]));
                &[]
            }
        };
        for (j, v) in vs.iter().enumerate() {
            let Some(vt) = v.as_table() else {
                warns.push(Msg::ViewsViewNotTable.fill(&[&FILE_NAME, &p, &(j + 1)]));
                continue;
            };
            check_view(vt, &mut warns);
            if !mine {
                continue;
            }
            // new_note は TOML の日付を保つため、表から直に読み直す(serde では文字列に崩れる)。
            let read = NativeView::deserialize(v.clone())
                .map_err(|e| e.to_string())
                .and_then(|mut nv| {
                    if let Some(n) = vt.get("new_note") {
                        nv.new_note = Some(NewNote::from_toml(n)?);
                    }
                    Ok(nv)
                });
            match read {
                Ok(nv) => views.push(nv),
                Err(e) => warns.push(Msg::ViewsViewUnreadable.fill(&[
                    &FILE_NAME,
                    &p,
                    &(j + 1),
                    &config::squash_ws(&e.to_string()),
                ])),
            }
        }
    }
    (views, warns)
}

/// 対象 `target` の表(`[[target]]`)。無ければ・読めなければ None。
fn target_table(dir: &Path, target: &Path) -> Option<toml::Table> {
    let text = std::fs::read_to_string(dir.join(FILE_NAME)).ok()?;
    let table = parse_table(&text).ok()?;
    let key = config::target_key(target);
    table
        .get("target")?
        .as_array()?
        .iter()
        .filter_map(|t| t.as_table())
        .find(|t| {
            t.get("path")
                .and_then(|p| p.as_str())
                .is_some_and(|p| same_target(p, &key))
        })
        .cloned()
}

/// NV-25: 対象 `target` の既定のビューの名前(`[[target]]` の `default_view`)。無ければ・読めなければ None。
pub fn load_default_view(dir: &Path, target: &Path) -> Option<String> {
    target_table(dir, target)?
        .get("default_view")?
        .as_str()
        .map(str::to_string)
}

/// NV-25: 対象 `target` の既定のビューを書く(None なら消す)。ほかの項目と対象はそのまま。
/// 置き場の中の一時ファイル → 名前の変更で書く。壊れた views.toml は書き換えずに Err。
pub fn save_default_view(dir: &Path, target: &Path, name: Option<&str>) -> io::Result<()> {
    edit_target(dir, target, name.is_some(), |t| match name {
        Some(n) => {
            t.insert("default_view".into(), toml::Value::String(n.to_string()));
        }
        None => {
            t.remove("default_view");
        }
    })
}

/// NV-26: 対象のタブの好み(`[[target]]` の `tab_order`・`hidden_tabs`・`tab_hint`)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabPrefs {
    /// タブを見せる順(ビューの名前)。ここに無いタブは元の順で後ろ。
    pub order: Vec<String>,
    /// タブの行と `[` `]` に出さないビューの名前。
    pub hidden: Vec<String>,
    /// タブの行の切り替えの案内(「[ ] で切り替え」)を出すか。
    pub hint: bool,
}

impl Default for TabPrefs {
    fn default() -> Self {
        TabPrefs {
            order: Vec::new(),
            hidden: Vec::new(),
            hint: true,
        }
    }
}

fn names_of(v: Option<&toml::Value>) -> Vec<String> {
    v.and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// NV-26: 対象 `target` のタブの好み。無ければ・読めなければ既定(元の順・隠さない・案内あり)。
pub fn load_tab_prefs(dir: &Path, target: &Path) -> TabPrefs {
    let Some(t) = target_table(dir, target) else {
        return TabPrefs::default();
    };
    TabPrefs {
        order: names_of(t.get("tab_order")),
        hidden: names_of(t.get("hidden_tabs")),
        hint: t.get("tab_hint").and_then(|v| v.as_bool()).unwrap_or(true),
    }
}

/// NV-26: 対象 `target` のタブの好みを書く。既定の項目は書かずに消す。ほかの項目と対象はそのまま。
pub fn save_tab_prefs(dir: &Path, target: &Path, p: &TabPrefs) -> io::Result<()> {
    let names =
        |v: &[String]| toml::Value::Array(v.iter().cloned().map(toml::Value::String).collect());
    edit_target(dir, target, *p != TabPrefs::default(), |t| {
        for (k, v) in [
            ("tab_order", (!p.order.is_empty()).then(|| names(&p.order))),
            (
                "hidden_tabs",
                (!p.hidden.is_empty()).then(|| names(&p.hidden)),
            ),
            ("tab_hint", (!p.hint).then_some(toml::Value::Boolean(false))),
        ] {
            match v {
                Some(v) => {
                    t.insert(k.into(), v);
                }
                None => {
                    t.remove(k);
                }
            }
        }
    })
}

/// 対象 `target` の表を `f` で書き換えて書く(表が無ければ、`create` のときだけ作る)。ほかの対象はそのまま。
/// 置き場の中の一時ファイル → 名前の変更で書く。壊れた views.toml は書き換えずに Err。
fn edit_target(
    dir: &Path,
    target: &Path,
    create: bool,
    f: impl FnOnce(&mut toml::Table),
) -> io::Result<()> {
    let path = dir.join(FILE_NAME);
    let key = config::target_key(target);
    let mut table = match std::fs::read(&path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| invalid(Msg::ViewsSaveNotUtf8.fill(&[&FILE_NAME])))?;
            parse_table(&text).map_err(|e| invalid(Msg::ViewsSaveBroken.fill(&[&e])))?
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => toml::Table::new(),
        Err(e) => return Err(e),
    };
    let targets = table
        .entry("target")
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    let toml::Value::Array(targets) = targets else {
        return Err(invalid(Msg::ViewsSaveTargetNotArray.fill(&[&FILE_NAME])));
    };
    let pos = targets.iter().position(|t| {
        t.get("path")
            .and_then(|p| p.as_str())
            .is_some_and(|p| same_target(p, &key))
    });
    let t = match pos {
        Some(i) => &mut targets[i],
        None => {
            if !create {
                return Ok(());
            }
            let mut t = toml::Table::new();
            t.insert("path".into(), toml::Value::String(key.clone()));
            targets.push(toml::Value::Table(t));
            targets.last_mut().expect("just pushed")
        }
    };
    if let toml::Value::Table(t) = t {
        f(t);
    }
    let text = toml::to_string(&table).map_err(|e| invalid(e.to_string()))?;
    config::write_atomic(dir, FILE_NAME, text.as_bytes())
}

fn invalid(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

/// 前のビューの表の知らない項目(ビューと settings の段)を、同じ名前の新しいビューの表に移す。
fn keep_unknown(new: &mut toml::Table, old: &toml::Table) {
    for (k, v) in old {
        if !VIEW_KEYS.contains(&k.as_str()) {
            new.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    keep_unknown_in(new, old, "settings", SETTINGS_KEYS, true);
    // new_note は新しいビューにあるときだけ移す(無い表を作ると「設定の代わりに空の決まり」に変わるため)。
    keep_unknown_in(new, old, "new_note", NEW_NOTE_KEYS, false);
}

/// 前のビューの表 `key` の知らない項目を、新しいビューの同じ表に移す。`create` なら表が無くても作る。
fn keep_unknown_in(
    new: &mut toml::Table,
    old: &toml::Table,
    key: &str,
    allowed: &[&str],
    create: bool,
) {
    let Some(os) = old.get(key).and_then(|s| s.as_table()) else {
        return;
    };
    let unknown: Vec<(&String, &toml::Value)> = os
        .iter()
        .filter(|(k, _)| !allowed.contains(&k.as_str()))
        .collect();
    if unknown.is_empty() || (!create && !new.contains_key(key)) {
        return;
    }
    let ns = new
        .entry(key.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    if let toml::Value::Table(ns) = ns {
        for (k, v) in unknown {
            ns.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
}

/// 対象のビューを全部書き直す。ほかの対象のビューは保つ。同じ対象の知らない項目と、同じ名前のビューの
/// 知らない項目と、読めなかったビュー(同じ名前の新しいビューが無い限り)も保つ。TOML のコメントと書式は
/// 保てない(表として読んで書き直すため)。置き場(dir)の中の一時ファイル → 名前の変更で書く。
/// 壊れた views.toml は書き換えずに Err。
pub fn save_views(dir: &Path, target: &Path, views: &[NativeView]) -> io::Result<()> {
    let path = dir.join(FILE_NAME);
    let key = config::target_key(target);
    let mut table = match std::fs::read(&path) {
        Ok(bytes) => {
            let text = String::from_utf8(bytes)
                .map_err(|_| invalid(Msg::ViewsSaveNotUtf8.fill(&[&FILE_NAME])))?;
            parse_table(&text).map_err(|e| invalid(Msg::ViewsSaveBroken.fill(&[&e])))?
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => toml::Table::new(),
        Err(e) => return Err(e),
    };
    let targets = match table.remove("target") {
        None => Vec::new(),
        Some(toml::Value::Array(a)) => a,
        Some(_) => return Err(invalid(Msg::ViewsSaveTargetNotArray.fill(&[&FILE_NAME]))),
    };
    // 同じ対象の表は最初の位置に1つにまとめ、ほかの対象の表は順も中身もそのまま。
    // 同じ対象の古い表からは、知らない項目と、読めなかったビュー(同じ名前で上書きされない限り)を引き継ぐ。
    let mut out = Vec::with_capacity(targets.len() + 1);
    let mut pos = None;
    let mut extra = toml::Table::new();
    let mut old_views: Vec<toml::Table> = Vec::new();
    let mut unreadable: Vec<toml::Value> = Vec::new();
    for t in targets {
        let mine = t
            .get("path")
            .and_then(|p| p.as_str())
            .is_some_and(|p| same_target(p, &key));
        if !mine {
            out.push(t);
            continue;
        }
        pos.get_or_insert(out.len());
        let toml::Value::Table(t) = t else {
            continue;
        };
        for (k, v) in t {
            match (k.as_str(), v) {
                ("path", _) => {}
                ("view", toml::Value::Array(vs)) => {
                    for v in vs {
                        match (NativeView::deserialize(v.clone()), v) {
                            (Ok(_), toml::Value::Table(vt)) => old_views.push(vt),
                            (_, v) => unreadable.push(v),
                        }
                    }
                }
                ("view", v) => unreadable.push(v),
                (_, v) => {
                    extra.entry(k).or_insert(v);
                }
            }
        }
    }
    let mut arr = Vec::with_capacity(views.len() + unreadable.len());
    let mut used = vec![false; old_views.len()];
    for nv in views {
        let mut t = match toml::Value::try_from(nv) {
            Ok(toml::Value::Table(t)) => t,
            Ok(_) => return Err(io::Error::other(Msg::ViewsNotATable.text())),
            Err(e) => return Err(io::Error::other(config::squash_ws(&e.to_string()))),
        };
        // try_from は日付を内部の表にしてしまうので、new_note は日付の値を保つ形で置き直す。
        if let Some(n) = &nv.new_note {
            t.insert("new_note".to_string(), n.to_toml());
        }
        let old = old_views.iter().enumerate().position(|(i, o)| {
            !used[i] && o.get("name").and_then(|n| n.as_str()) == Some(nv.name.as_str())
        });
        if let Some(i) = old {
            used[i] = true;
            keep_unknown(&mut t, &old_views[i]);
        }
        arr.push(toml::Value::Table(t));
    }
    for u in unreadable {
        let name = u.get("name").and_then(|n| n.as_str());
        if !name.is_some_and(|n| views.iter().any(|v| v.name == n)) {
            arr.push(u);
        }
    }
    if !arr.is_empty() || !extra.is_empty() {
        let mut t = extra;
        t.insert("path".to_string(), toml::Value::String(key.clone()));
        if !arr.is_empty() {
            t.insert("view".to_string(), toml::Value::Array(arr));
        }
        out.insert(pos.unwrap_or(out.len()), toml::Value::Table(t));
    }
    if !out.is_empty() {
        table.insert("target".to_string(), toml::Value::Array(out));
    }
    let text =
        toml::to_string(&table).map_err(|e| io::Error::other(config::squash_ws(&e.to_string())))?;
    config::write_atomic(dir, FILE_NAME, text.as_bytes())
}

// ---- 式の文字列 ----

/// expr と Obsidian の文字列の式(`"…"`)。
fn str_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn is_simple_ident(s: &str) -> bool {
    let mut cs = s.chars();
    cs.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 素の名前にすると別の意味になる名前。
const RESERVED: &[&str] = &[
    "true", "false", "null", "note", "file", "formula", "this", "if", "date", "now", "today",
    "duration",
];

/// 列の参照。`file.x`・`formula.x` はそのまま、ASCII の名前は素のまま、ほかは `note["…"]`。
fn col_ref(col: &str) -> String {
    for p in ["file.", "formula."] {
        if col.strip_prefix(p).is_some_and(is_simple_ident) {
            return col.to_string();
        }
    }
    if is_simple_ident(col) && !RESERVED.contains(&col) {
        col.to_string()
    } else {
        format!("note[{}]", str_lit(col))
    }
}

/// リストか(文字列・日付は toString が自分と等しく、数・真偽・Null は length が無い)。
fn list_expr(v: &str) -> String {
    format!("({v}.toString() != {v} && {v}.length >= 0)")
}

/// 空(settings の value_keys が「(空)」だけ): Null・空の文字列・空のリスト・空の要素だけのリスト。
/// 要素が n 個のリストの toString は要素の文字を `, ` でつないだもので、長さが 2n - 2 なら要素の文字は全部空。
/// マップ(式では Null)と、空の入れ子のリストの要素は、式では空と区別できない(empty_note)。
fn empty_expr(v: &str) -> String {
    format!(
        "({v}.isEmpty() || ({v}.toString() != {v} && {v}.toString().length == {v}.length * 2 - 2))"
    )
}

fn empty_note(col: &str) -> String {
    Msg::ExportEmptyNote.fill(&[&col])
}

fn approx(col: &str, what: &str) -> String {
    Msg::ExportApprox.fill(&[&col, &what])
}

/// 日時・日付として読める文字か(expr の date() が値を返す)。
fn when_like(k: &str) -> bool {
    let Ok(e) = expr::parse(&format!("date({})", str_lit(k))) else {
        return false;
    };
    let file = FileInfo {
        name: String::new(),
        basename: String::new(),
        ext: String::new(),
        path: String::new(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    };
    let prop = |_: &str| None;
    let formula = |_: &str| None;
    let env = Env {
        prop: &prop,
        file: &file,
        formula: &formula,
        today: 0,
        now: 0,
    };
    !matches!(expr::eval(&e, &env), Val::Null)
}

/// 日時の値の文字(式では秒まで補った `YYYY-MM-DDTHH:MM:SS` になる)だけでできた文字か。
fn datetime_chars(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || "-:tTzZ.".contains(c) || c.is_whitespace())
}

/// リストの要素と比べる値の候補(文字列と、その文字が表す数・真偽)。
fn lit_alts(k: &str) -> Vec<String> {
    let mut out = vec![str_lit(k)];
    if let Ok(f) = k.parse::<f64>() {
        if f.is_finite() && format!("{f}") == k {
            out.push(k.to_string());
        }
    }
    if k == "true" || k == "false" {
        out.push(k.to_string());
    }
    out
}

/// 値の一覧のチェック(Keep)。リストは要素ごとに、スカラーは表示の文字列で、「(空)」は空で当てる。
fn keep_expr(col: &str, v: &str, keys: &[Option<String>], notes: &mut Vec<String>) -> String {
    let vals: Vec<&String> = keys.iter().flatten().collect();
    let mut parts = Vec::new();
    if !vals.is_empty() {
        let list: Vec<String> = vals.iter().map(|k| str_lit(k)).collect();
        let mut scalar = format!("[{}].contains({v}.toString())", list.join(", "));
        for k in &vals {
            // 式は日時の値を秒まで補った形にするので、日付だけでない日時の形の値は時刻として比べる。
            if when_like(k) {
                if types::parse_date(k).is_none() {
                    scalar.push_str(&format!(" || {v} == date({})", str_lit(k)));
                }
                notes.push(approx(col, &Msg::ApproxDateValue.fill(&[k])));
            }
            if k.as_str() == "{…}" {
                notes.push(approx(col, Msg::ApproxMapValue.text()));
            }
        }
        let alts: Vec<String> = vals.iter().flat_map(|k| lit_alts(k)).collect();
        parts.push(format!(
            "if({}, {v}.containsAny({}), {scalar})",
            list_expr(v),
            alts.join(", ")
        ));
    }
    if keys.contains(&None) {
        parts.push(empty_expr(v));
        notes.push(empty_note(col));
    }
    if parts.is_empty() {
        "false".to_string()
    } else {
        parts.join(" || ")
    }
}

/// 含む(大文字小文字を区別しない)。式はリストの要素を `, ` でつないだ文字の中を探す。
fn contains_expr(col: &str, v: &str, needle: &str, notes: &mut Vec<String>) -> String {
    let n = needle.to_lowercase();
    if datetime_chars(&n) {
        notes.push(approx(col, &Msg::ApproxContainsDateTime.fill(&[&needle])));
    }
    if n.contains(',') || n.starts_with(char::is_whitespace) {
        notes.push(approx(col, &Msg::ApproxContainsJoined.fill(&[&needle])));
    }
    if n.contains(['{', '}', '…']) {
        notes.push(approx(col, Msg::ApproxContainsMap.text()));
    }
    format!("{v}.toString().lower().contains({})", str_lit(&n))
}

/// 比較の値の読み方。
enum Operand {
    Num(f64),
    /// date(…) の式。
    When(String),
    Text(String),
    /// 列の型で読めない(settings ではどの行も残らない)。
    None,
}

/// 比べる値を列の型で読む(settings の operand と同じ)。型が分からなければ値の形で決める。
fn operand(s: &str, kind: Option<Kind>) -> Operand {
    let t = s.trim();
    let num = || t.parse::<f64>().ok().filter(|f| !f.is_nan());
    let when = |ok: bool| {
        if ok {
            Operand::When(format!("date({})", str_lit(t)))
        } else {
            Operand::None
        }
    };
    let datetime = || {
        types::parse_date(t).is_some() || types::fits(Kind::DateTime, &Value::Str(t.to_string()))
    };
    match kind {
        Some(Kind::Number) => num().map_or(Operand::None, Operand::Num),
        Some(Kind::Date) => when(types::parse_date(t).is_some()),
        Some(Kind::DateTime) => when(datetime()),
        Some(_) => Operand::Text(str_lit(t)),
        None => match num().filter(|f| f.is_finite()) {
            Some(f) => Operand::Num(f),
            None if datetime() => when(true),
            None => Operand::Text(str_lit(t)),
        },
    }
}

fn cmp_sym(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
    }
}

/// 比較(Cmp)。数・日付は型の合わない値と空が比較で Null になって残らない。≠ は「より小さいか大きい」。
fn cmp_expr(
    col: &str,
    v: &str,
    op: CmpOp,
    s: &str,
    kind: Option<Kind>,
    notes: &mut Vec<String>,
) -> String {
    let ord = |rhs: &str| match op {
        CmpOp::Ne => format!("({v} < {rhs} || {v} > {rhs})"),
        op => format!("{v} {} {rhs}", cmp_sym(op)),
    };
    // 数の値か(Null・文字列・リストは比較が Null)。
    let is_num = format!("({v} >= 0 || {v} < 0)");
    if matches!(kind, Some(Kind::Number | Kind::Date | Kind::DateTime)) {
        notes.push(approx(col, Msg::ApproxCmpList.text()));
    }
    match operand(s, kind) {
        Operand::None => "false".to_string(),
        Operand::Num(f) if f.is_finite() => ord(&format!("{f}")),
        // 無限大は式に書けないので、数かどうかで決める(+∞ より小さい数は全部、大きい数は無い)。
        Operand::Num(f) => {
            let below = f > 0.0;
            match op {
                CmpOp::Ne => is_num,
                CmpOp::Eq => "false".to_string(),
                CmpOp::Lt | CmpOp::Le if below => is_num,
                CmpOp::Gt | CmpOp::Ge if !below => is_num,
                _ => "false".to_string(),
            }
        }
        // 日付の列は日付の値だけ(式は日時の値も読むので、toString の長さで日付だけにする)。
        Operand::When(rhs) => {
            if kind.is_some() {
                notes.push(approx(col, Msg::ApproxCmpDateLoose.text()));
            }
            if kind == Some(Kind::Date) {
                format!("({v}.toString().length == 10 && {})", ord(&rhs))
            } else {
                ord(&rhs)
            }
        }
        Operand::Text(rhs) => {
            if kind.is_none() {
                notes.push(approx(col, &Msg::ApproxCmpUnknownType.fill(&[&s.trim()])));
            }
            if kind == Some(Kind::List) || kind.is_none() {
                notes.push(approx(col, Msg::ApproxCmpListJoined.text()));
            }
            notes.push(approx(col, Msg::ApproxCmpMapEmpty.text()));
            if !s.trim().chars().next().is_some_and(|c| c > '9') {
                notes.push(approx(col, Msg::ApproxCmpDateTimePadded.text()));
            }
            format!(
                "(!{} && {v}.toString() {} {rhs})",
                empty_expr(v),
                cmp_sym(op)
            )
        }
    }
}

/// 設定の条件1つを、expr で評価して settings::matches と同じ行が残る式にする。絞らない条件は None。
/// kind は列の型(分からなければ None。そのとき Cmp は比べる値の形を列の型とみなす: 数の値なら数の列、
/// 日付・日時の値なら日付・日時の列)。式で settings と行がずれうる条件は、その注意を2つめに返す。
pub fn cond_expr(c: &Cond, kind: Option<Kind>) -> (Option<String>, Vec<String>) {
    let v = col_ref(&c.col);
    let col = c.col.as_str();
    let mut notes = Vec::new();
    let e = match &c.op {
        Op::Keep(keys) => Some(keep_expr(col, &v, keys, &mut notes)),
        Op::Drop(keys) if keys.is_empty() => None,
        Op::Drop(keys) => Some(format!("!({})", keep_expr(col, &v, keys, &mut notes))),
        Op::Contains(s) | Op::NotContains(s) if s.is_empty() => {
            notes.push(empty_note(col));
            let e = empty_expr(&v);
            Some(if matches!(c.op, Op::Contains(_)) {
                format!("!{e}")
            } else {
                e
            })
        }
        Op::Contains(s) => Some(contains_expr(col, &v, s, &mut notes)),
        Op::NotContains(s) => Some(format!("!{}", contains_expr(col, &v, s, &mut notes))),
        Op::Cmp(op, s) => Some(cmp_expr(col, &v, *op, s, kind, &mut notes)),
        Op::Empty | Op::NotEmpty => {
            notes.push(empty_note(col));
            let e = empty_expr(&v);
            Some(if matches!(c.op, Op::NotEmpty) {
                format!("!{e}")
            } else {
                e
            })
        }
    };
    (e, notes)
}

// ---- `.base` との変換 ----

/// YAML の文字列のスカラー。印字できる文字だけなら単一引用符、ほかは二重引用符で逃がす。
fn yq(s: &str) -> String {
    let plain = |c: char| {
        !(c.is_control() || c == '\u{2028}' || c == '\u{2029}' || c == '\u{feff}' || c == '\u{85}')
    };
    if s.chars().all(plain) {
        return format!("'{}'", s.replace('\'', "''"));
    }
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if !plain(c) => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn dir_text(d: Dir) -> &'static str {
    match d {
        Dir::Asc => "ASC",
        Dir::Desc => "DESC",
    }
}

/// BV-19: `.base` の YAML の文字列にする(ファイルは書かない)。表せない部分は落として、その説明を返す。
/// 列の型を知らないので、比較(Cmp)は比べる値の形を列の型とみなす(to_base_typed の kind が全部 None)。
pub fn to_base(view: &NativeView) -> (String, Vec<String>) {
    to_base_typed(view, &|_| None)
}

/// to_base に列の型(分からなければ None)を渡すもの。型が分かれば比較(Cmp)を settings と同じ読み方にする。
/// 落とした部分と、settings と行がずれうる条件(近似)の説明を返す。
pub fn to_base_typed(
    view: &NativeView,
    kind: &dyn Fn(&str) -> Option<Kind>,
) -> (String, Vec<String>) {
    let mut dropped = Vec::new();
    let mut filters: Vec<String> = view
        .filters_expr
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();
    for c in &view.settings.filters {
        let (e, notes) = cond_expr(c, kind(&c.col));
        filters.extend(e);
        for n in notes {
            if !dropped.contains(&n) {
                dropped.push(n);
            }
        }
    }

    // formula の参照は残す(式の定義を持たないので、Obsidian でも mdgrid でも評価できない)。
    let mut refs: Vec<String> = Vec::new();
    let cols = view
        .order
        .iter()
        .chain(&view.hidden)
        .chain(view.settings.sorts.iter().map(|(c, _)| c))
        .chain(view.settings.filters.iter().map(|c| &c.col))
        .chain(match &view.settings.group {
            Group::By { col, .. } => Some(col),
            _ => None,
        });
    for c in cols {
        if let Some(n) = c.strip_prefix("formula.") {
            refs.push(n.to_string());
        }
    }
    for e in &view.filters_expr {
        refs.extend(base::formula_refs(e));
    }
    refs.sort();
    refs.dedup();
    if !refs.is_empty() {
        let sep = format!("{}formula.", Msg::ListSep.text());
        dropped.push(Msg::ExportFormulaRefs.fill(&[&refs.join(&sep)]));
    }

    let columns: Vec<&String> = view
        .order
        .iter()
        .filter(|c| !view.hidden.contains(c))
        .collect();
    if view.order.is_empty() && !view.hidden.is_empty() {
        dropped.push(Msg::ExportHiddenDropped.fill(&[&view.hidden.join(", ")]));
    }

    let mut y = String::from("views:\n  - type: table\n");
    y.push_str(&format!("    name: {}\n", yq(&view.name)));
    if !filters.is_empty() {
        y.push_str("    filters:\n      and:\n");
        for f in &filters {
            y.push_str(&format!("        - {}\n", yq(f)));
        }
    }
    if !columns.is_empty() {
        y.push_str("    order:\n");
        for c in &columns {
            y.push_str(&format!("      - {}\n", yq(c)));
        }
    }
    if !view.settings.sorts.is_empty() {
        y.push_str("    sort:\n");
        for (c, d) in &view.settings.sorts {
            y.push_str(&format!(
                "      - property: {}\n        direction: {}\n",
                yq(c),
                dir_text(*d)
            ));
        }
    }
    if let Group::By {
        col,
        dir,
        hide_empty,
    } = &view.settings.group
    {
        y.push_str(&format!(
            "    groupBy:\n      property: {}\n      direction: {}\n",
            yq(col),
            dir_text(*dir)
        ));
        if *hide_empty {
            dropped.push(Msg::ExportHideEmptyDropped.fill(&[col]));
        }
    }
    // SR-20: 表示の切り替えは .base に無い(落としたものの一覧の最後)。
    if !view.settings.display.is_empty() {
        dropped.push(Msg::ExportDisplayDropped.text().to_string());
    }
    (y, dropped)
}

fn dir_of(d: base::Dir) -> Dir {
    match d {
        base::Dir::Asc => Dir::Asc,
        base::Dir::Desc => Dir::Desc,
    }
}

/// BV-19: `.base` のビュー(全体の filters と合わせる)を取り込む。filters は式のまま filters_expr に、
/// order・sort・groupBy は対応する欄に入れる。解釈できない部分・欄の無い部分は落として説明を返す。
pub fn from_base(b: &Base, view: usize) -> (NativeView, Vec<String>) {
    let Some(v) = b.views.get(view) else {
        return (
            NativeView::default(),
            vec![Msg::ImportNoView.fill(&[&view])],
        );
    };
    let mut dropped = Vec::new();
    let mut filters_expr = b.filter_exprs(&mut dropped);
    filters_expr.extend(v.filter_exprs(&mut dropped));
    match v.kind.as_str() {
        "table" => {}
        "" => dropped.push(Msg::ImportNoType.text().to_string()),
        k => dropped.push(Msg::ImportTypeAsTable.fill(&[&k])),
    }
    if let Some(n) = v.limit {
        dropped.push(Msg::ImportLimitDropped.fill(&[&n]));
    }
    let formulas = b.formula_names();
    if !formulas.is_empty() {
        dropped.push(Msg::ImportFormulasDropped.fill(&[&formulas.join(", ")]));
    }
    let display = b.display_names();
    if !display.is_empty() {
        dropped.push(Msg::ImportDisplayNameDropped.fill(&[&display.join(", ")]));
    }
    let settings = Settings {
        filters: Vec::new(),
        sorts: v
            .sort
            .iter()
            .map(|(c, d)| (c.clone(), dir_of(*d)))
            .collect(),
        group: match &v.group_by {
            Some((col, d)) => Group::By {
                col: col.clone(),
                dir: dir_of(*d),
                hide_empty: false,
            },
            None => Group::Inherit,
        },
        ..Default::default()
    };
    (
        NativeView {
            name: v.name.clone(),
            order: v.order.clone(),
            hidden: Vec::new(),
            filters_expr,
            settings,
            new_note: None,
        },
        dropped,
    )
}

#[cfg(test)]
#[path = "test_views_unit.rs"]
mod tests;
