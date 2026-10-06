//! 列の型(タスク 5。CE-2・CE-5・CV-2・BV-12)。形は docs/design.md。
//!
//! 日付は 1970-01-01 からの日数(`i64`)。types.json の読み取りは依存を足さず、小さな JSON の読み取りで行う。

use crate::frontmatter::Value;
use crate::i18n::Msg;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Text,
    Number,
    Checkbox,
    Date,
    DateTime,
    List,
}

/// 複数の根の types.json を合わせた結果(BV-12)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Declared {
    One(Kind),
    Conflict,
}

/// `.obsidian/types.json`(`{"types": {"due": "date", ...}}`)を読む。text・number・checkbox・date・datetime・
/// multitext・tags・aliases を Kind に写す(multitext・tags・aliases は List)。知らない型の名前は Text。
/// 読めない・壊れているなら空(起動は止めない)。
pub fn read_types_json(bytes: &[u8]) -> HashMap<String, Kind> {
    let mut out = HashMap::new();
    let Ok(text) = std::str::from_utf8(bytes) else {
        return out;
    };
    let Some(Json::Object(top)) = json::parse(text) else {
        return out;
    };
    // 同じキーが2回あれば後の方(JSON.parse と同じ)。
    let Some((_, Json::Object(types))) = top.iter().rev().find(|(k, _)| k == "types") else {
        return out;
    };
    for (k, v) in types {
        // types の中の同じキーも後の方が残る(順に上書き)。
        if let Json::Str(name) = v {
            out.insert(k.clone(), kind_of_name(name));
        }
    }
    out
}

fn kind_of_name(name: &str) -> Kind {
    match name {
        "number" => Kind::Number,
        "checkbox" => Kind::Checkbox,
        "date" => Kind::Date,
        "datetime" => Kind::DateTime,
        "multitext" | "tags" | "aliases" => Kind::List,
        _ => Kind::Text,
    }
}

/// 空の値(推定と候補で飛ばす): Null・空の文字列・空のリスト。
pub(crate) fn is_empty(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::Str(s) => s.is_empty(),
        Value::List(l) => l.is_empty(),
        _ => false,
    }
}

/// Obsidian の推定(CE-2): 最初に見つかった空でない値(Null・空の文字列・空のリストは飛ばす)で決める。
/// `YYYY-MM-DD` の文字列 → Date、`YYYY-MM-DDTHH:MM`(秒は任意)→ DateTime、Int・Float → Number、
/// Bool → Checkbox、List → List、ほか → Text。何も無ければ Text。
/// 日付になった列に日時の形の値もあれば DateTime(日付だけの値も日時の列に合う。mixed-datetime)。
pub fn infer<'a>(mut values: impl Iterator<Item = &'a Value>) -> Kind {
    let Some(v) = values.find(|v| !is_empty(v)) else {
        return Kind::Text;
    };
    match v {
        Value::Int(_) | Value::Float(_) => Kind::Number,
        Value::Bool(_) => Kind::Checkbox,
        Value::List(_) => Kind::List,
        Value::Str(s) if date_shape(s) => {
            if values.any(|v| matches!(v, Value::Str(s) if datetime_shape(s))) {
                Kind::DateTime
            } else {
                Kind::Date
            }
        }
        Value::Str(s) if datetime_shape(s) => Kind::DateTime,
        _ => Kind::Text,
    }
}

/// 値が列の型に合うか(CV-2)。Null と空の文字列はどの型にも合う。Date は実在する日付だけ。
/// DateTime は日付だけの値も合う。List は1つの文字列も合う(Obsidian は1つの項目のリストとして見せる)。
pub fn fits(kind: Kind, v: &Value) -> bool {
    if matches!(v, Value::Null) || matches!(v, Value::Str(s) if s.is_empty()) {
        return true;
    }
    match kind {
        Kind::Text => true,
        Kind::Number => matches!(v, Value::Int(_) | Value::Float(_)),
        Kind::Checkbox => matches!(v, Value::Bool(_)),
        Kind::Date => matches!(v, Value::Str(s) if parse_date(s).is_some()),
        Kind::DateTime => matches!(v, Value::Str(s) if valid_datetime(s)),
        Kind::List => matches!(v, Value::List(_) | Value::Str(_)),
    }
}

/// 複数の根の types.json を合わせる。同じキーで型が食い違えば Conflict(BV-12)。
pub fn merge(per_root: &[HashMap<String, Kind>]) -> HashMap<String, Declared> {
    let mut out: HashMap<String, Declared> = HashMap::new();
    for m in per_root {
        for (k, &v) in m {
            out.entry(k.clone())
                .and_modify(|d| {
                    if *d != Declared::One(v) {
                        *d = Declared::Conflict;
                    }
                })
                .or_insert(Declared::One(v));
        }
    }
    out
}

/// ASCII の数字だけのバイト列を数にする(空・数字でないものがあれば None)。文字列を切らずにバイトで見る。
fn ascii_num(b: &[u8]) -> Option<u32> {
    if b.is_empty() || b.len() > 9 || !b.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some(b.iter().fold(0, |n, d| n * 10 + u32::from(d - b'0')))
}

/// `YYYY-MM-DD` の形なら (年, 月, 日)。実在するかは見ない。
fn date_parts(b: &[u8]) -> Option<(u32, u32, u32)> {
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    Some((
        ascii_num(&b[..4])?,
        ascii_num(&b[5..7])?,
        ascii_num(&b[8..])?,
    ))
}

fn date_shape(s: &str) -> bool {
    date_parts(s.as_bytes()).is_some()
}

/// (時, 分, 秒)。
type Hms = (u32, u32, u32);

/// `HH:MM` か `HH:MM:SS`(秒の小数は任意)なら (時, 分, 秒)。範囲は見ない。
fn time_parts(b: &[u8]) -> Option<Hms> {
    if b.len() < 5 || b[2] != b':' {
        return None;
    }
    let h = ascii_num(&b[..2])?;
    let m = ascii_num(&b[3..5])?;
    let rest = &b[5..];
    if rest.is_empty() {
        return Some((h, m, 0));
    }
    let sec = rest.strip_prefix(b":")?;
    let (whole, frac) = match sec.iter().position(|&c| c == b'.') {
        Some(i) => (&sec[..i], Some(&sec[i + 1..])),
        None => (sec, None),
    };
    if whole.len() != 2 {
        return None;
    }
    if frac.is_some_and(|f| f.is_empty() || !f.iter().all(u8::is_ascii_digit)) {
        return None;
    }
    Some((h, m, ascii_num(whole)?))
}

/// `YYYY-MM-DDTHH:MM`(秒は任意)の形なら (日付の部分, 時刻の部分)。
fn datetime_parts(s: &str) -> Option<(&[u8], Hms)> {
    datetime_full(s).map(|(d, t, _)| (d, t))
}

/// 時刻の後ろの時差(`Z`・`±HH:MM`・`±HHMM`・`±HH`)を外す: (時刻の部分, 時差の秒)。時差が無ければ None の時差。
/// 時差の形が崩れていれば None(読めない)。
fn split_offset(rest: &[u8]) -> Option<(&[u8], Option<i64>)> {
    if let Some(t) = rest.strip_suffix(b"Z") {
        return Some((t, Some(0)));
    }
    // 時刻は `HH:MM` で始まるので、符号はそれより後ろにしか無い。
    let Some(i) = rest
        .iter()
        .rposition(|&c| c == b'+' || c == b'-')
        .filter(|&i| i >= 5)
    else {
        return Some((rest, None));
    };
    let tz = &rest[i + 1..];
    let (h, m) = match tz.len() {
        2 => (ascii_num(tz)?, 0),
        4 => (ascii_num(&tz[..2])?, ascii_num(&tz[2..])?),
        5 if tz[2] == b':' => (ascii_num(&tz[..2])?, ascii_num(&tz[3..])?),
        _ => return None,
    };
    if h > 23 || m > 59 {
        return None;
    }
    let secs = i64::from(h * 3600 + m * 60);
    Some((&rest[..i], Some(if rest[i] == b'-' { -secs } else { secs })))
}

/// `YYYY-MM-DD` と `T` か空白と時刻(と時差)の形なら (日付の部分, 時刻, 時差の秒)。範囲は見ない。
fn datetime_full(s: &str) -> Option<(&[u8], Hms, Option<i64>)> {
    let b = s.as_bytes();
    if b.len() <= 11 || !(b[10] == b'T' || b[10] == b' ') {
        return None;
    }
    date_parts(&b[..10])?;
    let (time, off) = split_offset(&b[11..])?;
    Some((&b[..10], time_parts(time)?, off))
}

/// 日付の列・日時の列に書いてよい形か(WB-18): 日付か、`T` の区切りの日時(時差は付けてよい。読んだ値の
/// `Z`・`+09:00` をそのまま書き戻すため)。空白の区切りは書かない(読むときだけ受ける)。
pub fn writable_datetime(s: &str) -> bool {
    parse_date(s).is_some()
        || (s.as_bytes().get(10) == Some(&b'T') && parse_datetime_at(s, 0).is_some())
}

/// 日付(`YYYY-MM-DD`)か日時(`YYYY-MM-DD` と `T` か空白と `HH:MM[:SS[.fff]]`、後ろに `Z`・`±HH:MM`・
/// `±HHMM`・`±HH` の時差を付けてよい)を、地域の時計の秒(1970-01-01 0時からの秒。日付は0時)にする。
/// 時差の付いた値は地域の時刻に直す(時差の無い値は地域の時刻のまま。Obsidian と同じ)。読めなければ None。
pub fn parse_datetime(s: &str) -> Option<i64> {
    parse_datetime_at(s, crate::print::local_offset())
}

/// `parse_datetime` の地域の時差(秒)を渡す形(試験と、時差を決めて読むとき)。
pub fn parse_datetime_at(s: &str, local: i64) -> Option<i64> {
    if let Some(d) = parse_date(s) {
        return Some(d * 86_400);
    }
    let (date, (h, m, sec), off) = datetime_full(s)?;
    let day = parse_date(std::str::from_utf8(date).ok()?)?;
    if h > 23 || m > 59 || sec > 59 {
        return None;
    }
    let wall = day * 86_400 + i64::from(h * 3600 + m * 60 + sec);
    Some(match off {
        None => wall,
        Some(o) => wall - o + local,
    })
}

fn datetime_shape(s: &str) -> bool {
    datetime_parts(s).is_some()
}

/// 文字全体が実在する日付(`YYYY-MM-DD`)か日時(`YYYY-MM-DDTHH:MM(:SS)`)の形か(WB-18)。前後の空白や
/// ほかの文字があれば偽。囲まずに書く形なので、空白の区切りと時差は含めない(読むときの形より狭い)。
pub fn date_or_datetime(s: &str) -> bool {
    parse_date(s).is_some()
        || (s.as_bytes().get(10) == Some(&b'T')
            && datetime_full(s).is_some_and(|(_, _, off)| off.is_none())
            && valid_datetime(s))
}

fn valid_datetime(s: &str) -> bool {
    if parse_date(s).is_some() {
        return true;
    }
    let Some((date, (h, m, sec))) = datetime_parts(s) else {
        return false;
    };
    // date は ASCII の数字と `-` だけなので、文字列として読める。
    std::str::from_utf8(date)
        .ok()
        .and_then(parse_date)
        .is_some()
        && h < 24
        && m < 60
        && sec < 60
}

fn leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

/// `YYYY-MM-DD` を 1970-01-01 からの日数にする。実在しない日付は None(CE-5)。
pub fn parse_date(s: &str) -> Option<i64> {
    let (y, m, d) = date_parts(s.as_bytes())?;
    let y = i64::from(y);
    if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

/// 0000-01-01(`parse_date` で読める最初の日)の日数。
const FIRST_DAY: i64 = -719_528;
/// 9999-12-31(`parse_date` で読める最後の日)の日数。
const LAST_DAY: i64 = 2_932_896;
/// format_date が計算に使う日数の幅。外は丸める(計算をオーバーフローさせない)。
const CALC_LIMIT: i64 = i64::MAX / 4;

/// 日数が `parse_date` で読める範囲(0000-01-01〜9999-12-31)か。CE-5 の入力(今日からの日数など)はこれで確かめてから書く。
pub fn date_in_range(days: i64) -> bool {
    (FIRST_DAY..=LAST_DAY).contains(&days)
}

/// 1970-01-01 からの日数を `YYYY-MM-DD` にする。
///
/// 範囲(`date_in_range`)の外では、年が4桁に収まらない・負の年(`-0001-…`)の文字列を返し、
/// `parse_date` で読み戻せない。とても大きな日数は ±`i64::MAX / 4` に丸めて計算する(オーバーフローしない)。
pub fn format_date(days: i64) -> String {
    let (y, m, d) = civil_from_days(days.clamp(-CALC_LIMIT, CALC_LIMIT));
    format!("{y:04}-{m:02}-{d:02}")
}

/// 日数の (年, 月, 日)。`format_date` と同じく、とても大きな日数は丸めて計算する。
fn ymd(days: i64) -> (i64, u32, u32) {
    civil_from_days(days.clamp(-CALC_LIMIT, CALC_LIMIT))
}

/// 曜日。0 = 日 … 6 = 土(1970-01-01 は木曜)。
fn weekday_sun0(days: i64) -> usize {
    ((days.rem_euclid(7) + 4) % 7) as usize
}

/// 曜日の短い名前(`ddd`)。日曜から。
const WEEKDAY_NAMES: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];

/// 日付の形の部品(CE-22)。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Part {
    /// YYYY(4桁)
    Year4,
    /// YY(下2桁)
    Year2,
    /// MM(true: 0で埋める)・M
    Month(bool),
    /// DD(true: 0で埋める)・D
    Day(bool),
    /// ddd(曜日の短い名前)
    Weekday,
    /// 区切りの文字の並び
    Lit(String),
}

impl Part {
    fn is_number(&self) -> bool {
        matches!(
            self,
            Part::Year4 | Part::Year2 | Part::Month(_) | Part::Day(_)
        )
    }
}

/// 日付の形(CE-22)。表の見せ方と打ち込みに使う。ノートに書く形はいつも `YYYY-MM-DD`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateFormat {
    pattern: String,
    parts: Vec<Part>,
}

impl Default for DateFormat {
    fn default() -> Self {
        DateFormat::iso()
    }
}

impl DateFormat {
    /// 設定の形を読む。使える部品は YYYY・YY・MM・M・DD・D・ddd と、英数字でない区切りの文字(`-` `/` `.` 空白 `年` など)。
    /// 月と日は1つずつ要り、年と曜日は0か1つ。0で埋めない M・D は、ほかの数の部品と区切り無しで並べられない。
    /// 読めない形は Err(理由1行)。
    pub fn parse(pattern: &str) -> Result<DateFormat, String> {
        let quoted = format!("{pattern:?}");
        let bad = |why: String| Msg::DateFormatBad.fill(&[&quoted, &why]);
        let mut parts: Vec<Part> = Vec::new();
        let mut rest = pattern;
        while let Some(c) = rest.chars().next() {
            let (part, len) = if rest.starts_with("YYYY") {
                (Part::Year4, 4)
            } else if rest.starts_with("YY") {
                (Part::Year2, 2)
            } else if rest.starts_with("MM") {
                (Part::Month(true), 2)
            } else if rest.starts_with('M') {
                (Part::Month(false), 1)
            } else if rest.starts_with("DD") {
                (Part::Day(true), 2)
            } else if rest.starts_with('D') {
                (Part::Day(false), 1)
            } else if rest.starts_with("ddd") {
                (Part::Weekday, 3)
            } else if c.is_ascii_alphanumeric() || c.is_control() {
                return Err(bad(Msg::DateFormatBadChar.fill(&[&format!("{c:?}")])));
            } else {
                (Part::Lit(c.to_string()), c.len_utf8())
            };
            rest = &rest[len..];
            match (parts.last_mut(), part) {
                (Some(Part::Lit(prev)), Part::Lit(s)) => prev.push_str(&s),
                (_, part) => parts.push(part),
            }
        }
        let count = |f: fn(&Part) -> bool| parts.iter().filter(|p| f(p)).count();
        let years = count(|p| matches!(p, Part::Year4 | Part::Year2));
        let months = count(|p| matches!(p, Part::Month(_)));
        let days = count(|p| matches!(p, Part::Day(_)));
        let weekdays = count(|p| matches!(p, Part::Weekday));
        if months != 1 || days != 1 {
            return Err(bad(Msg::DateFormatMonthDay.text().to_string()));
        }
        if years > 1 || weekdays > 1 {
            return Err(bad(Msg::DateFormatYearWeekday.text().to_string()));
        }
        for w in parts.windows(2) {
            let variable = |p: &Part| matches!(p, Part::Month(false) | Part::Day(false));
            if w[0].is_number() && w[1].is_number() && (variable(&w[0]) || variable(&w[1])) {
                return Err(bad(Msg::DateFormatAdjacent.text().to_string()));
            }
        }
        Ok(DateFormat {
            pattern: pattern.to_string(),
            parts,
        })
    }

    /// `YYYY-MM-DD`(既定。ノートに書く形と同じ)。
    pub fn iso() -> DateFormat {
        DateFormat {
            pattern: "YYYY-MM-DD".to_string(),
            parts: vec![
                Part::Year4,
                Part::Lit("-".to_string()),
                Part::Month(true),
                Part::Lit("-".to_string()),
                Part::Day(true),
            ],
        }
    }

    /// 設定に書かれた形の文字列。
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// 日数(1970-01-01 から)をこの形の文字列にする(表の見せ方。CE-22)。
    pub fn format(&self, days: i64) -> String {
        let (y, m, d) = ymd(days);
        let mut out = String::new();
        for p in &self.parts {
            match p {
                Part::Year4 => out.push_str(&format!("{y:04}")),
                Part::Year2 => out.push_str(&format!("{:02}", y.rem_euclid(100))),
                Part::Month(true) => out.push_str(&format!("{m:02}")),
                Part::Month(false) => out.push_str(&m.to_string()),
                Part::Day(true) => out.push_str(&format!("{d:02}")),
                Part::Day(false) => out.push_str(&d.to_string()),
                Part::Weekday => out.push_str(WEEKDAY_NAMES[weekday_sun0(days)]),
                Part::Lit(s) => out.push_str(s),
            }
        }
        out
    }

    /// この形の打ち込みを読む。形に合わなければ None、合っても実在しない日付・曜日の食い違いなら Some(Err)。
    /// 年の無い形は today の年、YY は today の世紀。数の部品は 1〜2 桁(YYYY は4桁、YY は2桁)。
    fn read(&self, t: &str, today: i64) -> Option<Result<i64, String>> {
        let (today_y, _, _) = ymd(today);
        let (mut y, mut m, mut d, mut wd) = (today_y, 0u32, 0u32, None);
        let mut rest = t;
        for p in &self.parts {
            match p {
                Part::Lit(s) => rest = rest.strip_prefix(s.as_str())?,
                Part::Weekday => {
                    let (i, name) = WEEKDAY_NAMES
                        .iter()
                        .enumerate()
                        .find(|(_, n)| rest.starts_with(**n))?;
                    wd = Some(i);
                    rest = &rest[name.len()..];
                }
                Part::Year4 => y = i64::from(take_digits(&mut rest, 4, 4)?),
                Part::Year2 => {
                    y = today_y.div_euclid(100) * 100 + i64::from(take_digits(&mut rest, 2, 2)?)
                }
                Part::Month(_) => m = take_digits(&mut rest, 1, 2)?,
                Part::Day(_) => d = take_digits(&mut rest, 1, 2)?,
            }
        }
        if !rest.is_empty() {
            return None;
        }
        let day = (0..=9999)
            .contains(&y)
            .then(|| parse_date(&format!("{y:04}-{m:02}-{d:02}")))
            .flatten();
        Some(match (day, wd) {
            (None, _) => Err(Msg::DateNotReal.fill(&[&t])),
            // 曜日の名前は日付の形(ddd)の値なので訳さない。
            (Some(day), Some(w)) if w != weekday_sun0(day) => Err(Msg::DateWeekdayMismatch
                .fill(&[&t, &format_date(day), &WEEKDAY_NAMES[weekday_sun0(day)]])),
            (Some(day), _) => Ok(day),
        })
    }
}

/// 先頭の ASCII の数字を min〜max 桁(多いほう)読んで進める。
fn take_digits(rest: &mut &str, min: usize, max: usize) -> Option<u32> {
    let n = rest
        .bytes()
        .take(max)
        .take_while(u8::is_ascii_digit)
        .count();
    if n < min {
        return None;
    }
    let v = ascii_num(&rest.as_bytes()[..n])?;
    *rest = &rest[n..];
    Some(v)
}

/// 日付の列の打ち込みを読む(CE-5・CE-22): `YYYY-MM-DD`、設定の形(年が無い形なら今年)、`+N`・`-N`(today から)。
/// 前後の空白は除く。空は Ok(None)(null にする。CE-9)。実在しない日付・読めない打ち込み・範囲の外は Err(理由1行)。
pub fn parse_date_input(s: &str, fmt: &DateFormat, today: i64) -> Result<Option<i64>, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    if let Some(r) = relative_day(t, today) {
        return r.map(Some);
    }
    if let Some(d) = parse_date(t) {
        return Ok(Some(d));
    }
    if date_shape(t) {
        return Err(Msg::DateNotReal.fill(&[&t]));
    }
    if let Some(r) = fmt.read(t, today) {
        return r.map(Some);
    }
    let mut forms: Vec<&str> = Vec::new();
    if fmt.pattern != "YYYY-MM-DD" {
        forms.push(&fmt.pattern);
    }
    forms.extend(["YYYY-MM-DD", "+3", "-2", Msg::DateEmptyClears.text()]);
    Err(Msg::DateUnreadable.fill(&[&t, &forms.join(Msg::ListSep.text())]))
}

/// `+3`・`-2`(today からの日数)。その形でなければ None、範囲の外なら Some(Err)。
fn relative_day(t: &str, today: i64) -> Option<Result<i64, String>> {
    let digits = t.strip_prefix('+').or_else(|| t.strip_prefix('-'))?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n = digits
        .parse::<i64>()
        .ok()
        .map(|n| if t.starts_with('-') { -n } else { n });
    Some(
        n.and_then(|n| today.checked_add(n))
            .filter(|d| date_in_range(*d))
            .ok_or_else(|| Msg::DateOutOfRange.fill(&[&t])),
    )
}

/// カレンダーの週の始まり(CE-21)。既定は日曜。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeekStart {
    #[default]
    Sun,
    Mon,
}

/// カレンダーの1か月の格子(CE-21): その月の日を週ごとに並べ、前後の月の日は None。週の始まりは `start`。
/// 月が 1〜12 でなければ空。
pub fn month_grid(year: i32, month: u32, start: WeekStart) -> Vec<[Option<u32>; 7]> {
    let n = days_in_month(i64::from(year), month);
    if n == 0 {
        return Vec::new();
    }
    let first = days_from_civil(i64::from(year), month, 1);
    let shift = match start {
        WeekStart::Sun => 0,
        WeekStart::Mon => 1,
    };
    let mut col = (weekday_sun0(first) + 7 - shift) % 7;
    let mut grid = vec![[None; 7]];
    for day in 1..=n {
        if col == 7 {
            grid.push([None; 7]);
            col = 0;
        }
        if let Some(week) = grid.last_mut() {
            week[col] = Some(day);
        }
        col += 1;
    }
    grid
}

/// 月を n だけ足し引きした日(同じ日が無ければその月の末日。CE-21 の PageUp・PageDown)。
/// 範囲(`date_in_range`)の外の日はそのまま返し、行き先が範囲の外なら範囲の端で止める。
pub fn add_months(days: i64, n: i32) -> i64 {
    if !date_in_range(days) {
        return days;
    }
    let (y, m, d) = ymd(days);
    let total = y * 12 + i64::from(m) - 1 + i64::from(n);
    let (ny, nm) = (total.div_euclid(12), total.rem_euclid(12) as u32 + 1);
    if ny < 0 {
        return FIRST_DAY;
    }
    if ny > 9999 {
        return LAST_DAY;
    }
    days_from_civil(ny, nm, d.min(days_in_month(ny, nm)))
}

// 日付と日数の変換(Howard Hinnant の days_from_civil / civil_from_days)。
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// types.json を読むための最小の JSON の値。
enum Json {
    Str(String),
    Object(Vec<(String, Json)>),
    /// 型の名前に関わらない値(数・真偽・null・配列)。
    Other,
}

/// 依存を足さない小さな JSON の読み取り(RFC 8259 の文法を確かめる。値は types.json に要る分だけ持つ)。
mod json {
    use super::Json;

    /// 入れ子の深さの上限(壊れた入力で再帰が深くならないように)。
    const MAX_DEPTH: usize = 64;

    pub(super) fn parse(text: &str) -> Option<Json> {
        let mut p = Parser {
            s: text.as_bytes(),
            i: 0,
        };
        let v = p.value(0)?;
        p.ws();
        (p.i == p.s.len()).then_some(v)
    }

    struct Parser<'a> {
        s: &'a [u8],
        i: usize,
    }

    impl Parser<'_> {
        fn ws(&mut self) {
            while matches!(self.s.get(self.i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
                self.i += 1;
            }
        }

        fn eat(&mut self, c: u8) -> bool {
            self.ws();
            if self.s.get(self.i) == Some(&c) {
                self.i += 1;
                true
            } else {
                false
            }
        }

        fn lit(&mut self, word: &[u8]) -> Option<Json> {
            self.s[self.i..].starts_with(word).then(|| {
                self.i += word.len();
                Json::Other
            })
        }

        fn value(&mut self, depth: usize) -> Option<Json> {
            if depth > MAX_DEPTH {
                return None;
            }
            self.ws();
            match *self.s.get(self.i)? {
                b'{' => self.object(depth),
                b'[' => self.array(depth),
                b'"' => self.string().map(Json::Str),
                b't' => self.lit(b"true"),
                b'f' => self.lit(b"false"),
                b'n' => self.lit(b"null"),
                b'-' | b'0'..=b'9' => self.number(),
                _ => None,
            }
        }

        fn object(&mut self, depth: usize) -> Option<Json> {
            self.i += 1;
            let mut out = Vec::new();
            if self.eat(b'}') {
                return Some(Json::Object(out));
            }
            loop {
                self.ws();
                if self.s.get(self.i) != Some(&b'"') {
                    return None;
                }
                let k = self.string()?;
                if !self.eat(b':') {
                    return None;
                }
                let v = self.value(depth + 1)?;
                out.push((k, v));
                if self.eat(b',') {
                    continue;
                }
                return self.eat(b'}').then_some(Json::Object(out));
            }
        }

        fn array(&mut self, depth: usize) -> Option<Json> {
            self.i += 1;
            if self.eat(b']') {
                return Some(Json::Other);
            }
            loop {
                self.value(depth + 1)?;
                if self.eat(b',') {
                    continue;
                }
                return self.eat(b']').then_some(Json::Other);
            }
        }

        fn number(&mut self) -> Option<Json> {
            let start = self.i;
            if self.s.get(self.i) == Some(&b'-') {
                self.i += 1;
            }
            let int = self.run_digits();
            if int == 0 || (int > 1 && self.s[self.i - int] == b'0') {
                return None;
            }
            if self.s.get(self.i) == Some(&b'.') {
                self.i += 1;
                if self.run_digits() == 0 {
                    return None;
                }
            }
            if matches!(self.s.get(self.i), Some(b'e' | b'E')) {
                self.i += 1;
                if matches!(self.s.get(self.i), Some(b'+' | b'-')) {
                    self.i += 1;
                }
                if self.run_digits() == 0 {
                    return None;
                }
            }
            (self.i > start).then_some(Json::Other)
        }

        fn run_digits(&mut self) -> usize {
            let start = self.i;
            while matches!(self.s.get(self.i), Some(b'0'..=b'9')) {
                self.i += 1;
            }
            self.i - start
        }

        fn hex4(&mut self) -> Option<u32> {
            let h = self.s.get(self.i..self.i + 4)?;
            let h = std::str::from_utf8(h).ok()?;
            let v = u32::from_str_radix(h, 16).ok()?;
            if !h.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            self.i += 4;
            Some(v)
        }

        /// `"` から閉じの `"` まで。入力は UTF-8 の文字列から来るので、エスケープ以外はそのまま写す。
        fn string(&mut self) -> Option<String> {
            self.i += 1;
            let mut out: Vec<u8> = Vec::new();
            loop {
                let c = *self.s.get(self.i)?;
                self.i += 1;
                match c {
                    b'"' => return String::from_utf8(out).ok(),
                    b'\\' => {
                        let e = *self.s.get(self.i)?;
                        self.i += 1;
                        let ch = match e {
                            b'"' => '"',
                            b'\\' => '\\',
                            b'/' => '/',
                            b'b' => '\u{8}',
                            b'f' => '\u{c}',
                            b'n' => '\n',
                            b'r' => '\r',
                            b't' => '\t',
                            b'u' => {
                                let hi = self.hex4()?;
                                let cp = if (0xD800..0xDC00).contains(&hi) {
                                    if self.s.get(self.i..self.i + 2) != Some(b"\\u") {
                                        return None;
                                    }
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if !(0xDC00..0xE000).contains(&lo) {
                                        return None;
                                    }
                                    0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                                } else {
                                    hi
                                };
                                char::from_u32(cp)?
                            }
                            _ => return None,
                        };
                        let mut buf = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                    }
                    0x00..=0x1f => return None,
                    _ => out.push(c),
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "test_types_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_mixed_datetime_unit.rs"]
mod test_mixed_datetime_unit;

#[cfg(test)]
#[path = "test_datetime_offsets_unit.rs"]
mod test_datetime_offsets_unit;
