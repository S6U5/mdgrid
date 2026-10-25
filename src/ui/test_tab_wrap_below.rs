//! 下に並べたタブの行(NV-31)の置き場と、検索の欄の行との間の空き行の設定 `display.tab_wrap_gap` の受け入れ試験。
//! 記録: specs/_changes/2026-10-15-tab-wrap-below.md。
//!
//! 内部の状態のフィールドには頼らず、設定の文字・端末の大きさ・クリック・描いた画面の文字で確かめる。置いた仮定:
//! - ビューの帯の真ん中の行は、検索の欄「\ 検索」のある行(NV-23)。
//! - タブの字はビューの名前を含む(今のタブは「[名前]」、ほかは「 名前 」)。
//! - 表の1行目は、ノートの title「会議のメモ」の字のある行。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const VIEWS: &[&str] = &[
    "未完了",
    "期限切れ",
    "今日まで",
    "7日以内まで",
    "todo",
    "doing",
    "waiting",
    "deferred",
    "完了",
    "やめた",
];

fn base_text() -> String {
    let mut s =
        String::from("views:\n  - type: table\n    name: 未完了\n    order: [title, status]\n");
    for v in &VIEWS[1..] {
        s.push_str(&format!(
            "  - type: table\n    name: {v}\n    filters: 'status == \"{v}\"'\n    order: [title, status]\n"
        ));
    }
    s
}

/// 10個のビューの `tasks.base` を、設定の文字で幅 `w`・高さ 24 で開く。
fn boot(name: &str, toml: &str, w: u16) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ntitle: 会議のメモ\nstatus: todo\n---\n");
    tmp.write("b.md", "---\ntitle: 買い物\nstatus: doing\n---\n");
    let p = tmp.notes().join("tasks.base");
    std::fs::write(&p, base_text()).unwrap();
    let (config, warns) = mdgrid::config::parse(toml).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(w, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p.clone(),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn scr(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

fn row_of(s: &str, t: &str) -> usize {
    s.lines()
        .position(|l| l.contains(t))
        .unwrap_or_else(|| panic!("「{t}」が無い:\n{s}"))
}

fn has_tab(line: &str) -> bool {
    let l = format!("{line} ");
    VIEWS
        .iter()
        .any(|v| l.contains(&format!("[{v}]")) || l.contains(&format!(" {v} ")))
}

/// 表の1行目より上で、タブのある行の番号。
fn tab_lines(s: &str) -> Vec<usize> {
    (0..row_of(s, "会議のメモ"))
        .filter(|&r| has_tab(s.lines().nth(r).unwrap()))
        .collect()
}

const WRAP: &str = "[display]\ntab_wrap = true\n";
const GAP: &str = "[display]\ntab_wrap = true\ntab_wrap_gap = true\n";

#[test]
fn test_nv_31_tabs_right_below_search_by_default() {
    // [NV-31] 囲まない帯(既定)・余白 1・幅 60 で折り返す → タブの行は検索の欄の行のすぐ下から間を空けずに続き、
    // 検索の欄の行とヘッダーの間は余白の1行。ほかのタブの行の下は余白の空き行。
    let (_t, a) = boot("nv31_below_plain", WRAP, 60);
    let s = scr(&a);
    let mid = row_of(&s, "\\ 検索");
    let rows = tab_lines(&s);
    assert!(rows.len() >= 2, "タブが2行以上にならない:\n{s}");
    assert_eq!(
        rows,
        (mid + 1..mid + 1 + rows.len()).collect::<Vec<_>>(),
        "{s}"
    );
    assert_eq!(row_of(&s, "tasks") + 2, mid, "{s}");
    let after = rows.last().unwrap() + 1;
    assert_eq!(s.lines().nth(after).unwrap().trim(), "", "{s}");
}

#[test]
fn test_nv_31_gap_puts_one_empty_row() {
    // [NV-31] tab_wrap_gap = true → 検索の欄の行とタブの行の間に空き行が1つ。表は既定よりちょうど1行下がる。
    let (_t, a) = boot("nv31_gap_on", GAP, 60);
    let (_t2, b) = boot("nv31_gap_off", WRAP, 60);
    let s = scr(&a);
    let s2 = scr(&b);
    let mid = row_of(&s, "\\ 検索");
    let rows = tab_lines(&s);
    assert_eq!(rows[0], mid + 2, "{s}");
    assert_eq!(s.lines().nth(mid + 1).unwrap().trim(), "", "{s}");
    assert_eq!(rows.len(), tab_lines(&s2).len(), "{s}\n----\n{s2}");
    assert_eq!(
        row_of(&s, "会議のメモ"),
        row_of(&s2, "会議のメモ") + 1,
        "{s}\n----\n{s2}"
    );
}

#[test]
fn test_nv_31_gap_without_wrap_changes_nothing() {
    // [NV-31] 折り返さないとき(幅 200、または tab_wrap = false)は tab_wrap_gap があっても空き行を入れない。
    for (w, gap, plain) in [
        (200, GAP, WRAP),
        (
            60,
            "[display]\ntab_wrap_gap = true\n",
            "[display]\ntab_wrap = false\n",
        ),
    ] {
        let (_t, a) = boot("nv31_gap_nowrap", gap, w);
        let (_t2, b) = boot("nv31_gap_nowrap_ref", plain, w);
        assert_eq!(scr(&a), scr(&b));
    }
}

#[test]
fn test_nv_31_zero_padding_tabs_below() {
    // [NV-31] 余白 0 → 検索の欄の行はヘッダーのすぐ下、タブの行はそのすぐ下。
    let (_t, a) = boot(
        "nv31_below_zero",
        "[display]\ntab_wrap = true\nbar_padding = 0\n",
        60,
    );
    let s = scr(&a);
    let mid = row_of(&s, "\\ 検索");
    assert_eq!(row_of(&s, "tasks") + 1, mid, "{s}");
    assert_eq!(tab_lines(&s)[0], mid + 1, "{s}");
}

#[test]
fn test_nv_31_click_tab_below_after_gap() {
    // [NV-31] 余白 2・tab_wrap_gap で、下に並べたタブをクリック → そのビュー。空き行のクリックでは変わらない。
    let (_t, mut a) = boot(
        "nv31_below_click",
        "[display]\ntab_wrap = true\ntab_wrap_gap = true\nbar_padding = 2\n",
        60,
    );
    let s = scr(&a);
    let rows = tab_lines(&s);
    let y = *rows.last().unwrap();
    let line = s.lines().nth(y).unwrap();
    let target = VIEWS[1..]
        .iter()
        .find(|v| format!("{line} ").contains(&format!(" {v} ")))
        .unwrap_or_else(|| panic!("今のでないタブが無い:\n{s}"));
    let x = width::width(&line[..line.find(&format!(" {target} ")).unwrap() + 1]) as u16;
    let before = a.view_names()[a.view_index()].clone();
    a.click(x, (rows[0] - 1) as u16);
    assert_eq!(
        a.view_names()[a.view_index()],
        before,
        "空き行では変わらない"
    );
    a.click(x, y as u16);
    assert_eq!(a.view_names()[a.view_index()], *target, "{}", scr(&a));
}

#[test]
fn test_nv_31_box_tabs_below_bottom_line() {
    // [NV-31] 囲む帯(box)・余白 1 → タブの行は囲みの下の線の行の下(検索の欄の行との間に線の行が1つ)。
    let (_t, a) = boot(
        "nv31_below_box",
        "[display]\ntab_wrap = true\nbuttons = \"box\"\n",
        60,
    );
    let s = scr(&a);
    let mid = row_of(&s, "\\ 検索");
    assert_eq!(tab_lines(&s)[0], mid + 2, "{s}");
    assert!(!has_tab(s.lines().nth(mid + 1).unwrap()), "{s}");
}
