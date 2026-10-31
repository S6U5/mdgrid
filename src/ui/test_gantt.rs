//! ガントの画面(GA-1〜GA-5・GA-7)。値は作ったもの。今日は 2026-10-06(火)に固定する。
//! 行番号や字の位置は決め打ちせず、行の中身(ノートの名前・`█`・`░`・`◆`・`┊` の数と並び)で確かめる。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, edit_outside, press, text, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::gantt::Unit;
use mdgrid::i18n::Msg;
use mdgrid::settings::{Wbs, WbsValue};
use mdgrid::source::markdown::Markdown;
use mdgrid::types::parse_date;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// 今日(2026-10-06)。
const TODAY: &str = "2026-10-06";

/// GA-2 の例の行: alpha は 10/01〜10/07 の7日、beta は due だけ(10/03)、gamma は 10/05〜10/06、delta は日付なし。
const NOTES: &[(&str, &str)] = &[
    (
        "alpha.md",
        "---\nstart: 2026-10-01\ndue: 2026-10-07\nstatus: todo\n---\n",
    ),
    ("beta.md", "---\ndue: 2026-10-03\nstatus: todo\n---\n"),
    (
        "gamma.md",
        "---\nstart: 2026-10-05\ndue: 2026-10-06\nstatus: done\n---\n",
    ),
    ("delta.md", "---\nstatus: todo\n---\n"),
];

/// GA-3 の例の行: alpha(親。日付なし)の下に beta(10/01〜10/03・done)と gamma(10/05〜10/07・todo)。delta は親なし。
const TREE_NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstatus: doing\n---\n"),
    (
        "beta.md",
        "---\nparent: \"[[alpha]]\"\nstart: 2026-10-01\ndue: 2026-10-03\nstatus: done\nn: 1\n---\n",
    ),
    (
        "gamma.md",
        "---\nparent: \"[[alpha]]\"\nstart: 2026-10-05\ndue: 2026-10-07\nstatus: todo\nn: 2\n---\n",
    ),
    (
        "delta.md",
        "---\nstart: 2026-10-02\ndue: 2026-10-02\nstatus: todo\n---\n",
    ),
];

const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 印(根 = 一時フォルダ)の [[table]] に `line` を書いて、`notes` を表として開く(幅 120、今日は 2026-10-06)。
fn boot_color(tag: &str, notes: &[(&str, &str)], line: &str, color: ColorMode) -> (Tmp, App) {
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
        config: Config::default(),
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

fn boot_with(tag: &str, notes: &[(&str, &str)], line: &str) -> (Tmp, App) {
    boot_color(tag, notes, line, ColorMode::None)
}

fn boot(tag: &str) -> (Tmp, App) {
    boot_with(tag, NOTES, GANTT)
}

/// App の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 画面の行と桁で、`what` の最初の字の位置。
fn find(s: &str, what: &str) -> Option<(u16, u16)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(what).map(|i| (width(&l[..i]) as u16, y as u16)))
}

/// 画面の中の、ノート `name` の行(最初の1行)。
fn row_line(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{s}"))
        .to_string()
}

fn count(line: &str, c: char) -> usize {
    line.chars().filter(|x| *x == c).count()
}

/// 行の中で、桁 `x` から始まる字。
fn char_at(line: &str, x: usize) -> Option<char> {
    let mut col = 0;
    for c in line.chars() {
        if col == x {
            return Some(c);
        }
        col += width(&c.to_string());
    }
    None
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &mdgrid::source::RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// ガントで選んでいる行のノートの名前。
fn sel_name(a: &App) -> String {
    let g = a.gantt.as_ref().expect("ガント");
    lab(a, &g.rows[g.sel])
}

fn tree_on(a: &mut App, wbs: Option<Wbs>) {
    a.settings.tree = Some("parent".into());
    a.settings.wbs = wbs;
    a.settings.sorts = vec![("n".into(), mdgrid::settings::Dir::Asc)];
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

/// 画面の型のメニューを開いて描いた文字(閉じて戻す)。
fn menu_text(a: &mut App) -> String {
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu, "{:?}", a.message);
    let s = screen(a);
    a.close_screen_menu();
    s
}

#[test]
fn test_ga_1_c_opens_gantt_and_menu_offers_it() {
    // [GA-1] gantt = { start, end } の表 → C でガントの画面(題に列の名前)。C でもう一度押すと表。
    // ヘッダーの「▦ 表 ▾」の窓に「ガント」と切り替えのキー C が並び、選ぶとガント。
    let (_t, mut a) = boot("ga1c");
    assert_eq!(a.message, None, "警告が無い");
    assert!(a.gantt_ready());
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let g = a.gantt.as_ref().expect("ガント");
    assert!(!g.inferred);
    let s = screen(&a);
    assert!(s.contains("ガント ▾"), "{s}");
    assert!(
        s.contains("start") && s.contains("due"),
        "題に列の名前:\n{s}"
    );
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.gantt.is_none());
    let s = screen(&a);
    let (x, y) = find(&s, "▦ 表 ▾").expect("ヘッダーのメニュー");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::ScreenMenu);
    let s = screen(&a);
    let (x, y) = s
        .lines()
        .enumerate()
        .skip(y as usize + 1)
        .find_map(|(yy, l)| {
            l.find(" ガント")
                .filter(|_| {
                    l.trim_end()
                        .trim_end_matches(['│', '|'])
                        .trim_end()
                        .ends_with('C')
                })
                .map(|i| (width(&l[..i]) as u16 + 1, yy as u16))
        })
        .unwrap_or_else(|| panic!("窓に「ガント」と切り替えのキー C:\n{s}"));
    a.click(x, y);
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert!(a.gantt.is_some());
}

#[test]
fn test_ga_1_infers_from_date_columns_and_says_so() {
    // [GA-1] 設定なしで start と due の日付の列がある → 推定して開き、題に「推定」と推定した列を示す。
    let (_t, mut a) = boot_with("ga1inf", NOTES, "");
    assert_eq!(a.message, None, "{:?}", a.message);
    assert!(a.gantt_ready());
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert!(a.gantt.as_ref().unwrap().inferred);
    let s = screen(&a);
    assert!(s.contains(Msg::GanttInferred.text()), "{s}");
    assert!(s.contains("start") && s.contains("due"), "{s}");
    // 推定でも棒は出る。
    assert_eq!(count(&row_line(&s, "alpha"), '█'), 7, "{s}");
}

#[test]
fn test_ga_1_no_date_columns_gives_reason() {
    // [GA-1] 設定なしで日付の列が無い → 理由1行で開かない。窓に「ガント」は出ない。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nstatus: todo\ntitle: x\n---\n"),
        ("b.md", "---\nstatus: done\ntitle: y\n---\n"),
    ];
    let (_t, mut a) = boot_with("ga1none", notes, "");
    assert!(!a.gantt_ready());
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.gantt.is_none());
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains(Msg::GanttNotSet.text()), "{m}");
    assert!(!menu_text(&mut a).contains("ガント"));
}

#[test]
fn test_ga_1_missing_column_warns_and_no_gantt() {
    // [GA-1] 設定の start の列が表に無い → 読み終えたときに CLI-3 の形の警告で、ガントを出さず、窓にも出ない。
    let (_t, mut a) = boot_with(
        "ga1col",
        NOTES,
        "gantt = { start = \"none\", end = \"due\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].gantt") && m.contains("none"),
        "{m}"
    );
    assert!(!a.gantt_ready());
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.gantt.is_none());
    assert!(!menu_text(&mut a).contains("ガント"));
}

#[test]
fn test_ga_2_bars_points_today_and_no_color() {
    // [GA-2] 10/01〜10/07 の行は日の単位で `█` が7つ。due だけの行は `◆` が1つ。日付の無い行は棒なしで、
    // 今日(10/06)の列に `┊`。今日の列は alpha の棒の中(`█`)。◆(10/03)は今日の3桁左。色ありでも同じ字。
    let (_t, mut a) = boot("ga2");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Day);
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    assert_eq!(count(&alpha, '█'), 7, "{s}");
    let beta = row_line(&s, "beta");
    assert_eq!(count(&beta, '◆'), 1, "{s}");
    assert_eq!(count(&beta, '█'), 0, "{s}");
    let gamma = row_line(&s, "gamma");
    assert_eq!(count(&gamma, '█'), 2, "{s}");
    let delta = row_line(&s, "delta");
    assert_eq!(
        count(&delta, '█') + count(&delta, '◆') + count(&delta, '░'),
        0,
        "{s}"
    );
    let today_x = delta
        .find('┊')
        .map(|i| width(&delta[..i]))
        .unwrap_or_else(|| panic!("今日の列の印 ┊:\n{s}"));
    assert_eq!(
        char_at(&alpha, today_x),
        Some('█'),
        "今日は alpha の棒の中:\n{s}"
    );
    let point_x = beta.find('◆').map(|i| width(&beta[..i])).unwrap();
    assert_eq!(today_x, point_x + 3, "◆ は 10/03、今日は 10/06:\n{s}");
    // 選んでいる行(先頭)は印で見分けられる(色なし)。
    assert!(
        s.lines().any(|l| l.contains("alpha") && l.contains('>')),
        "{s}"
    );
    // 色ありでも同じ字。
    let (_t2, mut b) = boot_color("ga2rgb", NOTES, GANTT, ColorMode::Rgb);
    ch(&mut b, 'C');
    let s2 = screen(&b);
    for name in ["alpha", "beta", "gamma", "delta"] {
        assert_eq!(row_line(&s2, name), row_line(&s, name));
    }
}

#[test]
fn test_ga_2_wide_range_opens_coarser_unit() {
    // [GA-2] 幅 100 で範囲 400 日 → 日では入らないので、週か月の単位で開く。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\nstart: 2026-01-01\ndue: 2027-02-04\n---\n"),
        ("b.md", "---\nstart: 2026-06-01\ndue: 2026-06-30\n---\n"),
    ];
    let (_t, mut a) = boot_with("ga2wide", notes, GANTT);
    a.resize(100, 30);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let unit = a.gantt.as_ref().unwrap().unit;
    assert!(matches!(unit, Unit::Week | Unit::Month), "{unit:?}");
    // 棒は幅に収まる(行は幅を超えない)。
    for l in super::view::render(&a, 99, 29) {
        let lw: usize = l.spans.iter().map(|s| width(&s.content)).sum();
        assert!(lw <= 99, "{lw}: {l:?}");
    }
}

#[test]
fn test_ga_3_parent_bar_spans_children_and_wbs_fills() {
    // [GA-3] 親子(parent)で、親に日付が無く子が 10/01〜10/03 と 10/05〜10/07 → 親の棒は 10/01〜10/07 の7マス。
    // WBS で子の割合が 100 と 0 → 親の棒の塗り(`█`)が半分、残り(`░`)が半分。行の並びと番号は表と同じ。
    let (_t, mut a) = boot_with("ga3", TREE_NOTES, GANTT);
    tree_on(&mut a, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    assert_eq!(count(&alpha, '█') + count(&alpha, '░'), 7, "{s}");
    assert_eq!(count(&row_line(&s, "beta"), '█'), 3, "{s}");
    assert_eq!(count(&row_line(&s, "gamma"), '█'), 3, "{s}");
    let ys: Vec<u16> = ["alpha", "beta", "gamma"]
        .iter()
        .map(|n| find(&s, n).unwrap().1)
        .collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "{ys:?}\n{s}");
    // WBS をオン。
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
    tree_on(&mut a, Some(wbs()));
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    let (full, rest) = (count(&alpha, '█'), count(&alpha, '░'));
    assert!(
        (3..=4).contains(&full) && (3..=4).contains(&rest) && full + rest == 7,
        "{s}"
    );
    let beta = row_line(&s, "beta");
    assert_eq!(count(&beta, '█') + count(&beta, '░'), 3, "{s}");
    assert!(beta.contains("1.1"), "番号:\n{s}");
    assert!(row_line(&s, "gamma").contains("1.2"), "番号:\n{s}");
}

#[test]
fn test_ga_3_z_folds_children() {
    // [GA-3] 親の行で Z → 子の行が消える(表と同じ畳み)。もう一度 Z → 戻る。
    let (_t, mut a) = boot_with("ga3z", TREE_NOTES, GANTT);
    tree_on(&mut a, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    for _ in 0..8 {
        press(&mut a, KeyCode::Up);
    }
    assert_eq!(sel_name(&a), "alpha");
    ch(&mut a, 'Z');
    assert_eq!(a.mode, Mode::Gantt);
    let s = screen(&a);
    assert!(s.contains("alpha") && s.contains("delta"), "{s}");
    assert!(!s.contains("beta") && !s.contains("gamma"), "畳む:\n{s}");
    ch(&mut a, 'Z');
    let s = screen(&a);
    assert!(s.contains("beta") && s.contains("gamma"), "開く:\n{s}");
}

#[test]
fn test_ga_4_keys_select_scroll_zoom_today_and_back() {
    // [GA-4] j j → Enter で表の3行目。l ×3 で時間軸が3単位進み、h で1つ戻る。- で日 → 週 → 月、+ で戻る。
    // t で今日が見える位置に戻る。Esc・q・C で表に戻る。
    let (_t, mut a) = boot("ga4");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, 'j');
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.gantt.is_none());
    assert_eq!(a.cur_row(), Some(a.rows[2].clone()));
    // 時間軸。
    ch(&mut a, 'C');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Day);
    let origin = a.gantt.as_ref().unwrap().origin;
    for _ in 0..3 {
        ch(&mut a, 'l');
    }
    assert_eq!(a.gantt.as_ref().unwrap().origin, origin + 3);
    ch(&mut a, 'h');
    assert_eq!(a.gantt.as_ref().unwrap().origin, origin + 2);
    press(&mut a, KeyCode::Right);
    assert_eq!(a.gantt.as_ref().unwrap().origin, origin + 3);
    // 単位。
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Week);
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Month);
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Month);
    ch(&mut a, '+');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Week);
    ch(&mut a, '+');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Day);
    ch(&mut a, '+');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Day);
    // 今日。
    for _ in 0..60 {
        ch(&mut a, 'l');
    }
    assert!(!screen(&a).contains('┊'), "今日が見えない:\n{}", screen(&a));
    ch(&mut a, 't');
    let s = screen(&a);
    assert!(s.contains('┊'), "今日に戻る:\n{s}");
    assert!(a.gantt.as_ref().unwrap().origin <= a.today);
    assert_eq!(a.mode, Mode::Gantt);
    // 戻るキー。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.gantt.is_none());
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt);
    ch(&mut a, 'q');
    assert_eq!(a.mode, Mode::Table);
    assert!(!a.quit);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_ga_4_mouse_click_and_wheel() {
    // [GA-4] 行のクリックで選ぶ。選んでいる行をもう一度クリック → 表のその行。ホイールで行、横のホイールで時間軸。
    let (_t, mut a) = boot("ga4mouse");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    let (x, y) = find(&s, "gamma").unwrap_or_else(|| panic!("gamma:\n{s}"));
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Gantt);
    assert_eq!(sel_name(&a), "gamma");
    a.click(x + 1, y);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(lab(&a, &a.cur_row().unwrap()), "gamma");
    // ホイール。
    ch(&mut a, 'C');
    // 見本の行は名前の順(alpha・beta・delta・gamma)で gamma が最後なので、先に上へ。
    let before = a.gantt.as_ref().unwrap().sel;
    a.wheel(false);
    assert_eq!(a.gantt.as_ref().unwrap().sel, before - 1);
    a.wheel(true);
    assert_eq!(a.gantt.as_ref().unwrap().sel, before);
    let origin = a.gantt.as_ref().unwrap().origin;
    a.hwheel(true);
    assert_eq!(a.gantt.as_ref().unwrap().origin, origin + 1);
    a.hwheel(false);
    assert_eq!(a.gantt.as_ref().unwrap().origin, origin);
    assert_eq!(a.mode, Mode::Gantt);
}

#[test]
fn test_ga_5_read_only_and_rebuilds_on_external_change() {
    // [GA-5] ガントでは値を直せない: 編集のキー(e・Space・Backspace・Delete・文字)を押しても、ためた変更は無く
    // エディタも開かない。開いたまま外で alpha の due を 10/04 に直す → poll → 棒が4つになり、選んでいた行は同じ。
    let (t, mut a) = boot("ga5");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, 'j');
    assert_eq!(sel_name(&a), "beta");
    for code in [
        KeyCode::Char('e'),
        KeyCode::Char(' '),
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Char('i'),
        KeyCode::Char('1'),
        KeyCode::Char('u'),
    ] {
        press(&mut a, code);
        assert_eq!(a.changes.count(), 0, "{code:?}");
        assert!(a.editor_request.is_none(), "{code:?}");
        assert_eq!(a.mode, Mode::Gantt, "{code:?}");
    }
    assert_eq!(sel_name(&a), "beta");
    assert_eq!(count(&row_line(&screen(&a), "alpha"), '█'), 7);
    edit_outside(
        &t,
        "alpha.md",
        "---\nstart: 2026-10-01\ndue: 2026-10-04\nstatus: todo\n---\n",
    );
    a.poll();
    assert_eq!(a.mode, Mode::Gantt);
    let s = screen(&a);
    assert_eq!(count(&row_line(&s, "alpha"), '█'), 4, "{s}");
    assert_eq!(sel_name(&a), "beta");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ga_7_menu_lists_only_openable_types() {
    // [GA-7] 印に tree と gantt の両方がある表 → 窓に 表・木・ガント。どちらも無く日付の列が start・due の表 →
    // 表・ガント(推定)で、木は出ない。
    let notes: &[(&str, &str)] = &[
        (
            "a.md",
            "---\nfolder: x > y\nstart: 2026-10-01\ndue: 2026-10-03\n---\n",
        ),
        ("b.md", "---\nfolder: x\ndue: 2026-10-05\n---\n"),
    ];
    let (_t, mut a) = boot_with(
        "ga7both",
        notes,
        "tree = { path = \"folder\" }\ngantt = { start = \"start\", end = \"due\" }",
    );
    assert_eq!(a.message, None, "{:?}", a.message);
    assert!(a.tree_ready() && a.gantt_ready());
    let s = menu_text(&mut a);
    assert!(
        s.contains("表") && s.contains("木") && s.contains("ガント"),
        "{s}"
    );
    let (_t2, mut b) = boot_with("ga7infer", notes, "");
    assert!(!b.tree_ready() && b.gantt_ready());
    let s = menu_text(&mut b);
    assert!(s.contains("表") && s.contains("ガント"), "{s}");
    assert!(!s.contains('木'), "{s}");
    ch(&mut b, 'C');
    assert_eq!(b.mode, Mode::Gantt, "{:?}", b.message);
    assert!(b.gantt.as_ref().unwrap().inferred);
}
