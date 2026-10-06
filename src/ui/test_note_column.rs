//! [SR-29] 左のノートの欄: 全部の行に共通のフォルダと `.md` を除き、file.name の列があれば名前と見出しを出さない。
//! specs/_changes/2026-10-07-note-column.md。

use super::grid::common_folder;
use super::test_grid::open;
use super::test_screen::{edit_cell, edit_outside, screen, Tmp};
use super::{App, ColorMode};
use mdgrid::source::markdown::Markdown;

fn folder_app(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

#[test]
fn test_sr_29_common_folder() {
    let f = |v: &[&str]| common_folder(v.iter().map(|s| s.to_string()));
    assert_eq!(f(&["Tasks/a.md", "Tasks/b.md"]), "Tasks/");
    assert_eq!(f(&["Tasks/x/a.md", "Tasks/y/b.md"]), "Tasks/");
    assert_eq!(f(&["Tasks/a.md", "Notes/c.md"]), "");
    assert_eq!(f(&["a.md", "Tasks/b.md"]), "");
    // 名前の途中では切らない(フォルダの区切りの単位)。
    assert_eq!(f(&["Task/a.md", "Tasks/b.md"]), "");
    assert_eq!(f(&[]), "");
}

#[test]
fn test_sr_29_note_column_drops_common_folder_and_md() {
    let tmp = Tmp::new("sr29common");
    std::fs::create_dir_all(tmp.notes().join("Tasks")).unwrap();
    tmp.write("Tasks/a.md", "---\nstatus: x\n---\n");
    tmp.write("Tasks/b.md", "---\nstatus: y\n---\n");
    let a = folder_app(&tmp);
    let s = screen(&a);
    assert!(s.lines().any(|l| l.starts_with(">a ")), "{s}");
    assert!(s.lines().any(|l| l.starts_with(" b ")), "{s}");
    assert!(!s.contains("Tasks/") && !s.contains(".md"), "{s}");

    let tmp = Tmp::new("sr29mixed");
    std::fs::create_dir_all(tmp.notes().join("Tasks")).unwrap();
    std::fs::create_dir_all(tmp.notes().join("Notes")).unwrap();
    tmp.write("Tasks/a.md", "---\nstatus: x\n---\n");
    tmp.write("Notes/c.md", "---\nstatus: y\n---\n");
    let a = folder_app(&tmp);
    let s = screen(&a);
    assert!(s.contains("Tasks/a ") && s.contains("Notes/c "), "{s}");
}

#[test]
fn test_sr_29_name_column_hides_names_but_keeps_marks() {
    let tmp = Tmp::new("sr29name");
    tmp.write("alpha.md", "---\nstatus: x\n---\n");
    tmp.write("beta.md", "---\nstatus: y\n---\n");
    let base = "views:\n  - type: table\n    name: v\n    order: [file.name, status]\n";
    let mut a = open(&tmp, base, None);
    let s = screen(&a);
    assert!(!s.contains("ノート"), "見出しも出さない: {s}");
    // 名前は file.name の列にだけ出る(1行に1回)。
    let row = s.lines().find(|l| l.contains("alpha")).unwrap();
    assert_eq!(row.matches("alpha").count(), 1, "{s}");
    // 直してためた行が外で変わったときの `!` は残る(WB-16)。
    let i = a
        .rows
        .iter()
        .position(|r| a.src.label(r) == "alpha.md")
        .unwrap();
    edit_cell(&mut a, i, "status", "z");
    edit_outside(&tmp, "alpha.md", "---\nstatus: changed\n---\n");
    a.poll();
    let s = screen(&a);
    let row = s.lines().find(|l| l.contains("alpha")).unwrap();
    assert!(row.contains('!'), "{s}");
}
