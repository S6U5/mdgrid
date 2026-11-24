//! ルーチンの決まり(routine.rule)の語の表と読み方。ここは決まりの文を読んで、今日までの一番新しい回の日を
//! 決めるだけで、文言は持たない(打ち込みを読む語なので、画面の言語によらず英語と日本語の形をどちらも読む。
//! 日付の言葉の datewords と同じ)。
//!
//! 読む形(英語は大文字と小文字を区別しない。全角の英数字と空白は半角として読み、`,`・`、`・`・` は空白と同じ):
//! - 毎日: `daily`・`毎日`
//! - 毎週: `weekly mon thu`・`毎週 月 木`(曜日は `mon`〜`sun`・`monday`〜`sunday`・`月`〜`日`・`月曜`・`月曜日`)
//! - 毎月: `monthly 1 15`・`monthly last`・`毎月 1日 15日`・`毎月 末日`(31 日の無い月には 31 日の回が無い)
//! - N 日ごと: `every 3d`・`3日ごと`(起点の列 `start` の日から)
//!
//! 隔週(`every 2w`)・第2火曜(`monthly 2nd tue`)は読まない(tbd~routine.rule-syntax)。

use crate::types;

/// 決まり。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    /// 毎日。
    Daily,
    /// 毎週。曜日(0 = 日 … 6 = 土)の並び(小さい順、重なりなし)。
    Weekly(Vec<u32>),
    /// 毎月。日(1〜31)の並び(小さい順、重なりなし)と、月の末日か。
    Monthly { days: Vec<u32>, last: bool },
    /// 起点の日から N 日ごと(N は 1 以上)。
    Every(i64),
}

/// 英語の頭の語。
const EN_DAILY: &str = "daily";
const EN_WEEKLY: &str = "weekly";
const EN_MONTHLY: &str = "monthly";
const EN_EVERY: &str = "every";
const EN_LAST: &str = "last";
/// 英語の曜日(日曜から)。短い形と長い形。
const EN_WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
const EN_WEEKDAYS_LONG: [&str; 7] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
];

/// 日本語の頭の語と、日・末日・日ごと・曜日の語(日曜から)。
const JA_DAILY: &str = "毎日";
const JA_WEEKLY: &str = "毎週";
const JA_MONTHLY: &str = "毎月";
const JA_LAST: &str = "末日";
const JA_DAY: &str = "日";
const JA_EVERY: &str = "日ごと";
const JA_WEEKDAYS: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];
const JA_WEEKDAY_TAILS: [&str; 2] = ["曜日", "曜"];
const JA_SEPARATORS: [char; 2] = ['、', '・'];

impl Rule {
    /// 決まりの文を読む。読めなければ None(空の文も None。空は呼び手が「決まりが無い」として先に見る)。
    pub fn parse(text: &str) -> Option<Rule> {
        let text = normalize(text);
        let words: Vec<&str> = text.split_whitespace().collect();
        let (&head, rest) = words.split_first()?;
        match head {
            EN_DAILY | JA_DAILY if rest.is_empty() => return Some(Rule::Daily),
            EN_WEEKLY => return weekly(rest),
            EN_MONTHLY => return monthly(rest),
            EN_EVERY => return every_en(&rest.concat()),
            _ => {}
        }
        // 日本語の頭の語は、続きと空白なしで書いてもよい(`毎週月 木`)。
        let compact = words.concat();
        if let Some(r) = head.strip_prefix(JA_WEEKLY) {
            return weekly(&with_head_rest(r, rest));
        }
        if let Some(r) = head.strip_prefix(JA_MONTHLY) {
            return monthly(&with_head_rest(r, rest));
        }
        let n = compact.strip_suffix(JA_EVERY)?;
        every(n)
    }

    /// 起点の日が要る決まりか(`every`)。
    pub fn needs_start(&self) -> bool {
        matches!(self, Rule::Every(_))
    }

    /// 今日 `today` までの一番新しい回の日(1970-01-01 からの日数)。今日より後の回は返さない。まだ回が無い
    /// (起点が今日より後・起点の要る決まりで起点が無い)なら None。
    pub fn latest(&self, start: Option<i64>, today: i64) -> Option<i64> {
        match self {
            Rule::Daily => Some(today),
            Rule::Weekly(days) => (0..7)
                .map(|k| today - k)
                .find(|d| days.contains(&(types::weekday_sun0(*d) as u32))),
            // 31 日の回は、31 日の無い月を挟むと 2 か月前になる(最も離れるのは 62 日)。
            Rule::Monthly { days, last } => (0..=62).map(|k| today - k).find(|d| {
                let (y, m, dd) = types::ymd(*d);
                days.contains(&dd) || (*last && dd == types::days_in_month(y, m))
            }),
            Rule::Every(n) => {
                let s = start?;
                (s <= today).then(|| s + (today - s) / n * n)
            }
        }
    }
}

/// 頭の語のすぐ後ろに続けて書いた語(`毎週月` の `月`)を、残りの語の前に足す。
fn with_head_rest<'a>(glued: &'a str, rest: &[&'a str]) -> Vec<&'a str> {
    let mut v = Vec::new();
    if !glued.is_empty() {
        v.push(glued);
    }
    v.extend_from_slice(rest);
    v
}

/// 全角の英数字・記号と全角の空白を半角にし、英字を小文字にし、`,`・`、`・`・` を空白にする。
fn normalize(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(c) - 0xFEE0).unwrap_or(c),
            '\u{3000}' => ' ',
            c => c,
        })
        .map(|c| {
            if c == ',' || JA_SEPARATORS.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .flat_map(char::to_lowercase)
        .collect()
}

/// 毎週の曜日の並び。1つも無い・知らない語があれば None。
fn weekly(words: &[&str]) -> Option<Rule> {
    let mut days: Vec<u32> = Vec::new();
    for w in words {
        let ja = JA_WEEKDAY_TAILS
            .iter()
            .find_map(|t| w.strip_suffix(t))
            .unwrap_or(w);
        let k = EN_WEEKDAYS
            .iter()
            .position(|x| x == w)
            .or_else(|| EN_WEEKDAYS_LONG.iter().position(|x| x == w))
            .or_else(|| JA_WEEKDAYS.iter().position(|x| *x == ja))?;
        days.push(k as u32);
    }
    days.sort_unstable();
    days.dedup();
    (!days.is_empty()).then_some(Rule::Weekly(days))
}

/// 毎月の日(1〜31。日本語は `1日`)と末日。1つも無い・知らない語・範囲の外の数があれば None。
fn monthly(words: &[&str]) -> Option<Rule> {
    let (mut days, mut last) = (Vec::new(), false);
    for w in words {
        if *w == EN_LAST || *w == JA_LAST {
            last = true;
            continue;
        }
        let n: u32 = w.strip_suffix(JA_DAY).unwrap_or(w).parse().ok()?;
        if !(1..=31).contains(&n) {
            return None;
        }
        days.push(n);
    }
    days.sort_unstable();
    days.dedup();
    (!days.is_empty() || last).then_some(Rule::Monthly { days, last })
}

/// `every` の続き(空白を除いた `3d`)。
fn every_en(rest: &str) -> Option<Rule> {
    every(rest.strip_suffix('d')?)
}

/// N 日ごと(N は 1 以上の数)。
fn every(n: &str) -> Option<Rule> {
    let n: i64 = n.trim().parse().ok()?;
    (n >= 1).then_some(Rule::Every(n))
}
