//! 絞り込みの窓(NV-29)と、上の欄の「検索」・`/` の「探す」の呼び分け(NV-1・NV-2・NV-23・NV-24)の受け入れ試験。
//! 記録: specs/_changes/2026-10-11-filter-button.md(タスク #1)。
//!
//! 窓の中の状態のフィールドには頼らず、キー・クリック・画面の文字・`mode`・`settings.filters`・行の数・
//! `message` で確かめる。窓の段取りは記録の仮定のとおり: 条件の並びと最後の「+ 絞り込みを足す」→ Enter で
//! 列を選ぶ → 種類(値の一覧・含む・含まない・比べる・空である・空でない)→ 値の一覧なら先頭の行が残す/隠すの
//! 切り替え、Space でチェック、Enter で決める。含む・比べるは値を打って Enter。
//! 選んでいる行は、並べ替えの窓(NV-24)と同じく行の頭の「> 」で見分ける。

use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, make, press, text, typing, Tmp};
use super::*;
use mdgrid::settings::{CmpOp, Cond, Op};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

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

/// 下の帯の行(80×24 で添字 21。test_language と同じ)。
const BAND: usize = 21;

fn boot(name: &str) -> (Tmp, App) {
    make(name, NOTES)
}

/// 見た目の状態の置き場を一時フォルダに持つ起動(開き直し・読むだけの試験)。
fn boot_with_state(tmp: &Tmp, toml: &str, readonly: bool) -> App {
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn notes_tmp(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    tmp
}

fn names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// 今の端末の大きさで描いた画面の文字。
fn scr(a: &App) -> String {
    let mut term = Terminal::new(TestBackend::new(a.size.0, a.size.1)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// 検索の欄の行(ビューの帯。表より上で「検索」を含む行。SR-1)の添字と文字。
fn bar_line(a: &App) -> (usize, String) {
    let s = scr(a);
    s.lines()
        .enumerate()
        .take(view::data_y(a))
        .find(|(_, l)| l.contains("検索"))
        .map(|(i, l)| (i, l.to_string()))
        .unwrap_or_else(|| panic!("検索の欄が無い:\n{s}"))
}

/// 検索の欄の行の「絞り込み」のボタン(の文字の真ん中)をクリックする。
fn click_filter_button(a: &mut App) {
    let (y, line) = bar_line(a);
    let at = line
        .find("絞り込み")
        .unwrap_or_else(|| panic!("絞り込みのボタンが無い: {line:?}"));
    let x = width::width(&line[..at]) as u16 + 2;
    a.click(x, y as u16);
}

/// 窓の中で、頭に「> 」の付いた(選んでいる)行が `target` を含むか。
fn selected_has(a: &App, target: &str) -> bool {
    scr(a).lines().any(|l| {
        // 値の一覧の行はチェックの印が頭に付くので、「> 」の少し後ろまで許す。
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

/// status の done を隠す条件を足す(値の一覧: 隠す方式にして done にチェック → Enter)。
fn hide_done(a: &mut App) {
    start_add(a, "status", "値の一覧");
    // 先頭の行が残す/隠すの切り替え。残す方式なら隠す方式に切り替える。
    if scr(a).contains("チェックした値の行だけ残す") {
        select(a, "方式");
        ch(a, ' ');
    }
    assert!(scr(a).contains("チェックした値の行を隠す"), "{}", scr(a));
    select(a, "done");
    ch(a, ' ');
    press(a, KeyCode::Enter);
}

fn drop_done() -> Cond {
    Cond {
        col: "status".to_string(),
        op: Op::Drop(vec![Some("done".to_string())]),
    }
}

fn filter_menu() -> Action {
    Action::by_name("filter_menu").expect("動作 filter_menu")
}

// ---- ボタンの並び ----

#[test]
fn test_nv_29_buttons_filter_then_sort_at_right_end() {
    // [NV-29][NV-23][NV-24] 検索の欄の右に「絞り込み」と「並べ替え」がこの順に並ぶ。欄の頭は「検索」のまま。
    // SR-1: 欄とボタンはビューの帯(タブと同じ行)にあり、ボタンは欄の右に 絞り込み・並べ替え・分類… の順。
    let (_t, a) = boot("nv29btn");
    let (y, line) = bar_line(&a);
    // 2026-10-12-search-box・view-bar-plain: 3行の帯では、字の行の下に1行(空きの行か囲みの下の線)ある。
    assert_eq!(
        y + 2 + usize::from(super::bands::three_rows(&a)),
        view::data_y(&a),
        "ビューの帯は列の見出しのすぐ上:\n{}",
        scr(&a)
    );
    assert!(line.contains("既定の表"), "タブと同じ行: {line:?}");
    let lead = line
        .find("\\ 検索")
        .unwrap_or_else(|| panic!("欄の頭: {line:?}"));
    let f = line.find("絞り込み").expect("絞り込みのボタン");
    let s = line.rfind("並べ替え").unwrap();
    assert!(lead < f, "ボタンは欄の右: {line:?}");
    assert!(f < s, "絞り込みは並べ替えの左: {line:?}");
    let g = line.find("分類").expect("分類のボタン");
    assert!(s < g, "並べ替えの次は分類: {line:?}");
    // 二つのボタンの間には検索の欄の文字が入らない(隣り合う。間は並べ替えのキーの添えだけ。NV-23)。
    // 囲みの縦の線(2026-10-12-search-box)は除いて見る。
    let sort_key =
        super::keymap::key_for(&a.keys, Mode::Table, Action::by_name("sort_menu").unwrap())
            .unwrap_or_default();
    assert!(
        line[f + "絞り込み".len()..s]
            .replace('│', "")
            .trim()
            .trim_end_matches(sort_key.as_str())
            .trim()
            .is_empty(),
        "隣り合う: {line:?}"
    );
}

// ---- 開く・隠す・ボタンの数・帯 ----

#[test]
fn test_nv_29_click_opens_window_and_hide_done() {
    // [NV-29] 「絞り込み」を押す → 窓が開き「+ 絞り込みを足す」。status の done を隠す → すぐ done の行が消え、
    // ボタンに「絞り込み 1」、上の帯に「status: done を除く」。設定のフィルターそのものが変わる(NV-20)。
    let (_t, mut a) = boot("nv29open");
    assert_eq!(a.rows.len(), 6);
    click_filter_button(&mut a);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert!(scr(&a).contains("+ 絞り込みを足す"), "{}", scr(&a));
    assert!(a.settings.filters.is_empty());
    hide_done(&mut a);
    assert_eq!(a.settings.filters, [drop_done()]);
    assert_eq!(names(&a), ["d.md", "e.md", "f.md"], "すぐ効く");
    assert_eq!(a.mode, Mode::FilterWin, "決めたら条件の並びに戻る");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    let s = scr(&a);
    let (_, line) = bar_line(&a);
    assert!(line.contains("絞り込み 1"), "{line:?}");
    assert!(
        line.find("絞り込み 1").unwrap() < line.find("並べ替え").unwrap(),
        "並べ替えは絞り込みの右のまま: {line:?}"
    );
    assert!(s.contains("status: done を除く"), "{s}");
}

#[test]
fn test_nv_29_compare_adds_and_remove_restores() {
    // [NV-29] done を隠した上に priority の `≥ 3` を足す → 両方を満たす行だけ、ボタンは「絞り込み 2」。
    // 窓で条件を外す(d)→ 外した分だけ戻り、全部外すと元の行。
    let (_t, mut a) = boot("nv29cmp");
    click_filter_button(&mut a);
    hide_done(&mut a);
    start_add(&mut a, "priority", "比べる");
    select(&mut a, "≥");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText, "値を打つ段:\n{}", scr(&a));
    typing(&mut a, "3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterWin);
    assert_eq!(
        a.settings.filters,
        [
            drop_done(),
            Cond {
                col: "priority".to_string(),
                op: Op::Cmp(CmpOp::Ge, "3".to_string()),
            }
        ]
    );
    assert_eq!(names(&a), ["d.md", "f.md"], "両方を満たす行だけ");
    assert!(bar_line(&a).1.contains("絞り込み 2"), "{}", scr(&a));
    // status の条件を外す。
    select(&mut a, "status");
    ch(&mut a, 'd');
    assert_eq!(a.settings.filters.len(), 1);
    assert_eq!(a.settings.filters[0].col, "priority");
    assert_eq!(names(&a), ["a.md", "c.md", "d.md", "f.md"]);
    // 残りも外す → 元の行。ボタンは「絞り込み」に戻る(数が無い)。
    select(&mut a, "priority");
    ch(&mut a, 'd');
    assert!(a.settings.filters.is_empty());
    assert_eq!(a.rows.len(), 6, "元の行");
    assert_eq!(a.mode, Mode::FilterWin, "窓は開いたまま");
    press(&mut a, KeyCode::Esc);
    let (_, line) = bar_line(&a);
    assert!(line.contains("絞り込み"), "{line:?}");
    assert!(!line.contains("絞り込み 1"), "{line:?}");
}

#[test]
fn test_nv_29_bad_operand_is_refused_with_reason() {
    // [NV-29] 比べる値が列の型(数)で読めない → 理由を出し、条件を足さない(値を打つ段のまま)。
    // 打ち直せば足せる。値が空の Enter も足さない。
    let (_t, mut a) = boot("nv29bad");
    click_filter_button(&mut a);
    start_add(&mut a, "priority", "比べる");
    select(&mut a, "≥");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::FilterText);
    press(&mut a, KeyCode::Enter);
    assert!(a.message.is_some(), "空の値は理由を出す");
    assert!(a.settings.filters.is_empty());
    typing(&mut a, "abc");
    press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("abc") && m.contains("数"), "理由: {m:?}");
    assert!(a.settings.filters.is_empty(), "条件を足さない");
    assert_eq!(a.rows.len(), 6);
    assert_eq!(a.mode, Mode::FilterText, "打ち直せる");
    for _ in 0..3 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "4");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.settings.filters.len(), 1);
    assert_eq!(names(&a), ["a.md", "c.md", "d.md"]);
}

#[test]
fn test_nv_29_contains_and_empty_kinds() {
    // [NV-29] 種類の「含む」は値を打って Enter、「空である」はすぐ足す。値を打つ段の Esc は足さずに窓へ戻る。
    let (_t, mut a) = boot("nv29kinds");
    click_filter_button(&mut a);
    start_add(&mut a, "status", "含む");
    assert_eq!(a.mode, Mode::FilterText);
    typing(&mut a, "zzz");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::FilterWin, "Esc は窓へ戻る");
    assert!(a.settings.filters.is_empty());
    start_add(&mut a, "status", "含む");
    typing(&mut a, "ing");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.settings.filters,
        [Cond {
            col: "status".to_string(),
            op: Op::Contains("ing".to_string()),
        }]
    );
    assert_eq!(names(&a), ["f.md"]);
    select(&mut a, "status");
    ch(&mut a, 'd');
    start_add(&mut a, "memo", "空である");
    assert_eq!(
        a.settings.filters,
        [Cond {
            col: "memo".to_string(),
            op: Op::Empty,
        }]
    );
    assert_eq!(a.rows.len(), 5, "memo の無い行だけ");
}

// ---- 閉じる ----

#[test]
fn test_nv_29_close_by_outside_click_and_esc() {
    // [NV-29] 窓の外のクリックで閉じる。Esc でも閉じる。閉じても足した条件は効いたまま。
    let (_t, mut a) = boot("nv29close");
    click_filter_button(&mut a);
    assert_eq!(a.mode, Mode::FilterWin);
    a.click(0, a.size.1 - 3);
    assert_eq!(a.mode, Mode::Table, "窓の外のクリック:\n{}", scr(&a));
    assert!(!scr(&a).contains("+ 絞り込みを足す"), "{}", scr(&a));
    click_filter_button(&mut a);
    hide_done(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(!scr(&a).contains("+ 絞り込みを足す"), "{}", scr(&a));
    assert_eq!(a.rows.len(), 3, "閉じても効いたまま");
}

// ---- パレット・操作の一覧・狭い端末 ----

#[test]
fn test_nv_29_palette_and_menu_open_window() {
    // [NV-29] パレットの動作 filter_menu(読むだけでも出る)。検索の欄を出さない設定では操作の一覧からも開ける。
    let act = filter_menu();
    assert_eq!(act.name(), "filter_menu");
    assert!(
        super::keymap::commands(false).any(|c| c.action == act),
        "パレットに絞り込み"
    );
    assert!(
        super::keymap::commands(true).any(|c| c.action == act),
        "読むだけでもパレットに絞り込み"
    );
    let tmp = notes_tmp("nv29pal");
    let mut a = boot_with_state(&tmp, "[display]\nsearch_bar = false\n", false);
    assert!(super::menu::items(&a).iter().any(|i| i.action == act));
    a.apply(act);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    assert!(scr(&a).contains("+ 絞り込みを足す"), "{}", scr(&a));
    hide_done(&mut a);
    assert_eq!(a.rows.len(), 3);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_nv_29_narrow_terminal_hides_button_but_palette_opens() {
    // [NV-29] 端末が狭く、ボタンが入らない → 「絞り込み」のボタンを出さない(右の端を押しても窓は開かない)。
    // パレットの filter_menu では開ける。
    let (_t, mut a) = boot("nv29narrow");
    a.resize(30, 24);
    a.filter = Some("x".repeat(40));
    let s = scr(&a);
    let (y, _) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("検索:"))
        .unwrap_or_else(|| panic!("検索の欄:\n{s}"));
    let line = s.lines().nth(y).unwrap();
    assert!(!line.contains("絞り込み"), "ボタンを出さない: {line:?}");
    // 右の端の「並べ替え」の場所に「絞り込み」が出ていないことを、各桁のクリックで確かめる。
    // 欄に入ったら Enter で語を残して抜ける(Esc は語を消して欄が短くなり、ボタンが入ってしまう)。
    for x in 0..30u16 {
        a.filter = Some("x".repeat(40));
        a.click(x, y as u16);
        assert_ne!(a.mode, Mode::FilterWin, "桁 {x} で窓が開いた");
        if a.mode == Mode::Filter {
            press(&mut a, KeyCode::Enter);
        } else if a.mode != Mode::Table {
            press(&mut a, KeyCode::Esc);
        }
    }
    a.apply(filter_menu());
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
}

// ---- 見た目の状態・読むだけ ----

#[test]
fn test_nv_29_kept_after_restart() {
    // [NV-29][SR-12] 窓で足した条件は、終了して開き直しても同じ(ボタンも「絞り込み 1」)。
    let tmp = notes_tmp("nv29keep");
    let mut a = boot_with_state(&tmp, "", false);
    click_filter_button(&mut a);
    hide_done(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.settings.filters, [drop_done()]);
    ch(&mut a, 'q');
    let b = boot_with_state(&tmp, "", false);
    assert_eq!(b.settings.filters, [drop_done()]);
    assert_eq!(names(&b), ["d.md", "e.md", "f.md"]);
    assert!(bar_line(&b).1.contains("絞り込み 1"), "{}", scr(&b));
}

#[test]
fn test_nv_29_readonly_filters_without_writing() {
    // [NV-29] 読むだけの起動でも絞れる。ノートも状態も書かない。
    let tmp = notes_tmp("nv29ro");
    let before: Vec<Vec<u8>> = NOTES
        .iter()
        .map(|(n, _)| std::fs::read(tmp.notes().join(n)).unwrap())
        .collect();
    let mut a = boot_with_state(&tmp, "", true);
    click_filter_button(&mut a);
    assert_eq!(a.mode, Mode::FilterWin, "{}", scr(&a));
    hide_done(&mut a);
    assert_eq!(names(&a), ["d.md", "e.md", "f.md"]);
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 's');
    ch(&mut a, 'q');
    let after: Vec<Vec<u8>> = NOTES
        .iter()
        .map(|(n, _)| std::fs::read(tmp.notes().join(n)).unwrap())
        .collect();
    assert_eq!(after, before, "ノートを書かない");
    assert!(
        !tmp.0.join("state").exists()
            || std::fs::read_dir(tmp.0.join("state"))
                .unwrap()
                .next()
                .is_none(),
        "状態を書かない"
    );
}

// ---- 呼び名(NV-1・NV-2) ----

#[test]
fn test_nv_29_names_find_and_search() {
    // [NV-1][NV-2] `/` の呼び名は「探す」(ヘルプと下の帯)、上の欄で打っている間の下の帯のモード名は「検索」。
    assert_eq!(Mode::Search.label(), "探す");
    assert_eq!(Mode::Filter.label(), "検索");
    let (_t, mut a) = boot("nv29names");
    ch(&mut a, '/');
    assert_eq!(a.mode, Mode::Search);
    let s = scr(&a);
    assert!(s.contains("探す"), "下の帯に「探す」:\n{s}");
    press(&mut a, KeyCode::Esc);
    ch(&mut a, '\\');
    assert_eq!(a.mode, Mode::Filter);
    let s = scr(&a);
    let band = s.lines().nth(BAND).unwrap_or_default();
    assert!(band.contains("検索"), "下の帯のモード名: {band:?}\n{s}");
    assert!(!band.contains("絞り込み"), "{band:?}");
    press(&mut a, KeyCode::Esc);
    // ヘルプで `/` の動作は「探す」(ヘルプは長いので、下へ流しながら `/` の行を探す)。
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let found = (0..300).any(|_| {
        let hit = scr(&a)
            .lines()
            .any(|l| l.trim_start().starts_with("/ ") && l.contains("探す"));
        ch(&mut a, 'j');
        hit
    });
    assert!(found, "ヘルプの `/`:\n{}", scr(&a));
}

#[test]
fn test_nv_29_english_bar_lead_is_search() {
    // [NV-23][NV-2] 英語の画面では、上の欄の頭は Search、`/` は Find。
    let _g = mdgrid::i18n::scoped(mdgrid::i18n::Lang::En);
    let (_t, a) = boot("nv29en");
    let s = scr(&a);
    let (_, bar) = s
        .lines()
        .enumerate()
        .take(view::data_y(&a))
        .find(|(_, l)| l.contains("Search"))
        .unwrap_or_else(|| panic!("検索の欄:\n{s}"));
    assert!(bar.contains("\\ Search…"), "{bar:?}\n{s}");
    assert!(!bar.contains("検索"), "{bar:?}\n{s}");
    assert_eq!(Mode::Search.label(), "Find");
    assert_eq!(Mode::Filter.label(), "Search");
}
