//! 見本のルートフォルダ(examples/quirks と examples/studio)を開く試験(記録 2026-10-01-examples のタスク 1、
//! 見本を作り直した記録 2026-10-15-examples-studio)。
//! BV-1・BV-3・BV-5・BV-7・SC-8・CE-16。仕様: specs/base-view/spec.md、specs/scope/spec.md。
//!
//! 見本は一時フォルダに写してから開く(リポの examples の中は書かない)。
//! 見本が仕様や実装の変更で開けなくなったら、ここで気づく。

use mdgrid::base::{self, Base, Grid, Shown};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source, Value};
use std::path::{Path, PathBuf};

/// 2026-10-01(1970-01-01 からの日数)。
const TODAY: i64 = 20727;
const NOW: i64 = TODAY * 86_400;

/// quirks のノートの数(タスク 12・メモ 5)。
const TASKS: usize = 12;
const NOTES: usize = 5;

// ---- 一時フォルダ ----

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
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), &dst).unwrap();
        }
    }
}

/// examples の下の見本を一時フォルダに写す。返すのは (一時フォルダ, 写した見本の根)。
fn sample_of(example: &str, name: &str) -> (TempDir, PathBuf) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(example);
    let dir = TempDir::new(name);
    let root = dir.path().join(example);
    copy_dir(&src, &root);
    (dir, root)
}

/// examples/quirks(扱いにくいノートを集めた見本)を写す。
fn sample(name: &str) -> (TempDir, PathBuf) {
    sample_of("quirks", name)
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

fn labels(src: &dyn Source, rows: &[RowId]) -> Vec<String> {
    rows.iter().map(|r| src.label(r)).collect()
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| panic!("row {label} not found in {:?}", labels(src, &src.rows())))
}

fn build(b: &Base, view: usize, src: &dyn Source) -> Result<Grid, String> {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    b.build(view, src, &prop, TODAY, NOW)
}

fn cell(b: &Base, src: &dyn Source, row: &RowId, col: &str) -> Shown {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    b.cell(src, &prop, row, col, TODAY, NOW)
}

fn view_index(b: &Base, name: &str) -> usize {
    b.views
        .iter()
        .position(|v| v.name == name)
        .unwrap_or_else(|| panic!("view {name} not found"))
}

// ---- フォルダで開く ----

#[test]
fn test_examples_open_folder() {
    // [BV-1] フォルダで開く → タスク・メモの全ノート(フロントマターの無いノート・重複キー・競合ファイルも)が行に入る。
    let (_dir, vault) = sample("examples-folder");
    let mut md = Markdown::open(std::slice::from_ref(&vault)).expect("Markdown::open");
    load_all(&mut md);
    let src: &dyn Source = &md;

    let rows = labels(src, &src.rows());
    assert_eq!(rows.len(), TASKS + NOTES, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|l| l.starts_with("Tasks/")).count(),
        TASKS
    );
    for col in ["status", "due", "priority", "done", "tags", "owner"] {
        assert!(src.columns().iter().any(|c| c == col), "column {col}");
    }
    assert_eq!(base::default_grid(src).rows.len(), TASKS + NOTES);

    // [WB-3] フロントマターの無いノートと空のフロントマターのノートは、書ける(lock なし、値は無し)。
    for label in ["Notes/Snack list.md", "Notes/Blank.md"] {
        let plain = row_of(src, label);
        assert!(src.get(&plain, "status").lock.is_none(), "{label}");
        assert_eq!(src.get(&plain, "status").value, None, "{label}");
    }
    // [WB-5] 同じキーが2回あるノートは読むだけ。
    let dup = row_of(src, "Notes/Weekly review.md");
    assert!(src.get(&dup, "status").lock.is_some());
    // [WB-13] 同期の競合ファイルに印。
    let conflict = row_of(src, "Notes/Ideas.sync-conflict-20260930-1.md");
    assert!(src.mark(&conflict).is_some());
    assert!(src.mark(&row_of(src, "Notes/Ideas.md")).is_none());

    // CV-1 の材料: null・空の文字列・キーなし の3通り。
    let get = |label: &str, col: &str| src.get(&row_of(src, label), col).value;
    assert_eq!(
        get("Tasks/Return the library books.md", "status"),
        Some(Value::Null)
    );
    assert_eq!(
        get("Tasks/Book the team lunch.md", "status"),
        Some(Value::Str(String::new()))
    );
    assert_eq!(get("Tasks/Clean up the contact list.md", "status"), None);
    // [CE-8] ネストした map・ブロックスカラーは読むだけ。
    assert!(src
        .get(
            &row_of(src, "Tasks/Clean up the contact list.md"),
            "details"
        )
        .lock
        .is_some());
    assert!(src
        .get(&row_of(src, "Tasks/Service the studio bikes.md"), "steps")
        .lock
        .is_some());
    // [CE-16] リストの値は書ける形なら編集できる: 1行のフロー(tags)も縦のリスト(packing)も lock なし。
    assert!(src
        .get(&row_of(src, "Tasks/Sort the receipts.md"), "tags")
        .lock
        .is_none());
    assert!(src
        .get(&row_of(src, "Tasks/Pack for the offsite.md"), "packing")
        .lock
        .is_none());
    // [CE-16] [CE-19] tags の候補はルートフォルダの中の要素と件数、件数の多い順。
    let tags = src.list_candidates("tags");
    assert_eq!(tags.first(), Some(&("office".to_string(), 6)), "{tags:?}");
    assert!(tags.contains(&("money".to_string(), 3)), "{tags:?}");
    let counts: Vec<usize> = tags.iter().map(|(_, n)| *n).collect();
    let mut sorted = counts.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(counts, sorted, "candidates are in descending count order");
}

// ---- .base で開く ----

#[test]
fn test_examples_base_views() {
    let (_dir, vault) = sample("examples-base");
    let base_path = vault.join("Tasks.base");
    let before = std::fs::read(&base_path).unwrap();

    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let b = base::parse(&String::from_utf8(before.clone()).unwrap()).expect("base::parse");
    let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["Doing", "By status", "Due", "Unsupported", "Cards"]);

    // [BV-5] table のビューは全部 build できる。全体の filters でタスクのノートだけ。
    for (i, v) in b.views.iter().enumerate() {
        if v.kind != "table" {
            continue;
        }
        let g = build(&b, i, src).unwrap_or_else(|e| panic!("view {}: {e}", v.name));
        assert!(!g.rows.is_empty(), "view {} has no rows", v.name);
        for l in labels(src, &g.rows) {
            assert!(l.starts_with("Tasks/"), "view {}: {l}", v.name);
        }
    }

    // Doing: status == "doing" を due の昇順で。displayName が列の見出しに出る。
    let g = build(&b, view_index(&b, "Doing"), src).unwrap();
    assert_eq!(
        labels(src, &g.rows),
        [
            "Tasks/Sort the receipts.md",
            "Tasks/Service the studio bikes.md",
            "Tasks/Pack for the offsite.md"
        ]
    );
    let titles: Vec<&str> = g.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(&titles[..3], ["Status", "Due", "Priority"]);

    // By status: groupBy status。null・空の文字列・キーなし は1つの空のまとまり。
    let g = build(&b, view_index(&b, "By status"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    let heads: Vec<&str> = g.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["doing", "done", "todo", "(空)"]);

    // Due: formulas の残りの日数と if。型の合わない日付(someday)は末尾の側。
    let g = build(&b, view_index(&b, "Due"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    assert!(g.notes.is_empty(), "{:?}", g.notes);
    let first = &g.rows[0];
    assert_eq!(src.label(first), "Tasks/Replace the hallway bulb.md");
    assert!(matches!(
        cell(&b, src, first, "formula.state"),
        Shown::Computed(mdgrid::expr::Val::Str(ref s)) if s == "done"
    ));
    let late = row_of(src, "Tasks/Get quotes for new chairs.md");
    let pos = g.rows.iter().position(|r| *r == late).unwrap();
    assert!(pos >= TASKS - 2, "someday at {pos}");

    // [BV-7] Unsupported: build は Ok、該当の列は全行 Unsupported、notes に理由。
    let g = build(&b, view_index(&b, "Unsupported"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    for col in ["formula.hours_fixed", "formula.embeds"] {
        assert!(g.notes.iter().any(|n| n.starts_with(col)), "{:?}", g.notes);
        for r in &g.rows {
            assert!(
                matches!(cell(&b, src, r, col), Shown::Unsupported(_)),
                "{col} {}",
                src.label(r)
            );
        }
    }
    // ノートのキーの列は普通の値。
    let r = row_of(src, "Tasks/Collect the tax documents.md");
    assert!(matches!(cell(&b, src, &r, "hours"), Shown::Prop(_)));

    // [SC-8] カード(type: cards)は開かない。
    let e = build(&b, view_index(&b, "Cards"), src).unwrap_err();
    assert!(e.contains("cards"), "{e}");

    // [BV-3] 操作の前後で `.base` のバイトが同じ。
    assert_eq!(std::fs::read(&base_path).unwrap(), before);
}

// ---- 機能を有効にした設定と主役の見本(examples/configs/studio.toml と examples/studio) ----

/// studio のノートの数(タスク 20・案件 4・顧客 3・人 5・会議 7)。
const STUDIO_TASKS: usize = 20;
const STUDIO_NOTES: usize = 20 + 4 + 3 + 5 + 7;
/// 2026-10-06(studio の画面の写しと同じ今日)。
const STUDIO_TODAY: i64 = 20732;

#[test]
fn test_examples_showcase() {
    // [CLI-3] [BV-1] [BV-5] [CE-22] [CE-16]
    let (_dir, examples) = sample_of(".", "examples-showcase");

    // 設定は警告なしで読め、日付の形・週の始まり・検索の欄・キーの割り当て直しが入る。
    let text = std::fs::read_to_string(examples.join("configs/studio.toml")).unwrap();
    let (c, warnings) = mdgrid::config::parse(&text).expect("config::parse");
    assert!(warnings.is_empty(), "{warnings:?}");
    let r = c.resolved();
    assert_eq!(
        r.date_format.format(TODAY + 1),
        "2026/10/02 (金)",
        "dates.format"
    );
    assert!(matches!(r.week_start, mdgrid::types::WeekStart::Mon));
    assert!(r.display.search_bar);
    assert!(!c.keys.is_empty());

    // フォルダで開く → 5つのフォルダの全ノートが行に入る。
    let root = examples.join("studio");
    let mut md = Markdown::open(std::slice::from_ref(&root)).expect("Markdown::open");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let rows = labels(src, &src.rows());
    assert_eq!(rows.len(), STUDIO_NOTES, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|l| l.starts_with("Tasks/")).count(),
        STUDIO_TASKS
    );
    let cols = [
        "status",
        "project",
        "owner",
        "due",
        "priority",
        "estimate",
        "billable",
        "tags",
        "depends_on",
        "client",
        "lead",
        "team",
        "when",
        "attendees",
    ];
    for col in cols {
        assert!(src.columns().iter().any(|c| c == col), "column {col}");
    }
    // [CE-16] リストの列は縦の形(depends_on)も1行の形(tags)も編集でき、候補は複数で、重なる要素がある。
    let task = row_of(src, "Tasks/Build the menu page.md");
    assert!(src.get(&task, "tags").lock.is_none());
    assert!(src.get(&task, "depends_on").lock.is_none());
    for col in ["tags", "attendees"] {
        let cands = src.list_candidates(col);
        assert!(cands.len() >= 5, "{col}: {cands:?}");
        assert!(
            cands.iter().filter(|(_, n)| *n >= 2).count() >= 3,
            "{col}: {cands:?}"
        );
    }

    // `.base` で開く: table のビューは全部 build でき、行が0でない。
    let base_path = root.join("Tasks.base");
    let before = std::fs::read(&base_path).unwrap();
    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let b = base::parse(&String::from_utf8(before.clone()).unwrap()).expect("base::parse");
    let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Open",
            "This week",
            "By status",
            "By project",
            "By owner",
            "Dependencies",
            "Workload"
        ]
    );
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    let at = |view: &str| {
        b.build(
            view_index(&b, view),
            src,
            &prop,
            STUDIO_TODAY,
            STUDIO_TODAY * 86_400,
        )
        .unwrap_or_else(|e| panic!("view {view}: {e}"))
    };
    for v in &b.views {
        assert_eq!(v.kind, "table", "view {}", v.name);
        let g = at(&v.name);
        assert!(!g.rows.is_empty(), "view {} has no rows", v.name);
        assert!(g.notes.is_empty(), "view {}: {:?}", v.name, g.notes);
        for l in labels(src, &g.rows) {
            assert!(l.starts_with("Tasks/"), "view {}: {l}", v.name);
        }
    }

    // Open: 済んでいない 15 件。
    assert_eq!(at("Open").rows.len(), 15);

    // This week(今日を 2026-10-06 に固定): 済んでいないもので期日が 7 日以内(過ぎたものも)を期日 → 優先度の順に。
    let g = at("This week");
    assert_eq!(
        labels(src, &g.rows),
        [
            "Tasks/Review privacy requirements with the clinic.md",
            "Tasks/Set up hosting for the bakery site.md",
            "Tasks/Write the menu page copy.md",
            "Tasks/Write the ordering estimate.md",
            "Tasks/Build the appointment API.md",
            "Tasks/Photograph the seasonal pastries.md",
            "Tasks/Send the final invoice to Lumen Books.md",
        ]
    );
    let titles: Vec<&str> = g.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(&titles[..2], ["State", "due"]);
    let state = |row: &RowId| {
        b.cell(
            src,
            &prop,
            row,
            "formula.state",
            STUDIO_TODAY,
            STUDIO_TODAY * 86_400,
        )
    };
    assert!(matches!(
        state(&g.rows[0]),
        Shown::Computed(mdgrid::expr::Val::Str(ref s)) if s == "overdue"
    ));
    assert!(matches!(
        state(&g.rows[2]),
        Shown::Computed(mdgrid::expr::Val::Str(ref s)) if s == "soon"
    ));

    // By status: status ごとのまとまり(done・doing・todo)。
    let g = at("By status");
    assert_eq!(g.rows.len(), STUDIO_TASKS);
    let heads: Vec<&str> = g.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["doing", "done", "todo"]);

    // By project・By owner: リンクの列の groupBy。案件の無いタスクは空のまとまり。
    let g = at("By project");
    assert_eq!(g.rows.len(), STUDIO_TASKS);
    assert_eq!(g.groups.len(), 5, "4 projects and (empty)");
    let g = at("By owner");
    assert_eq!(g.rows.len(), STUDIO_TASKS);
    assert_eq!(g.groups.len(), 5, "5 people");

    // Dependencies: 済んでいないもので depends_on のあるもの。
    assert_eq!(at("Dependencies").rows.len(), 7);

    // Workload: 優先度 → 見積の大きい順。式の列(見積 × 8)は計算できる。
    let g = at("Workload");
    let first = &g.rows[0];
    assert_eq!(src.label(first), "Tasks/Build the appointment API.md");
    assert!(matches!(
        b.cell(src, &prop, first, "formula.hours", STUDIO_TODAY, STUDIO_TODAY * 86_400),
        Shown::Computed(mdgrid::expr::Val::Num(n)) if n == 64.0
    ));

    assert_eq!(std::fs::read(&base_path).unwrap(), before);
}
