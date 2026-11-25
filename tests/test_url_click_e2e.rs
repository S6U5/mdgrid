#![cfg(unix)]
//! 選んでいる URL のもう一度のクリックで開く(screen.url-click-open・tree.url-click-open)の、本物の実行ファイルでの
//! 確かめ(端末 80×24)。値は作ったもの。本物のブラウザは開かない: PATH の先頭に偽の開くコマンド(macOS の `open`、
//! ほかの `xdg-open`)を置き、受け取った引数をファイルに書かせる。字の位置は決め打ちせず、画面の文字で探す。

use crate::pty;

use pty::{assert_gone, TempDir, Tui, LIMIT};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthStr;

/// 列の幅で途中が切れて見える長い URL(問い合わせと `#` を含む)。
const LONG: &str = "https://docs.example.com/guides/workspace-setup/advanced-configuration/a-very-long-page-name-to-check-whether-a-cut-off-link-still-opens-in-full?from=mytestgrid&section=long-url-check#open-the-whole-link";

/// 偽の開くコマンドを `home/bin` に置き、そのフォルダを先頭にした PATH と、引数を書くログのパスを返す。
fn fake_opener(home: &Path) -> (String, PathBuf) {
    let bin = home.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let log = home.join("opened.log");
    let tmp = home.join("opened.log.tmp");
    let script = format!(
        "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\"; done > '{}'\nmv '{}' '{}'\n",
        tmp.display(),
        tmp.display(),
        log.display()
    );
    use std::os::unix::fs::PermissionsExt;
    for name in ["open", "xdg-open"] {
        let p = bin.join(name);
        std::fs::write(&p, &script).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
    );
    (path, log)
}

/// 開くコマンドが呼ばれるまで待ち(上限 10 秒)、受け取った引数を返す。
fn wait_opened(log: &Path, tui: &Tui) -> Vec<String> {
    let start = Instant::now();
    while !log.exists() {
        assert!(
            start.elapsed() <= LIMIT,
            "開くコマンドが呼ばれない。画面:\n{}",
            tui.screen()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    std::fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

/// 画面の中で `needle` が出る最初の所(桁と行。0 始まり)。
fn find(s: &str, needle: &str) -> Option<(usize, usize)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(needle).map(|i| (l[..i].width(), y)))
}

/// 桁 x・行 y(0 始まり)の左クリック(SGR 1006 の押す・離す)。
fn click(tui: &mut Tui, x: usize, y: usize) {
    tui.send(&format!("\x1b[<0;{};{}M", x + 1, y + 1));
    tui.send(&format!("\x1b[<0;{};{}m", x + 1, y + 1));
}

/// 印の [[table]] に `line` を書いた根と、その下の表 `t`(長い URL の行 long と、URL の無い行 memo)。
fn root(tag: &str, line: &str) -> TempDir {
    let home = TempDir::new(tag);
    let h = home.path();
    let w = |rel: &str, text: &str| {
        let p = h.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        ".mdgrid/workspace.toml",
        &format!("[[table]]\nname = \"T\"\npath = \"t\"\n{line}\n"),
    );
    w(
        "t/long.md",
        &format!("---\nfolder: bar\nurl: {LONG}\nstatus: todo\n---\n"),
    );
    w("t/memo.md", "---\nfolder: bar\nstatus: done\n---\n");
    std::fs::create_dir_all(h.join("state")).unwrap();
    home
}

#[test]
fn test_url_cell_click_twice_opens_full_url_e2e() {
    // [stest -> req~screen.url-click-open~1, req~screen.url-click-hint~1] 切って見せた長い URL のセルのクリックで、そのセルを選んで案内を出し(開かない)、もう一度のクリックで切る前の全体の URL を開くコマンドに渡す。
    let home = root("url-click-e2e-table", "");
    let (path, log) = fake_opener(home.path());
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[("PATH", &path)]);
    tui.wait_until("表が出る", |s| s.contains("long") && s.contains("memo"));
    let s = tui.screen();
    let (x, y) = find(&s, "https://docs").unwrap_or_else(|| panic!("URL のセル:\n{s}"));
    assert!(!s.contains(LONG), "URL が切れて見えていない:\n{s}");
    click(&mut tui, x + 2, y);
    tui.wait_for("URL: もう一度クリックか O で開く");
    std::thread::sleep(Duration::from_millis(300));
    assert!(!log.exists(), "1回目のクリックで開いた");
    click(&mut tui, x + 2, y);
    assert_eq!(wait_opened(&log, &tui), vec![LONG.to_string()]);
    tui.wait_for("開く: https://docs.example.com/guides");
    // 編集は始まらない(下の帯のモードは表のまま)。
    assert!(tui.screen().contains("未保存 0  表 "), "{}", tui.screen());
    assert_gone(tui.stop());
}

#[test]
fn test_tree_url_click_twice_opens_full_url_e2e() {
    // [stest -> req~tree.url-click-open~1, req~screen.url-click-hint~1] 木の画面の右の欄の URL の値のクリックでその行を選び(開かない)、もう一度のクリックで木の画面のまま全体の URL を開くコマンドに渡す。
    let home = root("url-click-e2e-tree", "tree = { path = \"folder\" }");
    let (path, log) = fake_opener(home.path());
    let mut tui = Tui::spawn(&["t", "--no-color"], home.path(), &[("PATH", &path)]);
    tui.wait_until("表が出る", |s| s.contains("long") && s.contains("memo"));
    tui.send("T");
    tui.wait_for("╭─ 行: bar");
    // 右の欄の memo を選んでおく(名前の順で long・memo。木を開いたときは表で選んでいた行)。
    let s = tui.screen();
    let (x, y) = find(&s, "https://docs").unwrap_or_else(|| panic!("URL の値:\n{s}"));
    let (mx, my) = find(&s, "memo").unwrap_or_else(|| panic!("memo:\n{s}"));
    click(&mut tui, mx + 1, my);
    tui.wait_until("memo を選ぶ", |s| s.contains(">memo"));
    click(&mut tui, x + 2, y);
    tui.wait_until("long を選ぶ", |s| s.contains(">long"));
    tui.wait_for("URL: もう一度クリックか o で開く");
    std::thread::sleep(Duration::from_millis(300));
    assert!(!log.exists(), "1回目のクリックで開いた");
    click(&mut tui, x + 2, y);
    assert_eq!(wait_opened(&log, &tui), vec![LONG.to_string()]);
    tui.wait_for("開く: https://docs.example.com/guides");
    assert!(
        tui.screen().contains("╭─ 行: bar"),
        "木の画面のまま:\n{}",
        tui.screen()
    );
    assert_gone(tui.stop());
}
