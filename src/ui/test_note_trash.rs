//! ノートの削除(ごみ箱へ動かす)・作成と削除の取り消しとやり直し・ごみ箱から戻す。
//! 期待値は specs/requirements/cell-edit.md の cell-edit.delete-note-*・cell-edit.new-note-undo・
//! cell-edit.new-note-redo・cell-edit.trash-restore-* の受け入れの例から。

use super::keymap::{Action, Mode};
use super::test_screen::{app_of, ch, ctrl, make, press, screen, typing, Tmp};
use super::*;
use mdgrid::source::csv::Csv;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

const MEMO: &str = "---\nstatus: todo\n---\nメモの本文\n";

fn notes(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("a.md", "---\nstatus: doing\n---\n"),
            ("メモ.md", MEMO),
            ("予定.md", "---\nstatus: done\n---\n"),
        ],
    )
}

fn select(a: &mut App, label: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains(label)))
        .unwrap();
}

fn labels(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

fn selected(a: &App) -> String {
    a.src.label(&a.cur_row().expect("行が選ばれている"))
}

/// 「ノートを消す」を始めて、確かめに `answer` と打って Enter。
fn delete(a: &mut App, label: &str, answer: &str) {
    select(a, label);
    a.apply(Action::DeleteNote);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    a.palette.as_mut().unwrap().query = answer.into();
    press(a, KeyCode::Enter);
}

/// `a` で新しいノートを作る。
fn create(a: &mut App, name: &str) {
    ch(a, 'a');
    typing(a, name);
    ctrl(a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}\n{}", a.message, screen(a));
}

/// ごみ箱の中のノートのバイト(元の場所の順)。
fn trashed(a: &App) -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = a
        .src
        .trashed()
        .iter()
        .map(|t| (t.rel.clone(), std::fs::read(&t.at).unwrap()))
        .collect();
    v.sort();
    v
}

#[test]
fn test_delete_note_moves_it_to_the_trash_and_selects_the_row_below() {
    // [utest -> req~cell-edit.delete-note-menu~1] 「メモ」を消して y → 元の場所に無く、ごみ箱に同じバイトであり、行が無くなって下の行が選ばれる。
    let (t, mut a) = notes("dn-menu");
    delete(&mut a, "メモ", "y");
    assert!(!t.notes().join("メモ.md").exists());
    assert_eq!(
        trashed(&a),
        vec![("メモ.md".to_string(), MEMO.as_bytes().to_vec())]
    );
    assert_eq!(labels(&a), vec!["a.md", "予定.md"]);
    assert_eq!(selected(&a), "予定.md");
}

#[test]
fn test_delete_note_is_offered_in_the_action_menu() {
    // [utest -> req~cell-edit.delete-note-menu~1] 操作の一覧の行の節に「ノートを消す」が出る。
    let (_t, mut a) = notes("dn-offer");
    select(&mut a, "メモ");
    let items = super::menu::items(&a);
    assert_eq!(
        items
            .iter()
            .filter(|i| i.action == Action::DeleteNote)
            .count(),
        1
    );
}

#[test]
fn test_delete_note_asks_with_the_name_and_esc_keeps_the_note() {
    // [utest -> req~cell-edit.delete-note-confirm~1] 「メモ」を消すかを名前つきで聞かれ、Esc では `メモ.md` が同じバイトで残る。
    let (t, mut a) = notes("dn-esc");
    select(&mut a, "メモ");
    a.apply(Action::DeleteNote);
    assert!(screen(&a).contains("メモ.md"), "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        MEMO.as_bytes()
    );
    assert_eq!(trashed(&a), Vec::new());
}

#[test]
fn test_delete_note_needs_y_and_other_answers_keep_the_note() {
    // [utest -> req~cell-edit.delete-note-confirm~1] `y` 以外の打ち込み(空・n・yes)では消さず、`メモ.md` が同じバイトで残る。
    let (t, mut a) = notes("dn-n");
    for answer in ["", "n", "yes"] {
        delete(&mut a, "メモ", answer);
        assert_eq!(
            std::fs::read(t.notes().join("メモ.md")).unwrap(),
            MEMO.as_bytes(),
            "{answer:?}"
        );
        assert_eq!(labels(&a), vec!["a.md", "メモ.md", "予定.md"], "{answer:?}");
    }
}

#[test]
fn test_delete_note_that_cannot_reach_the_trash_stays_in_the_table() {
    // [utest -> req~cell-edit.delete-note-trash-only~1] ごみ箱へ動かせないとき、`メモ.md` は同じバイトで残り、行もそのままで、理由が出る。
    // 前提: `.trash` という名前のファイルがあって、ごみ箱のフォルダを作れない。
    let (t, mut a) = notes("dn-blocked");
    t.write(".trash", "ファイル");
    delete(&mut a, "メモ", "y");
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        MEMO.as_bytes()
    );
    assert_eq!(labels(&a), vec!["a.md", "メモ.md", "予定.md"]);
    assert!(
        a.message.as_deref().unwrap_or("").contains("ごみ箱"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_delete_note_twice_keeps_the_older_one_in_the_trash() {
    // [utest -> req~cell-edit.delete-note-trash-keep-old~1] 「古い」の `メモ.md` を消し、「新しい」の `メモ.md` を作って消す → ごみ箱に両方が同じバイトである。
    let (t, mut a) = make(
        "dn-twice",
        &[("a.md", "---\nx: 1\n---\n"), ("メモ.md", "古い")],
    );
    delete(&mut a, "メモ", "y");
    t.write("メモ.md", "新しい");
    let mut a = app_of(&t, ColorMode::None);
    delete(&mut a, "メモ", "y");
    assert_eq!(
        trashed(&a),
        vec![
            ("メモ.md".to_string(), "古い".as_bytes().to_vec()),
            ("メモ.md".to_string(), "新しい".as_bytes().to_vec()),
        ]
    );
}

#[test]
fn test_delete_note_with_pending_changes_is_refused() {
    // [utest -> req~cell-edit.delete-note-refuse-pending~1] status をためた変更のある行では断って理由を出し、`メモ.md` は残り、ためた変更は1件のまま。
    let (t, mut a) = notes("dn-pending");
    select(&mut a, "メモ");
    let row = a.cur_row().unwrap();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("done".into()))
        .unwrap();
    a.apply(Action::DeleteNote);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("ためた変更"),
        "{:?}",
        a.message
    );
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        MEMO.as_bytes()
    );
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_delete_note_is_hidden_when_readonly() {
    // [utest -> req~cell-edit.delete-note-readonly~1] 読むだけの起動では、操作の一覧とパレットに「ノートを消す」が出ず、動作でも消えない。
    let (t, mut a) = notes("dn-ro");
    assert_eq!(
        super::help::candidates(&a, "消す")
            .iter()
            .filter(|c| matches!(c.target, super::help::Target::Run(Action::DeleteNote)))
            .count(),
        1,
        "読むだけでなければ出る"
    );
    a.readonly = true;
    select(&mut a, "メモ");
    assert_eq!(
        super::menu::items(&a)
            .iter()
            .filter(|i| i.action == Action::DeleteNote)
            .count(),
        0
    );
    assert_eq!(
        super::help::candidates(&a, "消す")
            .iter()
            .filter(|c| matches!(c.target, super::help::Target::Run(Action::DeleteNote)))
            .count(),
        0
    );
    a.apply(Action::DeleteNote);
    assert_eq!(a.mode, Mode::Table);
    assert!(t.notes().join("メモ.md").exists());
}

#[test]
fn test_delete_note_is_refused_on_a_csv_table() {
    // [utest -> req~cell-edit.delete-note-csv~1] `台帳.csv` で「ノートを消す」→ CSV では使えない理由が出て、`台帳.csv` は1バイトも変わらない。
    let t = Tmp::new("dn-csv");
    t.write("台帳.csv", "name,qty\r\nりんご,3\r\n");
    let src = Csv::open(&t.notes().join("台帳.csv")).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    a.apply(Action::DeleteNote);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("CSV"),
        "{:?}",
        a.message
    );
    assert_eq!(
        std::fs::read(t.notes().join("台帳.csv")).unwrap(),
        "name,qty\r\nりんご,3\r\n".as_bytes()
    );
}

#[test]
fn test_undo_right_after_creating_a_note_moves_it_to_the_trash() {
    // [utest -> req~cell-edit.new-note-undo~1] 「買い物」を作って `u` → `買い物.md` が元の場所に無く、ごみ箱に同じバイトであり、行が無くなる。
    let (t, mut a) = notes("nn-undo");
    create(&mut a, "買い物");
    let made = std::fs::read(t.notes().join("買い物.md")).unwrap();
    ch(&mut a, 'u');
    assert!(!t.notes().join("買い物.md").exists());
    assert_eq!(trashed(&a), vec![("買い物.md".to_string(), made)]);
    assert_eq!(labels(&a), vec!["a.md", "メモ.md", "予定.md"]);
}

#[test]
fn test_undo_after_editing_the_new_note_reverts_only_the_edit() {
    // [utest -> req~cell-edit.new-note-undo~1] 「買い物」を作って status を直してから `u` を1回 → status の直しだけが戻り、`買い物.md` は残る。
    let (t, mut a) = notes("nn-undo-edit");
    create(&mut a, "買い物");
    let row = a.cur_row().unwrap();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("done".into()))
        .unwrap();
    assert_eq!(a.changes.count(), 1);
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    assert!(t.notes().join("買い物.md").exists());
    assert_eq!(trashed(&a), Vec::new());
}

#[test]
fn test_undo_of_creation_leaves_an_older_pending_change_alone() {
    // [utest -> req~cell-edit.new-note-undo~1] 先に「メモ」の status を直してから「買い物」を作り、`u` を1回 → 「買い物」だけがごみ箱へ行き、ためた変更は1件のまま。
    let (t, mut a) = notes("nn-undo-order");
    select(&mut a, "メモ");
    let row = a.cur_row().unwrap();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("done".into()))
        .unwrap();
    create(&mut a, "買い物");
    ch(&mut a, 'u');
    assert!(!t.notes().join("買い物.md").exists());
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_redo_brings_the_undone_new_note_back() {
    // [utest -> req~cell-edit.new-note-redo~1] 「買い物」を作って `u` のあと Ctrl+R → 作ったときと同じバイトで元の場所に戻り、行が選ばれ、ごみ箱に無い。
    let (t, mut a) = notes("nn-redo");
    create(&mut a, "買い物");
    let made = std::fs::read(t.notes().join("買い物.md")).unwrap();
    ch(&mut a, 'u');
    ctrl(&mut a, 'r');
    assert_eq!(std::fs::read(t.notes().join("買い物.md")).unwrap(), made);
    assert_eq!(selected(&a), "買い物.md");
    assert_eq!(trashed(&a), Vec::new());
}

#[test]
fn test_redo_of_creation_is_dropped_after_a_new_edit() {
    // [utest -> req~cell-edit.new-note-redo~1] 作成を取り消したあとに別のセルを直すと、Ctrl+R では「買い物」は戻らない(やり直しは新しい手で消える)。
    let (t, mut a) = notes("nn-redo-stale");
    create(&mut a, "買い物");
    ch(&mut a, 'u');
    select(&mut a, "メモ");
    let row = a.cur_row().unwrap();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("done".into()))
        .unwrap();
    ctrl(&mut a, 'r');
    assert!(!t.notes().join("買い物.md").exists());
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_undo_and_redo_of_a_deletion() {
    // [utest -> req~cell-edit.delete-note-undo~1] 「メモ」を消して `u` → 同じバイトで戻って行が選ばれる。続けて Ctrl+R → 元の場所に無く、ごみ箱にある。
    let (t, mut a) = notes("dn-undo");
    delete(&mut a, "メモ", "y");
    ch(&mut a, 'u');
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        MEMO.as_bytes()
    );
    assert_eq!(selected(&a), "メモ.md");
    assert_eq!(trashed(&a), Vec::new());
    ctrl(&mut a, 'r');
    assert!(!t.notes().join("メモ.md").exists());
    assert_eq!(
        trashed(&a),
        vec![("メモ.md".to_string(), MEMO.as_bytes().to_vec())]
    );
}

#[test]
fn test_restore_from_the_trash_after_a_restart() {
    // [utest -> req~cell-edit.trash-restore-list~1] `プロジェクト/メモ.md` を消して起動し直し、「ごみ箱から戻す」で選ぶ → 同じバイトで戻り、行が選ばれる。
    let t = Tmp::new("tr-list");
    std::fs::create_dir_all(t.notes().join("プロジェクト")).unwrap();
    t.write("a.md", "---\nstatus: doing\n---\n");
    t.write("プロジェクト/メモ.md", MEMO);
    let mut a = app_of(&t, ColorMode::None);
    delete(&mut a, "メモ", "y");
    drop(a);
    let mut a = app_of(&t, ColorMode::None);
    a.apply(Action::RestoreNote);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    let shown = screen(&a);
    assert!(shown.contains("メモ  プロジェクト/メモ.md  20"), "{shown}");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        std::fs::read(t.notes().join("プロジェクト/メモ.md")).unwrap(),
        MEMO.as_bytes()
    );
    assert_eq!(selected(&a), "プロジェクト/メモ.md");
    assert_eq!(trashed(&a), Vec::new());
}

#[test]
fn test_restore_with_an_empty_trash_says_so() {
    // [utest -> req~cell-edit.trash-restore-list~1] ごみ箱が空のとき、一覧を出さず、空だと知らせる。
    let (_t, mut a) = notes("tr-empty");
    a.apply(Action::RestoreNote);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.message.as_deref(), Some("ごみ箱は空"));
}

#[test]
fn test_restore_does_not_replace_a_note_made_later() {
    // [utest -> req~cell-edit.trash-restore-keep-existing~1] `メモ.md` を消したあと別の中身の `メモ.md` を作って「ごみ箱から戻す」→ 断って理由が出て、両方が同じバイトで残る。
    let (t, mut a) = notes("tr-exists");
    delete(&mut a, "メモ", "y");
    t.write("メモ.md", "あとから作ったほう");
    a.apply(Action::RestoreNote);
    press(&mut a, KeyCode::Enter);
    assert!(
        a.message.as_deref().unwrap_or("").contains("もうある"),
        "{:?}",
        a.message
    );
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        "あとから作ったほう".as_bytes()
    );
    assert_eq!(
        trashed(&a),
        vec![("メモ.md".to_string(), MEMO.as_bytes().to_vec())]
    );
}

#[test]
fn test_undo_of_a_deletion_does_not_replace_a_note_made_later() {
    // [utest -> req~cell-edit.trash-restore-keep-existing~1] `メモ.md` を消したあと別の中身の `メモ.md` ができていたら、`u` は断り、両方が同じバイトで残る。
    let (t, mut a) = notes("tr-undo-exists");
    delete(&mut a, "メモ", "y");
    t.write("メモ.md", "あとから作ったほう");
    ch(&mut a, 'u');
    assert_eq!(
        std::fs::read(t.notes().join("メモ.md")).unwrap(),
        "あとから作ったほう".as_bytes()
    );
    assert_eq!(
        trashed(&a),
        vec![("メモ.md".to_string(), MEMO.as_bytes().to_vec())]
    );
}

#[test]
fn test_restore_is_hidden_when_readonly() {
    // [utest -> req~cell-edit.trash-restore-readonly~1] 読むだけの起動では、パレットに「ごみ箱から戻す」が出ず、動作でも戻らない。
    let (t, mut a) = notes("tr-ro");
    delete(&mut a, "メモ", "y");
    assert_eq!(
        super::help::candidates(&a, "ごみ箱")
            .iter()
            .filter(|c| matches!(c.target, super::help::Target::Run(Action::RestoreNote)))
            .count(),
        1,
        "読むだけでなければ出る"
    );
    a.readonly = true;
    assert_eq!(
        super::help::candidates(&a, "ごみ箱")
            .iter()
            .filter(|c| matches!(c.target, super::help::Target::Run(Action::RestoreNote)))
            .count(),
        0
    );
    a.apply(Action::RestoreNote);
    assert_eq!(a.mode, Mode::Table);
    assert!(!t.notes().join("メモ.md").exists());
}

#[test]
fn test_delete_note_shows_how_many_notes_link_to_it_and_keeps_the_links() {
    // [utest -> req~cell-edit.delete-note-links~1] `[[メモ]]` を持つノートが2つあるとき、確かめに 2 個と出て、y のあとも2つのノートは同じバイトで、ためた変更は0件。
    let a1 = "---\nproject: \"[[メモ]]\"\n---\n";
    let a2 = "---\nrelated:\n  - \"[[メモ|覚え書き]]\"\n---\n";
    let (t, mut a) = make(
        "dn-links",
        &[
            ("a.md", a1),
            ("b.md", a2),
            ("メモ.md", MEMO),
            ("メモ帳.md", "---\nx: \"[[メモ帳]]\"\n---\n"),
        ],
    );
    select(&mut a, "メモ.md");
    a.apply(Action::DeleteNote);
    let shown = screen(&a);
    assert!(
        shown.contains("メモ.md(2 個のノートがリンクで指している)"),
        "{shown}"
    );
    a.palette.as_mut().unwrap().query = "y".into();
    press(&mut a, KeyCode::Enter);
    assert!(!t.notes().join("メモ.md").exists());
    assert_eq!(
        std::fs::read(t.notes().join("a.md")).unwrap(),
        a1.as_bytes()
    );
    assert_eq!(
        std::fs::read(t.notes().join("b.md")).unwrap(),
        a2.as_bytes()
    );
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_delete_note_shows_no_count_when_nothing_links_to_it() {
    // [utest -> req~cell-edit.delete-note-links~1] 指すノートが無いとき、確かめに数を出さない。
    let (_t, mut a) = notes("dn-nolinks");
    select(&mut a, "メモ");
    a.apply(Action::DeleteNote);
    let shown = screen(&a);
    assert!(shown.contains("消すノート: メモ.md"), "{shown}");
    assert!(!shown.contains("リンクで指している"), "{shown}");
}
