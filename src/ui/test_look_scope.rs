//! 範囲ごとの上書き(SR-44)が起動で効くことと、設定の画面の「見た目」の区画の保存先(SR-43): この表・
//! このワークスペース・このビュー、出どころの表示、この範囲の上書きを外す、印のワークスペースには書かない。
//! タブの行の3つの値(SR-34)。

use super::keymap::Mode;
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{press, screen, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::theme::Theme;
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};

/// notes/tasks・notes/projects の2つの表を作る(印 `marker` があれば notes に置く)。
fn tables(name: &str, marker: Option<&str>) -> Tmp {
    let tmp = Tmp::new(name);
    for dir in ["tasks", "projects"] {
        std::fs::create_dir_all(tmp.notes().join(dir)).unwrap();
        std::fs::write(
            tmp.notes().join(dir).join("a.md"),
            "---\nstatus: todo\n---\n",
        )
        .unwrap();
    }
    if let Some(m) = marker {
        std::fs::create_dir_all(tmp.notes().join(".mdgrid")).unwrap();
        std::fs::write(tmp.notes().join(".mdgrid/workspace.toml"), m).unwrap();
    }
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn real(p: &Path) -> String {
    std::fs::canonicalize(p)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn boot_at(tmp: &Tmp, table: &str, config: &str) -> App {
    let target: PathBuf = tmp.notes().join(table);
    let src = Markdown::open(std::slice::from_ref(&target)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::Rgb);
    app.prof.ui = mdgrid::uifile::load(&tmp.0.join("config")).0;
    app.resize(100, 26);
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

fn choose(a: &mut App, field: usize, name: &str) {
    look_section(a, field);
    press(a, KeyCode::Enter);
    let items = a.look_pick_items(field);
    let i = items
        .iter()
        .position(|s| s.starts_with(name))
        .unwrap_or_else(|| panic!("{name} が無い: {items:?}"));
    if let Some(Pick::Look { sel, .. }) = a.draft.as_mut().unwrap().pick.as_mut() {
        *sel = i;
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

/// workspaces.toml にワークスペース W(tasks・projects)を書く。`ws`・`table` は `[workspace.look]`・tasks の
/// `[workspace.table.look]` に足す行。
fn app_workspace(tmp: &Tmp, ws: &str, table: &str) {
    std::fs::write(
        tmp.0.join("config/workspaces.toml"),
        format!(
            "[[workspace]]\nname = \"W\"\n\n[workspace.look]\n{ws}\n[[workspace.table]]\nname = \"tasks\"\npath = {}\n\n[workspace.table.look]\n{table}\n[[workspace.table]]\nname = \"projects\"\npath = {}\n",
            toml::Value::String(real(&tmp.notes().join("tasks"))),
            toml::Value::String(real(&tmp.notes().join("projects")))
        ),
    )
    .unwrap();
}

#[test]
fn test_sr_44_workspace_and_table_overrides_at_start() {
    // [utest -> req~screen.profile-scopes-order~3] config.toml の sumi、workspaces.toml のワークスペースの nord、その表の項目の dracula
    // → tasks は dracula、projects は nord。views.toml の表の項目はワークスペースの表の項目より先(同じ範囲では画面が書いたもの)。
    // ワークスペースの外の表は config.toml の sumi。
    let tmp = tables("sr44start", None);
    app_workspace(&tmp, "theme = \"nord\"", "theme = \"dracula\"");
    let cfg = "[look]\ntheme = \"sumi\"\n";
    assert_eq!(boot_at(&tmp, "tasks", cfg).theme, Theme::Dracula);
    assert_eq!(boot_at(&tmp, "projects", cfg).theme, Theme::Nord);
    std::fs::write(
        tmp.0.join("config/views.toml"),
        format!(
            "[[table]]\npath = {}\n[table.look]\ntheme = \"paper\"\n[table.display]\nzebra = true\n",
            toml::Value::String(real(&tmp.notes().join("tasks")))
        ),
    )
    .unwrap();
    let a = boot_at(&tmp, "tasks", cfg);
    assert_eq!(a.theme, Theme::Paper);
    assert!(a.display.zebra);
    // ワークスペースの外の表は config.toml のとおり。
    let other = tables("sr44none", None);
    assert_eq!(boot_at(&other, "tasks", cfg).theme, Theme::Sumi);
}

#[test]
fn test_sr_43_save_to_this_table_shows_origin_and_reset() {
    // [utest -> req~screen.look-section-save~2, req~screen.look-section-clear~1] 保存先「この表」でテーマ nord を反映 → views.toml の対象の [table.look]、横に「(この表)」、
    // ほかの表と ui.toml は変わらない。「この範囲の上書きを外す」→ その表も既定。
    let tmp = tables("sr43table", None);
    let mut a = boot_at(&tmp, "tasks", "");
    choose(&mut a, 0, "この表");
    choose(&mut a, 1, "nord");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Nord);
    let views = std::fs::read_to_string(tmp.0.join("config/views.toml")).unwrap();
    assert!(
        views.contains("[table.look]") && views.contains("theme = \"nord\""),
        "{views}"
    );
    assert!(!tmp.0.join("config/ui.toml").exists());
    look_section(&mut a, 1);
    assert!(screen(&a).contains("▾ nord  (この表)"), "{}", screen(&a));
    press(&mut a, KeyCode::Esc);
    assert_eq!(boot_at(&tmp, "projects", "").theme, Theme::Default);
    // 外す(保存先をこの表にしてから最後の行)。
    let mut b = boot_at(&tmp, "tasks", "");
    assert_eq!(b.theme, Theme::Nord);
    choose(&mut b, 0, "この表");
    let n = b.draft.as_ref().unwrap().look_n;
    look_section(&mut b, n - 1);
    press(&mut b, KeyCode::Enter);
    assert_eq!(b.theme, Theme::Default);
    let views = std::fs::read_to_string(tmp.0.join("config/views.toml")).unwrap_or_default();
    assert!(!views.contains("nord"), "{views}");
}

#[test]
fn test_sr_43_save_to_app_workspace() {
    // [utest -> req~screen.look-section-save~2, req~screen.look-section-marker~1] workspaces.toml のワークスペースなら「このワークスペース」で [workspace.look] に書く。
    let tmp = tables("sr43ws", None);
    std::fs::write(
        tmp.0.join("config/workspaces.toml"),
        format!(
            "# mine\n[[workspace]]\nname = \"W\"\n\n[[workspace.table]]\nname = \"tasks\"\npath = {}\n\n[[workspace.table]]\nname = \"projects\"\npath = {}\n",
            toml::Value::String(real(&tmp.notes().join("tasks"))),
            toml::Value::String(real(&tmp.notes().join("projects")))
        ),
    )
    .unwrap();
    let mut a = boot_at(&tmp, "tasks", "");
    choose(&mut a, 0, "このワークスペース");
    choose(&mut a, 1, "gruvbox");
    apply(&mut a);
    let ws = std::fs::read_to_string(tmp.0.join("config/workspaces.toml")).unwrap();
    assert!(
        ws.contains("# mine") && ws.contains("[workspace.look]"),
        "{ws}"
    );
    assert_eq!(boot_at(&tmp, "projects", "").theme, Theme::Gruvbox);
}

#[test]
fn test_marker_look_is_below_global() {
    // [utest -> req~screen.profile-scopes-marker-look~1] 印の [look] は作り手の勧める既定で、全体より広い範囲に重ねる: ui.toml・config.toml の
    // テーマが印に勝つ。人が何も選ばなければ印のまま(印の表の項目は印の最上位より狭い)。項目ごとに重ね、[display] は今までどおり。
    let marker = "name = \"N\"\n[look]\ntheme = \"dozy-pink\"\n";
    let tmp = tables("mlookui", Some(marker));
    std::fs::write(tmp.0.join("config/ui.toml"), "[look]\ntheme = \"sumi\"\n").unwrap();
    let mut a = boot_at(&tmp, "tasks", "");
    assert_eq!(a.theme, Theme::Sumi, "ui.toml が印に勝つ");
    look_section(&mut a, 1);
    assert!(screen(&a).contains("▾ sumi  (全体)"), "{}", screen(&a));
    let tmp = tables("mlookcfg", Some(marker));
    let cfg = "[look]\ntheme = \"sumi\"\n";
    assert_eq!(
        boot_at(&tmp, "tasks", cfg).theme,
        Theme::Sumi,
        "config.toml が印に勝つ"
    );
    let marker = "name = \"N\"\n[look]\ntheme = \"nord\"\npreset = \"saas\"\n[[table]]\npath = \"tasks\"\n[table.look]\ntheme = \"dracula\"\n[[table]]\npath = \"projects\"\n";
    let tmp = tables("mlooknone", Some(marker));
    let a = boot_at(&tmp, "tasks", "");
    assert_eq!(a.theme, Theme::Dracula);
    assert_eq!(a.style.preset.name(), "saas");
    let b = boot_at(&tmp, "projects", "");
    assert_eq!(b.theme, Theme::Nord);
    assert_eq!(b.style.preset.name(), "saas");
    let marker =
        "name = \"N\"\n[look]\ntheme = \"nord\"\npreset = \"saas\"\n[display]\nzebra = true\n";
    let tmp = tables("mlookmix", Some(marker));
    let a = boot_at(
        &tmp,
        "tasks",
        "[look]\ntheme = \"sumi\"\n[display]\nzebra = false\n",
    );
    assert_eq!(a.theme, Theme::Sumi, "テーマは config.toml");
    assert_eq!(a.style.preset.name(), "saas", "config.toml に無い組は印");
    assert!(
        a.display.zebra,
        "[display] は印のワークスペースが全体より先"
    );
}

#[test]
fn test_global_choice_beats_marker_look() {
    // [utest -> req~screen.profile-scopes-marker-look~1, req~screen.look-section-save~2] 印に dozy-pink の表で、保存先「全体」のまま sumi を選んで
    // 反映 → 画面が sumi になり、ui.toml に残り、出どころは「(全体)」。
    let marker = "name = \"N\"\n[look]\ntheme = \"dozy-pink\"\npreset = \"dozy-pink\"\n";
    let tmp = tables("mlookpick", Some(marker));
    let mut a = boot_at(&tmp, "tasks", "");
    assert_eq!(a.theme, Theme::DozyPink);
    choose(&mut a, 1, "sumi");
    apply(&mut a);
    assert_eq!(a.theme, Theme::Sumi);
    let ui = std::fs::read_to_string(tmp.0.join("config/ui.toml")).unwrap();
    assert!(ui.contains("theme = \"sumi\""), "{ui}");
    let mut b = boot_at(&tmp, "tasks", "");
    assert_eq!(b.theme, Theme::Sumi);
    look_section(&mut b, 1);
    let s = screen(&b);
    assert!(s.contains("▾ sumi  (全体)"), "{s}");
    assert!(s.contains("▾ dozy-pink  (印)"), "組は印のまま: {s}");
}

#[test]
fn test_marker_origin_is_shown_as_marker() {
    // [utest -> req~screen.look-section-origin~3] 印から来たテーマと組の横は「(印)」で、「(このワークスペース)」ではない。
    let marker = "name = \"N\"\n[look]\ntheme = \"dozy-pink\"\npreset = \"dozy-pink\"\n";
    let tmp = tables("mlookorigin", Some(marker));
    let mut a = boot_at(&tmp, "tasks", "");
    look_section(&mut a, 1);
    let s = screen(&a);
    assert!(s.contains("▾ dozy-pink  (印)"), "{s}");
    assert!(!s.contains("(このワークスペース)"), "{s}");
}

#[test]
fn test_sr_43_marker_workspace_is_not_written() {
    // [utest -> req~screen.look-section-marker~1] 印のワークスペースを保存先にして反映 → 理由を出し、印は変わらない。
    let marker = "name = \"N\"\n";
    let tmp = tables("sr43marker", Some(marker));
    let mut a = boot_at(&tmp, "tasks", "");
    choose(&mut a, 0, "このワークスペース");
    choose(&mut a, 1, "nord");
    apply(&mut a);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains(".mdgrid/workspace.toml"),
        "{:?}",
        a.message
    );
    let m = std::fs::read_to_string(tmp.notes().join(".mdgrid/workspace.toml")).unwrap();
    assert_eq!(m, marker);
}

#[test]
fn test_sr_43_save_to_this_view() {
    // [utest -> req~screen.look-section-save~2] 保存先「このビュー」は、ビューの設定に残し、開き直しても効く。
    let tmp = tables("sr43view", None);
    let mut a = boot_at(&tmp, "tasks", "");
    choose(&mut a, 0, "このビュー");
    choose(&mut a, 2, "grid");
    apply(&mut a);
    assert_eq!(a.style.preset.name(), "grid");
    assert_eq!(
        a.settings.look.preset.map(mdgrid::style::Preset::name),
        Some("grid")
    );
    a.persist_state();
    let b = boot_at(&tmp, "tasks", "");
    assert_eq!(b.style.preset.name(), "grid");
    assert!(!tmp.0.join("config/ui.toml").exists());
}

#[test]
fn test_sr_34_tabs_never_and_cycle() {
    // [utest -> req~screen.tabs-setting-auto-single~1, req~screen.tabs-setting-cycle~1] display.tabs = "never" → タブが無い(SR-1 でタブはビューの帯の中。帯の行は検索の欄のために残る)。
    // 表示の区画のタブの項目は always → auto → never の順。
    let tmp = tables("sr34never", None);
    let a = boot_at(&tmp, "tasks", "[display]\ntabs = \"never\"\n");
    assert!(super::bands::view_bar(&a, a.size.0 as usize)
        .tabs
        .is_empty());
    let mut b = boot_at(&tmp, "tasks", "");
    press(&mut b, KeyCode::Char('o'));
    let i = mdgrid::display::ITEMS
        .iter()
        .position(|(it, _)| *it == mdgrid::display::Item::Tabs)
        .unwrap();
    use mdgrid::display::TabsMode;
    assert_eq!(b.draft_tabs(), TabsMode::Always);
    b.toggle_display(i);
    assert_eq!(b.draft_tabs(), TabsMode::Auto);
    b.toggle_display(i);
    assert_eq!(b.draft_tabs(), TabsMode::Never);
    b.toggle_display(i);
    assert_eq!(b.draft_tabs(), TabsMode::Always);
}
