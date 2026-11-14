//! ガントの時間軸の変更(記録: specs/_changes/2026-11-01-gantt-axis.md)の、レビューの指摘から足した受け入れの試験
//! (錠のある `test_gantt_axis.rs` の共有部分を変えないため、別のファイルに置く):
//! テーマのある組の帯の見分けやすさ(#1)、曖昧な字を2桁で数える設定での右の縁(#2)、9999-12 の印(#3)、
//! 週の単位の月の数の地(#5)、ビューの案内(#6)、ヘルプの下の表の案内(#7)。
//!
//! 値は作ったもの。今日は 2026-10-06(火)に固定する。2026-10-01 は木曜。行番号や字の位置は決め打ちせず、
//! 見出しの語・ノートの名前・探す字で探し、字の位置は表示の幅で数える。色は値を決め打ちせず、違うか同じかと差の大きさを見る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, text, Tmp};
use super::width::width;
use mdgrid::gantt::Unit;
use mdgrid::settings::{Dir, Group};
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

/// 表の見出しの行の案内(「見出しの行(n件)。Enter で開閉」)の語。
const HEAD_HINT: &str = "見出しの行";

/// 集計が式(customAverage)で未対応のビューを1つ持つ `.base`(列は start・due)。
const VIEW_BASE: &str = r#"summaries:
  customAverage: 'values.mean()'
views:
  - type: table
    name: Plan
    order: [start, due]
    summaries:
      due: customAverage
"#;

// ---- 道具(test_gantt_axis.rs から写したもの) ----

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

// ---- 道具(テーマ・幅・メッセージ行) ----

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

// ---- 帯の見分けやすさ(#1)と週の単位の月の数(#5) ----

#[test]
fn test_ga_2_band_contrast_with_themes() {
    // [GA-2] 24 ビットのテーマのある組(default 以外の全部)で、帯は見分けられる: 日の単位の見出しの2行と選んでいない行で、
    // 10/07(水)と 11/04(水)(帯のある月と無い月の平日)、10/03(土)と 10/07(水)、11/07(土)と 11/04(水)の地が、
    // RGB のどれかの成分で 8 以上違う。
    for theme in themes() {
        let tag = format!("ga2bandgap{}", theme.name());
        let config = theme_config(theme);
        let (_t, mut a) = boot_paint(&tag, BAND_NOTES, GANTT, ColorMode::Rgb, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        assert_eq!(unit(&a), Unit::Day);
        let (s, buf) = draw(&a);
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
            let what = format!("{} {name}", theme.name());
            let c = |d: &str| rgb_of(bg(&buf, at(d), y), &format!("{what} {d}"));
            let (wd10, wd11) = (c("2026-10-07"), c("2026-11-04"));
            let (sat10, sat11) = (c("2026-10-03"), c("2026-11-07"));
            assert!(
                gap(wd10, wd11) >= 8,
                "{what}: 10月と11月の平日の地の差が小さい {wd10:?} {wd11:?}\n{s}"
            );
            assert!(
                gap(sat10, wd10) >= 8,
                "{what}: 10月の土と平日の地の差が小さい {sat10:?} {wd10:?}"
            );
            assert!(
                gap(sat11, wd11) >= 8,
                "{what}: 11月の土と平日の地の差が小さい {sat11:?} {wd11:?}"
            );
        }
    }
}

#[test]
fn test_ga_2_band_week_month_digits_follow_month() {
    // [GA-2] 週の単位で、見出しの下の行の月の数「11」(11/01 を含む 10/26 の週から2字)は、2字とも11月の週(11/09 の週)と
    // 同じ地で、10月の週(10/12 の週)とは違う地。24 ビットの、default 以外の全部のテーマで。週の位置は棒の字で測る。
    let notes: &[(&str, &str)] = &[
        ("oct.md", "---\nstart: 2026-10-12\ndue: 2026-10-18\n---\n"),
        ("nov.md", "---\nstart: 2026-11-09\ndue: 2026-11-15\n---\n"),
    ];
    for theme in themes() {
        let tag = format!("ga2bandweek{}", theme.name());
        let config = theme_config(theme);
        let (_t, mut a) = boot_paint(&tag, notes, GANTT, ColorMode::Rgb, &config);
        assert_eq!(a.theme, theme, "{config}");
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        ch(&mut a, '-');
        assert_eq!(unit(&a), Unit::Week);
        let (s, buf) = draw(&a);
        let (_, ly) = header_ys(&s);
        let low = line(&s, ly);
        let (x_oct, x_nov) = (bar_x(&s, "oct"), bar_x(&s, "nov"));
        let mm = must_col(&low, "11");
        let b = |x: usize| bg(&buf, x, ly);
        let what = theme.name();
        assert_eq!(b(mm), b(x_nov), "{what}: 「11」の1字目と11月の週の地\n{s}");
        assert_eq!(b(mm + 1), b(x_nov), "{what}: 「11」の2字目と11月の週の地");
        assert_ne!(b(mm), b(x_oct), "{what}: 「11」と10月の週の地が同じ");
    }
}

// ---- 曖昧な字を2桁で数える設定(CV-6。#2) ----

#[test]
fn test_ga_2_ambiguous_wide_keeps_right_edge() {
    // [GA-2][SR-9] `[terminal] ambiguous_wide = true` で、日の単位と月の単位のガントのどの行も描く幅(端末の幅 - 1)を
    // 超えず、窓の右の縁の字(ASCII の `|`)が、題の行の次から下の縁の前まで、全部の行で同じ桁にある
    // (棒 `█`・`░`、印 `◆`、今日 `┊` が幅2になっても)。
    let notes: &[(&str, &str)] = &[
        ("alpha.md", "---\nstart: 2026-10-01\ndue: 2026-10-07\n---\n"),
        ("beta.md", "---\ndue: 2026-10-03\n---\n"),
        ("gamma.md", "---\nstart: 2026-10-05\ndue: 2026-10-20\n---\n"),
        ("delta.md", "---\nstatus: todo\n---\n"),
    ];
    let config = "[terminal]\nambiguous_wide = true\n";
    let (_t, mut a) = boot_paint("ga2ambig", notes, GANTT, ColorMode::None, config);
    assert!(a.ambiguous_wide, "設定が効く");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(unit(&a), Unit::Day);
    let w = usize::from(a.size.0) - 1;
    for round in ["日", "月"] {
        let lines = render_lines(&a);
        let all = lines.join("\n");
        for l in &lines {
            assert!(width(l) <= w, "{round}: 幅 {} の行: {l}\n{all}", width(l));
        }
        let t = lines
            .iter()
            .position(|l| l.contains("ガント:"))
            .unwrap_or_else(|| panic!("{round}: 題の行が無い:\n{all}"));
        let b = (t + 1..lines.len())
            .find(|&i| lines[i].trim_start().starts_with(['+', '╰']))
            .unwrap_or_else(|| panic!("{round}: 下の縁が無い:\n{all}"));
        assert!(b > t + 3, "{round}: 見出しと行がある:\n{all}");
        let want = right_edge(&lines[t + 1]);
        for (i, l) in lines.iter().enumerate().take(b).skip(t + 1) {
            assert_eq!(right_edge(l), want, "{round}: {i} 行目: {l}\n{all}");
        }
        ch(&mut a, '-');
        ch(&mut a, '-');
    }
}

// ---- 日付の端(9999-12。#3) ----

#[test]
fn test_ga_2_month_point_once_at_year_9999() {
    // [GA-2] due だけが 9999-12-30 の行(ylast)は、月の単位で ◆ がちょうど1つ(9999-12 より右の字に繰り返さない)。
    // 開いた所と、`l` で原点を 9999-12 まで動かした所の両方で。start・due のある普通の行(ynorm)を混ぜる。
    let notes: &[(&str, &str)] = &[
        ("ylast.md", "---\ndue: 9999-12-30\n---\n"),
        ("ynorm.md", "---\nstart: 9999-10-05\ndue: 9999-10-25\n---\n"),
    ];
    let (_t, mut a) = boot("ga2year9999", notes);
    open_month(&mut a);
    let s = screen(&a);
    assert_eq!(count(&row_line(&s, "ylast"), '◆'), 1, "開いた所:\n{s}");
    for _ in 0..12 {
        ch(&mut a, 'l');
    }
    assert_eq!(
        a.gantt.as_ref().unwrap().origin,
        parse_date("9999-12-01").unwrap()
    );
    let s = screen(&a);
    assert_eq!(count(&row_line(&s, "ylast"), '◆'), 1, "原点 9999-12:\n{s}");
}

// ---- ヘルプの下の表の案内(#7)と、ビューの案内(#6) ----

#[test]
fn test_ga_2_no_table_hint_under_help() {
    // [GA-2] 表で見出しの行を選んだまま `C` でガントにし、`?` でヘルプを開く → メッセージ行(描く行の最後の行)に
    // 「見出しの行」の案内が出ない。
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
    let (_t, mut a) = boot("ga2hinthelp", &list);
    group_by_status(&mut a);
    let m = message_line(&a);
    assert!(m.contains(HEAD_HINT), "表では案内が出る: {m}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help, "{:?}", a.message);
    let m = message_line(&a);
    assert!(!m.contains(HEAD_HINT), "ガントのヘルプに表の案内: {m}");
}

#[test]
fn test_tr_1_no_table_hint_under_help() {
    // [TR-1] 表で見出しの行を選んだまま `T` で木にし、`?` でヘルプを開く → メッセージ行に「見出しの行」の案内が出ない。
    let list = [
        ("alpha.md", "---\nfolder: x > y\nstatus: todo\n---\n"),
        ("beta.md", "---\nfolder: x\nstatus: done\n---\n"),
        ("gamma.md", "---\nfolder: z\nstatus: todo\n---\n"),
    ];
    let (_t, mut a) = boot_paint(
        "tr1hinthelp",
        &list,
        "tree = { path = \"folder\" }",
        ColorMode::None,
        "",
    );
    group_by_status(&mut a);
    let m = message_line(&a);
    assert!(m.contains(HEAD_HINT), "表では案内が出る: {m}");
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree, "{:?}", a.message);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help, "{:?}", a.message);
    let m = message_line(&a);
    assert!(!m.contains(HEAD_HINT), "木のヘルプに表の案内: {m}");
}

#[test]
fn test_ga_2_view_hint_still_shown() {
    // [GA-2][BV-7] ガントの画面でも、ビューの案内はメッセージ行に出る: `.base` のビューの集計が式(customAverage)で
    // 未対応 → 表で出る「未対応: …customAverage…」が、`C` で開いたガント(日付の列 start・due から推定)のメッセージ行にも
    // 出る(表の行とセルの案内とは違い、ビューの案内は画面の型によらない)。
    let tmp = Tmp::new("ga2viewhint");
    tmp.write("a.md", "---\nstart: 2026-10-01\ndue: 2026-10-05\n---\n");
    tmp.write("b.md", "---\nstart: 2026-10-03\ndue: 2026-10-08\n---\n");
    let mut a = super::test_grid::open(&tmp, VIEW_BASE, None);
    a.resize(120, 30);
    a.today = parse_date(TODAY).unwrap();
    assert!(a.view_error.is_none(), "{:?}", a.view_error);
    assert!(
        a.notes.iter().any(|n| n.contains("customAverage")),
        "{:?}",
        a.notes
    );
    let hint = |m: &str| m.contains("未対応") && m.contains("customAverage");
    // 開いたときに一度だけ出る知らせを消し、案内だけを見る。セルは読むだけでない列(start)を選ぶ。
    a.message = None;
    col_named(&mut a, "start");
    let m = message_line(&a);
    assert!(hint(&m), "表のビューの案内: {m}");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(a.message, None);
    let m = message_line(&a);
    assert!(hint(&m), "ガントにビューの案内が無い: {m}");
}
