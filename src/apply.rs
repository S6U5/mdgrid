//! `--apply <ファイル>`(CLI-17): `--print --with-path` の形の CSV・JSON を読み、各行の `path` のノートの、
//! 今と違うセルだけをためる変更にする。書き込みは画面の保存と同じ道(`Changes`。WB-1〜WB-8)。
//! 理由が1つでもあれば、どれも書かない。既定は差分だけを出し、`--yes` のときだけ書く。
//!
//! 「違うか」は、今の値を `--print` と同じ形(CSV の文字・JSON の値)にして比べる。同じなら型の読み直しも
//! 読むだけの確かめもしない(書き出したものをそのまま戻しても、何も書かない)。

use crate::diff::{diff, DiffLine};
use crate::edit::{edit_error_text, parse, save_error_text, Entry};
use mdgrid::changes::{Changes, Outcome};
use mdgrid::i18n::Msg;
use mdgrid::print::{value_json, value_plain};
use mdgrid::source::{NewValue, RowId, Source, Value};
use mdgrid::types::{self, DateFormat, Kind};
use std::collections::{HashMap, HashSet};

/// 読んだ1行(何行目か(CSV はファイルの行、JSON は何番目か)、path、見出しの名前と値)。
pub(crate) struct Record {
    pub line: usize,
    pub path: String,
    pub cells: Vec<(String, Cell)>,
}

/// セルの値。JSON はその型のまま、CSV は文字。
pub(crate) enum Cell {
    Json(serde_json::Value),
    Text(String),
}

/// 読んだもの: 行と、見出しの名前(CSV は見出しの行、JSON は出てきた鍵の順。`path` を除く)。
pub(crate) struct Input {
    pub records: Vec<Record>,
    pub headers: Vec<String>,
}

/// 中身から形を決めて読む: 最初の空白でない文字が `[` なら JSON(オブジェクトの配列)、ほかは CSV(見出しつき)。
pub(crate) fn read_input(text: &str) -> Result<Input, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.trim_start().starts_with('[') {
        read_json(text)
    } else {
        read_csv(text)
    }
}

fn read_json(text: &str) -> Result<Input, String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| Msg::ApplyBadJson.fill(&[&e]))?;
    let Some(items) = v.as_array() else {
        return Err(Msg::ApplyBadJson.fill(&[&"not an array"]));
    };
    let mut records = Vec::new();
    let mut headers: Vec<String> = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let Some(obj) = item.as_object() else {
            return Err(Msg::ApplyNoPath.fill(&[&(i + 1)]));
        };
        let Some(path) = obj.get("path").and_then(|p| p.as_str()) else {
            return Err(Msg::ApplyNoPath.fill(&[&(i + 1)]));
        };
        let mut cells = Vec::new();
        for (k, v) in obj.iter().filter(|(k, _)| k.as_str() != "path") {
            if !headers.contains(k) {
                headers.push(k.clone());
            }
            cells.push((k.clone(), Cell::Json(v.clone())));
        }
        records.push(Record {
            line: i + 1,
            path: path.to_string(),
            cells,
        });
    }
    Ok(Input { records, headers })
}

/// RFC 4180 の CSV(`"` で囲んだ欄、`""` は `"`、欄の中の改行)。行の終わりは LF か CRLF で、
/// 囲んだ欄の中の CRLF も LF にそろえる(表計算の書き出し)。返すのは (ファイルの何行目から始まるか, 欄)。
fn csv_rows(text: &str) -> Result<Vec<(usize, Vec<String>)>, String> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    let mut any = false;
    let mut line = 1;
    let mut start = 1;
    while let Some(c) = chars.next() {
        any = true;
        if c == '\n' {
            line += 1;
        }
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => quoted = false,
                '\r' if chars.peek() == Some(&'\n') => {}
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => quoted = true,
            ',' => row.push(std::mem::take(&mut field)),
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push((start, std::mem::take(&mut row)));
                start = line;
                any = false;
            }
            _ => field.push(c),
        }
    }
    if quoted {
        return Err(Msg::ApplyBadCsv.text().into());
    }
    if any {
        row.push(field);
        rows.push((start, row));
    }
    Ok(rows)
}

fn read_csv(text: &str) -> Result<Input, String> {
    let mut rows = csv_rows(text)?.into_iter();
    let Some((_, head)) = rows.next() else {
        return Ok(Input {
            records: Vec::new(),
            headers: Vec::new(),
        });
    };
    let Some(at) = head.iter().position(|h| h == "path") else {
        return Err(Msg::ApplyNoPathColumn.text().into());
    };
    let mut seen = HashSet::new();
    for h in &head {
        if !seen.insert(h) {
            return Err(Msg::ApplyDuplicateHeader.fill(&[h]));
        }
    }
    let mut records = Vec::new();
    for (line, r) in rows {
        if r.len() == 1 && r[0].is_empty() {
            continue;
        }
        if r.len() != head.len() {
            return Err(Msg::ApplyCsvWidth.fill(&[&line, &r.len(), &head.len()]));
        }
        records.push(Record {
            line,
            path: r[at].clone(),
            cells: head
                .iter()
                .zip(r.iter())
                .enumerate()
                .filter(|(k, _)| *k != at)
                .map(|(_, (h, v))| (h.clone(), Cell::Text(v.clone())))
                .collect(),
        });
    }
    let headers = head
        .into_iter()
        .enumerate()
        .filter(|(k, _)| *k != at)
        .map(|(_, h)| h)
        .collect();
    Ok(Input { records, headers })
}

/// 見出しの名前 → 列の id。開いたビューの列の表示名を先に、次に列の id、`note.` を外した名前、ノートのキー。
/// 表示名と別の列の id が同じ名前・ビューの列にもノートのキーにも無い名前は理由(書くと別の列を変えるか、
/// 打ち間違いのキーを足すため)。`file.*`・`formula.*` は None(読まない)。
pub(crate) fn header_ids(
    headers: &[String],
    view: &[(String, String)],
    keys: &[String],
) -> Result<HashMap<String, Option<String>>, Vec<String>> {
    let mut out = HashMap::new();
    let mut problems = Vec::new();
    for h in headers {
        let by_title: Vec<&String> = view
            .iter()
            .filter(|(_, t)| t == h)
            .map(|(id, _)| id)
            .collect();
        let bare = h.strip_prefix("note.").unwrap_or(h);
        let by_id: Vec<&String> = view
            .iter()
            .filter(|(id, _)| id == bare)
            .map(|(id, _)| id)
            .collect();
        let id = match (by_title.as_slice(), by_id.as_slice()) {
            ([a], []) => (*a).clone(),
            ([], [b]) => (*b).clone(),
            ([a], [b]) if a == b => (*a).clone(),
            ([], []) => {
                if bare.starts_with("file.") || bare.starts_with("formula.") {
                    out.insert(h.clone(), None);
                    continue;
                }
                if !keys.iter().any(|k| k == bare) {
                    problems.push(Msg::ApplyUnknownColumn.fill(&[h]));
                    continue;
                }
                bare.to_string()
            }
            _ => {
                problems.push(Msg::ApplyAmbiguousColumn.fill(&[h]));
                continue;
            }
        };
        let id = (!id.starts_with("file.") && !id.starts_with("formula.")).then_some(id);
        // 2つの見出しが同じ列に当たる(`status` と `note.status`)なら、どちらを当てるか決められない。
        if let Some(i) = &id {
            if out.values().any(|x: &Option<String>| x.as_ref() == Some(i)) {
                problems.push(Msg::ApplyAmbiguousColumn.fill(&[h]));
                continue;
            }
        }
        out.insert(h.clone(), id);
    }
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems)
    }
}

/// 列の型の入り方(画面の入力と同じ)。
fn entry_of(kind: Kind) -> Entry {
    match kind {
        Kind::Number => Entry::Number,
        Kind::Checkbox => Entry::Checkbox,
        Kind::Date => Entry::Date,
        Kind::DateTime => Entry::DateTime,
        Kind::Text | Kind::List => Entry::Text,
    }
}

/// 文字を列の型で読む。日付は `YYYY-MM-DD`(日時は時刻つき)だけ(`+3` のような今日からの日数は、
/// 当てる日で値が変わるので読まない)。
fn text_value(s: &str, kind: Kind, today: i64, fmt: &DateFormat) -> Result<NewValue, String> {
    let t = s.trim();
    if matches!(kind, Kind::Date | Kind::DateTime)
        && !t.is_empty()
        && types::parse_date(t).is_none()
        && !(kind == Kind::DateTime && types::fits(Kind::DateTime, &Value::Str(t.into())))
    {
        return Err(Msg::ApplyDateForm.fill(&[&t]));
    }
    parse(entry_of(kind), s, today, fmt)
}

/// 今の値が CSV のリストの欄から元に戻せる形か(要素に `, ` や前後の空白があると、分けると別の要素になる)。
fn list_round_trips(v: Option<&Value>) -> bool {
    match v {
        Some(Value::List(items)) => items.iter().all(|x| match x {
            Value::Str(s) => !s.contains(", ") && s.trim() == s,
            _ => true,
        }),
        _ => true,
    }
}

/// セルの値を列の型で書く値にする(画面の入力と同じ読み方)。
fn to_value(
    cell: &Cell,
    kind: Kind,
    cur: Option<&Value>,
    today: i64,
    fmt: &DateFormat,
) -> Result<NewValue, String> {
    match cell {
        Cell::Text(s) if kind == Kind::List || matches!(cur, Some(Value::List(_))) => {
            if !list_round_trips(cur) {
                return Err(Msg::ApplyListAmbiguous.text().into());
            }
            let items: Vec<String> = s
                .split(", ")
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(String::from)
                .collect();
            // 要素を全部外すのは空のリスト(書き戻しが `key:` と書く。CE-19)。
            Ok(NewValue::List(items))
        }
        Cell::Text(s) => text_value(s, kind, today, fmt),
        Cell::Json(v) => match v {
            serde_json::Value::Null => Ok(NewValue::Null),
            serde_json::Value::Bool(b) if kind == Kind::Checkbox => Ok(NewValue::Bool(*b)),
            serde_json::Value::Number(n) if kind == Kind::Number => match n.as_i64() {
                Some(i) => Ok(NewValue::Int(i)),
                // i64 に入らない整数は丸めて書かない。
                None if n.is_u64() => Err(Msg::ApplyBigNumber.fill(&[n])),
                None => Ok(NewValue::Float(n.as_f64().unwrap_or(f64::NAN))),
            },
            serde_json::Value::String(s) if kind == Kind::List => Ok(if s.is_empty() {
                NewValue::List(Vec::new())
            } else {
                NewValue::List(vec![s.clone()])
            }),
            // テキストの列の文字はそのまま(空や空白だけの文字も消さない)。
            serde_json::Value::String(s) if kind == Kind::Text => Ok(NewValue::Str(s.clone())),
            serde_json::Value::String(s) => text_value(s, kind, today, fmt),
            serde_json::Value::Array(items) => {
                let strs: Option<Vec<String>> =
                    items.iter().map(|x| x.as_str().map(String::from)).collect();
                match strs {
                    Some(s) if kind == Kind::List || kind == Kind::Text => Ok(NewValue::List(s)),
                    _ => Err(Msg::ApplyNotList.text().into()),
                }
            }
            serde_json::Value::Object(_) => Err(Msg::ApplyNotList.text().into()),
            other => text_value(&other.to_string(), kind, today, fmt),
        },
    }
}

/// 読んだセルが、今の値を `--print` で出したものと同じか(同じなら変えない)。改行は LF にそろえて比べる。
fn unchanged(cell: &Cell, cur: Option<&Value>) -> bool {
    match cell {
        Cell::Text(s) => {
            let now = cur.map(value_plain).unwrap_or_default();
            s.replace("\r\n", "\n") == now.replace("\r\n", "\n")
        }
        Cell::Json(v) => {
            let now = cur.map(value_json).unwrap_or_else(|| "null".into());
            serde_json::from_str::<serde_json::Value>(&now).is_ok_and(|n| &n == v)
        }
    }
}

/// 当てた結果: ためた変更と、見つけた理由(1つでもあれば書かない)。
pub(crate) struct Plan {
    pub changes: Changes,
    pub problems: Vec<String>,
    /// 書けない列(`file.*`・`formula.*`)で、今と違う値の直しの数と列の名前(当てずに知らせる)。
    pub ignored: (usize, Vec<String>),
}

/// path から行を引いた結果。
pub(crate) enum Found {
    Row(RowId),
    Missing,
    /// 起動の引数のフォルダによって別のノートになる。
    Ambiguous,
}

/// 読んだ行をためる変更にする。`find` は path から行を引く。`ids` は `header_ids` の結果。
pub(crate) fn plan(
    src: &dyn Source,
    input: &Input,
    find: &dyn Fn(&str) -> Found,
    ids: &HashMap<String, Option<String>>,
    shown: &dyn Fn(&RowId, &str) -> Option<String>,
    today: i64,
    fmt: &DateFormat,
) -> Plan {
    let mut changes = Changes::new();
    let mut problems = Vec::new();
    let mut done: HashMap<(RowId, String), usize> = HashMap::new();
    let mut ignored = (0usize, Vec::<String>::new());
    for r in &input.records {
        let row = match find(&r.path) {
            Found::Row(row) => row,
            Found::Missing => {
                problems.push(Msg::ApplyUnknownPath.fill(&[&r.line, &r.path]));
                continue;
            }
            Found::Ambiguous => {
                problems.push(Msg::ApplyAmbiguousPath.fill(&[&r.line, &r.path]));
                continue;
            }
        };
        for (name, cell) in &r.cells {
            let id = match ids.get(name) {
                Some(Some(id)) => id,
                // 書けない列: 今の表の値と違えば数えて知らせる(当てない)。
                Some(None) => {
                    let bare = name.strip_prefix("note.").unwrap_or(name);
                    if (bare.starts_with("file.") || bare.starts_with("formula."))
                        && !shown_same(shown, &row, bare, cell)
                    {
                        ignored.0 += 1;
                        if !ignored.1.contains(name) {
                            ignored.1.push(name.clone());
                        }
                    }
                    continue;
                }
                None => continue,
            };
            // 同じノートの同じ列は1回だけ(最初に当てた行を理由に出す)。
            let first = *done.entry((row.clone(), id.clone())).or_insert(r.line);
            if first != r.line {
                problems.push(Msg::ApplyDuplicateRow.fill(&[&r.line, &r.path, id, &first]));
                continue;
            }
            let cur = src.get(&row, id);
            // 書き出したまま(同じ)なら変えない。型の読み直しも、読むだけの確かめもしない。
            if unchanged(cell, cur.value.as_ref()) {
                continue;
            }
            let ck = src.kind(id);
            let v = match to_value(cell, ck.kind, cur.value.as_ref(), today, fmt) {
                Ok(v) => v,
                Err(e) => {
                    problems.push(Msg::ApplyBadValue.fill(&[&r.line, &r.path, id, &e]));
                    continue;
                }
            };
            // キーの無いノートに空の値を入れない(キーを足さない)。
            if cur.value.is_none() && (v == NewValue::Null || v == NewValue::List(Vec::new())) {
                continue;
            }
            if let Some(lock) = ck.lock.or(cur.lock) {
                problems.push(Msg::ApplyLocked.fill(&[&r.line, &r.path, id, &lock]));
                continue;
            }
            if let Err(s) = changes.set(src, &row, id, v) {
                problems.push(Msg::ApplyLocked.fill(&[&r.line, &r.path, id, &s.reason]));
            }
        }
    }
    Plan {
        changes,
        problems,
        ignored,
    }
}

/// 書けない列のセルが、今の表(`--print` の素の文字)と同じか。表に無ければ同じとみなす。
fn shown_same(
    shown: &dyn Fn(&RowId, &str) -> Option<String>,
    row: &RowId,
    id: &str,
    cell: &Cell,
) -> bool {
    let Some(now) = shown(row, id) else {
        return true;
    };
    match cell {
        Cell::Text(s) => s.replace("\r\n", "\n") == now.replace("\r\n", "\n"),
        Cell::Json(serde_json::Value::String(s)) => *s == now,
        Cell::Json(serde_json::Value::Null) => now.is_empty(),
        Cell::Json(v) => *v == now,
    }
}

/// 差分の文字(WB-9 と同じ `-`・`+`・前後の文脈)。`label` は行の見せる名前。返すのは (差分, 理由, ファイルの数)。
pub(crate) fn diff_text(
    plan: &Plan,
    src: &dyn Source,
    label: &dyn Fn(&RowId) -> String,
) -> (String, Vec<String>, usize) {
    let mut out = String::new();
    let mut problems = Vec::new();
    let mut files = 0;
    for p in plan.changes.previews(src) {
        match p {
            Ok(p) => {
                files += 1;
                let name = label(&p.row);
                out.push_str(&format!("--- {name}\n+++ {name}\n"));
                for l in diff(&p.before, &p.after) {
                    match l {
                        DiffLine::Same(t) => out.push_str(&format!(" {t}\n")),
                        DiffLine::Del(t) => out.push_str(&format!("-{t}\n")),
                        DiffLine::Add(t) => out.push_str(&format!("+{t}\n")),
                        DiffLine::Gap => out.push_str("@@\n"),
                    }
                }
            }
            Err((row, e)) => problems.push(format!("{}: {}", label(&row), edit_error_text(&e))),
        }
    }
    (out, problems, files)
}

/// `--yes`: 書く。書いた行の名前と、書けなかった行の理由。
pub(crate) fn save(
    plan: &mut Plan,
    src: &mut dyn Source,
    label: &dyn Fn(&RowId) -> String,
) -> (Vec<String>, Vec<String>) {
    let mut saved = Vec::new();
    let mut failed = Vec::new();
    for (row, outcome) in plan.changes.save(src) {
        match outcome {
            Outcome::Saved => saved.push(label(&row)),
            Outcome::Changed => failed.push(Msg::ApplyChangedOutside.fill(&[&label(&row)])),
            Outcome::Failed(e) => failed.push(format!("{}: {}", label(&row), save_error_text(&e))),
        }
    }
    (saved, failed)
}
