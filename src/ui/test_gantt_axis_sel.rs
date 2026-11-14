//! ガントの時間軸の変更(記録: specs/_changes/2026-11-01-gantt-axis.md)の、レビュー3回目の指摘から足した受け入れの試験
//! (錠のある `test_gantt_axis*.rs` を変えないため、別のファイルに置く): 今日の数の字と選んでいる行の選びの地が帯・土日と
//! 見分けられること(#17)、24 ビットで4つの段と選びの地のどの2つも離れること(#18)、地から各段への差が
//! 土日 < 帯 < 帯の上の土日 の順であること(#20)。
//!
//! 値は作ったもの。今日は 2026-10-06(火)に固定し、#17 だけ月曜に変える。2026-10-01 は木曜で、10/05 と 11/02 は月曜。
//! 行番号や字の位置は決め打ちせず、ノートの名前と探す字で探し、字の位置は表示の幅で数える。色は値を決め打ちせず、
//! 違うか同じかと差の大きさを見る。256 色の番号は xterm の RGB に戻して比べる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, text, Tmp};
use super::width::width;
use mdgrid::gantt::Unit;
use mdgrid::source::markdown::Markdown;
use mdgrid::theme::Theme;
use mdgrid::types::parse_date;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::Terminal;
use std::path::PathBuf;

/// 今日(2026-10-06・火)。
const TODAY: &str = "2026-10-06";
const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 日の単位の地の帯の例の行(範囲 10/01〜11/10 で日の単位、原点 10/01)。ant が先頭で選ばれる。
const BAND_NOTES: &[(&str, &str)] = &[
    ("ant.md", "---\nstart: 2026-10-01\ndue: 2026-10-02\n---\n"),
    ("beta.md", "---\nstart: 2026-11-09\ndue: 2026-11-10\n---\n"),
    ("gamma.md", "---\nstatus: todo\n---\n"),
];

// ---- 道具(test_gantt_axis*.rs から写したもの) ----

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

/// App の大きさで描いた画面の文字と、そのバッファ。
fn draw(a: &App) -> (String, Buffer) {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    let buf = t.backend().buffer().clone();
    (text(&buf), buf)
}

/// 時間軸の見出しの2行の画面の行(窓の題「ガント:」の次の2行)。
fn header_ys(s: &str) -> (usize, usize) {
    let i = s
        .lines()
        .position(|l| l.contains("ガント:"))
        .unwrap_or_else(|| panic!("ガントの窓の題が無い:\n{s}"));
    (i + 1, i + 2)
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

fn unit(a: &App) -> Unit {
    a.gantt.as_ref().expect("ガント").unit
}

/// default 以外の全部のテーマ(暗い地のテーマも明るい地のテーマも)。
fn themes() -> Vec<Theme> {
    Theme::ALL
        .into_iter()
        .filter(|t| *t != Theme::Default)
        .collect()
}

/// テーマ `t` を当てる設定の文。
fn theme_config(t: Theme) -> String {
    format!("[look]\ntheme = \"{}\"\n", t.name())
}

/// 2つの色の、RGB の成分ごとの差の最大。
fn gap(a: [u8; 3], b: [u8; 3]) -> u8 {
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}

/// 256 色の番号の xterm の RGB。16〜231 は立方体(成分は 0・95・135・175・215・255)、232〜255 は灰(8 + 10×i)。
/// 16 より前の番号は端末ごとに色が違うので、ここでは読まない。
fn xterm_rgb(n: u8) -> [u8; 3] {
    const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match n {
        16..=231 => {
            let k = usize::from(n - 16);
            [CUBE[k / 36], CUBE[(k / 6) % 6], CUBE[k % 6]]
        }
        232..=255 => {
            let v = 8 + 10 * (n - 232);
            [v, v, v]
        }
        _ => panic!("16 より前の番号は端末ごとに色が違う: {n}"),
    }
}

// ---- 地の段の読み方 ----

/// セルの地の色の RGB(24 ビットはそのまま、256 色は xterm の RGB に戻す)。ほかの色なら `what` を添えて落ちる。
fn rgb_any(c: Color, what: &str) -> [u8; 3] {
    match c {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Indexed(n) => xterm_rgb(n),
        other => panic!("{what}: 24 ビットの色でも 256 色の番号でもない {other:?}"),
    }
}

/// 日の単位の画面(BAND_NOTES、原点 10/01)の地の段と選びの地。
struct Levels {
    /// 窓の地(選んでいない行 gamma の名前の欄の空き)。
    ground: Color,
    /// 選んでいない行(gamma)の、帯の月の平日・帯の無い月の土日・帯の月の土日(帯の上の土日)の地。
    band_wd: Color,
    plain_we: Color,
    band_we: Color,
    /// 選んでいる行(ant)の時間軸の地(選びの地)。
    sel: Color,
    /// 帯の付く月が11月か(10月なら false)。
    nov_band: bool,
}

/// `a` を描いて地の段を読む。帯の付く月は、10月(10/07・水)と11月(11/04・水)の平日の地を窓の地と比べて決める
/// (一方だけが窓の地と同じことも確かめる)。土日は 10/03(土)と 11/07(土)。
fn levels(a: &App, what: &str) -> Levels {
    let (s, buf) = draw(a);
    let x0 = bar_x(&s, "ant");
    let first = parse_date("2026-10-01").unwrap();
    let at = |d: &str| x0 + usize::try_from(parse_date(d).unwrap() - first).unwrap();
    let gy = row_y(&s, "gamma");
    let b = |d: &str| bg(&buf, at(d), gy);
    let ground = bg(&buf, x0 - 2, gy);
    let (oct_wd, nov_wd) = (b("2026-10-07"), b("2026-11-04"));
    let (oct_we, nov_we) = (b("2026-10-03"), b("2026-11-07"));
    assert!(
        (oct_wd == ground) != (nov_wd == ground),
        "{what}: 一方の月だけが帯 10月 {oct_wd:?} 11月 {nov_wd:?} 地 {ground:?}\n{s}"
    );
    let nov_band = oct_wd == ground;
    let (band_wd, plain_we, band_we) = if nov_band {
        (nov_wd, oct_we, nov_we)
    } else {
        (oct_wd, nov_we, oct_we)
    };
    let sel = bg(&buf, at("2026-10-07"), row_y(&s, "ant"));
    Levels {
        ground,
        band_wd,
        plain_we,
        band_we,
        sel,
        nov_band,
    }
}

/// 今日を `today`(月曜)にして描き、見出しの下の行の今日の数の字と、その左隣(日曜)・右隣(火曜)の地。
/// 今日の字が月曜の日の数に重なっていることも確かめる。
fn today_cells(a: &mut App, today: &str, what: &str) -> (Color, Color, Color) {
    a.today = parse_date(today).unwrap();
    let (s, buf) = draw(a);
    let x0 = bar_x(&s, "ant");
    let first = parse_date("2026-10-01").unwrap();
    let day = parse_date(today).unwrap() - first;
    let x = x0 + usize::try_from(day).unwrap();
    let (_, ly) = header_ys(&s);
    let low = line(&s, ly);
    let c = char_at(&low, x);
    assert!(
        c.is_some_and(|c| c.is_ascii_digit()),
        "{what} {today}: 今日の字が月曜の数に重なる: {c:?}\n{low}"
    );
    let b = |x: usize| bg(&buf, x, ly);
    (b(x), b(x - 1), b(x + 1))
}

/// テーマ・色の種類・見た目(classic か)で開いたガント(日の単位)。
fn open_themed(theme: Theme, color: ColorMode, classic: bool, tag: &str) -> (Tmp, App) {
    let mut config = theme_config(theme);
    if classic {
        config.push_str("mode = \"classic\"\n");
    }
    let (t, mut a) = boot_paint(tag, BAND_NOTES, GANTT, color, &config);
    assert_eq!(a.theme, theme, "{config}");
    assert_eq!(a.look_classic, classic, "{config}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(unit(&a), Unit::Day);
    (t, a)
}

// ---- #17: 今日の数の字と選んでいる行の選びの地 ----

#[test]
fn test_ga_2_today_digit_stands_out_from_bands() {
    // [GA-2][SR-15] default 以外の全部のテーマを、24 ビットと 256 色、モダンな見た目と classic で:
    // - 帯の月の月曜を今日にすると、見出しの下の行の今日の数の字(選びの地)と、右隣の帯の平日・左隣の帯の上の日曜の地の
    //   成分の差の最大が 8 以上。
    // - 帯の無い月の月曜を今日にすると、今日の数の字と左隣の日曜(土日の段)の地の差が 8 以上。
    // - 選んでいる行(ant)の時間軸の地と、選んでいない行(gamma)の帯の平日・土日・帯の上の土日の地の差が 8 以上。
    // 10/05 と 11/02 が月曜(数「5」「2」)。帯の付く月は見て決める。
    for theme in themes() {
        for color in [ColorMode::Rgb, ColorMode::Indexed] {
            for classic in [false, true] {
                let look = if classic { "classic" } else { "modern" };
                let what = format!("{} {color:?} {look}", theme.name());
                let tag = format!("ga2sel{}{color:?}{look}", theme.name());
                let (_t, mut a) = open_themed(theme, color, classic, &tag);
                let lv = levels(&a, &what);
                let c = |x: Color, label: &str| rgb_any(x, &format!("{what} {label}"));
                let sel = c(lv.sel, "選んでいる行");
                for (label, x) in [
                    ("帯の平日", lv.band_wd),
                    ("土日", lv.plain_we),
                    ("帯の上の土日", lv.band_we),
                ] {
                    let v = c(x, label);
                    let g = gap(sel, v);
                    assert!(
                        g >= 8,
                        "{what}: 選んでいる行の地 {sel:?} と {label} {v:?} の差が {g}"
                    );
                }
                let (band_mon, plain_mon) = if lv.nov_band {
                    ("2026-11-02", "2026-10-05")
                } else {
                    ("2026-10-05", "2026-11-02")
                };
                let (d, left, right) = today_cells(&mut a, band_mon, &what);
                let d = c(d, "今日の数の字");
                for (label, x) in [("右隣の帯の平日", right), ("左隣の帯の上の日曜", left)]
                {
                    let v = c(x, label);
                    let g = gap(d, v);
                    assert!(
                        g >= 8,
                        "{what} 今日 {band_mon}: 今日の数の字 {d:?} と {label} {v:?} の差が {g}"
                    );
                }
                let (d, left, _) = today_cells(&mut a, plain_mon, &what);
                let (d, v) = (c(d, "今日の数の字"), c(left, "左隣の日曜"));
                let g = gap(d, v);
                assert!(
                    g >= 8,
                    "{what} 今日 {plain_mon}: 今日の数の字 {d:?} と左隣の日曜(土日) {v:?} の差が {g}"
                );
            }
        }
    }
}

// ---- #18: 24 ビットの4つの段と選びの地 ----

#[test]
fn test_ga_2_band_levels_far_from_selection() {
    // [GA-2] 24 ビットで、default 以外の全部のテーマについて、4つの段(地・帯・土日・帯の上の土日)と選びの地(選んでいる行の
    // 時間軸の地)のどの2つも、成分の差の最大が 8 以上(一行おきの色 (58,58,58) を避けて寄せる gruvbox も含む)。
    for theme in themes() {
        let what = theme.name();
        let tag = format!("ga2lvsel{what}");
        let (_t, a) = open_themed(theme, ColorMode::Rgb, false, &tag);
        let lv = levels(&a, what);
        let cells: Vec<(&str, [u8; 3])> = [
            ("地", lv.ground),
            ("帯", lv.band_wd),
            ("土日", lv.plain_we),
            ("帯の上の土日", lv.band_we),
            ("選びの地", lv.sel),
        ]
        .into_iter()
        .map(|(label, x)| (label, rgb_any(x, &format!("{what} {label}"))))
        .collect();
        for (i, &(la, va)) in cells.iter().enumerate() {
            for &(lb, vb) in &cells[i + 1..] {
                let g = gap(va, vb);
                assert!(g >= 8, "{what}: {la} {va:?} と {lb} {vb:?} の差が {g}");
            }
        }
    }
}

// ---- #20: 地から各段への差の順 ----

#[test]
fn test_ga_2_band_levels_ordered_from_ground() {
    // [GA-2] 24 ビットと 256 色で、default 以外の全部のテーマについて、地から各段への差(成分の差の最大。256 色は xterm の
    // RGB に戻す)が 土日 < 帯 < 帯の上の土日 の順(帯が主で、土日は控えめ)。
    for theme in themes() {
        for color in [ColorMode::Rgb, ColorMode::Indexed] {
            let what = format!("{} {color:?}", theme.name());
            let tag = format!("ga2lvord{}{color:?}", theme.name());
            let (_t, a) = open_themed(theme, color, false, &tag);
            let lv = levels(&a, &what);
            let c = |x: Color, label: &str| rgb_any(x, &format!("{what} {label}"));
            let ground = c(lv.ground, "地");
            let we = gap(ground, c(lv.plain_we, "土日"));
            let band = gap(ground, c(lv.band_wd, "帯"));
            let both = gap(ground, c(lv.band_we, "帯の上の土日"));
            assert!(
                we < band && band < both,
                "{what}: 地からの差 土日 {we}・帯 {band}・帯の上の土日 {both}"
            );
        }
    }
}
