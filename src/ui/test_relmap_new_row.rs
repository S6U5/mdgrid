//! 関係マップの「+ 新規」は、表の画面のビューの帯の「+ 新規」と同じ行・同じ桁(CE-25)。関係マップには帯が
//! 無いので、ヘッダーの下に帯の上の余白と同じ行の数を空け、マップの窓の上の縁の右の端に重ねる。
//! 記録: specs/_changes/2026-10-12-header-menu.md(タスク #5)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, text};
use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn lines(a: &App) -> Vec<String> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
        .lines()
        .map(str::to_string)
        .collect()
}

/// 「+ 新規」の「+」の (行, 桁)。
fn new_at(a: &App) -> Option<(usize, usize)> {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    let b = term.backend().buffer();
    (0..h).find_map(|y| {
        (0..w.saturating_sub(2))
            .find(|&x| b[(x, y)].symbol() == "+" && b[(x + 2, y)].symbol() == "新")
            .map(|x| (y as usize, x as usize))
    })
}

#[test]
fn test_ce_25_relmap_new_same_row_as_view_bar() {
    // [CE-25] 既定(余白 1)の 120×30: 表の画面の「+ 新規」は帯の真ん中の行(4行目)。関係マップでも同じ行・同じ
    // 位置で、ヘッダーとの間の行(3行目)は空いている。
    let tmp = workspace("ce25_row");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    let table = new_at(&a).expect("表の画面の「+ 新規」");
    assert_eq!(table.0, super::bands::mid_row(&a));
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations);
    let l = lines(&a);
    let s = l.join("\n");
    assert_eq!(
        new_at(&a),
        Some(table),
        "関係マップでも同じ行・同じ位置:\n{s}"
    );
    let head = super::bands::head_y(&a) as usize;
    assert!(
        l[head + 1].trim().is_empty(),
        "ヘッダーの下は帯の上の余白と同じく空き行:\n{s}"
    );
}

#[test]
fn test_ce_25_relmap_new_click_on_same_row() {
    // [CE-25] 関係マップで、表の画面と同じ行の「+ 新規」を押す → 表の画面に戻って名前の欄が出る。
    let tmp = workspace("ce25_rowclick");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    ch(&mut a, 'R');
    let (y, x) = new_at(&a).expect("関係マップの「+ 新規」");
    a.click(x as u16 + 1, y as u16);
    assert!(a.relmap.is_none(), "表の画面に戻る");
    assert!(a.note.flow.is_some(), "名前の欄が出る");
}

#[test]
fn test_ce_25_relmap_new_row_follows_bar_padding() {
    // [CE-25] 帯の余白が 0 なら、表の画面でも関係マップでもヘッダーのすぐ下の行(同じ行のまま)。
    let tmp = workspace("ce25_row0");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    a.display.bar_padding = mdgrid::display::Pad(0);
    let table = new_at(&a).expect("表の画面の「+ 新規」");
    assert_eq!(table.0, super::bands::head_y(&a) as usize + 1);
    ch(&mut a, 'R');
    assert_eq!(new_at(&a), Some(table), "{}", lines(&a).join("\n"));
}
