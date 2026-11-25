//! 見た目の区画の「カスタム」(screen.look-section-custom・custom-colors・custom-save・custom-label): テーマの一覧の最後の
//! 「カスタム」で、今のテーマを元に役割ごとの色を選んで直し、反映で選んだ範囲に元のテーマと上書きの色として残す。
//! 項目の並び: 0 保存先・1 テーマ・2 組・3 丸い札の端・(カスタムのとき)4 元のテーマ・5〜20 役割(colors::ROLES の順)・
//! テンプレート…・名前を付けて保存・この範囲の上書きを外す。

use super::keymap::Mode;
use super::look_section::LOOK_FIELDS;
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{press, screen, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::theme::Theme;
use ratatui::crossterm::event::KeyCode;

/// notes/tasks の表(印 `marker` があれば notes に置く)と、設定の置き場(`ui` があれば ui.toml)。
fn setup(name: &str, marker: Option<&str>, ui: Option<&str>) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join("tasks")).unwrap();
    std::fs::write(tmp.notes().join("tasks/a.md"), "---\nstatus: todo\n---\n").unwrap();
    if let Some(m) = marker {
        std::fs::create_dir_all(tmp.notes().join(".mdgrid")).unwrap();
        std::fs::write(tmp.notes().join(".mdgrid/workspace.toml"), m).unwrap();
    }
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    if let Some(u) = ui {
        std::fs::write(tmp.0.join("config/ui.toml"), u).unwrap();
    }
    tmp
}

fn boot(tmp: &Tmp, config: &str) -> App {
    let target = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&target)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::Rgb);
    app.prof.ui = mdgrid::uifile::load(&tmp.0.join("config")).0;
    app.resize(100, 40);
    app.start(Startup {
        config: mdgrid::config::parse(config).unwrap().0,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

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
    a.draft.as_mut().unwrap().sel[Sec::Look as usize] = row;
}

/// 項目 `field` の選び手を開き、`name` で始まる行を選んで決める。
fn choose(a: &mut App, field: usize, name: &str) {
    look_section(a, field);
    press(a, KeyCode::Enter);
    let items = a.look_pick_items(field);
    let i = items
        .iter()
        .position(|s| s.starts_with(name))
        .unwrap_or_else(|| panic!("{name} が無い: {items:?}"));
    match a.draft.as_mut().unwrap().pick.as_mut() {
        Some(Pick::Look { sel, .. }) => *sel = i,
        other => panic!("選び手が開かない: {other:?}"),
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

/// 役割の名前の添字(colors::ROLES の順)。
fn role(name: &str) -> usize {
    mdgrid::colors::ROLES
        .iter()
        .position(|r| *r == name)
        .unwrap()
}

/// 役割の行の項目の番号(カスタムのとき)。
fn role_row(name: &str) -> usize {
    LOOK_FIELDS + 1 + role(name)
}

/// 写しの役割の色。
fn draft_color(a: &App, name: &str) -> Option<[u8; 3]> {
    a.draft.as_ref().unwrap().look.colors[role(name)]
}

fn ui_toml(tmp: &Tmp) -> String {
    std::fs::read_to_string(tmp.0.join("config/ui.toml")).unwrap_or_default()
}

const ORANGE: [u8; 3] = [0xea, 0x8c, 0x34];
const AMBER: [u8; 3] = [0xe0, 0xa4, 0x58];

#[test]
fn test_custom_opens_role_rows_and_named_theme_drops_them() {
    // [utest -> req~screen.look-section-custom~1, req~screen.look-section-items~2] テーマの一覧の最後は「カスタム」。選ぶとテーマの欄が
    // 「カスタム(元: dozy-pink)」になり、元のテーマの行と16の役割の行が出る。元のテーマを変えても色は残り、名前のテーマを
    // 選び直すと役割の行と上書きの色が消える。
    let tmp = setup("custom-open", None, None);
    let mut a = boot(&tmp, "[look]\ntheme = \"dozy-pink\"\n");
    look_section(&mut a, 1);
    let n0 = a.draft.as_ref().unwrap().look_n;
    assert_eq!(
        a.look_pick_items(1).last().map(String::as_str),
        Some("カスタム")
    );
    choose(&mut a, 1, "カスタム");
    let s = screen(&a);
    assert!(s.contains("カスタム(元: dozy-pink)"), "{s}");
    assert!(s.contains("元のテーマ") && s.contains("dozy-pink"), "{s}");
    for label in ["地", "文字", "アクセント", "選んだ行の地"] {
        assert!(s.contains(label), "{label} の行が無い: {s}");
    }
    assert_eq!(
        a.draft.as_ref().unwrap().look_n,
        n0 + 1 + mdgrid::colors::ROLES.len()
    );
    choose(&mut a, role_row("accent"), "orange");
    assert_eq!(draft_color(&a, "accent"), Some(ORANGE));
    // 元のテーマを nord に替えても、アクセントは orange のまま。
    choose(&mut a, LOOK_FIELDS, "nord");
    assert!(screen(&a).contains("カスタム(元: nord)"), "{}", screen(&a));
    assert_eq!(draft_color(&a, "accent"), Some(ORANGE));
    // 名前のテーマを選び直すと、カスタムをやめる。
    choose(&mut a, 1, "sumi");
    let d = a.draft.as_ref().unwrap();
    assert!(d.look.colors.iter().all(Option::is_none), "{:?}", d.look);
    assert_eq!(d.look_n, n0);
    let s = screen(&a);
    assert!(s.contains("▾ sumi") && !s.contains("元のテーマ"), "{s}");
}

#[test]
fn test_custom_colors_pick_type_and_keep_theme() {
    // [utest -> req~screen.look-section-custom-colors~1] 役割の色は、見本の色(orange → #ea8c34)・打った #rrggbb・テーマのまま から選ぶ。
    // 読めない色は理由を出して入力の欄を開いたままにし、色は変えない。
    let tmp = setup("custom-colors", None, None);
    let mut a = boot(&tmp, "[look]\ntheme = \"dozy-pink\"\n");
    choose(&mut a, 1, "カスタム");
    let row = role_row("accent");
    choose(&mut a, row, "orange");
    look_section(&mut a, row);
    assert!(screen(&a).contains("#ea8c34"), "{}", screen(&a));
    // 打つ。
    choose(&mut a, row, "#rrggbb");
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "#e0a458");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert_eq!(draft_color(&a, "accent"), Some(AMBER));
    look_section(&mut a, row);
    assert!(screen(&a).contains("#e0a458"), "{}", screen(&a));
    // 読めない色。
    choose(&mut a, row, "#rrggbb");
    typing(&mut a, "#zz");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText, "欄は開いたまま");
    assert!(
        a.message.as_deref().unwrap_or("").contains("読めない"),
        "{:?}",
        a.message
    );
    assert_eq!(draft_color(&a, "accent"), Some(AMBER));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Settings);
    // テーマのまま。
    choose(&mut a, row, "テーマのまま");
    assert_eq!(draft_color(&a, "accent"), None);
    let theme = Theme::DozyPink.palette().unwrap().colhead;
    look_section(&mut a, row);
    let hex = mdgrid::colors::hex(theme);
    assert!(screen(&a).contains(&hex), "{hex}: {}", screen(&a));
}

#[test]
fn test_custom_apply_saves_base_theme_and_colors() {
    // [utest -> req~screen.look-section-custom-save~1, req~screen.profile-scopes-marker-look~1] 印に dozy-pink の表で、保存先「全体」・カスタムで
    // アクセントを #e0a458 にして反映 → 画面の色が変わり、ui.toml に theme = "dozy-pink" と accent だけ。開き直しても同じで、
    // テーマの欄は「カスタム(元: dozy-pink)」の(全体)。
    let tmp = setup(
        "custom-save",
        Some("name = \"N\"\n[look]\ntheme = \"dozy-pink\"\n"),
        None,
    );
    let mut a = boot(&tmp, "");
    choose(&mut a, 1, "カスタム");
    choose(&mut a, role_row("accent"), "#rrggbb");
    typing(&mut a, "#e0a458");
    press(&mut a, KeyCode::Enter);
    apply(&mut a);
    assert_eq!(a.theme, Theme::DozyPink);
    assert_eq!(a.colors.role("accent"), Some(AMBER));
    assert_eq!(a.palette().unwrap().colhead, AMBER, "画面のアクセントの色");
    let t: toml::Table = ui_toml(&tmp).parse().unwrap();
    assert_eq!(t["look"]["theme"].as_str(), Some("dozy-pink"), "{t}");
    let colors = t["look"]["colors"].as_table().unwrap();
    assert_eq!(colors.len(), 1, "上書きの無い役割は書かない: {colors:?}");
    assert_eq!(colors["accent"].as_str(), Some("#e0a458"));
    let mut b = boot(&tmp, "");
    assert_eq!(b.colors.role("accent"), Some(AMBER));
    look_section(&mut b, 1);
    let s = screen(&b);
    assert!(s.contains("▾ カスタム(元: dozy-pink)  (全"), "{s}");
    assert_eq!(b.look_origin("look.theme"), mdgrid::i18n::Msg::OriginGlobal);
    // この範囲の上書きを外すと、色も外れる。
    let n = b.draft.as_ref().unwrap().look_n;
    look_section(&mut b, n - 1);
    press(&mut b, KeyCode::Enter);
    assert_eq!(b.colors.role("accent"), None);
    assert!(!ui_toml(&tmp).contains("accent"), "{}", ui_toml(&tmp));
}

#[test]
fn test_named_theme_drops_custom_colors_keeps_value_colors() {
    // [utest -> req~screen.look-section-custom-save~1] ui.toml にカスタム(dozy-pink + accent)と値の色がある。保存先「全体」で nord を
    // 選んで反映 → theme は nord、accent は消え、値の色 done は残る。
    let tmp = setup(
        "custom-drop",
        None,
        Some("[look]\ntheme = \"dozy-pink\"\n\n[look.colors]\naccent = \"#e0a458\"\n\n[look.colors.values]\ndone = \"green\"\n"),
    );
    let mut a = boot(&tmp, "");
    assert_eq!(a.colors.role("accent"), Some(AMBER));
    choose(&mut a, 1, "nord");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Nord);
    assert_eq!(a.colors.role("accent"), None);
    assert!(a.colors.value("done").is_some());
    let t: toml::Table = ui_toml(&tmp).parse().unwrap();
    assert_eq!(t["look"]["theme"].as_str(), Some("nord"), "{t}");
    let colors = t["look"]["colors"].as_table().unwrap();
    assert!(colors.get("accent").is_none(), "{colors:?}");
    assert!(colors["values"].get("done").is_some(), "{colors:?}");
}

#[test]
fn test_custom_template_keeps_colors() {
    // [utest -> req~screen.look-section-custom-save~1, req~screen.look-section-template-save~1] カスタムの写しを「夜」で保存 → ui.toml の
    // [templates.夜.look] に元のテーマ、[templates.夜.look.colors] に accent。
    let tmp = setup("custom-template", None, None);
    let mut a = boot(&tmp, "[look]\ntheme = \"dozy-pink\"\n");
    choose(&mut a, 1, "カスタム");
    choose(&mut a, role_row("accent"), "#rrggbb");
    typing(&mut a, "#e0a458");
    press(&mut a, KeyCode::Enter);
    // テンプレートが無いので、役割の行のあとの最初の行が「名前を付けて保存」。
    look_section(&mut a, role_row("highlight") + 1);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "夜");
    press(&mut a, KeyCode::Enter);
    let t: toml::Table = ui_toml(&tmp).parse().unwrap();
    let look = &t["templates"]["夜"]["look"];
    assert_eq!(look["theme"].as_str(), Some("dozy-pink"), "{t}");
    assert_eq!(look["colors"]["accent"].as_str(), Some("#e0a458"), "{t}");
}

#[test]
fn test_custom_label_from_config_colors() {
    // [utest -> req~screen.look-section-custom-label~1] config.toml に sumi と accent = "orange" → テーマの欄は「カスタム(元: sumi)」で、
    // アクセントの行が #ea8c34。値の色だけなら「sumi」のまま。
    let tmp = setup("custom-label", None, None);
    let mut a = boot(
        &tmp,
        "[look]\ntheme = \"sumi\"\n[look.colors]\naccent = \"orange\"\n",
    );
    look_section(&mut a, 1);
    let s = screen(&a);
    assert!(s.contains("カスタム(元: sumi)"), "{s}");
    look_section(&mut a, role_row("accent"));
    assert!(screen(&a).contains("#ea8c34"), "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    let mut b = boot(
        &tmp,
        "[look]\ntheme = \"sumi\"\n[look.colors.values]\ndone = \"green\"\n",
    );
    look_section(&mut b, 1);
    let s = screen(&b);
    assert!(s.contains("▾ sumi") && !s.contains("カスタム("), "{s}");
}
