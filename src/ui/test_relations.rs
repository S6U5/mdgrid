//! リレーションの画面(REL-2・REL-3・REL-10。行き先を開く・つながった行は test_relations_open.rs)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

/// notes/ の下に tasks・projects・members。tasks の行が表の行。
pub(super) fn workspace(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let n = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = n.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("projects/mdgrid.md", "---\nstatus: active\n---\n");
    w("projects/Other Project.md", "---\nstatus: idea\n---\n");
    w("members/Alice.md", "---\nrole: dev\n---\n");
    w(
        "tasks/a build.md",
        "---\nproject: \"[[mdgrid]]\"\nassignee: ../members/Alice\nrelated:\n  - \"[[mdgrid]]\"\nstatus: done\n---\n",
    );
    w(
        "tasks/b docs.md",
        "---\nproject: \"[[nothing]]\"\nassignee: ../members/Alice\nrelated:\n  - \"[[mdgrid]]\"\n  - \"[[ghost]]\"\nstatus: todo\n---\n",
    );
    w(
        "tasks/c test.md",
        "---\nproject: \"[mdgrid](../projects/mdgrid.md)\"\nstatus: todo\n---\n",
    );
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

/// tasks のフォルダを開く(projects・members は登録した表)。
pub(super) fn boot_tasks(tmp: &Tmp) -> App {
    let cfg = tmp.0.join("config");
    for (name, dir) in [
        ("Projects", "projects"),
        ("Members", "members"),
        ("Tasks", "tasks"),
    ] {
        places::save(
            &cfg,
            Place {
                name: name.into(),
                group: String::new(),
                path: tmp.notes().join(dir).canonicalize().unwrap(),
                view: None,
            },
        )
        .unwrap();
    }
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

pub(super) fn row_of(a: &App, name: &str) -> RowId {
    a.src
        .rows()
        .into_iter()
        .find(|r| r.0.ends_with(&format!("{name}.md")))
        .unwrap_or_else(|| panic!("{name} の行が無い"))
}

/// その行・列に移る。
pub(super) fn goto(a: &mut App, name: &str, col: &str) {
    let r = row_of(a, name);
    assert!(a.select_note(std::path::Path::new(&r.0)), "{name} の位置");
    a.col = a
        .cols
        .iter()
        .position(|c| c == col)
        .unwrap_or_else(|| panic!("{col} の列"));
}

fn line_of<'a>(s: &'a str, name: &str) -> &'a str {
    s.lines()
        .find(|l| l.contains(name))
        .unwrap_or_else(|| panic!("{name} の行が画面に無い:\n{s}"))
}

#[test]
fn test_rel_2_names_in_cells() {
    // [REL-2] リンクは行き先のノートの名前で見せる(3つの書き方のどれでも。リストは並べる)。普通の値はそのまま。
    let tmp = workspace("rel_names");
    let a = boot_tasks(&tmp);
    let s = screen(&a);
    let build = line_of(&s, "a build");
    assert!(
        build.contains("mdgrid") && !build.contains("[[mdgrid]]"),
        "{build}"
    );
    assert!(
        build.contains("Alice") && !build.contains("../members"),
        "{build}"
    );
    assert!(
        build.contains("[mdgrid]") && build.contains("done"),
        "{build}"
    );
    let test = line_of(&s, "c test");
    assert!(
        test.contains("mdgrid") && !test.contains("../projects"),
        "{test}"
    );
}

#[test]
fn test_rel_10_missing_target_marked() {
    // [REL-10] 行き先の無い [[…]] は、書いた名前に印 `?` を付けて見せる。リストの要素も。
    let tmp = workspace("rel_missing");
    let a = boot_tasks(&tmp);
    let s = screen(&a);
    let docs = line_of(&s, "b docs");
    assert!(docs.contains("?nothing"), "{docs}");
    assert!(docs.contains("[mdgrid, ?ghost]"), "{docs}");
}

#[test]
fn test_rel_3_pick_from_target_folder() {
    // [REL-3] リンクの列の候補は行き先のフォルダのノートの名前。選ぶと列の形(ここでは [[…]])で書く。
    let tmp = workspace("rel_pick");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "project");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let s = screen(&a);
    assert!(s.contains("Other Project") && s.contains("mdgrid"), "{s}");
    assert!(!s.contains("[[Other Project]]"), "候補は名前で見せる:\n{s}");
    typing(&mut a, "other");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    let row = row_of(&a, "a build");
    assert_eq!(
        a.changes.pending(&row, "project"),
        Some(&NewValue::Str("[[Other Project]]".into()))
    );
    assert!(
        line_of(&screen(&a), "a build").contains("*Other Project"),
        "{}",
        screen(&a)
    );
}

#[test]
fn test_rel_3_list_column_many_to_many() {
    // [REL-3] リンクのリストの列(多対多)は、行き先のフォルダのノートを名前で並べ、複数を付けられる。
    let tmp = workspace("rel_many");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "related");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{}", screen(&a));
    let s = screen(&a);
    assert!(
        s.contains("[x] mdgrid") && s.contains("[ ] Other Project"),
        "{s}"
    );
    typing(&mut a, "Other Project");
    press(&mut a, KeyCode::Enter); // 検索中の Enter は付けるだけ
    press(&mut a, KeyCode::Enter); // 確定
    let row = row_of(&a, "a build");
    assert_eq!(
        a.changes.pending(&row, "related"),
        Some(&NewValue::List(vec![
            "[[mdgrid]]".into(),
            "[[Other Project]]".into()
        ]))
    );
}

#[test]
fn test_rel_1_plain_text_column_unchanged() {
    // [REL-1] リンクでない列(status)は今までどおりの候補と見せ方。
    let tmp = workspace("rel_plain");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "status");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("todo") && s.contains("done"), "{s}");
    assert!(!s.contains("Other Project"), "{s}");
    let _ = PathBuf::new();
}
