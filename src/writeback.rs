//! 書き戻し(タスク 3)。形は docs/design.md。
//!
//! 値のバイトの範囲だけを置き換える(WB-1)。書く前に結果を読み直して検査し(WB-6)、
//! 同じフォルダの一時ファイル → 読み直し → fsync → 直前の基準の再検査 → 名前の変更 で置き換える(WB-4・WB-8・WB-12)。

use crate::frontmatter::{
    is_int_form, is_toml, list_item_sources, parse, resolve_plain, Entry, Frontmatter, ReadOnly,
    Shape, Value,
};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub enum NewValue {
    Null,
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    /// リストの要素の文字列の並び(CE-16・CE-18)。空の Vec は `key:`(CE-19)。
    List(Vec<String>),
    /// 日付・日時の列に書く日付(WB-18)。`YYYY-MM-DD` か `YYYY-MM-DDTHH:MM(:SS)`。元の値が引用符で
    /// 囲んであれば元の引用符で、そうでなければ囲まずに書く。日付の形でなければ EditError。
    Date(String),
    /// キーの名前を、この名前に変える(CE-29。値と書き方はそのまま)。
    RenameKey(String),
    /// キーを、その値の行ごと消す(CE-29)。
    DeleteKey,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub key: String,
    pub value: NewValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    ReadOnly(ReadOnly),
    NotEditable(String),
    Newline,
    Verify(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baseline {
    pub mtime: SystemTime,
    pub len: u64,
    pub hash: [u8; 32],
}

#[derive(Debug)]
pub enum SaveError {
    Changed,
    Edit(EditError),
    Io(std::io::Error),
    /// WB-5: 利用者がそのファイルに書けない(`chmod 444` など)。書かずに止めた。
    NoPermission,
}

/// WB-5: 書き込みの権限が無いノートの理由(セルの lock と保存の止まった理由に出す)。日本語の文。
/// 表示には今の言語の `Msg::NoPermission.text()` を使う(SR-23)。
pub const NO_PERMISSION: &str = crate::i18n::Msg::NoPermission.ja();

/// WB-5: 利用者がそのファイルに書けるか(unix の access(2) の W_OK に当たる判定)。
/// std だけで、書き込みに開けるかで見る(書かずに閉じる。中身も更新時刻も変わらない)。持ち主の書き込みの bit・
/// 持ち主が自分か・グループ・ACL・読むだけのファイルシステムを OS がまとめて判定し、root は 0444 でも書ける。
/// 書き込みに開いて閉じるので、Linux ではファイルの監視(inotify の IN_CLOSE_WRITE など)を見る外の道具が反応しうる。
/// 権限で断られたときだけ書けないとし、ほかの失敗(消えたなど)は書けるとして、保存の検査に任せる。
pub fn can_write(path: &Path) -> bool {
    match File::options().write(true).open(path) {
        Ok(_) => true,
        Err(e) => !matches!(
            e.kind(),
            io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem
        ),
    }
}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

// ---- apply ----

/// 元のバイト列に編集を当てた結果を返す(ファイルには触らない)。
pub fn apply(original: &[u8], edits: &[Edit]) -> Result<Vec<u8>, EditError> {
    apply_inner(original, edits, false)
}

/// `misquote` は Faults::misquote(文字列の値をクオートせず素のまま書く。WB-6 の検査が落ちるはず)。
fn apply_inner(original: &[u8], edits: &[Edit], misquote: bool) -> Result<Vec<u8>, EditError> {
    // WB-3: フロントマターの無いノート(1行目が `---` でない)は先頭にフロントマターを足し、空のフロントマター
    // (`---` が2行だけ)は区切りの間にキーの行を足して書く。BOM・UTF-8 でない・閉じないなどは、これまでどおり
    // 書かない(WB-5)。書くかの設定(add_frontmatter)は知らない(読み込み口と画面の lock が決める)。
    let before = match parse(original) {
        Ok(fm) => Some(fm),
        // WB-20: TOML のフロントマター(`+++`)の前に `---` を足さない(読み込み口も読むだけにする。念のための止め)。
        Err(ReadOnly::NoFrontmatter) if is_toml(original) => {
            return Err(EditError::NotEditable(
                crate::i18n::Msg::LockToml.text().to_string(),
            ))
        }
        // CE-29: キーの名前の変更・削除は、キーの無いノートでは何もしない(足す編集だけを当てる)。
        Err(ReadOnly::NoFrontmatter | ReadOnly::EmptyFrontmatter)
            if edits.iter().any(is_key_op) =>
        {
            let rest: Vec<Edit> = edits.iter().filter(|e| !is_key_op(e)).cloned().collect();
            if rest.is_empty() {
                return Ok(original.to_vec());
            }
            return apply_inner(original, &rest, misquote);
        }
        Err(ReadOnly::NoFrontmatter) => None,
        Err(ReadOnly::EmptyFrontmatter) => {
            check_edits(edits)?;
            return fill_empty_frontmatter(original, edits, misquote);
        }
        Err(r) => return Err(EditError::ReadOnly(r)),
    };
    check_edits(edits)?;
    // フロントマターがあれば、parse が通ったのでその改行はそろっている。開きの行の改行に合わせる。
    // 無ければ本文の最初の改行に合わせる(改行が無ければ LF。WB-1・WB-3)。
    let newline = first_newline(original);
    let Some(before) = before else {
        return prepend_frontmatter(original, edits, newline, misquote);
    };
    let mut out = original.to_vec();
    apply_edits(&mut out, edits, newline, misquote)?;
    verify(original, &before, &out, edits)?;
    Ok(out)
}

/// 最初の行の改行(CRLF か LF。改行が無ければ LF)。
fn first_newline(bytes: &[u8]) -> &'static [u8] {
    let first_nl = bytes.iter().position(|&b| b == b'\n').unwrap_or(0);
    if first_nl > 0 && bytes[first_nl - 1] == b'\r' {
        b"\r\n"
    } else {
        b"\n"
    }
}

/// 改行を含むキー・値と、日付の形でない Date を断る。
/// キーの名前の変更か削除か(CE-29)。
fn is_key_op(e: &Edit) -> bool {
    matches!(e.value, NewValue::RenameKey(_) | NewValue::DeleteKey)
}

fn check_edits(edits: &[Edit]) -> Result<(), EditError> {
    for e in edits {
        if let NewValue::RenameKey(to) = &e.value {
            if has_newline(to) {
                return Err(EditError::Newline);
            }
            if to.trim().is_empty() {
                return Err(EditError::NotEditable(e.key.clone()));
            }
        }
        let bad = match &e.value {
            NewValue::Str(s) => has_newline(s),
            NewValue::List(items) => items.iter().any(|s| has_newline(s)),
            _ => false,
        };
        if has_newline(&e.key) || bad {
            return Err(EditError::Newline);
        }
        if let NewValue::Date(s) = &e.value {
            if !is_date(s) {
                return Err(EditError::NotEditable(format!(
                    "{}: not a date: {s}",
                    e.key
                )));
            }
        }
    }
    Ok(())
}

/// WB-3: 空のフロントマター(`---` が2行だけ)の閉じの `---` の行の前に、編集のキーの行(edits の順)を足す。
/// 足す行の改行は開きの行に合わせる(閉じの行の後ろに改行が無くても)。最初の編集の行を足し、残りの編集は
/// 今のキーを足す・書き換える道で当てる(同じキーを2回書けばあとの値)。ほかのバイト(区切りの行の末尾の空白・
/// 本文)はそのまま(WB-1)。編集が無ければ元のまま。
fn fill_empty_frontmatter(
    original: &[u8],
    edits: &[Edit],
    misquote: bool,
) -> Result<Vec<u8>, EditError> {
    let Some((first, rest)) = edits.split_first() else {
        return Ok(original.to_vec());
    };
    // parse が EmptyFrontmatter を返したので、開きの行は改行で終わり、その直後が閉じの行。
    let at = original
        .iter()
        .position(|&b| b == b'\n')
        .map(|i| i + 1)
        .ok_or_else(|| EditError::Verify("empty frontmatter without a newline".into()))?;
    let newline = first_newline(original);
    let mut out = original[..at].to_vec();
    out.extend(new_key_lines(first, newline, misquote));
    out.extend_from_slice(&original[at..]);
    apply_edits(&mut out, rest, newline, misquote)?;
    verify_filled(original, at, &out, edits)?;
    Ok(out)
}

/// WB-3: フロントマターの無いノートの先頭に `---`・編集のキーの行(edits の順。画面から保存すると Changes の列の名前の順)・`---` を足す。元のバイトはその後ろに
/// そのまま置く(WB-1)。最初の編集でフロントマターを作り、残りの編集は今のキーを足す・書き換える道で当てるので、
/// 同じキーを2回書けばあとの値になる。編集が無ければ元のまま。
fn prepend_frontmatter(
    original: &[u8],
    edits: &[Edit],
    newline: &[u8],
    misquote: bool,
) -> Result<Vec<u8>, EditError> {
    let Some((first, rest)) = edits.split_first() else {
        return Ok(original.to_vec());
    };
    let mut out = b"---".to_vec();
    out.extend_from_slice(newline);
    out.extend(new_key_lines(first, newline, misquote));
    out.extend_from_slice(b"---");
    out.extend_from_slice(newline);
    out.extend_from_slice(original);
    apply_edits(&mut out, rest, newline, misquote)?;
    verify_prepended(original, &out, edits, newline)?;
    Ok(out)
}

/// 編集を順に当てる。編集ごとに読み直すので、同じ apply の中の足したキーや重なる編集も正しい位置に当たる。
fn apply_edits(
    out: &mut Vec<u8>,
    edits: &[Edit],
    newline: &[u8],
    misquote: bool,
) -> Result<(), EditError> {
    // CE-29: 名前の変更・削除は値の直しのあとに当てる(全部のキーを消して空のフロントマターになったあとに、
    // 値の直しが読み直せなくならないように)。
    let ordered: Vec<&Edit> = edits
        .iter()
        .filter(|e| !is_key_op(e))
        .chain(edits.iter().filter(|e| is_key_op(e)))
        .collect();
    for e in ordered {
        // CE-29・WB-1: 名前の変更はキーの文字だけ、削除はキーの行と値の続きの行だけを変える。
        if is_key_op(e) {
            key_op(out, e)?;
            continue;
        }
        let fm = parse(out).map_err(|r| EditError::Verify(format!("reparse: {r:?}")))?;
        if let NewValue::List(items) = &e.value {
            let entry = fm.entries.iter().find(|x| x.key == e.key);
            write_list(out, entry, fm.end, &e.key, items, newline)?;
            continue;
        }
        match fm.entries.iter().find(|x| x.key == e.key) {
            Some(entry) => {
                if matches!(entry.shape, Shape::FlowList | Shape::BlockList) {
                    // リストにスカラーを当てると書き方が変わる(WB-7)。
                    return Err(EditError::NotEditable(e.key.clone()));
                }
                let span = entry
                    .span
                    .clone()
                    .ok_or_else(|| EditError::NotEditable(e.key.clone()))?;
                if e.value == NewValue::Null {
                    // CE-9: `key:` にする(コロンの後ろに空白も値も書かない)。行末のコメントは残す。
                    let (start, end) = null_range(out, span);
                    out.splice(start..end, std::iter::empty());
                    continue;
                }
                let mut text = match &e.value {
                    NewValue::Str(s) if misquote => s.clone(),
                    // WB-18: 囲んだ空でない値なら元の引用符、空の文字列・素の値・Null は囲まない。
                    NewValue::Date(s) => match (&entry.value, entry.shape) {
                        (Value::Str(old), Shape::DoubleQuoted | Shape::SingleQuoted)
                            if !old.is_empty() =>
                        {
                            render_value(entry.shape, &NewValue::Str(s.clone()))
                        }
                        _ => s.clone(),
                    },
                    v => render_value(entry.shape, v),
                }
                .into_bytes();
                if span.is_empty() {
                    // `key:` の空の値: コロンの直後なら空白を、コメントの直前なら空白を補う。
                    if out[span.start - 1] == b':' {
                        text.insert(0, b' ');
                    }
                    if out.get(span.end) == Some(&b'#') {
                        text.push(b' ');
                    }
                }
                out.splice(span, text);
            }
            None => {
                let lines = new_key_lines(e, newline, misquote);
                out.splice(fm.end..fm.end, lines);
            }
        }
    }
    Ok(())
}

/// CE-29: キーの名前を変えるか消す。キーが無ければ何もしない。新しい名前のキーが既にあれば NotEditable。
fn key_op(out: &mut Vec<u8>, e: &Edit) -> Result<(), EditError> {
    let layout = crate::frontmatter::key_lines(out)
        .ok_or_else(|| EditError::Verify("reparse: key lines".into()))?;
    let Some(at) = layout.iter().find(|k| k.key == e.key) else {
        return Ok(());
    };
    match &e.value {
        NewValue::RenameKey(to) => {
            // Obsidian はプロパティの名前の大文字小文字を区別しないので、大文字小文字だけ違う名前とも重ねない。
            let clash = layout
                .iter()
                .any(|k| k.key != e.key && k.key.to_lowercase() == to.to_lowercase());
            if clash {
                return Err(EditError::NotEditable(to.clone()));
            }
            out.splice(at.token.clone(), render_key(to).into_bytes());
        }
        _ => {
            // 消す行にアンカー(`&名前`)があり、残る行がその別名(`*名前`)を使うなら消さない(読めない YAML になる)。
            let removed = String::from_utf8_lossy(&out[at.lines.clone()]).into_owned();
            let mut rest = out[..at.lines.start].to_vec();
            rest.extend_from_slice(&out[at.lines.end..]);
            let rest = String::from_utf8_lossy(&rest).into_owned();
            let used = removed.split('&').skip(1).any(|t| {
                let name: String = t
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .collect();
                !name.is_empty() && rest.contains(&format!("*{name}"))
            });
            if used {
                return Err(EditError::NotEditable(e.key.clone()));
            }
            out.splice(at.lines.clone(), std::iter::empty());
        }
    }
    Ok(())
}

/// 無いキーを足すときの行(WB-3)。スカラーは `key: 値` の1行(Null は `key:`。CE-9)、リストは Obsidian と同じ
/// 縦の形(`key:` の次の行から `  - a`。空の並びは `key:`。CE-19)。どの行も `newline` で終える。
/// 空のフロントマターに足すとき(WB-3)もこの行を使う。
fn new_key_lines(e: &Edit, newline: &[u8], misquote: bool) -> Vec<u8> {
    let key = render_key(&e.key);
    let mut out = match &e.value {
        NewValue::Null | NewValue::List(_) => format!("{key}:"),
        NewValue::Str(s) if misquote => format!("{key}: {s}"),
        v => format!("{key}: {}", render_value(Shape::Plain, v)),
    }
    .into_bytes();
    out.extend_from_slice(newline);
    if let NewValue::List(items) = &e.value {
        out.extend(block_lines(items, "  ", newline));
    }
    out
}

/// CE-18・CE-19: リストを書く。元の書き方(1行のフロー・複数行のブロック)と字下げを保ち、並びの中で変わらない
/// 要素は元の書き方のまま書く。キーが無い・Null・空の文字列なら Obsidian と同じ縦の形(`key:` の次の行から `  - a`)。
/// 空の並びは `key:`。書けない形(span の無いリスト・文字列でない要素・空でないスカラー)は NotEditable。
fn write_list(
    out: &mut Vec<u8>,
    entry: Option<&Entry>,
    fm_end: usize,
    key: &str,
    items: &[String],
    newline: &[u8],
) -> Result<(), EditError> {
    let not_editable = || EditError::NotEditable(key.to_string());
    let Some(entry) = entry else {
        let edit = Edit {
            key: key.to_string(),
            value: NewValue::List(items.to_vec()),
        };
        out.splice(fm_end..fm_end, new_key_lines(&edit, newline, false));
        return Ok(());
    };
    let span = entry.span.clone().ok_or_else(not_editable)?;
    match (entry.shape, &entry.value) {
        (Shape::FlowList | Shape::BlockList, Value::List(vals)) => {
            let text = std::str::from_utf8(&out[span.clone()]).map_err(|_| not_editable())?;
            let sources = list_item_sources(entry.shape, text)
                .filter(|s| s.len() == vals.len())
                .ok_or_else(not_editable)?;
            let mut values = Vec::new();
            for v in vals {
                match v {
                    Value::Str(v) => values.push(v.clone()),
                    _ => return Err(not_editable()),
                }
            }
            if entry.shape == Shape::FlowList {
                if items.is_empty() {
                    let (start, end) = null_range(out, span);
                    out.splice(start..end, std::iter::empty());
                } else {
                    let old: Vec<(String, String)> = values
                        .into_iter()
                        .zip(sources.iter().map(|s| s.to_string()))
                        .collect();
                    let parts = pick_items(items, &old, |s| render_item(s, true));
                    out.splice(span, format!("[{}]", parts.join(", ")).into_bytes());
                }
            } else {
                // WB-1: 残る要素は元の行全体(字下げ・中身・末尾の空白・改行)のまま並べ、消す要素は行ごと消し、
                // 新しい要素の行だけを最初の要素の行の字下げと `- ` で作る。
                let lines: Vec<String> = text.split_inclusive('\n').map(String::from).collect();
                if lines.len() != values.len() {
                    return Err(not_editable());
                }
                let indent: String = text
                    .chars()
                    .take_while(|c| matches!(c, ' ' | '\t'))
                    .collect();
                let nl = String::from_utf8_lossy(newline).into_owned();
                let old: Vec<(String, String)> = values.into_iter().zip(lines).collect();
                let parts = pick_items(items, &old, |s| {
                    format!("{indent}- {}{nl}", render_item(s, false))
                });
                out.splice(span, parts.concat().into_bytes());
            }
            Ok(())
        }
        (Shape::Plain | Shape::SingleQuoted | Shape::DoubleQuoted, v)
            if matches!(v, Value::Null) || matches!(v, Value::Str(s) if s.is_empty()) =>
        {
            let (start, end) = null_range(out, span);
            out.splice(start..end, std::iter::empty());
            if !items.is_empty() {
                let at = out[start..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .map(|i| start + i + 1)
                    .ok_or_else(not_editable)?;
                let lines = block_lines(items, "  ", newline);
                out.splice(at..at, lines);
            }
            Ok(())
        }
        // CE-19: 1つの文字列で書かれたリストは、値の範囲だけをフローのリストに書き換える。元の要素は、
        // フローの中でもそのまま読める書き方なら元のまま使う。
        (Shape::Plain | Shape::SingleQuoted | Shape::DoubleQuoted, Value::Str(s)) => {
            if items.is_empty() {
                let (start, end) = null_range(out, span);
                out.splice(start..end, std::iter::empty());
            } else {
                let text = std::str::from_utf8(&out[span.clone()])
                    .map_err(|_| not_editable())?
                    .to_string();
                let keep = entry.shape != Shape::Plain || render_item(s, true) == *s;
                let old = if keep {
                    vec![(s.clone(), text)]
                } else {
                    Vec::new()
                };
                let parts = pick_items(items, &old, |v| render_item(v, true));
                out.splice(span, format!("[{}]", parts.join(", ")).into_bytes());
            }
            Ok(())
        }
        _ => Err(not_editable()),
    }
}

/// 要素ごとの書き方。元の並び `old`((値, 元の書き方))にある要素は、まだ使っていない最初のものの書き方を
/// そのまま使い、無ければ `fresh` で作る。
fn pick_items(
    items: &[String],
    old: &[(String, String)],
    fresh: impl Fn(&str) -> String,
) -> Vec<String> {
    let mut used = vec![false; old.len()];
    items
        .iter()
        .map(
            |s| match (0..old.len()).find(|&i| !used[i] && old[i].0 == *s) {
                Some(i) => {
                    used[i] = true;
                    old[i].1.clone()
                }
                None => fresh(s),
            },
        )
        .collect()
}

/// リストの1つの要素(WB-7)。フローの中では区切り・括弧と、引用符(`x 'y` は引用符つきの値の始まりと
/// 読まれうる)を含む値もクオートする。
fn render_item(s: &str, flow: bool) -> String {
    if plain_safe(s) && !(flow && s.contains([',', '[', ']', '{', '}', '\'', '"'])) {
        s.to_string()
    } else {
        double_quote(s)
    }
}

/// 新しいブロックのリストの要素の行(キーが無い・Null・空の文字列のとき)。
fn block_lines(items: &[String], indent: &str, newline: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for s in items {
        out.extend_from_slice(indent.as_bytes());
        out.extend_from_slice(b"- ");
        out.extend_from_slice(render_item(s, false).as_bytes());
        out.extend_from_slice(newline);
    }
    out
}

/// CE-9: Null を書くときに消す範囲。値の前の空白(コロンの直後から)と値、
/// 値の後ろが行末まで空白だけならその空白も消す。行末のコメントがあれば、その前の空白は残す。
fn null_range(out: &[u8], span: std::ops::Range<usize>) -> (usize, usize) {
    let mut start = span.start;
    while start > 0 && matches!(out[start - 1], b' ' | b'\t') {
        start -= 1;
    }
    let mut end = span.end;
    let mut rest = end;
    while matches!(out.get(rest), Some(b' ' | b'\t')) {
        rest += 1;
    }
    if matches!(out.get(rest), None | Some(b'\r' | b'\n')) {
        end = rest;
    } else if out.get(end) == Some(&b'#') && start < span.start {
        // `key: # c` の空の値: コメントの前の空白を1つ残す(`key:#` は別の意味になる)。
        start += 1;
    }
    (start, end)
}

/// WB-18: 書ける日付・日時か(実在する `YYYY-MM-DD`、または `YYYY-MM-DDTHH:MM(:SS)`)。
fn is_date(s: &str) -> bool {
    !s.is_empty() && crate::types::writable_datetime(s)
}

fn has_newline(s: &str) -> bool {
    s.contains(['\n', '\r'])
}

/// WB-6: 結果を読み直し、対象のキーが意図した値で、他のキーと本文が変わらないことを確かめる。
fn verify(
    original: &[u8],
    before: &Frontmatter,
    out: &[u8],
    edits: &[Edit],
) -> Result<(), EditError> {
    let fail = |m: String| Err(EditError::Verify(m));
    let after = match parse(out) {
        Ok(fm) => fm,
        // CE-29: 全部のキーを消すと空のフロントマター(`---` が2行)。本文が同じなら通す。
        Err(ReadOnly::EmptyFrontmatter)
            if before.entries.iter().all(|b| {
                edits
                    .iter()
                    .any(|e| e.key == b.key && e.value == NewValue::DeleteKey)
            }) =>
        {
            return if out.ends_with(&original[before.end..]) {
                Ok(())
            } else {
                fail("body changed".into())
            };
        }
        Err(r) => return fail(format!("result is not readable: {r:?}")),
    };
    if original[before.end..] != out[after.end..] {
        return fail("body changed".into());
    }
    verify_entries(&before.entries, &after, edits)
}

/// WB-6(フロントマターを足したとき。WB-3): 結果の先頭が `---` の行で、足したフロントマターが読め、その閉じの
/// `---` の行の直後から元のバイトがそのまま続き、キーは書いたものだけで、どれも意図した値であることを確かめる。
fn verify_prepended(
    original: &[u8],
    out: &[u8],
    edits: &[Edit],
    newline: &[u8],
) -> Result<(), EditError> {
    let fail = |m: String| Err(EditError::Verify(m));
    let after = match parse(out) {
        Ok(fm) => fm,
        Err(r) => return fail(format!("added frontmatter is not readable: {r:?}")),
    };
    let mut delimiter = b"---".to_vec();
    delimiter.extend_from_slice(newline);
    if !out.starts_with(&delimiter) {
        return fail("the added frontmatter does not open with ---".into());
    }
    if out.get(after.end..after.end + delimiter.len()) != Some(delimiter.as_slice()) {
        return fail("the added frontmatter does not close with ---".into());
    }
    if out[after.end + delimiter.len()..] != *original {
        return fail("body changed".into());
    }
    verify_entries(&[], &after, edits)
}

/// WB-6(空のフロントマターに足したとき。WB-3): 結果が読め、開きの行(`original[..at]`)と、閉じの `---` の行から
/// 後ろが元のバイトと同じで(足した行の外は変わらない)、キーは書いたものだけで、どれも意図した値であることを確かめる。
fn verify_filled(original: &[u8], at: usize, out: &[u8], edits: &[Edit]) -> Result<(), EditError> {
    let fail = |m: String| Err(EditError::Verify(m));
    let after = match parse(out) {
        Ok(fm) => fm,
        Err(r) => return fail(format!("result is not readable: {r:?}")),
    };
    if after.end < at || out[..at] != original[..at] {
        return fail("the opening --- line changed".into());
    }
    if out[after.end..] != original[at..] {
        return fail("the closing --- line or the body changed".into());
    }
    verify_entries(&[], &after, edits)
}

/// WB-6: 対象のキーが意図した値で、`before` の他のキーが変わらず、知らないキーが増えていないこと。
fn verify_entries(before: &[Entry], after: &Frontmatter, edits: &[Edit]) -> Result<(), EditError> {
    let fail = |m: String| Err(EditError::Verify(m));
    let find = |fm: &'_ Frontmatter, k: &str| -> Option<Entry> {
        fm.entries.iter().find(|e| e.key == k).cloned()
    };
    for (i, e) in edits.iter().enumerate() {
        if edits[i + 1..].iter().any(|later| later.key == e.key) {
            continue; // あとの編集が勝つ
        }
        // CE-29: 名前を変えたキーは、元の値と形のまま新しい名前にあり、元の名前には無い。消したキーは無い。
        match &e.value {
            NewValue::RenameKey(to) => {
                let Some(b) = before.iter().find(|b| b.key == e.key) else {
                    continue;
                };
                match find(after, to) {
                    Some(a) if a.shape == b.shape && same_value(&a.value, &b.value) => {}
                    _ => return fail(format!("{}: not renamed to {to}", e.key)),
                }
                if to != &e.key && find(after, &e.key).is_some() {
                    return fail(format!("{}: still there after rename", e.key));
                }
                continue;
            }
            NewValue::DeleteKey => {
                if find(after, &e.key).is_some() {
                    return fail(format!("{}: still there after delete", e.key));
                }
                continue;
            }
            _ => {}
        }
        let expected = expected_value(&e.value);
        match find(after, &e.key) {
            Some(a) if same_value(&a.value, &expected) => {}
            Some(a) => {
                return fail(format!(
                    "{}: expected {expected:?}, read back {:?}",
                    e.key, a.value
                ))
            }
            None => return fail(format!("{}: missing after write", e.key)),
        }
    }
    for b in before {
        if edits.iter().any(|e| e.key == b.key) {
            continue;
        }
        match find(after, &b.key) {
            Some(a) if a.shape == b.shape && same_value(&a.value, &b.value) => {}
            _ => return fail(format!("{}: changed by the write", b.key)),
        }
    }
    for a in &after.entries {
        let renamed_to = edits
            .iter()
            .any(|e| matches!(&e.value, NewValue::RenameKey(to) if to == &a.key));
        if !before.iter().any(|b| b.key == a.key)
            && !edits.iter().any(|e| e.key == a.key)
            && !renamed_to
        {
            return fail(format!("{}: unexpected key", a.key));
        }
    }
    Ok(())
}

fn expected_value(v: &NewValue) -> Value {
    match v {
        NewValue::Null => Value::Null,
        // WB-18: 素の日付も文字列として読み直される。
        NewValue::Str(s) | NewValue::Date(s) => Value::Str(s.clone()),
        NewValue::Bool(b) => Value::Bool(*b),
        NewValue::Int(i) => Value::Int(*i),
        NewValue::Float(f) => Value::Float(*f),
        // CE-19: 空の並びは `key:`(null)。
        NewValue::List(items) if items.is_empty() => Value::Null,
        NewValue::List(items) => Value::List(items.iter().map(|s| Value::Str(s.clone())).collect()),
        // 名前の変更・削除は verify_entries が別に確かめる(ここには来ない)。
        NewValue::RenameKey(_) | NewValue::DeleteKey => Value::Null,
    }
}

/// NaN を等しいと見なす比較。
fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Float(x), Value::Float(y)) => x == y || (x.is_nan() && y.is_nan()),
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same_value(p, q))
        }
        _ => a == b,
    }
}

// ---- 値の書き方(WB-7) ----

fn render_value(shape: Shape, v: &NewValue) -> String {
    match v {
        NewValue::Null => "null".into(),
        NewValue::Bool(b) => b.to_string(),
        NewValue::Int(i) => i.to_string(),
        NewValue::Float(f) => render_float(*f),
        // リストは write_list が書く。ここに来るときは1行のフローの形にする。
        NewValue::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|s| render_item(s, true))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        // WB-18: 素の日付(引用符を保つときは apply が Str として書く)。
        NewValue::Date(s) => s.clone(),
        // 名前の変更・削除は値を書かない(key_op が当てる)。
        NewValue::RenameKey(_) | NewValue::DeleteKey => String::new(),
        NewValue::Str(s) => match shape {
            Shape::DoubleQuoted => double_quote(s),
            Shape::SingleQuoted => single_quote(s).unwrap_or_else(|| double_quote(s)),
            _ if plain_safe(s) => s.clone(),
            _ => double_quote(s),
        },
    }
}

fn render_key(k: &str) -> String {
    if plain_safe(k) && !k.contains(':') {
        k.to_string()
    } else {
        double_quote(k)
    }
}

/// YAML 1.1 でも 1.2 でも浮動小数点として読まれる形(小数点と符号つきの指数)。
fn render_float(f: f64) -> String {
    if f.is_nan() {
        return ".nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { ".inf" } else { "-.inf" }.into();
    }
    let s = format!("{f:?}");
    match s.split_once('e') {
        Some((m, e)) => {
            let m = if m.contains('.') {
                m.to_string()
            } else {
                format!("{m}.0")
            };
            let e = if e.starts_with('-') {
                e.to_string()
            } else {
                format!("+{e}")
            };
            format!("{m}e{e}")
        }
        None if s.contains('.') => s,
        None => format!("{s}.0"),
    }
}

/// クオートなしで書いてよい文字列か。YAML 1.1 と 1.2 のどちらで読んでも文字列のままになるものだけ。
fn plain_safe(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };
    if s.starts_with([' ', '\t']) || s.ends_with([' ', '\t']) {
        return false;
    }
    // 先頭の記号・数字・符号・小数点(数・日付・.inf などに見えうる)。
    if "-?:,[]{}#&*!|>'\"%@`~+.=<".contains(first) || first.is_ascii_digit() {
        return false;
    }
    if s.contains(": ")
        || s.contains(":\t")
        || s.ends_with(':')
        || s.contains(" #")
        || s.contains("\t#")
    {
        return false;
    }
    if s.chars().any(needs_escape) {
        return false;
    }
    const WORDS: &[&str] = &["y", "n", "yes", "no", "true", "false", "on", "off", "null"];
    let lower = s.to_ascii_lowercase();
    if WORDS.contains(&lower.as_str()) {
        return false;
    }
    matches!(resolve_plain(s), Value::Str(_)) && !is_int_form(s)
}

/// 二重引用符の中でエスケープが要る文字(制御文字・YAML 1.1 の改行扱いの文字・BOM)。
fn needs_escape(c: char) -> bool {
    c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{FEFF}')
}

fn double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            c if needs_escape(c) => {
                let n = c as u32;
                if n <= 0xFF {
                    out.push_str(&format!("\\x{n:02X}"));
                } else {
                    out.push_str(&format!("\\u{n:04X}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 単一引用符で安全に書けなければ None。
fn single_quote(s: &str) -> Option<String> {
    if s.chars().any(|c| needs_escape(c) || c == '\t') {
        return None;
    }
    Some(format!("'{}'", s.replace('\'', "''")))
}

// ---- baseline と save ----

pub fn baseline(path: &Path) -> std::io::Result<Baseline> {
    let (_, base) = read_with_baseline(path)?;
    Ok(base)
}

fn read_with_baseline(path: &Path) -> io::Result<(Vec<u8>, Baseline)> {
    let mut f = File::open(path)?;
    let meta = f.metadata()?;
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    f.read_to_end(&mut bytes)?;
    let base = Baseline {
        mtime: meta.modified()?,
        len: bytes.len() as u64,
        hash: sha256(&bytes),
    };
    Ok((bytes, base))
}

/// 基準の検査 → apply → 一時ファイル → 読み直しの検査 → fsync → 直前の再検査 → 名前の変更。新しい基準を返す。
pub fn save(path: &Path, base: &Baseline, edits: &[Edit]) -> Result<Baseline, SaveError> {
    save_with(path, base, edits, &Faults::default())
}

/// 試験のための失敗を差し込む口(タスク 16)。既定はどれも差し込まない。
#[doc(hidden)]
#[derive(Debug, Default, Clone)]
pub struct Faults {
    /// 一時ファイルに書くバイトの、対象の値の1バイトを変える(WB-6 の読み直しの検査が落ちるはず)。
    pub corrupt: bool,
    /// 一時ファイルへの書き込みの途中で io::Error。
    pub fail_write: bool,
    /// fsync で io::Error。
    pub fail_fsync: bool,
    /// 文字列の値をクオートせず素のまま書く(文字列の `true` → 真偽値になる)。編集の結果を作る段で効き、
    /// WB-6 の読み直しの検査(apply の verify)が落ちるはず。
    pub misquote: bool,
    /// fsync のあと名前を変える前に、ノートをこの内容で外から書き換える(WB-12 の直前の再検査で止まるはず)。
    pub before_rename: Option<Vec<u8>>,
}

/// `save` に失敗を差し込めるもの。`save` は `save_with(.., &Faults::default())`。
#[doc(hidden)]
pub fn save_with(
    path: &Path,
    base: &Baseline,
    edits: &[Edit],
    faults: &Faults,
) -> Result<Baseline, SaveError> {
    // シンボリックリンクは辿った先を書く(リンクはそのまま残る)。
    let real = fs::canonicalize(path)?;
    let meta = fs::metadata(&real)?;
    refuse_hard_link(&meta)?;
    refuse_no_permission(&real)?;
    let (current, now) = read_with_baseline(&real)?;
    if &now != base {
        return Err(SaveError::Changed);
    }
    let new = apply_inner(&current, edits, faults.misquote).map_err(SaveError::Edit)?;

    let dir = real
        .parent()
        .ok_or_else(|| io::Error::other("the note has no parent folder"))?;
    let (tmp_path, mut tmp) = create_temp(dir, &real)?;
    let result = (|| -> Result<Baseline, SaveError> {
        copy_permissions(&tmp, &meta)?;
        let bytes = if faults.corrupt {
            std::borrow::Cow::Owned(corrupt_target(&new, edits))
        } else {
            std::borrow::Cow::Borrowed(new.as_slice())
        };
        if faults.fail_write {
            tmp.write_all(&bytes[..bytes.len() / 2])?;
            return Err(io::Error::other("injected write failure").into());
        }
        tmp.write_all(&bytes)?;
        if faults.fail_fsync {
            return Err(io::Error::other("injected fsync failure").into());
        }
        tmp.sync_all()?;
        // 新しい基準は、名前を変える前に一時ファイルの fd と書いたバイト列から作る。
        // 名前の変更のあとに読み直すと、その間の外の書き込みを基準に取り込んでしまう(WB-4・WB-12)。
        // 名前の変更は更新時刻を変えない。
        let tmeta = tmp.metadata()?;
        if tmeta.len() != new.len() as u64 {
            return Err(SaveError::Edit(EditError::Verify(
                "the temporary file size differs from the checked content".into(),
            )));
        }
        let next = Baseline {
            mtime: tmeta.modified()?,
            len: tmeta.len(),
            hash: sha256(&new),
        };
        // 一時ファイルを読み直し、検査済みの内容と同じか確かめる(WB-6)。
        let written = fs::read(&tmp_path)?;
        if written != new {
            return Err(SaveError::Edit(EditError::Verify(
                "the temporary file differs from the checked content".into(),
            )));
        }
        if let Some(outside) = &faults.before_rename {
            fs::write(&real, outside)?;
        }
        // 名前を変える直前にもう一度、基準を検査する(WB-12)。
        let meta = fs::metadata(&real)?;
        refuse_hard_link(&meta)?;
        // 名前の変更はフォルダに書ければ通るので、ファイルの権限はここでも確かめる(WB-5)。
        refuse_no_permission(&real)?;
        let (_, again) = read_with_baseline(&real)?;
        if &again != base {
            return Err(SaveError::Changed);
        }
        fs::rename(&tmp_path, &real)?;
        Ok(next)
    })();
    let next = match result {
        Ok(next) => next,
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }
    };
    // フォルダの項目の変更も確実にする(対応しないファイルシステムもあるので失敗は無視)。
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(next)
}

/// Faults::corrupt: 最初の編集の対象の値の1バイトを変えた写しを返す(長さは同じ)。
/// 値の範囲が取れなければ、フロントマターの最初のバイトを変える。
fn corrupt_target(new: &[u8], edits: &[Edit]) -> Vec<u8> {
    let mut out = new.to_vec();
    let at = parse(new)
        .ok()
        .and_then(|fm| {
            let key = &edits.first()?.key;
            let span = fm.entries.iter().find(|e| &e.key == key)?.span.clone()?;
            (!span.is_empty()).then_some(span.start)
        })
        .unwrap_or(0);
    if let Some(b) = out.get_mut(at) {
        *b ^= 0x01;
    }
    out
}

/// WB-5: ハードリンクのあるノートは書かない(置き換えるとリンクが切れるため)。
fn refuse_hard_link(meta: &fs::Metadata) -> Result<(), SaveError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() > 1 {
            return Err(SaveError::Edit(EditError::ReadOnly(ReadOnly::HardLink)));
        }
    }
    #[cfg(not(unix))]
    let _ = meta;
    Ok(())
}

/// WB-5: 利用者が書けないノートは書かない(一時ファイルからの名前の変更なら書けてしまうため)。
fn refuse_no_permission(real: &Path) -> Result<(), SaveError> {
    if can_write(real) {
        Ok(())
    } else {
        Err(SaveError::NoPermission)
    }
}

fn create_temp(dir: &Path, real: &Path) -> io::Result<(PathBuf, File)> {
    let name = real
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    for attempt in 0..16u32 {
        let p = dir.join(format!(
            ".{name}.mdgrid-tmp.{}.{nanos}.{attempt}",
            std::process::id()
        ));
        match File::options().write(true).create_new(true).open(&p) {
            Ok(f) => return Ok((p, f)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other("could not create a temporary file"))
}

/// 元のファイルの権限(と、unix では持ち主)を一時ファイルに写す。
fn copy_permissions(tmp: &File, meta: &fs::Metadata) -> io::Result<()> {
    tmp.set_permissions(meta.permissions())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let tmeta = tmp.metadata()?;
        if tmeta.uid() != meta.uid() || tmeta.gid() != meta.gid() {
            std::os::unix::fs::fchown(tmp, Some(meta.uid()), Some(meta.gid()))?;
        }
    }
    Ok(())
}

// ---- SHA-256(FIPS 180-4。依存を足さないための最小の実装) ----

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for block in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

#[cfg(test)]
#[path = "test_writeback_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_big_int_write_unit.rs"]
mod test_big_int_write_unit;

#[cfg(test)]
#[path = "test_key_ops_unit.rs"]
mod test_key_ops_unit;
