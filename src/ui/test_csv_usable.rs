//! [SC-17] CSV・TSV の表で使えるべき機能: 型ごとの入力・まとめての直し・取り消し・絞り込み・並べ替え・
//! グループ分け・mdgrid のビュー。specs/_changes/2026-10-10-csv-source.md。

use super::app::App;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, ctrl, edit_cell, press, screen, typing, Tmp};
use super::ColorMode;
use mdgrid::settings::{Cond, Dir, Group, Op, Settings};
use mdgrid::source::csv::Csv;
use mdgrid::source::NewValue;
use mdgrid::views::{save_views, NativeView};
use ratatui::crossterm::event::KeyCode;

const LEDGER: &str = "id,qty,bought,ok,place\nA,3,2026-04-01,true,東京\nB,12,2026-05-12,false,大阪\nC,1,2025-11-20,true,東京\n";

fn boot(t: &Tmp) -> App {
    let p = t.notes().join("台帳.csv");
    let src = Csv::open(&p).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(t.0.join("state")),
        config_dir: Some(t.0.join("config")),
        target: p,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

fn open(name: &str) -> (Tmp, App) {
    let t = Tmp::new(name);
    t.write("台帳.csv", LEDGER);
    let a = boot(&t);
    (t, a)
}

fn disk(t: &Tmp) -> String {
    std::fs::read_to_string(t.notes().join("台帳.csv")).unwrap()
}

fn save(a: &mut App) {
    ctrl(a, 's');
    press(a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
}

fn row(a: &App, id: &str) -> usize {
    (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(k) if a.src.label(&a.rows[k]) == id))
        .unwrap()
}

#[test]
fn test_sc_17_typed_input_date_and_checkbox() {
    // [SC-17][CE-4][CE-5] 日付の列は日付の入力で、真偽の列は Enter で切り替え。書くのは直した値の文字だけ。
    let (t, mut a) = open("sc17typed");
    let b = row(&a, "B");
    edit_cell(&mut a, b, "bought", "2026-12-01");
    a.row = b;
    col_named(&mut a, "ok");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.changes.pending(&a.cur_row().unwrap(), "ok"),
        Some(&NewValue::Bool(true))
    );
    save(&mut a);
    assert_eq!(
        disk(&t),
        LEDGER.replace("B,12,2026-05-12,false", "B,12,2026-12-01,true")
    );
}

#[test]
fn test_sc_17_bulk_edit_and_undo() {
    // [SC-17][CE-10][WB-10] 印を付けた2行の qty をまとめて直し、取り消すと戻る。やり直して保存 → 2行だけ変わる。
    let (t, mut a) = open("sc17bulk");
    a.row = row(&a, "A");
    col_named(&mut a, "qty");
    press(&mut a, KeyCode::Char(' '));
    press(&mut a, KeyCode::Char(' '));
    a.row = row(&a, "A");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    a.input.as_mut().unwrap().text.clear();
    a.input.as_mut().unwrap().cursor = 0;
    typing(&mut a, "0");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 2, "{:?}", a.message);
    a.apply(Action::Undo);
    assert_eq!(a.changes.count(), 0);
    a.apply(Action::Redo);
    assert_eq!(a.changes.count(), 2);
    save(&mut a);
    assert_eq!(
        disk(&t),
        LEDGER.replace("A,3,", "A,0,").replace("B,12,", "B,0,")
    );
}

#[test]
fn test_sc_17_filter_and_sort() {
    // [SC-17][NV-3][NV-1] 絞り込み(検索の欄)と並べ替え(列の見出しの s)。
    let (_t, mut a) = open("sc17find");
    ch(&mut a, '\\');
    typing(&mut a, "東京");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 2, "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    let (_t2, mut b) = open("sc17sort");
    col_named(&mut b, "qty");
    b.apply(Action::Sort);
    let order: Vec<String> = b.rows.iter().map(|r| b.src.label(r)).collect();
    assert_eq!(order, ["C", "A", "B"]);
}

#[test]
fn test_sc_17_saved_view_with_group() {
    // [SC-17][BV-17][NV-15] CSV のファイルにも mdgrid のビューを保存でき、開くとタブになり、絞り込みとグループ分けが効く。
    let t = Tmp::new("sc17view");
    t.write("台帳.csv", LEDGER);
    let p = t.notes().join("台帳.csv");
    std::fs::create_dir_all(t.0.join("config")).unwrap();
    let settings = Settings {
        filters: vec![Cond {
            col: "ok".into(),
            op: Op::Keep(vec![Some("true".into())]),
        }],
        group: Group::By {
            col: "place".into(),
            dir: Dir::Asc,
            hide_empty: false,
        },
        ..Default::default()
    };
    save_views(
        &t.0.join("config"),
        &p,
        &[NativeView {
            name: "使える".into(),
            settings,
            ..Default::default()
        }],
    )
    .unwrap();
    let mut a = boot(&t);
    for _ in 0..4 {
        if screen(&a).contains("[使える]") {
            break;
        }
        ch(&mut a, ']');
    }
    let s = screen(&a);
    assert!(s.contains("[使える]"), "{s}");
    assert_eq!(a.rows.len(), 2, "{s}");
    assert_eq!(a.groups.len(), 1, "{s}");
}
