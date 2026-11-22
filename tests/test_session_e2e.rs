//! 名前を付けたセッション(specs/requirements/cli.md の「セッション」)を、本物の実行ファイルで確かめる。
//! `--sessions` と起動できないときは子の標準出力・標準エラーと終了コードで、画面は疑似端末(80×24)で見る。
//! 一時のホーム(HOME・XDG_CONFIG_HOME・XDG_STATE_HOME)の下に、ノートのフォルダ notes/tasks と設定の置き場を作る。
//! 期待値は要求の受け入れの例から取った。

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

/// notes/tasks に3つのノート(meeting memo・shopping・meeting prep)と、設定の置き場の sessions.toml(`sessions`)。
fn home(tag: &str, sessions: Option<&str>) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in ["config/mdgrid", "state", "notes/tasks"] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    let w = |rel: &str, text: &str| std::fs::write(p.join(rel), text).unwrap();
    w(
        "notes/tasks/a.md",
        "---\ntitle: meeting memo\nstatus: doing\ndue: 2026-10-05\npriority: 1\n---\n",
    );
    w(
        "notes/tasks/b.md",
        "---\ntitle: shopping\nstatus: done\ndue: 2026-09-01\npriority: 2\n---\n",
    );
    w(
        "notes/tasks/c.md",
        "---\ntitle: meeting prep\nstatus: doing\ndue: 2026-10-01\npriority: 3\n---\n",
    );
    if let Some(s) = sessions {
        w("config/mdgrid/sessions.toml", s);
    }
    h
}

/// 受け入れの例のセッション「進行中」: status が doing・「meeting」を含む・due の昇順・title・status・due の3列。
const SESSION: &str = "\
[sessions.\"進行中\"]
folders = [\"~/notes/tasks\"]
columns = [\"title\", \"status\", \"due\"]
hidden = [\"priority\"]

[[sessions.\"進行中\".filters]]
kind = \"value\"
key = \"status\"
value = \"doing\"

[[sessions.\"進行中\".filters]]
kind = \"text\"
value = \"meeting\"

[[sessions.\"進行中\".sort]]
key = \"due\"
dir = \"asc\"
";

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 理由1行と終了コード 2(cli.fail-fast と同じ)。
fn one_line_failure(o: &Output) -> String {
    assert_eq!(o.status.code(), Some(2), "{}", err(o));
    let e = err(o);
    assert_eq!(e.lines().count(), 1, "{e}");
    assert!(out(o).is_empty(), "{}", out(o));
    e
}

#[test]
fn test_session_list_on_the_command_line() {
    // [stest -> req~cli.session-screen-list~2] `--sessions` → `進行中<TAB>~/notes/tasks` の形の行と終了コード 0。
    // セッションが1つも無い → 何も出さず終了コード 0。
    let h = home("ses-list", Some(SESSION));
    let o = run(h.path(), &["--sessions"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "進行中\t~/notes/tasks\n");
    let h = home("ses-list-none", None);
    let o = run(h.path(), &["--sessions"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "");
}

#[test]
fn test_session_unknown_name_fails() {
    // [stest -> req~cli.session-errors~2] `mdgrid -s 無い` → 「セッション「無い」はありません(ある名前: 進行中)」の1行と終了コード 2。
    // [stest -> req~cli.session-save~2] 無い名前 → 理由1行と終了コード 2。
    let h = home("ses-unknown", Some(SESSION));
    let o = run(h.path(), &["-s", "無い"]);
    let e = one_line_failure(&o);
    assert_eq!(
        e,
        "mdgrid: セッション「無い」はありません(ある名前: 進行中)\n"
    );
    let o = run(h.path(), &["--session", "無い"]);
    one_line_failure(&o);
}

#[test]
fn test_session_with_a_path_fails() {
    // [stest -> req~cli.session-errors~2] `mdgrid -s 進行中 ~/notes` → どちらを開くか曖昧なので理由1行と終了コード 2。
    let h = home("ses-path", Some(SESSION));
    let notes = h.path().join("notes");
    let o = run(h.path(), &["-s", "進行中", &notes.to_string_lossy()]);
    let e = one_line_failure(&o);
    assert!(e.contains("-s"), "{e}");
}

#[test]
fn test_session_broken_file_and_missing_folder_fail() {
    // [stest -> req~cli.session-errors~2] 壊れた sessions.toml・セッションの開くフォルダが無い → 理由1行と終了コード 2。
    let h = home("ses-broken", Some("[sessions.\"進行中\"\nfolders = 1\n"));
    let e = one_line_failure(&run(h.path(), &["-s", "進行中"]));
    assert!(e.contains("sessions.toml が読めない(1 行目)"), "{e}");
    let e = one_line_failure(&run(h.path(), &["--sessions"]));
    assert!(e.contains("sessions.toml が読めない"), "{e}");
    let gone = SESSION.replace("~/notes/tasks", "~/notes/gone");
    let h = home("ses-gone", Some(&gone));
    let e = one_line_failure(&run(h.path(), &["-s", "進行中"]));
    assert_eq!(
        e,
        "mdgrid: セッション「進行中」の開くフォルダ ~/notes/gone が無い\n"
    );
}

#[test]
fn test_session_flags_in_help() {
    // [stest -> req~cli.session-save~2] 使い方に `-s <名前>`(`--session`)と `--sessions` がある(どの言語でも)。
    let h = home("ses-help", None);
    for lang in ["ja_JP.UTF-8", "en_US.UTF-8"] {
        let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
            .arg("--help")
            .current_dir(h.path())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", h.path())
            .env("LANG", lang)
            .output()
            .unwrap();
        assert!(o.status.success());
        let text = out(&o);
        assert!(text.contains("-s, --session"), "{lang}: {text}");
        assert!(text.contains("--sessions"), "{lang}: {text}");
    }
    // `--help` は、ほかの命令と同じくセッションより先(無い名前でも使い方を出す)。
    let o = run(h.path(), &["-s", "無い", "--help"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("--sessions"));
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, SESSION};
    use crate::pty::Tui;

    /// 起動の直後の1打は、端末の地の色の問い合わせに吸われることがあるので、効くまで押す。
    fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done(&t.screen()) && std::time::Instant::now() < end {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    #[test]
    fn test_session_opens_the_saved_view() {
        // [stest -> req~cli.session-save~2] `mdgrid -s 進行中` で同じフォルダ・同じ絞り込み・並び・列の表が開く。
        // [stest -> req~cli.session-content-fields~2] 同じ値の絞り込みと `\` の絞り込みの2つ・due の昇順・3列。
        // [stest -> req~cli.session-content-fresh-values~1] 保存したあとノートの値を直して開く → 直した値が出る。
        let h = home("ses-open", Some(SESSION));
        std::fs::write(
            h.path().join("notes/tasks/c.md"),
            "---\ntitle: meeting prep v2\nstatus: doing\ndue: 2026-10-01\npriority: 3\n---\n",
        )
        .unwrap();
        let t = Tui::spawn(&["-s", "進行中"], h.path(), &[]);
        t.wait_for("meeting prep v2");
        t.wait_for("meeting memo");
        let s = t.screen();
        assert!(!s.contains("shopping"), "{s}");
        let prep = s.find("meeting prep v2").unwrap();
        let memo = s.find("meeting memo").unwrap();
        assert!(prep < memo, "due の昇順: {s}");
        assert!(!s.contains("priority"), "隠した列: {s}");
        let head = s.lines().find(|l| l.contains("title")).unwrap_or_default();
        let (ti, st, du) = (
            head.find("title").unwrap(),
            head.find("status").unwrap(),
            head.find("due").unwrap(),
        );
        assert!(ti < st && st < du, "列の並び: {head}");
        drop(t);
    }

    #[test]
    fn test_session_unknown_item_warns_on_start() {
        // [stest -> req~cli.session-store~1] 手で `color = "red"` を足す → 起動して警告(開くのは止めない)。
        let text = SESSION.replace(
            "hidden = [\"priority\"]\n",
            "hidden = [\"priority\"]\ncolor = \"red\"\n",
        );
        let h = home("ses-warn", Some(&text));
        let t = Tui::spawn(&["-s", "進行中"], h.path(), &[]);
        t.wait_for("meeting memo");
        t.wait_for("sessions.toml: sessions.進行中.color");
        assert!(t.screen().contains("知らない項目"), "{}", t.screen());
        drop(t);
    }

    #[test]
    fn test_session_save_from_the_screen_then_open() {
        // [stest -> req~cli.session-screen-save~1] 画面で今の表示を名前 `進行中` で保存 → sessions.toml に `[sessions.進行中]`。
        // [stest -> req~cli.session-store~1] ノートのフォルダに新しいファイルが無い。
        // [stest -> req~cli.session-save~2] 保存して終了 → `mdgrid -s 進行中` で同じ絞り込みの表が開く。
        let h = home("ses-screen", None);
        let mut t = Tui::spawn(&["notes/tasks"], h.path(), &[]);
        t.wait_for("shopping");
        // `\` で検索の欄に入り(欄が「検索: 」になる)、meeting と打って決める。
        press_until(&mut t, "\\", |s| s.contains("検索: "));
        t.send("meeting\r");
        t.wait_until("絞った表", |s| !s.contains("shopping"));
        t.send(":");
        t.wait_for("パレット");
        t.send("save_session\r");
        t.wait_for("セッションの名前");
        t.send("進行中\r");
        t.wait_for("セッション「進行中」を保存した");
        t.send("q");
        let code = t.wait_exit();
        assert_eq!(code, 0, "{}", t.screen());
        drop(t);
        let file = h.path().join("config/mdgrid/sessions.toml");
        let text = std::fs::read_to_string(&file).unwrap();
        let table: toml::Table = text.parse().unwrap();
        assert!(
            table["sessions"].as_table().unwrap().contains_key("進行中"),
            "{text}"
        );
        let mut notes: Vec<String> = std::fs::read_dir(h.path().join("notes/tasks"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        notes.sort();
        assert_eq!(notes, ["a.md", "b.md", "c.md"]);
        let t = Tui::spawn(&["-s", "進行中"], h.path(), &[]);
        t.wait_for("meeting memo");
        t.wait_for("meeting prep");
        assert!(!t.screen().contains("shopping"), "{}", t.screen());
        drop(t);
    }
}
