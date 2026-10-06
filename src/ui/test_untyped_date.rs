//! 型の決まらない列の日付を囲まずに書く(untyped-date)の画面の受け入れ試験(WB-18。関係: WB-7・CE-2・CE-26・WB-9)。
//! 仕様: specs/write-back/spec.md の WB-18。記録: specs/_changes/2026-10-03-new-note-rest.md のタスク 1。
//! 決定: specs/_decisions/2026-10-03-untyped-date.md。
//! 実装を見ずに、キー・画面・ファイルで確かめる。今日は 2026-10-03 に固定する。
//!
//! 「型の決まらない列」= 型の設定(CE-2 の types.json)が無く、読み込んだどのノートにも空でない値が無い列。
//! - そこに日付・日時の形だけの値を書く → 囲まない(`deadline: 2026-10-05`)。
//! - 日付に見えるが日付の形だけでない値(`2026-10-05 ごろ`)→ WB-7 で囲む。
//! - テキストと決まった列(空でない値か型の設定)に日付に見える文字 → WB-7 で囲む(今の振る舞いを守る)。
//!
//! 仮定(実装役に渡す):
//! - 新しいノートの聞く項目は、test_new_note と同じ道筋(`a` → 名前 → Enter → 聞く項目に打って Enter)。
//! - セルの編集は test_screen の `edit_cell`(Enter → 打つ → Enter)、保存は Ctrl+S → 確定の Enter。
//! - 値の無い列でも、どれかのノートにキーがあるか `.base` の order にあれば表の列に出る。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, edit_cell, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::frontmatter::{parse, Value};
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const TODAY: &str = "2026-10-03";

// ---- 材料 ----

/// 型の設定の無い保管庫。どのノートにも deadline は無い。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ntitle: 会議\nstatus: todo\n---\n");
    tmp.write("b.md", "---\ntitle: 本\nstatus: done\n---\n");
    tmp
}

/// `.obsidian/types.json` を書く。
fn types_json(tmp: &Tmp, json: &str) {
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(tmp.notes().join(".obsidian/types.json"), json).unwrap();
}

fn config(toml: &str) -> Config {
    let (c, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    c
}

/// main と同じ道筋で、`.base` なしのフォルダを開く。利用者の ~/.config には触らない。
fn boot(tmp: &Tmp, cfg: Config) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.today = types::parse_date(TODAY).unwrap();
    app.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = types::parse_date(TODAY).unwrap();
    app
}

fn read(tmp: &Tmp, rel: &str) -> String {
    let p: PathBuf = tmp.notes().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|_| panic!("{} が無い", p.display()))
}

/// 読み直した値の文字列(WB-6・WB-7: 囲んでも囲まなくても、値は打った文字のまま)。
fn reads_as(text: &str, key: &str) -> String {
    let fm = parse(text.as_bytes()).expect("読み直せる");
    match &fm
        .entries
        .iter()
        .find(|e| e.key == key)
        .unwrap_or_else(|| panic!("{key} が無い: {text:?}"))
        .value
    {
        Value::Str(s) => s.clone(),
        other => panic!("{key} の値が文字列でない: {other:?}"),
    }
}

/// `key: <value>` が二重か一重の引用符で囲んで書かれている(WB-7)。
fn assert_quoted(text: &str, key: &str, value: &str, why: &str) {
    let dq = format!("\n{key}: \"{value}\"\n");
    let sq = format!("\n{key}: '{value}'\n");
    assert!(
        text.contains(&dq) || text.contains(&sq),
        "[WB-18][WB-7] {why}: 囲んでいない: {text:?}"
    );
    assert_eq!(reads_as(text, key), value, "[WB-6] 読み直した値");
}

/// `a` → 名前 → 聞く項目(1つ)に `answer` → 作ったファイルの中身。
fn new_note_with_answer(tmp: &Tmp, a: &mut App, name: &str, answer: &str) -> String {
    ch(a, 'a');
    assert_ne!(a.mode, Mode::Table, "`a` で名前の欄が開く:\n{}", screen(a));
    typing(a, name);
    press(a, KeyCode::Enter);
    assert!(
        !tmp.notes().join(format!("{name}.md")).exists(),
        "聞き終えるまで作らない"
    );
    assert_ne!(a.mode, Mode::Table, "deadline の入力:\n{}", screen(a));
    assert!(screen(a).contains("deadline"), "{}", screen(a));
    typing(a, answer);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る:\n{}", screen(a));
    read(tmp, &format!("{name}.md"))
}

/// 表の行の位置(ノートの名前から)。
fn row_of(a: &App, label: &str) -> usize {
    a.rows
        .iter()
        .position(|r| a.src.label(r) == label)
        .unwrap_or_else(|| panic!("行 {label} が無い"))
}

fn assert_has_col(a: &App, col: &str) {
    assert!(
        a.cols.iter().any(|c| c == col),
        "列 {col} が表にある: {:?}",
        a.cols
    );
}

/// Ctrl+S → 差分の確認 → Enter で書く。
fn save_all(a: &mut App) {
    ctrl(a, 's');
    assert_eq!(a.mode, Mode::Confirm, "保存の確認:\n{}", screen(a));
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{}", screen(a));
    assert_eq!(a.changes.count(), 0, "保存したら未保存 0");
}

// ---- 新しいノートの聞く項目(CE-26) ----

#[test]
fn test_wb_18_untyped_new_note_date_unquoted() {
    // [WB-18][CE-26] どのノートにも無い deadline を聞いて 2026-10-05 → `deadline: 2026-10-05`(囲まない)。
    let tmp = vault("wb18nnd");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"deadline\"]\n"));
    let text = new_note_with_answer(&tmp, &mut a, "締め切り", "2026-10-05");
    assert_eq!(
        text, "---\ndeadline: 2026-10-05\n---\n",
        "[WB-18] 型の決まらない列の日付は囲まない"
    );
}

#[test]
fn test_wb_18_untyped_new_note_not_only_date_quoted() {
    // [WB-18][WB-7] 同じ列に `2026-10-05 ごろ`(日付の形だけでない)→ 囲む。
    let tmp = vault("wb18nnq");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"deadline\"]\n"));
    let text = new_note_with_answer(&tmp, &mut a, "締め切り", "2026-10-05 ごろ");
    assert_quoted(&text, "deadline", "2026-10-05 ごろ", "日付の形だけでない値");
}

#[test]
fn test_wb_18_untyped_new_note_datetime_unquoted() {
    // [WB-18][CE-26] 日時の形 `2026-10-05T09:30` → `deadline: 2026-10-05T09:30`(囲まない)。
    let tmp = vault("wb18nnt");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"deadline\"]\n"));
    let text = new_note_with_answer(&tmp, &mut a, "締め切り", "2026-10-05T09:30");
    assert_eq!(
        text, "---\ndeadline: 2026-10-05T09:30\n---\n",
        "[WB-18] 型の決まらない列の日時は囲まない"
    );
}

#[test]
fn test_wb_18_untyped_new_note_typed_text_quoted() {
    // [WB-18][CE-2][WB-7] 型の設定で text と宣言した列は、どのノートにも値が無くても型が決まっている
    // → 2026-10-05 は囲む。
    let tmp = vault("wb18nntx");
    types_json(&tmp, r#"{"types": {"deadline": "text"}}"#);
    let mut a = boot(&tmp, config("[new_note]\nask = [\"deadline\"]\n"));
    let text = new_note_with_answer(&tmp, &mut a, "締め切り", "2026-10-05");
    assert_quoted(&text, "deadline", "2026-10-05", "型の設定が text の列");
}

// ---- 表のセルの編集(WB-9 の保存まで) ----

#[test]
fn test_wb_18_untyped_cell_empty_values_only_unquoted() {
    // [WB-18] あるノートに `deadline:`(空)だけがある列 → 型が決まらない。
    // 空のセルと、キーの無いノートのセルに 2026-10-05 → 保存すると囲まない。
    let tmp = Tmp::new("wb18cell");
    tmp.write("a.md", "---\ntitle: 会議\ndeadline:\n---\n");
    tmp.write("b.md", "---\ntitle: 本\n---\n");
    let mut a = boot(&tmp, Config::default());
    assert_has_col(&a, "deadline");
    let ra = row_of(&a, "a.md");
    edit_cell(&mut a, ra, "deadline", "2026-10-05");
    let rb = row_of(&a, "b.md");
    edit_cell(&mut a, rb, "deadline", "2026-10-05T09:30");
    assert_eq!(a.changes.count(), 2, "{}", screen(&a));
    save_all(&mut a);
    assert_eq!(
        read(&tmp, "a.md"),
        "---\ntitle: 会議\ndeadline: 2026-10-05\n---\n",
        "[WB-18] 空の値だけの列の日付は囲まない"
    );
    assert_eq!(
        read(&tmp, "b.md"),
        "---\ntitle: 本\ndeadline: 2026-10-05T09:30\n---\n",
        "[WB-18] キーの無いノートに足す日時も囲まない"
    );
}

#[test]
fn test_wb_18_untyped_cell_base_order_only_unquoted() {
    // [WB-18] `.base` の order にあるが、どのノートにも無いキー deadline → 型が決まらない。
    // セルに 2026-10-05 → 保存すると `deadline: 2026-10-05` の1行が足される(囲まない)。
    let tmp = vault("wb18base");
    let base = "views:\n  - type: table\n    name: 表\n    order: [title, deadline]\n";
    let mut a = super::test_grid::open(&tmp, base, None);
    assert_has_col(&a, "deadline");
    let ra = row_of(&a, "a.md");
    edit_cell(&mut a, ra, "deadline", "2026-10-05");
    assert_eq!(a.changes.count(), 1, "{}", screen(&a));
    save_all(&mut a);
    assert_eq!(
        read(&tmp, "a.md"),
        "---\ntitle: 会議\nstatus: todo\ndeadline: 2026-10-05\n---\n",
        "[WB-18] どのノートにも無い列の日付は囲まない"
    );
}

#[test]
fn test_wb_18_untyped_cell_text_column_quoted() {
    // [WB-18][WB-7] あるノートに `memo: hello` → memo はテキストと決まった列。
    // 2026-10-05 を書く → 今どおり囲む(値のあるセルも、キーの無いノートのセルも)。
    let tmp = Tmp::new("wb18text");
    tmp.write("a.md", "---\ntitle: 会議\nmemo: hello\n---\n");
    tmp.write("b.md", "---\ntitle: 本\n---\n");
    let mut a = boot(&tmp, Config::default());
    let ra = row_of(&a, "a.md");
    edit_cell(&mut a, ra, "memo", "2026-10-05");
    let rb = row_of(&a, "b.md");
    edit_cell(&mut a, rb, "memo", "2026-10-05");
    assert_eq!(a.changes.count(), 2, "{}", screen(&a));
    save_all(&mut a);
    let ta = read(&tmp, "a.md");
    assert_quoted(&ta, "memo", "2026-10-05", "テキストの列(値のあるセル)");
    let tb = read(&tmp, "b.md");
    assert_quoted(&tb, "memo", "2026-10-05", "テキストの列(キーの無いノート)");
}

#[test]
fn test_wb_18_untyped_cell_typed_text_quoted() {
    // [WB-18][CE-2][WB-7] 型の設定で deadline を text と宣言。ノートには `deadline:`(空)だけ
    // → 型は決まっている(text)ので、2026-10-05 は囲む。
    let tmp = Tmp::new("wb18ttx");
    types_json(&tmp, r#"{"types": {"deadline": "text"}}"#);
    tmp.write("a.md", "---\ntitle: 会議\ndeadline:\n---\n");
    tmp.write("b.md", "---\ntitle: 本\n---\n");
    let mut a = boot(&tmp, Config::default());
    assert_has_col(&a, "deadline");
    let ra = row_of(&a, "a.md");
    edit_cell(&mut a, ra, "deadline", "2026-10-05");
    assert_eq!(a.changes.count(), 1, "{}", screen(&a));
    save_all(&mut a);
    let ta = read(&tmp, "a.md");
    assert_quoted(&ta, "deadline", "2026-10-05", "型の設定が text の列");
}
