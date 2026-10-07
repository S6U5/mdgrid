//! 型ごとの入力の読み方(CE-2・CE-5・CE-7・CV-2)と、編集のモードの案内(input.rs から分けた純関数)。
//! 日付は `YYYY-MM-DD`・設定の形(CE-22)・今日からの日数(`+3`・`-2`)・空、数は数として読める値だけ。
//! 読めなければ理由を返す。

use super::app::App;
use super::keymap::Mode;
use mdgrid::i18n::Msg;

// 入力の種類と読み方は --apply と共有するので src/edit.rs にある(今までの道はそのまま使える)。
pub(crate) use crate::edit::{date_forms, parse, Entry};

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
