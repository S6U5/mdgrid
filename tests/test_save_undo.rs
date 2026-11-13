//! 保存の取り消し(WB-21)の核の受け入れテスト。仕様: specs/write-back/spec.md の WB-21。
//! 記録: specs/_changes/2026-11-13-save-undo.md(設計の「核の口」)。実装を見ずに、要件の例と口の形だけで書いた。
//!
//! - `writeback::restore(path, base, bytes)`: 今のファイルが基準と同じときだけ、渡したバイトで置き換える。
//! - `Changes::save` → `saved_units`(単位・書いたあとの基準・保存の前のバイト)→ `undo_save`。
//!
//! 値はどれも作ったもの。

use mdgrid::changes::{Changes, Outcome};
use mdgrid::source::csv::Csv;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId, Source, Value};
use mdgrid::writeback::{baseline, restore, save, Edit, SaveError};
use std::path::{Path, PathBuf};
use std::time::Duration;

// ---- 一時フォルダ(tests/test_changes.rs と同じ形) ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-save-undo-{}-{}-{}",
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

    /// フォルダの中の名前(並べたもの)。控えや一時ファイルが残っていないことを見る。
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

// ---- 材料 ----

/// クオートと空白の書式を持つノート(バイトごと戻ることを見るため、値の逆の書き換えでは同じにならない形)。
const A: &str = "---\nstatus:   todo\ntitle: \"A\"\ntags:\n  - x\n  - y\n---\nbody a\n";
const A_DONE: &str = "---\nstatus:   done\ntitle: \"A\"\ntags:\n  - x\n  - y\n---\nbody a\n";
const B: &str = "---\nstatus: todo\ntitle: B\n---\nbody b\n";
const B_DONE: &str = "---\nstatus: done\ntitle: B\n---\nbody b\n";
/// 保存のあとに外で title を直した b.md(status は保存で書いた done のまま)。
const B_DONE_EXT: &str = "---\nstatus: done\ntitle: B changed outside\n---\nbody b\n";

const NOTE: &[u8] = b"---\ntitle: t\nstatus: todo\n---\nbody\n";
const NOTE_DONE: &[u8] = b"---\ntitle: t\nstatus: done\n---\nbody\n";

fn load_all(src: &mut dyn Source) {
    let mut guard = 0;
    loop {
        if src.load(100).done {
            break;
        }
        guard += 1;
        assert!(guard < 10_000, "load が終わらない");
    }
}

/// a.md・b.md を書き、Markdown で開いて読み切る。
fn setup(name: &str) -> (TempDir, Markdown) {
    let dir = TempDir::new(name);
    dir.write("a.md", A.as_bytes());
    dir.write("b.md", B.as_bytes());
    let mut src = Markdown::open(&[dir.path().to_path_buf()]).expect("open");
    load_all(&mut src);
    (dir, src)
}

fn row(src: &dyn Source, name: &str) -> RowId {
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

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
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
        .unwrap_or_else(|| panic!("結果に {r:?} が無い: {res:?}"))
        .1
}

fn pending_str(ch: &Changes, r: &RowId, col: &str) -> Option<String> {
    match ch.pending(r, col) {
        Some(NewValue::Str(v)) => Some(v.clone()),
        Some(other) => panic!("文字でない値がためてある: {other:?}"),
        None => None,
    }
}

/// 全部の行を保存し、どれも書けたことを確かめる。
fn save_ok(ch: &mut Changes, src: &mut dyn Source) {
    let res = ch.save(src);
    assert!(!res.is_empty(), "保存の結果が空");
    for (r, o) in &res {
        assert!(matches!(o, Outcome::Saved), "{r:?} が書けない: {o:?}");
    }
}

// ---- writeback::restore ----

#[test]
fn test_wb_21_restore_replaces_bytes_when_file_matches_base() {
    // [WB-21] 今のファイルが基準と同じ → 渡したバイトで置き換わり、保存の前と1バイトも違わない。
    // 返る基準は書いたあとのファイルの基準(WB-12)。一時ファイル・控えを残さない(WB-8・WB-2)。
    let dir = TempDir::new("restore-ok");
    let note = dir.write("note.md", NOTE);
    let base = baseline(&note).unwrap();
    let saved = save(&note, &base, &[edit("status", s("done"))]).expect("save");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE_DONE);

    let back = restore(&note, &saved, NOTE).expect("restore");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE);
    assert_eq!(back, baseline(&note).unwrap());
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[test]
fn test_wb_21_restore_writes_given_bytes_verbatim() {
    // [WB-21] 戻すのは値でなくバイト: フロントマターとして読めない中身・CRLF・BOM でもそのまま置く。
    let dir = TempDir::new("restore-verbatim");
    let before: &[u8] = b"\xEF\xBB\xBF---\r\nstatus:    'todo'   \r\n---\r\nbody\r\n\r\n";
    let note = dir.write("note.md", NOTE_DONE);
    let base = baseline(&note).unwrap();
    restore(&note, &base, before).expect("restore");
    assert_eq!(std::fs::read(&note).unwrap(), before);
}

#[test]
fn test_wb_21_restore_refuses_when_file_changed_outside() {
    // [WB-21] 保存のあとに外で変わった(今の中身が基準と違う)→ 書かず SaveError::Changed。外の変更を消さない。
    let dir = TempDir::new("restore-changed");
    let note = dir.write("note.md", NOTE);
    let base = baseline(&note).unwrap();
    let saved = save(&note, &base, &[edit("status", s("done"))]).expect("save");
    let outside: &[u8] = b"---\ntitle: t outside\nstatus: done\n---\nbody\n";
    std::fs::write(&note, outside).unwrap();

    let r = restore(&note, &saved, NOTE);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), outside);
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[test]
fn test_wb_21_restore_refuses_stale_base() {
    // [WB-21] [WB-12] 保存の前の古い基準では戻せない(今のファイルと同じ基準だけが通る)。
    let dir = TempDir::new("restore-stale");
    let note = dir.write("note.md", NOTE);
    let base = baseline(&note).unwrap();
    save(&note, &base, &[edit("status", s("done"))]).expect("save");

    let r = restore(&note, &base, NOTE);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE_DONE);
}

#[test]
fn test_wb_21_restore_same_length_change_is_detected() {
    // [WB-21] [WB-4] 大きさが同じ外の変更でも、中身が違えば書かない。
    let dir = TempDir::new("restore-samelen");
    let note = dir.write("note.md", NOTE_DONE);
    let base = baseline(&note).unwrap();
    let outside: &[u8] = b"---\ntitle: u\nstatus: done\n---\nbody\n";
    assert_eq!(outside.len(), NOTE_DONE.len());
    std::fs::write(&note, outside).unwrap();
    let f = std::fs::File::options().write(true).open(&note).unwrap();
    f.set_modified(base.mtime + Duration::from_secs(10))
        .unwrap();
    drop(f);

    let r = restore(&note, &base, NOTE);
    assert!(matches!(r, Err(SaveError::Changed)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), outside);
}

#[cfg(unix)]
#[test]
fn test_wb_21_restore_keeps_permissions() {
    // [WB-21] [WB-8] 戻したあとも権限は同じ。
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("restore-perm");
    let note = dir.write("note.md", NOTE_DONE);
    std::fs::set_permissions(&note, std::fs::Permissions::from_mode(0o640)).unwrap();
    let base = baseline(&note).unwrap();
    restore(&note, &base, NOTE).expect("restore");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE);
    let mode = std::fs::metadata(&note).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode, 0o640);
}

#[cfg(unix)]
#[test]
fn test_wb_21_restore_keeps_symlink() {
    // [WB-21] [WB-8] リンクのノートはリンクのまま、指す先の中身が戻る。
    let dir = TempDir::new("restore-link");
    let target = dir.write("target.md", NOTE_DONE);
    let link = dir.path().join("link.md");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let base = baseline(&link).unwrap();
    restore(&link, &base, NOTE).expect("restore");
    let meta = std::fs::symlink_metadata(&link).unwrap();
    assert!(meta.file_type().is_symlink(), "リンクのままでない");
    assert_eq!(std::fs::read_link(&link).unwrap(), target);
    assert_eq!(std::fs::read(&target).unwrap(), NOTE);
    assert_eq!(
        dir.names(),
        vec!["link.md".to_string(), "target.md".to_string()]
    );
}

#[cfg(unix)]
#[test]
fn test_wb_21_restore_refuses_hard_linked_file() {
    // [WB-21] [WB-5] ハードリンクのあるファイルは、保存と同じく断る(名前の付け替えでリンクが切れるため)。
    let dir = TempDir::new("restore-hardlink");
    let note = dir.write("note.md", NOTE_DONE);
    std::fs::hard_link(&note, dir.path().join("other.md")).unwrap();
    let base = baseline(&note).unwrap();
    let r = restore(&note, &base, NOTE);
    assert!(r.is_err(), "ハードリンクのあるファイルを戻した");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE_DONE);
    assert_eq!(
        std::fs::read(dir.path().join("other.md")).unwrap(),
        NOTE_DONE
    );
}

// ---- Changes: save → saved_units → undo_save ----

#[test]
fn test_wb_21_undo_save_restores_file_bytes_and_pending_value() {
    // [WB-21] status: todo を done に直して保存 → 取り消す → ファイルは保存の前と1バイトも違わず、
    // 書いた値はためる変更に戻り(未保存 1)、`u` で消える(未保存 0)。やり直しも1手。
    let (dir, mut src) = setup("basic");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    assert_eq!(read(&dir, "a.md"), A_DONE);
    assert_eq!(ch.count(), 0);

    // 覚え: 単位・書いたあとの基準・保存の前のバイト。
    let units = ch.saved_units();
    assert_eq!(units.len(), 1, "{units:?}");
    let (unit, stamp, before) = &units[0];
    assert_eq!(unit, &a);
    assert_eq!(Some(*stamp), src.stamp(&a), "書いたあとの基準でない");
    assert_eq!(before.as_slice(), A.as_bytes());
    // 見るだけでは書かない。
    assert_eq!(read(&dir, "a.md"), A_DONE);

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 1, "{res:?}");
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved), "{res:?}");
    assert_eq!(
        std::fs::read(dir.path().join("a.md")).unwrap(),
        A.as_bytes()
    );
    // 読み込み口の値も保存の前に戻る。
    assert_eq!(
        src.get(&a, "status").value,
        Some(Value::Str("todo".to_string()))
    );
    // 書いた値はためる変更に戻る。
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert_eq!(ch.count(), 1);
    assert_eq!(ch.rows(), vec![a.clone()]);
    // 戻した単位は覚えから消える。
    assert!(ch.saved_units().is_empty());

    // [WB-10] 1手で消え、やり直しで戻る。ファイルには触らない。
    assert!(ch.undo());
    assert_eq!(ch.count(), 0);
    assert!(ch.pending(&a, "status").is_none());
    assert!(ch.redo());
    assert_eq!(ch.count(), 1);
    assert_eq!(read(&dir, "a.md"), A);
}

#[test]
fn test_wb_21_undo_save_then_fix_and_save_again() {
    // [WB-21] 戻したあと、値を直してもう一度保存できる(基準は戻したあとのファイル)。
    let (dir, mut src) = setup("resave");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    ch.undo_save(&mut src);
    assert_eq!(read(&dir, "a.md"), A);

    assert!(ch.set(&src, &a, "status", s("doing")).is_ok());
    assert_eq!(ch.count(), 1);
    save_ok(&mut ch, &mut src);
    assert_eq!(ch.count(), 0);
    assert_eq!(read(&dir, "a.md"), A.replace("todo", "doing"));
}

#[test]
fn test_wb_21_undo_save_restores_cells_as_one_step() {
    // [WB-21] [WB-10] 1回の保存で書いた2つのファイル・3つのセルが、ためる変更として1手で戻る。
    let (dir, mut src) = setup("onestep");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "title", s("New B")).is_ok());
    save_ok(&mut ch, &mut src);
    assert_eq!(ch.saved_units().len(), 2);

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 2, "{res:?}");
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved));
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B);
    assert_eq!(ch.count(), 3);
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert_eq!(pending_str(&ch, &b, "status").as_deref(), Some("done"));
    assert_eq!(pending_str(&ch, &b, "title").as_deref(), Some("New B"));

    assert!(ch.undo());
    assert_eq!(ch.count(), 0, "戻した値が1手で消えない");
}

#[test]
fn test_wb_21_undo_save_skips_unit_changed_outside() {
    // [WB-21] 保存のあとに外で title を直した b.md は書かずに飛ばし(Changed)、a.md だけが戻る。
    // 飛ばした単位の値はためる変更に戻さない。読み込み口が外の変化を見る前でも同じ。
    let (dir, mut src) = setup("outside");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    external_edit(&dir, "b.md", B_DONE_EXT);

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 2, "{res:?}");
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved), "{res:?}");
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed), "{res:?}");
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B_DONE_EXT, "外の変更を消した");
    assert_eq!(ch.count(), 1);
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert!(ch.pending(&b, "status").is_none());
    // 設計: 飛ばした単位は覚えに残り、戻した単位は消える。
    let left: Vec<RowId> = ch.saved_units().into_iter().map(|u| u.0).collect();
    assert_eq!(left, vec![b.clone()]);
    // もう一度呼んでも、外で変わったままなら書かない。
    let res = ch.undo_save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed), "{res:?}");
    assert_eq!(read(&dir, "b.md"), B_DONE_EXT);
    assert_eq!(read(&dir, "a.md"), A);
}

#[test]
fn test_wb_21_undo_save_skips_changed_unit_after_source_saw_it() {
    // [WB-21] 読み込み口が外の変化を読み直したあと(基準が外の中身に進んだあと)でも、
    // 比べる相手は「保存で書いた中身」なので書かない。
    let (dir, mut src) = setup("outside-seen");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    external_edit(&dir, "b.md", B_DONE_EXT);
    let changed = src.changed();
    assert!(changed.contains(&b), "changed() が外の変更を返さない");
    ch.note_external(&changed);

    let res = ch.undo_save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed), "{res:?}");
    assert_eq!(read(&dir, "b.md"), B_DONE_EXT);
    assert_eq!(ch.count(), 0);
}

#[test]
fn test_wb_21_no_save_means_nothing_to_undo() {
    // [WB-21] まだ保存していない → 覚えは空で、取り消しは何も書かず、結果も空。ためた変更にも触らない。
    let (dir, mut src) = setup("nosave");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    assert!(ch.saved_units().is_empty());
    assert!(ch.undo_save(&mut src).is_empty());

    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.saved_units().is_empty(), "保存の前に覚えがある");
    assert!(ch.undo_save(&mut src).is_empty());
    assert_eq!(ch.count(), 1);
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B);
}

#[test]
fn test_wb_21_only_last_save_is_undone_other_file() {
    // [WB-21] 2回保存した(1回目 a.md、2回目 b.md)→ 戻るのは2回目だけ。a.md は書いたまま。
    let (dir, mut src) = setup("last-other");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);

    let units = ch.saved_units();
    assert_eq!(units.len(), 1, "{units:?}");
    assert_eq!(units[0].0, b);
    assert_eq!(units[0].2.as_slice(), B.as_bytes());

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 1, "{res:?}");
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved));
    assert_eq!(read(&dir, "b.md"), B);
    assert_eq!(read(&dir, "a.md"), A_DONE, "1回目の保存まで戻った");
    assert_eq!(ch.count(), 1);
    assert!(ch.pending(&a, "status").is_none());
    // さかのぼらない: もう戻す保存は無い。
    assert!(ch.saved_units().is_empty());
    assert!(ch.undo_save(&mut src).is_empty());
    assert_eq!(read(&dir, "a.md"), A_DONE);
}

#[test]
fn test_wb_21_only_last_save_is_undone_same_file() {
    // [WB-21] 同じファイルを2回保存した → 戻る先は2回目の保存の前(1回目に書いた中身)。
    let (dir, mut src) = setup("last-same");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    assert!(ch.set(&src, &b, "title", s("B2")).is_ok());
    save_ok(&mut ch, &mut src);
    assert_eq!(read(&dir, "b.md"), B_DONE.replace("title: B", "title: B2"));

    let units = ch.saved_units();
    assert_eq!(units.len(), 1, "{units:?}");
    assert_eq!(units[0].2.as_slice(), B_DONE.as_bytes());

    ch.undo_save(&mut src);
    assert_eq!(read(&dir, "b.md"), B_DONE);
    assert_eq!(ch.count(), 1);
    assert_eq!(pending_str(&ch, &b, "title").as_deref(), Some("B2"));
    assert!(ch.pending(&b, "status").is_none());
}

#[test]
fn test_wb_21_save_that_wrote_nothing_keeps_previous_memory() {
    // [WB-21] 設計: 1つも書けなかった保存(外で変わって止まった)は、前の保存の覚えを置き換えない。
    let (dir, mut src) = setup("keep-memory");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);

    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    external_edit(
        &dir,
        "b.md",
        "---\nstatus: todo\ntitle: B outside\n---\nbody b\n",
    );
    let res = ch.save(&mut src);
    assert!(matches!(outcome_of(&res, &b), Outcome::Changed), "{res:?}");

    let units: Vec<RowId> = ch.saved_units().into_iter().map(|u| u.0).collect();
    assert_eq!(units, vec![a.clone()]);
    let res = ch.undo_save(&mut src);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved), "{res:?}");
    assert_eq!(read(&dir, "a.md"), A);
    // 止まった行のためた変更は残る(WB-14)。
    assert_eq!(pending_str(&ch, &b, "status").as_deref(), Some("done"));
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert_eq!(ch.count(), 2);
}

#[test]
fn test_wb_21_undo_save_keeps_values_set_after_the_save() {
    // [WB-21] 設計: 保存のあとにその行へ新しくためた値は残す(別の列は両方、同じ列は新しい方)。
    // 基準は戻したあとのファイルに進むので、そのまま保存できる。
    let (dir, mut src) = setup("newer");
    let a = row(&src, "a.md");
    let b = row(&src, "b.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    assert!(ch.set(&src, &b, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    // 保存のあと: a は別の列、b は同じ列に新しい値をためる。
    assert!(ch.set(&src, &a, "title", s("A2")).is_ok());
    assert!(ch.set(&src, &b, "status", s("wip")).is_ok());
    assert_eq!(ch.count(), 2);

    let res = ch.undo_save(&mut src);
    assert!(matches!(outcome_of(&res, &a), Outcome::Saved), "{res:?}");
    assert!(matches!(outcome_of(&res, &b), Outcome::Saved), "{res:?}");
    assert_eq!(read(&dir, "a.md"), A);
    assert_eq!(read(&dir, "b.md"), B);
    assert_eq!(pending_str(&ch, &a, "status").as_deref(), Some("done"));
    assert_eq!(pending_str(&ch, &a, "title").as_deref(), Some("A2"));
    assert_eq!(pending_str(&ch, &b, "status").as_deref(), Some("wip"));
    assert_eq!(ch.count(), 3);
    assert!(!ch.external(&a) && !ch.external(&b), "外で変更の印が付いた");

    save_ok(&mut ch, &mut src);
    assert_eq!(ch.count(), 0);
    assert_eq!(read(&dir, "b.md"), B.replace("status: todo", "status: wip"));
    let text = read(&dir, "a.md");
    assert!(text.contains("status:   done\n"), "{text}");
    assert!(text.contains("A2"), "{text}");
}

#[test]
fn test_wb_21_no_backup_file_in_the_folder() {
    // [WB-21] [WB-2] 保存しても取り消しても、ノートのフォルダに控えのファイルを作らない。
    let (dir, mut src) = setup("nobackup");
    let a = row(&src, "a.md");
    let before = dir.names();
    assert_eq!(before, vec!["a.md".to_string(), "b.md".to_string()]);
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);
    assert_eq!(dir.names(), before, "保存のあとに控えがある");
    ch.undo_save(&mut src);
    assert_eq!(dir.names(), before, "取り消しのあとに控えがある");
}

#[test]
fn test_wb_21_memory_lives_only_in_the_running_changes() {
    // [WB-21] 覚えるのは起動している間だけ: 新しく作った Changes(起動し直し)には戻す保存が無い。
    let (dir, mut src) = setup("session");
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &a, "status", s("done")).is_ok());
    save_ok(&mut ch, &mut src);

    let mut fresh = Changes::new();
    assert!(fresh.saved_units().is_empty());
    assert!(fresh.undo_save(&mut src).is_empty());
    assert_eq!(read(&dir, "a.md"), A_DONE);
}

// ---- CSV の読み込み口 ----

/// BOM・CRLF・引用符を持つ台帳(バイトごと戻ることを見る)。
const LEDGER: &str = "\u{FEFF}name,qty,note\r\napple,3,\"red, sweet\"\r\norange,12,\r\n";

#[test]
fn test_wb_21_undo_save_works_through_csv_source() {
    // [WB-21] CSV の読み込み口でも: 2つの行を直して保存 → 取り消す → ファイル(書く単位)は保存の前と
    // 1バイトも違わず、書いた値はためる変更に戻る。
    let dir = TempDir::new("csv");
    let file = dir.write("ledger.csv", LEDGER.as_bytes());
    let mut src = Csv::open(&file).expect("open csv");
    load_all(&mut src);
    let apple = row(&src, "apple");
    let orange = row(&src, "orange");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &apple, "qty", s("5")).is_ok());
    assert!(ch.set(&src, &orange, "note", s("sour")).is_ok());
    save_ok(&mut ch, &mut src);
    let saved = std::fs::read(&file).unwrap();
    assert_ne!(saved, LEDGER.as_bytes(), "保存で書かれていない");
    assert_eq!(ch.count(), 0);

    let units = ch.saved_units();
    assert_eq!(units.len(), 1, "CSV の書く単位はファイル1つ: {units:?}");
    assert_eq!(units[0].0, src.unit(&apple));
    assert_eq!(units[0].1.len, saved.len() as u64);
    assert_eq!(units[0].2.as_slice(), LEDGER.as_bytes());

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 1, "{res:?}");
    assert!(matches!(res[0].1, Outcome::Saved), "{res:?}");
    assert_eq!(std::fs::read(&file).unwrap(), LEDGER.as_bytes());
    assert_eq!(ch.count(), 2);
    assert_eq!(pending_str(&ch, &apple, "qty").as_deref(), Some("5"));
    assert_eq!(pending_str(&ch, &orange, "note").as_deref(), Some("sour"));
    assert_eq!(src.get(&apple, "qty").value, Some(Value::Int(3)));
    assert_eq!(dir.names(), vec!["ledger.csv".to_string()]);

    // 1手で消え、直してもう一度保存できる。
    assert!(ch.undo());
    assert_eq!(ch.count(), 0);
    assert!(ch.set(&src, &apple, "qty", s("7")).is_ok());
    save_ok(&mut ch, &mut src);
    assert_eq!(
        std::fs::read(&file).unwrap(),
        LEDGER.replace("apple,3", "apple,7").as_bytes()
    );
}

#[test]
fn test_wb_21_csv_changed_outside_is_not_restored() {
    // [WB-21] CSV でも、保存のあとに外で変わったファイルは書かない。
    let dir = TempDir::new("csv-outside");
    let file = dir.write("ledger.csv", LEDGER.as_bytes());
    let mut src = Csv::open(&file).expect("open csv");
    load_all(&mut src);
    let apple = row(&src, "apple");
    let mut ch = Changes::new();
    assert!(ch.set(&src, &apple, "qty", s("5")).is_ok());
    save_ok(&mut ch, &mut src);
    let outside = format!(
        "{}melon,1,\r\n",
        String::from_utf8(std::fs::read(&file).unwrap()).unwrap()
    );
    external_edit(&dir, "ledger.csv", &outside);

    let res = ch.undo_save(&mut src);
    assert_eq!(res.len(), 1, "{res:?}");
    assert!(matches!(res[0].1, Outcome::Changed), "{res:?}");
    assert_eq!(std::fs::read(&file).unwrap(), outside.as_bytes());
    assert_eq!(ch.count(), 0);
}
