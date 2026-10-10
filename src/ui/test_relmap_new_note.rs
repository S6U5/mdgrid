//! 関係マップでも「+ 新規」(CE-25): いつも同じ位置に出し、選んでいる表で名前の欄を出す。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 1行目の「+ 新規」の「+」の桁。無ければ None。
fn button_x(a: &App) -> Option<u16> {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    let b = t.backend().buffer().clone();
    let first = super::new_note::button().trim().chars().next()?;
    let next = super::new_note::button().trim().chars().nth(2)?;
    (0..b.area.width.saturating_sub(2)).find(|&x| {
        b[(x, 0)].symbol() == first.to_string() && b[(x + 2, 0)].symbol() == next.to_string()
    })
}

#[test]
fn test_ce_25_relmap_new_note() {
    // [CE-25] 関係マップでも「+ 新規」が表の画面と同じ桁に出る。今の表(Tasks)を選んで押す → 表の画面で名前の欄。
    let tmp = workspace("ce25_relmap");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let at = button_x(&a).expect("表の画面のボタン");
    super::test_screen::ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    assert_eq!(button_x(&a), Some(at), "関係マップでも同じ位置");
    // 押す(ボタンのクリック)。
    a.click(at + 1, 0);
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert!(a.note.flow.is_some(), "名前の欄が出る");
}

#[test]
fn test_ce_25_relmap_new_note_other_table() {
    // [CE-25] ほかの表(Projects)を選んで a → その表へ移り、開いたら名前の欄(main が start_new_note)。
    let tmp = workspace("ce25_relmap_other");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    super::test_screen::ch(&mut a, 'R');
    super::test_screen::ch(&mut a, 'j');
    let rm = a.relmap.as_ref().unwrap();
    assert_eq!(rm.infos[rm.sel].name, "Projects");
    super::test_screen::ch(&mut a, 'a');
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.name.as_str()),
        Some("Projects")
    );
    assert!(a.switch_new_note, "開き直したら名前の欄");
}
