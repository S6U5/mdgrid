//! AI の案を画面で受ける(ai.proposal-notice・ai.proposal-take・ai.proposal-discard)。台帳の根 `notes/`(印あり)の
//! `.mdgrid/proposals/` に案を置き、表「tasks」(notes/tasks)を開く。

use super::startup::Startup;
use super::test_screen::{ch, press, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const A: &str = "---\nstatus: todo\ndue: 2026-12-01\n---\n";
const B: &str = "---\nstatus: todo\n---\n";

/// 台帳を作る: 印、notes/tasks/a.md・b.md、notes/other/x.md。
fn ledger(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    let root = tmp.notes();
    for (rel, text) in [
        (".mdgrid/workspace.toml", ""),
        ("tasks/a.md", A),
        ("tasks/b.md", B),
        ("other/x.md", B),
    ] {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    tmp
}

fn proposal_dir(tmp: &Tmp) -> PathBuf {
    tmp.notes().join(".mdgrid/proposals")
}

/// 案を1つ置く(`name` は拡張子なしのファイルの名前)。
fn put(tmp: &Tmp, name: &str, json: &str) -> PathBuf {
    let dir = proposal_dir(tmp);
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{name}.json"));
    std::fs::write(&p, json).unwrap();
    p
}

/// 表「tasks」を開く。
fn open(tmp: &Tmp) -> App {
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

fn footer(a: &App) -> String {
    super::bands::footer(a, 200)
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect()
}

fn palette(a: &mut App, name: &str) {
    ch(a, ':');
    for c in name.chars() {
        ch(a, c);
    }
    press(a, KeyCode::Enter);
}

fn row_of(a: &App, name: &str) -> mdgrid::source::RowId {
    a.rows
        .iter()
        .find(|r| a.src.label(r).ends_with(name))
        .cloned()
        .unwrap_or_else(|| panic!("{name} の行が無い"))
}

const TWO_ROWS: &str = r#"{"note": "締め切りを見直す", "created": "2026-11-28T10:00", "changes": [
  {"path": "tasks/a.md", "set": {"due": "2026-12-20"}, "was": {"due": "2026-12-01"}},
  {"path": "tasks/b.md", "set": {"status": "done"}, "was": {"status": "todo"}}]}"#;
const OTHER: &str = r#"{"note": "ほかの表", "created": "2026-11-28T11:00", "changes": [
  {"path": "other/x.md", "set": {"status": "done"}, "was": {"status": "todo"}}]}"#;
const DUE: &str = r#"{"note": "締め切り", "created": "2026-11-28T10:00", "changes": [
  {"path": "tasks/a.md", "set": {"due": "2026-12-20"}, "was": {"due": "2026-12-01"}}]}"#;

#[test]
fn test_proposal_notice_counts_only_this_table() {
    // [utest -> req~ai.proposal-notice~1] 開いた表の2つのノートを指す案と、ほかの表のノートだけを指す案 → 下の帯に「AI の案 1件」。
    // 「AI の案を見る」で、その案の理由・作った日時・セルの数が1行に並ぶ。
    let tmp = ledger("pr_notice");
    put(&tmp, "p1", TWO_ROWS);
    put(&tmp, "p2", OTHER);
    let mut a = open(&tmp);
    assert!(footer(&a).contains("AI の案 1件"), "{}", footer(&a));
    palette(&mut a, "ai_proposals");
    let s = super::test_screen::screen(&a);
    assert!(
        s.lines().any(|l| l.contains("締め切りを見直す")
            && l.contains("11-28 10:00")
            && l.contains('2')),
        "{s}"
    );
    assert!(!s.contains("ほかの表"), "{s}");
}

#[test]
fn test_proposal_notice_follows_sync() {
    // [utest -> req~ai.proposal-notice~1] 開いているあいだに案のファイルが届く → 次の見回りで下の帯に案の数。
    let tmp = ledger("pr_poll");
    let mut a = open(&tmp);
    assert!(!footer(&a).contains("AI の案"), "{}", footer(&a));
    put(&tmp, "p1", DUE);
    a.poll();
    assert!(footer(&a).contains("AI の案 1件"), "{}", footer(&a));
}

#[test]
fn test_proposal_take_makes_pending_and_removes_file() {
    // [utest -> req~ai.proposal-take~1] due を 2026-12-20 にする案を選んで「入れる」→ due に 2026-12-20 のためる変更(未保存 1)。
    // ノートのファイルは変わらず、案のファイルは消える。
    let tmp = ledger("pr_take");
    let file = put(&tmp, "p1", DUE);
    let mut a = open(&tmp);
    palette(&mut a, "ai_proposals");
    press(&mut a, KeyCode::Enter); // 案を選ぶ
    press(&mut a, KeyCode::Enter); // 入れる
    assert_eq!(a.changes.count(), 1, "{:?}", a.message);
    let r = row_of(&a, "a.md");
    assert!(
        matches!(a.changes.pending(&r, "due"), Some(NewValue::Date(v) | NewValue::Str(v)) if v == "2026-12-20"),
        "{:?}",
        a.changes.pending(&r, "due")
    );
    assert_eq!(
        std::fs::read_to_string(tmp.notes().join("tasks/a.md")).unwrap(),
        A
    );
    assert!(!file.exists());
}

#[test]
fn test_proposal_take_skips_cells_changed_since() {
    // [utest -> req~ai.proposal-take~1] 案を作ったあとに外で due を 2026-12-05 に直した → 「入れる」で due はためず、知らせに
    // 入れなかったセルの数 1。案のファイルは消える。
    let tmp = ledger("pr_stale");
    let file = put(&tmp, "p1", DUE);
    std::fs::write(
        tmp.notes().join("tasks/a.md"),
        A.replace("2026-12-01", "2026-12-05"),
    )
    .unwrap();
    let mut a = open(&tmp);
    palette(&mut a, "ai_proposals");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains('1'), "{m}");
    assert!(!file.exists());
}

#[test]
fn test_proposal_discard_removes_file_only() {
    // [utest -> req~ai.proposal-discard~1] 案を選んで「捨てる」→ 案のファイルが消え、未保存は 0 のまま、下の帯の案の数が消える。
    let tmp = ledger("pr_discard");
    let file = put(&tmp, "p1", DUE);
    let mut a = open(&tmp);
    palette(&mut a, "ai_proposals");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert!(!file.exists());
    assert_eq!(a.changes.count(), 0);
    assert!(!footer(&a).contains("AI の案"), "{}", footer(&a));
    assert_eq!(
        std::fs::read_to_string(tmp.notes().join("tasks/a.md")).unwrap(),
        A
    );
}
