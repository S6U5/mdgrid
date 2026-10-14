//! [BV-19] ルートフォルダ(`.obsidian` のあるフォルダ)の下のフォルダを開いた表を、パレットの
//! 「.base に書き出す」で書き出すと、書いた `.base` の絞り込みに `file.inFolder("<ルートからのフォルダ>")`
//! が入り、その `.base` を開くと書き出す前と同じ行(そのフォルダの下のノートだけ。入れ子も)が出る。
//! 開いたフォルダがルートフォルダそのものなら、フォルダの条件は足さない。
//! 変更の記録: specs/_changes/2026-10-14-export-infolder.md(タスク 1)。
//! 実装を見ずに、パレットのキーとファイルと行で確かめる(材料のノートは作り物)。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::base;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::{Path, PathBuf};

// ---- 材料 ----

/// ルート(`.obsidian` あり)にノート2つ、`Tasks/` に2つ、`Tasks/sub/` に1つ、`Other/` に1つ。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let root = tmp.notes();
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::create_dir_all(root.join("Tasks/sub")).unwrap();
    std::fs::create_dir_all(root.join("Other")).unwrap();
    tmp.write("home.md", "---\ntitle: ホーム\nstatus: todo\n---\n");
    tmp.write("idea.md", "---\ntitle: 思いつき\nstatus: done\n---\n");
    tmp.write(
        "Tasks/buy.md",
        "---\ntitle: 買い物\nstatus: todo\npriority: 2\n---\n",
    );
    tmp.write(
        "Tasks/call.md",
        "---\ntitle: 電話\nstatus: doing\npriority: 1\n---\n",
    );
    tmp.write(
        "Tasks/sub/deep.md",
        "---\ntitle: 奥の用事\nstatus: todo\npriority: 3\n---\n",
    );
    tmp.write("Other/memo.md", "---\ntitle: 別のメモ\nstatus: todo\n---\n");
    tmp
}

fn state_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("state")
}

fn conf_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

/// main と同じ道筋(`Markdown::open` → `App::new` → `start` → 読み込み)で、フォルダを開く。
fn boot_folder(tmp: &Tmp, folder: &Path) -> App {
    let src = Markdown::open(&[folder.to_path_buf()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(state_dir(tmp)),
        config_dir: Some(conf_dir(tmp)),
        target: folder.to_path_buf(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// main と同じ道筋(`crate::open_target`)で `.base` を開く。
fn boot_base(path: &Path) -> App {
    let t = crate::open_target(std::slice::from_ref(&path.to_path_buf()), None).unwrap();
    let mut b = App::new(Box::new(t.src), ColorMode::None);
    b.resize(80, 24);
    b.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: true,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: path.to_path_buf(),
        base: t.base,
    });
    while !b.loaded() {
        b.load_step(100);
    }
    b
}

fn labels(app: &App) -> Vec<String> {
    let mut v: Vec<String> = app.rows.iter().map(|r| app.src.label(r)).collect();
    v.sort();
    v
}

fn settle(a: &mut App) {
    for _ in 0..4 {
        if a.mode == Mode::Table {
            return;
        }
        press(a, KeyCode::Esc);
    }
}

/// パレットのコマンドを、画面に出る名前で実行する。
fn palette(a: &mut App, label: &str) {
    ch(a, ':');
    typing(a, label);
    press(a, KeyCode::Enter);
}

/// 入力の欄を空にして打ち、Enter で決める。
fn enter_text(a: &mut App, text: &str) {
    for _ in 0..40 {
        press(a, KeyCode::Backspace);
    }
    typing(a, text);
    press(a, KeyCode::Enter);
}

/// パレットで `name` に書き出し、書いた `.base` のパスを返す(`dir` に書かれる)。
fn export(a: &mut App, dir: &Path, name: &str) -> PathBuf {
    palette(a, ".base に書き出す");
    enter_text(a, name);
    settle(a);
    let out = dir.join(name);
    assert!(
        out.is_file(),
        "書き出した .base が開いたフォルダにある: {}\n知らせ: {:?}\n{}",
        out.display(),
        a.message,
        screen(a)
    );
    out
}

/// 書いた `.base` の絞り込みの式(全体の filters と、各ビューの filters)。空白は除く。
fn filter_exprs(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let parsed = base::parse(&text).unwrap_or_else(|e| panic!("読める .base: {e}\n{text}"));
    let mut dropped = Vec::new();
    let mut out = parsed.filter_exprs(&mut dropped);
    for v in &parsed.views {
        out.extend(v.filter_exprs(&mut dropped));
    }
    out.into_iter()
        .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
        .collect()
}

// ---- 試験 ----

#[test]
fn test_bv_19_export_infolder_subfolder_adds_infolder_filter() {
    // 下のフォルダ Tasks を開いて書き出す → 書いた .base の絞り込みに file.inFolder("Tasks")。
    let tmp = vault("bv19infolder-filter");
    let target = tmp.notes().join("Tasks");
    let mut a = boot_folder(&tmp, &target);
    let out = export(&mut a, &target, "tasks-export.base");
    let exprs = filter_exprs(&out);
    assert!(
        exprs
            .iter()
            .any(|e| e.contains(r#"file.inFolder("Tasks")"#)),
        "絞り込みに file.inFolder(\"Tasks\") がある: {exprs:?}\n{}",
        std::fs::read_to_string(&out).unwrap()
    );
}

#[test]
fn test_bv_19_export_infolder_subfolder_reopens_same_rows() {
    // 下のフォルダ Tasks の表を書き出し、その .base を開く → 書き出す前と同じ行
    // (Tasks の下のノートだけ。入れ子の Tasks/sub も入り、ルートと Other のノートは出ない)。
    let tmp = vault("bv19infolder-rows");
    let target = tmp.notes().join("Tasks");
    let mut a = boot_folder(&tmp, &target);
    let before = labels(&a);
    assert_eq!(
        before.len(),
        3,
        "Tasks の下のノートは3つ(入れ子を含む): {before:?}"
    );
    assert!(
        before.iter().any(|l| l.contains("deep")),
        "入れ子の Tasks/sub のノートも行にある: {before:?}"
    );
    assert!(
        !before
            .iter()
            .any(|l| l.contains("home") || l.contains("idea") || l.contains("memo")),
        "開いたフォルダの外のノートは出ない: {before:?}"
    );
    let out = export(&mut a, &target, "tasks-rows.base");
    let b = boot_base(&out);
    assert_eq!(
        labels(&b),
        before,
        "書き出した .base でも同じ行\n{}",
        std::fs::read_to_string(&out).unwrap()
    );
}

#[test]
fn test_bv_19_export_infolder_nested_subfolder_uses_path_from_root() {
    // さらに下のフォルダ Tasks/sub を開いて書き出す → ルートフォルダからのフォルダ "Tasks/sub" で絞り、
    // 開くと同じ行(deep だけ)。
    let tmp = vault("bv19infolder-nested");
    let target = tmp.notes().join("Tasks/sub");
    let mut a = boot_folder(&tmp, &target);
    let before = labels(&a);
    assert_eq!(before.len(), 1, "Tasks/sub の下のノートは1つ: {before:?}");
    let out = export(&mut a, &target, "sub.base");
    let exprs = filter_exprs(&out);
    assert!(
        exprs
            .iter()
            .any(|e| e.contains(r#"file.inFolder("Tasks/sub")"#)),
        "絞り込みに file.inFolder(\"Tasks/sub\") がある: {exprs:?}"
    );
    let b = boot_base(&out);
    assert_eq!(labels(&b), before, "書き出した .base でも同じ行");
}

#[test]
fn test_bv_19_export_infolder_root_adds_no_infolder() {
    // 境: 開いたフォルダがルートフォルダそのもの → フォルダの条件は足さない。
    let tmp = vault("bv19infolder-root");
    let root = tmp.notes();
    let mut a = boot_folder(&tmp, &root);
    let out = export(&mut a, &root, "root.base");
    let exprs = filter_exprs(&out);
    assert!(
        !exprs.iter().any(|e| e.contains("inFolder")),
        "ルートフォルダを開いたときは inFolder を足さない: {exprs:?}"
    );
}
