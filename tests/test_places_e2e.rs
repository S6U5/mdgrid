#![cfg(unix)]
//! 登録した表(CLI-1・CLI-19)の本物の実行ファイルでの確かめ: 引数なしで起動すると一覧が重なり、
//! 選んだ表で開き直す。

mod pty;

use pty::{TempDir, Tui};

#[test]
fn test_cli_19_no_args_lists_and_switches() {
    let home = TempDir::new("places-e2e");
    let h = home.path();
    std::fs::create_dir_all(h.join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(h.join("state")).unwrap();
    std::fs::create_dir_all(h.join("shelf_vault")).unwrap();
    std::fs::write(h.join("here.md"), "---\nstatus: todo\n---\n").unwrap();
    std::fs::write(
        h.join("shelf_vault/novel.md"),
        "---\nauthor: someone\n---\n",
    )
    .unwrap();
    std::fs::write(
        h.join("config/mdgrid/places.toml"),
        "[[place]]\nname = \"本棚\"\ngroup = \"趣味\"\npath = \"~/shelf_vault\"\n",
    )
    .unwrap();
    let mut tui = Tui::spawn(&[], h, &[]);
    tui.wait_for("今のフォルダ");
    tui.wait_for("趣味 › 本棚");
    tui.send("本棚");
    tui.send("\r");
    tui.wait_until("開き直した表", |s| {
        s.contains("shelf_vault") && s.contains("novel")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_cli_1_no_places_opens_cwd() {
    // [CLI-1] 登録が無ければ、今までどおり今のフォルダを開く(一覧は出さない)。
    let home = TempDir::new("places-e2e-none");
    let h = home.path();
    std::fs::create_dir_all(h.join("config")).unwrap();
    std::fs::write(h.join("here.md"), "---\nstatus: todo\n---\n").unwrap();
    let mut tui = Tui::spawn(&[], h, &[]);
    tui.wait_for("here");
    assert!(!tui.screen().contains("今のフォルダ"));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
