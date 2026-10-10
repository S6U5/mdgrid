#![cfg(unix)]
//! 端末に従うテーマ(SR-39)の配線: theme = "auto" で、問い合わせに答えない端末でも待ち続けずに表が出て、
//! キーをふつうに受ける(遅れた答えをキーと読まない)。

mod pty;

use pty::{TempDir, Tui};

#[test]
fn test_sr_39_e2e_auto_theme_without_answer_starts() {
    // [SR-39] 疑似端末は OSC 11 に答えない → 分からない(暗い地)として起動し、表が出る。
    let home = TempDir::new("sr39_auto");
    let notes = home.path().join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(notes.join("a.md"), "---\nstatus: todo\n---\n").unwrap();
    let cfg = home.path().join("auto.toml");
    std::fs::write(&cfg, "theme = \"auto\"\n").unwrap();
    let mut tui = Tui::spawn(
        &["--config", cfg.to_str().unwrap(), notes.to_str().unwrap()],
        home.path(),
        &[("COLORTERM", "truecolor")],
    );
    tui.wait_for("status");
    tui.send("q");
    let code = tui.wait_exit();
    assert_eq!(code, 0, "{}", tui.screen());
}
