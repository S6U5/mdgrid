#![cfg(unix)]
//! 変更 2026-10-03-e2e(Q-10)の受け入れ: 本物の実行ファイルを疑似端末(80×24)で動かし、
//! 設定の editor(SR-8)・add_frontmatter(WB-3)・`--config`(CLI-2・CLI-3)の配線を確かめる。
//! 実装(src/main.rs)を見ずに、要件と公開の振る舞い(USAGE・docs/config.md・docs/keys.md)だけで書いた。
//!
//! - 表で `e` = open_editor、Enter = edit、`j` = down、Ctrl+S = save。review で Enter = save_all(docs/keys.md)。
//! - 疑似端末の道具は tests/pty/mod.rs。子は環境を空にして、PATH・HOME・TERM・LANG・XDG_CONFIG_HOME・
//!   XDG_STATE_HOME(どれも一時フォルダ)と、試験ごとに決めた VISUAL・EDITOR だけを渡す。
//! - 子は Drop で kill と wait をする(落ちても止まる)。画面の待ちは 10 秒で落ちる。

mod pty;

use std::path::{Path, PathBuf};
use std::time::Duration;

use pty::{assert_gone, fake_editor, wait_file, FakeEditor, TempDir, Tui};

// ---------------------------------------------------------------- 場

/// 試験の場: HOME(一時フォルダ)の下に、保管庫・XDG の置き場・偽のエディタを置く。
struct Env {
    home: TempDir,
}

impl Env {
    fn new(tag: &str) -> Env {
        let home = TempDir::new(tag);
        for d in ["vault", "config", "state", "bin"] {
            std::fs::create_dir_all(home.path().join(d)).unwrap();
        }
        let v = home.path().join("vault");
        std::fs::write(v.join("alpha.md"), "---\nstatus: todo\n---\nbody\n").unwrap();
        std::fs::write(v.join("beta note.md"), "---\nstatus: done\n---\nbody\n").unwrap();
        // フロントマターの無いノート。
        std::fs::write(v.join("gamma.md"), "# plain\ntext\n").unwrap();
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

    /// 既定の置き場($XDG_CONFIG_HOME/mdgrid/config.toml)に設定を書く。
    fn write_default_config(&self, body: &str) {
        let d = self.home().join("config").join("mdgrid");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("config.toml"), body).unwrap();
    }

    /// 既定の置き場の外に設定を書き、そのパスを返す(`--config` に渡す)。
    fn write_config(&self, name: &str, body: &str) -> PathBuf {
        let p = self.home().join(name);
        std::fs::write(&p, body).unwrap();
        p
    }

    /// 偽のエディタを `<home>/bin` に作る。
    fn fake_editor(&self, name: &str, rewrite: Option<&str>) -> FakeEditor {
        fake_editor(&self.home().join("bin"), name, rewrite)
    }
}

/// 起動して表が出るまで待つ。
fn open(env: &Env, extra: &[&str], vars: &[(&str, &str)]) -> Tui {
    let vault = env.vault();
    let mut args: Vec<&str> = extra.to_vec();
    args.push(vault.to_str().unwrap());
    let t = Tui::spawn(&args, env.home(), vars);
    // SR-29: 左のノートの欄は `.md` を除いた名前。
    t.wait_for("alpha");
    t.wait_for("gamma");
    t
}

/// 偽のエディタの引数のパスが、そのノートを指す(絶対か、子の作業フォルダからの相対)。
fn assert_points_to(arg: &str, env: &Env, note: &str) {
    let p = Path::new(arg);
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        env.home().join(p)
    };
    assert_eq!(
        p.canonicalize().ok(),
        Some(env.note(note).canonicalize().unwrap()),
        "エディタに渡ったパス {arg:?} が {note} を指さない"
    );
}

/// 表の行(名前の順: alpha.md・beta note.md・gamma.md)へ下りる。
fn go_row(t: &mut Tui, index: usize) {
    t.send_settle(&"j".repeat(index));
}

/// エディタから戻り、表が描き直された(印が消えた)ことを待つ。
fn wait_back_to_table(t: &Tui) {
    t.wait_until("エディタから戻った表", |s| {
        s.contains("alpha") && s.contains("gamma") && !s.contains(FakeEditor::MARK)
    });
}

// ---------------------------------------------------------------- SR-8

#[test]
fn test_sr_8_e2e_config_editor_with_args_gets_note_path_as_one_arg() {
    // [SR-8] 設定の editor = "<偽のエディタ> --flag"。空白を含むノートの名前でも、パスは1つの引数。
    // 戻ったら画面が戻り、ノートを読み直す。
    let env = Env::new("sr8-args");
    let ed = env.fake_editor("fake-editor", Some("---\nstatus: edited\n---\nbody\n"));
    let cfg = env.write_config("c.toml", &format!("editor = \"{} --flag\"\n", ed.path()));
    let mut t = open(&env, &["--config", cfg.to_str().unwrap()], &[]);
    go_row(&mut t, 1); // beta note.md
    t.send("e");
    let args = ed.wait_args(&t);
    assert_eq!(args.len(), 2, "引数は --flag とノートのパスの2つ: {args:?}");
    assert_eq!(args[0], "--flag");
    assert!(args[1].ends_with("beta note.md"), "{args:?}");
    assert_points_to(&args[1], &env, "beta note.md");
    // エディタが画面を持ち、戻ると表が描き直される(印が消え、書き換えた値が出る)。
    t.wait_for(FakeEditor::MARK);
    wait_back_to_table(&t);
    t.wait_for("edited");
    assert_gone(t.stop());
}

#[test]
fn test_sr_8_e2e_visual_before_editor_without_config() {
    // [SR-8] 設定が無く VISUAL と EDITOR が両方ある → VISUAL。
    let env = Env::new("sr8-visual");
    let vis = env.fake_editor("visual-editor", None);
    let edi = env.fake_editor("editor-editor", None);
    let mut t = open(&env, &[], &[("VISUAL", vis.path()), ("EDITOR", edi.path())]);
    t.send("e"); // alpha.md
    let args = vis.wait_args(&t);
    assert_eq!(args.len(), 1, "{args:?}");
    assert_points_to(&args[0], &env, "alpha.md");
    wait_back_to_table(&t);
    assert!(!edi.was_called(), "EDITOR が呼ばれた(VISUAL が先のはず)");
    assert_gone(t.stop());
}

#[test]
fn test_sr_8_e2e_config_editor_before_visual_and_editor() {
    // [SR-8] 既定の置き場の設定に editor があれば、VISUAL・EDITOR より先。
    let env = Env::new("sr8-config");
    let cfg_ed = env.fake_editor("config-editor", None);
    let vis = env.fake_editor("visual-editor", None);
    let edi = env.fake_editor("editor-editor", None);
    env.write_default_config(&format!("editor = \"{}\"\n", cfg_ed.path()));
    let mut t = open(&env, &[], &[("VISUAL", vis.path()), ("EDITOR", edi.path())]);
    t.send("e");
    let args = cfg_ed.wait_args(&t);
    assert_eq!(args.len(), 1, "{args:?}");
    assert_points_to(&args[0], &env, "alpha.md");
    wait_back_to_table(&t);
    assert!(!vis.was_called(), "VISUAL が呼ばれた(設定が先のはず)");
    assert!(!edi.was_called(), "EDITOR が呼ばれた(設定が先のはず)");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- WB-3

/// フロントマターの無いノート(gamma.md)のセルで Enter を押し、読むだけの理由が出て入力が開かないこと。
fn assert_gamma_readonly(t: &mut Tui) {
    go_row(t, 2);
    t.send("\r");
    t.wait_for("読むだけ");
    std::thread::sleep(Duration::from_millis(300));
    let s = t.screen();
    assert!(
        !s.contains("Enter 確定"),
        "入力が開いた(読むだけのはず)。画面:\n{s}"
    );
}

/// フロントマターの無いノートのセルで Enter を押すと入力が開くこと。
fn assert_gamma_editable(t: &mut Tui) {
    go_row(t, 2);
    t.send("\r");
    t.wait_for("Enter 確定");
    assert!(!t.screen().contains("読むだけ"), "画面:\n{}", t.screen());
}

#[test]
fn test_wb_3_e2e_add_frontmatter_false_makes_cell_readonly_with_reason() {
    // [WB-3] add_frontmatter = false → フロントマターの無いノートのセルは読むだけで理由が出る。
    let env = Env::new("wb3-false");
    let cfg = env.write_config("c.toml", "[edit]\nadd_frontmatter = false\n");
    let mut t = open(&env, &["--config", cfg.to_str().unwrap()], &[]);
    assert_gamma_readonly(&mut t);
    assert_eq!(
        std::fs::read_to_string(env.note("gamma.md")).unwrap(),
        "# plain\ntext\n"
    );
    assert_gone(t.stop());
}

#[test]
fn test_wb_3_e2e_default_writes_and_adds_frontmatter_on_save() {
    // [WB-3] 既定(設定なし)ではフロントマターの無いノートのセルに書け、保存で先頭に足される。
    let env = Env::new("wb3-default");
    let mut t = open(&env, &[], &[]);
    go_row(&mut t, 2);
    t.send("\r");
    t.wait_for("Enter 確定");
    t.send_settle("wip");
    t.send_settle("\r"); // edit: commit
    t.wait_for("wip");
    t.send("\x13"); // table: save(Ctrl+S)→ 保存の確認
    t.wait_for("+status: wip");
    t.send("\r"); // review: save_all
    let body = wait_file(&env.note("gamma.md"), &t, |s| s.starts_with("---"));
    assert_eq!(body, "---\nstatus: wip\n---\n# plain\ntext\n");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- CLI-2・CLI-3

#[test]
fn test_cli_2_e2e_config_flag_wins_over_default_location_false() {
    // [CLI-2][CLI-3] 既定の置き場に add_frontmatter = true、--config に false → --config が効く。
    let env = Env::new("cli2-false");
    env.write_default_config("[edit]\nadd_frontmatter = true\n");
    let cfg = env.write_config("c.toml", "[edit]\nadd_frontmatter = false\n");
    let mut t = open(&env, &["--config", cfg.to_str().unwrap()], &[]);
    assert_gamma_readonly(&mut t);
    assert_gone(t.stop());
}

#[test]
fn test_cli_2_e2e_config_flag_wins_over_default_location_true() {
    // [CLI-2][CLI-3] 逆向き: 既定の置き場に false、--config に true → 書ける。
    let env = Env::new("cli2-true");
    env.write_default_config("[edit]\nadd_frontmatter = false\n");
    let cfg = env.write_config("c.toml", "[edit]\nadd_frontmatter = true\n");
    let mut t = open(&env, &["--config", cfg.to_str().unwrap()], &[]);
    assert_gamma_editable(&mut t);
    assert_gone(t.stop());
}

#[test]
fn test_cli_2_e2e_default_location_is_read_without_flag() {
    // [CLI-3] --config が無ければ既定の置き場($XDG_CONFIG_HOME/mdgrid/config.toml)を読む。
    // 上の2つが「置き場を読まない」ことで通ってしまわないための対照。
    let env = Env::new("cli3-default");
    env.write_default_config("[edit]\nadd_frontmatter = false\n");
    let mut t = open(&env, &[], &[]);
    assert_gamma_readonly(&mut t);
    assert_gone(t.stop());
}
