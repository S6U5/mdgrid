//! リンクの値の見せ方と行き先の表の不具合の直し(specs/_changes/2026-10-15-link-labels.md)。
//! REL-2: まとまりの見出しと頻度表(`%`)も、セルと同じに行き先の名前で見せる。
//! REL-4: `.base` の表から、`.base` の外のフォルダのノートへのリンクを開ける。
//! REL-7: 一部のフォルダだけの範囲で、関係マップに今の表の根のフォルダの余分な箱を出さない。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_relations::{boot_tasks, goto, workspace};
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::settings::{Dir, Group};
use mdgrid::source::markdown::Markdown;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

/// workspace() の notes をルートフォルダ(`.obsidian`)にし、clients/ と tasks の `.base` を足す。
/// d dep は a build(done)に依る。Tasks.base のビュー Open は done を隠す。
fn vault(name: &str) -> Tmp {
    let tmp = workspace(name);
    let n = tmp.notes();
    std::fs::create_dir_all(n.join(".obsidian")).unwrap();
    std::fs::create_dir_all(n.join("clients")).unwrap();
    std::fs::write(n.join("clients/Acme.md"), "---\nindustry: retail\n---\n").unwrap();
    std::fs::write(
        n.join("tasks/d dep.md"),
        "---\nproject: \"[[mdgrid]]\"\ndepends: \"[[a build]]\"\nstatus: todo\n---\n",
    )
    .unwrap();
    std::fs::write(
        n.join("Tasks.base"),
        "filters:\n  and:\n    - file.inFolder(\"tasks\")\nviews:\n  - type: table\n    name: Open\n    filters:\n      and:\n        - status != \"done\"\n",
    )
    .unwrap();
    tmp
}

/// `-w` で選んだワークスペースの範囲にする(ルートフォルダの `.obsidian` で検知した範囲の代わりに)。
fn choose(a: &mut App, tmp: &Tmp, tables: &[(&str, &str)]) {
    a.scope = Some(Scope {
        name: "Work".into(),
        source: Source::Chosen,
        tables: tables
            .iter()
            .map(|(name, path)| WsTable {
                name: (*name).into(),
                path: tmp.notes().join(path).canonicalize().unwrap(),
                ..WsTable::default()
            })
            .collect(),
        profile: Default::default(),
        warnings: Vec::new(),
    });
}

fn start(
    tmp: &Tmp,
    src: Markdown,
    target: PathBuf,
    base: Option<(mdgrid::base::Base, String, Option<usize>)>,
) -> App {
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
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

/// Tasks.base を開く。
fn boot_base(tmp: &Tmp) -> App {
    let path = tmp.notes().join("Tasks.base");
    let text = std::fs::read_to_string(&path).unwrap();
    let base = mdgrid::base::parse(&text).unwrap();
    let src = Markdown::open_vault(&path).unwrap();
    start(tmp, src, path, Some((base, "Tasks.base".into(), None)))
}

/// tasks のフォルダを開く。
fn boot_folder(tmp: &Tmp) -> App {
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    start(tmp, src, dir, None)
}

fn real(p: PathBuf) -> PathBuf {
    p.canonicalize().unwrap()
}

#[test]
fn test_rel_2_group_heading_shows_link_names() {
    // [REL-2] リンクの列でまとめた見出しは、`[[…]]` でなく行き先の名前。行き先の無いものは `?` を付ける。
    let tmp = workspace("ll_heading");
    let mut a = boot_tasks(&tmp);
    a.settings.group = Group::By {
        col: "project".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    assert!(!a.groups.is_empty());
    let s = screen(&a);
    assert!(s.contains("project: mdgrid"), "{s}");
    assert!(s.contains("project: ?nothing"), "{s}");
    assert!(
        !s.contains("[[mdgrid]]") && !s.contains("[[nothing]]"),
        "{s}"
    );
}

#[test]
fn test_rel_2_list_group_heading_shows_link_names() {
    // [REL-2] リストのリンクの列でまとめた見出しも、要素ごとに名前(`, ` でつなぐ)。
    let tmp = workspace("ll_heading_list");
    let mut a = boot_tasks(&tmp);
    a.settings.group = Group::By {
        col: "related".into(),
        dir: Dir::Asc,
        hide_empty: true,
    };
    a.refresh();
    let s = screen(&a);
    assert!(s.contains("related: mdgrid, ?ghost"), "{s}");
    assert!(!s.contains("[["), "{s}");
}

#[test]
fn test_rel_2_frequency_shows_link_names() {
    // [REL-2] `%` の件数の表も、リンクは行き先の名前で見せる(絞り込みは元の値のまま)。
    let tmp = workspace("ll_freq");
    let mut a = boot_tasks(&tmp);
    goto(&mut a, "a build", "related");
    ch(&mut a, '%');
    assert_eq!(a.mode, Mode::Freq, "{:?}", a.message);
    let f = a.freq.clone().unwrap();
    let mdgrid_at = f
        .items
        .iter()
        .position(|(v, _)| v.as_deref() == Some("[[mdgrid]]"))
        .unwrap();
    assert_eq!(f.labels[mdgrid_at], "mdgrid");
    assert!(f.labels.iter().any(|l| l == "?ghost"), "{:?}", f.labels);
    let s = screen(&a);
    assert!(!s.contains("[[mdgrid]]") && !s.contains("[[ghost]]"), "{s}");
    // Enter で絞ると、元の値で同じ行が残る。
    a.freq.as_mut().unwrap().sel = mdgrid_at;
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 2);
}

#[test]
fn test_rel_4_open_link_from_base_to_registered_table() {
    // [REL-4] `.base` の表(ルートフォルダを読む)から、`.base` の外のフォルダのノートへのリンクを開くと、
    // 「隠れている」でなく、行き先を含む登録した表に移る。
    let tmp = vault("ll_base_open");
    let mut a = boot_base(&tmp);
    choose(
        &mut a,
        &tmp,
        &[
            ("Tasks", "Tasks.base"),
            ("Projects", "projects"),
            ("Members", "members"),
        ],
    );
    goto(&mut a, "d dep", "project");
    palette(&mut a, "open_link");
    assert!(a.quit, "移る: {:?}", a.message);
    assert_eq!(a.switch_to.clone().expect("移る表").name, "Projects");
    assert_eq!(
        a.switch_select.clone(),
        Some(real(tmp.notes().join("projects/mdgrid.md")))
    );
}

#[test]
fn test_rel_4_open_link_from_base_without_registered_tables() {
    // [REL-4] 範囲に行き先を含む表が無ければ、行き先のフォルダを開く。
    let tmp = vault("ll_base_open_none");
    let mut a = boot_base(&tmp);
    choose(&mut a, &tmp, &[("Tasks", "Tasks.base")]);
    goto(&mut a, "d dep", "project");
    palette(&mut a, "open_link");
    assert!(a.quit, "移る: {:?}", a.message);
    let place = a.switch_to.clone().expect("移る表");
    assert_eq!(place.path, real(tmp.notes().join("projects")));
}

#[test]
fn test_rel_4_base_row_hidden_by_view_filter_gives_reason() {
    // [REL-4] 行き先が `.base` の表のノートで、ビューの絞り込みで隠れているだけなら、今までどおり理由を出す。
    let tmp = vault("ll_base_hidden");
    let mut a = boot_base(&tmp);
    goto(&mut a, "d dep", "depends");
    palette(&mut a, "open_link");
    assert!(!a.quit && a.switch_to.is_none());
    assert!(
        a.message.as_deref().unwrap_or("").contains("隠れている"),
        "{:?}",
        a.message
    );
}

/// 関係マップの箱の名前。
fn relmap_names(a: &mut App) -> Vec<String> {
    a.open_relmap();
    assert_eq!(a.mode, Mode::Relations, "{:?}", a.message);
    a.relmap
        .as_ref()
        .unwrap()
        .infos
        .iter()
        .map(|t| t.name.clone())
        .collect()
}

#[test]
fn test_rel_7_partial_workspace_has_no_extra_table() {
    // [REL-7] ルートフォルダの下の一部のフォルダだけを表にした範囲で、その1つを開くと、関係マップの箱は
    // その表だけ(ルートフォルダの残りのノートの余分な箱が出ない)。今の表の箱を選ぶ。
    let tmp = vault("ll_partial");
    let mut a = boot_folder(&tmp);
    choose(
        &mut a,
        &tmp,
        &[
            ("Tasks", "tasks"),
            ("Projects", "projects"),
            ("Members", "members"),
        ],
    );
    let mut names = relmap_names(&mut a);
    names.sort();
    assert_eq!(names, ["Members", "Projects", "Tasks"]);
    let rm = a.relmap.as_ref().unwrap();
    assert_eq!(rm.infos[rm.sel].name, "Tasks");
    assert_eq!(rm.infos[rm.sel].rows, 4);
}

#[test]
fn test_rel_7_folder_inside_base_table_has_no_extra_table() {
    // [REL-7] 今のフォルダが範囲の `.base` の表(ルートフォルダ)の中なら、今の表の箱を足さず、`.base` の箱を選ぶ。
    let tmp = vault("ll_inside_base");
    let mut a = boot_folder(&tmp);
    choose(
        &mut a,
        &tmp,
        &[
            ("Tasks", "Tasks.base"),
            ("Projects", "projects"),
            ("Members", "members"),
        ],
    );
    let mut names = relmap_names(&mut a);
    names.sort();
    assert_eq!(names, ["Members", "Projects", "Tasks"]);
    let rm = a.relmap.as_ref().unwrap();
    assert_eq!(rm.infos[rm.sel].name, "Tasks");
}
