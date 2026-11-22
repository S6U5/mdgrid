//! 日付の言葉の読みを見せる(cell-edit.date-words-preview)と、画面の側の日付の打ち込みの読み方(`App::date_input`)。
//!
//! 日付・日時の入力で言葉(字を含む打ち込み: `明日`・`来週の金曜`・`+3d`)を打っているあいだ、入力の欄の横に
//! 読んだ日(`2026-11-27(金)`)を、読めなければ「読めない」を出す。`2026-11-03`・`+3` のような字の無い形と、
//! 日時の時刻つきの形(`2026-11-03T09:00`)には出さない(数字を打っている途中で「読めない」と出さない)。
//! カレンダー(calendar.rs)は、打つたびに同じ読み方でその日へ動く。言葉そのものの読みは src/datewords.rs。

use super::app::App;
use super::entry::Entry;
use super::keymap::Mode;
use super::list::splice;
use super::settings::TextKind;
use super::view::visible_layout;
use super::width::{sanitize, width};
use mdgrid::i18n::Msg;
use mdgrid::source::Value;
use mdgrid::types::{self, DateInput, Kind};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

impl App {
    /// 日付の打ち込みの読み方: 設定の日付の形(CE-22)と週の始まり(dates.week_start。日付の言葉の「今週」「来週」の区切り)。
    pub(crate) fn date_input(&self) -> DateInput<'_> {
        DateInput {
            fmt: &self.date_format,
            week: self.week_start,
        }
    }
}

/// 打ち込み `text` の読みの見せ方: 読めれば `2026-11-27(金)`、読めなければ「読めない」。
/// 日付の言葉でない(字を含まないか、日時の列の時刻つきの形)なら None。`datetime` は日時の列か。
pub(crate) fn preview(app: &App, datetime: bool, text: &str) -> Option<String> {
    let t = text.trim();
    if !t.chars().any(char::is_alphabetic) {
        return None;
    }
    if datetime && types::fits(Kind::DateTime, &Value::Str(t.to_string())) {
        return None;
    }
    Some(
        match types::parse_date_input(t, app.date_input(), app.today) {
            Ok(Some(d)) => {
                Msg::DateWordsDay.fill(&[&types::format_date(d), &types::weekday_short(d)])
            }
            _ => Msg::DateWordsUnreadable.text().to_string(),
        },
    )
}

/// 絞り込みの比べる値(`kind` が比べる種類)の読み。日付・日時の列だけ(cell-edit.date-words)。
pub(crate) fn operand_preview(app: &App, col: &str, kind: TextKind, text: &str) -> Option<String> {
    if !matches!(kind, TextKind::Cmp(_)) {
        return None;
    }
    match app.column_kind(col) {
        Kind::Date => preview(app, false, text),
        Kind::DateTime => preview(app, true, text),
        _ => None,
    }
}

/// 今のセルの入力の読み(編集のモードの日付・日時の入力で、開いたときの文字から打ち変えたときだけ)。
fn input_preview(app: &App) -> Option<String> {
    if app.mode != Mode::Edit {
        return None;
    }
    let i = app.input.as_ref()?;
    if !matches!(i.entry, Entry::Date | Entry::DateTime) || i.text == i.initial {
        return None;
    }
    preview(app, i.entry == Entry::DateTime, &i.text)
}

/// 入力の欄の横に読みを重ねる(下の帯より上の行 `lines` に)。表のセルの入力の欄と新しいノートの欄は、欄の右
/// (入らなければ左)。詳細の表示の欄は右の端まで広いので、欄の中の右に寄せる(打った文字に重なるなら出さない)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(label) = input_preview(app) else {
        return;
    };
    let text = format!(" {label} ");
    let tw = width(&text);
    let at = if app.detail.is_some() {
        super::detail::detail_input_at(app, w).and_then(|(ix, iy, room)| {
            let typed = app.input.as_ref().map_or(0, |i| width(&sanitize(&i.text))) + 1;
            (typed + tw <= room && ix + room <= w).then(|| (ix + room - tw, iy))
        })
    } else {
        let (lay, cols) = visible_layout(app);
        super::input::input_box(app, &lay, &cols).and_then(|b| {
            if b.x + b.w + tw <= w {
                Some((b.x + b.w, b.y))
            } else {
                b.x.checked_sub(tw).map(|x| (x, b.y))
            }
        })
    };
    let Some((x, y)) = at else {
        return;
    };
    if let Some(line) = lines.get_mut(y) {
        let st = Style::default().add_modifier(Modifier::BOLD);
        *line = splice(line, x, Span::styled(text, st), tw, w);
    }
}
