//! 行のファイル名のクリックで $EDITOR を開く(SR-19。関係 SR-6・SR-8・WB-15)の受け入れの試験。
//! 本物のエディタは起動しない: 起動する試験は偽のエディタ(小さなシェルスクリプト)を渡し、
//! 起動してはいけない試験は端末を戻す関数で失敗を返して、起動の手前で止める。
use super::external::editor_argv;
use super::keymap::Mode;
use super::test_external::editor_lock;
use super::test_grid::{open, vault};
use super::test_screen::*;
use super::*;
use std::io;
use std::path::Path;

/// EDITOR にする小さなシェルスクリプト: 渡されたパスのノートを書き換え、受け取った引数を記録する(test_external と同じ形)。
#[cfg(unix)]
fn script(tmp: &Tmp) -> String {
    let path = tmp.0.join("ed.sh");
    std::fs::write(
        &path,
        "#!/bin/sh\nprintf '%s' \"$1\" > \"$(dirname \"$1\")/../arg\"\nprintf -- '---\\nstatus: 外で直した\\n---\\n' > \"$1\"\n",
    )
    .unwrap();
    format!("/bin/sh {}", path.display())
}

/// 画面の行 `i`(slots の添字)の、行の名前の欄(表の左の、ノートの名前の欄)の位置。
fn name_xy(a: &App, i: usize) -> (u16, u16) {
    let lay = view::layout(a);
    assert!(lay.label_w >= 1);
    // 選択の印1桁の右が名前の欄。
    (1, (view::data_y(a) + i - a.top) as u16)
}

/// 画面の行 `i` の、列 `name` のセルの位置(見えている列)。
fn cell_xy(a: &App, i: usize, name: &str) -> (u16, u16) {
    let (lay, cols) = view::visible_layout(a);
    let j = a.cols.iter().position(|c| c == name).unwrap();
    let mut x = lay.data_x();
    for &(k, w) in &cols {
        if k == j {
            return (x as u16, (view::data_y(a) + i - a.top) as u16);
        }
        x += w + 1;
    }
    panic!("列 {name} が見えていない");
}

fn cur_label(a: &App) -> String {
    a.src.label(&a.cur_row().unwrap())
}

/// 起動させない端末の関数: 呼ばれたら記録し、失敗を返す(open_editor は起動の手前で止まる)。
fn no_launch(calls: &mut Vec<bool>) -> impl FnMut(bool) -> io::Result<()> + '_ {
    move |leave| {
        calls.push(leave);
        Err(io::Error::other(
            "試験: 端末を戻さない(エディタを起動させない)",
        ))
    }
}

const TWO: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\nstatus: 前a\n---\n"),
    ("b.md", "---\ntitle: b\nstatus: 前b\n---\n"),
];

#[test]
fn test_sr_19_name_click_other_row_selects_only() {
    // [SR-19] [SR-6] 別の行の行の名前の欄を1回クリック → その行を選ぶだけで、エディタは開かない(編集も始めない)。
    let (_t, mut a) = make("sr19sel", TWO);
    col_named(&mut a, "title");
    assert_eq!(a.row, 0);
    let (x, y) = name_xy(&a, 1);
    a.click(x, y);
    assert_eq!(a.row, 1);
    assert_eq!(cur_label(&a), "b.md");
    assert!(!a.wants_editor(), "選んでいない行のクリックで開いた");
    assert_eq!(a.mode, Mode::Table);
}

#[cfg(unix)]
#[test]
fn test_sr_19_name_click_selected_row_opens_editor_and_reloads() {
    // [SR-19] [SR-8] 選んだ行の行の名前の欄をクリック → そのノートのパスでエディタが起動し、
    // 戻るとその行を読み直す(エディタが書き換えた値が表に出る)。編集は始めない。
    let _lock = editor_lock();
    let (t, mut a) = make("sr19open", TWO);
    col_named(&mut a, "title");
    // 1回目: b.md の行を選ぶだけ。
    let (x, y) = name_xy(&a, 1);
    a.click(x, y);
    assert!(!a.wants_editor());
    // 2回目: 選んだ行の名前の欄 → エディタで開くよう頼む。
    let (x, y) = name_xy(&a, 1);
    a.click(x, y);
    assert_eq!(a.mode, Mode::Table, "名前の欄のクリックで編集が始まった");
    assert!(a.input.is_none());
    assert!(a.wants_editor(), "選んだ行の名前の欄のクリックで開かない");
    let mut calls = Vec::new();
    a.open_editor(&script(&t), &mut |leave| {
        calls.push(leave);
        Ok(())
    });
    assert_eq!(calls, vec![true, false]);
    assert!(!a.wants_editor());
    // そのノートのパスで起動した。
    let arg = std::fs::read_to_string(t.0.join("arg")).unwrap();
    assert!(arg.ends_with("b.md"), "{arg}");
    // 戻ると読み直して表に出る。もう1つの行は変わらない。
    let b = a.rows.iter().find(|r| a.src.label(r) == "b.md").unwrap();
    assert_eq!(
        a.src.get(b, "status").value,
        Some(mdgrid::source::Value::Str("外で直した".into()))
    );
    assert!(screen(&a).contains("外で直した"), "{}", screen(&a));
    assert!(a.message.as_deref().unwrap().contains("エディタから戻った"));
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    assert_eq!(a.mode, Mode::Table);
}

/// `file.name` と `file.path` を出したビュー。
const BASE_FILE: &str =
    "views:\n  - type: table\n    name: 全部\n    order: [title, file.name, file.path]\n";

#[cfg(unix)]
#[test]
fn test_sr_19_file_name_cell_click_opens_editor() {
    // [SR-19] `file.name` の列を出したビューで、別の行の `file.name` のセルは選ぶだけ、
    // 選んだ行の `file.name` のセルのクリックでエディタが起動し、戻ると読み直す。
    let _lock = editor_lock();
    let tmp = vault("sr19fn");
    let mut a = open(&tmp, BASE_FILE, None);
    col_named(&mut a, "title");
    let start = a.row;
    let other = if start == 0 { 1 } else { 0 };
    let (x, y) = cell_xy(&a, other, "file.name");
    a.click(x, y);
    assert_eq!(a.row, other);
    assert!(
        !a.wants_editor(),
        "選んでいない行の file.name のクリックで開いた"
    );
    let label = cur_label(&a);
    let (x, y) = cell_xy(&a, other, "file.name");
    a.click(x, y);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.wants_editor(),
        "選んだ行の file.name のクリックで開かない: {:?}",
        a.message
    );
    let mut calls = Vec::new();
    a.open_editor(&script(&tmp), &mut |leave| {
        calls.push(leave);
        Ok(())
    });
    assert_eq!(calls, vec![true, false]);
    let arg = std::fs::read_to_string(tmp.0.join("arg")).unwrap();
    assert!(arg.ends_with(&label), "{arg} / {label}");
    let r = a.rows.iter().find(|r| a.src.label(r) == label).unwrap();
    assert_eq!(
        a.src.get(r, "status").value,
        Some(mdgrid::source::Value::Str("外で直した".into()))
    );
    assert!(a.message.as_deref().unwrap().contains("エディタから戻った"));
}

#[test]
fn test_sr_19_file_path_cell_click_does_not_open() {
    // [SR-19] `file.path` など他の file.* の列のセルは今までどおり: 選んだ行でクリックしても開かない
    // (読むだけのセルとして理由を出す)。
    let tmp = vault("sr19fp");
    let mut a = open(&tmp, BASE_FILE, None);
    col_named(&mut a, "title");
    let i = a.row;
    let (x, y) = cell_xy(&a, i, "file.path");
    a.click(x, y);
    assert_eq!(a.row, i);
    assert!(!a.wants_editor(), "file.path のクリックで開いた");
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message.as_deref().unwrap_or("").contains("読むだけ"),
        "{:?}",
        a.message
    );
}

/// 1つの操作(`e` かクリック)の結果。頼んだ行・その起動の引数・モード・メッセージと、
/// main と同じく open_editor を呼んだあとの端末の呼び出し・メッセージ・モード・残った頼み。
#[derive(Debug, PartialEq)]
struct Outcome {
    request: Option<mdgrid::source::RowId>,
    argv: Option<Vec<std::ffi::OsString>>,
    mode: Mode,
    message: Option<String>,
    calls: Vec<bool>,
    after_message: Option<String>,
    after_mode: Mode,
    after_wants: bool,
}

/// `act` をしてからの結果を集める。open_editor の端末の関数は失敗を返すので、本物のエディタ(vi など)は起動しない。
fn outcome(a: &mut App, editor: &str, act: impl FnOnce(&mut App)) -> Outcome {
    a.message = None;
    act(a);
    let request = a.editor_request.clone();
    let argv = request
        .as_ref()
        .map(|r| editor_argv(editor, Path::new(&r.0)).unwrap());
    let (mode, message) = (a.mode, a.message.clone());
    let mut calls = Vec::new();
    a.open_editor(editor, &mut no_launch(&mut calls));
    Outcome {
        request,
        argv,
        mode,
        message,
        calls,
        after_message: a.message.clone(),
        after_mode: a.mode,
        after_wants: a.wants_editor(),
    }
}

fn press_e(a: &mut App) {
    ch(a, 'e');
}

#[test]
fn test_sr_19_empty_editor_click_same_as_e() {
    // [SR-19] [SR-8] EDITOR が空(無い)でも、選んだ行のファイル名のクリック(名前の欄・file.name のセル)は
    // `e` を押したのと全く同じ結果: 同じ行を同じ引数(空なら vi + ノートのパス)で起動するよう頼み、
    // 同じモード・同じメッセージになる。EDITOR の値は main と同じく open_editor に渡す。
    let _lock = editor_lock();
    let (_t, mut a) = make("sr19ee", TWO);
    col_named(&mut a, "title");
    let row = a.row;
    let by_e = outcome(&mut a, "", press_e);
    // `e` の今の振る舞い(変えない前提): 空なら vi で起動を頼む。
    let argv = by_e.argv.clone().expect("`e` で起動を頼まない");
    assert_eq!(argv[0], std::ffi::OsString::from("vi"));
    assert_eq!(argv.len(), 2);
    assert_eq!(by_e.mode, Mode::Table);
    // 選んだ行の名前の欄のクリック。
    let (x, y) = name_xy(&a, row);
    let by_click = outcome(&mut a, "", |a| a.click(x, y));
    assert_eq!(a.row, row);
    assert_eq!(by_click, by_e, "名前の欄のクリックが `e` と違う");

    // file.name のセルでも同じ。
    let tmp = vault("sr19ef");
    let mut b = open(&tmp, BASE_FILE, None);
    col_named(&mut b, "title");
    let i = b.row;
    let by_e = outcome(&mut b, "", press_e);
    assert!(by_e.request.is_some());
    let (x, y) = cell_xy(&b, i, "file.name");
    let by_click = outcome(&mut b, "", |b| b.click(x, y));
    assert_eq!(by_click, by_e, "file.name のクリックが `e` と違う");
}

#[test]
fn test_sr_19_readonly_click_same_as_e() {
    // [SR-19] [WB-15] 読むだけの起動(`--readonly`)でも、選んだ行のファイル名のクリック(名前の欄・file.name のセル)は
    // `e` を押したのと全く同じ結果: 同じ行を同じ引数で起動するよう頼み、同じモード・同じメッセージになる
    // (読むだけの案内に変えて編集を拒むのではない)。
    let _lock = editor_lock();
    let (t, mut a) = make("sr19re", TWO);
    a.readonly = true;
    col_named(&mut a, "title");
    let row = a.row;
    let by_e = outcome(&mut a, "ed -w", press_e);
    // `e` の今の振る舞い(変えない前提): 読むだけでも起動を頼む。
    let argv = by_e.argv.clone().expect("`e` で起動を頼まない");
    assert_eq!(argv[..2], [std::ffi::OsString::from("ed"), "-w".into()]);
    assert_eq!(by_e.mode, Mode::Table);
    let (x, y) = name_xy(&a, row);
    let by_click = outcome(&mut a, "ed -w", |a| a.click(x, y));
    assert_eq!(a.row, row);
    assert_eq!(
        by_click, by_e,
        "読むだけで、名前の欄のクリックが `e` と違う"
    );

    let tmp = vault("sr19rf");
    let mut b = open(&tmp, BASE_FILE, None);
    b.readonly = true;
    col_named(&mut b, "title");
    let i = b.row;
    let by_e = outcome(&mut b, "ed -w", press_e);
    assert!(by_e.request.is_some());
    let (x, y) = cell_xy(&b, i, "file.name");
    let by_click = outcome(&mut b, "ed -w", |b| b.click(x, y));
    assert_eq!(
        by_click, by_e,
        "読むだけで、file.name のクリックが `e` と違う"
    );
    // 起動はしていないので、どのファイルも変わらない。
    assert_eq!(read(&t, "a.md"), TWO[0].1);
}
