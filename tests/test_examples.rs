//! サンプルの保管庫(examples/vault)を開く試験(記録 2026-10-01-examples のタスク 1)。
//! BV-1・BV-3・BV-5・BV-7・SC-8・CE-16。仕様: specs/base-view/spec.md、specs/scope/spec.md。
//!
//! サンプルは一時フォルダに写してから開く(リポの examples の中は書かない)。
//! サンプルが仕様や実装の変更で開けなくなったら、ここで気づく。

use mdgrid::base::{self, Base, Grid, Shown};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source, Value};
use std::path::{Path, PathBuf};

/// 2026-10-01(1970-01-01 からの日数)。
const TODAY: i64 = 20727;
const NOW: i64 = TODAY * 86_400;

/// サンプルのノートの数(タスク 12・メモ 4)。
const TASKS: usize = 12;
const MEMOS: usize = 4;

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

/// examples/vault を一時フォルダに写す。返すのは (一時フォルダ, 写した保管庫の根)。
fn sample(name: &str) -> (TempDir, PathBuf) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/vault");
    let dir = TempDir::new(name);
    let vault = dir.path().join("vault");
    copy_dir(&src, &vault);
    (dir, vault)
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
    assert_eq!(rows.len(), TASKS + MEMOS, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|l| l.starts_with("タスク/")).count(),
        TASKS
    );
    for col in ["status", "due", "priority", "done", "tags", "担当"] {
        assert!(src.columns().iter().any(|c| c == col), "column {col}");
    }
    assert_eq!(base::default_grid(src).rows.len(), TASKS + MEMOS);

    // [WB-3] フロントマターの無いノートは、フロントマターを足して書ける(lock なし、値は無し)。
    let plain = row_of(src, "メモ/買い物メモ.md");
    assert!(src.get(&plain, "status").lock.is_none());
    assert_eq!(src.get(&plain, "status").value, None);
    // [WB-5] 同じキーが2回あるノートは読むだけ。
    let dup = row_of(src, "メモ/週の振り返り.md");
    assert!(src.get(&dup, "status").lock.is_some());
    // [WB-13] 同期の競合ファイルに印。
    let conflict = row_of(src, "メモ/アイデア.sync-conflict-20260930-1.md");
    assert!(src.mark(&conflict).is_some());
    assert!(src.mark(&row_of(src, "メモ/アイデア.md")).is_none());

    // CV-1 の材料: null・空の文字列・キーなし の3通り。
    let get = |label: &str, col: &str| src.get(&row_of(src, label), col).value;
    assert_eq!(get("タスク/本を返す.md", "status"), Some(Value::Null));
    assert_eq!(
        get("タスク/健康診断を予約する.md", "status"),
        Some(Value::Str(String::new()))
    );
    assert_eq!(get("タスク/住所録を見直す.md", "status"), None);
    // [CE-8] ネストした map・ブロックスカラーは読むだけ。
    assert!(src
        .get(&row_of(src, "タスク/住所録を見直す.md"), "詳細")
        .lock
        .is_some());
    assert!(src
        .get(&row_of(src, "タスク/自転車を点検する.md"), "手順")
        .lock
        .is_some());
    // [CE-16] リストの値は書ける形なら編集できる: 1行のフロー(tags)も縦のリスト(持ち物)も lock なし。
    assert!(src
        .get(&row_of(src, "タスク/請求書を整理する.md"), "tags")
        .lock
        .is_none());
    assert!(src
        .get(&row_of(src, "タスク/旅行の荷物をまとめる.md"), "持ち物")
        .lock
        .is_none());
    // [CE-16] [CE-19] tags の候補は保管庫の中の要素と件数、件数の多い順。
    let tags = src.list_candidates("tags");
    assert_eq!(tags.first(), Some(&("家".to_string(), 5)), "{tags:?}");
    assert!(tags.contains(&("手続き".to_string(), 3)), "{tags:?}");
    let counts: Vec<usize> = tags.iter().map(|(_, n)| *n).collect();
    let mut sorted = counts.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(counts, sorted, "candidates are in descending count order");
}

// ---- .base で開く ----

#[test]
fn test_examples_base_views() {
    let (_dir, vault) = sample("examples-base");
    let base_path = vault.join("タスク.base");
    let before = std::fs::read(&base_path).unwrap();

    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let b = base::parse(&String::from_utf8(before.clone()).unwrap()).expect("base::parse");
    let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(
        names,
        ["進行中", "状態ごと", "期限", "未対応の式", "カード"]
    );

    // [BV-5] table のビューは全部 build できる。全体の filters でタスクのノートだけ。
    for (i, v) in b.views.iter().enumerate() {
        if v.kind != "table" {
            continue;
        }
        let g = build(&b, i, src).unwrap_or_else(|e| panic!("view {}: {e}", v.name));
        assert!(!g.rows.is_empty(), "view {} has no rows", v.name);
        for l in labels(src, &g.rows) {
            assert!(l.starts_with("タスク/"), "view {}: {l}", v.name);
        }
    }

    // 進行中: status == "doing" を due の昇順で。displayName が列の見出しに出る。
    let g = build(&b, view_index(&b, "進行中"), src).unwrap();
    assert_eq!(
        labels(src, &g.rows),
        [
            "タスク/請求書を整理する.md",
            "タスク/自転車を点検する.md",
            "タスク/旅行の荷物をまとめる.md"
        ]
    );
    let titles: Vec<&str> = g.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(&titles[..4], ["名前", "状態", "期限", "優先度"]);

    // 状態ごと: groupBy status。null・空の文字列・キーなし は1つの空のまとまり。
    let g = build(&b, view_index(&b, "状態ごと"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    let heads: Vec<&str> = g.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["doing", "done", "todo", "(空)"]);

    // 期限: formulas の残りの日数と if。型の合わない日付(someday)は末尾の側。
    let g = build(&b, view_index(&b, "期限"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    assert!(g.notes.is_empty(), "{:?}", g.notes);
    let first = &g.rows[0];
    assert_eq!(src.label(first), "タスク/電球を替える.md");
    assert!(matches!(
        cell(&b, src, first, "formula.状況"),
        Shown::Computed(mdgrid::expr::Val::Str(ref s)) if s == "済"
    ));
    let late = row_of(src, "タスク/引っ越しの見積もりを頼む.md");
    let pos = g.rows.iter().position(|r| *r == late).unwrap();
    assert!(pos >= TASKS - 2, "someday at {pos}");

    // [BV-7] 未対応の式: build は Ok、該当の列は全行 Unsupported、notes に理由。
    let g = build(&b, view_index(&b, "未対応の式"), src).unwrap();
    assert_eq!(g.rows.len(), TASKS);
    for col in ["formula.埋め込み", "formula.時間"] {
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
    let r = row_of(src, "タスク/書類を集める.md");
    assert!(matches!(cell(&b, src, &r, "見積時間"), Shown::Prop(_)));

    // [SC-8] カード(type: cards)は開かない。
    let e = build(&b, view_index(&b, "カード"), src).unwrap_err();
    assert!(e.contains("cards"), "{e}");

    // [BV-3] 操作の前後で `.base` のバイトが同じ。
    assert_eq!(std::fs::read(&base_path).unwrap(), before);
}

// ---- 機能を有効にした見本(examples/showcase。記録 2026-10-02-showcase のタスク 1) ----

/// showcase のノートの数(プロジェクト 17・メモ 4)。
const SHOWCASE_TASKS: usize = 17;
const SHOWCASE_MEMOS: usize = 4;

/// examples/showcase を一時フォルダに写す。返すのは (一時フォルダ, 写した showcase の根)。
fn showcase(name: &str) -> (TempDir, PathBuf) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/showcase");
    let dir = TempDir::new(name);
    let root = dir.path().join("showcase");
    copy_dir(&src, &root);
    (dir, root)
}

#[test]
fn test_examples_showcase() {
    // [CLI-3] [BV-1] [BV-5] [CE-22] [CE-16]
    let (_dir, root) = showcase("examples-showcase");

    // 設定は警告なしで読め、日付の形・週の始まり・検索の欄・キーの割り当て直しが入る。
    let text = std::fs::read_to_string(root.join("config.toml")).unwrap();
    let (c, warnings) = mdgrid::config::parse(&text).expect("config::parse");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(
        c.date_format.format(TODAY + 1),
        "2026/10/02 (金)",
        "date_format"
    );
    assert!(matches!(c.week_start, mdgrid::types::WeekStart::Mon));
    assert!(c.search_bar);
    assert!(!c.keys.is_empty());

    // フォルダで開く → プロジェクト・メモの全ノートが行に入る。
    let vault = root.join("vault");
    let mut md = Markdown::open(std::slice::from_ref(&vault)).expect("Markdown::open");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let rows = labels(src, &src.rows());
    assert_eq!(rows.len(), SHOWCASE_TASKS + SHOWCASE_MEMOS, "{rows:?}");
    assert_eq!(
        rows.iter()
            .filter(|l| l.starts_with("プロジェクト/"))
            .count(),
        SHOWCASE_TASKS
    );
    let cols = [
        "status",
        "priority",
        "owner",
        "start",
        "due",
        "estimate",
        "done",
        "tags",
        "assignees",
        "parent",
    ];
    for col in cols {
        assert!(src.columns().iter().any(|c| c == col), "column {col}");
    }
    // [CE-16] リストの列は縦の形も1行の形も編集でき、候補は複数で、重なる要素がある。
    assert!(src
        .get(&row_of(src, "プロジェクト/要件を洗い出す.md"), "tags")
        .lock
        .is_none());
    assert!(src
        .get(&row_of(src, "プロジェクト/要件を洗い出す.md"), "assignees")
        .lock
        .is_none());
    for col in ["tags", "assignees"] {
        let cands = src.list_candidates(col);
        assert!(cands.len() >= 5, "{col}: {cands:?}");
        assert!(
            cands.iter().filter(|(_, n)| *n >= 2).count() >= 3,
            "{col}: {cands:?}"
        );
    }

    // `.base` で開く: table のビューは全部 build でき、行が0でない。
    let base_path = vault.join("プロジェクト.base");
    let before = std::fs::read(&base_path).unwrap();
    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let src: &dyn Source = &md;
    let b = base::parse(&String::from_utf8(before.clone()).unwrap()).expect("base::parse");
    let names: Vec<&str> = b.views.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(
        names,
        ["今週の作業", "担当ごと", "優先度と見積", "完了", "全部"]
    );
    for (i, v) in b.views.iter().enumerate() {
        assert_eq!(v.kind, "table", "view {}", v.name);
        let g = build(&b, i, src).unwrap_or_else(|e| panic!("view {}: {e}", v.name));
        assert!(!g.rows.is_empty(), "view {} has no rows", v.name);
        assert!(g.notes.is_empty(), "view {}: {:?}", v.name, g.notes);
    }

    // 今週の作業(今日を 2026-10-01 に固定): 未完了で期日が 7 日以内(過ぎたものも)を期日の順に。
    let g = build(&b, view_index(&b, "今週の作業"), src).unwrap();
    assert_eq!(
        labels(src, &g.rows),
        [
            "プロジェクト/契約書の確認を頼む.md",
            "プロジェクト/見積書を出す.md",
            "プロジェクト/打ち合わせの資料をまとめる.md",
            "プロジェクト/ログイン画面を作る.md",
        ]
    );
    let titles: Vec<&str> = g.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(&titles[..3], ["タスク", "状況", "期日"]);

    // 担当ごと: 主担当(owner)の groupBy。主担当の無いタスクは空のまとまり。
    let g = build(&b, view_index(&b, "担当ごと"), src).unwrap();
    assert_eq!(g.rows.len(), SHOWCASE_TASKS);
    let heads: Vec<&str> = g.groups.iter().map(|(h, _)| h.as_str()).collect();
    for h in ["佐藤", "鈴木", "高橋", "田中", "(空)"] {
        assert!(heads.contains(&h), "{heads:?}");
    }

    // 優先度と見積: limit で 8 行、式の列は計算できる。
    let g = build(&b, view_index(&b, "優先度と見積"), src).unwrap();
    assert_eq!(g.rows.len(), 8);
    let first = &g.rows[0];
    assert_eq!(src.label(first), "プロジェクト/試作を作る.md");
    assert!(matches!(
        cell(&b, src, first, "formula.優先"),
        Shown::Computed(mdgrid::expr::Val::Str(ref s)) if s == "高"
    ));
    assert!(matches!(
        cell(&b, src, first, "formula.時間"),
        Shown::Computed(mdgrid::expr::Val::Num(n)) if n == 80.0
    ));

    // 完了: done のものだけ。
    let g = build(&b, view_index(&b, "完了"), src).unwrap();
    assert_eq!(g.rows.len(), 3);

    // 全部: メモも入る。
    let g = build(&b, view_index(&b, "全部"), src).unwrap();
    assert_eq!(g.rows.len(), SHOWCASE_TASKS + SHOWCASE_MEMOS);

    assert_eq!(std::fs::read(&base_path).unwrap(), before);
}
