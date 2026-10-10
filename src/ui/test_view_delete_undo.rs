//! ビューの削除を確かめ、消した直後の u で戻す(BV-18)。

use super::keymap::Mode;
use super::settings::{Sec, VIEW_BUTTONS};
use super::startup::Startup;
use super::test_grid::{base_path, vault, TASKS};
use super::test_screen::{ch, press, typing, Tmp};
use super::*;
use mdgrid::views::{self, NativeView, TabPrefs};
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p.clone(),
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn setup(name: &str) -> Tmp {
    let tmp = vault(name);
    let p = base_path(&tmp, TASKS);
    let dir = tmp.0.join("config");
    std::fs::create_dir_all(&dir).unwrap();
    let v = |n: &str| NativeView {
        name: n.into(),
        ..Default::default()
    };
    views::save_views(&dir, &p, &[v("未完了"), v("メモ"), v("後")]).unwrap();
    tmp
}

fn names(tmp: &Tmp) -> Vec<String> {
    views::load_views(&tmp.0.join("config"), &tmp.notes().join("tasks.base"))
        .0
        .into_iter()
        .map(|v| v.name)
        .collect()
}

fn delete_current(a: &mut App, answer: &str) {
    ch(a, 'o');
    a.view_button(VIEW_BUTTONS + 3);
    assert_eq!(a.mode, Mode::SettingsText, "確かめの入力が開く");
    typing(a, answer);
    press(a, KeyCode::Enter);
}

#[test]
fn test_bv_18_delete_confirm_and_undo() {
    // [BV-18] 「メモ」を開いて削除 → y を打たなければ消さない。y で消える。表で u → 同じ位置に戻る。
    let tmp = setup("bv18undo");
    let mut a = boot(&tmp);
    a.select_view_named("メモ");
    delete_current(&mut a, "n");
    assert_eq!(names(&tmp), ["未完了", "メモ", "後"]);
    assert!(
        a.message.as_deref().unwrap_or("").contains("消さなかった"),
        "{:?}",
        a.message
    );
    press(&mut a, KeyCode::Esc);
    delete_current(&mut a, "y");
    assert_eq!(names(&tmp), ["未完了", "後"]);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("u"),
        "{:?}",
        a.message
    );
    ch(&mut a, 'u');
    assert_eq!(names(&tmp), ["未完了", "メモ", "後"], "{:?}", a.message);
    assert!(a.view_names().contains(&"メモ".to_string()));
    // 2回目の u は何もしない(戻すものはもう無い)。
    ch(&mut a, 'u');
    assert_eq!(names(&tmp), ["未完了", "メモ", "後"]);
}

#[test]
fn test_bv_18_undo_restores_tab_prefs_and_default() {
    // [BV-18] 区画から開いていない「後」を消す → タブの順・隠す・既定も、u で消す前に戻る。
    let tmp = setup("bv18prefs");
    let dir = tmp.0.join("config");
    let target = tmp.notes().join("tasks.base");
    let p = TabPrefs {
        order: vec!["後".into(), "進行中".into()],
        hidden: vec!["メモ".into()],
        hint: true,
    };
    views::save_tab_prefs(&dir, &target, &p).unwrap();
    views::save_default_view(&dir, &target, Some("後")).unwrap();
    let mut a = boot(&tmp);
    ch(&mut a, 'o');
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Views {
            break;
        }
        press(&mut a, KeyCode::Tab);
    }
    let i = a.section_tabs().iter().position(|t| t.0 == "後").unwrap();
    a.draft.as_mut().unwrap().sel[Sec::Views as usize] = i;
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert!(!names(&tmp).contains(&"後".to_string()));
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'u');
    assert!(names(&tmp).contains(&"後".to_string()), "{:?}", a.message);
    assert_eq!(views::load_tab_prefs(&dir, &target), p);
    assert_eq!(
        views::load_default_view(&dir, &target).as_deref(),
        Some("後")
    );
}

#[test]
fn test_bv_18_cell_change_after_delete_undone_first() {
    // [BV-18] 消したあとにセルの変更をためたら、u はまずその変更を戻し、次の u でビューを戻す。
    let tmp = setup("bv18order");
    let mut a = boot(&tmp);
    a.select_view_named("メモ");
    delete_current(&mut a, "y");
    let col = a.cols.iter().position(|c| c == "status").unwrap();
    a.col = col;
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(_)))
        .unwrap();
    press(&mut a, KeyCode::Backspace);
    assert!(a.changes.count() > 0);
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    assert!(!names(&tmp).contains(&"メモ".to_string()));
    ch(&mut a, 'u');
    assert!(names(&tmp).contains(&"メモ".to_string()), "{:?}", a.message);
}
