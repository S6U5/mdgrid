//! 開き直し(resume.*): 端末の道具が窓を立て直すときに、同じ表とビューで mdgrid を開き直すための命令。
//!
//! - Herdr の窓では、今の表とビューで開き直す命令を `pane report-agent … -- mdgrid …` で伝え、表かビューを
//!   移るたびに伝え直し、画面を終えるときに `pane release-agent` で窓を返す(resume.herdr-*)。Herdr の道具は
//!   裏の糸で走らせ、返らなければ止めて、画面の操作を待たせない(resume.herdr-no-wait)。
//! - 命令と一緒に、窓の題(表とビューの名前)を `pane report-metadata` で伝え、終えるときは窓を返す前に消す
//!   (resume.herdr-title・resume.herdr-title-form・resume.herdr-title-clear)。
//! - 命令を受け取れない道具(tmux の復元のプラグインなど)のために、起動したフォルダごとの「最後の画面」を
//!   見た目の状態の置き場に書き、`--continue` で読む(resume.last-screen-save・resume.continue)。

use crate::workspace::{self, WsTable};
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
    /// stack.page-resume: 開いているページ(`--page`)。あれば表とビューの代わりに、ワークスペースとページで開き直す。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
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
            page: None,
        }
    }

    /// 開いている表を `paths` に替えたもの(stack.focus: 縦に並べた表では、選んでいる表。旗はそのまま)。
    pub fn retarget(&self, paths: &[PathBuf]) -> Reopen {
        let mut r = Reopen::new(
            paths,
            self.workspace.clone(),
            self.readonly,
            self.no_color,
            None,
        );
        r.config.clone_from(&self.config);
        r
    }

    /// 起動の引数(`mdgrid` を除く。ビューは `with_view` が真のときだけ)。ページを開いているときは
    /// (stack.page-resume)、`-w <ワークスペース> --page <名前>`(印のワークスペースは `<表のパス> --page <名前>`)。
    pub fn args(&self, with_view: bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if let Some(p) = &self.page {
            match &self.workspace {
                Some(w) => {
                    out.push("-w".into());
                    out.push(w.clone());
                }
                None => out.extend(self.paths.iter().map(|p| p.to_string_lossy().into_owned())),
            }
            out.push("--page".into());
            out.push(p.clone());
        } else {
            out.extend(self.paths.iter().map(|p| p.to_string_lossy().into_owned()));
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

/// resume.herdr-title-form: 窓の題の表の名前。開いた表と同じパスのワークスペースの表(`tables`。いくつもあれば
/// ビューが `view` のもの)があればその `name`、無ければ `.base` のファイル名(拡張子を除く)かフォルダの名前
/// (フォルダを複数開いていれば `, ` でつなぐ)。
pub fn table_name(paths: &[PathBuf], tables: &[WsTable], view: Option<&str>) -> String {
    if let [opened] = paths {
        let same: Vec<&WsTable> = tables
            .iter()
            .filter(|t| !t.name.trim().is_empty() && workspace::same_path(&t.path, opened))
            .collect();
        let pick = same
            .iter()
            .find(|t| t.view.is_some() && t.view.as_deref() == view)
            .or_else(|| same.first());
        if let Some(t) = pick {
            return t.name.clone();
        }
    }
    paths
        .iter()
        .map(|p| workspace::stem_of(p))
        .collect::<Vec<_>>()
        .join(", ")
}

/// resume.herdr-title・resume.herdr-title-form: 窓の題「表 · ビュー」。ビューを添えないとき(ビューのタブを出さない
/// 表)は表の名前だけ。
pub fn title(table: &str, view: Option<&str>) -> String {
    match view {
        Some(v) => format!("{table} · {v}"),
        None => table.to_string(),
    }
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
        .map_or(0, |d| d.as_micros() as u64)
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
        .map(std::string::ToString::to_string)
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
        .map(std::string::ToString::to_string)
        .chain(std::iter::once(seq.to_string()))
        .collect()
    }

    /// `pane report-metadata` の引数。`Some` は窓の題をその文字にし、`None` は `--clear-title` で消す
    /// (resume.herdr-title・resume.herdr-title-clear)。
    pub fn title_args(&self, seq: u64, title: Option<&str>) -> Vec<String> {
        let mut out: Vec<String> = [
            "pane",
            "report-metadata",
            &self.pane,
            "--source",
            NAME,
            "--agent",
            NAME,
        ]
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
        match title {
            Some(t) => {
                out.push("--title".into());
                out.push(t.to_string());
            }
            None => out.push("--clear-title".into()),
        }
        out.push("--seq".into());
        out.push(seq.to_string());
        out
    }
}

/// 窓に伝えたいこと(新しいものが古いものを置き換える)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Want {
    /// 開き直す命令と窓の題を伝える。命令を受け付けられなければ窓を返す(resume.herdr-unsendable)。
    Show { argv: Vec<String>, title: String },
    /// 題を消して窓を返す(画面を終える。resume.herdr-release・resume.herdr-title-clear)。
    Release,
}

/// Herdr に伝え終えたこと(裏の糸が持つ)。伝えたいことから、走らせる呼び出しの並びを決める。
#[derive(Debug, Default)]
pub struct Sent {
    seq: Seq,
    /// 窓を mdgrid が持っている(report-agent を伝え、まだ返していない)。
    held: bool,
    /// 題を伝え、まだ消していない。
    titled: bool,
}

impl Sent {
    /// `want` を伝える呼び出しの並び(`now` は今の時刻のマイクロ秒)。題(`--agent mdgrid` つき)は、窓を mdgrid が
    /// 持っている間に伝える: 初めて持つときは report-agent のあと、持っている間は report-agent の前(ビューを移った
    /// 知らせが届いたときには題も替わっている)、返すときは release-agent の前に消す。題に Herdr が受け付けない文字が
    /// あれば(命令と同じ決まり。resume.herdr-unsendable)題を伝えず、前の題があれば消す。
    pub fn plan(&mut self, herdr: &Herdr, want: &Want, now: u64) -> Vec<Vec<String>> {
        let mut out = Vec::new();
        match want {
            Want::Show { argv, title } if herdr_accepts(argv) => {
                let title = Some(title.as_str())
                    .filter(|t| !t.trim().is_empty() && herdr_accepts(&[(*t).to_string()]));
                let held = self.held;
                if !held {
                    out.push(herdr.report_args(self.seq.next(now), argv));
                }
                if title.is_some() || self.titled {
                    out.push(herdr.title_args(self.seq.next(now), title));
                }
                if held {
                    out.push(herdr.report_args(self.seq.next(now), argv));
                }
                self.held = true;
                self.titled = title.is_some();
            }
            Want::Show { .. } | Want::Release => {
                if self.titled {
                    out.push(herdr.title_args(self.seq.next(now), None));
                }
                out.push(herdr.release_args(self.seq.next(now)));
                self.held = false;
                self.titled = false;
            }
        }
        out
    }
}

/// 1回の呼び出しを待つ上限。返らなければ止める(resume.herdr-no-wait)。裏の糸で待つので画面は待たせない。
/// 短すぎると、作ったばかりの実行ファイルの初めての起動(macOS の確かめ)の途中で止めてしまう。
const CALL_LIMIT: Duration = Duration::from_secs(10);
/// 終えるときに、窓を返す呼び出しを待つ上限(前の呼び出しが返らなければ、待たずに終わる)。
const FINISH_LIMIT: Duration = Duration::from_millis(2500);

enum Job {
    Send(Want),
    Stop,
}

/// Herdr に伝える係(resume.herdr-*)。伝えたいことは裏の糸で1つずつ伝え、たまったら最後の1つだけを伝える。
pub struct Reporter {
    tx: mpsc::Sender<Job>,
    done: mpsc::Receiver<()>,
    /// 前に伝えたいとしたこと(同じなら伝え直さない)。
    last: Option<Want>,
}

impl Reporter {
    pub fn start(herdr: Herdr) -> Reporter {
        let (tx, rx) = mpsc::channel::<Job>();
        let (done_tx, done) = mpsc::channel();
        std::thread::spawn(move || {
            let mut sent = Sent::default();
            while let Ok(first) = rx.recv() {
                // たまったものは最後の1つだけ(古い命令と題は新しいもので置き換わる)。止める知らせは、
                // その前の最後のもの(題を消して窓を返す)を伝えてから効かせる。
                let mut want = None;
                let mut stop = false;
                for job in std::iter::once(first).chain(std::iter::from_fn(|| rx.try_recv().ok())) {
                    match job {
                        Job::Send(w) => want = Some(w),
                        Job::Stop => stop = true,
                    }
                }
                if let Some(w) = want {
                    for args in sent.plan(&herdr, &w, now_us()) {
                        call(&herdr.bin, &args);
                    }
                }
                if stop {
                    break;
                }
            }
            let _ = done_tx.send(());
        });
        Reporter {
            tx,
            done,
            last: None,
        }
    }

    /// 今の命令と窓の題を伝える。前と同じなら何もしない。受け付けられない命令なら、代わりに窓を返して前の命令を
    /// 取り下げる(resume.herdr-unsendable)。
    pub fn update(&mut self, argv: &[String], title: &str) {
        let want = Want::Show {
            argv: argv.to_vec(),
            title: title.to_string(),
        };
        if self.last.as_ref() == Some(&want) {
            return;
        }
        self.last = Some(want.clone());
        let _ = self.tx.send(Job::Send(want));
    }

    /// 画面を終える: 題を消して窓を返し、返し終わるまで(上限つきで)待つ。
    pub fn finish(self) {
        let _ = self.tx.send(Job::Send(Want::Release));
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
