#![cfg(unix)]
//! 新しいノートの窓の欄を、セルを直す窓と同じ候補とキーで使えることを、本物の実行ファイルで確かめる(疑似端末 80×24)。
//! 期待値は specs/requirements/cell-edit.md の cell-edit.new-note-form-link-candidates・cell-edit.new-note-form-move・
//! cell-edit.new-note-form-esc-popup の受け入れの例から。

use crate::pty;

use pty::{assert_gone, wait_file, TempDir, Tui};

const MARKER: &str = "[types]\ntags = \"list\"\npage = \"number\"\n\n\
[new_note]\nfolder = \"Quotes\"\nask = [\"source\", \"tags\", \"page\"]\n";

/// 本が2冊(Library/)と、どれも霧の灯台守を指す抜き書き(Quotes/)の台帳 `t`。
fn reading(tag: &str) -> TempDir {
    let home = TempDir::new(tag);
    let t = home.path().join("t");
    for d in [".mdgrid", "Library", "Quotes", "state"] {
        std::fs::create_dir_all(t.join(d)).unwrap();
    }
    std::fs::write(t.join(".mdgrid/workspace.toml"), MARKER).unwrap();
    std::fs::write(t.join("Library/霧の灯台守.md"), "---\nkind: book\n---\n").unwrap();
    std::fs::write(t.join("Library/雨の日の数学.md"), "---\nkind: book\n---\n").unwrap();
    std::fs::write(
        t.join("Quotes/q1.md"),
        "---\nsource: \"[[霧の灯台守]]\"\ntags:\n  - 仕事\npage: 3\n---\n",
    )
    .unwrap();
    std::fs::write(
        t.join("Quotes/q2.md"),
        "---\nsource: \"[[霧の灯台守]]\"\ntags:\n  - 家\npage: 9\n---\n",
    )
    .unwrap();
    home
}

#[test]
fn test_new_note_form_link_list_keys_and_esc_e2e() {
    // [stest -> req~cell-edit.new-note-form-link-candidates~1, req~cell-edit.new-note-form-move~3, req~cell-edit.new-note-form-esc-popup~1]
    // source の欄の候補に、まだ指されていない本「雨の日の数学」が出る。候補の Esc は候補だけを閉じて窓と打った名前を残す。
    // tags の欄は Enter でチェックを付け(窓は開いたまま)、Esc で付け外しの窓だけを閉じ、Enter でまた開き、Tab で次の欄へ進む。
    let home = reading("note-form-e2e");
    let mut t = Tui::spawn(&["t"], home.path(), &[]);
    t.wait_for("q1");
    t.send("a");
    t.wait_for("新しいノート");
    t.send("抜き書き\r");
    t.wait_until(
        "source の欄の候補に、指されていない本も出る",
        |s| s.contains("雨の日の数学") && s.contains("霧の灯台守"),
    );
    assert!(!t.screen().contains("[[霧の灯台守]]"), "{}", t.screen());
    t.send("\x1b");
    t.wait_until("候補だけが閉じ、窓と名前は残る", |s| {
        !s.contains("雨の日の数学") && s.contains("新しいノート") && s.contains("抜き書き")
    });
    // 名前の欄へ戻ってから source の欄に入り直すと、候補がまた出る。
    t.send_settle("\x1b[Z");
    t.send("\r");
    t.wait_for("雨の日の数学");
    // 打って絞り、↓ で選んで決める。
    t.send_settle("雨");
    t.send_settle("\x1b[B");
    t.send("\r");
    t.wait_until("tags の付け外しの窓", |s| s.contains("[ 決める ]"));
    t.send("\r");
    t.wait_until(
        "Enter でチェックが付き、窓は開いたまま",
        |s| s.contains("[x] ") && s.contains("[ 決める ]"),
    );
    assert!(!home.path().join("t/Quotes/抜き書き.md").exists());
    t.send("\x1b");
    t.wait_until("付け外しの窓だけが閉じる", |s| {
        !s.contains("[ 決める ]") && s.contains("新しいノート")
    });
    t.send("\r");
    t.wait_until(
        "Enter でまた開き、付けたチェックは入っていない",
        |s| s.contains("[ 決める ]") && !s.contains("[x] "),
    );
    t.send("\r");
    t.wait_for("[x] ");
    t.send("\t");
    t.wait_until("Tab で page の欄へ", |s| !s.contains("[ 決める ]"));
    t.send("\x13"); // Ctrl+S: 作る
    let note = home.path().join("t/Quotes/抜き書き.md");
    let text = wait_file(&note, &t, |s| s.contains("source:"));
    assert!(text.contains("source: \"[[雨の日の数学]]\""), "{text:?}");
    assert!(text.contains("tags:\n  - "), "{text:?}");
    assert_gone(t.stop());
}

#[test]
fn test_new_note_form_list_sits_below_the_form_e2e() {
    // [stest -> req~cell-edit.new-note-form-popup-place~1] 80×24 の画面(窓の右には候補の窓が入らない)で source の欄を開くと、
    // 候補のリストは新しいノートの窓の下の縁より下に出て、窓の欄の名前(名前・source・tags・page)は欠けない。
    let home = reading("note-form-place-e2e");
    let mut t = Tui::spawn(&["t"], home.path(), &[]);
    t.wait_for("q1");
    t.send("a");
    t.wait_for("新しいノート");
    t.send("抜き書き\r");
    t.wait_for("雨の日の数学");
    let s = t.screen();
    let lines: Vec<&str> = s.lines().collect();
    let top = lines
        .iter()
        .position(|l| l.contains("新しいノート"))
        .unwrap_or_else(|| panic!("新しいノートの窓:\n{s}"));
    let bottom = (top + 1..lines.len())
        .find(|&y| lines[y].contains('╰') || lines[y].contains('└'))
        .unwrap_or_else(|| panic!("窓の下の縁:\n{s}"));
    for label in ["名前", "source", "tags", "page"] {
        assert!(
            lines[top..bottom].iter().any(|l| l.contains(label)),
            "窓の欄の名前「{label}」が欠ける:\n{s}"
        );
    }
    let cand = lines
        .iter()
        .position(|l| l.contains("雨の日の数学"))
        .unwrap();
    assert!(
        cand > bottom,
        "候補は窓の下の縁({bottom} 行目)より下(候補は {cand} 行目):\n{s}"
    );
    assert_gone(t.stop());
}
