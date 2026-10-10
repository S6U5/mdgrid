//! 既定のビュー(NV-25・BV-13): 「既定のビューにする」で views.toml に名前を残し、次の起動はそのビュー。

use super::startup::Startup;
use super::test_grid::{base_path, vault, TASKS};
use super::test_screen::Tmp;
use super::*;

/// main と同じ道筋で `.base` を開く(`--view` は `view`)。設定の置き場は tmp/config。
fn boot(tmp: &Tmp, view: Option<&str>, readonly: bool) -> App {
    let p = tmp.notes().join("tasks.base");
    let t = crate::open_target(std::slice::from_ref(&p), view).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
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

fn current(a: &App) -> String {
    a.view_names()[a.view_index()].clone()
}

#[test]
fn test_nv_25_default_view() {
    // [NV-25][BV-13] 「完了」を既定のビューにする → views.toml に default_view。開き直すと「完了」。
    // `--view` があればそちら。先頭(進行中)を選ぶと名前を消す。
    let tmp = vault("nv25");
    base_path(&tmp, TASKS);
    let mut a = boot(&tmp, None, false);
    assert_eq!(current(&a), "進行中");
    a.select_view_named("完了");
    a.apply(super::keymap::Action::SetDefaultView);
    let text = std::fs::read_to_string(tmp.0.join("config/views.toml")).unwrap();
    assert!(text.contains("default_view = \"完了\""), "{text}");
    let b = boot(&tmp, None, false);
    assert_eq!(current(&b), "完了");
    let c = boot(&tmp, Some("全部"), false);
    assert_eq!(current(&c), "全部", "--view が勝つ");
    let mut d = boot(&tmp, None, false);
    d.select_view_named("進行中");
    d.apply(super::keymap::Action::SetDefaultView);
    let text = std::fs::read_to_string(tmp.0.join("config/views.toml")).unwrap();
    assert!(!text.contains("default_view"), "{text}");
    assert_eq!(current(&boot(&tmp, None, false)), "進行中");
}

#[test]
fn test_nv_25_missing_and_readonly() {
    // [NV-25] 既定のビューの名前が無くなっていたら、警告せず先頭。読むだけでは書かない。
    let tmp = vault("nv25miss");
    base_path(&tmp, TASKS);
    let dir = tmp.0.join("config");
    std::fs::create_dir_all(&dir).unwrap();
    mdgrid::views::save_default_view(&dir, &tmp.notes().join("tasks.base"), Some("消えたビュー"))
        .unwrap();
    let a = boot(&tmp, None, false);
    assert_eq!(current(&a), "進行中");
    assert_eq!(a.message, None);
    let mut r = boot(&tmp, None, true);
    r.select_view_named("完了");
    r.apply(super::keymap::Action::SetDefaultView);
    let text = std::fs::read_to_string(dir.join("views.toml")).unwrap();
    assert!(
        text.contains("消えたビュー"),
        "読むだけでは書かない: {text}"
    );
}
