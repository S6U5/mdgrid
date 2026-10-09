//! 狭く低い関係マップ(1つの段組み)で、選んでいるつながりが窓に入り、押せる(REL-9・REL-12)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, press, text};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

#[test]
fn test_rel_9_narrow_list_scrolls_to_selection() {
    // [REL-9][REL-12] 窓より一覧が長くても、選んだつながりの行は見え、その行のクリックで選べる。
    let tmp = workspace("rm_scroll");
    let mut a = boot_tasks(&tmp);
    a.resize(70, 12);
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    let n = a.relmap.as_ref().unwrap().links.len();
    let inner = 12 - 1 - 3 - 2;
    assert!(
        a.relmap.as_ref().unwrap().infos.len() + 1 + n > inner,
        "一覧が窓より長い見本"
    );
    // 選んだ表のつながりを最後まで回す。
    for _ in 0..n {
        press(&mut a, KeyCode::Right);
        let rm = a.relmap.as_ref().unwrap();
        let k = rm.link.unwrap();
        let want = mdgrid::relmap::link_text(&rm.links[k]);
        let s = screen(&a);
        assert!(
            s.contains(&format!(">{want}")),
            "選んだつながりが見える: {want}\n{s}"
        );
    }
    // 見えている別のつながりの行を押すと、それを選ぶ。
    let rm = a.relmap.as_ref().unwrap();
    let cur = rm.link.unwrap();
    let s = screen(&a);
    let (other, y) = (0..rm.links.len())
        .filter(|&k| k != cur)
        .find_map(|k| {
            let t = mdgrid::relmap::link_text(&rm.links[k]);
            s.lines()
                .position(|l| l.contains(&format!(" {t}")))
                .map(|y| (k, y))
        })
        .expect("ほかのつながりの行が見える");
    a.click(3, y as u16);
    assert_eq!(a.relmap.as_ref().unwrap().link, Some(other));
}
