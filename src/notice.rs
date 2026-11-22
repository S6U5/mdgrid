//! 通知の欄(notice.*)の核: 設定の `[[notices]]` を読み、件数の項目と連絡の項目の文を作る。
//! 行を数えるのは画面(src/ui/notices.rs)と `--notices`(main.rs)の側。ここは画面に依存しない。

use crate::frontmatter;
use crate::i18n::Msg;
use crate::profile::{not, warn};
use std::path::{Path, PathBuf};

/// notice.scope: 件数の項目が数える範囲。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// 今の表(既定)。
    Table,
    /// 今の範囲の表の全部。
    Workspace,
    /// 名前を挙げた表(ワークスペースの表の `name`)。
    Tables(Vec<String>),
}

/// 通知の欄の1項目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// notice.count-item: 名前の条件に当たる行の数。
    Count {
        cond: String,
        /// `{count}` を件数に置き換える文(無ければ条件の見せる名前と件数)。
        text: Option<String>,
        scope: Scope,
        /// notice.zero: 0 件でも出す。
        show_zero: bool,
    },
    /// notice.note-item: ノートの本文の頭の行。
    Note {
        /// 書いたパス(先頭の `~` はホーム、相対は設定のファイルのフォルダから。`resolve_notes` で直す)。
        path: PathBuf,
        lines: usize,
    },
}

/// notice.layout: 欄の行の上限。
pub const MAX_LINES: usize = 3;

/// config.toml の `[[notices]]` を読む(書いた順)。形の問題は警告して、その項目を外す。
pub fn read(v: &toml::Value, file: &str, w: &mut Vec<String>) -> Vec<Item> {
    let Some(items) = v.as_array() else {
        warn(w, file, "notices", not(Msg::WantNoticeArray));
        return Vec::new();
    };
    if items.iter().any(|t| !t.is_table()) {
        warn(w, file, "notices", not(Msg::WantNoticeArray));
        return Vec::new();
    }
    let mut out = Vec::new();
    for (i, t) in items.iter().enumerate() {
        let path = format!("notices[{}]", i + 1);
        let Some(t) = t.as_table() else {
            continue;
        };
        if let Some(item) = read_item(t, file, &path, w) {
            out.push(item);
        }
    }
    out
}

fn read_item(t: &toml::Table, file: &str, path: &str, w: &mut Vec<String>) -> Option<Item> {
    let cond = t.get("cond").and_then(toml::Value::as_str);
    let note = t.get("note").and_then(toml::Value::as_str);
    let known: &[&str] = match (cond, note) {
        (Some(_), None) => &["cond", "text", "scope", "show_zero"],
        (None, Some(_)) => &["note", "lines"],
        _ => {
            warn(w, file, path, Msg::RsnNoticeKind.text().to_string());
            return None;
        }
    };
    for k in t.keys() {
        if !known.contains(&k.as_str()) {
            warn(
                w,
                file,
                &format!("{path}.{k}"),
                Msg::RsnUnknown.text().to_string(),
            );
        }
    }
    if let Some(n) = note {
        let lines = match t.get("lines") {
            None => 1,
            Some(v) => match v.as_integer().and_then(|n| usize::try_from(n).ok()) {
                Some(n) if n >= 1 => n,
                _ => {
                    warn(w, file, &format!("{path}.lines"), not(Msg::WantIntAtLeast1));
                    1
                }
            },
        };
        return Some(Item::Note {
            path: PathBuf::from(n),
            lines,
        });
    }
    let text = t.get("text").and_then(|v| {
        let s = v.as_str().map(str::to_string);
        if s.is_none() {
            warn(w, file, &format!("{path}.text"), not(Msg::WantString));
        }
        s
    });
    let scope = t.get("scope").map_or(Scope::Table, |v| {
        read_scope(v).unwrap_or_else(|| {
            warn(w, file, &format!("{path}.scope"), not(Msg::WantNoticeScope));
            Scope::Table
        })
    });
    let show_zero = match t.get("show_zero") {
        None => false,
        Some(v) => v.as_bool().unwrap_or_else(|| {
            warn(w, file, &format!("{path}.show_zero"), not(Msg::WantBool));
            false
        }),
    };
    Some(Item::Count {
        cond: cond.unwrap_or_default().to_string(),
        text,
        scope,
        show_zero,
    })
}

fn read_scope(v: &toml::Value) -> Option<Scope> {
    match v {
        toml::Value::String(s) if s == "table" => Some(Scope::Table),
        toml::Value::String(s) if s == "workspace" => Some(Scope::Workspace),
        toml::Value::Array(xs) => xs
            .iter()
            .map(|x| x.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .filter(|v| !v.is_empty())
            .map(Scope::Tables),
        _ => None,
    }
}

/// 連絡の項目のパスを直す: 先頭の `~` はホーム、相対は `base`(設定のファイルのフォルダ)から。
pub fn resolve_notes(items: &mut [Item], base: Option<&Path>, home: Option<&Path>) {
    for item in items {
        if let Item::Note { path, .. } = item {
            let s = path.to_string_lossy().into_owned();
            if let (Some(rest), Some(h)) = (s.strip_prefix("~/"), home) {
                *path = h.join(rest);
            } else if s == "~" {
                if let Some(h) = home {
                    *path = h.to_path_buf();
                }
            } else if path.is_relative() {
                if let Some(b) = base {
                    *path = b.join(&*path);
                }
            }
        }
    }
}

/// notice.count-item: 件数の項目の文。`count` が None なら数えている途中(「…」)。`partial` は読めない表が
/// あった(notice.unreadable の「?」)。
pub fn count_text(text: Option<&str>, label: &str, count: Option<usize>, partial: bool) -> String {
    let n = count.map_or_else(|| "…".to_string(), |n| n.to_string());
    let mut s = match text {
        Some(t) => t.replace("{count}", &n),
        None => format!("{label} {n}"),
    };
    if partial {
        // 「期限切れ 2 件?」: 数のあとに添える(文に `{count}` が無ければ末尾)。
        s = match text {
            Some(t) if t.contains("{count}") => t.replace("{count}", &format!("{n}?")),
            _ => format!("{s}?"),
        };
    }
    s
}

/// notice.note-item: ノートの本文(フロントマターを除く)の空でない行を頭から `lines` 行。
/// ファイルが無いか読めなければ Err(知らせの文)。
pub fn note_lines(path: &Path, lines: usize) -> Result<Vec<String>, String> {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let bytes = std::fs::read(path).map_err(|_| Msg::NoticeNoNote.fill(&[&name]))?;
    // 本文は、フロントマターの閉じの区切りの行の次から(フロントマターが無ければ頭から)。
    let body_at = frontmatter::parse(&bytes).ok().map_or(0, |fm| {
        bytes[fm.end..]
            .iter()
            .position(|&c| c == b'\n')
            .map_or(bytes.len(), |p| fm.end + p + 1)
    });
    let body = String::from_utf8_lossy(&bytes[body_at.min(bytes.len())..]).into_owned();
    Ok(body
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty())
        .take(lines)
        .map(str::to_string)
        .collect())
}

/// notice.layout: 欄の行。件数の項目は1行に ` · ` でつなぎ、連絡の行はそれぞれ1行。`MAX_LINES` を超える分は
/// 最後の行を「ほか N 行」にする。
pub fn layout(counts: &[String], notes: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if !counts.is_empty() {
        out.push(counts.join(" · "));
    }
    out.extend(notes.iter().cloned());
    if out.len() > MAX_LINES {
        let rest = out.len() - (MAX_LINES - 1);
        out.truncate(MAX_LINES - 1);
        out.push(Msg::NoticeMore.fill(&[&rest]));
    }
    out
}
