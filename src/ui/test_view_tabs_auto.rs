//! ビューが1つならタブの行を出さない設定(SR-34)と、既定の表の英語の名前。

use super::startup::Startup;
use super::test_screen::{screen, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::views::{save_views, NativeView};

fn folder(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ntitle: alpha\n---\n");
    tmp.write("b.md", "---\ntitle: beta\n---\n");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn boot(tmp: &Tmp, config: Config) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
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

fn auto() -> Config {
    let (c, w) = mdgrid::config::parse("view_tabs = \"auto\"\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    c
}

/// その文字を含む最初の行の番号。
fn row_of(s: &str, text: &str) -> usize {
    s.lines()
        .position(|l| l.contains(text))
        .unwrap_or_else(|| panic!("{text} が無い:\n{s}"))
}

#[test]
fn test_sr_34_auto_hides_single_tab() {
    // [SR-34] 既定(always)はタブの行に既定の表。auto でビューが1つなら、タブの行もヘッダーの名前も無く、表が1行上がる。
    let tmp = folder("sr34_auto");
    let a = boot(&tmp, Config::default());
    let s = screen(&a);
    assert!(s.lines().nth(1).unwrap().contains("既定の表"), "{s}");
    let before = row_of(&s, "alpha");
    let b = boot(&tmp, auto());
    let s = screen(&b);
    assert!(
        !s.contains("既定の表"),
        "タブの行もヘッダーの名前も無い:\n{s}"
    );
    assert_eq!(row_of(&s, "alpha"), before - 1, "表が1行上がる:\n{s}");
}

#[test]
fn test_sr_34_auto_shows_tabs_with_two_views() {
    // [SR-34] auto でも、ビューを保存して2つ以上になればタブの行を出す。
    let tmp = folder("sr34_two");
    save_views(
        &tmp.0.join("config"),
        &tmp.notes(),
        &[NativeView {
            name: "mine".into(),
            ..Default::default()
        }],
    )
    .unwrap();
    let a = boot(&tmp, auto());
    let s = screen(&a);
    let tabs = s.lines().nth(1).unwrap();
    assert!(tabs.contains("既定の表") && tabs.contains("mine"), "{s}");
}

#[test]
fn test_sr_34_unknown_value_warns() {
    // [SR-34] 知らない値は警告して always。
    let (c, w) = mdgrid::config::parse("view_tabs = \"sometimes\"\n").unwrap();
    assert!(!c.view_tabs_auto);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("view_tabs"), "{w:?}");
}

#[test]
fn test_sr_23_default_tab_english_name() {
    // [SR-23] 英語の既定の表の名前は All notes。前の名前 Default は別名として受け、ビューの名前には使えない。
    let _g = mdgrid::i18n::scoped(mdgrid::i18n::Lang::En);
    let tmp = folder("sr34_en");
    let mut a = boot(&tmp, Config::default());
    let s = screen(&a);
    assert!(s.lines().nth(1).unwrap().contains("All notes"), "{s}");
    a.select_view_named("Default");
    assert!(a.message.is_none(), "{:?}", a.message);
    assert_eq!(a.view_index(), 0);
    assert!(
        a.name_problem(&[], "Default", None).is_some(),
        "前の名前は使えない"
    );
}
