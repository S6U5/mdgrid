//! 端末の貼り付け(output.paste): 選んだセルを左上にして、下の行と右の列へためる変更として貼る。値は作ったもの。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, press, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::settings::{Dir, Group};
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: one\nstatus: s1\npriority: 1\n---\n"),
    ("b.md", "---\ntitle: two\nstatus: s2\npriority: 2\n---\n"),
    ("c.md", "---\ntitle: three\nstatus: s3\npriority: 3\n---\n"),
];

fn boot(name: &str, notes: &[(&str, &str)], readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"priority": "number"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    (tmp, a)
}

/// 行(名前に `name` を含む)の列の今の値(ためた値を重ねたもの)。
fn val(a: &App, name: &str, col: &str) -> String {
    let row = a
        .rows
        .iter()
        .find(|r| a.src.label(r).contains(name))
        .unwrap();
    a.plain(row, col)
}

fn column(a: &App, col: &str) -> Vec<String> {
    ["a.md", "b.md", "c.md"]
        .iter()
        .map(|n| val(a, n, col))
        .collect()
}

// [utest -> req~output.paste~2]
/// 改行で分けた3つの値は、選んだセルから下の3行に入り、取り消し1回で全部戻る。
#[test]
fn test_paste_lines_go_down_in_one_step() {
    let (_t, mut a) = boot("paste1", NOTES, false);
    col_named(&mut a, "status");
    a.paste("done\ndoing\ntodo\n");
    assert_eq!(column(&a, "status"), vec!["done", "doing", "todo"]);
    assert_eq!(a.changes.count(), 3);
    ch(&mut a, 'u');
    assert_eq!(column(&a, "status"), vec!["s1", "s2", "s3"]);
    assert_eq!(a.changes.count(), 0);
}

// [utest -> req~output.paste~2]
/// タブで分けた値は、右の列に入る。
#[test]
fn test_paste_tabs_go_right() {
    let (_t, mut a) = boot("paste2", NOTES, false);
    let (ti, si) = (
        a.cols.iter().position(|c| c == "title").unwrap(),
        a.cols.iter().position(|c| c == "status").unwrap(),
    );
    assert_eq!(si, ti + 1, "status は title のすぐ右: {:?}", a.cols);
    col_named(&mut a, "title");
    a.paste("A\tx\r\nB\ty");
    assert_eq!(column(&a, "title"), vec!["A", "B", "three"]);
    assert_eq!(column(&a, "status"), vec!["x", "y", "s3"]);
}

// [utest -> req~output.paste~2]
/// 行が足りなければ入る分だけ貼り、入らなかった数を知らせる。
#[test]
fn test_paste_overflow_is_reported() {
    let (_t, mut a) = boot("paste3", &NOTES[..2], false);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Down);
    a.paste("p\nq\nr");
    assert_eq!(val(&a, "a.md", "status"), "s1");
    assert_eq!(val(&a, "b.md", "status"), "p");
    assert_eq!(a.changes.count(), 1);
    let m = a.message.clone().unwrap();
    assert!(m.contains('2'), "入らなかった2つを知らせる: {m}");
}

// [utest -> req~output.paste~2]
/// 列の型に合わない値のセルは飛ばし、理由を出す。
#[test]
fn test_paste_skips_values_that_do_not_fit() {
    let (_t, mut a) = boot("paste4", NOTES, false);
    col_named(&mut a, "priority");
    a.paste("5\nabc");
    assert_eq!(val(&a, "a.md", "priority"), "5");
    assert_eq!(val(&a, "b.md", "priority"), "2");
    assert_eq!(a.changes.count(), 1);
    assert!(a.message.is_some());
}

// [utest -> req~output.paste~2]
/// まとまりの見出しの行は飛ばして、次の行に続きを貼る。
#[test]
fn test_paste_skips_group_headings() {
    let (_t, mut a) = boot("paste5", NOTES, false);
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.regrid = true;
    a.refresh_if_needed();
    col_named(&mut a, "title");
    // 先頭は s1 の見出し。その下の a.md の行へ。
    while a.cur_row().is_none() {
        press(&mut a, KeyCode::Down);
    }
    assert_eq!(val(&a, "a.md", "title"), "one");
    a.paste("X\nY");
    assert_eq!(column(&a, "title"), vec!["X", "Y", "three"]);
}

// [utest -> req~output.paste~2]
/// 読むだけの起動では貼らず、理由を出す。
#[test]
fn test_paste_refused_when_readonly() {
    let (_t, mut a) = boot("paste6", NOTES, true);
    col_named(&mut a, "status");
    a.message = None;
    a.paste("done");
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.is_some());
}

// [utest -> req~output.paste~2]
/// 入力の欄を開いているときは、1行目の文字が入力に入り、ほかのセルは変わらない。
#[test]
fn test_paste_into_open_input_takes_first_line() {
    let (_t, mut a) = boot("paste7", NOTES, false);
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    if let Some(i) = a.input.as_mut() {
        i.text.clear();
        i.cursor = 0;
        i.list = None;
    }
    a.paste("abc\ndef");
    assert_eq!(a.input.as_ref().unwrap().text, "abc");
    assert_eq!(a.changes.count(), 0);
    assert_eq!(column(&a, "title"), vec!["one", "two", "three"]);
}
