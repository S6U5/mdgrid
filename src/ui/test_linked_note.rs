//! 選んだ行につながるノートを作る(cell-edit.new-linked-note): `c` で、ほかの表のその表を指す列のノートを作る。
//! 台帳は test_relations の見本(tasks の project・related が projects を、assignee が members を指す)。

use super::native_io::Ask;
use super::startup::Startup;
use super::test_relations::workspace;
use super::test_screen::{ch, ctrl, press, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::i18n::Msg;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::RowId;
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};

/// Projects・Members・Tasks を登録して、`dir`(notes/ の下のフォルダ)を開く。`before` は読み込みの前に App に当てる。
fn boot_dir(tmp: &Tmp, dir: &str, before: impl FnOnce(&mut App)) -> App {
    let cfg = tmp.0.join("config");
    for (name, d) in [
        ("Projects", "projects"),
        ("Members", "members"),
        ("Tasks", "tasks"),
    ] {
        places::save(
            &cfg,
            Place {
                name: name.into(),
                group: String::new(),
                path: tmp.notes().join(d).canonicalize().unwrap(),
                view: None,
            },
        )
        .unwrap();
    }
    let dir = tmp.notes().join(dir);
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(120, 30);
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
    before(&mut app);
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// 名前(拡張子なし)が `name` の行を選び、そのノートのパスを返す。
fn select(a: &mut App, name: &str) -> PathBuf {
    let i = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).trim_end_matches(".md").ends_with(name)))
        .unwrap_or_else(|| panic!("{name} の行が無い"));
    a.row = i;
    PathBuf::from(&a.cur_row().unwrap().0)
}

fn same_dir(p: &Path, dir: &Path) -> bool {
    p.canonicalize().ok() == dir.canonicalize().ok()
}

#[test]
fn test_linked_note_moves_to_the_table_that_points_here() {
    // [utest -> req~cell-edit.new-linked-note~1] 表「メンバー」の「Alice」で `c`(親子なし。Alice を指すのは tasks の assignee だけ)→ 表「タスク」へ
    // 移る頼みと、移った先で assignee に Alice へのリンクを入れて新しいノートの窓を出す頼み、作るかやめたら Alice へ戻る頼み。
    let tmp = workspace("lk_member");
    let mut a = boot_dir(&tmp, "members", |_| {});
    let alice = select(&mut a, "Alice");
    ch(&mut a, 'c');
    let to = a
        .switch_to
        .as_ref()
        .unwrap_or_else(|| panic!("{:?}", a.message));
    assert!(
        same_dir(&to.path, &tmp.notes().join("tasks")),
        "{:?}",
        to.path
    );
    assert!(a.switch_new_note);
    assert_eq!(
        a.switch_new_link,
        Some(("assignee".to_string(), alice.clone()))
    );
    let (back, row) = a.switch_return.clone().unwrap();
    assert!(
        same_dir(&back.path, &tmp.notes().join("members")),
        "{:?}",
        back.path
    );
    assert_eq!(row, alice);
}

/// 表「タスク」を、「メンバー」の「Alice」の `c` で移ってきた形で開く。
fn arrived(tmp: &Tmp) -> (App, PathBuf) {
    let alice = tmp.notes().join("members/Alice.md").canonicalize().unwrap();
    let back = Place {
        name: "Members".into(),
        group: String::new(),
        path: tmp.notes().join("members").canonicalize().unwrap(),
        view: None,
    };
    let al = alice.clone();
    let a = boot_dir(tmp, "tasks", move |a| {
        a.new_note_after_load = true;
        a.new_link_after_load = Some(("assignee".into(), al.clone()));
        a.return_after_note = Some((back, al));
    });
    (a, alice)
}

#[test]
fn test_linked_note_form_has_the_link_and_returns_after_creating() {
    // [utest -> req~cell-edit.new-linked-note~1] 移った先の窓の assignee に Alice へのリンク。名前を打って作る → ノートの assignee は Alice を指し、
    // 元の表「メンバー」の Alice へ戻る頼み。
    let tmp = workspace("lk_arrive");
    let (mut a, alice) = arrived(&tmp);
    assert!(a.note.flow.is_some(), "窓が無い: {:?}", a.message);
    let s = super::test_screen::screen(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("assignee") && l.contains("Alice")),
        "{s}"
    );
    typing(&mut a, "d review");
    ctrl(&mut a, 's');
    let body = std::fs::read_to_string(tmp.notes().join("tasks/d review.md")).unwrap();
    assert!(
        body.lines()
            .any(|l| l.starts_with("assignee:") && l.contains("Alice")),
        "{body}"
    );
    let to = a
        .switch_to
        .as_ref()
        .unwrap_or_else(|| panic!("戻らない: {:?}", a.message));
    assert!(
        same_dir(&to.path, &tmp.notes().join("members")),
        "{:?}",
        to.path
    );
    assert_eq!(a.switch_select, Some(alice));
}

#[test]
fn test_linked_note_cancel_returns_and_creates_nothing() {
    // [utest -> req~cell-edit.new-linked-note~1] 移った先の窓を Esc でやめる → 何も作らず、元の表「メンバー」の Alice へ戻る頼み。
    let tmp = workspace("lk_cancel");
    let (mut a, alice) = arrived(&tmp);
    let before = std::fs::read_dir(tmp.notes().join("tasks"))
        .unwrap()
        .count();
    press(&mut a, KeyCode::Esc);
    if a.note.flow.is_some() {
        press(&mut a, KeyCode::Esc);
    }
    assert_eq!(
        std::fs::read_dir(tmp.notes().join("tasks"))
            .unwrap()
            .count(),
        before
    );
    let to = a
        .switch_to
        .as_ref()
        .unwrap_or_else(|| panic!("戻らない: {:?}", a.message));
    assert!(
        same_dir(&to.path, &tmp.notes().join("members")),
        "{:?}",
        to.path
    );
    assert_eq!(a.switch_select, Some(alice));
}

#[test]
fn test_linked_note_asks_which_when_two_columns_point_here() {
    // [utest -> req~cell-edit.new-linked-note~1] 「mdgrid」を指すのは tasks の project と related の2つ → どちらを作るか選ぶ窓(2つの項目)。
    let tmp = workspace("lk_two");
    let mut a = boot_dir(&tmp, "projects", |_| {});
    select(&mut a, "mdgrid");
    ch(&mut a, 'c');
    assert!(a.switch_to.is_none());
    match &a.nv.ask {
        Some(Ask::LinkedNote { items, .. }) => {
            assert_eq!(items.len(), 2, "{items:?}");
            assert!(items.iter().any(|i| i.contains("project")), "{items:?}");
            assert!(items.iter().any(|i| i.contains("related")), "{items:?}");
        }
        _ => panic!("選ぶ窓が無い: {:?}", a.message),
    }
}

#[test]
fn test_linked_note_none_says_why() {
    // [utest -> req~cell-edit.new-linked-note~1] 親子が無く、どの表からも指されていない表「タスク」で `c` → 親子で並べていない理由(ビューの設定のキー)。
    let tmp = workspace("lk_none");
    let mut a = boot_dir(&tmp, "tasks", |_| {});
    select(&mut a, "a build");
    ch(&mut a, 'c');
    assert!(a.switch_to.is_none());
    assert!(a.note.flow.is_none());
    assert_eq!(
        a.message.as_deref(),
        Some(Msg::TreeOff.fill(&[&"o"]).as_str())
    );
    let _ = RowId(String::new());
}

/// 1つの根に、印に中身を書いた表「人」(People/)と「会議」(Meetings/)。会議の attendees が人を指す。表「人」を開く。
fn people_ledger(tag: &str) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    let root = tmp.notes();
    let w = |rel: &str, text: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("People/林.md", "---\nrole: dev\n---\n");
    w("Meetings/定例.md", "---\nattendees:\n  - \"[[林]]\"\n---\n");
    w(
        ".mdgrid/workspace.toml",
        "open = \"人\"\n\n[types]\nattendees = \"list\"\n\n\
         [[table]]\nname = \"人\"\n[table.new_note]\nfolder = \"People\"\n[table.base]\nfilters = { and = ['file.inFolder(\"People\")'] }\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n\n\
         [[table]]\nname = \"会議\"\n[table.new_note]\nfolder = \"Meetings\"\n[table.base]\nfilters = { and = ['file.inFolder(\"Meetings\")'] }\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"attendees\"]\n",
    );
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let (table, warns) = mdgrid::workspace::marker_open(&root);
    assert!(warns.is_empty(), "{warns:?}");
    let path = table.map(|t| t.path).unwrap();
    let t = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: warns,
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: crate::state_target(std::slice::from_ref(&path)),
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

#[test]
fn test_linked_note_from_a_table_written_in_the_marker() {
    // [utest -> req~cell-edit.new-linked-note~1] 印に中身を書いた表「人」(根を開き、People/ に絞る)の「林」で `c` → 会議の attendees を指す
    // つながりから、表「会議」へ移る頼み(attendees に林へのリンク)。
    let (_t, mut a) = people_ledger("lk_marker");
    let hayashi = select(&mut a, "林");
    ch(&mut a, 'c');
    let to = a
        .switch_to
        .as_ref()
        .unwrap_or_else(|| panic!("{:?}", a.message));
    assert!(
        to.name == "会議" || to.path.to_string_lossy().contains("会議"),
        "{to:?}"
    );
    assert_eq!(a.switch_new_link, Some(("attendees".to_string(), hayashi)));
}
