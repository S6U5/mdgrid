//! [SC-17] CSV・TSV の表では、ノートだけの機能を出さずに断り、`a` で末尾に行を足す。
//! specs/_changes/2026-10-10-csv-source.md。

use super::app::App;
use super::keymap::{Action, Mode};
use super::test_screen::{ch, ctrl, edit_cell, press, screen, typing, Tmp};
use super::ColorMode;
use mdgrid::source::csv::Csv;
use ratatui::crossterm::event::KeyCode;

const LEDGER: &str = "name,qty\r\nりんご,3\r\nみかん,12\r\n";

fn open(name: &str, text: &str) -> (Tmp, App) {
    let t = Tmp::new(name);
    t.write("台帳.csv", text);
    let src = Csv::open(&t.notes().join("台帳.csv")).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    (t, a)
}

fn disk(t: &Tmp) -> String {
    std::fs::read_to_string(t.notes().join("台帳.csv")).unwrap()
}

#[test]
fn test_sc_17_note_only_actions_refused() {
    // [SC-17] エディタで開く・名前の変更・キーの名前の変更と削除・列を足す・関係マップ・親子の並べ方 → 理由を出して何もしない。
    let (t, mut a) = open("sc17no", LEDGER);
    for act in Action::NOTE_ONLY {
        a.apply(*act);
        assert_eq!(a.mode, Mode::Table, "{act:?}");
        assert!(
            a.message.as_deref().unwrap_or("").contains("CSV"),
            "{act:?}: {:?}",
            a.message
        );
    }
    ch(&mut a, 'e');
    assert!(a.message.as_deref().unwrap_or("").contains("CSV"));
    assert_eq!(disk(&t), LEDGER);
}

#[test]
fn test_sc_17_note_only_actions_hidden() {
    // [SC-17] 操作の一覧とパレットに、ノートだけの動作を出さない。
    let (_t, a) = open("sc17hide", LEDGER);
    let items = super::menu::items(&a);
    assert!(!items.is_empty());
    assert!(items.iter().all(|i| !Action::NOTE_ONLY.contains(&i.action)));
    for q in ["rename", "editor", "relation", "tree", "名前"] {
        assert!(
            super::help::candidates(&a, q)
                .iter()
                .all(|c| !matches!(c.target, super::help::Target::Run(x) if Action::NOTE_ONLY.contains(&x))),
            "{q}"
        );
    }
}

#[test]
fn test_sc_17_add_row() {
    // [SC-17] `a` → ファイルの末尾に空の行が書かれ、その行の1列目の入力が開く。打って保存 → その値だけが入る。
    let (t, mut a) = open("sc17add", LEDGER);
    ch(&mut a, 'a');
    assert_eq!(disk(&t), "name,qty\r\nりんご,3\r\nみかん,12\r\n,\r\n");
    assert_eq!(a.rows.len(), 3);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    typing(&mut a, "ぶどう");
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    assert_eq!(disk(&t), "name,qty\r\nりんご,3\r\nみかん,12\r\nぶどう,\r\n");
    assert!(screen(&a).contains("ぶどう"));
}

#[test]
fn test_sc_17_add_row_keeps_pending_edits() {
    // [SC-17][WB-4] 直しをためたまま行を足しても、ためた直しは外の変更として止まらずに書ける。
    let (t, mut a) = open("sc17keep", LEDGER);
    edit_cell(&mut a, 0, "qty", "5");
    ch(&mut a, 'a');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
    assert_eq!(disk(&t), "name,qty\r\nりんご,5\r\nみかん,12\r\n,\r\n");
}

#[test]
fn test_sc_17_single_column_empty_row_kept() {
    // [SC-17] 1列の表に空の行を足しても、空の行として読み飛ばされず行になる。
    let (t, mut a) = open("sc17one", "name\nx\n");
    ch(&mut a, 'a');
    assert_eq!(disk(&t), "name\nx\n\"\"\n");
    assert_eq!(a.rows.len(), 2);
}

#[test]
fn test_sc_17_no_note_label_column() {
    // [SC-17] 左の欄に「ノート」の見出しと行の名前(1列目と同じ値)を重ねて出さない。
    let (_t, a) = open("sc17label", LEDGER);
    let s = screen(&a);
    let head = s.lines().nth(3).unwrap();
    assert!(!head.contains("ノート") && head.contains("name"), "{s}");
    let first = s.lines().nth(4).unwrap();
    assert_eq!(first.matches("りんご").count(), 1, "{s}");
}

#[test]
fn test_sc_17_add_row_shown_with_pending_edits() {
    // [SC-17][NV-12] 直した行を留めている間に行を足しても、足した行が表に出る。
    let (_t, mut a) = open("sc17shown", "id,status\nA,x\nB,y\n");
    edit_cell(&mut a, 1, "status", "z");
    ch(&mut a, 'a');
    assert_eq!(a.rows.len(), 3, "{:?}", a.message);
    assert_eq!(a.mode, Mode::Edit);
}
