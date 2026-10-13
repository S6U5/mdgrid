//! 設定を右上の別のボタンに、画面の型をメニューに、「+ 新規」をビューの帯に移し、ボタンのキーの字を
//! 目立たせる(SR-1・SR-42・NV-23・REL-8・CE-25)受け入れの試験。
//! 記録: specs/_changes/2026-10-12-header-menu.md(タスク #1)。
//!
//! 新しいモードの名前には頼らず、モード(表・関係マップ・設定)・描いた画面の文字・見た目で確かめる。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, make, press, text, Tmp};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

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

/// 行 y の文字 `t` をクリックする。
fn click_in(a: &mut App, y: u16, t: &str) {
    let x = col_in(a, y, t).unwrap_or_else(|| panic!("行 {y} に「{t}」が無い:\n{}", scr(a)));
    a.click(x, y);
}

/// 画面のどこかの行で、1行目より下にある文字 `t` をクリックする(窓の中の項目)。
fn click_below(a: &mut App, t: &str) {
    let h = a.size.1;
    let y = (head(a) + 1..h)
        .find(|&y| col_in(a, y, t).is_some())
        .unwrap_or_else(|| panic!("「{t}」が無い:\n{}", scr(a)));
    click_in(a, y, t);
}

/// つながる表のある tasks を開く(幅 120)。
fn boot_linked(name: &str) -> (Tmp, App) {
    let tmp = workspace(name);
    let a = boot_tasks(&tmp);
    (tmp, a)
}

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: done\n---\n"),
    ("b.md", "---\nstatus: todo\n---\n"),
];

// ---- SR-42: 右上の画面の型のメニューと設定のボタン ----

#[test]
fn test_sr_42_settings_top_right_after_menu() {
    // [SR-42][SR-1] 1行目の右の端に「▦ 表 ▾」と「o ⚙ 設定」がこの順。「+ 新規」は1行目に無い。
    let (_t, a) = boot_linked("hmtopright");
    let h = line(&a, head(&a));
    let m = h.find("▦ 表 ▾").unwrap_or_else(|| panic!("{h:?}"));
    let s = h.find("o ⚙ 設定").unwrap_or_else(|| panic!("{h:?}"));
    assert!(m < s, "メニューの右に設定: {h:?}");
    assert!(h.trim_end().ends_with("o ⚙ 設定"), "設定が右の端: {h:?}");
    assert!(!h.contains("新規"), "「+ 新規」は1行目に無い: {h:?}");
}

#[test]
fn test_sr_42_settings_button_opens_settings() {
    // [SR-42] 「⚙ 設定」をクリック → ビューの設定の画面。関係マップから押しても、表の画面に戻って開く。
    let (_t, mut a) = boot_linked("hmsettings");
    click_head(&mut a, "設定");
    assert_eq!(a.mode, Mode::Settings, "{}", scr(&a));
    let (_t, mut a) = boot_linked("hmsettingsrel");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    click_head(&mut a, "設定");
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert_eq!(a.mode, Mode::Settings, "{}", scr(&a));
}

#[test]
fn test_sr_42_settings_dropped_first_when_narrow() {
    // [SR-42] 幅 60 → 設定のボタンが無く、パンくずと「表 ▾」は出る。
    let (_t, mut a) = boot_linked("hmnarrow");
    a.resize(60, 24);
    let h = line(&a, head(&a));
    assert!(!h.contains("設定"), "{h:?}");
    assert!(h.contains("表 ▾"), "{h:?}");
    assert!(h.contains("tasks ▾"), "{h:?}");
}

// ---- REL-8: 画面の型のメニュー ----

#[test]
fn test_rel_8_menu_lists_types_and_switches() {
    // [REL-8] 「▦ 表 ▾」を押す → 窓に「表」と「関係マップ」。関係マップを選ぶ → 関係マップの画面で、
    // ボタンは「関係マップ ▾」。
    let (_t, mut a) = boot_linked("hmmenu");
    click_head(&mut a, "表 ▾");
    assert_ne!(a.mode, Mode::Table, "窓が開く:\n{}", scr(&a));
    let s = scr(&a);
    let below: String = s.lines().skip(1).collect::<Vec<_>>().join("\n");
    assert!(below.contains("関係マップ"), "窓に関係マップ:\n{s}");
    click_below(&mut a, "関係マップ");
    assert_eq!(a.mode, Mode::Relations, "{}", scr(&a));
    assert!(line(&a, head(&a)).contains("関係マップ ▾"), "{}", scr(&a));
}

#[test]
fn test_rel_8_menu_esc_closes_without_change() {
    // [REL-8] 窓を開いて Esc → 窓が閉じ、表の画面のまま。
    let (_t, mut a) = boot_linked("hmesc");
    click_head(&mut a, "表 ▾");
    assert_ne!(a.mode, Mode::Table);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.relmap.is_none());
}

#[test]
fn test_rel_8_menu_back_to_table() {
    // [REL-8] 関係マップで「関係マップ ▾」を押して「表」を選ぶ → 表の画面。R のキーは窓なしで切り替える。
    let (_t, mut a) = boot_linked("hmback");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "R は窓なしで切り替える");
    click_head(&mut a, "関係マップ ▾");
    assert_ne!(a.mode, Mode::Relations, "窓が開く:\n{}", scr(&a));
    let h = a.size.1;
    let y = (head(&a) + 1..h)
        .find(|&y| {
            let l = line(&a, y);
            // 窓の中の「表」だけの項目(2026-10-12-relmap-new-row で、窓は関係マップの窓の上の縁と同じ行に
            // 重なりうるので、縁の題「関係マップ」の有無では見分けない)。
            l.split('│').any(|seg| seg.trim() == "表")
        })
        .unwrap_or_else(|| panic!("窓に「表」:\n{}", scr(&a)));
    click_in(&mut a, y, "表");
    assert_eq!(a.mode, Mode::Table, "{}", scr(&a));
    assert!(a.relmap.is_none());
}

#[test]
fn test_rel_8_menu_without_links_has_only_table() {
    // [REL-8] つながる表が無い → ボタンは出て、窓には「表」だけ(関係マップは無い)。
    let (_t, mut a) = make("hmnolink", NOTES);
    assert!(line(&a, head(&a)).contains("表 ▾"), "{}", scr(&a));
    click_head(&mut a, "表 ▾");
    assert!(!scr(&a).contains("関係マップ"), "{}", scr(&a));
}

// ---- CE-25: ビューの帯の右の端の「+ 新規」 ----

#[test]
fn test_ce_25_new_in_view_bar_right_end() {
    // [CE-25][NV-23][SR-1] ビューの帯(2026-10-12-search-box・view-bar-plain の3行の帯では真ん中の行)の右の端に
    // 「a + 新規」。クリック → 名前の欄が開く。
    let (_t, mut a) = make("hmnew", NOTES);
    let y = mid(&a);
    let bar = line(&a, y);
    assert!(bar.trim_end().ends_with("a + 新規"), "{bar:?}");
    click_in(&mut a, y, "新規");
    assert_ne!(a.mode, Mode::Table, "名前の欄:\n{}", scr(&a));
}

#[test]
fn test_ce_25_new_same_place_in_relations() {
    // [CE-25] 関係マップでも「+ 新規」は表の画面と同じ行(ビューの帯の真ん中の行。2026-10-12-relmap-new-row)・同じ桁。
    let (_t, mut a) = boot_linked("hmnewrel");
    let x = col_in(&a, mid(&a), "+ 新規").unwrap_or_else(|| panic!("{}", scr(&a)));
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    assert_eq!(col_in(&a, mid(&a), "+ 新規"), Some(x), "{}", scr(&a));
}

// ---- NV-23: ビューの帯に設定は無く、キーの字が目立つ ----

#[test]
fn test_nv_23_no_settings_in_view_bar() {
    // [NV-23] ビューの帯に ⚙ も「設定」も無い。分類・列は残る。
    let (_t, a) = make("hmnogear", NOTES);
    let bar = line(&a, mid(&a));
    assert!(!bar.contains('⚙') && !bar.contains("設定"), "{bar:?}");
    for t in ["分類", "列"] {
        assert!(bar.contains(t), "{bar:?}");
    }
}

#[test]
fn test_nv_23_key_char_looks_different() {
    // [NV-23][SR-42] ボタンの頭のキーの字は太字で、名前の字と見た目が違う(ビューの帯の S、ヘッダーの o・w)。
    let (_t, a) = boot_linked("hmkey");
    let b = buf(&a);
    for (y, t) in [
        (mid(&a), "S 並べ替え"),
        (head(&a), "o ⚙ 設定"),
        (head(&a), "w "),
    ] {
        let x = col_in(&a, y, t).unwrap_or_else(|| panic!("行 {y} に「{t}」:\n{}", scr(&a)));
        let key = b[(x, y)].style();
        let name = b[(x + 2, y)].style();
        assert!(
            key.add_modifier.contains(ratatui::style::Modifier::BOLD),
            "「{t}」のキーは太字"
        );
        assert_ne!(key, name, "「{t}」のキーと名前は見た目が違う");
    }
}

/// ビューの帯の字の行(2026-10-12-search-box・view-bar-plain の3行の帯では3行目、1行の帯では2行目)。
fn mid(a: &App) -> u16 {
    super::bands::mid_row(a) as u16
}

/// ヘッダーの行(2026-10-12-bar-padding-half の上の余白の分だけ下がる)。
fn head(a: &App) -> u16 {
    super::bands::head_y(a)
}

/// ヘッダーの行の文字 `t` をクリックする。
fn click_head(a: &mut App, t: &str) {
    let y = head(a);
    click_in(a, y, t);
}
