#![cfg(unix)]
//! 表のほかのモード(docs/keys.md の edit・review・quit・palette・help・search・filter・detail・settings・
//! settings_input・chips・list_select・freq・sorts・drawer・filter_window・filter_input・screen_menu・
//! column_type)を、本物の実行ファイルで1つずつキー(とクリック)で操作し、ノートのファイルか画面が要求どおりに
//! 変わることを確かめる(疑似端末 80×24)。期待値は specs/requirements の受け入れの例から取った。
//!
//! - 行の名前は拡張子なしで出る。行の選びは行の頭の `>`、位置は下の帯の「2/3行」「4/4列」で見る。
//! - 起動の直後の1打は、端末への問い合わせに吸われることがあるので、表が出たら、何度押しても同じになる `$`(末尾の
//!   列)を効くまで押し直し、`0` で先頭の列に戻してから操作を始める(開く・閉じるのキーを2回押さないように)。
//! - Esc のあとは、効いたのを待ってから次のキーを送る(続けて送ると Alt+キーと読まれる)。
//! - 子は Drop で止まる(落ちても残らない)。

use crate::pty;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use pty::{assert_gone, wait_file, TempDir, Tui, LIMIT};
use unicode_width::UnicodeWidthStr;

const ESC: &str = "\x1b";
const ENTER: &str = "\r";
const TAB: &str = "\t";
const BACKTAB: &str = "\x1b[Z";
const UP: &str = "\x1b[A";
const DOWN: &str = "\x1b[B";
const BACKSPACE: &str = "\x7f";
const CTRL_P: &str = "\x10";
const CTRL_S: &str = "\x13";

// ---------------------------------------------------------------- 場

const ALPHA: &str =
    "---\ntitle: 会議の準備\nstatus: todo\npriority: 3\ntags:\n  - 会議\n---\n本文の一行目\n二行目\n";
const BETA: &str = "---\ntitle: 買い物\nstatus: done\npriority: 1\ntags:\n  - 家\n---\n";
const GAMMA: &str =
    "---\ntitle: 会議の記録\nstatus: doing\npriority: 2\ntags:\n  - 会議\n  - 家\n---\n";
/// 名前の順に alpha(todo・3・会議・本文あり)・beta(done・1・家)・gamma(doing・2・会議と家)。
/// 列は title・status・priority・tags の順。
const NOTES: [(&str, &str); 3] = [("alpha.md", ALPHA), ("beta.md", BETA), ("gamma.md", GAMMA)];
const NAMES: [&str; 4] = ["alpha", "beta", "delta", "gamma"];

/// HOME の下に設定と状態の置き場と notes のノートを置く。
fn home(tag: &str, notes: &[(&str, &str)]) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "notes"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    for (name, text) in notes {
        std::fs::write(h.path().join("notes").join(name), text).unwrap();
    }
    h
}

fn note(h: &TempDir, name: &str) -> PathBuf {
    h.path().join("notes").join(name)
}

fn read(h: &TempDir, name: &str) -> String {
    std::fs::read_to_string(note(h, name)).unwrap()
}

/// notes を開き、表(`rows` 行・`cols` 列)が出たら、キーが吸われない所まで進める(`start`)。
fn open(h: &TempDir, rows: usize, cols: usize) -> Tui {
    start(&["notes"], h, rows, cols)
}

/// `args` で起動し、表(`rows` 行・`cols` 列。`cols` は 2 以上)が出るまで待つ。起動の直後の1打は端末への
/// 問い合わせに吸われることがあるので、何度押しても同じになる `$`(末尾の列へ)を効くまで押し直し、`0` で先頭の
/// 列に戻す。ここからあとのキーは吸われない。
fn start(args: &[&str], h: &TempDir, rows: usize, cols: usize) -> Tui {
    let mut t = Tui::spawn(args, h.path(), &[]);
    let band = format!("表 1/{rows}行 1/{cols}列");
    t.wait_until("表", |s| s.contains(&band));
    let last = format!("表 1/{rows}行 {cols}/{cols}列");
    press_until(&mut t, "$", "末尾の列", |s| s.contains(&last));
    t.send("0");
    t.wait_for(&band);
    t
}

/// `done` になるまでキーを押し直す。押すたびに少し待ってから押し直す(効いたキーをもう一度押さないように)。
fn press_until(t: &mut Tui, key: &str, what: &str, done: impl Fn(&str) -> bool) {
    let start = Instant::now();
    loop {
        t.send(key);
        let sent = Instant::now();
        while sent.elapsed() < Duration::from_millis(500) {
            if done(&t.screen()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(
            start.elapsed() <= LIMIT,
            "{what} を待って時間切れ。画面:\n{}",
            t.screen()
        );
    }
}

/// 表の行(行の頭が空白か `>` で、続けて名前と空白)の名前を、上からの順で。
fn rows(s: &str) -> Vec<String> {
    s.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix(' ').or_else(|| l.strip_prefix('>'))?;
            NAMES
                .iter()
                .find(|n| rest.strip_prefix(**n).is_some_and(|r| r.starts_with(' ')))
                .map(|n| (*n).to_string())
        })
        .collect()
}

/// 選んでいる行(行の頭の `>`)の名前。
fn selected(s: &str) -> Option<String> {
    s.lines().find_map(|l| {
        let rest = l.strip_prefix('>')?;
        NAMES
            .iter()
            .find(|n| rest.strip_prefix(**n).is_some_and(|r| r.starts_with(' ')))
            .map(|n| (*n).to_string())
    })
}

/// `text` の画面の上の (桁, 行)。0 始まり。
fn cell_of(s: &str, text: &str) -> Option<(usize, usize)> {
    s.lines()
        .enumerate()
        .find_map(|(y, l)| l.find(text).map(|i| (UnicodeWidthStr::width(&l[..i]), y)))
}

/// (桁, 行) の左クリック(SGR 1006 の押すと離す)。
fn click(x: usize, y: usize) -> String {
    format!("\x1b[<0;{};{}M\x1b[<0;{};{}m", x + 1, y + 1, x + 1, y + 1)
}

/// 選んでいるセルで Enter を押し、打った値で確定する(値のリストが出ても、打つと自由入力になる)。
fn edit_cell(t: &mut Tui, band: &str, value: &str) {
    t.send(ENTER);
    t.wait_for(&format!("編集 {band}"));
    t.send(value);
    t.wait_for(value);
    t.send(ENTER);
    t.wait_until("確定", |s| s.contains(&format!("*{value}")));
}

// ---------------------------------------------------------------- edit・review

#[test]
fn test_modes_edit_and_review_write_only_the_values() {
    // [stest -> req~cell-edit.inline-enter-esc~1, req~cell-edit.inline-pending~1, req~write-back.pending-hold~1, req~write-back.pending-diff-confirm~1]
    // title の入力で打って Esc → 入力が閉じて元の値のまま(ためる変更なし)。打って Enter → ためる変更になり、ファイルは
    // まだ変わらない。別々のノートの2つのセルを直す →「未保存 2」。Ctrl+S → 書く前に2ファイルの差分。Esc で表に
    // 戻ってもためた変更は残る。もう一度 Ctrl+S から Enter → 2つのファイルの title の値だけが変わる。
    let h = home("modes-edit-review", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send(ENTER);
    t.wait_for("編集 1/3行");
    t.send("案2");
    t.wait_for("案2");
    t.send(ESC);
    t.wait_until("Esc で入力が閉じる", |s| {
        s.contains("未保存 0  表 1/3行") && s.contains("会議の準備") && !s.contains("案2")
    });
    let s = t.screen();
    assert!(s.contains("未保存 0") && !s.contains("案2"), "{s}");
    edit_cell(&mut t, "1/3行", "新しい題");
    t.wait_for("未保存 1");
    assert_eq!(
        read(&h, "alpha.md"),
        ALPHA,
        "確定してもファイルはまだ変わらない"
    );
    t.send("j");
    t.wait_for("表 2/3行");
    edit_cell(&mut t, "2/3行", "買い物リスト");
    t.wait_for("未保存 2");
    assert_eq!(read(&h, "alpha.md"), ALPHA);
    assert_eq!(read(&h, "beta.md"), BETA);

    t.send(CTRL_S);
    t.wait_until("2ファイルの差分", |s| {
        s.contains("保存の確認  2ファイル")
            && s.contains("-title: 会議の準備")
            && s.contains("+title: 新しい題")
            && s.contains("-title: 買い物")
            && s.contains("+title: 買い物リスト")
    });
    assert_eq!(read(&h, "alpha.md"), ALPHA, "確認の間は書かない");
    assert_eq!(read(&h, "beta.md"), BETA, "確認の間は書かない");
    t.send(ESC);
    t.wait_until("表に戻り、ためた変更は残る", |s| {
        s.contains("未保存 2  表") && s.contains("*新しい題")
    });
    assert_eq!(read(&h, "alpha.md"), ALPHA);

    t.send(CTRL_S);
    t.wait_for("保存の確認  2ファイル");
    t.send(ENTER);
    let alpha = wait_file(&note(&h, "alpha.md"), &t, |s| s.contains("新しい題"));
    let beta = wait_file(&note(&h, "beta.md"), &t, |s| s.contains("買い物リスト"));
    assert_eq!(alpha, ALPHA.replace("title: 会議の準備", "title: 新しい題"));
    assert_eq!(beta, BETA.replace("title: 買い物", "title: 買い物リスト"));
    assert_eq!(read(&h, "gamma.md"), GAMMA);
    t.wait_for("未保存 0");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- quit

#[test]
fn test_modes_quit_asks_and_discard_leaves_the_note() {
    // [stest -> req~write-back.quit-confirm~1] 未保存 1 で `q` → 保存する・捨てる・戻る の確認。Esc で表に戻り(終わらない)、
    // もう一度 `q` から捨てる → 書かずに終わり、ノートは前のまま。
    let h = home("modes-quit-discard", &NOTES);
    let mut t = open(&h, 3, 4);
    edit_cell(&mut t, "1/3行", "捨てる題");
    t.wait_for("未保存 1");
    t.send("q");
    t.wait_until("終了の確認", |s| {
        s.contains("終了の確認")
            && s.contains("s 保存する")
            && s.contains("d 捨てて終わる")
            && s.contains("Esc 戻る")
    });
    t.send(ESC);
    t.wait_for("未保存 1  表 1/3行");
    assert_eq!(t.try_exit(), None, "Esc では終わらない");
    t.send("q");
    t.wait_for("終了の確認");
    t.send("d");
    assert_eq!(t.wait_exit(), 0);
    assert_eq!(read(&h, "alpha.md"), ALPHA);
}

#[test]
fn test_modes_quit_save_reviews_writes_and_quits() {
    // [stest -> req~write-back.quit-confirm~1] 終了の確認で保存する → 差分の確認、確定で書いて終わる。
    let h = home("modes-quit-save", &NOTES);
    let mut t = open(&h, 3, 4);
    edit_cell(&mut t, "1/3行", "残す題");
    t.wait_for("未保存 1");
    t.send("q");
    t.wait_for("終了の確認");
    t.send("s");
    t.wait_until("差分の確認", |s| {
        s.contains("保存の確認") && s.contains("+title: 残す題")
    });
    assert_eq!(read(&h, "alpha.md"), ALPHA);
    t.send(ENTER);
    assert_eq!(t.wait_exit(), 0);
    assert_eq!(
        read(&h, "alpha.md"),
        ALPHA.replace("title: 会議の準備", "title: 残す題")
    );
}

// ---------------------------------------------------------------- palette

#[test]
fn test_modes_palette_row_number_and_wq() {
    // [stest -> req~navigation.jump~1, req~screen.key-modes-palette-commands~3] パレットの行番号 `3` → 3行目が選ばれる。
    // 未保存 1 でパレットの `wq` → 差分の確認が出て、確定で書いて終わる。
    let h = home("modes-palette-wq", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send(":");
    t.wait_for("パレット  Enter 実行");
    t.send("3");
    t.wait_for("3 行目へ移動");
    t.send(ENTER);
    t.wait_until("3行目", |s| {
        s.contains("表 3/3行") && selected(s).as_deref() == Some("gamma")
    });
    assert_eq!(selected(&t.screen()).as_deref(), Some("gamma"));
    edit_cell(&mut t, "3/3行", "記録の題");
    t.wait_for("未保存 1");
    t.send(":");
    t.wait_for("パレット  Enter 実行");
    t.send("wq");
    t.wait_for("保存して終わる");
    t.send(ENTER);
    t.wait_until("差分の確認", |s| {
        s.contains("保存の確認") && s.contains("+title: 記録の題")
    });
    assert_eq!(read(&h, "gamma.md"), GAMMA, "確定の前は書かない");
    t.send(ENTER);
    assert_eq!(t.wait_exit(), 0);
    assert_eq!(
        read(&h, "gamma.md"),
        GAMMA.replace("title: 会議の記録", "title: 記録の題")
    );
}

#[test]
fn test_modes_palette_finds_save_with_its_key() {
    // [stest -> req~screen.palette-fuzzy~2, req~screen.palette-keys~1] `:` で「save」と打つ → 保存が候補の先頭に出て、横に
    // そのキー(Ctrl+S。画面の書き方は ^S)。Ctrl+P で開いて「save」と打つ → 同じ。Esc で閉じる。
    let h = home("modes-palette-save", &NOTES);
    let mut t = open(&h, 3, 4);
    let first_is_save = |s: &str| {
        s.lines()
            .find(|l| l.starts_with("> "))
            .is_some_and(|l| l.starts_with("> 保存 ") && l.contains("^S"))
    };
    t.send(":");
    t.wait_for("パレット  Enter 実行");
    t.send("save");
    t.wait_until("保存が先頭", first_is_save);
    t.send(ESC);
    t.wait_for("未保存 0  表 1/3行");
    t.send(CTRL_P);
    t.wait_for("パレット  Enter 実行");
    t.send("save");
    t.wait_until("Ctrl+P でも保存が先頭", first_is_save);
    let s = t.screen();
    assert!(first_is_save(&s), "{s}");
    t.send(ESC);
    t.wait_for("未保存 0  表 1/3行");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- help

#[test]
fn test_modes_help_shows_current_keys_first_and_marks_last() {
    // [stest -> req~screen.help~2] 表で `?` → 先頭に今押せるキー(表の操作)。末尾の「印」の節に `∅` と `#` の意味。Esc で閉じる。
    let h = home("modes-help", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("?");
    t.wait_for("今押せるキー(表)");
    let s = t.screen();
    let lines: Vec<&str> = s.lines().collect();
    let head = lines
        .iter()
        .position(|l| l.contains("今押せるキー(表)"))
        .unwrap();
    assert!(head <= 2, "今押せるキーが先頭にある:\n{s}");
    assert!(
        lines[head + 1].contains("Enter") && lines[head + 1].contains("編集"),
        "先頭は表の操作:\n{s}"
    );
    t.send("G");
    t.wait_until("末尾の印の節", |s| {
        s.lines().any(|l| l.trim() == "印")
    });
    let s = t.screen();
    let meaning = |mark: &str| {
        s.lines()
            .map(str::trim)
            .find_map(|l| l.strip_prefix(mark).filter(|r| r.starts_with(' ')))
            .map(str::trim)
            .unwrap_or_default()
            .to_string()
    };
    assert!(!meaning("∅").is_empty(), "∅ の意味:\n{s}");
    assert!(!meaning("#").is_empty(), "# の意味:\n{s}");
    let marks = s.lines().position(|l| l.trim() == "印").unwrap();
    let null = s.lines().position(|l| l.trim().starts_with("∅ ")).unwrap();
    assert!(marks < null, "∅ は印の節の中:\n{s}");
    t.send(ESC);
    t.wait_until("ヘルプを閉じる", |s| {
        s.contains("未保存 0  表 1/3行") && !s.contains("今押せるキー")
    });
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- search

#[test]
fn test_modes_find_is_smartcase_and_n_moves() {
    // [stest -> req~navigation.find~1] `/Todo` は大文字小文字を区別(alpha の todo を飛ばして gamma の Todo へ。`n` でも
    // ほかに移らない)、`/todo` は区別しない(`n` で alpha の todo にも移る)。探しても行は減らない。
    let h = home(
        "modes-find",
        &[
            ("alpha.md", "---\ntitle: a\nstatus: todo\n---\n"),
            ("beta.md", "---\ntitle: b\nstatus: done\n---\n"),
            ("gamma.md", "---\ntitle: c\nstatus: Todo\n---\n"),
        ],
    );
    let mut t = open(&h, 3, 2);
    t.send("/");
    t.wait_for("探す 1/3行");
    t.send("Todo");
    t.wait_for("/Todo");
    t.send(ENTER);
    t.wait_until("Todo の行", |s| {
        s.contains("表 3/3行") && selected(s).as_deref() == Some("gamma")
    });
    t.send_settle("n");
    let s = t.screen();
    assert!(
        s.contains("表 3/3行") && selected(&s).as_deref() == Some("gamma"),
        "/Todo は todo に当たらない:\n{s}"
    );
    t.send("/");
    t.wait_for("探す 3/3行");
    t.send("todo");
    t.wait_for("/todo");
    t.send(ENTER);
    t.wait_for("表 3/3行");
    t.send("n");
    t.wait_until("todo の行へ", |s| {
        s.contains("表 1/3行") && selected(s).as_deref() == Some("alpha")
    });
    assert_eq!(
        rows(&t.screen()),
        ["alpha", "beta", "gamma"],
        "行は減らない"
    );
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- filter

#[test]
fn test_modes_quick_filter_narrows_as_you_type_and_esc_clears() {
    // [stest -> req~navigation.search-incremental~1, req~navigation.search-box-hits~1, req~navigation.search-esc~1]
    // `\` で上の欄に入り「会議」と打つ → 「会議」を含む行だけ(打つたび。Enter の前)、欄に「会議」と「2/3行」。Esc → 元に戻る。
    let h = home("modes-quick-filter", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("\\");
    t.wait_for("\\ 検索:");
    t.send("会議");
    t.wait_until("会議を含む行だけ", |s| {
        rows(s) == ["alpha", "gamma"]
    });
    let s = t.screen();
    assert_eq!(rows(&s), ["alpha", "gamma"], "{s}");
    let bar = s.lines().find(|l| l.contains("\\ 検索:")).unwrap();
    assert!(bar.contains("会議") && bar.contains("2/3行"), "{s}");
    t.send(ESC);
    t.wait_until("Esc で元に戻る", |s| {
        rows(s) == ["alpha", "beta", "gamma"] && s.contains("\\ 検索…")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "beta", "gamma"]);
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- detail

#[test]
fn test_modes_detail_shows_properties_and_body() {
    // [stest -> req~navigation.detail-view~1, req~navigation.detail-body~1, req~screen.key-modes-detail-defaults~1]
    // 行で `K` → その行の全プロパティが縦に並び、下に「本文」の見出しと本文の先頭の行。Esc で閉じる。本文の無いノートでは
    // 見出しを出さない。`K` でも閉じる。
    let h = home("modes-detail", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("K");
    t.wait_for("詳細  alpha.md");
    let s = t.screen();
    let lines: Vec<&str> = s.lines().map(str::trim_end).collect();
    let prop = |key: &str, value: &str| {
        lines.iter().any(|l| {
            l.get(1..)
                .and_then(|r| r.strip_prefix(key))
                .is_some_and(|r| r.trim() == value)
        })
    };
    assert!(prop("title", "会議の準備"), "{s}");
    assert!(prop("status", "todo"), "{s}");
    assert!(prop("priority", "3"), "{s}");
    assert!(prop("tags", "[会議]"), "{s}");
    let body = lines
        .iter()
        .position(|l| l.trim() == "本文")
        .unwrap_or_else(|| panic!("本文の見出し:\n{s}"));
    let tags = lines.iter().position(|l| l.contains("[会議]")).unwrap();
    assert!(tags < body, "本文はプロパティの下:\n{s}");
    assert_eq!(lines[body + 1].trim(), "本文の一行目", "{s}");
    assert_eq!(lines[body + 2].trim(), "二行目", "{s}");
    t.send(ESC);
    t.wait_for("未保存 0  表 1/3行");
    t.send("j");
    t.wait_for("表 2/3行");
    t.send("K");
    t.wait_for("詳細  beta.md");
    let s = t.screen();
    assert!(
        !s.lines().any(|l| l.trim() == "本文"),
        "本文の無いノート:\n{s}"
    );
    t.send("K");
    t.wait_until("K で閉じる", |s| {
        s.contains("未保存 0  表 2/3行") && !s.contains("詳細  beta.md")
    });
    assert_gone(t.stop());
}

#[test]
fn test_modes_detail_edits_a_property_and_saves() {
    // [stest -> req~navigation.detail-edit~1, req~screen.key-modes-detail-defaults~1] 詳細の表示で status を選んで Enter →
    // 選んだプロパティの編集。done に直して保存 → ノートの status が done になり、ほかは変わらない。
    let h = home("modes-detail-edit", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("K");
    t.wait_for("詳細  alpha.md");
    t.send("j");
    t.wait_until("status を選ぶ", |s| {
        s.lines().any(|l| l.starts_with(">status"))
    });
    t.send(ENTER);
    t.wait_until("status の値のリスト", |s| {
        s.contains("編集 1/3行") && s.contains(">*todo")
    });
    t.send(DOWN);
    t.wait_for("> done");
    t.send(ENTER);
    t.wait_until("ためる変更", |s| {
        s.contains("未保存 1  詳細の表示")
            && s.lines()
                .any(|l| l.starts_with(">status") && l.contains("*done"))
    });
    t.send(ESC);
    t.wait_for("未保存 1  表 1/3行");
    t.send(CTRL_S);
    t.wait_until("差分の確認", |s| s.contains("+status: done"));
    t.send(ENTER);
    let alpha = wait_file(&note(&h, "alpha.md"), &t, |s| s.contains("status: done"));
    assert_eq!(alpha, ALPHA.replace("status: todo", "status: done"));
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- settings・settings_input

/// ビューの設定のフィルターの区画で、列 due(title の次)の「比べる」「<」に 2026-10-01 を打って足す(反映はしない)。
fn add_due_before(t: &mut Tui) {
    t.wait_for(">+ 条件を足す");
    t.send(ENTER);
    t.wait_for("フィルターの列を選ぶ");
    t.send("j");
    t.wait_for(">due");
    t.send(ENTER);
    t.wait_for("due の条件の種類");
    t.send("jjj");
    t.wait_for(">比べる");
    t.send(ENTER);
    t.wait_for("due の比べ方");
    t.send("jj");
    t.wait_for("><  より小さい");
    t.send(ENTER);
    t.wait_until("設定の値の入力", |s| {
        s.contains("設定の値の入力") && s.contains("due <:")
    });
    t.send("2026-10-01");
    t.wait_for("due <: 2026-10-01");
    t.send(ENTER);
    t.wait_until("未反映の条件", |s| {
        s.contains("未反映 1") && s.contains("due: < 2026-10-01")
    });
}

#[test]
fn test_modes_settings_compare_filter_applies_and_cancel_keeps_table() {
    // [stest -> req~navigation.view-settings~2, req~navigation.filter-conditions-more~1] `o` のフィルターの区画で、日付の列の
    // `< 2026-10-01` を打って足す。取り消しで閉じる → 表は開く前のまま。反映 → それより前の行だけになり、`someday` の
    // ような型の合わない値の行は残らない。
    let h = home(
        "modes-settings",
        &[
            ("alpha.md", "---\ntitle: a\ndue: 2026-09-15\n---\n"),
            ("beta.md", "---\ntitle: b\ndue: 2026-10-20\n---\n"),
            ("gamma.md", "---\ntitle: c\ndue: someday\n---\n"),
        ],
    );
    let mut t = open(&h, 3, 2);
    t.send("o");
    t.wait_for("ビューの設定  notes");
    add_due_before(&mut t);
    t.send(ESC);
    t.wait_until("取り消して表に戻る", |s| s.contains("表 1/3行"));
    assert_eq!(
        rows(&t.screen()),
        ["alpha", "beta", "gamma"],
        "{}",
        t.screen()
    );

    t.send("o");
    t.wait_for("ビューの設定  notes");
    add_due_before(&mut t);
    // 反映のボタン(フィルター → 列 → ボタン)。
    t.send(BACKTAB);
    t.wait_for(">列");
    t.send(BACKTAB);
    t.wait_for("[>反映<]");
    t.send(ENTER);
    t.wait_until("反映して絞る", |s| s.contains("表 1/1行"));
    let s = t.screen();
    assert_eq!(rows(&s), ["alpha"], "{s}");
    assert!(!s.contains("someday") && !s.contains("2026-10-20"), "{s}");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- filter_window・filter_input・chips

/// 絞り込みの窓(開いて「+ 絞り込みを足す」を選んでいるところ)で、status の値の一覧の done を隠す条件を足す
/// (done にチェックして Tab で決める。screen.checklist-keys)。
fn hide_done(t: &mut Tui) {
    t.send(ENTER);
    t.wait_for("絞り込む列");
    t.send("j");
    t.wait_for("│> status");
    t.send(ENTER);
    t.wait_for("status の条件の種類");
    t.send(ENTER);
    t.wait_for("status の値");
    t.send("j");
    t.wait_for("> [ ] done");
    t.send(" ");
    t.wait_for("[x] done");
    t.send(TAB);
}

#[test]
fn test_modes_filter_window_adds_and_removes_conditions() {
    // [stest -> req~navigation.filter-window-open~1, req~navigation.filter-window-live~1, req~navigation.filter-window-count~1, req~navigation.filter-window-edit~2, req~navigation.filter-window-close~2]
    // `|` → 窓に「+ 絞り込みを足す」。status の done を外す → すぐ done の行が消え、帯に「status: done を除く」、ボタンに
    // 「絞り込み 1」。続けて priority の `≥ 3` を打って足す → 両方を満たす行だけ。窓で条件を外す → 元の行。Esc で閉じる。
    let h = home("modes-filter-window", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("|");
    t.wait_for("+ 絞り込みを足す");
    hide_done(&mut t);
    t.wait_until("done の行が消える", |s| {
        rows(s) == ["alpha", "gamma"]
            && s.contains("status: done を除く")
            && s.contains("絞り込み 1")
    });
    let s = t.screen();
    assert_eq!(rows(&s), ["alpha", "gamma"], "{s}");
    assert!(s.contains("[status: done を除く]"), "帯の項目:\n{s}");
    assert!(s.contains("| 絞り込み 1"), "ボタンの条件の数:\n{s}");
    t.send("j");
    t.wait_for("> + 絞り込みを足す");
    t.send(ENTER);
    t.wait_for("絞り込む列");
    t.send("jj");
    t.wait_for("│> priority");
    t.send(ENTER);
    t.wait_for("priority の条件の種類");
    t.send("jjj");
    t.wait_for("> 比べる");
    t.send(ENTER);
    t.wait_for("priority の比べ方");
    t.send("jjjjj");
    t.wait_for("> ≥");
    t.send(ENTER);
    t.wait_until("値の入力", |s| {
        s.contains("絞り込みの値") && s.contains("priority ≥")
    });
    t.send("3");
    t.send(ENTER);
    t.wait_until("両方を満たす行だけ", |s| {
        rows(s) == ["alpha"] && s.contains("絞り込み 2")
    });
    assert_eq!(rows(&t.screen()), ["alpha"], "{}", t.screen());
    // 選んでいる priority の条件を外し、残る status の条件も外す。
    t.send("d");
    t.wait_until("priority の条件を外す", |s| {
        rows(s) == ["alpha", "gamma"] && s.contains("絞り込み 1")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "gamma"], "{}", t.screen());
    t.send("k");
    t.wait_for("│>  status");
    t.send("d");
    t.wait_until("元の行", |s| {
        rows(s) == ["alpha", "beta", "gamma"] && !s.contains("[status: done を除く]")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "beta", "gamma"]);
    t.send(ESC);
    t.wait_until("窓を閉じる", |s| {
        s.contains("表 1/3行") && !s.contains("+ 絞り込みを足す")
    });
    assert!(!t.screen().contains("+ 絞り込みを足す"), "{}", t.screen());
    assert_gone(t.stop());
}

#[test]
fn test_modes_chips_remove_item_brings_rows_back() {
    // [stest -> req~navigation.settings-reset-bar-item~1] 帯の「status: done を除く」を `f` で選んで外す → done の行が戻る。
    let h = home("modes-chips", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("|");
    t.wait_for("+ 絞り込みを足す");
    hide_done(&mut t);
    t.wait_for("絞り込み 1");
    t.send(ESC);
    t.wait_until("帯に項目", |s| {
        s.contains("設定: [status: done を除く]") && rows(s) == ["alpha", "gamma"]
    });
    t.send("f");
    t.wait_until("帯の項目を選ぶ", |s| {
        s.contains("設定の帯") && s.contains(">[status: done を除く]")
    });
    t.send(BACKSPACE);
    t.wait_until("done の行が戻る", |s| {
        rows(s) == ["alpha", "beta", "gamma"] && !s.contains("[status: done を除く]")
    });
    let s = t.screen();
    assert_eq!(rows(&s), ["alpha", "beta", "gamma"], "{s}");
    assert!(!s.contains("[status: done を除く]"), "{s}");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- list_select

#[test]
fn test_modes_value_list_picks_status_and_saves() {
    // [stest -> req~cell-edit.value-list-open~2, req~cell-edit.value-list-mark~1, req~cell-edit.value-list-none~1]
    // status が doing のセルで Enter → todo・done・doing と「なし」のリストが出て、今の値の doing に印。done を選んで保存 →
    // ノートの status が done になる。
    let h = home("modes-value-list", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("jj");
    t.wait_for("表 3/3行");
    t.send("l");
    t.wait_for("3/3行 2/4列");
    t.send(ENTER);
    t.wait_until("値のリスト", |s| {
        s.contains("編集 3/3行") && s.contains(">*doing")
    });
    let s = t.screen();
    for v in ["todo", "done", "なし"] {
        assert!(
            s.lines().any(|l| l.contains(&format!("│  {v}"))),
            "リストに {v}:\n{s}"
        );
    }
    t.send(UP);
    t.wait_for("│> done");
    t.send(ENTER);
    t.wait_until("ためる変更", |s| {
        s.contains("*done") && s.contains("未保存 1")
    });
    assert_eq!(read(&h, "gamma.md"), GAMMA, "保存の前は書かない");
    t.send(CTRL_S);
    t.wait_until("差分の確認", |s| s.contains("+status: done"));
    t.send(ENTER);
    let gamma = wait_file(&note(&h, "gamma.md"), &t, |s| s.contains("status: done"));
    assert_eq!(gamma, GAMMA.replace("status: doing", "status: done"));
    assert_gone(t.stop());
}

#[test]
fn test_modes_list_select_adds_a_tag_as_one_line() {
    // [stest -> req~cell-edit.list-items-toggle~2, req~write-back.byte-range-add-item~1] tags のセルで 家 を付けて確定 → tags に
    // 家 が足されたためる変更。保存 → 字下げ2の縦のリストに `  - 家` の1行だけが増え、ほかのバイトは同じ。
    let h = home("modes-list-select", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("$");
    t.wait_for("1/3行 4/4列");
    t.send(ENTER);
    t.wait_until("リストの選択", |s| {
        s.contains("リストの選択") && s.contains("[x] 会議") && s.contains("[ ] 家")
    });
    t.send(DOWN);
    t.wait_for(">[ ] 家");
    t.send(" ");
    t.wait_for("[x] 家");
    t.send(TAB);
    t.wait_until("ためる変更", |s| {
        s.contains("*会議 · 家") && s.contains("未保存 1")
    });
    assert_eq!(read(&h, "alpha.md"), ALPHA);
    t.send(CTRL_S);
    t.wait_until("差分の確認", |s| s.contains("+  - 家"));
    t.send(ENTER);
    let alpha = wait_file(&note(&h, "alpha.md"), &t, |s| s.contains("- 家"));
    assert_eq!(alpha, ALPHA.replace("  - 会議\n", "  - 会議\n  - 家\n"));
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- freq

#[test]
fn test_modes_freq_counts_values_and_enter_narrows() {
    // [stest -> req~navigation.frequency-open~1, req~navigation.frequency-order~1, req~navigation.frequency-enter~1, req~navigation.frequency-esc~1]
    // tags の列で `%` → 表の上に頻度表の窓。件数の多い順に件数と全行に対する割合(会議 3 (75%)・家 2 (50%))。Esc → 何も
    // 変わらない。会議で Enter → tags に会議を持つ行だけになり、ヘッダーに絞り込みが出る。
    let delta = "---\ntitle: 予定\nstatus: todo\npriority: 4\ntags:\n  - 会議\n---\n";
    let mut notes = NOTES.to_vec();
    notes.push(("delta.md", delta));
    let h = home("modes-freq", &notes);
    let mut t = open(&h, 4, 4);
    t.send("$");
    t.wait_for("1/4行 4/4列");
    t.send("%");
    t.wait_for("頻度表: tags");
    let s = t.screen();
    // 窓の1行(枠の `│` と選びの `>` を除く)が「値 件数 (割合)」の3つだけの行。
    let entry = |value: &str, count: &str, share: &str| {
        s.lines().position(|l| {
            let words: Vec<&str> = l
                .rsplit('│')
                .nth(1)
                .unwrap_or_default()
                .trim_start_matches('>')
                .split_whitespace()
                .collect();
            words == [value, count, share]
        })
    };
    let meeting = entry("会議", "3", "(75%)");
    let home_ = entry("家", "2", "(50%)");
    assert!(
        matches!((meeting, home_), (Some(a), Some(b)) if a < b),
        "多い順に件数と割合:\n{s}"
    );
    t.send(ESC);
    t.wait_until("Esc で閉じる", |s| {
        !s.contains("頻度表: tags") && s.contains("表 1/4行")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "beta", "delta", "gamma"]);
    t.send("%");
    t.wait_for("頻度表: tags");
    t.send(ENTER);
    t.wait_until("会議の行だけ", |s| {
        rows(s) == ["alpha", "delta", "gamma"] && s.contains("tags が 会議 を含む")
    });
    let s = t.screen();
    assert_eq!(rows(&s), ["alpha", "delta", "gamma"], "{s}");
    assert!(
        s.contains("tags が 会議 を含む"),
        "ヘッダーに絞り込み:\n{s}"
    );
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- sorts

#[test]
fn test_modes_sort_window_adds_flips_and_removes_a_rule() {
    // [stest -> req~navigation.sort-window-open~1, req~navigation.sort-window-live~1, req~navigation.sort-window-label~2, req~navigation.sort-window-edit~2, req~navigation.sort-window-close~2]
    // `S` → 窓に「+ 並べ替えを足す」。priority を足す → すぐ priority の昇順、ボタンに「↑ priority」。その行で Enter →
    // 降順「↓ priority」。決まりを外す → 元の並び。Esc で閉じる。
    let h = home("modes-sorts", &NOTES);
    let mut t = open(&h, 3, 4);
    t.send("S");
    t.wait_for("+ 並べ替えを足す");
    t.send(ENTER);
    t.wait_for("並べ替えを足す: 列");
    t.send("jj");
    t.wait_for("│> priority");
    t.send(ENTER);
    t.wait_until("priority の昇順", |s| {
        rows(s) == ["beta", "gamma", "alpha"] && s.contains("S ↑ priority")
    });
    let s = t.screen();
    assert_eq!(rows(&s), ["beta", "gamma", "alpha"], "{s}");
    assert!(s.contains("S ↑ priority"), "ボタンに先頭の決まり:\n{s}");
    t.send(ENTER);
    t.wait_until("priority の降順", |s| {
        rows(s) == ["alpha", "gamma", "beta"] && s.contains("↓ priority")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "gamma", "beta"]);
    t.send("d");
    t.wait_until("元の並び", |s| {
        rows(s) == ["alpha", "beta", "gamma"] && s.contains("S 並べ替え")
    });
    assert_eq!(rows(&t.screen()), ["alpha", "beta", "gamma"]);
    t.send(ESC);
    t.wait_until("窓を閉じる", |s| {
        s.contains("表 ") && !s.contains("+ 並べ替えを足す")
    });
    assert!(!t.screen().contains("+ 並べ替えを足す"), "{}", t.screen());
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- drawer・screen_menu

/// ワークスペース「仕事」(tasks・projects)。tasks は2行で、列は status・owner(`link` なら、t1 から projects の
/// p1 へのリンクの project も)。
fn workspace(tag: &str, link: bool) -> TempDir {
    let h = TempDir::new(tag);
    let w = |rel: &str, text: &str| {
        let p = h.path().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    w(
        "ws/.mdgrid/workspace.toml",
        "name = \"仕事\"\n[[table]]\nname = \"tasks\"\npath = \"tasks\"\n\
         [[table]]\nname = \"projects\"\npath = \"projects\"\n",
    );
    if link {
        w(
            "ws/tasks/t1.md",
            "---\nstatus: todo\nowner: a\nproject: \"[[p1]]\"\n---\n",
        );
    } else {
        w("ws/tasks/t1.md", "---\nstatus: todo\nowner: a\n---\n");
    }
    w("ws/tasks/t2.md", "---\nstatus: done\nowner: b\n---\n");
    w("ws/projects/p1.md", "---\nowner: me\n---\n");
    for d in ["config/mdgrid", "state"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    h
}

#[test]
fn test_modes_drawer_lists_tables_and_switches() {
    // [stest -> req~workspace.drawer-hidden~1, req~workspace.drawer-overlay~1, req~workspace.drawer-current-mark~1, req~workspace.drawer-close~2, req~workspace.drawer-switch~2]
    // 普段は引き出しが無い。tasks で `w` → 左に重ねて tasks(印付き)・projects。Esc → 何も変わらずに閉じる。
    // projects を Enter → projects の表になり、引き出しは閉じる。
    let h = workspace("modes-drawer", false);
    let mut t = start(&["ws/tasks"], &h, 2, 2);
    let s = t.screen();
    assert!(s.contains("仕事 › tasks") && !s.contains("╭ 仕事"), "{s}");
    t.send("w");
    t.wait_for("╭ 仕事");
    let s = t.screen();
    let current = s.lines().position(|l| l.contains("● tasks"));
    let other = s
        .lines()
        .position(|l| l.contains("projects") && l.starts_with('│'));
    assert!(
        matches!((current, other), (Some(a), Some(b)) if a < b),
        "tasks(印付き)・projects:\n{s}"
    );
    t.send(ESC);
    t.wait_until("Esc で閉じる", |s| {
        !s.contains("╭ 仕事") && s.contains("仕事 › tasks") && s.contains("表 1/2行")
    });
    let s = t.screen();
    assert!(!s.contains("╭ 仕事") && s.contains("仕事 › tasks"), "{s}");
    t.send("w");
    t.wait_for("╭ 仕事");
    t.send("j");
    t.wait_until("projects を選ぶ", |s| {
        s.lines()
            .any(|l| l.starts_with("│>") && l.contains("projects"))
    });
    t.send(ENTER);
    t.wait_until("projects の表", |s| {
        s.contains("仕事 › projects") && s.contains("表 1/1行") && !s.contains("╭ 仕事")
    });
    let s = t.screen();
    assert!(s.contains("仕事 › projects") && s.contains("p1"), "{s}");
    assert!(!s.contains("╭ 仕事"), "表を選ぶと引き出しは閉じる:\n{s}");
    assert_gone(t.stop());
}

#[test]
fn test_modes_screen_menu_lists_types_and_switches() {
    // [stest -> req~relations.screen-menu-open~2, req~relations.screen-menu-current~1, req~relations.screen-menu-key-hint~1, req~relations.screen-menu-close~2, req~relations.screen-menu-switch~1]
    // 表の画面で「▦ 表 ▾」を押す → ボタンのすぐ下の窓に「表」(今の型の印 ●)と「関係マップ」(切り替えのキー R)。Esc → 窓が
    // 閉じ、画面は変わらない。窓で「関係マップ」を選ぶ → 関係マップの画面になり、ボタンは「関係マップ ▾」。
    let h = workspace("modes-screen-menu", true);
    let mut t = start(&["ws/tasks"], &h, 2, 3);
    let (x, y) = cell_of(&t.screen(), "▦ 表 ▾").unwrap();
    t.send(&click(x, y));
    t.wait_for("画面の型");
    let s = t.screen();
    let menu = s.lines().position(|l| l.contains("画面の型")).unwrap();
    assert_eq!(menu, y + 1, "ボタンのすぐ下:\n{s}");
    assert!(s.contains("● 表"), "{s}");
    let map = s
        .lines()
        .find(|l| l.contains("関係マップ"))
        .unwrap_or_else(|| panic!("関係マップ:\n{s}"));
    assert!(map.contains(" R "), "{s}");
    t.send(ESC);
    t.wait_until("窓を閉じる", |s| {
        !s.contains("画面の型") && s.contains("▦ 表 ▾") && s.contains("表 1/2行")
    });
    let s = t.screen();
    assert!(!s.contains("画面の型") && s.contains("表 1/2行"), "{s}");
    t.send(&click(x, y));
    t.wait_for("画面の型");
    t.send("j");
    t.wait_until("関係マップを選ぶ", |s| {
        s.lines()
            .any(|l| l.contains("│>") && l.contains("関係マップ"))
    });
    t.send(ENTER);
    t.wait_until("関係マップの画面", |s| {
        s.contains("関係マップ ▾") && s.contains("tasks.project → projects")
    });
    let s = t.screen();
    assert!(
        s.contains("関係マップ ▾") && !s.contains("▦ 表 ▾"),
        "ボタンは「関係マップ ▾」:\n{s}"
    );
    t.send(ESC);
    t.wait_for("▦ 表 ▾");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- column_type

#[test]
fn test_modes_column_type_menu_sets_number() {
    // [stest -> req~cell-edit.type-config-dialog~1, req~cell-edit.type-config-dialog-items~1, req~cell-edit.type-config-dialog-write~1, req~cell-edit.type-config-notes-untouched~1]
    // 見出しの型の印(組 classic で `◷ due`)をクリック → 型の窓に「自動」(推定の日付を添える)と各型。「数」を選ぶ →
    // views.toml の表に `due = "number"`、日付の値に `!`、見出しの印は数の `#`。ノートは1バイトも変わらない。
    // もう一度開く →「自動」に日付が添えられて各型と並び、数に印。due のセルの入力は数の入力になる。
    let alpha = "---\ntitle: a\ndue: 2026-09-15\n---\n";
    let beta = "---\ntitle: b\ndue: 2026-10-20\n---\n";
    let h = home(
        "modes-column-type",
        &[("alpha.md", alpha), ("beta.md", beta)],
    );
    std::fs::write(
        h.path().join("config/mdgrid/config.toml"),
        "[look]\npreset = \"classic\"\n",
    )
    .unwrap();
    let mut t = open(&h, 2, 2);
    t.wait_for("◷ due");
    let (x, y) = cell_of(&t.screen(), "◷ due").unwrap();
    t.send(&click(x, y));
    t.wait_for("╭due の型");
    let s = t.screen();
    assert!(s.contains("● 自動(日付)"), "{s}");
    for ty in ["テキスト", "数", "チェック", "日付", "日時", "リスト"] {
        assert!(
            s.lines().any(|l| l.contains(&format!("   {ty} "))),
            "型 {ty}:\n{s}"
        );
    }
    t.send("jj");
    t.wait_until("数を選ぶ", |s| {
        s.lines().any(|l| l.contains('>') && l.contains("   数 "))
    });
    t.send(ENTER);
    t.wait_until("数の列になる", |s| {
        s.contains("# due") && s.contains("!2026-09-15") && s.contains("!2026-10-20")
    });
    let views = std::fs::read_to_string(h.path().join("config/mdgrid/views.toml")).unwrap();
    assert!(
        views.contains("[table.types]") && views.contains("due = \"number\""),
        "{views}"
    );
    assert_eq!(read(&h, "alpha.md"), alpha);
    assert_eq!(read(&h, "beta.md"), beta);

    let (x, y) = cell_of(&t.screen(), "# due").unwrap();
    t.send(&click(x, y));
    t.wait_for("╭due の型");
    let s = t.screen();
    assert!(s.contains("自動(日付)") && !s.contains("● 自動"), "{s}");
    assert!(s.contains("● 数"), "{s}");
    t.send(ESC);
    t.wait_until("窓を閉じる", |s| !s.contains("╭due の型"));
    t.send("$");
    t.wait_for("1/2行 2/2列");
    t.send(ENTER);
    t.wait_until("数の入力", |s| {
        s.contains("編集 1/2行") && s.contains("数: 数として読める値")
    });
    t.send(ESC);
    t.wait_for("表 1/2行");
    assert_eq!(read(&h, "alpha.md"), alpha);
    assert_gone(t.stop());
}
