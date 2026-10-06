//! フォルダを2つ以上開いたときに作る場所を選ぶ欄(new-note-rest のタスク 2)の画面の受け入れ試験。CE-25
//! (関係: WB-15)。仕様: specs/cell-edit/spec.md、記録 specs/_changes/2026-10-03-new-note-rest.md。
//! 画面の実装を見ずに、キー・クリック・画面の文字・ファイルで確かめる。
//!
//! 仮定(実装役に渡す):
//! - 起動は main と同じ道筋(`open_target(&[A, B])` → `App::new` → `start(Startup)` で `target` は
//!   `state_target(&[A, B])` → 読み込み)。作る場所の一覧は、この道筋で渡るもの(読み込み口か `target`)から作る。
//! - 一覧は今あるリストの選択の見せ方(選んでいる行の先頭に `>`)で、各行にフォルダの名前(最後の部分)が出る。
//! - 一覧の間と名前の欄の間は、モードが表(`Mode::Table`)でない。作り終えるか Esc で表に戻る。
//! - 読むだけのときのメッセージは「読むだけ」を含む(既存の READONLY の文)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use ratatui::crossterm::event::KeyCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 開くフォルダの名前(ほかの画面の文字に紛れない名前)。
const A: &str = "folder_aa";
const B: &str = "folder_bb";

// ---- 材料 ----

/// `<一時>/notes/folder_aa` と `<一時>/notes/folder_bb` に1つずつノートを置く(`.obsidian` は置かない:
/// 置くと2つが1つの根にまとまるため)。
fn two_folders(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    for (dir, note) in [(A, "a1.md"), (B, "b1.md")] {
        let d = tmp.notes().join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(note), "---\nstatus: todo\n---\n").unwrap();
    }
    tmp
}

fn dir(tmp: &Tmp, name: &str) -> PathBuf {
    tmp.notes().join(name)
}

/// main と同じ道筋でフォルダの並び `paths` を開く。利用者の ~/.config には触らない。
fn boot(tmp: &Tmp, paths: &[PathBuf], readonly: bool) -> App {
    let t = crate::open_target(paths, None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: crate::state_target(paths),
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// A・B の順で開く。
fn boot_ab(tmp: &Tmp, readonly: bool) -> App {
    boot(tmp, &[dir(tmp, A), dir(tmp, B)], readonly)
}

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn message(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

fn header(a: &App) -> String {
    screen(a).lines().next().unwrap_or("").to_string()
}

/// ヘッダー(1行目。開いたフォルダの名前が出る)を除いた行。
fn body(a: &App) -> Vec<String> {
    screen(a).lines().skip(1).map(str::to_string).collect()
}

/// 画面の文字 `label` をクリックする。
fn click_label(a: &mut App, label: &str) {
    let s = screen(a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(label))
        .unwrap_or_else(|| panic!("画面に「{label}」が無い:\n{s}"));
    let x = width::width(&line[..line.find(label).unwrap()]);
    a.click(x as u16 + 1, y as u16);
}

/// 作る場所の一覧が出ていて、`sel` の行に選びの印(`>`)があり、`other` の行には無い。
fn assert_folder_list(a: &App, sel: &str, other: &str) {
    assert_ne!(a.mode, Mode::Table, "一覧の間は表でない:\n{}", screen(a));
    let lines = body(a);
    let with = |name: &str| -> Vec<&String> { lines.iter().filter(|l| l.contains(name)).collect() };
    assert!(
        !with(sel).is_empty() && !with(other).is_empty(),
        "{sel} と {other} の一覧が出る:\n{}",
        screen(a)
    );
    assert!(
        with(sel).iter().any(|l| l.contains('>')),
        "{sel} に選びの印:\n{}",
        screen(a)
    );
    assert!(
        with(other).iter().all(|l| !l.contains('>')),
        "{other} には印が無い:\n{}",
        screen(a)
    );
}

/// 作ったあと: 表に戻り、`path` の行が選ばれている。未保存は増えない。
fn assert_created_and_selected(a: &App, path: &Path) {
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る:\n{}", screen(a));
    let real = path.canonicalize().expect("作ったファイル");
    let row = a.cur_row().expect("行が選ばれている");
    assert_eq!(
        PathBuf::from(&row.0).canonicalize().ok(),
        Some(real),
        "作った行が選ばれている"
    );
    assert_eq!(a.changes.count(), 0, "その場で作るので、ためる変更は無い");
}

// ---- CE-25: フォルダが2つ以上のとき作る場所を選ぶ ----

#[test]
fn test_ce_25_folders_pick_second_creates_there() {
    // [CE-25] A・B を開く → ヘッダーの右の端に「+ 新規」。`a` → A と B の一覧(A に印)。
    // ↓ で B → Enter → 名前の欄 → 「メモ」Enter → `B/メモ.md` ができ、A には作らない。その行が選ばれる。
    let tmp = two_folders("ce25f_pick");
    let mut a = boot_ab(&tmp, false);
    let h = header(&a);
    assert!(
        h.trim_end().ends_with("+ 新規"),
        "2つ開いてもヘッダーの右の端に「+ 新規」: {h:?}"
    );
    ch(&mut a, 'a');
    assert_folder_list(&a, A, B);
    press(&mut a, KeyCode::Down);
    assert_folder_list(&a, B, A);
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::Table, "選んだら名前の欄:\n{}", screen(&a));
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    let made = dir(&tmp, B).join("メモ.md");
    assert!(made.is_file(), "B に作る:\n{}", screen(&a));
    assert!(!dir(&tmp, A).join("メモ.md").exists(), "A には作らない");
    assert!(!tmp.notes().join("メモ.md").exists(), "親にも作らない");
    assert_created_and_selected(&a, &made);
}

#[test]
fn test_ce_25_folders_default_is_first() {
    // [CE-25] 「+ 新規」のクリック → 一覧(既定は最初の A)。動かさずに Enter → 「メモ」→ `A/メモ.md`。
    let tmp = two_folders("ce25f_default");
    let mut a = boot_ab(&tmp, false);
    click_label(&mut a, "+ 新規");
    assert_folder_list(&a, A, B);
    press(&mut a, KeyCode::Enter);
    assert_ne!(a.mode, Mode::Table, "選んだら名前の欄:\n{}", screen(&a));
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    let made = dir(&tmp, A).join("メモ.md");
    assert!(made.is_file(), "A に作る:\n{}", screen(&a));
    assert!(!dir(&tmp, B).join("メモ.md").exists(), "B には作らない");
    assert_created_and_selected(&a, &made);
}

#[test]
fn test_ce_25_folders_order_follows_arguments() {
    // [CE-25] 一覧は開いた引数の順で、既定は最初の引数のフォルダ。B・A の順で開く → B に印。
    let tmp = two_folders("ce25f_order");
    let mut a = boot(&tmp, &[dir(&tmp, B), dir(&tmp, A)], false);
    ch(&mut a, 'a');
    assert_folder_list(&a, B, A);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    let made = dir(&tmp, B).join("メモ.md");
    assert!(made.is_file(), "最初の引数の B に作る:\n{}", screen(&a));
    assert!(!dir(&tmp, A).join("メモ.md").exists());
    assert_created_and_selected(&a, &made);
}

#[test]
fn test_ce_25_folders_esc_creates_nothing() {
    // [CE-25] 一覧で Esc → 何も作らず表に戻る。
    let tmp = two_folders("ce25f_esc");
    let mut a = boot_ab(&tmp, false);
    let before = snapshot(&tmp.notes());
    let rows = a.rows.clone();
    ch(&mut a, 'a');
    assert_folder_list(&a, A, B);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "表に戻る:\n{}", screen(&a));
    assert_eq!(snapshot(&tmp.notes()), before, "何も作らない");
    assert_eq!(a.rows, rows);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_25_folders_single_folder_goes_straight_to_name() {
    // [CE-25] フォルダが1つなら一覧を出さず、すぐ名前の欄(今の振る舞いを守る)。
    let tmp = two_folders("ce25f_single");
    let only = dir(&tmp, A);
    let mut a = boot(&tmp, std::slice::from_ref(&only), false);
    ch(&mut a, 'a');
    assert_ne!(a.mode, Mode::Table, "名前の欄が開く:\n{}", screen(&a));
    assert!(
        !screen(&a).contains("|>"),
        "作る場所の一覧は出さない:\n{}",
        screen(&a)
    );
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    let made = only.join("メモ.md");
    assert!(made.is_file(), "打った名前でそのまま作る:\n{}", screen(&a));
    assert_created_and_selected(&a, &made);
}

#[test]
fn test_ce_25_folders_readonly_does_not_start() {
    // [CE-25][WB-15] `--readonly` では2つ以上でも始めない(一覧も名前の欄も出さず、何も作らない)。
    let tmp = two_folders("ce25f_ro");
    let mut a = boot_ab(&tmp, true);
    let before = snapshot(&tmp.notes());
    ch(&mut a, 'a');
    assert_eq!(a.mode, Mode::Table, "始めない:\n{}", screen(&a));
    assert!(message(&a).contains("読むだけ"), "{:?}", message(&a));
    a.message = None;
    if header(&a).contains("+ 新規") {
        click_label(&mut a, "+ 新規");
        assert_eq!(a.mode, Mode::Table, "クリックでも始めない:\n{}", screen(&a));
        assert!(message(&a).contains("読むだけ"), "{:?}", message(&a));
    }
    // 始まっていたとしても、Enter・打って Enter で作らない。
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "x");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Esc);
    assert_eq!(snapshot(&tmp.notes()), before, "何も作らない");
}
