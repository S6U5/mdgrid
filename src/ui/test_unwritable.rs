//! 書かないノート(異常系)の画面の試験(WB-5。関係: WB-3・CE-8・CE-10・SR-15)。材料は test_screen の Tmp を使う。
//! 記録: specs/_changes/2026-10-02-abnormal-tests.md。
//!
//! WB-5 の7つの形のノートと、書ける3つ(普通のノート・フロントマターの無いノート(WB-3)・空のフロントマターの
//! ノート(WB-3。2026-10-02 empty-frontmatter-config で書ける側へ移した))を1つのフォルダに置き、
//! 形ごとに Enter・描画・一括の入力・保存を回す。落ちた形はまとめて報告する。

use super::keymap::Mode;
use super::test_screen::{app_of, col_named, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::source::{NewValue, RowId};
use ratatui::crossterm::event::KeyCode;

/// 1つの形: (表の名前, ファイル名, 理由に含まれるべき言葉)。
const FORMS: &[(&str, &str, &str)] = &[
    ("同じキーが2回", "duplicate.md", "同じキー"),
    ("BOM", "bom.md", "BOM"),
    ("YAML として読めない", "invalid.md", "読めない"),
    ("閉じない", "unclosed.md", "閉じていない"),
    ("UTF-8 でない", "notutf8.md", "UTF-8"),
    ("改行コードが混ざる", "mixed.md", "改行コード"),
    #[cfg(unix)]
    ("ハードリンク", "hard.md", "ハードリンク"),
    #[cfg(unix)]
    ("ハードリンク(相手)", "hard-link.md", "ハードリンク"),
];

/// 書ける側の対照(ok.md は普通のノート、nofm.md はフロントマターの無いノート(WB-3)、
/// empty.md は空のフロントマターのノート(WB-3。区切りの間に足す))。
const WRITABLE: &[&str] = &["ok.md", "nofm.md", "empty.md"];

/// 形のノートのバイト列(ハードリンクの相手は hard.md から張る)。
fn bytes_of(file: &str) -> Vec<u8> {
    match file {
        "duplicate.md" => b"---\nstatus: a\nstatus: b\n---\nbody\n".to_vec(),
        "bom.md" => {
            let mut v = vec![0xEF, 0xBB, 0xBF];
            v.extend_from_slice(b"---\nstatus: draft\n---\nbody\n");
            v
        }
        "invalid.md" => b"---\ntitle: [unclosed\nstatus: draft\n---\nbody\n".to_vec(),
        "unclosed.md" => b"---\nstatus: draft\n\nbody\n".to_vec(),
        "notutf8.md" => {
            let mut v = b"---\ntitle: ".to_vec();
            v.extend_from_slice(&[0xFF, 0xFE]);
            v.extend_from_slice(b"\nstatus: draft\n---\nbody\n");
            v
        }
        "mixed.md" => b"---\ntitle: t\r\nstatus: draft\n---\nbody\n".to_vec(),
        "empty.md" => b"---\n---\nbody\n".to_vec(),
        "hard.md" => b"---\ntitle: t\nstatus: draft\n---\nbody\n".to_vec(),
        "ok.md" => b"---\nstatus: todo\n---\nok\n".to_vec(),
        "nofm.md" => b"# nofm\nbody\n".to_vec(),
        _ => unreachable!("{file}"),
    }
}

/// 全部のノートを置いて開く。ハードリンクは hard.md と hard-link.md の2行になる。
fn setup(name: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (_, file, _) in FORMS {
        if *file == "hard-link.md" {
            continue;
        }
        std::fs::write(tmp.notes().join(file), bytes_of(file)).unwrap();
    }
    for file in WRITABLE {
        std::fs::write(tmp.notes().join(file), bytes_of(file)).unwrap();
    }
    #[cfg(unix)]
    std::fs::hard_link(
        tmp.notes().join("hard.md"),
        tmp.notes().join("hard-link.md"),
    )
    .unwrap();
    let app = app_of(&tmp, ColorMode::None);
    (tmp, app)
}

fn idx(a: &App, label: &str) -> usize {
    a.rows
        .iter()
        .position(|r| a.src.label(r) == label)
        .unwrap_or_else(|| panic!("row {label}"))
}

fn go(a: &mut App, label: &str) {
    a.row = idx(a, label);
    col_named(a, "status");
}

fn reason(a: &App, label: &str) -> Option<String> {
    let r: RowId = a.rows[idx(a, label)].clone();
    a.src.get(&r, "status").lock
}

fn report(stage: &str, fails: Vec<String>) {
    assert!(
        fails.is_empty(),
        "段「{stage}」で落ちた形:\n  {}",
        fails.join("\n  ")
    );
}

#[test]
fn test_wb_5_enter_does_not_open_and_shows_reason() {
    // [WB-5] [CE-8] 7つの形のノートの status のセルで Enter → 入力が開かず(Mode::Table のまま)、
    // メッセージ行に「読むだけ」と形ごとの理由(ほかの形の言葉を含まない)、ためる変更は 0。
    // 書ける側(普通のノート・フロントマターの無いノート・空のフロントマターのノート(WB-3))は入力が開く。
    let (_t, mut a) = setup("wb5enter");
    let mut fails = Vec::new();
    for (name, file, word) in FORMS {
        go(&mut a, file);
        a.message = None;
        press(&mut a, KeyCode::Enter);
        if a.mode != Mode::Table {
            fails.push(format!("{name}: Enter で {:?} に入った", a.mode));
            press(&mut a, KeyCode::Esc);
            a.set_mode(Mode::Table);
        }
        let msg = a.message.clone().unwrap_or_default();
        let Some(r) = reason(&a, file) else {
            fails.push(format!("{name}: セルの lock が None"));
            continue;
        };
        if !msg.contains("読むだけ") || !msg.contains(&r) || !msg.contains(word) {
            fails.push(format!(
                "{name}: メッセージ {msg:?} に「読むだけ」と理由 {r:?}({word:?})が無い"
            ));
        }
        for (other, _, w) in FORMS {
            if !other.starts_with(name) && !name.starts_with(other) && msg.contains(w) {
                fails.push(format!(
                    "{name}: メッセージ {msg:?} がほかの形「{other}」の言葉 {w:?} を含む"
                ));
            }
        }
        let line = screen(&a).lines().nth(22).unwrap_or("").to_string();
        if !line.contains("読むだけ") {
            fails.push(format!(
                "{name}: 画面のメッセージ行 {line:?} に「読むだけ」が無い"
            ));
        }
        if a.changes.count() != 0 {
            fails.push(format!("{name}: ためる変更が {}", a.changes.count()));
        }
    }
    for file in WRITABLE {
        go(&mut a, file);
        press(&mut a, KeyCode::Enter);
        if a.mode != Mode::Edit {
            fails.push(format!(
                "{file}(書ける側): 入力が開かない {:?} {:?}",
                a.mode, a.message
            ));
        }
        press(&mut a, KeyCode::Esc);
    }
    report("Enter", fails);
}

#[test]
fn test_wb_5_read_only_cell_drawn_with_hash() {
    // [WB-5] [SR-15] 7つの形のノートの status のセルは、表の描画で先頭に `#`(読むだけの印)。書ける側には付かない。
    let (_t, a) = setup("wb5hash");
    let s = screen(&a);
    let mut fails = Vec::new();
    for (name, file, _) in FORMS {
        let r = a.rows[idx(&a, file)].clone();
        let shown = super::cell::shown(&a, &r, "status");
        if !shown.text.starts_with('#') {
            fails.push(format!(
                "{name}: セルの表示 {:?} の先頭に # が無い",
                shown.text
            ));
        }
        // 画面の行(表示名のあとに `#`)。表示名は `.md` を除いた名前(SR-29)。
        let stem = file.trim_end_matches(".md");
        match s.lines().find(|l| l.contains(stem)) {
            Some(l) if l[l.find(stem).unwrap() + stem.len()..].contains('#') => {}
            l => fails.push(format!("{name}: 画面の行 {l:?} に # が無い")),
        }
    }
    for file in WRITABLE {
        let r = a.rows[idx(&a, file)].clone();
        let shown = super::cell::shown(&a, &r, "status");
        if shown.text.starts_with('#') {
            fails.push(format!(
                "{file}(書ける側): セルの表示 {:?} に #",
                shown.text
            ));
        }
    }
    report("描画の #", fails);
}

#[test]
fn test_wb_5_bulk_skips_read_only_rows_and_keeps_bytes() {
    // [WB-5] [CE-10] [WB-3] 全部の行を選んで status に done → 読むだけの行(7つの形。ハードリンクは2行)を飛ばし、
    // 「N行を飛ばした(理由)」に形ごとの理由が全部出る。ためるのは書ける3行(フロントマターの無いノート・空のフロントマターのノートも)だけ。
    // 保存の流れを通しても、読むだけのファイルのバイトは1バイトも変わらない。
    let (t, mut a) = setup("wb5bulk");
    let before: Vec<(String, Vec<u8>)> = FORMS
        .iter()
        .map(|(_, f, _)| (f.to_string(), std::fs::read(t.notes().join(f)).unwrap()))
        .collect();
    ctrl(&mut a, 'a');
    go(&mut a, "ok.md");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
    press(&mut a, KeyCode::End);
    let n = a.input.as_ref().unwrap().text.chars().count();
    for _ in 0..n {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);

    let mut fails = Vec::new();
    let msg = a.message.clone().unwrap_or_default();
    let skipped = FORMS.len();
    if !msg.contains(&format!("{skipped}行を飛ばした(")) {
        fails.push(format!(
            "メッセージ {msg:?} に「{skipped}行を飛ばした(」が無い"
        ));
    }
    for (name, file, _) in FORMS {
        match reason(&a, file) {
            Some(r) if msg.contains(&r) => {}
            r => fails.push(format!("{name}: メッセージ {msg:?} に理由 {r:?} が無い")),
        }
        let row = a.rows[idx(&a, file)].clone();
        if a.changes.pending(&row, "status").is_some() {
            fails.push(format!("{name}: ためる変更がある"));
        }
    }
    if a.changes.count() != WRITABLE.len() {
        fails.push(format!(
            "ためた数 期待 {}、実際 {}",
            WRITABLE.len(),
            a.changes.count()
        ));
    }
    for file in WRITABLE {
        let row = a.rows[idx(&a, file)].clone();
        if a.changes.pending(&row, "status") != Some(&NewValue::Str("done".into())) {
            fails.push(format!("{file}(書ける側): たまらない"));
        }
    }

    // 保存の流れ。
    let outcomes = a.changes.save(a.src.as_mut());
    if outcomes.len() != WRITABLE.len() {
        fails.push(format!(
            "保存した行の数が {} ({outcomes:?})",
            outcomes.len()
        ));
    }
    for (f, b) in &before {
        let now = std::fs::read(t.notes().join(f)).unwrap();
        if &now != b {
            fails.push(format!(
                "{f}: バイトが変わった {:?} → {:?}",
                String::from_utf8_lossy(b),
                String::from_utf8_lossy(&now)
            ));
        }
    }
    for file in WRITABLE {
        let now = std::fs::read_to_string(t.notes().join(file)).unwrap();
        if !now.contains("status: done") {
            fails.push(format!("{file}(書ける側): 書かれない {now:?}"));
        }
    }
    report("一括と保存", fails);
}
