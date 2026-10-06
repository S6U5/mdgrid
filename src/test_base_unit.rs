use super::*;
use crate::source::markdown::Markdown;
use std::path::PathBuf;

struct Tmp(PathBuf);
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// x.md(p: 2、s: b)、y.md(p なし、s: a)、z.md(p: 1、s: a)、w.md(p: "high"、s: b) の保管庫。
fn vault(name: &str) -> (Tmp, Markdown) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-base-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".obsidian")).unwrap();
    std::fs::write(
        d.join(".obsidian/types.json"),
        r#"{"types": {"p": "number"}}"#,
    )
    .unwrap();
    std::fs::write(d.join("x.md"), "---\np: 2\ns: b\n---\n").unwrap();
    std::fs::write(d.join("y.md"), "---\ns: a\n---\n").unwrap();
    std::fs::write(d.join("z.md"), "---\np: 1\ns: a\n---\n").unwrap();
    std::fs::write(d.join("w.md"), "---\np: high\ns: b\n---\n").unwrap();
    std::fs::write(d.join("t.base"), "views: []\n").unwrap();
    let mut md = Markdown::open_vault(&d.join("t.base")).unwrap();
    while !md.load(100).done {}
    (Tmp(d), md)
}

fn build(b: &Base, view: usize, src: &dyn Source) -> Result<Grid, String> {
    let prop = |r: &RowId, c: &str| src.get(r, c).value;
    b.build(view, src, &prop, 0, 0)
}

fn cell(b: &Base, src: &dyn Source, label: &str, col: &str) -> Shown {
    let prop = |r: &RowId, c: &str| src.get(r, c).value;
    let row = src
        .rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap();
    b.cell(src, &prop, &row, col, 0, 0)
}

fn labels(src: &dyn Source, g: &Grid) -> Vec<String> {
    g.rows.iter().map(|r| src.label(r)).collect()
}

#[test]
fn parse_errors_and_empty() {
    assert!(parse("views: [").is_err());
    assert!(parse("- a\n- b\n").is_err());
    assert!(parse("").unwrap().views.is_empty());
    assert!(parse("# only a comment\n").unwrap().views.is_empty());
}

#[test]
fn sort_puts_mismatched_then_empty_last_in_both_directions() {
    let (_t, md) = vault("sort");
    let b = parse(
        "views:\n  - type: table\n    sort:\n      - property: p\n        direction: DESC\n  - type: table\n    sort:\n      - property: p\n",
    )
    .unwrap();
    assert_eq!(
        labels(&md, &build(&b, 0, &md).unwrap()),
        vec!["x.md", "z.md", "w.md", "y.md"]
    );
    assert_eq!(
        labels(&md, &build(&b, 1, &md).unwrap()),
        vec!["z.md", "x.md", "w.md", "y.md"]
    );
}

#[test]
fn order_missing_gives_file_name_and_note_keys() {
    let (_t, md) = vault("noorder");
    let b = parse("views:\n  - type: table\n").unwrap();
    let g = build(&b, 0, &md).unwrap();
    let ids: Vec<&str> = g.columns.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids[0], "file.name");
    assert!(ids.contains(&"p") && ids.contains(&"s"));
}

#[test]
fn limit_trims_groups() {
    let (_t, md) = vault("limit");
    let b = parse(
        "views:\n  - type: table\n    groupBy:\n      property: s\n    sort:\n      - property: file.name\n    limit: 3\n",
    )
    .unwrap();
    let g = build(&b, 0, &md).unwrap();
    assert_eq!(labels(&md, &g), vec!["y.md", "z.md", "w.md"]);
    assert_eq!(
        g.groups,
        vec![("a".to_string(), 0..2), ("b".to_string(), 2..3)]
    );
}

#[test]
fn group_of_empty_values_is_last_with_heading() {
    let (_t, md) = vault("group-empty");
    let b =
        parse("views:\n  - type: table\n    groupBy:\n      property: p\n      direction: DESC\n")
            .unwrap();
    let g = build(&b, 0, &md).unwrap();
    let heads: Vec<&str> = g.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, vec!["2", "1", "high", EMPTY_HEADING]);
}

#[test]
fn formula_cycle_is_unsupported_and_stops() {
    let (_t, md) = vault("cycle");
    let b = parse(
        "formulas:\n  a: 'formula.b'\n  b: 'formula.a'\n  c: 'formula.c + 1'\n  d: 'p + 1'\nviews:\n  - type: table\n    order: [formula.a, formula.c, formula.d]\n    sort:\n      - property: formula.a\n  - type: table\n    filters:\n      not:\n        - 'formula.c > 1'\n",
    )
    .unwrap();
    let g = build(&b, 0, &md).unwrap();
    assert_eq!(g.rows.len(), 4);
    assert!(g.notes.iter().any(|n| n.contains("循環")), "{:?}", g.notes);
    assert!(
        matches!(cell(&b, &md, "x.md", "formula.a"), Shown::Unsupported(r) if r.contains("循環"))
    );
    assert!(
        matches!(cell(&b, &md, "x.md", "formula.c"), Shown::Unsupported(r) if r.contains("循環"))
    );
    assert_eq!(
        cell(&b, &md, "x.md", "formula.d"),
        Shown::Computed(Val::Num(3.0))
    );
    // 循環する formula を使う filters → ビューは開かない(not で全行を残さない)。
    let e = build(&b, 1, &md).unwrap_err();
    assert!(e.contains("formula.c が循環"), "{e}");
}

#[test]
fn notes_follow_formula_references() {
    let (_t, md) = vault("transitive");
    let b = parse(
        "formulas:\n  a: 'formula.missing + 1'\n  b: 'formula[\"a\"] * 2'\n  s: '\"formula.missing\"'\nviews:\n  - type: table\n    order: [formula.b, formula.s]\n    sort:\n      - property: formula.a\n",
    )
    .unwrap();
    let g = build(&b, 0, &md).unwrap();
    assert!(
        g.notes
            .iter()
            .any(|n| n.starts_with("formula.b") && n.contains("formula.missing")),
        "{:?}",
        g.notes
    );
    assert!(
        g.notes
            .iter()
            .any(|n| n.starts_with("formula.a で並べられない")),
        "{:?}",
        g.notes
    );
    assert!(
        !g.notes.iter().any(|n| n.starts_with("formula.s")),
        "string literal is not a reference: {:?}",
        g.notes
    );
    assert_eq!(
        cell(&b, &md, "x.md", "formula.s"),
        Shown::Computed(Val::Str("formula.missing".into()))
    );
    assert_eq!(
        formula_refs("formula.a + note.formula + formula [ 'b c' ] + 'formula.x'"),
        vec!["a", "b c"]
    );
}

#[test]
fn filter_leaves_use_truthiness() {
    let (_t, md) = vault("truthy");
    // p: 2・なし・1・"high"。数の値そのもの・文字列・Null で判定する。
    let b = parse(
        "views:\n  - type: table\n    filters: 'p - 1'\n  - type: table\n    filters: 's'\n  - type: table\n    filters: 'p'\n",
    )
    .unwrap();
    let mut g = labels(&md, &build(&b, 0, &md).unwrap());
    g.sort();
    assert_eq!(g, vec!["x.md"], "p - 1: 1 is true, 0 and Null are false");
    assert_eq!(build(&b, 1, &md).unwrap().rows.len(), 4);
    let mut g = labels(&md, &build(&b, 2, &md).unwrap());
    g.sort();
    assert_eq!(g, vec!["w.md", "x.md", "z.md"]);
}

#[test]
fn alias_expansion_is_limited() {
    // 別名を10個ずつ重ねた .base(展開すると 10^7 ノード)は読む前に Err。
    let mut text = String::from("a0: &a0 [x, x, x, x, x, x, x, x, x, x]\n");
    for i in 1..8 {
        let refs = vec![format!("*a{}", i - 1); 10].join(", ");
        text.push_str(&format!("a{i}: &a{i} [{refs}]\n"));
    }
    let t = std::time::Instant::now();
    let e = parse(&text).unwrap_err();
    assert!(e.contains("別名"), "{e}");
    assert!(t.elapsed().as_secs() < 1);
    // 上限の内のアンカーと別名は使える。
    let b =
        parse("base: &v\n  type: table\n  order: [file.name]\nviews:\n  - *v\n  - *v\n").unwrap();
    assert_eq!(b.views.len(), 2);
    assert_eq!(b.views[1].order, vec!["file.name"]);
}

#[test]
fn formula_using_unsupported_formula_is_unsupported() {
    let (_t, md) = vault("propagate");
    let b = parse(
        "formulas:\n  bad: 'p.toFixed(1)'\n  uses: 'formula.bad'\n  ok: 'p + 1'\nviews:\n  - type: table\n    order: [formula.uses, formula.missing, file.embeds]\n  - type: table\n    filters: 'formula.bad == 1'\n",
    )
    .unwrap();
    let g = build(&b, 0, &md).unwrap();
    assert!(
        g.notes.iter().any(|n| n.contains("formula.missing")),
        "{:?}",
        g.notes
    );
    assert!(
        g.notes.iter().any(|n| n.contains("embeds")),
        "{:?}",
        g.notes
    );
    assert!(
        matches!(cell(&b, &md, "x.md", "formula.uses"), Shown::Unsupported(r) if r.contains("toFixed"))
    );
    assert!(matches!(
        cell(&b, &md, "x.md", "formula.missing"),
        Shown::Unsupported(_)
    ));
    assert_eq!(
        cell(&b, &md, "x.md", "formula.ok"),
        Shown::Computed(Val::Num(3.0))
    );
    // filters が評価できない formula を使う → ビューは開かない。
    let e = build(&b, 1, &md).unwrap_err();
    assert!(e.contains("toFixed"), "{e}");
}

#[test]
fn filter_forms() {
    let (_t, md) = vault("filters");
    let b = parse(
        "views:\n  - type: table\n    filters: 's == \"a\"'\n  - type: table\n    filters:\n      not: 's == \"a\"'\n  - type: table\n    filters:\n      xor: [a]\n  - type: table\n    filters:\n      or: []\n  - name: no type\n",
    )
    .unwrap();
    let mut g = labels(&md, &build(&b, 0, &md).unwrap());
    g.sort();
    assert_eq!(g, vec!["y.md", "z.md"]);
    let mut g = labels(&md, &build(&b, 1, &md).unwrap());
    g.sort();
    assert_eq!(g, vec!["w.md", "x.md"]);
    assert!(build(&b, 2, &md).unwrap_err().contains("xor"));
    assert_eq!(build(&b, 3, &md).unwrap().rows.len(), 4);
    assert!(build(&b, 4, &md).is_err());
    assert!(build(&b, 9, &md).is_err());
}

#[test]
fn note_prefix_and_display_name() {
    let (_t, md) = vault("prefix");
    let b = parse(
        "properties:\n  note.s:\n    displayName: S\nviews:\n  - type: table\n    order: [note.s]\n",
    )
    .unwrap();
    let g = build(&b, 0, &md).unwrap();
    assert_eq!(
        g.columns,
        vec![Column {
            id: "s".into(),
            title: "S".into()
        }]
    );
    assert!(
        matches!(cell(&b, &md, "y.md", "note.s"), Shown::Prop(c) if c.value == Some(Value::Str("a".into())))
    );
}
