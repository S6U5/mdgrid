//! ビューのタブが帯の行に入らないとき、帯の上に行を足して折り返す設定 `display.tab_wrap`(NV-31)の受け入れ試験。
//! 記録: specs/_proposals/2026-10-15-tab-wrap.md。
//!
//! 内部の状態のフィールドや新しい関数には頼らず、設定の文字・端末の大きさ・クリック・描いた画面の文字・
//! 今のビューの名前で確かめる。置いた仮定:
//! - ビューの帯の真ん中の行は、検索の欄「\ 検索」のある行(NV-23)。足した行はその行より上にある。
//! - タブの字はビューの名前を含む(今のタブは「[名前]」、ほかは「 名前 」)。
//! - 表の1行目の高さは、ノートの title の字のある行で見る。

use super::startup::Startup;
use super::test_screen::{text, Tmp};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

// ---- 材料 ----

/// タブの名前(`.base` のビューの順)。今のビューは先頭の「未完了」。
const VIEWS: &[&str] = &[
    "未完了",
    "todo",
    "doing",
    "waiting",
    "deferred",
    "完了",
    "やめた",
];

/// ビューが7つの `.base`。「未完了」は絞り込まず全部、ほかは status が同じ名前の行だけ。
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

/// ノートは status ごとに1つ。title は表の行を画面から探す印(ビューの名前と重ならない字)。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 会議のメモ\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: 買い物\nstatus: doing\n---\n"),
    ("c.md", "---\ntitle: 返事待ち\nstatus: waiting\n---\n"),
    ("d.md", "---\ntitle: 先送り\nstatus: deferred\n---\n"),
    ("e.md", "---\ntitle: 掃除\nstatus: 完了\n---\n"),
    ("f.md", "---\ntitle: 中止\nstatus: やめた\n---\n"),
];

/// 設定の文字を渡し、7つのビューの `.base` を幅 `w`・高さ 24 で開く。
fn boot(name: &str, toml: &str, w: u16) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
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

const WRAP: &str = "[display]\ntab_wrap = true\n";
const NO_WRAP: &str = "[display]\ntab_wrap = false\n";

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

/// その文字を含む最初の行の番号。
fn row_of(s: &str, t: &str) -> usize {
    s.lines()
        .position(|l| l.contains(t))
        .unwrap_or_else(|| panic!("「{t}」が無い:\n{s}"))
}

/// ビューの帯の真ん中の行(検索の欄のある行)。
fn search_row(s: &str) -> usize {
    row_of(s, "\\ 検索")
}

/// 帯の真ん中の行とその上の行(ヘッダーと、足した行)だけを見る(表の status の字とタブを取り違えないように)。
fn bar_part(s: &str) -> Vec<String> {
    s.lines()
        .take(search_row(s) + 1)
        .map(|l| l.to_string())
        .collect()
}

/// 帯とその上で、タブの名前 `name` の字のある行(「完了」は「未完了」と分けるため、タブの字の形で探す)。
fn tab_row(s: &str, name: &str) -> Option<usize> {
    let shapes = [format!("[{name}]"), format!(" {name} ")];
    bar_part(s)
        .iter()
        .position(|l| shapes.iter().any(|t| format!("{l} ").contains(t.as_str())))
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

fn current(a: &App) -> String {
    a.view_names()[a.view_index()].clone()
}

// ---- 試験 ----

#[test]
fn test_nv_31_narrow_wraps_all_tabs() {
    // [NV-31] tab_wrap = true で幅 60 → 帯の上に行が足され、7つのタブの名前が全部見え、検索の欄と「絞り込み」も
    // 見える。足した行のぶん表は下がる。
    let (_t, a) = boot("nv31_wrap", WRAP, 60);
    let s = scr(&a);
    for v in VIEWS {
        assert!(tab_row(&s, v).is_some(), "タブ「{v}」が帯に無い:\n{s}");
    }
    let mid = search_row(&s);
    assert!(
        s.lines().nth(mid).unwrap().contains("絞り込み"),
        "帯の真ん中の行に「絞り込み」が無い:\n{s}"
    );
    // 足した行(帯の真ん中の行より上)にタブがある。
    assert!(
        VIEWS
            .iter()
            .any(|v| tab_row(&s, v).is_some_and(|r| r < mid)),
        "帯の上の行にタブが無い:\n{s}"
    );

    // 折り返さないとき(false)と比べて、帯の真ん中の行と表の1行目が同じ行の数だけ下がる。
    let (_t2, b) = boot("nv31_wrap_off", NO_WRAP, 60);
    let s2 = scr(&b);
    let added = mid
        .checked_sub(search_row(&s2))
        .unwrap_or_else(|| panic!("帯が上がった:\n{s}\n----\n{s2}"));
    assert!(added >= 1, "帯の上に行が足されていない:\n{s}\n----\n{s2}");
    assert_eq!(
        row_of(&s, "会議のメモ"),
        row_of(&s2, "会議のメモ") + added,
        "表は足した行のぶん下がる:\n{s}\n----\n{s2}"
    );
}

#[test]
fn test_nv_31_remaining_tabs_left_of_bar() {
    // [NV-31] 折り返す幅(60〜140 を 10 刻み)のどれでも、7つのタブが全部見え、足した行は先頭のタブ(今のビュー
    // 「[未完了]」)から左の端に並び、帯の真ん中の行に残ったタブは検索の欄より左にある。真ん中の行にタブが
    // 残る幅が1つはある(幅 60 では検索の欄とボタンで埋まり、全部が上の行に行く)。
    let mut kept_somewhere = false;
    let mut wrapped_somewhere = false;
    for w in (60..=140).step_by(10) {
        let (_t, a) = boot(&format!("nv31_rest_{w}"), WRAP, w);
        let s = scr(&a);
        let mid = search_row(&s);
        let line = s.lines().nth(mid).unwrap();
        let at_search = line.find("\\ 検索").unwrap();
        for v in VIEWS {
            assert!(tab_row(&s, v).is_some(), "幅 {w}: タブ「{v}」が無い:\n{s}");
        }
        let first = tab_row(&s, "未完了").unwrap();
        if first < mid {
            wrapped_somewhere = true;
            assert!(
                s.lines().nth(first).unwrap().starts_with("[未完了]"),
                "幅 {w}: 足した行は左の端から並ぶ:\n{s}"
            );
        }
        for v in VIEWS.iter().filter(|v| tab_row(&s, v) == Some(mid)) {
            assert!(
                line.find(v).unwrap() < at_search,
                "幅 {w}: 帯の行のタブ「{v}」が検索の欄より右:\n{s}"
            );
            if first < mid {
                kept_somewhere = true;
            }
        }
    }
    assert!(wrapped_somewhere, "どの幅でも折り返していない");
    assert!(
        kept_somewhere,
        "折り返したとき、帯の真ん中の行にタブが残る幅が無い"
    );
}

#[test]
fn test_nv_31_click_wrapped_tab_selects_view() {
    // [NV-31] 足した行のタブをクリック → そのビューになる。
    let (_t, mut a) = boot("nv31_click", WRAP, 60);
    let s = scr(&a);
    let mid = search_row(&s);
    let target = VIEWS[1..]
        .iter()
        .find(|v| tab_row(&s, v).is_some_and(|r| r < mid))
        .unwrap_or_else(|| panic!("足した行に今のでないタブが無い:\n{s}"));
    let y = tab_row(&s, target).unwrap() as u16;
    let x = col_in(&a, y, target).unwrap_or_else(|| panic!("行 {y} に「{target}」が無い:\n{s}"));
    assert_eq!(current(&a), "未完了");
    a.click(x, y);
    assert_eq!(current(&a), *target, "{}", scr(&a));
}

#[test]
fn test_nv_31_wide_same_as_off() {
    // [NV-31] 幅 200 ではタブが入りきるので、tab_wrap = true でも false と同じ画面(足す行は無い)。
    let (_t, a) = boot("nv31_wide_on", WRAP, 200);
    let (_t2, b) = boot("nv31_wide_off", NO_WRAP, 200);
    let s = scr(&a);
    assert_eq!(s, scr(&b));
    let mid = search_row(&s);
    for v in VIEWS {
        assert_eq!(
            tab_row(&s, v),
            Some(mid),
            "タブ「{v}」は帯の真ん中の行:\n{s}"
        );
    }
}

#[test]
fn test_nv_31_off_keeps_current_tab_only() {
    // [NV-31・NV-23] tab_wrap = false(既定)で幅 60 → 今のタブだけ(今のでないタブは省く)。既定の設定も同じ。
    for (name, toml) in [("nv31_off", NO_WRAP), ("nv31_default", "")] {
        let (_t, a) = boot(name, toml, 60);
        let s = scr(&a);
        let mid = search_row(&s);
        assert_eq!(tab_row(&s, "未完了"), Some(mid), "{s}");
        for v in &VIEWS[1..] {
            assert_eq!(tab_row(&s, v), None, "タブ「{v}」は省かれる:\n{s}");
        }
    }
}
