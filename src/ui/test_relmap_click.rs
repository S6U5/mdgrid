//! 関係マップのクリックとホイール(REL-12)。見本は test_relations.rs の notes/(tasks・projects・members)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

/// App の大きさで描いた画面。
fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    t.backend().buffer().clone()
}

/// 1行目より下で、桁 `from` から右に `s` が描かれた最初の (桁, 行)。
fn find(a: &App, s: &str, from: u16) -> (u16, u16) {
    let b = buf(a);
    for y in 1..b.area.height {
        let row: Vec<String> = (0..b.area.width)
            .map(|x| b[(x, y)].symbol().to_string())
            .collect();
        for x in from as usize..row.len() {
            if row[x..].concat().starts_with(s) {
                return (x as u16, y);
            }
        }
    }
    panic!("{s} が画面に無い");
}

fn sel_name(a: &App) -> String {
    let rm = a.relmap.as_ref().expect("関係マップ");
    rm.infos[rm.sel].name.clone()
}

fn open_map(w: u16, h: u16, name: &str) -> (super::test_screen::Tmp, App) {
    let tmp = workspace(name);
    let mut a = boot_tasks(&tmp);
    a.resize(w, h);
    super::test_screen::ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    (tmp, a)
}

#[test]
fn test_rel_12_click_selects_and_opens() {
    // [REL-12] 左の一覧の行で選び、選んだ表をもう一度で開く。盤の箱でも選ぶ。枠は何もしない。
    let (_t, mut a) = open_map(160, 40, "rmc_open");
    assert_eq!(sel_name(&a), "Tasks");
    let (x, y) = find(&a, "Projects", 0);
    a.click(x + 1, y);
    assert_eq!(sel_name(&a), "Projects");
    assert_eq!(a.mode, Mode::Relations, "1回目は選ぶだけ");
    // 盤の Members の箱の題(一覧より右)。
    let (x, y) = find(&a, "Members (", 30);
    a.click(x, y);
    assert_eq!(sel_name(&a), "Members");
    // 枠と空きは何も変えない。
    a.click(0, 1);
    a.click(100, 20); // 盤の空き
    assert_eq!(sel_name(&a), "Members");
    assert_eq!(a.mode, Mode::Relations);
    // 選んでいる箱をもう一度 → その表へ移る。
    let (x, y) = find(&a, "Members (", 30);
    a.click(x, y);
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.name.as_str()),
        Some("Members")
    );
}

#[test]
fn test_rel_12_click_link_and_linked_row() {
    // [REL-12] 盤の線・矢印・札でつながりを選ぶ。下のつながった行で、その元のノートを開く。
    let (_t, mut a) = open_map(160, 40, "rmc_link");
    // 盤のセルからつながりの1つ(線か矢印)の位置を探し、画面の位置に直す(盤は窓に収まるのでずらしは無い)。
    let rm = a.relmap.as_ref().unwrap();
    let want = rm
        .links
        .iter()
        .position(|l| l.from == "Tasks" && l.column == "assignee")
        .expect("tasks.assignee のつながり");
    let (mx, my) = rm
        .map
        .cells
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.iter()
                .position(|c| c.link == Some(want) && c.table.is_none())
                .map(|x| (x, y))
        })
        .expect("つながりのセル");
    // 一覧の窓は 0 から 26 桁、盤の窓の中は 27 桁目・2 行目から。
    a.click((26 + 1 + mx) as u16, (2 + my) as u16);
    assert_eq!(a.relmap.as_ref().unwrap().link, Some(want));
    // つながった行(下の窓)の1行目 → その元のノートを開く(今の表の行なので、その行を選んで表に戻る)。
    let first = a.relmap.as_ref().unwrap().links[want].pairs[0].0.clone();
    let (x, y) = find(&a, &mdgrid::relations::file_stem(&first), 0);
    a.click(x, y);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.relmap.is_none());
    let row = a.cur_row().expect("選んだ行");
    assert_eq!(std::path::Path::new(&row.0), first.as_path());
}

#[test]
fn test_rel_12_narrow_list_and_wheel() {
    // [REL-12] 狭い画面(1つの段組み)では、つながりの行のクリックでそれを選ぶ。ホイールで表を上下に選ぶ。
    let (_t, mut a) = open_map(70, 24, "rmc_narrow");
    let before = a.relmap.as_ref().unwrap().sel;
    a.wheel(true);
    let n = a.relmap.as_ref().unwrap().infos.len();
    assert_eq!(a.relmap.as_ref().unwrap().sel, (before + 1) % n);
    a.wheel(false);
    assert_eq!(a.relmap.as_ref().unwrap().sel, before);
    let rm = a.relmap.as_ref().unwrap();
    let last = rm.links.len() - 1;
    let text = mdgrid::relmap::link_text(&rm.links[last]);
    let (x, y) = find(&a, &text, 0);
    a.click(x + 1, y);
    assert_eq!(a.relmap.as_ref().unwrap().link, Some(last));
}
