//! ガントの時間軸を読みやすくする変更の受け入れの試験(記録: specs/_changes/2026-11-01-gantt-axis.md のタスク 1〜5)。
//! 月の単位は1か月3字で旬ごとに塗る・日の単位の見出しは月曜の日だけ・月か年ごとの地の帯(色なしでは付けない)・
//! 月の単位でも `t` で今日を左寄りに・ほかの画面の型(ガント・木・関係マップ)のメッセージ行に表の案内を出さない
//! (GA-2・GA-3・GA-4・SR-15・TR-1・REL-7)。
//!
//! 値は作ったもの。今日は 2026-10-06(火)に固定する。2026-10-01 は木曜。行番号や字の位置は決め打ちせず、
//! 見出しの語・ノートの名前・探す字で探し、字の位置は表示の幅で数える。色は値を決め打ちせず、違うか同じかだけを見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, text, Tmp};
use super::width::width;
use mdgrid::gantt::{civil, Unit};
use mdgrid::settings::{Dir, Group, Wbs, WbsValue};
use mdgrid::source::markdown::Markdown;
use mdgrid::types::{format_date, parse_date};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;
use std::path::PathBuf;

/// 今日(2026-10-06・火)。
const TODAY: &str = "2026-10-06";
const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 月の単位の例の行。原点は 2026-09-01 になる(開くと日か週、`-` で月)。
/// - long: 10/01〜11/30(10月と11月の6字)
/// - tiny: 10/03〜10/07(10月の上旬の1字)
/// - mid: 10/10〜10/11(上旬と中旬の2字)
/// - late: 10/21〜10/31(10月の下旬の1字)
/// - feb: 2027-02-21〜02-28(28日の月の下旬の1字)
/// - point: due だけの 11/15(11月の中旬に ◆)
/// - nodate: 日付なし(今日の ┊ だけ)
const MONTH_NOTES: &[(&str, &str)] = &[
    ("long.md", "---\nstart: 2026-10-01\ndue: 2026-11-30\n---\n"),
    ("tiny.md", "---\nstart: 2026-10-03\ndue: 2026-10-07\n---\n"),
    ("mid.md", "---\nstart: 2026-10-10\ndue: 2026-10-11\n---\n"),
    ("late.md", "---\nstart: 2026-10-21\ndue: 2026-10-31\n---\n"),
    ("feb.md", "---\nstart: 2027-02-21\ndue: 2027-02-28\n---\n"),
    ("point.md", "---\ndue: 2026-11-15\n---\n"),
    ("nodate.md", "---\nstatus: todo\n---\n"),
];

/// 親 alpha(日付なし)の下に beta(10/01〜10/31・done)と gamma(11/01〜11/30・todo)。
const TREE_MONTH: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: doing\n---\n"),
    (
        "beta.md",
        "---\nparent: \"[[alpha]]\"\nstart: 2026-10-01\ndue: 2026-10-31\nstatus: done\nn: 1\n---\n",
    ),
    (
        "gamma.md",
        "---\nparent: \"[[alpha]]\"\nstart: 2026-11-01\ndue: 2026-11-30\nstatus: todo\nn: 2\n---\n",
    ),
];

/// 日の単位の地の帯の例の行(範囲 10/01〜11/10 で日の単位、原点 10/01)。ant が先頭で選ばれる。
const BAND_NOTES: &[(&str, &str)] = &[
    ("ant.md", "---\nstart: 2026-10-01\ndue: 2026-10-02\n---\n"),
    ("beta.md", "---\nstart: 2026-11-09\ndue: 2026-11-10\n---\n"),
    ("gamma.md", "---\nstatus: todo\n---\n"),
];

/// 月の単位の地の帯の例の行。棒の字で 2026・2027・2028 年の月の位置を示す。aoct が先頭で選ばれる。
const YEAR_NOTES: &[(&str, &str)] = &[
    ("aoct.md", "---\nstart: 2026-10-21\ndue: 2026-10-31\n---\n"),
    ("y26.md", "---\nstart: 2026-11-01\ndue: 2026-11-30\n---\n"),
    ("y27.md", "---\nstart: 2027-03-01\ndue: 2027-03-31\n---\n"),
    ("y28.md", "---\nstart: 2028-03-01\ndue: 2028-03-31\n---\n"),
    ("z_none.md", "---\nstatus: todo\n---\n"),
];

/// 色の組: (色の種類, 設定の文, 端末の地)。テーマなしの組(24 ビット・256 色・端末の地が明るいと分かっているとき)は、
/// テーマが default のときに帯を塗らないこと(決定 gantt-bands-default-theme)を確かめる
/// `test_ga_2_band_none_with_default_theme` が使う。帯のある組は、帯の2本が本体の中で default 以外の全部のテーマと組む。
const PAINTS: &[(ColorMode, &str, Option<[u8; 3]>)] = &[
    (ColorMode::Rgb, "", None),
    (ColorMode::Indexed, "", None),
    (ColorMode::Rgb, "[look]\ntheme = \"nord\"\n", None),
    (ColorMode::Rgb, "", Some([250, 250, 250])),
];

// ---- 道具 ----

/// 印(根 = 一時フォルダ)の [[table]] に `line` を書き、設定の文 `config` を当てて、`notes` を表として開く
/// (幅 120・高さ 30、今日は 2026-10-06)。
fn boot_paint(
    tag: &str,
    notes: &[(&str, &str)],
    line: &str,
    color: ColorMode,
    config: &str,
) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), color);
    app.resize(120, 30);
    app.start(Startup {
        config: mdgrid::config::parse(config).unwrap().0,
        warnings: Vec::new(),
        readonly: false,
        no_color: matches!(color, ColorMode::None),
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = parse_date(TODAY).unwrap();
    (tmp, app)
}

/// 色なしで、ガントの印を書いて開く。
fn boot(tag: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    boot_paint(tag, notes, GANTT, ColorMode::None, "")
}

/// App の大きさで描いた画面の文字と、そのバッファ。
fn draw(a: &App) -> (String, Buffer) {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    let buf = t.backend().buffer().clone();
    (text(&buf), buf)
}

fn screen(a: &App) -> String {
    draw(a).0
}

/// 時間軸の見出しの2行の画面の行(窓の題「ガント:」の次の2行)。
fn header_ys(s: &str) -> (usize, usize) {
    let i = s
        .lines()
        .position(|l| l.contains("ガント:"))
        .unwrap_or_else(|| panic!("ガントの窓の題が無い:\n{s}"));
    (i + 1, i + 2)
}

/// 時間軸の見出しの2行(上・下)。
fn header(s: &str) -> (String, String) {
    let (t, l) = header_ys(s);
    (line(s, t), line(s, l))
}

fn line(s: &str, y: usize) -> String {
    s.lines().nth(y).unwrap_or("").to_string()
}

/// ノート `name` の行の画面の行(最初の1行)。
fn row_y(s: &str, name: &str) -> usize {
    s.lines()
        .position(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{s}"))
}

fn row_line(s: &str, name: &str) -> String {
    line(s, row_y(s, name))
}

fn count(line: &str, c: char) -> usize {
    line.chars().filter(|x| *x == c).count()
}

/// `pat` の最初の位置(表示の幅で)。
fn col(line: &str, pat: &str) -> Option<usize> {
    line.find(pat).map(|i| width(&line[..i]))
}

/// 字 `c` のある位置の全部(表示の幅で)。
fn cols_of(line: &str, c: char) -> Vec<usize> {
    let mut out = Vec::new();
    let mut x = 0;
    for ch in line.chars() {
        if ch == c {
            out.push(x);
        }
        x += width(&ch.to_string());
    }
    out
}

/// 位置 `x`(表示の幅)から始まる字。
fn char_at(line: &str, x: usize) -> Option<char> {
    let mut at = 0;
    for c in line.chars() {
        if at == x {
            return Some(c);
        }
        at += width(&c.to_string());
    }
    None
}

/// 位置 `x` から右の字(窓の右の縁と行末の空白を除く)。
fn chars_from(line: &str, x: usize) -> Vec<char> {
    let mut out = Vec::new();
    let mut at = 0;
    for c in line.chars() {
        if at >= x {
            out.push(c);
        }
        at += width(&c.to_string());
    }
    let edge = |c: &char| c.is_whitespace() || matches!(c, '│' | '|' | '┃' | '║');
    while out.last().is_some_and(edge) {
        out.pop();
    }
    out
}

/// `pat` の最初の位置(表示の幅で)。無ければ行を見せて落ちる。
fn must_col(line: &str, pat: &str) -> usize {
    match col(line, pat) {
        Some(x) => x,
        None => panic!("「{pat}」が無い:\n{line}"),
    }
}

/// ノート `name` の行の、棒の最初の字 `█` の位置(表示の幅で)。
fn bar_x(s: &str, name: &str) -> usize {
    match cols_of(&row_line(s, name), '█').first() {
        Some(&x) => x,
        None => panic!("{name} の行に棒が無い:\n{s}"),
    }
}

/// バッファのセル (x, y) の地の色。
fn bg(buf: &Buffer, x: usize, y: usize) -> Color {
    buf[(u16::try_from(x).unwrap(), u16::try_from(y).unwrap())].bg
}

/// 日 `d` が月曜か(1970-01-01 は木曜)。
fn is_monday(d: i64) -> bool {
    (d + 3).rem_euclid(7) == 0
}

fn unit(a: &App) -> Unit {
    a.gantt.as_ref().expect("ガント").unit
}

/// `C` でガントを開き、`-` を2回で月の単位にする(日 → 週 → 月。月はそれより粗くならない)。
fn open_month(a: &mut App) {
    ch(a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(a, '-');
    ch(a, '-');
    assert_eq!(unit(a), Unit::Month);
}

fn tree_on(a: &mut App, wbs: Option<Wbs>) {
    a.settings.tree = Some("parent".into());
    a.settings.wbs = wbs;
    a.settings.sorts = vec![("n".into(), Dir::Asc)];
    a.regrid = true;
    a.refresh_if_needed();
}

fn wbs() -> Wbs {
    let v = |value: &str, percent: u8, label: &str| WbsValue {
        value: value.into(),
        percent,
        label: label.into(),
    };
    Wbs {
        key: "status".into(),
        map: vec![
            v("done", 100, "完了"),
            v("doing", 50, "作業中"),
            v("todo", 0, "未着手"),
        ],
    }
}

/// 表を status でまとめ、先頭(1つ目のまとまりの見出しの行)を選ぶ。
fn group_by_status(a: &mut App) {
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    a.row = 0;
    a.top = 0;
    assert!(a.cur_head().is_some(), "見出しの行を選んでいる");
}

/// 表の見出しの行の案内(「見出しの行(n件)。Enter で開閉」)の語。
const HEAD_HINT: &str = "見出しの行";

// ---- タスク 1: 月の単位は1か月3字、旬で塗る ----

#[test]
fn test_ga_2_month_header_numbers_and_years() {
    // [GA-2] 月の単位(原点 2026-09)の見出し: 下の行は1か月3字の月の数「 9 10 11 12  1」、上の行の年は1月のマスの上に
    // 出て、年どうしが隣り合わない(「20262027」のようにくっつかない)。次の年 2028 は 2027 の 36 字(12か月)右。
    let (_t, mut a) = boot("ga2monhdr", MONTH_NOTES);
    open_month(&mut a);
    assert_eq!(
        a.gantt.as_ref().unwrap().origin,
        parse_date("2026-09-01").unwrap()
    );
    let (top, low) = header(&screen(&a));
    assert!(
        low.contains(" 9 10 11 12  1"),
        "下の行に1か月3字の月の数:\n{low}"
    );
    for tok in top
        .split_whitespace()
        .filter(|t| t.chars().any(|c| c.is_ascii_digit()))
    {
        assert!(
            tok.chars().count() == 4 && tok.chars().all(|c| c.is_ascii_digit()),
            "上の行の年どうしが隣り合う「{tok}」:\n{top}"
        );
    }
    // 1月のマスの始まり = 12月の「12」の2字あとの空き(12月のマスの3字目)の次。
    let jan = col(&low, "12  1").expect("12月と1月") + 3;
    let y27 = must_col(&top, "2027");
    assert!(
        (jan..jan + 3).contains(&y27),
        "2027 は1月のマスの上({jan}〜{}): {y27}\n{top}\n{low}",
        jan + 2
    );
    assert_eq!(top.matches("2027").count(), 1, "{top}");
    let y28 = must_col(&top, "2028");
    assert_eq!(y28, y27 + 36, "1年は36字:\n{top}");
}

#[test]
fn test_ga_2_month_bars_by_thirds() {
    // [GA-2][GA-3] 月の単位の棒は旬(1〜10日・11〜20日・21日〜末日)の字で塗る: 10/01〜11/30 は `█` が6字、
    // 10/03〜10/07 は1字、10/10〜10/11 は2字、10/21〜10/31 は1字(10月の3字目)、2027-02-21〜02-28 は1字(2月の3字目)。
    // 字は見出しの月の数に揃う(10月のマスの先頭 = 「10」の字)。due だけの 11/15 は ◆ が11月の2字目に1つ、
    // 日付の無い行の今日(10/06)の ┊ は10月の1字目に1つ。
    let (_t, mut a) = boot("ga2monbar", MONTH_NOTES);
    open_month(&mut a);
    let s = screen(&a);
    let (_, low) = header(&s);
    let long = row_line(&s, "long");
    let tiny = row_line(&s, "tiny");
    let mid = row_line(&s, "mid");
    let late = row_line(&s, "late");
    let feb = row_line(&s, "feb");
    assert_eq!(count(&long, '█'), 6, "10/01〜11/30 は6字:\n{s}");
    assert_eq!(count(&tiny, '█'), 1, "10/03〜10/07 は1字:\n{s}");
    assert_eq!(count(&mid, '█'), 2, "10/10〜10/11 は2字:\n{s}");
    assert_eq!(count(&late, '█'), 1, "10/21〜10/31 は1字:\n{s}");
    assert_eq!(count(&feb, '█'), 1, "02/21〜02/28 は1字:\n{s}");
    let oct = must_col(&low, "10");
    let six: Vec<usize> = (oct..oct + 6).collect();
    assert_eq!(cols_of(&long, '█'), six, "{s}");
    assert_eq!(cols_of(&tiny, '█'), vec![oct], "{s}");
    assert_eq!(cols_of(&mid, '█'), vec![oct, oct + 1], "{s}");
    assert_eq!(cols_of(&late, '█'), vec![oct + 2], "{s}");
    // 2027年2月は10月の4か月あと(12字右)の、3字目。
    assert_eq!(cols_of(&feb, '█'), vec![oct + 12 + 2], "{s}");
    assert_eq!(cols_of(&row_line(&s, "point"), '◆'), vec![oct + 4], "{s}");
    assert_eq!(cols_of(&row_line(&s, "nodate"), '┊'), vec![oct], "{s}");
}

#[test]
fn test_ga_2_month_wbs_fill_counts_chars() {
    // [GA-2][GA-3] 月の単位で、親(日付なし)の棒は子の範囲 10/01〜11/30 の6字。子の割合が 100 と 0 → 塗り(`█`)は
    // 棒の字の半分の3字、残り(`░`)が3字。子の beta(10月)と gamma(11月)の棒は、どちらも1か月の3字
    // (子の行の塗りは GA-3 が決めていないので見ない。字の数だけ)。
    let (_t, mut a) = boot("ga2monwbs", TREE_MONTH);
    tree_on(&mut a, Some(wbs()));
    open_month(&mut a);
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    assert_eq!(
        (count(&alpha, '█'), count(&alpha, '░')),
        (3, 3),
        "親の棒は6字で半分を塗る:\n{s}"
    );
    for n in ["beta", "gamma"] {
        let l = row_line(&s, n);
        assert_eq!(count(&l, '█') + count(&l, '░'), 3, "{n} の棒は3字:\n{s}");
    }
}

#[test]
fn test_ga_2_month_today_keeps_digits_reversed() {
    // [GA-2][SR-15] 月の単位で、今日(10/06)の字は10月のマスの1字目で月の数「10」の「1」に重なる → 数は消えず
    // (見出しに ▾ は出ない)、その字を反転で示す(色なし)。11月の数は反転しない。
    let (_t, mut a) = boot("ga2montoday", MONTH_NOTES);
    open_month(&mut a);
    let (s, buf) = draw(&a);
    let (_, ly) = header_ys(&s);
    let low = line(&s, ly);
    assert!(low.contains(" 9 10 11"), "月の数が残る:\n{low}");
    assert!(!low.contains('▾'), "今日は ▾ でなく反転:\n{low}");
    let oct = must_col(&low, "10");
    let cell = &buf[(u16::try_from(oct).unwrap(), u16::try_from(ly).unwrap())];
    assert_eq!(cell.symbol(), "1", "{low}");
    assert!(cell.modifier.contains(Modifier::REVERSED), "{cell:?}");
    let nov = must_col(&low, "11");
    let cell = &buf[(u16::try_from(nov).unwrap(), u16::try_from(ly).unwrap())];
    assert!(!cell.modifier.contains(Modifier::REVERSED), "{cell:?}");
}

// ---- タスク 2: 日の単位の見出しは月曜の日だけ ----

#[test]
fn test_ga_2_day_header_mondays_only() {
    // [GA-2] 日の単位(原点 10/01・木)の見出しの下の行は、月曜の字にその日の数だけ: 10/05 に「5」、10/12 に「12」、
    // 10/19 に「19」、10/26 に「26」(11月より先の月曜も同じ形)。ほかの日の数は無く、今日 10/06(火)の字に ▾。
    let note = "---\nstart: 2026-10-01\ndue: 2026-10-31\n---\n";
    let (_t, mut a) = boot("ga2dayhdr", &[("alpha.md", note)]);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(unit(&a), Unit::Day);
    let s = screen(&a);
    let (_, low) = header(&s);
    // alpha の棒の最初の字が 10/01。
    let x0 = bar_x(&s, "alpha");
    let first = parse_date("2026-10-01").unwrap();
    for (day, want) in [(5, "5"), (12, "12"), (19, "19"), (26, "26")] {
        let x = x0 + day - 1;
        let got: String = chars_from(&low, x).into_iter().take(want.len()).collect();
        assert_eq!(got, want, "10/{day}(月)の字に「{want}」:\n{low}");
    }
    assert_eq!(
        char_at(&low, x0 + 5),
        Some('▾'),
        "今日 10/06 の字に ▾:\n{low}"
    );
    // 数の並びは、どれも月曜の字から始まるその日の数(右の端で切れたものは先頭の字だけでよい)。
    let axis = chars_from(&low, x0);
    let mut j = 0;
    while j < axis.len() {
        if !axis[j].is_ascii_digit() {
            j += 1;
            continue;
        }
        let run: String = axis[j..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let d = first + i64::try_from(j).unwrap();
        assert!(
            is_monday(d),
            "月曜でない日 {} の字に数「{run}」:\n{low}",
            format_date(d)
        );
        let want = civil(d).2.to_string();
        let at_end = j + run.len() == axis.len();
        assert!(
            run == want || (at_end && want.starts_with(&run)),
            "{} の字は「{want}」: 「{run}」\n{low}",
            format_date(d)
        );
        j += run.len();
    }
}

// ---- タスク 3: 地の帯 ----

#[test]
fn test_ga_2_band_day_months_and_weekends() {
    // [GA-2] 色ありの日の単位で、時間軸の字の地は月ごとに交互に変わり、土日は別の地: 見出しの2行と選んでいない行
    // (beta・gamma)の 10/07(水)と 11/04(水)の地が違い、10/07 と 10/08(木)・12/02(水)は同じ。10/03(土)と 10/02(金)、
    // 10/04(日)と 10/05(月)、11/07(土)と 11/06(金)の地が違う。選んでいる行(ant)は選びの見た目が先で、10月と11月で同じ地。
    // テーマのある組で: default 以外の全部のテーマ(暗い地も明るい地も)を、24 ビットと 256 色のどちらでも
    // (決定 gantt-bands-default-theme。default では帯を塗らない: test_ga_2_band_none_with_default_theme)。
    fn painted() -> Vec<(mdgrid::theme::Theme, ColorMode)> {
        let mut out = Vec::new();
        for t in mdgrid::theme::Theme::ALL {
            if t == mdgrid::theme::Theme::Default {
                continue;
            }
            for c in [ColorMode::Rgb, ColorMode::Indexed] {
                out.push((t, c));
            }
        }
        out
    }
    for (k, (theme, color)) in painted().into_iter().enumerate() {
        let tag = format!("ga2bandday{k}");
        let config = format!("[look]\ntheme = \"{}\"\n", theme.name());
        let (_t, mut a) = boot_paint(&tag, BAND_NOTES, GANTT, color, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        assert_eq!(unit(&a), Unit::Day);
        let (s, buf) = draw(&a);
        let what = format!("{} {color:?}", theme.name());
        let x0 = bar_x(&s, "ant");
        let first = parse_date("2026-10-01").unwrap();
        let at = |d: &str| x0 + usize::try_from(parse_date(d).unwrap() - first).unwrap();
        let (ty, ly) = header_ys(&s);
        for (name, y) in [
            ("見出しの上の行", ty),
            ("見出しの下の行", ly),
            ("beta の行", row_y(&s, "beta")),
            ("gamma の行", row_y(&s, "gamma")),
        ] {
            let b = |d: &str| bg(&buf, at(d), y);
            assert_ne!(
                b("2026-10-07"),
                b("2026-11-04"),
                "{what} {name}: 10月と11月の地が同じ\n{s}"
            );
            assert_eq!(
                b("2026-10-07"),
                b("2026-10-08"),
                "{what} {name}: 10月の平日どうし"
            );
            assert_eq!(
                b("2026-10-07"),
                b("2026-12-02"),
                "{what} {name}: 1つおきの月(10月と12月)は同じ地"
            );
            assert_ne!(
                b("2026-10-03"),
                b("2026-10-02"),
                "{what} {name}: 土と金の地が同じ"
            );
            assert_ne!(
                b("2026-10-04"),
                b("2026-10-05"),
                "{what} {name}: 日と月の地が同じ"
            );
            assert_ne!(
                b("2026-11-07"),
                b("2026-11-06"),
                "{what} {name}: 11月の土と金の地が同じ"
            );
        }
        let ay = row_y(&s, "ant");
        assert_eq!(
            bg(&buf, at("2026-10-07"), ay),
            bg(&buf, at("2026-11-04"), ay),
            "{what}: 選んでいる行は行の全体が選びの地"
        );
    }
}

#[test]
fn test_ga_2_band_month_by_year() {
    // [GA-2] 色ありの月の単位で、時間軸の字の地は年ごとに交互に変わる: 見出しの2行と日付の無い行(z_none)で、
    // 2026-11 と 2027-03 の字の地が違い、2026-11 と 2028-03 は同じ。同じ年の 2026-10 と 2026-11 は同じ地(月ごとには変えない)。
    // 月の位置は棒の字(aoct: 2026-10 の下旬、y26・y27・y28: 各月の最初の字)で測る。
    // テーマのある組で: default 以外の全部のテーマ(暗い地も明るい地も)を、24 ビットと 256 色のどちらでも
    // (決定 gantt-bands-default-theme。default では帯を塗らない: test_ga_2_band_none_with_default_theme)。
    fn painted() -> Vec<(mdgrid::theme::Theme, ColorMode)> {
        let mut out = Vec::new();
        for t in mdgrid::theme::Theme::ALL {
            if t == mdgrid::theme::Theme::Default {
                continue;
            }
            for c in [ColorMode::Rgb, ColorMode::Indexed] {
                out.push((t, c));
            }
        }
        out
    }
    for (k, (theme, color)) in painted().into_iter().enumerate() {
        let tag = format!("ga2bandmon{k}");
        let config = format!("[look]\ntheme = \"{}\"\n", theme.name());
        let (_t, mut a) = boot_paint(&tag, YEAR_NOTES, GANTT, color, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        ch(&mut a, '-');
        ch(&mut a, '-');
        assert_eq!(unit(&a), Unit::Month);
        let (s, buf) = draw(&a);
        let what = format!("{} {color:?}", theme.name());
        let x_oct = bar_x(&s, "aoct");
        let x26 = bar_x(&s, "y26");
        let x27 = bar_x(&s, "y27");
        let x28 = bar_x(&s, "y28");
        let (ty, ly) = header_ys(&s);
        for (name, y) in [
            ("見出しの上の行", ty),
            ("見出しの下の行", ly),
            ("z_none の行", row_y(&s, "z_none")),
        ] {
            let b = |x: usize| bg(&buf, x, y);
            assert_ne!(
                b(x26),
                b(x27),
                "{what} {name}: 2026年と2027年の地が同じ\n{s}"
            );
            assert_eq!(
                b(x26),
                b(x28),
                "{what} {name}: 1年おき(2026年と2028年)は同じ地"
            );
            assert_eq!(
                b(x_oct),
                b(x26),
                "{what} {name}: 同じ年の10月と11月は同じ地"
            );
        }
    }
}

#[test]
fn test_ga_2_band_none_without_color() {
    // [GA-2][SR-15] 色なしでは時間軸の字に地の色を付けない(見出しの2行と選んでいない行の時間軸の字の地は既定のまま)。
    // 字は色あり(24 ビット)と同じ: 日の単位と月の単位の、見出しの2行と全部の行。
    let (_t, mut a) = boot_paint("ga2bandnone", BAND_NOTES, GANTT, ColorMode::None, "");
    let (_t2, mut b) = boot_paint("ga2bandrgb", BAND_NOTES, GANTT, ColorMode::Rgb, "");
    ch(&mut a, 'C');
    ch(&mut b, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(b.mode, Mode::Gantt, "{:?}", b.message);
    // 時間軸の左端(日の単位の原点 10/01 = ant の棒の最初の字)。名前の欄の幅は単位で変わらない。
    let x0 = bar_x(&screen(&a), "ant");
    for round in ["日", "週", "月"] {
        let (s, buf) = draw(&a);
        let s2 = screen(&b);
        let (ty, ly) = header_ys(&s);
        for (name, y) in [
            ("見出しの上の行", ty),
            ("見出しの下の行", ly),
            ("beta の行", row_y(&s, "beta")),
            ("gamma の行", row_y(&s, "gamma")),
        ] {
            let end = usize::from(a.size.0);
            for x in x0..end {
                assert_eq!(
                    bg(&buf, x, y),
                    Color::Reset,
                    "{round} {name}: 色なしで桁 {x} に地の色\n{s}"
                );
            }
        }
        assert_eq!(header(&s), header(&s2), "{round}: 見出しの字が違う");
        for n in ["ant", "beta", "gamma"] {
            assert_eq!(
                row_line(&s, n),
                row_line(&s2, n),
                "{round}: {n} の行の字が色ありと違う"
            );
        }
        ch(&mut a, '-');
        ch(&mut b, '-');
    }
}

#[test]
fn test_ga_2_band_none_with_default_theme() {
    // [GA-2][SR-40] テーマが default(設定なしと theme = "default")のときは、`PAINTS` のテーマなしの組(24 ビット・256 色・
    // 端末の地が明るいと分かっているとき)のどれでも、時間軸の字に地の色を付けない: 日・週・月の単位で、見出しの2行と
    // 選んでいない行(beta・gamma)の時間軸の字(11月の字も土日の字も)の地が Reset。決定 gantt-bands-default-theme。
    // ただし見出しの下の行で今日の字が月の数に重なる1字は、今日の印(数を残して反転。モダンな見た目では選びの地。
    // SR-33・SR-40)で帯ではないので除く。数に重ならない今日の字(▾)は除かない。

    /// 行の、窓の右の縁の字(`│` か ASCII の `|`)の位置(表示の幅で)。無ければ行の幅。
    fn right_edge(l: &str) -> usize {
        match l.rfind(['│', '|']) {
            Some(i) => width(&l[..i]),
            None => width(l),
        }
    }
    let mut combos = Vec::new();
    for &(color, _, term_bg) in PAINTS.iter().filter(|p| p.1.is_empty()) {
        for config in ["", "[look]\ntheme = \"default\"\n"] {
            combos.push((color, config, term_bg));
        }
    }
    assert!(combos.len() >= 6, "テーマなしの組: {combos:?}");
    for (k, (color, config, term_bg)) in combos.into_iter().enumerate() {
        let tag = format!("ga2banddef{k}");
        let (_t, mut a) = boot_paint(&tag, BAND_NOTES, GANTT, color, config);
        a.term_bg = term_bg;
        assert_eq!(a.theme, mdgrid::theme::Theme::Default, "{config:?}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        let what = format!("{config:?} {color:?} {term_bg:?}");
        // 時間軸の左端(日の単位の原点 10/01)。名前の欄の幅は単位で変わらない。
        let x0 = bar_x(&screen(&a), "ant");
        for round in ["日", "週", "月"] {
            let (s, buf) = draw(&a);
            let (ty, ly) = header_ys(&s);
            // 今日の字の桁は、日付の無い gamma の行の ┊ で測る。
            let low = line(&s, ly);
            let today = cols_of(&row_line(&s, "gamma"), '┊').first().copied();
            let digit = |x: usize| char_at(&low, x).is_some_and(|c| c.is_ascii_digit());
            let mark = today.filter(|&x| digit(x));
            for (name, y) in [
                ("見出しの上の行", ty),
                ("見出しの下の行", ly),
                ("beta の行", row_y(&s, "beta")),
                ("gamma の行", row_y(&s, "gamma")),
            ] {
                for x in x0..right_edge(&line(&s, y)) {
                    if y == ly && Some(x) == mark {
                        continue;
                    }
                    assert_eq!(
                        bg(&buf, x, y),
                        Color::Reset,
                        "{what} {round} {name}: 桁 {x} に地の色\n{s}"
                    );
                }
            }
            ch(&mut a, '-');
        }
    }
}

// ---- タスク 4: 月の単位でも t で今日を左寄りに ----

#[test]
fn test_ga_4_today_month_left_of_half() {
    // [GA-4] 月の単位で、今日から遠い所(1990 年)を見ているときに `t` → 今日(10/06)の字が時間軸の左から半分より前に
    // 見え、原点は月の1日。時間軸の両端は、全部の範囲を覆う棒(1990〜2099 年)の `█` の両端で測る。
    let list = [
        ("span.md", "---\nstart: 1990-01-01\ndue: 2099-12-31\n---\n"),
        ("nodate.md", "---\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot("ga4todaymon", &list);
    open_month(&mut a);
    assert!(
        !row_line(&screen(&a), "nodate").contains('┊'),
        "開いた所では今日は見えない:\n{}",
        screen(&a)
    );
    ch(&mut a, 't');
    let s = screen(&a);
    let bars = cols_of(&row_line(&s, "span"), '█');
    let (left, right) = (bars[0], *bars.last().unwrap());
    assert_eq!(bars.len(), right - left + 1, "時間軸の全部が棒:\n{s}");
    let today = cols_of(&row_line(&s, "nodate"), '┊');
    assert_eq!(today.len(), 1, "今日の字が1つ見える:\n{s}");
    let off = today[0] - left;
    let len = right - left + 1;
    assert!(
        off * 2 < len,
        "今日は時間軸({len}字)の左から半分より前: {off}字目\n{s}"
    );
    let origin = a.gantt.as_ref().unwrap().origin;
    assert_eq!(civil(origin).2, 1, "{}", format_date(origin));
}

// ---- タスク 5: ほかの画面の型で表の案内を出さない ----

#[test]
fn test_ga_2_no_table_hint_head_row() {
    // [GA-2] 表で見出しの行を選んだまま `C` → ガントのメッセージ行に「見出しの行」の案内が出ない。Esc で表に戻ると
    // 案内はまた出る。ガントの操作の知らせ(子の無い行で Z →「この行に子の行は無い」)は今どおり出る。
    let list = [
        (
            "alpha.md",
            "---\nstart: 2026-10-01\ndue: 2026-10-03\nstatus: todo\n---\n",
        ),
        (
            "beta.md",
            "---\nstart: 2026-10-02\ndue: 2026-10-05\nstatus: done\n---\n",
        ),
    ];
    let (_t, mut a) = boot("ga2hinthead", &list);
    group_by_status(&mut a);
    let s = screen(&a);
    assert!(s.contains(HEAD_HINT), "表では案内が出る:\n{s}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains(HEAD_HINT), "ガントに表の案内:\n{s}");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.cur_head().is_some());
    let s = screen(&a);
    assert!(s.contains(HEAD_HINT), "表に戻ると案内:\n{s}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, 'Z');
    let s = screen(&a);
    assert!(
        s.contains(mdgrid::i18n::Msg::TreeNoKids.text()),
        "ガントの知らせは出る:\n{s}"
    );
}

#[test]
fn test_ga_2_no_table_hint_read_only_row() {
    // [GA-2] 表で読むだけのノート(同じキーが2回ある)の行を選んだまま `C` → ガントのメッセージ行に「読むだけ」の
    // 理由が出ない(表の行とセルの案内は表の画面のもの)。
    let list = [
        (
            "a_dup.md",
            "---\nstatus: todo\nstatus: done\nstart: 2026-10-01\ndue: 2026-10-03\n---\n",
        ),
        ("b_ok.md", "---\nstart: 2026-10-02\ndue: 2026-10-05\n---\n"),
    ];
    let (_t, mut a) = boot("ga2hintro", &list);
    let s = screen(&a);
    assert!(s.contains("読むだけ:"), "表では読むだけの理由が出る:\n{s}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains("読むだけ:"), "ガントに表の案内:\n{s}");
}

#[test]
fn test_tr_1_no_table_hint_head_row() {
    // [TR-1] 表で見出しの行を選んだまま `T` → 木の画面のメッセージ行に「見出しの行」の案内が出ない。
    let list = [
        ("alpha.md", "---\nfolder: x > y\nstatus: todo\n---\n"),
        ("beta.md", "---\nfolder: x\nstatus: done\n---\n"),
        ("gamma.md", "---\nfolder: z\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot_paint(
        "tr1hinthead",
        &list,
        "tree = { path = \"folder\" }",
        ColorMode::None,
        "",
    );
    assert_eq!(a.message, None, "{:?}", a.message);
    group_by_status(&mut a);
    let s = screen(&a);
    assert!(s.contains(HEAD_HINT), "表では案内が出る:\n{s}");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains(HEAD_HINT), "木に表の案内:\n{s}");
}

#[test]
fn test_rel_7_no_table_hint_head_row() {
    // [REL-7] 表で見出しの行を選んだまま `R` → 関係マップのメッセージ行に「見出しの行」の案内が出ない。
    let tmp = super::test_relations::workspace("rel7hinthead");
    let mut a = super::test_relations::boot_tasks(&tmp);
    group_by_status(&mut a);
    let s = screen(&a);
    assert!(s.contains(HEAD_HINT), "表では案内が出る:\n{s}");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains(HEAD_HINT), "関係マップに表の案内:\n{s}");
}
