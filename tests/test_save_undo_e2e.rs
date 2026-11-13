#![cfg(unix)]
//! 保存の取り消し(WB-21)の e2e: 本物の実行ファイルを疑似端末(80×24)で動かす。
//! 仕様: specs/write-back/spec.md の WB-21。記録: specs/_changes/2026-11-13-save-undo.md。
//! 実装を見ずに、要件の例と、記録の設計にある文言・docs/keys.md のキーだけで書いた。
//!
//! - 表で Enter = edit、Ctrl+U = clear_input、Ctrl+S = save、`:` = palette、`u` = undo。
//!   保存の確認で Enter = save_all、Esc = back。
//! - 疑似端末の道具は tests/pty/mod.rs。値はどれも作ったもの。

mod pty;

use std::path::{Path, PathBuf};

use pty::{assert_gone, wait_file, TempDir, Tui};

const ALPHA: &str = "---\nstatus:   todo\ntitle: \"Alpha\"\n---\nbody\n";
const ALPHA_DONE: &str = "---\nstatus:   done\ntitle: \"Alpha\"\n---\nbody\n";
const BETA: &str = "---\nstatus: later\ntitle: Beta\n---\nbody\n";

struct Env {
    home: TempDir,
}

impl Env {
    fn new(tag: &str) -> Env {
        let home = TempDir::new(tag);
        for d in ["vault", "config", "state"] {
            std::fs::create_dir_all(home.path().join(d)).unwrap();
        }
        let v = home.path().join("vault");
        std::fs::write(v.join("alpha.md"), ALPHA).unwrap();
        std::fs::write(v.join("beta.md"), BETA).unwrap();
        Env { home }
    }

    fn home(&self) -> &Path {
        self.home.path()
    }

    fn vault(&self) -> PathBuf {
        self.home().join("vault")
    }

    fn note(&self, name: &str) -> PathBuf {
        self.vault().join(name)
    }

    /// ノートのフォルダの中の名前(並べたもの)。
    fn vault_names(&self) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(self.vault())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }
}

/// 起動して表が出るまで待つ。
fn open(env: &Env, vars: &[(&str, &str)]) -> Tui {
    let vault = env.vault();
    let t = Tui::spawn(&[vault.to_str().unwrap()], env.home(), vars);
    t.wait_for("alpha");
    t.wait_for("beta");
    t
}

/// 先頭の行(alpha)の status を done に直して保存する(差分を見て確定)。
fn edit_status_and_save(t: &mut Tui, env: &Env) {
    t.send("\r"); // table: edit(先頭の行・先頭のキーの列 status)
    t.wait_for("Enter 確定");
    t.send_settle("\x15"); // edit: clear_input(Ctrl+U)
    t.send_settle("done");
    t.send_settle("\r"); // edit: commit
    t.wait_for("未保存 1");
    t.send("\x13"); // table: save(Ctrl+S)→ 保存の確認
    t.wait_for("+status:   done");
    t.send("\r"); // review: save_all
    let body = wait_file(&env.note("alpha.md"), t, |s| s.contains("done"));
    assert_eq!(body, ALPHA_DONE);
    t.wait_for("未保存 0");
}

/// パレットを開き、語を打って、その表示名の候補が出たら実行する。
fn run_from_palette(t: &mut Tui, label: &str) {
    t.send(":");
    t.wait_until("パレット", |s| s.contains(": "));
    t.send(label);
    t.wait_until("パレットの候補", |s| {
        s.lines().any(|l| l.starts_with('>') && l.contains(label))
    });
    t.send("\r");
}

#[test]
fn test_wb_21_e2e_palette_undo_save_shows_diff_and_restores_bytes() {
    // [WB-21] status: todo を done に直して保存 → ファイルは done。パレットの「保存を取り消す」→
    // 今の中身から保存の前への差分(-done・+todo)。確定 → ファイルは保存の前と1バイトも違わず、
    // status のセルは done のまま「未保存 1」。`u` → 「未保存 0」。ノートのフォルダに控えを作らない。
    let env = Env::new("wb21-undo");
    let mut t = open(&env, &[]);
    edit_status_and_save(&mut t, &env);

    run_from_palette(&mut t, "保存を取り消す");
    t.wait_for("保存の取り消し: 1 ファイル");
    t.wait_until("取り消しの差分", |s| {
        s.contains("-status:   done") && s.contains("+status:   todo")
    });
    // 確定までは書かない。
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA_DONE.as_bytes()
    );

    t.send("\r"); // review: save_all(取り消しの確定)
    wait_file(&env.note("alpha.md"), &t, |s| s.contains("todo"));
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA.as_bytes(),
        "保存の前のバイトに戻っていない"
    );
    assert_eq!(std::fs::read(env.note("beta.md")).unwrap(), BETA.as_bytes());
    // 書いた値はためる変更に戻る: 表に戻り、セルは done のまま、未保存 1。
    t.wait_until("表に戻って未保存 1", |s| {
        s.contains("未保存 1") && s.contains("done") && !s.contains("保存の取り消し:")
    });
    assert_eq!(env.vault_names(), ["alpha.md", "beta.md"]);

    // [WB-10] `u` でそのためた変更が消える。ファイルは戻したまま。
    t.send("u");
    t.wait_for("未保存 0");
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA.as_bytes()
    );
    assert_gone(t.stop());
}

#[test]
fn test_wb_21_e2e_cancel_undo_review_writes_nothing() {
    // [WB-21] 取り消しの確認を Esc でやめる → 何も書かず表に戻る。
    let env = Env::new("wb21-esc");
    let mut t = open(&env, &[]);
    edit_status_and_save(&mut t, &env);

    run_from_palette(&mut t, "保存を取り消す");
    t.wait_for("保存の取り消し: 1 ファイル");
    t.send("\x1b"); // review: back
    t.wait_until("表に戻る", |s| {
        !s.contains("保存の取り消し:") && s.contains("未保存 0")
    });
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA_DONE.as_bytes()
    );
    assert_gone(t.stop());
}

#[test]
fn test_wb_21_e2e_no_save_yet_says_nothing_to_undo() {
    // [WB-21] まだ保存していないとき → 何も書かず、取り消す保存が無いと知らせる(確認の画面は出ない)。
    let env = Env::new("wb21-none");
    let mut t = open(&env, &[]);
    run_from_palette(&mut t, "保存を取り消す");
    t.wait_for("取り消す保存は無い");
    let s = t.screen();
    assert!(
        !s.contains("保存の取り消し:"),
        "確認の画面が出た。画面:\n{s}"
    );
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA.as_bytes()
    );
    assert_eq!(std::fs::read(env.note("beta.md")).unwrap(), BETA.as_bytes());
    assert_eq!(env.vault_names(), ["alpha.md", "beta.md"]);
    assert_gone(t.stop());
}

#[test]
fn test_wb_21_e2e_english_command_and_message() {
    // [WB-21] 英語の画面: パレットの「Undo the last save」→ 「there is no save to undo」。
    let env = Env::new("wb21-en");
    let mut t = open(&env, &[("LANG", "en_US.UTF-8")]);
    run_from_palette(&mut t, "Undo the last save");
    t.wait_for("there is no save to undo");
    assert_eq!(
        std::fs::read(env.note("alpha.md")).unwrap(),
        ALPHA.as_bytes()
    );
    assert_gone(t.stop());
}

#[test]
fn test_wb_21_e2e_readonly_has_no_undo_save_command() {
    // [WB-21] [WB-15] 読むだけの起動(--readonly)では、パレットに「保存を取り消す」を出さない。
    let env = Env::new("wb21-ro");
    let vault = env.vault();
    let mut t = Tui::spawn(&["--readonly", vault.to_str().unwrap()], env.home(), &[]);
    t.wait_for("alpha");
    t.send(":");
    t.wait_until("パレット", |s| s.contains(": "));
    t.send("undo_save");
    // 候補が落ち着くのを待ってから見る。
    std::thread::sleep(std::time::Duration::from_millis(500));
    let s = t.screen();
    assert!(
        !s.contains("保存を取り消す"),
        "読むだけなのにコマンドが出た。画面:\n{s}"
    );
    assert_gone(t.stop());
}
