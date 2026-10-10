//! ワークスペースの画面(WS-2・WS-4・WS-6): パレットで足す・外す・開く、範囲で絞るリンクと関係マップ、
//! ヘッダーの名前。見本は test_relations.rs の notes/(tasks・projects・members)。

use super::keymap::Mode;
use super::test_relations::{boot_tasks, workspace};
use super::test_screen::{ch, press, screen, typing};
use super::*;
use mdgrid::workspace::{self, WsTable};
use ratatui::crossterm::event::KeyCode;

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

fn table(tmp: &super::test_screen::Tmp, name: &str, dir: &str) -> WsTable {
    WsTable {
        name: name.into(),
        path: tmp.notes().join(dir).canonicalize().unwrap(),
        view: None,
        profile: Default::default(),
    }
}

/// Work に tasks と projects、Hobby に members を書く。
fn write_two(tmp: &super::test_screen::Tmp) {
    let cfg = tmp.0.join("config");
    workspace::add(&cfg, "Work", table(tmp, "Tasks", "tasks")).unwrap();
    workspace::add(&cfg, "Work", table(tmp, "Projects", "projects")).unwrap();
    workspace::add(&cfg, "Hobby", table(tmp, "Members", "members")).unwrap();
}

#[test]
fn test_ws_2_add_and_remove_from_palette() {
    // [WS-2] 「この表をワークスペースに足す」→ 新しい名前を打つ → workspaces.toml に入り、範囲になる。外すと消える。
    let tmp = workspace("ws_add");
    let cfg = tmp.0.join("config");
    let mut a = boot_tasks(&tmp);
    assert!(a.scope.is_none(), "書く前は範囲なし(登録した表)");
    palette(&mut a, "この表をワークスペースに足す");
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    typing(&mut a, "Product");
    assert!(
        screen(&a).contains("+ 新しいワークスペース: Product"),
        "{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("Product"),
        "{:?}",
        a.message
    );
    let (list, _) = workspace::load(&cfg);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].tables, vec![table(&tmp, "tasks", "tasks")]);
    assert_eq!(a.scope.as_ref().map(|s| s.name.as_str()), Some("Product"));
    // 外す: 入っているワークスペースから選ぶ。
    palette(&mut a, "この表をワークスペースから外す");
    assert!(screen(&a).contains("Product"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(workspace::load(&cfg).0[0].tables.is_empty());
    // どこにも無ければ理由。
    palette(&mut a, "この表をワークスペースから外す");
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains("どのワークスペースにも無い"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_ws_2_pick_existing_and_open() {
    // [WS-2] 足す先は今の名前から選べる。「ワークスペースを開く」→ 名前 → 表 → その表へ移る(範囲はそのワークスペース)。
    let tmp = workspace("ws_open");
    write_two(&tmp);
    let mut a = boot_tasks(&tmp);
    palette(&mut a, "この表をワークスペースに足す");
    let s = screen(&a);
    assert!(s.contains("Work") && s.contains("Hobby"), "{s}");
    press(&mut a, KeyCode::Esc);
    a.set_mode(Mode::Table);
    palette(&mut a, "ワークスペースを開く");
    typing(&mut a, "Hobby");
    press(&mut a, KeyCode::Enter);
    assert!(
        screen(&a).contains("Hobby のどの表を開く"),
        "{}",
        screen(&a)
    );
    assert!(screen(&a).contains("Members"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    let to = a.switch_to.as_ref().expect("移る表");
    assert_eq!(to.path, tmp.notes().join("members").canonicalize().unwrap());
    assert_eq!(a.switch_workspace.as_deref(), Some("Hobby"));
    assert!(a.quit, "ためた変更が無ければすぐ移る");
}

#[test]
fn test_ws_2_open_none_says_how() {
    // [WS-2] ワークスペースが無ければ、作り方を出して一覧を開かない。
    let tmp = workspace("ws_none");
    let mut a = boot_tasks(&tmp);
    palette(&mut a, "ワークスペースを開く");
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("workspace_new"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_ws_4_scope_limits_tables_and_relmap() {
    // [WS-4] tasks を含む Work の表(tasks・projects)だけが範囲。Hobby の members は関係マップに出ない。
    let tmp = workspace("ws_scope");
    write_two(&tmp);
    let mut a = boot_tasks(&tmp);
    let names: Vec<String> = a.scope_places().into_iter().map(|p| p.name).collect();
    assert_eq!(names, ["Tasks", "Projects"]);
    ch(&mut a, 'R');
    assert_eq!(a.mode, Mode::Relations, "{}", screen(&a));
    let s = screen(&a);
    assert!(s.contains("Projects"), "{s}");
    assert!(!s.contains("Members"), "{s}");
}

#[test]
fn test_ws_6_header_shows_workspace() {
    // [WS-6] 書いたワークスペースの範囲なら、ヘッダーにその名前。範囲が無ければ出さない
    // (ヘッダーのボタン「ワークスペース」(SR-42)は範囲の表示に数えない)。
    let tmp = workspace("ws_header");
    let a = boot_tasks(&tmp);
    assert!(!screen(&a).contains("· ワークスペース"), "{}", screen(&a));
    write_two(&tmp);
    let a = boot_tasks(&tmp);
    let head = screen(&a).lines().next().unwrap_or_default().to_string();
    assert!(head.contains("ワークスペース Work"), "{head}");
}

/// projects のフォルダを開く(登録した表は無し。範囲はワークスペースで決まる)。
fn boot_projects(tmp: &super::test_screen::Tmp) -> App {
    use super::startup::Startup;
    use mdgrid::config::Config;
    use mdgrid::source::markdown::Markdown;
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
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

#[test]
fn test_ws_4_linked_rows_follow_scope() {
    // [WS-4][REL-5] つながった行は範囲の表から探す。tasks を含む Work なら出る。tasks の無い範囲なら出ない。
    let tmp = workspace("ws_back");
    write_two(&tmp);
    let mut a = boot_projects(&tmp);
    assert_eq!(a.scope.as_ref().map(|s| s.name.as_str()), Some("Work"));
    super::test_relations::goto(&mut a, "mdgrid", "status");
    palette(&mut a, "linked_rows");
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    assert!(
        screen(&a).contains("Tasks › project ← a build"),
        "{}",
        screen(&a)
    );
    // projects と members だけの範囲にする(tasks は外す)。
    let cfg = tmp.0.join("config");
    workspace::remove(&cfg, "Work", Some(&tmp.notes().join("tasks"))).unwrap();
    let mut a = boot_projects(&tmp);
    super::test_relations::goto(&mut a, "mdgrid", "status");
    palette(&mut a, "linked_rows");
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains("指すノートは無い"),
        "{:?}",
        a.message
    );
}
