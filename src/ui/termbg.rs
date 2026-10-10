//! 端末の地の明るさ(SR-39): `COLORFGBG` を見て、無ければ端末に問い合わせる(OSC 11)。答えを短く待ち、
//! 無ければ諦める(分からない)。問い合わせのあとに装置の属性の問い合わせ(DA1)も送り、その答えが先に
//! 来たら OSC 11 に答えない端末とみなしてすぐやめる(待ち時間を使い切らず、遅れた答えをキーと読まない)。

use std::time::Duration;

/// 地が明るいか。分からなければ None。
pub(crate) fn light(env: impl Fn(&str) -> Option<String>, wait: Duration) -> Option<bool> {
    if let Some(v) = env("COLORFGBG") {
        if let Some(l) = mdgrid::theme::light_from_colorfgbg(&v) {
            return Some(l);
        }
    }
    query(wait)
}

#[cfg(unix)]
fn query(wait: Duration) -> Option<bool> {
    use ratatui::crossterm::terminal;
    use std::io::{Read, Write};
    use std::os::raw::{c_int, c_short};
    use std::os::unix::io::AsRawFd;
    use std::time::Instant;

    #[repr(C)]
    struct PollFd {
        fd: c_int,
        events: c_short,
        revents: c_short,
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    type Nfds = std::os::raw::c_ulong;
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    type Nfds = std::os::raw::c_uint;
    extern "C" {
        fn poll(fds: *mut PollFd, nfds: Nfds, timeout: c_int) -> c_int;
    }
    /// Linux・macOS・BSD で共通の値。
    const POLLIN: c_short = 1;

    let mut tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;
    let was_raw = terminal::is_raw_mode_enabled().unwrap_or(false);
    if !was_raw {
        terminal::enable_raw_mode().ok()?;
    }
    let got = (|| {
        tty.write_all(b"\x1b]11;?\x1b\\\x1b[c").ok()?;
        tty.flush().ok()?;
        let fd = tty.as_raw_fd();
        let end = Instant::now() + wait;
        let mut buf = Vec::new();
        loop {
            let left = end.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            let mut p = PollFd {
                fd,
                events: POLLIN,
                revents: 0,
            };
            // SAFETY: p は1つの有効な pollfd で、呼んでいる間だけ借りる。
            let n = unsafe { poll(&mut p, 1, left.as_millis().clamp(1, 1000) as c_int) };
            if n <= 0 {
                break;
            }
            let mut b = [0u8; 256];
            let k = tty.read(&mut b).ok()?;
            if k == 0 {
                break;
            }
            buf.extend_from_slice(&b[..k]);
            // DA1 の答え(ESC [ ? … c)が来たら、OSC 11 の答えはもう来ない。
            let s = String::from_utf8_lossy(&buf);
            if let Some(i) = s.find("\x1b[?") {
                if s[i..].contains('c') {
                    break;
                }
            }
        }
        mdgrid::theme::light_from_osc11(&String::from_utf8_lossy(&buf))
    })();
    if !was_raw {
        let _ = terminal::disable_raw_mode();
    }
    got
}

#[cfg(not(unix))]
fn query(_wait: Duration) -> Option<bool> {
    None
}
