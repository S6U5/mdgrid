//! ヘッダーのパンくずを、キーと ▾ を添えた引き出しのボタンにする(SR-42)受け入れの試験。
//! 記録: specs/_changes/2026-10-12-crumb-button.md(タスク #1)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{text, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::workspace::{self as ws, WsTable};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

fn buf(a: &App) -> Buffer {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

/// ヘッダーの行(2026-10-12-bar-padding-half の上の余白の分だけ下がる)。
fn head(a: &App) -> String {
    text(&buf(a))
        .lines()
        .nth(hy(a) as usize)
        .unwrap_or_default()
        .to_string()
}

/// ヘッダーの画面の行。
fn hy(a: &App) -> u16 {
    super::bands::head_y(a)
}

/// ヘッダーの行で文字 `t` の始まる桁。
fn col(a: &App, t: &str) -> Option<u16> {
    let b = buf(a);
    let cs: Vec<String> = t.chars().map(|c| c.to_string()).collect();
    (0..b.area.width).find(|&x| {
        let mut xx = x;
        cs.iter().all(|c| {
            let hit = xx < b.area.width && b[(xx, hy(a))].symbol() == c;
            xx += width::width(c).max(1) as u16;
            hit
        })
    })
}

/// ワークスペース Work(Tasks・Projects)の tasks を開く。
fn boot_work(name: &str) -> (Tmp, App) {
    let tmp = workspace(name);
    let dir = |d: &str| tmp.notes().join(d).canonicalize().unwrap();
    let t = |name: &str, d: &str| WsTable {
        name: name.into(),
        path: dir(d),
        view: None,
        profile: Default::default(),
    };
    let cfg = tmp.0.join("config");
    ws::add(&cfg, "Work", t("Tasks", "tasks")).unwrap();
    ws::add(&cfg, "Work", t("Projects", "projects")).unwrap();
    let a = boot_tasks(&tmp);
    assert_eq!(a.scope.as_ref().map(|s| s.name.as_str()), Some("Work"));
    (tmp, a)
}

/// 設定の文字を渡して、ノートのフォルダ notes を開く(範囲なし)。
fn boot_config(name: &str, toml: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    let (config, _) = mdgrid::config::parse(toml).unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

#[test]
fn test_sr_42_crumb_button_key_and_arrow() {
    // [SR-42] ワークスペース Work の tasks → 1行目の左が「w Work › tasks ▾  3行」。
    let (_t, a) = boot_work("crumbbtn");
    let h = head(&a);
    let left = h.trim_start();
    assert!(left.starts_with("w Work › "), "{h:?}");
    let rest = left["w Work › ".len()..].to_lowercase();
    assert!(rest.starts_with("tasks ▾"), "表の名前の後ろに ▾: {h:?}");
    let arrow = h.find('▾').unwrap();
    assert!(h[arrow..].contains("3行"), "行の数は ▾ の右: {h:?}");
}

#[test]
fn test_sr_42_crumb_button_looks_like_a_button() {
    // [SR-42] 名前から ▾ までが1つのボタンの見た目(同じ見た目)で、行の数とは見た目が違う。頭のキーの字は
    // 太字で名前と違う見た目(2026-10-12-header-menu。NV-23 と同じ)。
    let (_t, a) = boot_work("crumbstyle");
    let b = buf(&a);
    let x_key = col(&a, "w Work").unwrap();
    let x_name = x_key + 2;
    let x_sep = col(&a, "›").unwrap();
    let x_arrow = col(&a, "▾").unwrap();
    let x_rows = col(&a, "3行").unwrap();
    let st = |x: u16| b[(x, hy(&a))].style();
    assert_eq!(st(x_name), st(x_arrow), "名前と ▾ は同じボタン");
    assert_eq!(st(x_name), st(x_sep), "区切りもボタンの中");
    assert!(
        st(x_key)
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD),
        "キーの字は太字"
    );
    assert_ne!(st(x_key), st(x_name), "キーの字は名前と違う見た目");
    assert_ne!(st(x_name), st(x_rows), "行の数はボタンの外");
    assert_ne!(st(x_name), b[(0, 23)].style(), "ボタンは地の字と違う見た目");
}

#[test]
fn test_sr_42_crumb_button_click_key_and_arrow() {
    // [SR-42][WS-8] キーの字を押す → 引き出しが開く。▾ を押す → 閉じる。行の数は押しても開かない。
    let (_t, mut a) = boot_work("crumbclick");
    let x_key = col(&a, "w Work").unwrap();
    let x_arrow = col(&a, "▾").unwrap();
    let x_rows = col(&a, "3行").unwrap();
    a.click(x_key, hy(&a));
    assert_eq!(a.mode, Mode::Drawer);
    a.click(x_arrow, hy(&a));
    assert_eq!(a.mode, Mode::Table);
    a.click(x_rows, hy(&a));
    assert_eq!(a.mode, Mode::Table, "行の数はボタンの外");
}

#[test]
fn test_sr_42_crumb_button_without_scope() {
    // [SR-42] 範囲が無い → 「w tasks ▾  3行」(「›」なし)。
    let tmp = workspace("crumbnone");
    let a = boot_tasks(&tmp);
    assert!(a.scope.is_none());
    let h = head(&a);
    assert!(!h.contains('›'), "{h:?}");
    assert!(
        h.trim_start().to_lowercase().starts_with("w tasks ▾"),
        "{h:?}"
    );
}

#[test]
fn test_sr_42_crumb_button_follows_rebound_key() {
    // [SR-42][SR-13] 引き出しのキーを W に割り当て直す → 「W notes ▾」。外す → キーなしで「notes ▾」。
    let (_t, a) = boot_config(
        "crumbrebind",
        "[keys.table]\nw = \"none\"\nW = \"workspace_drawer\"\n",
    );
    let h = head(&a);
    assert!(h.trim_start().starts_with("W notes ▾"), "{h:?}");
    let (_t, a) = boot_config("crumbunbound", "[keys.table]\nw = \"none\"\n");
    let h = head(&a);
    assert!(h.trim_start().starts_with("notes ▾"), "{h:?}");
}
