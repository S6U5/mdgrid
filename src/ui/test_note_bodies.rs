//! [cell-edit.body-choose] `[new_note] body` がフォルダのとき、新しいノートの窓で本文の雛形を選ぶ。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

/// 雛形のフォルダ `templates/`(bug.md・idea.md)のある表。`extra` は `[new_note]` に足す行。
fn app_with(tag: &str, extra: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    std::fs::create_dir_all(tmp.notes().join("templates")).unwrap();
    tmp.write("templates/bug.md", "# {name}\n\n## 再現の手順\n");
    tmp.write("templates/idea.md", "# {name}\n\n## ねらい\n");
    tmp.write("templates/memo.txt", "雛形ではない\n");
    let toml = format!("[new_note]\nbody = \"templates\"\nask = [\"status\"]\n{extra}");
    let (cfg, w) = mdgrid::config::parse(&toml).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

fn body_of(tmp: &Tmp, name: &str) -> String {
    let text = std::fs::read_to_string(tmp.notes().join(name)).unwrap();
    text.split_once("---\n")
        .and_then(|(_, rest)| rest.split_once("---\n"))
        .map(|(_, body)| body.to_string())
        .unwrap_or(text)
}

// [utest -> req~cell-edit.body-choose-field~2]
#[test]
fn test_body_folder_adds_a_template_field_listing_its_md_files() {
    // body がフォルダなら、名前の次に「雛形」の欄が出て、そのフォルダの .md の名前が名前の順に並ぶ。
    let (_t, mut a) = app_with("bodies-field", "");
    ch(&mut a, 'a');
    typing(&mut a, "t");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("雛形"), "{s}");
    let (bug, idea) = (s.find("bug").unwrap(), s.find("idea").unwrap());
    assert!(bug < idea, "{s}");
    assert!(!s.contains("memo"), "{s}");
}

// [utest -> req~cell-edit.body-choose-pick~2]
#[test]
fn test_body_picked_in_the_field_becomes_the_body() {
    // 欄で idea を選んで作ると、idea.md の中身(変数を埋めたもの)が本文になる。
    let (t, mut a) = app_with("bodies-pick", "");
    ch(&mut a, 'a');
    typing(&mut a, "t");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(body_of(&t, "t.md"), "# t\n\n## ねらい\n");
    let text = std::fs::read_to_string(t.notes().join("t.md")).unwrap();
    assert!(
        !text.contains("idea"),
        "雛形の名前はノートに書かない: {text}"
    );
}

// [utest -> req~cell-edit.body-choose-default~1]
#[test]
fn test_body_defaults_to_the_first_template_when_not_picked() {
    // 欄に寄らずに作ると、名前の順で最初の雛形(bug)が本文になる。
    let (t, mut a) = app_with("bodies-default", "");
    ch(&mut a, 'a');
    typing(&mut a, "t");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(body_of(&t, "t.md"), "# t\n\n## 再現の手順\n");
}

// [utest -> req~cell-edit.body-choose-none~1]
#[test]
fn test_body_none_item_makes_a_note_without_a_body() {
    // 欄で「なし」を選んで作ると、本文の無いノートになる。
    let (t, mut a) = app_with("bodies-none", "");
    ch(&mut a, 'a');
    typing(&mut a, "t");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert_eq!(body_of(&t, "t.md"), "");
}
