//! 保存のあとの git(write-back.git-*): mdgrid が書いたノートだけを、設定の段(`[git] auto`)まで git に渡す。
//! `add` はステージまで、`commit` はそのノートだけの1つのコミットまで、`push` はそのあとの push まで。
//! ほかの変更中のファイル・既にステージしたファイルはコミットに入れない(`git commit --only -- <パス>`)。
//! git のリポでないフォルダのファイルは飛ばす。git は端末に問いを出さない形で走らせ、長く返らなければ止める。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// どこまでやるか(`[git] auto`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Level {
    #[default]
    Off,
    Add,
    Commit,
    Push,
}

impl Level {
    pub fn parse(s: &str) -> Option<Level> {
        Some(match s {
            "off" => Level::Off,
            "add" => Level::Add,
            "commit" => Level::Commit,
            "push" => Level::Push,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Level::Off => "off",
            Level::Add => "add",
            Level::Commit => "commit",
            Level::Push => "push",
        }
    }
}

/// いつ走らせるか(`[git] when`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum When {
    /// 保存のたび。
    #[default]
    Save,
    /// 終了のときに1回。
    Quit,
}

impl When {
    pub fn parse(s: &str) -> Option<When> {
        Some(match s {
            "save" => When::Save,
            "quit" => When::Quit,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            When::Save => "save",
            When::Quit => "quit",
        }
    }
}

/// コミットの文の決め方(`[git] message`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Message {
    /// 自動の文。
    #[default]
    Auto,
    /// 保存のあとに1行の欄で聞く(空なら自動の文)。
    Ask,
}

impl Message {
    pub fn parse(s: &str) -> Option<Message> {
        Some(match s {
            "auto" => Message::Auto,
            "ask" => Message::Ask,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Message::Auto => "auto",
            Message::Ask => "ask",
        }
    }
}

/// 設定の `[git]`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Git {
    pub auto: Level,
    pub when: When,
    pub message: Message,
}

/// 走らせた結果(リポをまたいだ合計)。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Done {
    /// ステージしたファイルの数。
    pub added: usize,
    /// 作ったコミットの数(変わっていないリポでは作らない)。
    pub commits: usize,
    /// push したリポの数。
    pub pushed: usize,
}

/// git を待つ上限。
const LIMIT: Duration = Duration::from_secs(60);

/// `path` を含む git のリポの根(`.git` のあるいちばん近い上のフォルダ)。無ければ None。
pub fn repo_root(path: &Path) -> Option<PathBuf> {
    let real = std::fs::canonicalize(path).ok()?;
    let start = if real.is_dir() { &real } else { real.parent()? };
    start
        .ancestors()
        .find(|d| d.join(".git").exists())
        .map(Path::to_path_buf)
}

/// 自動の文の1行目に続けて、空行とリポの根からのパスを1行ずつ(`first` が1行目)。
pub fn message_with_paths(first: &str, root: &Path, files: &[PathBuf]) -> String {
    let mut out = format!("{}\n\n", first.trim());
    for f in files {
        let rel = f.strip_prefix(root).unwrap_or(f);
        out.push_str(&rel.to_string_lossy().replace('\\', "/"));
        out.push('\n');
    }
    out
}

/// `files`(mdgrid が書いたノート)を、リポごとに `level` まで git に渡す。`first` はコミットの文の1行目。
/// リポの外のファイルは飛ばす。失敗したら、そこで止めて理由(git の標準エラーの最後の行)を返す。
pub fn run(level: Level, files: &[PathBuf], first: &str) -> Result<Done, String> {
    let mut done = Done::default();
    if level == Level::Off {
        return Ok(done);
    }
    let mut by_root: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for f in files {
        // 消えたファイル(名前を変えた元など)も、親のフォルダからリポを引く。
        let (root, real) = match std::fs::canonicalize(f) {
            Ok(real) => (repo_root(&real), real),
            Err(_) => match f.parent().and_then(|d| std::fs::canonicalize(d).ok()) {
                Some(d) => (repo_root(&d), d.join(f.file_name().unwrap_or_default())),
                None => continue,
            },
        };
        if let Some(root) = root {
            let list = by_root.entry(root).or_default();
            if !list.contains(&real) {
                list.push(real);
            }
        }
    }
    for (root, files) in &by_root {
        let paths: Vec<&str> = files.iter().filter_map(|f| f.to_str()).collect();
        let mut add = vec!["add", "--"];
        add.extend(&paths);
        git(root, &add)?;
        done.added += paths.len();
        if level < Level::Commit {
            continue;
        }
        // そのノートに、HEAD との差が無ければコミットしない。
        let mut diff = vec!["diff", "--cached", "--quiet", "--"];
        diff.extend(&paths);
        if git_status(root, &diff)? != 0 {
            let msg = message_with_paths(first, root, files);
            let mut commit = vec!["commit", "--quiet", "--only", "-m", &msg, "--"];
            commit.extend(&paths);
            git(root, &commit)?;
            done.commits += 1;
        }
        if level == Level::Push {
            git(root, &["push", "--quiet"])?;
            done.pushed += 1;
        }
    }
    Ok(done)
}

/// git を走らせ、0 で終わらなければ理由を返す。
fn git(root: &Path, args: &[&str]) -> Result<(), String> {
    let (code, err) = git_run(root, args)?;
    if code == 0 {
        Ok(())
    } else {
        let last = err.lines().rev().find(|l| !l.trim().is_empty());
        Err(format!(
            "git {}: {}",
            args[0],
            last.unwrap_or("failed").trim()
        ))
    }
}

/// git を走らせて終了コードを返す(`diff --quiet` のように、0 でないことが失敗でないもの)。
fn git_status(root: &Path, args: &[&str]) -> Result<i32, String> {
    git_run(root, args).map(|(c, _)| c)
}

fn git_run(root: &Path, args: &[&str]) -> Result<(i32, String), String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git: {e}"))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > LIMIT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("git {}: timed out", args[0]));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("git: {e}")),
        }
    }
    let out = child.wait_with_output().map_err(|e| format!("git: {e}"))?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    ))
}

#[cfg(test)]
#[path = "test_gitsync_unit.rs"]
mod tests;
