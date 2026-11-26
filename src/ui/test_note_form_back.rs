//! 新しいノートの窓で Shift+Tab で前の欄に戻るとき、今の欄の値を確定してから戻ること
//! (cell-edit.new-note-form-back-keeps)。期待値は specs/requirements/cell-edit.md の受け入れの例から。
//! 今日は 2026-10-02 に固定する。

use super::keymap::Mode;
use super::listpick::PickRow;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

const TYPES: &str = "[types]\ntags = \"list\"\npriority = \"number\"\ndue = \"date\"\n";

/// tags の候補に 仕事・家 が出るノートを置く。
fn vault(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    tmp.write("a.md", "---\ntags:\n  - 仕事\npriority: 1\n---\n");
    tmp.write("b.md", "---\ntags:\n  - 家\npriority: 2\n---\n");
    tmp
}

fn boot(tmp: &Tmp, ask: &str) -> App {
    let toml = format!("{TYPES}[new_note]\nask = {ask}\n");
    let (cfg, warnings): (Config, _) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
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
    app.today = types::parse_date("2026-10-02").unwrap();
    app
}

/// 窓の欄 `label` の行(見出しで探す。今の欄の印 `>` があっても同じ)。
fn field_line(a: &App, label: &str) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| {
            l.split('│').skip(1).any(|rest| {
                let rest = rest.strip_prefix(' ').unwrap_or(rest);
                let rest = rest.strip_prefix("> ").unwrap_or(rest).trim_start();
                rest.starts_with(label)
            })
        })
        .unwrap_or_else(|| panic!("欄 {label} の行が無い:\n{s}"))
        .to_string()
}

/// 今の欄(0 は名前、k は ask の k 番目)。
fn step(a: &App) -> usize {
    a.note.flow.as_ref().expect("窓が開いている").step
}

fn input_text(a: &App) -> String {
    a.input.as_ref().map(|i| i.text.clone()).unwrap_or_default()
}

// [utest -> req~cell-edit.new-note-form-back-keeps~1]
/// due の欄に打った日付は、Shift+Tab で priority の欄に戻っても due の欄に残る。
#[test]
fn test_back_keeps_typed_date() {
    let tmp = vault("nnback1");
    let mut a = boot(&tmp, "[\"priority\", \"due\"]");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(step(&a), 2, "due の欄:\n{}", screen(&a));
    typing(&mut a, "2026-10-09");
    press(&mut a, KeyCode::BackTab);
    assert_eq!(step(&a), 1, "priority の欄へ:\n{}", screen(&a));
    let due = field_line(&a, "due");
    assert!(due.contains("2026-10-09"), "{due}");
}

// [utest -> req~cell-edit.new-note-form-back-keeps~1]
/// リストの欄で付けたチェックは、Shift+Tab で名前の欄に戻っても tags の欄に残る。
#[test]
fn test_back_keeps_list_checks() {
    let tmp = vault("nnback2");
    let mut a = boot(&tmp, "[\"tags\", \"priority\"]");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::ListPick, "{}", screen(&a));
    let p = a.pick.as_mut().expect("付け外しの窓");
    let at = p
        .rows()
        .iter()
        .position(|r| matches!(r, PickRow::Cand(i) if p.cands[*i].shown() == "仕事"))
        .expect("候補に 仕事");
    p.sel = at;
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::BackTab);
    assert_eq!(step(&a), 0, "名前の欄へ:\n{}", screen(&a));
    assert_eq!(input_text(&a), "メモ");
    let tags = field_line(&a, "tags");
    assert!(tags.contains("仕事"), "{tags}");
}

// [utest -> req~cell-edit.new-note-form-back-keeps~1]
/// 数の欄に読めない `12a` を打って Shift+Tab を押すと、戻らずに理由が出る。
#[test]
fn test_back_with_bad_value_stays() {
    let tmp = vault("nnback3");
    let mut a = boot(&tmp, "[\"priority\", \"due\"]");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "12a");
    a.message = None;
    press(&mut a, KeyCode::BackTab);
    assert_eq!(step(&a), 1, "priority の欄のまま:\n{}", screen(&a));
    assert_eq!(input_text(&a), "12a");
    assert!(a.message.is_some(), "理由が出る");
}
