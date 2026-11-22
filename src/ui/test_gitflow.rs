//! 保存のあとの git(write-back.git-*)の画面の試験。一時フォルダに本物の git のリポを作り、キーで直して保存し、
//! git の履歴とインデックスで確かめる(名前とメールは試験のリポの中だけに決め、署名とフックは切る)。

use super::keymap::Mode;
use super::test_screen::{app_of, ch, ctrl, edit_cell, press, read, typing, Tmp};
use super::*;
use ratatui::crossterm::event::KeyCode;
use std::path::Path;
use std::process::Command;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\n---\n"),
    ("b.md", "---\nstatus: todo\n---\n"),
];

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

/// ノート a.md・b.md と other.txt を最初のコミットに入れたリポを作り、設定 `conf`(config.toml の中身)で開く。
/// `repo` が偽なら git のリポにしない。
fn boot(tag: &str, conf: &str, repo: bool) -> (Tmp, App) {
    let t = Tmp::new(tag);
    for (n, text) in NOTES {
        t.write(n, text);
    }
    std::fs::write(t.0.join("other.txt"), "o\n").unwrap();
    if repo {
        g(&t.0, &["init", "--quiet", "-b", "main"]);
        for (k, v) in [
            ("user.name", "Test"),
            ("user.email", "test@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            g(&t.0, &["config", k, v]);
        }
        g(&t.0, &["add", "."]);
        g(&t.0, &["commit", "--quiet", "-m", "first"]);
    }
    let mut a = app_of(&t, ColorMode::None);
    let (c, warns) = mdgrid::config::parse(conf).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    a.configure(&c);
    (t, a)
}

fn row(a: &App, name: &str) -> usize {
    a.rows.iter().position(|r| r.0.ends_with(name)).unwrap()
}

/// `name` の status を `value` に直して保存する(差分の確認を Enter で通す)。
fn save_status(a: &mut App, name: &str, value: &str) {
    let r = row(a, name);
    edit_cell(a, r, "status", value);
    ctrl(a, 's');
    press(a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0, "{:?}", a.message);
}

fn commits(dir: &Path) -> usize {
    g(dir, &["rev-list", "--count", "HEAD"])
        .trim()
        .parse()
        .unwrap()
}

fn last_files(dir: &Path) -> Vec<String> {
    g(dir, &["show", "--name-only", "--format=", "HEAD"])
        .lines()
        .map(String::from)
        .collect()
}

fn last_subject(dir: &Path) -> String {
    g(dir, &["log", "-1", "--format=%s"]).trim().to_string()
}

fn msg(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

#[test]
fn test_default_runs_no_git() {
    // [utest -> req~write-back.git-off-default~1] 設定なし(auto は既定の off)で保存 → コミットは増えず、書いたノートは変更中のまま、メッセージに git の知らせが無い。
    let (t, mut a) = boot("git-off", "", true);
    save_status(&mut a, "a.md", "done");
    assert_eq!(commits(&t.0), 1);
    assert_eq!(g(&t.0, &["status", "--porcelain"]), " M notes/a.md\n");
    assert!(!msg(&a).contains("git"), "{}", msg(&a));
}

#[test]
fn test_commit_on_every_save_holds_only_the_saved_note() {
    // [utest -> req~write-back.git-commit~2, req~write-back.git-when~2] auto = "commit"(when は既定の save)→ 保存1回で1つのコミット。中身は書いたノートだけで、変更中の other.txt は入らない。知らせが1行出る。
    let (t, mut a) = boot("git-commit", "[git]\nauto = \"commit\"\n", true);
    std::fs::write(t.0.join("other.txt"), "o2\n").unwrap();
    save_status(&mut a, "a.md", "done");
    assert_eq!(commits(&t.0), 2);
    assert_eq!(last_files(&t.0), ["notes/a.md"]);
    assert_eq!(g(&t.0, &["status", "--porcelain"]), " M other.txt\n");
    assert!(msg(&a).contains("git: 1 件をコミットした"), "{}", msg(&a));
    save_status(&mut a, "b.md", "doing");
    assert_eq!(commits(&t.0), 3);
    assert_eq!(last_files(&t.0), ["notes/b.md"]);
}

#[test]
fn test_auto_message_counts_the_notes() {
    // [utest -> req~write-back.git-message-auto~2] 自動の文の1行目は、1件なら「mdgrid: 1 件のノートを更新」、2件まとめてなら「mdgrid: 2 件のノートを更新」。
    let (t, mut a) = boot("git-msg", "[git]\nauto = \"commit\"\n", true);
    save_status(&mut a, "a.md", "done");
    assert_eq!(last_subject(&t.0), "mdgrid: 1 件のノートを更新");
    let (t, mut a) = boot(
        "git-msg2",
        "[git]\nauto = \"commit\"\nwhen = \"quit\"\n",
        true,
    );
    save_status(&mut a, "a.md", "done");
    save_status(&mut a, "b.md", "doing");
    assert_eq!(a.git_flush_at_quit(), None);
    assert_eq!(last_subject(&t.0), "mdgrid: 2 件のノートを更新");
}

#[test]
fn test_when_quit_commits_once_at_exit() {
    // [utest -> req~write-back.git-when~2] when = "quit" → 保存を2回しても、そのあいだコミットは増えない。終了のときに、2つのノートが1つのコミットになる。
    let (t, mut a) = boot(
        "git-quit",
        "[git]\nauto = \"commit\"\nwhen = \"quit\"\n",
        true,
    );
    save_status(&mut a, "a.md", "done");
    save_status(&mut a, "b.md", "doing");
    assert_eq!(commits(&t.0), 1);
    assert!(!msg(&a).contains("git"), "{}", msg(&a));
    assert_eq!(a.git_flush_at_quit(), None);
    assert_eq!(commits(&t.0), 2);
    assert_eq!(last_files(&t.0), ["notes/a.md", "notes/b.md"]);
}

#[test]
fn test_new_note_from_the_screen_is_committed() {
    // [utest -> req~write-back.git-when~2] 画面から新しいノート c を作る(when は save)→ c.md だけのコミットができる。
    let (t, mut a) = boot("git-new", "[git]\nauto = \"commit\"\n", true);
    ch(&mut a, 'a');
    typing(&mut a, "c");
    ctrl(&mut a, 's');
    assert!(t.notes().join("c.md").is_file(), "{:?}", a.message);
    assert_eq!(commits(&t.0), 2);
    assert_eq!(last_files(&t.0), ["notes/c.md"]);
}

#[test]
fn test_ask_takes_the_typed_first_line() {
    // [utest -> req~write-back.git-message-ask~2] message = "ask" → 保存のあとに1行の欄が開き、「見直し」と打って Enter → 1行目が「見直し」のコミット。
    let (t, mut a) = boot(
        "git-ask",
        "[git]\nauto = \"commit\"\nmessage = \"ask\"\n",
        true,
    );
    save_status(&mut a, "a.md", "done");
    assert_eq!(a.mode, Mode::Palette, "欄が開く: {:?}", a.message);
    assert_eq!(commits(&t.0), 1, "打つまでコミットしない");
    typing(&mut a, "見直し");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(last_subject(&t.0), "見直し");
    assert_eq!(last_files(&t.0), ["notes/a.md"]);
}

#[test]
fn test_ask_empty_uses_the_auto_message_and_esc_leaves_it_for_exit() {
    // [utest -> req~write-back.git-message-ask~2] 境界: 欄を空のまま Enter → 自動の文でコミット。Esc → コミットせず、終了のときに自動の文でコミットする。
    let (t, mut a) = boot(
        "git-ask2",
        "[git]\nauto = \"commit\"\nmessage = \"ask\"\n",
        true,
    );
    save_status(&mut a, "a.md", "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(last_subject(&t.0), "mdgrid: 1 件のノートを更新");
    save_status(&mut a, "b.md", "doing");
    assert_eq!(a.mode, Mode::Palette);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(commits(&t.0), 2, "Esc ではコミットしない");
    assert_eq!(a.git_flush_at_quit(), None);
    assert_eq!(commits(&t.0), 3);
    assert_eq!(last_files(&t.0), ["notes/b.md"]);
}

#[test]
fn test_outside_a_repo_nothing_happens_and_nothing_is_said() {
    // [utest -> req~write-back.git-not-repo~1] git のリポでないフォルダで auto = "push"・message = "ask" → 保存は済み、欄も開かず、メッセージに git の知らせが無い。
    let (t, mut a) = boot(
        "git-norepo",
        "[git]\nauto = \"push\"\nmessage = \"ask\"\n",
        false,
    );
    save_status(&mut a, "a.md", "done");
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(read(&t, "a.md"), "---\nstatus: done\n---\n");
    assert!(!msg(&a).contains("git"), "{}", msg(&a));
    assert_eq!(a.git_flush_at_quit(), None);
}

#[test]
fn test_git_failure_keeps_the_save_and_says_so_in_one_line() {
    // [utest -> req~write-back.git-failure~1] push 先の無いリポで auto = "push" → ノートは保存され(未保存 0)、メッセージに「git が失敗した(保存は済んでいる)」と git push の理由が1行で出る。
    let (t, mut a) = boot("git-fail", "[git]\nauto = \"push\"\n", true);
    save_status(&mut a, "a.md", "done");
    assert_eq!(read(&t, "a.md"), "---\nstatus: done\n---\n");
    let m = msg(&a);
    assert!(
        m.contains("git が失敗した(保存は済んでいる): git push: ") && !m.contains('\n'),
        "{m}"
    );
}

#[test]
fn test_git_section_is_read_from_the_config_only() {
    // [utest -> req~write-back.git-global-only~2] config.toml の [git] は読め、知らない値は警告1行で既定(off)のまま。台帳の印に書いた [git] は警告して使わない。
    let (c, warns) = mdgrid::config::parse("[git]\nauto = \"push\"\nwhen = \"quit\"\n").unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(c.git.auto, mdgrid::gitsync::Level::Push);
    assert_eq!(c.git.when, mdgrid::gitsync::When::Quit);
    let (c, warns) = mdgrid::config::parse("[git]\nauto = \"sometimes\"\n").unwrap();
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(warns[0].contains("git.auto"), "{warns:?}");
    assert_eq!(c.git.auto, mdgrid::gitsync::Level::Off);
    let t = Tmp::new("git-marker");
    std::fs::create_dir_all(t.0.join(".mdgrid")).unwrap();
    std::fs::write(
        t.0.join(".mdgrid/workspace.toml"),
        "[git]\nauto = \"push\"\n\n[[table]]\nname = \"T\"\npath = \"notes\"\n",
    )
    .unwrap();
    let (_, warns) = mdgrid::workspace::read_marker(&t.0);
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(warns[0].contains("git"), "{warns:?}");
}
