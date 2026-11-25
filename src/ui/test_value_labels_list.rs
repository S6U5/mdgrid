//! 値の候補の窓(セルを直すときと、新しいノートの窓の欄の候補)に、`[rules.<列>] labels` の名前を値に添える試験
//! (「未着手 todo」。書く値は値のまま。navigation.value-labels-picker)。値は作ったもの。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::writeback::NewValue;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

/// 候補 todo・doing と、todo の名前「未着手」(doing は名前なし)。
const RULES: &str =
    "[rules.status]\nvalues = [\"todo\", \"doing\"]\nlabels = { todo = \"未着手\" }\n";

/// 印の先頭に RULES を書いて、alpha(doing)・beta(todo)の表を開く。
fn boot(tag: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    tmp.write("alpha.md", "---\nstatus: doing\n---\n");
    tmp.write("beta.md", "---\nstatus: todo\n---\n");
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{RULES}\n[[table]]\nname = \"T\"\npath = \"notes\"\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 24);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 今の入力の候補の文字(上から)。
fn texts(a: &App) -> Vec<String> {
    a.input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn test_value_labels_in_cell_candidates() {
    // [utest -> req~navigation.value-labels-picker~2] 名前のある値の候補は「未着手 todo」(名前と値)、名前の無い値は doing のまま。選んで書く値は todo。
    let (_t, mut a) = boot("vl_cell");
    a.row = 0;
    a.col = a.cols.iter().position(|c| c == "status").unwrap();
    press(&mut a, KeyCode::Enter);
    let t = texts(&a);
    assert!(t.iter().any(|x| x == "未着手 todo"), "{t:?}");
    assert!(t.iter().any(|x| x == "doing"), "{t:?}");
    assert!(!t.iter().any(|x| x == "todo"), "{t:?}");
    // 「未着手 todo」を選んで確定する。
    let at = t.iter().position(|x| x == "未着手 todo").unwrap();
    let sel = a.input.as_ref().unwrap().list.as_ref().unwrap().sel;
    for _ in sel..at {
        press(&mut a, KeyCode::Down);
    }
    for _ in at..sel {
        press(&mut a, KeyCode::Up);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let row = a.rows[0].clone();
    assert_eq!(
        a.changes.pending(&row, "status"),
        Some(&NewValue::Str("todo".into()))
    );
}

#[test]
fn test_value_labels_in_new_note_candidates() {
    // [utest -> req~navigation.value-labels-picker~2] 新しいノートの窓の status の欄の候補も「未着手 todo」と「doing」。
    let (_t, mut a) = boot("vl_new_note");
    ch(&mut a, 'a');
    typing(&mut a, "gamma");
    press(&mut a, KeyCode::Enter);
    let t = texts(&a);
    assert!(
        t.iter().any(|x| x == "未着手 todo"),
        "{t:?}\n{}",
        screen(&a)
    );
    assert!(t.iter().any(|x| x == "doing"), "{t:?}");
}
