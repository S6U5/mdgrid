#![cfg(unix)]
//! [WS-9][WS-8] 印に中身を書いた表へ、引き出しでほかの表から戻れる。

mod pty;

use pty::{TempDir, Tui};

/// 台帳(中身を印に書いた表)とルーチン(フォルダの表)のある見本。
fn sample(root: &std::path::Path) {
    for d in ["todo", "routine", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(
        root.join("todo/a.md"),
        "---\ntitle: 一つめ\nstatus: todo\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("routine/r.md"), "---\ntitle: 毎週の点検\n---\n").unwrap();
    std::fs::write(
        root.join(".mdgrid/workspace.toml"),
        "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\n\n[table.base]\nfilters = { or = ['file.inFolder(\"todo\")'] }\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"title\", \"status\"]\n\n[[table]]\nname = \"ルーチン\"\npath = \"routine\"\n",
    )
    .unwrap();
}

#[test]
fn test_ws_9_switch_back_to_marker_base_table() {
    // [WS-9][WS-8] 台帳 → 引き出しでルーチン → 引き出しで台帳に戻ると、台帳の行が出る。
    let t = TempDir::new("mb-switch");
    let root = t.path();
    sample(root);
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("台帳", |s| s.contains("一つめ"));
    tui.send("w");
    tui.wait_until("引き出し", |s| s.contains("ルーチン"));
    tui.send("\x1b[B\r");
    tui.wait_until("ルーチンの表", |s| {
        s.contains("毎週の点検") && !s.contains("一つめ")
    });
    tui.send("w");
    tui.wait_until("引き出し", |s| s.contains("台帳"));
    tui.send("\x1b[A\r");
    tui.wait_until("台帳に戻る", |s| {
        s.contains("一つめ") && !s.contains("毎週の点検")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
