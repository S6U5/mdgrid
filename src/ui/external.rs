//! 外とのやりとり: エディタ(SR-8)とコピー(OUT-1)。`impl App` の続き。
//! ノートのパスは RowId から得る(Markdown では RowId.0 が実体のパス。SC-14: frontmatter・vault は直接使わない)。

use super::app::App;
use super::view::LABEL_HEADER;
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// コピーの送り先(OUT-1)。試験では OS のクリップボードに書かないものに差し替える。
pub trait Clipboard {
    /// 端末へ OSC 52 で送る(長すぎるときは送らずに理由)。
    fn osc52(&mut self, text: &str) -> Result<(), String>;
    /// OS のクリップボードへ送る。
    fn system(&mut self, text: &str) -> Result<(), String>;
}

/// 本物の送り先: 標準出力(端末)への OSC 52 と、OS のクリップボードのコマンド。
#[cfg_attr(test, allow(dead_code))]
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn osc52(&mut self, text: &str) -> Result<(), String> {
        write_osc52(&mut io::stdout(), text)
    }

    fn system(&mut self, text: &str) -> Result<(), String> {
        os_copy(text)
    }
}

/// `--pick`(OUT-3)の送り先: 画面と同じ端末(`/dev/tty`)へ OSC 52 を送る(標準出力の結果に混ぜない)。
/// OS のクリップボードへは `SystemClipboard` と同じ。
#[cfg_attr(test, allow(dead_code))]
pub struct TtyClipboard;

impl Clipboard for TtyClipboard {
    fn osc52(&mut self, text: &str) -> Result<(), String> {
        let mut tty = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/tty")
            .map_err(|e| e.to_string())?;
        write_osc52(&mut tty, text)
    }

    fn system(&mut self, text: &str) -> Result<(), String> {
        os_copy(text)
    }
}

/// OSC 52 の列を `out` に書く(tmux の中なら素通しの形)。
#[cfg_attr(test, allow(dead_code))]
fn write_osc52(out: &mut dyn Write, text: &str) -> Result<(), String> {
    let tmux = std::env::var_os("TMUX").is_some_and(|v| !v.is_empty());
    for seq in osc52_seqs(text, tmux)? {
        out.write_all(&seq).map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

/// 試験の送り先: 送ったものを覚えるだけ。
#[cfg(test)]
#[derive(Default)]
pub(crate) struct Memory(pub std::rc::Rc<std::cell::RefCell<MemoryLog>>);

#[cfg(test)]
#[derive(Default)]
pub(crate) struct MemoryLog {
    pub osc: Vec<Vec<u8>>,
    pub system: Vec<String>,
    /// OS のクリップボードを失敗させる。
    pub fail_system: bool,
}

#[cfg(test)]
impl Clipboard for Memory {
    fn osc52(&mut self, text: &str) -> Result<(), String> {
        let seqs = osc52_seqs(text, false)?;
        self.0.borrow_mut().osc.extend(seqs);
        Ok(())
    }

    fn system(&mut self, text: &str) -> Result<(), String> {
        let mut log = self.0.borrow_mut();
        if log.fail_system {
            return Err(Msg::ClipTestFailure.text().into());
        }
        log.system.push(text.to_string());
        Ok(())
    }
}

/// 既定の送り先。試験のビルドでは OS のクリップボードに書かない。
pub(crate) fn default_clipboard() -> Box<dyn Clipboard> {
    #[cfg(test)]
    {
        Box::new(Memory::default())
    }
    #[cfg(not(test))]
    {
        Box::new(SystemClipboard)
    }
}

/// OS のクリップボードのコマンドが応答するまで待つ長さ。
const CLIP_TIMEOUT: Duration = Duration::from_secs(2);

/// OS のクリップボードのコマンドに標準入力で渡す。macOS は pbcopy、ほかの Unix は wl-copy、
/// それが無いか失敗したら xclip。
#[cfg_attr(test, allow(dead_code))]
fn os_copy(text: &str) -> Result<(), String> {
    let cmds: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
        &[("pbcopy", &[])]
    } else if cfg!(unix) {
        &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"])]
    } else {
        &[]
    };
    let mut errs = Vec::new();
    for (prog, args) in cmds {
        match pipe_to(prog, args, text, CLIP_TIMEOUT) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                errs.push(Msg::ClipNoCommand.fill(&[&prog]))
            }
            Err(e) => errs.push(format!("{prog}: {e}")),
        }
    }
    if errs.is_empty() {
        return Err(Msg::ClipNoOsClipboard.text().into());
    }
    Err(errs.join(Msg::ReasonSep.text()))
}

/// 子のプロセスの標準入力に `text` を渡して終わりを待つ。書き込みは別のスレッドで行い、
/// `timeout` を過ぎたら子を kill して wait し、「応答しない」の誤りを返す(画面を固めない)。
/// 書き込みに失敗しても子は wait して残さない。
pub(crate) fn pipe_to(prog: &str, args: &[&str], text: &str, timeout: Duration) -> io::Result<()> {
    let mut child = Command::new(prog)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take();
    let data = text.as_bytes().to_vec();
    // 孫のプロセスが標準入力を持ったまま残ると書き込みが終わらないことがあるので、join は待たない。
    let writer = std::thread::spawn(move || match stdin {
        Some(mut s) => s.write_all(&data),
        None => Ok(()),
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    Msg::ClipTimeout.text(),
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e);
            }
        }
    };
    // 子が終わったあとの書き込みの結果(終わる前に読まずに閉じたら EPIPE)。少しだけ待つ。
    let wait_writer = Instant::now();
    while !writer.is_finished() && wait_writer.elapsed() < Duration::from_millis(200) {
        std::thread::sleep(Duration::from_millis(2));
    }
    if writer.is_finished() {
        if let Ok(Err(e)) = writer.join() {
            return Err(io::Error::new(e.kind(), Msg::ClipCannotPass.fill(&[&e])));
        }
    }
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(Msg::ClipExitCode.fill(&[&status])))
    }
}

/// base64(標準の字母、`=` で埋める)。
pub fn base64(bytes: &[u8]) -> String {
    const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(ABC[((n >> (18 - 6 * k)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// OSC 52 で送る base64 の長さの上限(端末の多くはこれより長い列を捨てる)。
pub const OSC52_MAX: usize = 100_000;

/// 送る列: `ESC ] 52 ; c ; <base64> BEL`。tmux の中ではアプリの OSC 52 が捨てられることがあるので、DCS のパススルー
/// (`ESC P tmux; ` + ESC を2重にした OSC 52 + `ESC \`)でも送る。base64 が上限を超えたら送らずに理由。
pub fn osc52_seqs(text: &str, tmux: bool) -> Result<Vec<Vec<u8>>, String> {
    let b64 = base64(text.as_bytes());
    if b64.len() > OSC52_MAX {
        return Err(Msg::Osc52TooLong.fill(&[&b64.len(), &OSC52_MAX]));
    }
    let plain = format!("\x1b]52;c;{b64}\x07");
    let mut out = vec![plain.clone().into_bytes()];
    if tmux {
        out.push(format!("\x1bPtmux;{}\x1b\\", plain.replace('\x1b', "\x1b\x1b")).into_bytes());
    }
    Ok(out)
}

/// `$EDITOR` の値を、シェルを通さずに POSIX の単語分けの一部で分ける: 空白で区切り、
/// `'…'`(中はそのまま)・`"…"`(中の `\` は `$` `` ` `` `"` `\` と改行の前だけ効く)・`\` のエスケープを解く。
/// 変数の展開・グロブ・リダイレクトはしない。閉じていない引用符は誤り。
pub fn split_words(s: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            ' ' | '\t' | '\n' => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            '\\' => match it.next() {
                Some('\n') => {}
                Some(n) => {
                    cur.push(n);
                    in_word = true;
                }
                None => return Err(Msg::WordsTrailingBackslash.text().into()),
            },
            '\'' => {
                in_word = true;
                loop {
                    match it.next() {
                        Some('\'') => break,
                        Some(n) => cur.push(n),
                        None => return Err(Msg::WordsUnclosedSingle.text().into()),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match it.next() {
                        Some('"') => break,
                        Some('\\') => match it.peek() {
                            Some(&n) if matches!(n, '$' | '`' | '"' | '\\' | '\n') => {
                                it.next();
                                if n != '\n' {
                                    cur.push(n);
                                }
                            }
                            _ => cur.push('\\'),
                        },
                        Some(n) => cur.push(n),
                        None => return Err(Msg::WordsUnclosedDouble.text().into()),
                    }
                }
            }
            _ => {
                cur.push(c);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(cur);
    }
    Ok(words)
}

/// エディタの値(`config::resolve_editor` で決めたもの)を単語に分け(`split_words`)、最後にノートのパスを1つの引数として足す(SR-8)。
/// シェルは通さない。空なら vi。
pub fn editor_argv(editor: &str, path: &Path) -> Result<Vec<OsString>, String> {
    let mut argv: Vec<OsString> = split_words(editor)?
        .into_iter()
        .map(OsString::from)
        .collect();
    if argv.is_empty() {
        argv.push("vi".into());
    }
    argv.push(path.as_os_str().to_owned());
    Ok(argv)
}

/// エディタを待つ間、親(mdgrid)で SIGINT と SIGQUIT を無視する(SR-8)。端末を戻したあとの Ctrl+C は
/// 前面のプロセスグループ全体に届くので、無視しないと mdgrid も Drop なしで終わり、ためた変更が消える。
/// 依存の crate を足さず、std が既にリンクしている libc の `signal` を直接呼ぶ。最後の Drop で元に戻す。
#[cfg(unix)]
pub(crate) mod sig {
    use std::os::raw::c_int;
    use std::sync::{Mutex, MutexGuard};

    extern "C" {
        fn signal(signum: c_int, handler: usize) -> usize;
    }

    /// Linux・macOS・BSD で共通の番号。
    pub const SIGINT: c_int = 2;
    pub const SIGQUIT: c_int = 3;
    const SIG_DFL: usize = 0;
    const SIG_IGN: usize = 1;
    const SIG_ERR: usize = usize::MAX;

    /// 無視の参照の数と、最初の無視の前の元の扱い。テストは並列に走り、無視が重なるので数で数える。
    /// 1つ目の `Ignore` で無視にして元を覚え、最後の `Ignore` の Drop でだけ元に戻す。
    static STATE: Mutex<(usize, Vec<(c_int, usize)>)> = Mutex::new((0, Vec::new()));

    fn state() -> MutexGuard<'static, (usize, Vec<(c_int, usize)>)> {
        STATE.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub struct Ignore(());

    impl Ignore {
        pub fn new() -> Ignore {
            let mut st = state();
            if st.0 == 0 {
                st.1.clear();
                for s in [SIGINT, SIGQUIT] {
                    // SAFETY: signal は任意のスレッドから呼べ、SIG_IGN は関数ポインタでない既定の値。
                    let old = unsafe { signal(s, SIG_IGN) };
                    if old != SIG_ERR {
                        st.1.push((s, old));
                    }
                }
            }
            st.0 += 1;
            Ignore(())
        }
    }

    impl Drop for Ignore {
        fn drop(&mut self) {
            let mut st = state();
            st.0 -= 1;
            if st.0 == 0 {
                for (s, old) in st.1.drain(..) {
                    // SAFETY: 前に signal が返した値(元の扱い)をそのまま戻す。
                    unsafe {
                        signal(s, old);
                    }
                }
            }
        }
    }

    /// 試験用: signal の今の扱いが無視かを読む(一度 SIG_IGN にして返った値をすぐ戻す)。
    /// 読む間に `Ignore` が扱いを変えないよう、同じ錠の中で読む。
    #[cfg(test)]
    pub fn is_ignored(s: c_int) -> bool {
        let _st = state();
        // SAFETY: 返った値(今の扱い)をすぐそのまま戻す。
        let cur = unsafe { signal(s, SIG_IGN) };
        unsafe {
            signal(s, cur);
        }
        cur == SIG_IGN
    }

    /// 子(exec の前)で SIGINT と SIGQUIT を既定に戻す。無視は exec を越えて引き継がれるため。
    /// fork のあとの子で呼ぶので、async-signal-safe な signal だけを使う。
    pub fn reset_in_child() -> std::io::Result<()> {
        for s in [SIGINT, SIGQUIT] {
            // SAFETY: signal は async-signal-safe。
            unsafe {
                signal(s, SIG_DFL);
            }
        }
        Ok(())
    }
}

/// エディタを起動して終わりを待つ。待つ間は親で SIGINT・SIGQUIT を無視し、子では既定に戻す。
pub(crate) fn run_editor(argv: &[OsString]) -> io::Result<std::process::ExitStatus> {
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: reset_in_child は async-signal-safe な signal だけを呼ぶ。
        unsafe {
            cmd.pre_exec(sig::reset_in_child);
        }
        let _ignore = sig::Ignore::new();
        cmd.status()
    }
    #[cfg(not(unix))]
    {
        cmd.status()
    }
}

/// TSV の1つの欄: タブと改行は空白にする。
fn tsv(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ")
}

impl App {
    /// コピーする値(ためた変更があればその値)。null とキーの無いセルは空。
    /// 計算の列(`file.*`・`formula.*`)は画面と同じ値(`Base::cell`。OUT-1)。
    fn copy_value(&self, row: &RowId, col: &str) -> String {
        tsv(&self.plain(row, col))
    }

    /// 選んだセルを、見出しつきのタブ区切り1行(`列の名前<TAB>値`)にする(OUT-1)。
    pub(crate) fn cell_tsv(&self) -> Option<String> {
        let (row, col) = self.selected()?;
        Some(format!("{}\t{}", tsv(&col), self.copy_value(&row, &col)))
    }

    /// 行を、見出しの行(ノートと列の名前)と値の行のタブ区切りにする(OUT-1)。
    /// `Y` は、行の選択(NV-5)があれば選んだ行を渡す。
    pub(crate) fn rows_tsv(&self, rows: &[RowId]) -> String {
        let mut lines = Vec::with_capacity(rows.len() + 1);
        let head: Vec<String> = std::iter::once(LABEL_HEADER.text().to_string())
            .chain(self.cols.iter().map(|c| tsv(c)))
            .collect();
        lines.push(head.join("\t"));
        for r in rows {
            let vals: Vec<String> = std::iter::once(tsv(&self.src.label(r)))
                .chain(self.cols.iter().map(|c| self.copy_value(r, c)))
                .collect();
            lines.push(vals.join("\t"));
        }
        lines.join("\n")
    }

    /// `y` と Ctrl+C: 選んだセルをコピーする。行の選択(NV-5)があれば、`Y` と同じく選んだ行を
    /// 見出しつきで(OUT-1)。
    pub(crate) fn copy_cell(&mut self) {
        let sel = self.selection();
        if !sel.is_empty() {
            let t = self.rows_tsv(&sel);
            return self.send_copy(&t, &Msg::CopyWhatSelected.fill(&[&sel.len()]));
        }
        match self.cell_tsv() {
            Some(t) => self.send_copy(&t, Msg::CopyWhatCell.text()),
            None => self.message = Some(Msg::CopyNoCell.text().into()),
        }
    }

    /// `Y`: 選んだ行(NV-5 の選択があればその全部、無ければ今の行)をコピーする。
    pub(crate) fn copy_row(&mut self) {
        let sel = self.selection();
        if !sel.is_empty() {
            let t = self.rows_tsv(&sel);
            return self.send_copy(&t, &Msg::CopyWhatSelected.fill(&[&sel.len()]));
        }
        match self.cur_row() {
            Some(r) => {
                let t = self.rows_tsv(&[r]);
                self.send_copy(&t, Msg::CopyWhatRow.text());
            }
            None => self.message = Some(Msg::CopyNoRow.text().into()),
        }
    }

    /// OSC 52 と OS のクリップボードの両方へ送る。片方が失敗しても止めず、メッセージ行で知らせる(OUT-1)。
    pub(crate) fn send_copy(&mut self, text: &str, what: &str) {
        let osc = self.clipboard.osc52(text);
        let sys = self.clipboard.system(text);
        self.message = Some(match (osc, sys) {
            (Ok(()), Ok(())) => Msg::CopiedBoth.fill(&[&what]),
            (Ok(()), Err(e)) => Msg::CopiedOscOnly.fill(&[&what, &e]),
            (Err(e), Ok(())) => Msg::CopiedOsOnly.fill(&[&what, &e]),
            (Err(a), Err(b)) => Msg::CopyFailed.fill(&[&a, &b]),
        });
    }

    /// `e`: 選んだ行をエディタで開くよう頼む(端末を持つ main が `open_editor` を呼ぶ)。
    pub(crate) fn request_editor(&mut self) {
        match self.cur_row() {
            Some(r) => self.editor_request = Some(r),
            None => self.message = Some(Msg::EditorNoRow.text().into()),
        }
    }

    /// エディタで開くよう頼まれているか。
    pub fn wants_editor(&self) -> bool {
        self.editor_request.is_some()
    }

    /// 頼まれた行のノートを `editor`(設定の editor・$VISUAL・$EDITOR から `config::resolve_editor` で決めた値)で開く(SR-8)。
    /// `term(true)` で端末を戻してから起動し、終わったら `term(false)` で画面に戻して、ノートを読み直す。
    /// 中身が変わっていれば外の変更と同じに扱う(ためた変更があれば「外で変更」の印。WB-16)。
    pub fn open_editor(&mut self, editor: &str, term: &mut dyn FnMut(bool) -> io::Result<()>) {
        let Some(row) = self.editor_request.take() else {
            return;
        };
        let path = PathBuf::from(&row.0);
        let argv = match editor_argv(editor, &path) {
            Ok(a) => a,
            Err(e) => {
                self.message = Some(Msg::EditorBadSetting.fill(&[&e]));
                return;
            }
        };
        let before = self.src.stamp(&row).map(|s| s.hash);
        if let Err(e) = term(true) {
            let _ = term(false);
            self.message = Some(Msg::EditorNoTerminal.fill(&[&e]));
            return;
        }
        let status = run_editor(&argv);
        let back = term(false);
        let label = self.src.label(&row);
        let mut msg = match &status {
            Err(e) => Msg::EditorCannotStart.fill(&[&argv[0].to_string_lossy(), &e]),
            Ok(s) if !s.success() => Msg::EditorFailed.fill(&[&s, &label]),
            Ok(_) => Msg::EditorReturned.fill(&[&label]),
        };
        if let Err(e) = back {
            msg.push_str(&Msg::EditorTermNotRestored.fill(&[&e]));
        }
        if status.is_ok() {
            match self.src.reload(&row) {
                Err(e) => msg.push_str(&Msg::EditorCannotReload.fill(&[&e])),
                Ok(()) => {
                    let after = self.src.stamp(&row).map(|s| s.hash);
                    if after != before {
                        let rows = std::slice::from_ref(&row);
                        self.changes.note_external(rows);
                        // WB-17: $EDITOR で、ためた値と同じ値に直したセルは外す。
                        self.changes.drop_same(self.src.as_ref(), rows);
                    }
                }
            }
            self.refresh();
        }
        self.message = Some(msg);
    }
}
