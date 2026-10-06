//! 型ごとの入力の読み方(CE-2・CE-5・CE-7・CV-2)と、編集のモードの案内(input.rs から分けた純関数)。
//! 日付は `YYYY-MM-DD`・設定の形(CE-22)・今日からの日数(`+3`・`-2`)・空、数は数として読める値だけ。
//! 読めなければ理由を返す。

use super::app::App;
use super::keymap::Mode;
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, Value};
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

/// 編集のモードのメッセージ行の案内(入り方と一括の行の数)。直前の知らせがあればそちらを出す。
pub(crate) fn hint(app: &App) -> Option<String> {
    if app.mode != Mode::Edit {
        return None;
    }
    let i = app.input.as_ref()?;
    let mut t = match &i.bulk {
        Some(b) => Msg::EditBulkLead.fill(&[&b.len()]),
        None => String::new(),
    };
    if i.mismatch {
        t.push_str(Msg::EditMismatch.text());
    }
    let forms = date_forms(&app.date_format);
    t.push_str(&if app.active_list().is_some() {
        Msg::EditHintList.text().to_string()
    } else {
        match i.entry {
            Entry::Text => String::new(),
            Entry::Number => Msg::EditHintNumber.text().into(),
            Entry::Checkbox => Msg::EditHintCheckbox.text().into(),
            Entry::Date => Msg::EditHintDate.fill(&[&forms]),
            Entry::DateTime => Msg::EditHintDateTime.fill(&[&forms]),
        }
    });
    if i.list.as_ref().is_some_and(|l| l.free) {
        t.push_str(if app.list_view().is_some() {
            Msg::EditHintPickNarrowed.text()
        } else {
            Msg::EditHintBackToList.text()
        });
    }
    (!t.is_empty()).then_some(t)
}
