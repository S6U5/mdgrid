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
    // [utest -> req~relations.map~2] 表の形(行の数・多い順の列)と、つながりとリンクの組。
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
    // [utest -> req~relations.map~2] 層は元が左・行き先が右。箱は重ならず、矢印は行き先の箱の題の行の左に入る。数の札が付く。
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
    // [utest -> req~relations.map~2, req~screen.frames~1] ASCII の盤は罫線を使わない。
    let (_t, tables) = workspace("ascii");
    let (infos, links) = read(&tables);
    let s = text(&layout(&infos, &links, true));
    assert!(!s.chars().any(|c| "╭╮╰╯─│├┤┼┐┘└┌▶◀→".contains(c)), "{s}");
    assert!(s.contains("+-") && s.contains("->"), "{s}");
}

#[test]
fn test_rel_7_back_and_self_links_do_not_panic() {
    // [utest -> req~relations.map~2] 輪(行き先が左か同じ層)と自分を指すつながりでも並べられる。
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
    // [utest -> req~relations.map~2] 表もつながりも無ければ空の盤。
    let m = layout(&[], &[], false);
    assert!(m.boxes.is_empty());
}

/// tasks の routine が routines のノートを、parent が tasks のノートを指すワークスペース。
fn self_workspace(name: &str) -> (Tmp, Vec<Table>) {
    let t = Tmp::new(name);
    t.write("routines/weekly.md", "---\nevery: week\n---\n");
    t.write(
        "tasks/a.md",
        "---\ndue: 2026-11-01\nroutine: \"[[weekly]]\"\n---\n",
    );
    t.write(
        "tasks/b.md",
        "---\ndue: 2026-11-02\nroutine: \"[[weekly]]\"\nparent: \"[[a]]\"\n---\n",
    );
    t.write("tasks/c.md", "---\nparent: \"[[a]]\"\n---\n");
    let tables = vec![
        Table::new("Tasks", &t.0.join("tasks")),
        Table::new("Routines", &t.0.join("routines")),
    ];
    (t, tables)
}

/// つながり `k` の矢印の先のセル(盤の全部で1つだけのはず)。
fn arrows_of(m: &Map, k: usize) -> Vec<(usize, usize, char)> {
    let mut out = Vec::new();
    for (y, row) in m.cells.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            if c.role == Role::Arrow && c.link == Some(k) {
                out.push((x, y, c.ch));
            }
        }
    }
    out
}

#[test]
fn test_self_link_counts_as_second_link() {
    // [utest -> req~relations.map-self-links~1] routine が routines を、parent が tasks を指すと、つながりは tasks → routines と tasks → tasks の2つで、盤にも矢印が2本ある。
    let (_t, tables) = self_workspace("self_two");
    let (infos, links) = read(&tables);
    let mut got: Vec<(&str, &str, &str, bool)> = links
        .iter()
        .map(|l| (l.from.as_str(), l.column.as_str(), l.to.as_str(), l.many))
        .collect();
    got.sort_unstable();
    assert_eq!(
        got,
        vec![
            ("Tasks", "parent", "Tasks", false),
            ("Tasks", "routine", "Routines", false),
        ]
    );
    for ascii in [false, true] {
        let m = layout(&infos, &links, ascii);
        for (k, l) in links.iter().enumerate() {
            assert_eq!(
                arrows_of(&m, k).len(),
                1,
                "{}.{} の矢印(ascii {ascii}):\n{}",
                l.from,
                l.column,
                text(&m)
            );
        }
    }
}

#[test]
fn test_self_link_loops_back_to_own_box() {
    // [utest -> req~relations.map-self-links~1, req~relations.layout-readable~1] 同じ表を指す列は、その列の行の横から出て、同じ箱へ戻る矢印になる。行き先の側に多対一の 1。箱の文字は線で消えない。
    let (_t, tables) = self_workspace("self_loop");
    let (infos, links) = read(&tables);
    let k = links
        .iter()
        .position(|l| l.from == "Tasks" && l.to == "Tasks")
        .expect("tasks → tasks のつながり");
    for ascii in [false, true] {
        let m = layout(&infos, &links, ascii);
        let s = text(&m);
        let b = m.boxes[0];
        let (right, left) = if ascii { ('<', '>') } else { ('◀', '▶') };
        let arrows = arrows_of(&m, k);
        assert_eq!(arrows.len(), 1, "{s}");
        let (ax, ay, ach) = arrows[0];
        // 矢印は箱の縁のすぐ外から、箱の中を向く。
        let into_box = (ax == b.x + b.w && ach == right) || (ax + 1 == b.x && ach == left);
        assert!(into_box, "({ax}, {ay}) {ach}:\n{s}");
        assert!(b.y < ay && ay + 1 < b.y + b.h, "箱の行に入る:\n{s}");
        // 出る所は parent の行の、矢印と同じ側の縁のすぐ外で、元の側の札 N。
        let py = (b.y..b.y + b.h)
            .find(|&y| m.line(y).contains("parent"))
            .expect("parent の行");
        let sx = if ach == right { b.x + b.w } else { b.x - 1 };
        let start = m.cells[py][sx];
        assert_eq!(
            (start.ch, start.role, start.link),
            ('N', Role::Label, Some(k)),
            "{s}"
        );
        // 行き先の側の札は多対一の 1(矢印の手前)。
        let bx = if ach == right { ax + 1 } else { ax - 1 };
        assert_eq!(
            (m.cells[ay][bx].ch, m.cells[ay][bx].link),
            ('1', Some(k)),
            "{s}"
        );
        // 箱の文字は残る。
        assert!(m.line(b.y + 1).contains("Tasks (3)"), "{s}");
        assert!(m.line(py).contains("parent"), "{s}");
    }
}

#[test]
fn test_column_marks_by_kind() {
    // [utest -> req~relations.map-column-types~1] 箱の列の印: 数・日付・日時・チェック・リスト・札の列は見出しと同じ種類の印、リンクの列(つながりの元と、値の多くがリンクの列)はリンクの印。ただの文字の列は印なし。
    let t = Tmp::new("marks");
    t.write("routines/weekly.md", "---\nevery: week\n---\n");
    t.write(
        "tasks/a.md",
        "---\nn: 3\ndue: 2026-11-01\nat: 2026-11-01T10:00\ndone: true\ntags: [x, y]\nstatus: todo\ntitle: first one\nroutine: \"[[weekly]]\"\nref: \"[[nowhere]]\"\n---\n",
    );
    t.write(
        "tasks/b.md",
        "---\nn: 5\ndue: 2026-11-02\nat: 2026-11-02T11:00\ndone: false\ntags: [y]\nstatus: todo\ntitle: second one\nroutine: \"[[weekly]]\"\nparent: \"[[a]]\"\nref: \"[[nowhere]]\"\n---\n",
    );
    t.write(
        "tasks/c.md",
        "---\nstatus: done\ntitle: third one\nparent: \"[[a]]\"\n---\n",
    );
    let tables = vec![
        Table::new("Tasks", &t.0.join("tasks")),
        Table::new("Routines", &t.0.join("routines")),
    ];
    let (infos, links, marks) = read_marked(&tables);
    assert_eq!(infos.len(), marks.len());
    assert_eq!(links.len(), 2, "{links:?}");
    let mut tasks = marks[0].clone();
    tasks.sort();
    let mut want = vec![
        ("n".to_string(), Mark::Number),
        ("due".to_string(), Mark::Date),
        ("at".to_string(), Mark::DateTime),
        ("done".to_string(), Mark::Check),
        ("tags".to_string(), Mark::List),
        ("status".to_string(), Mark::Select),
        ("routine".to_string(), Mark::Link),
        ("parent".to_string(), Mark::Link),
        ("ref".to_string(), Mark::Link),
    ];
    want.sort();
    assert_eq!(tasks, want, "title はただの文字の列なので印なし");
    // routines の every は1行だけの文字(くり返しが無い)なので印なし。
    assert_eq!(marks[1], Vec::new());
}

#[test]
fn test_layout_puts_marks_before_column_names() {
    // [utest -> req~relations.map-column-types~1] 印は箱の列の名前の前(つながりの列も)に、空白を挟んで添える。
    let (_t, tables) = self_workspace("marks_layout");
    let (infos, links) = read(&tables);
    let mark = |t: usize, c: &str| -> Option<String> {
        (t == 0).then(|| match c {
            "due" => "D".to_string(),
            _ => "L".to_string(),
        })
    };
    let m = layout_with(&infos, &links, false, &mark);
    let s = text(&m);
    assert!(s.contains("D due"), "{s}");
    assert!(s.contains("L parent → Tasks"), "{s}");
    assert!(s.contains("L routine → Routines"), "{s}");
}

#[test]
#[ignore = "見本の盤を手で見るための出力(cargo test -- --ignored show_sample)"]
fn show_sample() {
    let (_t, tables) = workspace("show");
    let (infos, links) = read(&tables);
    println!("{}", text(&layout(&infos, &links, false)));
    // 自分へ戻る輪(relations.map-self-links)と、列の印(relations.map-column-types)。
    let (t, mut tables) = self_workspace("show_self");
    t.write(
        "routines/weekly.md",
        "---\nevery: week\nexample: \"[[a]]\"\n---\n",
    );
    t.write(
        "tasks/d.md",
        "---\nparent: \"[[a]]\"\nblocks:\n  - \"[[b]]\"\n  - \"[[c]]\"\n---\n",
    );
    tables.push(Table::new("Empty", &t.0.join("empty")));
    let (infos, links, marks) = read_marked(&tables);
    let mark = |i: usize, c: &str| {
        marks[i]
            .iter()
            .find(|(x, _)| x == c)
            .map(|(_, m)| format!("{m:?}").chars().next().unwrap().to_string())
    };
    println!("{}", text(&layout_with(&infos, &links, false, &mark)));
    println!("{}", text(&layout_with(&infos, &links, true, &mark)));
}

#[test]
fn test_rel_7_control_chars_are_not_drawn() {
    // [utest -> req~relations.map~2, req~cell-view.no-color-env~1, req~cell-view.no-color-term-dumb~1, req~cell-view.no-color-not-tty~1] ノートの列の名前に制御文字があっても、盤には `?` で描く(端末を操作させない)。
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
    assert!(!link_text(&l).chars().any(char::is_control));
}

/// 1つの印に、同じフォルダの中を `file.inFolder` で分けた表「タスク」と「ルーチン」を書いた台帳(スターター tasks-ai と同じ形)。
fn marker_ledger(name: &str) -> (Tmp, Vec<Table>) {
    let t = Tmp::new(name);
    t.write(
        ".mdgrid/workspace.toml",
        "open = \"タスク\"\n\n[[table]]\nname = \"タスク\"\n\n[table.base]\nfilters = { and = ['file.inFolder(\"Tasks\")'] }\n\n[[table]]\nname = \"ルーチン\"\n\n[table.base]\nfilters = { and = ['file.inFolder(\"Routines\")'] }\n",
    );
    t.write(
        "Tasks/請求書.md",
        "---\nstatus: todo\nroutine: \"[[毎月の請求]]\"\n---\n",
    );
    t.write("Tasks/掃除.md", "---\nstatus: done\n---\n");
    t.write(
        "Routines/毎月の請求.md",
        "---\nrecurrence: monthly 1\n---\n",
    );
    let tables = vec![
        Table::new("タスク", &t.0.join("タスク.base")),
        Table::new("ルーチン", &t.0.join("ルーチン.base")),
    ];
    (t, tables)
}

#[test]
fn test_marker_tables_split_by_in_folder() {
    // [utest -> req~relations.map~2] 同じフォルダの中を `file.inFolder` で分けた表は、それぞれ自分のフォルダのノートだけを数え、
    // タスクの routine はルーチンへのつながりになる(どのノートも最後の表に入るのではない)。
    let (_t, tables) = marker_ledger("inf1");
    let (infos, links) = read(&tables);
    let rows: Vec<(&str, usize)> = infos.iter().map(|i| (i.name.as_str(), i.rows)).collect();
    assert_eq!(rows, vec![("タスク", 2), ("ルーチン", 1)]);
    let edges: Vec<(&str, &str, &str)> = links
        .iter()
        .map(|l| (l.from.as_str(), l.column.as_str(), l.to.as_str()))
        .collect();
    assert_eq!(edges, vec![("タスク", "routine", "ルーチン")]);
}

#[test]
fn test_vault_base_files_split_by_in_folder() {
    // [utest -> req~relations.map~2] 保管庫(.obsidian のあるフォルダ)の根に置いた `.base` も、全体の絞り込みの
    // `file.inFolder` のフォルダのノートを、その表のノートとして数える。
    let t = Tmp::new("inf2");
    std::fs::create_dir_all(t.0.join(".obsidian")).unwrap();
    t.write(
        "Tasks.base",
        "filters:\n  and:\n    - file.inFolder(\"Tasks\")\n",
    );
    t.write(
        "People.base",
        "filters:\n  and:\n    - file.inFolder(\"People\")\n",
    );
    t.write("Tasks/a.md", "---\nowner: \"[[Alice]]\"\n---\n");
    t.write("People/Alice.md", "---\nrole: dev\n---\n");
    let tables = vec![
        Table::new("Tasks", &t.0.join("Tasks.base")),
        Table::new("People", &t.0.join("People.base")),
    ];
    let (infos, links) = read(&tables);
    let rows: Vec<(&str, usize)> = infos.iter().map(|i| (i.name.as_str(), i.rows)).collect();
    assert_eq!(rows, vec![("Tasks", 1), ("People", 1)]);
    assert_eq!(links.len(), 1, "{links:?}");
    assert_eq!(
        (links[0].from.as_str(), links[0].to.as_str()),
        ("Tasks", "People")
    );
}
