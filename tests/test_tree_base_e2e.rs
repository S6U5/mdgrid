#![cfg(unix)]
//! TR-2: 印の `[table.base]`(WS-9。`path` の無い表)で、根を `mdgrid .` で開いても木の画面が出る(端末 80×24)。
//! 値は作ったもの。

mod pty;

use pty::{TempDir, Tui};

#[test]
fn test_tr_2_base_only_marker_table_tree_e2e() {
    // [TR-2] open = "ブックマーク" と [table.base] だけの表 → T で木の画面。
    let home = TempDir::new("tree-base-e2e");
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        "open = \"ブックマーク\"\n\n[[table]]\nname = \"ブックマーク\"\n\
         tree = { path = \"folder\", separator = \" > \", order = \"order\" }\n\n[table.base]\n\
         filters = 'file.inFolder(\"items\")'\n\
         views = [{ type = \"table\", name = \"全部\" }]\n",
    );
    w(
        "items/alpha.md",
        "---\nfolder: bar > 開発\norder: 10\n---\n",
    );
    w("items/beta.md", "---\nfolder: other\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    let mut tui = Tui::spawn(&["."], h, &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾")
    });
    tui.send("T");
    tui.wait_until("木の画面", |s| {
        s.contains("▤ 木 ▾") && s.contains("▾ bar") && s.contains("other")
    });
    tui.send("q");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
