#![cfg(unix)]
//! 変更 2026-10-15-horizontal-wheel の受け入れ(SR-6 の横のホイール)。
//! 本物の実行ファイルを疑似端末(80×24)で動かし、端末のマウスの報告(SGR 1006 の形)を送って、
//! 下の帯の列の位置で確かめる。横のホイールの右は 67、左は 66、縦の下は 65 に shift(+4)で 69。

mod pty;

use pty::{assert_gone, TempDir, Tui};

#[test]
fn test_sr_6_hwheel_e2e_terminal_reports() {
    // [SR-6] 横のホイールの右 → 2列目、左 → 1列目、shift+ホイールの下 → 2列目。
    let home = TempDir::new("sr6-hwheel");
    let v = home.path().join("vault");
    for d in ["config", "state"] {
        std::fs::create_dir_all(home.path().join(d)).unwrap();
    }
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("alpha.md"), "---\ntitle: x\nstatus: todo\n---\n").unwrap();
    std::fs::write(v.join("beta.md"), "---\ntitle: y\nstatus: done\n---\n").unwrap();
    let mut t = Tui::spawn(&[v.to_str().unwrap()], home.path(), &[]);
    t.wait_for("1/2行");
    t.wait_for("1/2列");

    t.send("\x1b[<67;20;10M");
    t.wait_until("横のホイールの右で 2/2列", |s| {
        s.contains("2/2列")
    });
    t.send("\x1b[<66;20;10M");
    t.wait_until("横のホイールの左で 1/2列", |s| {
        s.contains("1/2列")
    });
    t.send("\x1b[<69;20;10M");
    t.wait_until(
        "shift+ホイールの下で 2/2列、行はそのまま",
        |s| s.contains("2/2列") && s.contains("1/2行"),
    );
    assert_gone(t.stop());
}
