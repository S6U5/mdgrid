//! 並べ替えの窓の「×」(NV-24): 決まりの行の右の端を押すと外す。ほかの所は向きを変える。

use super::keymap::Mode;
use super::test_screen::{ch, make, press, screen};
use mdgrid::settings::Dir;
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_nv_24_click_x_removes_rule() {
    // [NV-24] 決まりを足して、行の「×」をクリック → 外れる。行の真ん中のクリック → 向きが変わる。
    let (_t, mut a) = make(
        "nv24x",
        &[
            ("a.md", "---\ndue: 2026-10-03\n---\n"),
            ("b.md", "---\ndue: 2026-10-01\n---\n"),
        ],
    );
    ch(&mut a, 'S');
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter); // 最初の候補の列(due)
    assert_eq!(a.settings.sorts, [("due".to_string(), Dir::Asc)]);
    let s = screen(&a);
    let y = s
        .lines()
        .position(|l| l.contains("↑ due") && l.contains('×'))
        .unwrap_or_else(|| panic!("× のある行が無い:\n{s}")) as u16;
    let line = s.lines().nth(y as usize).unwrap();
    let xc = super::width::width(&line[..line.find('×').unwrap()]) as u16;
    let mid = super::width::width(&line[..line.rfind("↑ due").unwrap() + "↑ ".len()]) as u16;
    a.click(mid, y);
    assert_eq!(
        a.settings.sorts,
        [("due".to_string(), Dir::Desc)],
        "行の真ん中は向き"
    );
    a.click(xc, y);
    assert!(a.settings.sorts.is_empty(), "× で外す:\n{}", screen(&a));
    assert_eq!(a.mode, Mode::Sorts, "窓は開いたまま");
}
