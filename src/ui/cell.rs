//! セルの見せ方の純関数(CV-1・CV-2・CV-3・CV-4・BV-7・SR-15・CE-22)。状態は書き換えない。

use super::app::App;
use super::width::{sanitize, take, width, wrap, Align};
use mdgrid::base;
use mdgrid::cells::Part;
use mdgrid::expr::Val;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::{self, Kind};
use std::cell::Cell;

/// 読むだけのセルの印(SR-15)。
pub(crate) const LOCK_MARK: char = '#';
/// 型の合わない値の印(CV-2)。
pub(crate) const MISFIT_MARK: char = '!';
/// `MISFIT_MARK` の文字列。
const MISFIT: &str = "!";
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
    /// 値の部品の並び(SR-36 の形。text は印 `mark` と chips::parts_named の文字)。`list` はリストの列の値。
    Chips {
        /// (値, 見せる文字)。部品の色と形は値で、文字は見せる文字(名前のある値は名前。navigation.value-labels-cells)。
        items: Vec<(String, String)>,
        list: bool,
        /// 部品の前に出すセルの印(決まりに合わない `!` など。cell-view.marks-on-parts)。
        mark: String,
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

/// 値の部品の文字と部品(SR-36)。形が plain なら None。印(`mark`)は呼ぶ側が決めたあとに入れる。
fn parts_of(
    app: &App,
    items: Vec<(String, String)>,
    list: bool,
    part: &mut CellPart,
) -> Option<String> {
    let t = super::chips::parts_named(app, &items, list)?.text();
    *part = CellPart::Chips {
        items,
        list,
        mark: String::new(),
    };
    Some(t)
}

/// 決まりの values と比べる値の文字(rules の比べ方と同じ。真偽はチェックボックスで見せるので名前を当てない)。
fn scalar_key(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) if !s.is_empty() => Some(s.clone()),
        Value::Int(i) => Some(i.to_string()),
        Value::Float(f) => Some(float_text(*f)),
        _ => None,
    }
}

/// navigation.value-labels-cells: 列 `col` の値の文字 `key` の見せる文字(`[rules.<列>] icons`・`labels` の「字 名前」。
/// 1行にして無害にしたもの)。名前も字も無い値と、決まりの values に無い値は None。
pub(crate) fn label_of(app: &App, col: &str, key: &str) -> Option<String> {
    app.prof
        .resolved
        .value_shown(col, key)
        .map(|t| first_line_marked(&t))
}

/// cell-view.number-format: 列 `col` に `[format.<列>]` があれば、数の値を見せる文字(桁の区切り・小数の桁・頭と
/// 末尾の字)。名前のある値(`[rules.<列>] labels`・`icons`)は名前が先なので None。書く値・並べ替え・書き出しは
/// 数のまま(cell-view.number-format-display-only)。
pub(crate) fn number_text(app: &App, col: &str, v: &Value) -> Option<String> {
    let f = app.prof.resolved.format.get(col)?;
    let (key, text) = match v {
        Value::Int(i) => (i.to_string(), f.int(*i)),
        Value::Float(x) => (float_text(*x), f.float(*x)),
        _ => return None,
    };
    // 頭と末尾の字は読むときに外しているが、描くときも制御の文字を見える字にする(二重の守り)。
    label_of(app, col, &key).is_none().then(|| sanitize(&text))
}

/// `number_text` の計算の列(`formula.*`・`file.*`)の値の版。数でなければ None。
pub(crate) fn number_val_text(app: &App, col: &str, v: &Val) -> Option<String> {
    match v {
        Val::Num(n) => Some(sanitize(&app.prof.resolved.format.get(col)?.float(*n))),
        _ => None,
    }
}

/// 値の文字(CV-1)に、名前のある値は名前を当てたもの(リストは要素ごと。navigation.value-labels-cells)。
pub(crate) fn labeled_text(app: &App, col: &str, v: &Value) -> String {
    match v {
        Value::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|x| labeled_text(app, col, x))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => scalar_key(v)
            .and_then(|k| label_of(app, col, &k))
            .unwrap_or_else(|| value_text(v)),
    }
}

/// ためた値の文字に、名前のある値は名前を当てたもの(navigation.value-labels-cells)。
fn labeled_new_text(app: &App, col: &str, v: &NewValue) -> String {
    let named = |k: String| label_of(app, col, &k);
    match v {
        NewValue::Str(s) if !s.is_empty() => named(s.clone()),
        NewValue::Int(i) => named(i.to_string()),
        NewValue::Float(f) => named(float_text(*f)),
        NewValue::List(items) => Some(format!(
            "[{}]",
            items
                .iter()
                .map(|s| named(s.clone()).unwrap_or_else(|| first_line_marked(s)))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        _ => None,
    }
    .unwrap_or_else(|| new_value_text(v))
}

/// 値を見せる所(木の画面の右の欄など)の素の文字: `App::plain` に名前を当てたもの(リストは要素を `, ` でつなぐ。
/// navigation.value-labels-cells)。比べる・書く所は `plain` のまま使う。
pub(crate) fn shown_plain(app: &App, row: &RowId, col: &str) -> String {
    let one = |k: &str| label_of(app, col, k).unwrap_or_else(|| k.to_string());
    if let Some(nv) = app.changes.pending(row, col) {
        return match nv {
            NewValue::List(items) => items.iter().map(|s| one(s)).collect::<Vec<_>>().join(", "),
            _ => one(&app.plain(row, col)),
        };
    }
    let scalar = |x: &Value| {
        scalar_key(x)
            .and_then(|k| label_of(app, col, &k))
            .unwrap_or_else(|| mdgrid::print::value_plain(x))
    };
    match app.cell(row, col) {
        base::Shown::Prop(c) => match &c.value {
            Some(Value::List(items)) => items.iter().map(scalar).collect::<Vec<_>>().join(", "),
            Some(v) => scalar(v),
            None => String::new(),
        },
        _ => app.plain(row, col),
    }
}

thread_local! {
    /// 改行を残すか(CV-8。伸ばした表の行の文字を作る間だけ。`cell_lines` が置いて戻す)。
    static KEEP_NEWLINES: Cell<bool> = const { Cell::new(false) };
}

/// 改行を含む値は1行目と `…⏎`(CV-3)。伸ばした表の行の文字を作る間(CV-8)は、行ごとに整えて改行を残す。
fn first_line_marked(s: &str) -> String {
    if KEEP_NEWLINES.with(Cell::get) {
        return s
            .replace("\r\n", "\n")
            .split(['\n', '\r'])
            .map(sanitize)
            .collect::<Vec<_>>()
            .join("\n");
    }
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

/// cell-view.lookup-shown: 式の列の値の文字。ノートは名前(ノートの欄と同じ名前の正規表現を当てたもの)、日付は日付の
/// 見せ方(CE-22)、ほかのノートのキー `key` から引いた値は、そのキーの値の名前(navigation.value-labels-cells)。
/// 式の列のセルの文字: 値の名前(labels)のある値は名前、無ければ数の見せ方(cell-view.number-format)、どちらも
/// 無ければ引いた値の見せ方(cell-view.lookup-shown)。
fn computed_cell_text(app: &App, col: &str, v: &Val) -> String {
    let key = app.lookup_key(col);
    let labeled = key
        .as_deref()
        .is_some_and(|k| label_of(app, k, &mdgrid::print::val_plain(v)).is_some());
    if labeled {
        return computed_text(app, key.as_deref(), v);
    }
    number_val_text(app, col, v).unwrap_or_else(|| computed_text(app, key.as_deref(), v))
}

fn computed_text(app: &App, key: Option<&str>, v: &Val) -> String {
    match v {
        Val::File(stem) => {
            let name = stem.rsplit('/').next().unwrap_or(stem);
            sanitize(&super::view::note_name_of(app, name))
        }
        Val::Date(d) => app.date_format.format(*d),
        Val::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|x| computed_text(app, key, x))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Val::Str(_) | Val::Num(_) | Val::Bool(_) => {
            let text = val_text(v);
            key.and_then(|k| label_of(app, k, &mdgrid::print::val_plain(v)))
                .unwrap_or(text)
        }
        other => val_text(other),
    }
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
            NewValue::List(items) if !items.is_empty() && app.rich(col, Part::Chips) => Some(
                items
                    .iter()
                    .map(|s| label_of(app, col, s).unwrap_or_else(|| s.clone()))
                    .collect::<Vec<_>>()
                    .join(" · "),
            ),
            // cell-view.number-format: ためた数も見せ方を当てる(`*¥5,000`)。
            NewValue::Int(i) => number_text(app, col, &Value::Int(*i)),
            NewValue::Float(f) => number_text(app, col, &Value::Float(*f)),
            _ => None,
        }
        // REL-2: ためたリンクも行き先の名前で見せる。
        .or_else(|| app.link_text_new(row, nv));
        // CV-9: ためた値も決まりに合わなければ `!` を(保存の前に気づけるよう)。
        let bad = if app.problem(row, col).is_some() {
            "!"
        } else {
            ""
        };
        return Shown {
            text: format!(
                "*{bad}{}",
                text.unwrap_or_else(|| labeled_new_text(app, col, nv))
            ),
            null: false,
            pending: true,
            locked: app.src.get(row, col).lock,
            unsupported: None,
            part: CellPart::None,
        };
    }
    let mut part = CellPart::None;
    // `mark` は値の前の印(型か決まりに合わない `!`。CV-2・CV-9)。部品のセルでも落とさない(cell-view.marks-on-parts)。
    let (text, mark, null, lock) = match app.cell(row, col) {
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
        base::Shown::Computed(v) => (
            computed_cell_text(app, col, &v),
            "",
            false,
            app.src.get(row, col).lock,
        ),
        base::Shown::Prop(cell) => {
            let (text, mark, null) = match &cell.value {
                // キーが無いときは空欄(CV-1)。必須の決まり(CE-36)があれば `!` だけ(CV-9)。
                None if app.problem(row, col).is_some() => (String::new(), MISFIT, false),
                None => (String::new(), "", false),
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
                    // cell-view.number-format: 数の見せ方(名前のある値は名前が先)。
                    .or_else(|| number_text(app, col, v))
                    // navigation.value-labels-cells: 名前のある値は名前で(書く値は値のまま)。
                    .unwrap_or_else(|| labeled_text(app, col, v));
                    // CV-2・CV-9: 列の型か決まりに合わない値は、そのまま見せて `!` を付ける。
                    let bad = !fits || app.problem(row, col).is_some();
                    (t, if bad { MISFIT } else { "" }, matches!(v, Value::Null))
                }
            };
            (text, mark, null, cell.lock)
        }
    };
    // 読むだけのセルは薄い表示に加えて先頭に `#`(色や薄い表示に頼らない。SR-15)。
    // `#` は、ためる変更の `*`・未対応の `?`(BV-7)・型の合わない `!`(CV-2)・負の数の `-` と重ならない。
    // cell-view.formula-empty-mark: 式の列の空の値には付けず、空のセルにする(`#` だけが残ると値があるように見える)。
    // cell-view.formula-mark-no-color: 式の列は、色のある表示では付けない(薄い表示と見出しの ƒ で見分ける。
    // 色を使わない表示では薄い表示が出ないので付ける)。
    // 読むだけの理由(`locked`)は残す(選んだときのメッセージ行と、直せないことは同じ)。
    let formula = col.starts_with("formula.");
    let bare = text.is_empty() && formula;
    let quiet = formula && app.color != super::app::ColorMode::None;
    let lead = match &lock {
        Some(_) if !bare && !quiet => format!("{LOCK_MARK}{mark}"),
        _ => mark.to_string(),
    };
    if let CellPart::Chips { mark, .. } = &mut part {
        mark.clone_from(&lead);
    }
    Shown {
        text: format!("{lead}{text}"),
        null,
        pending: false,
        locked: lock,
        unsupported: None,
        part,
    }
}

/// CV-8: セル `s`(`shown` の結果)の文字を幅 `cw` で折り返した行。最大 `max` 行で、入らなければ最後の行の
/// 後ろに `…`。改行は改行で分ける(CV-3)。札(部品の並び)と、列の幅に入り改行の無い文字は1行のまま。
pub(crate) fn cell_lines(
    app: &App,
    s: &Shown,
    row: &RowId,
    col: &str,
    cw: usize,
    max: usize,
) -> Vec<String> {
    let one = max <= 1
        || matches!(s.part, CellPart::Chips { .. })
        || (width(&s.text) <= cw && !s.text.contains('⏎'));
    if one {
        return vec![s.text.clone()];
    }
    KEEP_NEWLINES.with(|k| k.set(true));
    let full = shown(app, row, col).text;
    KEEP_NEWLINES.with(|k| k.set(false));
    let mut out = wrap(full.trim_end_matches('\n'), cw);
    if out.len() > max {
        out.truncate(max);
        let last = out.pop().unwrap_or_default();
        let ell = width("…");
        out.push(format!("{}…", take(&last, cw.saturating_sub(ell))));
    }
    out
}

/// SR-35: 真偽・リスト・札の列の値の部品の文字。部品にしなければ None。部品の文字は、名前のある値は名前
/// (色と形は値のまま。navigation.value-labels-cells)。
fn rich_text(app: &App, col: &str, v: &Value, part: &mut CellPart) -> Option<String> {
    let pair = |x: &Value| {
        let t = value_text(x);
        let shown = scalar_key(x)
            .and_then(|k| label_of(app, col, &k))
            .unwrap_or_else(|| t.clone());
        (t, shown)
    };
    match v {
        Value::Bool(b) if app.rich(col, Part::Checkbox) => {
            let (on, off) = check_marks(app.style.check)?;
            Some(if *b { on } else { off }.to_string())
        }
        Value::List(items) if !items.is_empty() && app.rich(col, Part::Chips) => {
            parts_of(app, items.iter().map(pair).collect(), true, part)
        }
        // 札を強いた列(`"chip"`)の数も札に。
        Value::Int(_) | Value::Float(_) if app.cells.forced_chip(col) && app.is_select(col) => {
            parts_of(app, vec![pair(v)], false, part)
        }
        Value::Str(s) if !s.is_empty() && app.is_select(col) => {
            parts_of(app, vec![pair(v)], false, part)
        }
        _ => None,
    }
}
