//! [CE-32] 画面から新しいノートを作るとき、開いたフォルダの外を指す本文の雛形は読まずに理由を出し、
//! ノートを作らない。specs/_changes/2026-10-10-body-path.md。

use super::startup::Startup;
use super::test_screen::{ch, ctrl, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;

fn boot(tmp: &Tmp, toml: &str) -> App {
    let (cfg, warnings): (Config, _) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

#[test]
fn test_ce_32_body_outside_refused_on_screen() {
    let tmp = Tmp::new("ce32out");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    std::fs::write(tmp.0.join("secret.txt"), "秘密の中身").unwrap();
    let mut a = boot(&tmp, "[new_note]\nbody = \"../secret.txt\"\n");
    ch(&mut a, 'a');
    typing(&mut a, "新しい");
    ctrl(&mut a, 's');
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("開いたフォルダの中"), "{m:?}");
    assert!(!tmp.notes().join("新しい.md").exists());
    let leaked = std::fs::read_dir(tmp.notes())
        .unwrap()
        .flatten()
        .any(|e| std::fs::read_to_string(e.path()).is_ok_and(|t| t.contains("秘密の中身")));
    assert!(!leaked);
}
