//! 並べ替えの窓の残り(NV-24・SR-12): 読むだけでも並べ替えられて何も書かない、窓で変えた並べ替えは
//! 開き直しても残る、検索の欄を隠しても開ける、ボタンを省いた幅ではそこは検索の欄のまま。

use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_grid::labels;
use super::test_screen::{ch, press, Tmp};
use super::*;
use mdgrid::settings::Dir;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

fn notes(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ndue: 2026-10-03\n---\n");
    tmp.write("b.md", "---\ndue: 2026-10-01\n---\n");
    tmp.write("c.md", "---\ndue: 2026-10-02\n---\n");
    tmp
}

fn boot(tmp: &Tmp, toml: &str, readonly: bool) -> App {
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn add_due(a: &mut App) {
    ch(a, 'S');
    assert_eq!(a.mode, Mode::Sorts);
    press(a, KeyCode::Enter);
    press(a, KeyCode::Enter); // 最初の候補の列(due)
    press(a, KeyCode::Esc);
}

#[test]
fn test_nv_24_window_sort_kept_after_restart() {
    // [NV-24][SR-12] 窓で決めた並べ替えは、開き直しても同じ。
    let tmp = notes("nv24keep");
    let mut a = boot(&tmp, "", false);
    add_due(&mut a);
    assert_eq!(a.settings.sorts, [("due".to_string(), Dir::Asc)]);
    ch(&mut a, 'q');
    let b = boot(&tmp, "", false);
    assert_eq!(b.settings.sorts, [("due".to_string(), Dir::Asc)]);
    assert_eq!(labels(&b), ["b.md", "c.md", "a.md"]);
}

#[test]
fn test_nv_24_readonly_sorts_without_writing() {
    // [NV-24] 読むだけの起動でも並べ替えられ、ノートも状態も書かない。
    let tmp = notes("nv24ro");
    let before = std::fs::read(tmp.notes().join("a.md")).unwrap();
    let mut a = boot(&tmp, "", true);
    add_due(&mut a);
    assert_eq!(labels(&a), ["b.md", "c.md", "a.md"]);
    ch(&mut a, 's');
    assert_eq!(std::fs::read(tmp.notes().join("a.md")).unwrap(), before);
    assert!(
        !tmp.0.join("state").exists()
            || std::fs::read_dir(tmp.0.join("state"))
                .unwrap()
                .next()
                .is_none()
    );
}

#[test]
fn test_nv_24_hidden_bar_and_narrow() {
    // [NV-24] 検索の欄を隠しても、パレットのコマンドと操作の一覧で開ける。ボタンを省いた幅では、
    // 右の端は検索の欄のまま(並べ替えの窓を開かない)。
    let tmp = notes("nv24bar");
    let mut a = boot(&tmp, "[display]\nsearch_bar = false\n", false);
    let all = super::keymap::commands(false).any(|c| c.action == Action::SortMenu);
    assert!(all, "パレットに並べ替え");
    assert!(super::menu::items(&a)
        .iter()
        .any(|i| i.action == Action::SortMenu));
    a.apply(Action::SortMenu);
    assert_eq!(a.mode, Mode::Sorts);
    press(&mut a, KeyCode::Esc);
    // 狭い端末で長い語: ボタンを省く。
    let mut b = boot(&tmp, "", false);
    b.resize(30, 24);
    b.filter = Some("x".repeat(40));
    let y = (0..24u16)
        .find(|&y| super::bands::bar_at(&b, 0, y))
        .expect("検索の欄の行");
    assert!(!super::bands::sort_button_at(&b, 29, y));
}
