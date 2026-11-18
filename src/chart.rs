//! グラフ(specs/_history/decidespec/chart/spec.md の CH-1〜CH-9)の核。画面に依存しない: 種類と集め方の名前(CH-1)、印の設定の形
//! (CH-1)、分ける列の推定(CH-2)、数え方(件数・合計・平均・最小・最大)と数の列の区間・数の書き方(CH-3)。
//! 日付の区切りはガントの単位(`gantt::Unit`。日・週・月)を使う。

use crate::gantt::{floor_unit, Unit};
use crate::types::{format_date, Kind};

/// グラフの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    /// 棒(1項目1行の横棒)。
    Bar,
    /// 積み上げ棒(内訳の列で塗り分けた横棒)。
    Stacked,
    /// 折れ線(日付か数の列の区切りごとの値)。
    Line,
    /// 散布図(数か日付の列2つ)。
    Scatter,
    /// ヒートマップ(2つの列の値の組ごとの値)。
    Heatmap,
}

impl ChartKind {
    /// 全部の種類(`Tab` で移る順)。
    pub const ALL: [ChartKind; 5] = [
        ChartKind::Bar,
        ChartKind::Stacked,
        ChartKind::Line,
        ChartKind::Scatter,
        ChartKind::Heatmap,
    ];

    /// 印に書く名前。
    pub fn name(self) -> &'static str {
        match self {
            ChartKind::Bar => "bar",
            ChartKind::Stacked => "stacked",
            ChartKind::Line => "line",
            ChartKind::Scatter => "scatter",
            ChartKind::Heatmap => "heatmap",
        }
    }

    /// 印の名前から(前後の空白と大文字小文字を問わない)。
    pub fn parse(s: &str) -> Option<ChartKind> {
        let s = s.trim().to_lowercase();
        ChartKind::ALL.into_iter().find(|k| k.name() == s)
    }

    /// 内訳の列を使う種類か(積み上げ棒・ヒートマップは要る。折れ線・散布図は任意)。
    pub fn needs_by(self) -> bool {
        matches!(self, ChartKind::Stacked | ChartKind::Heatmap)
    }

    /// 内訳の列を持てる種類か。
    pub fn takes_by(self) -> bool {
        !matches!(self, ChartKind::Bar)
    }
}

/// 値の集め方。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agg {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

impl Agg {
    /// 全部の集め方(`a` で移る順。件数は値の列が無いときだけ)。
    pub const ALL: [Agg; 5] = [Agg::Count, Agg::Sum, Agg::Avg, Agg::Min, Agg::Max];

    /// 印に書く名前。
    pub fn name(self) -> &'static str {
        match self {
            Agg::Count => "count",
            Agg::Sum => "sum",
            Agg::Avg => "avg",
            Agg::Min => "min",
            Agg::Max => "max",
        }
    }

    /// 印の名前から(前後の空白と大文字小文字を問わない)。
    pub fn parse(s: &str) -> Option<Agg> {
        let s = s.trim().to_lowercase();
        Agg::ALL.into_iter().find(|a| a.name() == s)
    }
}

/// 印の `chart = { kind, x, y, agg, by, unit, cumulative, name }` の1つ。省いた鍵は None(画面が推定で埋める)。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChartSpec {
    pub name: Option<String>,
    pub kind: Option<ChartKind>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub agg: Option<Agg>,
    pub by: Option<String>,
    pub unit: Option<Unit>,
    pub cumulative: bool,
}

impl ChartSpec {
    /// 書いた列の名前(鍵の名前と列)。表にあるかを確かめる(CH-1)。
    pub fn columns(&self) -> Vec<(&'static str, &str)> {
        [("x", &self.x), ("y", &self.y), ("by", &self.by)]
            .into_iter()
            .filter_map(|(k, c)| c.as_deref().map(|c| (k, c)))
            .collect()
    }
}

/// 推定に使う列の様子。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColInfo {
    pub name: String,
    pub kind: Kind,
    /// 列の決まり(CE-36)の `values` がある。
    pub has_values: bool,
}

/// CH-2: 設定が無いときの分ける列。状態の列の名前(KB-1 と同じ名前で、リストでない列)→ 決まりの `values` の
/// ある列 → チェックボックスの列 → リストの列 → 日付の列 → 最初の列。
pub fn infer_x(cols: &[ColInfo]) -> Option<String> {
    let names = crate::kanban::column_names();
    let status = |c: &&ColInfo| {
        c.kind != Kind::List && {
            let lower = c.name.to_lowercase();
            names.iter().any(|n| n.to_lowercase() == lower)
        }
    };
    let rules: [&dyn Fn(&&ColInfo) -> bool; 5] = [
        &status,
        &|c| c.has_values,
        &|c| c.kind == Kind::Checkbox,
        &|c| c.kind == Kind::List,
        &|c| matches!(c.kind, Kind::Date | Kind::DateTime),
    ];
    rules
        .iter()
        .find_map(|f| cols.iter().find(f))
        .or_else(|| cols.first())
        .map(|c| c.name.clone())
}

/// 折れ線と散布図の横の軸にできる列(日付・日時・数)。
pub fn axis_ok(kind: Kind) -> bool {
    matches!(kind, Kind::Date | Kind::DateTime | Kind::Number)
}

/// 内訳にできる列(文字・チェックボックス・リスト)。
pub fn by_ok(kind: Kind) -> bool {
    matches!(kind, Kind::Text | Kind::Checkbox | Kind::List)
}

/// CH-3: 項目1つの数え。行の数と、数の値のある行の合計・最小・最大。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Acc {
    rows: usize,
    nums: usize,
    sum: f64,
    min: f64,
    max: f64,
}

impl Acc {
    /// 1行を数える。`v` が Some なら数も足す(数でない値の行は None で、件数にだけ入る)。
    pub fn add(&mut self, v: Option<f64>) {
        self.rows += 1;
        if let Some(v) = v.filter(|v| v.is_finite()) {
            if self.nums == 0 {
                self.min = v;
                self.max = v;
            } else {
                self.min = self.min.min(v);
                self.max = self.max.max(v);
            }
            self.nums += 1;
            self.sum += v;
        }
    }

    /// 集めた値。件数は行の数。合計・平均・最小・最大は数のある行だけで、1つも無ければ None。
    pub fn value(&self, agg: Agg) -> Option<f64> {
        if agg == Agg::Count {
            return Some(self.rows as f64);
        }
        if self.nums == 0 {
            return None;
        }
        // 合計があふれた(f64 を越えた)ときは値なしにする(棒の長さの比が崩れないように)。
        Some(match agg {
            Agg::Count => self.rows as f64,
            Agg::Sum => self.sum,
            Agg::Avg => self.sum / self.nums as f64,
            Agg::Min => self.min,
            Agg::Max => self.max,
        })
        .filter(|v| v.is_finite())
    }

    /// 数えた行の数。
    pub fn rows(&self) -> usize {
        self.rows
    }
}

/// CH-3: 数の列の区間の数の上限。
pub const MAX_BINS: usize = 10;

/// CH-3: 数の列の区間。整数だけで幅(最大 − 最小 + 1)が 10 以下なら1ずつ(`(v, v + 1)`)、同じ値だけなら1つ、
/// ほかは最小から最大までを等しい幅の 10 の区間にする。区間は下の端を含み上の端を含まない(最後の区間は上の端も含む)。
pub fn bins(min: f64, max: f64, all_int: bool) -> Vec<(f64, f64)> {
    if !min.is_finite() || !max.is_finite() || max < min {
        return Vec::new();
    }
    if all_int && max - min < MAX_BINS as f64 {
        let n = (max - min) as usize + 1;
        return (0..n)
            .map(|i| (min + i as f64, min + i as f64 + 1.0))
            .collect();
    }
    if max == min {
        return vec![(min, max)];
    }
    // 端は最小と最大の重みつきの和で出す(-1e308 と 1e308 のように差が f64 を越える値でも、端が有限になる)。
    let n = MAX_BINS as f64;
    let at = |i: usize| -> f64 {
        match i {
            0 => min,
            MAX_BINS => max,
            _ => {
                let t = i as f64 / n;
                min * (1.0 - t) + max * t
            }
        }
    };
    (0..MAX_BINS).map(|i| (at(i), at(i + 1))).collect()
}

/// 値 `v` の入る区間の番号(最後の区間は上の端も含む)。どこにも入らなければ None。
pub fn bin_of(bins: &[(f64, f64)], v: f64) -> Option<usize> {
    let last = bins.len().checked_sub(1)?;
    bins.iter()
        .enumerate()
        .position(|(i, (lo, hi))| v >= *lo && (v < *hi || (i == last && v <= *hi)))
}

/// 区間の名前。1ずつの整数の区間はその数、ほかは `下~上`。端は小数2桁までで書き、区間の幅が狭いときは、隣の区間と
/// 名前が重ならない桁まで足す(0.123~0.126 が 0.12~0.13 に丸まって並ばないように)。
pub fn bin_label(b: (f64, f64), unit_int: bool) -> String {
    if unit_int {
        return fmt_num(b.0);
    }
    // 幅は半分にしてから出す(端の差が f64 を越えないように)。
    let w = (b.1 * 0.5 - b.0 * 0.5) * 2.0;
    let digits = if w > 0.0 && w.is_finite() {
        ((-w.log10()).ceil().max(0.0) as usize).clamp(2, 12)
    } else {
        2
    };
    format!("{}~{}", fmt_dec(b.0, digits), fmt_dec(b.1, digits))
}

/// 数を小数 `digits` 桁までで書き、末尾の0を除く(とても大きい数は `fmt_num` と同じ指数の形)。
fn fmt_dec(v: f64, digits: usize) -> String {
    if !v.is_finite() || v.abs() >= 1e15 {
        return fmt_num(v);
    }
    let s = format!("{v:.digits$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    if s == "-0" {
        "0".to_string()
    } else {
        s
    }
}

/// 数の書き方: 整数ならそのまま、ほかは小数2桁までで末尾の0を除く。
pub fn fmt_num(v: f64) -> String {
    if !v.is_finite() {
        return "-".to_string();
    }
    // とても大きい数は丸めずに指数の形で書く(100 倍で f64 を越えないように。桁が長すぎて枠を押さないように)。
    if v.abs() >= 1e15 {
        return format!("{v:e}");
    }
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() && r.abs() < 1e15 {
        // -0 を 0 にする。
        return format!("{}", r as i64);
    }
    let s = format!("{r:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// 日付の区切りの名前(区切りの始まりの日 `start_day`)。日 `2026-10-01`、週は週の始まりの日、月 `2026-10`。
pub fn bucket_label(start_day: i64, unit: Unit) -> String {
    let d = format_date(floor_unit(start_day, unit));
    match unit {
        Unit::Month => d.get(..7).unwrap_or(&d).to_string(),
        Unit::Day | Unit::Week => d,
    }
}
