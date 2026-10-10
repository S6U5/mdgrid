//! 見た目の区画は変えた項目だけを look.toml に残し、設定の auto を保つ(SR-43)。

use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{press, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp, config: mdgrid::config::Config) -> App {
    let p = tmp.notes().to_path_buf();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 26);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn setup(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn choose(a: &mut App, field: usize, name: &str) {
    press(a, KeyCode::Char('o'));
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Look {
            break;
        }
        press(a, KeyCode::Tab);
    }
    a.draft.as_mut().unwrap().sel[Sec::Look as usize] = field;
    press(a, KeyCode::Enter);
    let i = App::look_pick_items(field)
        .iter()
        .position(|s| s.starts_with(name))
        .unwrap();
    if let Some(Pick::Look { sel, .. }) = a.draft.as_mut().unwrap().pick.as_mut() {
        *sel = i;
    }
    press(a, KeyCode::Enter);
    let d = a.draft.as_mut().unwrap();
    d.sec = Sec::Buttons;
    d.sel[Sec::Buttons as usize] = 0;
    press(a, KeyCode::Enter);
}

#[test]
fn test_sr_43_only_changed_items_saved() {
    // [SR-43] テーマだけを変えて反映 → look.toml には theme だけ。config.toml の部品ごとの形(status = pill)は残る。
    let tmp = setup("sr43keep");
    let mut c = mdgrid::config::Config::default();
    c.style.status = mdgrid::style::Status::Pill;
    let mut a = boot(&tmp, c);
    choose(&mut a, 0, "nord");
    let t = std::fs::read_to_string(tmp.0.join("config/look.toml")).unwrap();
    assert!(t.contains("theme = \"nord\""), "{t}");
    assert!(!t.contains("preset") && !t.contains("nerd_font"), "{t}");
    assert_eq!(
        a.style.status,
        mdgrid::style::Status::Pill,
        "部品ごとの形は残る"
    );
}

#[test]
fn test_sr_43_auto_kept() {
    // [SR-43] 設定が nerd_font = "auto"(既定)なら、区画は auto と見せ、ほかを変えても look.toml に false を書かない。
    let tmp = setup("sr43auto");
    let mut a = boot(&tmp, mdgrid::config::Config::default());
    press(&mut a, KeyCode::Char('o'));
    assert_eq!(
        a.draft.as_ref().unwrap().look.nerd,
        mdgrid::look::Nerd::Auto
    );
    press(&mut a, KeyCode::Esc);
    choose(&mut a, 1, "paper");
    let t = std::fs::read_to_string(tmp.0.join("config/look.toml")).unwrap();
    assert!(t.contains("preset = \"paper\""), "{t}");
    assert!(!t.contains("nerd_font"), "{t}");
    // 日本語の選び手は訳つき。
    assert!(App::look_pick_items(2).iter().any(|s| s.contains("オフ")));
}
