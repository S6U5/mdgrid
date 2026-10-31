//! ガントの画面の追加の試験(GA-1〜GA-4。レビューの指摘から足した。錠のある `test_gantt.rs` は変えない):
//! 流れた状態のクリック、時間軸の見出し(原点が月末・週と月の単位)、塗りが棒の始まりから数えられること、
//! `days` の列と日時の値、`Z` のあとの選び、無い列の警告の場所、題の無い列、`days` の巨大な値。
//! 今日は 2026-10-06(火)に固定する。

use super::app::{App, ColorMode};
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, press, text, Tmp};
use mdgrid::config::Config;
use mdgrid::gantt::{Bar, Unit};
use mdgrid::settings::{Wbs, WbsValue};
use mdgrid::source::markdown::Markdown;
use mdgrid::types::parse_date;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

const TODAY: &str = "2026-10-06";
const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

/// 印(根 = 一時フォルダ)の [[table]] に `line` を書いて、`notes` を表として開く(今日は 2026-10-06)。
fn boot_size(tag: &str, notes: &[(String, String)], line: &str, size: (u16, u16)) -> (Tmp, App) {
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
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(size.0, size.1);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
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

fn notes(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(n, t)| (n.to_string(), t.to_string()))
        .collect()
}

fn boot(tag: &str, list: &[(&str, &str)], line: &str) -> (Tmp, App) {
    boot_size(tag, &notes(list), line, (120, 30))
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

fn row_line(s: &str, name: &str) -> String {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{s}"))
        .to_string()
}

fn count(line: &str, c: char) -> usize {
    line.chars().filter(|x| *x == c).count()
}

/// `pat` の最初の位置(字の数で)。
fn cpos(s: &str, pat: &str) -> Option<usize> {
    s.find(pat).map(|i| s[..i].chars().count())
}

/// 表で選んでいる行のノートの名前。
fn lab(a: &App) -> String {
    let r = a.cur_row().expect("表の行");
    a.src.label(&r).trim_end_matches(".md").to_string()
}

fn sel_name(a: &App) -> String {
    let g = a.gantt.as_ref().expect("ガント");
    a.src
        .label(&g.rows[g.sel])
        .trim_end_matches(".md")
        .to_string()
}

/// 時間軸の見出しの2行(窓の題の次の2行)。
fn header(s: &str) -> (String, String) {
    let lines: Vec<&str> = s.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.contains("ガント:"))
        .unwrap_or_else(|| panic!("題:\n{s}"));
    (lines[i + 1].to_string(), lines[i + 2].to_string())
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

/// 親 alpha(日付なし)の下に beta(10/01〜10/03・done)と gamma(10/05〜10/07・todo)、親なしの delta。
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

#[test]
fn test_ga_4_click_selects_the_clicked_row_when_scrolled() {
    // [GA-4] 行が窓より多く、下まで動かして流れた状態でも、クリックした行(その行そのもの)を選ぶ。
    // 枠の下の縁の行をクリックしても選びは変わらない。
    let many: Vec<(String, String)> = (0..30)
        .map(|i| {
            (
                format!("n{i:02}.md"),
                "---\nstart: 2026-10-01\ndue: 2026-10-03\n---\n".to_string(),
            )
        })
        .collect();
    let (_t, mut a) = boot_size("ga4scroll", &many, GANTT, (100, 16));
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    for _ in 0..40 {
        ch(&mut a, 'j');
    }
    assert_eq!(sel_name(&a), "n29");
    let s = screen(&a);
    // 見えている行のうち、選んでいる行の2つ上をクリック。
    let (title_y, _) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("ガント:"))
        .map(|(y, l)| (y, l.to_string()))
        .unwrap();
    let shown: Vec<(usize, String)> = s
        .lines()
        .enumerate()
        .skip(title_y + 3)
        .filter(|(_, l)| l.contains("n2") || l.contains("n1") || l.contains("n0"))
        .map(|(y, l)| (y, l.to_string()))
        .collect();
    assert!(shown.len() >= 3, "{s}");
    let (y, line) = &shown[shown.len() - 3];
    let name = line
        .split_whitespace()
        .find(|w| w.starts_with('n') && w.len() == 3)
        .unwrap()
        .to_string();
    a.click(4, u16::try_from(*y).unwrap());
    assert_eq!(a.mode, Mode::Gantt);
    assert_eq!(sel_name(&a), name, "{s}");
    // 枠の下の縁(最後の行の次)。
    let last_y = shown[shown.len() - 1].0;
    let before = sel_name(&a);
    a.click(4, u16::try_from(last_y + 1).unwrap());
    assert_eq!(a.mode, Mode::Gantt);
    assert_eq!(sel_name(&a), before);
}

#[test]
fn test_ga_2_header_labels_do_not_swallow_the_next_month() {
    // [GA-2] 原点が 10/27(月末の6日以内)でも、11月の字「2026-11」が出る。先頭は収まらないので「10」だけ。
    // 週の単位: 上の行は年、下の行は1日を含む週に月の数(11・12・01)。月の単位: 下の行は月の一の位が並び、
    // 1月のマスに年。
    let list = [("a.md", "---\nstart: 2026-10-27\ndue: 2026-11-10\n---\n")];
    let (_t, mut a) = boot("ga2hdr", &list, GANTT);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Day);
    let (top, low) = header(&screen(&a));
    assert!(top.contains("2026-11"), "{top}");
    assert!(!top.contains("2026-10"), "{top}");
    let i10 = cpos(&top, "10").unwrap();
    let i11 = cpos(&top, "2026-11").unwrap();
    assert_eq!(i11 - i10, 5, "{top}");
    assert!(low.contains("7890123"), "{low}");
    // 週。
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Week);
    let (top, low) = header(&screen(&a));
    assert!(top.contains("2026") && top.contains("2027"), "{top}");
    assert!(!top.contains("2026-"), "{top}");
    for m in ["11", "12", "01"] {
        assert!(low.contains(m), "{m}: {low}");
    }
    // 月(今日 10/06 のマスは ▾ なので、10・11・12・1… は「▾12123…」)。1月のマスの上に年。
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Month);
    let (top, low) = header(&screen(&a));
    assert!(low.contains("▾12123456789012"), "{low}");
    assert!(top.contains("2027"), "{top}");
    let i_jan = cpos(&low, "▾121").unwrap() + 3;
    assert_eq!(cpos(&top, "2027"), Some(i_jan), "{top}\n{low}");
}

#[test]
fn test_ga_2_week_today_keeps_the_month_digits() {
    // [GA-2] 週の単位で、今日(10/06)の週が 10/01 を含む週の次の週 → 月の数「10」の2字目に今日が重なるが、
    // 数は消えず(「10」が残り ▾ は出ない)、そのマスは反転で示す。
    let list = [("a.md", "---\nstart: 2026-09-28\ndue: 2026-10-20\n---\n")];
    let (_t, mut a) = boot("ga2wk", &list, GANTT);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Week);
    let s = screen(&a);
    let (_, low) = header(&s);
    assert!(low.contains("10"), "{low}");
    assert!(!low.contains('▾'), "{low}");
    // 反転のマス: 「10」の 0 の位置。
    let y = s.lines().position(|l| l == low).unwrap();
    let x = cpos(&low, "10").unwrap() + 1;
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, &a)).unwrap();
    let cell = t
        .backend()
        .buffer()
        .cell((u16::try_from(x).unwrap(), u16::try_from(y).unwrap()))
        .cloned()
        .unwrap();
    assert_eq!(cell.symbol(), "0", "{low}");
    assert!(
        cell.modifier.contains(ratatui::style::Modifier::REVERSED),
        "{:?}",
        cell
    );
    // 今日の週でない月の数は反転しない。
    let x11 = cpos(&low, "11").unwrap();
    let cell = t
        .backend()
        .buffer()
        .cell((u16::try_from(x11).unwrap(), u16::try_from(y).unwrap()))
        .cloned()
        .unwrap();
    assert!(!cell.modifier.contains(ratatui::style::Modifier::REVERSED));
}

#[test]
fn test_ga_3_fill_counts_from_the_bar_start_when_scrolled_off() {
    // [GA-3] 親の棒(10/01〜10/07、塗りは半分 = 4マス)の左が画面の外に出ても、塗りは棒の始まりから数える:
    // 原点を 10/05 にすると見えている3マスは全部 `░`。
    let (_t, mut a) = boot("ga3left", TREE_NOTES, GANTT);
    tree_on(&mut a, Some(wbs()));
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    assert_eq!(count(&alpha, '█'), 4, "{s}");
    assert_eq!(count(&alpha, '░'), 3, "{s}");
    for _ in 0..4 {
        ch(&mut a, 'l');
    }
    let s = screen(&a);
    let alpha = row_line(&s, "alpha");
    assert_eq!(count(&alpha, '█'), 0, "{s}");
    assert_eq!(count(&alpha, '░'), 3, "{s}");
}

#[test]
fn test_ga_1_days_column_and_datetime_values() {
    // [GA-1] gantt = { start, days } と、日時の値 → 日の部分で棒。3日なら `█` が3つ。題は「start → days」。
    let list = [
        ("zeta.md", "---\nstart: 2026-10-01T09:30\ndays: 3\n---\n"),
        ("yota.md", "---\nstart: 2026-10-05\ndays: 1\n---\n"),
    ];
    let (_t, mut a) = boot(
        "ga1days",
        &list,
        "gantt = { start = \"start\", days = \"days\" }",
    );
    assert_eq!(a.message, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    assert_eq!(count(&row_line(&s, "zeta"), '█'), 3, "{s}");
    assert_eq!(count(&row_line(&s, "yota"), '█'), 1, "{s}");
    assert!(s.contains("start → days"), "{s}");
}

#[test]
fn test_ga_3_z_keeps_the_selected_row() {
    // [GA-3] 親で Z → 子が消え、選びは親のまま。j で次の見える行(delta)。子の無い行で Z は理由。もう一度親で Z → 戻る。
    let (_t, mut a) = boot("ga3zsel", TREE_NOTES, GANTT);
    tree_on(&mut a, None);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(sel_name(&a), "alpha");
    ch(&mut a, 'Z');
    assert_eq!(sel_name(&a), "alpha");
    assert_eq!(a.gantt.as_ref().unwrap().rows.len(), 2);
    ch(&mut a, 'j');
    assert_eq!(sel_name(&a), "delta");
    ch(&mut a, 'Z');
    assert_eq!(sel_name(&a), "delta");
    assert_eq!(
        a.message.as_deref(),
        Some(mdgrid::i18n::Msg::TreeNoKids.text())
    );
    ch(&mut a, 'k');
    ch(&mut a, 'Z');
    assert_eq!(sel_name(&a), "alpha");
    assert_eq!(a.gantt.as_ref().unwrap().rows.len(), 4);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ga_3_z_keeps_the_table_row_by_key() {
    // [GA-3] 表で delta(最後の行)を選んでからガントで alpha を畳んで Esc → 表の行は delta のまま(添字でなく鍵で保つ)。
    // 表で gamma(alpha の子)を選んでからガントで alpha を畳んで Esc → 表の行は畳んだ親の alpha。
    let (_t, mut a) = boot("ga3zrow", TREE_NOTES, GANTT);
    tree_on(&mut a, None);
    ch(&mut a, 'G');
    assert_eq!(lab(&a), "delta");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    assert_eq!(sel_name(&a), "delta");
    for _ in 0..3 {
        ch(&mut a, 'k');
    }
    assert_eq!(sel_name(&a), "alpha");
    ch(&mut a, 'Z');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(lab(&a), "delta");
    // 親を開き直してから、表で子を選んだ状態で畳む。
    ch(&mut a, 'C');
    ch(&mut a, 'k');
    assert_eq!(sel_name(&a), "alpha");
    ch(&mut a, 'Z');
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    ch(&mut a, 'j');
    ch(&mut a, 'j');
    assert_eq!(lab(&a), "gamma");
    ch(&mut a, 'C');
    for _ in 0..2 {
        ch(&mut a, 'k');
    }
    assert_eq!(sel_name(&a), "alpha");
    ch(&mut a, 'Z');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(lab(&a), "alpha");
}

#[test]
fn test_ga_1_missing_column_warning_names_the_key() {
    // [GA-1] 無い列が end なら、警告の場所は `table[1].gantt.end`(start ではない)。
    let list = [("a.md", "---\nstart: 2026-10-01\n---\n")];
    let (_t, a) = boot(
        "ga1key",
        &list,
        "gantt = { start = \"start\", end = \"none\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("table[1].gantt.end") && m.contains("none"),
        "{m}"
    );
    assert!(!m.contains("gantt.start"), "{m}");
}

#[test]
fn test_ga_7_title_shows_dash_for_missing_column() {
    // [GA-7] 推定で終わりの列だけ → 題は「- → due」。
    let list = [("a.md", "---\ndue: 2026-10-03\n---\n")];
    let (_t, mut a) = boot("ga7dash", &list, "");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("- → due"), "{s}");
}

#[test]
fn test_ga_1_huge_days_do_not_overflow() {
    // [GA-1] days の巨大な値でも溢れず、日付の範囲(0000-01-01〜9999-12-31)に収まる(年が負にならない)。
    let first = parse_date("0000-01-01").unwrap();
    let last = parse_date("9999-12-31").unwrap();
    let b = Bar::from_values(Some(0), None, Some(i64::MAX));
    assert_eq!(b.start, Some(0));
    assert_eq!(b.end, Some(last));
    let b = Bar::from_values(None, Some(0), Some(i64::MAX));
    assert_eq!(b.start, Some(first));
    assert_eq!(b.end, Some(0));
    // 今日から 100 万日(約 2700 年)前は年が負になるので、0000-01-01 で止まる(年は負にならない)。
    let b = Bar::from_values(None, Some(parse_date(TODAY).unwrap()), Some(1_000_000));
    assert_eq!(b.start, Some(first));
    assert_eq!(mdgrid::gantt::civil(b.start.unwrap()).0, 0);
}

#[test]
fn test_ga_4_palette_action_applies_to_the_selected_row() {
    // [GA-4] ガントで j で beta を選び、パレットから表の動作(行に印)を選ぶ → 表に戻って beta が選ばれ、印は beta に付く
    // (表で選んでいた alpha ではない)。
    let (_t, mut a) = boot("ga4pal", TREE_NOTES, GANTT);
    assert_eq!(lab(&a), "alpha");
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, 'j');
    assert_eq!(sel_name(&a), "beta");
    let beta = a.gantt.as_ref().map(|g| g.rows[g.sel].clone()).unwrap();
    a.gantt_action(Action::Mark);
    assert_eq!(a.mode, Mode::Table);
    // 印は beta に付く(印のあとは次の行へ動くので、表の行は見ない)。
    assert!(a.marked.contains(&beta), "{:?}", a.marked);
    assert_eq!(a.marked.len(), 1);
}

#[test]
fn test_ga_4_scroll_past_the_last_year_keeps_origin_on_a_boundary() {
    // [GA-4] 月の単位で 9999-12 を越えて l を押しても、原点は月の1日のまま(見出しの切れ目と h の戻りがずれない)。
    let list = [("a.md", "---\nstart: 9999-10-01\ndue: 9999-11-15\n---\n")];
    let (_t, mut a) = boot("ga4edge", &list, GANTT);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, '-');
    ch(&mut a, '-');
    assert_eq!(a.gantt.as_ref().unwrap().unit, Unit::Month);
    for _ in 0..5 {
        ch(&mut a, 'l');
        let o = a.gantt.as_ref().unwrap().origin;
        assert_eq!(
            mdgrid::gantt::civil(o).2,
            1,
            "{}",
            mdgrid::types::format_date(o)
        );
    }
    assert_eq!(
        a.gantt.as_ref().unwrap().origin,
        parse_date("9999-12-01").unwrap()
    );
    ch(&mut a, 'h');
    assert_eq!(
        a.gantt.as_ref().unwrap().origin,
        parse_date("9999-11-01").unwrap()
    );
    let _ = screen(&a);
}
