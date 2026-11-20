//! 開き直し(resume.*): 端末の道具が窓を立て直すときに、同じ表とビューで mdgrid を開き直すための命令。
//!
//! - Herdr の窓では、今の表とビューで開き直す命令を `pane report-agent … -- mdgrid …` で伝え、表かビューを
//!   移るたびに伝え直し、画面を終えるときに `pane release-agent` で窓を返す(resume.herdr-*)。Herdr の道具は
//!   裏の糸で走らせ、返らなければ止めて、画面の操作を待たせない(resume.herdr-no-wait)。
//! - 命令を受け取れない道具(tmux の復元のプラグインなど)のために、起動したフォルダごとの「最後の画面」を
//!   見た目の状態の置き場に書き、`--continue` で読む(resume.last-screen-save・resume.continue)。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 開き直す命令の中身(resume.reopen-args)。ビューは画面の今のビューを添える。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reopen {
    /// 開いている表の絶対パス(フォルダの並びか、`.base`・`.csv` の1つ)。
    pub paths: Vec<PathBuf>,
    /// フォルダ1つを開いている(`--raw` を添えて、印の `open` に置き換わらないようにする)。
    #[serde(default)]
    pub raw: bool,
    /// 今のビューの名前(既定の表だけなら無し)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    /// 起動で渡した `-w`(ワークスペースを開いて移ったらその名前)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub no_color: bool,
    /// 起動で渡した `--config`(絶対パス)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<PathBuf>,
}

/// 命令の最初の語(Herdr は PATH の上の名前だけを受け付ける)。
pub const PROGRAM: &str = "mdgrid";

impl Reopen {
    /// 開いている表から作る。パスは絶対にし、フォルダ1つなら `raw`。
    pub fn new(
        paths: &[PathBuf],
        workspace: Option<String>,
        readonly: bool,
        no_color: bool,
        config: Option<&Path>,
    ) -> Reopen {
        let paths: Vec<PathBuf> = paths.iter().map(|p| absolute(p)).collect();
        let raw = matches!(paths.as_slice(), [p] if p.is_dir());
        Reopen {
            paths,
            raw,
            view: None,
            workspace,
            readonly,
            no_color,
            config: config.map(absolute),
        }
    }

    /// 起動の引数(`mdgrid` を除く。ビューは `with_view` が真のときだけ)。
    pub fn args(&self, with_view: bool) -> Vec<String> {
        let mut out: Vec<String> = self
            .paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        if self.raw {
            out.push("--raw".into());
        }
        if let (true, Some(v)) = (with_view, &self.view) {
            out.push("--view".into());
            out.push(v.clone());
        }
        if let Some(w) = &self.workspace {
            out.push("-w".into());
            out.push(w.clone());
        }
        if self.readonly {
            out.push("--readonly".into());
        }
        if self.no_color {
            out.push("--no-color".into());
        }
        if let Some(c) = &self.config {
            out.push("--config".into());
            out.push(c.to_string_lossy().into_owned());
        }
        out
    }

    /// resume.reopen-args: 開き直す命令(`mdgrid` から)。
    pub fn argv(&self) -> Vec<String> {
        let mut out = vec![PROGRAM.to_string()];
        out.extend(self.args(true));
        out
    }
}

/// 絶対パスにする(シンボリックリンクはたどらない。今のフォルダが読めなければそのまま)。
fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}

/// resume.herdr-unsendable: Herdr が受け付ける命令か(アポストロフィ・制御文字が無く、64 個まで、合わせて 8 KiB まで)。
pub fn herdr_accepts(argv: &[String]) -> bool {
    let total: usize = argv.iter().map(|a| a.len() + 1).sum();
    !argv.is_empty()
        && argv.len() <= 64
        && total <= 8 * 1024
        && argv
            .iter()
            .all(|a| !a.contains('\'') && !a.chars().any(char::is_control))
}

/// resume.herdr-seq: 伝えるたびに大きくなる数。UNIX の時刻のマイクロ秒を下限にして、起動し直しても前の回より
/// 大きくする(同じ時刻に続けて伝えたら1ずつ足す。起動し直しはマイクロ秒より長くかかる)。
#[derive(Debug, Default)]
pub struct Seq {
    last: u64,
}

impl Seq {
    /// 次の数(`now` は今の時刻のマイクロ秒)。
    pub fn next(&mut self, now: u64) -> u64 {
        self.last = (self.last + 1).max(now);
        self.last
    }
}

/// 今の時刻のマイクロ秒。
pub fn now_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Herdr の窓(resume.herdr-report): 道具のパスと窓の ID。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Herdr {
    pub bin: PathBuf,
    pub pane: String,
}

/// Herdr に名乗る名前(`--source` と `--agent`)。`herdr:` で始めない(Herdr の組み込みの連携の印)。
const NAME: &str = "mdgrid";

impl Herdr {
    /// `HERDR_ENV` が `1` で、`HERDR_PANE_ID` と `HERDR_BIN_PATH` があるときだけ。
    pub fn from_env(env: impl Fn(&str) -> Option<String>) -> Option<Herdr> {
        if env("HERDR_ENV").as_deref() != Some("1") {
            return None;
        }
        let pane = env("HERDR_PANE_ID").filter(|v| !v.is_empty())?;
        let bin = env("HERDR_BIN_PATH").filter(|v| !v.is_empty())?;
        Some(Herdr {
            bin: PathBuf::from(bin),
            pane,
        })
    }

    /// `pane report-agent` の引数(状態はいつも idle。tbd~resume.herdr-state)。
    pub fn report_args(&self, seq: u64, argv: &[String]) -> Vec<String> {
        let mut out: Vec<String> = [
            "pane",
            "report-agent",
            &self.pane,
            "--source",
            NAME,
            "--agent",
            NAME,
            "--state",
            "idle",
            "--seq",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        out.push(seq.to_string());
        out.push("--".into());
        out.extend(argv.iter().cloned());
        out
    }

    /// `pane release-agent` の引数。
    pub fn release_args(&self, seq: u64) -> Vec<String> {
        [
            "pane",
            "release-agent",
            &self.pane,
            "--source",
            NAME,
            "--agent",
            NAME,
            "--seq",
        ]
        .iter()
        .map(|s| s.to_string())
        .chain(std::iter::once(seq.to_string()))
        .collect()
    }
}

/// 1回の呼び出しを待つ上限。返らなければ止める(resume.herdr-no-wait)。裏の糸で待つので画面は待たせない。
/// 短すぎると、作ったばかりの実行ファイルの初めての起動(macOS の確かめ)の途中で止めてしまう。
const CALL_LIMIT: Duration = Duration::from_secs(10);
/// 終えるときに、窓を返す呼び出しを待つ上限(前の呼び出しが返らなければ、待たずに終わる)。
const FINISH_LIMIT: Duration = Duration::from_millis(2500);

enum Job {
    Run(Vec<String>),
    Stop,
}

/// Herdr に伝える係(resume.herdr-*)。呼び出しは裏の糸で1つずつ走らせ、たまったら最後の1つだけを走らせる。
pub struct Reporter {
    herdr: Herdr,
    seq: Seq,
    tx: mpsc::Sender<Job>,
    done: mpsc::Receiver<()>,
    /// 前に決めた命令(同じなら伝え直さない)。
    last: Option<Vec<String>>,
}

impl Reporter {
    pub fn start(herdr: Herdr) -> Reporter {
        let (tx, rx) = mpsc::channel::<Job>();
        let (done_tx, done) = mpsc::channel();
        let bin = herdr.bin.clone();
        std::thread::spawn(move || {
            while let Ok(first) = rx.recv() {
                // たまった呼び出しは最後の1つだけ(古い命令は新しい命令で置き換わる)。止める知らせは、
                // その前の最後の呼び出し(窓を返す)を走らせてから効かせる。
                let mut run = None;
                let mut stop = false;
                for job in std::iter::once(first).chain(std::iter::from_fn(|| rx.try_recv().ok())) {
                    match job {
                        Job::Run(args) => run = Some(args),
                        Job::Stop => stop = true,
                    }
                }
                if let Some(args) = run {
                    call(&bin, &args);
                }
                if stop {
                    break;
                }
            }
            let _ = done_tx.send(());
        });
        Reporter {
            herdr,
            seq: Seq::default(),
            tx,
            done,
            last: None,
        }
    }

    /// 今の命令を伝える。前と同じなら何もしない。受け付けられない命令なら、代わりに窓を返して前の命令を取り下げる。
    pub fn update(&mut self, argv: &[String]) {
        if self.last.as_deref() == Some(argv) {
            return;
        }
        self.last = Some(argv.to_vec());
        let seq = self.seq.next(now_us());
        let args = if herdr_accepts(argv) {
            self.herdr.report_args(seq, argv)
        } else {
            self.herdr.release_args(seq)
        };
        let _ = self.tx.send(Job::Run(args));
    }

    /// 画面を終える: 窓を返し、返し終わるまで(上限つきで)待つ。
    pub fn finish(mut self) {
        let seq = self.seq.next(now_us());
        let _ = self.tx.send(Job::Run(self.herdr.release_args(seq)));
        let _ = self.tx.send(Job::Stop);
        let _ = self.done.recv_timeout(FINISH_LIMIT);
    }
}

/// Herdr の道具を1回走らせる。入出力はつながず、上限を過ぎたら止める。失敗は黙って捨てる。
fn call(bin: &Path, args: &[String]) {
    let Ok(mut child) = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return;
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) if start.elapsed() < CALL_LIMIT => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
        }
    }
}

/// 最後の画面の置き場(resume.last-screen-save): 見た目の状態の置き場の下のフォルダ。
const LAST_DIR: &str = "last";

/// 起動したフォルダの最後の画面のファイル。
#[derive(Debug, Serialize, Deserialize)]
struct LastFile {
    /// 起動したフォルダ(実体のパス。ハッシュの重なりで別のフォルダのものを読まないため)。
    folder: String,
    #[serde(flatten)]
    screen: Reopen,
}

/// 起動したフォルダの鍵(実体のパス。tbd~resume.last-screen-key)。
fn folder_key(folder: &Path) -> String {
    std::fs::canonicalize(folder)
        .unwrap_or_else(|_| folder.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn last_file_name(key: &str) -> String {
    let mut name: String = crate::source::content_hash(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    name.push_str(".toml");
    name
}

/// 最後の画面を書く(state_dir は見た目の状態の置き場。一時ファイルから置き換える)。
pub fn save_last(state_dir: &Path, folder: &Path, screen: &Reopen) -> std::io::Result<()> {
    let key = folder_key(folder);
    let file = LastFile {
        folder: key.clone(),
        screen: screen.clone(),
    };
    let text = toml::to_string(&file).map_err(|e| std::io::Error::other(e.to_string()))?;
    crate::config::write_atomic(
        &state_dir.join(LAST_DIR),
        &last_file_name(&key),
        text.as_bytes(),
    )
}

/// 最後の画面を読む。無い・壊れている・別のフォルダのもの → None(resume.continue-fallback)。
pub fn load_last(state_dir: &Path, folder: &Path) -> Option<Reopen> {
    let key = folder_key(folder);
    let text = std::fs::read_to_string(state_dir.join(LAST_DIR).join(last_file_name(&key))).ok()?;
    let file: LastFile = toml::from_str(&text).ok()?;
    (file.folder == key && !file.screen.paths.is_empty()).then_some(file.screen)
}

#[cfg(test)]
#[path = "test_resume_unit.rs"]
mod tests;
