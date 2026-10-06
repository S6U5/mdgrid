//! 列の型・候補・ファイルの属性の受け入れテスト(タスク 5)。
//! CE-2・CE-3・CE-5・CV-2・BV-12。
//! 仕様: specs/cell-edit/spec.md、specs/base-view/spec.md、specs/cell-view/spec.md。
//! 形: docs/design.md の「核の公開のインターフェース(残り: タスク 5・6・8)」の
//! src/types.rs と src/source.rs(kind・candidates・file)。

use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source, Value};
use mdgrid::types::{fits, format_date, infer, merge, parse_date, read_types_json, Declared, Kind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---- 一時フォルダ(tests/test_source.rs から写した) ----

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

    fn mkdir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 助け ----

fn open(folders: &[PathBuf]) -> Markdown {
    let mut md = Markdown::open(folders).expect("Markdown::open");
    load_all(&mut md);
    md
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

fn st(v: &str) -> Value {
    Value::Str(v.to_string())
}

fn kinds(pairs: &[(&str, Kind)]) -> HashMap<String, Kind> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn infer_of(values: &[Value]) -> Kind {
    infer(values.iter())
}

// ---- read_types_json ----

#[test]
fn test_ce_2_read_types_json_maps_names() {
    // [CE-2] types.json の型の名前を Kind に写す。multitext・tags・aliases は List、知らない名前は Text
    let json = br#"{"types": {
        "title": "text", "count": "number", "done": "checkbox",
        "due": "date", "at": "datetime",
        "items": "multitext", "tags": "tags", "aliases": "aliases",
        "weird": "something-unknown"
    }}"#;
    let m = read_types_json(json);
    assert_eq!(m.get("title"), Some(&Kind::Text));
    assert_eq!(m.get("count"), Some(&Kind::Number));
    assert_eq!(m.get("done"), Some(&Kind::Checkbox));
    assert_eq!(m.get("due"), Some(&Kind::Date));
    assert_eq!(m.get("at"), Some(&Kind::DateTime));
    assert_eq!(m.get("items"), Some(&Kind::List));
    assert_eq!(m.get("tags"), Some(&Kind::List));
    assert_eq!(m.get("aliases"), Some(&Kind::List));
    assert_eq!(m.get("weird"), Some(&Kind::Text));
    assert_eq!(m.len(), 9);
}

#[test]
fn test_ce_2_read_types_json_broken_is_empty() {
    // [CE-2] 読めない・壊れている types.json は空(起動は止めない)
    assert!(read_types_json(b"{\"types\": {\"due\": \"date\"").is_empty());
    assert!(read_types_json(b"not json at all").is_empty());
    assert!(read_types_json(b"").is_empty());
    assert!(read_types_json(&[0xff, 0xfe, 0x00]).is_empty());
}

// ---- infer ----

#[test]
fn test_ce_2_infer_skips_empty_values() {
    // [CE-2] Null・空の文字列・空のリストは飛ばし、最初の空でない値で決める
    let vals = vec![
        Value::Null,
        st(""),
        Value::List(vec![]),
        st("2026-10-01"),
        st("hello"),
    ];
    assert_eq!(infer_of(&vals), Kind::Date);
}

#[test]
fn test_ce_2_infer_nothing_is_text() {
    // [CE-2] 空でない値が何も無ければ Text
    assert_eq!(infer_of(&[]), Kind::Text);
    assert_eq!(infer_of(&[Value::Null, st("")]), Kind::Text);
}

#[test]
fn test_ce_2_infer_date() {
    // [CE-2] YYYY-MM-DD の文字列 → Date
    assert_eq!(infer_of(&[st("2026-10-01")]), Kind::Date);
}

#[test]
fn test_ce_2_infer_datetime() {
    // [CE-2] YYYY-MM-DDTHH:MM(秒は任意)→ DateTime
    assert_eq!(infer_of(&[st("2026-10-01T09:30")]), Kind::DateTime);
    assert_eq!(infer_of(&[st("2026-10-01T09:30:15")]), Kind::DateTime);
}

#[test]
fn test_ce_2_infer_number() {
    // [CE-2] Int・Float → Number
    assert_eq!(infer_of(&[Value::Int(3)]), Kind::Number);
    assert_eq!(infer_of(&[Value::Float(1.5)]), Kind::Number);
}

#[test]
fn test_ce_2_infer_checkbox() {
    // [CE-2] Bool → Checkbox
    assert_eq!(infer_of(&[Value::Bool(false)]), Kind::Checkbox);
}

#[test]
fn test_ce_2_infer_list() {
    // [CE-2] 空でないリスト → List
    assert_eq!(infer_of(&[Value::List(vec![st("a")])]), Kind::List);
}

#[test]
fn test_ce_2_infer_text() {
    // [CE-2] ほかの文字列 → Text。最初の空でない値だけで決める(後ろの日付は見ない)
    assert_eq!(infer_of(&[st("someday")]), Kind::Text);
    assert_eq!(infer_of(&[st("someday"), st("2026-10-01")]), Kind::Text);
}

// ---- fits ----

#[test]
fn test_cv_2_fits_date_column() {
    // [CV-2] date の列: 実在する日付は合い、someday や実在しない日付は合わない
    assert!(fits(Kind::Date, &st("2026-10-01")));
    assert!(!fits(Kind::Date, &st("someday")));
    assert!(!fits(Kind::Date, &st("2026-02-30")));
    assert!(!fits(Kind::Date, &st("2026-13-01")));
}

#[test]
fn test_cv_2_fits_null_and_empty_always() {
    // [CV-2] Null と空の文字列はどの型にも合う
    for k in [
        Kind::Text,
        Kind::Number,
        Kind::Checkbox,
        Kind::Date,
        Kind::DateTime,
        Kind::List,
    ] {
        assert!(fits(k, &Value::Null), "{k:?} Null");
        assert!(fits(k, &st("")), "{k:?} empty string");
    }
}

#[test]
fn test_cv_2_fits_other_kinds() {
    // [CV-2] 数・真偽の列に合う値と合わない値
    assert!(fits(Kind::Number, &Value::Int(1)));
    assert!(fits(Kind::Number, &Value::Float(2.5)));
    assert!(!fits(Kind::Number, &st("abc")));
    assert!(fits(Kind::Checkbox, &Value::Bool(true)));
    assert!(!fits(Kind::Checkbox, &st("abc")));
}

// ---- merge ----

#[test]
fn test_bv_12_merge_conflict() {
    // [BV-12] 同じキーで型が食い違えば Conflict。一致・片方だけなら One
    let a = kinds(&[
        ("due", Kind::Date),
        ("count", Kind::Number),
        ("only_a", Kind::Checkbox),
    ]);
    let b = kinds(&[("due", Kind::Text), ("count", Kind::Number)]);
    let m = merge(&[a, b]);
    assert!(matches!(m.get("due"), Some(Declared::Conflict)));
    assert!(matches!(m.get("count"), Some(Declared::One(Kind::Number))));
    assert!(matches!(
        m.get("only_a"),
        Some(Declared::One(Kind::Checkbox))
    ));
    assert_eq!(m.len(), 3);
}

#[test]
fn test_bv_12_merge_single_root() {
    // [BV-12] 根が1つなら全部 One
    let m = merge(&[kinds(&[("due", Kind::Date)])]);
    assert!(matches!(m.get("due"), Some(Declared::One(Kind::Date))));
}

// ---- parse_date / format_date ----

#[test]
fn test_ce_5_parse_and_format_date_roundtrip() {
    // [CE-5] 日付は 1970-01-01 からの日数。読み書きが往復する
    assert_eq!(parse_date("1970-01-01"), Some(0));
    assert_eq!(parse_date("1970-01-02"), Some(1));
    assert_eq!(parse_date("2026-10-01"), Some(20727));
    assert_eq!(format_date(0), "1970-01-01");
    assert_eq!(format_date(20727), "2026-10-01");
    for s in [
        "1999-12-31",
        "2000-02-29",
        "2024-02-29",
        "2026-01-01",
        "2026-12-31",
    ] {
        let d = parse_date(s).unwrap_or_else(|| panic!("{s} should parse"));
        assert_eq!(format_date(d), s);
    }
}

#[test]
fn test_ce_5_parse_date_rejects_nonexistent() {
    // [CE-5] 実在しない日付は None
    assert_eq!(parse_date("2026-02-30"), None);
    assert_eq!(parse_date("2026-02-29"), None);
    assert_eq!(parse_date("2100-02-29"), None);
    assert_eq!(parse_date("2026-13-01"), None);
    assert_eq!(parse_date("2026-00-10"), None);
    assert_eq!(parse_date("someday"), None);
}

// ---- Markdown の kind ----

#[test]
fn test_ce_2_kind_follows_types_json() {
    // [CE-2] types.json があればそれに従う(値が someday でも date の列)
    let t = TempDir::new("types-json");
    t.write(".obsidian/types.json", br#"{"types": {"due": "date"}}"#);
    t.write("a.md", b"---\ndue: someday\ncount: 3\n---\n");
    let md = open(&[t.path().to_path_buf()]);
    let k = md.kind("due");
    assert_eq!(k.kind, Kind::Date);
    assert!(k.lock.is_none());
    // types.json に無いキーは推定
    assert_eq!(md.kind("count").kind, Kind::Number);
}

#[test]
fn test_ce_2_kind_infers_without_types_json() {
    // [CE-2] types.json が無ければ読んだ全行の値から推定する
    let t = TempDir::new("types-infer");
    t.mkdir(".obsidian");
    t.write("a.md", b"---\ndue:\ndone: true\nn: 1.5\n---\n");
    t.write("b.md", b"---\ndue: 2026-10-01\ntitle: hello\n---\n");
    let md = open(&[t.path().to_path_buf()]);
    assert_eq!(md.kind("due").kind, Kind::Date);
    assert_eq!(md.kind("done").kind, Kind::Checkbox);
    assert_eq!(md.kind("n").kind, Kind::Number);
    assert_eq!(md.kind("title").kind, Kind::Text);
    assert!(md.kind("due").lock.is_none());
}

#[test]
fn test_bv_12_kind_conflict_between_roots() {
    // [BV-12] 2つの根で types.json の型が食い違えば Text で lock あり
    let t = TempDir::new("types-conflict");
    let a = t.mkdir("A");
    let b = t.mkdir("B");
    t.write(
        "A/.obsidian/types.json",
        br#"{"types": {"due": "date", "n": "number"}}"#,
    );
    t.write(
        "B/.obsidian/types.json",
        br#"{"types": {"due": "text", "n": "number"}}"#,
    );
    t.write("A/x.md", b"---\ndue: 2026-10-01\nn: 1\n---\n");
    t.write("B/y.md", b"---\ndue: later\nn: 2\n---\n");
    let md = open(&[a, b]);
    let k = md.kind("due");
    assert_eq!(k.kind, Kind::Text);
    assert!(k.lock.is_some(), "conflicting column is read-only");
    let n = md.kind("n");
    assert_eq!(n.kind, Kind::Number);
    assert!(n.lock.is_none());
    // [BV-12] 食い違う列のセルは、どちらの根の行でも Cell.lock が Some(読むだけ)。食い違わない列は None。
    let rows = md.rows();
    assert_eq!(rows.len(), 2);
    for r in &rows {
        let c = md.get(r, "due");
        assert!(c.value.is_some());
        assert!(c.lock.is_some(), "due のセルは読むだけ: {}", md.label(r));
        assert!(md.get(r, "n").lock.is_none());
    }
}

// ---- candidates ----

fn sorted_strs(vals: &[Value]) -> Vec<String> {
    let mut v: Vec<String> = vals.iter().map(|x| format!("{x:?}")).collect();
    v.sort();
    v
}

#[test]
fn test_ce_3_candidates_distinct_without_null_and_empty() {
    // [CE-3] status が todo・doing・done だけなら3つ(Null と空は除く)
    let t = TempDir::new("cand");
    t.mkdir(".obsidian");
    t.write("a.md", b"---\nstatus: todo\n---\n");
    t.write("b.md", b"---\nstatus: doing\n---\n");
    t.write("c.md", b"---\nstatus: done\n---\n");
    t.write("d.md", b"---\nstatus: todo\n---\n");
    t.write("e.md", b"---\nstatus:\n---\n");
    t.write("f.md", b"---\nstatus: \"\"\n---\n");
    let md = open(&[t.path().to_path_buf()]);
    let c = md.candidates("status", 20).expect("within max");
    assert_eq!(c.len(), 3, "{c:?}");
    assert_eq!(
        sorted_strs(&c),
        sorted_strs(&[st("doing"), st("done"), st("todo")])
    );
}

#[test]
fn test_ce_3_candidates_over_max_is_none() {
    // [CE-3] 異なる値が上限を超えたら None。ちょうど上限なら Some
    let t = TempDir::new("cand-max");
    t.mkdir(".obsidian");
    for i in 0..5 {
        t.write(
            &format!("n{i}.md"),
            format!("---\nstatus: s{i}\n---\n").as_bytes(),
        );
    }
    let md = open(&[t.path().to_path_buf()]);
    assert!(md.candidates("status", 4).is_none());
    assert_eq!(md.candidates("status", 5).map(|v| v.len()), Some(5));
}

#[test]
fn test_ce_3_candidates_include_notes_outside_given_folder() {
    // [CE-3] 渡したフォルダの外(根の下)のノートの値も候補に入る
    let t = TempDir::new("cand-root");
    t.mkdir("vault/.obsidian");
    let sub = t.mkdir("vault/sub");
    t.write("vault/sub/a.md", b"---\nstatus: todo\n---\n");
    t.write("vault/other/b.md", b"---\nstatus: doing\n---\n");
    t.write("vault/c.md", b"---\nstatus: done\n---\n");
    let md = open(&[sub]);
    let c = md.candidates("status", 20).expect("within max");
    assert_eq!(
        sorted_strs(&c),
        sorted_strs(&[st("doing"), st("done"), st("todo")])
    );
}

// ---- file ----

#[test]
fn test_bv_6_file_info_fields_and_tags() {
    // [BV-6] file.* の属性(name・basename・ext・path・folder・size)と tags
    let t = TempDir::new("fileinfo");
    t.mkdir(".obsidian");
    let body = b"---\ntags: [proj, area/work]\n---\nText with #idea and #a/b here.\n";
    t.write("notes/deep/My Note.md", body);
    t.write("top.md", b"---\nx: 1\n---\n");
    let md = open(&[t.path().to_path_buf()]);

    let f = md
        .file(&row_of(&md, "notes/deep/My Note.md"))
        .expect("file info");
    assert_eq!(f.name, "My Note.md");
    assert_eq!(f.basename, "My Note");
    assert_eq!(f.ext, "md");
    assert_eq!(f.path, "notes/deep/My Note.md");
    assert_eq!(f.folder, "notes/deep");
    assert_eq!(f.size, body.len() as u64);
    assert!(f.mtime > 0);
    for tag in ["proj", "area/work", "idea", "a/b"] {
        assert!(f.tags.iter().any(|t| t == tag), "{tag} in {:?}", f.tags);
    }
    assert!(f.tags.iter().all(|t| !t.starts_with('#')), "{:?}", f.tags);

    let top = md.file(&row_of(&md, "top.md")).expect("file info");
    assert_eq!(top.folder, "", "root-level folder is empty");
    assert!(top.tags.is_empty());

    assert!(md.file(&RowId("/no/such/row.md".into())).is_none());
}
