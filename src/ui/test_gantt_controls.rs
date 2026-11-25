//! ガントの操作の帯・期間・棒を動かすキー(gantt.controls-bar・controls-click・range-presets・range-key・
//! edit-move・edit-resize・edit-refuse・edit-undo)。値は作ったもの。今日は 2026-10-06(火)に固定する。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, text, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::gantt::Unit;
use mdgrid::source::markdown::Markdown;
use mdgrid::types::{format_date, parse_date};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;

/// alpha は 10/01〜10/07、beta は due だけ(10/03)、gamma は 10/05〜10/06、delta は日付なし。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\nstart: 2026-10-01\ndue: 2026-10-07\n---\n"),
    ("beta.md", "---\ndue: 2026-10-03\n---\n"),
    ("gamma.md", "---\nstart: 2026-10-05\ndue: 2026-10-06\n---\n"),
    ("delta.md", "---\nstatus: todo\n---\n"),
];

const GANTT: &str = "gantt = { start = \"start\", end = \"due\" }";

fn boot_opts(tag: &str, notes: &[(&str, &str)], line: &str, readonly: bool) -> (Tmp, App) {
    boot_cfg(tag, notes, line, readonly, Config::default())
}

/// `boot_opts` と同じで、設定を `config` にする。
fn boot_cfg(
    tag: &str,
    notes: &[(&str, &str)],
    line: &str,
    readonly: bool,
    config: Config,
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
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = parse_date("2026-10-06").unwrap();
    (tmp, app)
}

fn boot_with(tag: &str, notes: &[(&str, &str)], line: &str) -> (Tmp, App) {
    boot_opts(tag, notes, line, false)
}

/// 例の4行でガントを開く(alpha を選んでいる)。
fn open(tag: &str) -> (Tmp, App) {
    let (t, mut a) = boot_with(tag, NOTES, GANTT);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    (t, a)
}

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// 操作の帯の行(`◀` のある行)。
fn bar(a: &App) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.contains('◀'))
        .unwrap_or_else(|| panic!("操作の帯が無い:\n{s}"))
        .to_string()
}

/// 画面の最初の `what` をクリックする。
fn click(a: &mut App, what: &str) {
    let s = screen(a);
    let (x, y) = s
        .lines()
        .enumerate()
        .find_map(|(y, l)| l.find(what).map(|i| (width(&l[..i]), y)))
        .unwrap_or_else(|| panic!("画面に「{what}」が無い:\n{s}"));
    a.click(x as u16, y as u16);
}

fn origin(a: &App) -> String {
    format_date(a.gantt.as_ref().unwrap().origin)
}

fn unit(a: &App) -> Unit {
    a.gantt.as_ref().unwrap().unit
}

/// 行(名前に `name` を含む)の列の今の値(ためた値を重ねたもの)。
fn val(a: &App, name: &str, col: &str) -> String {
    let row = a
        .rows
        .iter()
        .find(|r| a.src.label(r).contains(name))
        .unwrap_or_else(|| panic!("{name} の行が無い"));
    a.plain(row, col)
}

/// ガントで、名前に `name` を含む行を選ぶ。
fn select(a: &mut App, name: &str) {
    for _ in 0..10 {
        let g = a.gantt.as_ref().unwrap();
        if a.src.label(&g.rows[g.sel]).contains(name) {
            return;
        }
        ch(a, 'j');
    }
    panic!("{name} を選べない");
}

// [utest -> req~gantt.controls-bar~1]
/// 窓の下の縁に、操作の帯の項目が決まった順に並ぶ。
#[test]
fn test_controls_bar_lists_items_in_order() {
    let (_t, a) = open("gc_bar1");
    let line = bar(&a);
    let mut at = 0;
    for item in [
        "◀",
        "▶",
        "[日]",
        "週",
        "月",
        "今日",
        "今週",
        "今月",
        "四半期",
        "全体",
    ] {
        let i = line[at..]
            .find(item)
            .unwrap_or_else(|| panic!("{item} が順に無い: {line}"));
        at += i + item.len();
    }
}

// [utest -> req~gantt.controls-bar~1]
/// 単位を変えると、帯の今の単位の囲みが移る。
#[test]
fn test_controls_bar_marks_current_unit() {
    let (_t, mut a) = open("gc_bar2");
    ch(&mut a, '-');
    let line = bar(&a);
    assert!(line.contains("[週]"), "{line}");
    assert!(!line.contains("[日]"), "{line}");
}

// [utest -> req~gantt.controls-click~1]
/// 帯の `週` のクリックで、単位が週になる。
#[test]
fn test_click_unit_sets_unit() {
    let (_t, mut a) = open("gc_click1");
    assert_eq!(unit(&a), Unit::Day);
    click(&mut a, "週");
    assert_eq!(unit(&a), Unit::Week);
    assert!(bar(&a).contains("[週]"));
}

// [utest -> req~gantt.controls-click~1]
/// 帯の `▶` のクリックで、時間軸が1単位進む。
#[test]
fn test_click_arrow_pans() {
    let (_t, mut a) = open("gc_click2");
    assert_eq!(origin(&a), "2026-10-01");
    click(&mut a, "▶");
    assert_eq!(origin(&a), "2026-10-02");
}

// [utest -> req~gantt.controls-click~1]
/// 帯の `今月` のクリックで、今月の1日が左端になる。
#[test]
fn test_click_this_month_sets_range() {
    let (_t, mut a) = open("gc_click3");
    ch(&mut a, 'l');
    ch(&mut a, 'l');
    click(&mut a, "今月");
    assert_eq!(origin(&a), "2026-10-01");
}

// [utest -> req~gantt.range-presets~2, req~gantt.range-key~2]
/// `r` は、今週(週の始まりの日から日の単位。週の始まりを書いていないので日曜)→ 今月(1日から)→ 四半期 → 全体 →
/// 今週 の順に切り替える。
#[test]
fn test_r_cycles_ranges() {
    let (_t, mut a) = open("gc_range1");
    ch(&mut a, 'r');
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-10-04", Unit::Day));
    ch(&mut a, 'r');
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-10-01", Unit::Day));
    ch(&mut a, 'r');
    ch(&mut a, 'r');
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-10-01", Unit::Day));
    ch(&mut a, 'r');
    assert_eq!(origin(&a), "2026-10-04");
}

// [utest -> req~gantt.range-presets~2]
/// `[dates] week_start = "mon"` なら、今週の左端は月曜。
#[test]
fn test_this_week_starts_on_monday_when_set() {
    let (cfg, w) = mdgrid::config::parse("[dates]\nweek_start = \"mon\"\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (_t, mut a) = boot_cfg("gc_range_mon", NOTES, GANTT, false, cfg);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    click(&mut a, "今週");
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-10-05", Unit::Day));
}

// [utest -> req~gantt.week-start~1]
/// 10/04(日)〜10/10(土)の棒は、週の単位で、週の始まりを書いていなければ(日曜)1マス、`"mon"` なら2マス。
#[test]
fn test_week_unit_cells_follow_week_start() {
    let notes: &[(&str, &str)] = &[("alpha.md", "---\nstart: 2026-10-04\ndue: 2026-10-10\n---\n")];
    let (mon, w) = mdgrid::config::parse("[dates]\nweek_start = \"mon\"\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    for (tag, cfg, want) in [
        ("gc_week_sun", Config::default(), 1),
        ("gc_week_mon", mon, 2),
    ] {
        let (_t, mut a) = boot_cfg(tag, notes, GANTT, false, cfg);
        ch(&mut a, 'C');
        assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
        ch(&mut a, '-');
        assert_eq!(unit(&a), Unit::Week);
        let s = screen(&a);
        let line = s
            .lines()
            .find(|l| l.contains("alpha"))
            .unwrap_or_else(|| panic!("alpha の行が無い:\n{s}"));
        assert_eq!(
            line.chars().filter(|c| *c == '█').count(),
            want,
            "{tag}:\n{s}"
        );
    }
}

// [utest -> req~gantt.range-presets~2]
/// 四半期は、四半期の最初の日を含む単位の境を左端に、末日まで収まる最も細かい単位にする。
#[test]
fn test_quarter_range_fits_unit() {
    let (_t, mut a) = open("gc_range2");
    a.today = parse_date("2026-11-20").unwrap();
    click(&mut a, "四半期");
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-09-27", Unit::Week));
}

// [utest -> req~gantt.range-presets~2]
/// 全体は、開いたときと同じ左端と単位に戻す。
#[test]
fn test_all_range_restores_fit() {
    let (_t, mut a) = open("gc_range3");
    ch(&mut a, '-');
    for _ in 0..5 {
        ch(&mut a, 'l');
    }
    click(&mut a, "全体");
    assert_eq!((origin(&a).as_str(), unit(&a)), ("2026-10-01", Unit::Day));
}

// [utest -> req~gantt.range-presets~2]
/// 期間を選んでも、期間の外の行は隠れない。
#[test]
fn test_range_keeps_rows() {
    let (_t, mut a) = open("gc_range4");
    click(&mut a, "今週");
    assert_eq!(a.gantt.as_ref().unwrap().rows.len(), 4);
    assert!(screen(&a).contains("beta"));
}

// [utest -> req~gantt.edit-move~1]
/// 日の単位の `L` は、始まりと終わりを1日後ろへ動かしてためる(ファイルは変えない)。
#[test]
fn test_move_later_by_day() {
    let (t, mut a) = open("gc_move1");
    ch(&mut a, 'L');
    assert_eq!(val(&a, "alpha", "start"), "2026-10-02");
    assert_eq!(val(&a, "alpha", "due"), "2026-10-08");
    assert_eq!(
        std::fs::read_to_string(t.notes().join("alpha.md")).unwrap(),
        NOTES[0].1
    );
    assert_eq!(a.mode, Mode::Gantt);
}

// [utest -> req~gantt.edit-move~1]
/// 週の単位の `H` は、7日前へ動かす。
#[test]
fn test_move_earlier_by_week() {
    let (_t, mut a) = open("gc_move2");
    ch(&mut a, '-');
    ch(&mut a, 'H');
    assert_eq!(val(&a, "alpha", "start"), "2026-09-24");
    assert_eq!(val(&a, "alpha", "due"), "2026-09-30");
}

// [utest -> req~gantt.edit-move~1]
/// 月の単位の `L` は1か月後ろへ動かし、月の末日を越える日は末日に寄せる。
#[test]
fn test_move_by_month_clamps_to_month_end() {
    let (_t, mut a) = boot_with(
        "gc_move3",
        &[("alpha.md", "---\nstart: 2026-01-31\ndue: 2026-01-31\n---\n")],
        GANTT,
    );
    ch(&mut a, 'C');
    ch(&mut a, '-');
    ch(&mut a, '-');
    assert_eq!(unit(&a), Unit::Month);
    ch(&mut a, 'L');
    assert_eq!(val(&a, "alpha", "start"), "2026-02-28");
    assert_eq!(val(&a, "alpha", "due"), "2026-02-28");
}

// [utest -> req~gantt.edit-move~1]
/// 終わりだけの行(印)は、終わりだけを動かす。
#[test]
fn test_move_point_row_writes_only_its_date() {
    let (_t, mut a) = open("gc_move4");
    select(&mut a, "beta");
    ch(&mut a, 'L');
    assert_eq!(val(&a, "beta", "due"), "2026-10-04");
    assert_eq!(val(&a, "beta", "start"), "");
    assert_eq!(a.changes.count(), 1);
}

// [utest -> req~gantt.edit-move~1]
/// 日時の値は、日の部分だけを動かして時刻を残す。
#[test]
fn test_move_keeps_time_of_datetime() {
    let (_t, mut a) = boot_with(
        "gc_move5",
        &[(
            "alpha.md",
            "---\nstart: 2026-10-01T09:30\ndue: 2026-10-07T18:00\n---\n",
        )],
        GANTT,
    );
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    ch(&mut a, 'L');
    assert_eq!(val(&a, "alpha", "start"), "2026-10-02T09:30");
}

const DAYS: &str = "gantt = { start = \"start\", days = \"days\" }";
const DAYS_NOTE: &[(&str, &str)] = &[("alpha.md", "---\nstart: 2026-10-01\ndays: 3\n---\n")];

// [utest -> req~gantt.edit-move~1]
/// 日数の設定では、動かすときに始まりだけを書く。
#[test]
fn test_move_with_days_writes_only_start() {
    let (_t, mut a) = boot_with("gc_move6", DAYS_NOTE, DAYS);
    ch(&mut a, 'C');
    ch(&mut a, 'L');
    assert_eq!(val(&a, "alpha", "start"), "2026-10-02");
    assert_eq!(val(&a, "alpha", "days"), "3");
    assert_eq!(a.changes.count(), 1);
}

// [utest -> req~gantt.edit-resize~1]
/// `>` は終わりだけを1単位伸ばし、`<` は1単位縮める。
#[test]
fn test_resize_end() {
    let (_t, mut a) = open("gc_size1");
    ch(&mut a, '>');
    assert_eq!(val(&a, "alpha", "due"), "2026-10-08");
    assert_eq!(val(&a, "alpha", "start"), "2026-10-01");
    ch(&mut a, '<');
    ch(&mut a, '<');
    assert_eq!(val(&a, "alpha", "due"), "2026-10-06");
    assert_eq!(val(&a, "alpha", "start"), "2026-10-01");
}

// [utest -> req~gantt.edit-resize~1]
/// 始まりと終わりが同じ日の棒は、それ以上縮めずに理由を出す。
#[test]
fn test_shrink_stops_at_start() {
    let (_t, mut a) = boot_with(
        "gc_size2",
        &[("alpha.md", "---\nstart: 2026-10-02\ndue: 2026-10-02\n---\n")],
        GANTT,
    );
    ch(&mut a, 'C');
    a.message = None;
    ch(&mut a, '<');
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_some());
}

// [utest -> req~gantt.edit-resize~1]
/// 日数の設定では、伸ばすと日数の列を書く。
#[test]
fn test_resize_with_days_writes_days() {
    let (_t, mut a) = boot_with("gc_size3", DAYS_NOTE, DAYS);
    ch(&mut a, 'C');
    ch(&mut a, '>');
    assert_eq!(val(&a, "alpha", "days"), "4");
    assert_eq!(val(&a, "alpha", "start"), "2026-10-01");
    assert_eq!(a.changes.count(), 1);
}

// [utest -> req~gantt.edit-refuse~1]
/// 自分の日付が無い親の行(子の範囲の棒)は動かさず、理由を出す。
#[test]
fn test_refuse_parent_without_own_dates() {
    let (_t, mut a) = boot_with(
        "gc_ref1",
        &[
            ("alpha.md", "---\nstatus: doing\n---\n"),
            (
                "beta.md",
                "---\nparent: \"[[alpha]]\"\nstart: 2026-10-01\ndue: 2026-10-07\n---\n",
            ),
        ],
        GANTT,
    );
    a.settings.tree = Some("parent".into());
    a.regrid = true;
    a.refresh_if_needed();
    ch(&mut a, 'C');
    select(&mut a, "alpha");
    a.message = None;
    ch(&mut a, 'L');
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_some());
}

// [utest -> req~gantt.edit-refuse~1]
/// 読むだけの起動では動かさず、理由を出す。
#[test]
fn test_refuse_when_readonly() {
    let (_t, mut a) = boot_opts("gc_ref2", NOTES, GANTT, true);
    ch(&mut a, 'C');
    a.message = None;
    ch(&mut a, 'L');
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_some());
}

// [utest -> req~gantt.edit-refuse~1]
/// 終わりが読むだけの列なら伸ばさず、理由を出す。
#[test]
fn test_refuse_readonly_column() {
    let (_t, mut a) = boot_with(
        "gc_ref3",
        &[("alpha.md", "---\nstart: 2026-10-01\n---\n")],
        "gantt = { start = \"start\", end = \"file.mtime\" }",
    );
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    a.message = None;
    ch(&mut a, '>');
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_some());
}

// [utest -> req~gantt.edit-undo~1]
/// `u` で取り消し、`U` でやり直す(ガントのまま)。
#[test]
fn test_undo_and_redo_in_gantt() {
    let (_t, mut a) = open("gc_undo1");
    ch(&mut a, 'L');
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    assert_eq!(a.mode, Mode::Gantt);
    ch(&mut a, 'U');
    assert_eq!(val(&a, "alpha", "start"), "2026-10-02");
    assert_eq!(val(&a, "alpha", "due"), "2026-10-08");
}

// [utest -> req~gantt.edit-undo~1]
/// 変更をためてから Ctrl+S で、保存の確認の画面が出る。
#[test]
fn test_ctrl_s_opens_save_confirm() {
    let (_t, mut a) = open("gc_undo2");
    ch(&mut a, 'L');
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
}

/// 下の帯(未保存の数で始まる行)。
fn band(a: &App) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.trim_start().starts_with("未保存"))
        .unwrap_or_else(|| panic!("下の帯が無い:\n{s}"))
        .to_string()
}

/// 棒を動かすキーと伸ばす・縮めるキーの、下の帯の案内。
const MOVE_HINT: &str = "H/L 棒を動かす";
const RESIZE_HINT: &str = ">/< 伸ばす・縮める";

// [utest -> req~gantt.band-edit-keys~1]
/// 自分の日付のある行(alpha)を選んだガントの下の帯に、`H`・`L`(棒を動かす)と `>`・`<`(伸ばす・縮める)の説明が出る。
/// 幅 120 と 80 の端末のどちらでも(80 では入りきらない案内を省いても、この2つは残る)。
#[test]
fn test_band_shows_bar_keys_for_movable_bar() {
    let (_t, mut a) = open("gc_band1");
    for w in [120u16, 80] {
        a.resize(w, 30);
        let b = band(&a);
        assert!(b.contains(MOVE_HINT), "幅 {w}: {b}");
        assert!(b.contains(RESIZE_HINT), "幅 {w}: {b}");
    }
    // 終わりだけの行(beta)も動かせる・伸ばせる。
    select(&mut a, "beta");
    let b = band(&a);
    assert!(b.contains(MOVE_HINT) && b.contains(RESIZE_HINT), "{b}");
}

// [utest -> req~gantt.band-edit-keys~1]
/// 読むだけの起動のガントの下の帯には、棒を動かすキーも伸ばす・縮めるキーも出ない。
#[test]
fn test_band_hides_bar_keys_when_readonly() {
    let (_t, mut a) = boot_opts("gc_band2", NOTES, GANTT, true);
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    for w in [120u16, 300] {
        a.resize(w, 30);
        let b = band(&a);
        assert!(b.contains("Enter"), "幅 {w}: {b}");
        for k in ["H/L", ">/<", "棒を"] {
            assert!(!b.contains(k), "幅 {w}: 「{k}」が出る: {b}");
        }
    }
}

// [utest -> req~gantt.band-edit-keys~1]
/// 動かせない棒(日付の無い行 delta、読むだけのノート(同じキーが2回ある)の行)を選んでいるときも、棒を動かすキーは
/// 出ない(gantt.edit-refuse で断る棒。`L` を押しても変わらない)。
#[test]
fn test_band_hides_bar_keys_for_unmovable_bar() {
    let (_t, mut a) = open("gc_band3");
    a.resize(300, 30);
    select(&mut a, "delta");
    let b = band(&a);
    assert!(b.contains("Enter"), "{b}");
    for k in ["H/L", ">/<", "棒を"] {
        assert!(!b.contains(k), "日付の無い行: 「{k}」が出る: {b}");
    }
    let (_t2, mut a) = boot_with(
        "gc_band4",
        &[
            (
                "alpha.md",
                "---\nstatus: todo\nstatus: done\nstart: 2026-10-01\ndue: 2026-10-03\n---\n",
            ),
            ("beta.md", "---\nstart: 2026-10-02\ndue: 2026-10-05\n---\n"),
        ],
        GANTT,
    );
    ch(&mut a, 'C');
    assert_eq!(a.mode, Mode::Gantt, "{:?}", a.message);
    a.resize(300, 30);
    select(&mut a, "alpha");
    let b = band(&a);
    for k in ["H/L", ">/<", "棒を"] {
        assert!(!b.contains(k), "読むだけのノート: 「{k}」が出る: {b}");
    }
    ch(&mut a, 'L');
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
}
