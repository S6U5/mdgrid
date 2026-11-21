#![cfg(unix)]
//! 開き直し(specs/requirements/resume.md)と、画面の `--view` で mdgrid のビューを探すこと(cli.view-own-views)を、
//! 本物の実行ファイルで確かめる(疑似端末 80×24)。Herdr の道具は、受け取った引数をログに書く偽の道具にする。

use crate::pty;

use pty::{TempDir, Tui};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const BASE: &str = "views:\n  - type: table\n    name: 進行中\n  - type: table\n    name: 全部\n";

/// 作業のフォルダ: notes に2つのノートと tasks.base(ビュー「進行中」「全部」)、空のフォルダ other。
fn home(tag: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "notes", "other"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    std::fs::write(h.path().join("notes/a.md"), "---\nstatus: doing\n---\n").unwrap();
    std::fs::write(h.path().join("notes/b.md"), "---\nstatus: done\n---\n").unwrap();
    std::fs::write(h.path().join("notes/tasks.base"), BASE).unwrap();
    h
}

/// 実体のパス(一時フォルダはシンボリックリンクの下にあることがある)。
fn real(p: &Path) -> String {
    std::fs::canonicalize(p)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// 偽の Herdr の道具。呼ばれるたびに引数を `|` でつないだ1行をログに足す。`before` は書く前に走らせる文。
struct FakeHerdr {
    script: PathBuf,
    log: PathBuf,
}

fn fake_herdr(dir: &Path, before: &str) -> FakeHerdr {
    let script = dir.join("herdr");
    let log = dir.join("herdr.log");
    let s = format!(
        "#!/bin/sh\n{before}\nout=''\nfor a in \"$@\"; do out=\"$out$a|\"; done\nprintf '%s\\n' \"$out\" >> '{}'\n",
        log.display()
    );
    std::fs::write(&script, s).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    FakeHerdr { script, log }
}

impl FakeHerdr {
    fn env(&self) -> Vec<(&'static str, String)> {
        vec![
            ("HERDR_ENV", "1".to_string()),
            ("HERDR_PANE_ID", "w1:p1".to_string()),
            ("HERDR_BIN_PATH", self.script.to_string_lossy().into_owned()),
        ]
    }

    /// これまでの呼び出し(1回ごとの引数)。
    fn calls(&self) -> Vec<Vec<String>> {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(|l| {
                l.trim_end_matches('|')
                    .split('|')
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    /// 条件に合う呼び出しが来るまで待つ(上限 10 秒)。
    fn wait(&self, tui: &Tui, cond: impl Fn(&[String]) -> bool) -> Vec<String> {
        let start = Instant::now();
        loop {
            if let Some(c) = self.calls().into_iter().find(|c| cond(c)) {
                return c;
            }
            assert!(
                start.elapsed() <= Duration::from_secs(10),
                "偽の Herdr の呼び出しが来ない: {:?}\n画面:\n{}",
                self.calls(),
                tui.screen()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn spawn(args: &[&str], h: &Path, env: &[(&'static str, String)]) -> Tui {
    let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (*k, v.as_str())).collect();
    Tui::spawn(args, h, &env)
}

/// `report-agent` の `--` の後ろ(開き直す命令)。
fn command_of(call: &[String]) -> Vec<String> {
    let at = call.iter().position(|a| a == "--").expect("-- が無い");
    call[at + 1..].to_vec()
}

fn is_report(c: &[String]) -> bool {
    c.get(1).map(String::as_str) == Some("report-agent")
}

fn is_release(c: &[String]) -> bool {
    c.get(1).map(String::as_str) == Some("release-agent")
}

fn seq_of(c: &[String]) -> u64 {
    let at = c.iter().position(|a| a == "--seq").unwrap();
    c[at + 1].parse().unwrap()
}

fn is_metadata(c: &[String]) -> bool {
    c.get(1).map(String::as_str) == Some("report-metadata")
}

/// `report-metadata` の `--title` の値(題を消す呼び出しなら None)。
fn title_of(c: &[String]) -> Option<&str> {
    let at = c.iter().position(|a| a == "--title")?;
    c.get(at + 1).map(String::as_str)
}

fn is_clear_title(c: &[String]) -> bool {
    is_metadata(c) && c.iter().any(|a| a == "--clear-title")
}

#[test]
fn test_herdr_report_follow_and_release() {
    // [stest -> req~resume.herdr-report~1, req~resume.herdr-follow~1, req~resume.herdr-release~1] Herdr の窓で `.base` のビュー「進行中」を開く → report-agent で開き直す命令を伝え、ビューを「全部」へ移ると 2 秒以内に伝え直し、行を動かしても伝え直さず、q で release-agent を呼んでから終わる。
    let h = home("resume-report");
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(
        &["notes/tasks.base", "--view", "進行中"],
        h.path(),
        &herdr.env(),
    );
    t.wait_for("doing");
    let first = herdr.wait(&t, is_report);
    let base = real(&h.path().join("notes/tasks.base"));
    let mut want: Vec<String> = [
        "pane",
        "report-agent",
        "w1:p1",
        "--source",
        "mdgrid",
        "--agent",
        "mdgrid",
        "--state",
        "idle",
        "--seq",
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect();
    want.push(seq_of(&first).to_string());
    want.extend(["--", "mdgrid", &base, "--view", "進行中"].map(String::from));
    assert_eq!(first, want);

    let start = Instant::now();
    t.send("]");
    let next = herdr.wait(&t, |c| {
        is_report(c) && command_of(c).contains(&"全部".to_string())
    });
    assert!(
        start.elapsed() <= Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(command_of(&next), vec!["mdgrid", &base, "--view", "全部"]);

    let before = herdr.calls().len();
    t.send_settle("j");
    t.send_settle("k");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(herdr.calls().len(), before, "{:?}", herdr.calls());

    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let calls = herdr.calls();
    let last = calls.last().unwrap();
    assert_eq!(
        last[..last.len() - 1],
        [
            "pane",
            "release-agent",
            "w1:p1",
            "--source",
            "mdgrid",
            "--agent",
            "mdgrid",
            "--seq"
        ]
    );
    // [stest -> req~resume.herdr-seq~1] 伝えた数は、この順に大きくなる。
    let seqs: Vec<u64> = calls.iter().map(|c| seq_of(c)).collect();
    assert!(seqs.windows(2).all(|w| w[0] < w[1]), "{seqs:?}");
}

#[test]
fn test_herdr_follow_switch_from_places_list() {
    // [stest -> req~resume.herdr-follow~1, req~resume.reopen-args~1] 登録した表の一覧で別の表(フォルダ)へ移る → その表の絶対パスと --raw の命令を伝える。
    let h = home("resume-places");
    std::fs::create_dir_all(h.path().join("shelf")).unwrap();
    std::fs::write(
        h.path().join("shelf/novel.md"),
        "---\nauthor: someone\n---\n",
    )
    .unwrap();
    std::fs::write(
        h.path().join("config/mdgrid/places.toml"),
        "[[place]]\nname = \"本棚\"\ngroup = \"趣味\"\npath = \"~/shelf\"\n",
    )
    .unwrap();
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&[], h.path(), &herdr.env());
    t.wait_for("趣味 › 本棚");
    t.send("本棚");
    t.send("\r");
    t.wait_until("開き直した表", |s| s.contains("novel"));
    let call = herdr.wait(&t, |c| {
        is_report(c) && command_of(c).iter().any(|a| a.ends_with("shelf"))
    });
    let cmd = command_of(&call);
    assert_eq!(cmd.len(), 3, "{cmd:?}");
    assert_eq!(cmd[0], "mdgrid");
    assert_eq!(real(Path::new(&cmd[1])), real(&h.path().join("shelf")));
    assert!(Path::new(&cmd[1]).is_absolute());
    assert_eq!(cmd[2], "--raw");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

#[test]
fn test_herdr_not_called_outside_herdr() {
    // [stest -> req~resume.herdr-report~1] HERDR_ENV の無い窓(窓の ID と道具のパスはある)→ Herdr の道具は呼ばれない。
    let h = home("resume-noenv");
    let herdr = fake_herdr(h.path(), "");
    let env: Vec<(&'static str, String)> = herdr.env().into_iter().skip(1).collect();
    let mut t = spawn(&["notes/tasks.base"], h.path(), &env);
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    assert!(!herdr.log.exists(), "{:?}", herdr.calls());
}

#[test]
fn test_herdr_unsendable_folder_releases() {
    // [stest -> req~resume.herdr-unsendable~1] フォルダ `it's` を開く → report-agent は呼ばれず、release-agent が呼ばれる。
    let h = home("resume-quote");
    std::fs::create_dir_all(h.path().join("it's")).unwrap();
    std::fs::write(h.path().join("it's/c.md"), "---\nstatus: quoted\n---\n").unwrap();
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["it's"], h.path(), &herdr.env());
    t.wait_for("quoted");
    herdr.wait(&t, is_release);
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let calls = herdr.calls();
    assert!(!calls.iter().any(|c| is_report(c)), "{calls:?}");
    assert!(calls.iter().all(|c| is_release(c)), "{calls:?}");
}

#[test]
fn test_herdr_slow_tool_does_not_block() {
    // [stest -> req~resume.herdr-no-wait~1] 呼ばれると 30 秒眠る道具 → 表が出てキーが効き、q から 5 秒以内に終わる。
    let h = home("resume-slow");
    let herdr = fake_herdr(h.path(), "exec sleep 30");
    let mut t = spawn(&["notes/tasks.base"], h.path(), &herdr.env());
    t.wait_for("doing");
    // コマンドのパレットを開いて打てる(キーが効いている)。
    t.send_settle(":");
    t.send_settle("ノートを消す");
    t.wait_for("ノートを消す");
    t.send_settle("\x1b");
    let start = Instant::now();
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    assert!(
        start.elapsed() <= Duration::from_secs(5),
        "{:?}",
        start.elapsed()
    );
}

#[test]
fn test_herdr_failing_tool_is_quiet() {
    // [stest -> req~resume.herdr-no-wait~1] 終了コード 1 で終わる道具 → 知らせの行に何も出さず、ビューを移っても伝え直して動き続ける。
    let h = home("resume-fail");
    let herdr = fake_herdr(h.path(), "");
    // ログに書いてから 1 で終わる道具にする。
    let s = std::fs::read_to_string(&herdr.script).unwrap() + "exit 1\n";
    std::fs::write(&herdr.script, s).unwrap();
    let mut t = spawn(&["notes/tasks.base"], h.path(), &herdr.env());
    t.wait_for("doing");
    herdr.wait(&t, is_report);
    t.send("]");
    herdr.wait(&t, |c| {
        is_report(c) && command_of(c).contains(&"全部".to_string())
    });
    std::thread::sleep(Duration::from_millis(300));
    let screen = t.screen();
    assert!(!screen.to_lowercase().contains("herdr"), "{screen}");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

#[test]
fn test_herdr_config_off() {
    // [stest -> req~resume.herdr-config-off~1] `[terminal] herdr = false` → Herdr の窓でも道具は1回も呼ばれない。書かなければ呼ばれる(既定は true)。
    let h = home("resume-off");
    std::fs::write(
        h.path().join("config/mdgrid/config.toml"),
        "[terminal]\nherdr = false\n",
    )
    .unwrap();
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["notes/tasks.base"], h.path(), &herdr.env());
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    assert!(!herdr.log.exists(), "{:?}", herdr.calls());
    // 書かない → 呼ばれる。
    std::fs::remove_file(h.path().join("config/mdgrid/config.toml")).unwrap();
    let mut t = spawn(&["notes/tasks.base"], h.path(), &herdr.env());
    t.wait_for("doing");
    herdr.wait(&t, is_report);
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

#[test]
fn test_herdr_skipped_for_pick() {
    // [stest -> req~resume.herdr-skip-pick~1] Herdr の窓で `--pick path` を開いて Esc で取りやめる → 道具は呼ばれない。
    let h = home("resume-pick");
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["notes", "--pick", "path"], h.path(), &herdr.env());
    t.wait_for("doing");
    t.send("\x1b");
    assert_eq!(t.wait_exit(), 1);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!herdr.log.exists(), "{:?}", herdr.calls());
}

#[test]
fn test_herdr_title_follows_view_and_clears() {
    // [stest -> req~resume.herdr-title~1, req~resume.herdr-title-clear~1] Herdr の窓で tasks.base をビュー「進行中」で開く → report-metadata で題「tasks · 進行中」を伝え、「全部」へ移ると「tasks · 全部」に替わる。q で終える → --clear-title のあとに release-agent。`herdr = false` → 題も伝えない。
    let h = home("resume-title");
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(
        &["notes/tasks.base", "--view", "進行中"],
        h.path(),
        &herdr.env(),
    );
    t.wait_for("doing");
    let first = herdr.wait(&t, |c| is_metadata(c) && title_of(c).is_some());
    let mut want: Vec<String> = [
        "pane",
        "report-metadata",
        "w1:p1",
        "--source",
        "mdgrid",
        "--agent",
        "mdgrid",
        "--title",
        "tasks · 進行中",
        "--seq",
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect();
    want.push(seq_of(&first).to_string());
    assert_eq!(first, want);

    t.send("]");
    herdr.wait(&t, |c| title_of(c) == Some("tasks · 全部"));

    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let calls = herdr.calls();
    let clear = calls
        .iter()
        .position(|c| is_clear_title(c))
        .unwrap_or_else(|| panic!("--clear-title が無い: {calls:?}"));
    let release = calls
        .iter()
        .position(|c| is_release(c))
        .unwrap_or_else(|| panic!("release-agent が無い: {calls:?}"));
    assert!(clear < release, "{calls:?}");
    assert_eq!(release, calls.len() - 1, "{calls:?}");

    // `[terminal] herdr = false` → 題も伝えない(道具を1回も呼ばない)。
    std::fs::write(
        h.path().join("config/mdgrid/config.toml"),
        "[terminal]\nherdr = false\n",
    )
    .unwrap();
    std::fs::remove_file(&herdr.log).unwrap();
    let mut t = spawn(
        &["notes/tasks.base", "--view", "進行中"],
        h.path(),
        &herdr.env(),
    );
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    assert!(!herdr.log.exists(), "{:?}", herdr.calls());
}

#[test]
fn test_herdr_title_form() {
    // [stest -> req~resume.herdr-title-form~1] ワークスペースの表 `name = "タスク"` の notes/tasks.base を「進行中」で開く → 題「タスク · 進行中」。ビューの無いフォルダ notes → 「notes」。ビューが1つの `.base`(ビューのタブを出さない表)→ 表の名前だけ。
    let h = home("resume-title-form");
    let base = real(&h.path().join("notes/tasks.base"));
    std::fs::write(
        h.path().join("config/mdgrid/workspaces.toml"),
        format!("[[workspace]]\nname = \"仕事\"\n\n[[workspace.table]]\nname = \"タスク\"\npath = \"{base}\"\n"),
    )
    .unwrap();
    std::fs::write(
        h.path().join("notes/one.base"),
        "views:\n  - type: table\n    name: ひとつ\n",
    )
    .unwrap();
    let herdr = fake_herdr(h.path(), "");
    for (args, want) in [
        (
            &["notes/tasks.base", "--view", "進行中"][..],
            "タスク · 進行中",
        ),
        (&["notes"][..], "notes"),
        (&["notes/one.base"][..], "one"),
    ] {
        let _ = std::fs::remove_file(&herdr.log);
        let mut t = spawn(args, h.path(), &herdr.env());
        t.wait_for("doing");
        let call = herdr.wait(&t, |c| is_metadata(c) && title_of(c).is_some());
        assert_eq!(title_of(&call), Some(want), "{args:?}");
        t.send("q");
        assert_eq!(t.wait_exit(), 0);
    }
}

/// 最後の画面のファイル(state/mdgrid/last/*.toml)の中身を全部つなげたもの。
fn last_screens(h: &Path) -> String {
    let dir = h.join("state/mdgrid/last");
    let mut out = String::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        if e.path().extension().is_some_and(|x| x == "toml") {
            out += &std::fs::read_to_string(e.path()).unwrap();
        }
    }
    out
}

#[test]
fn test_last_screen_and_continue() {
    // [stest -> req~resume.last-screen-save~1, req~resume.continue~1] tasks.base のビュー「全部」に移って終える → 最後の画面に tasks.base と「全部」が書かれ、notes に新しいファイルは無い。同じフォルダで --continue → 「全部」で開く。別のフォルダの --continue には使わない。
    let h = home("resume-continue");
    let mut t = Tui::spawn(&["notes/tasks.base"], h.path(), &[]);
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let saved = last_screens(h.path());
    assert!(
        saved.contains("tasks.base") && saved.contains("全部"),
        "{saved}"
    );
    let mut names: Vec<String> = std::fs::read_dir(h.path().join("notes"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["a.md", "b.md", "tasks.base"]);

    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["--continue"], h.path(), &herdr.env());
    t.wait_for("doing");
    let call = herdr.wait(&t, is_report);
    let base = real(&h.path().join("notes/tasks.base"));
    assert_eq!(command_of(&call), vec!["mdgrid", &base, "--view", "全部"]);
    t.send("q");
    assert_eq!(t.wait_exit(), 0);

    // 別のフォルダ other で --continue → notes の最後の画面は使わず、other を引数なしと同じに開く。
    std::fs::remove_file(&herdr.log).unwrap();
    let owned = herdr.env();
    let mut env: Vec<(&str, &str)> = owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let mdgrid = env!("CARGO_BIN_EXE_mdgrid");
    env.push(("MDGRID", mdgrid));
    let mut t = Tui::spawn_program(
        "/bin/sh",
        &["-c", "cd other && exec \"$MDGRID\" --continue"],
        h.path(),
        &env,
    );
    let call = herdr.wait(&t, is_report);
    assert_eq!(
        command_of(&call),
        vec![
            "mdgrid".to_string(),
            real(&h.path().join("other")),
            "--raw".to_string()
        ]
    );
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

#[test]
fn test_continue_fallback() {
    // [stest -> req~resume.continue-fallback~1] 最後の画面が無い → 引数なしと同じ。壊れている → 同じ。覚えたビューを .base から消す → 表を開いて、ビューが無い理由を知らせの行に出す。
    let h = home("resume-fallback");
    let herdr = fake_herdr(h.path(), "");
    let here = real(h.path());
    let no_args = vec!["mdgrid".to_string(), here.clone(), "--raw".to_string()];

    let mut t = spawn(&["--continue"], h.path(), &herdr.env());
    let call = herdr.wait(&t, is_report);
    assert_eq!(command_of(&call), no_args);
    t.send("q");
    assert_eq!(t.wait_exit(), 0);

    // 覚えさせてから壊す。
    let mut t = Tui::spawn(&["notes/tasks.base"], h.path(), &[]);
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let dir = h.path().join("state/mdgrid/last");
    let file = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "toml"))
        .unwrap();
    let good = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, "paths = [[[\n").unwrap();
    std::fs::remove_file(&herdr.log).unwrap();
    let mut t = spawn(&["--continue"], h.path(), &herdr.env());
    let call = herdr.wait(&t, is_report);
    assert_eq!(command_of(&call), no_args);
    t.send("q");
    assert_eq!(t.wait_exit(), 0);

    // 覚えたビュー「全部」を .base から消す → tasks.base が開き、理由が出る。
    std::fs::write(&file, good).unwrap();
    std::fs::write(
        h.path().join("notes/tasks.base"),
        "views:\n  - type: table\n    name: 進行中\n",
    )
    .unwrap();
    let mut t = Tui::spawn(&["--continue"], h.path(), &[]);
    t.wait_for("doing");
    t.wait_for("ビュー 全部 が無いので先頭のビューを出した");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

fn run(h: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(h)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", h)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", h.join("config"))
        .env("XDG_STATE_HOME", h.join("state"))
        .output()
        .unwrap()
}

#[test]
fn test_continue_conflicts() {
    // [stest -> req~resume.continue-conflict~1] --continue とパス・--view・-w・--raw・--print・--pick を一緒に渡す → 理由1行と終了コード 2。
    let h = home("resume-conflict");
    for args in [
        &["--continue", "notes"][..],
        &["--continue", "--view", "全部"],
        &["--continue", "-w", "x"],
        &["--continue", "--raw"],
        &["--continue", "--print"],
        &["--continue", "--pick", "path"],
    ] {
        let o = run(h.path(), args);
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(err.trim_end().lines().count(), 1, "{args:?}: {err}");
        assert!(o.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn test_continue_with_readonly() {
    // [stest -> req~resume.continue-conflict~1] `mdgrid --continue --readonly` → 最後の画面を読むだけで開く。
    let h = home("resume-continue-ro");
    let mut t = Tui::spawn(&["notes/tasks.base"], h.path(), &[]);
    t.wait_for("doing");
    t.send_settle("]");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["--continue", "--readonly"], h.path(), &herdr.env());
    t.wait_for("doing");
    let call = herdr.wait(&t, is_report);
    let base = real(&h.path().join("notes/tasks.base"));
    assert_eq!(
        command_of(&call),
        vec!["mdgrid", &base, "--view", "全部", "--readonly"]
    );
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

/// views.toml に、フォルダ notes2 の mdgrid のビュー「進行中」を書く。
fn own_view_home(tag: &str) -> TempDir {
    let h = home(tag);
    std::fs::create_dir_all(h.path().join("notes2")).unwrap();
    std::fs::write(h.path().join("notes2/x.md"), "---\nstatus: own\n---\n").unwrap();
    let p = real(&h.path().join("notes2"));
    std::fs::write(
        h.path().join("config/mdgrid/views.toml"),
        format!("[[table]]\npath = \"{p}\"\n\n  [[table.view]]\n  name = \"進行中\"\n  order = []\n  hidden = []\n  filters_expr = []\n"),
    )
    .unwrap();
    h
}

#[test]
fn test_view_option_finds_own_views() {
    // [stest -> req~cli.view-own-views~1] views.toml にビュー「進行中」のあるフォルダで `mdgrid notes2 --view 進行中` → 「進行中」のタブで開く。
    let h = own_view_home("resume-own-view");
    let herdr = fake_herdr(h.path(), "");
    let mut t = spawn(&["notes2", "--view", "進行中"], h.path(), &herdr.env());
    t.wait_for("own");
    let call = herdr.wait(&t, is_report);
    let cmd = command_of(&call);
    assert_eq!(cmd[cmd.len() - 2..], ["--view", "進行中"], "{cmd:?}");
    t.send("q");
    assert_eq!(t.wait_exit(), 0);
}

#[test]
fn test_view_option_unknown_name_fails() {
    // [stest -> req~cli.view-own-views~1] `mdgrid notes2 --view 無い` → 「進行中」を並べた理由1行と終了コード 2。
    let h = own_view_home("resume-own-view-missing");
    let o = run(h.path(), &["notes2", "--view", "無い"]);
    assert_eq!(o.status.code(), Some(2));
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(err.trim_end().lines().count(), 1, "{err}");
    assert!(err.contains("無い") && err.contains("進行中"), "{err}");
}
