//! 開いたときに黙らせない知らせ(specs/_changes/2026-10-06-trust-notices.md)。
//! [BV-14][BV-7] 未対応の集計・式の理由は、開いたときに下の行へ一度出す。
//! [BV-2] `.obsidian/` の無い場所の `.base` で0行なら、探した根を下の行に出す。

use super::test_screen::{press, screen, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;

const BASE: &str = "summaries:\n  customAverage: 'values.mean()'\nviews:\n  - type: table\n    name: F\n    order: [title, estimate]\n    summaries:\n      estimate: customAverage\n";

#[test]
fn test_bv_14_unsupported_summary_shown_on_open() {
    let tmp = Tmp::new("tn_sum");
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    tmp.write("a.md", "---\ntitle: x\nestimate: 3\n---\n");
    let app = super::test_grid::open(&tmp, BASE, None);
    assert!(app.selected().is_some(), "セルを選んでいても");
    let s = screen(&app);
    assert!(s.contains("customAverage"), "{s}");
}

#[test]
fn test_bv_14_unsupported_summary_shown_once() {
    let tmp = Tmp::new("tn_once");
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    tmp.write("a.md", "---\ntitle: x\nestimate: 3\n---\n");
    let mut app = super::test_grid::open(&tmp, BASE, None);
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Down);
    app.refresh();
    assert!(app.message.is_none(), "{:?}", app.message);
}

#[test]
fn test_bv_2_vault_root_shown_when_empty() {
    let tmp = Tmp::new("tn_root");
    std::fs::create_dir_all(tmp.notes().join("Projects")).unwrap();
    tmp.write("Projects/a.md", "---\nx: 1\n---\n");
    let base = "filters:\n  and:\n    - file.inFolder(\"Projects\")\nviews:\n  - type: table\n    name: v\n";
    let dir = tmp.notes().join("Bases");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("t.base");
    std::fs::write(&path, base).unwrap();
    let t = crate::open_target(&[path], None).unwrap();
    assert!(t.guessed_root.is_some());
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.guessed_root = t.guessed_root;
    let (b, name, idx) = t.base.unwrap();
    app.set_base(b, name, idx.unwrap_or(0));
    while !app.loaded() {
        app.load_step(1000);
    }
    assert!(app.rows.is_empty());
    let s = screen(&app);
    assert!(s.contains(".obsidian"), "{s}");
}
