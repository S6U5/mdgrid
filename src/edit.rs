//! 値の打ち込みの読み方と、書けない理由の文。画面(ui)と `--apply`(apply.rs)が共有する(画面に頼らない)。
//! 型ごとの入力の読み方(CE-2・CE-5・CE-7・CV-2): 日付は `YYYY-MM-DD`・設定の形(CE-22)・今日からの日数
//! (`+3`・`-2`)・空、数は数として読める値だけ。読めなければ理由を返す。

use mdgrid::i18n::Msg;
use mdgrid::source::{EditError, NewValue, SaveError, Value};
use mdgrid::types::{self, DateFormat, Kind};

/// 入力の種類(列の型から。CE-2)。確定のときの読み方を決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Entry {
    Text,
    Number,
    Checkbox,
    Date,
    DateTime,
}

/// 入力の文字を列の型で読む(CE-5・CE-7・CE-22・CV-2)。空は Null(CE-9)。読めなければ理由。
/// 日付は `types::parse_date_input`(`fmt` は設定の日付の形)で読む。
pub(crate) fn parse(
    entry: Entry,
    text: &str,
    today: i64,
    fmt: &DateFormat,
) -> Result<NewValue, String> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(NewValue::Null);
    }
    match entry {
        Entry::Text => Ok(NewValue::Str(text.to_string())),
        Entry::Number => {
            if let Ok(i) = t.parse::<i64>() {
                return Ok(NewValue::Int(i));
            }
            match t.parse::<f64>() {
                Ok(f) if f.is_finite() && t.bytes().any(|b| b.is_ascii_digit()) => {
                    Ok(NewValue::Float(f))
                }
                _ => Err(Msg::EntryNotNumber.fill(&[&t])),
            }
        }
        Entry::Checkbox => match t {
            "true" => Ok(NewValue::Bool(true)),
            "false" => Ok(NewValue::Bool(false)),
            _ => Err(Msg::EntryNotBool.fill(&[&t])),
        },
        Entry::Date | Entry::DateTime => {
            // 日時は時刻つきの形をそのまま書く(日付だけの打ち込みは日付の列と同じに読む)。
            if entry == Entry::DateTime
                && types::parse_date(t).is_none()
                && types::fits(Kind::DateTime, &Value::Str(t.into()))
            {
                // 空白の区切り(`2026-10-06 10:00`)は読めるが、書くのは `T` の区切り(WB-18)。
                let t = if t.as_bytes().get(10) == Some(&b' ') {
                    format!("{}T{}", &t[..10], &t[11..])
                } else {
                    t.to_string()
                };
                return Ok(NewValue::Date(t));
            }
            // CE-5・CE-22: `YYYY-MM-DD`・設定の形・`+3`・`-2`。ノートに書くのはいつも `YYYY-MM-DD`。
            // WB-18: 日付の列の値は NewValue::Date(元が囲んでいなければ囲まずに書く)。
            match types::parse_date_input(t, fmt, today) {
                Ok(Some(d)) => Ok(NewValue::Date(types::format_date(d))),
                Ok(None) => Ok(NewValue::Null),
                Err(e) if entry == Entry::DateTime && is_date_unreadable(&e) => {
                    Err(Msg::EntryNotDateTime.fill(&[&t, &date_forms(fmt)]))
                }
                Err(e) => Err(e),
            }
        }
    }
}

/// `types::parse_date_input` の誤りが「日付として読めない」(`Msg::DateUnreadable`)か。
fn is_date_unreadable(e: &str) -> bool {
    let lead = Msg::DateUnreadable.text().split("{0}").next().unwrap_or("");
    !lead.is_empty() && e.starts_with(lead)
}

/// 打ち込める日付の形の並び(CE-22: 設定の形と `YYYY-MM-DD`)。
pub(crate) fn date_forms(fmt: &DateFormat) -> String {
    if fmt.pattern() == "YYYY-MM-DD" {
        "YYYY-MM-DD".to_string()
    } else {
        format!("{}{}YYYY-MM-DD", fmt.pattern(), Msg::ListSep.text())
    }
}

/// 書けない理由の短い日本語(表示用)。
pub(crate) fn edit_error_text(e: &EditError) -> String {
    match e {
        EditError::ReadOnly(r) => Msg::ReviewReadOnly.fill(&[&format!("{r:?}")]),
        EditError::NotEditable(s) => s.clone(),
        EditError::Newline => Msg::ReviewNewline.into(),
        EditError::Verify(s) => Msg::ReviewVerify.fill(&[&s]),
    }
}

pub(crate) fn save_error_text(e: &SaveError) -> String {
    match e {
        SaveError::Changed => Msg::ReviewChanged.into(),
        SaveError::Edit(e) => edit_error_text(e),
        SaveError::Io(e) => Msg::ReviewIo.fill(&[&e]),
        SaveError::NoPermission => Msg::ReviewReadOnly.fill(&[&Msg::NoPermission.text()]),
    }
}
