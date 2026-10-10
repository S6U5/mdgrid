//! セルの見せ方の純関数(CV-1・CV-2・CV-3・CV-4・BV-7・SR-15・CE-22)。状態は書き換えない。

use super::app::App;
use super::width::{sanitize, Align};
use mdgrid::base;
use mdgrid::cells::Part;
use mdgrid::expr::Val;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::{self, Kind};

/// 読むだけのセルの印(SR-15)。
pub(crate) const LOCK_MARK: char = '#';
/// 型の合わない値の印(CV-2)。
pub(crate) const MISFIT_MARK: char = '!';
/// 評価できない式の印(BV-7。薄く出す)。
pub(crate) const UNSUPPORTED_MARK: &str = "?";

/// セルの見せ方。
pub(crate) struct Shown {
    pub text: String,
    /// 薄く出す(null の `∅`・未対応の `?`)。
    pub null: bool,
    /// ためる変更(色と `*`。SR-15)。
    pub pending: bool,
    /// 読むだけ(薄く。理由は選んだときメッセージ行に)。
    pub locked: Option<String>,
    /// 評価できない式(理由。選んだとき下の帯とメッセージ行に。BV-7)。
    pub unsupported: Option<String>,
    /// SR-35: 部品(札・リンク)。文字(text)は部品の見せ方の文字で、幅もこれで数える。
    pub part: CellPart,
}

/// セルの部品(SR-35)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CellPart {
    None,
    /// 値の部品の並び(SR-36 の形。text は chips::parts の文字)。`list` はリストの列の値。
    Chips {
        items: Vec<String>,
        list: bool,
    },
    /// リンク(アクセントの色)。
    Link,
}

/// 真偽の部品(SR-35)。
pub(crate) const CHECK_ON: &str = "☑";
pub(crate) const CHECK_OFF: &str = "☐";

/// 真偽の部品の文字(SR-36 の `check`)。`text` なら None(値の文字のまま)。
pub(crate) fn check_marks(check: mdgrid::style::Check) -> Option<(&'static str, &'static str)> {
    use mdgrid::style::Check;
    match check {
        Check::Box_ => Some((CHECK_ON, CHECK_OFF)),
        Check::Tick => Some(("✓", "·")),
        Check::Bracket => Some(("[x]", "[ ]")),
        Check::Text => None,
    }
}

/// 値の部品の文字と部品(SR-36)。形が plain なら None。
fn parts_of(app: &App, items: Vec<String>, list: bool, part: &mut CellPart) -> Option<String> {
    let t = super::chips::parts(app, &items, list)?.text();
    *part = CellPart::Chips { items, list };
    Some(t)
}

/// 改行を含む値は1行目と `…⏎`(CV-3)。
fn first_line_marked(s: &str) -> String {
    match s.find(['\n', '\r']) {
        Some(i) => format!("{}…⏎", sanitize(&s[..i])),
        None => sanitize(s),
    }
}

fn float_text(f: f64) -> String {
    format!("{f:?}")
}

/// 値の文字(リレーションのリストの要素など、ほかの所から)。
pub(crate) fn value_plain(v: &Value) -> String {
    value_text(v)
}

/// 値の文字(CV-1: null は `∅`、空の文字列は `""`)。
fn value_text(v: &Value) -> String {
    match v {
        Value::Null => "∅".into(),
        Value::Str(s) if s.is_empty() => "\"\"".into(),
        Value::Str(s) => first_line_marked(s),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => float_text(*f),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(value_text).collect::<Vec<_>>().join(", ")
        ),
        Value::Other => "{…}".into(),
    }
}

fn new_value_text(v: &NewValue) -> String {
    match v {
        NewValue::Null => "∅".into(),
        NewValue::Str(s) if s.is_empty() => "\"\"".into(),
        NewValue::Str(s) | NewValue::Date(s) => first_line_marked(s),
        NewValue::Bool(b) => b.to_string(),
        NewValue::Int(i) => i.to_string(),
        NewValue::Float(f) => float_text(*f),
        // CE-29: キーの名前の変更・削除のためる変更(保存までは今の値の代わりに出す)。
        NewValue::RenameKey(to) => format!("→ {}", first_line_marked(to)),
        NewValue::DeleteKey => "×".into(),
        NewValue::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|s| first_line_marked(s))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// 計算の列の値の文字(`file.*`・`formula.*`)。null は空欄。文字列は1行にする(核の `print::val_text_with`)。
pub(crate) fn val_text(v: &Val) -> String {
    mdgrid::print::val_text_with(v, &first_line_marked)
}

/// 列の型の寄せ方(CV-4)。
pub(crate) fn align_of(kind: Kind) -> Align {
    match kind {
        Kind::Number => Align::Right,
        Kind::Checkbox => Align::Center,
        _ => Align::Left,
    }
}

/// 日付の列の値を設定の日付の形で見せる(CE-22。ファイルの `YYYY-MM-DD` は変えない)。日付でなければ None。
fn date_text(app: &App, col: &str, s: &str) -> Option<String> {
    if app.kind_of(col) != Kind::Date {
        return None;
    }
    types::parse_date(s).map(|d| app.date_format.format(d))
}

/// セル1つの見せ方。ためた値が先、ほかは `App::cell`(`.base` があれば Base::cell)から。
pub(crate) fn shown(app: &App, row: &RowId, col: &str) -> Shown {
    if let Some(nv) = app.changes.pending(row, col) {
        let text = match nv {
            NewValue::Str(s) | NewValue::Date(s) => date_text(app, col, s),
            // ためた真偽とリストも、決まった値と同じ形で(`*false`・`*[a, b]` のような生の形にしない)。
            NewValue::Bool(b) if app.rich(col, Part::Checkbox) => {
                check_marks(app.style.check).map(|(on, off)| if *b { on } else { off }.to_string())
            }
            NewValue::List(items) if !items.is_empty() && app.rich(col, Part::Chips) => {
                Some(items.join(" · "))
            }
            _ => None,
        }
        // REL-2: ためたリンクも行き先の名前で見せる。
        .or_else(|| app.link_text_new(row, nv));
        return Shown {
            text: format!("*{}", text.unwrap_or_else(|| new_value_text(nv))),
            null: false,
            pending: true,
            locked: app.src.get(row, col).lock,
            unsupported: None,
            part: CellPart::None,
        };
    }
    let mut part = CellPart::None;
    let (text, null, lock) = match app.cell(row, col) {
        base::Shown::Unsupported(reason) => {
            // BV-7: 薄い `?`。本当の値の `?` とは薄さと、選んだときの下の帯で見分ける。
            return Shown {
                text: UNSUPPORTED_MARK.into(),
                null: true,
                pending: false,
                locked: None,
                unsupported: Some(reason),
                part: CellPart::None,
            };
        }
        base::Shown::Computed(v) => (val_text(&v), false, app.src.get(row, col).lock),
        base::Shown::Prop(cell) => {
            let (text, null) = match &cell.value {
                // キーが無いときは空欄(CV-1)。
                None => (String::new(), false),
                Some(v) => {
                    let date = match v {
                        Value::Str(s) => date_text(app, col, s),
                        _ => None,
                    };
                    // REL-2・REL-10: リンクは行き先の名前で(行き先の無いものは印を付けて)見せる。
                    let link = date.is_none().then(|| app.link_text(row, v)).flatten();
                    let fits = types::fits(app.kind_of(col), v);
                    // SR-35: 部品(型に合う値だけ。リンクと日付は札にしない)。
                    let rich = fits && cell.lock.is_none();
                    let t = if link.is_some() && rich && app.rich(col, Part::Links) {
                        part = CellPart::Link;
                        link
                    } else if let Some(t) = date.or(link) {
                        Some(t)
                    } else if rich {
                        rich_text(app, col, v, &mut part)
                    } else {
                        None
                    }
                    .unwrap_or_else(|| value_text(v));
                    // CV-2: 列の型に合わない値は、そのまま見せて `!` を付ける。
                    let t = if fits { t } else { format!("{MISFIT_MARK}{t}") };
                    (t, matches!(v, Value::Null))
                }
            };
            (text, null, cell.lock)
        }
    };
    // 読むだけのセルは薄い表示に加えて先頭に `#`(色や薄い表示に頼らない。SR-15)。
    // `#` は、ためる変更の `*`・未対応の `?`(BV-7)・型の合わない `!`(CV-2)・負の数の `-` と重ならない。
    let text = match &lock {
        Some(_) => format!("{LOCK_MARK}{text}"),
        None => text,
    };
    Shown {
        text,
        null,
        pending: false,
        locked: lock,
        unsupported: None,
        part,
    }
}

/// SR-35: 真偽・リスト・札の列の値の部品の文字。部品にしなければ None。
fn rich_text(app: &App, col: &str, v: &Value, part: &mut CellPart) -> Option<String> {
    match v {
        Value::Bool(b) if app.rich(col, Part::Checkbox) => {
            let (on, off) = check_marks(app.style.check)?;
            Some(if *b { on } else { off }.to_string())
        }
        Value::List(items) if !items.is_empty() && app.rich(col, Part::Chips) => {
            parts_of(app, items.iter().map(value_text).collect(), true, part)
        }
        // 札を強いた列(`"chip"`)の数も札に。
        Value::Int(_) | Value::Float(_) if app.cells.forced_chip(col) && app.is_select(col) => {
            parts_of(app, vec![value_text(v)], false, part)
        }
        Value::Str(s) if !s.is_empty() && app.is_select(col) => {
            parts_of(app, vec![first_line_marked(s)], false, part)
        }
        _ => None,
    }
}
