//! タスク 16 の受け入れテスト(失敗を仕込む: WB-6・WB-8)。
//! 仕様: specs/write-back/spec.md の WB-6・WB-8。
//! 形: docs/design.md の「核の公開のインターフェース」の `Faults` と `save_with`。

use mdgrid::frontmatter::{parse, Value};
use mdgrid::writeback::{baseline, save, save_with, Edit, EditError, Faults, NewValue, SaveError};
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

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
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

    fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(&self.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn only_note() -> Vec<String> {
    vec!["note.md".to_string()]
}

// ---- 失敗なし(Faults::default) ----

#[test]
fn wb_8_save_with_default_faults_writes_like_save() {
    // [WB-8] Faults::default() の save_with は save と同じく書け、一時ファイルを残さない。
    let dir = TempDir::new("faults-default");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    save_with(
        &note,
        &base,
        &[edit("status", s("done"))],
        &Faults::default(),
    )
    .expect("save_with");
    let written = std::fs::read(&note).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&written),
        "---\ntitle: t\nstatus: done\n---\nbody\n"
    );
    assert_eq!(dir.names(), only_note());

    // save で書いた結果と同じバイトになる。
    let dir2 = TempDir::new("faults-default-save");
    let note2 = dir2.write("note.md", &fixture("plain.md"));
    let base2 = baseline(&note2).unwrap();
    save(&note2, &base2, &[edit("status", s("done"))]).expect("save");
    assert_eq!(std::fs::read(&note2).unwrap(), written);
}

#[test]
fn wb_8_save_with_default_faults_returns_usable_baseline() {
    // [WB-8] 返った新しい基準で続けて save_with できる。
    let dir = TempDir::new("faults-default-chain");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let next = save_with(
        &note,
        &base,
        &[edit("status", s("done"))],
        &Faults::default(),
    )
    .expect("first save_with");
    save_with(&note, &next, &[edit("title", s("u"))], &Faults::default())
        .expect("second save_with");
    let written = std::fs::read(&note).unwrap();
    assert_eq!(value_of(&written, "status"), Value::Str("done".into()));
    assert_eq!(value_of(&written, "title"), Value::Str("u".into()));
    assert_eq!(dir.names(), only_note());
}

// ---- WB-6: 読み直しの検査が落ちる ----

#[test]
fn wb_6_corrupt_fails_verify_and_keeps_original() {
    // [WB-6] 一時ファイルの値を1バイト壊すと読み直しの検査で止まり、元のファイルは同じバイト、一時ファイルは残らない。
    let dir = TempDir::new("wb6-corrupt");
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    let base = baseline(&note).unwrap();
    let faults = Faults {
        corrupt: true,
        ..Faults::default()
    };
    let r = save_with(&note, &base, &[edit("status", s("done"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Edit(EditError::Verify(_)))),
        "corrupt must be caught by the verify step"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), only_note());
}

#[test]
fn wb_6_corrupt_with_trailing_comment_keeps_original() {
    // [WB-6] 行末コメントのある材料でも、壊れた一時ファイルで元のファイルを置き換えない。
    let dir = TempDir::new("wb6-corrupt-rich");
    let original = fixture("trailing_comment.md");
    let note = dir.write("note.md", &original);
    let key = parse(&original)
        .expect("parse")
        .entries
        .iter()
        .find(|e| e.span.is_some())
        .expect("an editable key")
        .key
        .clone();
    let base = baseline(&note).unwrap();
    let faults = Faults {
        corrupt: true,
        ..Faults::default()
    };
    let r = save_with(&note, &base, &[edit(&key, s("changed"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Edit(EditError::Verify(_)))),
        "corrupt must be caught by the verify step"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), only_note());
}

#[test]
fn wb_6_after_corrupt_failure_a_clean_save_succeeds() {
    // [WB-6] 失敗のあと、元の基準のまま失敗なしで保存し直せる(元のファイルが変わっていない)。
    let dir = TempDir::new("wb6-retry");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let faults = Faults {
        corrupt: true,
        ..Faults::default()
    };
    assert!(save_with(&note, &base, &[edit("status", s("done"))], &faults).is_err());
    save_with(
        &note,
        &base,
        &[edit("status", s("done"))],
        &Faults::default(),
    )
    .expect("retry with the same baseline");
    assert_eq!(
        String::from_utf8_lossy(&std::fs::read(&note).unwrap()),
        "---\ntitle: t\nstatus: done\n---\nbody\n"
    );
    assert_eq!(dir.names(), only_note());
}

#[test]
fn wb_6_misquote_fails_verify_and_keeps_original() {
    // [WB-6] 値の書き方を誤らせる(文字列の `true` をクオートせず書く → 真偽値に読める)と、
    // 結果の読み直しの検査(対象のキーが意図した値か)で止まり、元のファイルは同じバイト、一時ファイルは残らない。
    let dir = TempDir::new("wb6-misquote");
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    let base = baseline(&note).unwrap();
    let faults = Faults {
        misquote: true,
        ..Faults::default()
    };
    let r = save_with(&note, &base, &[edit("status", s("true"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Edit(EditError::Verify(_)))),
        "a misquoted value must be caught by the verify step, got {r:?}"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), only_note());

    // 誤らせなければ、同じ基準で `true` は文字列のまま書ける。
    save_with(
        &note,
        &base,
        &[edit("status", s("true"))],
        &Faults::default(),
    )
    .expect("clean save");
    assert_eq!(
        value_of(&std::fs::read(&note).unwrap(), "status"),
        Value::Str("true".into())
    );
}

#[test]
fn wb_6_misquote_on_added_key_fails_verify() {
    // [WB-6] 足すキーでも、誤った書き方は読み直しの検査で止まる。
    let dir = TempDir::new("wb6-misquote-add");
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    let base = baseline(&note).unwrap();
    let faults = Faults {
        misquote: true,
        ..Faults::default()
    };
    let r = save_with(&note, &base, &[edit("flag", s("123"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Edit(EditError::Verify(_)))),
        "a misquoted added value must be caught by the verify step, got {r:?}"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), only_note());
}

// ---- WB-12: 名前を変える直前の再検査 ----

#[test]
fn wb_12_change_just_before_rename_stops_save() {
    // [WB-12] fsync のあと名前を変える前に外で書き換わる → 直前の再検査で Changed。
    // 外の内容が残り、一時ファイルは残らない。
    let dir = TempDir::new("wb12-before-rename");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let outside = b"---\ntitle: t\nstatus: changed outside\n---\nbody\n".to_vec();
    let faults = Faults {
        before_rename: Some(outside.clone()),
        ..Faults::default()
    };
    let r = save_with(&note, &base, &[edit("status", s("done"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Changed)),
        "a change just before rename must stop the save, got {r:?}"
    );
    assert_eq!(std::fs::read(&note).unwrap(), outside);
    assert_eq!(dir.names(), only_note());
}

// ---- WB-8: 書き込み・fsync の途中の失敗 ----

fn assert_io_failure_keeps_original(name: &str, faults: Faults) {
    let dir = TempDir::new(name);
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    let base = baseline(&note).unwrap();
    let r = save_with(&note, &base, &[edit("status", s("done"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Io(_))),
        "{name}: an injected io failure must be SaveError::Io"
    );
    assert_eq!(
        std::fs::read(&note).unwrap(),
        original,
        "{name}: original changed"
    );
    assert_eq!(dir.names(), only_note(), "{name}: temp file left behind");
}

#[test]
fn wb_8_fail_write_keeps_original_and_no_temp() {
    // [WB-8] 一時ファイルへの書き込みの途中で落ちても、元のファイルは同じで、一時ファイルは残らない。
    assert_io_failure_keeps_original(
        "wb8-fail-write",
        Faults {
            fail_write: true,
            ..Faults::default()
        },
    );
}

#[test]
fn wb_8_fail_fsync_keeps_original_and_no_temp() {
    // [WB-8] fsync で落ちても、元のファイルは同じで、一時ファイルは残らない。
    assert_io_failure_keeps_original(
        "wb8-fail-fsync",
        Faults {
            fail_fsync: true,
            ..Faults::default()
        },
    );
}

#[cfg(unix)]
fn assert_io_failure_keeps_permissions(name: &str, faults: Faults) {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new(name);
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    std::fs::set_permissions(&note, std::fs::Permissions::from_mode(0o640)).unwrap();
    let base = baseline(&note).unwrap();
    let r = save_with(&note, &base, &[edit("status", s("done"))], &faults);
    assert!(
        matches!(r, Err(SaveError::Io(_))),
        "{name}: expected SaveError::Io"
    );
    let mode = std::fs::metadata(&note).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode, 0o640, "{name}: permissions changed");
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), only_note());
}

#[cfg(unix)]
#[test]
fn wb_8_fail_write_keeps_permissions() {
    // [WB-8] 書き込みの失敗のあとも、元のファイルの権限は保たれる。
    assert_io_failure_keeps_permissions(
        "wb8-fail-write-perm",
        Faults {
            fail_write: true,
            ..Faults::default()
        },
    );
}

#[cfg(unix)]
#[test]
fn wb_8_fail_fsync_keeps_permissions() {
    // [WB-8] fsync の失敗のあとも、元のファイルの権限は保たれる。
    assert_io_failure_keeps_permissions(
        "wb8-fail-fsync-perm",
        Faults {
            fail_fsync: true,
            ..Faults::default()
        },
    );
}

#[cfg(unix)]
#[test]
fn wb_8_fail_fsync_on_symlink_keeps_link_and_target() {
    // [WB-8] シンボリックリンクのノートで fsync が落ちても、リンクはリンクのまま、先のファイルは同じ。
    let dir = TempDir::new("wb8-fsync-link");
    let original = fixture("plain.md");
    let target = dir.write("target.md", &original);
    let link = dir.path().join("link.md");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let base = baseline(&link).unwrap();
    let faults = Faults {
        fail_fsync: true,
        ..Faults::default()
    };
    let r = save_with(&link, &base, &[edit("status", s("done"))], &faults);
    assert!(matches!(r, Err(SaveError::Io(_))), "expected SaveError::Io");
    let meta = std::fs::symlink_metadata(&link).unwrap();
    assert!(meta.file_type().is_symlink(), "link must remain a symlink");
    assert_eq!(std::fs::read_link(&link).unwrap(), target);
    assert_eq!(std::fs::read(&target).unwrap(), original);
    assert_eq!(
        dir.names(),
        vec!["link.md".to_string(), "target.md".to_string()]
    );
}

#[test]
fn wb_8_after_io_failure_a_clean_save_succeeds() {
    // [WB-8] 書き込みの失敗のあと、元の基準のまま失敗なしで保存し直せる。
    let dir = TempDir::new("wb8-retry");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let faults = Faults {
        fail_write: true,
        ..Faults::default()
    };
    assert!(save_with(&note, &base, &[edit("status", s("done"))], &faults).is_err());
    save_with(
        &note,
        &base,
        &[edit("status", s("done"))],
        &Faults::default(),
    )
    .expect("retry with the same baseline");
    assert_eq!(
        String::from_utf8_lossy(&std::fs::read(&note).unwrap()),
        "---\ntitle: t\nstatus: done\n---\nbody\n"
    );
    assert_eq!(dir.names(), only_note());
}
