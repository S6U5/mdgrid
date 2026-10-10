//! 関係マップの「+ 新規」でほかの表へ移ったとき(CE-25): 読み込みが終わってから名前の欄を出し、
//! 確認から戻ったら移ったあとの名前の欄の頼みも消す。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::*;

#[test]
fn test_ce_25_relmap_new_note_after_load() {
    // [CE-25] 開き直した App は、読み込みが終わった時に一度だけ名前の欄を出す(途中では出さない)。
    let tmp = workspace("ce25_after_load");
    let src = mdgrid::source::markdown::Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::Rgb);
    a.resize(120, 30);
    a.note.places = vec![tmp.notes()];
    a.new_note_after_load = true;
    assert!(a.note.flow.is_none(), "読み込みの前は出さない");
    while !a.loaded() {
        a.load_step(100);
    }
    assert!(a.note.flow.is_some(), "読み込みの後に名前の欄");
    assert!(!a.new_note_after_load, "一度だけ");
}

#[test]
fn test_ce_25_relmap_new_note_cancel_clears() {
    // [CE-25] 終わりの確認に入ったあと取りやめたら、移ったあとの名前の欄の頼みも消す。
    let tmp = workspace("ce25_cancel");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    super::test_screen::ch(&mut a, 'R');
    super::test_screen::ch(&mut a, 'j');
    super::test_screen::ch(&mut a, 'a');
    assert!(a.switch_new_note);
    a.apply(super::keymap::Action::Quit);
    assert!(!a.switch_new_note);
    assert_ne!(a.mode, Mode::Relations);
}
