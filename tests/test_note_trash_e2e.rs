#![cfg(unix)]
//! ノートの削除(ごみ箱へ動かす)・作成の取り消し・ごみ箱から戻す を、本物の実行ファイルの画面で確かめる。
//! 期待値は specs/requirements/cell-edit.md の cell-edit.delete-note-menu・cell-edit.new-note-undo・
//! cell-edit.trash-restore-list の受け入れの例から。

use crate::pty;

use std::path::{Path, PathBuf};

use pty::{TempDir, Tui};

const BETA: &str = "---\nstatus: doing\n---\nbeta の本文\n";

/// 3つのノート(alpha・beta・gamma)の保管庫。
fn vault(tag: &str) -> (TempDir, PathBuf) {
    let home = TempDir::new(&format!("note-trash-{tag}"));
    for d in ["notes", "config", "state"] {
        std::fs::create_dir_all(home.path().join(d)).unwrap();
    }
    let v = home.path().join("notes");
    std::fs::write(v.join("alpha.md"), "---\nstatus: todo\n---\n").unwrap();
    std::fs::write(v.join("beta.md"), BETA).unwrap();
    std::fs::write(v.join("gamma.md"), "---\nstatus: done\n---\n").unwrap();
    (home, v)
}

fn open(home: &TempDir, v: &Path) -> Tui {
    let t = Tui::spawn(&[v.to_str().unwrap()], home.path(), &[]);
    t.wait_for("alpha");
    t.wait_for("gamma");
    t
}

/// ごみ箱(`.trash/`)の中のファイル(名前, バイト)。
fn trashed(v: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![v.join(".trash")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                out.push((name, std::fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

/// 表にその名前の行があるか(選んだ行の印 `>` を除いた最初の語がノートの名前。メッセージの行の `beta.md` は数えない)。
fn row_shown(screen: &str, name: &str) -> bool {
    screen
        .lines()
        .any(|l| l.trim_start_matches('>').split_whitespace().next() == Some(name))
}

/// beta の行で、パレットから「ノートを消す」を選び、確かめに y と答える。
fn delete_beta(t: &mut Tui, v: &Path) {
    t.send_settle("j");
    t.send_settle(":");
    t.send_settle("ノートを消す");
    t.wait_for("ノートを消す");
    t.send_settle("\r");
    t.wait_for("ごみ箱へ動かす?");
    t.wait_for("消すノート: beta.md");
    t.send("y\r");
    let gone = v.join("beta.md");
    t.wait_until("beta.md が元の場所から無くなる", |_| {
        !gone.exists()
    });
}

#[test]
fn test_delete_note_from_the_palette_moves_it_to_the_trash() {
    // [stest -> req~cell-edit.delete-note-menu~1] beta の行で「ノートを消す」に y → `beta.md` が元の場所に無く、ごみ箱に同じバイトであり、表から beta の行が無くなる。
    let (home, v) = vault("delete");
    let mut t = open(&home, &v);
    delete_beta(&mut t, &v);
    t.wait_until("表から beta の行が無くなる", |s| {
        !row_shown(s, "beta")
    });
    assert!(!v.join("beta.md").exists());
    assert_eq!(
        trashed(&v),
        vec![("beta.md".to_string(), BETA.as_bytes().to_vec())]
    );
    let screen = t.screen();
    assert!(screen.contains("alpha"), "{screen}");
    assert!(screen.contains("gamma"), "{screen}");
}

#[test]
fn test_undo_right_after_creating_a_note_moves_it_to_the_trash() {
    // [stest -> req~cell-edit.new-note-undo~1] 「zeta」を作って `u` → `zeta.md` が元の場所に無く、ごみ箱に同じバイトであり、表から zeta の行が無くなる。
    let (home, v) = vault("undo");
    let mut t = open(&home, &v);
    t.send_settle("a");
    t.send_settle("zeta");
    t.send("\x13");
    let made_at = v.join("zeta.md");
    t.wait_until("zeta.md ができる", |_| made_at.exists());
    t.wait_for("zeta");
    let made = std::fs::read(&made_at).unwrap();
    t.send("u");
    t.wait_until("zeta.md が元の場所から無くなる", |_| {
        !made_at.exists()
    });
    t.wait_until("表から zeta の行が無くなる", |s| {
        !row_shown(s, "zeta")
    });
    assert!(!made_at.exists());
    assert_eq!(trashed(&v), vec![("zeta.md".to_string(), made)]);
}

#[test]
fn test_restore_from_the_trash_after_a_restart() {
    // [stest -> req~cell-edit.trash-restore-list~1] `beta.md` を消して起動し直し、「ごみ箱から戻す」で beta を選ぶ → `beta.md` が同じバイトで戻り、表に beta の行が出る。
    let (home, v) = vault("restore");
    let mut t = open(&home, &v);
    delete_beta(&mut t, &v);
    t.stop();
    let mut t = open(&home, &v);
    t.send_settle(":");
    t.send_settle("ごみ箱から戻す");
    t.send_settle("\r");
    t.wait_for("ごみ箱から戻すノート");
    t.wait_for("beta.md");
    t.send("\r");
    let back = v.join("beta.md");
    t.wait_until("beta.md が元の場所に戻る", |_| back.exists());
    t.wait_until("表に beta の行が出る", |s| row_shown(s, "beta"));
    assert_eq!(std::fs::read(&back).unwrap(), BETA.as_bytes());
    assert_eq!(trashed(&v), Vec::new());
}
