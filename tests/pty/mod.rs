#![cfg(unix)]
#![allow(dead_code)]
//! 本物の実行ファイルを疑似端末で動かす試験の道具(tests/ の試験から `mod pty;` で使う)。
//!
//! - 疑似端末(80×24)で `mdgrid` を起動する。環境は空にして、PATH・HOME・TERM・LANG・
//!   XDG_CONFIG_HOME(`<home>/config`)・XDG_STATE_HOME(`<home>/state`)と、呼ぶ側が決めた変数だけを渡す。
//! - 読む糸が出力を vt100::Parser に流し、端末への問い合わせ(カーソルの位置・端末の属性)に答える。
//! - 画面やファイルの待ちは 10 秒で落ち、落ちたときは画面の中身を出す。
//! - 子は Drop で kill と wait をする(落ちても止まる)。読む糸は子の側の端が閉じると終わる。
//!
//! このファイルには要件の ID を書かない(錠の対象にしない)。

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

pub const LIMIT: Duration = Duration::from_secs(10);
pub const ROWS: u16 = 24;
pub const COLS: u16 = 80;

/// 試験ごとの一時フォルダ。Drop で消す。
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> TempDir {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "mdgrid-e2e-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        TempDir(p.canonicalize().unwrap())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 偽のエディタ。受け取った引数を1行ずつログに書く。
pub struct FakeEditor {
    pub script: PathBuf,
    pub log: PathBuf,
}

/// 偽のエディタを `dir/<name>` に作る(chmod 755)。
/// 引数を1行ずつ `dir/<name>.log` に書き(tmp から mv で置く)、
/// rewrite が Some なら最後の引数のファイルをその中身に書き換え、
/// 画面を消して `FAKE-EDITOR-RAN` を出し、0.5 秒待って終わる。
pub fn fake_editor(dir: &Path, name: &str, rewrite: Option<&str>) -> FakeEditor {
    let script = dir.join(name);
    let log = dir.join(format!("{name}.log"));
    let tmp = dir.join(format!("{name}.log.tmp"));
    let mut s = String::from("#!/bin/sh\n");
    s += &format!(
        "for a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\n",
        tmp.display()
    );
    s += &format!("mv '{}' '{}'\n", tmp.display(), log.display());
    if let Some(body) = rewrite {
        let body_file = dir.join(format!("{name}.body"));
        std::fs::write(&body_file, body).unwrap();
        s += "for a in \"$@\"; do last=\"$a\"; done\n";
        s += &format!("cat '{}' > \"$last\"\n", body_file.display());
    }
    s += &format!("printf '\\033[2J\\033[H{}\\n'\n", FakeEditor::MARK);
    s += "sleep 0.5\n";
    std::fs::write(&script, s).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    FakeEditor { script, log }
}

impl FakeEditor {
    /// エディタが画面に出す印。
    pub const MARK: &'static str = "FAKE-EDITOR-RAN";

    pub fn path(&self) -> &str {
        self.script.to_str().unwrap()
    }

    pub fn was_called(&self) -> bool {
        self.log.exists()
    }

    /// 呼ばれるまで待ち(上限 10 秒)、受け取った引数を返す。
    pub fn wait_args(&self, tui: &Tui) -> Vec<String> {
        let start = Instant::now();
        while !self.log.exists() {
            if start.elapsed() > LIMIT {
                panic!(
                    "偽のエディタ {} が呼ばれない。画面:\n{}",
                    self.script.display(),
                    tui.screen()
                );
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        std::fs::read_to_string(&self.log)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

/// 疑似端末で動く mdgrid。Drop で kill と wait をする。
pub struct Tui {
    child: Box<dyn Child + Send + Sync>,
    pid: Option<u32>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _master: Box<dyn MasterPty + Send>,
    parser: Arc<Mutex<vt100::Parser>>,
    done: Arc<AtomicBool>,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Tui {
    /// `home` を作業フォルダと HOME にして起動する。env は足す環境変数。
    pub fn spawn(args: &[&str], home: &Path, env: &[(&str, &str)]) -> Tui {
        Tui::spawn_program(env!("CARGO_BIN_EXE_mdgrid"), args, home, env)
    }

    /// `spawn` と同じ環境で、mdgrid の代わりに program(`/bin/sh` など)を疑似端末の上に起動する。
    /// 子の標準出力だけをファイルやパイプに向けたいときに、sh を挟んで使う。
    pub fn spawn_program(program: &str, args: &[&str], home: &Path, env: &[(&str, &str)]) -> Tui {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: ROWS,
                cols: COLS,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut cmd = CommandBuilder::new(program);
        cmd.args(args);
        cmd.cwd(home);
        cmd.env_clear();
        cmd.env(
            "PATH",
            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into()),
        );
        cmd.env("HOME", home);
        cmd.env("TERM", "xterm-256color");
        // 文言の言語(SR-23)は日本語に固定する(呼ぶ側の env で上書きできる)。
        cmd.env("LANG", "ja_JP.UTF-8");
        cmd.env("XDG_CONFIG_HOME", home.join("config"));
        cmd.env("XDG_STATE_HOME", home.join("state"));
        for (k, v) in env {
            cmd.env(k, v);
        }
        let child = pair.slave.spawn_command(cmd).unwrap();
        let pid = child.process_id();
        drop(pair.slave);
        let mut rd = pair.master.try_clone_reader().unwrap();
        let writer = Arc::new(Mutex::new(pair.master.take_writer().unwrap()));
        let parser = Arc::new(Mutex::new(vt100::Parser::new(ROWS, COLS, 0)));
        let done = Arc::new(AtomicBool::new(false));
        let (p2, d2, w2) = (parser.clone(), done.clone(), writer.clone());
        // 読む糸: 子の側の端が全部閉じると read が 0 か誤りを返して終わる。
        let reader = std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut tail: Vec<u8> = Vec::new();
            loop {
                let n = match rd.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                let mut p = p2.lock().unwrap();
                p.process(&buf[..n]);
                // 端末への問い合わせ(カーソルの位置・端末の属性)に本物の端末のように答える。
                tail.extend_from_slice(&buf[..n]);
                let mut replies = Vec::new();
                for _ in 0..count(&tail, b"\x1b[6n") {
                    let (r, c) = p.screen().cursor_position();
                    replies.push(format!("\x1b[{};{}R", r + 1, c + 1));
                }
                for _ in 0..count(&tail, b"\x1b[c") {
                    replies.push("\x1b[?1;2c".to_string());
                }
                drop(p);
                // 読みの境で切れた問い合わせの頭だけを次に持ち越す。
                match tail.iter().rposition(|&b| b == 0x1b) {
                    Some(i) if tail.len() - i < 4 && !tail[i..].ends_with(b"n") => {
                        tail.drain(..i);
                    }
                    _ => tail.clear(),
                }
                if !replies.is_empty() {
                    let mut w = w2.lock().unwrap();
                    for r in replies {
                        let _ = w.write_all(r.as_bytes());
                    }
                    let _ = w.flush();
                }
            }
            d2.store(true, Ordering::SeqCst);
        });
        Tui {
            child,
            pid,
            writer,
            _master: pair.master,
            parser,
            done,
            reader: Some(reader),
        }
    }

    /// 今の画面の文字。
    pub fn screen(&self) -> String {
        self.parser.lock().unwrap().screen().contents()
    }

    /// 画面が条件を満たすまで待つ(上限 10 秒。上限で落ちる)。
    pub fn wait_until(&self, what: &str, cond: impl Fn(&str) -> bool) {
        let start = Instant::now();
        loop {
            let s = self.screen();
            if cond(&s) {
                return;
            }
            if start.elapsed() > LIMIT {
                panic!("{what} を待って時間切れ。画面:\n{s}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// 画面のどこかに text が出るまで待つ。
    pub fn wait_for(&self, text: &str) {
        self.wait_until(&format!("{text:?}"), |s| s.contains(text));
    }

    /// キー(端末に打つバイト)を送る。
    pub fn send(&mut self, keys: &str) {
        let mut w = self.writer.lock().unwrap();
        w.write_all(keys.as_bytes()).unwrap();
        w.flush().unwrap();
    }

    /// キーを送ったあと、画面が落ち着くまで少し置く。
    pub fn send_settle(&mut self, keys: &str) {
        self.send(keys);
        std::thread::sleep(Duration::from_millis(300));
    }

    /// 子が終わっていれば終了コードを返す(待たない)。
    pub fn try_exit(&mut self) -> Option<u32> {
        match self.child.try_wait() {
            Ok(Some(st)) => Some(st.exit_code()),
            _ => None,
        }
    }

    /// 子が終わるまで待ち(上限 10 秒。上限で落ちる)、終了コードを返す。
    pub fn wait_exit(&mut self) -> u32 {
        let start = Instant::now();
        loop {
            if let Some(code) = self.try_exit() {
                return code;
            }
            if start.elapsed() > LIMIT {
                panic!("子が終わらない。画面:\n{}", self.screen());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// 止める(Drop で kill と wait)。子の pid を返す。
    pub fn stop(self) -> Option<u32> {
        self.pid
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // 読む糸を待つ(上限つき)。孫が端を持ったままでも試験は止まらない。
        let start = Instant::now();
        while !self.done.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(20));
        }
        if let Some(h) = self.reader.take() {
            if self.done.load(Ordering::SeqCst) {
                let _ = h.join();
            }
        }
    }
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    hay.windows(needle.len()).filter(|w| *w == needle).count()
}

/// 止めた子が残っていない(wait 済みで、その pid のプロセスが無い)。
pub fn assert_gone(pid: Option<u32>) {
    let pid = pid.expect("子の pid が取れない");
    let out = std::process::Command::new("ps")
        .args(["-o", "pid=,stat=,comm=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.trim().is_empty(),
        "mdgrid の子 {pid} が残っている: {text}"
    );
}

/// ファイルの中身が条件を満たすまで待つ(上限 10 秒)。
pub fn wait_file(path: &Path, tui: &Tui, cond: impl Fn(&str) -> bool) -> String {
    let start = Instant::now();
    loop {
        let s = std::fs::read_to_string(path).unwrap_or_default();
        if cond(&s) {
            return s;
        }
        if start.elapsed() > LIMIT {
            panic!(
                "{} が期待の中身にならない: {s:?}\n画面:\n{}",
                path.display(),
                tui.screen()
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
