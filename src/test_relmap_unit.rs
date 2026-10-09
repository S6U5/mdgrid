//! 関係マップの核(REL-7): 表の形・つながりの組の読み取りと、盤への並べ方。

use super::*;

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "mdgrid-relmap-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(std::fs::canonicalize(&d).unwrap())
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

fn workspace(name: &str) -> (Tmp, Vec<Table>) {
    let t = Tmp::new(name);
    t.write(
        "projects/mdgrid.md",
        "---\nstatus: active\nlead: \"[[Alice]]\"\n---\n",
    );
    t.write(
        "projects/Website.md",
        "---\nstatus: idea\nlead: \"[[Bob]]\"\n---\n",
    );
    t.write("members/Alice.md", "---\nrole: dev\n---\n");
    t.write("members/Bob.md", "---\nrole: pm\n---\n");
    t.write(
        "tasks/a.md",
        "---\nstatus: todo\nproject: \"[[mdgrid]]\"\nassignee: \"[[Alice]]\"\nrelated:\n  - \"[[Website]]\"\n---\n",
    );
    t.write(
        "tasks/b.md",
        "---\nstatus: done\nproject: \"[[mdgrid]]\"\nassignee: \"[[Bob]]\"\n---\n",
    );
    t.write(
        "tasks/c.md",
        "---\nstatus: todo\nproject: \"[[Website]]\"\n---\n",
    );
    let tables = vec![
        Table::new("Tasks", &t.0.join("tasks")),
        Table::new("Projects", &t.0.join("projects")),
        Table::new("Members", &t.0.join("members")),
    ];
    (t, tables)
}

#[test]
fn test_rel_7_read_tables_and_links() {
    // [REL-7] 表の形(行の数・多い順の列)と、つながりとリンクの組。
    let (_t, tables) = workspace("read");
    let (infos, links) = read(&tables);
    let tasks = &infos[0];
    assert_eq!(tasks.rows, 3);
    assert_eq!(
        tasks.columns[..2],
        ["status".to_string(), "project".to_string()]
    );
    let find = |from: &str, col: &str| {
        links
            .iter()
            .find(|l| l.from == from && l.column == col)
            .unwrap()
    };
    let p = find("Tasks", "project");
    assert_eq!(
        (p.to.as_str(), p.many, p.pairs.len()),
        ("Projects", false, 3)
    );
    assert!(find("Tasks", "related").many);
    assert_eq!(find("Projects", "lead").to, "Members");
    assert_eq!(
        top_targets(p, 5),
        vec![("mdgrid".to_string(), 2), ("Website".to_string(), 1)]
    );
    assert_eq!(link_text(p), "Tasks.project → Projects (N:1)");
}

/// 盤の文字(幅2の右の空の印は除く)。
fn text(m: &Map) -> String {
    (0..m.h)
        .map(|y| m.line(y).replace('\u{0}', ""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_rel_7_layout_connects_boxes() {
    // [REL-7] 層は元が左・行き先が右。箱は重ならず、矢印は行き先の箱の題の行の左に入る。数の札が付く。
    let (_t, tables) = workspace("layout");
    let (infos, links) = read(&tables);
    let m = layout(&infos, &links, false);
    let s = text(&m);
    let (t, p, mb) = (m.boxes[0], m.boxes[1], m.boxes[2]);
    assert!(t.x + t.w < p.x, "Tasks の右に Projects:\n{s}");
    assert!(p.x + p.w < mb.x, "Projects の右に Members:\n{s}");
    // 箱は重ならない。
    for (i, a) in m.boxes.iter().enumerate() {
        for b in m.boxes.iter().skip(i + 1) {
            let apart =
                a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
            assert!(apart, "{a:?} {b:?}\n{s}");
        }
    }
    // Projects の題の行の左の縁のすぐ左に矢印。
    assert_eq!(m.get(p.x - 1, p.y + 1).ch, '▶', "{s}");
    assert!(m.line(p.y + 1).contains("Projects (2)"), "{s}");
    // つながりの列に → 行き先、数の札 N と 1。
    assert!(s.contains("project → Projects"), "{s}");
    assert!(s.contains("lead → Members"), "{s}");
    let labels: Vec<char> = m
        .cells
        .iter()
        .flatten()
        .filter(|c| c.role == Role::Label)
        .map(|c| c.ch)
        .collect();
    assert!(labels.contains(&'N') && labels.contains(&'1'), "{s}");
    // 線と矢印はどのつながりかを持つ。
    assert!(m
        .cells
        .iter()
        .flatten()
        .filter(|c| matches!(c.role, Role::Line | Role::Arrow))
        .all(|c| c.link.is_some()));
}

#[test]
fn test_rel_7_ascii_lines() {
    // [REL-7][SR-32] ASCII の盤は罫線を使わない。
    let (_t, tables) = workspace("ascii");
    let (infos, links) = read(&tables);
    let s = text(&layout(&infos, &links, true));
    assert!(!s.chars().any(|c| "╭╮╰╯─│├┤┼┐┘└┌▶◀→".contains(c)), "{s}");
    assert!(s.contains("+-") && s.contains("->"), "{s}");
}

#[test]
fn test_rel_7_back_and_self_links_do_not_panic() {
    // [REL-7] 輪(行き先が左か同じ層)と自分を指すつながりでも並べられる。
    let t = Tmp::new("cycle");
    t.write("a/x.md", "---\nnext: \"[[y]]\"\n---\n");
    t.write("b/y.md", "---\nback: \"[[x]]\"\nself: \"[[z]]\"\n---\n");
    t.write("b/z.md", "---\n---\n");
    let tables = vec![
        Table::new("A", &t.0.join("a")),
        Table::new("B", &t.0.join("b")),
    ];
    let (infos, links) = read(&tables);
    assert_eq!(links.len(), 3, "{links:?}");
    let m = layout(&infos, &links, false);
    let s = text(&m);
    assert!(s.contains('▶') || s.contains('◀'), "{s}");
}

#[test]
fn test_rel_7_empty_workspace() {
    // [REL-7] 表もつながりも無ければ空の盤。
    let m = layout(&[], &[], false);
    assert!(m.boxes.is_empty());
}

#[test]
#[ignore]
fn show_sample() {
    let (_t, tables) = workspace("show");
    let (infos, links) = read(&tables);
    println!("{}", text(&layout(&infos, &links, false)));
}

#[test]
fn test_rel_7_control_chars_are_not_drawn() {
    // [REL-7][SR-10] ノートの列の名前に制御文字があっても、盤には `?` で描く(端末を操作させない)。
    let infos = vec![TableInfo {
        name: "T\u{1b}[31m".into(),
        dir: PathBuf::from("/t"),
        rows: 1,
        columns: vec!["bad\u{1b}]0;x\u{7}".into()],
    }];
    let m = layout(&infos, &[], false);
    assert!(m.cells.iter().flatten().all(|c| !c.ch.is_control()));
    let l = Link {
        from: "a\u{1b}".into(),
        column: "c".into(),
        to: "b".into(),
        many: false,
        pairs: Vec::new(),
    };
    assert!(!link_text(&l).chars().any(|c| c.is_control()));
}
