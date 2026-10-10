//! ビューの設定の画面の「見た目」の区画(SR-43): テーマ・組・丸い札の端を選んで反映し、look.toml に残す。
//! テンプレートの保存・当てる・消す、config.toml に戻す、読むだけ。

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

fn setup(name: &str) -> Tmp {
    let tmp = vault(name);
    base_path(&tmp, TASKS);
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn look_toml(tmp: &Tmp) -> String {
    std::fs::read_to_string(tmp.0.join("config/look.toml")).unwrap_or_default()
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
    let items = App::look_pick_items(field);
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
    // [SR-43] 左に「見た目 sumi」。dracula と dozy-pink を選ぶ → 「未反映 1」、反映 → 画面のテーマと組が変わり、
    // look.toml に残る。開き直すと区画の写しは look.toml の値。
    let tmp = setup("sr43a");
    let mut a = boot(&tmp, false);
    look_section(&mut a, 0);
    assert!(screen(&a).contains("見た目      sumi"), "{}", screen(&a));
    choose(&mut a, 0, "dracula");
    choose(&mut a, 1, "dozy-pink");
    choose(&mut a, 2, "true");
    let s = screen(&a);
    assert!(s.contains("未反映 1"), "{s}");
    assert!(s.contains("▾ dracula") && s.contains("▾ dozy-pink"), "{s}");
    assert_eq!(a.theme, Theme::Default, "反映までは変えない");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Dracula);
    assert_eq!(a.style.preset, Preset::DozyPink);
    assert!(a.nerd_font);
    let t = look_toml(&tmp);
    assert!(t.contains("theme = \"dracula\""), "{t}");
    assert!(t.contains("preset = \"dozy-pink\""), "{t}");
    assert!(t.contains("nerd_font = true"), "{t}");
    let mut b = boot(&tmp, false);
    look_section(&mut b, 0);
    assert!(screen(&b).contains("▾ dracula"), "{}", screen(&b));
    assert!(!screen(&b).contains("未反映"));
}

#[test]
fn test_sr_43_templates_save_use_delete_and_reset() {
    // [SR-43] 今の写しを「夜」で保存 → 一覧に出る。別の組にしてから「夜」を当てる → 写しが夜の見た目に。
    // d で消す。config.toml に戻す → look.toml の見た目が消え、テンプレートは残る。
    let tmp = setup("sr43b");
    let mut a = boot(&tmp, false);
    choose(&mut a, 0, "nord");
    choose(&mut a, 1, "paper");
    apply(&mut a);
    // 保存(項目の 3 番目のあと: テンプレートが無いので 3 が「名前を付けて保存」)。
    look_section(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "夜");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert!(
        look_toml(&tmp).contains("name = \"夜\""),
        "{}",
        look_toml(&tmp)
    );
    assert!(screen(&a).contains("◆ 夜"), "{}", screen(&a));
    // 別の組にしてから夜を当てる。
    choose(&mut a, 1, "grid");
    look_section(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    let d = a.draft.as_ref().unwrap();
    assert_eq!(d.look.preset, Preset::Paper);
    assert_eq!(d.look.theme.name(), "nord");
    // 消す。
    look_section(&mut a, 3);
    press(&mut a, KeyCode::Char('d'));
    assert!(!look_toml(&tmp).contains("夜"), "{}", look_toml(&tmp));
    // テンプレートを1つ足して、config.toml に戻す(最後の行)。
    look_section(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "朝");
    press(&mut a, KeyCode::Enter);
    let n = a.draft.as_ref().unwrap().look_n;
    look_section(&mut a, n - 1);
    press(&mut a, KeyCode::Enter);
    let t = look_toml(&tmp);
    assert!(!t.contains("theme = \"nord\"\npreset"), "{t}");
    assert!(t.contains("name = \"朝\""), "{t}");
    assert!(
        a.message.as_deref().unwrap_or("").contains("config.toml"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_sr_43_readonly_applies_without_writing() {
    // [SR-43][WB-15] 読むだけでも反映で画面には効くが、look.toml は書かない。テンプレートの保存は理由を出す。
    let tmp = setup("sr43c");
    let mut a = boot(&tmp, true);
    choose(&mut a, 0, "gruvbox");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Gruvbox);
    assert!(!tmp.0.join("config/look.toml").exists());
    look_section(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::SettingsText);
    assert!(
        a.message.as_deref().unwrap_or("").contains("読むだけ"),
        "{:?}",
        a.message
    );
}
