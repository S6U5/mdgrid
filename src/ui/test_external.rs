//! $EDITOR(SR-8)とコピー(OUT-1)の試験。
use super::external::{
    base64, editor_argv, osc52_seqs, pipe_to, split_words, Memory, MemoryLog, OSC52_MAX,
};
use super::keymap::Mode;
use super::test_screen::*;
use ratatui::crossterm::event::KeyCode;
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

/// エディタを起こす試験(親で SIGINT・SIGQUIT を無視する)を互いに並べず走らせる錠。
/// 無視の数え方の試験が「全部落とすと元に戻る」を他の試験の無視に邪魔されずに確かめるため。
pub(super) static EDITOR_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn editor_lock() -> MutexGuard<'static, ()> {
    EDITOR_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn test_sr_8_editor_argv_split() {
    // [SR-8] 引数つきの EDITOR は空白で分け、ノートのパスは(空白を含んでも)1つの引数。シェルは通さない。
    let p = Path::new("/tmp/my notes/a b.md");
    assert_eq!(
        editor_argv("code -w", p).unwrap(),
        vec![
            OsString::from("code"),
            OsString::from("-w"),
            OsString::from("/tmp/my notes/a b.md")
        ]
    );
    assert_eq!(
        editor_argv("  vim  ", p).unwrap(),
        vec![
            OsString::from("vim"),
            OsString::from("/tmp/my notes/a b.md")
        ]
    );
    assert_eq!(editor_argv("", p).unwrap()[0], OsString::from("vi"));
    // シェルの記号もそのまま1つの語(展開しない)。
    assert_eq!(
        editor_argv("ed $HOME;", p).unwrap()[1],
        OsString::from("$HOME;")
    );
}

#[test]
fn test_sr_8_editor_quotes_and_escapes() {
    // [SR-8] 空白を含むエディタのパスは引用符か `\` で書ける(POSIX の単語分けの一部。シェルは通さない)。
    let p = Path::new("/n/a b.md");
    let want = vec![
        OsString::from("/Applications/My Editor"),
        OsString::from("-w"),
        OsString::from("/n/a b.md"),
    ];
    assert_eq!(
        editor_argv("\"/Applications/My Editor\" -w", p).unwrap(),
        want
    );
    assert_eq!(
        editor_argv("'/Applications/My Editor' -w", p).unwrap(),
        want
    );
    assert_eq!(
        editor_argv("/Applications/My\\ Editor -w", p).unwrap(),
        want
    );
    // 語の途中の引用符はつながる。'…' の中の \ はそのまま、"…" の中は \" と \\ だけ効く。
    assert_eq!(split_words("a'b c'd").unwrap(), vec!["ab cd"]);
    assert_eq!(split_words("'a\\b'").unwrap(), vec!["a\\b"]);
    assert_eq!(
        split_words("\"a\\\"b\\\\c\\d\"").unwrap(),
        vec!["a\"b\\c\\d"]
    );
    // 空の引用符は空の引数。
    assert_eq!(split_words("x '' y").unwrap(), vec!["x", "", "y"]);
    // 閉じていない引用符は誤り(起動しない)。
    assert!(split_words("'abc").is_err());
    assert!(split_words("\"abc").is_err());
    let (_t, mut a) = make("sr8q", &[("a.md", "---\ns: 1\n---\n")]);
    ch(&mut a, 'e');
    let mut called = false;
    a.open_editor("'vim", &mut |_| {
        called = true;
        Ok(())
    });
    assert!(!called);
    assert!(a
        .message
        .as_deref()
        .unwrap()
        .contains("エディタの設定を読めない"));
}

/// EDITOR にする小さなシェルスクリプト: 渡されたパスのノートを書き換え、受け取った引数の数を記録する。
#[cfg(unix)]
fn script(tmp: &Tmp) -> String {
    let path = tmp.0.join("ed.sh");
    std::fs::write(
        &path,
        "#!/bin/sh\necho \"$#\" > \"$(dirname \"$1\")/../argc\"\nprintf -- '---\\nstatus: 外で直した\\n---\\n' > \"$1\"\n",
    )
    .unwrap();
    format!("/bin/sh {}", path.display())
}

#[cfg(unix)]
#[test]
fn test_sr_8_editor_reload() {
    // [SR-8] `e` で選んだ行のノートを開く → 戻ると読み直して行が新しい値になる。
    // パスに空白を含むノートでも、パスは1つの引数で届く。
    let _lock = editor_lock();
    let (t, mut a) = make(
        "sr8",
        &[
            ("my note.md", "---\nstatus: 前\n---\n"),
            ("z.md", "---\nstatus: z\n---\n"),
        ],
    );
    assert_eq!(a.src.label(&a.rows[0]), "my note.md");
    ch(&mut a, 'e');
    assert!(a.wants_editor());
    let mut calls = Vec::new();
    a.open_editor(&script(&t), &mut |leave| {
        calls.push(leave);
        Ok(())
    });
    // 端末を戻してから起動し、戻ったら画面に戻す。
    assert_eq!(calls, vec![true, false]);
    assert!(!a.wants_editor());
    assert_eq!(
        std::fs::read_to_string(t.0.join("argc")).unwrap().trim(),
        "1"
    );
    let row = a.rows[0].clone();
    let v = a.src.get(&row, "status").value;
    assert_eq!(v, Some(mdgrid::source::Value::Str("外で直した".into())));
    assert!(a.message.as_deref().unwrap().contains("エディタから戻った"));
    assert_eq!(a.mode, Mode::Table);
}

#[cfg(unix)]
#[test]
fn test_sr_8_editor_with_pending_marks_external() {
    // [SR-8] [WB-16] ためた変更のある行をエディタで変えたら、外の変更と同じに「外で変更」の印。
    let _lock = editor_lock();
    let (t, mut a) = make("sr8b", &[("a.md", "---\nstatus: 前\ntitle: t\n---\n")]);
    // エディタは title のキーを消すので、空にするためた値は WB-17・SR-18 で外れる。status を空にしてためる。
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    ch(&mut a, 'e');
    a.open_editor(&script(&t), &mut |_| Ok(()));
    let row = a.rows[0].clone();
    assert!(a.changes.external(&row));
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_sr_8_editor_missing_program() {
    // [SR-8] 起動できなければ止まらずに知らせる。端末は戻す。
    let _lock = editor_lock();
    let (_t, mut a) = make("sr8c", &[("a.md", "---\ns: 1\n---\n")]);
    ch(&mut a, 'e');
    let mut calls = Vec::new();
    a.open_editor("/nonexistent/mdgrid-editor -w", &mut |l| {
        calls.push(l);
        Ok(())
    });
    assert_eq!(calls, vec![true, false]);
    assert!(a
        .message
        .as_deref()
        .unwrap()
        .contains("エディタを起動できない"));
}

#[test]
fn test_out_1_base64_and_osc52() {
    // [OUT-1] base64 は標準の字母と `=` の埋め。OSC 52 は ESC ] 52 ; c ; <base64> BEL。
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"f"), "Zg==");
    assert_eq!(base64(b"fo"), "Zm8=");
    assert_eq!(base64(b"foo"), "Zm9v");
    assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    assert_eq!(base64("あ".as_bytes()), "44GC");
    assert_eq!(
        osc52_seqs("a\tb", false).unwrap(),
        vec![b"\x1b]52;c;YQli\x07".to_vec()]
    );
}

#[test]
fn test_out_1_osc52_tmux_and_limit() {
    // [OUT-1] tmux の中では DCS のパススルーでも送る(中の ESC は2重)。
    let seqs = osc52_seqs("a\tb", true).unwrap();
    assert_eq!(seqs.len(), 2);
    assert_eq!(seqs[0], b"\x1b]52;c;YQli\x07".to_vec());
    assert_eq!(seqs[1], b"\x1bPtmux;\x1b\x1b]52;c;YQli\x07\x1b\\".to_vec());
    // base64 が上限を超えたら送らず、理由を返す。メッセージ行で知らせ、OS のクリップボードには送る。
    let big = "x".repeat(OSC52_MAX / 4 * 3 + 3);
    assert!(osc52_seqs(&big, false).unwrap_err().contains("長すぎる"));
    assert!(osc52_seqs(&"x".repeat(OSC52_MAX / 4 * 3), false).is_ok());
    let (_t, mut a) = make("out1l", &[("a.md", &format!("---\nt: {big}\n---\n"))]);
    let log = memory(&mut a, false);
    ch(&mut a, 'y');
    assert!(log.borrow().osc.is_empty());
    assert_eq!(log.borrow().system.len(), 1);
    let m = a.message.clone().unwrap();
    assert!(
        m.contains("OS のクリップボードだけ") && m.contains("長すぎる"),
        "{m}"
    );
}

#[cfg(unix)]
#[test]
fn test_out_1_clipboard_command_hang_times_out() {
    // [OUT-1] 標準入力を読まずに止まるコマンドでも、画面を固めずに打ち切って知らせる(64KB を超える文字)。
    let big = "x".repeat(300_000);
    let start = std::time::Instant::now();
    let e = pipe_to("sleep", &["5"], &big, std::time::Duration::from_millis(300)).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::TimedOut);
    assert!(e.to_string().contains("応答しない"));
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
    // 読まずにすぐ終わるコマンドは、書き込みの失敗を返す(子は wait 済み)。
    let e = pipe_to("true", &[], &big, std::time::Duration::from_secs(2)).unwrap_err();
    assert!(e.to_string().contains("渡せない"), "{e}");
    // 読んで終わるコマンドは成功。
    pipe_to("cat", &[], &big, std::time::Duration::from_secs(5)).unwrap();
}

fn memory(a: &mut super::App, fail_system: bool) -> Rc<RefCell<MemoryLog>> {
    let log = Rc::new(RefCell::new(MemoryLog {
        fail_system,
        ..Default::default()
    }));
    a.clipboard = Box::new(Memory(log.clone()));
    log
}

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 会議\nstatus: 進行中\n---\n"),
    ("b.md", "---\ntitle: 買い物\tリスト\nstatus: 完了\n---\n"),
];

#[test]
fn test_out_1_copy_cell_and_row() {
    // [OUT-1] `y` と Ctrl+C は選んだセルを見出しつきのタブ区切り1行で、OSC 52 と OS のクリップボードの両方へ。
    let (_t, mut a) = make("out1", NOTES);
    let log = memory(&mut a, false);
    col_named(&mut a, "status");
    ch(&mut a, 'y');
    {
        let l = log.borrow();
        assert_eq!(l.system, vec!["status\t進行中".to_string()]);
        let want = format!("\x1b]52;c;{}\x07", base64("status\t進行中".as_bytes()));
        assert_eq!(l.osc, vec![want.into_bytes()]);
    }
    assert!(a.message.as_deref().unwrap().contains("コピーした"));
    ctrl(&mut a, 'c');
    assert_eq!(log.borrow().system.len(), 2);
    // `Y` は行を見出しの行 + 値の行で。値の中のタブは空白にする。ためた変更はその値。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Backspace);
    ch(&mut a, 'Y');
    let got = log.borrow().system.last().unwrap().clone();
    assert_eq!(got, "ノート\ttitle\tstatus\nb.md\t買い物 リスト\t");
    // 複数の行(行の選択 NV-5 が入ったら同じ口で)。
    let rows = a.rows.clone();
    assert_eq!(
        a.rows_tsv(&rows),
        "ノート\ttitle\tstatus\na.md\t会議\t進行中\nb.md\t買い物 リスト\t"
    );
}

#[test]
fn test_out_1_one_side_fails_still_copies() {
    // [OUT-1] OS のクリップボードが失敗しても止まらず、OSC 52 は出してメッセージ行で知らせる。
    let (_t, mut a) = make("out1b", NOTES);
    let log = memory(&mut a, true);
    ch(&mut a, 'y');
    assert_eq!(log.borrow().osc.len(), 1);
    let m = a.message.clone().unwrap();
    assert!(m.contains("OSC 52 だけ") && m.contains("試験の失敗"), "{m}");
}

#[test]
fn test_sr_18_ctrl_c_copies_while_loading() {
    // [SR-18] [OUT-1] 読み込み中に Ctrl+C → コピーになり、読み込みは止まらない。
    let tmp = Tmp::new("sr18c");
    for i in 0..50 {
        tmp.write(&format!("n{i:02}.md"), &format!("---\nn: {i}\n---\n"));
    }
    let src = mdgrid::source::markdown::Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = super::App::new(Box::new(src), super::ColorMode::None);
    while a.rows.is_empty() {
        a.load_step(1);
    }
    assert!(!a.loaded());
    let log = memory(&mut a, false);
    ctrl(&mut a, 'c');
    assert_eq!(log.borrow().osc.len(), 1);
    assert!(!a.cancelled);
    while !a.loaded() {
        a.load_step(100);
    }
    assert_eq!(a.rows.len(), 50);
}

#[cfg(unix)]
#[test]
fn test_sr_8_sigint_while_editor_does_not_kill_mdgrid() {
    // [SR-8] エディタを待つ間に Ctrl+C(SIGINT)・SIGQUIT が mdgrid に届いても落ちない(ためた変更が消えない)。
    // 子のスクリプトが親(この試験のプロセス)に SIGINT と SIGQUIT を送る。子は既定の扱いに戻っている。
    let _lock = editor_lock();
    let (t, mut a) = make("sr8s", &[("a.md", "---\nstatus: 前\ntitle: t\n---\n")]);
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Backspace);
    let sh = t.0.join("int.sh");
    std::fs::write(
        &sh,
        "#!/bin/sh\nkill -INT $PPID\nkill -QUIT $PPID\ntrap '' INT\necho ok > \"$(dirname \"$1\")/../done\"\n",
    )
    .unwrap();
    ch(&mut a, 'e');
    a.open_editor(&format!("/bin/sh {}", sh.display()), &mut |_| Ok(()));
    assert!(a.message.as_deref().unwrap().contains("エディタから戻った"));
    assert!(t.0.join("done").exists());
    assert_eq!(a.changes.count(), 1);
}

#[cfg(unix)]
#[test]
fn test_sr_8_nested_ignore_restores_only_on_last_drop() {
    // [SR-8] 無視は数で数える。重ねた無視の先のほうを落としても無視のまま、両方落とすと元に戻る
    // (並列の試験が互いの無視を早く戻して、エディタ待ちの間の SIGINT で落ちないため)。
    use super::external::sig::{is_ignored, Ignore, SIGINT, SIGQUIT};
    let _lock = editor_lock();
    let before = [is_ignored(SIGINT), is_ignored(SIGQUIT)];
    let first = Ignore::new();
    let second = Ignore::new();
    assert!(is_ignored(SIGINT) && is_ignored(SIGQUIT));
    drop(first);
    assert!(is_ignored(SIGINT) && is_ignored(SIGQUIT));
    drop(second);
    assert_eq!([is_ignored(SIGINT), is_ignored(SIGQUIT)], before);
    assert_eq!(before, [false, false]);
}
