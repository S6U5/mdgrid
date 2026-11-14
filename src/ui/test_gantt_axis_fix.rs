//! ガントの時間軸の変更(記録: specs/_changes/2026-11-01-gantt-axis.md)の、レビュー2回目の指摘から足した受け入れの試験
//! (錠のある `test_gantt_axis.rs`・`test_gantt_axis_more.rs` を変えないため、別のファイルに置く):
//! 256 色の帯の段の見分けやすさ(#12)、月の境が週末に当たるときの帯の平日と帯なしの土日(#13)、9999-12 より右の
//! 月の単位の帯(#14)、関係マップの開けないビューの案内(#15)、classic の見た目の今日の数の字の色(#16)。
//!
//! 値は作ったもの。今日は 2026-10-06(火)に固定する(#16 だけ月曜に変える)。2026-10-01 は木曜。行番号や字の位置は
//! 決め打ちせず、見出しの語・ノートの名前・探す字で探し、字の位置は表示の幅で数える。色は値を決め打ちせず、違うか同じかと
//! 差の大きさを見る。

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

/// 1つ目のビューの絞り込みが評価できない(開けない)`.base`。2つ目は開けるビュー。
const BAD_BASE: &str = r#"views:
  - type: table
    name: 壊れた
    filters: 'bar(status)'
  - type: table
    name: 全部
"#;

// ---- 道具(test_gantt_axis.rs・test_gantt_axis_more.rs から写したもの) ----

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

/// `C` でガントを開き、`-` を2回で月の単位にする(日 → 週 → 月。月はそれより粗くならない)。
fn open_month(a: &mut App) {
    ch(a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(a, '-');
    ch(a, '-');
    assert_eq!(unit(a), Unit::Month);
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

/// 24 ビットの色の成分。ほかの色なら `what` を添えて落ちる。
fn rgb_of(c: Color, what: &str) -> [u8; 3] {
    match c {
        Color::Rgb(r, g, b) => [r, g, b],
        other => panic!("{what}: 24 ビットの色でない {other:?}"),
    }
}

/// 2つの色の、RGB の成分ごとの差の最大。
fn gap(a: [u8; 3], b: [u8; 3]) -> u8 {
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}

/// 行の、窓の右の縁の字(`│` か ASCII の `|`)の位置(表示の幅で)。無ければ行の幅。
fn right_edge(l: &str) -> usize {
    match l.rfind(['│', '|']) {
        Some(i) => width(&l[..i]),
        None => width(l),
    }
}

/// 描く幅と高さ(端末の右の1桁と最下行を除く。`ui::draw` と同じ)で組んだ画面の行の文字。
fn render_lines(a: &App) -> Vec<String> {
    let w = usize::from(a.size.0).saturating_sub(1);
    let h = usize::from(a.size.1).saturating_sub(1);
    super::view::render(a, w, h)
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect()
}

/// メッセージ行(描く行の最後の行)の文字。
fn message_line(a: &App) -> String {
    render_lines(a).pop().unwrap_or_default()
}

// ---- 256 色の番号 ----

/// 256 色の番号。ほかの色なら `what` を添えて落ちる。
fn index_of(c: Color, what: &str) -> u8 {
    match c {
        Color::Indexed(n) => n,
        other => panic!("{what}: 256 色の番号でない {other:?}"),
    }
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

// ---- #12: 256 色の帯の段 ----

#[test]
fn test_ga_2_band_indexed_levels_far_apart() {
    // [GA-2][SR-15] 256 色で、default 以外の全部のテーマについて、日の単位の見出しの上の行と選んでいない行(gamma)の
    // 地の番号(10/07 = 10月の平日、10/03 = 10月の土、11/04 = 11月の平日、11/07 = 11月の土。帯の無い月と帯の月の、平日と
    // 土日)を xterm の RGB に戻すと、どの2つの組も(月の境と週末の境で、どの組も隣り合いうる)成分の差の最大が 8 以上。
    // 帯の無い月の地は theme::paint が塗ったテーマの地の番号で、描いた buffer のセルの番号のまま見る。
    const DAYS: [(&str, &str); 4] = [
        ("10月の平日", "2026-10-07"),
        ("10月の土", "2026-10-03"),
        ("11月の平日", "2026-11-04"),
        ("11月の土", "2026-11-07"),
    ];
    for theme in themes() {
        let tag = format!("ga2idx{}", theme.name());
        let config = theme_config(theme);
        let (_t, mut a) = boot_paint(&tag, BAND_NOTES, GANTT, ColorMode::Indexed, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        assert_eq!(unit(&a), Unit::Day);
        let (s, buf) = draw(&a);
        let x0 = bar_x(&s, "ant");
        let first = parse_date("2026-10-01").unwrap();
        let at = |d: &str| x0 + usize::try_from(parse_date(d).unwrap() - first).unwrap();
        let (ty, _) = header_ys(&s);
        for (name, y) in [("見出しの上の行", ty), ("gamma の行", row_y(&s, "gamma"))] {
            let what = format!("{} {name}", theme.name());
            let cells: Vec<(&str, u8)> = DAYS
                .iter()
                .map(|&(label, d)| (label, index_of(bg(&buf, at(d), y), &what)))
                .collect();
            for (i, &(la, na)) in cells.iter().enumerate() {
                for &(lb, nb) in &cells[i + 1..] {
                    let g = gap(xterm_rgb(na), xterm_rgb(nb));
                    assert!(
                        g >= 8,
                        "{what}: {la}({na} = {:?})と {lb}({nb} = {:?})の差が {g}",
                        xterm_rgb(na),
                        xterm_rgb(nb)
                    );
                }
            }
        }
    }
}

// ---- #13: 月の境が週末に当たるとき ----

#[test]
fn test_ga_2_band_weekday_vs_weekend_across_month() {
    // [GA-2] 24 ビットで、default 以外の全部のテーマについて、帯の月の平日と帯の無い月の土日の地が、RGB のどれかの成分で
    // 8 以上違う。月の境が週末に当たる 2026-07-31(金)と 08-01(土)を使う(どちらの月が帯でも、帯の平日と帯なしの土日か、
    // その逆がここで隣り合う)。帯の付く月は、7月(07-29・水)と8月(08-05・水)の平日の地を、窓の地(名前の欄の空き)と
    // 比べて決める。見出しの上の行と選んでいない行(gamma)で。
    let notes: &[(&str, &str)] = &[
        ("ant.md", "---\nstart: 2026-07-20\ndue: 2026-07-21\n---\n"),
        ("beta.md", "---\nstart: 2026-08-10\ndue: 2026-08-11\n---\n"),
        ("gamma.md", "---\nstatus: todo\n---\n"),
    ];
    for theme in themes() {
        let tag = format!("ga2edge{}", theme.name());
        let config = theme_config(theme);
        let (_t, mut a) = boot_paint(&tag, notes, GANTT, ColorMode::Rgb, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        assert_eq!(unit(&a), Unit::Day);
        let (s, buf) = draw(&a);
        let x0 = bar_x(&s, "ant");
        let first = parse_date("2026-07-20").unwrap();
        let at = |d: &str| x0 + usize::try_from(parse_date(d).unwrap() - first).unwrap();
        let (ty, _) = header_ys(&s);
        for (name, y) in [("見出しの上の行", ty), ("gamma の行", row_y(&s, "gamma"))] {
            let what = format!("{} {name}", theme.name());
            let c = |d: &str| rgb_of(bg(&buf, at(d), y), &format!("{what} {d}"));
            let ground = rgb_of(bg(&buf, x0 - 2, y), &format!("{what} 名前の欄"));
            let (jul, aug) = (c("2026-07-29"), c("2026-08-05"));
            assert!(
                (jul == ground) != (aug == ground),
                "{what}: 7月と8月の一方だけが帯 {jul:?} {aug:?} 地 {ground:?}\n{s}"
            );
            // 帯の月の平日と、帯の無い月の土日。
            let (band_wd, plain_we) = if aug == ground {
                (c("2026-07-31"), c("2026-08-01"))
            } else {
                (c("2026-08-03"), c("2026-07-25"))
            };
            assert!(
                gap(band_wd, plain_we) >= 8,
                "{what}: 帯の月の平日 {band_wd:?} と帯の無い月の土日 {plain_we:?}"
            );
            // 隣り合う 07-31(金)と 08-01(土)。
            let (fri, sat) = (c("2026-07-31"), c("2026-08-01"));
            assert!(
                gap(fri, sat) >= 8,
                "{what}: 07-31(金){fri:?} と 08-01(土){sat:?}"
            );
        }
    }
}

// ---- #14: 9999-12 より右の月の単位の帯 ----

/// 原点から `months` か月分のマス(1か月3字)の3字の地が同じで、その右(9999-12 より右)の字どうしも同じ地であること。
/// 見出しの2行と選んでいない行(ylast・znone)で見る。時間軸の左端は、全部の範囲を覆う棒(aspan)の最初の字。
fn assert_even_cells(a: &App, months: usize, what: &str) {
    let (s, buf) = draw(a);
    let x0 = bar_x(&s, "aspan");
    let (ty, ly) = header_ys(&s);
    for (name, y) in [
        ("見出しの上の行", ty),
        ("見出しの下の行", ly),
        ("ylast の行", row_y(&s, "ylast")),
        ("znone の行", row_y(&s, "znone")),
    ] {
        let b = |x: usize| bg(&buf, x, y);
        for k in 0..months {
            let x = x0 + 3 * k;
            assert!(
                b(x) == b(x + 1) && b(x) == b(x + 2),
                "{what} {name}: {k} か月目のマスの3字の地 {:?} {:?} {:?}\n{s}",
                b(x),
                b(x + 1),
                b(x + 2)
            );
        }
        let rest = x0 + 3 * months;
        for x in rest..right_edge(&line(&s, y)) {
            assert_eq!(
                b(x),
                b(rest),
                "{what} {name}: 9999-12 より右の桁 {x} と {rest} の地\n{s}"
            );
        }
    }
}

#[test]
fn test_ga_2_band_month_cells_even_past_9999() {
    // [GA-2] テーマのある組(default 以外の全部、24 ビットと 256 色)で月の単位にし、時間軸を 9999-12 より右まで見せる
    // (`l` で原点を 9999-12、`h` で 9999-11)と、1か月3字のマス(9999-11・9999-12)の3字の地が同じで、9999-12 より右の
    // 字どうしも同じ地(3字ごとの縞にならない)。
    let notes: &[(&str, &str)] = &[
        ("aspan.md", "---\nstart: 9999-01-01\ndue: 9999-12-31\n---\n"),
        ("ylast.md", "---\ndue: 9999-12-30\n---\n"),
        ("znone.md", "---\nstatus: todo\n---\n"),
    ];
    for theme in themes() {
        for color in [ColorMode::Rgb, ColorMode::Indexed] {
            let tag = format!("ga2even{}{color:?}", theme.name());
            let config = theme_config(theme);
            let (_t, mut a) = boot_paint(&tag, notes, GANTT, color, &config);
            assert_eq!(a.theme, theme, "{config}");
            open_month(&mut a);
            for _ in 0..24 {
                ch(&mut a, 'l');
            }
            let origin = a.gantt.as_ref().unwrap().origin;
            assert_eq!(origin, parse_date("9999-12-01").unwrap());
            let what = format!("{} {color:?}", theme.name());
            assert_even_cells(&a, 1, &format!("{what} 原点 9999-12"));
            ch(&mut a, 'h');
            let origin = a.gantt.as_ref().unwrap().origin;
            assert_eq!(origin, parse_date("9999-11-01").unwrap());
            assert_even_cells(&a, 2, &format!("{what} 原点 9999-11"));
        }
    }
}

// ---- #15: 関係マップの開けないビューの案内 ----

#[test]
fn test_rel_7_no_view_error_hint() {
    // [REL-7][BV-7] 今の表のビューが開けない(`.base` の評価できない絞り込み)ときに `R` で関係マップを開く → メッセージ行に
    // 開けないビューの案内(「このビューは開けない(理由は表の欄)。[ ] でほかのビューへ」)が出ない(関係マップには表の欄も
    // `[` `]` も無い)。関係マップは今の表だけでも開く。
    let tmp = Tmp::new("rel7viewerr");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    let mut a = super::test_grid::open(&tmp, BAD_BASE, None);
    a.resize(120, 30);
    assert!(a.view_error.is_some(), "開けないビュー");
    let hint = mdgrid::i18n::Msg::HintViewError.text();
    // 開いたときの知らせを消し、案内だけを見る。
    a.message = None;
    let m = message_line(&a);
    assert!(m.contains(hint), "表では案内が出る: {m}");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    let m = message_line(&a);
    assert!(!m.contains(hint), "関係マップに開けないビューの案内: {m}");
}

// ---- #16: classic の見た目の今日の数の字 ----

#[test]
fn test_ga_2_classic_today_digit_same_in_band() {
    // [GA-2][SR-15] classic の見た目([look] mode = "classic")と default 以外の全部のテーマ、24 ビットで、日の単位の見出しの
    // 下の行の今日の字が月曜の日の数に重なるとき(今日を 2026-10-05(月)と 2026-11-02(月)にする)、帯のある月の今日の数の
    // 字と、帯の無い月の今日の数の字の、地と文字の色が同じ(どちらも選びの色)。帯のある月かどうかは、その週の水曜(今日の
    // 2日あと)の見出しの下の行の地を、窓の地(名前の欄の空き)と比べて見る。
    for theme in themes() {
        let config = format!("{}mode = \"classic\"\n", theme_config(theme));
        let mut seen = Vec::new();
        for (k, today) in ["2026-10-05", "2026-11-02"].into_iter().enumerate() {
            let tag = format!("ga2classic{}{k}", theme.name());
            let (_t, mut a) = boot_paint(&tag, BAND_NOTES, GANTT, ColorMode::Rgb, &config);
            assert_eq!(a.theme, theme, "{config}");
            assert!(a.look_classic, "{config}");
            a.today = parse_date(today).unwrap();
            ch(&mut a, 'C');
            assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
            assert_eq!(unit(&a), Unit::Day);
            let (s, buf) = draw(&a);
            let x0 = bar_x(&s, "ant");
            let first = parse_date("2026-10-01").unwrap();
            let day = parse_date(today).unwrap() - first;
            let x = x0 + usize::try_from(day).unwrap();
            let (_, ly) = header_ys(&s);
            let low = line(&s, ly);
            let digit = char_at(&low, x);
            assert!(
                digit.is_some_and(|c| c.is_ascii_digit()),
                "{} {today}: 今日の字が月曜の数に重なる: {digit:?}\n{low}",
                theme.name()
            );
            let cell = &buf[(u16::try_from(x).unwrap(), u16::try_from(ly).unwrap())];
            let banded = bg(&buf, x + 2, ly) != bg(&buf, x0 - 2, ly);
            seen.push((today, banded, cell.bg, cell.fg));
        }
        let name = theme.name();
        assert_ne!(seen[0].1, seen[1].1, "{name}: 一方の月だけが帯 {seen:?}");
        assert_eq!(
            (seen[0].2, seen[0].3),
            (seen[1].2, seen[1].3),
            "{name}: 今日の数の字の地と文字の色(10月と11月) {seen:?}"
        );
    }
}
