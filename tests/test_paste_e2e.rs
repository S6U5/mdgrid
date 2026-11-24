#![cfg(unix)]
//! 端末の貼り付け(output.paste)を、本物の実行ファイルと疑似端末で確かめる。
//! 端末は、貼り付けの文字を ESC [ 200 ~ と ESC [ 201 ~ ではさんで送る(bracketed paste)。

use crate::pty;

use pty::{TempDir, Tui};

#[test]
fn test_paste_fills_cells_downward_e2e() {
    // [stest -> req~output.paste~2] 2行の値を貼ると、選んだセルから下の2行の値が替わり、ためる変更になる。
    let home = TempDir::new("paste-e2e");
    let h = home.path();
    for d in ["notes", "config", "state"] {
        std::fs::create_dir_all(h.join(d)).unwrap();
    }
    std::fs::write(h.join("notes/a.md"), "---\nstatus: s1\n---\n").unwrap();
    std::fs::write(h.join("notes/b.md"), "---\nstatus: s2\n---\n").unwrap();
    let mut tui = Tui::spawn(&["notes"], h, &[]);
    tui.wait_until("表が出る", |s| s.contains("s1") && s.contains("s2"));
    tui.send("\x1b[200~pasted1\npasted2\x1b[201~");
    tui.wait_until("貼った値が出る", |s| {
        s.contains("pasted1") && s.contains("pasted2")
    });
    assert!(!tui.screen().contains("s1"), "{}", tui.screen());
    // 保存していないので、ファイルは変わらない。
    assert_eq!(
        std::fs::read_to_string(h.join("notes/a.md")).unwrap(),
        "---\nstatus: s1\n---\n"
    );
}
