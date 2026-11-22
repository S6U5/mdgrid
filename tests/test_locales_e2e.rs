//! 文言の台帳(screen.locales-all-in-ledger・screen.locales-fit・screen.locales-languages)を、本物の実行ファイルで
//! 確かめる。試しの言語(環境変数 `MDGRID_PSEUDO_LOCALE`。どの文言も頭に `⟦` が付く)では、台帳の外の文言が
//! 出ていれば `⟦` の無い文か、日本語の字として見つかる。見本のノートの値・列の名前・パスは英字だけにしてある。

use mdgrid::i18n::{Lang, Msg};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mdgrid-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(std::fs::canonicalize(&dir).unwrap_or(dir))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 英字だけの見本(ノートの名前・列の名前・値)と設定 `conf`。
fn home(tag: &str, conf: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "notes"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    std::fs::write(
        h.path().join("notes/alpha.md"),
        "---\nstatus: todo\nowner: kim\n---\n",
    )
    .unwrap();
    std::fs::write(
        h.path().join("notes/beta.md"),
        "---\nstatus: done\nowner: lee\n---\n",
    )
    .unwrap();
    std::fs::write(h.path().join("config/mdgrid/config.toml"), conf).unwrap();
    h
}

fn run(home: &Path, args: &[&str], pseudo: bool) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "en_US.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"));
    if pseudo {
        c.env("MDGRID_PSEUDO_LOCALE", "1");
    }
    c.output().unwrap()
}

/// 日本語・中国語・韓国語の字があるか(試しの言語は英語の文に `⟦` を付けたものなので、あれば台帳の外の文言)。
fn has_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c as u32, 0x3040..=0x30ff | 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xac00..=0xd7af | 0xff01..=0xff60)
    })
}

#[test]
fn test_locales_all_in_ledger_on_the_command_line() {
    // [stest -> req~screen.locales-all-in-ledger~1] 試しの言語では、--help・--print-config・設定の警告の文言がどれも台帳から出る。
    let h = home("ledger-cli", "unknown_item = 1\n");
    let help = String::from_utf8_lossy(&run(h.path(), &["--help"], true).stdout).into_owned();
    assert!(help.starts_with('⟦'), "{help}");
    assert!(!has_cjk(&help), "{help}");
    let pc = String::from_utf8_lossy(&run(h.path(), &["--print-config"], true).stdout).into_owned();
    assert!(pc.starts_with("# ⟦"), "{pc}");
    // 説明・範囲・例の見出しの行は、どれも台帳の文言(頭に ⟦)。
    let scope_lines: Vec<&str> = pc.lines().filter(|l| l.contains("Scope")).collect();
    assert!(!scope_lines.is_empty());
    assert!(
        scope_lines.iter().all(|l| l.starts_with("# ⟦")),
        "{scope_lines:?}"
    );
    assert!(!has_cjk(&pc), "{pc}");
    let o = run(h.path(), &["notes", "--print"], true);
    let err = String::from_utf8_lossy(&o.stderr).into_owned();
    assert!(err.contains("unknown_item") && err.contains('⟦'), "{err}");
    assert!(!has_cjk(&err), "{err}");
}

#[test]
fn test_locales_languages_have_ledgers() {
    // [stest -> req~screen.locales-languages~1] 9つの言語の台帳があり、LANG=ko_KR.UTF-8 の --help は韓国語。
    let codes: Vec<&str> = Lang::ALL.iter().map(|l| l.code()).collect();
    for c in [
        "en", "ja", "zh-CN", "zh-TW", "ko", "es", "fr", "de", "pt-BR",
    ] {
        assert!(codes.contains(&c), "{c} の台帳が無い: {codes:?}");
    }
    let h = home("ledger-ko", "");
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg("--help")
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", h.path())
        .env("LANG", "ko_KR.UTF-8")
        .env("XDG_CONFIG_HOME", h.path().join("config"))
        .output()
        .unwrap();
    let ko = Lang::ALL
        .iter()
        .copied()
        .find(|l| l.code() == "ko")
        .unwrap();
    let want = Msg::Usage.get(ko);
    assert_eq!(
        String::from_utf8_lossy(&o.stdout).trim_end(),
        want.trim_end()
    );
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{has_cjk, home};
    use crate::pty::Tui;
    use mdgrid::i18n::{Lang, Msg};

    /// 表が出るまで待ち、起動の直後に吸われうる1打を、末尾の列へ移る `$` で吸わせておく。
    fn ready(t: &mut Tui) {
        t.wait_for("alpha");
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < end {
            t.send("$");
            std::thread::sleep(std::time::Duration::from_millis(300));
            t.send("0");
            std::thread::sleep(std::time::Duration::from_millis(200));
            if t.screen().contains("alpha") {
                break;
            }
        }
    }

    #[test]
    fn test_locales_all_in_ledger_on_screen() {
        // [stest -> req~screen.locales-all-in-ledger~1] 試しの言語で、表の画面・ヘルプ・パレット・ビューの設定の画面に、台帳の外の文言(日本語の字)が無く、帯の文言には ⟦ が付く。
        let h = home("ledger-screen", "");
        let mut t = Tui::spawn(
            &["notes"],
            h.path(),
            &[("MDGRID_PSEUDO_LOCALE", "1"), ("LANG", "en_US.UTF-8")],
        );
        ready(&mut t);
        let s = t.screen();
        assert!(!has_cjk(&s), "{s}");
        let bottom = s
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or_default();
        assert!(bottom.contains('⟦'), "{s}");
        for (key, back) in [("?", "\u{1b}"), (":", "\u{1b}"), ("o", "\u{1b}")] {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(500));
            let s = t.screen();
            assert!(!has_cjk(&s), "{key}: {s}");
            assert!(s.contains('⟦'), "{key}: {s}");
            t.send(back);
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        drop(t);
    }

    #[test]
    fn test_locales_fit_every_language() {
        // [stest -> req~screen.locales-fit~1] どの言語でも、80桁の表の画面でヘッダーの右の端のボタンが重ならずに残り、ビューの設定の画面の「反映」と「取り消し」のボタンが出る(収まらなければ … で切る)。
        for &lang in Lang::ALL {
            let conf = format!("language = \"{}\"\n", lang.code());
            let h = home("ledger-fit", &conf);
            let mut t = Tui::spawn(&["notes"], h.path(), &[]);
            ready(&mut t);
            let s = t.screen();
            let header = s
                .lines()
                .find(|l| l.contains('▦'))
                .unwrap_or_default()
                .to_string();
            assert!(
                header.contains('▦') && header.contains('⚙'),
                "{}: {s}",
                lang.code()
            );
            t.send("o");
            std::thread::sleep(std::time::Duration::from_millis(600));
            let s = t.screen();
            let shown = |m: Msg| {
                let text = m.get(lang);
                let head: String = text.chars().take(2).collect();
                s.contains(text) || s.contains(&head)
            };
            assert!(
                shown(Msg::BtnApply) && shown(Msg::BtnCancel),
                "{}: {s}",
                lang.code()
            );
            drop(t);
        }
    }
}
