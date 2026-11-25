//! グラフの画面(specs/_history/decidespec/chart/spec.md の CH-1〜CH-9。`impl App` の続き): 今のビューの行を、分ける列の値ごとに数えるか
//! 数の列を集めて、棒・積み上げ棒・折れ線・散布図・ヒートマップで見せる。表の画面とは別の画面の型(SC-9)で、
//! 画面の型のメニュー(REL-8)とキー `P` で切り替える。設定は印の `chart`(CH-1。1つか配列)、無ければ列から推定し
//! (CH-2)、種類と列は画面の中のキーでも切り替える(CH-8。印は書き換えない)。数え方は画面に依存しない
//! `mdgrid::chart`(CH-3)。
//!
//! 読むだけの画面で、値は直さない。項目の `Enter` は、頻度表(NV-9)と同じ同じ値の絞り込みで表に戻る。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use super::relmap::panel;
use super::width::{fit, sanitize, width, Align};
use mdgrid::chart::{self, Acc, Agg, ChartKind, ChartSpec, ColInfo};
use mdgrid::gantt::{self, Unit};
use mdgrid::i18n::Msg;
use mdgrid::settings::value_keys;
use mdgrid::source::RowId;
use mdgrid::theme::{indexed_rgb, to_indexed};
use mdgrid::types::{format_date, parse_date, Kind, WeekStart};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;

/// 日付の区切りの数の上限。超える単位は月まで粗くする(極端な日付の範囲で項目が膨らまないように)。
const MAX_BUCKETS: usize = 2000;
/// 内訳の数の上限(積み上げ棒・折れ線・散布図。CH-4・CH-5)。残りは「ほか」か、線にしない。
const MAX_SERIES: usize = 5;
/// ヒートマップの横の値の数の上限(残りは「ほか」)。
const MAX_HEAT_COLS: usize = 12;
/// 内訳ごとの塗りの字(積み上げ棒。最後は「ほか」)。
const FILLS: [char; 6] = ['█', '▓', '▒', '░', '▞', ':'];
const FILLS_ASCII: [char; 6] = ['#', '=', '+', '-', '~', ':'];
/// 内訳ごとの点の字(折れ線・散布図。最後は「ほか」)。
const MARKS: [char; 6] = ['*', '+', 'o', 'x', '#', '.'];
/// 濃さの字(ヒートマップ。0 と 4 段)。
const SHADES: [char; 5] = [' ', '░', '▒', '▓', '█'];
const SHADES_ASCII: [char; 5] = [' ', '.', ':', '+', '#'];
/// 棒の端の 8 分の 1 の字(1〜7)。
const EIGHTHS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];

/// 印の鍵と推定で全部決めたグラフ。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Resolved {
    pub name: Option<String>,
    pub kind: ChartKind,
    pub x: String,
    /// 値の列(None は件数)。
    pub y: Option<String>,
    /// 集め方(値の列が無ければ件数)。
    pub agg: Agg,
    pub by: Option<String>,
    /// 日付の区切りの単位(None は開くときに収まる最も細かい単位)。
    pub unit: Option<Unit>,
    pub cumulative: bool,
}

/// 分ける列の分け方。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Axis {
    /// 値ごと(文字・チェックボックス・リスト)。
    Values,
    /// 日付の区切り(最初の区切りの始まりの日・単位)。
    Dates { origin: i64, unit: Unit },
    /// 数の区間。
    Bins,
}

/// 項目1つ(棒の1本・折れ線の区切り・ヒートマップの1行)。
pub(crate) struct Item {
    pub label: String,
    /// 絞り込みの値の鍵(値ごとの項目だけ。空の文字は値の無い行)。日付と数の区切りは None。
    pub key: Option<String>,
    pub total: Acc,
    /// 内訳ごとの数え(`Data::series` と同じ順)。
    pub parts: Vec<Acc>,
}

/// 内訳の値1つ。
pub(crate) struct Series {
    pub label: String,
    /// 絞り込みの値の鍵(「ほか」は None)。
    pub key: Option<String>,
}

/// 行の分ける値(値ごとの鍵か、日付・数)。
enum XV {
    Keys(Vec<Option<String>>),
    Num(f64),
}

/// 数える行1つ: (行, 分ける値, 内訳の鍵, 値の列の数)。
type RowVals<'a> = (&'a RowId, XV, Vec<Option<String>>, Option<f64>);

/// 散布図の点。
pub(crate) struct Point {
    pub row: RowId,
    /// 行の名前(表のノートの欄と同じ見せ方)。
    pub name: String,
    pub x: f64,
    pub y: f64,
    /// 内訳の番号(`Data::series`。内訳が無ければ 0)。
    pub s: usize,
}

/// 数えた結果。
pub(crate) struct Data {
    pub items: Vec<Item>,
    pub series: Vec<Series>,
    /// 折れ線で線にしなかった内訳の数。
    pub dropped: usize,
    pub points: Vec<Point>,
    /// 数えた行の数(外した行を除く)。
    pub rows: usize,
    /// 外した行の数(分ける列か値の列が読めない)。
    pub skipped: usize,
    pub axis: Axis,
    /// 横の軸が日付の列か(散布図の目盛りの書き方)。
    pub x_date: bool,
    /// 散布図の縦が日付の列か。
    pub y_date: bool,
    /// 描けない理由(種類に合う列が無いなど)。あれば体にこれだけを出す。
    pub why: Option<String>,
    /// cell-view.number-format-totals: 値の字の見せ方(値の列の `[format.<列>]`。件数と散布図は None)。
    pub fmt: Option<mdgrid::numfmt::NumFormat>,
}

/// グラフの画面の状態(開いている間だけ)。
pub(crate) struct Chart {
    pub cur: Resolved,
    /// 印のグラフの番号(使える印のグラフの並びの中。印に無ければ 0)。
    pub idx: usize,
    /// 推定で埋めた列があるか。
    pub inferred: bool,
    pub data: Data,
    /// 選んでいる項目(棒の1本・ヒートマップの行・折れ線の区切り・散布図の点)。
    pub sel: usize,
    /// ヒートマップの選んでいる横の値。
    pub sel2: usize,
    /// 割合(積み上げ棒)。
    pub pct: bool,
    /// 縦に見せる最初の項目(棒・積み上げ棒・ヒートマップの行。選びが見える所まで、動いたときだけずらす)。
    pub top: usize,
    /// ヒートマップの横に見せる最初の値。
    pub left: usize,
}

/// 描く字の組(枠の文字が ASCII のときは ASCII。CV-6・SR-32)。
struct Glyphs {
    ascii: bool,
    vbar: char,
    corner: char,
    hline: char,
}

fn glyphs(app: &App) -> Glyphs {
    let f = super::popup::frame(app);
    let ascii = f.v == '|';
    Glyphs {
        ascii,
        vbar: if ascii { '|' } else { '│' },
        corner: if ascii { '+' } else { '└' },
        hline: if ascii { '-' } else { '─' },
    }
}

/// 種類の名前(画面の言語)。
pub(crate) fn kind_text(k: ChartKind) -> &'static str {
    match k {
        ChartKind::Bar => Msg::ChartKindBar.text(),
        ChartKind::Stacked => Msg::ChartKindStacked.text(),
        ChartKind::Line => Msg::ChartKindLine.text(),
        ChartKind::Scatter => Msg::ChartKindScatter.text(),
        ChartKind::Heatmap => Msg::ChartKindHeatmap.text(),
    }
}

fn agg_text(a: Agg) -> &'static str {
    match a {
        Agg::Count => Msg::ChartCount.text(),
        Agg::Sum => Msg::ChartAggSum.text(),
        Agg::Avg => Msg::ChartAggAvg.text(),
        Agg::Min => Msg::ChartAggMin.text(),
        Agg::Max => Msg::ChartAggMax.text(),
    }
}

fn unit_text(u: Unit) -> &'static str {
    match u {
        Unit::Day => Msg::GanttUnitDay.text(),
        Unit::Week => Msg::GanttUnitWeek.text(),
        Unit::Month => Msg::GanttUnitMonth.text(),
    }
}

/// 値の文(件数・<列> の合計など)。
fn value_text(app: &App, r: &Resolved) -> String {
    match (&r.y, r.agg) {
        (Some(y), a) if a != Agg::Count => {
            Msg::ChartValueOf.fill(&[&sanitize(&app.title(y)), &agg_text(a)])
        }
        _ => Msg::ChartCount.text().to_string(),
    }
}

/// 窓の題(CH-2)。列の名前は列の見出しと同じ displayName(base-view.display-name-everywhere)。
fn title(app: &App, c: &Chart) -> String {
    let r = &c.cur;
    let col = |id: &str| sanitize(&app.title(id));
    let mut t = Msg::ChartTitlePrefix.text().to_string();
    if let Some(n) = &r.name {
        t.push_str(&Msg::ChartTitleNamed.fill(&[&sanitize(n)]));
    }
    if r.kind == ChartKind::Scatter {
        let y = r.y.as_deref().map_or_else(|| "-".to_string(), col);
        t.push_str(&Msg::ChartTitleScatter.fill(&[&kind_text(r.kind), &col(&r.x), &y]));
    } else {
        t.push_str(&Msg::ChartTitle.fill(&[&kind_text(r.kind), &col(&r.x), &value_text(app, r)]));
    }
    if let (true, Some(b)) = (r.kind.takes_by(), &r.by) {
        t.push_str(&Msg::ChartTitleBy.fill(&[&col(b)]));
    }
    // 日付の区切りの単位(散布図は区切らない)。
    if let (Axis::Dates { unit, .. }, false) = (&c.data.axis, r.kind == ChartKind::Scatter) {
        if c.data.why.is_none() && !c.data.items.is_empty() {
            t.push_str(&Msg::ChartTitleUnit.fill(&[&unit_text(*unit)]));
        }
    }
    if r.kind == ChartKind::Line && r.cumulative {
        t.push_str(Msg::ChartTitleCumulative.text());
    }
    if r.kind == ChartKind::Stacked && c.pct {
        t.push_str(Msg::ChartTitlePercent.text());
    }
    if c.inferred {
        t.push_str(Msg::ChartTitleInferred.text());
    }
    t
}

/// 数の値の文字(`fmt_num`。値なしは `-`)。
fn num(v: Option<f64>) -> String {
    v.map_or_else(|| "-".to_string(), chart::fmt_num)
}

/// 値の字(cell-view.number-format-totals): 値の列に数の見せ方があれば当てる。小数の桁を決めていない見せ方は、
/// `fmt_num` と同じく小数2桁までに丸めてから当てる。
fn vnum(c: &Chart, v: Option<f64>) -> String {
    match (&c.data.fmt, v) {
        (Some(f), Some(x)) if x.is_finite() => {
            let x = if f.decimals.is_none() && x.abs() < 1e15 {
                (x * 100.0).round() / 100.0
            } else {
                x
            };
            // 頭と末尾の字の制御の文字は見える字にする(cell-view.number-format-invalid)。
            sanitize(&f.float(x))
        }
        _ => num(v),
    }
}

/// 画面の地の色(chart.series-contrast・chart.heatmap-readable): テーマの地、無ければ端末の地(問い合わせた答え)、
/// 分からなければ暗い地(黒。screen.theme-auto-detect と同じく暗いとみなす)。
fn ground(app: &App) -> [u8; 3] {
    app.palette()
        .map(|p| p.bg)
        .or(app.term_bg)
        .unwrap_or([0, 0, 0])
}

/// 明るい地か(テーマの明暗の判断 `Theme::is_light` と同じ)。
fn is_light(c: [u8; 3]) -> bool {
    c.iter().map(|x| u32::from(*x)).sum::<u32>() > 3 * 128
}

/// 端末に出る色(256 色では、近い番号の色)。色の対比は出る色で比べる。
fn shown(c: [u8; 3], indexed: bool) -> [u8; 3] {
    if indexed {
        indexed_rgb(to_indexed(c)).unwrap_or(c)
    } else {
        c
    }
}

/// 端末の色にする。
fn term_color(c: [u8; 3], indexed: bool) -> Color {
    if indexed {
        Color::Indexed(to_indexed(c))
    } else {
        Color::Rgb(c[0], c[1], c[2])
    }
}

/// WCAG の相対輝度。
fn luminance(c: [u8; 3]) -> f64 {
    let lin = |v: u8| {
        let x = f64::from(v) / 255.0;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// WCAG の色の対比(1〜21)。
fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// 内訳の色(色があるとき。chart.series-contrast): 暗い地には明るい組、明るい地には濃い組。どれも、どのテーマの地とも
/// 対比 3 以上(試験が全部のテーマで確かめる)。最後は「ほか」の灰。
fn series_color(app: &App, s: usize) -> Option<Color> {
    const DARK: [[u8; 3]; 6] = [
        [86, 182, 194],
        [198, 120, 221],
        [229, 192, 123],
        [152, 195, 121],
        [97, 175, 239],
        [150, 150, 150],
    ];
    const DARK_IDX: [u8; 6] = [73, 176, 180, 107, 75, 246];
    const LIGHT: [[u8; 3]; 6] = [
        [0, 122, 135],
        [142, 68, 173],
        [166, 100, 0],
        [52, 128, 40],
        [36, 98, 196],
        [112, 112, 112],
    ];
    let i = s.min(5);
    let light = is_light(ground(app));
    match app.color {
        ColorMode::None => None,
        ColorMode::Indexed if light => Some(Color::Indexed(to_indexed(LIGHT[i]))),
        ColorMode::Indexed => Some(Color::Indexed(DARK_IDX[i])),
        ColorMode::Rgb => Some(term_color(if light { LIGHT[i] } else { DARK[i] }, false)),
    }
}

/// 積み上げ棒の内訳の塗りの見た目: 字の色は内訳の色。地を埋めない塗りの字(`▓` `▒` `░` `▞` `:` と ASCII の字)は、
/// 地を内訳の色へ少し寄せて、点の粗い字でも棒の形が画面の地から見分けられるようにする(chart.series-contrast)。
fn fill_style(app: &App, s: usize, fill: char) -> Style {
    let Some(col) = series_color(app, s) else {
        return Style::default();
    };
    let st = Style::default().fg(col);
    let indexed = match (app.color, col) {
        (_, _) if fill == '█' => return st,
        (ColorMode::Indexed, Color::Indexed(_)) => true,
        (ColorMode::Rgb, Color::Rgb(..)) => false,
        _ => return st,
    };
    let rgb = match col {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Indexed(n) => match indexed_rgb(n) {
            Some(c) => c,
            None => return st,
        },
        _ => return st,
    };
    let base = ground(app);
    let tint = super::look::mix(base, rgb, 0.22);
    if shown(tint, indexed) == shown(base, indexed) {
        return st;
    }
    st.bg(term_color(tint, indexed))
}

/// 選んだものの見た目(色なしでは反転。SR-15)。
fn sel_style(app: &App) -> Style {
    match super::look::look(app) {
        Some(l) => l.selected(),
        None => Style::default().add_modifier(Modifier::REVERSED),
    }
}

/// 体の始まりの画面の行(ヘッダーの上の余白・ヘッダー・帯の上の余白の下)。
fn body_y(app: &App) -> usize {
    super::bands::top_rows(app) + 1 + super::bands::bar_padding(app)
}

/// 窓の中の幅と、グラフに使える行の数(題の下の説明の行を除く)を、今の画面の大きさから。
fn inner_now(app: &App) -> (usize, usize) {
    let (w, h) = super::popup::screen(app);
    let h = h + 2;
    let bh = h.saturating_sub(body_y(app) + 2);
    (w.saturating_sub(2), bh.saturating_sub(2))
}

/// 体の上の説明の行の数(1行目の数・2行目の選び・凡例)。
fn head_rows(c: &Chart) -> usize {
    if c.data.why.is_some() {
        return 1 + usize::from(c.data.rows + c.data.skipped > 0);
    }
    let legend = matches!(
        c.cur.kind,
        ChartKind::Stacked | ChartKind::Line | ChartKind::Scatter
    ) && !c.data.series.is_empty();
    2 + usize::from(legend)
}

impl App {
    /// グラフの推定に使う、表の列の様子(ノートのキーの列)。
    fn chart_cols(&self) -> Vec<ColInfo> {
        self.src
            .columns()
            .into_iter()
            .map(|c| {
                let has_values = self
                    .prof
                    .resolved
                    .rules
                    .get(&c)
                    .and_then(|r| r.values.as_ref())
                    .is_some_and(|v| !v.is_empty());
                ColInfo {
                    kind: self.col_type(&c),
                    name: c,
                    has_values,
                }
            })
            .collect()
    }

    /// 列が表にあるか(ノートのキーの列か、計算の列・ファイルの列)。
    fn chart_col_ok(&self, col: &str) -> bool {
        col.starts_with("formula.") || col.starts_with("file.") || self.has_column(col)
    }

    /// CH-1: 使える印のグラフ(列が出そろうまでは全部、出そろったら列が表にあるものだけ)。
    pub(crate) fn chart_saved(&self) -> Vec<ChartSpec> {
        let settled = self.columns_settled();
        self.chart_specs
            .iter()
            .filter(|(s, _)| !settled || s.columns().iter().all(|(_, c)| self.chart_col_ok(c)))
            .map(|(s, _)| s.clone())
            .collect()
    }

    /// CH-2: この表でグラフを出せるか。グラフにする列があれば出す。列が無くても、列が出そろう前(読み込みの途中・
    /// 行の無い表)は使える印のグラフがあれば出す(GA-1 と同じく、行ができてから決める)。行があって列が1つも無い
    /// 表(前付けの無いノートだけ)では出さない。
    pub(crate) fn chart_ready(&self) -> bool {
        if !self.src.columns().is_empty() {
            return true;
        }
        !self.columns_settled() && !self.chart_saved().is_empty()
    }

    /// CH-1: 列が出そろったときに、印のグラフの列が表に無ければ CLI-3 の形で1回ずつ警告する(そのグラフは出さない)。
    pub(crate) fn check_chart_columns(&mut self) {
        let mut warns = Vec::new();
        for (spec, at) in &self.chart_specs {
            for (k, c) in spec.columns() {
                if !self.chart_col_ok(c) {
                    warns.push(Msg::SettingWarn.fill(&[
                        &format!(
                            "{}/{}",
                            mdgrid::workspace::MARKER_DIR,
                            mdgrid::workspace::MARKER_FILE
                        ),
                        &format!("{at}.{k}"),
                        &Msg::RsnChartNoColumn.fill(&[&c]),
                    ]));
                }
            }
        }
        for w in warns {
            self.message = Some(match self.message.take() {
                Some(m) => format!("{m} / {w}"),
                None => w,
            });
        }
    }

    /// CH-2・CH-8: 印のグラフ(か何も書いていないグラフ)の省いた鍵を推定で埋める。種類に合わない列は、合う最初の
    /// 列に替える。替えられなければ、その種類のまま描けない理由を返す。
    fn chart_resolve(&self, spec: &ChartSpec) -> (Resolved, bool, Option<String>) {
        let cols = self.chart_cols();
        let kind = spec.kind.unwrap_or(ChartKind::Bar);
        let mut inferred = false;
        let x = if let Some(x) = &spec.x {
            x.clone()
        } else {
            inferred = true;
            let axis = matches!(kind, ChartKind::Line | ChartKind::Scatter);
            // 書いた y(散布図の縦)と by(内訳)の列は、分ける列に選ばない(同じ列になって描けない・1色に
            // ならないように)。
            let taken: Vec<&str> = [
                (kind == ChartKind::Scatter)
                    .then_some(spec.y.as_deref())
                    .flatten(),
                kind.takes_by().then_some(spec.by.as_deref()).flatten(),
            ]
            .into_iter()
            .flatten()
            .collect();
            let rest: Vec<ColInfo> = cols
                .iter()
                .filter(|c| !taken.contains(&c.name.as_str()))
                .cloned()
                .collect();
            let pick = if axis { first_axis(&rest, None) } else { None };
            // 折れ線・散布図に合う列が無ければ推定の列にし、描けない理由は chart_unfit が出す。
            pick.or_else(|| chart::infer_x(&rest))
                .or_else(|| chart::infer_x(&cols))
                .unwrap_or_default()
        };
        let agg = match (&spec.y, spec.agg) {
            (None, _) | (_, Some(Agg::Count)) => Agg::Count,
            (Some(_), a) => a.unwrap_or(Agg::Sum),
        };
        // 散布図は集め方を使わないので、書いた y は agg によらず使う(CH-1)。
        let mut y = if agg == Agg::Count && kind != ChartKind::Scatter {
            None
        } else {
            spec.y.clone()
        };
        let mut agg = agg;
        if kind == ChartKind::Scatter {
            if y.is_none() {
                y = scatter_y(&cols, &x);
                inferred |= y.is_some();
            }
            agg = Agg::Sum;
        }
        let mut by = if kind.takes_by() {
            spec.by.clone()
        } else {
            None
        };
        if kind.needs_by() && by.is_none() {
            by = first_by(&cols, &x, None);
            inferred |= by.is_some();
        }
        let r = Resolved {
            name: spec.name.clone(),
            kind,
            x,
            y,
            agg,
            by,
            unit: spec.unit,
            cumulative: spec.cumulative,
        };
        let why = self.chart_unfit(&r);
        (r, inferred, why)
    }

    /// 種類に合わない列なら理由(CH-8)。
    fn chart_unfit(&self, r: &Resolved) -> Option<String> {
        let kind_of = |c: &str| self.col_type(c);
        let skip = |why: Msg| Some(Msg::ChartKindSkipped.fill(&[&kind_text(r.kind), &why.text()]));
        if r.x.is_empty() {
            return Some(Msg::ChartNoColumns.text().to_string());
        }
        match r.kind {
            ChartKind::Line if !chart::axis_ok(kind_of(&r.x)) => skip(Msg::ChartNeedAxis),
            // CH-6: 横も縦も数か日付の列。
            ChartKind::Scatter
                if !chart::axis_ok(kind_of(&r.x))
                    || r.y
                        .as_deref()
                        .is_none_or(|y| y == r.x || !chart::axis_ok(kind_of(y))) =>
            {
                skip(Msg::ChartNeedScatter)
            }
            // 値は数の列(合計・平均・最小・最大)。
            ChartKind::Bar | ChartKind::Stacked | ChartKind::Line | ChartKind::Heatmap
                if r.agg != Agg::Count
                    && r.y.as_deref().is_some_and(|y| kind_of(y) != Kind::Number) =>
            {
                skip(Msg::ChartNoNumber)
            }
            // 内訳は分ける列と別の、文字・チェックボックス・リストの列(同じ列では1色になる)。
            ChartKind::Stacked | ChartKind::Heatmap
                if r.by
                    .as_deref()
                    .is_none_or(|b| b == r.x || !chart::by_ok(kind_of(b))) =>
            {
                skip(Msg::ChartNeedBy)
            }
            _ => None,
        }
    }

    /// 種類 `kind` に移すときのグラフ(今の列を保ち、合わない列は合う最初の列に替える)と、列を推し量って替えたか。
    /// 移せなければ理由。
    fn chart_with_kind(&self, cur: &Resolved, kind: ChartKind) -> Result<(Resolved, bool), String> {
        let cols = self.chart_cols();
        let mut r = cur.clone();
        let mut guessed = false;
        r.kind = kind;
        let skip = |why: Msg| Msg::ChartKindSkipped.fill(&[&kind_text(kind), &why.text()]);
        let axis = |c: &str| chart::axis_ok(self.col_type(c));
        if kind == ChartKind::Scatter {
            // CH-6: 数か日付の列を2つ。今の横と縦が使えればそのまま、横だけ使えれば縦を探し、横が使えなければ
            // 横と縦の両方を探す。
            let y_ok = |x: &str, y: Option<&str>| y.is_some_and(|y| y != x && axis(y));
            if !axis(&r.x) || !y_ok(&r.x, r.y.as_deref()) {
                let pair = std::iter::once(r.x.clone())
                    .filter(|x| axis(x))
                    .chain(
                        cols.iter()
                            .filter(|c| chart::axis_ok(c.kind))
                            .map(|c| c.name.clone()),
                    )
                    .find_map(|x| scatter_y(&cols, &x).map(|y| (x, y)));
                let (x, y) = pair.ok_or_else(|| skip(Msg::ChartNeedScatter))?;
                guessed = true;
                r.x = x;
                r.y = Some(y);
            }
            if r.agg == Agg::Count {
                r.agg = Agg::Sum;
            }
        } else {
            if kind == ChartKind::Line && !axis(&r.x) {
                r.x = first_axis(&cols, None).ok_or_else(|| skip(Msg::ChartNeedAxis))?;
                guessed = true;
            }
            // 散布図の縦にした日付の列は、合計にできないので件数に戻す。
            if r.y
                .as_deref()
                .is_some_and(|y| self.col_type(y) != Kind::Number)
            {
                r.y = None;
                r.agg = Agg::Count;
            }
        }
        if kind.needs_by() {
            let ok =
                r.by.as_deref()
                    .is_some_and(|b| b != r.x && chart::by_ok(self.col_type(b)));
            if !ok {
                r.by = Some(first_by(&cols, &r.x, None).ok_or_else(|| skip(Msg::ChartNeedBy))?);
                guessed = true;
            }
        }
        if !kind.takes_by() || r.by.as_deref() == Some(r.x.as_str()) {
            r.by = None;
        }
        Ok((r, guessed))
    }

    /// 区切りの軸の長さ(棒・積み上げ棒・ヒートマップは縦の行の数、折れ線は横の字の数)。
    fn chart_axis_len(&self, kind: ChartKind) -> usize {
        let (iw, vis) = inner_now(self);
        match kind {
            ChartKind::Line => iw.saturating_sub(12).max(1),
            _ => vis.saturating_sub(3).max(1),
        }
    }

    /// 行の列の値を日付として読む(ためた値を重ねた文字から。日時は日の部分。ガントと同じ)。
    fn chart_day(&self, row: &RowId, col: &str) -> Option<i64> {
        let s = self.plain(row, col);
        let s = s.trim();
        parse_date(s).or_else(|| s.get(..10).and_then(parse_date))
    }

    /// 行の列の値を数として読む。
    fn chart_num(&self, row: &RowId, col: &str) -> Option<f64> {
        self.plain(row, col)
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
    }

    /// CH-3: 今のビューの行を数える。
    fn chart_data(&self, r: &Resolved, why: Option<String>) -> Data {
        let mut data = Data {
            items: Vec::new(),
            series: Vec::new(),
            dropped: 0,
            points: Vec::new(),
            rows: 0,
            skipped: 0,
            axis: Axis::Values,
            x_date: false,
            y_date: false,
            why,
            fmt: None,
        };
        if data.why.is_some() {
            return data;
        }
        data.fmt =
            r.y.as_ref()
                .filter(|_| r.agg != Agg::Count && r.kind != ChartKind::Scatter)
                .and_then(|y| self.prof.resolved.format.get(y))
                .cloned();
        let xk = self.col_type(&r.x);
        let x_date = matches!(xk, Kind::Date | Kind::DateTime);
        let x_num = xk == Kind::Number;
        data.x_date = x_date;
        // 散布図の縦は数か日付(日付は日の数)、ほかの値は数の列。
        let y_date = r.kind == ChartKind::Scatter
            && r.y
                .as_deref()
                .is_some_and(|y| matches!(self.col_type(y), Kind::Date | Kind::DateTime));
        data.y_date = y_date;
        let yv = |row: &RowId| {
            r.y.as_deref().and_then(|y| {
                if y_date {
                    self.chart_day(row, y).map(|d| d as f64)
                } else {
                    self.chart_num(row, y)
                }
            })
        };
        // 行ごとの (分ける値, 内訳の鍵, 値)。読めない行は外す。
        let mut rows: Vec<RowVals<'_>> = Vec::new();
        for row in &self.rows {
            let xv = if x_date {
                self.chart_day(row, &r.x).map(|d| XV::Num(d as f64))
            } else if x_num {
                self.chart_num(row, &r.x).map(XV::Num)
            } else {
                Some(XV::Keys(value_keys(self.setting_value(row, &r.x).as_ref())))
            };
            let y = yv(row);
            let y_bad = r.agg != Agg::Count && y.is_none();
            let Some(xv) = xv else {
                data.skipped += 1;
                continue;
            };
            if y_bad {
                data.skipped += 1;
            } else {
                data.rows += 1;
            }
            let by = match &r.by {
                Some(b) if r.kind.takes_by() => value_keys(self.setting_value(row, b).as_ref()),
                _ => Vec::new(),
            };
            rows.push((row, xv, by, y));
        }
        // 内訳: 全体で多い順に上限まで(同じなら名前の順)。残りは「ほか」(折れ線は線にしない)。
        let has_by = r.kind.takes_by() && r.by.is_some();
        let mut sidx: HashMap<Option<String>, usize> = HashMap::new();
        if has_by {
            let mut tot: HashMap<Option<String>, Acc> = HashMap::new();
            for (_, _, by, y) in &rows {
                let mut keys = by.clone();
                keys.sort();
                keys.dedup();
                for k in keys {
                    tot.entry(k).or_default().add(*y);
                }
            }
            let rules = r.by.as_deref().and_then(|b| self.rules_values(b));
            let cap = if r.kind == ChartKind::Heatmap {
                MAX_HEAT_COLS
            } else {
                MAX_SERIES
            };
            // CH-4・chart.bar-stacked-top5: 上限までの内訳は、値が数の列(`y`)なら全体の `y` の合計の大きい順、件数なら
            // 行の多い順(集め方の平均・最小・最大と列の決まりによらない。同じなら名前の順で、値の無い行も多ければ
            // 入る)で選ぶ。選んだ内訳は、列の決まりの順 → 同じ大きさの順 → 値の無い行の順に並べる。
            // 散布図は値を集めない(縦の列)ので、件数で選ぶ。
            let by_sum = r.agg != Agg::Count && r.y.is_some() && r.kind != ChartKind::Scatter;
            let size = |a: &Acc| -> f64 {
                if by_sum {
                    a.value(Agg::Sum).unwrap_or(f64::NEG_INFINITY)
                } else {
                    a.rows() as f64
                }
            };
            let mut all: Vec<(Option<String>, Acc)> = tot.into_iter().collect();
            all.sort_by(|(ka, a), (kb, b)| {
                size(b)
                    .total_cmp(&size(a))
                    .then(ka.is_none().cmp(&kb.is_none()))
                    .then_with(|| ka.cmp(kb))
            });
            let rest: Vec<(Option<String>, Acc)> = all.split_off(all.len().min(cap));
            let mut order = all;
            sort_keys(
                &mut order,
                rules.as_deref(),
                if by_sum { Agg::Sum } else { Agg::Count },
            );
            let col = r.by.clone().unwrap_or_default();
            let keyed: Vec<(Option<String>, usize)> =
                order.iter().map(|(k, a)| (k.clone(), a.rows())).collect();
            let labels = value_labels(self, &col, &keyed);
            for (i, ((k, _), label)) in order.iter().zip(labels).enumerate() {
                sidx.insert(k.clone(), i);
                data.series.push(Series {
                    label: if k.is_none() {
                        Msg::ChartNoValue.text().to_string()
                    } else {
                        label
                    },
                    key: Some(k.clone().unwrap_or_default()),
                });
            }
            if r.kind == ChartKind::Line {
                data.dropped = rest.len();
            } else if !rest.is_empty() {
                for (k, _) in &rest {
                    sidx.insert(k.clone(), cap);
                }
                data.series.push(Series {
                    label: Msg::ChartOther.text().to_string(),
                    key: None,
                });
            }
        }
        let ns = data.series.len();
        // 散布図: 1行1点。
        if r.kind == ChartKind::Scatter {
            for (row, xv, by, y) in &rows {
                let (XV::Num(x), Some(y)) = (xv, y) else {
                    continue;
                };
                let s = by.first().and_then(|k| sidx.get(k).copied()).unwrap_or(0);
                let name = super::view::note_name(self, row);
                let name = if name.trim().is_empty() {
                    sanitize(&self.src.label(row))
                } else {
                    name
                };
                data.points.push(Point {
                    row: (*row).clone(),
                    name,
                    x: *x,
                    y: *y,
                    s,
                });
            }
            data.points
                .sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
            data.axis = if x_date {
                Axis::Dates {
                    origin: 0,
                    unit: Unit::Day,
                }
            } else {
                Axis::Bins
            };
            return data;
        }
        let add = |item: &mut Item, by: &[Option<String>], y: Option<f64>| {
            item.total.add(y);
            let mut keys: Vec<&Option<String>> = by.iter().collect();
            keys.sort();
            keys.dedup();
            let mut seen = Vec::new();
            for k in keys {
                if let Some(&s) = sidx.get(k) {
                    if !seen.contains(&s) {
                        seen.push(s);
                        item.parts[s].add(y);
                    }
                }
            }
        };
        let new_item = |label: String, key: Option<String>| Item {
            label,
            key,
            total: Acc::default(),
            parts: vec![Acc::default(); ns],
        };
        if x_date || x_num {
            let nums: Vec<f64> = rows
                .iter()
                .filter_map(|(_, xv, _, _)| match xv {
                    XV::Num(v) => Some(*v),
                    XV::Keys(_) => None,
                })
                .collect();
            let (Some(min), Some(max)) = (
                nums.iter().copied().reduce(f64::min),
                nums.iter().copied().reduce(f64::max),
            ) else {
                return data;
            };
            if x_date {
                let (lo, hi) = (min as i64, max as i64);
                let len = self.chart_axis_len(r.kind);
                let ws = self.week_start;
                let mut unit = r.unit.unwrap_or_else(|| fit_unit(lo, hi, len, ws));
                // 折れ線は横の幅に収まる単位まで粗くする(区切りが字より多いと点が重なる)。
                if r.kind == ChartKind::Line {
                    while unit != Unit::Month && buckets(lo, hi, unit, ws) > len {
                        unit = unit.coarser();
                    }
                }
                while unit != Unit::Month && buckets(lo, hi, unit, ws) > MAX_BUCKETS {
                    unit = unit.coarser();
                }
                // 月でも多すぎる範囲(極端な日付の行)はグラフにせず、範囲を理由に出す。
                if buckets(lo, hi, unit, ws) > MAX_BUCKETS {
                    data.why = Some(Msg::ChartTooWide.fill(&[&format_date(lo), &format_date(hi)]));
                    // 日付の列で分けていることは変わらない(`+` `-` が単位を知らせるように)。
                    data.axis = Axis::Dates {
                        origin: chart::floor_bucket(lo, unit, ws),
                        unit,
                    };
                    return data;
                }
                let origin = chart::floor_bucket(lo, unit, ws);
                let n = buckets(lo, hi, unit, ws);
                data.items = (0..n)
                    .map(|k| {
                        let s = gantt::cell_start(k as i64, origin, unit);
                        new_item(chart::bucket_label(s, unit), None)
                    })
                    .collect();
                for (_, xv, by, y) in &rows {
                    if let XV::Num(d) = xv {
                        let k = gantt::cell_of(*d as i64, origin, unit);
                        if let Some(item) =
                            usize::try_from(k).ok().and_then(|k| data.items.get_mut(k))
                        {
                            add(item, by, *y);
                        }
                    }
                }
                data.axis = Axis::Dates { origin, unit };
            } else {
                let all_int = nums.iter().all(|v| v.fract() == 0.0);
                let bins = chart::bins(min, max, all_int);
                let unit_int = all_int && bins.iter().all(|(lo, hi)| hi - lo == 1.0);
                data.items = bins
                    .iter()
                    .map(|b| new_item(chart::bin_label(*b, unit_int), None))
                    .collect();
                for (_, xv, by, y) in &rows {
                    if let XV::Num(v) = xv {
                        if let Some(i) = chart::bin_of(&bins, *v) {
                            add(&mut data.items[i], by, *y);
                        }
                    }
                }
                data.axis = Axis::Bins;
            }
            return data;
        }
        // 値ごと: 決まりの順(行の無い値も)→ ほかの値を値の大きい順 → 値の無い行。
        let mut acc: HashMap<Option<String>, Item> = HashMap::new();
        let mut first_seen: Vec<Option<String>> = Vec::new();
        for (_, xv, by, y) in &rows {
            if let XV::Keys(keys) = xv {
                let mut keys = keys.clone();
                keys.sort();
                keys.dedup();
                for k in keys {
                    let item = acc.entry(k.clone()).or_insert_with(|| {
                        first_seen.push(k.clone());
                        new_item(String::new(), Some(k.clone().unwrap_or_default()))
                    });
                    add(item, by, *y);
                }
            }
        }
        let rules = self.rules_values(&r.x);
        for v in rules.iter().flatten() {
            let k = Some(v.trim().to_string()).filter(|s| !s.is_empty());
            if k.is_some() && !acc.contains_key(&k) {
                first_seen.push(k.clone());
                acc.insert(k.clone(), new_item(String::new(), k.clone()));
            }
        }
        let mut order: Vec<(Option<String>, Acc)> = first_seen
            .iter()
            .map(|k| (k.clone(), acc[k].total))
            .collect();
        sort_keys(&mut order, rules.as_deref(), r.agg);
        let keyed: Vec<(Option<String>, usize)> =
            order.iter().map(|(k, a)| (k.clone(), a.rows())).collect();
        let labels = value_labels(self, &r.x, &keyed);
        for ((k, _), label) in order.into_iter().zip(labels) {
            if let Some(mut item) = acc.remove(&k) {
                item.label = if k.is_none() {
                    Msg::ChartNoValue.text().to_string()
                } else {
                    label
                };
                data.items.push(item);
            }
        }
        data
    }

    /// 列の決まり(CE-36)の値。
    fn rules_values(&self, col: &str) -> Option<Vec<String>> {
        self.prof
            .resolved
            .rules
            .get(col)
            .and_then(|r| r.values.clone())
            .filter(|v| !v.is_empty())
    }

    /// CH-2: グラフの画面を開く。列が無い・行が無いときは理由。印のグラフがあれば最初のグラフで、無ければ推定で。
    pub(crate) fn open_chart(&mut self) {
        if !self.chart_ready() {
            self.message = Some(Msg::ChartNoColumns.text().into());
            return;
        }
        if self.rows.is_empty() {
            self.message = Some(Msg::ChartNoRows.text().into());
            return;
        }
        let spec = self.chart_saved().into_iter().next().unwrap_or_default();
        let (cur, inferred, why) = self.chart_resolve(&spec);
        let data = self.chart_data(&cur, why);
        self.leave_screens();
        self.chart = Some(Chart {
            cur,
            idx: 0,
            inferred,
            data,
            sel: 0,
            sel2: 0,
            pct: false,
            top: 0,
            left: 0,
        });
        self.set_mode(Mode::Chart);
    }

    /// 表の画面に戻る。
    pub(crate) fn close_chart(&mut self) {
        self.chart = None;
        self.set_mode(Mode::Table);
    }

    /// 今のグラフ `cur` で数え直す(選びの番号は数の範囲に収める)。
    fn chart_recount(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let cur = c.cur.clone();
        let why = self.chart_unfit(&cur);
        let data = self.chart_data(&cur, why);
        if let Some(c) = self.chart.as_mut() {
            c.data = data;
            let n = c
                .data
                .items
                .len()
                .max(c.data.points.len())
                .saturating_sub(1);
            c.sel = c.sel.min(n);
            c.sel2 = c.sel2.min(c.data.series.len().saturating_sub(1));
        }
        self.chart_follow();
    }

    /// CH-8: 外で変わったときに、今のビューを通った行で数え直す。列が出そろって使える列が無くなったら閉じる。
    /// 今のグラフの列が表から消えたら、印のグラフ(か推定)から決め直す。
    pub(crate) fn rebuild_chart(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        if !self.chart_ready() && self.columns_settled() && self.mode == Mode::Chart {
            self.close_chart();
            self.message = Some(Msg::ChartNoColumns.text().into());
            return;
        }
        let cur = c.cur.clone();
        let gone = self.columns_settled()
            && [Some(&cur.x), cur.y.as_ref(), cur.by.as_ref()]
                .into_iter()
                .flatten()
                .any(|col| !self.chart_col_ok(col));
        if gone {
            let saved = self.chart_saved();
            let idx = c.idx.min(saved.len().saturating_sub(1));
            let spec = saved.into_iter().nth(idx).unwrap_or_default();
            let (cur, inferred, _) = self.chart_resolve(&spec);
            if let Some(c) = self.chart.as_mut() {
                c.cur = cur;
                c.inferred = inferred;
                c.idx = idx;
            }
        }
        self.chart_recount();
    }

    /// CH-8: 種類を次・前へ。合う列の無い種類は理由を出して飛ばす。
    fn chart_kind_step(&mut self, forward: bool) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let all = ChartKind::ALL;
        let at = all.iter().position(|k| *k == c.cur.kind).unwrap_or(0);
        let mut why = None;
        for step in 1..all.len() {
            let i = if forward {
                (at + step) % all.len()
            } else {
                (at + all.len() - step) % all.len()
            };
            match self.chart_with_kind(&c.cur, all[i]) {
                Ok((r, guessed)) => {
                    if let Some(c) = self.chart.as_mut() {
                        c.cur = r;
                        // CH-2: 種類に合わせて列を推し量ったら、題に推定と出す。
                        c.inferred |= guessed;
                        c.sel = 0;
                        c.sel2 = 0;
                        c.pct = false;
                    }
                    self.chart_recount();
                    self.message = why;
                    return;
                }
                Err(w) => why = Some(w),
            }
        }
        self.message = why;
    }

    /// CH-8: 分ける列を次へ(種類に合う列の中で)。
    fn chart_next_x(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let cur = c.cur.clone();
        let axis = matches!(cur.kind, ChartKind::Line | ChartKind::Scatter);
        let cands: Vec<String> = self
            .chart_cols()
            .into_iter()
            .filter(|ci| !axis || chart::axis_ok(ci.kind))
            .filter(|ci| Some(&ci.name) != cur.by.as_ref())
            .filter(|ci| cur.kind != ChartKind::Scatter || Some(&ci.name) != cur.y.as_ref())
            .map(|ci| ci.name)
            .collect();
        let Some(next) = next_of(&cands, Some(&cur.x)) else {
            self.message = Some(Msg::ChartNoOtherX.text().into());
            return;
        };
        if let Some(c) = self.chart.as_mut() {
            c.cur.x = next;
            c.cur.unit = None;
            c.sel = 0;
            c.sel2 = 0;
        }
        self.chart_recount();
    }

    /// CH-8: 値を次へ(件数 → 数の列。散布図は横と別の数か日付の列)。
    fn chart_next_y(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let cur = c.cur.clone();
        let scatter = cur.kind == ChartKind::Scatter;
        let nums: Vec<String> = self
            .chart_cols()
            .into_iter()
            .filter(|ci| {
                if scatter {
                    chart::axis_ok(ci.kind) && ci.name != cur.x
                } else {
                    ci.kind == Kind::Number
                }
            })
            .map(|ci| ci.name)
            .collect();
        let next = if scatter {
            next_of(&nums, cur.y.as_ref()).map(Some)
        } else {
            // 件数(None)→ 数の列 → 件数。
            let mut cands: Vec<Option<String>> = vec![None];
            cands.extend(nums.into_iter().map(Some));
            let now = if cur.agg == Agg::Count {
                None
            } else {
                cur.y.clone()
            };
            if cands.len() < 2 {
                None
            } else {
                let at = cands.iter().position(|c| *c == now).unwrap_or(0);
                Some(cands[(at + 1) % cands.len()].clone())
            }
        };
        let Some(next) = next else {
            // 散布図の縦は数か日付の列なので、その言い方で知らせる。
            let why = if scatter {
                Msg::ChartNoOtherY
            } else {
                Msg::ChartNoNumber
            };
            self.message = Some(why.text().into());
            return;
        };
        if let Some(c) = self.chart.as_mut() {
            if let Some(y) = next {
                c.cur.y = Some(y);
                if c.cur.agg == Agg::Count {
                    c.cur.agg = Agg::Sum;
                }
            } else {
                c.cur.y = None;
                c.cur.agg = Agg::Count;
            }
        }
        self.chart_recount();
    }

    /// CH-8: 集め方を次へ(合計 → 平均 → 最小 → 最大)。件数のときは理由。
    fn chart_next_agg(&mut self) {
        let Some(c) = self.chart.as_mut() else {
            return;
        };
        if c.cur.agg == Agg::Count || c.cur.y.is_none() || c.cur.kind == ChartKind::Scatter {
            self.message = Some(Msg::ChartAggForCount.text().into());
            return;
        }
        c.cur.agg = match c.cur.agg {
            Agg::Sum => Agg::Avg,
            Agg::Avg => Agg::Min,
            Agg::Min => Agg::Max,
            Agg::Max | Agg::Count => Agg::Sum,
        };
        self.chart_recount();
    }

    /// CH-8: 内訳の列を次へ(折れ線と散布図は「なし」も)。棒では理由。
    fn chart_next_by(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let cur = c.cur.clone();
        if !cur.kind.takes_by() {
            self.message = Some(Msg::ChartByNotBar.text().into());
            return;
        }
        let mut cands: Vec<Option<String>> = Vec::new();
        if !cur.kind.needs_by() {
            cands.push(None);
        }
        cands.extend(
            self.chart_cols()
                .into_iter()
                .filter(|ci| chart::by_ok(ci.kind) && ci.name != cur.x)
                .map(|ci| Some(ci.name)),
        );
        if cands.len() < 2 {
            self.message = Some(Msg::ChartNoOtherBy.text().into());
            return;
        }
        let at = cands.iter().position(|b| *b == cur.by).unwrap_or(0);
        let next = cands[(at + 1) % cands.len()].clone();
        if let Some(c) = self.chart.as_mut() {
            c.cur.by = next;
            c.sel2 = 0;
        }
        self.chart_recount();
    }

    /// CH-8: 日付の単位を細かく・粗く。日付の列で分けていなければ理由。細かくして区切りが収まらなければ理由。
    fn chart_zoom(&mut self, finer: bool) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let (Axis::Dates { unit, .. }, false) = (&c.data.axis, c.cur.kind == ChartKind::Scatter)
        else {
            self.message = Some(Msg::ChartNoUnit.text().into());
            return;
        };
        let unit = *unit;
        let next = if finer { unit.finer() } else { unit.coarser() };
        if finer && next != unit && c.cur.kind == ChartKind::Line {
            let days: Vec<i64> = self
                .rows
                .iter()
                .filter_map(|r| self.chart_day(r, &c.cur.x))
                .collect();
            let (lo, hi) = (
                days.iter().copied().min().unwrap_or(0),
                days.iter().copied().max().unwrap_or(0),
            );
            if buckets(lo, hi, next, self.week_start) > self.chart_axis_len(ChartKind::Line) {
                self.message = Some(Msg::ChartUnitNoRoom.text().into());
                return;
            }
        }
        if let Some(c) = self.chart.as_mut() {
            c.cur.unit = Some(next);
            c.sel = 0;
        }
        self.chart_recount();
        // 区切りが多すぎて数える側で粗くしたら、実際の単位を覚えて、細かくできなかったと知らせる(次の `+` も
        // 実際の単位から数える)。
        let got = match self.chart.as_ref().map(|c| &c.data.axis) {
            Some(Axis::Dates { unit, .. }) => Some(*unit),
            _ => None,
        };
        match got {
            Some(u) if u != next => {
                if let Some(c) = self.chart.as_mut() {
                    c.cur.unit = Some(u);
                }
                self.message = Some(Msg::ChartUnitNoRoom.text().into());
            }
            Some(u) => self.message = Some(Msg::ChartUnit.fill(&[&unit_text(u)])),
            None => {}
        }
    }

    /// CH-8: 印に書いた前・次のグラフ。
    fn chart_saved_step(&mut self, forward: bool) {
        let saved = self.chart_saved();
        if saved.len() < 2 {
            self.message = Some(Msg::ChartOneSaved.text().into());
            return;
        }
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let n = saved.len();
        let idx = if forward {
            (c.idx + 1) % n
        } else {
            (c.idx + n - 1) % n
        };
        let (cur, inferred, why) = self.chart_resolve(&saved[idx]);
        let data = self.chart_data(&cur, why);
        if let Some(c) = self.chart.as_mut() {
            c.cur = cur;
            c.inferred = inferred;
            c.idx = idx;
            c.data = data;
            c.sel = 0;
            c.sel2 = 0;
            c.pct = false;
        }
    }

    /// 選びを動かす(`d` は −1 か +1)。`vertical` は j・k(と縦のホイール)。
    fn chart_move(&mut self, d: isize, vertical: bool) {
        let Some(c) = self.chart.as_mut() else {
            return;
        };
        let step = |v: usize, n: usize| -> usize {
            if n == 0 {
                0
            } else {
                v.saturating_add_signed(d).min(n - 1)
            }
        };
        match c.cur.kind {
            ChartKind::Bar | ChartKind::Stacked => {
                if vertical {
                    c.sel = step(c.sel, c.data.items.len());
                }
            }
            ChartKind::Line => c.sel = step(c.sel, c.data.items.len()),
            ChartKind::Scatter => c.sel = step(c.sel, c.data.points.len()),
            ChartKind::Heatmap => {
                if vertical {
                    c.sel = step(c.sel, c.data.items.len());
                } else {
                    c.sel2 = step(c.sel2, c.data.series.len());
                }
            }
        }
    }

    /// CH-4・CH-6・CH-7・CH-8: `Enter`。棒・積み上げ棒・ヒートマップはその値の行だけに絞って表へ、散布図は表のその行へ。
    fn chart_run(&mut self) {
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        match c.cur.kind {
            ChartKind::Line => {
                self.message = Some(Msg::ChartLineNoFilter.text().into());
            }
            ChartKind::Scatter => {
                let row = c.data.points.get(c.sel).map(|p| p.row.clone());
                if row.is_none() {
                    self.message = Some(Msg::ChartNoPoint.text().into());
                    return;
                }
                self.select_in_table(row);
            }
            ChartKind::Bar | ChartKind::Stacked | ChartKind::Heatmap => {
                let Some(item) = c.data.items.get(c.sel) else {
                    self.message = Some(Msg::ChartNoItem.text().into());
                    return;
                };
                let Some(k) = item.key.clone() else {
                    self.message = Some(Msg::ChartNoFilter.text().into());
                    return;
                };
                let mut conds = vec![(c.cur.x.clone(), k)];
                if c.cur.kind == ChartKind::Heatmap {
                    let s = c.data.series.get(c.sel2).and_then(|s| s.key.clone());
                    let (Some(by), Some(s)) = (c.cur.by.clone(), s) else {
                        // 「ほか」の列は複数の値をまとめたもので、1つの値では絞れない。
                        self.message = Some(Msg::ChartOtherNoFilter.text().into());
                        return;
                    };
                    conds.push((by, s));
                }
                self.close_chart();
                self.push_same(conds);
            }
        }
    }

    /// CH-8: グラフの画面の動作。動いたあと、選びが見える所までだけずらしを直す。
    pub(crate) fn chart_action(&mut self, action: Action) {
        self.chart_act(action);
        self.chart_follow();
    }

    /// 選んでいる項目が見える所までだけ、縦と横のずらしを直して覚える(今の画面の大きさで。描画と当たり判定は
    /// 覚えたずらしから同じ計算をする)。
    fn chart_follow(&mut self) {
        let (iw, vis) = inner_now(self);
        let Some(c) = self.chart.as_ref() else {
            return;
        };
        let rows = vis.saturating_sub(head_rows(c));
        let n = c.data.items.len();
        let (top, left) = match c.cur.kind {
            ChartKind::Bar | ChartKind::Stacked => (scroll(c.top, c.sel, rows, n), c.left),
            ChartKind::Heatmap => (
                scroll(c.top, c.sel, rows.saturating_sub(1), n),
                heat_layout(c, iw).left,
            ),
            ChartKind::Line | ChartKind::Scatter => (c.top, c.left),
        };
        if let Some(c) = self.chart.as_mut() {
            c.top = top;
            c.left = left;
        }
    }

    fn chart_act(&mut self, action: Action) {
        if self.chart.is_none() {
            return self.set_mode(Mode::Table);
        }
        match action {
            Action::Up => self.chart_move(-1, true),
            Action::Down => self.chart_move(1, true),
            Action::Left => self.chart_move(-1, false),
            Action::Right => self.chart_move(1, false),
            Action::ChartKindNext => self.chart_kind_step(true),
            Action::ChartKindPrev => self.chart_kind_step(false),
            Action::ChartX => self.chart_next_x(),
            Action::ChartY => self.chart_next_y(),
            Action::ChartAgg => self.chart_next_agg(),
            Action::ChartBy => self.chart_next_by(),
            Action::ChartFiner => self.chart_zoom(true),
            Action::ChartCoarser => self.chart_zoom(false),
            Action::ChartPrevSaved => self.chart_saved_step(false),
            Action::ChartNextSaved => self.chart_saved_step(true),
            Action::ChartCumulative => {
                let Some(c) = self.chart.as_mut() else {
                    return;
                };
                if c.cur.kind == ChartKind::Line {
                    c.cur.cumulative = !c.cur.cumulative;
                } else {
                    self.message = Some(Msg::ChartNotLine.text().into());
                }
            }
            Action::ChartPercent => {
                let Some(c) = self.chart.as_mut() else {
                    return;
                };
                if c.cur.kind == ChartKind::Stacked {
                    c.pct = !c.pct;
                } else {
                    self.message = Some(Msg::ChartNotStacked.text().into());
                }
            }
            Action::Run => self.chart_run(),
            Action::Close | Action::ChartView => self.close_chart(),
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            // パレットから選んだグラフに無い動作は、表に戻ってする。散布図の点を選んでいればその行を表で選んでから。
            a => {
                let row = self.chart.as_ref().and_then(|c| {
                    (c.cur.kind == ChartKind::Scatter)
                        .then(|| c.data.points.get(c.sel).map(|p| p.row.clone()))
                        .flatten()
                });
                let no_row = match self.chart.as_ref().map(|c| c.cur.kind) {
                    Some(ChartKind::Scatter) => Msg::ChartNoPoint,
                    _ => Msg::ChartNoRow,
                };
                self.act_from_screen(row, a, no_row);
            }
        }
    }

    /// 画面の (x, y) が指す選び(項目の番号と、ヒートマップの横の番号)。
    fn chart_hit(&self, x: u16, y: u16) -> Option<(usize, Option<usize>)> {
        let c = self.chart.as_ref()?;
        let (iw, vis) = inner_now(self);
        let top = body_y(self) + 1 + head_rows(c);
        let y = usize::from(y).checked_sub(top)?;
        let x = usize::from(x).checked_sub(1)?;
        let rows = vis.saturating_sub(head_rows(c));
        if y >= rows || x >= iw {
            return None;
        }
        let g = glyphs(self);
        match c.cur.kind {
            ChartKind::Bar | ChartKind::Stacked => {
                let first = scroll(c.top, c.sel, rows, c.data.items.len());
                let i = first + y;
                (i < c.data.items.len()).then_some((i, None))
            }
            ChartKind::Heatmap => {
                let lay = heat_layout(c, iw);
                let r = y.checked_sub(1)?;
                let first = scroll(c.top, c.sel, rows.saturating_sub(1), c.data.items.len());
                let i = first + r;
                if i >= c.data.items.len() {
                    return None;
                }
                let cx = x.checked_sub(lay.x0)?;
                let j = lay.left + cx / (lay.cw + 1);
                (j < c.data.series.len() && j < lay.left + lay.shown).then_some((i, Some(j)))
            }
            ChartKind::Line => {
                let lay = plot_layout(c, iw, rows, &g);
                let px = x.checked_sub(lay.x0)?;
                if px >= lay.pw {
                    return None;
                }
                let xs = line_xs(c.data.items.len(), lay.pw);
                xs.iter()
                    .enumerate()
                    .min_by_key(|(_, &cx)| cx.abs_diff(px))
                    .map(|(i, _)| (i, None))
            }
            ChartKind::Scatter => {
                let lay = plot_layout(c, iw, rows, &g);
                let px = x.checked_sub(lay.x0)?;
                let cells = scatter_cells(c, lay.pw, lay.ph);
                cells
                    .iter()
                    .position(|&(cx, cy)| cx == px && cy == y)
                    .map(|i| (i, None))
            }
        }
    }

    /// CH-8: グラフの画面のクリック。項目で選び、選んでいる項目をもう一度で `Enter` と同じ。
    pub(crate) fn chart_click(&mut self, x: u16, y: u16) {
        let Some((i, j)) = self.chart_hit(x, y) else {
            return;
        };
        let same = self
            .chart
            .as_ref()
            .is_some_and(|c| c.sel == i && j.is_none_or(|j| c.sel2 == j));
        if same {
            self.chart_run();
            // 絞り込みを当てた表を組み直す(キーの道は `apply` が組み直す)。
            self.refresh_if_needed();
            return;
        }
        if let Some(c) = self.chart.as_mut() {
            c.sel = i;
            if let Some(j) = j {
                c.sel2 = j;
            }
        }
        self.chart_follow();
    }
}

/// `lo..=hi`(日)の区切りの数(週は `week` の曜日で区切る。chart.buckets-week-start)。
fn buckets(lo: i64, hi: i64, unit: Unit, week: WeekStart) -> usize {
    let origin = chart::floor_bucket(lo.min(hi), unit, week);
    usize::try_from(gantt::cell_of(lo.max(hi), origin, unit) + 1).unwrap_or(1)
}

/// 範囲 `lo..=hi`(日)の区切りが `len` 個に収まる、最も細かい単位(日 → 週 → 月。月でも収まらなければ月)。
/// `gantt::fit_unit` と同じ決め方で、週の区切りだけ `week` の曜日にする。
fn fit_unit(lo: i64, hi: i64, len: usize, week: WeekStart) -> Unit {
    [Unit::Day, Unit::Week]
        .into_iter()
        .find(|u| buckets(lo, hi, *u, week) <= len.max(1))
        .unwrap_or(Unit::Month)
}

/// 鍵の並べ方(CH-3): 決まりの値の順 → ほかを値の大きい順(同じなら名前の順)→ 値の無い鍵。
fn sort_keys(order: &mut [(Option<String>, Acc)], rules: Option<&[String]>, agg: Agg) {
    let rank = |k: &Option<String>| -> usize {
        match (k, rules) {
            (Some(v), Some(r)) => r.iter().position(|x| x.trim() == v).unwrap_or(usize::MAX),
            _ => usize::MAX,
        }
    };
    order.sort_by(|(ka, a), (kb, b)| {
        let none = ka.is_none().cmp(&kb.is_none());
        let ra = rank(ka);
        let rb = rank(kb);
        let va = a.value(agg).unwrap_or(f64::NEG_INFINITY);
        let vb = b.value(agg).unwrap_or(f64::NEG_INFINITY);
        none.then(ra.cmp(&rb))
            .then(vb.total_cmp(&va))
            .then_with(|| ka.cmp(kb))
    });
}

/// 推定の順(CH-2)のうち、日付・日時の列、無ければ数の列の最初(`skip` を除く)。
fn first_axis(cols: &[ColInfo], skip: Option<&str>) -> Option<String> {
    let ok = |c: &&ColInfo| Some(c.name.as_str()) != skip;
    cols.iter()
        .filter(ok)
        .find(|c| matches!(c.kind, Kind::Date | Kind::DateTime))
        .or_else(|| cols.iter().filter(ok).find(|c| c.kind == Kind::Number))
        .map(|c| c.name.clone())
}

/// CH-6: 散布図の縦にする列(`x` と別の、数の列を先に、無ければ日付・日時の列)。
fn scatter_y(cols: &[ColInfo], x: &str) -> Option<String> {
    cols.iter()
        .find(|c| c.kind == Kind::Number && c.name != x)
        .or_else(|| {
            cols.iter()
                .find(|c| matches!(c.kind, Kind::Date | Kind::DateTime) && c.name != x)
        })
        .map(|c| c.name.clone())
}

/// 内訳にできる最初の列(推定の順。`x` と `skip` を除く)。
fn first_by(cols: &[ColInfo], x: &str, skip: Option<&str>) -> Option<String> {
    let rest: Vec<ColInfo> = cols
        .iter()
        .filter(|c| c.name != x && Some(c.name.as_str()) != skip && chart::by_ok(c.kind))
        .cloned()
        .collect();
    chart::infer_x(&rest)
}

/// chart.value-labels: 値の鍵の並び `keyed` の見せる名前。列 `col` の決まりに `icons`・`labels` のある値は、まとまりの
/// 見出しと同じ「字 名前」(navigation.group-values-icon。`value_shown`)、ほかは頻度表と同じ名前(リンクは行き先の
/// 名前、値の無い鍵は「(なし)」)。絞り込みの鍵(`Item::key`・`Series::key`)は値のまま。
fn value_labels(app: &App, col: &str, keyed: &[(Option<String>, usize)]) -> Vec<String> {
    let base = super::freq::labels(app, col, keyed);
    keyed
        .iter()
        .zip(base)
        .map(|((k, _), label)| {
            k.as_deref()
                .and_then(|v| app.prof.resolved.value_shown(col, v))
                .map_or(label, |s| sanitize(&s))
        })
        .collect()
}

/// 並び `cands` の中で `now` の次(無ければ最初)。候補が1つで、それが今のものなら None。
fn next_of(cands: &[String], now: Option<&String>) -> Option<String> {
    match now.and_then(|n| cands.iter().position(|c| c == n)) {
        Some(at) if cands.len() > 1 => Some(cands[(at + 1) % cands.len()].clone()),
        Some(_) => None,
        None => cands.first().cloned(),
    }
}

/// 見せる数 `vis` の窓(全部で `n`)で、今のずらし `top` から `pos` 番目が見える所までだけずらす(見えていれば
/// 動かさない。クリックした項目が跳ねないように)。
fn scroll(top: usize, pos: usize, vis: usize, n: usize) -> usize {
    let vis = vis.max(1);
    let mut top = top.min(n.saturating_sub(vis));
    if pos < top {
        top = pos;
    } else if pos >= top + vis {
        top = pos + 1 - vis;
    }
    top
}

/// 項目の値(割合・累積は描くときに)。
fn item_value(item: &Item, agg: Agg) -> Option<f64> {
    item.total.value(agg)
}

/// 棒の字(長さ `len` の割合 `t`。8 分の 1 の字で端を描く。ASCII は字の単位)。
fn bar_text(t: f64, len: usize, ascii: bool) -> String {
    if t.is_nan() || t <= 0.0 || len == 0 {
        return String::new();
    }
    let t = t.min(1.0);
    if ascii {
        let n = ((t * len as f64).round() as usize).max(1);
        return "#".repeat(n);
    }
    let e = ((t * len as f64 * 8.0).round() as usize).max(1);
    let mut s = "█".repeat(e / 8);
    s.push_str(EIGHTHS[e % 8]);
    s
}

/// 選んでいる項目の説明(体の2行目)。
fn detail(c: &Chart) -> String {
    let d = &c.data;
    let agg = c.cur.agg;
    match c.cur.kind {
        ChartKind::Bar => d
            .items
            .get(c.sel)
            .map(|i| format!("{}: {}", i.label, vnum(c, item_value(i, agg))))
            .unwrap_or_default(),
        ChartKind::Stacked => d
            .items
            .get(c.sel)
            .map(|i| {
                let mut s = format!("{}: {}", i.label, vnum(c, item_value(i, agg)));
                let sum: f64 = i.parts.iter().filter_map(|p| p.value(agg)).sum();
                for (p, se) in i.parts.iter().zip(&d.series) {
                    let v = p.value(agg);
                    if c.pct {
                        let pc = v.filter(|_| sum > 0.0).map(|v| v / sum * 100.0);
                        let pc = pc.map_or_else(|| "-".to_string(), |p| format!("{}%", p.round()));
                        s.push_str(&format!(" · {} {pc}", se.label));
                    } else {
                        s.push_str(&format!(" · {} {}", se.label, vnum(c, v)));
                    }
                }
                s
            })
            .unwrap_or_default(),
        ChartKind::Line => {
            let vals = line_values(c);
            d.items
                .get(c.sel)
                .map(|i| {
                    if d.series.is_empty() {
                        format!("{}: {}", i.label, vnum(c, vals[0][c.sel]))
                    } else {
                        let parts: Vec<String> = d
                            .series
                            .iter()
                            .enumerate()
                            .map(|(s, se)| format!("{} {}", se.label, vnum(c, vals[s][c.sel])))
                            .collect();
                        format!("{}: {}", i.label, parts.join(" · "))
                    }
                })
                .unwrap_or_default()
        }
        ChartKind::Scatter => d
            .points
            .get(c.sel)
            .map(|p| {
                format!(
                    "{}: {} × {}",
                    p.name,
                    axis_num(p.x, d.x_date),
                    axis_num(p.y, d.y_date)
                )
            })
            .unwrap_or_default(),
        ChartKind::Heatmap => match (d.items.get(c.sel), d.series.get(c.sel2)) {
            (Some(i), Some(s)) => format!(
                "{} × {}: {}",
                i.label,
                s.label,
                vnum(c, i.parts.get(c.sel2).and_then(|p| p.value(agg)))
            ),
            _ => String::new(),
        },
    }
}

/// 横の軸の値の文字(日付の列なら日付)。
fn axis_num(v: f64, date: bool) -> String {
    if date {
        format_date(v as i64)
    } else {
        chart::fmt_num(v)
    }
}

/// 折れ線の線ごとの値(内訳が無ければ1本)。累積なら区切りまでの合計。
fn line_values(c: &Chart) -> Vec<Vec<Option<f64>>> {
    let d = &c.data;
    let agg = c.cur.agg;
    // 件数と合計は、行の無い区切りを 0 にする(平均・最小・最大は値なしで、線を切る)。
    let zero = |v: Option<f64>| {
        if matches!(agg, Agg::Count | Agg::Sum) {
            v.or(Some(0.0))
        } else {
            v
        }
    };
    let mut out: Vec<Vec<Option<f64>>> = if d.series.is_empty() {
        vec![d.items.iter().map(|i| zero(i.total.value(agg))).collect()]
    } else {
        (0..d.series.len())
            .map(|s| {
                d.items
                    .iter()
                    .map(|i| zero(i.parts[s].value(agg)))
                    .collect()
            })
            .collect()
    };
    if c.cur.cumulative {
        for line in &mut out {
            let mut run = 0.0;
            for v in line.iter_mut() {
                run += v.unwrap_or(0.0);
                *v = Some(run);
            }
        }
    }
    out
}

/// 折れ線と散布図の枠の寸法。
struct PlotLayout {
    /// 目盛りの幅。
    yw: usize,
    /// 描く所の左の字(体の中の)。
    x0: usize,
    pw: usize,
    ph: usize,
    lo: f64,
    hi: f64,
}

/// 値 `v` の、範囲 `lo..=hi` の中の位置(0〜1)。幅が f64 を越える範囲(-1e308 と 1e308)でもあふれないように、
/// 半分にしてから比べる。
fn frac(v: f64, lo: f64, hi: f64) -> f64 {
    let t = (v * 0.5 - lo * 0.5) / (hi * 0.5 - lo * 0.5);
    if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// 値の範囲(最小は 0 以下、同じなら上を 1 広げる)。
fn value_range(vals: impl Iterator<Item = f64>, from_zero: bool) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for v in vals {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if !lo.is_finite() {
        return (0.0, 1.0);
    }
    if from_zero {
        lo = lo.min(0.0);
    }
    if hi <= lo {
        hi = lo + 1.0;
    }
    (lo, hi)
}

/// 縦の目盛りの字(散布図の縦が日付の列なら日付)。
fn y_label(c: &Chart, v: f64) -> String {
    if c.cur.kind == ChartKind::Scatter {
        axis_num(v, c.data.y_date)
    } else {
        vnum(c, Some(v))
    }
}

fn plot_layout(c: &Chart, iw: usize, rows: usize, _g: &Glyphs) -> PlotLayout {
    let (lo, hi) = if c.cur.kind == ChartKind::Scatter {
        value_range(c.data.points.iter().map(|p| p.y), false)
    } else {
        value_range(line_values(c).into_iter().flatten().flatten(), true)
    };
    let yw = width(&y_label(c, hi)).max(width(&y_label(c, lo)));
    let x0 = yw + 1;
    PlotLayout {
        yw,
        x0,
        pw: iw.saturating_sub(x0 + 1).max(1),
        ph: rows.saturating_sub(2).max(1),
        lo,
        hi,
    }
}

/// 折れ線の区切りの横の字の位置。
fn line_xs(n: usize, pw: usize) -> Vec<usize> {
    match n {
        0 => Vec::new(),
        1 => vec![pw / 2],
        _ => (0..n)
            .map(|i| ((i as f64) * (pw - 1) as f64 / (n - 1) as f64).round() as usize)
            .collect(),
    }
}

/// 散布図の点の字の位置(横, 縦)。
fn scatter_cells(c: &Chart, pw: usize, ph: usize) -> Vec<(usize, usize)> {
    let (xl, xh) = value_range(c.data.points.iter().map(|p| p.x), false);
    let (yl, yh) = value_range(c.data.points.iter().map(|p| p.y), false);
    c.data
        .points
        .iter()
        .map(|p| {
            let cx = (frac(p.x, xl, xh) * (pw.saturating_sub(1)) as f64).round() as usize;
            let cy = ((1.0 - frac(p.y, yl, yh)) * (ph.saturating_sub(1)) as f64).round() as usize;
            (cx.min(pw.saturating_sub(1)), cy.min(ph.saturating_sub(1)))
        })
        .collect()
}

/// ヒートマップの寸法。
struct HeatLayout {
    lw: usize,
    cw: usize,
    x0: usize,
    left: usize,
    shown: usize,
}

fn heat_layout(c: &Chart, iw: usize) -> HeatLayout {
    let d = &c.data;
    let lw = d
        .items
        .iter()
        .map(|i| width(&i.label))
        .max()
        .unwrap_or(1)
        .clamp(1, (iw / 3).max(4));
    let vw = d
        .items
        .iter()
        .flat_map(|i| i.parts.iter().map(|p| width(&vnum(c, p.value(c.cur.agg)))))
        .max()
        .unwrap_or(1);
    let sw = d.series.iter().map(|s| width(&s.label)).max().unwrap_or(1);
    // 見出しは 14 字まで(値の名前に幅2の字(Nerd Font のアイコンなど)を付けた「字 名前」の、全角5字の名前が切れない
    // 幅。chart.value-labels)。
    let cw = (vw + 2).max(sw.min(14)).max(4);
    let x0 = 1 + lw + 2;
    let shown = (iw.saturating_sub(x0 + 1) / (cw + 1)).max(1);
    let n = d.series.len();
    let left = scroll(c.left, c.sel2, shown, n);
    HeatLayout {
        lw,
        cw,
        x0,
        left,
        shown: shown.min(n.saturating_sub(left)),
    }
}

/// 体の中身(説明の行とグラフ)。
fn body_lines(app: &App, c: &Chart, iw: usize, vis: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let d = &c.data;
    // 1行目: 数えた行と外した行。
    let mut sum = format!(" {}", Msg::ChartRows.fill(&[&d.rows]));
    if d.skipped > 0 {
        sum.push_str(&Msg::ChartSkipped.fill(&[&d.skipped]));
    }
    if let Some(why) = &d.why {
        // 数えたあとで描けないと分かったとき(広すぎる日付の範囲)は、数えた行も出す。
        if d.rows + d.skipped > 0 {
            lines.push(Line::from(fit(&sum, iw, Align::Left)));
        }
        lines.push(Line::from(fit(&format!(" {why}"), iw, Align::Left)));
        return lines;
    }
    if d.dropped > 0 {
        sum.push_str(&Msg::ChartSeriesDropped.fill(&[&d.dropped]));
    }
    lines.push(Line::from(fit(&sum, iw, Align::Left)));
    // 2行目: 選んでいるもの。
    lines.push(Line::from(Span::styled(
        fit(&format!(" {}", sanitize(&detail(c))), iw, Align::Left),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    let g = glyphs(app);
    // 凡例。
    if head_rows(c) == 3 {
        let marks = matches!(c.cur.kind, ChartKind::Line | ChartKind::Scatter);
        let mut spans = vec![Span::raw(" ")];
        for (s, se) in d.series.iter().enumerate() {
            let ch = if marks {
                MARKS[s.min(5)]
            } else if g.ascii {
                FILLS_ASCII[s.min(5)]
            } else {
                FILLS[s.min(5)]
            };
            let st = if marks {
                series_color(app, s).map_or(Style::default(), |col| Style::default().fg(col))
            } else {
                fill_style(app, s, ch)
            };
            spans.push(Span::styled(ch.to_string(), st));
            spans.push(Span::raw(format!(" {}  ", sanitize(&se.label))));
        }
        let used: usize = spans.iter().map(|s| width(&s.content)).sum();
        if used > iw {
            let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
            lines.push(Line::from(fit(&text, iw, Align::Left)));
        } else {
            lines.push(Line::from(spans));
        }
    }
    let rows = vis.saturating_sub(lines.len());
    match c.cur.kind {
        ChartKind::Bar | ChartKind::Stacked => lines.extend(bar_lines(app, c, iw, rows, &g)),
        ChartKind::Line | ChartKind::Scatter => lines.extend(plot_lines(app, c, iw, rows, &g)),
        ChartKind::Heatmap => lines.extend(heat_lines(app, c, iw, rows, &g)),
    }
    // とても長い値の字などで中の幅を越える行は、枠を押し出さないように切る。
    lines.into_iter().map(|l| clip(l, iw)).collect()
}

/// 行を幅 `w` で切る(字の見た目は保つ)。
fn clip(line: Line<'static>, w: usize) -> Line<'static> {
    let total: usize = line.spans.iter().map(|s| width(&s.content)).sum();
    if total <= w {
        return line;
    }
    let mut out = Vec::new();
    let mut used = 0;
    for s in line.spans {
        let sw = width(&s.content);
        if used + sw <= w {
            used += sw;
            out.push(s);
            continue;
        }
        let rest = w - used;
        if rest > 0 {
            out.push(Span::styled(fit(&s.content, rest, Align::Left), s.style));
        }
        break;
    }
    Line::from(out).style(line.style)
}

/// CH-4: 棒と積み上げ棒。
fn bar_lines(app: &App, c: &Chart, iw: usize, rows: usize, g: &Glyphs) -> Vec<Line<'static>> {
    let d = &c.data;
    let agg = c.cur.agg;
    let lw = d
        .items
        .iter()
        .map(|i| width(&i.label))
        .max()
        .unwrap_or(1)
        .clamp(1, (iw / 3).max(4));
    let vals: Vec<Option<f64>> = d.items.iter().map(|i| item_value(i, agg)).collect();
    let vw = vals.iter().map(|v| width(&vnum(c, *v))).max().unwrap_or(1);
    let bw = iw.saturating_sub(1 + lw + 2 + 1 + vw + 1).max(1);
    let stacked = c.cur.kind == ChartKind::Stacked && !d.series.is_empty();
    let sums: Vec<f64> = d
        .items
        .iter()
        .map(|i| {
            i.parts
                .iter()
                .filter_map(|p| p.value(agg))
                .filter(|v| *v > 0.0)
                .sum()
        })
        .collect();
    let max = if stacked {
        sums.iter().copied().fold(0.0, f64::max)
    } else {
        vals.iter().flatten().copied().fold(0.0, f64::max)
    };
    let sel_st = sel_style(app);
    let first = scroll(c.top, c.sel, rows, d.items.len());
    let mut out = Vec::new();
    for (i, item) in d.items.iter().enumerate().skip(first).take(rows) {
        let on = i == c.sel;
        let head = format!(
            "{}{} {}",
            if on { ">" } else { " " },
            fit(&sanitize(&item.label), lw, Align::Left),
            g.vbar
        );
        let mut spans = vec![if on {
            Span::styled(head, sel_st)
        } else {
            Span::raw(head)
        }];
        let mut used;
        if stacked {
            // 内訳ごとの塗り。境を累積で丸めて、字の数のずれをためない。
            let total = if c.pct { sums[i] } else { max };
            let mut cum = 0.0;
            let mut at = 0usize;
            for (s, p) in item.parts.iter().enumerate() {
                let v = p.value(agg).filter(|v| *v > 0.0).unwrap_or(0.0);
                cum += v;
                let end = if total > 0.0 {
                    ((cum / total) * bw as f64).round() as usize
                } else {
                    0
                };
                let end = end.min(bw);
                if end > at {
                    let ch = if g.ascii {
                        FILLS_ASCII[s.min(5)]
                    } else {
                        FILLS[s.min(5)]
                    };
                    let st = fill_style(app, s, ch);
                    spans.push(Span::styled(ch.to_string().repeat(end - at), st));
                    at = end;
                }
            }
            used = at;
        } else {
            let t = vals[i]
                .filter(|v| *v > 0.0)
                .map_or(0.0, |v| v / max.max(f64::MIN_POSITIVE));
            let bar = bar_text(t, bw, g.ascii);
            used = width(&bar);
            let st = series_color(app, 0).map_or(Style::default(), |col| Style::default().fg(col));
            spans.push(Span::styled(bar, st));
        }
        let value = vnum(c, vals[i]);
        spans.push(Span::raw(format!(" {value}")));
        used += 1 + width(&value);
        let rest = iw.saturating_sub(1 + lw + 2 + used);
        spans.push(Span::raw(" ".repeat(rest)));
        out.push(Line::from(spans));
    }
    out
}

/// 点字の格子(横 2・縦 4 の点を1字に)。
struct Braille {
    w: usize,
    h: usize,
    bits: Vec<u8>,
    owner: Vec<usize>,
}

impl Braille {
    fn new(w: usize, h: usize) -> Braille {
        Braille {
            w,
            h,
            bits: vec![0; w * h],
            owner: vec![0; w * h],
        }
    }

    /// 点 (px, py)(点の単位)を打つ。
    fn dot(&mut self, px: usize, py: usize, s: usize) {
        let (cx, cy) = (px / 2, py / 4);
        if cx >= self.w || cy >= self.h {
            return;
        }
        const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
        let i = cy * self.w + cx;
        self.bits[i] |= BITS[px % 2][py % 4];
        self.owner[i] = s;
    }

    /// 点から点への線(ブレゼンハム)。
    fn line(&mut self, a: (i64, i64), b: (i64, i64), s: usize) {
        let (mut x0, mut y0) = a;
        let (x1, y1) = b;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && y0 >= 0 {
                self.dot(x0 as usize, y0 as usize, s);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}

/// CH-5・CH-6: 折れ線と散布図(左に目盛り、下に軸と区切りの名前)。
fn plot_lines(app: &App, c: &Chart, iw: usize, rows: usize, g: &Glyphs) -> Vec<Line<'static>> {
    let lay = plot_layout(c, iw, rows, g);
    let (pw, ph) = (lay.pw, lay.ph);
    // 字ごとの (字, 内訳, 選んでいる)。
    let mut cells: Vec<Option<(char, usize, bool)>> = vec![None; pw * ph];
    let d = &c.data;
    let mut xs = Vec::new();
    if c.cur.kind == ChartKind::Line {
        let vals = line_values(c);
        xs = line_xs(d.items.len(), pw);
        let mut br = Braille::new(pw, ph);
        let dot_y = |v: f64| -> i64 {
            let t = 1.0 - frac(v, lay.lo, lay.hi);
            (t * (ph * 4 - 1) as f64).round() as i64
        };
        for (s, line) in vals.iter().enumerate() {
            for k in 1..line.len() {
                if let (Some(a), Some(b)) = (line[k - 1], line[k]) {
                    br.line(
                        ((xs[k - 1] * 2) as i64, dot_y(a)),
                        ((xs[k] * 2) as i64, dot_y(b)),
                        s,
                    );
                }
            }
        }
        for (i, b) in br.bits.iter().enumerate() {
            if *b != 0 {
                let ch = char::from_u32(0x2800 + u32::from(*b)).unwrap_or('.');
                cells[i] = Some((ch, br.owner[i], false));
            }
        }
        for (s, line) in vals.iter().enumerate() {
            for (k, v) in line.iter().enumerate() {
                if let Some(v) = v {
                    let cy = (dot_y(*v) / 4).clamp(0, ph as i64 - 1) as usize;
                    let cx = xs[k].min(pw - 1);
                    cells[cy * pw + cx] = Some((MARKS[s.min(5)], s, false));
                }
            }
        }
    } else {
        let pos = scatter_cells(c, pw, ph);
        for (i, (p, (cx, cy))) in d.points.iter().zip(&pos).enumerate() {
            if i == c.sel {
                continue;
            }
            cells[cy * pw + cx] = Some((MARKS[p.s.min(5)], p.s, false));
        }
        if let (Some(p), Some((cx, cy))) = (d.points.get(c.sel), pos.get(c.sel)) {
            cells[cy * pw + cx] = Some((MARKS[p.s.min(5)], p.s, true));
        }
    }
    let sel_st = sel_style(app);
    let mut out = Vec::new();
    for r in 0..ph {
        let label = if r == 0 {
            y_label(c, lay.hi)
        } else if r + 1 == ph {
            y_label(c, lay.lo)
        } else {
            String::new()
        };
        let mut spans = vec![Span::raw(format!(
            "{}{}",
            fit(&label, lay.yw, Align::Right),
            g.vbar
        ))];
        let mut run = String::new();
        let flush = |run: &mut String, spans: &mut Vec<Span<'static>>| {
            if !run.is_empty() {
                spans.push(Span::raw(std::mem::take(run)));
            }
        };
        for x in 0..pw {
            match cells[r * pw + x] {
                Some((ch, s, on)) => {
                    flush(&mut run, &mut spans);
                    let st = if on {
                        sel_st
                    } else {
                        series_color(app, s)
                            .map_or(Style::default(), |col| Style::default().fg(col))
                    };
                    spans.push(Span::styled(ch.to_string(), st));
                }
                None => run.push(' '),
            }
        }
        flush(&mut run, &mut spans);
        out.push(Line::from(spans));
    }
    // 軸: 選んでいる区切りの下に `^`。
    let mut axis: Vec<char> = vec![g.hline; pw];
    if c.cur.kind == ChartKind::Line {
        if let Some(&x) = xs.get(c.sel) {
            axis[x.min(pw - 1)] = '^';
        }
    }
    out.push(Line::from(format!(
        "{}{}{}",
        " ".repeat(lay.yw),
        g.corner,
        axis.into_iter().collect::<String>()
    )));
    // 区切りの名前(最初と最後)。
    let (first, last) = if c.cur.kind == ChartKind::Line {
        (
            d.items.first().map(|i| i.label.clone()).unwrap_or_default(),
            d.items.last().map(|i| i.label.clone()).unwrap_or_default(),
        )
    } else {
        let (xl, xh) = value_range(d.points.iter().map(|p| p.x), false);
        (axis_num(xl, d.x_date), axis_num(xh, d.x_date))
    };
    let mut labels = format!("{}{}", " ".repeat(lay.x0), sanitize(&first));
    let room = (lay.x0 + pw).saturating_sub(width(&labels) + width(&last));
    if room >= 2 && last != first {
        labels.push_str(&" ".repeat(room));
        labels.push_str(&sanitize(&last));
    }
    out.push(Line::from(fit(&labels, iw, Align::Left)));
    out
}

/// CH-7: ヒートマップ。
fn heat_lines(app: &App, c: &Chart, iw: usize, rows: usize, g: &Glyphs) -> Vec<Line<'static>> {
    let d = &c.data;
    let agg = c.cur.agg;
    let lay = heat_layout(c, iw);
    let max = d
        .items
        .iter()
        .flat_map(|i| i.parts.iter().filter_map(|p| p.value(agg)))
        .fold(0.0, f64::max);
    let sel_st = sel_style(app);
    let paint = heat_paint(app);
    let mut out = Vec::new();
    // 見出し: 横の値。
    let mut head = format!("{}{} ", " ".repeat(1 + lay.lw + 1), g.vbar);
    for s in d.series.iter().skip(lay.left).take(lay.shown) {
        head.push_str(&fit(&sanitize(&s.label), lay.cw, Align::Right));
        head.push(' ');
    }
    if lay.left + lay.shown < d.series.len() {
        head.push('›');
    }
    out.push(Line::from(Span::styled(
        fit(&head, iw, Align::Left),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    let first = scroll(c.top, c.sel, rows.saturating_sub(1), d.items.len());
    for (i, item) in d
        .items
        .iter()
        .enumerate()
        .skip(first)
        .take(rows.saturating_sub(1))
    {
        let on_row = i == c.sel;
        let mut spans = vec![Span::raw(format!(
            "{}{} {}",
            if on_row { ">" } else { " " },
            fit(&sanitize(&item.label), lay.lw, Align::Left),
            g.vbar
        ))];
        for j in lay.left..lay.left + lay.shown {
            let v = item.parts.get(j).and_then(|p| p.value(agg));
            let level = match v {
                Some(v) if v > 0.0 && max > 0.0 => {
                    ((v / max) * 4.0).ceil().clamp(1.0, 4.0) as usize
                }
                _ => 0,
            };
            let shade = if g.ascii {
                SHADES_ASCII[level]
            } else {
                SHADES[level]
            };
            let text = fit(&vnum(c, v), lay.cw.saturating_sub(1), Align::Right);
            // chart.heatmap-shade-blend: 色のあるときの濃さの字は、ます目の地と同じ色(塊に見せない)。
            let (shade_st, st) = if on_row && j == c.sel2 {
                let blend = match (&paint, sel_st.bg) {
                    (Some(_), Some(b)) => Style::default().fg(b).bg(b),
                    _ => sel_st,
                };
                (blend, sel_st)
            } else {
                match (&paint, level.checked_sub(1)) {
                    (Some(p), Some(i)) => {
                        let (bg, fg) = p[i];
                        (
                            Style::default().fg(bg).bg(bg),
                            Style::default().fg(fg).bg(bg),
                        )
                    }
                    _ => (Style::default(), Style::default()),
                }
            };
            spans.push(Span::raw(" "));
            spans.push(Span::styled(shade.to_string(), shade_st));
            spans.push(Span::styled(text, st));
        }
        out.push(Line::from(spans));
    }
    out
}

/// ヒートマップの濃さの段 1〜4 の (地の色, 数の字の色)。色を使わない表示では None。
///
/// chart.heatmap-readable: 地は、画面の地からテーマのアクセント(列の見出しの色。既定のテーマは青)へ寄せた色で、
/// 段が上がるほど画面の地から遠い(明るい地では濃く、暗い地では明るく)。256 色で前の段と同じ番号になるときは、
/// 番号が変わるまで寄せ、アクセントでも変わらなければアクセントから文字の色へ寄せる。数の字は、テーマの文字の色か
/// 地の色のうち対比 4.5 以上のもの、無ければそれを黒か白へ少しずつ寄せて 4.5 に届いた色。
fn heat_paint(app: &App) -> Option<[(Color, Color); 4]> {
    const STEPS: [f32; 4] = [0.3, 0.55, 0.8, 1.0];
    let indexed = match app.color {
        ColorMode::None => return None,
        ColorMode::Indexed => true,
        ColorMode::Rgb => false,
    };
    let base = ground(app);
    let light = is_light(base);
    let (text, accent) = match app.palette() {
        Some(p) => (p.fg, p.colhead),
        None if light => ([0, 0, 0], [34, 98, 150]),
        None => ([255, 255, 255], [56, 156, 214]),
    };
    let at = |t: f32| {
        if t <= 1.0 {
            super::look::mix(base, accent, t)
        } else {
            super::look::mix(accent, text, t - 1.0)
        }
    };
    let mut out = [(Color::Reset, Color::Reset); 4];
    let mut prev = shown(base, indexed);
    let mut last = 0.0f32;
    for (k, step) in STEPS.into_iter().enumerate() {
        let mut t = step.max(last);
        let mut c = at(t);
        while shown(c, indexed) == prev && t < 1.95 {
            t += 0.05;
            c = at(t);
        }
        prev = shown(c, indexed);
        last = t;
        let fg = readable_on(c, [text, base], indexed);
        out[k] = (term_color(c, indexed), term_color(fg, indexed));
    }
    Some(out)
}

/// 地 `cell` の上で対比 4.5 以上になる字の色。`cands` の順に、そのまま → 黒か白(地と逆の向き)へ 1/4 ずつ寄せた色を
/// 試し、最初に届いた色。どれも届かなければ黒と白の対比の大きい方。
fn readable_on(cell: [u8; 3], cands: [[u8; 3]; 2], indexed: bool) -> [u8; 3] {
    const BLACK: [u8; 3] = [0, 0, 0];
    const WHITE: [u8; 3] = [255, 255, 255];
    let on = shown(cell, indexed);
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        for base in cands {
            let far = if luminance(base) < luminance(cell) {
                BLACK
            } else {
                WHITE
            };
            let c = super::look::mix(base, far, t);
            if contrast(shown(c, indexed), on) >= 4.5 {
                return c;
            }
        }
    }
    if contrast(BLACK, on) >= contrast(WHITE, on) {
        BLACK
    } else {
        WHITE
    }
}

/// 画面の全部の行(幅 `w`、高さ `h`)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(c) = app.chart.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let mut lines = super::bands::top_lines(app, w);
    lines.push(super::bands::header(app, w));
    for _ in 0..super::bands::bar_padding(app) {
        lines.push(Line::from(" ".repeat(w)));
    }
    let bh = h.saturating_sub(lines.len() + 2);
    let iw = w.saturating_sub(2);
    let body = body_lines(app, c, iw, bh.saturating_sub(2));
    lines.extend(panel(app, &title(app, c), body, w, bh, true));
    let body_end = h.saturating_sub(2);
    while lines.len() < body_end {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body_end);
    // REL-8: 画面の型のメニューの窓(グラフから開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // workspace.switch-open: お気に入りのワークスペースの切り替えの一覧。
    super::switch::overlay(app, &mut lines, w);
    // パレット(グラフから開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
