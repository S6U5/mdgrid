//! ビューの設定の画面の「見た目」の区画(SR-43): 保存先(既定は全体)・テーマ・組・丸い札の端を選んで反映し、
//! 全体なら ui.toml に残す。テンプレートの保存・当てる・消す、この範囲の上書きを外す、読むだけ。
//! 項目の並び: 0 保存先・1 テーマ・2 組・3 丸い札の端・4〜 テンプレート・名前を付けて保存・上書きを外す。

use super::keymap::Mode;
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_grid::{base_path, vault, TASKS};
use super::test_screen::{press, screen, typing, Tmp};
use super::*;
use mdgrid::style::Preset;
use mdgrid::theme::Theme;
use ratatui::crossterm::event::KeyCode;

fn boot(tmp: &Tmp, readonly: bool) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 26);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly,
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

/// 起動の前に main がするように、ui.toml を読んで渡して起動する。
fn boot_ui(tmp: &Tmp) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.prof.ui = mdgrid::uifile::load(&tmp.0.join("config")).0;
    app.resize(100, 26);
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
    base_path(&tmp, TASKS);
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn ui_toml(tmp: &Tmp) -> String {
    std::fs::read_to_string(tmp.0.join("config/ui.toml")).unwrap_or_default()
}

/// 設定の画面を開いて見た目の区画の `row` 行目を選ぶ。
fn look_section(a: &mut App, row: usize) {
    if a.mode != Mode::Settings {
        press(a, KeyCode::Char('o'));
    }
    for _ in 0..12 {
        if a.draft.as_ref().unwrap().sec == Sec::Look {
            break;
        }
        press(a, KeyCode::Tab);
    }
    let d = a.draft.as_mut().unwrap();
    assert_eq!(d.sec, Sec::Look);
    d.sel[Sec::Look as usize] = row;
}

/// 項目 `field` の選び手を開いて、`name` の行を選んで決める。
fn choose(a: &mut App, field: usize, name: &str) {
    look_section(a, field);
    press(a, KeyCode::Enter);
    let items = a.look_pick_items(field);
    let i = items.iter().position(|s| s.starts_with(name)).unwrap();
    if let Some(Pick::Look { sel, .. }) = a.draft.as_mut().unwrap().pick.as_mut() {
        *sel = i;
    } else {
        panic!("選び手が開かない");
    }
    press(a, KeyCode::Enter);
}

fn apply(a: &mut App) {
    let d = a.draft.as_mut().unwrap();
    d.pick = None;
    d.sec = Sec::Buttons;
    d.sel[Sec::Buttons as usize] = 0;
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_sr_43_choose_apply_and_save() {
    // [SR-43] 左に「見た目 sumi」、保存先は「全体」。dracula と dozy-pink を選ぶ → 「未反映 1」、反映 → 画面の
    // テーマと組が変わり、ui.toml に残る。開き直すと区画の写しはその値で、出どころは「全体」。
    let tmp = setup("sr43a");
    let mut a = boot(&tmp, false);
    look_section(&mut a, 0);
    let s = screen(&a);
    assert!(s.contains("見た目      sumi"), "{s}");
    assert!(s.contains("▾ 全体"), "{s}");
    assert!(s.contains("▾ default  (既定)"), "{s}");
    choose(&mut a, 1, "dracula");
    choose(&mut a, 2, "dozy-pink");
    choose(&mut a, 3, "true");
    let s = screen(&a);
    assert!(s.contains("未反映 1"), "{s}");
    assert!(s.contains("▾ dracula") && s.contains("▾ dozy-pink"), "{s}");
    assert_eq!(a.theme, Theme::Default, "反映までは変えない");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Dracula);
    assert_eq!(a.style.preset, Preset::DozyPink);
    assert!(a.nerd_font);
    let t = ui_toml(&tmp);
    assert!(t.contains("[look]"), "{t}");
    assert!(t.contains("theme = \"dracula\""), "{t}");
    assert!(t.contains("preset = \"dozy-pink\""), "{t}");
    assert!(t.contains("nerd_font = true"), "{t}");
    let mut b = boot_ui(&tmp);
    look_section(&mut b, 0);
    assert!(screen(&b).contains("▾ dracula  (全体)"), "{}", screen(&b));
    assert!(!screen(&b).contains("未反映"));
}

#[test]
fn test_sr_43_templates_save_use_delete_and_reset() {
    // [SR-43] 今の写しを「夜」で保存 → ui.toml の [templates] と一覧に出る。別の組にしてから「夜」を当てる → 写しが
    // 夜の見た目に。d で消す。この範囲の上書きを外す → ui.toml の見た目が消え、テンプレートは残る。
    let tmp = setup("sr43b");
    let mut a = boot(&tmp, false);
    choose(&mut a, 1, "nord");
    choose(&mut a, 2, "paper");
    apply(&mut a);
    // 保存(テンプレートが無いので 4 が「名前を付けて保存」)。
    look_section(&mut a, 4);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "夜");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert!(
        ui_toml(&tmp).contains("[templates.夜.look]"),
        "{}",
        ui_toml(&tmp)
    );
    assert!(screen(&a).contains("◆ 夜"), "{}", screen(&a));
    // 別の組にしてから夜を当てる。
    choose(&mut a, 2, "grid");
    look_section(&mut a, 4);
    press(&mut a, KeyCode::Enter);
    let d = a.draft.as_ref().unwrap();
    assert_eq!(d.look.preset, Preset::Paper);
    assert_eq!(d.look.theme.label(), "nord");
    // 消す。
    look_section(&mut a, 4);
    press(&mut a, KeyCode::Char('d'));
    assert!(!ui_toml(&tmp).contains("夜"), "{}", ui_toml(&tmp));
    // テンプレートを1つ足して、この範囲の上書きを外す(最後の行)。
    look_section(&mut a, 4);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "朝");
    press(&mut a, KeyCode::Enter);
    let n = a.draft.as_ref().unwrap().look_n;
    look_section(&mut a, n - 1);
    press(&mut a, KeyCode::Enter);
    let t = ui_toml(&tmp);
    assert!(!t.contains("theme = \"nord\""), "{t}");
    assert!(t.contains("[templates.朝.look]"), "{t}");
    assert_eq!(
        a.theme,
        Theme::Default,
        "上書きを外すとすぐ config.toml のとおり"
    );
    assert!(
        a.message.as_deref().unwrap_or("").contains("全体"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_sr_43_readonly_applies_without_writing() {
    // [SR-43][WB-15] 読むだけでも反映で画面には効くが、ui.toml は書かない。テンプレートの保存は理由を出す。
    let tmp = setup("sr43c");
    let mut a = boot(&tmp, true);
    choose(&mut a, 1, "gruvbox");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Gruvbox);
    assert!(!tmp.0.join("config/ui.toml").exists());
    look_section(&mut a, 4);
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::SettingsText);
    assert!(
        a.message.as_deref().unwrap_or("").contains("読むだけ"),
        "{:?}",
        a.message
    );
}
