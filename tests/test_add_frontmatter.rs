//! フロントマターの無いノートに入力したらフロントマターを足す(add-frontmatter)の受け入れテスト。
//! WB-3・WB-1・WB-5・WB-6・WB-7・WB-18・CE-19・CE-9・CE-8。
//! 仕様: specs/write-back/spec.md の WB-3・WB-1、specs/cell-edit/spec.md の CE-10。
//! 記録: specs/_changes/2026-10-02-add-frontmatter.md。決定: specs/_decisions/2026-10-02-add-frontmatter.md。
//!
//! 実装を見ずに、仕様と公開のインターフェース(writeback::apply・save・save_with・Source)だけから書いた。

use mdgrid::frontmatter::{parse, Value};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source};
use mdgrid::writeback::{apply, baseline, save, save_with, Edit, Faults, NewValue};
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
        std::fs::write(&p, bytes).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 道具 ----

const BODY: &str = "# 見出し\n本文\n";

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn todo() -> Vec<Edit> {
    vec![edit("status", s("todo"))]
}

fn prio() -> Edit {
    edit("priority", NewValue::Int(2))
}

/// status に todo、priority に 2(この順)。
fn two() -> Vec<Edit> {
    vec![edit("status", s("todo")), prio()]
}

/// apply して、結果を文字列で返す(UTF-8 でなければ落とす)。
fn applied(original: &[u8], edits: &[Edit]) -> String {
    let out = apply(original, edits).unwrap_or_else(|e| {
        panic!(
            "apply failed on {:?}: {e:?}",
            String::from_utf8_lossy(original)
        )
    });
    String::from_utf8(out).expect("result is UTF-8")
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

// ---- apply: 先頭に足す ----

#[test]
fn test_wb_3_body_only_note_gets_frontmatter() {
    // [WB-3] 本文だけの `# 見出し\n本文\n` の status に todo → 先頭に3行、本文はそのまま。
    // [WB-1] 足すのは先頭のフロントマターの行だけ(元のバイトは後ろにそのまま)。
    let out = applied(BODY.as_bytes(), &todo());
    assert_eq!(out, "---\nstatus: todo\n---\n# 見出し\n本文\n");
    assert!(out.ends_with(BODY), "[WB-1] body bytes kept");
}

#[test]
fn test_wb_3_crlf_note_gets_crlf_frontmatter() {
    // [WB-3] [WB-1] CRLF のノート → 足す3行も CRLF、本文はそのまま。
    let before = "# 見出し\r\n本文\r\n";
    let out = applied(before.as_bytes(), &todo());
    assert_eq!(out, "---\r\nstatus: todo\r\n---\r\n# 見出し\r\n本文\r\n");
    let head = &out[..out.len() - before.len()];
    assert!(
        !head.replace("\r\n", "").contains('\n'),
        "[WB-1] no bare LF in added lines: {head:?}"
    );
}

#[test]
fn test_wb_3_no_newline_file_gets_lf() {
    // [WB-3] 改行の無いファイル(`本文`)→ LF で足し、本文はそのまま(末尾に改行を足さない)。
    let out = applied("本文".as_bytes(), &todo());
    assert_eq!(out, "---\nstatus: todo\n---\n本文");
}

#[test]
fn test_wb_3_empty_file_gets_frontmatter() {
    // [WB-3] 空のファイル → `---\nstatus: todo\n---\n`。
    let out = applied(b"", &todo());
    assert_eq!(out, "---\nstatus: todo\n---\n");
}

#[test]
fn test_wb_3_two_keys_one_frontmatter_in_written_order() {
    // [WB-3] 1回の apply に2つの編集 → フロントマターは1つで2行、書いた順。
    let out = applied(BODY.as_bytes(), &two());
    assert_eq!(out, "---\nstatus: todo\npriority: 2\n---\n# 見出し\n本文\n");
    assert_eq!(out.matches("---").count(), 2, "one frontmatter: {out:?}");

    // 逆の順に書けば逆の順。
    let out = applied(BODY.as_bytes(), &[prio(), edit("status", s("todo"))]);
    assert_eq!(out, "---\npriority: 2\nstatus: todo\n---\n# 見出し\n本文\n");

    // CRLF でも1つのフロントマターで2行、どれも CRLF。
    let out = applied("# 見出し\r\n本文\r\n".as_bytes(), &two());
    assert_eq!(
        out,
        "---\r\nstatus: todo\r\npriority: 2\r\n---\r\n# 見出し\r\n本文\r\n"
    );
}

#[test]
fn test_wb_3_list_value_is_vertical() {
    // [WB-3] [CE-19] [CE-18] リストの値 → Obsidian の縦の形(`tags:` と `  - a` の行)。
    let out = applied(
        BODY.as_bytes(),
        &[edit("tags", NewValue::List(vec!["a".into(), "b".into()]))],
    );
    assert_eq!(out, "---\ntags:\n  - a\n  - b\n---\n# 見出し\n本文\n");

    // CRLF のノートでは要素の行も CRLF。
    let out = applied(
        "# 見出し\r\n本文\r\n".as_bytes(),
        &[edit("tags", NewValue::List(vec!["a".into()]))],
    );
    assert_eq!(out, "---\r\ntags:\r\n  - a\r\n---\r\n# 見出し\r\n本文\r\n");

    // [CE-19] 空のリスト → `tags:`。
    let out = applied(BODY.as_bytes(), &[edit("tags", NewValue::List(vec![]))]);
    assert_eq!(out, "---\ntags:\n---\n# 見出し\n本文\n");
}

#[test]
fn test_wb_3_date_value_is_unquoted() {
    // [WB-3] [WB-18] 日付 → 囲まずに `due: 2026-11-03`。
    let out = applied(
        BODY.as_bytes(),
        &[edit("due", NewValue::Date("2026-11-03".into()))],
    );
    assert_eq!(out, "---\ndue: 2026-11-03\n---\n# 見出し\n本文\n");
}

#[test]
fn test_wb_3_null_value_is_bare_key() {
    // [WB-3] [CE-9] Null → `status:`。
    let out = applied(BODY.as_bytes(), &[edit("status", NewValue::Null)]);
    assert_eq!(out, "---\nstatus:\n---\n# 見出し\n本文\n");
}

#[test]
fn test_wb_3_string_is_quoted_like_wb_7() {
    // [WB-3] [WB-7] 型が変わる文字列(`no`)は、足すときも二重引用符で囲む。真偽値と数も素のまま。
    // (キー名は YAML 1.1 の真偽値の語(`n` など)を避ける。キー名の囲み方はここでは見ない。)
    let out = applied(BODY.as_bytes(), &[edit("status", s("no"))]);
    assert_eq!(out, "---\nstatus: \"no\"\n---\n# 見出し\n本文\n");
    let flags = [
        edit("done", NewValue::Bool(true)),
        edit("k", NewValue::Int(3)),
    ];
    let out = applied(BODY.as_bytes(), &flags);
    assert_eq!(out, "---\ndone: true\nk: 3\n---\n# 見出し\n本文\n");
}

#[test]
fn test_wb_3_dashes_later_in_body_are_body() {
    // [WB-3] 本文の後ろの行に `---` があるだけ(1行目でない)のノートは、本文として扱って足せる。
    let before = "# 見出し\n---\n本文\n---\n";
    let out = applied(before.as_bytes(), &todo());
    assert_eq!(out, "---\nstatus: todo\n---\n# 見出し\n---\n本文\n---\n");
    let fm = parse(out.as_bytes()).expect("reparse");
    let keys: Vec<&str> = fm.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, ["status"], "body dashes are not keys");
}

#[test]
fn test_wb_3_added_frontmatter_reparses_to_written_values() {
    // [WB-3] [WB-6] 足したあと読み直すと、書いた値が読める(他にキーは無い)。
    let out = applied(
        BODY.as_bytes(),
        &[
            edit("status", s("todo")),
            edit("priority", NewValue::Int(2)),
            edit("tags", NewValue::List(vec!["a".into(), "b".into()])),
            edit("memo", NewValue::Null),
        ],
    );
    let fm = parse(out.as_bytes()).expect("reparse added frontmatter");
    let got: Vec<(String, Value)> = fm
        .entries
        .iter()
        .map(|e| (e.key.clone(), e.value.clone()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("status".to_string(), Value::Str("todo".into())),
            ("priority".to_string(), Value::Int(2)),
            (
                "tags".to_string(),
                Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
            ),
            ("memo".to_string(), Value::Null),
        ]
    );
    assert!(out.ends_with(BODY));
}

// ---- apply: これまでどおり書かない(WB-5) ----

#[test]
fn test_wb_3_unreadable_notes_still_refused() {
    // [WB-3] [WB-5] BOM で始まる本文だけのノート・UTF-8 でないファイル・1行目が `---` で閉じないノートは、
    // これまでどおり書かない(Err)。
    let cases: [(&str, Vec<u8>); 4] = [
        ("bom body", "\u{FEFF}# 見出し\n本文\n".as_bytes().to_vec()),
        ("not utf-8", b"# \xff\xfe heading\nbody\n".to_vec()),
        ("unclosed", "---\n# 見出し\n本文\n".as_bytes().to_vec()),
        ("unclosed with key", b"---\nstatus: x\nbody\n".to_vec()),
    ];
    for (name, bytes) in cases {
        let r = apply(&bytes, &todo());
        assert!(r.is_err(), "[WB-5] {name}: must not write, got {r:?}");
    }
    // [WB-3] 空のフロントマター(`---\n---\n`)は読めないノートではなく、区切りの間に1行足して書ける
    // (2026-10-02 empty-frontmatter-config。詳しくは tests/test_empty_frontmatter.rs)。
    let out = applied("---\n---\n本文\n".as_bytes(), &todo());
    assert_eq!(out, "---\nstatus: todo\n---\n本文\n");
}

// ---- save と Source ----

#[test]
fn test_wb_3_save_adds_frontmatter_and_keeps_body() {
    // [WB-3] [WB-6] save で書ける。本文はそのまま、読み直しの検査を通る。
    // 読み直しの検査に落ちる失敗を差し込めば、元のファイルは変わらない。
    let dir = TempDir::new("addfm-save");
    let p = dir.write("a.md", BODY.as_bytes());
    let base = baseline(&p).unwrap();
    save(&p, &base, &todo()).expect("save adds frontmatter");
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "---\nstatus: todo\n---\n# 見出し\n本文\n"
    );

    let q = dir.write("b.md", BODY.as_bytes());
    let base = baseline(&q).unwrap();
    let faults = Faults {
        corrupt: true,
        ..Faults::default()
    };
    let r = save_with(&q, &base, &todo(), &faults);
    assert!(r.is_err(), "[WB-6] corrupted write must be refused");
    assert_eq!(
        std::fs::read_to_string(&q).unwrap(),
        BODY,
        "[WB-6] original unchanged"
    );
}

#[test]
fn test_wb_3_source_cell_of_no_frontmatter_note_is_writable() {
    // [WB-3] フロントマターの無いノートと空のフロントマターのノートのセルは、既定で lock なし(値は無し)。
    // ただし file.* の列は lock あり(CE-8)。BOM・閉じないノートはこれまでどおり lock あり(WB-5)。
    let dir = TempDir::new("addfm-source");
    dir.write("a.md", b"---\nstatus: done\n---\nbody\n");
    dir.write("plain.md", BODY.as_bytes());
    dir.write("empty.md", b"");
    dir.write("bom.md", "\u{FEFF}# 見出し\n本文\n".as_bytes());
    dir.write("unclosed.md", b"---\nstatus: x\nbody\n");
    dir.write("emptyfm.md", b"---\n---\nbody\n");

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &dyn Source = &md;

    for note in ["plain.md", "empty.md", "emptyfm.md"] {
        let row = row_of(src, note);
        let c = src.get(&row, "status");
        assert_eq!(c.value, None, "{note}: no value");
        assert!(c.lock.is_none(), "{note}: writable, got {:?}", c.lock);
        let f = src.get(&row, "file.name");
        assert!(f.lock.is_some(), "{note}: file.* stays locked");
    }
    for note in ["bom.md", "unclosed.md"] {
        let c = src.get(&row_of(src, note), "status");
        let reason = c
            .lock
            .unwrap_or_else(|| panic!("[WB-5] {note}: must stay locked"));
        assert!(!reason.is_empty(), "{note}: lock has a reason");
    }
}

#[test]
fn test_wb_3_source_preview_save_and_reread() {
    // [WB-3] Source で: 差分の前後 → 保存 → 読み直すと値が読め、ファイルは期待のバイト。
    let dir = TempDir::new("addfm-source-save");
    let p = dir.write("plain.md", BODY.as_bytes());
    let q = dir.write("crlf.md", "# 見出し\r\n本文\r\n".as_bytes());

    let mut md = open(dir.path());
    load_all(&mut md);
    let src: &mut dyn Source = &mut md;

    let row = row_of(src, "plain.md");
    let edits = two();
    let (before, after) = src.preview(&row, &edits).expect("preview");
    assert_eq!(before, BODY.as_bytes());
    assert_eq!(
        String::from_utf8(after).unwrap(),
        "---\nstatus: todo\npriority: 2\n---\n# 見出し\n本文\n"
    );
    let stamp = src.stamp(&row).expect("stamp");
    src.save(&row, &stamp, &edits).expect("save via Source");
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "---\nstatus: todo\npriority: 2\n---\n# 見出し\n本文\n"
    );
    let c = src.get(&row, "status");
    assert_eq!(c.value, Some(Value::Str("todo".into())));
    assert!(c.lock.is_none());
    assert_eq!(src.get(&row, "priority").value, Some(Value::Int(2)));

    let row = row_of(src, "crlf.md");
    let stamp = src.stamp(&row).expect("stamp");
    let saved = src.save(&row, &stamp, &todo());
    saved.expect("save crlf via Source");
    assert_eq!(
        std::fs::read_to_string(&q).unwrap(),
        "---\r\nstatus: todo\r\n---\r\n# 見出し\r\n本文\r\n"
    );
    assert_eq!(
        src.get(&row, "status").value,
        Some(Value::Str("todo".into()))
    );
}
