//! `.base` の table ビューの受け入れテスト(タスク 6)。
//! BV-3・BV-4・BV-5・BV-7・BV-8・BV-13・SC-8・CV-2(並べ替え)・BV-1(default_grid)。
//! 仕様: specs/base-view/spec.md、specs/scope/spec.md、specs/cell-view/spec.md。
//! 形: docs/design.md の「核の公開のインターフェース(残り: タスク 5・6・8)」の src/base.rs。
//!
//! 材料は一時フォルダの保管庫(`.obsidian/` を持つ)に数個のノートと `.base` を書き、
//! `Markdown::open_vault(&base_path)` で開いて読み込みを最後まで進める。

use mdgrid::base::{self, Base, Dir, Grid, Shown};
use mdgrid::expr::Val;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source, Value};
use std::path::{Path, PathBuf};

// ---- 一時フォルダ(tests/test_source.rs から写す) ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, bytes).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 材料 ----

/// 2026-01-01(1970-01-01 からの日数)。
const TODAY: i64 = 20454;
const NOW: i64 = TODAY * 86_400;

/// 保管庫に5つのノートを書く。
///
/// | note | tags  | status | due        | priority |
/// |------|-------|--------|------------|----------|
/// | a.md | a     | done   | 2026-03-01 | 2        |
/// | b.md | a, b  | todo   | 2026-01-15 | 1        |
/// | c.md | b     | done   | 2026-02-01 | 3        |
/// | d.md | a     | done   | someday    | 1        |
/// | e.md | a     | todo   | 2026-02-10 | 3        |
fn vault(name: &str) -> TempDir {
    let dir = TempDir::new(name);
    std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
    dir.write(
        ".obsidian/types.json",
        br#"{"types": {"due": "date", "priority": "number", "status": "text"}}"#,
    );
    dir.write(
        "a.md",
        b"---\ntags: [a]\nstatus: done\ndue: 2026-03-01\npriority: 2\n---\nA\n",
    );
    dir.write(
        "b.md",
        b"---\ntags: [a, b]\nstatus: todo\ndue: 2026-01-15\npriority: 1\n---\nB\n",
    );
    dir.write(
        "c.md",
        b"---\ntags: [b]\nstatus: done\ndue: 2026-02-01\npriority: 3\n---\nC\n",
    );
    dir.write(
        "d.md",
        b"---\ntags: [a]\nstatus: done\ndue: someday\npriority: 1\n---\nD\n",
    );
    dir.write(
        "e.md",
        b"---\ntags: [a]\nstatus: todo\ndue: 2026-02-10\npriority: 3\n---\nE\n",
    );
    dir
}

// ---- 助け ----

/// 読み込みを最後まで進める。
fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

/// `.base` を書いて、保管庫として開き、読み切る。(Markdown, Base, .base のパス)
fn open_base(dir: &TempDir, base_text: &str) -> (Markdown, Base, PathBuf) {
    let base_path = dir.write("tasks.base", base_text.as_bytes());
    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let text = std::fs::read_to_string(&base_path).unwrap();
    let b = base::parse(&text).unwrap_or_else(|e| panic!("base::parse failed: {e}"));
    (md, b, base_path)
}

fn build(b: &Base, view: usize, src: &dyn Source) -> Grid {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    match b.build(view, src, &prop, TODAY, NOW) {
        Ok(g) => g,
        Err(e) => panic!("build({view}) failed: {e}"),
    }
}

fn build_err(b: &Base, view: usize, src: &dyn Source) -> String {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    match b.build(view, src, &prop, TODAY, NOW) {
        Ok(g) => panic!(
            "build({view}) should fail, got rows {:?}",
            labels(src, &g.rows)
        ),
        Err(e) => e,
    }
}

fn cell(b: &Base, src: &dyn Source, row: &RowId, col: &str) -> Shown {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    b.cell(src, &prop, row, col, TODAY, NOW)
}

/// 行の表示名(根からの相対パス)の列。
fn labels(src: &dyn Source, rows: &[RowId]) -> Vec<String> {
    rows.iter().map(|r| src.label(r)).collect()
}

fn sorted_labels(src: &dyn Source, rows: &[RowId]) -> Vec<String> {
    let mut v = labels(src, rows);
    v.sort();
    v
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| panic!("row {label} not found"))
}

fn ids(g: &Grid) -> Vec<String> {
    g.columns.iter().map(|c| c.id.clone()).collect()
}

fn titles(g: &Grid) -> Vec<String> {
    g.columns.iter().map(|c| c.title.clone()).collect()
}

// ---- BV-4: filters ----

#[test]
fn test_bv_4_global_and_view_filters_are_anded() {
    // [BV-4] 全体の filters `file.hasTag("a")` とビューの `status == "done"` を AND でつなぐ。
    let dir = vault("base-bv4");
    let text = r#"filters:
  and:
    - file.hasTag("a")
views:
  - type: table
    name: Done
    filters:
      and:
        - 'status == "done"'
    order:
      - file.name
      - status
  - type: table
    name: All tagged
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;

    // 両方を満たすノートだけ: tag a かつ done → a, d(c は done だが tag a が無い)。
    let g = build(&b, 0, src);
    assert_eq!(sorted_labels(src, &g.rows), vec!["a.md", "d.md"]);

    // ビューに filters が無ければ全体の filters だけ。
    let g = build(&b, 1, src);
    assert_eq!(
        sorted_labels(src, &g.rows),
        vec!["a.md", "b.md", "d.md", "e.md"]
    );
}

#[test]
fn test_bv_4_nested_and_or_not() {
    // [BV-4] and・or・not の入れ子。
    // or( and(status == "todo", priority >= 3), not(file.hasTag("a")) ) → e(todo・3)と c(tag a なし)。
    let dir = vault("base-bv4-nested");
    let text = r#"views:
  - type: table
    name: Nested
    filters:
      or:
        - and:
            - 'status == "todo"'
            - 'priority >= 3'
        - not:
            - file.hasTag("a")
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    assert_eq!(sorted_labels(src, &g.rows), vec!["c.md", "e.md"]);
}

// ---- BV-5: order・displayName・sort・groupBy・limit ----

#[test]
fn test_bv_5_order_and_display_name_give_column_ids_and_titles() {
    // [BV-5] order の順に列。id はノートのキーなら素の名前(`note.status` も `status`)、
    // ほかは `file.name`・`formula.x`。title は properties の displayName、無ければ id。
    let dir = vault("base-bv5-order");
    let text = r#"formulas:
  double: 'priority * 2'
properties:
  status:
    displayName: Status
  file.name:
    displayName: Name
  formula.double:
    displayName: "Double"
views:
  - type: table
    name: Cols
    order:
      - file.name
      - note.status
      - priority
      - formula.double
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    assert_eq!(
        ids(&g),
        vec!["file.name", "status", "priority", "formula.double"]
    );
    assert_eq!(titles(&g), vec!["Name", "Status", "priority", "Double"]);
    assert_eq!(g.rows.len(), 5, "no filters → all notes under the root");
    assert!(g.groups.is_empty(), "no groupBy → no groups");
}

#[test]
fn test_bv_5_parse_reads_view_fields() {
    // [BV-5] parse が type・name・order・sort・groupBy・limit を View に読む。
    let text = r#"views:
  - type: table
    name: Main
    order:
      - file.name
      - status
    sort:
      - property: priority
        direction: DESC
      - property: file.name
        direction: ASC
    groupBy:
      property: status
      direction: ASC
    limit: 2
"#;
    let b = base::parse(text).unwrap_or_else(|e| panic!("parse: {e}"));
    assert_eq!(b.views.len(), 1);
    let v = &b.views[0];
    assert_eq!(v.kind, "table");
    assert_eq!(v.name, "Main");
    assert_eq!(v.order.len(), 2);
    assert_eq!(v.sort.len(), 2);
    assert_eq!(v.sort[0].0, "priority");
    assert!(matches!(v.sort[0].1, Dir::Desc));
    assert_eq!(v.sort[1].0, "file.name");
    assert!(matches!(v.sort[1].1, Dir::Asc));
    let (gp, gd) = v.group_by.as_ref().expect("groupBy");
    assert_eq!(gp, "status");
    assert!(matches!(gd, Dir::Asc));
    assert_eq!(v.limit, Some(2));
}

#[test]
fn test_bv_5_sort_multiple_keys_asc_desc() {
    // [BV-5] sort の順に並べる: priority DESC、同じなら file.name ASC。
    let dir = vault("base-bv5-sort");
    let text = r#"views:
  - type: table
    name: Sorted
    order:
      - file.name
      - priority
    sort:
      - property: priority
        direction: DESC
      - property: file.name
        direction: ASC
  - type: table
    name: Reverse
    order:
      - file.name
      - priority
    sort:
      - property: priority
        direction: ASC
      - property: file.name
        direction: DESC
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;

    let g = build(&b, 0, src);
    assert_eq!(
        labels(src, &g.rows),
        vec!["c.md", "e.md", "a.md", "b.md", "d.md"]
    );

    let g = build(&b, 1, src);
    assert_eq!(
        labels(src, &g.rows),
        vec!["d.md", "b.md", "a.md", "e.md", "c.md"]
    );
}

#[test]
fn test_bv_5_group_by_headings_ranges_and_order_within_group() {
    // [BV-5] groupBy: {property: status, direction: ASC} → status ごとのまとまりで並び、
    // まとまりの中は sort(priority ASC、同じなら file.name ASC)。
    let dir = vault("base-bv5-group");
    let text = r#"views:
  - type: table
    name: Grouped
    order:
      - file.name
      - priority
    groupBy:
      property: status
      direction: ASC
    sort:
      - property: priority
        direction: ASC
      - property: file.name
        direction: ASC
  - type: table
    name: Grouped desc
    order:
      - file.name
    groupBy:
      property: note.status
      direction: DESC
    sort:
      - property: file.name
        direction: ASC
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;

    let g = build(&b, 0, src);
    // done: a(2), c(3), d(1) → d, a, c。todo: b(1), e(3) → b, e。
    assert_eq!(
        labels(src, &g.rows),
        vec!["d.md", "a.md", "c.md", "b.md", "e.md"]
    );
    assert_eq!(g.groups.len(), 2, "two groups: done and todo");
    assert!(
        g.groups[0].0.contains("done"),
        "first heading is done: {:?}",
        g.groups[0].0
    );
    assert_eq!(g.groups[0].1, 0..3);
    assert!(
        g.groups[1].0.contains("todo"),
        "second heading is todo: {:?}",
        g.groups[1].0
    );
    assert_eq!(g.groups[1].1, 3..5);

    // 降順のまとまり: todo が先、`note.status` でも同じ。
    let g = build(&b, 1, src);
    assert_eq!(
        labels(src, &g.rows),
        vec!["b.md", "e.md", "a.md", "c.md", "d.md"]
    );
    assert_eq!(g.groups.len(), 2);
    assert!(g.groups[0].0.contains("todo"), "{:?}", g.groups[0].0);
    assert_eq!(g.groups[0].1, 0..2);
    assert!(g.groups[1].0.contains("done"), "{:?}", g.groups[1].0);
    assert_eq!(g.groups[1].1, 2..5);
}

#[test]
fn test_bv_5_limit_applies_after_sort() {
    // [BV-5] limit は並べ替えの後に効く。
    let dir = vault("base-bv5-limit");
    let text = r#"views:
  - type: table
    name: Top
    limit: 2
    order:
      - file.name
    sort:
      - property: priority
        direction: DESC
      - property: file.name
        direction: ASC
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    assert_eq!(labels(src, &g.rows), vec!["c.md", "e.md"]);
}

// ---- BV-13: ビューの切り替え ----

#[test]
fn test_bv_13_three_views_change_rows_and_columns() {
    // [BV-13] views が3つ → build(0/1/2) で行と列が変わる。
    let dir = vault("base-bv13");
    let text = r#"views:
  - type: table
    name: All
    order:
      - file.name
      - status
  - type: table
    name: Done
    filters:
      and:
        - 'status == "done"'
    order:
      - file.name
      - priority
      - due
  - type: table
    name: Tag b
    filters:
      and:
        - file.hasTag("b")
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    assert_eq!(b.views.len(), 3);
    let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, vec!["All", "Done", "Tag b"]);

    let g0 = build(&b, 0, src);
    assert_eq!(
        sorted_labels(src, &g0.rows),
        vec!["a.md", "b.md", "c.md", "d.md", "e.md"]
    );
    assert_eq!(ids(&g0), vec!["file.name", "status"]);

    let g1 = build(&b, 1, src);
    assert_eq!(sorted_labels(src, &g1.rows), vec!["a.md", "c.md", "d.md"]);
    assert_eq!(ids(&g1), vec!["file.name", "priority", "due"]);

    let g2 = build(&b, 2, src);
    assert_eq!(sorted_labels(src, &g2.rows), vec!["b.md", "c.md"]);
    assert_eq!(ids(&g2), vec!["file.name"]);
}

// ---- SC-8: table 以外のビュー ----

#[test]
fn test_sc_8_cards_view_is_err() {
    // [SC-8] table 以外のビュー(cards)は開かず、理由を返す。table のビューは開ける。
    let dir = vault("base-sc8");
    let text = r#"views:
  - type: cards
    name: Cards
    order:
      - file.name
  - type: table
    name: Table
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    assert_eq!(b.views[0].kind, "cards");
    let e = build_err(&b, 0, src);
    assert!(!e.is_empty(), "reason is given");
    assert!(e.contains("cards"), "reason names the view type: {e}");
    let g = build(&b, 1, src);
    assert_eq!(g.rows.len(), 5);
}

#[test]
fn test_sc_8_list_map_kanban_views_are_err() {
    // [SC-8] list・map・kanban のビューも開かず、型の名前つきで未対応の理由を返す。
    let dir = vault("base-sc8-more");
    let text = r#"views:
  - type: list
    name: L
  - type: map
    name: M
  - type: kanban
    name: K
  - type: table
    name: T
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    for (i, kind) in ["list", "map", "kanban"].iter().enumerate() {
        assert_eq!(b.views[i].kind, *kind);
        let e = build_err(&b, i, src);
        assert!(e.contains(kind) && e.contains("未対応"), "{e}");
    }
    assert_eq!(build(&b, 3, src).rows.len(), 5);
}

// ---- BV-7: 未対応の式 ----

#[test]
fn test_bv_7_unsupported_function_in_filters_is_err_with_name() {
    // [BV-7] 評価できない filters のビューは開かず、関数名つきの理由を返す。推測で行を作らない。
    let dir = vault("base-bv7-filter");
    let text = r#"views:
  - type: table
    name: Linked
    filters:
      and:
        - link("Textbook")
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let e = build_err(&b, 0, src);
    assert!(e.contains("link"), "reason names the function: {e}");
}

#[test]
fn test_bv_7_unsupported_function_in_global_filters_is_err() {
    // [BV-7] 全体の filters に未対応の関数があっても同じ。
    let dir = vault("base-bv7-global");
    let text = r#"filters:
  or:
    - link("Textbook")
views:
  - type: table
    name: Any
    order:
      - file.name
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let e = build_err(&b, 0, src);
    assert!(e.contains("link"), "reason names the function: {e}");
}

#[test]
fn test_bv_7_unsupported_formula_column_is_marked() {
    // [BV-7] 評価できない式の列: build は Ok、セルは Shown::Unsupported、notes に説明。
    let dir = vault("base-bv7-formula");
    let text = r#"formulas:
  fixed: 'priority.toFixed(2)'
  double: 'priority * 2'
views:
  - type: table
    name: Formulas
    order:
      - file.name
      - formula.fixed
      - formula.double
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    assert_eq!(g.rows.len(), 5, "the view still opens");
    assert_eq!(
        ids(&g),
        vec!["file.name", "formula.fixed", "formula.double"]
    );
    assert!(
        g.notes.iter().any(|n| n.contains("toFixed")),
        "notes describe the unsupported expression: {:?}",
        g.notes
    );
    for row in &g.rows {
        match cell(&b, src, row, "formula.fixed") {
            Shown::Unsupported(s) => assert!(!s.is_empty(), "unsupported reason"),
            _ => panic!("{}: formula.fixed should be Unsupported", src.label(row)),
        }
    }
    // 対応している式の列は値が出る。
    let a = row_of(src, "a.md");
    match cell(&b, src, &a, "formula.double") {
        Shown::Computed(v) => assert_eq!(v, Val::Num(4.0)),
        _ => panic!("formula.double should be Computed"),
    }
}

// ---- formulas の値 ----

#[test]
fn test_bv_5_formula_columns_are_computed() {
    // [BV-5] [BV-6] formulas の列は Shown::Computed、ノートのキーの列は Shown::Prop。
    let dir = vault("base-formula");
    let text = r#"formulas:
  double: 'priority * 2'
  label: 'if(status == "done", "yes", "no")'
views:
  - type: table
    name: F
    order:
      - file.name
      - status
      - formula.double
      - formula.label
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    assert!(g.notes.is_empty(), "no unsupported columns: {:?}", g.notes);

    let a = row_of(src, "a.md");
    let bb = row_of(src, "b.md");
    match cell(&b, src, &a, "formula.double") {
        Shown::Computed(v) => assert_eq!(v, Val::Num(4.0)),
        _ => panic!("a formula.double should be Computed"),
    }
    match cell(&b, src, &bb, "formula.double") {
        Shown::Computed(v) => assert_eq!(v, Val::Num(2.0)),
        _ => panic!("b formula.double should be Computed"),
    }
    match cell(&b, src, &a, "formula.label") {
        Shown::Computed(v) => assert_eq!(v, Val::Str("yes".into())),
        _ => panic!("a formula.label should be Computed"),
    }
    match cell(&b, src, &bb, "formula.label") {
        Shown::Computed(v) => assert_eq!(v, Val::Str("no".into())),
        _ => panic!("b formula.label should be Computed"),
    }
    match cell(&b, src, &a, "status") {
        Shown::Prop(c) => assert_eq!(c.value, Some(Value::Str("done".into()))),
        _ => panic!("status should be Prop"),
    }
}

// ---- BV-8: 知らないキー ----

#[test]
fn test_bv_8_unknown_keys_are_ignored() {
    // [BV-8] ビューと全体に独自のキーを足しても parse でき、ビューも開ける。
    let dir = vault("base-bv8");
    let text = r#"myGlobalKey: 42
summaries:
  custom: 'values.mean()'
views:
  - type: table
    name: Custom
    myPluginKey:
      nested:
        - 1
        - 2
    columnSize:
      file.name: 200
    rowHeight: medium
    order:
      - file.name
      - status
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;
    assert_eq!(b.views.len(), 1);
    assert_eq!(b.views[0].name, "Custom");
    let g = build(&b, 0, src);
    assert_eq!(g.rows.len(), 5);
    assert_eq!(ids(&g), vec!["file.name", "status"]);
}

// ---- BV-3: `.base` を書き換えない ----

#[test]
fn test_bv_3_base_file_bytes_unchanged() {
    // [BV-3] build・cell の前後で `.base` のバイトと更新時刻が同じ。
    let dir = vault("base-bv3");
    let text = r#"# comment kept
filters:
  and:
    - file.hasTag("a")
formulas:
  double: 'priority * 2'
properties:
  status:
    displayName: Status
views:
  - type: table
    name: One
    order: [file.name, status, formula.double]
    sort:
      - property: priority
        direction: DESC
    myPluginKey: keep me
  - type: table
    name: Two
    groupBy:
      property: status
      direction: ASC
    limit: 3
"#;
    let base_path = dir.write("tasks.base", text.as_bytes());
    let before = std::fs::read(&base_path).unwrap();
    let mtime_before = std::fs::metadata(&base_path).unwrap().modified().unwrap();

    let mut md = Markdown::open_vault(&base_path).expect("open_vault");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let b = base::parse(&String::from_utf8(before.clone()).unwrap()).expect("parse");
    for v in 0..b.views.len() {
        let g = build(&b, v, src);
        for row in &g.rows {
            for c in &g.columns {
                let _ = cell(&b, src, row, &c.id);
            }
        }
    }
    drop(md);

    assert_eq!(
        std::fs::read(&base_path).unwrap(),
        before,
        "bytes unchanged"
    );
    assert_eq!(
        std::fs::metadata(&base_path).unwrap().modified().unwrap(),
        mtime_before,
        "mtime unchanged"
    );
}

// ---- CV-2: 型の合わない値の並べ替え ----

#[test]
fn test_cv_2_mismatched_date_sorts_last_both_directions() {
    // [CV-2] date の列に `someday` が混ざる → 昇順でも降順でも末尾。
    let dir = vault("base-cv2");
    let text = r#"views:
  - type: table
    name: Asc
    order:
      - file.name
      - due
    sort:
      - property: due
        direction: ASC
  - type: table
    name: Desc
    order:
      - file.name
      - due
    sort:
      - property: due
        direction: DESC
"#;
    let (md, b, _) = open_base(&dir, text);
    let src: &dyn Source = &md;

    let g = build(&b, 0, src);
    assert_eq!(
        labels(src, &g.rows),
        vec!["b.md", "c.md", "e.md", "a.md", "d.md"]
    );

    let g = build(&b, 1, src);
    assert_eq!(
        labels(src, &g.rows),
        vec!["a.md", "e.md", "c.md", "b.md", "d.md"]
    );
}

// ---- BV-1: `.base` なしの既定の表 ----

#[test]
fn test_bv_1_default_grid_uses_source_columns_and_rows() {
    // [BV-1] default_grid: 列は src.columns()、行は src.rows()、グループなし。
    let dir = vault("base-default");
    let mut md = Markdown::open(&[dir.path().to_path_buf()]).expect("open");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let g = base::default_grid(src);
    assert_eq!(ids(&g), src.columns());
    let g_rows: Vec<String> = g.rows.iter().map(|r| r.0.clone()).collect();
    let s_rows: Vec<String> = src.rows().iter().map(|r| r.0.clone()).collect();
    assert_eq!(g_rows, s_rows);
    assert!(g.groups.is_empty());
    assert_eq!(g.rows.len(), 5);
}
