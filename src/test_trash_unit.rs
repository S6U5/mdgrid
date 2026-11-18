//! ごみ箱(trash.rs)の単体の試験。期待値は specs/requirements/cell-edit.md の受け入れの例から。

use super::*;

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-trash-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Tmp(dir.canonicalize().unwrap())
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_put_moves_the_note_into_the_trash_with_the_same_bytes() {
    // [utest -> req~cell-edit.delete-note-menu~1] 消したノートは元の場所に無く、ごみ箱に同じバイトである。
    let t = Tmp::new("put");
    let p = t.write("プロジェクト/メモ.md", "---\nstatus: todo\n---\n本文\n");
    let item = put(&t.0, &p, 1_800_000_000).unwrap();
    assert!(!p.exists());
    assert_eq!(
        std::fs::read(&item.at).unwrap(),
        "---\nstatus: todo\n---\n本文\n".as_bytes()
    );
    assert!(item.at.starts_with(t.0.join(".trash")));
    assert_eq!(item.rel, "プロジェクト/メモ.md");
    assert_eq!(item.when, 1_800_000_000);
}

#[test]
fn test_put_fails_and_leaves_the_note_when_the_trash_cannot_be_made() {
    // [utest -> req~cell-edit.delete-note-trash-only~1] ごみ箱へ動かせないとき、ノートは元の場所に同じバイトで残る。
    // 前提: `.trash` という名前のファイルがあって、ごみ箱のフォルダを作れない。
    let t = Tmp::new("blocked");
    t.write(".trash", "ファイル");
    let p = t.write("メモ.md", "---\nstatus: todo\n---\n");
    let r = put(&t.0, &p, 1_800_000_000);
    assert!(r.is_err());
    assert_eq!(
        std::fs::read(&p).unwrap(),
        "---\nstatus: todo\n---\n".as_bytes()
    );
    assert_eq!(
        std::fs::read(t.0.join(".trash")).unwrap(),
        "ファイル".as_bytes()
    );
}

#[test]
fn test_put_twice_with_the_same_name_keeps_both_in_the_trash() {
    // [utest -> req~cell-edit.delete-note-trash-keep-old~1] 同じ名前のノートを同じ時刻に2回消しても、ごみ箱に両方が同じバイトである。
    let t = Tmp::new("twice");
    let p = t.write("メモ.md", "古い");
    let old = put(&t.0, &p, 1_800_000_000).unwrap();
    let p = t.write("メモ.md", "新しい");
    let new = put(&t.0, &p, 1_800_000_000).unwrap();
    assert_ne!(old.at, new.at);
    assert_eq!(std::fs::read(&old.at).unwrap(), "古い".as_bytes());
    assert_eq!(std::fs::read(&new.at).unwrap(), "新しい".as_bytes());
}

#[test]
fn test_restore_puts_the_note_back_where_it_was() {
    // [utest -> req~cell-edit.trash-restore-list~1] ごみ箱のノートは、元のフォルダへ同じバイトで戻り、ごみ箱から無くなる。
    let t = Tmp::new("restore");
    let p = t.write("プロジェクト/メモ.md", "---\nstatus: todo\n---\n");
    let item = put(&t.0, &p, 1_800_000_000).unwrap();
    let back = restore(&item).unwrap();
    assert_eq!(back, p);
    assert_eq!(
        std::fs::read(&p).unwrap(),
        "---\nstatus: todo\n---\n".as_bytes()
    );
    assert!(!item.at.exists());
    assert_eq!(list(&t.0), Vec::new());
}

#[test]
fn test_restore_refuses_when_a_file_is_at_the_original_place() {
    // [utest -> req~cell-edit.trash-restore-keep-existing~1] 元の場所に同じ名前のファイルがあれば断り、どちらのファイルも同じバイトで残る。
    let t = Tmp::new("exists");
    let p = t.write("メモ.md", "消したほう");
    let item = put(&t.0, &p, 1_800_000_000).unwrap();
    t.write("メモ.md", "あとから作ったほう");
    let r = restore(&item);
    assert_eq!(r.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&p).unwrap(), "あとから作ったほう".as_bytes());
    assert_eq!(std::fs::read(&item.at).unwrap(), "消したほう".as_bytes());
}

#[test]
fn test_list_shows_the_original_place_and_time_newest_first() {
    // [utest -> req~cell-edit.trash-restore-list~1] 一覧は、元の場所と消した時刻を持ち、新しい順に並ぶ。ごみ箱が無ければ空。
    let t = Tmp::new("list");
    assert_eq!(list(&t.0), Vec::new());
    let a = t.write("a.md", "a");
    let b = t.write("下/b.md", "b");
    let ia = put(&t.0, &a, 1_800_000_000).unwrap();
    let ib = put(&t.0, &b, 1_800_000_060).unwrap();
    assert_eq!(list(&t.0), vec![ib, ia]);
}
