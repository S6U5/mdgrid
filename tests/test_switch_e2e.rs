#![cfg(unix)]
//! お気に入りのワークスペースと切り替え(specs/requirements/workspace.md の「お気に入りのワークスペースと切り替え」)を、
//! 本物の実行ファイルで確かめる(疑似端末 80×24)。期待値は要求の受け入れの例から取った。
//! 見本: notes/ に tasks(状態の列があり、カンバンを開ける)・books・scratch。workspaces.toml に work(tasks)と
//! 趣味(books)。印のワークスペース lab(x)。

use crate::pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::time::{Duration, Instant};

/// 見本の作業のフォルダ。`favorites` は workspaces.toml の最上位に書く行(空なら書かない)。
fn home(tag: &str, favorites: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    let w = |rel: &str, text: &str| {
        let f = p.join(rel);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, text).unwrap();
    };
    w("notes/tasks/a.md", "---\nstatus: todo\n---\n");
    w("notes/tasks/b.md", "---\nstatus: done\n---\n");
    w("notes/books/dune.md", "---\nauthor: Herbert\n---\n");
    w("notes/scratch/memo.md", "---\nkind: scribble\n---\n");
    w("lab/x/one.md", "---\nkind: lab-note\n---\n");
    w(
        "lab/.mdgrid/workspace.toml",
        "[[table]]\nname = \"x\"\npath = \"x\"\n",
    );
    std::fs::create_dir_all(p.join("state")).unwrap();
    let fav = if favorites.is_empty() {
        String::new()
    } else {
        format!("{favorites}\n\n")
    };
    w(
        "config/mdgrid/workspaces.toml",
        &format!(
            "{fav}[[workspace]]\nname = \"work\"\n\n[[workspace.table]]\nname = \"tasks\"\npath = \"{}\"\n\n\
             [[workspace]]\nname = \"趣味\"\n\n[[workspace.table]]\nname = \"books\"\npath = \"{}\"\n",
            p.join("notes/tasks").display(),
            p.join("notes/books").display()
        ),
    );
    h
}

/// 画面が `done` になるまで `key` を押す(起動の直後の1打は、端末の地の色の問い合わせに吸われることがある)。
fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !done(&t.screen()) && Instant::now() < end {
        t.send(key);
        std::thread::sleep(Duration::from_millis(400));
    }
}

fn spawn(h: &Path, rel: &str, row: &str) -> Tui {
    let t = Tui::spawn(&[rel], h, &[]);
    t.wait_for(row);
    t
}

#[test]
fn test_switch_list_opens_on_the_kanban() {
    // [stest -> req~workspace.switch-open~2] お気に入りが work・趣味 → カンバンの画面で切り替えのキー → 「1 work」「2 趣味」の一覧。
    let h = home("switch-kanban", "favorites = [\"work\", \"趣味\"]");
    let mut t = spawn(h.path(), "notes/tasks", "todo");
    press_until(&mut t, "B", |s| s.contains("カンバン"));
    t.wait_for("カンバン");
    t.send("W");
    t.wait_until("切り替えの一覧", |s| {
        s.contains("1 work") && s.contains("2 趣味")
    });
    let s = t.screen();
    let at = |w: &str| s.find(w).unwrap_or_else(|| panic!("{w} が無い:\n{s}"));
    assert!(at("1 work") < at("2 趣味"), "{s}");
    assert!(s.contains("カンバン"), "カンバンの上に開く:\n{s}");
    t.send("\x1b");
    t.wait_until("閉じる", |s| !s.contains("2 趣味"));
    assert!(t.screen().contains("カンバン"), "{}", t.screen());
}

#[test]
fn test_switch_list_without_favorites_shows_how() {
    // [stest -> req~workspace.switch-open~2] お気に入りが無い → 一覧に、お気に入りの入れ方の案内の1行。
    let h = home("switch-none", "");
    let mut t = spawn(h.path(), "notes/tasks", "todo");
    press_until(&mut t, "W", |s| s.contains("お気に入りが無い"));
    assert!(t.screen().contains("お気に入りが無い"), "{}", t.screen());
}

#[test]
fn test_switch_back_to_where_you_started() {
    // [stest -> req~workspace.switch-origin~2] ワークスペースでない scratch を開き、切り替えで work へ移る → もう一度開くと先頭に
    // 「最初に開いた所: scratch」→ 選ぶ → scratch の表に戻る。scratch にいるあいだは「最初に開いた所」は無い。
    let h = home("switch-origin", "favorites = [\"work\", \"趣味\"]");
    let mut t = spawn(h.path(), "notes/scratch", "scribble");
    press_until(&mut t, "W", |s| s.contains("1 work"));
    assert!(!t.screen().contains("最初に開いた所"), "{}", t.screen());
    t.send("1");
    t.wait_until("work の表", |s| {
        s.contains("todo") && !s.contains("scribble")
    });
    t.send("W");
    t.wait_for("最初に開いた所: scratch");
    let s = t.screen();
    let at = |w: &str| s.find(w).unwrap_or_else(|| panic!("{w} が無い:\n{s}"));
    assert!(at("最初に開いた所: scratch") < at("1 work"), "{s}");
    t.send("\r");
    t.wait_until("scratch に戻る", |s| {
        s.contains("scribble") && !s.contains("todo")
    });
    assert!(t.screen().contains("scribble"), "{}", t.screen());
}

#[test]
fn test_favorite_marker_workspace_is_stored_by_path() {
    // [stest -> req~workspace.favorite-store~2] 「趣味」と印の lab をお気に入りにする → workspaces.toml に favorites = ["趣味", "~/lab"]、lab の印のバイトは同じ。
    let h = home("switch-store", "favorites = [\"趣味\"]");
    let p = h.path();
    let marker = p.join("lab/.mdgrid/workspace.toml");
    let before = std::fs::read(&marker).unwrap();
    let mut t = spawn(p, "lab/x", "lab-note");
    press_until(&mut t, ":", |s| s.lines().any(|l| l.starts_with(": ")));
    t.send("workspace_favorite");
    t.wait_for("> 今のワークスペースをお");
    t.send("\r");
    let file = p.join("config/mdgrid/workspaces.toml");
    let text = pty::wait_file(&file, &t, |s| s.contains("~/lab"));
    assert!(
        text.starts_with("favorites = [\"趣味\", \"~/lab\"]\n"),
        "{text}"
    );
    assert_eq!(std::fs::read(&marker).unwrap(), before, "印は書かない");
}

#[test]
fn test_switch_back_with_zero_from_a_marker_workspace() {
    // [stest -> req~workspace.switch-origin-key~1] 印のワークスペース lab を開き、切り替えで work へ移る → 一覧の先頭は
    // 「0 最初に開いた所: lab」→ `0` → lab の表に戻る。
    let h = home("switch-zero", "favorites = [\"work\"]");
    let mut t = spawn(h.path(), "lab/x", "lab-note");
    press_until(&mut t, "W", |s| s.contains("1 work"));
    t.send("1");
    t.wait_until("work の表", |s| {
        s.contains("todo") && !s.contains("lab-note")
    });
    t.send("W");
    t.wait_for("0 最初に開いた所: lab");
    t.send("0");
    t.wait_until("lab に戻る", |s| {
        s.contains("lab-note") && !s.contains("todo")
    });
    let s = t.screen();
    assert!(s.contains("lab-note") && !s.contains("todo"), "{s}");
}
