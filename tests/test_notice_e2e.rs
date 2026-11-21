//! 通知の欄(specs/requirements/notice.md)を、本物の実行ファイルで確かめる。`--notices` は子の標準出力で、
//! 画面は疑似端末(80×24)で見る。今日は `MDGRID_TODAY=2026-11-21`。期待値は要求の受け入れの例から取った。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TODAY: &str = "2026-11-21";

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

/// ワークスペース ws(印あり)の tasks に期限切れ2行・projects に1行、ホームに inbox.md、設定は `conf`。
fn home(tag: &str, conf: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in [
        "config/mdgrid",
        "state",
        "ws/tasks",
        "ws/projects",
        "ws/.mdgrid",
        "notes",
    ] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    let w = |rel: &str, text: &str| std::fs::write(p.join(rel), text).unwrap();
    w("ws/tasks/a.md", "---\ndue: 2026-11-10\nstatus: todo\n---\n");
    w(
        "ws/tasks/b.md",
        "---\ndue: 2026-11-11\nstatus: doing\n---\n",
    );
    w("ws/tasks/c.md", "---\ndue: 2026-12-11\nstatus: todo\n---\n");
    w(
        "ws/projects/p.md",
        "---\ndue: 2026-11-01\nstatus: todo\n---\n",
    );
    w("ws/.mdgrid/workspace.toml", "name = \"work\"\n");
    w(
        "notes/inbox.md",
        "---\ntitle: inbox\n---\n\n- 金曜に請求書\n- 鍵を返す\n- 歯医者\n",
    );
    w("config/mdgrid/config.toml", conf);
    h
}

const OVERDUE: &str = "[conditions.overdue]\nexpr = 'due < today() && status != \"done\"'\n";

fn conf(items: &str) -> String {
    format!("{OVERDUE}{items}")
}

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
        .env("MDGRID_TODAY", TODAY)
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn test_notice_cli_prints_items() {
    // [stest -> req~notice.cli~1] 件数と連絡を1項目1行で出して終了コード 0。項目が無ければ何も出さない。
    let items = "[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {count} 件\"\n\
                 [[notices]]\nnote = \"~/notes/inbox.md\"\n";
    let h = home("notice-cli", &conf(items));
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(out(&o), "期限切れ 2 件\n- 金曜に請求書\n");
    let h = home("notice-cli-none", OVERDUE);
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert!(o.status.success());
    assert_eq!(out(&o), "");
}

#[test]
fn test_notice_scope_on_the_command_line() {
    // [stest -> req~notice.scope~1] table は今の表、workspace は範囲の全部、名前の並びはその表だけ。
    let items = "[[notices]]\ncond = \"overdue\"\ntext = \"t{count}\"\n\
                 [[notices]]\ncond = \"overdue\"\ntext = \"w{count}\"\nscope = \"workspace\"\n\
                 [[notices]]\ncond = \"overdue\"\ntext = \"p{count}\"\nscope = [\"projects\"]\n";
    let h = home("notice-scope", &conf(items));
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert_eq!(out(&o), "t2\nw3\np1\n");
}

#[test]
fn test_notice_note_item_on_the_command_line() {
    // [stest -> req~notice.note-item~1] 本文の空でない行を頭から lines 行。無いノートは知らせの1行で、止めない。
    let items = "[[notices]]\nnote = \"~/notes/inbox.md\"\nlines = 2\n[[notices]]\nnote = \"~/notes/none.md\"\n";
    let h = home("notice-note", &conf(items));
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert!(o.status.success());
    assert_eq!(
        out(&o),
        "- 金曜に請求書\n- 鍵を返す\nnone.md がありません\n"
    );
}

#[test]
fn test_notice_cli_plain_and_unreadable() {
    // [stest -> req~notice.cli-plain~1] 読めない表は「?」を添え、エスケープの文字を出さない。
    let items =
        "[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {count} 件\"\nscope = \"workspace\"\n";
    let h = home("notice-plain", &conf(items));
    std::fs::write(
        h.path().join("ws/.mdgrid/workspace.toml"),
        "name = \"work\"\n[[table]]\nname = \"tasks\"\npath = \"tasks\"\n[[table]]\nname = \"gone\"\npath = \"gone\"\n",
    )
    .unwrap();
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert_eq!(out(&o), "期限切れ 2? 件\n");
    assert!(!o.stdout.contains(&0x1b));
}

#[test]
fn test_notice_cli_plain_strips_control_characters() {
    // [stest -> req~notice.cli-plain~1] ノートの中の端末の制御の文字(ESC)は、そのまま出さない。
    let items = "[[notices]]\nnote = \"~/notes/inbox.md\"\n";
    let h = home("notice-esc", &conf(items));
    std::fs::write(
        h.path().join("notes/inbox.md"),
        "\u{1b}[31mred\u{1b}]0;title\u{7}\n",
    )
    .unwrap();
    let o = run(h.path(), &["ws/tasks", "--notices"]);
    assert!(o.status.success());
    assert!(!o.stdout.contains(&0x1b), "{:?}", out(&o));
    assert!(out(&o).contains("red"), "{:?}", out(&o));
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{conf, home, TODAY};
    use crate::pty::Tui;

    #[test]
    fn test_notice_opt_in_on_screen() {
        // [stest -> req~notice.opt-in~1] 項目を書くとヘッダーの上に欄が出る。
        let items = "[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {count} 件\"\n";
        let h = home("notice-screen", &conf(items));
        let t = Tui::spawn(&["ws/tasks"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        t.wait_until("欄", |s| {
            s.lines()
                .next()
                .unwrap_or_default()
                .contains("期限切れ 2 件")
        });
        let first = t.screen().lines().next().unwrap_or_default().to_string();
        assert!(first.contains("期限切れ 2 件"), "{first}");
        drop(t);
    }

    #[test]
    fn test_notice_layout_on_screen() {
        // [stest -> req~notice.layout~1] 件数の項目を1行目に · でつなぎ、連絡は次の行。
        let items = "[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {count} 件\"\n\
                     [[notices]]\ncond = \"overdue\"\ntext = \"今日まで {count} 件\"\nscope = \"workspace\"\n\
                     [[notices]]\nnote = \"~/notes/inbox.md\"\n";
        let h = home("notice-layout", &conf(items));
        let t = Tui::spawn(&["ws/tasks"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        t.wait_until("欄の2行", |s| {
            let mut l = s.lines();
            l.next()
                .unwrap_or_default()
                .contains("期限切れ 2 件 · 今日まで 3 件")
                && l.next().unwrap_or_default().contains("- 金曜に請求書")
        });
        let s = t.screen();
        let lines: Vec<&str> = s.lines().take(2).collect();
        assert!(lines[0].contains("期限切れ 2 件 · 今日まで 3 件"), "{s}");
        assert!(lines[1].contains("- 金曜に請求書"), "{s}");
        drop(t);
    }

    #[test]
    fn test_notice_jump_on_screen() {
        // [stest -> req~notice.jump~1] 欄へ移って Enter → 今の表が期限切れの2行になり、帯に条件。
        let items = "[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {count} 件\"\n";
        let h = home("notice-jump", &conf(items));
        let mut t = Tui::spawn(&["ws/tasks"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        t.wait_for("2026-12-11");
        // 起動の直後の1打は、端末の地の色の問い合わせ(テーマの auto)に吸われることがあるので、移るまで押す。
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !t.screen().contains(">期限切れ 2 件") && std::time::Instant::now() < end {
            t.send("m");
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
        t.wait_for(">期限切れ 2 件");
        t.send("\r");
        t.wait_until("絞る", |s| {
            s.contains("overdue") && !s.contains("2026-12-11")
        });
        let s = t.screen();
        assert!(s.contains("2026-11-10") && s.contains("2026-11-11"), "{s}");
        assert!(!s.contains("2026-12-11"), "{s}");
        drop(t);
    }
}
