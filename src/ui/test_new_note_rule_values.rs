//! 新しいノートの窓で、決まりの選べる値(`[rules.<列>] values`)が、その列を持つノートが1件も無くても
//! 候補のリストに並ぶことの画面の試験(見本なしのスターターから作った台帳の最初のノート)。

use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const TOP: &str = "[types]\nstart = \"date\"\ndue = \"date\"\n\n\
[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n\n\
[rules.priority]\nvalues = [\"p1\", \"p2\", \"p3\"]\n\n\
[new_note]\nask = [\"priority\"]\n";

/// 印(根 = 一時フォルダ)の先頭に `top`、[[table]] に `line` を書いて、`notes` を表として開く。
fn boot(tag: &str, notes: &[(&str, &str)], top: &str, line: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{top}\n[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

#[test]
fn test_new_note_form_lists_rule_values_before_any_note_has_the_column() {
    // [utest -> req~cell-edit.new-note-form-rule-values~1] ノートが1件も無い表で、決まりに values のある priority を聞く → 候補のリストに p1・p2・p3 が決まりの順で並ぶ。
    let (_t, mut a) = boot("nn-rule-values", &[], TOP, "");
    ch(&mut a, 'a');
    typing(&mut a, "first");
    press(&mut a, KeyCode::Enter);
    let list = a
        .input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .expect("priority の欄に候補のリスト");
    let texts: Vec<&str> = list.items.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(&texts[..3], ["p1", "p2", "p3"], "{}", screen(&a));
    assert_eq!(texts.len(), 4, "決まりの3つと「なし」: {texts:?}");
}
