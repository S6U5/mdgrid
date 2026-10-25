#![cfg(unix)]
//! 木の画面(TR-1・TR-2・TR-3・TR-4・TR-6)の、本物の実行ファイルでの確かめ(端末 80×24)。値は作ったもの。

mod pty;

use pty::{TempDir, Tui};

/// 印の [[table]] に `tree_line` を書いた根と、その下の表 `bm`。
fn root(tag: &str, tree_line: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        &format!("[[table]]\nname = \"BM\"\npath = \"bm\"\n{tree_line}\n"),
    );
    w(
        "bm/alpha.md",
        "---\nfolder: bar > 開発\norder: 20\nurl: https://example.com/alpha\n---\n",
    );
    w("bm/beta.md", "---\nfolder: bar > 開発\norder: 10\n---\n");
    w(
        "bm/gamma.md",
        "---\nfolder: bar > 開発 > Web\norder: 1\n---\n",
    );
    w("bm/delta.md", "---\nfolder: other\n---\n");
    w("bm/eps.md", "---\nfolder: \"\"\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

/// 行に `a` と `b` がこの順で、間に空白だけを挟んで並ぶか(木の名前と数)。
fn name_count(s: &str, name: &str, count: &str) -> bool {
    s.lines().any(|l| {
        l.find(name).is_some_and(|i| {
            let rest = l[i + name.len()..].trim_start();
            rest.starts_with(count)
                && !rest[count.len()..].starts_with(|c: char| c.is_ascii_digit())
        })
    })
}

#[test]
fn test_tr_1_tree_screen_counts_and_jump_e2e() {
    // [TR-1] T で木の画面。[TR-3] 左に bar(3)・開発(3)・Web(1)・other(1)・(なし)(1)。[TR-6] 80 桁では並べる。
    // [TR-4] 開発 の行は order の順(beta 10・alpha 20)。行で Enter → 表の画面でその行。
    let home = root(
        "tree-e2e",
        "tree = { path = \"folder\", order = \"order\" }",
    );
    let mut tui = Tui::spawn(&["bm"], home.path(), &[]);
    tui.wait_until("表が出る", |s| {
        s.contains("alpha") && s.contains("▦ 表 ▾")
    });
    tui.send("T");
    tui.wait_until("木の画面", |s| {
        s.contains("▤ 木 ▾") && s.contains("フォルダ") && s.contains("(なし)")
    });
    let s = tui.screen();
    for (name, n) in [
        ("▾ bar", "3"),
        ("▾ 開発", "3"),
        ("Web", "1"),
        ("other", "1"),
        ("(なし)", "1"),
    ] {
        assert!(name_count(&s, name, n), "{name} {n}:\n{s}");
    }
    // 80 桁でも左右に並べる(フォルダの窓と行の窓の題が同じ行)。
    assert!(
        s.lines()
            .any(|l| l.contains("フォルダ") && l.contains("行: ")),
        "{s}"
    );
    // 先頭の bar から 開発 へ、右の欄へ移って、先頭の行(beta)の次(alpha)で Enter。
    tui.send("k");
    tui.send("k");
    tui.send("j");
    tui.wait_until("開発 を選ぶ", |s| s.contains("行: bar > 開発"));
    let s = tui.screen();
    let beta = s.lines().position(|l| l.contains("beta")).unwrap();
    let alpha = s.lines().position(|l| l.contains("alpha.md")).unwrap();
    assert!(beta < alpha, "order の順:\n{s}");
    tui.send("\t");
    tui.send("k");
    tui.send("j");
    tui.send("\r");
    tui.wait_until("表の画面でその行", |s| {
        s.contains("▦ 表 ▾")
            && s.lines()
                .any(|l| l.trim_start().starts_with('>') && l.contains("alpha"))
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_tr_2_empty_separator_warns_e2e() {
    // [TR-2] separator = "" → 警告1行(CLI-3 の形)で、T は木を開かない。
    let home = root(
        "tree-e2e-sep",
        "tree = { path = \"folder\", separator = \"\" }",
    );
    let mut tui = Tui::spawn(&["bm"], home.path(), &[]);
    tui.wait_until("警告", |s| s.contains("table[1].tree.separator"));
    tui.send("T");
    tui.wait_until("木の設定が無い", |s| s.contains("tree = {"));
    assert!(!tui.screen().contains("▤ 木 ▾"));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
