//! 候補の出し方(cell-edit.candidates-sort・-where・-table-rows)。期待値は specs/requirements/cell-edit.md の
//! 受け入れの例から。台帳の根(印 `.mdgrid/workspace.toml` のあるフォルダ)を開く。

use super::grid::Slot;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, press, screen, typing, Tmp};
use super::*;
use mdgrid::source::RowId;
use ratatui::crossterm::event::KeyCode;
use std::path::Path;

/// 印に `marker` を書き、`files`(根からの相対のパスと中身)を置いて、根のフォルダを開く。
fn boot(tag: &str, marker: &str, files: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (rel, text) in files {
        let p = tmp.notes().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    let dir = tmp.notes().join(".mdgrid");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("workspace.toml"), marker).unwrap();
    let a = open_root(&tmp);
    (tmp, a)
}

/// 根を main と同じ道筋で開く(印の `open` の表に置き換える。CLI-1)。印の警告はメッセージ行に出す。
fn open_root(tmp: &Tmp) -> App {
    let root = tmp.notes();
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let (table, warns) = mdgrid::workspace::marker_open(&root);
    let path = table.map_or(root, |t| t.path);
    let t = crate::open_target(std::slice::from_ref(&path), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 40);
    a.start(Startup {
        config: mdgrid::config::Config::default(),
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
    a
}

/// タスクの表(Tasks/。親子は parent)の印。`rules` は表の範囲の決まりの行。
fn tasks_marker(rules: &str) -> String {
    format!(
        "open = \"タスク\"\n\n[types]\ntags = \"list\"\n\n[[table]]\nname = \"タスク\"\nchildren = {{ key = \"parent\" }}\n\n\
         [table.new_note]\nfolder = \"Tasks\"\nask = [\"parent\", \"tags\"]\n\n\
         [table.base]\nfilters = {{ and = ['file.inFolder(\"Tasks\")'] }}\n\n\
         [[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"status\", \"parent\", \"tags\"]\n\n{rules}\n"
    )
}

/// 「設計」の子が3つ・「調査」の子が1つ・「片づけ」の子が無い。
const FAMILY: &[(&str, &str)] = &[
    ("Tasks/設計.md", "---\nstatus: todo\n---\n"),
    ("Tasks/調査.md", "---\nstatus: todo\n---\n"),
    ("Tasks/片づけ.md", "---\nstatus: todo\n---\n"),
    (
        "Tasks/画面.md",
        "---\nstatus: todo\nparent: \"[[設計]]\"\n---\n",
    ),
    (
        "Tasks/部品.md",
        "---\nstatus: todo\nparent: \"[[設計]]\"\n---\n",
    ),
    (
        "Tasks/API.md",
        "---\nstatus: todo\nparent: \"[[設計]]\"\n---\n",
    ),
    (
        "Tasks/文献.md",
        "---\nstatus: todo\nparent: \"[[調査]]\"\n---\n",
    ),
];

/// 開いている1つの値の入力の候補の見せる名前。
fn items(a: &App) -> Vec<String> {
    a.input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
        .unwrap_or_default()
}

/// 付け外しの窓の候補の名前。
fn pick_names(a: &App) -> Vec<String> {
    let p = a.pick.as_ref().expect("付け外しの窓が開いている");
    p.cands.iter().map(|c| c.shown().to_string()).collect()
}

fn at(list: &[String], name: &str) -> usize {
    list.iter()
        .position(|x| x == name)
        .unwrap_or_else(|| panic!("候補に {name} が無い: {list:?}"))
}

/// 新しいノートの窓を開き、名前を打って Enter(最初の聞く欄 parent へ)。
fn form_parent(a: &mut App) {
    ch(a, 'a');
    typing(a, "新しい");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "parent の欄:\n{}", screen(a));
}

/// 新しいノートの窓で tags の欄(parent の次)まで進む。
fn form_tags(a: &mut App) {
    form_parent(a);
    press(a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::ListPick, "tags の欄:\n{}", screen(a));
}

fn row(a: &App, rel: &str) -> RowId {
    let want: Vec<&str> = rel.split('/').collect();
    a.src
        .rows()
        .into_iter()
        .find(|r| {
            let got: Vec<String> = Path::new(&r.0)
                .components()
                .rev()
                .take(want.len())
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            got.iter()
                .rev()
                .map(String::as_str)
                .eq(want.iter().copied())
        })
        .unwrap_or_else(|| panic!("{rel} が無い"))
}

fn select(a: &mut App, rel: &str) {
    let r = row(a, rel);
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(k) if a.rows[k] == r))
        .unwrap_or_else(|| panic!("{rel} の行が表に無い:\n{}", screen(a)));
}

// [utest -> req~cell-edit.candidates-sort~1]
/// sort = "count": 親を指す列の候補は、子の多い 設計・調査 が先頭から並び、子の無い 片づけ はその後ろ。
#[test]
fn test_parent_candidates_by_children() {
    let marker = tasks_marker("[table.rules.parent.candidates]\nsort = \"count\"");
    let (_t, mut a) = boot("cand_count", &marker, FAMILY);
    form_parent(&mut a);
    let got = items(&a);
    assert_eq!(&got[..2], ["設計", "調査"], "{got:?}");
    assert!(at(&got, "調査") < at(&got, "片づけ"), "{got:?}");
    assert_eq!(
        got.last().map(String::as_str),
        Some(super::list::NONE_LABEL.text()),
        "{got:?}"
    );
}

// [utest -> req~cell-edit.candidates-sort~1]
/// sort = "name": 片づけ・設計・調査 がこの順に並ぶ。
#[test]
fn test_parent_candidates_by_name() {
    let marker = tasks_marker("[table.rules.parent.candidates]\nsort = \"name\"");
    let (_t, mut a) = boot("cand_name", &marker, FAMILY);
    form_parent(&mut a);
    let got = items(&a);
    assert!(
        at(&got, "片づけ") < at(&got, "設計") && at(&got, "設計") < at(&got, "調査"),
        "{got:?}"
    );
}

// [utest -> req~cell-edit.candidates-sort~1]
/// sort = "recent": 一番新しく直したノートのタグ「家」が、件数の多い「仕事」より先に並ぶ。
#[test]
fn test_tag_candidates_by_recent() {
    let marker = tasks_marker("[table.rules.tags.candidates]\nsort = \"recent\"");
    let files: &[(&str, &str)] = &[
        ("Tasks/a.md", "---\ntags:\n  - 仕事\n---\n"),
        ("Tasks/b.md", "---\ntags:\n  - 仕事\n---\n"),
        ("Tasks/c.md", "---\ntags:\n  - 仕事\n---\n"),
        ("Tasks/d.md", "---\ntags:\n  - 家\n---\n"),
    ];
    let (t, first) = boot("cand_recent", &marker, files);
    drop(first);
    // d.md を一番新しくする(ほかは 1 時間前)。
    let now = std::time::SystemTime::now();
    for (rel, _) in files {
        let f = std::fs::File::options()
            .write(true)
            .open(t.notes().join(rel))
            .unwrap();
        let when = if rel.ends_with("d.md") {
            now
        } else {
            now - std::time::Duration::from_secs(3600)
        };
        f.set_modified(when).unwrap();
    }
    let mut a = open_root(&t);
    form_tags(&mut a);
    let got = pick_names(&a);
    assert!(at(&got, "家") < at(&got, "仕事"), "{got:?}");
}

// [utest -> req~cell-edit.candidates-sort~1]
/// 知らない sort は、sort と選べる値を言う警告を出し、書かなかったときと同じ(リンクの列は名前の順)に並べる。
#[test]
fn test_unknown_sort_warns() {
    let marker = tasks_marker("[table.rules.parent.candidates]\nsort = \"often\"");
    let (_t, mut a) = boot("cand_badsort", &marker, FAMILY);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("candidates.sort") && m.contains("recent"), "{m}");
    form_parent(&mut a);
    let got = items(&a);
    assert!(
        at(&got, "片づけ") < at(&got, "設計") && at(&got, "設計") < at(&got, "調査"),
        "{got:?}"
    );
}

const OLD: &[(&str, &str)] = &[
    ("Tasks/設計.md", "---\nstatus: todo\n---\n"),
    ("Tasks/旧案.md", "---\nstatus: done\n---\n"),
    (
        "Tasks/名残.md",
        "---\nstatus: todo\nparent: \"[[旧案]]\"\n---\n",
    ),
];

// [utest -> req~cell-edit.candidates-where~1]
/// where = { status = [todo, doing] }: done の 旧案 は親の候補に出ず、「なし」は残る。
#[test]
fn test_where_drops_done_parents() {
    let marker =
        tasks_marker("[table.rules.parent.candidates]\nwhere = { status = [\"todo\", \"doing\"] }");
    let (_t, mut a) = boot("cand_where1", &marker, OLD);
    form_parent(&mut a);
    let got = items(&a);
    assert!(got.contains(&"設計".to_string()), "{got:?}");
    assert!(!got.contains(&"旧案".to_string()), "{got:?}");
    assert_eq!(
        got.last().map(String::as_str),
        Some(super::list::NONE_LABEL.text()),
        "{got:?}"
    );
}

// [utest -> req~cell-edit.candidates-where~1]
/// 今の値が絞りに当たらない 旧案 を指すセルを直すときは、旧案 も候補に残り、今の値として選ばれている。
#[test]
fn test_where_keeps_current_value() {
    let marker =
        tasks_marker("[table.rules.parent.candidates]\nwhere = { status = [\"todo\", \"doing\"] }");
    let (_t, mut a) = boot("cand_where2", &marker, OLD);
    select(&mut a, "Tasks/名残.md");
    col_named(&mut a, "parent");
    press(&mut a, KeyCode::Enter);
    let l = a
        .input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .expect("候補のリスト");
    assert_eq!(
        l.items.get(l.sel).map(|i| i.text.as_str()),
        Some("旧案"),
        "{:?}",
        items(&a)
    );
}

// [utest -> req~cell-edit.candidates-where~1]
/// where = { status = [todo] }: done のノートにだけあるタグ「昔」は候補に出ない。
#[test]
fn test_where_drops_values_of_done_notes() {
    let marker = tasks_marker("[table.rules.tags.candidates]\nwhere = { status = [\"todo\"] }");
    let files: &[(&str, &str)] = &[
        ("Tasks/a.md", "---\nstatus: todo\ntags:\n  - 仕事\n---\n"),
        ("Tasks/b.md", "---\nstatus: done\ntags:\n  - 昔\n---\n"),
    ];
    let (_t, mut a) = boot("cand_where3", &marker, files);
    form_tags(&mut a);
    let got = pick_names(&a);
    assert!(got.contains(&"仕事".to_string()), "{got:?}");
    assert!(!got.contains(&"昔".to_string()), "{got:?}");
}

// [utest -> req~cell-edit.candidates-where~1]
/// where が表でなければ、where を言う警告を出し、絞らない。
#[test]
fn test_where_not_table_warns() {
    let marker = tasks_marker("[table.rules.parent.candidates]\nwhere = \"status\"");
    let (_t, mut a) = boot("cand_where4", &marker, OLD);
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("candidates.where"), "{m}");
    form_parent(&mut a);
    assert!(items(&a).contains(&"旧案".to_string()), "{:?}", items(&a));
}

// [utest -> req~cell-edit.candidates-table-rows~2]
/// candidates を書いた列の値の候補は今の表のノートから: サイトの表のタグ「料理」はタスクの窓に出ない。
#[test]
fn test_values_come_from_this_table() {
    let marker = format!(
        "{}\n[[table]]\nname = \"サイト\"\n\n[table.base]\nfilters = {{ and = ['file.inFolder(\"Sites\")'] }}\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n",
        tasks_marker("[table.rules.tags.candidates]\nsort = \"count\"")
    );
    let files: &[(&str, &str)] = &[
        ("Tasks/a.md", "---\ntags:\n  - 仕事\n---\n"),
        ("Sites/s.md", "---\ntags:\n  - 料理\n---\n"),
    ];
    let (_t, mut a) = boot("cand_table", &marker, files);
    form_tags(&mut a);
    let got = pick_names(&a);
    assert!(got.contains(&"仕事".to_string()), "{got:?}");
    assert!(!got.contains(&"料理".to_string()), "{got:?}");
}
