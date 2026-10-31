//! ガント(specs/gantt/spec.md の GA-1〜GA-7)の核。画面に依存しない: 印の設定と列の名前からの推定(GA-1)、
//! 時間軸の単位(日・週・月)とマスの往復(GA-2)、棒の範囲と親の棒(GA-3)。日は 1970-01-01 からの日数
//! (`types::parse_date` と同じ)。

use crate::i18n::{Lang, Msg};
use crate::types::{add_months, clamp_date, format_date, parse_date};

/// GA-1: 設定が無いときに始まりの列とみなす名前(大文字小文字を問わない)。全部の言語の表(locales の
/// `GanttStartNames`、空白区切り)から集める: 訳ではなく、どの言語の画面でも同じ名前を推定に使う
/// (コードに日本語のリテラルを置かない。SR-23)。
pub fn start_names() -> Vec<&'static str> {
    names(Msg::GanttStartNames)
}

/// GA-1: 設定が無いときに終わりの列とみなす名前(大文字小文字を問わない)。`GanttEndNames` から。
pub fn end_names() -> Vec<&'static str> {
    names(Msg::GanttEndNames)
}

fn names(m: Msg) -> Vec<&'static str> {
    Lang::ALL
        .iter()
        .flat_map(|l| m.get(*l).split_whitespace())
        .collect()
}

/// 印の `gantt = { start, end, days }`、または推定した列。`end` か `days` のどちらかで終わりを決める。
/// 推定では `start`・`end` の片方だけになることがある(その行は印になる。GA-2)。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GanttSpec {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<String>,
}

impl GanttSpec {
    /// 使う列の名前(ある分だけ)。
    pub fn columns(&self) -> Vec<&str> {
        [&self.start, &self.end, &self.days]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect()
    }
}

/// GA-1: 列の名前と日付の列かの組から、始まりと終わりの列を推定する。始まりは `start_names()` に当たる最初の
/// 日付の列、終わりは `end_names()` に当たる最初の日付の列(始まりと同じ列は選ばない)。どちらも無ければ None。
pub fn infer(cols: &[(String, bool)]) -> Option<GanttSpec> {
    let pick = |names: &[&str], skip: Option<&str>| -> Option<String> {
        cols.iter()
            .filter(|(_, is_date)| *is_date)
            .map(|(n, _)| n)
            .find(|n| {
                let lower = n.to_lowercase();
                names.iter().any(|x| *x == lower) && skip != Some(n.as_str())
            })
            .cloned()
    };
    let start = pick(&start_names(), None);
    let end = pick(&end_names(), start.as_deref());
    if start.is_none() && end.is_none() {
        return None;
    }
    Some(GanttSpec {
        start,
        end,
        days: None,
    })
}

/// GA-2: 時間軸の1マスの単位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Day,
    Week,
    Month,
}

impl Unit {
    /// 1段細かい単位(日より細かい単位は無いので、日は日のまま)。
    pub fn finer(self) -> Unit {
        match self {
            Unit::Day => Unit::Day,
            Unit::Week => Unit::Day,
            Unit::Month => Unit::Week,
        }
    }

    /// 1段粗い単位(月より粗い単位は無いので、月は月のまま)。
    pub fn coarser(self) -> Unit {
        match self {
            Unit::Day => Unit::Week,
            Unit::Week => Unit::Month,
            Unit::Month => Unit::Month,
        }
    }
}

/// 日 `d` の (年, 月, 日)。
pub fn civil(d: i64) -> (i64, u32, u32) {
    let s = format_date(d);
    let mut it = s.split('-');
    let y = it.next().and_then(|x| x.parse().ok()).unwrap_or(1970);
    let m = it.next().and_then(|x| x.parse().ok()).unwrap_or(1);
    let day = it.next().and_then(|x| x.parse().ok()).unwrap_or(1);
    (y, m, day)
}

/// GA-2: 日 `d` を単位の境に切り下げる(週は月曜、月は1日、日はそのまま)。
pub fn floor_unit(d: i64, unit: Unit) -> i64 {
    match unit {
        Unit::Day => d,
        // 1970-01-01 は木曜。月曜からの日数は (d + 3) mod 7。
        Unit::Week => d - (d + 3).rem_euclid(7),
        Unit::Month => {
            let (y, m, _) = civil(d);
            parse_date(&format!("{y:04}-{m:02}-01")).unwrap_or(d)
        }
    }
}

/// GA-2: 日 `d` が、左端のマスの始まりの日 `origin`(単位の境)から数えて何マス目か(左は負)。
pub fn cell_of(d: i64, origin: i64, unit: Unit) -> i64 {
    match unit {
        Unit::Day => d - origin,
        Unit::Week => (d - origin).div_euclid(7),
        Unit::Month => {
            let (oy, om, _) = civil(origin);
            let (y, m, _) = civil(d);
            (y - oy) * 12 + (i64::from(m) - i64::from(om))
        }
    }
}

/// GA-2: `origin` から `k` マス目の始まりの日。
pub fn cell_start(k: i64, origin: i64, unit: Unit) -> i64 {
    match unit {
        Unit::Day => origin + k,
        Unit::Week => origin + 7 * k,
        Unit::Month => add_months(origin, i32::try_from(k).unwrap_or(i32::MAX)),
    }
}

/// GA-2: 範囲 `first..=last`(日)が幅 `cols` のマスに収まる、最も細かい単位(日 → 週 → 月。月でも収まらなければ月)。
pub fn fit_unit(first: i64, last: i64, cols: usize) -> Unit {
    let cols = i64::try_from(cols.max(1)).unwrap_or(i64::MAX);
    for unit in [Unit::Day, Unit::Week, Unit::Month] {
        let origin = floor_unit(first.min(last), unit);
        if cell_of(first.max(last), origin, unit) < cols {
            return unit;
        }
    }
    Unit::Month
}

/// 1行の棒(日)。片方だけなら印(◆)の行、どちらも無ければ棒なし。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bar {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl Bar {
    /// GA-1: 始まり・終わり・日数の値から棒を決める。終わりが無く日数(1 以上)があれば `start + days - 1`、
    /// 始まりが無く終わりと日数があれば `end - days + 1`。始まりが終わりより後なら入れ替える。
    /// 日数が大きすぎても溢れず、日付の範囲(0000-01-01〜9999-12-31)に収める(年が負になると
    /// `civil` が読めない)。
    pub fn from_values(start: Option<i64>, end: Option<i64>, days: Option<i64>) -> Bar {
        let n = days.filter(|d| *d >= 1);
        let end2 = match (start, end, n) {
            (_, Some(e), _) => Some(e),
            (Some(s), None, Some(n)) => Some(clamp_date(s.saturating_add(n - 1))),
            _ => None,
        };
        let start2 = match (start, end, n) {
            (Some(s), _, _) => Some(s),
            (None, Some(e), Some(n)) => Some(clamp_date(e.saturating_sub(n - 1))),
            _ => None,
        };
        match (start2, end2) {
            (Some(s), Some(e)) if s > e => Bar {
                start: Some(e),
                end: Some(s),
            },
            _ => Bar {
                start: start2,
                end: end2,
            },
        }
    }

    /// 片方だけの行(印)。
    pub fn is_point(&self) -> bool {
        self.start.is_some() != self.end.is_some()
    }

    /// 印の日(片方だけのとき)。
    pub fn point(&self) -> Option<i64> {
        if self.is_point() {
            self.start.or(self.end)
        } else {
            None
        }
    }

    /// 棒の始まりと終わり(両方あるとき)。
    pub fn range(&self) -> Option<(i64, i64)> {
        Some((self.start?, self.end?))
    }

    /// 何か日付を持つか。
    pub fn is_some(&self) -> bool {
        self.start.is_some() || self.end.is_some()
    }

    /// 最初の日と最後の日(印はその日が両方)。
    pub fn bounds(&self) -> Option<(i64, i64)> {
        match (self.start, self.end) {
            (Some(s), Some(e)) => Some((s.min(e), s.max(e))),
            (Some(d), None) | (None, Some(d)) => Some((d, d)),
            (None, None) => None,
        }
    }

    /// GA-3: 子孫の棒を包む範囲(最初の日の最小から最後の日の最大)。日付の無い子は数えない。
    pub fn span(bars: &[Bar]) -> Bar {
        let mut out: Option<(i64, i64)> = None;
        for b in bars {
            if let Some((s, e)) = b.bounds() {
                out = Some(match out {
                    Some((s0, e0)) => (s0.min(s), e0.max(e)),
                    None => (s, e),
                });
            }
        }
        match out {
            Some((s, e)) => Bar {
                start: Some(s),
                end: Some(e),
            },
            None => Bar::default(),
        }
    }
}

#[cfg(test)]
#[path = "test_gantt_unit.rs"]
mod tests;
