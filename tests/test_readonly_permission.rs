//! 書き込みの権限が無いノートを読むだけにする(readonly-permission)の受け入れテスト。WB-5(関係: WB-14・CE-10)。
//! 仕様: specs/write-back/spec.md の WB-5・WB-14、specs/cell-edit/spec.md の CE-10。
//! 記録: specs/_changes/2026-10-03-readonly-permission.md。
//! 決定: specs/_decisions/2026-10-03-readonly-permission.md。
//!
//! 実装を見ずに、仕様と公開のインターフェース(tests/test_unwritable.rs・tests/test_changes.rs と同じ口)だけから書いた。
//! 権限の理由は読み取りの層の ReadOnly には足さず、開く側(Source)の読むだけの理由として持つ(記録の「設計」)ので、
//! ここではセルの lock の文に「書き込めない権限」を含むことだけを確かめ、ReadOnly の種類には触れない。
//!
//! root で走ると 0444 でも書けてしまうので、その場合は各試験の頭で黙って飛ばす(`running_as_root`)。

#![cfg(unix)]

use mdgrid::changes::{Changes, Outcome};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{Edit, NewValue, RowId, Source};
use mdgrid::writeback::{baseline, save};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// 理由の文に含まれるべき言葉(WB-5 の例)。
const REASON: &str = "書き込めない権限";

const NOTE: &[u8] = b"---\ntitle: Locked\nstatus: todo\n---\nbody\n";
const OK: &[u8] = b"---\nstatus: todo\n---\nok\n";

// ---- 一時フォルダ(終わりに権限を戻して消す) ----

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
        // 試験で付けた 0444 を戻してから消す。
        let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        if let Ok(entries) = std::fs::read_dir(&self.0) {
            for e in entries.flatten() {
                let _ = std::fs::set_permissions(e.path(), std::fs::Permissions::from_mode(0o644));
            }
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 道具 ----

fn chmod(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

fn mode_of(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// root なら 0444 のファイルでも書き込みに開けてしまう。一時ファイルで確かめる(libc は使わない)。
fn running_as_root() -> bool {
    let dir = TempDir::new("perm-rootcheck");
    let p = dir.write("probe.md", b"x");
    chmod(&p, 0o444);
    std::fs::OpenOptions::new().write(true).open(&p).is_ok()
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load が終わらない");
}

fn open(dir: &TempDir) -> Markdown {
    let mut src = Markdown::open(&[dir.path().to_path_buf()]).expect("Markdown::open");
    load_all(&mut src);
    src
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| {
            let labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
            panic!("行 {label} が無い: {labels:?}")
        })
}

fn done() -> NewValue {
    NewValue::Str("done".into())
}

// ---- 1. 開いたときに読むだけになり、理由が出る ----

#[test]
fn test_wb_5_permission_0444_note_cells_are_locked_with_reason() {
    // [WB-5] 0444 のノートと普通のノートを一緒に開く → 0444 のノートのセル(status・title・無いキー)は lock が Some で、
    // 理由に「書き込めない権限」を含む。普通のノートは lock が None(行ごとの lock が漏れない)。
    // Changes::set は Skip(理由はセルの lock と同じ)で何もためない。普通のノートはためる。
    if running_as_root() {
        return;
    }
    let dir = TempDir::new("perm-lock");
    let locked = dir.write("locked.md", NOTE);
    dir.write("ok.md", OK);
    chmod(&locked, 0o444);
    let src = open(&dir);

    let row = row_of(&src, "locked.md");
    for col in ["status", "title", "missing"] {
        let lock = src.get(&row, col).lock;
        let r = lock.unwrap_or_else(|| {
            panic!("[WB-5] 0444 のノートの {col} が読むだけにならない(lock が None)")
        });
        assert!(
            r.contains(REASON),
            "[WB-5] 0444 のノートの {col} の理由 {r:?} に {REASON:?} が無い"
        );
    }

    let ok = row_of(&src, "ok.md");
    assert_eq!(
        src.get(&ok, "status").lock,
        None,
        "[WB-5] 普通のノートは書ける"
    );

    let lock = src.get(&row, "status").lock;
    let mut ch = Changes::new();
    match ch.set(&src, &row, "status", done()) {
        Err(s) => {
            assert_eq!(s.row, row);
            assert_eq!(
                Some(&s.reason),
                lock.as_ref(),
                "[WB-5] Skip の理由がセルの lock と違う"
            );
        }
        Ok(()) => panic!("[WB-5] 0444 のノートに変更をためられてしまう"),
    }
    assert_eq!(ch.count(), 0);
    assert!(ch.set(&src, &ok, "status", done()).is_ok());
    assert_eq!(ch.count(), 1);

    assert_eq!(std::fs::read(&locked).unwrap(), NOTE);
    assert_eq!(mode_of(&locked), 0o444);
}

// ---- 2. ためたあとに権限が無くなった → 保存で止まり、ためた変更を残す(WB-14) ----

#[test]
fn test_wb_5_permission_revoked_after_pending_stops_save_and_keeps_changes() {
    // [WB-5] [WB-14] 0644 のノートの status をためる → 外で chmod 444 → 保存 → その行は Saved にならず
    // (Failed か Changed)、ファイルのバイトと権限(0444)は前と同じ。未保存の数は減らない。
    // 隣の書けるノートは書かれる。
    if running_as_root() {
        return;
    }
    let dir = TempDir::new("perm-revoke");
    let note = dir.write("note.md", NOTE);
    dir.write("ok.md", OK);
    let mut src = open(&dir);
    let row = row_of(&src, "note.md");
    let ok = row_of(&src, "ok.md");
    assert_eq!(src.get(&row, "status").lock, None, "0644 のノートは書ける");

    let mut ch = Changes::new();
    ch.set(&src, &row, "status", done())
        .expect("0644 にはためられる");
    ch.set(&src, &row, "title", NewValue::Str("New".into()))
        .expect("0644 にはためられる");
    ch.set(&src, &ok, "status", done()).expect("ok.md にためる");
    assert_eq!(ch.count(), 3);

    chmod(&note, 0o444);
    let res = ch.save(&mut src);

    let mine: Vec<&Outcome> = res
        .iter()
        .filter(|(r, _)| *r == row)
        .map(|(_, o)| o)
        .collect();
    assert_eq!(
        mine.len(),
        1,
        "[WB-5] 止まった行が保存の結果に1つ出る: {res:?}"
    );
    assert!(
        !matches!(mine[0], Outcome::Saved),
        "[WB-5] 権限の無くなったノートに書けてしまう: {:?}",
        mine[0]
    );
    assert_eq!(
        std::fs::read(&note).unwrap(),
        NOTE,
        "[WB-5] ファイルの中身が変わった"
    );
    assert_eq!(mode_of(&note), 0o444, "[WB-5] ファイルの権限が変わった");

    assert_eq!(ch.count(), 2, "[WB-14] 止まった行のためた変更が減った");
    assert_eq!(ch.pending(&row, "status"), Some(&done()));
    assert!(ch.pending(&row, "title").is_some());

    let ok_res = res.iter().find(|(r, _)| *r == ok).map(|(_, o)| o);
    assert!(
        matches!(ok_res, Some(Outcome::Saved)),
        "隣の ok.md は書かれる: {ok_res:?}"
    );
    assert_eq!(
        std::fs::read(dir.path().join("ok.md")).unwrap(),
        b"---\nstatus: done\n---\nok\n"
    );
}

// ---- 3. 書き戻しの関数を直接呼んでも書かない ----

#[test]
fn test_wb_5_permission_writeback_save_refuses_0444() {
    // [WB-5] writeback::save を 0444 のファイルに直接呼ぶ → Err で、ファイルのバイトと権限は前と同じ。
    // 基準は 0644 のうちに取り、そのあと chmod 444(ためたあとに権限が無くなった形)でも、
    // 0444 のまま基準を取っても、どちらも書かない。
    if running_as_root() {
        return;
    }
    let dir = TempDir::new("perm-wbsave");
    let edits = vec![Edit {
        key: "status".into(),
        value: done(),
    }];

    // 0444 のまま基準を取る。
    let a = dir.write("a.md", NOTE);
    chmod(&a, 0o444);
    let base = baseline(&a).unwrap();
    let got = save(&a, &base, &edits);
    assert!(
        got.is_err(),
        "[WB-5] 0444 のファイルに書けてしまう: {got:?}"
    );
    assert_eq!(std::fs::read(&a).unwrap(), NOTE, "[WB-5] a.md が変わった");
    assert_eq!(mode_of(&a), 0o444, "[WB-5] a.md の権限が変わった");

    // 0644 で基準を取ってから chmod 444。
    let b = dir.write("b.md", NOTE);
    let base = baseline(&b).unwrap();
    chmod(&b, 0o444);
    let got = save(&b, &base, &edits);
    assert!(
        got.is_err(),
        "[WB-5] 権限の無くなったファイルに書けてしまう: {got:?}"
    );
    assert_eq!(std::fs::read(&b).unwrap(), NOTE, "[WB-5] b.md が変わった");
    assert_eq!(mode_of(&b), 0o444, "[WB-5] b.md の権限が変わった");

    // 対照: 0644 のファイルには書ける。
    let c = dir.write("c.md", NOTE);
    let base = baseline(&c).unwrap();
    save(&c, &base, &edits).expect("0644 には書ける");
    assert!(String::from_utf8_lossy(&std::fs::read(&c).unwrap()).contains("status: done"));
}

// ---- 4. 一括の設定(CE-10)では飛ばして数と理由を返す ----

#[test]
fn test_wb_5_permission_set_many_skips_0444_rows() {
    // [WB-5] [CE-10] 0444 のノート2つと普通のノート2つを選んで set_many → 0444 の2行は飛ばされ、
    // 飛ばした数(2)と理由(セルの lock と同じで「書き込めない権限」を含む)が返る。ためるのは書ける2行だけ。
    // 保存しても 0444 のファイルは変わらない。
    if running_as_root() {
        return;
    }
    let dir = TempDir::new("perm-many");
    let l1 = dir.write("l1.md", NOTE);
    let l2 = dir.write("l2.md", NOTE);
    dir.write("ok1.md", OK);
    dir.write("ok2.md", OK);
    chmod(&l1, 0o444);
    chmod(&l2, 0o444);
    let mut src = open(&dir);

    let locked = vec![row_of(&src, "l1.md"), row_of(&src, "l2.md")];
    let oks = vec![row_of(&src, "ok1.md"), row_of(&src, "ok2.md")];
    let rows: Vec<RowId> = vec![
        locked[0].clone(),
        oks[0].clone(),
        locked[1].clone(),
        oks[1].clone(),
    ];

    let mut ch = Changes::new();
    let skips = ch.set_many(&src, &rows, "status", done());
    assert_eq!(
        skips.len(),
        2,
        "[CE-10] 飛ばした数が 0444 の行の数でない: {skips:?}"
    );
    for s in &skips {
        assert!(locked.contains(&s.row), "[CE-10] 飛ばした行が違う: {s:?}");
        assert!(
            s.reason.contains(REASON),
            "[WB-5] 飛ばした理由 {:?} に {REASON:?} が無い",
            s.reason
        );
        assert_eq!(
            Some(&s.reason),
            src.get(&s.row, "status").lock.as_ref(),
            "[CE-10] 飛ばした理由がセルの lock と違う"
        );
    }
    assert_eq!(ch.count(), 2, "[CE-10] ためるのは書ける行だけ");
    for r in &oks {
        assert_eq!(ch.pending(r, "status"), Some(&done()));
    }
    for r in &locked {
        assert_eq!(ch.pending(r, "status"), None);
    }

    let _ = ch.save(&mut src);
    for p in [&l1, &l2] {
        assert_eq!(std::fs::read(p).unwrap(), NOTE, "[WB-5] {p:?} が変わった");
        assert_eq!(mode_of(p), 0o444);
    }
    for name in ["ok1.md", "ok2.md"] {
        assert_eq!(
            std::fs::read(dir.path().join(name)).unwrap(),
            b"---\nstatus: done\n---\nok\n"
        );
    }
}
