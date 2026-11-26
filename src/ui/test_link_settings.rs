//! リンクの書き方の設定(relations.link-format-setting・relations.link-path-setting)と、ノートの名前を変えたときの
//! リンクの書き換え(cell-edit.rename-note-links-resolved・-links-setting)と隠した名前の頭(cell-edit.rename-note-hidden-head)
//! の試験。期待値は要求の受け入れの例から取った。台帳の根(印 `.mdgrid/workspace.toml` のあるフォルダ)を開く。

use super::grid::Slot;
use super::keymap::{Action, Mode};
use super::startup::Startup;
use super::test_screen::{col_named, press, typing, Tmp};
use super::*;
use mdgrid::relations::{self, Notes};
use mdgrid::source::{RowId, Value};
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};

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
    let root = tmp.notes();
    let cdir = tmp.0.join("config");
    std::fs::create_dir_all(&cdir).unwrap();
    let t = crate::open_target(std::slice::from_ref(&root), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: mdgrid::config::Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cdir),
        target: root,
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 根からの相対のパスが `rel` の行(区切りは OS で違うので、パスの部品で比べる)。
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
        .unwrap_or_else(|| panic!("{rel} の行が表に無い"));
}

/// 開いている入力の候補の見せる名前。
fn items(a: &App) -> Vec<String> {
    a.input
        .as_ref()
        .and_then(|i| i.list.as_ref())
        .map(|l| l.items.iter().map(|i| i.text.clone()).collect())
        .unwrap_or_default()
}

/// `rel` の行の `col` のセルを開き、候補の `text` を ↑↓ で選んで Enter。
fn pick(a: &mut App, rel: &str, col: &str, text: &str) {
    select(a, rel);
    col_named(a, col);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    let n = items(a).len();
    for _ in 0..n {
        press(a, KeyCode::Up);
    }
    for _ in 0..=n {
        let l = a.input.as_ref().and_then(|i| i.list.as_ref()).unwrap();
        if l.items.get(l.sel).is_some_and(|it| it.text == text) {
            press(a, KeyCode::Enter);
            return;
        }
        press(a, KeyCode::Down);
    }
    panic!("候補に {text} が無い: {:?}", items(a));
}

/// `rel` の行の `col` の値(ためた変更があればその値)。
fn value(a: &App, rel: &str, col: &str) -> Option<Value> {
    a.prop(&row(a, rel), col)
}

fn s(v: &str) -> Option<Value> {
    Some(Value::Str(v.into()))
}

/// 値のリンクを、根 `root` の下の `note`(根からの相対)から解いた行き先(根からの相対。区切りは `/`)。
fn target_of(root: &Path, note: &str, v: &str) -> Option<String> {
    let link = relations::parse(v)?;
    let rels: Vec<String> = walk(root);
    let notes = Notes::from_rels(root, rels);
    let got = relations::resolve(&link, &root.join(note), &[&notes])?;
    let base = std::fs::canonicalize(root).unwrap();
    Some(
        got.strip_prefix(&base)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

/// 根の下の `.md`(根からの相対)。
fn walk(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "md") {
                let rel = p.strip_prefix(root).unwrap();
                out.push(
                    rel.components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/"),
                );
            }
        }
    }
    out
}

/// タスクの見本: 親タスクの列(parent)を、部屋の片づけだけが `existing` で持つ。
fn tasks(existing: &str) -> Vec<(&'static str, String)> {
    vec![
        ("Tasks/見積もりを出す.md", "---\nstatus: todo\n---\n".into()),
        ("Tasks/見積書を作る.md", "---\nstatus: todo\n---\n".into()),
        (
            "Tasks/部屋の片づけ.md",
            format!("---\nstatus: todo\nparent: \"{existing}\"\n---\n"),
        ),
        ("Projects/家の片づけ.md", "---\nstatus: todo\n---\n".into()),
    ]
}

fn boot_tasks(tag: &str, edit: &str, existing: &str) -> (Tmp, App) {
    let files = tasks(existing);
    let refs: Vec<(&str, &str)> = files.iter().map(|(p, t)| (*p, t.as_str())).collect();
    boot(tag, &format!("[edit]\n{edit}\n"), &refs)
}

#[test]
fn test_link_format_markdown_writes_markdown_links_in_a_wiki_column() {
    // [utest -> req~relations.link-format-setting~1] edit.link_format = "markdown" なら、列の値が [[…]] でも [名前](パス) の形で書く。
    let (_t, mut a) = boot_tasks("lf-md", "link_format = \"markdown\"", "[[見積もりを出す]]");
    pick(&mut a, "Tasks/見積書を作る.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Tasks/見積書を作る.md", "parent"),
        s("[見積もりを出す](見積もりを出す.md)")
    );
}

#[test]
fn test_link_format_wiki_writes_wiki_links_in_a_markdown_column() {
    // [utest -> req~relations.link-format-setting~1] edit.link_format = "wiki" なら、列の値が [文字](パス) でも [[…]] の形で書く。
    let (_t, mut a) = boot_tasks(
        "lf-wiki",
        "link_format = \"wiki\"",
        "[見積もりを出す](見積もりを出す.md)",
    );
    pick(&mut a, "Tasks/見積書を作る.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Tasks/見積書を作る.md", "parent"),
        s("[[見積もりを出す]]")
    );
}

#[test]
fn test_link_format_unknown_value_warns_and_follows_the_column() {
    // [utest -> req~relations.link-format-setting~1] 知らない値は edit.link_format を言う警告を出し、列の形(auto)で書く。
    let (_t, mut a) = boot_tasks("lf-bad", "link_format = \"html\"", "[[見積もりを出す]]");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("edit.link_format") && m.contains("\"markdown\""),
        "{m}"
    );
    pick(&mut a, "Tasks/見積書を作る.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Tasks/見積書を作る.md", "parent"),
        s("[[見積もりを出す]]")
    );
}

#[test]
fn test_link_path_absolute_writes_the_path_from_the_root() {
    // [utest -> req~relations.link-path-setting~1] edit.link_path = "absolute" なら、開いた根からのパスで書き、行き先はそのノート。
    let (t, mut a) = boot_tasks("lp-abs", "link_path = \"absolute\"", "[[見積もりを出す]]");
    pick(&mut a, "Tasks/見積書を作る.md", "parent", "見積もりを出す");
    let v = value(&a, "Tasks/見積書を作る.md", "parent");
    assert_eq!(v, s("[[Tasks/見積もりを出す]]"));
    assert_eq!(
        target_of(
            &t.notes(),
            "Tasks/見積書を作る.md",
            "[[Tasks/見積もりを出す]]"
        )
        .as_deref(),
        Some("Tasks/見積もりを出す.md")
    );
}

#[test]
fn test_link_path_relative_markdown_writes_the_path_from_the_note() {
    // [utest -> req~relations.link-path-setting~1] relative と markdown なら、書くノートのフォルダからの相対のパスの [名前](パス)。
    let (_t, mut a) = boot_tasks(
        "lp-rel-md",
        "link_path = \"relative\"\nlink_format = \"markdown\"",
        "[[見積もりを出す]]",
    );
    pick(&mut a, "Projects/家の片づけ.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Projects/家の片づけ.md", "parent"),
        s("[見積もりを出す](../Tasks/見積もりを出す.md)")
    );
}

#[test]
fn test_link_path_relative_wiki_writes_the_path_from_the_note() {
    // [utest -> req~relations.link-path-setting~1] relative なら [[…]] も書くノートのフォルダからの相対のパス(.md なし)で、行き先はそのノート。
    let (t, mut a) = boot_tasks("lp-rel", "link_path = \"relative\"", "[[見積もりを出す]]");
    pick(&mut a, "Projects/家の片づけ.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Projects/家の片づけ.md", "parent"),
        s("[[../Tasks/見積もりを出す]]")
    );
    assert_eq!(
        target_of(
            &t.notes(),
            "Projects/家の片づけ.md",
            "[[../Tasks/見積もりを出す]]"
        )
        .as_deref(),
        Some("Tasks/見積もりを出す.md")
    );
}

#[test]
fn test_link_path_unknown_value_warns_and_writes_the_shortest() {
    // [utest -> req~relations.link-path-setting~1] 知らない値は edit.link_path を言う警告を出し、shortest(名前だけ)で書く。
    let (_t, mut a) = boot_tasks("lp-bad", "link_path = \"longest\"", "[[見積もりを出す]]");
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("edit.link_path") && m.contains("\"relative\""),
        "{m}"
    );
    pick(&mut a, "Tasks/見積書を作る.md", "parent", "見積もりを出す");
    assert_eq!(
        value(&a, "Tasks/見積書を作る.md", "parent"),
        s("[[見積もりを出す]]")
    );
}

/// 「ノートの名前を変える」を始め、入力を `to` にして Enter。
fn rename(a: &mut App, rel: &str, to: &str) {
    select(a, rel);
    a.apply(Action::RenameNote);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    a.palette.as_mut().unwrap().query = to.into();
    press(a, KeyCode::Enter);
}

#[test]
fn test_rename_rewrites_every_form_that_resolves_to_the_note_only() {
    // [utest -> req~cell-edit.rename-note-links-resolved~1] 名前を変えたノートに解けるリンクだけを、形・フォルダ・表示の文字を残して直し、
    // 別のフォルダの同じ名前のノートへのリンクは直さない。
    let (t, mut a) = boot(
        "rn-forms",
        "",
        &[
            ("Tasks/見積もりを出す.md", "---\nstatus: todo\n---\n"),
            ("Archive/見積もりを出す.md", "---\nstatus: done\n---\n"),
            (
                "Tasks/a.md",
                "---\nparent: \"[見積もりを出す](見積もりを出す.md)\"\n---\n",
            ),
            (
                "Tasks/b.md",
                "---\nparent: \"[[Tasks/見積もりを出す|見積]]\"\n---\n",
            ),
            ("Tasks/c.md", "---\nref: Tasks/見積もりを出す.md\n---\n"),
            ("Archive/d.md", "---\nparent: \"[[見積もりを出す]]\"\n---\n"),
        ],
    );
    rename(&mut a, "Tasks/見積もりを出す.md", "見積もり");
    assert!(
        t.notes().join("Tasks/見積もり.md").exists(),
        "{:?}",
        a.message
    );
    assert_eq!(
        value(&a, "Tasks/a.md", "parent"),
        s("[見積もり](見積もり.md)")
    );
    assert_eq!(
        value(&a, "Tasks/b.md", "parent"),
        s("[[Tasks/見積もり|見積]]")
    );
    assert_eq!(value(&a, "Tasks/c.md", "ref"), s("Tasks/見積もり.md"));
    assert_eq!(value(&a, "Archive/d.md", "parent"), s("[[見積もりを出す]]"));
}

/// 名前の頭の日付を隠す表のタスク1件。
fn boot_dated(tag: &str) -> (Tmp, App) {
    boot(
        tag,
        "[display]\nnote_name = '^\\d{8}_'\n",
        &[(
            "Tasks/20261125_見積もりを出す.md",
            "---\nstatus: todo\n---\n",
        )],
    )
}

#[test]
fn test_rename_starts_from_the_shown_name() {
    // [utest -> req~cell-edit.rename-note-hidden-head~1] 隠す頭のある名前は、見せる名前から入力を始める。
    let (_t, mut a) = boot_dated("rn-head-start");
    select(&mut a, "Tasks/20261125_見積もりを出す.md");
    a.apply(Action::RenameNote);
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    assert_eq!(a.palette.as_ref().unwrap().query, "見積もりを出す");
}

#[test]
fn test_rename_keeps_the_hidden_head() {
    // [utest -> req~cell-edit.rename-note-hidden-head~1] 打った名前に隠した頭を付けて名前を変える。
    let (t, mut a) = boot_dated("rn-head-keep");
    rename(
        &mut a,
        "Tasks/20261125_見積もりを出す.md",
        "見積もりを出す(改)",
    );
    assert!(
        t.notes()
            .join("Tasks/20261125_見積もりを出す(改).md")
            .exists(),
        "{:?}",
        a.message
    );
    assert!(!t.notes().join("Tasks/20261125_見積もりを出す.md").exists());
}

#[test]
fn test_rename_with_a_full_name_uses_it_as_typed() {
    // [utest -> req~cell-edit.rename-note-hidden-head~1] 打った名前が note_name に当たるなら、打ったとおりの名前にする。
    let (t, mut a) = boot_dated("rn-head-full");
    rename(
        &mut a,
        "Tasks/20261125_見積もりを出す.md",
        "20261201_見積もり",
    );
    assert!(
        t.notes().join("Tasks/20261201_見積もり.md").exists(),
        "{:?}",
        a.message
    );
}

/// `Notes/` の会議の準備と、それを指す2件のノート(と、指さない1件)。印に表を書かないので、表はフォルダから決まる。
fn meeting(tag: &str, setting: &str, linked: bool) -> (Tmp, App) {
    let to = if linked {
        "[[会議の準備]]"
    } else {
        "[[ほか]]"
    };
    let memo = format!("---\nproject: \"{to}\"\n---\n");
    boot(
        tag,
        &format!("[edit]\n{setting}\n"),
        &[
            ("Notes/会議の準備.md", "---\nstatus: todo\n---\n"),
            ("Notes/メモ.md", memo.as_str()),
            ("Notes/日記.md", memo.as_str()),
            ("Notes/ほか.md", "---\nstatus: todo\n---\n"),
        ],
    )
}

#[test]
fn test_rename_links_ask_yes_stages_the_rewrites() {
    // [utest -> req~cell-edit.rename-note-links-setting~1] ask は件数 2 を見せて問い、y で2件の書き換えをためる。
    let (t, mut a) = meeting("rl-ask-y", "rename_links = \"ask\"", true);
    rename(&mut a, "Notes/会議の準備.md", "会議の準備(10月)");
    assert!(
        t.notes().join("Notes/会議の準備(10月).md").exists(),
        "{:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Palette, "問いが出ない: {:?}", a.message);
    let ask = super::native_io::ask_lead(&a);
    assert!(ask.contains('2'), "{ask}");
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        value(&a, "Notes/メモ.md", "project"),
        s("[[会議の準備(10月)]]")
    );
    assert_eq!(
        value(&a, "Notes/日記.md", "project"),
        s("[[会議の準備(10月)]]")
    );
}

#[test]
fn test_rename_links_ask_escape_keeps_the_links() {
    // [utest -> req~cell-edit.rename-note-links-setting~1] ask の問いに Esc なら、名前は変わり、書き換えはためない。
    let (t, mut a) = meeting("rl-ask-esc", "rename_links = \"ask\"", true);
    rename(&mut a, "Notes/会議の準備.md", "会議の準備(10月)");
    assert!(
        t.notes().join("Notes/会議の準備(10月).md").exists(),
        "{:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Palette, "問いが出ない: {:?}", a.message);
    press(&mut a, KeyCode::Esc);
    assert!(a.changes.rows().is_empty());
    assert_eq!(value(&a, "Notes/メモ.md", "project"), s("[[会議の準備]]"));
    assert_eq!(value(&a, "Notes/日記.md", "project"), s("[[会議の準備]]"));
}

#[test]
fn test_rename_links_ask_without_links_does_not_ask() {
    // [utest -> req~cell-edit.rename-note-links-setting~1] ask でも、指すリンクが無ければ問わずに名前を変える。
    let (t, mut a) = meeting("rl-ask-none", "rename_links = \"ask\"", false);
    rename(&mut a, "Notes/会議の準備.md", "会議の準備(10月)");
    assert!(
        t.notes().join("Notes/会議の準備(10月).md").exists(),
        "{:?}",
        a.message
    );
    assert_ne!(a.mode, Mode::Palette);
}

#[test]
fn test_rename_links_never_keeps_the_links() {
    // [utest -> req~cell-edit.rename-note-links-setting~1] never は名前だけ変え、書き換えはためない。
    let (t, mut a) = meeting("rl-never", "rename_links = \"never\"", true);
    rename(&mut a, "Notes/会議の準備.md", "会議の準備(10月)");
    assert!(
        t.notes().join("Notes/会議の準備(10月).md").exists(),
        "{:?}",
        a.message
    );
    assert_ne!(a.mode, Mode::Palette);
    assert!(a.changes.rows().is_empty());
    assert_eq!(value(&a, "Notes/メモ.md", "project"), s("[[会議の準備]]"));
}

#[test]
fn test_rename_links_unknown_value_warns_and_rewrites() {
    // [utest -> req~cell-edit.rename-note-links-setting~1] 知らない値は edit.rename_links を言う警告を出し、always と同じく書き換えをためる。
    let (_t, mut a) = meeting("rl-bad", "rename_links = \"sometimes\"", true);
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("edit.rename_links") && m.contains("\"never\""),
        "{m}"
    );
    rename(&mut a, "Notes/会議の準備.md", "会議の準備(10月)");
    assert_eq!(
        value(&a, "Notes/メモ.md", "project"),
        s("[[会議の準備(10月)]]")
    );
    assert_eq!(
        value(&a, "Notes/日記.md", "project"),
        s("[[会議の準備(10月)]]")
    );
}
