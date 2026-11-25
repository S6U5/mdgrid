//! 表の一覧の引き出し(WS-8)の続き: 取りやめたあとの範囲の旗と、割り当て直したキーで閉じること。
//! 見本は test_relations.rs の notes/(tasks・projects・members)。Work に Tasks・Projects、Hobby に Members。

use super::keymap::{self, Mode};
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, press, Tmp};
use super::*;
use mdgrid::source::NewValue;
use mdgrid::workspace::{self as ws, WsTable};
use ratatui::crossterm::event::KeyCode;

fn add(tmp: &Tmp, w: &str, name: &str, d: &str) {
    let t = WsTable {
        name: name.into(),
        path: tmp.notes().join(d).canonicalize().unwrap(),
        view: None,
        profile: Default::default(),
    };
    ws::add(&tmp.0.join("config"), w, t).unwrap();
}

fn boot(name: &str) -> (Tmp, App) {
    let tmp = workspace(name);
    add(&tmp, "Work", "Tasks", "tasks");
    add(&tmp, "Work", "Projects", "projects");
    add(&tmp, "Hobby", "Members", "members");
    let mut a = boot_tasks(&tmp);
    a.resize(120, 30);
    (tmp, a)
}

#[test]
fn test_ws_8_back_forgets_the_scope() {
    // [utest -> req~workspace.switch-unsaved~2] 変更をためて切り替えの一覧でほかのワークスペースを選び、確かめで戻る → 移る先の範囲の旗も残らない
    // (あとで引き出しなど別の道で移るときに、前に選んだワークスペースの範囲で開き直さない)。workspace.drawer-scope-only で
    // 引き出しからほかのワークスペースへ移れなくなったので、切り替えの一覧で確かめる。
    let (tmp, mut a) = boot("ws8_back_scope");
    mdgrid::favorites::set(&tmp.0.join("config"), "Hobby", true).unwrap();
    let row = a.src.rows()[0].clone();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("x".into()))
        .unwrap();
    ch(&mut a, 'W');
    ch(&mut a, '1');
    assert_eq!(a.mode, Mode::Quit);
    assert_eq!(a.switch_workspace.as_deref(), Some("Hobby"));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none());
    assert!(a.switch_workspace.is_none() && !a.switch_leave_workspace);
}

#[test]
fn test_ws_8_rebound_key_closes() {
    // [utest -> req~workspace.drawer-open~3, req~workspace.drawer-close~2, req~screen.keys-vim-and-arrows-remap~1] 開くキーを `t` に割り当て直す → `t` で開き、もう一度の `t` で閉じる。
    let (_tmp, mut a) = boot("ws8_rebind");
    let w = keymap::rebind(
        &mut a.keys,
        &[
            ("table".into(), "w".into(), "none".into()),
            ("table".into(), "t".into(), "workspace_drawer".into()),
        ],
    );
    assert!(w.is_empty(), "{w:?}");
    ch(&mut a, 't');
    assert_eq!(a.mode, Mode::Drawer);
    ch(&mut a, 't');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none() && !a.quit);
}
