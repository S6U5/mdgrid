//! 画面を出さずにビューの表を書き出す(CLI-5)。表の組み立ては画面(`src/ui/grid.rs`)と同じ核の道
//! (`.base` は `Base::build` と `Base::cell`、mdgrid のビューは `views::to_base` から作った `.base` に
//! `settings::apply` を重ねる、既定の表は `base::default_grid`)を通り、画面の見た目の状態(SR-12)と
//! 画面で掛けた絞り込みは使わない。グループ分けは並びだけに効き、見出しの行は出さない。
//! 形は csv(RFC 4180)・json・md。依存を足さずに手で書く。
//! 画面と共有する純関数(`today_now`・`val_text_with`・`val_value`・`kind_in`・`synth`)もここに置く。

use crate::base::{self, Base, Column, Shown};
use crate::expr::Val;
use crate::i18n::Msg;
use crate::settings;
use crate::source::{RowId, Source, Value};
use crate::types::{self, Kind};
use crate::views::{self, NativeView};

/// 書き出しの形(CLI-5)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Csv,
    /// CLI-5: タブ区切り(見出しの行つき、セルの中のタブと改行は空白)。
    Tsv,
    Json,
    Md,
}

impl Format {
    /// OUT-2: ファイル名の拡張子(`.csv`・`.tsv`・`.json`・`.md`。大文字小文字を問わない)の形。
    pub fn from_extension(path: &std::path::Path) -> Option<Format> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "csv" => Some(Format::Csv),
            "tsv" => Some(Format::Tsv),
            "json" => Some(Format::Json),
            "md" => Some(Format::Md),
            _ => None,
        }
    }
}

/// 表を組むビュー。
pub enum ViewDef<'a> {
    /// `.base` なしの既定の表(BV-1)。
    Default,
    /// `.base` とビューの添字。
    Base(&'a Base, usize),
    /// mdgrid のビュー(BV-17)。
    Native(&'a NativeView),
}

/// セル1つ(印を付けない素の値)。
#[derive(Clone, Debug, PartialEq)]
pub enum PrintCell {
    /// ノートのキー(None はキーが無い)。
    Prop(Option<Value>),
    /// `file.*`・`formula.*` の値。
    Computed(Val),
    /// 評価できない式(BV-7)。
    Unsupported,
}

/// 書き出す表: 列(見出しは displayName か id)、行ごとのセル、行の鍵(ノートの実体のパス。`rows` と同じ並び)、
/// 標準エラーに出す警告(列ごとに1行)。
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<PrintCell>>,
    pub row_ids: Vec<RowId>,
    pub warnings: Vec<String>,
}

// ---- 画面と共有する純関数 ----

// 時計(地域の時差・今日と今)は src/clock.rs。今までの道(`print::today_now` など)はここから引ける。
pub use crate::clock::{local_offset, set_local_offset, today_now, today_now_at};

/// 期間(ミリ秒)の見せ方: `1d 2h`・`30m`・`0s`。
fn duration_text(ms: i64) -> String {
    let sign = if ms < 0 { "-" } else { "" };
    // 1秒に満たない長さ0でない期間はミリ秒で(`0d` と見せると、期間 ÷ 数の結果がもっともらしい誤りに見える)。
    if ms != 0 && ms.unsigned_abs() < 1000 {
        return format!("{sign}{}ms", ms.unsigned_abs());
    }
    let mut s = ms.unsigned_abs() / 1000;
    let mut parts = Vec::new();
    for (unit, n) in [("d", 86_400), ("h", 3_600), ("m", 60), ("s", 1)] {
        if s >= n {
            parts.push(format!("{}{unit}", s / n));
            s %= n;
        }
    }
    if parts.is_empty() {
        // 長さ0は、日の差(日付どうしの引き算)の `-7d` などとそろえて日で見せる。
        parts.push("0d".into());
    }
    format!("{sign}{}", parts.join(" "))
}

/// 計算の列の値の文字(`file.*`・`formula.*`)。null は空欄、リストは `[a, b]`。
/// 文字列は `text` で見せる形にする(画面は1行にして制御文字を置き換える。CV-3・SR-9)。
pub fn val_text_with(v: &Val, text: &dyn Fn(&str) -> String) -> String {
    match v {
        Val::Null => String::new(),
        Val::Bool(b) => b.to_string(),
        Val::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
        Val::Num(n) => n.to_string(),
        Val::Str(s) => text(s),
        Val::Date(d) => types::format_date(*d),
        Val::DateTime(t) => {
            let (d, s) = (t.div_euclid(86_400), t.rem_euclid(86_400));
            format!(
                "{}T{:02}:{:02}:{:02}",
                types::format_date(d),
                s / 3600,
                s / 60 % 60,
                s % 60
            )
        }
        Val::Duration(ms) => duration_text(*ms),
        Val::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|x| val_text_with(x, text))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn same(s: &str) -> String {
    s.to_string()
}

/// 計算の列の値をフロントマターの値の形にする(ビューの設定で比べる。NV-20)。日付・日時・期間は見せる文字。
pub fn val_value(v: &Val) -> Value {
    match v {
        Val::Null => Value::Null,
        Val::Bool(b) => Value::Bool(*b),
        Val::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => Value::Int(*n as i64),
        Val::Num(n) => Value::Float(*n),
        Val::Str(s) => Value::Str(s.clone()),
        Val::List(items) => Value::List(items.iter().map(val_value).collect()),
        Val::Date(_) | Val::DateTime(_) | Val::Duration(_) => Value::Str(val_text_with(v, &same)),
    }
}

/// 列の型を `rows` から決める(CV-4・NV-19): ノートのキーは `Source::kind`、`file.size` は数、formula の列は
/// `rows` の順で最初の空でない値(数なら数、真偽ならチェック)、ほかの計算の列はテキスト。
/// `cell` はセルの見せ方の元(画面は `App::cell`)。
pub fn kind_in(
    src: &dyn Source,
    col: &str,
    rows: &[RowId],
    cell: &dyn Fn(&RowId, &str) -> Shown,
) -> Kind {
    if col == "file.size" {
        Kind::Number
    } else if col.starts_with("formula.") {
        formula_kind(col, rows, cell)
    } else if col.starts_with("file.") {
        Kind::Text
    } else {
        src.kind(col).kind
    }
}

fn formula_kind(col: &str, rows: &[RowId], cell: &dyn Fn(&RowId, &str) -> Shown) -> Kind {
    for row in rows {
        match cell(row, col) {
            Shown::Computed(Val::Null) => continue,
            Shown::Computed(Val::Str(s)) if s.is_empty() => continue,
            Shown::Computed(Val::List(l)) if l.is_empty() => continue,
            Shown::Computed(Val::Num(_)) => return Kind::Number,
            Shown::Computed(Val::Bool(_)) => return Kind::Checkbox,
            Shown::Computed(_) => return Kind::Text,
            _ => continue,
        }
    }
    Kind::Text
}

/// mdgrid のビューの列の並びと式の絞り込みを `.base` にする(ビューの設定は表の側で重ねるので入れない)。
pub fn synth(nv: &NativeView) -> Result<Base, String> {
    let v = NativeView {
        name: nv.name.clone(),
        order: nv.order.clone(),
        filters_expr: nv.filters_expr.clone(),
        ..Default::default()
    };
    base::parse(&views::to_base(&v).0)
}

// ---- 表の組み立て ----

/// ビューの表を組む(CLI-5)。ノートは読み終えていること。評価できない filters(BV-7)・table 以外は Err(理由)。
pub fn table(src: &dyn Source, view: ViewDef, today: i64, now: i64) -> Result<Table, String> {
    let prop = |r: &RowId, k: &str| src.get(r, k).value;
    match view {
        ViewDef::Default => {
            let g = base::default_grid(src);
            let cell = |r: &RowId, c: &str| Shown::Prop(src.get(r, c));
            Ok(collect(g.columns, &g.rows, g.notes, &cell))
        }
        ViewDef::Base(b, i) => {
            if b.views.is_empty() {
                return Err(Msg::PrintNoViews.text().into());
            }
            let g = b.build(i, src, &prop, today, now)?;
            let cell = |r: &RowId, c: &str| b.cell(src, &prop, r, c, today, now);
            Ok(collect(g.columns, &g.rows, g.notes, &cell))
        }
        ViewDef::Native(nv) => {
            let b = synth(nv).map_err(|e| Msg::PrintNativeView.fill(&[&nv.name, &e]))?;
            let g = b
                .build(0, src, &prop, today, now)
                .map_err(|e| Msg::PrintNativeView.fill(&[&nv.name, &e]))?;
            let cell = |r: &RowId, c: &str| b.cell(src, &prop, r, c, today, now);
            // 列は定義の並び(空ならノートのキー)から隠す列を除いたもの。
            let ids: Vec<String> = if nv.order.is_empty() {
                src.columns()
            } else {
                nv.order.clone()
            };
            let columns: Vec<Column> = ids
                .into_iter()
                .filter(|id| !nv.hidden.contains(id))
                .map(|id| {
                    let title = g
                        .columns
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.title.clone())
                        .unwrap_or_else(|| base::default_title(&id));
                    Column { id, title }
                })
                .collect();
            // NV-20: ビューの設定(絞る・並べる・まとめる)を重ねる。型は `.base` の結果の行の全部から決める。
            let rows = if nv.settings.no_conditions() {
                g.rows
            } else {
                let get = |r: &RowId, c: &str| match cell(r, c) {
                    Shown::Computed(v) => Some(val_value(&v)),
                    Shown::Unsupported(_) => None,
                    Shown::Prop(p) => p.value,
                };
                let all = g.rows.clone();
                let kind = |c: &str| kind_in(src, c, &all, &cell);
                settings::apply(g.rows, g.groups, &nv.settings, &get, &kind).0
            };
            Ok(collect(columns, &rows, g.notes, &cell))
        }
    }
}

/// 行と列からセルを集め、警告をまとめる(組み立ての説明と、評価できなかった列ごとに1行)。
fn collect(
    columns: Vec<Column>,
    rows_in: &[RowId],
    notes: Vec<String>,
    cell: &dyn Fn(&RowId, &str) -> Shown,
) -> Table {
    let mut warnings = notes;
    let mut bad: Vec<(String, String)> = Vec::new();
    let rows = rows_in
        .iter()
        .map(|r| {
            columns
                .iter()
                .map(|c| match cell(r, &c.id) {
                    Shown::Prop(p) => PrintCell::Prop(p.value),
                    Shown::Computed(v) => PrintCell::Computed(v),
                    Shown::Unsupported(why) => {
                        if !bad.iter().any(|(id, _)| *id == c.id) {
                            bad.push((c.id.clone(), why));
                        }
                        PrintCell::Unsupported
                    }
                })
                .collect()
        })
        .collect();
    for (id, why) in bad {
        let told = warnings.iter().any(|w| w.starts_with(&format!("{id}:")));
        if !told {
            warnings.push(format!("{id}: {why}"));
        }
    }
    let warnings = warnings.into_iter().map(|w| w.replace('\n', " ")).collect();
    Table {
        columns,
        rows,
        row_ids: rows_in.to_vec(),
        warnings,
    }
}

// ---- 書き出し ----

/// 表を形に書く(改行は LF、最後の行も LF で終わる)。
pub fn render(t: &Table, f: Format) -> String {
    match f {
        Format::Csv => csv(t),
        Format::Tsv => tsv(t),
        Format::Json => json(t),
        Format::Md => md(t),
    }
}

/// 素の文字(印を付けない。null・空・キー無し・評価できないは空、リストは `, ` でつなぐ、日付は YYYY-MM-DD)。
/// 画面の `--pick <列>`(OUT-3)も同じ決まりで出す(改行は `one_line` で空白に)。
pub fn plain(c: &PrintCell) -> String {
    match c {
        PrintCell::Prop(None) | PrintCell::Unsupported => String::new(),
        PrintCell::Prop(Some(v)) => value_plain(v),
        PrintCell::Computed(v) => val_plain(v),
    }
}

pub fn value_plain(v: &Value) -> String {
    match v {
        Value::Null | Value::Other => String::new(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{f:?}"),
        Value::List(items) => items.iter().map(value_plain).collect::<Vec<_>>().join(", "),
    }
}

fn val_plain(v: &Val) -> String {
    match v {
        Val::List(items) => items.iter().map(val_plain).collect::<Vec<_>>().join(", "),
        _ => val_text_with(v, &same),
    }
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv(t: &Table) -> String {
    let mut out = String::new();
    let line = |out: &mut String, cells: Vec<String>| {
        // 1列だけで値が空の行は空行になり読み手に落とされるので、`""` と括る。
        if let [only] = cells.as_slice() {
            if only.is_empty() {
                out.push_str("\"\"\n");
                return;
            }
        }
        let v: Vec<String> = cells.iter().map(|s| csv_field(s)).collect();
        out.push_str(&v.join(","));
        out.push('\n');
    };
    line(
        &mut out,
        t.columns.iter().map(|c| c.title.clone()).collect(),
    );
    for r in &t.rows {
        line(&mut out, r.iter().map(plain).collect());
    }
    out
}

/// TSV のセル: タブと改行を空白にする(区切りと行の終わりにしない)。
fn tsv_cell(s: &str) -> String {
    one_line(s).replace('\t', " ")
}

fn tsv(t: &Table) -> String {
    let mut out = String::new();
    let head: Vec<String> = t.columns.iter().map(|c| tsv_cell(&c.title)).collect();
    out.push_str(&head.join("\t"));
    out.push('\n');
    for r in &t.rows {
        let v: Vec<String> = r.iter().map(|c| tsv_cell(&plain(c))).collect();
        out.push_str(&v.join("\t"));
        out.push('\n');
    }
    out
}

/// 改行(CRLF・LF・CR)を空白1つにする(md のセルと `--pick <列>` の1行)。
pub fn one_line(s: &str) -> String {
    s.replace("\r\n", " ").replace(['\n', '\r'], " ")
}

fn md_cell(s: &str) -> String {
    // 逃がしの `\` を先に重ね、そのあと縦棒を逃がす(値の `\|` が `\\|` になって縦棒が逃げなくならない)。
    one_line(&s.replace('\\', "\\\\")).replace('|', "\\|")
}

fn md(t: &Table) -> String {
    let mut out = String::new();
    let line = |out: &mut String, cells: Vec<String>| {
        out.push('|');
        for c in cells {
            out.push(' ');
            out.push_str(&c);
            out.push_str(" |");
        }
        out.push('\n');
    };
    line(
        &mut out,
        t.columns.iter().map(|c| md_cell(&c.title)).collect(),
    );
    line(
        &mut out,
        t.columns.iter().map(|_| "---".to_string()).collect(),
    );
    for r in &t.rows {
        line(&mut out, r.iter().map(|c| md_cell(&plain(c))).collect());
    }
    out
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_float(f: f64) -> String {
    if f.is_finite() {
        format!("{f:?}")
    } else {
        "null".into()
    }
}

pub fn value_json(v: &Value) -> String {
    match v {
        Value::Null | Value::Other => "null".into(),
        Value::Str(s) => json_str(s),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => json_float(*f),
        Value::List(items) => format!(
            "[{}]",
            items.iter().map(value_json).collect::<Vec<_>>().join(", ")
        ),
    }
}

fn val_json(v: &Val) -> String {
    match v {
        Val::Null => "null".into(),
        Val::Bool(b) => b.to_string(),
        Val::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
        Val::Num(n) => json_float(*n),
        Val::Str(s) => json_str(s),
        Val::List(items) => format!(
            "[{}]",
            items.iter().map(val_json).collect::<Vec<_>>().join(", ")
        ),
        Val::Date(_) | Val::DateTime(_) | Val::Duration(_) => json_str(&val_text_with(v, &same)),
    }
}

fn cell_json(c: &PrintCell) -> String {
    match c {
        PrintCell::Prop(None) | PrintCell::Unsupported => "null".into(),
        PrintCell::Prop(Some(v)) => value_json(v),
        PrintCell::Computed(v) => val_json(v),
    }
}

/// JSON の鍵: 見出し(displayName か id)。見出しが前の列と重なれば `見出し (id)`、それでも重なれば
/// 後ろに `#列の番号`(1から)を添え、同じオブジェクトに同じ鍵を2つ出さない。
fn json_keys(columns: &[Column]) -> Vec<String> {
    let mut keys: Vec<String> = Vec::with_capacity(columns.len());
    for (i, c) in columns.iter().enumerate() {
        let mut k = c.title.clone();
        if keys.contains(&k) {
            k = format!("{} ({})", c.title, c.id);
        }
        if keys.contains(&k) {
            k = format!("{k} #{}", i + 1);
        }
        keys.push(k);
    }
    keys
}

fn json(t: &Table) -> String {
    if t.rows.is_empty() {
        return "[]\n".into();
    }
    let keys = json_keys(&t.columns);
    let items: Vec<String> = t
        .rows
        .iter()
        .map(|r| {
            let kv: Vec<String> = keys
                .iter()
                .zip(r)
                .map(|(k, v)| format!("{}: {}", json_str(k), cell_json(v)))
                .collect();
            format!("  {{{}}}", kv.join(", "))
        })
        .collect();
    format!("[\n{}\n]\n", items.join(",\n"))
}

#[cfg(test)]
#[path = "test_today_unit.rs"]
mod test_today_unit;
