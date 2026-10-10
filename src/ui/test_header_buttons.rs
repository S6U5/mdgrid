//! ヘッダーのワークスペースと設定のボタン(SR-42)。見本は test_relations.rs の notes/(tasks・projects・members)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, screen};
use super::*;
use mdgrid::workspace::{self, WsTable};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 1行目で文字 `t` の始まる桁。
fn col_of(a: &App, t: &str) -> Option<u16> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| super::draw(f, a)).unwrap();
    let b = term.backend().buffer().clone();
    let first: Vec<char> = t.chars().collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        first.iter().all(|c| {
            let ok = xx < b.area.width && b[(xx, 0)].symbol() == c.to_string();
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
    };
    workspace::add(&cfg, "Work", t("Tasks", "tasks")).unwrap();
    workspace::add(&cfg, "Work", t("Projects", "projects")).unwrap();
}

#[test]
fn test_sr_42_settings_button() {
    // [SR-42] 「設定」は表の画面と関係マップで同じ桁。押すとビューの設定の画面(関係マップからは表に戻ってから)。
    let tmp = workspace("sr42_settings");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let x = col_of(&a, "設定").expect("表の画面の設定");
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    assert_eq!(col_of(&a, "設定"), Some(x), "関係マップでも同じ桁");
    a.click(x, 0);
    assert!(a.relmap.is_none());
    assert_eq!(a.mode, Mode::Settings, "{}", screen(&a));
}

#[test]
fn test_sr_42_workspace_button() {
    // [SR-42] ワークスペースの範囲で開いたら、ボタンにその名前。押すとワークスペースの一覧。
    // ワークスペースが1つも無くても、ワークスペースのボタンは出す(WS-6)。
    let tmp = workspace("sr42_ws");
    let a = boot_tasks(&tmp);
    assert!(screen(&a)
        .lines()
        .next()
        .unwrap_or_default()
        .contains("ワークスペース"));
    add_work(&tmp);
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let x = col_of(&a, "▾ Work").expect("ワークスペースの名前のボタン");
    a.click(x + 2, 0);
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    assert!(screen(&a).contains("Work"), "{}", screen(&a));
}

#[test]
fn test_sr_42_narrow_hides_buttons() {
    // [SR-42] 幅が足りなければボタンを隠し、タブと「+ 新規」は残す。
    let tmp = workspace("sr42_narrow");
    add_work(&tmp);
    let mut a = boot_tasks(&tmp);
    a.resize(60, 24);
    let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
    term.draw(|f| super::draw(f, &a)).unwrap();
    let b = term.backend().buffer().clone();
    // 全角の字の後ろの桁は空白なので、空白を除いて比べる。
    let head: String = (0..60)
        .map(|x| b[(x, 0)].symbol().to_string())
        .collect::<String>()
        .replace(' ', "");
    assert!(!head.contains("設定") && !head.contains("▾"), "{head}");
    assert!(head.contains("新規"), "{head}");
}
