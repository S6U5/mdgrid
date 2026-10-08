//! リレーションをたどる(REL-4: 行き先を開く、REL-5: つながった行)。

use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_relations::{boot_tasks, goto, row_of, workspace};
use super::test_screen::{ch, press, screen, typing};
use super::*;
use mdgrid::config::Config;
use mdgrid::places;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

fn real(p: std::path::PathBuf) -> std::path::PathBuf {
    p.canonicalize().unwrap()
}

#[test]
fn test_rel_4_open_target_in_other_table() {
    // [REL-4] リンクのセルから「行き先を開く」→ 行き先を含む登録した表に移り、その行を選ぶ(main が開き直す)。
    let tmp = workspace("rel_open");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "project");
    palette(&mut a, "open_link");
    assert!(a.quit, "変更が無ければすぐ終わって移る: {:?}", a.message);
    let place = a.switch_to.clone().expect("移る表");
    assert_eq!(place.name, "Projects");
    assert_eq!(
        a.switch_select.clone(),
        Some(real(tmp.notes().join("projects/mdgrid.md")))
    );
}

#[test]
fn test_rel_4_open_target_in_same_table() {
    // [REL-4] 行き先が今の表の中なら、開き直さずにその行へ移る。
    let tmp = workspace("rel_open_same");
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: true,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    goto(&mut a, "a build", "assignee");
    palette(&mut a, "open_link");
    assert!(!a.quit && a.switch_to.is_none(), "{:?}", a.message);
    assert_eq!(
        a.cur_row(),
        Some(row_of(&a, "Alice")),
        "読むだけでも、同じ表の行へ移る"
    );
}

#[test]
fn test_rel_4_no_target_reason() {
    // [REL-4] 行き先の無いリンクは理由を出して移らない。
    let tmp = workspace("rel_open_none");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "b docs", "project");
    palette(&mut a, "open_link");
    assert!(!a.quit && a.switch_to.is_none());
    assert!(
        a.message.as_deref().unwrap_or("").contains("リンクが無い"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_rel_4_choose_among_targets() {
    // [REL-4] セルの行き先が2つ以上なら、選ぶ一覧を出す。
    let tmp = workspace("rel_open_many");
    std::fs::write(
        tmp.notes().join("tasks/d both.md"),
        "---\nrelated:\n  - \"[[mdgrid]]\"\n  - \"[[Other Project]]\"\n---\n",
    )
    .unwrap();
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "d both", "related");
    palette(&mut a, "open_link");
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    let s = screen(&a);
    assert!(s.contains("mdgrid") && s.contains("Other Project"), "{s}");
    typing(&mut a, "other");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        a.switch_select.clone(),
        Some(real(tmp.notes().join("projects/Other Project.md")))
    );
}

#[test]
fn test_rel_4_pending_changes_ask_first() {
    // [REL-4][WB-11] 保存していない変更があれば、移る前に確かめる。戻ると移らない。
    let tmp = workspace("rel_open_pending");
    let mut a = boot_tasks(&tmp);
    let row = row_of(&a, "c test");
    a.changes
        .set(
            a.src.as_ref(),
            &row,
            "status",
            NewValue::Str("doing".into()),
        )
        .unwrap();
    goto(&mut a, "a build", "project");
    palette(&mut a, "open_link");
    assert_eq!(a.mode, Mode::Quit);
    press(&mut a, KeyCode::Esc);
    assert!(a.switch_to.is_none() && a.switch_select.is_none());
}

#[test]
fn test_rel_4_menu_has_open_link_on_link_cell() {
    // [REL-4][SR-24] 操作の一覧: リンクのセルにだけ「行き先を開く」。
    let tmp = workspace("rel_menu");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "project");
    assert!(menu::item_actions(&a).contains(&Action::OpenLink));
    goto(&mut a, "a build", "status");
    assert!(!menu::item_actions(&a).contains(&Action::OpenLink));
}

#[test]
fn test_rel_5_linked_rows_lists_table_and_column() {
    // [REL-5] つながった行: どの表のどの列から指しているかを並べ、選ぶとそのノートの表へ移る。
    let tmp = workspace("rel_back");
    let cfg = tmp.0.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    places::save(
        &cfg,
        places::Place {
            name: "Tasks".into(),
            group: String::new(),
            path: real(tmp.notes().join("tasks")),
            view: None,
        },
    )
    .unwrap();
    let dir = tmp.notes().join("projects");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    goto(&mut a, "mdgrid", "status");
    palette(&mut a, "linked_rows");
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    let s = screen(&a);
    for want in [
        "Tasks › project ← a build",
        "Tasks › related ← a build",
        "Tasks › related ← b docs",
        "Tasks › project ← c test",
    ] {
        assert!(s.contains(want), "{want}:\n{s}");
    }
    typing(&mut a, "c test");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.switch_to.as_ref().map(|p| p.name.as_str()), Some("Tasks"));
    assert_eq!(
        a.switch_select.clone(),
        Some(real(tmp.notes().join("tasks/c test.md")))
    );
}

#[test]
fn test_rel_5_no_linked_rows_message() {
    // [REL-5] 指しているノートが無ければ、理由を出す。
    let tmp = workspace("rel_back_none");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "c test", "status");
    palette(&mut a, "linked_rows");
    assert_eq!(a.mode, Mode::Table);
    assert!(a
        .message
        .as_deref()
        .unwrap_or("")
        .contains("指すノートは無い"));
}

#[test]
fn test_rel_3_same_name_in_other_table_written_as_path() {
    // [REL-3] 別の表に同じ名前のノートがあれば、取り違えないようにパスで書く(開くと選んだノートへ)。
    let tmp = workspace("rel_same_name");
    // 登録した別の表(members)に同じ名前のノート。
    std::fs::write(tmp.notes().join("members/Other Project.md"), "---\n---\n").unwrap();
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "project");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "other");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    let row = row_of(&a, "a build");
    let Some(NewValue::Str(v)) = a.changes.pending(&row, "project").cloned() else {
        panic!("{:?}", a.changes.pending(&row, "project"));
    };
    assert_eq!(v, "[[../projects/Other Project]]");
    assert_eq!(
        a.link_target(&row, &v).and_then(|(_, t)| t),
        Some(real(tmp.notes().join("projects/Other Project.md")))
    );
}

#[test]
fn test_rel_4_hidden_row_in_same_table_gives_reason() {
    // [REL-4] 行き先が今の表にあるのに絞り込みで隠れていれば、開き直さずに理由を出す。
    let tmp = workspace("rel_hidden");
    let dir = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    ch(&mut a, '\\');
    typing(&mut a, "build");
    press(&mut a, KeyCode::Enter);
    goto(&mut a, "a build", "assignee");
    palette(&mut a, "open_link");
    assert!(!a.quit && a.switch_to.is_none());
    assert!(
        a.message.as_deref().unwrap_or("").contains("隠れている"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_rel_10_resolves_again_after_save() {
    // [REL-10][REL-1] 保存のあとは解き直す(行き先を作ったら印が消える)。
    let tmp = workspace("rel_resolve_again");
    let a = boot_tasks(&tmp);
    let row = row_of(&a, "b docs");
    assert!(a
        .link_target(&row, "[[nothing]]")
        .is_some_and(|(_, t)| t.is_none()));
    std::fs::write(tmp.notes().join("projects/nothing.md"), "---\n---\n").unwrap();
    a.links.invalidate();
    assert_eq!(
        a.link_target(&row, "[[nothing]]").and_then(|(_, t)| t),
        Some(real(tmp.notes().join("projects/nothing.md")))
    );
}

#[test]
fn test_rel_5_menu_offers_linked_rows_with_registered_tables() {
    // [REL-5][SR-24] 登録した表があれば、操作の一覧に「つながった行」を出す。
    let tmp = workspace("rel_menu_back");
    let a = boot_tasks(&tmp);
    assert!(menu::item_actions(&a).contains(&Action::LinkedRows));
}
