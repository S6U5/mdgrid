//! お気に入りのワークスペースと切り替え(specs/requirements/workspace.md の「お気に入りのワークスペースと切り替え」)。
//! 期待値は要求の受け入れの例から取った。見本: notes/ に tasks・projects(projects.base でも開く)・books・scratch。
//! workspaces.toml に work(tasks・projects.base)と 趣味(books)。印のワークスペース lab(x、木の設定つき)。
//! 一覧を開く本物の実行ファイルの画面は tests/test_switch_e2e.rs で見る。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::favorites;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use mdgrid::workspace::{self, WsTable};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const BASE: &str = "views:\n  - type: table\n    name: 全部\n  - type: table\n    name: 進行中\n";

fn real(p: PathBuf) -> PathBuf {
    p.canonicalize().unwrap()
}

/// 見本を作る。`favs` はお気に入りに入れるもの(名前か印の根)。
fn fixture(name: &str, favs: &[&str]) -> Tmp {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        "tasks/a.md",
        "---\nstatus: todo\nstart: 2026-11-01\ndue: 2026-11-10\n---\n",
    );
    w(
        "tasks/b.md",
        "---\nstatus: done\nstart: 2026-11-02\ndue: 2026-11-12\n---\n",
    );
    w("projects/p.md", "---\nstatus: active\n---\n");
    w("projects.base", BASE);
    w("books/dune.md", "---\nauthor: Herbert\n---\n");
    w("scratch/s.md", "---\nnote: x\n---\n");
    w("lab/x/one.md", "---\nfolder: a > b\n---\n");
    w(
        "lab/.mdgrid/workspace.toml",
        "[[table]]\nname = \"x\"\npath = \"x\"\ntree = { path = \"folder\" }\n",
    );
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    let table = |name: &str, rel: &str| WsTable {
        name: name.into(),
        path: real(n.join(rel)),
        ..WsTable::default()
    };
    workspace::add(&cfg, "work", table("tasks", "tasks")).unwrap();
    workspace::add(&cfg, "work", table("projects", "projects.base")).unwrap();
    workspace::add(&cfg, "趣味", table("books", "books")).unwrap();
    for f in favs {
        favorites::set(&cfg, f, true).unwrap();
    }
    tmp
}

/// `rel`(notes/ からのフォルダか .base)を開いた画面。`ws` は起動の `-w`。
fn boot(tmp: &Tmp, rel: &str, ws: Option<&str>) -> App {
    let target = real(tmp.notes().join(rel));
    let (src, base) = if target.extension().is_some_and(|x| x == "base") {
        let b = mdgrid::base::parse(&std::fs::read_to_string(&target).unwrap()).unwrap();
        let src = Markdown::open_vault(&target).unwrap();
        (src, Some((b, rel.to_string(), None)))
    } else {
        (Markdown::open(std::slice::from_ref(&target)).unwrap(), None)
    };
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.workspace_choice = ws.map(str::to_string);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

/// 今の画面を離れて、次の画面に見た表を渡す(main の run_switching と同じ)。
fn move_on(mut from: App, next: &mut App) {
    from.leave_visit();
    next.visits = std::mem::take(&mut from.visits);
}

#[test]
fn test_switch_lists_favorites_in_order_with_numbers() {
    // [workspace.switch-open] お気に入りが work・趣味 → 切り替えのキーで「1 work」「2 趣味」の一覧(今の work に印)。
    let tmp = fixture("sw_list", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let s = screen(&a);
    let pos = |t: &str| s.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{s}"));
    assert!(pos("1 work") < pos("2 趣味"), "{s}");
    assert!(s.contains("● 1 work"), "今いるワークスペースに印:\n{s}");
}

#[test]
fn test_switch_opens_from_every_screen_and_esc_keeps_it() {
    // [utest -> req~workspace.switch-keys~1] 表・カンバン・ガント・グラフ・関係マップ・木のどれからでも開き、Esc で元の画面のまま。
    let tmp = fixture("sw_every", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    for (open, mode) in [
        (None, Mode::Table),
        (Some('B'), Mode::Kanban),
        (Some('C'), Mode::Gantt),
        (Some('P'), Mode::Chart),
        (Some('R'), Mode::Relations),
    ] {
        a.close_screen();
        if let Some(k) = open {
            ch(&mut a, k);
        }
        assert_eq!(a.mode, mode, "{:?}", a.message);
        ch(&mut a, 'W');
        assert_eq!(a.mode, Mode::Switch, "{mode:?} から開く");
        assert!(screen(&a).contains("2 趣味"), "{mode:?}:\n{}", screen(&a));
        press(&mut a, KeyCode::Esc);
        assert_eq!(a.mode, mode, "Esc で {mode:?} のまま");
        assert!(a.switch_to.is_none() && !a.quit);
    }
    let mut t = boot(&tmp, "lab/x", None);
    ch(&mut t, 'T');
    assert_eq!(t.mode, Mode::Tree, "{:?}", t.message);
    ch(&mut t, 'W');
    assert_eq!(t.mode, Mode::Switch);
    press(&mut t, KeyCode::Esc);
    assert_eq!(t.mode, Mode::Tree);
}

#[test]
fn test_switch_from_palette_keeps_the_screen() {
    // [workspace.switch-open] パレットの「ワークスペースを切り替える」でも、カンバンの上に開く。
    let tmp = fixture("sw_palette", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'B');
    ch(&mut a, ':');
    typing(&mut a, "ワークスペースを切り替える");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Switch);
    assert!(a.kanban.is_some(), "カンバンは閉じない");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Kanban);
}

#[test]
fn test_switch_without_favorites_shows_how_to_add() {
    // [workspace.switch-open] お気に入りが無い → 一覧に、お気に入りの入れ方の案内の1行。
    let tmp = fixture("sw_none", &[]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert_eq!(a.mode, Mode::Switch);
    let s = screen(&a);
    assert!(s.contains("お気に入りが無い"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert!(a.switch_to.is_none() && !a.quit, "案内の行では移らない");
}

#[test]
fn test_switch_digit_moves_to_that_workspace() {
    // [utest -> req~workspace.switch-keys~1] 一覧で `2` → 趣味へ移る(範囲は趣味)。
    let tmp = fixture("sw_digit", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    ch(&mut a, '2');
    assert!(a.quit, "ためた変更が無ければすぐ移る");
    let to = a.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("books")));
    assert_eq!(a.switch_workspace.as_deref(), Some("趣味"));
    // 番号の無いキーは何もしない。
    let mut b = boot(&tmp, "tasks", None);
    ch(&mut b, 'W');
    ch(&mut b, '7');
    assert_eq!(b.mode, Mode::Switch);
    assert!(b.switch_to.is_none());
}

#[test]
fn test_switch_target_last_seen_table_and_view() {
    // [utest -> req~workspace.switch-target~1] work の projects(進行中)を見てから趣味へ移り、また work を選ぶ → projects の進行中。
    let tmp = fixture("sw_target", &["work", "趣味"]);
    let mut p = boot(&tmp, "projects.base", None);
    p.select_view_named("進行中");
    assert_eq!(p.current_view_name().as_deref(), Some("進行中"));
    let mut b = boot(&tmp, "books", Some("趣味"));
    move_on(p, &mut b);
    ch(&mut b, 'W');
    ch(&mut b, '1');
    let to = b.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("projects.base")));
    assert_eq!(to.view.as_deref(), Some("進行中"));
    assert_eq!(b.switch_workspace.as_deref(), Some("work"));
}

#[test]
fn test_switch_target_first_table_when_not_seen() {
    // [utest -> req~workspace.switch-target~1] この起動でまだ開いていない趣味を選ぶ → 趣味の最初の表。印のワークスペースは印の表。
    let tmp = fixture("sw_first", &["work", "趣味"]);
    let lab = real(tmp.notes().join("lab"));
    favorites::set(&tmp.0.join("config"), &favorites::marker_entry(&lab), true).unwrap();
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'W');
    assert!(screen(&a).contains("3 lab"), "{}", screen(&a));
    ch(&mut a, '2');
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(real(tmp.notes().join("books")))
    );
    let mut b = boot(&tmp, "tasks", None);
    ch(&mut b, 'W');
    ch(&mut b, '3');
    assert_eq!(
        b.switch_to.as_ref().map(|p| p.path.clone()),
        Some(lab.join("x"))
    );
    assert!(
        b.switch_leave_workspace,
        "印のワークスペースは -w を外して開く"
    );
}

#[test]
fn test_switch_origin_row_and_back() {
    // [workspace.switch-origin] ワークスペースでない scratch で起動し、work へ移る → 先頭に「最初に開いた所: scratch」→ 選ぶと scratch。
    // scratch にいるあいだは出さない。
    let tmp = fixture("sw_origin", &["work", "趣味"]);
    let mut s = boot(&tmp, "scratch", None);
    s.mark_origin(None);
    ch(&mut s, 'W');
    assert!(!screen(&s).contains("最初に開いた所"), "{}", screen(&s));
    press(&mut s, KeyCode::Esc);
    let mut w = boot(&tmp, "tasks", Some("work"));
    move_on(s, &mut w);
    ch(&mut w, 'W');
    let sc = screen(&w);
    let pos = |t: &str| sc.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{sc}"));
    assert!(pos("最初に開いた所: scratch") < pos("1 work"), "{sc}");
    press(&mut w, KeyCode::Enter);
    let to = w.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("scratch")));
    assert!(w.switch_leave_workspace, "-w を外して戻る");
}

#[test]
fn test_switch_origin_returns_to_the_last_view_there() {
    // [workspace.switch-target] 「最初に開いた所」を選ぶ → 起動したときの表で、そこで最後に見ていたビュー。
    let tmp = fixture("sw_origin_view", &["趣味"]);
    let mut p = boot(&tmp, "projects.base", None);
    p.mark_origin(None);
    p.select_view_named("進行中");
    let mut b = boot(&tmp, "books", Some("趣味"));
    move_on(p, &mut b);
    ch(&mut b, 'W');
    press(&mut b, KeyCode::Enter);
    let to = b.switch_to.clone().unwrap();
    assert_eq!(to.path, real(tmp.notes().join("projects.base")));
    assert_eq!(to.view.as_deref(), Some("進行中"));
}

#[test]
fn test_switch_asks_before_leaving_unsaved_changes() {
    // [utest -> req~workspace.switch-unsaved~1] 未保存の変更があるときに一覧で趣味を選ぶ → 保存の確かめ。取りやめ → 元の画面(カンバン)のまま。
    let tmp = fixture("sw_unsaved", &["work", "趣味"]);
    let mut a = boot(&tmp, "tasks", None);
    let row = a.src.rows()[0].clone();
    a.changes
        .set(
            a.src.as_ref(),
            &row,
            "status",
            NewValue::Str("doing".into()),
        )
        .unwrap();
    ch(&mut a, 'B');
    assert_eq!(a.mode, Mode::Kanban);
    ch(&mut a, 'W');
    ch(&mut a, '2');
    assert_eq!(a.mode, Mode::Quit, "保存の確かめ");
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Kanban, "取りやめたらカンバンのまま");
    assert!(a.switch_to.is_none());
    assert_eq!(a.changes.count(), 1, "変更は捨てない");
}

#[test]
fn test_favorite_mark_in_drawer_heading() {
    // [utest -> req~workspace.favorite-mark~1] 引き出しの「趣味」の見出しでお気に入りのキー → ★。もう一度 → ★ が消える。
    let tmp = fixture("sw_mark", &[]);
    let mut a = boot(&tmp, "tasks", None);
    ch(&mut a, 'w');
    assert_eq!(a.mode, Mode::Drawer);
    // 今の範囲 work の表(tasks・projects)の下に 趣味 の見出し。
    ch(&mut a, 'G');
    ch(&mut a, 'f');
    assert!(screen(&a).contains("趣味 ★"), "{}", screen(&a));
    assert_eq!(favorites::entries(&tmp.0.join("config")), vec!["趣味"]);
    ch(&mut a, 'f');
    assert!(!screen(&a).contains("★"), "{}", screen(&a));
    assert!(favorites::entries(&tmp.0.join("config")).is_empty());
    // 表の行では入れず、入れ方を言う。
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    ch(&mut a, 'f');
    assert!(a.message.as_deref().unwrap_or("").contains("見出し"));
}

#[test]
fn test_favorite_current_marker_workspace_from_palette() {
    // [utest -> req~workspace.favorite-mark~1] 印のワークスペース lab を開いてパレットの「今のワークスペースをお気に入りにする」→ 切り替えの一覧に lab。
    let tmp = fixture("sw_marker", &[]);
    let mut a = boot(&tmp, "lab/x", None);
    ch(&mut a, ':');
    typing(&mut a, "今のワークスペースをお気に入りにする");
    press(&mut a, KeyCode::Enter);
    assert!(
        a.message.as_deref().unwrap_or("").contains("lab"),
        "{:?}",
        a.message
    );
    let lab = real(tmp.notes().join("lab"));
    assert_eq!(
        favorites::entries(&tmp.0.join("config")),
        vec![favorites::marker_entry(&lab)]
    );
    ch(&mut a, 'W');
    assert!(screen(&a).contains("1 lab"), "{}", screen(&a));
}

#[test]
fn test_favorite_current_needs_a_workspace() {
    // [utest -> req~workspace.favorite-mark~1] ワークスペースでない表では入れずに理由。
    let tmp = fixture("sw_nows", &[]);
    let mut a = boot(&tmp, "scratch", None);
    a.apply(super::keymap::Action::WsFavorite);
    assert!(a
        .message
        .as_deref()
        .unwrap_or("")
        .contains("ワークスペース"));
    assert!(favorites::entries(&tmp.0.join("config")).is_empty());
}
