//! 最近開いた表(specs/requirements/cli.md の「最近開いた表」)を、本物の実行ファイルで確かめる。`--recent` は子の
//! 標準出力(パイプ)で見て、画面で開いて覚えるのは疑似端末(80×24)で見る。期待値は要求の受け入れの例から取った。
//! 覚えた表の置き場は見た目の状態の置き場(`XDG_STATE_HOME/mdgrid`)。

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

/// tasks・projects(と tasks.base のビュー「進行中」)のある作業のフォルダ。
fn home(tag: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in ["config/mdgrid", "state", "notes/tasks", "notes/projects"] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    std::fs::write(p.join("notes/tasks/a.md"), "---\nstatus: todo\n---\n").unwrap();
    std::fs::write(p.join("notes/projects/p.md"), "---\nstatus: active\n---\n").unwrap();
    std::fs::write(
        p.join("notes/tasks.base"),
        "views:\n  - type: table\n    name: 全部\n  - type: table\n    name: 進行中\n",
    )
    .unwrap();
    h
}

/// 覚えた表の置き場(`XDG_STATE_HOME/mdgrid`)。
fn state(home: &Path) -> PathBuf {
    home.join("state/mdgrid")
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
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn test_recent_cli_prints_newest_first() {
    // [stest -> req~cli.recent-cli~2] `mdgrid --recent` → 「パス(タブ)ビューの名前(タブ)最後に開いた日時」の行が新しい順。標準出力はパイプ。
    let h = home("recent-cli");
    let p = h.path();
    let base = p.join("notes/tasks.base");
    let projects = p.join("notes/projects");
    mdgrid::recent::record(&state(p), &projects, None, "2026-11-20 18:02").unwrap();
    mdgrid::recent::record(&state(p), &base, Some("進行中"), "2026-11-21 09:30").unwrap();
    let o = run(p, &["--recent"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(
        out(&o),
        format!(
            "{}\t進行中\t2026-11-21 09:30\n{}\t\t2026-11-20 18:02\n",
            base.display(),
            projects.display()
        )
    );
}

#[test]
fn test_recent_cli_nothing_remembered() {
    // [stest -> req~cli.recent-cli~2] 覚えた表が無い → 何も出さず終了コード 0。
    let h = home("recent-none");
    let o = run(h.path(), &["--recent"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(out(&o), "");
    assert!(
        o.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn test_recent_cli_leaves_out_missing() {
    // [stest -> req~cli.recent-missing~1] 覚えた projects のフォルダを消す → `--recent` に出さず、覚えた並びからも外す。
    let h = home("recent-missing");
    let p = h.path();
    let tasks = p.join("notes/tasks");
    let projects = p.join("notes/projects");
    mdgrid::recent::record(&state(p), &projects, None, "2026-11-20 18:02").unwrap();
    mdgrid::recent::record(&state(p), &tasks, None, "2026-11-21 09:30").unwrap();
    std::fs::remove_dir_all(&projects).unwrap();
    let o = run(p, &["--recent"]);
    assert!(o.status.success());
    assert_eq!(
        out(&o),
        format!("{}\t\t2026-11-21 09:30\n", tasks.display())
    );
    let kept = mdgrid::recent::load(&state(p));
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, tasks);
}

#[test]
fn test_recent_cli_takes_no_path() {
    // `--recent` はパスと一緒に渡せない(理由1行と終了コード 2)。
    let h = home("recent-conflict");
    let o = run(h.path(), &["--recent", "notes"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("--recent"));
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, out, run};
    use crate::pty::Tui;
    use std::path::Path;
    use std::time::{Duration, Instant};

    /// ノートのフォルダの中のファイルとフォルダの並び。
    fn listing(dir: &Path) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                out.push(p.strip_prefix(dir).unwrap().to_string_lossy().into_owned());
                if p.is_dir() {
                    stack.push(p);
                }
            }
        }
        out.sort();
        out
    }

    /// `rel` を画面で開いて、行が出たら `q` で終える(起動の直後の1打は端末の問い合わせに吸われることがあるので、
    /// 終わるまで押す)。
    fn open_and_quit(h: &Path, rel: &str, row: &str) {
        let mut t = Tui::spawn(&[rel], h, &[]);
        t.wait_for(row);
        let end = Instant::now() + Duration::from_secs(10);
        while t.try_exit().is_none() && Instant::now() < end {
            t.send("q");
            std::thread::sleep(Duration::from_millis(300));
        }
        assert_eq!(t.wait_exit(), 0, "{}", t.screen());
    }

    #[test]
    fn test_recent_records_tables_opened_on_screen() {
        // [stest -> req~cli.recent-record~2] tasks を開き、projects、また tasks を開く → tasks・projects の順で tasks は1件だけ。ノートのフォルダに新しいファイルは無い。
        let h = home("recent-screen");
        let p = h.path();
        let before = listing(&p.join("notes"));
        open_and_quit(p, "notes/tasks", "todo");
        open_and_quit(p, "notes/projects", "active");
        open_and_quit(p, "notes/tasks", "todo");
        let o = run(p, &["--recent"]);
        let lines: Vec<String> = out(&o)
            .lines()
            .map(|l| l.split('\t').next().unwrap_or_default().to_string())
            .collect();
        let tasks = p.join("notes/tasks").to_string_lossy().into_owned();
        let projects = p.join("notes/projects").to_string_lossy().into_owned();
        assert_eq!(lines, vec![tasks, projects], "{}", out(&o));
        assert_eq!(listing(&p.join("notes")), before, "ノートのフォルダは同じ");
    }
}
