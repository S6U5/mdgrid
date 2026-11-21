//! ページ(stack.page-store・stack.page-save-marker)の読み書きの試験。期待値は specs/requirements/stack.md の
//! 受け入れの例から取った。

use super::*;
use crate::workspace::{self, PageHome};
use std::path::{Path, PathBuf};

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-page-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Tmp(std::fs::canonicalize(&dir).unwrap_or(dir))
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// 印のあるワークスペース(tasks・projects のフォルダの表)。`extra` は印の末尾に足す文。
fn marker_ws(tag: &str, extra: &str) -> Tmp {
    let t = Tmp::new(tag);
    t.write("tasks/a.md", "---\nstatus: todo\n---\n");
    t.write("projects/p.md", "---\nstatus: doing\n---\n");
    t.write(
        ".mdgrid/workspace.toml",
        &format!(
            "name = \"work\"\n[[table]]\nname = \"tasks\"\npath = \"tasks\"\n[[table]]\nname = \"projects\"\npath = \"projects\"\n{extra}"
        ),
    );
    t
}

#[test]
fn test_page_read_skips_unknown_table_with_warning() {
    // [utest -> req~stack.page-store~1] 印に `[[page]]` `name = "朝"` と、無い表の名前を1つ含む表の並び → 警告が
    // 出て(ファイル: 道筋: 理由)、ページは残りの表で読める。
    let t = marker_ws(
        "store",
        "\n[[page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\", height = 2 },\n  { table = \"nothing\" },\n  { table = \"projects\" },\n]\n",
    );
    let (pages, warns) = workspace::marker_pages(t.path());
    assert_eq!(
        pages,
        vec![Page {
            name: "朝".into(),
            tables: vec![
                PageTable {
                    table: "tasks".into(),
                    view: Some("今日".into()),
                    height: 2,
                },
                PageTable {
                    table: "projects".into(),
                    view: None,
                    height: 1,
                },
            ],
        }]
    );
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(
        warns[0].starts_with(".mdgrid/workspace.toml: page[1].tables[2].table: "),
        "{warns:?}"
    );
    assert!(warns[0].contains("nothing"), "{warns:?}");
}

#[test]
fn test_page_read_bad_rows_are_warned_and_skipped() {
    // [utest -> req~stack.page-store~1] 読めない行(名前の無いページ・table の無い表・表の無いページ・知らない項目・
    // 高さの誤り)は、ファイル: 道筋: 理由の形で警告して飛ばす。
    let v: toml::Table = r#"
[[page]]
tables = [{ table = "tasks" }]

[[page]]
name = "a"
tables = [{ view = "x" }]

[[page]]
name = "b"
tables = [{ table = "tasks", height = 0, color = "red" }]
note = 1
"#
    .parse()
    .unwrap();
    let mut warns = Vec::new();
    let pages = read(
        v.get(KEY),
        &names(&["tasks"]),
        ("f.toml", "page"),
        &mut warns,
    );
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].name, "b");
    assert_eq!(pages[0].tables[0].height, 1);
    let at = |p: &str| {
        warns
            .iter()
            .any(|w| w.starts_with(&format!("f.toml: {p}: ")))
    };
    assert!(at("page[1]"), "{warns:?}");
    assert!(at("page[2].tables[1]"), "{warns:?}");
    assert!(at("page[2]"), "{warns:?}");
    assert!(at("page[3].tables[1].height"), "{warns:?}");
    assert!(at("page[3].tables[1].color"), "{warns:?}");
    assert!(at("page[3].note"), "{warns:?}");
}

#[test]
fn test_page_text_reads_back() {
    // [utest -> req~stack.page-store~1] 画面が書く形(`name` と `table`・`view`・`height` の並び)は、手で書いた
    // ものと同じに読める。
    let page = Page {
        name: "朝".into(),
        tables: vec![
            PageTable {
                table: "tasks".into(),
                view: Some("今日".into()),
                height: 1,
            },
            PageTable {
                table: "projects".into(),
                view: Some("進行中".into()),
                height: 1,
            },
        ],
    };
    let s = text(&page, "[[page]]");
    assert_eq!(
        s,
        "[[page]]\nname = \"朝\"\ntables = [\n  { table = \"tasks\", view = \"今日\", height = 1 },\n  { table = \"projects\", view = \"進行中\", height = 1 },\n]\n"
    );
    let v: toml::Table = s.parse().unwrap();
    let mut warns = Vec::new();
    let back = read(
        v.get(KEY),
        &names(&["tasks", "projects"]),
        ("f", "page"),
        &mut warns,
    );
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(back, vec![page]);
    assert_eq!(ratio(&[1, 1]), vec![1, 1]);
    assert_eq!(ratio(&[8, 4]), vec![2, 1]);
    assert_eq!(ratio(&[7, 5]), vec![7, 5]);
}

#[test]
fn test_page_save_marker_keeps_every_other_byte() {
    // [utest -> req~stack.page-save-marker~1] コメントと空行を含む印でページ「朝」を残す → 印の差分は足した
    // `[[page]]` の区切りだけ(前の文は1字も変わらず、その後ろに足す)。
    let before = "# 手で書いた印\nname = \"work\"   # 名前\n\n[[table]]\nname = \"tasks\"\npath = \"tasks\"\n\n# プロジェクト\n[[table]]\nname = \"projects\"\npath = \"projects\"\n";
    let t = Tmp::new("save-marker");
    t.write("tasks/a.md", "---\nstatus: todo\n---\n");
    t.write("projects/p.md", "---\nstatus: doing\n---\n");
    t.write(".mdgrid/workspace.toml", before);
    let page = Page {
        name: "朝".into(),
        tables: vec![
            PageTable {
                table: "tasks".into(),
                view: Some("今日".into()),
                height: 1,
            },
            PageTable {
                table: "projects".into(),
                view: None,
                height: 1,
            },
        ],
    };
    workspace::save_page(&PageHome::Marker(t.path().to_path_buf()), &page).unwrap();
    let after = std::fs::read_to_string(t.path().join(".mdgrid/workspace.toml")).unwrap();
    assert!(after.starts_with(before), "{after}");
    assert_eq!(
        &after[before.len()..],
        format!("\n{}", text(&page, "[[page]]"))
    );
    let (pages, warns) = workspace::marker_pages(t.path());
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(pages, vec![page.clone()]);
    // 同じ名前でもう一度残すと、その区画だけを置き換える(足さない)。
    let mut again = page;
    again.tables.reverse();
    workspace::save_page(&PageHome::Marker(t.path().to_path_buf()), &again).unwrap();
    let third = std::fs::read_to_string(t.path().join(".mdgrid/workspace.toml")).unwrap();
    assert!(third.starts_with(before), "{third}");
    assert_eq!(workspace::marker_pages(t.path()).0, vec![again]);
}

#[test]
fn test_page_save_app_workspace() {
    // [stack.page-save] workspaces.toml の work に `[[workspace.page]]`・`name = "朝"`・2つの表と
    // ビュー。ほかのワークスペースと表とコメントはそのまま。
    let t = Tmp::new("save-app");
    t.write("notes/tasks/a.md", "---\nstatus: todo\n---\n");
    t.write("notes/projects/p.md", "---\nstatus: doing\n---\n");
    let dir = t.path().join("config");
    let tasks = t.path().join("notes/tasks");
    let projects = t.path().join("notes/projects");
    let before = format!(
        "# 手で書いた\n[[workspace]]\nname = \"work\"\n\n[[workspace.table]]\nname = \"tasks\"\npath = '{}'\n\n[[workspace.table]]\nname = \"projects\"\npath = '{}'\n\n[[workspace]]\nname = \"home\"\n",
        tasks.display(),
        projects.display()
    );
    t.write("config/workspaces.toml", &before);
    let page = Page {
        name: "朝".into(),
        tables: vec![
            PageTable {
                table: "tasks".into(),
                view: Some("今日".into()),
                height: 1,
            },
            PageTable {
                table: "projects".into(),
                view: Some("進行中".into()),
                height: 1,
            },
        ],
    };
    let home = PageHome::App {
        dir: dir.clone(),
        ws: "work".into(),
    };
    workspace::save_page(&home, &page).unwrap();
    let after = std::fs::read_to_string(dir.join("workspaces.toml")).unwrap();
    assert!(
        after.contains("[[workspace.page]]\nname = \"朝\"\n"),
        "{after}"
    );
    assert!(after.starts_with("# 手で書いた\n"), "{after}");
    assert!(after.contains("name = \"home\""), "{after}");
    let (pages, warns) = workspace::load_pages(&home);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(pages, vec![page]);
    // ワークスペースの並びは前のまま(ページの区画で表が増えたり消えたりしない)。
    let (list, warns) = workspace::load(&dir);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].tables.len(), 2);
    // 表を足す書き直し(WS-2)でも、ページは残る。
    workspace::add(
        &dir,
        "work",
        workspace::WsTable {
            name: "more".into(),
            path: t.path().join("notes"),
            ..workspace::WsTable::default()
        },
    )
    .unwrap();
    assert_eq!(workspace::load_pages(&home).0.len(), 1);
}
