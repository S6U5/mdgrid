//! タスク 15 の受け入れテスト(CE-9・WB-5 のハードリンク)。
//! 仕様: specs/cell-edit/spec.md の CE-9、specs/write-back/spec.md の WB-5。

use mdgrid::frontmatter::{parse, ReadOnly, Value};
use mdgrid::writeback::{apply, baseline, save, Edit, EditError, NewValue, SaveError};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn value_of(bytes: &[u8], key: &str) -> Value {
    let fm = parse(bytes).expect("parse");
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("key {key} not found"))
        .value
        .clone()
}

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

    #[cfg_attr(not(unix), allow(dead_code))]
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

// ---- CE-9 ----

const DONE: &[u8] = b"---\ntitle: t\nstatus: done\npriority: 3\n---\nbody\n";

#[test]
fn ce_9_null_writes_key_with_nothing_after_colon() {
    // [CE-9] 値を空にすると `status:` の行が残り、他のバイトは同じ。
    let after = apply(DONE, &[edit("status", NewValue::Null)]).expect("apply");
    assert_eq!(
        String::from_utf8_lossy(&after),
        "---\ntitle: t\nstatus:\npriority: 3\n---\nbody\n"
    );
}

#[test]
fn ce_9_null_keeps_key_and_reads_back_as_null() {
    // [CE-9] キーは消えず、読み直すと Null。他のキーの値は変わらない。
    let after = apply(DONE, &[edit("status", NewValue::Null)]).expect("apply");
    assert_eq!(value_of(&after, "status"), Value::Null);
    assert_eq!(value_of(&after, "title"), Value::Str("t".into()));
    assert_eq!(value_of(&after, "priority"), Value::Int(3));
    let keys: Vec<String> = parse(&after)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.key.clone())
        .collect();
    assert_eq!(keys, ["title", "status", "priority"]);
}

#[test]
fn ce_9_null_on_fixture_plain() {
    // [CE-9] 材料の plain.md(status: draft)でも `status:` になる。
    let after = apply(&fixture("plain.md"), &[edit("status", NewValue::Null)]).expect("apply");
    assert_eq!(
        String::from_utf8_lossy(&after),
        "---\ntitle: t\nstatus:\n---\nbody\n"
    );
}

#[test]
fn ce_9_null_keeps_crlf() {
    // [CE-9] CRLF の材料でも行末の改行コードは保たれる。
    let after = apply(&fixture("crlf.md"), &[edit("status", NewValue::Null)]).expect("apply");
    assert_eq!(
        String::from_utf8_lossy(&after),
        "---\r\ntitle: t\r\nstatus:\r\n---\r\nbody\r\n"
    );
    assert_eq!(value_of(&after, "status"), Value::Null);
}

#[test]
fn ce_9_null_for_missing_key_appends_key_line() {
    // [CE-9] キーの無いノートに Null を足すと `key:` の1行が足される。
    let after = apply(&fixture("plain.md"), &[edit("due", NewValue::Null)]).expect("apply");
    assert_eq!(
        String::from_utf8_lossy(&after),
        "---\ntitle: t\nstatus: draft\ndue:\n---\nbody\n"
    );
    assert_eq!(value_of(&after, "due"), Value::Null);
}

#[test]
fn ce_9_null_for_missing_key_keeps_crlf() {
    // [CE-9] CRLF の材料に足す `key:` の行も CRLF で終わる。
    let after = apply(&fixture("crlf.md"), &[edit("due", NewValue::Null)]).expect("apply");
    assert_eq!(
        String::from_utf8_lossy(&after),
        "---\r\ntitle: t\r\nstatus: draft\r\ndue:\r\n---\r\nbody\r\n"
    );
}

#[test]
fn ce_9_save_writes_key_with_nothing_after_colon() {
    // [CE-9] save 経由でもファイルが `status:` になる。
    let dir = TempDir::new("ce9-save");
    let note = dir.write("note.md", DONE);
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", NewValue::Null)]).expect("save");
    let written = std::fs::read(&note).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&written),
        "---\ntitle: t\nstatus:\npriority: 3\n---\nbody\n"
    );
    assert_eq!(value_of(&written, "status"), Value::Null);
}

// ---- WB-5 ----

#[cfg(unix)]
#[test]
fn wb_5_save_refuses_hard_linked_note_with_reason() {
    // [WB-5] ハードリンクのあるノートは書かず、理由は ReadOnly::HardLink。両方のファイルが元のまま。
    let dir = TempDir::new("wb5-hardlink");
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    let other = dir.path().join("other.md");
    std::fs::hard_link(&note, &other).unwrap();
    let base = baseline(&note).unwrap();
    let r = save(
        &note,
        &base,
        &[edit("status", NewValue::Str("done".into()))],
    );
    assert!(
        matches!(
            r,
            Err(SaveError::Edit(EditError::ReadOnly(ReadOnly::HardLink)))
        ),
        "save must refuse a hard-linked note with ReadOnly::HardLink"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(std::fs::read(&other).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn wb_5_save_refuses_hard_linked_note_even_for_null() {
    // [WB-5] [CE-9] Null の書き込みでもハードリンクのあるノートは書かない。
    let dir = TempDir::new("wb5-hardlink-null");
    let note = dir.write("note.md", DONE);
    let other = dir.path().join("other.md");
    std::fs::hard_link(&note, &other).unwrap();
    let base = baseline(&note).unwrap();
    let r = save(&note, &base, &[edit("status", NewValue::Null)]);
    assert!(matches!(
        r,
        Err(SaveError::Edit(EditError::ReadOnly(ReadOnly::HardLink)))
    ));
    assert_eq!(std::fs::read(&note).unwrap(), DONE);
    assert_eq!(std::fs::read(&other).unwrap(), DONE);
}
