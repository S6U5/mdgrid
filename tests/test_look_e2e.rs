#![cfg(unix)]
//! SR-43 の受け入れ(本物の実行ファイル): 設定の置き場の look.toml の見た目を、起動のとき config.toml に重ねる。
//! 組 slate はリストを `#a` の形で見せる(SR-36)ので、画面の文字で組が効いたと分かる。壊れた look.toml は警告。

mod pty;

use pty::{TempDir, Tui};

fn home(tag: &str, look: &str) -> TempDir {
    let home = TempDir::new(tag);
    let v = home.path().join("vault");
    std::fs::create_dir_all(&v).unwrap();
    std::fs::create_dir_all(home.path().join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(home.path().join("state")).unwrap();
    std::fs::write(v.join("a.md"), "---\ntags: [red, blue]\n---\n").unwrap();
    std::fs::write(v.join("b.md"), "---\ntags: [red]\n---\n").unwrap();
    std::fs::write(home.path().join("config/mdgrid/look.toml"), look).unwrap();
    home
}

#[test]
fn test_sr_43_look_toml_applies_at_start() {
    // [SR-43] look.toml の preset = "slate" → リストが #red の形。
    let h = home("look-slate", "preset = \"slate\"\n");
    let mut t = Tui::spawn(&["vault"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("#red");
    t.send("q");
    pty::assert_gone(t.stop());
}

#[test]
fn test_sr_43_broken_look_toml_warns() {
    // [SR-43] 壊れた look.toml → 警告して config.toml のとおりに起動(リストは既定の組の形)。
    let h = home("look-broken", "preset = \n");
    let mut t = Tui::spawn(&["vault"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("look.toml が読めない");
    assert!(!t.screen().contains("#red"), "{}", t.screen());
    t.send("q");
    pty::assert_gone(t.stop());
}
