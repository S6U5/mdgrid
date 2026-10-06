//! `.base` のビューの組み込みの集計(BV-14)。(集計の名前, 値の並び)→ 結果の値 の純関数。
//! 意味は Obsidian のヘルプ(bases/syntax の Summaries)。形は docs/design.md。

use crate::expr::Val;
use crate::i18n::Msg;

/// 組み込みの集計。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Summary {
    Average,
    Min,
    Max,
    Sum,
    Range,
    Median,
    Stddev,
    Earliest,
    Latest,
    Checked,
    Unchecked,
    Empty,
    Filled,
    Unique,
}

impl Summary {
    /// 全部の集計。
    pub const ALL: [Summary; 14] = [
        Summary::Average,
        Summary::Min,
        Summary::Max,
        Summary::Sum,
        Summary::Range,
        Summary::Median,
        Summary::Stddev,
        Summary::Earliest,
        Summary::Latest,
        Summary::Checked,
        Summary::Unchecked,
        Summary::Empty,
        Summary::Filled,
        Summary::Unique,
    ];

    /// `.base` に書く名前(Obsidian の名前)。
    pub fn key(self) -> &'static str {
        self.msg().en()
    }

    /// `.base` の名前を読む(Obsidian と同じ綴り)。組み込みでなければ None(式の集計など)。
    pub fn parse(name: &str) -> Option<Summary> {
        let name = name.trim();
        Summary::ALL.into_iter().find(|s| s.key() == name)
    }

    /// 今の言語の見せ方の名前(SR-23)。英語は Obsidian の名前。
    pub fn label(self) -> &'static str {
        self.msg().text()
    }

    fn msg(self) -> Msg {
        match self {
            Summary::Average => Msg::SummaryAverage,
            Summary::Min => Msg::SummaryMin,
            Summary::Max => Msg::SummaryMax,
            Summary::Sum => Msg::SummarySum,
            Summary::Range => Msg::SummaryRange,
            Summary::Median => Msg::SummaryMedian,
            Summary::Stddev => Msg::SummaryStddev,
            Summary::Earliest => Msg::SummaryEarliest,
            Summary::Latest => Msg::SummaryLatest,
            Summary::Checked => Msg::SummaryChecked,
            Summary::Unchecked => Msg::SummaryUnchecked,
            Summary::Empty => Msg::SummaryEmpty,
            Summary::Filled => Msg::SummaryFilled,
            Summary::Unique => Msg::SummaryUnique,
        }
    }
}

/// 日付・日時を秒にする(日付はその日の 0 時)。
fn secs(v: &Val) -> Option<i64> {
    match v {
        Val::Date(d) => Some(d.saturating_mul(86_400)),
        Val::DateTime(s) => Some(*s),
        _ => None,
    }
}

fn is_empty(v: &Val) -> bool {
    match v {
        Val::Null => true,
        Val::Str(s) => s.is_empty(),
        Val::List(xs) => xs.is_empty(),
        _ => false,
    }
}

/// 種類を見分ける鍵(型の印と値)。同じ文字でも数と文字は別の種類。
fn unique_key(v: &Val) -> (u8, String) {
    match v {
        Val::Null => (0, String::new()),
        Val::Bool(b) => (1, b.to_string()),
        Val::Num(f) => (2, f.to_bits().to_string()),
        Val::Str(s) => (3, s.clone()),
        Val::Date(d) => (4, d.to_string()),
        Val::DateTime(s) => (5, s.to_string()),
        Val::Duration(ms) => (6, ms.to_string()),
        Val::List(xs) => (
            7,
            xs.iter()
                .map(|x| {
                    let (t, s) = unique_key(x);
                    format!("{t}:{}:{s}", s.len())
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
    }
}

/// 有限でない数(あふれ)は Null。
fn num(f: f64) -> Val {
    if f.is_finite() {
        Val::Num(f)
    } else {
        Val::Null
    }
}

fn count(n: usize) -> Val {
    Val::Num(n as f64)
}

/// 値の並びを集計する(並びは1回だけ走査する)。型の合わない値は数えない。
/// 数・日付の集計で数える値が無ければ Null(空欄)。数えるだけの集計(Checked・Unchecked・Empty・Filled・Unique)は数。
pub fn compute(s: Summary, values: impl IntoIterator<Item = Val>) -> Val {
    let values = values.into_iter();
    match s {
        Summary::Checked | Summary::Unchecked => {
            let want = s == Summary::Checked;
            count(values.filter(|v| *v == Val::Bool(want)).count())
        }
        Summary::Empty => count(values.filter(is_empty).count()),
        Summary::Filled => count(values.filter(|v| !is_empty(v)).count()),
        Summary::Unique => {
            let mut seen = std::collections::HashSet::new();
            for v in values.filter(|v| !is_empty(v)) {
                seen.insert(unique_key(&v));
            }
            count(seen.len())
        }
        Summary::Earliest | Summary::Latest => {
            let mut best: Option<(i64, Val)> = None;
            for v in values {
                let Some(t) = secs(&v) else { continue };
                let better = match &best {
                    None => true,
                    Some((b, _)) if s == Summary::Earliest => t < *b,
                    Some((b, _)) => t > *b,
                };
                if better {
                    best = Some((t, v));
                }
            }
            best.map_or(Val::Null, |(_, v)| v)
        }
        Summary::Median => {
            let mut xs: Vec<f64> = values
                .filter_map(|v| match v {
                    Val::Num(f) => Some(f),
                    _ => None,
                })
                .collect();
            if xs.is_empty() {
                return Val::Null;
            }
            xs.sort_by(f64::total_cmp);
            let m = xs.len() / 2;
            if xs.len() % 2 == 1 {
                num(xs[m])
            } else {
                num(xs[m - 1] / 2.0 + xs[m] / 2.0)
            }
        }
        Summary::Average
        | Summary::Min
        | Summary::Max
        | Summary::Sum
        | Summary::Range
        | Summary::Stddev => numeric(s, values),
    }
}

/// 数の集計と、Range(数が無ければ日付の Latest − Earliest)。Welford の方法で1回の走査。
fn numeric(s: Summary, values: impl Iterator<Item = Val>) -> Val {
    let (mut n, mut sum, mut mean, mut m2) = (0usize, 0.0f64, 0.0f64, 0.0f64);
    let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut dates: Option<(i64, i64)> = None;
    for v in values {
        match v {
            Val::Num(x) => {
                n += 1;
                sum += x;
                let d = x - mean;
                mean += d / n as f64;
                m2 += d * (x - mean);
                min = min.min(x);
                max = max.max(x);
            }
            other if s == Summary::Range => {
                if let Some(t) = secs(&other) {
                    dates = Some(dates.map_or((t, t), |(a, b)| (a.min(t), b.max(t))));
                }
            }
            _ => {}
        }
    }
    if n == 0 {
        return match (s, dates) {
            (Summary::Range, Some((a, b))) => {
                Val::Duration(b.saturating_sub(a).saturating_mul(1000))
            }
            _ => Val::Null,
        };
    }
    match s {
        Summary::Sum => num(sum),
        Summary::Average => num(sum / n as f64),
        Summary::Min => num(min),
        Summary::Max => num(max),
        Summary::Range => num(max - min),
        _ => num((m2 / n as f64).sqrt()),
    }
}

#[cfg(test)]
#[path = "test_summary_unit.rs"]
mod tests;
