//! 書き戻しの受け入れテスト(CE-8・WB-1〜WB-8・WB-12)。仕様: specs/write-back/spec.md、specs/cell-edit/spec.md。

use mdgrid::frontmatter::{parse, Frontmatter, ReadOnly, Value};
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

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn value_of(fm: &Frontmatter, key: &str) -> Value {
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("key {key} not found"))
        .value
        .clone()
}

fn span_of(bytes: &[u8], key: &str) -> std::ops::Range<usize> {
    let fm = parse(bytes).expect("parse");
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("key {key} not found"))
        .span
        .clone()
        .unwrap_or_else(|| panic!("key {key} has no span"))
}

/// 前後の共通の先頭と末尾を除いた、元のバイト列の中の変わった範囲。
fn changed_range(before: &[u8], after: &[u8]) -> std::ops::Range<usize> {
    let max = before.len().min(after.len());
    let mut p = 0;
    while p < max && before[p] == after[p] {
        p += 1;
    }
    let mut q = 0;
    while q < max - p && before[before.len() - 1 - q] == after[after.len() - 1 - q] {
        q += 1;
    }
    p..before.len() - q
}

/// 対象のキーを書いた前後の差分が、そのキーの値の範囲の中だけであることを確かめる。
fn assert_only_value_changed(file: &str, key: &str, value: NewValue) -> Vec<u8> {
    let before = fixture(file);
    let span = span_of(&before, key);
    let after = apply(&before, &[edit(key, value)]).expect("apply");
    assert_ne!(before, after, "{file}: nothing changed");
    let changed = changed_range(&before, &after);
    assert!(
        changed.start >= span.start && changed.end <= span.end,
        "{file}: changed {changed:?} is outside the value span {span:?}"
    );
    after
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

// ---- WB-1 ----

#[test]
fn wb_1_changes_only_the_value_range() {
    // [WB-1]
    let after = assert_only_value_changed("rich.md", "status", s("done"));
    let expected = String::from_utf8(fixture("rich.md"))
        .unwrap()
        .replace("status: draft  #", "status: done  #");
    assert_eq!(after, expected.into_bytes());
}

#[test]
fn wb_1_block_list_neighbour() {
    // [WB-1]
    let after = assert_only_value_changed("block_list.md", "status", s("done"));
    assert_eq!(
        after,
        b"---\ntags:\n  - one\n  - two\nstatus: done\n---\nbody\n"
    );
}

#[test]
fn wb_1_flow_list_neighbour() {
    // [WB-1]
    let after = assert_only_value_changed("flow_list.md", "status", s("done"));
    assert_eq!(after, b"---\ntags: [one, two]\nstatus: done\n---\nbody\n");
}

#[test]
fn wb_1_nested_map_neighbour() {
    // [WB-1]
    let after = assert_only_value_changed("nested_map.md", "status", s("done"));
    assert_eq!(
        after,
        b"---\nmeta:\n  owner: me\n  level: 2\nstatus: done\n---\nbody\n"
    );
}

#[test]
fn wb_1_block_scalar_neighbour() {
    // [WB-1]
    let after = assert_only_value_changed("block_scalar.md", "status", s("done"));
    assert_eq!(
        after,
        b"---\nliteral: |\n  line one\n  line two\nfolded: >\n  folded one\n  folded two\nstatus: done\n---\nbody\n"
    );
}

#[test]
fn wb_1_keeps_trailing_comment() {
    // [WB-1]
    let after = assert_only_value_changed("trailing_comment.md", "status", s("done"));
    assert_eq!(
        after,
        "---\nstatus: done # コメント\ntitle: t\n---\nbody\n".as_bytes()
    );
}

#[test]
fn wb_1_keeps_crlf() {
    // [WB-1]
    let after = assert_only_value_changed("crlf.md", "status", s("done"));
    assert_eq!(after, b"---\r\ntitle: t\r\nstatus: done\r\n---\r\nbody\r\n");
}

#[test]
fn wb_1_japanese_key() {
    // [WB-1]
    let after = assert_only_value_changed("japanese_key.md", "状態", s("完了"));
    assert_eq!(
        after,
        "---\n状態: 完了\nタイトル: 日本語\n---\n本文\n".as_bytes()
    );
}

#[test]
fn wb_1_dash_lines_in_body() {
    // [WB-1]
    let after = assert_only_value_changed("dash_line.md", "status", s("done"));
    assert_eq!(
        after,
        b"---\ntitle: t\nstatus: done\n---\nbody\n----\n---\nstatus: body\n----\n"
    );
}

#[test]
fn wb_1_quoted_and_special_values() {
    // [WB-1]
    assert_only_value_changed("quoted.md", "dq", s("done"));
    assert_only_value_changed("quoted.md", "sq", s("done"));
    assert_only_value_changed("special_chars.md", "colon", s("c: d"));
    assert_only_value_changed("special_chars.md", "hash", s("c #d"));
    assert_only_value_changed("number_like.md", "num", s("456"));
    assert_only_value_changed("number_like.md", "count", NewValue::Int(6));
}

// ---- WB-2 ----

#[test]
fn wb_2_body_is_unchanged() {
    // [WB-2]
    for (file, key) in [
        ("dash_line.md", "status"),
        ("rich.md", "status"),
        ("crlf.md", "status"),
        ("japanese_key.md", "状態"),
    ] {
        let before = fixture(file);
        let end_before = parse(&before).unwrap().end;
        let after = apply(&before, &[edit(key, s("done"))]).expect("apply");
        let end_after = parse(&after).unwrap().end;
        assert_eq!(&after[end_after..], &before[end_before..], "{file}");
    }
}

#[test]
fn wb_2_new_key_does_not_touch_body() {
    // [WB-2]
    let before = fixture("dash_line.md");
    let end_before = parse(&before).unwrap().end;
    let after = apply(&before, &[edit("priority", NewValue::Int(1))]).expect("apply");
    let end_after = parse(&after).unwrap().end;
    assert_eq!(&after[end_after..], &before[end_before..]);
}

// ---- WB-3 ----

#[test]
fn wb_3_appends_missing_key_as_one_line() {
    // [WB-3]
    let after = apply(&fixture("plain.md"), &[edit("priority", NewValue::Int(1))]).expect("apply");
    assert_eq!(
        after,
        b"---\ntitle: t\nstatus: draft\npriority: 1\n---\nbody\n"
    );
}

#[test]
fn wb_3_appended_line_uses_file_newline() {
    // [WB-3]
    let after = apply(&fixture("crlf.md"), &[edit("priority", s("high"))]).expect("apply");
    assert_eq!(
        after,
        b"---\r\ntitle: t\r\nstatus: draft\r\npriority: high\r\n---\r\nbody\r\n"
    );
}

#[test]
fn wb_3_no_frontmatter_gets_frontmatter_but_unreadable_is_not_written() {
    // [WB-3] フロントマターの無いノートは先頭に `---`・キー・`---` を足し、本文のバイトは変えない。
    let original = fixture("no_frontmatter.md");
    let out = apply(&original, &[edit("status", s("done"))]).expect("apply adds frontmatter");
    let mut want = b"---\nstatus: done\n---\n".to_vec();
    want.extend_from_slice(&original);
    assert_eq!(out, want);
    // [WB-3] [WB-5] 1行目が `---` で始まるが閉じないノートは、これまでどおり書かない。
    let r = apply(b"---\nstatus: doing\nbody\n", &[edit("status", s("done"))]);
    assert_eq!(r, Err(EditError::ReadOnly(ReadOnly::Unclosed)));
}

// ---- WB-5 ----

#[test]
fn wb_5_apply_refuses_read_only_notes() {
    // [WB-5]
    let cases = [
        ("duplicate_key.md", "a", ReadOnly::DuplicateKey("a".into())),
        ("bom.md", "status", ReadOnly::Bom),
        ("invalid_yaml.md", "status", ReadOnly::InvalidYaml),
        ("unclosed.md", "status", ReadOnly::Unclosed),
        ("not_utf8.md", "status", ReadOnly::NotUtf8),
        ("mixed_newlines.md", "status", ReadOnly::MixedNewlines),
    ];
    for (file, key, reason) in cases {
        let r = apply(&fixture(file), &[edit(key, s("done"))]);
        assert_eq!(r, Err(EditError::ReadOnly(reason)), "{file}");
    }
    // [WB-3] 空のフロントマターは WB-5 の形から外れ、書ける(区切りの間に1行足す)。
    let r = apply(
        &fixture("empty_frontmatter.md"),
        &[edit("status", s("done"))],
    );
    assert_eq!(r, Ok(b"---\nstatus: done\n---\nbody\n".to_vec()));
}

#[cfg(unix)]
#[test]
fn wb_5_hard_linked_note_is_not_written() {
    // [WB-5]
    let dir = TempDir::new("hardlink");
    let original = fixture("plain.md");
    let note = dir.write("note.md", &original);
    std::fs::hard_link(&note, dir.path().join("other.md")).unwrap();
    let base = baseline(&note).unwrap();
    let r = save(&note, &base, &[edit("status", s("done"))]);
    assert!(r.is_err(), "save must refuse a hard-linked note");
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(
        std::fs::read(dir.path().join("other.md")).unwrap(),
        original
    );
}

// ---- CE-8 ----

#[test]
fn ce_8_apply_refuses_read_only_values() {
    // [CE-8]
    for (file, key) in [
        ("nested_map.md", "meta"),
        ("block_scalar.md", "literal"),
        ("block_scalar.md", "folded"),
        ("anchor.md", "base"),
        ("anchor.md", "alias"),
        ("block_list.md", "tags"),
    ] {
        let r = apply(&fixture(file), &[edit(key, s("x"))]);
        assert_eq!(
            r,
            Err(EditError::NotEditable(key.to_string())),
            "{file}: {key}"
        );
    }
}

// ---- WB-6 ----

#[test]
fn wb_6_reparse_has_intended_value_and_others_unchanged() {
    // [WB-6]
    let cases: Vec<(&str, &str, NewValue, Value)> = vec![
        ("rich.md", "status", s("done"), Value::Str("done".into())),
        ("rich.md", "priority", NewValue::Int(7), Value::Int(7)),
        (
            "rich.md",
            "priority",
            NewValue::Float(1.5),
            Value::Float(1.5),
        ),
        (
            "rich.md",
            "status",
            NewValue::Bool(false),
            Value::Bool(false),
        ),
        ("rich.md", "status", NewValue::Null, Value::Null),
        (
            "number_like.md",
            "flag",
            NewValue::Bool(false),
            Value::Bool(false),
        ),
        ("quoted.md", "dq", s("a: b"), Value::Str("a: b".into())),
        (
            "japanese_key.md",
            "状態",
            s("完了"),
            Value::Str("完了".into()),
        ),
        ("crlf.md", "status", s("done"), Value::Str("done".into())),
        ("plain.md", "新しいキー", s("値"), Value::Str("値".into())),
    ];
    for (file, key, new, expected) in cases {
        let before = fixture(file);
        let fm_before = parse(&before).unwrap();
        let after = apply(&before, &[edit(key, new)]).expect("apply");
        let fm_after = parse(&after).expect("reparse");
        assert_eq!(value_of(&fm_after, key), expected, "{file}: {key}");
        for e in &fm_before.entries {
            if e.key != key {
                assert_eq!(
                    value_of(&fm_after, &e.key),
                    e.value,
                    "{file}: {} changed",
                    e.key
                );
            }
        }
    }
}

#[test]
fn wb_6_multiple_edits_in_one_apply() {
    // [WB-6]
    let before = fixture("rich.md");
    let fm_before = parse(&before).unwrap();
    let after = apply(
        &before,
        &[
            edit("status", s("done")),
            edit("priority", NewValue::Int(1)),
        ],
    )
    .expect("apply");
    let fm_after = parse(&after).unwrap();
    assert_eq!(value_of(&fm_after, "status"), Value::Str("done".into()));
    assert_eq!(value_of(&fm_after, "priority"), Value::Int(1));
    assert_eq!(value_of(&fm_after, "title"), value_of(&fm_before, "title"));
    assert_eq!(value_of(&fm_after, "tags"), value_of(&fm_before, "tags"));
    assert_eq!(value_of(&fm_after, "meta"), value_of(&fm_before, "meta"));
}

// ---- WB-7 ----

const RISKY: &[&str] = &[
    "no",
    "yes",
    "on",
    "off",
    "No",
    "YES",
    "y",
    "n",
    "true",
    "false",
    "null",
    "~",
    "123",
    "-1",
    "1.5",
    "1e3",
    "0x1F",
    "0o17",
    "012",
    ".inf",
    "a: b",
    "a #b",
    "-x",
    "*x",
    "&x",
    "!x",
    "@x",
    "`x",
    "[x",
    "]x",
    "{x",
    "}x",
    "%x",
    "|x",
    ">x",
    "?x",
    "#x",
    ",x",
    "'x",
    "\"x",
    "2026-09-30",
];

#[test]
fn wb_7_risky_strings_are_quoted_and_read_back_as_str() {
    // [WB-7]
    let before = fixture("plain.md");
    for v in RISKY {
        let after = apply(&before, &[edit("status", s(v))]).expect("apply");
        let span = span_of(&after, "status");
        let first = after[span.start];
        assert_eq!(
            first,
            b'"',
            "{v:?} must be double-quoted, got {:?}",
            String::from_utf8_lossy(&after[span])
        );
        let fm = parse(&after).unwrap();
        assert_eq!(value_of(&fm, "status"), Value::Str(v.to_string()), "{v:?}");
    }
}

#[test]
fn wb_7_no_is_written_as_double_quoted() {
    // [WB-7]
    let after = apply(&fixture("plain.md"), &[edit("status", s("no"))]).expect("apply");
    assert_eq!(after, b"---\ntitle: t\nstatus: \"no\"\n---\nbody\n");
}

#[test]
fn wb_7_keeps_double_quotes() {
    // [WB-7]
    let after = apply(&fixture("quoted.md"), &[edit("dq", s("done"))]).expect("apply");
    assert_eq!(
        after,
        b"---\ndq: \"done\"\nsq: 'draft'\ntitle: t\n---\nbody\n"
    );
    for v in RISKY {
        let after = apply(&fixture("quoted.md"), &[edit("dq", s(v))]).expect("apply");
        let span = span_of(&after, "dq");
        assert_eq!(after[span.start], b'"', "{v:?}");
        assert_eq!(
            value_of(&parse(&after).unwrap(), "dq"),
            Value::Str(v.to_string()),
            "{v:?}"
        );
    }
}

#[test]
fn wb_7_keeps_single_quotes() {
    // [WB-7]
    let after = apply(&fixture("quoted.md"), &[edit("sq", s("done"))]).expect("apply");
    assert_eq!(
        after,
        b"---\ndq: \"draft\"\nsq: 'done'\ntitle: t\n---\nbody\n"
    );
    let after = apply(&fixture("quoted.md"), &[edit("sq", s("no"))]).expect("apply");
    assert_eq!(
        value_of(&parse(&after).unwrap(), "sq"),
        Value::Str("no".into())
    );
}

#[test]
fn wb_7_plain_safe_string_stays_plain() {
    // [WB-7]
    let after = apply(&fixture("plain.md"), &[edit("status", s("done"))]).expect("apply");
    assert_eq!(after, b"---\ntitle: t\nstatus: done\n---\nbody\n");
}

#[test]
fn wb_7_newline_is_rejected() {
    // [WB-7]
    for v in ["a\nb", "a\r\nb", "a\rb", "\n"] {
        let r = apply(&fixture("plain.md"), &[edit("status", s(v))]);
        assert_eq!(r, Err(EditError::Newline), "{v:?}");
    }
}

// ---- WB-4 ----

#[test]
fn wb_4_external_change_stops_save() {
    // [WB-4]
    let dir = TempDir::new("wb4-size");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let external = b"---\ntitle: changed outside\nstatus: draft\n---\nbody\n";
    std::fs::write(&note, external).unwrap();
    let r = save(&note, &base, &[edit("status", s("done"))]);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), external);
}

#[test]
fn wb_4_same_size_same_mtime_change_is_detected_by_hash() {
    // [WB-4]
    let dir = TempDir::new("wb4-hash");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let mtime = std::fs::metadata(&note).unwrap().modified().unwrap();
    // 同じ大きさの別の中身にし、更新時刻を元に戻す。
    let external = b"---\ntitle: T\nstatus: draft\n---\nbody\n";
    assert_eq!(external.len(), fixture("plain.md").len());
    std::fs::write(&note, external).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&note)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    let r = save(&note, &base, &[edit("status", s("done"))]);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), external);
}

#[test]
fn wb_4_unchanged_file_saves() {
    // [WB-4]
    let dir = TempDir::new("wb4-ok");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", s("done"))]).expect("save");
    assert_eq!(
        std::fs::read(&note).unwrap(),
        b"---\ntitle: t\nstatus: done\n---\nbody\n"
    );
}

// ---- WB-12 ----

#[test]
fn wb_12_second_save_uses_new_baseline() {
    // [WB-12]
    let dir = TempDir::new("wb12");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    let base2 = save(&note, &base, &[edit("status", s("done"))]).expect("first save");
    assert_eq!(base2, baseline(&note).unwrap());
    let base3 = save(&note, &base2, &[edit("title", s("new"))]).expect("second save");
    assert_eq!(base3, baseline(&note).unwrap());
    assert_eq!(
        std::fs::read(&note).unwrap(),
        b"---\ntitle: new\nstatus: done\n---\nbody\n"
    );
}

#[test]
fn wb_12_old_baseline_is_stale_after_save() {
    // [WB-12]
    let dir = TempDir::new("wb12-stale");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", s("done"))]).expect("first save");
    let r = save(&note, &base, &[edit("title", s("new"))]);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
}

// ---- WB-8 ----

#[test]
fn wb_8_leaves_no_temp_file() {
    // [WB-8]
    let dir = TempDir::new("wb8-tmp");
    let note = dir.write("note.md", &fixture("plain.md"));
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", s("done"))]).expect("save");
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[test]
fn wb_8_failed_save_leaves_original_intact() {
    // [WB-8]
    let dir = TempDir::new("wb8-fail");
    let original = fixture("duplicate_key.md");
    let note = dir.write("note.md", &original);
    let base = baseline(&note).unwrap();
    let r = save(&note, &base, &[edit("a", s("3"))]);
    assert!(
        matches!(
            r,
            Err(SaveError::Edit(EditError::ReadOnly(
                ReadOnly::DuplicateKey(_)
            )))
        ),
        "{r:?}"
    );
    assert_eq!(std::fs::read(&note).unwrap(), original);
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[cfg(unix)]
#[test]
fn wb_8_symlink_stays_symlink() {
    // [WB-8]
    let dir = TempDir::new("wb8-link");
    let target = dir.write("target.md", &fixture("plain.md"));
    let link = dir.path().join("link.md");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let base = baseline(&link).unwrap();
    save(&link, &base, &[edit("status", s("done"))]).expect("save");
    let meta = std::fs::symlink_metadata(&link).unwrap();
    assert!(meta.file_type().is_symlink(), "link must remain a symlink");
    assert_eq!(std::fs::read_link(&link).unwrap(), target);
    assert_eq!(
        std::fs::read(&target).unwrap(),
        b"---\ntitle: t\nstatus: done\n---\nbody\n"
    );
    assert_eq!(
        dir.names(),
        vec!["link.md".to_string(), "target.md".to_string()]
    );
}

#[cfg(unix)]
#[test]
fn wb_8_keeps_permissions() {
    // [WB-8]
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("wb8-perm");
    let note = dir.write("note.md", &fixture("plain.md"));
    std::fs::set_permissions(&note, std::fs::Permissions::from_mode(0o640)).unwrap();
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", s("done"))]).expect("save");
    let mode = std::fs::metadata(&note).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode, 0o640);
}
