//! 保存の取り消し(WB-21)のレビューの指摘の直しの試験。記録: specs/_changes/2026-11-13-save-undo.md の
//! 「レビューの指摘」の 1・2・3・5。実装した本人が書いた(独立したテスト役ではない)。値はどれも作ったもの。

use mdgrid::changes::{Changes, Outcome};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId, Source};
use std::path::PathBuf;
use std::time::Duration;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-save-undo-fixes-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(dir: &TempDir, notes: &[(&str, &str)]) -> Markdown {
    for (name, text) in notes {
        std::fs::write(dir.0.join(name), text).unwrap();
    }
    let mut src = Markdown::open(std::slice::from_ref(&dir.0)).expect("open");
    for _ in 0..10_000 {
        if src.load(100).done {
            return src;
        }
    }
    panic!("load が終わらない");
}

fn row(src: &dyn Source, name: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == name)
        .unwrap_or_else(|| panic!("行 {name} が無い"))
}

fn read(dir: &TempDir, name: &str) -> String {
    std::fs::read_to_string(dir.0.join(name)).unwrap()
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn all_saved(res: &[(RowId, Outcome)]) {
    assert!(!res.is_empty(), "結果が空");
    for (r, o) in res {
        assert!(matches!(o, Outcome::Saved), "{r:?}: {o:?}");
    }
}

const TODO: &str = "---\nstatus: todo\nowner: mika\n---\n";
const DONE: &str = "---\nstatus: done\nowner: mika\n---\n";

#[test]
fn test_wb_21_undo_save_does_not_move_the_base_of_an_identical_note() {
    // [WB-21][WB-4] 保存のあとの a.md と1バイトも違わない b.md にためた変更の基準は、a.md を戻しても動かない
    // (b.md の次の保存が、外で変わっていないのに止まらない)。
    let dir = TempDir::new("same");
    let mut src = open(&dir, &[("a.md", TODO), ("b.md", DONE)]);
    let (a, b) = (row(&src, "a.md"), row(&src, "b.md"));
    let mut ch = Changes::new();
    ch.set(&src, &a, "status", s("done")).unwrap();
    all_saved(&ch.save(&mut src));
    assert_eq!(read(&dir, "a.md"), read(&dir, "b.md"), "材料: 中身が同じ");
    ch.set(&src, &b, "owner", s("ren")).unwrap();

    all_saved(&ch.undo_save(&mut src));
    assert_eq!(read(&dir, "a.md"), TODO);
    assert!(!ch.external(&b));

    let res = ch.save_rows(&mut src, std::slice::from_ref(&b));
    all_saved(&res);
    assert_eq!(read(&dir, "b.md"), DONE.replace("mika", "ren"));
    assert!(!ch.external(&b), "外で変わっていない b.md に印が付いた");
}

#[test]
fn test_wb_21_undo_save_restores_when_only_the_mtime_changed() {
    // [WB-21] 外で変わったかは中身で決める: 保存のあとに更新時刻だけが変わったノートは戻せる。
    let dir = TempDir::new("touch");
    let mut src = open(&dir, &[("a.md", TODO)]);
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    ch.set(&src, &a, "status", s("done")).unwrap();
    all_saved(&ch.save(&mut src));
    let p = dir.0.join("a.md");
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    let f = std::fs::File::options().write(true).open(&p).unwrap();
    f.set_modified(old + Duration::from_secs(30)).unwrap();
    drop(f);

    all_saved(&ch.undo_save(&mut src));
    assert_eq!(read(&dir, "a.md"), TODO);
    assert_eq!(ch.count(), 1);
}

#[test]
fn test_wb_21_undo_save_drops_a_pending_value_equal_to_the_restored_file() {
    // [WB-21][WB-17] 保存のあと、同じセルに保存の前と同じ値をためてから取り消すと、そのためた値は外れる
    // (戻したファイルの値と同じ値を、未保存と数えない)。
    let dir = TempDir::new("samevalue");
    let mut src = open(&dir, &[("a.md", TODO)]);
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    ch.set(&src, &a, "status", s("done")).unwrap();
    all_saved(&ch.save(&mut src));
    ch.set(&src, &a, "status", s("todo")).unwrap();
    assert_eq!(ch.count(), 1);

    all_saved(&ch.undo_save(&mut src));
    assert_eq!(read(&dir, "a.md"), TODO);
    assert_eq!(ch.count(), 0, "戻したファイルと同じ値が、ためたまま残った");
}

#[test]
fn test_wb_21_undo_save_moves_the_base_inside_the_undo_stack() {
    // [WB-21][WB-10] 保存 → 同じノートの別の列にためて捨てる → 保存を取り消す → 取り消しで捨てた手を戻す。
    // 戻った行の基準は、戻したあとのファイルなので、そのまま保存できる。
    let dir = TempDir::new("stack");
    let mut src = open(&dir, &[("a.md", TODO)]);
    let a = row(&src, "a.md");
    let mut ch = Changes::new();
    ch.set(&src, &a, "status", s("done")).unwrap();
    all_saved(&ch.save(&mut src));
    ch.set(&src, &a, "owner", s("ren")).unwrap();
    ch.discard(&a);
    assert_eq!(ch.count(), 0);

    all_saved(&ch.undo_save(&mut src));
    assert_eq!(ch.count(), 1, "status がためる変更に戻る");
    assert!(ch.undo(), "保存の取り消しの手");
    assert!(ch.undo(), "捨てた手");
    assert!(matches!(ch.pending(&a, "owner"), Some(NewValue::Str(v)) if v == "ren"));

    all_saved(&ch.save(&mut src));
    assert_eq!(read(&dir, "a.md"), TODO.replace("mika", "ren"));
    assert!(!ch.external(&a));
}
