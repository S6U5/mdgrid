//! タスク 4 の受け入れ: 保管庫の根・ノートの探索・読み込み・ポーリング・選択の保持。
//! 実装を見ずに、docs/design.md の `src/vault.rs` のインターフェースだけを使う。

use mdgrid::vault::{reselect, roots, Vault};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-vault-{}-{}-{}",
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

    /// `rel` の親フォルダも作って書く。書いたファイルの実体のパスを返す。
    fn write(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, bytes).unwrap();
        p.canonicalize().unwrap()
    }

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap()
}

/// 読み込みを最後まで進める(無限に回らないよう上限つき)。
fn load_all(v: &mut Vault) {
    for _ in 0..100_000 {
        if v.load(1000).done {
            return;
        }
    }
    panic!("load が done にならない");
}

fn open_loaded(folders: &[PathBuf]) -> Vault {
    let mut v = Vault::open(roots(folders).unwrap());
    load_all(&mut v);
    v
}

fn note_paths(v: &Vault) -> Vec<PathBuf> {
    v.notes().iter().map(|n| n.path.clone()).collect()
}

fn sorted(mut v: Vec<PathBuf>) -> Vec<PathBuf> {
    v.sort();
    v
}

/// 更新時刻を確実にずらす(時刻の粒度に頼らない)。
fn bump_mtime(p: &Path, secs: u64) {
    let f = std::fs::OpenOptions::new().write(true).open(p).unwrap();
    f.set_modified(SystemTime::now() + Duration::from_secs(secs))
        .unwrap();
}

// ---- BV-1 ----

#[test]
fn test_bv_1_folder_without_config_loads_all_markdown_notes() {
    // [BV-1] 設定も .base も無いフォルダ → 全ノートが入り、.md 以外は入らない
    let t = TempDir::new("bv1");
    let a = t.write("a.md", b"---\nstatus: todo\n---\nA\n");
    let b = t.write("sub/b.md", b"---\ntitle: B\n---\n");
    let c = t.write("sub/deeper/c.md", b"no frontmatter\n");
    t.write("readme.txt", b"not a note\n");
    t.write("image.png", b"\x89PNG");
    t.write("sub/data.base", b"views: []\n");

    let v = open_loaded(&[t.path().to_path_buf()]);
    assert_eq!(
        note_paths(&v),
        sorted(vec![a.clone(), b.clone(), c.clone()])
    );

    // bytes は読んだ中身、rel は根からの '/' 区切りの相対パス
    let nb = v.note(&b).expect("b.md が無い");
    assert_eq!(nb.bytes, b"---\ntitle: B\n---\n".to_vec());
    assert_eq!(nb.rel, "sub/b.md");
    assert_eq!(nb.stamp.len, nb.bytes.len() as u64);
    assert_eq!(v.note(&a).unwrap().rel, "a.md");
}

#[test]
fn test_bv_1_notes_are_sorted_by_path() {
    // [BV-1] notes() は path の昇順
    let t = TempDir::new("bv1sort");
    for name in ["c.md", "a.md", "b/z.md", "b/a.md"] {
        t.write(name, b"x\n");
    }
    let v = open_loaded(&[t.path().to_path_buf()]);
    let ps = note_paths(&v);
    assert_eq!(ps.len(), 4);
    assert_eq!(ps.clone(), sorted(ps));
}

#[test]
fn test_bv_1_skips_obsidian_git_and_trash() {
    // [BV-1] .obsidian/・.git/・.trash/ の下は探さない
    let t = TempDir::new("bv1skip");
    let n = t.write("note.md", b"x\n");
    t.write(".obsidian/workspace.md", b"x\n");
    t.write(".git/x.md", b"x\n");
    t.write(".trash/old.md", b"x\n");
    let v = open_loaded(&[t.path().to_path_buf()]);
    assert_eq!(note_paths(&v), vec![n]);
}

// ---- BV-2 ----

#[test]
fn test_bv_2_root_is_nearest_parent_with_obsidian() {
    // [BV-2] .obsidian/ を持つ最寄りの上のフォルダが根になる
    let t = TempDir::new("bv2up");
    t.mkdir("vault/.obsidian");
    let inner = t.mkdir("vault/projects/x");
    let r = roots(&[inner]).unwrap();
    assert_eq!(r, vec![canon(&t.path().join("vault"))]);
}

#[test]
fn test_bv_2_root_is_given_folder_without_obsidian() {
    // [BV-2] .obsidian/ が無ければ渡したフォルダが根
    let t = TempDir::new("bv2plain");
    let inner = t.mkdir("plain/sub");
    let r = roots(std::slice::from_ref(&inner)).unwrap();
    assert_eq!(r, vec![canon(&inner)]);
}

#[test]
fn test_bv_2_two_folders_side_by_side_load_both() {
    // [BV-2] 2つのリポのフォルダを並べて渡す → 両方のノートが入る
    let t = TempDir::new("bv2two");
    let a = t.write("repo_a/docs/one.md", b"---\nk: 1\n---\n");
    let b = t.write("repo_b/two.md", b"---\nk: 2\n---\n");
    let folders = vec![t.path().join("repo_a"), t.path().join("repo_b")];
    let r = roots(&folders).unwrap();
    assert_eq!(r.len(), 2);
    assert!(r.contains(&canon(&t.path().join("repo_a"))));
    assert!(r.contains(&canon(&t.path().join("repo_b"))));

    let v = open_loaded(&folders);
    assert_eq!(note_paths(&v), sorted(vec![a, b]));
}

#[test]
fn test_bv_2_nested_roots_merge_into_one() {
    // [BV-2] ほかの根の中に入る根は1つにまとめる
    let t = TempDir::new("bv2nest");
    let outer = t.mkdir("outer");
    let inner = t.mkdir("outer/inner");
    let r = roots(&[inner, outer.clone()]).unwrap();
    assert_eq!(r, vec![canon(&outer)]);
}

#[test]
fn test_bv_2_unreadable_path_is_error() {
    // [BV-2] 読めないパスは Err
    let t = TempDir::new("bv2err");
    assert!(roots(&[t.path().join("does-not-exist")]).is_err());
}

// ---- BV-11 ----

#[test]
fn test_bv_11_same_folder_twice_gives_one_note_each() {
    // [BV-11] 同じフォルダを2回渡す → ノートは1つずつ
    let t = TempDir::new("bv11twice");
    let a = t.write("v/a.md", b"a\n");
    let b = t.write("v/b.md", b"b\n");
    let dir = t.path().join("v");
    let v = open_loaded(&[dir.clone(), dir]);
    assert_eq!(note_paths(&v), sorted(vec![a, b]));
}

#[cfg(unix)]
#[test]
fn test_bv_11_symlink_to_same_folder_gives_one_note_each() {
    // [BV-11] 同じフォルダを2回と、そこを指すシンボリックリンク → ノートは1つずつ(path は実体)
    let t = TempDir::new("bv11link");
    let a = t.write("real/a.md", b"a\n");
    let b = t.write("real/sub/b.md", b"b\n");
    let real = t.path().join("real");
    let link = t.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    let v = open_loaded(&[real.clone(), real, link]);
    assert_eq!(note_paths(&v), sorted(vec![a, b]));
}

#[cfg(unix)]
#[test]
fn test_bv_11_symlink_inside_root_is_followed_once() {
    // [BV-11] 根の中のリンクが同じノートを指す → 1つ(path は実体)。リンクの輪は無視する
    let t = TempDir::new("bv11inner");
    let a = t.write("real/a.md", b"a\n");
    std::os::unix::fs::symlink(t.path().join("real"), t.path().join("alias")).unwrap();
    std::os::unix::fs::symlink(t.path().join("real/a.md"), t.path().join("a-link.md")).unwrap();
    // 輪: real/loop → real
    std::os::unix::fs::symlink(t.path().join("real"), t.path().join("real/loop")).unwrap();

    let v = open_loaded(&[t.path().to_path_buf()]);
    assert_eq!(note_paths(&v), vec![a]);
}

// ---- BV-9 ----

#[test]
fn test_bv_9_poll_reloads_only_the_changed_note() {
    // [BV-9] 外でノートを1つ直す → poll の changed にそれだけが入り、bytes が新しい
    let t = TempDir::new("bv9");
    let a = t.write("a.md", b"---\nstatus: todo\n---\n");
    let b = t.write("b.md", b"---\nstatus: todo\n---\n");
    let mut v = open_loaded(&[t.path().to_path_buf()]);

    let first = v.poll();
    assert!(first.changed.is_empty() && first.added.is_empty() && first.removed.is_empty());

    let new = b"---\nstatus: done and more\n---\n";
    std::fs::write(&a, new).unwrap();
    bump_mtime(&a, 10);

    let p = v.poll();
    assert_eq!(p.changed, vec![a.clone()]);
    assert!(p.added.is_empty());
    assert!(p.removed.is_empty());
    assert_eq!(v.note(&a).unwrap().bytes, new.to_vec());
    assert_eq!(v.note(&a).unwrap().stamp.len, new.len() as u64);
    assert_eq!(
        v.note(&b).unwrap().bytes,
        b"---\nstatus: todo\n---\n".to_vec()
    );

    // 変わっていなければ次の poll は空
    let again = v.poll();
    assert!(again.changed.is_empty());
}

#[test]
fn test_bv_9_paused_poll_does_nothing() {
    // [BV-9] 編集中(pause(true))は読み直さない。再開すると反映される
    let t = TempDir::new("bv9pause");
    let old = b"---\nk: 1\n---\n";
    let a = t.write("a.md", old);
    let mut v = open_loaded(&[t.path().to_path_buf()]);

    v.pause(true);
    let new = b"---\nk: 12345\n---\n";
    std::fs::write(&a, new).unwrap();
    bump_mtime(&a, 10);
    t.write("added.md", b"x\n");

    let p = v.poll();
    assert!(p.changed.is_empty());
    assert!(p.added.is_empty());
    assert!(p.removed.is_empty());
    assert_eq!(v.note(&a).unwrap().bytes, old.to_vec());

    v.pause(false);
    let p = v.poll();
    assert_eq!(p.changed, vec![a.clone()]);
    assert_eq!(v.note(&a).unwrap().bytes, new.to_vec());
}

#[test]
fn test_bv_9_poll_reports_added_and_removed() {
    // [BV-9] 追加・削除は added・removed
    let t = TempDir::new("bv9addrm");
    let a = t.write("a.md", b"a\n");
    let b = t.write("b.md", b"b\n");
    let mut v = open_loaded(&[t.path().to_path_buf()]);

    let c = t.write("sub/c.md", b"c\n");
    std::fs::remove_file(&b).unwrap();

    let p = v.poll();
    assert_eq!(p.added, vec![c.clone()]);
    assert_eq!(p.removed, vec![b.clone()]);
    assert!(p.changed.is_empty());
    assert_eq!(note_paths(&v), sorted(vec![a, c.clone()]));
    assert!(v.note(&b).is_none());
    assert_eq!(v.note(&c).unwrap().bytes, b"c\n".to_vec());
}

#[test]
fn test_bv_9_same_size_change_detected_by_mtime() {
    // [BV-9] 大きさが同じでも更新時刻が変われば読み直す
    let t = TempDir::new("bv9mtime");
    let a = t.write("a.md", b"---\nk: 1\n---\n");
    let mut v = open_loaded(&[t.path().to_path_buf()]);
    std::fs::write(&a, b"---\nk: 2\n---\n").unwrap();
    bump_mtime(&a, 20);
    let p = v.poll();
    assert_eq!(p.changed, vec![a.clone()]);
    assert_eq!(v.note(&a).unwrap().bytes, b"---\nk: 2\n---\n".to_vec());
}

// ---- BV-10 ----

fn pb(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

#[test]
fn test_bv_10_selection_stays_on_same_note() {
    // [BV-10] 同じ行が残ればその位置
    let old = pb(&["/v/a.md", "/v/b.md", "/v/c.md"]);
    assert_eq!(reselect(&old, 1, &old), Some(1));
    // 前に行が増えても、同じノートを選んだまま
    let new = pb(&["/v/0.md", "/v/a.md", "/v/b.md", "/v/c.md"]);
    assert_eq!(reselect(&old, 1, &new), Some(2));
}

#[test]
fn test_bv_10_removed_note_selects_next_row() {
    // [BV-10] 選んだ行のノートを消す → 同じ添字(下の行)が選ばれる
    let old = pb(&["/v/a.md", "/v/b.md", "/v/c.md"]);
    let new = pb(&["/v/a.md", "/v/c.md"]);
    assert_eq!(reselect(&old, 1, &new), Some(1));
}

#[test]
fn test_bv_10_removed_last_note_selects_last_row() {
    // [BV-10] 末尾を超えたら末尾
    let old = pb(&["/v/a.md", "/v/b.md", "/v/c.md"]);
    let new = pb(&["/v/a.md", "/v/b.md"]);
    assert_eq!(reselect(&old, 2, &new), Some(1));
}

#[test]
fn test_bv_10_empty_gives_none() {
    // [BV-10] 行が0なら None
    let old = pb(&["/v/a.md"]);
    assert_eq!(reselect(&old, 0, &[]), None);
}

// ---- BV-16 ----

#[test]
fn test_bv_16_load_is_incremental_and_cancellable() {
    // [BV-16] 読み込みを待たずに始まり、少しずつ増え、中止すると読んだ分が残る
    let t = TempDir::new("bv16");
    for i in 0..300 {
        t.write(
            &format!("n{:03}.md", i),
            format!("---\ni: {}\n---\n", i).as_bytes(),
        );
    }
    let mut v = Vault::open(roots(&[t.path().to_path_buf()]).unwrap());
    // open はまだ何も読まない
    assert!(v.notes().is_empty());

    let p = v.load(10);
    assert!(p.loaded <= 10, "loaded = {}", p.loaded);
    assert!(!p.done);
    let n1 = v.notes().len();
    assert!(n1 > 0 && n1 <= 10, "notes = {}", n1);

    let p2 = v.load(10);
    assert!(!p2.done);
    let n2 = v.notes().len();
    assert!(n2 > n1 && n2 <= 20, "notes = {} -> {}", n1, n2);

    v.cancel();
    v.load(10);
    assert_eq!(v.notes().len(), n2);
    v.load(1000);
    assert_eq!(v.notes().len(), n2);
}

#[test]
fn test_bv_16_load_finishes_with_all_notes() {
    // [BV-16] 最後まで読めば done=true で全ノート
    let t = TempDir::new("bv16all");
    for i in 0..300 {
        t.write(&format!("d{}/n{:03}.md", i % 7, i), b"x\n");
    }
    let mut v = Vault::open(roots(&[t.path().to_path_buf()]).unwrap());
    let mut last = 0;
    for step in 0.. {
        assert!(step < 10_000, "load が done にならない");
        let p = v.load(25);
        let n = v.notes().len();
        assert!(n >= last);
        last = n;
        if p.done {
            break;
        }
    }
    assert_eq!(v.notes().len(), 300);
}

// ---- WB-13 ----

#[test]
fn test_wb_13_sync_conflict_files_are_marked() {
    // [WB-13] 競合ファイルは conflict=true、普通のノートは false
    let t = TempDir::new("wb13");
    let normal = t.write("note.md", b"---\nk: 1\n---\n");
    let sync = t.write("note.sync-conflict-20260930-1.md", b"---\nk: 2\n---\n");
    let dropbox = t.write("a (conflicted copy).md", b"---\nk: 3\n---\n");
    let v = open_loaded(&[t.path().to_path_buf()]);

    assert_eq!(v.notes().len(), 3);
    assert!(!v.note(&normal).unwrap().conflict);
    assert!(v.note(&sync).unwrap().conflict);
    assert!(v.note(&dropbox).unwrap().conflict);
}
