//! ためる変更(タスク 7)の受け入れテスト(WB-4・WB-9・WB-10・WB-14・WB-16・CE-9・CE-10・SR-18)。
//! 仕様: specs/write-back/spec.md、specs/cell-edit/spec.md、specs/screen/spec.md。
//! 公開のインターフェース: docs/design.md「核の公開のインターフェース(縦切りの続き: タスク 4・18・15・7)」。

use mdgrid::changes::{Changes, Outcome};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId, Source};
use std::path::{Path, PathBuf};
use std::time::Duration;

// ---- 一時フォルダ(tests/writeback.rs から写す) ----

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

// ---- 材料 ----

const A: &str = "---\nstatus: todo\ntitle: A\ntags:\n  - x\n  - y\n---\nbody a\n";
const B: &str = "---\nstatus: todo\ntitle: B\n---\nbody b\n";
const C: &str = "---\nstatus: doing\ntitle: C\n---\nbody c\n";
const NOFM: &str = "no frontmatter here\n";
/// 1行目が `---` で始まるが閉じないノート(WB-5 で読むだけ)。
const UNCLOSED: &str = "---\nstatus: doing\n本文\n";

/// a.md・b.md・c.md・nofm.md を書き、Markdown で開いて読み切る。
fn setup(name: &str) -> (TempDir, Markdown) {
    let dir = TempDir::new(name);
    dir.write("a.md", A.as_bytes());
    dir.write("b.md", B.as_bytes());
    dir.write("c.md", C.as_bytes());
    dir.write("nofm.md", NOFM.as_bytes());
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

/// 表示名(根からの相対パス)で行を探す。
fn row(src: &Markdown, name: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == name)
        .unwrap_or_else(|| panic!("行 {name} が無い"))
}

fn read(dir: &TempDir, name: &str) -> String {
    String::from_utf8(std::fs::read(dir.path().join(name)).unwrap()).unwrap()
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

/// 外でノートを直す。更新時刻の粒度に頼らないよう、大きさを変えたうえで時刻も先にずらす。
fn external_edit(dir: &TempDir, name: &str, bytes: &str) {
    let p = dir.path().join(name);
    let old = std::fs::metadata(&p).unwrap();
    assert_ne!(old.len(), bytes.len() as u64, "外の変更は大きさを変える");
    std::fs::write(&p, bytes).unwrap();
    let f = std::fs::File::options().write(true).open(&p).unwrap();
    f.set_modified(old.modified().unwrap() + Duration::from_secs(10))
        .unwrap();
}

fn outcome_of<'a>(res: &'a [(RowId, Outcome)], r: &RowId) -> &'a Outcome {
    &res.iter()
        .find(|(id, _)| id == r)
        .expect("保存の結果にその行が無い")
        .1
}

// ---- WB-9 ----

#[test]
fn test_wb_9_pending_not_written_until_save() {
    // [WB-9] 2つのセルを直す → ファイルは変わらず未保存 2。差分を2ファイル分見せ、保存で書かれる。
    let (dir, src) = setup("wb9");
    let mut src = src;
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("wip")).is_ok());

    // [WB-9] ファイルのバイトは変わらない
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B);
    assert_eq!(ch.count(), 2);
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert!(matches!(ch.pending(&b, "status"), Some(NewValue::Str(v)) if v == "wip"));
    let mut rows = ch.rows();
    rows.sort();
    let mut want = vec![a.clone(), b.clone()];
    want.sort();
    assert!(rows == want);

    // [WB-9] 保存の前の差分: 2行分の (before, after)
    let pv = ch.previews(&src);
    assert_eq!(pv.len(), 2);
    for p in pv {
        let p = match p {
            Ok(p) => p,
            Err(_) => panic!("previews が Err"),
        };
        let before = String::from_utf8(p.before.clone()).unwrap();
        let after = String::from_utf8(p.after.clone()).unwrap();
        assert!(!p.external);
        if p.row == a {
            assert_eq!(before, A);
            assert!(after.contains("status: done\n"), "{after}");
        } else if p.row == b {
            assert_eq!(before, B);
            assert!(after.contains("status: wip\n"), "{after}");
        } else {
            panic!("ためていない行の差分");
        }
    }
    // previews ではファイルに触らない
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B);

    // [WB-9] 保存 → 両ファイルが書かれ、未保存 0
    let res = ch.save(&mut src);
    assert_eq!(res.len(), 2);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved));
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(ch.count(), 0);
    assert_eq!(
        read(&dir, "a.md"),
        A.replace("status: todo", "status: done")
    );
    assert_eq!(read(&dir, "b.md"), B.replace("status: todo", "status: wip"));
    assert!(ch.pending(&a, "status").is_none());
}

// ---- WB-10 ----

#[test]
fn test_wb_10_undo_redo_single_set() {
    // [WB-10] 直して取り消す → 元に戻り未保存 0。やり直しで戻る。
    let (dir, src) = setup("wb10");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();

    assert!(!ch.undo(), "積みが空なら取り消しは false");
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert_eq!(ch.count(), 1);

    assert!(ch.undo());
    assert!(ch.pending(&a, "status").is_none());
    assert_eq!(ch.count(), 0);

    assert!(ch.redo());
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert_eq!(ch.count(), 1);
    assert!(!ch.redo(), "やり直す手が無ければ false");

    assert_eq!(read(&dir, "a.md"), A);
}

#[test]
fn test_wb_10_undo_restores_previous_pending_value() {
    // [WB-10] 同じセルを2度直して取り消す → 1度目のためた値に戻る。
    let (_dir, src) = setup("wb10b");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", s("one")).is_ok());
    assert!(ch.set(&src, &a, "status", s("two")).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(ch.undo());
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "one"));
    assert!(ch.undo());
    assert!(ch.pending(&a, "status").is_none());
    assert!(ch.redo());
    assert!(ch.redo());
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "two"));
}

#[test]
fn test_wb_10_set_many_is_one_step() {
    // [WB-10] [CE-10] まとめて入れた変更は1手で取り消される。
    let (_dir, src) = setup("wb10many");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let c = row(&src, "c.md");
    let mut ch = Changes::new();

    let skips = ch.set_many(
        &src,
        &[a.clone(), b.clone(), c.clone()],
        "status",
        s("done"),
    );
    assert!(skips.is_empty());
    assert_eq!(ch.count(), 3);

    assert!(ch.undo());
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&a, "status").is_none());
    assert!(ch.pending(&b, "status").is_none());
    assert!(ch.pending(&c, "status").is_none());

    assert!(ch.redo());
    assert_eq!(ch.count(), 3);
    assert!(matches!(ch.pending(&c, "status"), Some(NewValue::Str(v)) if v == "done"));
}

// ---- CE-9 ----

#[test]
fn test_ce_9_null_keeps_key_line() {
    // [CE-9] 値を空(null)にして保存 → `status:` の行が残る。`status: null` ではない。
    let (dir, src) = setup("ce9");
    let mut src = src;
    let a = row(&src, "a.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", NewValue::Null).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Null)));

    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved));
    let text = read(&dir, "a.md");
    assert!(
        text.lines().any(|l| l.trim_end() == "status:"),
        "status: の行が無い: {text}"
    );
    assert!(!text.contains("null"), "{text}");
    assert!(text.contains("title: A\n"));
    assert!(text.ends_with("---\nbody a\n"));
}

// ---- SR-18 ----

#[test]
fn test_sr_18_null_on_missing_key_does_nothing() {
    // [SR-18] キーの無いセルを空にする(Backspace)→ 何もせず、キーを足さない。未保存 0。
    let (dir, src) = setup("sr18");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    // b.md には tags が無い(a.md にはある)
    assert!(src.columns().iter().any(|c| c == "tags"));
    assert!(src.get(&b, "tags").value.is_none());

    let _ = ch.set(&src, &b, "tags", NewValue::Null);
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&b, "tags").is_none());
    assert!(ch.rows().is_empty());

    let res = ch.save(&mut src);
    assert!(res.iter().all(|(_, o)| matches!(o, Outcome::Saved)));
    assert_eq!(read(&dir, "b.md"), B);
}

// ---- CE-10 ----

#[test]
fn test_ce_10_set_many_skips_unreadable_frontmatter() {
    // [CE-10] 3行のうち1行が読めないフロントマターのノート(WB-5。閉じない) → 2つためて、1行を飛ばし理由を返す。
    // [WB-3] フロントマターの無いノートは飛ばさず、ためる変更になる。
    let (dir, src) = setup_with(
        "ce10",
        &[("a.md", A), ("nofm.md", NOFM), ("bad.md", UNCLOSED)],
    );
    let a = row(&src, "a.md");
    let n = row(&src, "nofm.md");
    let u = row(&src, "bad.md");
    let mut ch = Changes::new();

    let skips = ch.set_many(
        &src,
        &[a.clone(), u.clone(), n.clone()],
        "status",
        s("done"),
    );
    assert_eq!(skips.len(), 1);
    assert!(skips[0].row == u);
    assert!(!skips[0].reason.is_empty());
    assert_eq!(ch.count(), 2);
    assert!(ch.pending(&u, "status").is_none());
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert!(matches!(ch.pending(&n, "status"), Some(NewValue::Str(v)) if v == "done"));

    // 保存前の差分は2ファイル。読めないノートは変わらない。
    assert_eq!(ch.previews(&src).len(), 2);
    assert_eq!(read(&dir, "bad.md"), UNCLOSED);
    // [WB-9] ためただけでは、フロントマターの無いノートも書かない。
    assert_eq!(read(&dir, "nofm.md"), NOFM);
}

#[test]
fn test_ce_10_set_on_read_only_cell_is_skip() {
    // [CE-10] 読むだけのセル(ネストした map・複数行のフローのリスト・読めないフロントマターのノート(WB-5))に
    // set → Err(Skip)。
    // [CE-16] 書ける形のブロックのリスト(a.md の tags)は読むだけでない。
    // [WB-3] フロントマターの無いノートのセルは読むだけでなく、ためられる。
    let (_dir, src) = setup_with(
        "ce10ro",
        &[
            ("a.md", A),
            ("m.md", "---\nstatus: todo\nmeta:\n  k: v\n---\nbody m\n"),
            ("f.md", "---\nstatus: todo\ntags: [x,\n  y]\n---\nbody f\n"),
            ("bad.md", UNCLOSED),
            ("nofm.md", NOFM),
        ],
    );
    let a = row(&src, "a.md");
    let m = row(&src, "m.md");
    let f = row(&src, "f.md");
    let n = row(&src, "bad.md");
    let nofm = row(&src, "nofm.md");
    let mut ch = Changes::new();

    assert!(src.get(&a, "tags").lock.is_none(), "[CE-16]");
    for (r, col) in [(&m, "meta"), (&f, "tags")] {
        assert!(src.get(r, col).lock.is_some(), "{col}");
        match ch.set(&src, r, col, s("z")) {
            Err(skip) => {
                assert!(skip.row == *r);
                assert!(!skip.reason.is_empty());
            }
            Ok(()) => panic!("読むだけのセル {col} にためた"),
        }
    }
    match ch.set(
        &src,
        &f,
        "tags",
        NewValue::List(vec!["x".into(), "z".into()]),
    ) {
        Err(skip) => assert!(!skip.reason.is_empty()),
        Ok(()) => panic!("複数行のフローのリストにリストをためた"),
    }
    match ch.set(&src, &n, "status", s("done")) {
        Err(skip) => {
            assert!(skip.row == n);
            assert!(!skip.reason.is_empty());
        }
        Ok(()) => panic!("読めないフロントマターのノートにためた"),
    }
    assert_eq!(ch.count(), 0);
    assert!(!ch.undo(), "飛ばしただけでは手を積まない");
    // [WB-3] フロントマターの無いノートは lock なしで、ためる変更になる。
    assert!(src.get(&nofm, "status").lock.is_none());
    assert!(ch.set(&src, &nofm, "status", s("done")).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(matches!(ch.pending(&nofm, "status"), Some(NewValue::Str(v)) if v == "done"));
}

// ---- WB-4 / WB-16 / WB-14 ----

const B_EXT: &str = "---\nstatus: todo\ntitle: B changed outside\n---\nbody b\n";

#[test]
fn test_wb_16_external_change_after_set_marks_and_stops_save() {
    // [WB-16] ためたあと外で直す → 印が付き、ためた変更は残る。
    // [WB-4] 保存 → 書かずに止まる(Changed)。[WB-14] 未保存の数は減らない。
    let (dir, src) = setup("wb16");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    assert!(!ch.external(&b));

    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    assert!(changed.contains(&b), "changed() が外の変更を返さない");
    ch.note_external(&changed);

    assert!(ch.external(&b));
    assert!(matches!(ch.pending(&b, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert_eq!(ch.count(), 1);

    // [WB-16] 差分には印が出る
    let pv = ch.previews(&src);
    assert_eq!(pv.len(), 1);
    match &pv[0] {
        Ok(p) => assert!(p.external),
        Err(_) => panic!("previews が Err"),
    }

    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed));
    assert_eq!(
        read(&dir, "b.md"),
        B_EXT,
        "止まった保存でファイルが変わった"
    );
    assert_eq!(ch.count(), 1);
    assert!(matches!(ch.pending(&b, "status"), Some(NewValue::Str(v)) if v == "done"));

    // [WB-4] [WB-16] 外の変更の上に書く → 選んだ時点の内容を基準にし、印が消え、保存は通る
    ch.overwrite(&mut src, &b).expect("overwrite");
    assert!(!ch.external(&b));
    assert_eq!(ch.count(), 1);
    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(ch.count(), 0);
    let text = read(&dir, "b.md");
    assert_eq!(text, B_EXT.replace("status: todo", "status: done"));
    assert!(text.contains("title: B changed outside\n"));
}

#[test]
fn test_wb_4_external_change_after_overwrite_stops_again() {
    // [WB-4] 外の変更の上に書くを選んでから書くまでにまた外で変わった → 止まる。
    let (dir, src) = setup("wb4again");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    ch.note_external(&changed);
    ch.overwrite(&mut src, &b).expect("overwrite");

    let again = "---\nstatus: todo\ntitle: B changed outside twice\n---\nbody b\n";
    external_edit(&dir, "b.md", again);
    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed));
    assert_eq!(read(&dir, "b.md"), again);
    assert_eq!(ch.count(), 1);
}

#[test]
fn test_wb_4_set_after_external_reload_keeps_old_baseline() {
    // [WB-4] [WB-16] ためたあとの読み直しでは保存の基準を変えない。
    // 外で変わった行の別のキーにさらにためても、基準は最初にためた時点のまま → 保存は止まる。
    let (dir, src) = setup("wb4keepbase");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    assert!(changed.contains(&b), "changed() が外の変更を返さない");
    ch.note_external(&changed);
    assert!(ch.external(&b));

    // 同じ行の別のキーにためる(読み直したあとの行)。
    assert!(ch.set(&src, &b, "title", s("B2")).is_ok());
    assert!(ch.external(&b), "2回目のためるで印が消えた");
    assert_eq!(ch.count(), 2);

    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed));
    assert_eq!(
        read(&dir, "b.md"),
        B_EXT,
        "止まった保存でファイルが変わった"
    );
    assert_eq!(ch.count(), 2);
}

#[test]
fn test_wb_4_external_change_before_set_is_reloaded() {
    // [WB-4] [WB-16] ためる前に外で直した → 読み直され、印は付かず、保存は止まらない。
    let (dir, src) = setup("wb4before");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    assert!(changed.contains(&b));
    ch.note_external(&changed);
    assert!(!ch.external(&b), "ためた変更の無い行に印が付いた");

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    assert!(!ch.external(&b));
    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(ch.count(), 0);
    assert_eq!(
        read(&dir, "b.md"),
        B_EXT.replace("status: todo", "status: done")
    );
}

#[test]
fn test_wb_4_same_size_change_is_detected_by_mtime_and_hash() {
    // [WB-4] 大きさが同じ外の変更でも、更新時刻(ずらす)と中身のハッシュで止まる。
    let (dir, src) = setup("wb4same");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    let same_len = B.replace("title: B", "title: Z");
    assert_eq!(same_len.len(), B.len());
    let p = dir.path().join("b.md");
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    std::fs::write(&p, &same_len).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old + Duration::from_secs(10))
        .unwrap();

    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed));
    assert_eq!(read(&dir, "b.md"), same_len);
    assert_eq!(ch.count(), 1);
}

#[test]
fn test_wb_16_discard_drops_row_changes() {
    // [WB-16] ためた変更を捨てる → その行の変更が消え、ほかの行は残る。1手として取り消せる。
    let (dir, src) = setup("wb16discard");
    let mut src = src;
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "title", s("New B")).is_ok());
    assert_eq!(ch.count(), 3);

    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    ch.note_external(&changed);
    assert!(ch.external(&b));

    ch.discard(&b);
    assert_eq!(ch.count(), 1);
    assert!(ch.pending(&b, "status").is_none());
    assert!(ch.pending(&b, "title").is_none());
    assert!(!ch.rows().contains(&b));
    assert!(ch.pending(&a, "status").is_some());

    // 捨てるのも1手
    assert!(ch.undo());
    assert_eq!(ch.count(), 3);
    assert!(ch.redo());
    assert_eq!(ch.count(), 1);

    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved));
    assert!(res.iter().all(|(r, _)| *r != b));
    assert_eq!(read(&dir, "b.md"), B_EXT, "捨てた行が書かれた");
    assert_eq!(
        read(&dir, "a.md"),
        A.replace("status: todo", "status: done")
    );
    assert_eq!(ch.count(), 0);
}

#[test]
fn test_wb_14_partial_save_keeps_stopped_rows() {
    // [WB-14] 2行ためて1行だけ外で変える → 1行 Saved・1行 Changed。未保存は残った分だけ。
    let (dir, src) = setup("wb14");
    let mut src = src;
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "title", s("New B")).is_ok());
    assert_eq!(ch.count(), 3);

    external_edit(&dir, "b.md", B_EXT);
    let changed = src.changed();
    ch.note_external(&changed);
    assert!(!ch.external(&a));
    assert!(ch.external(&b));

    let res = ch.save(&mut src);
    assert_eq!(res.len(), 2);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved));
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed));
    assert_eq!(ch.count(), 2, "止まった行の変更が減った");
    assert!(ch.pending(&a, "status").is_none());
    assert!(ch.pending(&b, "status").is_some());
    assert!(ch.pending(&b, "title").is_some());
    assert_eq!(
        read(&dir, "a.md"),
        A.replace("status: todo", "status: done")
    );
    assert_eq!(read(&dir, "b.md"), B_EXT);

    // 読み直してからやり直せる
    ch.overwrite(&mut src, &b).expect("overwrite");
    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(ch.count(), 0);
    assert_eq!(
        read(&dir, "b.md"),
        "---\nstatus: done\ntitle: New B\n---\nbody b\n"
    );
}

// ---- WB-17 ----

/// 指定のノートだけを書いたフォルダを Markdown で開いて読み切る。
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

#[test]
fn test_wb_17_set_back_to_original_drops_pending() {
    // [WB-17] status: todo を done に直す → 未保存 1。もう一度 todo → 未保存 0・ためていない・差分に出ない。
    // 外す操作も1手: undo → done に戻って未保存 1、redo → 未保存 0。
    let (dir, src) = setup("wb17revert");
    let mut src = src;
    let a = row(&src, "a.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert_eq!(ch.count(), 1);

    assert!(ch.set(&src, &a, "status", s("todo")).is_ok());
    assert_eq!(ch.count(), 0, "[WB-17] 元の値に戻したのに未保存に数える");
    assert!(
        ch.pending(&a, "status").is_none(),
        "[WB-17] 元の値に戻したセルが残る"
    );
    assert!(ch.rows().is_empty());
    assert!(
        ch.previews(&src).is_empty(),
        "[WB-17] 元の値に戻したセルが差分に出る"
    );

    // [WB-17] 外す操作も取り消し・やり直しの1手
    assert!(ch.undo());
    assert_eq!(ch.count(), 1);
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "done"));
    assert!(ch.redo());
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&a, "status").is_none());

    // [WB-17] 保存しても何も書かない
    let res = ch.save(&mut src);
    assert!(res.is_empty(), "[WB-17] 変更が無いのに保存の結果に行がある");
    assert_eq!(read(&dir, "a.md"), A);
}

#[test]
fn test_wb_17_same_value_on_clean_cell_is_noop() {
    // [WB-17] ためていないセルに今と同じ値 → ためず、手も積まない。
    let (dir, src) = setup("wb17same");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();

    let _ = ch.set(&src, &a, "status", s("todo"));
    assert_eq!(ch.count(), 0, "[WB-17] 今と同じ値をためた");
    assert!(ch.pending(&a, "status").is_none());
    assert!(!ch.undo(), "[WB-17] 何も変えない set が手を積んだ");
    assert_eq!(read(&dir, "a.md"), A);
}

#[test]
fn test_wb_17_unquoted_date_revert_keeps_bytes() {
    // [WB-17] クオートの無い日付を別の日付にしてから元の日付に戻して保存 → ファイルは1バイトも変わらない。
    let note = "---\ndue: 2026-09-30\ntitle: D\n---\nbody d\n";
    let (dir, src) = setup_with("wb17date", &[("d.md", note)]);
    let mut src = src;
    let d = row(&src, "d.md");
    let mut ch = Changes::new();

    assert!(ch.set(&src, &d, "due", s("2026-10-05")).is_ok());
    assert_eq!(ch.count(), 1);
    assert!(ch.set(&src, &d, "due", s("2026-09-30")).is_ok());
    assert_eq!(ch.count(), 0, "[WB-17] 元の日付に戻したのに未保存に数える");
    assert!(ch.pending(&d, "due").is_none());

    let _ = ch.save(&mut src);
    assert_eq!(
        read(&dir, "d.md"),
        note,
        "[WB-17] 元の値に戻したセルが書かれた"
    );
}

#[test]
fn test_wb_17_value_types_compared() {
    // [WB-17] 数の 3 に Int(3)・Float(3.0) → ためない。文字の "3" に Int(3) → ためる(型が違う)。
    // Null の値に Null → ためない。空の文字列 "" に Null → ためる。
    let note = "---\npriority: 3\ntitle: \"3\"\nempty:\nblank: \"\"\n---\nbody n\n";
    let (dir, src) = setup_with("wb17types", &[("n.md", note)]);
    let n = row(&src, "n.md");
    let mut ch = Changes::new();

    let _ = ch.set(&src, &n, "priority", NewValue::Int(3));
    assert!(
        ch.pending(&n, "priority").is_none(),
        "[WB-17] 数の 3 に Int(3) をためた"
    );
    let _ = ch.set(&src, &n, "priority", NewValue::Float(3.0));
    assert!(
        ch.pending(&n, "priority").is_none(),
        "[WB-17] 数の 3 に Float(3.0) をためた"
    );
    assert_eq!(ch.count(), 0);

    assert!(ch.set(&src, &n, "title", NewValue::Int(3)).is_ok());
    assert!(
        matches!(ch.pending(&n, "title"), Some(NewValue::Int(3))),
        "[WB-17] 文字の \"3\" に Int(3) をためない(型が違うのに同じとみた)"
    );
    assert_eq!(ch.count(), 1);

    let _ = ch.set(&src, &n, "empty", NewValue::Null);
    assert!(
        ch.pending(&n, "empty").is_none(),
        "[WB-17] Null の値に Null をためた"
    );
    assert_eq!(ch.count(), 1);

    assert!(ch.set(&src, &n, "blank", NewValue::Null).is_ok());
    assert!(
        matches!(ch.pending(&n, "blank"), Some(NewValue::Null)),
        "[WB-17] 空の文字列に Null をためない"
    );
    assert_eq!(ch.count(), 2);
    assert_eq!(read(&dir, "n.md"), note);
}

#[test]
fn test_wb_17_set_many_skips_same_value_rows() {
    // [WB-17] [CE-10] 3行のうち1行が元と同じ値 → その行はためず、ほかの2行だけ。
    let (_dir, src) = setup("wb17many");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let c = row(&src, "c.md");
    let mut ch = Changes::new();

    // c.md は status: doing
    let _ = ch.set_many(
        &src,
        &[a.clone(), b.clone(), c.clone()],
        "status",
        s("doing"),
    );
    assert_eq!(ch.count(), 2, "[WB-17] 元と同じ値の行もためた");
    assert!(ch.pending(&c, "status").is_none());
    assert!(matches!(ch.pending(&a, "status"), Some(NewValue::Str(v)) if v == "doing"));
    assert!(matches!(ch.pending(&b, "status"), Some(NewValue::Str(v)) if v == "doing"));
    assert_eq!(ch.previews(&src).len(), 2);
    assert!(!ch.rows().contains(&c));
}

#[test]
fn test_wb_17_compares_with_value_read_after_external_change() {
    // [WB-17] 外で読み直したあとは、新しく読んだ値と比べる。
    let (dir, src) = setup("wb17ext");
    let mut src = src;
    let b = row(&src, "b.md");
    let mut ch = Changes::new();

    let b_done = "---\nstatus: done\ntitle: B changed outside\n---\nbody b\n";
    external_edit(&dir, "b.md", b_done);
    let changed = src.changed();
    assert!(changed.contains(&b), "changed() が外の変更を返さない");
    ch.note_external(&changed);

    // 新しく読んだ値 done と同じ → ためない
    let _ = ch.set(&src, &b, "status", s("done"));
    assert_eq!(ch.count(), 0, "[WB-17] 新しく読んだ値と同じ値をためた");
    assert!(ch.pending(&b, "status").is_none());

    // 前に読んでいた値 todo は、今は違う値 → ためる
    assert!(ch.set(&src, &b, "status", s("todo")).is_ok());
    assert_eq!(ch.count(), 1, "[WB-17] 古い値と比べている");
    assert!(matches!(ch.pending(&b, "status"), Some(NewValue::Str(v)) if v == "todo"));
}
