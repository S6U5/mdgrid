//! [CE-33] 新しいノートをフォームかエディタかで作る。specs/_changes/2026-10-07-new-note-editor.md。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp, toml: &str) -> App {
    let (cfg, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    tmp.write("a.md", "---\nstatus: todo\n---\n");
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
    app.today = types::parse_date("2026-10-02").unwrap();
    app
}

#[test]
fn test_ce_33_editor_mode_creates_and_opens() {
    let tmp = Tmp::new("ce33editor");
    let mut a = boot(
        &tmp,
        "[new_note]\nmode = \"editor\"\nhidden = [\"created\"]\n\n[new_note.set]\ncreated = \"{date}\"\n",
    );
    ch(&mut a, 'a');
    // 名前だけの窓(見えている列 status は欄にしない)。
    let s = screen(&a);
    assert!(!s.lines().any(|l| l.starts_with("  status")), "{s}");
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let t = std::fs::read_to_string(tmp.notes().join("メモ.md")).unwrap();
    assert!(t.contains("created: 2026-10-02"), "{t}");
    // `e` と同じく、エディタで開く頼みが出る。
    assert!(a.wants_editor());
}

#[test]
fn test_ce_33_form_ctrl_e_creates_and_opens() {
    let tmp = Tmp::new("ce33ctrle");
    let mut a = boot(&tmp, "");
    ch(&mut a, 'a');
    assert!(screen(&a).contains("^E"), "下の帯に Ctrl+E: {}", screen(&a));
    typing(&mut a, "本文を書く");
    ctrl(&mut a, 'e');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert!(tmp.notes().join("本文を書く.md").is_file());
    assert!(a.wants_editor());
    // Ctrl+S ではエディタで開かない。
    let mut b = boot(&Tmp::new("ce33ctrls"), "");
    ch(&mut b, 'a');
    typing(&mut b, "x");
    ctrl(&mut b, 's');
    assert!(!b.wants_editor());
}

#[test]
fn test_ce_33_bad_mode_warns_and_uses_form() {
    let (c, warnings) = mdgrid::config::parse("[new_note]\nmode = \"vim\"\n").unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(!c.resolved().new_note.editor());
}
