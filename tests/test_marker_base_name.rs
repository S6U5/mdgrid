#![cfg(unix)]
//! [WS-9] 印の表に中身を書いた表は、ヘッダーに表の名前だけを出す(無い `.base` のファイルの名前を見せない)。

mod pty;

use pty::{TempDir, Tui};

#[test]
fn test_ws_9_marker_base_header_shows_table_name() {
    // [WS-9] `[[table]] name = "台帳"` と `[table.base]` の印 → ヘッダーに「台帳」が出て、「台帳.base」は出ない。
    let t = TempDir::new("mb-name");
    let root = t.path();
    for d in ["todo", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(
        root.join("todo/a.md"),
        "---\ntitle: 一つめ\nstatus: todo\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".mdgrid/workspace.toml"),
        "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\n\n[table.base]\nfilters = { or = ['file.inFolder(\"todo\")'] }\n\n[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"title\", \"status\"]\n",
    )
    .unwrap();
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("印の表", |s| s.contains("一つめ") && s.contains("台帳"));
    let s = tui.screen();
    assert!(
        !s.contains("台帳.base"),
        "ヘッダーに無いファイルの名前が出ている:\n{s}"
    );
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
