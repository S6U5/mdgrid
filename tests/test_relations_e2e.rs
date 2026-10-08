#![cfg(unix)]
//! リレーション(REL-2・REL-4)の本物の実行ファイルでの確かめ: リンクを名前で見せ、行き先を開くと
//! 行き先の表で開き直してその行を選ぶ。

mod pty;

use pty::{TempDir, Tui};

#[test]
fn test_rel_4_open_target_switches_table_and_selects_row() {
    let home = TempDir::new("rel-e2e");
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w("projects/alpha.md", "---\nstatus: idea\n---\n");
    w("projects/mdgrid.md", "---\nstatus: active\n---\n");
    w("projects/zeta.md", "---\nstatus: idea\n---\n");
    w("tasks/build.md", "---\nproject: \"[[mdgrid]]\"\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    let projects = h.join("projects").canonicalize().unwrap();
    w(
        "config/mdgrid/places.toml",
        &format!(
            "[[place]]\nname = \"Projects\"\npath = \"{}\"\n",
            projects.display()
        ),
    );
    let mut tui = Tui::spawn(&["tasks"], h, &[]);
    tui.wait_until("リンクを名前で見せる", |s| {
        s.contains("build") && s.contains("mdgrid") && !s.contains("[[mdgrid]]")
    });
    // project の列(先頭のキーの列)で、パレットから行き先を開く。
    tui.send(":open_link");
    tui.send("\r");
    tui.wait_until("行き先の表でその行を選ぶ", |s| {
        s.contains("projects")
            && s.lines()
                .any(|l| l.starts_with('>') && l.contains("mdgrid"))
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
