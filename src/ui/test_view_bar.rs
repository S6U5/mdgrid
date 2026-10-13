//! 表の上のパンくずのヘッダーとビューの帯(SR-1・SR-42・NV-23・NV-24・NV-29・WS-8)の受け入れ試験。
//! 記録: specs/_changes/2026-10-11-view-bar.md(タスク #2)。
//!
//! 内部の状態のフィールドや新しい関数には頼らず、キー・クリック・描いた画面の文字・`mode`・`settings`・
//! 行・`switch_to` で確かめる。置いた仮定(記録の「不明点と仮定」に沿う):
//! - ビューの帯は画面の2行目(添字 1)。ボタンの文字は「絞り込み」「並べ替え」「分類」「列」「⚙」で、
//!   キーのあるボタンはキーを頭に添える(「| 絞り込み」「S 並べ替え」「o ⚙」)。検索の欄は「\ 検索」。
//! - パンくずは「範囲 › 表  N行」。範囲の名前を押すと引き出し。表の名前は今の表の名前(フォルダの名前か、
//!   ワークスペースに書いた名前のどちらでもよいとする)。
//! - 窓はボタンの行のすぐ下に、左の端をボタンの左の端にそろえて置く(キーの添えの分の数桁のずれは許す)。
//! - 絞り込みの窓の条件の行は 列・比べ方・値・× の順に並ぶ。値の一覧で隠す条件の比べ方の字は「隠す」か「除く」。
//!   ←→ で欄を選び、Enter で選んだ欄を直す。

use super::keymap::Mode;
use super::settings::Sec;
use super::startup::Startup;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, make, press, text, typing, Tmp};
use super::*;
use mdgrid::settings::{CmpOp, Cond, Op};
use mdgrid::source::markdown::Markdown;
use mdgrid::workspace::{self as ws, WsTable};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

// ---- 材料 ----

/// status は done が3つ・todo が2つ・doing が1つ。priority は数。memo は a.md だけ。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\npriority: 5\nmemo: x\n---\n"),
    ("b.md", "---\nstatus: done\npriority: 1\n---\n"),
    ("c.md", "---\nstatus: done\npriority: 4\n---\n"),
    ("d.md", "---\nstatus: todo\npriority: 4\n---\n"),
    ("e.md", "---\nstatus: todo\npriority: 2\n---\n"),
    ("f.md", "---\nstatus: doing\npriority: 3\n---\n"),
];

/// ビューの帯の字の行(80×24 で添字 3。上の余白の1行(2026-10-12-bar-padding-half)と、2026-10-12-search-box・view-bar-plain の
/// 3行の帯(添字 2〜4)の真ん中の行。前は1行の帯で添字 1)。
const BAR: u16 = 3;
/// ビューの帯のすぐ下の行(80×24 で添字 5。3行の帯の下の1行の次)。
const BELOW: u16 = 5;
/// ヘッダーの行(80×24 で添字 1。2026-10-12-bar-padding-half の上の余白 0.5 は色なしで1行の空きになる。前は添字 0)。
const HEAD: u16 = 1;

fn boot(name: &str) -> (Tmp, App) {
    make(name, NOTES)
}

/// 設定ファイルの文字を渡して起動する。
fn boot_config(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 今の端末の大きさ(`a.size`)で描いたバッファ。
fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

fn scr(a: &App) -> String {
    text(&buf(a))
}

fn line(a: &App, y: u16) -> String {
    scr(a)
        .lines()
        .nth(y as usize)
        .unwrap_or_default()
        .to_string()
}

/// 行 y で文字 `t` の始まる桁(左から最初のもの)。
fn col_in(a: &App, y: u16, t: &str) -> Option<u16> {
    let b = buf(a);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        cs.iter().all(|c| {
            let hit = xx < b.area.width && b[(xx, y)].symbol() == c;
            xx += width::width(c).max(1) as u16;
            hit
        })
    })
}

/// 行 y の文字 `t` の上をクリックする。
fn click_text(a: &mut App, y: u16, t: &str) {
    let x = col_in(a, y, t).unwrap_or_else(|| panic!("行 {y} に「{t}」が無い:\n{}", scr(a)));
    a.click(x, y);
}

/// 行 y の窓の左上の角の桁。
fn corner_in(a: &App, y: u16) -> Option<u16> {
    let b = buf(a);
    (0..b.area.width).find(|&x| matches!(b[(x, y)].symbol(), "╭" | "┌" | "+"))
}

fn names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// 窓の中で、頭に「> 」の付いた(選んでいる)行が `target` を含むか。
fn selected_has(a: &App, target: &str) -> bool {
    scr(a).lines().any(|l| {
        l.match_indices("> ")
            .any(|(i, _)| l[i + 2..].find(target).is_some_and(|p| p <= 8))
    })
}

/// 窓の中の行を、先頭に戻ってから j で下りて `target` の行を選ぶ。
fn select(a: &mut App, target: &str) {
    for _ in 0..40 {
        ch(a, 'k');
    }
    for _ in 0..40 {
        if selected_has(a, target) {
            return;
        }
        ch(a, 'j');
    }
    panic!("窓に「{target}」の行が無い:\n{}", scr(a));
}

/// 「+ 絞り込みを足す」→ 列 → 種類 まで進む(窓は開いている)。
fn start_add(a: &mut App, col: &str, kind: &str) {
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(a));
    select(a, "+ 絞り込みを足す");
    press(a, KeyCode::Enter);
    select(a, col);
    press(a, KeyCode::Enter);
    select(a, kind);
    press(a, KeyCode::Enter);
}

/// status の done を隠す条件を足す。
fn hide_done(a: &mut App) {
    start_add(a, "status", "値の一覧");
    if scr(a).contains("チェックした値の行だけ残す") {
        select(a, "方式");
        ch(a, ' ');
    }
    assert!(scr(a).contains("チェックした値の行を隠す"), "{}", scr(a));
    select(a, "done");
    ch(a, ' ');
    press(a, KeyCode::Enter);
}

/// priority の `≥ 3` を足す。
fn add_ge3(a: &mut App) {
    start_add(a, "priority", "比べる");
    select(a, "≥");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText, "{}", scr(a));
    typing(a, "3");
    press(a, KeyCode::Enter);
}

fn drop_done() -> Cond {
    Cond {
        col: "status".to_string(),
        op: Op::Drop(vec![Some("done".to_string())]),
    }
}

fn ge3() -> Cond {
    Cond {
        col: "priority".to_string(),
        op: Op::Cmp(CmpOp::Ge, "3".to_string()),
    }
}

/// 条件を2つ(status の done を隠す・priority ≥ 3)足した窓を開いたまま返す。
fn two_conditions(name: &str) -> (Tmp, App) {
    let (t, mut a) = boot(name);
    ch(&mut a, '|');
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    hide_done(&mut a);
    add_ge3(&mut a);
    assert_eq!(a.settings.filters, [drop_done(), ge3()]);
    assert_eq!(a.mode, Mode::FilterWin);
    (t, a)
}

/// 窓の中の、`col` で始まる条件の行(左の枠の後ろの文字)。
fn cond_row(a: &App, col: &str) -> String {
    let s = scr(a);
    s.lines()
        .find(|l| {
            l.find(['│', '|'])
                .map(|i| {
                    l[i..]
                        .trim_start_matches(['│', '|', ' ', '>'])
                        .starts_with(col)
                })
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("条件の行「{col}」が無い:\n{s}"))
        .to_string()
}

/// 選んでいる条件の行で、欄を `i` 番目(0: 列・1: 比べ方・2: 値)にする。← で左の端(列)へ寄せてから → で進む
/// (← → は端で止まり、回らないとする)。
fn field(a: &mut App, i: usize) {
    for _ in 0..6 {
        press(a, KeyCode::Left);
    }
    for _ in 0..i {
        press(a, KeyCode::Right);
    }
}

/// 条件の行の中で、`parts` がこの順に並ぶか(どれかが無ければ偽)。各要素は候補のどれか。
fn in_order(row: &str, parts: &[&[&str]]) -> bool {
    let mut from = 0;
    for cands in parts {
        let hit = cands
            .iter()
            .filter_map(|c| row[from..].find(c).map(|p| (p, c.len())))
            .min_by_key(|(p, _)| *p);
        match hit {
            Some((p, n)) => from += p + n,
            None => return false,
        }
    }
    true
}

/// 「仕事」の範囲: notes/ に books を足し、Work に Tasks・Projects、Hobby に Books を書いて tasks を開く。
fn boot_work(name: &str) -> (Tmp, App) {
    let tmp = workspace(name);
    let dir = |d: &str| -> PathBuf { tmp.notes().join(d).canonicalize().unwrap() };
    let p = tmp.notes().join("books/Dune.md");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, "---\nauthor: Herbert\n---\n").unwrap();
    let t = |name: &str, d: &str| WsTable {
        name: name.into(),
        path: dir(d),
        view: None,
        profile: Default::default(),
    };
    let cfg = tmp.0.join("config");
    ws::add(&cfg, "Work", t("Tasks", "tasks")).unwrap();
    ws::add(&cfg, "Work", t("Projects", "projects")).unwrap();
    ws::add(&cfg, "Hobby", t("Books", "books")).unwrap();
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    assert_eq!(a.scope.as_ref().map(|s| s.name.as_str()), Some("Work"));
    (tmp, a)
}

// ---- SR-1: 行の並び ----

#[test]
fn test_sr_1_view_bar_rows() {
    // [SR-1] 80×24 で起動 → 1行目がヘッダー、2行目にビューのタブ・検索の欄・絞り込み・並べ替えが同じ行、
    // 3行目が列の見出し(条件が無いとき)。
    let (_t, a) = boot("sr1bar");
    let s = scr(&a);
    let head = line(&a, HEAD);
    assert!(
        head.contains("notes") && head.contains("6行"),
        "ヘッダー: {head:?}\n{s}"
    );
    let bar = line(&a, BAR);
    for t in ["既定の表", "検索", "絞り込み", "並べ替え"] {
        assert!(bar.contains(t), "2行目に「{t}」: {bar:?}\n{s}");
    }
    assert!(
        bar.find("既定の表").unwrap() < bar.find("検索").unwrap(),
        "タブが左、検索の欄が右: {bar:?}"
    );
    let cols = line(&a, BELOW);
    assert!(
        cols.contains("ノート") && cols.contains("status"),
        "帯の下が列の見出し: {cols:?}\n{s}"
    );
    assert!(s.contains("未保存 0"), "{s}");
}

#[test]
fn test_sr_1_view_bar_with_condition_then_chips() {
    // [SR-1][NV-16] 条件が効いているとき → 3行目が設定の帯、4行目が列の見出し。
    let (_t, mut a) = boot("sr1chips");
    ch(&mut a, '|');
    hide_done(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    let s = scr(&a);
    assert!(line(&a, BAR).contains("検索"), "2行目はビューの帯:\n{s}");
    assert!(
        line(&a, BELOW).contains("status: done を除く"),
        "帯の下が設定の帯:\n{s}"
    );
    let cols = line(&a, BELOW + 1);
    assert!(
        cols.contains("ノート") && cols.contains("status"),
        "4行目が列の見出し: {cols:?}\n{s}"
    );
}

#[test]
fn test_sr_1_view_bar_hidden_when_tabs_and_search_hidden() {
    // [SR-1][SR-20] タブ(`tabs = "never"`)と検索の欄(`search_bar = false`)を両方隠す → ビューの帯の行ごと
    // 出さず、2行目が列の見出し。
    let (_t, a) = boot_config(
        "sr1hide",
        "[display]\ntabs = \"never\"\nsearch_bar = false\n",
    );
    let s = scr(&a);
    let cols = line(&a, HEAD + 1);
    assert!(
        cols.contains("ノート") && cols.contains("status"),
        "2行目が列の見出し: {cols:?}\n{s}"
    );
    assert!(!s.contains("並べ替え"), "ボタンも出ない:\n{s}");
}

#[test]
fn test_sr_1_view_bar_tabs_hidden_keeps_search_and_buttons() {
    // [SR-1][SR-34] タブだけ隠す(`tabs = "never"`)→ ビューの帯は残り、検索の欄とボタンが2行目、3行目が見出し。
    let (_t, a) = boot_config("sr1tabs", "[display]\ntabs = \"never\"\n");
    let s = scr(&a);
    let bar = line(&a, BAR);
    assert!(!bar.contains("既定の表"), "タブは出さない: {bar:?}");
    for t in ["検索", "絞り込み", "並べ替え", "分類"] {
        assert!(bar.contains(t), "2行目に「{t}」: {bar:?}\n{s}");
    }
    assert!(line(&a, BELOW).contains("ノート"), "{s}");
}

// ---- NV-23: ビューの帯の中身 ----

#[test]
fn test_nv_23_view_bar_buttons_in_order_with_keys() {
    // [NV-23] 2行目に「\ 検索」の欄、その右に 絞り込み・並べ替え・分類・列、右の端に「+ 新規」がこの順。
    // キーのあるボタンにキーが添えてある(「| 絞り込み」「S 並べ替え」「a + 新規」)。設定の ⚙ は
    // 2026-10-12-header-menu で1行目の「o ⚙ 設定」に移り、2行目には無い。
    let (_t, a) = boot("nv23order");
    let bar = line(&a, BAR);
    for t in ["\\ 検索", "| 絞り込み", "S 並べ替え", "a + 新規"] {
        assert!(bar.contains(t), "「{t}」: {bar:?}\n{}", scr(&a));
    }
    let mut at = 0;
    for t in ["検索", "絞り込み", "並べ替え", "分類", "列", "新規"] {
        let p = bar[at..]
            .find(t)
            .unwrap_or_else(|| panic!("「{t}」が順に無い: {bar:?}"));
        at += p + t.len();
    }
    assert!(!bar.contains('⚙'), "{bar:?}");
    assert!(line(&a, HEAD).contains("o ⚙ 設定"), "{}", scr(&a));
}

#[test]
fn test_nv_23_view_bar_search_word_and_count() {
    // [NV-23][NV-2] 「会議」で絞る → ビューの帯の検索の欄に語と「3/5行」。ボタンは残る。
    let (_t, mut a) = make(
        "nv23word",
        &[
            ("a.md", "---\ntitle: 定例の会議\n---\n"),
            ("b.md", "---\ntitle: 会議の準備\n---\n"),
            ("c.md", "---\ntitle: 買い物\n---\n"),
            ("d.md", "---\ntitle: 会議室の予約\n---\n"),
            ("e.md", "---\ntitle: 読書\n---\n"),
        ],
    );
    ch(&mut a, '\\');
    assert_eq!(a.mode, Mode::Filter);
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 3);
    let bar = line(&a, BAR);
    assert!(bar.contains("会議"), "{bar:?}\n{}", scr(&a));
    assert!(bar.contains("3/5行"), "{bar:?}\n{}", scr(&a));
    assert!(
        bar.contains("絞り込み") && bar.contains("並べ替え"),
        "{bar:?}"
    );
}

#[test]
fn test_nv_23_view_bar_group_button_opens_settings_at_group() {
    // [NV-23] 「分類」をクリック → ビューの設定の画面の分類の区画。
    let (_t, mut a) = boot("nv23group");
    click_text(&mut a, BAR, "分類");
    assert_eq!(a.mode, Mode::Settings, "{}", scr(&a));
    assert_eq!(a.draft.as_ref().map(|d| d.sec), Some(Sec::Group));
}

#[test]
fn test_nv_23_view_bar_columns_button_opens_settings_at_columns() {
    // [NV-23] 「列」をクリック → ビューの設定の画面の列の区画。
    let (_t, mut a) = boot("nv23cols");
    let bar = line(&a, BAR);
    // 「列」は「分類」より右にあるもの(見出しなどの字と取り違えない)。
    let after = col_in(&a, BAR, "分類").unwrap_or_else(|| panic!("分類が無い: {bar:?}"));
    let b = buf(&a);
    let x = (after..b.area.width)
        .find(|&x| b[(x, BAR)].symbol() == "列")
        .unwrap_or_else(|| panic!("列のボタンが無い: {bar:?}"));
    a.click(x, BAR);
    assert_eq!(a.mode, Mode::Settings, "{}", scr(&a));
    assert_eq!(a.draft.as_ref().map(|d| d.sec), Some(Sec::Columns));
}

#[test]
fn test_nv_23_view_bar_gear_opens_settings() {
    // [NV-23][SR-42] ⚙(2026-10-12-header-menu で1行目の右の端に移った)をクリック → ビューの設定の画面。
    let (_t, mut a) = boot("nv23gear");
    click_text(&mut a, HEAD, "⚙");
    assert_eq!(a.mode, Mode::Settings, "{}", scr(&a));
}

#[test]
fn test_nv_23_view_bar_narrow_drops_gear_columns_group() {
    // [NV-23] 幅 60 → 列・分類が無く、今のタブ・検索の欄・絞り込み・並べ替え・「+ 新規」は出る。80 なら全部
    // 出る(⚙ は 2026-10-12-header-menu で帯から外れた)。
    let (_t, mut a) = boot("nv23narrow");
    let wide = line(&a, BAR);
    for t in ["分類", "列", "新規"] {
        assert!(wide.contains(t), "80 桁では「{t}」: {wide:?}");
    }
    a.resize(60, 24);
    let bar = line(&a, BAR);
    for t in ["分類", "列", "⚙"] {
        assert!(
            !bar.contains(t),
            "60 桁では「{t}」を省く: {bar:?}\n{}",
            scr(&a)
        );
    }
    for t in ["既定の表", "検索", "絞り込み", "並べ替え", "新規"] {
        assert!(bar.contains(t), "60 桁でも「{t}」: {bar:?}\n{}", scr(&a));
    }
}

#[test]
fn test_nv_23_view_bar_click_search_field_enters() {
    // [NV-23] ビューの帯の検索の欄をクリック → 欄に入る(打つモード)。
    let (_t, mut a) = boot("nv23click");
    click_text(&mut a, BAR, "検索");
    assert_eq!(a.mode, Mode::Filter, "{}", scr(&a));
}

#[test]
fn test_nv_23_view_bar_tab_click_switches_view() {
    // [NV-23][SR-1] ビューが3つの `.base` で、ビューの帯(検索の欄と同じ行)のタブ「完了」をクリック → ビューが変わる。
    let tmp = super::test_grid::vault("nv23tab");
    let mut a = super::test_grid::open(&tmp, super::test_grid::TASKS, None);
    let bar = line(&a, BAR);
    assert!(
        bar.contains("完了") && bar.contains("検索"),
        "タブと検索の欄が同じ行: {bar:?}\n{}",
        scr(&a)
    );
    assert_ne!(a.view_names()[a.view_index()], "完了");
    click_text(&mut a, BAR, "完了");
    assert_eq!(a.view_names()[a.view_index()], "完了", "{}", scr(&a));
}

// ---- NV-24・NV-29: 窓はボタンのすぐ下 ----

#[test]
fn test_nv_24_view_bar_sort_window_below_button() {
    // [NV-24] 「並べ替え」を押す → 窓がボタンのすぐ下の行に出て、左の端がボタンの左の端の近く(右の端に寄せない)。
    let (_t, mut a) = boot("nv24place");
    let bx = col_in(&a, BAR, "並べ替え").unwrap_or_else(|| panic!("2行目に並べ替え:\n{}", scr(&a)));
    a.click(bx + 1, BAR);
    assert_eq!(a.mode, Mode::Sorts, "{}", scr(&a));
    let x =
        corner_in(&a, BAR + 2).unwrap_or_else(|| panic!("ボタンのすぐ下に窓が無い:\n{}", scr(&a)));
    assert!(
        x.abs_diff(bx) <= 4,
        "窓の左の端 {x} がボタン {bx} の近く:\n{}",
        scr(&a)
    );
}

#[test]
fn test_nv_29_table_window_below_button() {
    // [NV-29] 「絞り込み」を押す → 窓がボタンのすぐ下の行に出て、左の端がボタンの左の端の近く。
    let (_t, mut a) = boot("nv29place");
    let bx = col_in(&a, BAR, "絞り込み").unwrap_or_else(|| panic!("2行目に絞り込み:\n{}", scr(&a)));
    a.click(bx + 1, BAR);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert!(scr(&a).contains("+ 絞り込みを足す"), "{}", scr(&a));
    let x =
        corner_in(&a, BAR + 2).unwrap_or_else(|| panic!("ボタンのすぐ下に窓が無い:\n{}", scr(&a)));
    assert!(
        x.abs_diff(bx) <= 4,
        "窓の左の端 {x} がボタン {bx} の近く:\n{}",
        scr(&a)
    );
}

// ---- NV-29: 条件の表の形 ----

#[test]
fn test_nv_29_table_rows_have_column_op_value_remove() {
    // [NV-29] 条件を2つ足した窓で、各条件の行に 列・比べ方・値・× がこの順に並ぶ。
    let (_t, a) = two_conditions("nv29rows");
    let st = cond_row(&a, "status");
    assert!(
        in_order(&st, &[&["status"], &["隠す", "除く"], &["done"], &["×"]]),
        "status の行は 列・比べ方・値・×: {st:?}\n{}",
        scr(&a)
    );
    let pr = cond_row(&a, "priority");
    assert!(
        in_order(&pr, &[&["priority"], &["≥"], &["3"], &["×"]]),
        "priority の行は 列・比べ方・値・×: {pr:?}\n{}",
        scr(&a)
    );
}

#[test]
fn test_nv_29_table_edit_op_in_place() {
    // [NV-29] status の条件の行で → を押して比べ方の欄を選び Enter → 種類を選ぶ段。「含む」を選んで「ing」と
    // 打って Enter → その条件が Contains("ing") に変わり、条件の数は増えない(priority の条件はそのまま)。
    let (_t, mut a) = two_conditions("nv29op");
    select(&mut a, "status");
    field(&mut a, 1);
    press(&mut a, KeyCode::Enter);
    let s = scr(&a);
    assert!(
        s.contains("含む") && s.contains("空である") && s.contains("値の一覧"),
        "種類を選ぶ段:\n{s}"
    );
    select(&mut a, "含む");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText, "{}", scr(&a));
    typing(&mut a, "ing");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert_eq!(
        a.settings.filters,
        [
            Cond {
                col: "status".to_string(),
                op: Op::Contains("ing".to_string()),
            },
            ge3()
        ],
        "その場で変わり、数は増えない"
    );
    assert_eq!(names(&a), ["f.md"]);
}

#[test]
fn test_nv_29_table_edit_column_in_place() {
    // [NV-29] status の条件の行で ← を押して列の欄を選び Enter → 列を選ぶ段。memo を選び、種類「空である」→
    // その条件が memo の空であるに変わり、条件の数は増えない。
    let (_t, mut a) = two_conditions("nv29col");
    select(&mut a, "status");
    field(&mut a, 0);
    press(&mut a, KeyCode::Enter);
    assert!(scr(&a).contains("絞り込む列"), "列を選ぶ段:\n{}", scr(&a));
    select(&mut a, "memo");
    press(&mut a, KeyCode::Enter);
    select(&mut a, "空である");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert_eq!(
        a.settings.filters,
        [
            Cond {
                col: "memo".to_string(),
                op: Op::Empty,
            },
            ge3()
        ]
    );
}

#[test]
fn test_nv_29_table_edit_value_field() {
    // [NV-29] 値の欄で Enter → 値の一覧(status の隠す条件。done にチェック)か、値の入力(priority ≥ 3 は「3」)。
    let (_t, mut a) = two_conditions("nv29val");
    select(&mut a, "status");
    field(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert!(
        scr(&a).contains("チェックした値の行を隠す"),
        "値の一覧:\n{}",
        scr(&a)
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::FilterWin);
    select(&mut a, "priority");
    field(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText, "値の入力:\n{}", scr(&a));
    press(&mut a, KeyCode::Backspace);
    typing(&mut a, "4");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.settings.filters,
        [
            drop_done(),
            Cond {
                col: "priority".to_string(),
                op: Op::Cmp(CmpOp::Ge, "4".to_string()),
            }
        ]
    );
}

// ---- SR-42: パンくず ----

#[test]
fn test_sr_42_crumb_scope_and_table() {
    // [SR-42][WS-6] ワークスペース Work の tasks を開く → 1行目の左が「w Work › tasks ▾」と行数。
    // 「ワークスペース」のボタンは無い。「設定」は 2026-10-12-header-menu で右の端のボタンになった。
    let (_tmp, a) = boot_work("sr42crumb");
    let head = line(&a, HEAD);
    let left = head.trim_start();
    assert!(left.starts_with("w Work › "), "{head:?}\n{}", scr(&a));
    let table = left["w Work › ".len()..].to_lowercase();
    assert!(table.starts_with("tasks"), "範囲 › 表: {head:?}");
    assert!(head.contains("3行"), "{head:?}");
    assert!(head.trim_end().ends_with("o ⚙ 設定"), "{head:?}");
    assert!(!head.contains("ワークスペース"), "{head:?}");
}

#[test]
fn test_sr_42_crumb_click_toggles_drawer() {
    // [SR-42][WS-8] パンくずの範囲の名前をクリック → 引き出しが開き、もう一度で閉じる。表の名前でも開く。
    let (_tmp, mut a) = boot_work("sr42click");
    click_text(&mut a, HEAD, "Work");
    assert_eq!(a.mode, Mode::Drawer, "{}", scr(&a));
    click_text(&mut a, HEAD, "Work");
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.switch_to.is_none() && !a.quit);
    let x = col_in(&a, HEAD, "› ").expect("区切り") + 2;
    a.click(x + 1, HEAD);
    assert_eq!(a.mode, Mode::Drawer, "表の名前でも開く:\n{}", scr(&a));
}

#[test]
fn test_sr_42_crumb_same_column_in_relations() {
    // [SR-42] 関係マップでもパンくずは同じ桁。押すと表の画面に戻って引き出しが開く。
    let (_tmp, mut a) = boot_work("sr42rel");
    let x = col_in(&a, HEAD, "Work ›").unwrap_or_else(|| panic!("{}", scr(&a)));
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{}", scr(&a));
    assert_eq!(col_in(&a, HEAD, "Work ›"), Some(x), "同じ桁:\n{}", scr(&a));
    a.click(x + 1, HEAD);
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert_eq!(a.mode, Mode::Drawer, "{}", scr(&a));
}

#[test]
fn test_sr_42_crumb_without_scope() {
    // [SR-42] 範囲が無い(登録した表だけ)→ 「›」が無く、表の名前と行数。
    let tmp = workspace("sr42none");
    let a = boot_tasks(&tmp);
    assert!(a.scope.is_none());
    let head = line(&a, HEAD);
    assert!(!head.contains('›'), "{head:?}");
    assert!(head.trim_start().starts_with("w tasks"), "{head:?}");
    assert!(head.contains("3行"), "{head:?}");
    // 「設定」は 2026-10-12-header-menu で右の端のボタンになった。
    assert!(head.trim_end().ends_with("o ⚙ 設定"), "{head:?}");
}

// ---- WS-8: パンくずから開いた引き出しで移る ----

#[test]
fn test_ws_8_crumb_drawer_switches_table() {
    // [WS-8][SR-42] パンくずの範囲の名前から開いた引き出しで Projects をクリック → その表へ移る。
    let (tmp, mut a) = boot_work("ws8crumb");
    click_text(&mut a, HEAD, "Work");
    assert_eq!(a.mode, Mode::Drawer, "{}", scr(&a));
    let s = scr(&a);
    let y = s
        .lines()
        .enumerate()
        .skip(1)
        .find(|(_, l)| l.contains("Projects"))
        .map(|(i, _)| i as u16)
        .unwrap_or_else(|| panic!("引き出しに Projects:\n{s}"));
    click_text(&mut a, y, "Projects");
    assert_ne!(a.mode, Mode::Drawer);
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(tmp.notes().join("projects").canonicalize().unwrap())
    );
    assert_eq!(a.switch_workspace.as_deref(), Some("Work"));
    assert!(a.quit);
}
