//! 日付の言葉(cell-edit.date-words・cell-edit.date-words-weekday・cell-edit.date-words-period)。
//!
//! 「明日」「来週の金曜」「next fri」「+2w」のような言葉を、今日と週の始まり(dates.week_start)から日にする。
//! 文言の言語によらず、日本語と英語の言葉をどちらも読む(訳の台帳は使わない。読み方は画面の言語で変えない)。
//! ここは言葉を日数にするだけで、範囲の確かめと、ほかの形(`YYYY-MM-DD`・設定の形・`+3`)との順は
//! `types::parse_date_input` が決める。ノートに書くのはいつも `YYYY-MM-DD`。
//!
//! 読む言葉(英語は大文字と小文字を区別しない。全角の英数字と空白は半角として読む):
//! - 日: `今日`・`today`、`明日`・`tomorrow`、`明後日`、`昨日`・`yesterday`、`一昨日`(かなの書き方も)
//! - 隔たり: `3日後`・`2日前`・`2週間後`・`1か月後`・`1年後`、`in 3 days`・`3 days ago`、`3d`・`+3d`・`-2d`・`+2w`・`+1m`・`+1y`
//!   (月と年は同じ日、無ければその月の最後の日。月の終わりを越えない)
//! - 曜日: `金曜`・`金曜日`・`fri`・`friday`(今日より後の最初のその曜日)、`今週の金曜`・`this fri`(今の週)、
//!   `来週の金曜`・`next fri`(次の週)。週は dates.week_start で区切る
//! - 区切り: `来週`・`next week`(次の週の初めの日)、`来月`・`next month`(次の月の1日)、`月末`・`eom`・
//!   `end of month`(今月の最後の日)、`来月末`・`end of next month`(次の月の最後の日)

use crate::types::{self, WeekStart};

/// 言葉を日(1970-01-01 からの日数)にする。言葉でなければ None。
/// 日付の範囲(0000-01-01〜9999-12-31)の外になる言葉(`+99999y` など)は、範囲の外の日を返す(呼び手が確かめる)。
pub fn read(text: &str, today: i64, week: WeekStart) -> Option<i64> {
    let spaced = normalize(text);
    // 日本語の言葉は空白を挟んでも同じに読む(`来週 の 金曜`)。
    let compact: String = spaced.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }
    fixed(&compact, today)
        .or_else(|| offset(&compact, &spaced, today))
        .or_else(|| weekday(&compact, &spaced, today, week))
        .or_else(|| period(&compact, &spaced, today, week))
}

/// 前後の空白を除き、全角の英数字・記号と全角の空白を半角にし、英字を小文字にし、続く空白を1つにする。
fn normalize(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let c = match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(c) - 0xFEE0).unwrap_or(c),
            '\u{3000}' => ' ',
            c => c,
        };
        out.extend(c.to_lowercase());
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 今日からの日数が決まった言葉。
const FIXED: [(&[&str], i64); 5] = [
    (&["今日", "きょう", "today"], 0),
    (&["明日", "あした", "あす", "tomorrow"], 1),
    (&["明後日", "あさって"], 2),
    (&["昨日", "きのう", "yesterday"], -1),
    (&["一昨日", "おととい"], -2),
];

fn fixed(compact: &str, today: i64) -> Option<i64> {
    FIXED
        .iter()
        .find(|(words, _)| words.contains(&compact))
        .map(|(_, n)| today + n)
}

/// 隔たりの単位。
#[derive(Clone, Copy)]
enum Unit {
    Day,
    Week,
    Month,
    Year,
}

/// 日本語の単位の書き方(長いものから)。
const JA_UNITS: [(&str, Unit); 10] = [
    ("週間", Unit::Week),
    ("か月", Unit::Month),
    ("ヶ月", Unit::Month),
    ("ケ月", Unit::Month),
    ("カ月", Unit::Month),
    ("ヵ月", Unit::Month),
    ("箇月", Unit::Month),
    ("日", Unit::Day),
    ("週", Unit::Week),
    ("年", Unit::Year),
];

/// 英語の単位の書き方(単数と複数)。
fn en_unit(s: &str) -> Option<Unit> {
    match s {
        "day" | "days" => Some(Unit::Day),
        "week" | "weeks" => Some(Unit::Week),
        "month" | "months" => Some(Unit::Month),
        "year" | "years" => Some(Unit::Year),
        _ => None,
    }
}

/// 隔たりの言葉: `3日後`・`2日前`、`in 3 days`・`3 days ago`、`3d`・`+3d`・`-2d`(d・w・m・y)。
fn offset(compact: &str, spaced: &str, today: i64) -> Option<i64> {
    // 3日後・2週間前(数の後ろに単位と「後」か「前」)。
    let (n, rest) = split_number(compact);
    if let Some(n) = n {
        let ja = rest
            .strip_suffix('後')
            .map(|u| (u, 1))
            .or_else(|| rest.strip_suffix('前').map(|u| (u, -1)));
        if let Some((u, sign)) = ja {
            if let Some((_, unit)) = JA_UNITS.iter().find(|(w, _)| *w == u) {
                return Some(shift(today, sign * n, *unit));
            }
        }
    }
    // +3d・-2w・3m(符号は無くてもよい)。
    let (sign, body) = match compact.as_bytes().first() {
        Some(b'+') => (1, &compact[1..]),
        Some(b'-') => (-1, &compact[1..]),
        _ => (1, compact),
    };
    if let (Some(n), rest) = split_number(body) {
        let unit = match rest {
            "d" => Some(Unit::Day),
            "w" => Some(Unit::Week),
            "m" => Some(Unit::Month),
            "y" => Some(Unit::Year),
            _ => None,
        };
        if let Some(unit) = unit {
            return Some(shift(today, sign * n, unit));
        }
    }
    // in 3 days・3 days ago。
    let words: Vec<&str> = spaced.split(' ').collect();
    let (n, unit, sign) = match words.as_slice() {
        ["in", n, u] => (*n, *u, 1),
        [n, u, "ago"] => (*n, *u, -1),
        _ => return None,
    };
    let (Some(n), "") = split_number(n) else {
        return None;
    };
    en_unit(unit).map(|u| shift(today, sign * n, u))
}

/// 先頭の ASCII の数字の並びを数にし、残りを返す。数字が無ければ None。桁があふれる数は i64::MAX
/// (範囲の外の日になる)。
fn split_number(s: &str) -> (Option<i64>, &str) {
    let n = s.bytes().take_while(u8::is_ascii_digit).count();
    if n == 0 {
        return (None, s);
    }
    (Some(s[..n].parse::<i64>().unwrap_or(i64::MAX)), &s[n..])
}

/// 今日から n 単位ずらした日。月と年は同じ日(無ければその月の最後の日)。範囲を大きく出る数は範囲の外の日。
fn shift(today: i64, n: i64, unit: Unit) -> i64 {
    match unit {
        Unit::Day => today.saturating_add(n),
        Unit::Week => today.saturating_add(n.saturating_mul(7)),
        Unit::Month => add_months(today, n),
        Unit::Year => add_months(today, n.saturating_mul(12)),
    }
}

/// 月を n だけ足し引きした日(同じ日が無ければその月の最後の日)。年が 0〜9999 を出れば範囲の外の日
/// (`types::add_months` は範囲の端で止めるが、言葉は範囲の外として知らせたい)。
fn add_months(day: i64, n: i64) -> i64 {
    let (y, m, d) = types::ymd(day);
    let Some(total) = (y * 12 + i64::from(m) - 1).checked_add(n) else {
        return if n < 0 { i64::MIN } else { i64::MAX };
    };
    let (ny, nm) = (total.div_euclid(12), total.rem_euclid(12) as u32 + 1);
    if ny < 0 {
        return i64::MIN;
    }
    if ny > 9999 {
        return i64::MAX;
    }
    types::days_from_civil(ny, nm, d.min(types::days_in_month(ny, nm)))
}

/// 曜日の言葉の指す週。
#[derive(Clone, Copy)]
enum Which {
    /// 今日より後の最初のその曜日。
    Ahead,
    /// 今の週(週の始まりで区切る)。
    This,
    /// 次の週。
    Next,
}

/// 日本語の曜日の字(日曜から)。`金曜`・`金曜日` の形で読む(`金` だけは読まない)。
const JA_DAYS: [char; 7] = ['日', '月', '火', '水', '木', '金', '土'];

/// 英語の曜日の名前(日曜から。短い名前と長い名前)。
const EN_DAYS: [[&str; 2]; 7] = [
    ["sun", "sunday"],
    ["mon", "monday"],
    ["tue", "tuesday"],
    ["wed", "wednesday"],
    ["thu", "thursday"],
    ["fri", "friday"],
    ["sat", "saturday"],
];

/// 日本語の曜日(`金曜`・`金曜日`)の、日曜を 0 とする番号。
fn ja_day(s: &str) -> Option<usize> {
    let s = s
        .strip_suffix('日')
        .filter(|r| r.ends_with('曜'))
        .unwrap_or(s);
    let mut cs = s.chars();
    let (Some(c), Some('曜'), None) = (cs.next(), cs.next(), cs.next()) else {
        return None;
    };
    JA_DAYS.iter().position(|x| *x == c)
}

/// 英語の曜日(`fri`・`friday`)の、日曜を 0 とする番号。
fn en_day(s: &str) -> Option<usize> {
    EN_DAYS.iter().position(|names| names.contains(&s))
}

/// 曜日の言葉: `金曜`・`fri`、`今週の金曜`・`this fri`、`来週の金曜`・`next fri`。
fn weekday(compact: &str, spaced: &str, today: i64, week: WeekStart) -> Option<i64> {
    let (ja_which, ja_name) = [
        ("今週の", Which::This),
        ("今週", Which::This),
        ("来週の", Which::Next),
        ("来週", Which::Next),
    ]
    .iter()
    .find_map(|(head, w)| compact.strip_prefix(head).map(|rest| (*w, rest)))
    .unwrap_or((Which::Ahead, compact));
    let (which, target) = ja_day(ja_name).map(|d| (ja_which, d)).or_else(|| {
        let (which, name) = match spaced.split_once(' ') {
            Some(("this", rest)) => (Which::This, rest),
            Some(("next", rest)) => (Which::Next, rest),
            _ => (Which::Ahead, spaced),
        };
        en_day(name).map(|d| (which, d))
    })?;
    let target = target as i64;
    Some(match which {
        Which::Ahead => {
            let now = types::weekday_sun0(today) as i64;
            let ahead = (target - now).rem_euclid(7);
            today + if ahead == 0 { 7 } else { ahead }
        }
        Which::This => week_first(today, week) + (target - first_day(week)).rem_euclid(7),
        Which::Next => week_first(today, week) + 7 + (target - first_day(week)).rem_euclid(7),
    })
}

/// 週の始まりの曜日の番号(日曜を 0)。
fn first_day(week: WeekStart) -> i64 {
    match week {
        WeekStart::Sun => 0,
        WeekStart::Mon => 1,
    }
}

/// 今日を含む週の初めの日。
fn week_first(today: i64, week: WeekStart) -> i64 {
    let now = types::weekday_sun0(today) as i64;
    today - (now - first_day(week)).rem_euclid(7)
}

/// 週と月の区切りの言葉: `来週`・`next week`、`来月`・`next month`、`月末`・`eom`、`来月末`。
fn period(compact: &str, spaced: &str, today: i64, week: WeekStart) -> Option<i64> {
    let (y, m, _) = types::ymd(today);
    let first = types::days_from_civil(y, m, 1);
    let last_of = |first: i64| {
        let (y, m, _) = types::ymd(first);
        first + i64::from(types::days_in_month(y, m)) - 1
    };
    Some(match (compact, spaced) {
        ("来週", _) | (_, "next week") => week_first(today, week) + 7,
        ("来月", _) | (_, "next month") => add_months(first, 1),
        ("月末" | "今月末", _) | (_, "eom" | "end of month" | "end of this month") => {
            last_of(first)
        }
        ("来月末", _) | (_, "end of next month") => {
            let next = add_months(first, 1);
            if types::date_in_range(next) {
                last_of(next)
            } else {
                next
            }
        }
        _ => return None,
    })
}
