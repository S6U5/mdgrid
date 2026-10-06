//! 日付・日時の列のカレンダー(CE-20・CE-21・CE-22・SR-9・SR-15)。`impl App` の続きと描画の純関数。
//!
//! 日付の列のセルの編集で、入力ボックスの下(入らなければ上)に月の格子を重ねる。入力ボックスはそのまま打てて、
//! 打つたびに読めればカレンダーをその日へ動かす。カレンダーで日を動かすと、入力ボックスの文字もその日
//! (日付の列は設定の形か `YYYY-MM-DD`(`input_date`)、日時の列は `YYYY-MM-DD` と元の時刻)に変わる。
//! 確定は、カレンダーで選んだままなら選んでいる日の日数から、打ち込んだなら入力の文字を読んで(entry.rs)書く。
//! 日時の列で時刻を打っていなければ日だけを変え、時刻は行ごとの元の値を保つ(`Choice::Day`。一括でも)。
//! ←→ で1日・↑↓ で1週はカレンダーが出ているときだけ(出ていなければ文字のカーソルと候補のまま)。
//! PageUp・PageDown で1か月、今日に戻す・空にして確定はカレンダーが出ていなくても効く。
//! Shift+←→ で1か月・Shift+↑↓ で1年(CE-23・CE-24)はカレンダーが出ているときだけ(出ていなければ ←→・↑↓ と同じ)。
//! 画面に収まらなければ格子を出さず、打ち込みだけにする(SR-9)。詳細の表示からの編集でも出さない。
//! 印(SR-15): 今日は下線、元の値は `*`、選んでいる日は反転と `>`。

use super::app::App;
use super::entry::Entry;
use super::input::{input_box, Input};
use super::keymap::{self, Action, Mode};
use super::list::splice;
use super::view::visible_layout;
use super::width::width;
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::{self, DateFormat, Kind, WeekStart};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// カレンダーの状態(入力ボックスが持つ)。
/// 選んでいる日と元の日は、入力の文字ではなく元の値(ノートの `YYYY-MM-DD`)の日数から作る
/// (設定の形が年を落としても、違う日にならない)。
#[derive(Debug, Clone)]
pub(crate) struct Cal {
    /// 選んでいる日(1970-01-01 からの日数)。
    pub sel: i64,
    /// 編集前の値の日(`*`)。空か読めない値なら None。
    pub orig: Option<i64>,
    /// 日時の列の時刻の部分(日付の後ろの文字。`T09:00` など)。日付の列と、時刻の無い値では空。
    pub time: String,
    /// 元の値の時刻の部分(打ち込みに時刻が無いときに保つ)。
    orig_time: String,
    /// time は打ち込みで書いた時刻か(一括では、そのときだけ全部の行に入れる。CE-10)。
    time_typed: bool,
    /// 入力の文字はカレンダーで選んだ日(確定は sel の日数から書き、文字を読み直さない)。
    picked: bool,
    /// 最後にカレンダーへ読んだ入力の文字(打つたびに読み直す)。
    seen: String,
}

impl Cal {
    /// 日付・日時の入力ならカレンダーを作る。`raw` は元の値の文字(ノートの形)、`text` は入力ボックスの最初の文字。
    /// 元の値が空か読めなければ(CV-2)今日の月を出す(CE-21)。
    pub(crate) fn open(entry: Entry, raw: &str, text: &str, today: i64) -> Option<Cal> {
        if !matches!(entry, Entry::Date | Entry::DateTime) {
            return None;
        }
        let raw = raw.trim();
        let orig = match entry {
            // 型に合わない値(CV-2)は元の日にしない(今日の月を出す。CE-21)。
            Entry::DateTime => iso_split(raw)
                .filter(|_| valid_datetime(raw))
                .map(|(d, _)| d),
            _ => types::parse_date(raw),
        };
        let orig_time = match entry {
            Entry::DateTime => time_of(raw),
            _ => String::new(),
        };
        Some(Cal {
            sel: orig.unwrap_or(today),
            orig,
            time: orig_time.clone(),
            orig_time,
            time_typed: false,
            picked: false,
            seen: text.to_string(),
        })
    }
}

/// `YYYY-MM-DD` で始まる文字の日と、その後ろ(時刻の部分)。
fn iso_split(s: &str) -> Option<(i64, &str)> {
    if s.len() < 10 || !s.is_char_boundary(10) {
        return None;
    }
    types::parse_date(&s[..10]).map(|d| (d, &s[10..]))
}

/// 日時の値の時刻の部分(`2026-10-30T09:00` → `T09:00`)。無いか、時刻として正しくない
/// (日時の列の型に合わない。CV-2。`T25:00`・` 午後` など)なら空(持ち越して書かない)。
fn time_of(s: &str) -> String {
    let s = s.trim();
    iso_split(s)
        .filter(|_| valid_datetime(s))
        // 空白の区切りは読めるが、書くのは `T` の区切り(WB-18)。
        .map(|(_, t)| match t.strip_prefix(' ') {
            Some(rest) => format!("T{rest}"),
            None => t.to_string(),
        })
        .unwrap_or_default()
}

/// 日時の列の型に合う値か(CV-2)。
fn valid_datetime(s: &str) -> bool {
    types::fits(Kind::DateTime, &Value::Str(s.to_string()))
}

/// 入力の文字を日と時刻の部分に分ける(CE-5・CE-22 の形)。日付として読めなければ None。
fn split(entry: Entry, text: &str, today: i64, fmt: &DateFormat) -> Option<(i64, String)> {
    let t = text.trim();
    if entry == Entry::DateTime {
        // 時刻として正しくない後ろの部分は時刻にしない(読めない打ち込みとして entry.rs が理由を出す)。
        if let Some((d, time)) =
            iso_split(t).filter(|(_, time)| !time.is_empty() && valid_datetime(t))
        {
            return Some((d, time.to_string()));
        }
    }
    match types::parse_date_input(t, fmt, today) {
        Ok(Some(d)) => Some((d, String::new())),
        _ => None,
    }
}

/// 入力ボックスに日付を出す形(CE-22): 設定の形が4桁の年(YYYY)を含めばその形、含まなければ `YYYY-MM-DD`
/// (年の無い形・2桁の年は、読み直すと違う年になりうる)。表の見せ方は設定の形のまま。
pub(crate) fn input_date(fmt: &DateFormat, day: i64) -> String {
    if fmt.pattern().contains("YYYY") {
        fmt.format(day)
    } else {
        types::format_date(day)
    }
}

/// 選んだ日の入力ボックスの文字: 日付の列は input_date、日時の列は `YYYY-MM-DD` と時刻(CE-21)。
fn day_text(entry: Entry, fmt: &DateFormat, day: i64, time: &str) -> String {
    match entry {
        Entry::Date => input_date(fmt, day),
        _ => format!("{}{time}", types::format_date(day)),
    }
}

/// 日付の入力の確定の値(CE-20・CE-21・CE-10)。
pub(crate) enum Choice {
    /// どの行にも同じ値。
    Value(NewValue),
    /// 日時の列で日だけを決めた: 行ごとに、その行の元の時刻を保つ(元に時刻が無い行は時刻なし)。
    Day(i64),
}

impl Choice {
    /// 行に書く値。
    pub(crate) fn value(&self, app: &App, row: &RowId, col: &str) -> NewValue {
        match self {
            Choice::Value(v) => v.clone(),
            Choice::Day(d) => {
                let time = match app.prop(row, col) {
                    Some(Value::Str(s)) => time_of(&s),
                    _ => String::new(),
                };
                NewValue::Date(format!("{}{time}", types::format_date(*d)))
            }
        }
    }
}

/// 日付・日時の入力の確定の値。入力の文字の読み方(entry.rs)に任せるときは None。
/// カレンダーで選んだまま(文字を打っていない)なら選んでいる日の日数から書く(文字を読み直して年を補わない)。
/// 日時の列で時刻を打っていなければ、日だけを決めて時刻は行ごとの元の値を保つ。
pub(crate) fn choice(i: &Input, today: i64, fmt: &DateFormat) -> Option<Choice> {
    let c = i.cal.as_ref()?;
    if i.text.trim().is_empty() {
        return None;
    }
    let day = if c.picked && c.seen == i.text {
        if i.entry == Entry::Date {
            return Some(Choice::Value(NewValue::Date(types::format_date(c.sel))));
        }
        if c.time_typed {
            let v = format!("{}{}", types::format_date(c.sel), c.time);
            return Some(Choice::Value(NewValue::Date(v)));
        }
        c.sel
    } else {
        if i.entry != Entry::DateTime {
            return None;
        }
        match split(i.entry, &i.text, today, fmt) {
            Some((d, time)) if time.is_empty() => d,
            _ => return None,
        }
    };
    Some(Choice::Day(day))
}

/// 日数の (年, 月, 日)。
fn ymd(day: i64) -> (i32, u32, u32) {
    let s = types::format_date(day);
    let n = |r: std::ops::Range<usize>| s.get(r).and_then(|x| x.parse().ok()).unwrap_or(1);
    (n(0..4) as i32, n(5..7), n(8..10))
}

/// その月の1日の日数。
fn first_of_month(y: i32, m: u32) -> i64 {
    types::parse_date(&format!("{y:04}-{m:02}-01")).unwrap_or(0)
}

/// 窓の幅(縁 + 7日 × 4桁)。高さは `height`(下の縁は、月・年のキー(CE-23・CE-24)と今日・空のキーが
/// 1行に入らないので2行。幅は変えない。矢印を幅2と数えるときは3行)。
const CAL_W: usize = 30;
/// 1日の桁の数(`>` か空白・日の2桁・`*` か空白)。
const CELL_W: usize = 4;
/// 週の数(月によらず6週分の高さを取る。動かしても窓の大きさが変わらない)。
const WEEKS: usize = 6;

/// 窓の位置。
struct Geom {
    x: usize,
    top: usize,
    /// 窓の高さ(`height`)。
    h: usize,
}

/// 窓の位置: 入力ボックスの下、入らなければ上。どちらにも入らない・幅が足りないなら None(SR-9)。
/// `limit` は下の帯より上の行の数。
fn geometry(app: &App, w: usize, limit: usize) -> Option<Geom> {
    if app.mode != Mode::Edit || app.detail.is_some() {
        return None;
    }
    app.input.as_ref()?.cal.as_ref()?;
    if w < CAL_W {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let b = input_box(app, &lay, &cols)?;
    // 幅の数え方(CV-6 の ambiguous_wide)は visible_layout が App の設定から置いたあとで数える。
    let h = height(app);
    let top = if limit.saturating_sub(b.y + 1) >= h {
        b.y + 1
    } else if b.y >= h {
        b.y - h
    } else {
        return None;
    };
    Some(Geom {
        x: b.x.min(w - CAL_W),
        top,
        h,
    })
}

/// 今の端末の大きさでの窓の位置(描画と同じ: 右端の1桁と最下行を除き、下の帯とメッセージ行の上)。
fn geometry_now(app: &App) -> Option<Geom> {
    let w = app.size.0.saturating_sub(1) as usize;
    let h = app.size.1.saturating_sub(1) as usize;
    geometry(app, w, h.saturating_sub(2))
}

/// カレンダーが画面に出ているか(CE-21・SR-9)。
pub(crate) fn shown(app: &App) -> bool {
    geometry_now(app).is_some()
}

/// 曜日の見出しの並び(CE-21: 週の始まり)。どの言語も幅2(日本語は1字、英語は2字)。
fn weekday_names(start: WeekStart) -> [Msg; 7] {
    use Msg::{WdFri, WdMon, WdSat, WdSun, WdThu, WdTue, WdWed};
    match start {
        WeekStart::Sun => [WdSun, WdMon, WdTue, WdWed, WdThu, WdFri, WdSat],
        WeekStart::Mon => [WdMon, WdTue, WdWed, WdThu, WdFri, WdSat, WdSun],
    }
}

/// 月の名前(1〜12)。
const MONTHS: [Msg; 12] = [
    Msg::Month1,
    Msg::Month2,
    Msg::Month3,
    Msg::Month4,
    Msg::Month5,
    Msg::Month6,
    Msg::Month7,
    Msg::Month8,
    Msg::Month9,
    Msg::Month10,
    Msg::Month11,
    Msg::Month12,
];

/// 縁の行: `+--- 文字 ---+`(中央か、左寄せ)。収まらなければ `-` だけ。
fn edge(text: &str, center: bool) -> String {
    let inner = CAL_W - 2;
    let tw = width(text);
    if tw > inner {
        return format!("+{}+", "-".repeat(inner));
    }
    let left = if center {
        (inner - tw) / 2
    } else {
        1.min(inner - tw)
    };
    format!(
        "+{}{text}{}+",
        "-".repeat(left),
        "-".repeat(inner - tw - left)
    )
}

/// 窓の行(各行は幅 CAL_W の span の並び)。
fn rows(app: &App, c: &Cal) -> Vec<Vec<Span<'static>>> {
    let (y, m, _) = ymd(c.sel);
    let month = MONTHS
        .get((m as usize).wrapping_sub(1))
        .map_or_else(|| m.to_string(), |n| n.text().to_string());
    let mut out = vec![vec![Span::styled(
        edge(&Msg::CalTitle.fill(&[&y, &month]), true),
        Style::default().add_modifier(Modifier::BOLD),
    )]];
    let names: String = weekday_names(app.week_start)
        .iter()
        .map(|n| format!(" {} ", n.text()))
        .collect();
    out.push(vec![Span::raw(format!("|{names}|"))]);
    let first = first_of_month(y, m);
    let grid = types::month_grid(y, m, app.week_start);
    for k in 0..WEEKS {
        let mut row = vec![Span::raw("|")];
        for cell in grid.get(k).copied().unwrap_or([None; 7]) {
            let Some(dd) = cell else {
                row.push(Span::raw(" ".repeat(CELL_W)));
                continue;
            };
            let day = first + i64::from(dd) - 1;
            let sel = day == c.sel;
            let base = if sel {
                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default()
            };
            let digits = if day == app.today {
                base.add_modifier(Modifier::UNDERLINED)
            } else {
                base
            };
            row.push(Span::styled(if sel { ">" } else { " " }, base));
            row.push(Span::styled(format!("{dd:>2}"), digits));
            row.push(Span::styled(
                if c.orig == Some(day) { "*" } else { " " },
                base,
            ));
        }
        row.push(Span::raw("|"));
        out.push(row);
    }
    for e in edges(app) {
        out.push(vec![Span::raw(e)]);
    }
    out
}

/// 下の縁の行: 月・年のキー(CE-23・CE-24)の行と、今日に戻す・空にするのキー(CE-21)の行。どれもキーの表から(SR-4)。
/// 1行に入らない項目は次の行に送り(`ambiguous_wide` で矢印が幅2になっても消さない)、縁の行を増やす。
/// 1項目だけで縁に入らないもの(割り当て直しの長いキー)は省く。
fn edges(app: &App) -> Vec<String> {
    let key = |a| keymap::key_for(&app.keys, Mode::Edit, a);
    let pair = |a, b| match (key(a), key(b)) {
        (Some(x), Some(y)) => Some(join_keys(&x, &y)),
        (x, y) => x.or(y),
    };
    let jump = [
        pair(Action::PrevMonth, Action::NextMonth).map(|k| Msg::CalMonthKeys.fill(&[&k])),
        pair(Action::PrevYear, Action::NextYear).map(|k| Msg::CalYearKeys.fill(&[&k])),
    ];
    let reset = [
        key(Action::Today).map(|k| Msg::CalTodayKey.fill(&[&k])),
        key(Action::Clear).map(|k| Msg::CalClearKey.fill(&[&k])),
    ];
    let fits = |items: &[&str]| width(&format!(" {} ", items.join(" "))) <= CAL_W - 2;
    let mut lines: Vec<String> = Vec::new();
    for group in [jump, reset] {
        let mut items: Vec<&str> = Vec::new();
        for p in group.iter().flatten() {
            if !fits(&[p]) {
                continue;
            }
            items.push(p);
            if !fits(&items) {
                items.pop();
                lines.push(edge(&format!(" {} ", items.join(" ")), false));
                items = vec![p];
            }
        }
        let text = if items.is_empty() {
            String::new()
        } else {
            format!(" {} ", items.join(" "))
        };
        lines.push(edge(&text, false));
    }
    lines
}

/// 窓の高さ(見出しの縁・曜日・6週・下の縁の行)。
fn height(app: &App) -> usize {
    2 + WEEKS + edges(app).len()
}

/// 対の2つのキー: 同じ修飾の矢印ならまとめ(`Shift+←` と `Shift+→` → `Shift+←→`)、ほかは `/` で並べる。
fn join_keys(a: &str, b: &str) -> String {
    let arrow = |s: &str| {
        s.chars()
            .last()
            .filter(|c| "←→↑↓".contains(*c))
            .map(|c| (s[..s.len() - c.len_utf8()].to_string(), c))
    };
    match (arrow(a), arrow(b)) {
        (Some((pa, ca)), Some((pb, cb))) if pa == pb => format!("{pa}{ca}{cb}"),
        _ => format!("{a}/{b}"),
    }
}

/// 下の帯より上の行(`lines`)にカレンダーを重ねる(CE-20)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(g) = geometry(app, w, lines.len()) else {
        return;
    };
    let Some(c) = app.input.as_ref().and_then(|i| i.cal.as_ref()) else {
        return;
    };
    for (k, row) in rows(app, c).into_iter().enumerate() {
        let Some(line) = lines.get_mut(g.top + k) else {
            continue;
        };
        let mut x = g.x;
        for span in row {
            let sw = width(&span.content);
            *line = splice(line, x, span, sw, w);
            x += sw;
        }
    }
}

impl App {
    /// 月・年のキー(CE-23・CE-24)は、カレンダーが出ていなければ ←→(文字のカーソル)・↑↓(候補)に読み替える。
    pub(crate) fn calendar_fallback(&self, action: Action) -> Action {
        let plain = match action {
            Action::PrevMonth => Action::CursorLeft,
            Action::NextMonth => Action::CursorRight,
            Action::PrevYear => Action::ListUp,
            Action::NextYear => Action::ListDown,
            a => return a,
        };
        if shown(self) {
            action
        } else {
            plain
        }
    }

    /// 日付の入力のカレンダーの操作(CE-21)。扱ったら true(入力ボックスの動作に渡さない)。
    /// ←→↑↓ はカレンダーが出ているときだけ日を動かす。今日に戻す・空にするは日付・日時の入力だけ。
    pub(crate) fn calendar_action(&mut self, action: Action) -> bool {
        let Some(sel) = self
            .input
            .as_ref()
            .and_then(|i| i.cal.as_ref())
            .map(|c| c.sel)
        else {
            if matches!(action, Action::Today | Action::Clear) && self.input.is_some() {
                self.message = Some(Msg::CalDateOnly.text().into());
                return true;
            }
            return false;
        };
        let on = shown(self);
        let next = match action {
            Action::CursorLeft if on => sel - 1,
            Action::CursorRight if on => sel + 1,
            Action::ListUp if on => sel - 7,
            Action::ListDown if on => sel + 7,
            Action::PageUp => types::add_months(sel, -1),
            Action::PageDown => types::add_months(sel, 1),
            // 月・年のキーは、カレンダーが出ていなければ `calendar_fallback` が ←→・↑↓ に読み替えて済み。
            Action::PrevMonth => types::add_months(sel, -1),
            Action::NextMonth => types::add_months(sel, 1),
            Action::PrevYear => types::add_months(sel, -12),
            Action::NextYear => types::add_months(sel, 12),
            Action::Today => self.today,
            Action::Clear => {
                // CE-5・CE-9: 空にして確定(null のためる変更)。
                if let Some(i) = &mut self.input {
                    i.text.clear();
                    i.cursor = 0;
                    i.touched = true;
                    i.fresh = false;
                }
                self.input_action(Action::Commit);
                return true;
            }
            _ => return false,
        };
        self.calendar_set(next);
        true
    }

    /// カレンダーで選ぶ日を決め、入力ボックスの文字をその日にする。範囲の外の日には動かない。
    fn calendar_set(&mut self, day: i64) {
        if !types::date_in_range(day) {
            return;
        }
        let fmt = &self.date_format;
        let Some(i) = &mut self.input else {
            return;
        };
        let Some(c) = &mut i.cal else {
            return;
        };
        c.sel = day;
        c.picked = true;
        i.text = day_text(i.entry, fmt, day, &c.time);
        i.cursor = i.text.len();
        i.touched = true;
        i.fresh = false;
        c.seen = i.text.clone();
    }

    /// 打った文字が日付として読めれば、カレンダーをその日へ動かす(CE-20)。
    /// 日時は打った時刻を覚え、打ち込みに時刻が無ければ元の値の時刻を保つ(CE-21)。
    pub(crate) fn calendar_follow(&mut self) {
        let (today, fmt) = (self.today, &self.date_format);
        let Some(i) = &mut self.input else {
            return;
        };
        let Some(c) = &mut i.cal else {
            return;
        };
        if c.seen == i.text {
            return;
        }
        c.seen = i.text.clone();
        c.picked = false;
        if let Some((d, time)) = split(i.entry, &i.text, today, fmt) {
            c.sel = d;
            c.time_typed = !time.is_empty();
            c.time = if time.is_empty() {
                c.orig_time.clone()
            } else {
                time
            };
        }
    }

    /// カレンダーの窓のクリック(CE-20): 日ならその日を選ぶ。窓の中なら true(確定のクリックにしない)。
    pub(crate) fn calendar_click(&mut self, x: u16, y: u16) -> bool {
        let Some(g) = geometry_now(self) else {
            return false;
        };
        let (x, y) = (x as usize, y as usize);
        if x < g.x || x >= g.x + CAL_W || y < g.top || y >= g.top + g.h {
            return false;
        }
        let week = (y - g.top).checked_sub(2).filter(|k| *k < WEEKS);
        let col = (x - g.x)
            .checked_sub(1)
            .map(|c| c / CELL_W)
            .filter(|c| *c < 7);
        let sel = self
            .input
            .as_ref()
            .and_then(|i| i.cal.as_ref())
            .map(|c| c.sel);
        if let (Some(k), Some(j), Some(sel)) = (week, col, sel) {
            let (yy, mm, _) = ymd(sel);
            if let Some(dd) = types::month_grid(yy, mm, self.week_start)
                .get(k)
                .and_then(|w| w[j])
            {
                self.message = None;
                self.calendar_set(first_of_month(yy, mm) + i64::from(dd) - 1);
            }
        }
        true
    }
}
