//! 列の型(CE-35)の照合の指摘の試験: セルの入力が設定の型に従う、「自動」が広い範囲の型を外す、読むだけの起動でも
//! パレットから開く、計算の列の見出しの印は並べ替えのまま、設定で文字でない型にした列は札にしない。
//! 記録: specs/_changes/2026-10-13-column-types.md(タスク #5)。

use super::startup::Startup;
use super::test_screen::{ch, press, typing, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::NewValue;
use mdgrid::types::Kind;
use ratatui::crossterm::event::KeyCode;

fn boot(name: &str, toml: &str, readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\ndue: 2026-10-01\nflag: x\nst: todo\n---\n");
    tmp.write("b.md", "---\ndue: 2026-10-05\nst: todo\n---\n");
    tmp.write("c.md", "---\nst: done\n---\n");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    let (config, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::Rgb);
    a.resize(100, 24);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly,
        no_color: false,
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

fn col(a: &mut App, name: &str) {
    a.col = a.cols.iter().position(|c| c == name).unwrap();
}

#[test]
fn test_ce_35_input_follows_set_type() {
    // [CE-35] `[types] flag = "checkbox"` → キーの無い行で Enter は切り替え(チェックの入力)になる。
    let (_t, mut a) = boot("ce35m_input", "[types]\nflag = \"checkbox\"\n", false);
    col(&mut a, "flag");
    a.row = a.rows.iter().position(|r| r.0.ends_with("b.md")).unwrap();
    press(&mut a, KeyCode::Enter);
    let r = a.rows[a.row].clone();
    assert_eq!(a.changes.pending(&r, "flag"), Some(&NewValue::Bool(true)));
}

#[test]
fn test_ce_35_auto_undoes_wider_type() {
    // [CE-35] 全体の設定で due = "number" のとき、型の窓で「自動」→ 表に due = "auto" を書き、推定(日付)に戻る。
    let (t, mut a) = boot("ce35m_auto", "[types]\ndue = \"number\"\n", false);
    col(&mut a, "due");
    assert_eq!(a.kind_of("due"), Kind::Number);
    a.set_column_type("due", None);
    assert_eq!(a.kind_of("due"), Kind::Date);
    let v = std::fs::read_to_string(t.0.join("config/views.toml")).unwrap();
    assert!(v.contains("due = \"auto\""), "{v}");
    // 表で number にしてから「自動」→ 全体の number ではなく推定に戻る。
    a.set_column_type("due", Some(Kind::Number));
    a.set_column_type("due", None);
    assert_eq!(a.kind_of("due"), Kind::Date);
}

#[test]
fn test_ce_35_palette_in_readonly() {
    // [CE-35] 読むだけの起動でも、パレットの column_type で型の窓が開く(ノートは書かない)。
    let (_t, mut a) = boot("ce35m_ro", "", true);
    col(&mut a, "due");
    ch(&mut a, ':');
    typing(&mut a, "column_type");
    press(&mut a, KeyCode::Enter);
    assert_eq!(format!("{:?}", a.mode), "ColumnType");
}

#[test]
fn test_ce_35_set_type_not_chips() {
    // [CE-35][SR-35] 札になる列(st)を number にすると札にしない。
    let (_t, mut a) = boot("ce35m_chip", "", false);
    assert!(a.is_select("st"), "前提: st は札の列");
    a.set_column_type("st", Some(Kind::Number));
    assert!(!a.is_select("st"));
}
