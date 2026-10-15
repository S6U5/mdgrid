//! Windows の書き戻しの安全(WB-5・WB-8・WB-14)。仕様: specs/write-back/spec.md。
//! ほかのアプリが開いているノート、読み取り専用の属性、日本語の名前を Windows で確かめる
//! (specs/_changes/2026-10-15-windows-target.md)。ハードリンクの見分けは Windows では未対応(docs/todo.md の O-11)。
#![cfg(windows)]

use mdgrid::writeback::{baseline, save, Edit, NewValue, SaveError};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const NOTE: &[u8] = b"---\nstatus: todo\n---\nbody\n";
const DONE: &[u8] = b"---\nstatus: done\n---\nbody\n";
// Win32 の共有の bit(FILE_SHARE_READ・FILE_SHARE_WRITE)。削除の共有(FILE_SHARE_DELETE)は付けない。
const SHARE_READ_WRITE: u32 = 0x1 | 0x2;

fn done() -> Vec<Edit> {
    vec![Edit {
        key: "status".to_string(),
        value: NewValue::Str("done".to_string()),
    }]
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-win-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
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

#[allow(clippy::permissions_set_readonly_false)]
fn set_readonly(p: &Path, on: bool) {
    let mut perm = std::fs::metadata(p).unwrap().permissions();
    perm.set_readonly(on);
    std::fs::set_permissions(p, perm).unwrap();
}

#[test]
fn test_wb_8_windows_note_held_open_by_another_app() {
    // [WB-8] [WB-14] ほかのアプリが削除の共有なしで開いているノートへの保存は、名前の変更で置き換えられれば
    // 書き終えた中身に、置き換えられなければ誤りで止まって元のまま。どちらでも一時ファイルを残さず、
    // 閉じたあとのやり直しで書ける。
    let dir = TempDir::new("held");
    let note = dir.write("note.md", NOTE);
    let base = baseline(&note).unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARE_READ_WRITE)
        .open(&note)
        .unwrap();
    let r = save(&note, &base, &done());
    drop(held);
    let now = std::fs::read(&note).unwrap();
    match r {
        Ok(_) => assert_eq!(now, DONE),
        Err(e) => {
            assert_eq!(now, NOTE, "{e:?}");
            let base = baseline(&note).unwrap();
            save(&note, &base, &done()).expect("save after the other app closed the note");
            assert_eq!(std::fs::read(&note).unwrap(), DONE);
        }
    }
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[test]
fn test_wb_5_windows_read_only_attribute_is_not_written() {
    // [WB-5] 読み取り専用の属性のノート(unix の chmod 444 に当たる)は書かずに止め、元のまま。
    let dir = TempDir::new("readonly");
    let note = dir.write("note.md", NOTE);
    let base = baseline(&note).unwrap();
    set_readonly(&note, true);
    let r = save(&note, &base, &done());
    set_readonly(&note, false);
    assert!(matches!(r, Err(SaveError::NoPermission)), "{r:?}");
    assert_eq!(std::fs::read(&note).unwrap(), NOTE);
    assert_eq!(dir.names(), vec!["note.md".to_string()]);
}

#[test]
fn test_wb_8_windows_japanese_name() {
    // [WB-8] 日本語と空白を含む名前のノートも、一時ファイルと名前の変更で書ける。
    let dir = TempDir::new("ja");
    let note = dir.write("会議 メモ.md", NOTE);
    let base = baseline(&note).unwrap();
    save(&note, &base, &done()).expect("save");
    assert_eq!(std::fs::read(&note).unwrap(), DONE);
    assert_eq!(dir.names(), vec!["会議 メモ.md".to_string()]);
}
