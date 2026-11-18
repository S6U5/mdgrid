//! 保存のあとの git(write-back.git-*)の単体の試験。一時フォルダに本物の git のリポを作って確かめる
//! (名前とメールは試験のリポの中だけに決め、署名とフックは切る)。

use super::*;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "mdgrid-git-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p.canonicalize().unwrap())
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 試験の git(成功を確かめて標準出力を返す)。
fn g(dir: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// リポを作り、a.md・b.md・other.txt を最初のコミットに入れる。
fn repo(tag: &str) -> Tmp {
    let t = Tmp::new(tag);
    g(&t.0, &["init", "--quiet", "-b", "main"]);
    for (k, v) in [
        ("user.name", "Test"),
        ("user.email", "test@example.com"),
        ("commit.gpgsign", "false"),
        ("core.hooksPath", "/dev/null"),
    ] {
        g(&t.0, &["config", k, v]);
    }
    for (n, text) in [("a.md", "a\n"), ("b.md", "b\n"), ("other.txt", "o\n")] {
        std::fs::write(t.0.join(n), text).unwrap();
    }
    g(&t.0, &["add", "."]);
    g(&t.0, &["commit", "--quiet", "-m", "first"]);
    t
}

fn head(dir: &Path) -> String {
    g(dir, &["rev-parse", "HEAD"]).trim().to_string()
}

fn status(dir: &Path) -> Vec<String> {
    g(dir, &["status", "--porcelain"])
        .lines()
        .map(String::from)
        .collect()
}

#[test]
fn test_off_touches_nothing() {
    // [utest -> req~write-back.git-off-default~1] auto = "off"(既定)→ 書いたノートがあっても、HEAD もインデックスも前と同じ。
    let t = repo("off");
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    let before = head(&t.0);
    let done = run(Level::Off, &[t.0.join("a.md")], "x").unwrap();
    assert_eq!(done, Done::default());
    assert_eq!(head(&t.0), before);
    assert_eq!(status(&t.0), [" M a.md"]);
    assert_eq!(Git::default().auto, Level::Off);
}

#[test]
fn test_add_stages_only_the_written_notes() {
    // [utest -> req~write-back.git-add~1] auto = "add" → 書いた a.md だけがステージされ、変更中の other.txt はそのまま。コミットは増えない。
    let t = repo("add");
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    std::fs::write(t.0.join("other.txt"), "o2\n").unwrap();
    let before = head(&t.0);
    let done = run(Level::Add, &[t.0.join("a.md")], "x").unwrap();
    assert_eq!((done.added, done.commits, done.pushed), (1, 0, 0));
    assert_eq!(head(&t.0), before);
    assert_eq!(status(&t.0), ["M  a.md", " M other.txt"]);
}

#[test]
fn test_commit_holds_only_the_written_notes() {
    // [utest -> req~write-back.git-commit~1] auto = "commit" → 書いた a.md と新しい c.md だけの1つのコミット。変更中の other.txt と、先にステージしてあった b.md は入らず、そのまま残る。
    let t = repo("commit");
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    std::fs::write(t.0.join("c.md"), "c\n").unwrap();
    std::fs::write(t.0.join("other.txt"), "o2\n").unwrap();
    std::fs::write(t.0.join("b.md"), "b2\n").unwrap();
    g(&t.0, &["add", "b.md"]);
    let done = run(
        Level::Commit,
        &[t.0.join("a.md"), t.0.join("c.md")],
        "mdgrid: update 2 notes",
    )
    .unwrap();
    assert_eq!((done.added, done.commits, done.pushed), (2, 1, 0));
    let files: Vec<String> = g(&t.0, &["show", "--name-only", "--format=", "HEAD"])
        .lines()
        .map(String::from)
        .collect();
    assert_eq!(files, ["a.md", "c.md"]);
    assert_eq!(status(&t.0), ["M  b.md", " M other.txt"]);
    assert_eq!(g(&t.0, &["rev-list", "--count", "HEAD"]).trim(), "2");
}

#[test]
fn test_commit_without_a_difference_makes_no_commit() {
    // [utest -> req~write-back.git-commit~1] 境界: 書いたノートが HEAD と同じ中身 → コミットは作らない(空のコミットを作らない)。
    let t = repo("nodiff");
    let before = head(&t.0);
    let done = run(Level::Commit, &[t.0.join("a.md")], "x").unwrap();
    assert_eq!(done.commits, 0);
    assert_eq!(head(&t.0), before);
}

#[test]
fn test_auto_message_has_the_first_line_and_the_paths() {
    // [utest -> req~write-back.git-message-auto~1] 自動の文は、1行目、空行、リポの根からのパスを1行ずつ。
    let t = repo("msg");
    std::fs::create_dir_all(t.0.join("Tasks")).unwrap();
    std::fs::write(t.0.join("Tasks/x.md"), "x\n").unwrap();
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    run(
        Level::Commit,
        &[t.0.join("a.md"), t.0.join("Tasks/x.md")],
        "mdgrid: update 2 notes",
    )
    .unwrap();
    assert_eq!(
        g(&t.0, &["log", "-1", "--format=%B"]).trim_end(),
        "mdgrid: update 2 notes\n\na.md\nTasks/x.md"
    );
}

#[test]
fn test_push_sends_the_commit_to_the_remote() {
    // [utest -> req~write-back.git-push~1] auto = "push" → コミットのあと push し、リモートの main が手元の HEAD と同じになる。
    let remote = Tmp::new("remote");
    g(&remote.0, &["init", "--quiet", "--bare", "-b", "main"]);
    let t = repo("push");
    g(
        &t.0,
        &["remote", "add", "origin", remote.0.to_str().unwrap()],
    );
    g(&t.0, &["push", "--quiet", "-u", "origin", "main"]);
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    let done = run(Level::Push, &[t.0.join("a.md")], "x").unwrap();
    assert_eq!((done.commits, done.pushed), (1, 1));
    assert_eq!(g(&remote.0, &["rev-parse", "main"]).trim(), head(&t.0));
}

#[test]
fn test_files_outside_a_repo_are_skipped_silently() {
    // [utest -> req~write-back.git-not-repo~1] git のリポでないフォルダのノート → 何もせず、失敗にもしない。
    let t = Tmp::new("norepo");
    std::fs::write(t.0.join("a.md"), "a\n").unwrap();
    // 一時フォルダの上に .git が無いことが前提。
    assert_eq!(repo_root(&t.0.join("a.md")), None);
    let done = run(Level::Push, &[t.0.join("a.md")], "x").unwrap();
    assert_eq!(done, Done::default());
    assert!(!t.0.join(".git").exists());
}

#[test]
fn test_failure_returns_one_line_and_keeps_the_note() {
    // [utest -> req~write-back.git-failure~1] push 先の無いリポで auto = "push" → 理由1行(改行なし。git push と言う)が返り、ノートのバイトは書いたまま。コミットは残る。
    let t = repo("fail");
    std::fs::write(t.0.join("a.md"), "a2\n").unwrap();
    let e = run(Level::Push, &[t.0.join("a.md")], "x").unwrap_err();
    assert!(e.starts_with("git push: ") && !e.contains('\n'), "{e:?}");
    assert_eq!(std::fs::read_to_string(t.0.join("a.md")).unwrap(), "a2\n");
    assert_eq!(g(&t.0, &["rev-list", "--count", "HEAD"]).trim(), "2");
}
