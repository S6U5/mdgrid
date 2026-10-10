//! [SC-17] CSV・TSV の表で、ほかの道から入るノートだけの機能も出さない(照合で見つけた点)。
//! specs/_changes/2026-10-10-csv-source.md。

use super::app::App;
use super::keymap::{Action, Mode};
use super::settings::Sec;
use super::test_screen::{screen, Tmp};
use super::ColorMode;
use mdgrid::source::csv::Csv;

fn open(name: &str) -> (Tmp, App) {
    let t = Tmp::new(name);
    t.write("台帳.csv", "name,qty\nりんご,3\n");
    let src = Csv::open(&t.notes().join("台帳.csv")).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    while !a.loaded() {
        a.load_step(100);
    }
    (t, a)
}

#[test]
fn test_sc_17_settings_without_tree_section() {
    // [SC-17] ビューの設定に「親子」の区画を出さず、Tab で巡っても入らない。
    let (_t, mut a) = open("sc17set");
    a.apply(Action::ViewSettings);
    assert_eq!(a.mode, Mode::Settings);
    let s = screen(&a);
    assert!(!s.contains("親子"), "{s}");
    let mut seen = Vec::new();
    for _ in 0..12 {
        a.apply(Action::NextSection);
        seen.push(a.draft.as_ref().unwrap().sec);
    }
    assert!(!seen.contains(&Sec::Tree), "{seen:?}");
    assert!(seen.contains(&Sec::Group) && seen.contains(&Sec::Display));
}

#[test]
fn test_sc_17_relation_map_and_new_note_paths() {
    // [SC-17] 関係マップは、どの道(タブのクリックなど)から開こうとしても断る。新しいノートの道は末尾に行を足す。
    let (t, mut a) = open("sc17rel");
    a.open_relmap();
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.as_deref().unwrap_or("").contains("CSV"));
    a.start_new_note();
    assert_eq!(a.rows.len(), 2);
    assert_eq!(
        std::fs::read_to_string(t.notes().join("台帳.csv")).unwrap(),
        "name,qty\nりんご,3\n,\n"
    );
}

#[test]
fn test_sc_17_csv_tables_left_out_of_links_and_map() {
    // [SC-17] 登録した表に CSV があっても、リンクと関係マップの範囲(ノートの表)には入れない。表の一覧には残る。
    let (t, mut a) = open("sc17links");
    let place = |name: &str, path: std::path::PathBuf| mdgrid::places::Place {
        name: name.into(),
        group: String::new(),
        path,
        view: None,
    };
    a.registered = vec![
        place("台帳", t.notes().join("台帳.csv")),
        place("notes", t.notes()),
    ];
    assert_eq!(a.scope_places().len(), 2);
    let links = a.link_places();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].name, "notes");
}
