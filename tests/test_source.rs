//! 読み込み口(Source)の受け入れテスト(タスク 18)。
//! SC-14・BV-1・CV-1・WB-3・WB-4・WB-5・WB-12・CE-8・CE-16・CE-18。
//! 仕様: specs/scope/spec.md、specs/base-view/spec.md、specs/write-back/spec.md、
//! specs/cell-edit/spec.md、specs/cell-view/spec.md。形: docs/design.md の src/source.rs。
//!
//! 核がこの口だけで使えること(SC-14)を確かめるため、Markdown::open のあとは
//! すべて `&dyn Source` / `&mut dyn Source` 越しに呼ぶ。

use mdgrid::source::markdown::Markdown;
use mdgrid::source::{Cell, Edit, NewValue, RowId, SaveError, Source, Stamp, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

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

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, bytes).unwrap();
        p
    }

    fn copy_fixture(&self, name: &str) -> PathBuf {
        self.write(name, &fixture(name))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

// ---- 助け ----

fn open(folder: &Path) -> Markdown {
    Markdown::open(&[folder.to_path_buf()]).expect("Markdown::open")
}

/// 読み込みを最後まで進める。
fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

/// 表示名(根からの相対パス)で行を探す。
fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| {
            let labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
            panic!("row {label} not found in {labels:?}")
        })
}

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn stamp(src: &dyn Source, row: &RowId) -> Stamp {
    src.stamp(row).expect("stamp of a loaded row")
}

fn read(p: &Path) -> String {
    String::from_utf8(std::fs::read(p).unwrap()).unwrap()
}

/// 外で書き換える。時刻の粒度に頼らないよう、更新時刻も前の時刻から 10 秒ずらす。
fn external_write(p: &Path, bytes: &[u8]) {
    let before = std::fs::metadata(p).unwrap().modified().unwrap();
    std::fs::write(p, bytes).unwrap();
    let f = std::fs::File::options().write(true).open(p).unwrap();
    f.set_modified(before + Duration::from_secs(10)).unwrap();
}

fn cell(src: &dyn Source, label: &str, col: &str) -> Cell {
    let row = row_of(src, label);
    src.get(&row, col)
}

// ---- BV-1・SC-14 ----

#[test]
fn test_bv_1_folder_rows_labels_and_columns() {
    // [BV-1] フォルダだけを渡す → ノートを行、フロントマターのキーの和を列にした既定の表。
    // [SC-14] Markdown::open のあとは &dyn Source だけで使う。
    let dir = TempDir::new("source-bv1");
    dir.write("a.md", b"---\ntitle: A\nstatus: draft\n---\nbody\n");
    dir.write("sub/b.md", b"---\nstatus: done\npriority: 2\n---\nbody\n");
    dir.write("sub/deeper/c.md", b"---\ntitle: C\nowner: me\n---\n");
    dir.write("not_a_note.txt", b"---\nignored: 1\n---\n");

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);

    let src: &dyn Source = src;
    let rows = src.rows();
    assert_eq!(rows.len(), 3, "rows = notes under the folder");

    let mut labels: Vec<String> = rows.iter().map(|r| src.label(r)).collect();
    labels.sort();
    assert_eq!(labels, vec!["a.md", "sub/b.md", "sub/deeper/c.md"]);

    // 行は昇順(Markdown では実体のパスの文字列)。
    let ids: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "rows are in ascending order");

    // 列はキーの和、最初に現れた順(行の順に a → sub/b → sub/deeper/c)。
    assert_eq!(
        src.columns(),
        vec!["title", "status", "priority", "owner"],
        "columns = union of keys in order of first appearance"
    );
}

#[test]
fn test_bv_1_rows_limited_to_given_folder_labels_from_root() {
    // [BV-1] 根が渡したフォルダより上でも、行は渡したフォルダの下のノートだけ。
    // label は根(.obsidian/ を持つ上のフォルダ)からの相対パス。
    let dir = TempDir::new("source-bv1-root");
    std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
    dir.write("outside.md", b"---\nx: 1\n---\n");
    dir.write("notes/a.md", b"---\ntitle: A\n---\n");
    dir.write("notes/sub/b.md", b"---\nstatus: done\n---\n");

    let mut md = open(&dir.path().join("notes"));
    load_all(&mut md);
    let src: &dyn Source = &md;

    let mut labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
    labels.sort();
    assert_eq!(labels, vec!["notes/a.md", "notes/sub/b.md"]);
    assert_eq!(src.columns(), vec!["title", "status"]);
}

// ---- CV-1・WB-3: 値・キーが無い・null ----

#[test]
fn test_cv_1_get_distinguishes_value_missing_and_null() {
    // [CV-1] null・空の文字列・キーが無い、を区別できる。
    // [WB-3] キーが無いセルは lock なし(末尾に1行足せる)。
    let dir = TempDir::new("source-cv1");
    dir.write("a.md", b"---\ntitle: A\nstatus: draft\n---\n");
    dir.write("b.md", b"---\ntitle: B\nstatus: null\n---\n");
    dir.write("c.md", b"---\ntitle: \"\"\n---\n");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    let c = cell(src, "a.md", "status");
    assert_eq!(c.value, Some(Value::Str("draft".into())));
    assert!(c.lock.is_none(), "plain value is editable: {:?}", c.lock);

    let c = cell(src, "b.md", "status");
    assert_eq!(c.value, Some(Value::Null), "null is Some(Value::Null)");
    assert!(c.lock.is_none(), "null value is editable: {:?}", c.lock);

    let c = cell(src, "c.md", "title");
    assert_eq!(c.value, Some(Value::Str(String::new())), "empty string");

    let c = cell(src, "c.md", "status");
    assert_eq!(c.value, None, "missing key is None");
    assert!(c.lock.is_none(), "missing key is not locked: {:?}", c.lock);
}

// ---- lock: WB-3・WB-5・CE-8・CE-16 ----

#[test]
fn test_wb_3_no_frontmatter_note_is_writable() {
    // [WB-3] フロントマターの無いノートは先頭にフロントマターを足して書ける → そのセルは lock なし。
    // file.* の列は CE-8 で lock あり、理由つき。
    let dir = TempDir::new("source-wb3");
    dir.copy_fixture("plain.md");
    dir.copy_fixture("no_frontmatter.md");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    let c = cell(src, "no_frontmatter.md", "status");
    assert_eq!(c.value, None, "body lines are not frontmatter");
    assert!(c.lock.is_none(), "no frontmatter → writable: {:?}", c.lock);
    let reason = cell(src, "no_frontmatter.md", "file.name")
        .lock
        .expect("file.* column stays locked");
    assert!(!reason.is_empty(), "lock has a reason");

    let c = cell(src, "plain.md", "status");
    assert!(c.lock.is_none());
}

#[test]
fn test_wb_5_duplicate_key_note_is_locked() {
    // [WB-5] 同じキーが2回あるノートは読むだけ、理由つき。
    let dir = TempDir::new("source-wb5");
    dir.copy_fixture("duplicate_key.md");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    let c = cell(src, "duplicate_key.md", "a");
    let reason = c.lock.expect("duplicate key → locked");
    assert!(!reason.is_empty(), "lock has a reason");
}

#[test]
fn test_ce_8_block_scalar_and_nested_are_locked() {
    // [CE-8] ブロックスカラー・ネストした map は読むだけ、理由つき。同じノートの素の値は書ける。
    let dir = TempDir::new("source-ce8");
    dir.copy_fixture("block_scalar.md");
    dir.copy_fixture("nested_map.md");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    for (note, key) in [
        ("block_scalar.md", "literal"),
        ("block_scalar.md", "folded"),
        ("nested_map.md", "meta"),
    ] {
        let c = cell(src, note, key);
        assert!(c.value.is_some(), "{note} {key}: value exists");
        let reason = c.lock.unwrap_or_else(|| panic!("{note} {key}: locked"));
        assert!(!reason.is_empty(), "{note} {key}: lock has a reason");
    }
    for note in ["block_scalar.md", "nested_map.md"] {
        let c = cell(src, note, "status");
        assert_eq!(c.value, Some(Value::Str("draft".into())));
        assert!(c.lock.is_none(), "{note} status is editable: {:?}", c.lock);
    }
}

#[test]
fn test_ce_16_list_values_are_editable_unless_ce_8_form() {
    // [CE-16] リストの値の列(tags)は、書ける形なら lock なし。フローのリストもブロックのリストも。
    // [CE-8] 書けない形(複数行のフロー・アンカー・入れ子の要素)は読むだけ、理由つき。
    let dir = TempDir::new("source-ce16");
    dir.copy_fixture("flow_list.md");
    dir.copy_fixture("block_list.md");
    dir.write("multi_flow.md", b"---\ntags: [one,\n  two]\n---\n");
    dir.write("anchor_list.md", b"---\ntags: &t [one, two]\n---\n");
    dir.write("nested_item.md", b"---\ntags:\n  - {k: v}\n---\n");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    for note in ["flow_list.md", "block_list.md"] {
        let c = cell(src, note, "tags");
        assert_eq!(
            c.value,
            Some(Value::List(vec![
                Value::Str("one".into()),
                Value::Str("two".into())
            ])),
            "{note}"
        );
        assert!(
            c.lock.is_none(),
            "{note}: writable list is locked: {:?}",
            c.lock
        );
    }
    for note in ["multi_flow.md", "anchor_list.md", "nested_item.md"] {
        let c = cell(src, note, "tags");
        let reason = c.lock.unwrap_or_else(|| panic!("{note} tags: locked"));
        assert!(!reason.is_empty(), "{note}: lock has a reason");
    }
}

#[test]
fn test_ce_18_list_save_keeps_form_through_source() {
    // [CE-18] Source 越しの保存: フローは1行のまま、ブロックは同じ字下げの行が増え、ほかのバイトは同じ。
    let dir = TempDir::new("source-ce18");
    let flow = dir.copy_fixture("flow_list.md");
    let block = dir.copy_fixture("block_list.md");
    let flow_before = read(&flow);
    let block_before = read(&block);

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let three = || NewValue::List(vec!["one".into(), "two".into(), "three".into()]);

    let row = row_of(src, "flow_list.md");
    let base = stamp(src, &row);
    src.save(&row, &base, &[edit("tags", three())])
        .expect("save flow list");
    assert_eq!(
        read(&flow),
        flow_before.replace("[one, two]", "[one, two, three]")
    );

    let row = row_of(src, "block_list.md");
    let base = stamp(src, &row);
    src.save(&row, &base, &[edit("tags", three())])
        .expect("save block list");
    assert_eq!(
        read(&block),
        block_before.replace("  - two\n", "  - two\n  - three\n")
    );
    assert_eq!(
        src.get(&row, "tags").value,
        Some(Value::List(vec![
            Value::Str("one".into()),
            Value::Str("two".into()),
            Value::Str("three".into())
        ]))
    );
}

// ---- preview ----

#[test]
fn test_preview_returns_before_and_after_without_touching_file() {
    // [SC-14] 保存の前の差分: (今のバイト, 編集後のバイト)。ファイルは変わらない。
    let dir = TempDir::new("source-preview");
    let p = dir.copy_fixture("plain.md");
    let original = std::fs::read(&p).unwrap();

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;
    let row = row_of(src, "plain.md");

    let (before, after) = src
        .preview(&row, &[edit("status", s("done"))])
        .expect("preview");
    assert_eq!(before, original, "before = current bytes");
    assert_eq!(
        String::from_utf8(after).unwrap(),
        "---\ntitle: t\nstatus: done\n---\nbody\n"
    );
    assert_eq!(std::fs::read(&p).unwrap(), original, "file unchanged");
}

// ---- save: WB-12・WB-4 ----

#[test]
fn test_wb_12_save_then_save_again_with_returned_stamp() {
    // [WB-12] 保存で返った基準を使えば、同じノートの別のキーを続けて保存できる。
    let dir = TempDir::new("source-wb12");
    let p = dir.copy_fixture("plain.md");

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let row = row_of(src, "plain.md");

    let base = stamp(src, &row);
    let next = src
        .save(&row, &base, &[edit("status", s("done"))])
        .expect("first save");
    assert_eq!(read(&p), "---\ntitle: t\nstatus: done\n---\nbody\n");
    assert_eq!(
        src.get(&row, "status").value,
        Some(Value::Str("done".into())),
        "re-read after save"
    );

    src.save(&row, &next, &[edit("title", s("u"))])
        .expect("second save with the returned stamp");
    assert_eq!(read(&p), "---\ntitle: u\nstatus: done\n---\nbody\n");
}

#[test]
fn test_wb_4_external_change_after_stamp_stops_save() {
    // [WB-4] 基準を取ったあとに外でファイルが変わる → SaveError::Changed、ファイルは外の内容のまま。
    let dir = TempDir::new("source-wb4");
    let p = dir.copy_fixture("plain.md");

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let row = row_of(src, "plain.md");

    let base = stamp(src, &row);
    let outside = b"---\ntitle: t\nstatus: draft\nowner: someone\n---\nbody\n";
    external_write(&p, outside);

    let r = src.save(&row, &base, &[edit("status", s("done"))]);
    match r {
        Err(SaveError::Changed) => {}
        Err(e) => panic!("expected SaveError::Changed, got {e:?}"),
        Ok(_) => panic!("expected SaveError::Changed, got Ok"),
    }
    assert_eq!(
        std::fs::read(&p).unwrap(),
        outside,
        "file keeps outside edit"
    );
}

#[test]
fn test_wb_4_reload_then_save_over_external_change() {
    // [WB-4] 外の変更の上に書く: reload のあとの stamp は今のファイルの内容で、その基準なら保存できる。
    // ためたキーの値だけが書かれ、外の変更は残る。
    let dir = TempDir::new("source-wb4-reload");
    let p = dir.copy_fixture("plain.md");

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let row = row_of(src, "plain.md");

    let old = stamp(src, &row);
    let outside = b"---\ntitle: t\nstatus: draft\nowner: someone\n---\nbody\n";
    external_write(&p, outside);

    src.reload(&row).expect("reload");
    let now = stamp(src, &row);
    assert_eq!(now.len, outside.len() as u64, "stamp is the current file");
    assert_ne!(now.hash, old.hash, "hash follows the current file");
    assert_eq!(
        src.get(&row, "owner").value,
        Some(Value::Str("someone".into())),
        "get sees the reloaded content"
    );

    src.save(&row, &now, &[edit("status", s("done"))])
        .expect("save on the reloaded stamp");
    assert_eq!(
        read(&p),
        "---\ntitle: t\nstatus: done\nowner: someone\n---\nbody\n"
    );
}

// ---- changed ----

#[test]
fn test_changed_reports_externally_edited_row_and_rereads() {
    // [SC-14] 外での変化(BV-9 のポーリング): 直したノートの RowId が返り、get が新しい値になる。
    let dir = TempDir::new("source-changed");
    let p = dir.copy_fixture("plain.md");
    dir.copy_fixture("quoted.md");

    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let row = row_of(src, "plain.md");
    let other = row_of(src, "quoted.md");

    external_write(&p, b"---\ntitle: t\nstatus: finished\n---\nbody\n");

    let changed = src.changed();
    assert!(changed.contains(&row), "edited row is reported");
    assert!(!changed.contains(&other), "untouched row is not reported");
    assert_eq!(
        src.get(&row, "status").value,
        Some(Value::Str("finished".into()))
    );
}
