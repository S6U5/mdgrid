//! 関係マップの画面(REL-7・REL-8・REL-9)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, press, text};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// App の大きさ(resize)で描いた画面の文字(段組みは幅で変わるので)。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}
use super::*;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_rel_7_map_shows_boxes_and_arrows() {
    // [REL-7] R で関係マップ: 表の箱(名前・行の数・列)と、つながりの列から行き先の箱への矢印。
    let tmp = workspace("rm_boxes");
    let mut a = boot_tasks(&tmp);
    a.resize(140, 36);
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    let s = screen(&a);
    for want in [
        "Tasks (3)",
        "Projects (2)",
        "project → Projects",
        "assignee → Members",
        "▶",
    ] {
        assert!(s.contains(want), "{want}:\n{s}");
    }
    // 選んだつながりの詳細(元の列・行き先・種類・リンクの数)。
    assert!(s.contains("Tasks.project → Projects"), "{s}");
    // R・Esc で表に戻る。
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Table);
    ch(&mut a, 'R');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_rel_7_select_link_and_open_table() {
    // [REL-7] ←→ でその表のつながりを選び、↓ で表を選び、Enter でその表へ移る。
    let tmp = workspace("rm_open");
    let mut a = boot_tasks(&tmp);
    a.resize(140, 36);
    ch(&mut a, 'R');
    press(&mut a, KeyCode::Right);
    let s = screen(&a);
    let d = s
        .lines()
        .find(|l| l.contains("Details") || l.contains("詳細"))
        .is_some();
    assert!(d, "{s}");
    let sel_name = |a: &App| {
        let rm = a.relmap.as_ref().unwrap();
        rm.infos[rm.sel].name.clone()
    };
    let first = sel_name(&a);
    press(&mut a, KeyCode::Down);
    let next = sel_name(&a);
    assert_ne!(first, next);
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "ほかの表へ移る: {:?}", a.message);
    assert_eq!(a.switch_to.as_ref().map(|p| p.name.clone()), Some(next));
}

#[test]
fn test_rel_8_screen_tabs_switch() {
    // [REL-8] 登録した表があれば、上の端の右に画面の型のタブ。クリックで関係マップ・表を切り替える。
    let tmp = workspace("rm_tabs");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let s = screen(&a);
    let head = s.lines().next().unwrap().to_string();
    assert!(head.contains("[表]") && head.contains(" 関係 "), "{head}");
    let x = mdgrid_width(&head[..head.find(" 関係 ").unwrap()]) + 2;
    a.click(x as u16, 0);
    assert_eq!(a.mode, Mode::Relations, "{}", screen(&a));
    let head = screen(&a).lines().next().unwrap().to_string();
    assert!(head.contains("[関係]"), "{head}");
    let x = mdgrid_width(&head[..head.find(" 表 ").unwrap()]) + 1;
    a.click(x as u16, 0);
    assert_eq!(a.mode, Mode::Table);
}

fn mdgrid_width(s: &str) -> usize {
    super::width::width(s)
}

#[test]
fn test_rel_8_no_tabs_without_places() {
    // [REL-8] 登録した表が無ければタブは出さない(今までのヘッダー)。
    let tmp = super::test_screen::Tmp::new("rm_notabs");
    tmp.write("a.md", "---\nx: 1\n---\n");
    let src = mdgrid::source::markdown::Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    let head = screen(&a).lines().next().unwrap().to_string();
    assert!(!head.contains("[表]") && !head.contains("関係"), "{head}");
}

#[test]
fn test_rel_9_layout_by_width() {
    // [REL-9] 幅 140 は3つ(表・マップ・詳細)、100 は2つ(マップ・詳細)、70 は一覧だけ。どの幅でも行の幅は端末の幅。
    let tmp = workspace("rm_widths");
    let mut a = boot_tasks(&tmp);
    for (w, tables, detail, list) in [
        (140u16, true, true, false),
        (100, false, true, false),
        (70, false, false, true),
    ] {
        a.resize(w, 30);
        ch(&mut a, 'R');
        let s = screen(&a);
        assert_eq!(s.contains("─ 表 ─"), tables, "幅 {w}:\n{s}");
        assert_eq!(s.contains("─ 詳細 ─"), detail, "幅 {w}:\n{s}");
        assert_eq!(
            s.contains("Tasks.project → Projects (N:1)"),
            list,
            "幅 {w}:\n{s}"
        );
        for l in s.lines() {
            assert!(
                super::width::width(l) <= w as usize,
                "幅 {w} を超える行: {l}"
            );
        }
        ch(&mut a, 'R');
    }
}

#[test]
fn test_rel_9_linked_records_when_tall() {
    // [REL-9] 高さ 30 以上なら、選んだつながりのつながった行を下に出す。低ければ出さない。
    let tmp = workspace("rm_tall");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 36);
    ch(&mut a, 'R');
    let s = screen(&a);
    assert!(
        s.contains("つながった行") && s.contains("a build  →  mdgrid"),
        "{s}"
    );
    ch(&mut a, 'R');
    a.resize(120, 24);
    ch(&mut a, 'R');
    assert!(!screen(&a).contains("つながった行"), "{}", screen(&a));
}

#[test]
fn test_rel_7_no_tables_message() {
    // [REL-7] 表が1つも無ければ開かずに理由。
    let tmp = super::test_screen::Tmp::new("rm_none");
    let src = mdgrid::source::markdown::Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("表が無い"),
        "{:?}",
        a.message
    );
}
