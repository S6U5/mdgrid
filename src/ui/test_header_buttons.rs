//! ヘッダーのワークスペースと設定のボタン(SR-42)。見本は test_relations.rs の notes/(tasks・projects・members)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, screen};
use super::*;
use mdgrid::workspace::{self, WsTable};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// ヘッダーの行で文字 `t` の始まる桁。
fn col_of(a: &App, t: &str) -> Option<u16> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| super::draw(f, a)).unwrap();
    let b = term.backend().buffer().clone();
    // ヘッダーの行(2026-10-12-bar-padding-half の上の余白の分だけ下がる)。
    let hy = super::bands::head_y(a);
    let first: Vec<char> = t.chars().collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        first.iter().all(|c| {
            let ok = xx < b.area.width && b[(xx, hy)].symbol() == c.to_string();
            xx += if super::width::width(&c.to_string()) == 2 {
                2
            } else {
                1
            };
            ok
        })
    })
}

fn add_work(tmp: &super::test_screen::Tmp) {
    let cfg = tmp.0.join("config");
    let t = |name: &str, dir: &str| WsTable {
        name: name.into(),
        path: tmp.notes().join(dir).canonicalize().unwrap(),
        view: None,
        profile: Default::default(),
    };
    workspace::add(&cfg, "Work", t("Tasks", "tasks")).unwrap();
    workspace::add(&cfg, "Work", t("Projects", "projects")).unwrap();
}

#[test]
fn test_sr_42_settings_button() {
    // [SR-42] ビューの設定の画面を開くボタンは、ヘッダーの右の端の「⚙ 設定」(表の画面と関係マップの1行目に
    // 出し、2行目のビューの帯には無い。2026-10-12-header-menu で帯から移った)。⚙ を押すとビューの設定の画面。
    let tmp = workspace("sr42_settings");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    // ヘッダーの行(2026-10-12-bar-padding-half の上の余白の分だけ下がる)とビューの帯の字の行。
    let head = |a: &App| {
        screen(a)
            .lines()
            .nth(super::bands::head_y(a) as usize)
            .unwrap_or_default()
            .to_string()
    };
    let bar = |a: &App| {
        screen(a)
            .lines()
            .nth(super::bands::mid_row(a))
            .unwrap_or_default()
            .to_string()
    };
    assert!(head(&a).contains("設定"), "{}", screen(&a));
    assert!(
        !bar(&a).contains('⚙'),
        "ビューの帯に ⚙ は無い:\n{}",
        screen(&a)
    );
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    assert!(
        head(&a).contains("設定"),
        "関係マップの1行目にもある:\n{}",
        screen(&a)
    );
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Table);
    let gear = col_of(&a, "⚙").expect("ヘッダーの ⚙");
    a.click(gear, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Settings, "{}", screen(&a));
}

#[test]
fn test_sr_42_workspace_button() {
    // [SR-42] ワークスペースの範囲で開いたら、パンくずにその名前。押すと表の一覧の引き出し(WS-8)、
    // もう一度押すと閉じる。ワークスペースが1つも無くても、引き出しの入口(パンくずの表の名前)は出す(WS-6)。
    // 前の配置のヘッダーのボタン「▾ Work」「ワークスペース」は、パンくずの「Work」と表の名前に置き換わった。
    let tmp = workspace("sr42_ws");
    let mut a = boot_tasks(&tmp);
    assert!(screen(&a)
        .lines()
        .nth(super::bands::head_y(&a) as usize)
        .unwrap_or_default()
        .contains("tasks"));
    let x = col_of(&a, "tasks").expect("パンくずの表の名前");
    a.click(x + 1, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Drawer, "{}", screen(&a));
    a.click(x + 1, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Table, "{}", screen(&a));
    add_work(&tmp);
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let x = col_of(&a, "Work").expect("パンくずのワークスペースの名前");
    a.click(x + 2, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Drawer, "{}", screen(&a));
    assert!(screen(&a).contains("Projects"), "{}", screen(&a));
    a.click(x + 2, super::bands::head_y(&a));
    assert_eq!(a.mode, Mode::Table, "{}", screen(&a));
    assert!(a.switch_to.is_none());
}

#[test]
fn test_sr_42_narrow_hides_buttons() {
    // [SR-42] 幅が足りなければボタンを隠し、画面の型のメニューと「+ 新規」(2026-10-12-header-menu で
    // ビューの帯に移った)は残す。
    let tmp = workspace("sr42_narrow");
    add_work(&tmp);
    let mut a = boot_tasks(&tmp);
    a.resize(60, 24);
    let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
    term.draw(|f| super::draw(f, &a)).unwrap();
    let b = term.backend().buffer().clone();
    // 全角の字の後ろの桁は空白なので、空白を除いて比べる。
    let hy = super::bands::head_y(&a);
    let head: String = (0..60)
        .map(|x| b[(x, hy)].symbol().to_string())
        .collect::<String>()
        .replace(' ', "");
    // ビューの帯の字の行(2026-10-12-search-box・view-bar-plain の3行の帯では3行目)。
    let mid = super::bands::mid_row(&a) as u16;
    let bar: String = (0..60)
        .map(|x| b[(x, mid)].symbol().to_string())
        .collect::<String>()
        .replace(' ', "");
    // ▾ はパンくずのボタン(SR-42)と画面の型のメニュー(REL-8)の2つだけ。前の右上の「▾ ワークスペース」の
    // ボタンは無い。
    assert!(
        !head.contains("設定") && head.matches('▾').count() == 2,
        "{head}"
    );
    assert!(head.contains("表▾"), "{head}");
    assert!(bar.contains("新規"), "{bar}");
}
