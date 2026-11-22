//! 最近開いた表(specs/requirements/cli.md の「最近開いた表」): パレットの「最近開いた表」と、引数なしの起動の一覧の
//! 「最近」の分類。期待値は要求の受け入れの例から取った。覚えた表は見た目の状態の置き場(`<一時>/state`)に、
//! 核の mdgrid::recent::record で前もって書く(画面で開いて覚えるのは tests/test_recent_e2e.rs)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

/// notes/ に tasks(tasks.base のビュー「進行中」も)・projects・books・members を作る。
fn fixture(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    for d in ["tasks", "projects", "books", "members"] {
        std::fs::create_dir_all(n.join(d)).unwrap();
        std::fs::write(n.join(d).join("a.md"), "---\nstatus: todo\n---\n").unwrap();
    }
    std::fs::write(
        n.join("tasks.base"),
        "views:\n  - type: table\n    name: 全部\n  - type: table\n    name: 進行中\n",
    )
    .unwrap();
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn path(tmp: &Tmp, rel: &str) -> PathBuf {
    tmp.notes().join(rel).canonicalize().unwrap()
}

/// 覚える(古いものから順に渡す。最後が新しい)。
fn remember(tmp: &Tmp, list: &[(&str, Option<&str>)]) {
    for (rel, view) in list {
        mdgrid::recent::record(
            &tmp.0.join("state"),
            &path(tmp, rel),
            *view,
            "2026-11-21 09:30",
        )
        .unwrap();
    }
}

fn boot(tmp: &Tmp, rel: &str) -> App {
    let target = path(tmp, rel);
    let src = Markdown::open(std::slice::from_ref(&target)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(110, 24);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

#[test]
fn test_recent_list_newest_first_and_open_with_view() {
    // [utest -> req~cli.recent-list~1] tasks(進行中)・projects の順に覚えている → 一覧の1行目が tasks(名前・置き場のフォルダ・日時)→ 選ぶ → tasks の進行中。
    let tmp = fixture("rc_list");
    remember(&tmp, &[("projects", None), ("tasks.base", Some("進行中"))]);
    let mut a = boot(&tmp, "members");
    palette(&mut a, "最近開いた表");
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    let s = screen(&a);
    let first = s
        .lines()
        .find(|l| l.contains("> "))
        .unwrap_or_else(|| panic!("選んだ行が無い:\n{s}"));
    assert!(first.contains("tasks · 進行中"), "{s}");
    assert!(first.contains("2026-11-21 09:30"), "{s}");
    assert!(s.contains("projects"), "{s}");
    press(&mut a, KeyCode::Enter);
    assert!(a.quit);
    let to = a.switch_to.clone().unwrap();
    assert_eq!(to.path, path(&tmp, "tasks.base"));
    assert_eq!(to.view.as_deref(), Some("進行中"));
}

#[test]
fn test_recent_list_filters_by_typing() {
    // [utest -> req~cli.recent-list~1] 打った字で一覧を絞れる(cli.places-list-filter と同じ)。
    let tmp = fixture("rc_filter");
    remember(&tmp, &[("projects", None), ("tasks.base", Some("進行中"))]);
    let mut a = boot(&tmp, "members");
    palette(&mut a, "最近開いた表");
    typing(&mut a, "proj");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(path(&tmp, "projects"))
    );
}

#[test]
fn test_recent_list_asks_before_leaving_changes() {
    // [utest -> req~cli.recent-list~1] ためた変更があれば cli.places-list-pending と同じく確かめる。
    let tmp = fixture("rc_pending");
    remember(&tmp, &[("projects", None)]);
    let mut a = boot(&tmp, "members");
    let row = a.src.rows()[0].clone();
    a.changes
        .set(a.src.as_ref(), &row, "status", NewValue::Str("x".into()))
        .unwrap();
    palette(&mut a, "最近開いた表");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    assert!(!a.quit);
}

#[test]
fn test_recent_list_leaves_out_the_open_table_and_missing() {
    // [utest -> req~cli.recent-missing~1] 覚えた projects のフォルダを消してから「最近開いた表」→ projects が無い(今の表も並べない)。
    let tmp = fixture("rc_missing");
    remember(
        &tmp,
        &[("projects", None), ("books", None), ("members", None)],
    );
    std::fs::remove_dir_all(tmp.notes().join("projects")).unwrap();
    let mut a = boot(&tmp, "members");
    palette(&mut a, "最近開いた表");
    let s = screen(&a);
    assert!(s.contains("books"), "{s}");
    assert!(!s.contains("projects"), "{s}");
    assert!(!s.contains("> members"), "{s}");
}

#[test]
fn test_recent_list_empty_message() {
    // [utest -> req~cli.recent-list~1] 覚えた表が無ければ一覧を出さずに知らせる。
    let tmp = fixture("rc_empty");
    let mut a = boot(&tmp, "members");
    palette(&mut a, "最近開いた表");
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.as_deref().unwrap_or("").contains("最近開いた表"));
}

#[test]
fn test_recent_in_launch_list_after_here() {
    // [utest -> req~cli.recent-in-launch-list~2] places.toml に 仕事(タスク)、最近が projects・books → 今のフォルダ・最近(projects・books)・仕事 › タスク。
    let tmp = fixture("rc_launch");
    places::save(
        &tmp.0.join("config"),
        Place {
            name: "タスク".into(),
            group: "仕事".into(),
            path: path(&tmp, "tasks"),
            view: None,
        },
    )
    .unwrap();
    remember(&tmp, &[("books", None), ("projects", None)]);
    let mut a = boot(&tmp, "members");
    a.start_open_places(true);
    let s = screen(&a);
    let pos = |t: &str| s.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{s}"));
    assert!(pos("今のフォルダ") < pos("最近 › projects"), "{s}");
    assert!(pos("最近 › projects") < pos("最近 › books"), "{s}");
    assert!(pos("最近 › books") < pos("仕事 › タスク"), "{s}");
    // 最近の表を選ぶと、その表へ移る。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.switch_to.as_ref().map(|p| p.path.clone()),
        Some(path(&tmp, "projects"))
    );
}

#[test]
fn test_recent_in_launch_list_at_most_five_and_none() {
    // [utest -> req~cli.recent-in-launch-list~2] 最近は 5 件まで。最近が無い → 「最近」の分類を出さない。
    let tmp = fixture("rc_launch5");
    let cfg = tmp.0.join("config");
    places::save(
        &cfg,
        Place {
            name: "タスク".into(),
            group: "仕事".into(),
            path: path(&tmp, "tasks"),
            view: None,
        },
    )
    .unwrap();
    let mut a = boot(&tmp, "members");
    a.start_open_places(true);
    assert!(!screen(&a).contains("最近"), "{}", screen(&a));
    for k in 0..7 {
        let d = tmp.notes().join(format!("t{k}"));
        std::fs::create_dir_all(&d).unwrap();
        mdgrid::recent::record(&tmp.0.join("state"), &d, None, "2026-11-21 09:30").unwrap();
    }
    let mut b = boot(&tmp, "members");
    b.start_open_places(true);
    let s = screen(&b);
    assert_eq!(s.matches("最近 › ").count(), 5, "{s}");
    assert!(s.contains("最近 › t6") && !s.contains("最近 › t1"), "{s}");
}
