//! ノートの名前の変更とリンクの書き換え(CE-34)。

use super::keymap::{Action, Mode};
use super::test_screen::{make, press, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;

fn select(a: &mut App, label: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).contains(label)))
        .unwrap();
}

fn rename(a: &mut App, to: &str) {
    a.apply(Action::RenameNote);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    a.palette.as_mut().unwrap().query = to.into();
    press(a, KeyCode::Enter);
}

fn notes(name: &str) -> (Tmp, App) {
    make(
        name,
        &[
            ("会議.md", "---\nstatus: todo\n---\n"),
            ("メモ.md", "---\nproject: \"[[会議]]\"\n---\n"),
            ("日記.md", "---\nrelated:\n  - \"[[会議|打ち合わせ]]\"\n  - \"[[ほか]]\"\nsee: \"[[会議#議題]]\"\n---\n"),
            ("ほか.md", "---\nproject: \"[[会議室]]\"\n---\n"),
        ],
    )
}

#[test]
fn test_ce_34_rename_and_stage_link_rewrites() {
    // [CE-34] 会議 → 会議(10月)。ファイルの名前が変わり、行は新しい名前で選ばれたまま。
    // メモの project、日記の related の要素と see を書き換える変更が3件ためられる。会議室は書き換えない。
    let (t, mut a) = notes("ce34a");
    select(&mut a, "会議");
    rename(&mut a, "会議(10月)");
    assert!(t.notes().join("会議(10月).md").exists());
    assert!(!t.notes().join("会議.md").exists());
    assert!(
        a.message.as_deref().unwrap_or("").contains('3'),
        "{:?}",
        a.message
    );
    let cur = a.cur_row().unwrap();
    assert!(a.src.label(&cur).contains("会議(10月)"));
    let text = |a: &App, label: &str, col: &str| {
        let r = a
            .rows
            .iter()
            .find(|r| a.src.label(r).contains(label))
            .unwrap()
            .clone();
        a.prop(&r, col)
    };
    use mdgrid::source::Value;
    assert_eq!(
        text(&a, "メモ", "project"),
        Some(Value::Str("[[会議(10月)]]".into()))
    );
    assert_eq!(
        text(&a, "日記", "see"),
        Some(Value::Str("[[会議(10月)#議題]]".into()))
    );
    assert_eq!(
        text(&a, "日記", "related"),
        Some(Value::List(vec![
            Value::Str("[[会議(10月)|打ち合わせ]]".into()),
            Value::Str("[[ほか]]".into())
        ]))
    );
    assert_eq!(
        text(&a, "ほか", "project"),
        Some(Value::Str("[[会議室]]".into()))
    );
    // ためた変更なので、ファイルはまだ書き換わっていない。
    let memo = std::fs::read_to_string(t.notes().join("メモ.md")).unwrap();
    assert!(memo.contains("[[会議]]"), "{memo}");
}

#[test]
fn test_ce_34_refuses_bad_names_and_pending_rows() {
    // [CE-34] / を含む名前・もうある名前は断り、ファイルはそのまま。ためた変更のある行は始めない。
    let (t, mut a) = notes("ce34b");
    select(&mut a, "会議");
    rename(&mut a, "a/b");
    assert!(t.notes().join("会議.md").exists());
    assert!(
        a.message.as_deref().unwrap_or("").contains('/'),
        "{:?}",
        a.message
    );
    press(&mut a, KeyCode::Esc);
    rename(&mut a, "メモ");
    assert!(t.notes().join("会議.md").exists());
    assert!(
        a.message.as_deref().unwrap_or("").contains("もうある"),
        "{:?}",
        a.message
    );
    press(&mut a, KeyCode::Esc);
    // status を消す変更をためてから。
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Backspace);
    a.apply(Action::RenameNote);
    assert_ne!(a.mode, Mode::Palette);
    assert!(
        a.message.as_deref().unwrap_or("").contains("ためた変更"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_ce_34_menu_offers_rename_note() {
    // [CE-34] 操作の一覧の行の節に「ノートの名前を変える」が出る。
    let (_t, mut a) = notes("ce34c");
    select(&mut a, "会議");
    let items = super::menu::items(&a);
    assert!(
        items.iter().any(|i| i.action == Action::RenameNote),
        "操作の一覧に名前の変更が無い"
    );
}
