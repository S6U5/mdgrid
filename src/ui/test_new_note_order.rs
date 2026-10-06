//! [CE-26] 作る場所を選ぶ欄(CE-25)があるときの順: 作る場所 → 名前 → 入力させる項目
//! (決定 specs/_decisions/2026-10-03-new-note-folders-order.md)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const A: &str = "folder_aa";
const B: &str = "folder_bb";

fn boot_ask(name: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (dir, note) in [(A, "a1.md"), (B, "b1.md")] {
        let d = tmp.notes().join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(note), "---\nstatus: todo\n---\n").unwrap();
    }
    let (config, warnings) = mdgrid::config::parse("[new_note]\nask = [\"status\"]\n").unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let paths: Vec<PathBuf> = [A, B].iter().map(|d| tmp.notes().join(d)).collect();
    let t = crate::open_target(&paths, None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: crate::state_target(&paths),
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

#[test]
fn test_ce_26_place_then_name_then_asked_items() {
    let (tmp, mut a) = boot_ask("ce26_order");
    ch(&mut a, 'a');
    // 1. 作る場所: 一覧に A と B。まだ名前の欄ではない(打った文字は名前に入らない)。
    let s = screen(&a);
    assert!(s.contains(A) && s.contains(B), "作る場所の一覧:\n{s}");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    // 2. 名前
    assert_ne!(a.mode, Mode::Table, "名前の欄:\n{}", screen(&a));
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    // 3. 入力させる項目(status)を、名前のあとに聞く。
    let s = screen(&a);
    assert!(s.contains("status"), "名前のあとに status を聞く:\n{s}");
    assert_ne!(a.mode, Mode::Table, "まだ聞いている:\n{s}");
    typing(&mut a, "doing");
    press(&mut a, KeyCode::Enter);
    let made = tmp.notes().join(B).join("メモ.md");
    let text = std::fs::read_to_string(&made).unwrap_or_default();
    assert!(
        text.contains("status: doing"),
        "B に status つきで作る: {text:?}\n{}",
        screen(&a)
    );
    assert!(!tmp.notes().join(A).join("メモ.md").exists());
}
