//! 親子で並べた表でも、直した行の留め(NV-12)は本来の位置と違う行にだけ印を付ける。

use super::test_screen::{col_named, make, press, typing};
use ratatui::crossterm::event::KeyCode;

#[test]
fn test_nv_12_tree_parent_edit_not_held() {
    // [NV-12][NV-27] 子を持つ「設計」の n を直す → 並びは変わらないので ~ は付かない。子の行も同じ。
    let (_t, mut a) = make(
        "nv12tree",
        &[
            ("設計.md", "---\nn: 1\n---\n"),
            ("画面.md", "---\nparent: \"[[設計]]\"\nn: 2\n---\n"),
            ("部品.md", "---\nparent: \"[[画面]]\"\nn: 3\n---\n"),
        ],
    );
    a.settings.tree = Some("parent".into());
    a.regrid = true;
    a.refresh_if_needed();
    for name in ["設計", "画面"] {
        let i = (0..a.slots.len())
            .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains(name)))
            .unwrap();
        a.row = i;
        col_named(&mut a, "n");
        press(&mut a, KeyCode::Enter);
        press(&mut a, KeyCode::Backspace);
        typing(&mut a, "9");
        press(&mut a, KeyCode::Enter);
        a.refresh_if_needed();
        assert!(
            a.held.is_empty(),
            "{name} を直したら留めた印が付いた: {:?}",
            a.held
        );
    }
}
