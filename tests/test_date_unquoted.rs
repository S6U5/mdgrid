//! 日付の列を囲まずに書く(date-unquoted)の受け入れテスト(WB-18・WB-17・WB-6・WB-7・CE-22)。
//! 仕様: specs/write-back/spec.md の WB-18、specs/cell-edit/spec.md の CE-22。
//! 形: specs/_changes/2026-10-02-date-unquoted.md の「設計」と docs/design.md「書き戻し(write-back)」の
//! `NewValue::Date(String)`。

use mdgrid::changes::{Changes, Outcome};
use mdgrid::frontmatter::{parse, Value};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source};
use mdgrid::writeback::{apply, baseline, save, Edit, NewValue};
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

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn date(v: &str) -> NewValue {
    NewValue::Date(v.to_string())
}

fn apply_str(note: &str, key: &str, value: NewValue) -> String {
    let out = apply(note.as_bytes(), &[edit(key, value)]).expect("apply");
    String::from_utf8(out).unwrap()
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

fn assert_reads_as(bytes: &[u8], key: &str, want: &str) {
    match value_of(bytes, key) {
        Value::Str(v) => assert_eq!(v, want, "[WB-18] 読み直した値が違う"),
        other => panic!("[WB-18] 読み直した値が文字列でない: {other:?}"),
    }
}

fn setup_with(name: &str, notes: &[(&str, &str)]) -> (TempDir, Markdown) {
    let dir = TempDir::new(name);
    for (n, body) in notes {
        dir.write(n, body.as_bytes());
    }
    let mut src = Markdown::open(&[dir.path().to_path_buf()]).expect("open");
    let mut guard = 0;
    loop {
        let p = src.load(100);
        if p.done {
            break;
        }
        guard += 1;
        assert!(guard < 10_000, "load が終わらない");
    }
    (dir, src)
}

fn row(src: &Markdown, name: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == name)
        .unwrap_or_else(|| panic!("行 {name} が無い"))
}

fn read(dir: &TempDir, name: &str) -> String {
    String::from_utf8(std::fs::read(dir.path().join(name)).unwrap()).unwrap()
}

// ---- apply ----

#[test]
fn test_wb_18_plain_date_stays_unquoted() {
    // [WB-18] `due: 2026-10-05` に Date("2026-11-03") → `due: 2026-11-03`。他のバイトは同じ。
    let note = "---\ntitle: A  # c\ndue: 2026-10-05\ntags:\n  - x\n---\nbody\n";
    let out = apply_str(note, "due", date("2026-11-03"));
    assert_eq!(
        out, "---\ntitle: A  # c\ndue: 2026-11-03\ntags:\n  - x\n---\nbody\n",
        "[WB-18] 素の日付を囲んだか、他のバイトが変わった"
    );
    // [WB-6] 読み直すと値の文字列は 2026-11-03
    assert_reads_as(out.as_bytes(), "due", "2026-11-03");
}

#[test]
fn test_wb_18_quoted_date_keeps_quotes() {
    // [WB-18] 元が囲んであれば元の引用符を保つ。
    let dq = apply_str("---\ndue: \"2026-10-05\"\n---\n", "due", date("2026-11-03"));
    assert_eq!(dq, "---\ndue: \"2026-11-03\"\n---\n", "[WB-18] 二重引用符");
    assert_reads_as(dq.as_bytes(), "due", "2026-11-03");

    let sq = apply_str("---\ndue: '2026-10-05'\n---\n", "due", date("2026-11-03"));
    assert_eq!(sq, "---\ndue: '2026-11-03'\n---\n", "[WB-18] 一重引用符");
    assert_reads_as(sq.as_bytes(), "due", "2026-11-03");
}

#[test]
fn test_wb_18_missing_key_adds_unquoted_line() {
    // [WB-18] キーの無いノート → `due: 2026-11-03` の1行が足される(囲まない)。
    let note = "---\ntitle: A\nstatus: todo\n---\nbody\n";
    let out = apply_str(note, "due", date("2026-11-03"));
    let added: Vec<&str> = out.lines().filter(|l| l.starts_with("due")).collect();
    assert_eq!(added, vec!["due: 2026-11-03"], "[WB-18] 足した行: {out:?}");
    // [WB-1] 足した1行を除けば元と同じ
    assert_eq!(
        out.replacen("due: 2026-11-03\n", "", 1),
        note,
        "[WB-18] 1行を足す以外の変更がある"
    );
    assert_reads_as(out.as_bytes(), "due", "2026-11-03");
}

#[test]
fn test_wb_18_null_and_empty_written_unquoted() {
    // [WB-18] `due:`(Null)と `due: ""` → 囲まない。
    let null = apply_str("---\ndue:\ntitle: A\n---\n", "due", date("2026-11-03"));
    assert_eq!(
        null, "---\ndue: 2026-11-03\ntitle: A\n---\n",
        "[WB-18] Null"
    );
    assert_reads_as(null.as_bytes(), "due", "2026-11-03");

    let empty = apply_str("---\ndue: \"\"\ntitle: A\n---\n", "due", date("2026-11-03"));
    assert_eq!(
        empty, "---\ndue: 2026-11-03\ntitle: A\n---\n",
        "[WB-18] 空の文字列"
    );
    assert_reads_as(empty.as_bytes(), "due", "2026-11-03");
}

#[test]
fn test_wb_18_datetime_unquoted() {
    // [WB-18] 日時 `at: 2026-10-05T09:00` に Date("2026-11-03T09:00") → 囲まない。
    let out = apply_str(
        "---\nat: 2026-10-05T09:00\n---\n",
        "at",
        date("2026-11-03T09:00"),
    );
    assert_eq!(out, "---\nat: 2026-11-03T09:00\n---\n", "[WB-18] 日時");
    assert_reads_as(out.as_bytes(), "at", "2026-11-03T09:00");
}

#[test]
fn test_wb_18_not_a_date_is_error() {
    // [WB-18] 日付の形に合わない Date は Err(書かない)。
    let note = "---\ndue: 2026-10-05\n---\n";
    for bad in ["someday", "2026-02-30", "", "2026-11-03 # x", "2026-13-01"] {
        assert!(
            apply(note.as_bytes(), &[edit("due", date(bad))]).is_err(),
            "[WB-18] Date({bad:?}) を受け付けた"
        );
    }
}

#[test]
fn test_wb_18_text_column_still_quoted() {
    // [WB-18][WB-7] Str("2026-11-03") をテキストの列に当てる → これまでどおり囲む。
    let out = apply_str(
        "---\nmemo: hello\n---\n",
        "memo",
        NewValue::Str("2026-11-03".into()),
    );
    assert_eq!(
        out, "---\nmemo: \"2026-11-03\"\n---\n",
        "[WB-7] テキストの列"
    );
    assert_reads_as(out.as_bytes(), "memo", "2026-11-03");

    let missing = apply_str(
        "---\ntitle: A\n---\n",
        "memo",
        NewValue::Str("2026-11-03".into()),
    );
    assert!(
        missing.contains("memo: \"2026-11-03\"\n"),
        "[WB-7] キーの無いノートのテキストの列: {missing:?}"
    );
}

// ---- save ----

#[test]
fn test_wb_18_save_writes_unquoted() {
    // [WB-18][WB-6] save でも素の日付は囲まずに書き、読み直しの検査を通る。
    let dir = TempDir::new("wb18save");
    let plain = dir.write("plain.md", b"---\ndue: 2026-10-05\ntitle: A\n---\nbody\n");
    let base = baseline(&plain).unwrap();
    save(&plain, &base, &[edit("due", date("2026-11-03"))]).expect("[WB-6] save");
    assert_eq!(
        read(&dir, "plain.md"),
        "---\ndue: 2026-11-03\ntitle: A\n---\nbody\n"
    );
    assert_reads_as(&std::fs::read(&plain).unwrap(), "due", "2026-11-03");

    let quoted = dir.write("quoted.md", b"---\ndue: \"2026-10-05\"\n---\n");
    let base = baseline(&quoted).unwrap();
    save(&quoted, &base, &[edit("due", date("2026-11-03"))]).expect("[WB-6] save");
    assert_eq!(read(&dir, "quoted.md"), "---\ndue: \"2026-11-03\"\n---\n");

    let missing = dir.write("missing.md", b"---\ntitle: A\n---\n");
    let base = baseline(&missing).unwrap();
    save(&missing, &base, &[edit("due", date("2026-11-03"))]).expect("[WB-6] save");
    assert_eq!(
        read(&dir, "missing.md"),
        "---\ntitle: A\ndue: 2026-11-03\n---\n"
    );

    // [WB-18] 日付でない Date は save も止め、ファイルは変わらない
    let bad = dir.write("bad.md", b"---\ndue: 2026-10-05\n---\n");
    let base = baseline(&bad).unwrap();
    assert!(save(&bad, &base, &[edit("due", date("someday"))]).is_err());
    assert_eq!(read(&dir, "bad.md"), "---\ndue: 2026-10-05\n---\n");
}

// ---- Changes を通した保存 ----

#[test]
fn test_wb_18_changes_save_date_unquoted() {
    // [WB-18][CE-22] Markdown を開いて Changes で Date を set して save → ファイルは囲まない形。
    let d = "---\ndue: 2026-10-05\ntitle: D\n---\nbody d\n";
    let q = "---\ndue: \"2026-10-05\"\ntitle: Q\n---\nbody q\n";
    let (dir, src) = setup_with("wb18changes", &[("d.md", d), ("q.md", q)]);
    let mut src = src;
    let rd = row(&src, "d.md");
    let rq = row(&src, "q.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &rd, "due", date("2026-11-03")).is_ok());
    assert!(ch.set(&src, &rq, "due", date("2026-11-03")).is_ok());
    assert_eq!(ch.count(), 2);

    let res = ch.save(&mut src);
    for r in [&rd, &rq] {
        let o = &res.iter().find(|(id, _)| id == r).expect("保存の結果").1;
        assert!(matches!(o, Outcome::Saved), "[WB-18] 保存に失敗");
    }
    assert_eq!(
        read(&dir, "d.md"),
        "---\ndue: 2026-11-03\ntitle: D\n---\nbody d\n",
        "[WB-18] 素の日付を囲んだ"
    );
    assert_eq!(
        read(&dir, "q.md"),
        "---\ndue: \"2026-11-03\"\ntitle: Q\n---\nbody q\n",
        "[WB-18] 元の引用符を保たない"
    );
}

#[test]
fn test_wb_17_same_date_not_pending() {
    // [WB-17] 元と同じ日付を Date で set → ためない。別の日付にしてから戻しても、ためない。
    let d = "---\ndue: 2026-10-05\ntitle: D\n---\nbody d\n";
    let (dir, src) = setup_with("wb17date18", &[("d.md", d)]);
    let mut src = src;
    let rd = row(&src, "d.md");
    let mut ch = Changes::new();

    let _ = ch.set(&src, &rd, "due", date("2026-10-05"));
    assert_eq!(ch.count(), 0, "[WB-17] 元と同じ日付をためた");
    assert!(ch.pending(&rd, "due").is_none());

    assert!(ch.set(&src, &rd, "due", date("2026-11-03")).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(ch.set(&src, &rd, "due", date("2026-10-05")).is_ok());
    assert_eq!(ch.count(), 0, "[WB-17] 元の日付に戻したのに未保存に数える");

    let res = ch.save(&mut src);
    assert!(res.is_empty());
    assert_eq!(read(&dir, "d.md"), d, "[WB-17] 書かれた");
}
