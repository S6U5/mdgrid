//! [WB-5][BV-1] BOM つきのノートは、読むだけ(理由つき)のまま、読める値を見せる。
//! specs/_changes/2026-10-06-bom-values.md。

use super::keymap::Mode;
use super::test_screen::{col_named, ctrl, make, press};
use mdgrid::source::Value;
use ratatui::crossterm::event::KeyCode;

const BOM: &str = "\u{FEFF}---\nstatus: draft\nowner: ken\n---\nbody\n";

#[test]
fn test_wb_5_bom_values_shown_read_only() {
    let (t, mut a) = make(
        "wb5bomv",
        &[("bom.md", BOM), ("ok.md", "---\nstatus: todo\n---\n")],
    );
    let i = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "bom.md")
        .unwrap();
    let id = a.rows[i].clone();
    let cell = a.src.get(&id, "status");
    assert_eq!(cell.value, Some(Value::Str("draft".into())));
    assert!(
        cell.lock.as_deref().is_some_and(|l| l.contains("BOM")),
        "{:?}",
        cell.lock
    );
    // BOM のノートにしか無いキーも列になる。
    assert!(a.src.columns().iter().any(|c| c == "owner"));
    // 編集は開かず、保存してもファイルは変わらない。
    a.row = i;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    ctrl(&mut a, 's');
    assert_eq!(
        std::fs::read_to_string(t.notes().join("bom.md")).unwrap(),
        BOM
    );
}
