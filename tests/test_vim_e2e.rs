//! vim モード(設定の `[keys]` の `vim`)を、本物の実行ファイルで確かめる。設定の読み取りは `--print-config --resolved`
//! で(どの OS でも)、入力の欄の Esc は疑似端末の画面で(unix)。
//! 期待値は specs/requirements/screen.md の screen.vim-setting・screen.vim-mode-shown の受け入れの例から。

use std::path::PathBuf;
use std::process::Command;

/// 一時フォルダ(設定と状態の置き場)。Drop で消す。
struct Tmp(PathBuf);

impl Tmp {
    fn new(tag: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-vim-e2e-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join("config/mdgrid")).unwrap();
        std::fs::create_dir_all(dir.join("state")).unwrap();
        Tmp(dir)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 設定(`config` があれば config.toml に書く)で `mdgrid --print-config --resolved` を走らせ、(標準出力, 標準エラー)。
fn resolved(tag: &str, config: Option<&str>) -> (String, String) {
    let t = Tmp::new(tag);
    if let Some(c) = config {
        std::fs::write(t.0.join("config/mdgrid/config.toml"), c).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(["--print-config", "--resolved"])
        .env("XDG_CONFIG_HOME", t.0.join("config"))
        .env("XDG_STATE_HOME", t.0.join("state"))
        .output()
        .expect("mdgrid を起動できる");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn test_vim_setting_is_false_by_default_and_read_from_keys() {
    // [stest -> req~screen.vim-setting~1] 設定が無い → vim モードは false(既定)。`[keys]` に `vim = true` → true として読み、警告を出さない
    // (`[keys]` の下のモードの表と一緒に書ける)。
    let (out, err) = resolved("default", None);
    assert!(out.contains("[keys]\n# default\nvim = false"), "{out}");
    assert!(err.trim().is_empty(), "{err}");
    let (out, err) = resolved(
        "on",
        Some("[keys]\nvim = true\n\n[keys.table]\n\"ctrl+f\" = \"search\"\n"),
    );
    assert!(out.contains("[keys]\n# config.toml\nvim = true"), "{out}");
    assert!(err.trim().is_empty(), "警告なし: {err}");
}

/// 入力の欄の Esc を疑似端末で(unix)。
#[cfg(unix)]
mod screen {
    use crate::pty;

    use pty::{TempDir, Tui};

    /// 作業のフォルダ: notes に2つのノート。`config` があれば設定のファイルに書く。
    fn home(tag: &str, config: Option<&str>) -> TempDir {
        let h = TempDir::new(tag);
        for d in ["config/mdgrid", "state", "notes"] {
            std::fs::create_dir_all(h.path().join(d)).unwrap();
        }
        std::fs::write(
            h.path().join("notes/a.md"),
            "---\ntitle: 会議の見積もり\n---\n",
        )
        .unwrap();
        std::fs::write(h.path().join("notes/b.md"), "---\ntitle: 買い物\n---\n").unwrap();
        if let Some(c) = config {
            std::fs::write(h.path().join("config/mdgrid/config.toml"), c).unwrap();
        }
        h
    }

    fn open(h: &TempDir) -> Tui {
        let notes = h.path().join("notes");
        let t = Tui::spawn(&[notes.to_str().unwrap()], h.path(), &[]);
        t.wait_for("会議の見積もり");
        t.wait_for("買い物");
        t
    }

    /// 下の帯(未保存の数で始まる行)。
    fn footer(screen: &str) -> String {
        screen
            .lines()
            .find(|l| l.trim_start().starts_with("未保存"))
            .unwrap_or_default()
            .to_string()
    }

    /// 下の帯のモードの名前(未保存の数の次の語)。
    fn mode(screen: &str) -> String {
        footer(screen)
            .split_whitespace()
            .nth(2)
            .unwrap_or_default()
            .to_string()
    }

    /// 下の帯のモードの名前が `want` になるまで待つ。
    fn wait_mode(t: &Tui, want: &str) {
        t.wait_until(&format!("下の帯のモード {want}"), |s| {
            mode(s) == want
        });
    }

    #[test]
    fn test_vim_setting_off_esc_cancels_the_cell_input() {
        // [stest -> req~screen.vim-setting~1] 設定が無い → セルの入力の欄で Esc を押すと、今どおり入力を取りやめて閉じる。
        let h = home("vim-off", None);
        let mut t = open(&h);
        t.send_settle("\r");
        wait_mode(&t, "編集");
        assert!(
            !footer(&t.screen()).contains("-- 挿入 --"),
            "{}",
            t.screen()
        );
        t.send_settle("\x1b");
        wait_mode(&t, "表");
        assert_eq!(mode(&t.screen()), "表", "取りやめて閉じる");
        let pid = t.stop();
        pty::assert_gone(pid);
    }

    #[test]
    fn test_vim_setting_on_esc_goes_to_normal_and_keeps_the_input() {
        // [stest -> req~screen.vim-setting~1, req~screen.vim-mode-shown~1] `vim = true` → 同じ Esc で入力の欄がノーマルのモードになり、
        // 閉じない。下の帯は「-- 挿入 --」から「-- ノーマル --」に変わり、もう一度 Esc で取りやめて閉じる。
        let h = home("vim-on", Some("[keys]\nvim = true\n"));
        let mut t = open(&h);
        t.send_settle("\r");
        wait_mode(&t, "編集");
        t.wait_until("挿入のモード", |s| footer(s).contains("-- 挿入 --"));
        t.send_settle("\x1b");
        t.wait_until("ノーマルのモード", |s| {
            footer(s).contains("-- ノーマル --")
        });
        assert_eq!(mode(&t.screen()), "編集", "閉じない");
        // ノーマルのモードの `0`・`dw` で入力の欄の語を消す(画面の入力の欄に出る)。
        t.send_settle("0dw");
        t.wait_for("の見積もり");
        t.send_settle("\x1b");
        wait_mode(&t, "表");
        assert!(t.screen().contains("会議の見積もり"), "値は前のまま");
        let pid = t.stop();
        pty::assert_gone(pid);
    }
}
