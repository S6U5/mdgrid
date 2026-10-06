//! 表から新しいノートを作る(new-note。CE-25・CE-26・CE-27)の核。画面に依存しない。
//!
//! 作る場所と名前の検査(`note_path`)、名前の雛形(`expand_name`)、ビューの絞り込みのうち値が1つに
//! 決まるものを前もって入れる編集にする(`prefill`)、中身を作る(`build`。`writeback::apply(b"", …)` で
//! フロントマターだけを作る。add-frontmatter と同じ書き方)、新しいファイルだけを書く(`create`。
//! `create_new` なので既にあるファイルは変えない。WB-2)。

use crate::config::Config;
use crate::expr::{self, Fixed, Val};
use crate::i18n::Msg;
use crate::settings::{CmpOp, Op, Settings};
use crate::types::{self, Kind};
use crate::views::NativeView;
use crate::writeback::{self, Edit, NewValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 新しいノートの決まり(設定の `[new_note]`、ビューの `[target.view.new_note]`)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewNote {
    /// 既定のフォルダ(開いたフォルダからの相対。空 = 開いたフォルダ)。
    pub folder: String,
    /// 名前の雛形(`{date}` = 今日)。空が既定。
    pub name: String,
    /// 名前のあとに聞く列の並び。
    pub ask: Vec<String>,
    /// 前もって入れる値(`[new_note.set]` の 列 = 値)。
    pub set: Vec<(String, NewValue)>,
}

/// TOML の値 → 入れる値。文字列・整数・小数・真偽・日付(TOML の日付と日時。時差つきは断る)・
/// 文字列か数か真偽の並び(リスト)。表と入れ子の並びは None。
pub(crate) fn value_from_toml(v: &toml::Value) -> Option<NewValue> {
    Some(match v {
        toml::Value::String(s) => NewValue::Str(s.clone()),
        toml::Value::Integer(n) => NewValue::Int(*n),
        toml::Value::Float(f) => NewValue::Float(*f),
        toml::Value::Boolean(b) => NewValue::Bool(*b),
        toml::Value::Datetime(d) => {
            if d.date.is_none() || d.offset.is_some() {
                return None;
            }
            NewValue::Date(d.to_string())
        }
        toml::Value::Array(items) => NewValue::List(
            items
                .iter()
                .map(|x| match x {
                    toml::Value::String(s) => Some(s.clone()),
                    toml::Value::Integer(n) => Some(n.to_string()),
                    toml::Value::Float(f) => Some(f.to_string()),
                    toml::Value::Boolean(b) => Some(b.to_string()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
        ),
        toml::Value::Table(_) => return None,
    })
}

/// 入れる値 → TOML の値。Null は TOML に無いので空の並び(どちらも `key:` と書く。CE-19)。
fn value_to_toml(v: &NewValue) -> toml::Value {
    match v {
        NewValue::Null => toml::Value::Array(Vec::new()),
        NewValue::Str(s) => toml::Value::String(s.clone()),
        NewValue::Bool(b) => toml::Value::Boolean(*b),
        NewValue::Int(n) => toml::Value::Integer(*n),
        NewValue::Float(f) => toml::Value::Float(*f),
        NewValue::List(items) => {
            toml::Value::Array(items.iter().cloned().map(toml::Value::String).collect())
        }
        // 秒の無い日時(`…T09:00`)は秒を補って TOML の日時にする。
        NewValue::Date(s) => match s
            .parse::<toml::value::Datetime>()
            .or_else(|_| format!("{s}:00").parse::<toml::value::Datetime>())
        {
            Ok(d) => toml::Value::Datetime(d),
            Err(_) => toml::Value::String(s.clone()),
        },
        // 新しいノートの決まりにキーの名前の変更・削除は無い(CE-29 は表の操作だけ)。
        NewValue::RenameKey(_) | NewValue::DeleteKey => toml::Value::Array(Vec::new()),
    }
}

impl NewNote {
    /// views.toml の表から読む。serde を通すと TOML の日付が文字列に崩れるので、views の読み込みはこちらを
    /// 使う(TOML の値をそのまま value_from_toml に渡す)。書けない列・重なりは捨てる(警告は check_view)。
    pub(crate) fn from_toml(v: &toml::Value) -> Result<NewNote, String> {
        let t = v.as_table().ok_or(Msg::NewNoteNotTable.text())?;
        let text = |k: &str| match t.get(k) {
            None => Ok(String::new()),
            Some(toml::Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(Msg::NewNoteNotString.fill(&[&k])),
        };
        let mut n = NewNote {
            folder: text("folder")?,
            name: text("name")?,
            ..NewNote::default()
        };
        if let Some(a) = t.get("ask") {
            let a = a.as_array().ok_or(Msg::NewNoteAskNotArray.text())?;
            for c in a {
                let c = c.as_str().ok_or(Msg::NewNoteAskNonString.text())?;
                if !not_a_key(c) && !n.ask.iter().any(|x| x == c) {
                    n.ask.push(c.to_string());
                }
            }
        }
        if let Some(s) = t.get("set") {
            let s = s.as_table().ok_or(Msg::NewNoteSetNotTable.text())?;
            for (k, v) in s {
                let v = value_from_toml(v).ok_or_else(|| Msg::NewNoteSetBadValue.fill(&[k]))?;
                if !not_a_key(k) {
                    n.set.push((k.clone(), v));
                }
            }
        }
        Ok(n)
    }

    /// views.toml に書く表(TOML の日付は本物の日付の値のまま)。`toml::Value::try_from` は日付を内部の表に
    /// してしまうので、views の保存はこちらを使う。
    pub(crate) fn to_toml(&self) -> toml::Value {
        let mut t = toml::Table::new();
        if !self.folder.is_empty() {
            t.insert("folder".into(), toml::Value::String(self.folder.clone()));
        }
        if !self.name.is_empty() {
            t.insert("name".into(), toml::Value::String(self.name.clone()));
        }
        if !self.ask.is_empty() {
            let ask = self.ask.iter().cloned().map(toml::Value::String).collect();
            t.insert("ask".into(), toml::Value::Array(ask));
        }
        if !self.set.is_empty() {
            let set = self
                .set
                .iter()
                .map(|(k, v)| (k.clone(), value_to_toml(v)))
                .collect();
            t.insert("set".into(), toml::Value::Table(set));
        }
        toml::Value::Table(t)
    }
}

/// ask・set に書けない列(ノートのキーでない `file.*`・`formula.*` と空)か。
pub(crate) fn not_a_key(col: &str) -> bool {
    col.trim().is_empty() || col.starts_with("file.") || col.starts_with("formula.")
}

/// views.toml の表の形(NewValue に serde が無いので、set は TOML の表で持つ)。
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Raw {
    #[serde(skip_serializing_if = "String::is_empty")]
    folder: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ask: Vec<String>,
    #[serde(skip_serializing_if = "toml::Table::is_empty")]
    set: toml::Table,
}

impl Serialize for NewNote {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        Raw {
            folder: self.folder.clone(),
            name: self.name.clone(),
            ask: self.ask.clone(),
            set: self
                .set
                .iter()
                .map(|(k, v)| (k.clone(), value_to_toml(v)))
                .collect(),
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for NewNote {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Raw::deserialize(d)?;
        let mut set = Vec::with_capacity(raw.set.len());
        for (k, v) in raw.set {
            let v = value_from_toml(&v)
                .ok_or_else(|| serde::de::Error::custom(Msg::NewNoteSetBadValue.fill(&[&k])))?;
            if !not_a_key(&k) {
                set.push((k, v));
            }
        }
        // 書けない列と重なりは捨てる(警告は views の check_view が出す)。
        let mut ask: Vec<String> = Vec::with_capacity(raw.ask.len());
        for c in raw.ask {
            if !not_a_key(&c) && !ask.contains(&c) {
                ask.push(c);
            }
        }
        Ok(NewNote {
            folder: raw.folder,
            name: raw.name,
            ask,
            set,
        })
    }
}

/// ビューに new_note があればそれ、無ければ設定の new_note(CE-26・CE-27)。
pub fn rule_for<'a>(config: &'a Config, view: Option<&'a NativeView>) -> &'a NewNote {
    view.and_then(|v| v.new_note.as_ref())
        .unwrap_or(&config.new_note)
}

/// 名前の雛形の `{date}` を今日(1970-01-01 からの日数)の YYYY-MM-DD にする(CE-27)。
pub fn expand_name(template: &str, today: i64) -> String {
    template.replace("{date}", &types::format_date(today))
}

/// Obsidian がファイルとフォルダの名前に許さない文字(`\` も)。
const BAD_CHARS: &[char] = &['#', '|', '^', '[', ']', ':', '*', '?', '"', '<', '>', '\\'];

/// 相対のパスを `/` で部品に分け、部品ごとに前後の空白を除く。除いたあとの `.` と空の部品は飛ばす。
/// `..`・絶対パス・制御文字・Obsidian が許さない文字・`.` で始まる部品(`.obsidian`・`.git`・`.trash` など
/// 読み込みで飛ばすフォルダを含む)は Err。
fn rel_parts(s: &str, what: Msg) -> Result<Vec<&str>, String> {
    let what = what.text();
    if s.chars().any(char::is_control) {
        return Err(Msg::NewNoteControl.fill(&[&what]));
    }
    if s.trim_start().starts_with('/') || Path::new(s.trim()).is_absolute() {
        return Err(Msg::NewNoteAbsolute.fill(&[&what]));
    }
    if let Some(c) = s.chars().find(|c| BAD_CHARS.contains(c)) {
        return Err(Msg::NewNoteBadChar.fill(&[&what, &c]));
    }
    let mut parts = Vec::new();
    for p in s.split('/').map(str::trim) {
        match p {
            "" | "." => {}
            ".." => return Err(Msg::NewNoteDotDot.fill(&[&what])),
            p if p.starts_with('.') => return Err(Msg::NewNoteDotPart.fill(&[&what, &p])),
            p => parts.push(p),
        }
    }
    Ok(parts)
}

/// 末尾の `.md`(大文字小文字を区別しない)を除く。
fn strip_md(s: &str) -> &str {
    let n = s.len();
    if n >= 3 && s.is_char_boundary(n - 3) && s[n - 3..].eq_ignore_ascii_case(".md") {
        &s[..n - 3]
    } else {
        s
    }
}

/// 作る場所の検査(CE-25)。root/folder/name に `.md` を補ったパス。`..` で外に出る・絶対パス・空・
/// 制御文字・既にある → Err(理由)。名前の前後の空白は除く。フォルダは作らない。
pub fn note_path(root: &Path, folder: &str, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Msg::NewNoteEmptyName.text().to_string());
    }
    let folder_parts = rel_parts(folder, Msg::NewNoteWhatFolder)?;
    let mut name_parts = rel_parts(name, Msg::NewNoteWhatName)?;
    let Some(last) = name_parts.pop() else {
        return Err(Msg::NewNoteEmptyName.text().to_string());
    };
    if name.trim_end().ends_with('/') {
        return Err(Msg::NewNoteNameIsFolder.text().to_string());
    }
    let stem = strip_md(last).trim();
    if stem.is_empty() {
        return Err(Msg::NewNoteEmptyName.text().to_string());
    }
    let mut path = root.to_path_buf();
    for p in folder_parts.iter().chain(name_parts.iter()) {
        path.push(p);
    }
    path.push(format!("{stem}.md"));
    if std::fs::symlink_metadata(&path).is_ok() {
        return Err(Msg::NewNoteExists.fill(&[&path.display()]));
    }
    Ok(path)
}

/// 文字の値を列の型の値にする。読めなければ None。
fn typed(kind: Kind, s: &str) -> Option<NewValue> {
    match kind {
        Kind::Text => (!s.is_empty()).then(|| NewValue::Str(s.to_string())),
        Kind::Number => {
            let t = s.trim();
            if let Ok(n) = t.parse::<i64>() {
                Some(NewValue::Int(n))
            } else {
                t.parse::<f64>()
                    .ok()
                    .filter(|f| f.is_finite())
                    .map(NewValue::Float)
            }
        }
        Kind::Checkbox => match s.trim() {
            "true" => Some(NewValue::Bool(true)),
            "false" => Some(NewValue::Bool(false)),
            _ => None,
        },
        Kind::Date => types::parse_date(s.trim()).map(|_| NewValue::Date(s.trim().to_string())),
        Kind::DateTime => {
            let t = s.trim();
            (!t.is_empty()
                && types::fits(Kind::DateTime, &crate::frontmatter::Value::Str(t.into())))
            .then(|| NewValue::Date(t.to_string()))
        }
        Kind::List => (!s.is_empty()).then(|| NewValue::List(vec![s.to_string()])),
    }
}

/// 式の `==` の値。式は型を厳密に比べるので、書いてある型のまま(文字列は Str)にする。日付・日時の列の
/// 文字列だけは日付として書く(読むと日付になるため)。リストの列は `==` では1つに決まらない。
fn typed_val(kind: Kind, v: &Val) -> Option<NewValue> {
    match (kind, v) {
        (Kind::List, _) => None,
        (Kind::Date | Kind::DateTime, Val::Str(s)) => typed(kind, s),
        (_, Val::Str(s)) => (!s.is_empty()).then(|| NewValue::Str(s.clone())),
        (_, Val::Bool(b)) => Some(NewValue::Bool(*b)),
        (_, Val::Num(n)) if n.is_finite() => {
            if n.fract() == 0.0 && n.abs() < 9.0e15 {
                Some(NewValue::Int(*n as i64))
            } else {
                Some(NewValue::Float(*n))
            }
        }
        _ => None,
    }
}

/// 前もって入れる編集に足す。同じ列が既にあれば、どちらもリストなら要素を合わせ、そうでなければ先のまま。
fn push_prefill(out: &mut Vec<Edit>, key: &str, value: NewValue) {
    if not_a_key(key) {
        return;
    }
    match out.iter_mut().find(|e| e.key == key) {
        Some(Edit {
            value: NewValue::List(have),
            ..
        }) => {
            if let NewValue::List(add) = value {
                for x in add {
                    if !have.contains(&x) {
                        have.push(x);
                    }
                }
            }
        }
        Some(_) => {}
        None => out.push(Edit {
            key: key.to_string(),
            value,
        }),
    }
}

/// 絞り込み(設定の条件と式の文字列)のうち値が1つに決まるものを、前もって入れる編集にする(CE-25)。
/// 設定の条件: Keep の値が1つ(リストの列はその要素1つのリスト)・Cmp の `==`。式: `&&` でつないだ項の
/// `列 == 値`(書いてある型のまま。日付・日時の列は日付)・`列.contains("値")`(リストの列だけ)・
/// `file.hasTag("x")`(tags の列)。列の型は kinds(無い列は Text)。
pub fn prefill(
    settings: &Settings,
    filters_expr: &[String],
    kinds: &HashMap<String, Kind>,
) -> Vec<Edit> {
    let kind = |c: &str| kinds.get(c).copied().unwrap_or(Kind::Text);
    let mut out = Vec::new();
    for c in &settings.filters {
        let k = kind(&c.col);
        let v = match &c.op {
            Op::Keep(vs) if vs.len() == 1 => vs[0].as_deref().and_then(|s| typed(k, s)),
            Op::Cmp(CmpOp::Eq, s) if k != Kind::List => typed(k, s.trim()),
            _ => None,
        };
        if let Some(v) = v {
            push_prefill(&mut out, &c.col, v);
        }
    }
    for src in filters_expr {
        let Ok(e) = expr::parse(src) else {
            continue;
        };
        for f in e.fixed_values() {
            let (col, v) = match f {
                Fixed::Eq(col, v) => {
                    let v = typed_val(kind(&col), &v);
                    (col, v)
                }
                Fixed::Contains(col, s) => {
                    let v = (kind(&col) == Kind::List && !s.is_empty())
                        .then(|| NewValue::List(vec![s]));
                    (col, v)
                }
                // Obsidian の file.hasTag はフロントマターの tags の列を見る。
                Fixed::Tag(t) => ("tags".to_string(), Some(NewValue::List(vec![t]))),
                // フォルダは値でなく作る場所(view_folder)。
                Fixed::Folder(_) => continue,
            };
            if let Some(v) = v {
                push_prefill(&mut out, &col, v);
            }
        }
    }
    out
}

/// 式の絞り込みの `file.inFolder("x")`(`&&` でつないだ項)のうち一番深いフォルダ(保管庫の根から)。
/// 新しいノートをそこに作れば、作った行がビューに残る(CE-25)。無ければ None。
pub fn view_folder(filters_expr: &[String]) -> Option<String> {
    filters_expr
        .iter()
        .filter_map(|src| expr::parse(src).ok())
        .flat_map(|e| e.fixed_values())
        .filter_map(|f| match f {
            Fixed::Folder(d) => Some(d),
            _ => None,
        })
        .max_by_key(|d| d.split('/').count())
}

/// 聞いた値が空(Null・空白だけの文字列・空のリスト)か。
fn is_empty_answer(v: &NewValue) -> bool {
    match v {
        NewValue::Null => true,
        NewValue::Str(s) => s.trim().is_empty(),
        NewValue::List(items) => items.is_empty(),
        _ => false,
    }
}

/// 新しいノートの中身(CE-25・CE-26)。絞り込みの値 → 設定の set(絞り込みに同じ列があれば絞り込みが先)
/// → 聞いた値(同じ列があれば聞いた値で置き換える。空ならその列は書かない: 絞り込みと set の値も外す)。本文は空。値が1つも無ければ空のファイル。
pub fn build(rule: &NewNote, prefill: &[Edit], answers: &[Edit]) -> Result<Vec<u8>, String> {
    let mut edits: Vec<Edit> = Vec::new();
    for e in prefill {
        if !edits.iter().any(|x| x.key == e.key) {
            edits.push(e.clone());
        }
    }
    for (k, v) in &rule.set {
        if !edits.iter().any(|x| &x.key == k) {
            edits.push(Edit {
                key: k.clone(),
                value: v.clone(),
            });
        }
    }
    for e in answers {
        // CE-26: 聞いて空のまま進めた列は書かない(前もって入れる値があっても外す)。
        if is_empty_answer(&e.value) {
            edits.retain(|x| x.key != e.key);
            continue;
        }
        match edits.iter_mut().find(|x| x.key == e.key) {
            Some(x) => x.value = e.value.clone(),
            None => edits.push(e.clone()),
        }
    }
    if let Some(e) = edits.iter().find(|e| e.key.trim().is_empty()) {
        return Err(Msg::NewNoteEmptyKey.fill(&[&format!("{:?}", e.value)]));
    }
    writeback::apply(b"", &edits).map_err(|e| match e {
        writeback::EditError::Newline => Msg::NewNoteNewline.text().to_string(),
        writeback::EditError::NotEditable(m) => Msg::NewNoteNotEditable.fill(&[&m]),
        e => Msg::NewNoteCannotBuild.fill(&[&format!("{e:?}")]),
    })
}

/// 下のフォルダを作り、create_new で書く(CE-25・WB-2)。既にあれば Err で、そのファイルは変えない。
/// 書き込みの途中で失敗したら、作ったファイルを消す。
pub fn create(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let r = f.write_all(bytes).and_then(|_| f.sync_all());
    if r.is_err() {
        drop(f);
        let _ = std::fs::remove_file(path);
    }
    r
}

#[cfg(test)]
#[path = "test_newnote_unit.rs"]
mod tests;
