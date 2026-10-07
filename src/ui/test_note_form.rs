//! 新しいノートのフォーム(CE-26・CE-27・CE-32)。specs/_changes/2026-10-07-new-note-form.md。
//! 今日は 2026-10-02 に固定する。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

const TODAY: &str = "2026-10-02";

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date", "priority": "number"}}"#,
    )
    .unwrap();
    tmp.write(
        "a.md",
        "---\ntitle: 会議\nstatus: todo\npriority: 3\ndue: 2026-09-20\n---\n",
    );
    tmp.write("b.md", "---\ntitle: 本\nstatus: done\npriority: 1\n---\n");
    tmp
}

fn boot(tmp: &Tmp, toml: &str) -> App {
    let (cfg, warnings): (Config, _) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
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

fn read(tmp: &Tmp, name: &str) -> String {
    std::fs::read_to_string(tmp.notes().join(name)).unwrap_or_default()
}

#[test]
fn test_ce_26_form_shows_fields_and_creates() {
    let tmp = vault("ce26form");
    let mut a = boot(
        &tmp,
        "[new_note]\nask = [\"priority\", \"tags\"]\n\n[new_note.set]\ntags = [\"inbox\"]\n",
    );
    ch(&mut a, 'a');
    let s = screen(&a);
    // 全部の欄が窓に並び、前もって入る値に出どころ。
    assert!(s.contains("新しいノート"), "{s}");
    assert!(s.contains("priority") && s.contains("tags"), "{s}");
    assert!(s.contains("inbox") && s.contains("(設定から)"), "{s}");
    typing(&mut a, "x");
    // どの欄からでも Ctrl+S で作る(priority は空なので書かない)。
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let t = read(&tmp, "x.md");
    assert!(t.contains("tags:\n  - inbox"), "{t}");
    assert!(!t.contains("priority"), "{t}");
}

#[test]
fn test_ce_26_form_defaults_to_visible_columns_and_back() {
    let tmp = vault("ce26cols");
    let mut a = boot(&tmp, "");
    ch(&mut a, 'a');
    let s = screen(&a);
    for c in ["title", "status", "priority", "due"] {
        assert!(s.contains(c), "見えている列 {c} が欄に並ぶ: {s}");
    }
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter); // title へ
    typing(&mut a, "題");
    press(&mut a, KeyCode::Enter); // status へ
                                   // 前の欄に戻っても、答えは残る。
    press(&mut a, KeyCode::BackTab);
    assert_eq!(a.input.as_ref().unwrap().text, "題");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    assert!(read(&tmp, "y.md").contains("title: 題"));
}

#[test]
fn test_ce_26_form_name_problem_shown_on_field() {
    let tmp = vault("ce26exists");
    let mut a = boot(&tmp, "");
    ch(&mut a, 'a');
    typing(&mut a, "a");
    press(&mut a, KeyCode::Enter); // title へ
    ctrl(&mut a, 's');
    // 既にある名前: 作らずに名前の欄へ戻り、理由を出す。
    assert_ne!(a.mode, Mode::Table);
    assert_eq!(a.note.flow.as_ref().unwrap().step, 0);
    assert!(a.message.is_some());
    assert_eq!(
        read(&tmp, "a.md"),
        "---\ntitle: 会議\nstatus: todo\npriority: 3\ndue: 2026-09-20\n---\n"
    );
}

#[test]
fn test_ce_27_required_and_date_template() {
    let tmp = vault("ce27req");
    let mut a = boot(
        &tmp,
        "[new_note]\nask = [\"due\", \"status\"]\nrequired = [\"status\"]\n\n[new_note.set]\ndue = \"{date+7}\"\n",
    );
    ch(&mut a, 'a');
    // 前もって入る due は7日後。
    assert!(screen(&a).contains("2026-10-09"), "{}", screen(&a));
    typing(&mut a, "z");
    ctrl(&mut a, 's');
    // status は必須で空 → 作らずに status の欄へ、理由を欄に出す。
    assert_ne!(a.mode, Mode::Table);
    assert!(!tmp.notes().join("z.md").exists());
    let s = screen(&a);
    assert!(s.contains("必須"), "{s}");
    typing(&mut a, "todo");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let t = read(&tmp, "z.md");
    assert!(
        t.contains("due: 2026-10-09") && t.contains("status: todo"),
        "{t}"
    );
}

#[test]
fn test_ce_32_hidden_created_and_body_template() {
    let tmp = vault("ce32body");
    std::fs::create_dir_all(tmp.notes().join("templates")).unwrap();
    std::fs::write(
        tmp.notes().join("templates/t.md"),
        "# {name}\n作成: {date}\n",
    )
    .unwrap();
    let mut a = boot(
        &tmp,
        "[new_note]\nask = [\"status\"]\nhidden = [\"created\"]\nbody = \"templates/t.md\"\n\n[new_note.set]\ncreated = \"{date}\"\n",
    );
    ch(&mut a, 'a');
    assert!(!screen(&a).contains("created"), "隠す欄は窓に出ない");
    typing(&mut a, "打ち合わせ");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
    let t = read(&tmp, "打ち合わせ.md");
    assert!(
        t.contains("created: 2026-10-02"),
        "作成日が自動で入る(囲まない): {t}"
    );
    assert!(t.ends_with("---\n# 打ち合わせ\n作成: 2026-10-02\n"), "{t}");
}
