//! 新しく当たった行を知らせる(specs/requirements/notify.md)を、本物の実行ファイルで確かめる。`--notify` は子の
//! 標準出力・標準エラー・終了コードで、開いている画面は疑似端末(80×24)で見る。今日は `MDGRID_TODAY=2026-11-21`。
//! 期待値は要求の受け入れの例から取った。
//!
//! 出す先は `"command"` にし、どの OS でも動くよう、この実行ファイルの `--new-note` を動かす(鳴った行の文を名前に
//! した `out/<文>.md` ができる)。同じ行をもう一度鳴らすと、同じ名前のノートは作らずに落ちるので、二度鳴らしたかも
//! 終了コードで分かる。OS の通知(`"desktop"`)は、試験を回すマシンに本物の通知が出るので使わない。

use std::collections::BTreeMap;
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

const OVERDUE: &str = "[conditions.overdue]\nexpr = 'due < today() && status != \"done\"'\n";

/// 鳴った行の文を名前にしたノートを out に作るコマンド(TOML のリテラルの文字列。Windows の `\` もそのまま)。
fn record(home: &Path) -> String {
    format!(
        "['{}', '{}', \"--new-note\", \"{{text}}\"]",
        env!("CARGO_BIN_EXE_mdgrid"),
        home.join("out").display()
    )
}

/// 終了コード 2 で終わるコマンド(知らないオプション)。
fn failing() -> String {
    format!("['{}', \"--no-such-option\"]", env!("CARGO_BIN_EXE_mdgrid"))
}

/// ワークスペース ws(印あり)の tasks に期限切れ a・b、先の c。設定は `items`(OVERDUE のあと)。
fn home(tag: &str, items: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in ["config/mdgrid", "state", "ws/tasks", "ws/.mdgrid", "out"] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    let w = |rel: &str, text: &str| std::fs::write(p.join(rel), text).unwrap();
    w("ws/tasks/a.md", "---\ndue: 2026-11-10\nstatus: todo\n---\n");
    w(
        "ws/tasks/b.md",
        "---\ndue: 2026-11-11\nstatus: doing\n---\n",
    );
    w("ws/tasks/c.md", "---\ndue: 2026-12-11\nstatus: todo\n---\n");
    w("ws/.mdgrid/workspace.toml", "name = \"work\"\n");
    w(
        "config/mdgrid/config.toml",
        &format!(
            "{OVERDUE}{}",
            items
                .replace("RECORD", &record(p))
                .replace("FAILING", &failing())
        ),
    );
    h
}

/// tasks に期限切れのノートを足す。
fn overdue(home: &Path, name: &str) {
    std::fs::write(
        home.join("ws/tasks").join(format!("{name}.md")),
        "---\ndue: 2026-11-01\nstatus: todo\n---\n",
    )
    .unwrap();
}

fn command(home: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("MDGRID_TODAY", TODAY);
    // Windows の子は SystemRoot が無いと動かないものがある(env_clear で消えるので戻す)。
    for k in ["SystemRoot", "SYSTEMROOT", "TEMP", "TMP"] {
        if let Some(v) = std::env::var_os(k) {
            c.env(k, v);
        }
    }
    c
}

fn run(home: &Path, args: &[&str]) -> Output {
    command(home, args).output().unwrap()
}

fn notify(home: &Path) -> Output {
    run(home, &["ws/tasks", "--notify"])
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// out に作られたノートの名前(鳴った行の文)。
fn rung(home: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(home.join("out"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn state(home: &Path) -> PathBuf {
    home.join("state").join("mdgrid").join("notify.json")
}

const ITEM: &str = "[[notify]]\ncond = \"overdue\"\nto = [\"command\"]\ncommand = RECORD\n";

#[test]
fn test_notify_cli_rings_new_rows_quietly() {
    // [stest -> req~notify.cli~1] 前の --notify のあとで新しく当たった行が1つなら、コマンドが1回動き、標準出力は空で
    // 終了コード 0。初めての確かめは黙って覚え、当たり続ける行はもう鳴らさない。
    let h = home("notify-cli", ITEM);
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert!(o.stdout.is_empty());
    assert!(rung(h.path()).is_empty(), "初めての確かめは黙って覚える");
    overdue(h.path(), "n");
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{:?}", o.stdout);
    assert_eq!(rung(h.path()), ["n.md"]);
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(rung(h.path()), ["n.md"]);
}

#[test]
fn test_notify_cli_nothing_to_do() {
    // [stest -> req~notify.cli~1] [[notify]] が無ければ何も鳴らさず、標準出力は空で終了コード 0。記録も作らない。
    let h = home("notify-none", "");
    overdue(h.path(), "n");
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert!(o.stdout.is_empty());
    assert!(!state(h.path()).exists());
}

#[test]
fn test_notify_cli_unknown_workspace_fails_fast() {
    // [stest -> req~notify.cli~1] -w に無いワークスペースの名前は、cli.fail-fast と同じく理由1行と終了コード 2。
    let h = home("notify-nows", ITEM);
    let o = run(h.path(), &["-w", "nosuch", "--notify"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(err(&o).trim_end().lines().count(), 1, "{}", err(&o));
    assert!(o.stdout.is_empty());
}

#[test]
fn test_notify_cli_failure_retries() {
    // [stest -> req~notify.fail~1] コマンドが落ちたら、出す先を含む1行を標準エラーに出して終了コード 1。覚えないので、
    // 次の --notify でもまた鳴らそうとする。
    let h = home(
        "notify-fail",
        "[[notify]]\ncond = \"overdue\"\nto = [\"command\"]\ncommand = FAILING\n",
    );
    assert!(notify(h.path()).status.success());
    overdue(h.path(), "n");
    for _ in 0..2 {
        let o = notify(h.path());
        assert_eq!(o.status.code(), Some(1), "{}", err(&o));
        let e = err(&o);
        assert_eq!(e.trim_end().lines().count(), 1, "{e}");
        assert!(e.contains("command") && e.contains('2'), "{e}");
    }
}

#[test]
fn test_notify_cli_one_destination_is_enough() {
    // [stest -> req~notify.fail~1] [notify.terminal] 端末の合図は、標準出力がパイプなら送らず、届けられなかったとは
    // 数えない。コマンドだけが落ちたら終了コード 1 だが、その行は覚え、次の --notify では鳴らさない。
    let h = home(
        "notify-partial",
        "[[notify]]\ncond = \"overdue\"\nto = [\"terminal\", \"command\"]\ncommand = FAILING\n",
    );
    assert!(notify(h.path()).status.success());
    overdue(h.path(), "n");
    let o = notify(h.path());
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(
        o.stdout.is_empty(),
        "パイプには合図を送らない: {:?}",
        o.stdout
    );
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert!(o.stderr.is_empty(), "{}", err(&o));
}

#[test]
fn test_notify_cli_unreadable_table() {
    // [stest -> req~notify.fail~1] 範囲の表の1つのフォルダが消えていたら、ほかの表は確かめて鳴らし、消えた表の理由を
    // 標準エラーに出して終了コード 1。
    let h = home("notify-gone", ITEM);
    std::fs::write(
        h.path().join("ws/.mdgrid/workspace.toml"),
        "name = \"work\"\n[[table]]\nname = \"tasks\"\npath = \"tasks\"\n[[table]]\nname = \"gone\"\npath = \"gone\"\n",
    )
    .unwrap();
    let o = notify(h.path());
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    overdue(h.path(), "n");
    let o = notify(h.path());
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(err(&o).contains("gone"), "{}", err(&o));
    assert_eq!(rung(h.path()), ["n.md"]);
}

#[test]
fn test_notify_cli_runs_at_once_ring_once() {
    // [stest -> req~notify.state~1] --notify を3つ同時に走らせても、新しく当たった行は1回だけ鳴る(2回目は同じ名前の
    // ノートを作れずに落ちるので、どれも終了コード 0 なら1回)。
    let h = home("notify-lock", ITEM);
    assert!(notify(h.path()).status.success());
    overdue(h.path(), "n");
    let kids: Vec<_> = (0..3)
        .map(|_| {
            command(h.path(), &["ws/tasks", "--notify"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for k in kids {
        let o = k.wait_with_output().unwrap();
        assert!(o.status.success(), "{}", err(&o));
    }
    assert_eq!(rung(h.path()), ["n.md"]);
}

/// フォルダの中のファイルと中身(パス → バイト)。
fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let bytes = std::fs::read(&p).unwrap();
                out.insert(p, bytes);
            }
        }
    }
    out
}

#[test]
fn test_notify_cli_leaves_notes_alone() {
    // [stest -> req~notify.state~1] --notify で鳴らしても、ノートのフォルダのファイルの数とバイトは変わらない。
    let h = home("notify-untouched", ITEM);
    assert!(notify(h.path()).status.success());
    overdue(h.path(), "n");
    let before = files(&h.path().join("ws"));
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(rung(h.path()), ["n.md"]);
    assert_eq!(files(&h.path().join("ws")), before);
}

#[test]
fn test_notify_cli_broken_state_is_rebuilt() {
    // [stest -> req~notify.state-broken~1] 壊れた記録は、当たっている行を鳴らさずに覚え直し、標準エラーに1行出して終了
    // コード 0。次の --notify では、新しく当たった行だけを鳴らす。
    let h = home("notify-broken", ITEM);
    std::fs::create_dir_all(h.path().join("state/mdgrid")).unwrap();
    std::fs::write(state(h.path()), "{ 壊れた").unwrap();
    overdue(h.path(), "m");
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o).trim_end().lines().count(), 1, "{}", err(&o));
    assert!(rung(h.path()).is_empty());
    overdue(h.path(), "n");
    let o = notify(h.path());
    assert!(o.status.success(), "{}", err(&o));
    assert!(o.stderr.is_empty(), "{}", err(&o));
    assert_eq!(rung(h.path()), ["n.md"]);
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, notify, overdue, rung, ITEM, TODAY};
    use crate::pty::Tui;
    use std::time::{Duration, Instant};

    #[test]
    fn test_notify_screen_then_cli_rings_once() {
        // [stest -> req~notify.state~1] 開いている画面が行を鳴らしたあとの --notify は、その行を鳴らさない。
        let h = home("notify-screen", ITEM);
        let p = h.path();
        assert!(notify(p).status.success());
        overdue(p, "n");
        let t = Tui::spawn(&["ws/tasks"], p, &[("MDGRID_TODAY", TODAY)]);
        let end = Instant::now() + Duration::from_secs(15);
        while rung(p).is_empty() && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(100));
        }
        drop(t);
        assert_eq!(rung(p), ["n.md"], "開いたときの確かめで鳴る");
        let o = notify(p);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(o.stderr.is_empty());
        assert_eq!(rung(p), ["n.md"]);
    }
}
