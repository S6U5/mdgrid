//! 見本 examples/studio(架空の制作会社 Northwind Studio)のつながりを確かめる試験。
//! REL-1・REL-5・REL-6・WS-7。仕様: specs/relations/spec.md、specs/workspace/spec.md。
//!
//! 見本は読むだけ(書かない)。リンクを足したり名前を変えたりして行き先が無くなったら、ここで気づく。

use mdgrid::frontmatter;
use mdgrid::relations::{self, backlinks, edges, file_stem, parse, resolve, Edge, Notes, Table};
use mdgrid::workspace;
use std::path::{Path, PathBuf};

fn studio() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/studio")
}

/// 印(.mdgrid/workspace.toml)に書いた表。
fn tables() -> Vec<Table> {
    let (ws, warns) = workspace::read_marker(&studio());
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(ws.name, "Northwind Studio");
    ws.tables
        .iter()
        .map(|t| Table::new(&t.name, &t.path))
        .collect()
}

fn edge(from: &str, column: &str, to: &str, many: bool) -> Edge {
    Edge {
        from: from.into(),
        column: column.into(),
        to: to.into(),
        many,
    }
}

#[test]
fn test_example_studio_marker_tables() {
    // [WS-7] 印に書いた5つの表。Tasks は .base(ビューを持つ)、ほかはフォルダ。
    let (ws, _) = workspace::read_marker(&studio());
    let names: Vec<&str> = ws.tables.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        ["Tasks", "Projects", "Clients", "People", "Meetings"]
    );
    assert!(ws.tables[0].path.ends_with("Tasks.base"));
}

#[test]
fn test_example_studio_edges() {
    // [REL-6] 設定なしで見つかるつながり。リストの列(team・attendees・depends_on)は多対多。
    let mut got = edges(&tables());
    got.sort_by(|a, b| (&a.from, &a.column).cmp(&(&b.from, &b.column)));
    assert_eq!(
        got,
        vec![
            edge("Clients", "account_manager", "People", false),
            edge("Meetings", "attendees", "People", true),
            edge("Meetings", "project", "Projects", false),
            edge("Projects", "client", "Clients", false),
            edge("Projects", "lead", "People", false),
            edge("Projects", "team", "People", true),
            edge("Tasks", "depends_on", "Tasks", true),
            edge("Tasks", "owner", "People", false),
            edge("Tasks", "project", "Projects", false),
        ]
    );
}

#[test]
fn test_example_studio_every_link_resolves() {
    // [REL-1] 見本のリンクは全部、行き先のノートがある(REL-10 の ? の印が出ない)。
    let root = studio();
    let notes = Notes::scan(&root);
    let all = [&notes];
    let mut links = 0;
    for rel in &notes.rels {
        let path = root.join(rel);
        let fm = frontmatter::parse(&std::fs::read(&path).unwrap())
            .unwrap_or_else(|e| panic!("{rel}: {e:?}"));
        for e in fm.entries {
            for s in relations::strings(&e.value) {
                let Some(l) = parse(s) else { continue };
                assert!(
                    resolve(&l, &path, &all).is_some(),
                    "{rel}: {}: {s} has no note",
                    e.key
                );
                links += 1;
            }
        }
    }
    assert!(links >= 80, "only {links} links");
}

#[test]
fn test_example_studio_linked_rows_of_a_project() {
    // [REL-5] 案件を指す行: タスクの project と、会議の project。
    let target = studio().join("Projects/Clinic Booking App.md");
    let mut got: Vec<(String, Option<String>, String)> = backlinks(&target, &tables())
        .into_iter()
        .map(|b| (file_stem(&b.note), b.table, b.column))
        .collect();
    got.sort();
    let from = |t: &str| {
        got.iter()
            .filter(|(_, tb, _)| tb.as_deref() == Some(t))
            .count()
    };
    assert_eq!(from("Tasks"), 7, "{got:?}");
    assert_eq!(from("Meetings"), 2, "{got:?}");
    assert!(got.iter().all(|(_, _, c)| c == "project"), "{got:?}");
}
