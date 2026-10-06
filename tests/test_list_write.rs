//! リストの値の付け外し(タスク 1: 書き戻しと候補の核)の受け入れテスト。
//! CE-16・CE-18・CE-19・CE-8・CE-9・WB-1・WB-6・WB-7・WB-17。
//! 仕様: specs/cell-edit/spec.md、specs/write-back/spec.md。
//! 形: docs/design.md「リストの値の付け外し(CE-16〜CE-19)」(NewValue::List・apply・
//! Source::list_candidates・Cell.lock・same_value)。
//!
//! 実装を見ずに、仕様と公開のインターフェースだけから書いた。

use mdgrid::changes::Changes;
use mdgrid::frontmatter::{parse, Frontmatter, Value};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{Cell, RowId, Source};
use mdgrid::writeback::{apply, baseline, save, Edit, EditError, NewValue};
use std::path::{Path, PathBuf};

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
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 助け ----

fn list(items: &[&str]) -> NewValue {
    NewValue::List(items.iter().map(|s| s.to_string()).collect())
}

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn str_list(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| Value::Str(s.to_string())).collect())
}

fn value_of(fm: &Frontmatter, key: &str) -> Option<Value> {
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.value.clone())
}

/// `src` の `key` に `items` を書いた結果を文字列で返す。
fn apply_list(src: &str, key: &str, items: &[&str]) -> String {
    let out = apply(src.as_bytes(), &[edit(key, list(items))])
        .unwrap_or_else(|e| panic!("apply {items:?} onto {src:?}: {e:?}"));
    String::from_utf8(out).unwrap()
}

/// 書いた結果を読み直すと、`key` は `items` の文字列のリストで、ほかのキーの値は元のまま。
fn assert_reads_back(before: &str, after: &str, key: &str, items: &[&str]) {
    let fb = parse(before.as_bytes()).expect("parse before");
    let fa = parse(after.as_bytes()).expect("parse after");
    assert_eq!(
        value_of(&fa, key),
        Some(str_list(items)),
        "{key} after writing {items:?}: {after:?}"
    );
    for e in &fb.entries {
        if e.key == key {
            continue;
        }
        assert_eq!(
            value_of(&fa, &e.key),
            Some(e.value.clone()),
            "other key {} changed: {after:?}",
            e.key
        );
    }
}

fn open(folder: &Path) -> Markdown {
    Markdown::open(&[folder.to_path_buf()]).expect("Markdown::open")
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| {
            let labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
            panic!("row {label} not found in {labels:?}")
        })
}

fn cell(src: &dyn Source, label: &str, col: &str) -> Cell {
    src.get(&row_of(src, label), col)
}

fn read(p: &Path) -> String {
    String::from_utf8(std::fs::read(p).unwrap()).unwrap()
}

fn cands(items: &[(&str, usize)]) -> Vec<(String, usize)> {
    items.iter().map(|(s, n)| (s.to_string(), *n)).collect()
}

// ---- CE-18: 1行のフローのリスト ----

#[test]
fn test_ce_18_flow_list_add_keeps_form() {
    // [CE-18] `tags: [a, b]` に c を足す → `tags: [a, b, c]`。他のバイトは同じ。
    let before = "---\ntitle: t\ntags: [a, b]\nstatus: draft\n---\nbody\n";
    let after = apply_list(before, "tags", &["a", "b", "c"]);
    assert_eq!(
        after,
        "---\ntitle: t\ntags: [a, b, c]\nstatus: draft\n---\nbody\n"
    );
    assert_reads_back(before, &after, "tags", &["a", "b", "c"]);
}

#[test]
fn test_ce_18_flow_list_remove_and_reorder() {
    // [CE-18] 外す・並べ替える → 1行のフローのまま、区切りは ", "。
    let before = "---\ntags: [a, b, c]\nstatus: draft\n---\nbody\n";
    assert_eq!(
        apply_list(before, "tags", &["a", "c"]),
        "---\ntags: [a, c]\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list(before, "tags", &["c", "a", "b"]),
        "---\ntags: [c, a, b]\nstatus: draft\n---\nbody\n"
    );
}

#[test]
fn test_ce_18_flow_list_keeps_trailing_comment_and_crlf() {
    // [CE-18] [WB-1] 行末のコメントと CRLF は残る。変わるのは値の範囲だけ。
    let before = "---\r\ntags: [a, b]  # memo\r\nstatus: draft\r\n---\r\nbody\r\n";
    assert_eq!(
        apply_list(before, "tags", &["a", "b", "c"]),
        "---\r\ntags: [a, b, c]  # memo\r\nstatus: draft\r\n---\r\nbody\r\n"
    );
}

#[test]
fn test_ce_18_empty_flow_list_stays_flow() {
    // [CE-18] 1行のフローの空のリスト `[]` に足す → 元の書き方(1行のフロー)を保つ。
    let before = "---\ntags: []\nstatus: draft\n---\n";
    assert_eq!(
        apply_list(before, "tags", &["a"]),
        "---\ntags: [a]\nstatus: draft\n---\n"
    );
}

#[test]
fn test_ce_18_string_onto_flow_list_is_not_editable() {
    // [CE-18] リストに文字列(NewValue::Str)を当てると書き方が変わるので、書かない。
    let src = b"---\ntags: [a, b]\n---\n";
    assert!(matches!(
        apply(src, &[edit("tags", NewValue::Str("x".into()))]),
        Err(EditError::NotEditable(_))
    ));
}

// ---- CE-18: 複数行のブロックのリスト ----

const BLOCK2: &str = "---\ntitle: t\ntags:\n  - a\n  - b\nstatus: draft\n---\nbody\n";
const BLOCK0: &str = "---\ntitle: t\ntags:\n- a\n- b\nstatus: draft\n---\nbody\n";

#[test]
fn test_ce_18_block_list_indent_2_add_remove_reorder() {
    // [CE-18] 字下げ2の `- a` の縦のリスト: 足す・外す・並べ替える → 同じ字下げの行、ほかの行は1バイトも変わらない。
    let after = apply_list(BLOCK2, "tags", &["a", "b", "c"]);
    assert_eq!(
        after,
        "---\ntitle: t\ntags:\n  - a\n  - b\n  - c\nstatus: draft\n---\nbody\n"
    );
    assert_reads_back(BLOCK2, &after, "tags", &["a", "b", "c"]);

    assert_eq!(
        apply_list(BLOCK2, "tags", &["a"]),
        "---\ntitle: t\ntags:\n  - a\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list(BLOCK2, "tags", &["b", "a"]),
        "---\ntitle: t\ntags:\n  - b\n  - a\nstatus: draft\n---\nbody\n"
    );
}

#[test]
fn test_ce_18_block_list_indent_0_add_remove_reorder() {
    // [CE-18] 字下げ0の `- a` の縦のリストも、字下げ0のまま。
    let after = apply_list(BLOCK0, "tags", &["a", "b", "c"]);
    assert_eq!(
        after,
        "---\ntitle: t\ntags:\n- a\n- b\n- c\nstatus: draft\n---\nbody\n"
    );
    assert_reads_back(BLOCK0, &after, "tags", &["a", "b", "c"]);

    assert_eq!(
        apply_list(BLOCK0, "tags", &["a"]),
        "---\ntitle: t\ntags:\n- a\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list(BLOCK0, "tags", &["b", "a"]),
        "---\ntitle: t\ntags:\n- b\n- a\nstatus: draft\n---\nbody\n"
    );
}

#[test]
fn test_ce_18_block_list_at_end_of_frontmatter() {
    // [CE-18] 縦のリストがフロントマターの最後のキーでも、閉じの区切りと本文はそのまま。
    let before = "---\nstatus: draft\ntags:\n  - a\n---\nbody\n";
    assert_eq!(
        apply_list(before, "tags", &["a", "b"]),
        "---\nstatus: draft\ntags:\n  - a\n  - b\n---\nbody\n"
    );
}

#[test]
fn test_ce_18_block_list_keeps_crlf() {
    // [CE-18] [WB-1] CRLF のノート: 足した行も CRLF、ほかの行は1バイトも変わらない。
    let before = "---\r\ntitle: t\r\ntags:\r\n  - a\r\n  - b\r\nstatus: draft\r\n---\r\nbody\r\n";
    assert_eq!(
        apply_list(before, "tags", &["a", "b", "c"]),
        "---\r\ntitle: t\r\ntags:\r\n  - a\r\n  - b\r\n  - c\r\nstatus: draft\r\n---\r\nbody\r\n"
    );
    assert_eq!(
        apply_list(before, "tags", &["b"]),
        "---\r\ntitle: t\r\ntags:\r\n  - b\r\nstatus: draft\r\n---\r\nbody\r\n"
    );
}

#[test]
fn test_ce_18_block_list_with_quoted_elements_reads_back() {
    // [CE-18] [WB-7] 引用符つきの要素の縦のリストも書ける。読み直すと意図した並び、ほかのキーは同じ。
    let before = "---\ntags:\n  - \"a\"\n  - 'b'\nstatus: draft\n---\nbody\n";
    let after = apply_list(before, "tags", &["a", "b", "c"]);
    assert_reads_back(before, &after, "tags", &["a", "b", "c"]);
    assert!(after.starts_with("---\ntags:\n"), "{after:?}");
    assert!(after.ends_with("status: draft\n---\nbody\n"), "{after:?}");
}

// ---- CE-18: キーが無い・Null・空の文字列 ----

#[test]
fn test_ce_18_missing_key_adds_obsidian_block_form() {
    // [CE-18] キーが無いノートに [a] → `tags:` と `  - a` の2行が閉じの区切りの前に足される。
    let before = "---\ntitle: t\nstatus: draft\n---\nbody\n";
    let after = apply_list(before, "tags", &["a"]);
    assert_eq!(
        after,
        "---\ntitle: t\nstatus: draft\ntags:\n  - a\n---\nbody\n"
    );
    assert_reads_back(before, &after, "tags", &["a"]);

    // 2つの要素。
    assert_eq!(
        apply_list(before, "tags", &["a", "b"]),
        "---\ntitle: t\nstatus: draft\ntags:\n  - a\n  - b\n---\nbody\n"
    );
}

#[test]
fn test_ce_18_missing_key_crlf() {
    // [CE-18] [WB-1] 足す行の改行コードはファイルに合わせる。
    let before = "---\r\ntitle: t\r\n---\r\nbody\r\n";
    assert_eq!(
        apply_list(before, "tags", &["a"]),
        "---\r\ntitle: t\r\ntags:\r\n  - a\r\n---\r\nbody\r\n"
    );
}

#[test]
fn test_ce_18_null_and_empty_string_become_block_form() {
    // [CE-18] `tags:`(Null)・`tags: ""` のノートに [a] → `tags:` の次の行から `  - a`。
    for before in [
        "---\ntags:\ntitle: t\n---\nbody\n",
        "---\ntags: \"\"\ntitle: t\n---\nbody\n",
    ] {
        let after = apply_list(before, "tags", &["a"]);
        assert_eq!(
            after, "---\ntags:\n  - a\ntitle: t\n---\nbody\n",
            "from {before:?}"
        );
        assert_reads_back(before, &after, "tags", &["a"]);
    }
    // 最後のキーが Null でも同じ。
    assert_eq!(
        apply_list("---\ntitle: t\ntags:\n---\n", "tags", &["a", "b"]),
        "---\ntitle: t\ntags:\n  - a\n  - b\n---\n"
    );
}

// ---- CE-19: 全部外す・クオート・書けない形 ----

#[test]
fn test_ce_19_remove_all_keeps_key() {
    // [CE-19] [CE-9] 全部外す(空の Vec)→ 縦のリストも1行のフローも `tags:` になり、キーは残る。
    assert_eq!(
        apply_list(BLOCK2, "tags", &[]),
        "---\ntitle: t\ntags:\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list(BLOCK0, "tags", &[]),
        "---\ntitle: t\ntags:\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list(
            "---\ntitle: t\ntags: [a, b]\nstatus: draft\n---\nbody\n",
            "tags",
            &[]
        ),
        "---\ntitle: t\ntags:\nstatus: draft\n---\nbody\n"
    );
    assert_eq!(
        apply_list("---\r\ntags:\r\n  - a\r\n---\r\nbody\r\n", "tags", &[]),
        "---\r\ntags:\r\n---\r\nbody\r\n"
    );
    let after = apply_list(BLOCK2, "tags", &[]);
    let fm = parse(after.as_bytes()).unwrap();
    assert_eq!(
        value_of(&fm, "tags"),
        Some(Value::Null),
        "key is kept as null"
    );
}

#[test]
fn test_ce_19_risky_elements_are_quoted() {
    // [CE-19] [WB-7] `no`・`123`・`a: b`・`#x` のような要素はクオートされ、読み直すと文字列。
    let risky = [
        "no", "yes", "on", "123", "1.5", "a: b", "#x", "x #y", "true", "null", "[z]",
    ];
    let mut items = vec!["a"];
    items.extend(risky);
    for before in [
        "---\ntags: [a]\nstatus: draft\n---\nbody\n",
        "---\ntags:\n  - a\nstatus: draft\n---\nbody\n",
        "---\ntags:\nstatus: draft\n---\nbody\n",
        "---\nstatus: draft\n---\nbody\n",
    ] {
        let after = apply_list(before, "tags", &items);
        assert_reads_back(before, &after, "tags", &items);
    }
    // 仕様の例: `no` を足す → `"no"` と書かれる。
    let after = apply_list("---\ntags: [a]\n---\n", "tags", &["a", "no"]);
    assert!(after.contains("\"no\""), "{after:?}");
    let after = apply_list("---\ntags:\n  - a\n---\n", "tags", &["a", "no"]);
    assert!(after.contains("  - \"no\"\n"), "{after:?}");
}

#[test]
fn test_ce_19_newline_in_element_is_rejected() {
    // [CE-19] [WB-7] 改行を含む要素は受け付けない。
    let r = apply(
        b"---\ntags: [a]\n---\n",
        &[edit("tags", list(&["a", "x\ny"]))],
    );
    assert_eq!(r, Err(EditError::Newline));
}

#[test]
fn test_ce_19_ce_8_list_forms_are_not_editable() {
    // [CE-19] [CE-8] 複数行のフロー・アンカーや別名・入れ子の要素・要素の中のコメントのリストは書かない。
    let cases = [
        "---\ntags: [a,\n  b]\n---\n",
        "---\ntags: &t [a, b]\n---\n",
        "---\ntags: &t\n  - a\n  - b\n---\n",
        "---\nbase: &t [a, b]\ntags: *t\n---\n",
        "---\ntags:\n  - {k: v}\n---\n",
        "---\ntags:\n  - a\n  - {k: v}\n---\n",
        "---\ntags:\n  - k: v\n---\n",
        "---\ntags:\n  - - x\n---\n",
        "---\ntags:\n  - [x, y]\n---\n",
        "---\ntags:\n  - a # memo\n  - b\n---\n",
    ];
    for src in cases {
        let r = apply(src.as_bytes(), &[edit("tags", list(&["a", "c"]))]);
        assert!(
            matches!(r, Err(EditError::NotEditable(ref k)) if k == "tags"),
            "{src:?}: {r:?}"
        );
        // 全部外すのも書かない。
        let r = apply(src.as_bytes(), &[edit("tags", list(&[]))]);
        assert!(
            matches!(r, Err(EditError::NotEditable(_))),
            "{src:?} (empty): {r:?}"
        );
    }
}

// ---- save と読み直しの検査(WB-6) ----

#[test]
fn test_ce_18_save_writes_same_form_and_passes_reread_check() {
    // [CE-18] [WB-6] save でも apply と同じ書き方で書かれ、読み直しの検査を通る。
    let dir = TempDir::new("list-save");
    let cases: Vec<(&str, &str, Vec<&str>, &str)> = vec![
        (
            "flow.md",
            "---\ntags: [a, b]\nstatus: draft\n---\nbody\n",
            vec!["a", "b", "c"],
            "---\ntags: [a, b, c]\nstatus: draft\n---\nbody\n",
        ),
        (
            "block.md",
            BLOCK2,
            vec!["a", "b", "c"],
            "---\ntitle: t\ntags:\n  - a\n  - b\n  - c\nstatus: draft\n---\nbody\n",
        ),
        (
            "missing.md",
            "---\ntitle: t\n---\nbody\n",
            vec!["a"],
            "---\ntitle: t\ntags:\n  - a\n---\nbody\n",
        ),
        (
            "null.md",
            "---\ntags:\ntitle: t\n---\nbody\n",
            vec!["a"],
            "---\ntags:\n  - a\ntitle: t\n---\nbody\n",
        ),
        (
            "clear.md",
            BLOCK0,
            vec![],
            "---\ntitle: t\ntags:\nstatus: draft\n---\nbody\n",
        ),
    ];
    for (name, before, items, want) in &cases {
        let p = dir.write(name, before.as_bytes());
        let base = baseline(&p).unwrap();
        save(&p, &base, &[edit("tags", list(items))])
            .unwrap_or_else(|e| panic!("{name}: save: {e:?}"));
        assert_eq!(read(&p), *want, "{name}");
    }

    // 書けない形は save でも書かず、ファイルはそのまま。
    let bad = "---\ntags: [a,\n  b]\n---\nbody\n";
    let p = dir.write("bad.md", bad.as_bytes());
    let base = baseline(&p).unwrap();
    assert!(save(&p, &base, &[edit("tags", list(&["a"]))]).is_err());
    assert_eq!(read(&p), bad);
}

#[test]
fn test_ce_18_source_save_and_preview_write_list() {
    // [CE-18] [WB-6] Source 越しの preview と save でも同じ書き方。保存のあと get はリストの値。
    let dir = TempDir::new("list-source-save");
    let p = dir.write("a.md", BLOCK2.as_bytes());
    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    let row = row_of(src, "a.md");
    let want = "---\ntitle: t\ntags:\n  - a\n  - b\n  - c\nstatus: draft\n---\nbody\n";

    let (before, after) = src
        .preview(&row, &[edit("tags", list(&["a", "b", "c"]))])
        .expect("preview");
    assert_eq!(before, BLOCK2.as_bytes());
    assert_eq!(String::from_utf8(after).unwrap(), want);

    let base = src.stamp(&row).unwrap();
    src.save(&row, &base, &[edit("tags", list(&["a", "b", "c"]))])
        .expect("save");
    assert_eq!(read(&p), want);
    assert_eq!(
        src.get(&row, "tags").value,
        Some(str_list(&["a", "b", "c"]))
    );
}

// ---- CE-16: 候補 ----

#[test]
fn test_ce_16_list_candidates_counts_and_order() {
    // [CE-16] [CE-19] 候補は要素と件数。件数の多い順、同じ件数は文字の順。
    // tags は先頭の # を除く。渡したフォルダの外(根の下)のノートの要素も入る(BV-2)。
    let dir = TempDir::new("list-cands");
    std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
    dir.write("outside.md", b"---\ntags: [gamma, zeta]\n---\n");
    dir.write(
        "notes/a.md",
        b"---\ntags: [gamma, alpha]\nkinds: [\"#k\", m]\n---\n",
    );
    dir.write(
        "notes/b.md",
        b"---\ntags:\n  - gamma\n  - beta\nkinds: [m]\n---\n",
    );
    dir.write("notes/c.md", b"---\ntags: [\"#alpha\", beta]\n---\n");
    dir.write("notes/d.md", b"---\nstatus: draft\n---\n");

    let mut md = open(&dir.path().join("notes"));
    load_all(&mut md);
    let src: &dyn Source = &md;

    // 行は渡したフォルダの下だけ(BV-1)。候補はそれより広い。
    assert_eq!(src.rows().len(), 4);
    assert_eq!(
        src.list_candidates("tags"),
        cands(&[("gamma", 3), ("alpha", 2), ("beta", 2), ("zeta", 1)])
    );
    // tags 以外のリストの列では # を除かない。
    assert_eq!(src.list_candidates("kinds"), cands(&[("m", 2), ("#k", 1)]));
    // 誰も持たない列は空。
    assert!(src.list_candidates("nothing").is_empty());
}

#[test]
fn test_ce_16_list_candidates_follow_saved_values() {
    // [CE-16] 保存で足した要素は、そのあとの候補に入る(候補は今ある要素)。
    let dir = TempDir::new("list-cands-save");
    dir.write("a.md", b"---\ntags: [x]\n---\n");
    dir.write("b.md", b"---\ntags: [x, y]\n---\n");
    let mut md = open(dir.path());
    let src: &mut dyn Source = &mut md;
    load_all(src);
    assert_eq!(src.list_candidates("tags"), cands(&[("x", 2), ("y", 1)]));

    let row = row_of(src, "a.md");
    let base = src.stamp(&row).unwrap();
    src.save(&row, &base, &[edit("tags", list(&["x", "y", "新規"]))])
        .expect("save");
    assert_eq!(
        src.list_candidates("tags"),
        cands(&[("x", 2), ("y", 2), ("新規", 1)])
    );
}

// ---- CE-16・CE-8: Cell.lock ----

#[test]
fn test_ce_16_writable_list_cells_have_no_lock() {
    // [CE-16] リストの値のセルは、書ける形なら lock なし。
    // [CE-8] 書けない形(複数行のフロー・アンカー・入れ子の要素)は理由つきで読むだけ。
    let dir = TempDir::new("list-lock");
    dir.write("flow.md", b"---\ntags: [a, b]\n---\n");
    dir.write("empty_flow.md", b"---\ntags: []\n---\n");
    dir.write("block2.md", BLOCK2.as_bytes());
    dir.write("block0.md", BLOCK0.as_bytes());
    dir.write("quoted.md", b"---\ntags:\n  - \"a\"\n  - 'b'\n---\n");
    dir.write("crlf.md", b"---\r\ntags:\r\n  - a\r\n---\r\n");
    dir.write("multi.md", b"---\ntags: [a,\n  b]\n---\n");
    dir.write("anchor.md", b"---\ntags: &t [a, b]\n---\n");
    dir.write("nested.md", b"---\ntags:\n  - {k: v}\n---\n");
    dir.write("nested2.md", b"---\ntags:\n  - k: v\n---\n");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    for note in [
        "flow.md",
        "empty_flow.md",
        "block2.md",
        "block0.md",
        "quoted.md",
        "crlf.md",
    ] {
        let c = cell(src, note, "tags");
        assert!(
            matches!(c.value, Some(Value::List(_))),
            "{note}: {:?}",
            c.value
        );
        assert!(
            c.lock.is_none(),
            "{note}: writable list is locked: {:?}",
            c.lock
        );
    }
    for note in ["multi.md", "anchor.md", "nested.md", "nested2.md"] {
        let c = cell(src, note, "tags");
        let reason = c
            .lock
            .unwrap_or_else(|| panic!("{note}: CE-8 list form must be locked"));
        assert!(!reason.is_empty(), "{note}: lock has a reason");
    }
}

// ---- WB-17 ----

#[test]
fn test_wb_17_list_set_back_to_original_is_not_pending() {
    // [WB-17] [CE-16] リストを直してから元と同じ並びに戻す → ためない。今と同じ並びを入れる → ためない。
    // 並びが違えば別の値としてためる。
    let note = "---\ntitle: t\ntags:\n  - x\n  - y\n---\nbody\n";
    let dir = TempDir::new("list-wb17");
    let p = dir.write("a.md", note.as_bytes());
    let mut md = open(dir.path());
    load_all(&mut md);
    let a = row_of(&md, "a.md");
    let mut ch = Changes::new();

    let _ = ch.set(&md, &a, "tags", list(&["x", "y"]));
    assert_eq!(ch.count(), 0, "[WB-17] 今と同じ並びをためた");
    assert!(ch.pending(&a, "tags").is_none());

    assert!(ch.set(&md, &a, "tags", list(&["x", "y", "z"])).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(ch.set(&md, &a, "tags", list(&["x", "y"])).is_ok());
    assert_eq!(ch.count(), 0, "[WB-17] 元の並びに戻したのに未保存に数える");
    assert!(ch.pending(&a, "tags").is_none());
    assert!(ch.previews(&md).is_empty());

    assert!(ch.set(&md, &a, "tags", list(&["y", "x"])).is_ok());
    assert_eq!(ch.count(), 1, "並びが違えば別の値");

    // 保存すると並べ替えが書かれる。
    let _ = ch.save(&mut md);
    assert_eq!(read(&p), "---\ntitle: t\ntags:\n  - y\n  - x\n---\nbody\n");
}

#[test]
fn test_wb_17_list_on_missing_or_null_is_pending() {
    // [WB-17] キーが無い・Null のセルに空でないリスト → ためる。Null のセルに空のリスト → 書いても
    // `tags:` のままなので、ためない扱いでも保存でバイトが変わらないことを確かめる。
    let dir = TempDir::new("list-wb17-null");
    let pm = dir.write("m.md", b"---\ntitle: t\n---\n");
    let pn = dir.write("n.md", b"---\ntags:\ntitle: t\n---\n");
    let mut md = open(dir.path());
    load_all(&mut md);
    let m = row_of(&md, "m.md");
    let n = row_of(&md, "n.md");
    let mut ch = Changes::new();

    assert!(ch.set(&md, &m, "tags", list(&["a"])).is_ok());
    assert_eq!(ch.count(), 1);
    let _ = ch.set(&md, &n, "tags", list(&[]));
    let _ = ch.save(&mut md);
    assert_eq!(read(&pm), "---\ntitle: t\ntags:\n  - a\n---\n");
    assert_eq!(read(&pn), "---\ntags:\ntitle: t\n---\n");
}

#[test]
fn test_wb_17_empty_list_on_missing_key_is_not_pending() {
    // [WB-17] [CE-18] キーの無いセルに空の並びを set → `key:` を足さず、ためない(count 0)。
    let dir = TempDir::new("list-wb17-missing-empty");
    let p = dir.write("m.md", b"---\ntitle: t\n---\n");
    let mut md = open(dir.path());
    load_all(&mut md);
    let m = row_of(&md, "m.md");
    let mut ch = Changes::new();
    let _ = ch.set(&md, &m, "tags", list(&[]));
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&m, "tags").is_none());
    let _ = ch.save(&mut md);
    assert_eq!(read(&p), "---\ntitle: t\n---\n");
}
