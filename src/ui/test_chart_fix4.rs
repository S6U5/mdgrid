//! グラフの実地の操作(2026-11-25。台帳の写しで撮った画面)で見つかった直しの試験: 値の名前とアイコン、週の始まり、
//! `y` のある積み上げ棒の内訳の選び方、明るい地のテーマでの色の対比。値は作ったもの。実装の前に書いた。
//!
//! 色の対比は WCAG の相対輝度の比を、この試験の中で計算する(実装の計算を使わない)。色は描き終えた画面のバッファ
//! (テーマの塗り替えのあと)から読む。256 色の番号は `theme::indexed_rgb` で色に戻す。テーマは `Theme::ALL` を
//! 回すので、テーマを足しても同じ試験が回る。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_chart::{
    assert_table, boot, chart, md, open, row_names, status_notes, Chart, SEPS,
};
use super::test_screen::{ch, press, text, Tmp};
use mdgrid::source::markdown::Markdown;
use mdgrid::theme::{indexed_rgb, Theme};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;
use ratatui::Terminal;
use std::path::PathBuf;

/// 値の名前とアイコンを書いた status の決まり。
const STATUS_RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\nicons = { todo = \"○\" }\nlabels = { todo = \"未着手\", doing = \"進行中\" }\n";

/// 名前と中身の並びを、`md` に渡せる形に。
fn refs(list: &[(String, String)]) -> Vec<(&str, &str)> {
    list.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

// ---- chart.value-labels ----

#[test]
fn test_chart_value_labels_bar_items_and_enter() {
    // [utest -> req~chart.value-labels~1] status の決まりに icons・labels → 棒の項目の名前は「○ 未着手」・「進行中」・「done」
    // (決まりの順)、最初の項目の説明は「○ 未着手: 2」。その項目で Enter → 表は status = todo の2行(値のままで絞る)。
    let (_t, mut a) = boot(
        "chlabbar",
        &status_notes(),
        STATUS_RULES,
        "chart = { x = \"status\" }",
    );
    let c = open(&mut a);
    let names: Vec<String> = c.items().into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["○ 未着手", "進行中", "done"], "{}", c.raw);
    assert_eq!(c.desc(), "○ 未着手: 2", "{}", c.raw);
    press(&mut a, KeyCode::Enter);
    assert_table(&a);
    let mut rows = row_names(&a);
    rows.sort();
    assert_eq!(rows, ["alpha", "beta"]);
}

/// owner(alice・bob)× status の行。alice は todo 2・done 1、bob は todo 1・doing 1。
fn owner_notes() -> Vec<(String, String)> {
    md(&[
        ("a1", "owner: alice\nstatus: todo"),
        ("a2", "owner: alice\nstatus: todo"),
        ("a3", "owner: alice\nstatus: done"),
        ("b1", "owner: bob\nstatus: todo"),
        ("b2", "owner: bob\nstatus: doing"),
    ])
}

/// 選びの説明より下の体(凡例・ヒートマップの見出しと行・棒)の文字。
fn below_text(c: &Chart) -> String {
    c.below()
        .iter()
        .map(|r| r.text.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_chart_value_labels_heatmap_heads_and_stacked_legend() {
    // [utest -> req~chart.value-labels~1] owner で分けて status を内訳にしたヒートマップ → 列の見出しに「○ 未着手」と「進行中」、
    // todo・doing の字は無い。`]` で積み上げ棒 → 凡例も同じ。
    let (_t, mut a) = boot(
        "chlabheat",
        &owner_notes(),
        STATUS_RULES,
        "chart = [{ kind = \"heatmap\", x = \"owner\", by = \"status\" }, { kind = \"stacked\", x = \"owner\", by = \"status\" }]",
    );
    let c = open(&mut a);
    assert_eq!(c.kind(), "ヒートマップ", "{}", c.raw);
    let body = below_text(&c);
    for v in ["○ 未着手", "進行中"] {
        assert!(body.contains(v), "ヒートマップの見出しに {v}:\n{}", c.raw);
    }
    for v in ["todo", "doing"] {
        assert!(
            !body.contains(v),
            "ヒートマップに値の {v} が残る:\n{}",
            c.raw
        );
    }
    assert!(
        c.desc().contains("○ 未着手"),
        "選んだます目の説明も名前で: {}\n{}",
        c.desc(),
        c.raw
    );
    ch(&mut a, ']');
    let c = chart(&a);
    assert_eq!(c.kind(), "積み上げ棒", "{}", c.raw);
    let legend = c.legend();
    for v in ["○ 未着手", "進行中"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    for v in ["todo", "doing"] {
        assert!(!legend.contains(v), "凡例に値の {v} が残る: {legend:?}");
    }
}

#[test]
fn test_chart_value_labels_heatmap_heads_fit_five_wide_names() {
    // [utest -> req~chart.value-labels~1] 全角5字の名前に幅2の字を付けた「印 あとで読む」(幅 13)は、ヒートマップの列の見出しで切れずに出る。
    let (_t, mut a) = boot(
        "chlabwide",
        &md(&[
            ("a1", "owner: alice\nstatus: later"),
            ("b1", "owner: bob\nstatus: done"),
        ]),
        "[rules.status]\nvalues = [\"later\", \"done\"]\nicons = { later = \"印\" }\nlabels = { later = \"あとで読む\" }\n",
        "chart = { kind = \"heatmap\", x = \"owner\", by = \"status\" }",
    );
    let c = open(&mut a);
    assert!(
        below_text(&c).contains("印 あとで読む"),
        "見出しが切れない:\n{}",
        c.raw
    );
}

// ---- chart.buckets-week-start ----

/// due が 2026-11-22(日)と 2026-11-23(月)の2行。
fn week_notes() -> Vec<(String, String)> {
    md(&[("sun", "due: 2026-11-22"), ("mon", "due: 2026-11-23")])
}

#[test]
fn test_chart_week_starts_on_sunday_by_default() {
    // [utest -> req~chart.buckets-week-start~1] dates.week_start を書いていない → 週は日曜で区切り、区切りは「2026-11-22」の1つで 2。
    let (_t, mut a) = boot(
        "chweeksun",
        &week_notes(),
        "[types]\ndue = \"date\"\n",
        "chart = { x = \"due\", unit = \"week\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        [("2026-11-22".to_string(), "2".to_string())],
        "{}",
        c.raw
    );
}

#[test]
fn test_chart_week_starts_on_monday_when_set() {
    // [utest -> req~chart.buckets-week-start~1] `[dates] week_start = "mon"` → 区切りは「2026-11-16」に 1 と「2026-11-23」に 1。
    let (_t, mut a) = boot(
        "chweekmon",
        &week_notes(),
        "[types]\ndue = \"date\"\n\n[dates]\nweek_start = \"mon\"\n",
        "chart = { x = \"due\", unit = \"week\" }",
    );
    let c = open(&mut a);
    assert_eq!(
        c.items(),
        [
            ("2026-11-16".to_string(), "1".to_string()),
            ("2026-11-23".to_string(), "1".to_string())
        ],
        "{}",
        c.raw
    );
}

// ---- chart.bar-stacked-top5 ----

#[test]
fn test_chart_stacked_picks_series_by_y_sum() {
    // [utest -> req~chart.bar-stacked-top5~2] y = amount の合計: housing 1行(80000)、food 10行(各 1000)、comm・daily・fun・health・
    // transport が3行ずつ(各 100・300・200・10・50)→ 内訳は housing・food・daily・fun・comm で、health と transport は「その他」。
    let mut list: Vec<(String, String)> = vec![(
        "rent".to_string(),
        "who: me\ncategory: housing\namount: 80000".to_string(),
    )];
    for k in 0..10 {
        list.push((
            format!("food-{k}"),
            "who: me\ncategory: food\namount: 1000".to_string(),
        ));
    }
    for (cat, amount) in [
        ("comm", 100),
        ("daily", 300),
        ("fun", 200),
        ("health", 10),
        ("transport", 50),
    ] {
        for k in 0..3 {
            list.push((
                format!("{cat}-{k}"),
                format!("who: me\ncategory: {cat}\namount: {amount}"),
            ));
        }
    }
    let (_t, mut a) = boot(
        "chtop5y",
        &md(&refs(&list)),
        "[types]\namount = \"number\"\n",
        "chart = { kind = \"stacked\", x = \"who\", by = \"category\", y = \"amount\" }",
    );
    let c = open(&mut a);
    let legend = c.legend();
    for v in ["housing", "food", "daily", "fun", "comm", "その他"] {
        assert!(legend.contains(v), "凡例に {v}: {legend:?}\n{}", c.raw);
    }
    for v in ["health", "transport"] {
        assert!(
            !legend.contains(v),
            "{v} は「その他」: {legend:?}\n{}",
            c.raw
        );
    }
}

// ---- 色の対比(chart.heatmap-readable・chart.heatmap-shade-blend・chart.series-contrast) ----

/// sRGB の成分を線形に。
fn linear(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG の相対輝度。
fn luminance(c: [u8; 3]) -> f64 {
    0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2])
}

/// WCAG の色の対比(1〜21)。
fn ratio(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

/// 画面の色の RGB。端末の既定の色(Reset)は `default`(None なら読めない色として None)。
fn rgb(c: Color, default: Option<[u8; 3]>) -> Option<[u8; 3]> {
    match c {
        Color::Rgb(r, g, b) => Some([r, g, b]),
        Color::Indexed(n) => {
            Some(indexed_rgb(n).unwrap_or_else(|| panic!("端末ごとに違う色の番号 {n}")))
        }
        Color::Reset => default,
        other => panic!("名前の色 {other:?} は RGB に戻せない"),
    }
}

/// 試すテーマと色の組: 全部のテーマ × 24 ビット・256 色。既定のテーマは端末の地が暗い(分からない)ときと明るいとき。
/// (名前, テーマ, 色, 端末の地)。
fn painted() -> Vec<(String, Theme, ColorMode, Option<[u8; 3]>)> {
    let mut out = Vec::new();
    for t in Theme::ALL {
        for c in [ColorMode::Rgb, ColorMode::Indexed] {
            out.push((format!("{} {c:?}", t.name()), t, c, None));
            if t == Theme::Default {
                out.push((
                    format!("{} {c:?} 明るい端末", t.name()),
                    t,
                    c,
                    Some([255, 255, 255]),
                ));
            }
        }
    }
    out
}

/// 設定 `config`(cli.config-file の形)で、`notes` を表として開き、読み終える(幅 120・高さ 30)。
fn boot_theme(
    tag: &str,
    notes: &[(String, String)],
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
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// テーマの組で開き、端末の地を決めて `P` でグラフを開く。
fn open_painted(
    tag: &str,
    notes: &[(String, String)],
    line: &str,
    theme: Theme,
    color: ColorMode,
    term_bg: Option<[u8; 3]>,
) -> (Tmp, App) {
    let config = format!("[look]\ntheme = \"{}\"\n", theme.name());
    let (t, mut a) = boot_theme(tag, notes, line, color, &config);
    assert_eq!(a.theme, theme, "{config}");
    a.term_bg = term_bg;
    ch(&mut a, 'P');
    assert_eq!(a.mode, Mode::Chart, "{:?}", a.message);
    (t, a)
}

/// 描いた画面(文字とバッファ)。
fn draw(a: &App) -> (String, Buffer) {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    let buf = t.backend().buffer().clone();
    (text(&buf), buf)
}

/// 画面の行 `y` のうち、桁 `from` より右で、グラフの区切りの縦の線(`SEPS`)の右から枠の右の辺の手前までのセル
/// (桁, 字)。
fn after_sep(buf: &Buffer, y: u16, from: u16) -> Vec<(u16, String)> {
    let w = buf.area.width;
    let mut x = from;
    while x < w && !SEPS.iter().any(|s| buf[(x, y)].symbol() == s.to_string()) {
        x += 1;
    }
    ((x + 1)..w.saturating_sub(1))
        .map(|x| (x, buf[(x, y)].symbol().to_string()))
        .collect()
}

/// heatmap の例: 縦 r は c4 4・c3 3・c2 2・c1 1・c5 0、縦 s は c5 1 だけ(r の c5 は 0)。
fn heat_notes() -> Vec<(String, String)> {
    let mut list = Vec::new();
    for (col, n) in [("c1", 1), ("c2", 2), ("c3", 3), ("c4", 4)] {
        for k in 0..n {
            list.push((format!("r-{col}-{k}"), format!("row: r\ncol: {col}")));
        }
    }
    list.push(("s-c5".to_string(), "row: s\ncol: c5".to_string()));
    md(&refs(&list))
}

#[test]
fn test_chart_heatmap_numbers_readable_in_every_theme() {
    // [utest -> req~chart.heatmap-readable~1, req~chart.heatmap-shade-blend~1] 全部のテーマ × 24 ビット・256 色(既定のテーマは暗い端末と明るい
    // 端末)で、値 0〜4 のます目のヒートマップを描く。選びを s の行へ動かし、r の行(値 4・3・2・1・0。どれも選んでいない)を見る:
    // 数の字と地の対比は 4.5 以上。値 1〜4 のます目の地と画面の地の対比は、値の大きい順に大きい。濃さの字の色はます目の地と同じ。
    let line = "chart = { kind = \"heatmap\", x = \"row\", by = \"col\" }";
    for (k, (what, theme, color, term_bg)) in painted().into_iter().enumerate() {
        let (_t, mut a) = open_painted(
            &format!("chheat{k}"),
            &heat_notes(),
            line,
            theme,
            color,
            term_bg,
        );
        ch(&mut a, 'j');
        let c = chart(&a);
        assert_eq!(c.kind(), "ヒートマップ", "{what}\n{}", c.raw);
        let row = c.heat_row("r");
        let (s, buf) = draw(&a);
        let screen_bg_default = term_bg.or(Some([0, 0, 0]));
        // 画面の地: 体の右の端の空いたセル。
        let screen_bg = rgb(buf[(buf.area.width - 3, row.y)].bg, screen_bg_default).unwrap();
        let mut levels: Vec<(u32, [u8; 3])> = Vec::new();
        for (x, sym) in after_sep(&buf, row.y, row.x) {
            let cell = &buf[(x, row.y)];
            let bg = rgb(cell.bg, screen_bg_default).unwrap();
            if let Some(d) = sym.chars().next().and_then(|ch| ch.to_digit(10)) {
                // 端末の既定の文字の色(テーマなしの値 0 のます目)は、端末に任せる。
                if let Some(fg) = rgb(cell.fg, None) {
                    let r = ratio(fg, bg);
                    assert!(
                        r >= 4.5,
                        "{what}: 値 {d} のます目の数の字 {fg:?} と地 {bg:?} の対比 {r:.2}\n{s}"
                    );
                }
                if d > 0 {
                    levels.push((d, bg));
                }
            } else if "░▒▓█".contains(sym.as_str()) {
                assert_eq!(
                    cell.fg, cell.bg,
                    "{what}: 濃さの字 {sym} の色はます目の地と同じ\n{s}"
                );
            }
        }
        levels.sort_by_key(|(d, _)| *d);
        levels.dedup_by_key(|(d, _)| *d);
        let ds: Vec<u32> = levels.iter().map(|(d, _)| *d).collect();
        assert_eq!(ds, [1, 2, 3, 4], "{what}: r の行の値\n{s}");
        let ratios: Vec<f64> = levels.iter().map(|(_, bg)| ratio(*bg, screen_bg)).collect();
        for w in ratios.windows(2) {
            assert!(
                w[1] > w[0],
                "{what}: 値の大きいます目ほど画面の地 {screen_bg:?} から遠い: {ratios:?}\n{s}"
            );
        }
    }
}

#[test]
fn test_chart_stacked_series_colors_stand_out_in_every_theme() {
    // [utest -> req~chart.series-contrast~1] 全部のテーマ × 24 ビット・256 色(既定のテーマは暗い端末と明るい端末)で、内訳が
    // 6つ(o1〜o5 と「その他」)の積み上げ棒を描く → 棒のどの字も、字の色と画面の地の対比が 3 以上。
    let mut list: Vec<(String, String)> = Vec::new();
    for (owner, n) in [
        ("o1", 7),
        ("o2", 6),
        ("o3", 5),
        ("o4", 4),
        ("o5", 3),
        ("o6", 2),
        ("o7", 1),
    ] {
        for k in 0..n {
            list.push((format!("{owner}-{k}"), format!("who: me\nowner: {owner}")));
        }
    }
    let notes = md(&refs(&list));
    let line = "chart = { kind = \"stacked\", x = \"who\", by = \"owner\" }";
    for (k, (what, theme, color, term_bg)) in painted().into_iter().enumerate() {
        let (_t, a) = open_painted(&format!("chser{k}"), &notes, line, theme, color, term_bg);
        let c = chart(&a);
        let bar = c.bar("me");
        let (s, buf) = draw(&a);
        let default = term_bg.or(Some([0, 0, 0]));
        // 画面の地: 棒の行の右の端の空いたセル(塗りの字の地は内訳の色へ寄せることがあるので、そこでは読まない)。
        let ground = rgb(buf[(buf.area.width - 3, bar.at.1 + 1)].bg, default).unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for (x, sym) in after_sep(&buf, bar.at.1, bar.at.0) {
            if !"█▓▒░▞:".contains(sym.as_str()) || sym.trim().is_empty() {
                continue;
            }
            let cell = &buf[(x, bar.at.1)];
            let fg = rgb(cell.fg, None)
                .unwrap_or_else(|| panic!("{what}: 棒の字 {sym} に色が無い\n{s}"));
            let r = ratio(fg, ground);
            assert!(
                r >= 3.0,
                "{what}: 棒の字 {sym} の色 {fg:?} と画面の地 {ground:?} の対比 {r:.2}\n{s}"
            );
            seen.insert(sym);
        }
        assert_eq!(
            seen.len(),
            6,
            "{what}: 6つの内訳の塗りが見える: {seen:?}\n{s}"
        );
    }
}
