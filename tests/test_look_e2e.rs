#![cfg(unix)]
//! SR-43 の受け入れ(本物の実行ファイル): 設定の置き場の look.toml の見た目を、起動のとき config.toml に重ねる。
//! 組 slate はリストを `#a` の形で見せる(SR-36)ので、画面の文字で組が効いたと分かる。壊れた look.toml は警告。

use crate::pty;

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
    // [stest -> req~screen.look-section-apply~2, req~screen.look-section-save~2, req~screen.look-section-preset~1] look.toml の preset = "slate" → リストが #red の形。
    let h = home("look-slate", "preset = \"slate\"\n");
    let mut t = Tui::spawn(&["vault"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("#red");
    t.send("q");
    pty::assert_gone(t.stop());
}

#[test]
fn test_sr_43_broken_look_toml_warns() {
    // [stest -> req~screen.look-section-save~2] 壊れた look.toml → 警告して config.toml のとおりに起動(リストは既定の組の形)。
    let h = home("look-broken", "preset = \n");
    let mut t = Tui::spawn(&["vault"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("look.toml が読めない");
    assert!(!t.screen().contains("#red"), "{}", t.screen());
    t.send("q");
    pty::assert_gone(t.stop());
}

/// 印に `[look] theme = "dozy-pink"` を書いた台帳 notes(表 tasks)と、空の設定の置き場。
fn marker_home(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let notes = home.path().join("notes");
    std::fs::create_dir_all(notes.join("tasks")).unwrap();
    std::fs::create_dir_all(notes.join(".mdgrid")).unwrap();
    std::fs::create_dir_all(home.path().join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(home.path().join("state")).unwrap();
    std::fs::write(
        notes.join(".mdgrid/workspace.toml"),
        "name = \"N\"\n[look]\ntheme = \"dozy-pink\"\n",
    )
    .unwrap();
    std::fs::write(notes.join("tasks/a.md"), "---\nstatus: todo\n---\n").unwrap();
    home
}

/// dozy-pink と sumi の地の色。
const DOZY_BG: [u8; 3] = [0xfb, 0xf1, 0xe1];
const SUMI_BG: [u8; 3] = [0x16, 0x17, 0x1b];

/// ビューの設定を開き、「見た目」の区画(8)のテーマの行の選び手を開く。
fn open_theme_pick(t: &mut Tui) {
    t.send("o");
    t.wait_for("見た目");
    t.send("8");
    t.wait_for("保存先");
    t.send("j");
    t.send("\r");
    t.wait_for("テーマを選ぶ");
}

/// 「反映」を押して表に戻る。
fn apply(t: &mut Tui) {
    t.send("\t");
    t.send("\r");
    t.wait_until("表に戻る", |s| !s.contains("保存先"));
}

#[test]
fn test_marker_look_yields_to_global_choice() {
    // [stest -> req~screen.profile-scopes-marker-look~1, req~screen.look-section-save~2, req~screen.look-section-apply~2] 印に dozy-pink を書いた表で、見た目の区画の保存先「全体」のまま sumi を選んで反映 → 画面の地が
    // sumi に変わり、ui.toml に theme = "sumi"。開き直しても sumi(印の dozy-pink は勝たない)。
    let h = marker_home("look-marker");
    let mut t = Tui::spawn(&["notes/tasks"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("todo");
    t.wait_color(DOZY_BG, true);
    open_theme_pick(&mut t);
    // テーマの並びは auto・default・nord・solarized-light・dracula・gruvbox・pink-monster・dozy-pink・sumi…。
    // 選び手は今のテーマ(dozy-pink)の行で開くので、1つ下が sumi。
    t.send("j");
    t.send("\r");
    t.wait_for("▾ sumi");
    apply(&mut t);
    t.wait_color(SUMI_BG, true);
    t.wait_color(DOZY_BG, false);
    let ui = h.path().join("config/mdgrid/ui.toml");
    pty::wait_file(&ui, &t, |s| s.contains("theme = \"sumi\""));
    t.send("q");
    pty::assert_gone(t.stop());
    let mut t = Tui::spawn(&["notes/tasks"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("todo");
    t.wait_color(SUMI_BG, true);
    t.wait_color(DOZY_BG, false);
    t.send("q");
    pty::assert_gone(t.stop());
}

#[test]
fn test_custom_accent_applies_and_saves() {
    // [stest -> req~screen.look-section-custom~1, req~screen.look-section-custom-save~1] 印に dozy-pink を書いた表で、テーマの一覧の最後の「カスタム」を選び、アクセントの行で
    // #e0a458 を打って反映 → 画面にその色が出て、ui.toml に元のテーマ dozy-pink と accent = "#e0a458"。
    let h = marker_home("look-custom");
    let mut t = Tui::spawn(&["notes/tasks"], h.path(), &[("COLORTERM", "truecolor")]);
    t.wait_for("todo");
    open_theme_pick(&mut t);
    // 一覧の最後まで下りる(カスタムは最後)。
    t.send(&"j".repeat(20));
    t.send("\r");
    t.wait_for("カスタム(元: dozy-pink)");
    // 項目は 保存先・テーマ・組・丸い札の端・元のテーマ・地・文字・1行目・選んだ行の地・選んだ行の文字・下の帯の地・
    // 下の帯の文字・アクセント…。テーマの行から11下がアクセント。
    t.send(&"j".repeat(11));
    t.send("\r");
    t.wait_for("アクセントの色を選ぶ");
    // 見本の色の一覧の最後が「#rrggbb を打つ」。
    t.send(&"j".repeat(20));
    t.send("\r");
    t.wait_for("アクセントの色(");
    t.send("#e0a458\r");
    t.wait_for("#e0a458");
    apply(&mut t);
    t.wait_color([0xe0, 0xa4, 0x58], true);
    let ui = h.path().join("config/mdgrid/ui.toml");
    let text = pty::wait_file(&ui, &t, |s| s.contains("accent = \"#e0a458\""));
    assert!(text.contains("theme = \"dozy-pink\""), "{text}");
    t.send("q");
    pty::assert_gone(t.stop());
}
