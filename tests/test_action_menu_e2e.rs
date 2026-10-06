#![cfg(unix)]
//! 変更 2026-10-05-action-menu の受け入れ(SR-24 のセルの右クリック)。
//! App にまだ右クリックの口が無いので、本物の実行ファイルを疑似端末(80×24)で動かし、
//! 端末のマウスの報告(SGR 1006 の形。右のボタンは 2)を送って確かめる。実装を見ずに書いた。
//!
//! - 疑似端末の道具は tests/pty/mod.rs。子の言語は日本語(道具の既定)。
//! - 一覧が開いたことは、中身が「セル」だけの行(節の見出し)が出たことで見る。
//! - そのセルが選ばれたことは、Esc で閉じたあとの下の帯の位置(「表 2/3行 2/2列」)で見る。

mod pty;

use pty::{assert_gone, TempDir, Tui};

/// 画面の行の、文字の位置 `i` までの桁(幅2の文字を2桁と数える)。
fn column_of(line: &str, i: usize) -> usize {
    line.chars()
        .take(i)
        .map(|c| if c.is_ascii() { 1 } else { 2 })
        .sum()
}

/// 枠と空白を除いた中身が `name` だけの行がある。
fn has_section(screen: &str, name: &str) -> bool {
    screen.lines().any(|l| {
        l.trim_matches(|c: char| c.is_whitespace() || "│┃|─━┌┐└┘╭╮╰╯├┤".contains(c)) == name
    })
}

#[test]
fn test_sr_24_e2e_right_click_selects_cell_and_opens_menu() {
    // [SR-24] 選んでいない行(beta.md)の status のセルを右クリック → そのセルが選ばれ、一覧が開く。
    // Esc で閉じると、選んだ位置は 2行目・2列目(status)。
    let home = TempDir::new("sr24-right");
    let v = home.path().join("vault");
    for d in ["config", "state"] {
        std::fs::create_dir_all(home.path().join(d)).unwrap();
    }
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(v.join("alpha.md"), "---\ntitle: x\nstatus: todo\n---\n").unwrap();
    std::fs::write(v.join("beta.md"), "---\ntitle: y\nstatus: done\n---\n").unwrap();
    std::fs::write(v.join("gamma.md"), "---\ntitle: z\nstatus: wait\n---\n").unwrap();
    let mut t = Tui::spawn(&[v.to_str().unwrap()], home.path(), &[]);
    t.wait_for("gamma"); // SR-29: 左の欄は `.md` を除いた名前
    t.wait_for("1/3行");

    let s = t.screen();
    let lines: Vec<&str> = s.lines().collect();
    let head = lines
        .iter()
        .position(|l| l.contains("status") && l.contains("title"))
        .expect("列の見出しの行");
    let i = lines[head].find("status").unwrap();
    let x = column_of(lines[head], lines[head][..i].chars().count());
    let y = lines
        .iter()
        .position(|l| l.contains("beta"))
        .expect("beta の行");
    // SGR の位置は 1 始まり。押して離す。
    t.send(&format!("\x1b[<2;{};{}M", x + 1, y + 1));
    t.send(&format!("\x1b[<2;{};{}m", x + 1, y + 1));
    t.wait_until("右クリックで開いた一覧(節「セル」)", |s| {
        has_section(s, "セル")
    });

    t.send("\x1b");
    t.wait_until("Esc で閉じた表の位置 2/3行 2/2列", |s| {
        !has_section(s, "セル") && s.contains("2/3行") && s.contains("2/2列")
    });
    assert_gone(t.stop());
}
