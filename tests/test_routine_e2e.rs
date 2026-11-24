//! ルーチンの回を作る(specs/requirements/routine.md)の `--runs` を、本物の実行ファイルで確かめる。子の標準出力・
//! 標準エラー・終了コードと、台帳のフォルダのファイルを見る。今日は `MDGRID_TODAY=2026-11-24`(火曜)。期待値は要求の
//! 受け入れの例から取った。設定と状態の置き場は一時のホームの中(利用者の置き場には書かない)。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TODAY: &str = "2026-11-24";

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

/// 表「ルーチン」に `runs` を書いた印(回は Runs/ に作る)。
const MARKER: &str = "name = \"暮らし\"\n\n[[table]]\nname = \"ルーチン\"\npath = \"ルーチン\"\nruns = { folder = \"Runs\", rule = \"recurrence\", set = { status = \"todo\" }, copy = [\"actor\"] }\n";

/// ホーム: 台帳 ws(印は `marker`)のルーチンに「週次レビュー」(`weekly mon`・`actor: both`)。
fn home(tag: &str, marker: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in ["config/mdgrid", "state", "ws/ルーチン", "ws/.mdgrid"] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    std::fs::write(p.join("ws/.mdgrid/workspace.toml"), marker).unwrap();
    std::fs::write(
        p.join("ws/ルーチン/週次レビュー.md"),
        "---\nrecurrence: weekly mon\nactor: both\n---\n",
    )
    .unwrap();
    h
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

fn runs(home: &Path) -> Output {
    run(home, &["ws", "--runs"])
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// フォルダの中のファイルと中身(パス → バイト)。無ければ空。
fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let bytes = std::fs::read(&p).unwrap();
                found.insert(p, bytes);
            }
        }
    }
    found
}

#[test]
fn test_runs_cli_makes_the_run_and_prints_its_path() {
    // [stest -> req~routine.cli~1] 作る回が1つある台帳の `--runs` は回のノートを作り、標準出力はそのパスの1行で、終了コード 0。
    let h = home("runs-one", MARKER);
    let o = runs(h.path());
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let made = h.path().join("ws/Runs/週次レビュー 2026-11-23.md");
    assert_eq!(out(&o), format!("{}\n", made.display()));
    assert_eq!(
        std::fs::read_to_string(&made).unwrap(),
        "---\nroutine: \"[[週次レビュー]]\"\ndate: 2026-11-23\nstatus: todo\nactor: both\n---\n# 週次レビュー 2026-11-23\n"
    );
}

#[test]
fn test_runs_cli_nothing_to_do() {
    // [stest -> req~routine.cli~1] 作る回の無い台帳(2回目)と、`runs` の無い印の `--runs` は何も書かず、標準出力は空で、終了コード 0。
    let h = home("runs-none", MARKER);
    assert_eq!(runs(h.path()).status.code(), Some(0));
    let before = files(h.path());
    let o = runs(h.path());
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(files(h.path()), before);

    let h = home(
        "runs-no-marker",
        "name = \"暮らし\"\n\n[[table]]\nname = \"ルーチン\"\npath = \"ルーチン\"\n",
    );
    let before = files(h.path());
    let o = runs(h.path());
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(files(h.path()), before, "何かを書いた");
}

#[test]
fn test_runs_cli_unknown_workspace_fails_fast() {
    // [stest -> req~routine.cli~1] `-w` に無いワークスペースの名前の `--runs` は、cli.fail-fast と同じく理由1行と終了コード 2。
    let h = home("runs-nows", MARKER);
    let o = run(h.path(), &["-w", "nosuch", "--runs"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(err(&o).trim_end().lines().count(), 1, "{}", err(&o));
    assert!(err(&o).contains("nosuch"), "{}", err(&o));
    assert!(o.stdout.is_empty());
    assert!(!h.path().join("ws/Runs").exists());
}

#[test]
fn test_runs_cli_readonly_writes_nothing() {
    // [stest -> req~routine.readonly~1] 作る回のある台帳の `--runs --readonly` は、理由1行と終了コード 2 で、ノートのフォルダは変わらない。
    let h = home("runs-readonly", MARKER);
    let before = files(&h.path().join("ws"));
    let o = run(h.path(), &["ws", "--runs", "--readonly"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(err(&o).trim_end().lines().count(), 1, "{}", err(&o));
    // 理由は、読むだけでは書けないこと(知らないオプションではない)。
    assert!(err(&o).contains("--readonly"), "{}", err(&o));
    assert!(o.stdout.is_empty());
    assert_eq!(files(&h.path().join("ws")), before);
}

#[test]
fn test_runs_cli_at_once_makes_one_note() {
    // [stest -> req~routine.once~1] `--runs` を2つ同時に走らせても、回のノートは1つだけできる。
    let h = home("runs-lock", MARKER);
    let kids: Vec<_> = (0..2)
        .map(|_| {
            command(h.path(), &["ws", "--runs"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let mut printed = 0;
    for k in kids {
        let o = k.wait_with_output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", err(&o));
        printed += out(&o).lines().count();
    }
    assert_eq!(printed, 1, "作ったと2回言った");
    let made: Vec<PathBuf> = files(&h.path().join("ws/Runs")).into_keys().collect();
    assert_eq!(made, [h.path().join("ws/Runs/週次レビュー 2026-11-23.md")]);
}

#[cfg(unix)]
fn set_mode(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
#[test]
fn test_runs_cli_unwritable_folder_fails_then_retries() {
    // [stest -> req~routine.fail~1] 書けない `Runs/` と作る回が1つなら、標準エラーにフォルダを含む1行が出て終了コード 1。書けるように
    // してから `--runs` すると、その回を作る。
    // 前提: Unix の許可(CI の ubuntu と macOS の test で回る。root では書けてしまう)。
    let h = home("runs-unwritable", MARKER);
    let folder = h.path().join("ws/Runs");
    std::fs::create_dir_all(&folder).unwrap();
    set_mode(&folder, 0o555);
    let o = runs(h.path());
    set_mode(&folder, 0o755);
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let e = err(&o);
    assert_eq!(e.trim_end().lines().count(), 1, "{e}");
    assert!(e.contains(&folder.display().to_string()), "{e}");
    assert!(o.stdout.is_empty());
    let o = runs(h.path());
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(folder.join("週次レビュー 2026-11-23.md").is_file());
}

#[cfg(unix)]
#[test]
fn test_runs_cli_one_unwritable_table_still_makes_the_other() {
    // [stest -> req~routine.fail~1] 2つのルーチンの表のうち1つの `folder` だけが書けなければ、書ける方の回は作り、終了コード 1。
    // 前提: Unix の許可(CI の ubuntu と macOS の test で回る)。
    let marker = format!(
        "{MARKER}\n[[table]]\nname = \"仕事\"\npath = \"仕事\"\nruns = {{ folder = \"Locked\", rule = \"recurrence\" }}\n"
    );
    let h = home("runs-two", &marker);
    std::fs::create_dir_all(h.path().join("ws/仕事")).unwrap();
    std::fs::write(
        h.path().join("ws/仕事/週報.md"),
        "---\nrecurrence: daily\n---\n",
    )
    .unwrap();
    let locked = h.path().join("ws/Locked");
    std::fs::create_dir_all(&locked).unwrap();
    set_mode(&locked, 0o555);
    let o = runs(h.path());
    set_mode(&locked, 0o755);
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    assert!(
        err(&o).contains(&locked.display().to_string()),
        "{}",
        err(&o)
    );
    let made = h.path().join("ws/Runs/週次レビュー 2026-11-23.md");
    assert_eq!(out(&o), format!("{}\n", made.display()));
}

/// フォルダを丸ごと写す。
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            std::fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

#[test]
fn test_runs_cli_on_the_routines_sample() {
    // [stest -> req~routine.cli~1] 見本 examples/routines の写しで、2026-10-06(見本の今日)の `--runs` は4つのルーチンの回を作り、
    // 4行のパスを出して終了コード 0。
    // 観点: 見本と説明書の場面(docs/manual-scenarios.toml の routine)が、仕様や実装の変更で開けなくならないこと。
    let h = TempDir::new("runs-sample");
    for d in ["config/mdgrid", "state"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/routines"),
        &h.path().join("ws"),
    );
    let o = command(h.path(), &["ws", "--runs"])
        .env("MDGRID_TODAY", "2026-10-06")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(o.stderr.is_empty(), "{}", err(&o));
    let mut made: Vec<String> = std::fs::read_dir(h.path().join("ws/Runs"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    made.sort();
    assert_eq!(
        made,
        [
            "Back up the server 2026-10-04.md",
            "Check the support inbox 2026-10-06.md",
            "Send invoices 2026-09-30.md",
            "Weekly review 2026-10-05.md",
        ]
    );
    assert_eq!(out(&o).lines().count(), 4, "{}", out(&o));
}
