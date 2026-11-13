#![cfg(unix)]
//! [CE-15] 正規表現の置き換えを、本物の実行ファイルを疑似端末(80×24)で動かして確かめる。
//! 記録: specs/_changes/2026-11-13-regex-replace.md。実装より先に、実装を見ずに書いた。
//!
//! - 表で `:` = パレット、Ctrl+S = save。review で Enter = save_all(docs/keys.md)。
//! - 動作の名前は `regex_replace`、表示名は「正規表現で置き換える」。聞く前置きは「探す正規表現: 」
//!   「置き換える文字: 」(記録の「名前と文言」)。
//! - 疑似端末の道具は tests/pty/mod.rs(子は Drop で止まる。画面の待ちは 10 秒で落ちる)。

mod pty;

use std::path::PathBuf;
use std::time::Duration;

use pty::{assert_gone, wait_file, TempDir, Tui};

/// 作った値の3つのノート(列は title だけ)。
const NOTES: &[(&str, &str)] = &[
    ("alpha.md", "---\ntitle: v1 draft\n---\nbody\n"),
    ("beta.md", "---\ntitle: v1 final\n---\nbody\n"),
    ("gamma.md", "---\ntitle: notes\n---\nbody\n"),
];

struct Env {
    home: TempDir,
}

impl Env {
    fn new(tag: &str) -> Env {
        let home = TempDir::new(tag);
        for d in ["vault", "config", "state"] {
            std::fs::create_dir_all(home.path().join(d)).unwrap();
        }
        for (name, text) in NOTES {
            std::fs::write(home.path().join("vault").join(name), text).unwrap();
        }
        Env { home }
    }

    fn note(&self, name: &str) -> PathBuf {
        self.home.path().join("vault").join(name)
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.note(name)).unwrap()
    }

    fn assert_unchanged(&self) {
        for (name, text) in NOTES {
            assert_eq!(&self.read(name), text, "{name} が書き換わった");
        }
    }
}

/// 起動して表が出るまで待つ(選んだ列は、ただ1つの列 title)。
fn open(env: &Env) -> Tui {
    let vault = env.home.path().join("vault");
    let t = Tui::spawn(&[vault.to_str().unwrap()], env.home.path(), &[]);
    t.wait_for("v1 draft");
    t.wait_for("notes");
    t
}

/// パレットから `regex_replace` を選び、探す正規表現を聞かれるまで待つ。
fn start_replace(t: &mut Tui) {
    t.send(":");
    t.wait_until("パレット", |s| s.contains(": "));
    t.send("regex_replace");
    t.wait_until("パレットの候補", |s| {
        s.contains("正規表現で置き換える")
    });
    t.send("\r");
    t.wait_for("探す正規表現:");
}

#[test]
fn test_ce_15_e2e_palette_replace_then_save_writes_files() {
    // [CE-15] パレット → `v(\d)` → `version $1` → 表の2行が変わり「未保存 2」。ファイルはまだ変わらない。
    // Ctrl+S → 保存の確認に差分 → Enter で確定 → 2つのノートに書かれ、一致しないノートはそのまま。
    let env = Env::new("ce15-save");
    let mut t = open(&env);
    start_replace(&mut t);
    t.send(r"v(\d)");
    t.send("\r");
    t.wait_for("置き換える文字:");
    t.send("version $1");
    t.send("\r");
    t.wait_until("置き換えた表", |s| {
        s.contains("version 1 draft") && s.contains("version 1 final") && s.contains("未保存 2")
    });
    let s = t.screen();
    assert!(s.contains("2 件を置き換えた"), "結果の知らせが無い:\n{s}");
    assert!(s.contains("notes"), "一致しない行が消えた:\n{s}");
    // ためただけ。保存するまでファイルは変わらない。
    env.assert_unchanged();
    t.send("\x13"); // table: save(Ctrl+S)→ 保存の確認
    t.wait_for("+title: version 1 draft");
    env.assert_unchanged();
    t.send("\r"); // review: save_all
    let alpha = wait_file(&env.note("alpha.md"), &t, |s| s.contains("version"));
    assert_eq!(alpha, "---\ntitle: version 1 draft\n---\nbody\n");
    let beta = wait_file(&env.note("beta.md"), &t, |s| s.contains("version"));
    assert_eq!(beta, "---\ntitle: version 1 final\n---\nbody\n");
    assert_eq!(env.read("gamma.md"), NOTES[2].1);
    t.wait_for("未保存 0");
    assert_gone(t.stop());
}

#[test]
fn test_ce_15_e2e_bad_regex_changes_nothing() {
    // [CE-15] 探す文字に `(` → 「正規表現として読めない」と出て、表もファイルも変わらない。
    let env = Env::new("ce15-bad");
    let mut t = open(&env);
    start_replace(&mut t);
    t.send("(");
    t.send("\r");
    t.wait_for("正規表現として読めない");
    std::thread::sleep(Duration::from_millis(300));
    let s = t.screen();
    assert!(
        !s.contains("置き換える文字:"),
        "置き換える文字を聞いた:\n{s}"
    );
    assert!(!s.contains("未保存 1") && !s.contains("未保存 2"), "{s}");
    env.assert_unchanged();
    assert_gone(t.stop());
}
