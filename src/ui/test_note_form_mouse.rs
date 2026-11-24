//! 新しいノートの窓: 隠している頭を名前の欄に出さないこと(cell-edit.new-note-name-head)と、
//! マウスの操作(cell-edit.new-note-form-click-field・-click-keys・-click-list)。今日は 2026-10-02 に固定する。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;

const HEAD: &str = "[display]\nnote_name = '^\\d{8}_'\n[new_note]\nname = \"{date:YYYYMMDD}_\"\n";

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
    app.today = types::parse_date("2026-10-02").unwrap();
    app
}

/// ノートのフォルダの下の `.md`(根からの相対。名前の順)。
fn notes_in(tmp: &Tmp) -> Vec<String> {
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, root, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p.strip_prefix(root).unwrap().to_string_lossy().into_owned());
            }
        }
    }
    let mut out = Vec::new();
    walk(&tmp.notes(), &tmp.notes(), &mut out);
    out.sort();
    out
}

/// 画面の最初の `label` をクリックする。
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

/// 窓の今の欄(`>` の印の付いた行)の見出し。
fn current_field(a: &App) -> String {
    let s = screen(a);
    let line = s
        .lines()
        .find(|l| l.contains("│ > "))
        .unwrap_or_else(|| panic!("今の欄が無い:\n{s}"));
    let rest = line.split("│ > ").nth(1).unwrap();
    rest.split_whitespace().next().unwrap().to_string()
}

fn input_text(a: &App) -> String {
    a.input.as_ref().map(|i| i.text.clone()).unwrap_or_default()
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 表で隠している頭は、名前の欄に出ない。
#[test]
fn test_hidden_head_not_shown_in_name_field() {
    let tmp = vault("nnhead1");
    let mut a = boot(&tmp, HEAD);
    ch(&mut a, 'a');
    assert_eq!(input_text(&a), "");
    assert!(!screen(&a).contains("20261002_"), "{}", screen(&a));
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 打った名前の前に、隠している頭が付いて作られる。
#[test]
fn test_hidden_head_added_on_create() {
    let tmp = vault("nnhead2");
    let mut a = boot(&tmp, HEAD);
    ch(&mut a, 'a');
    typing(&mut a, "買い物");
    ctrl(&mut a, 's');
    assert_eq!(notes_in(&tmp), vec!["20261002_買い物.md", "a.md"]);
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 下のフォルダを打った名前では、頭は最後の名前の部分に付く。
#[test]
fn test_hidden_head_goes_on_last_part() {
    let tmp = vault("nnhead3");
    let mut a = boot(&tmp, HEAD);
    ch(&mut a, 'a');
    typing(&mut a, "sub/買い物");
    ctrl(&mut a, 's');
    assert_eq!(notes_in(&tmp), vec!["a.md", "sub/20261002_買い物.md"]);
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 何も打たなければ、頭だけの名前のノートは作らない。
#[test]
fn test_hidden_head_alone_creates_nothing() {
    let tmp = vault("nnhead4");
    let mut a = boot(&tmp, HEAD);
    ch(&mut a, 'a');
    ctrl(&mut a, 's');
    assert_eq!(notes_in(&tmp), vec!["a.md"]);
    assert!(a.note.flow.is_some(), "窓は開いたまま");
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 頭を自分で打った名前には、頭を重ねない。
#[test]
fn test_typed_head_not_doubled() {
    let tmp = vault("nnhead5");
    let mut a = boot(&tmp, HEAD);
    ch(&mut a, 'a');
    typing(&mut a, "20250101_古い");
    ctrl(&mut a, 's');
    assert_eq!(notes_in(&tmp), vec!["20250101_古い.md", "a.md"]);
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// 名前を決めて次の欄に移ると、名前の欄に頭の付いた名前が見える。
#[test]
fn test_full_name_shown_after_leaving_name_field() {
    let tmp = vault("nnhead6");
    let mut a = boot(&tmp, &format!("{HEAD}ask = [\"priority\"]\n"));
    ch(&mut a, 'a');
    typing(&mut a, "買い物");
    press(&mut a, KeyCode::Enter);
    assert_eq!(current_field(&a), "priority");
    assert!(screen(&a).contains("20261002_買い物"), "{}", screen(&a));
}

// [utest -> req~cell-edit.new-note-name-head~1]
/// `display.note_name` が無ければ、雛形の頭は今までどおり名前の欄に入って出る。
#[test]
fn test_head_shown_without_note_name_rule() {
    let tmp = vault("nnhead7");
    let mut a = boot(&tmp, "[new_note]\nname = \"{date:YYYYMMDD}_\"\n");
    ch(&mut a, 'a');
    assert_eq!(input_text(&a), "20261002_");
}

// [utest -> req~cell-edit.new-note-form-click-field~1]
/// 前の欄の行をクリックすると、その欄が前の答えで開く。
#[test]
fn test_click_earlier_field_reopens_it() {
    let tmp = vault("nnclick1");
    let mut a = boot(&tmp, "[new_note]\nask = [\"priority\", \"due\"]\n");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(current_field(&a), "due");
    click_label(&mut a, "名前 *");
    assert_eq!(current_field(&a), "名前");
    assert_eq!(input_text(&a), "メモ");
}

// [utest -> req~cell-edit.new-note-form-click-field~1]
/// 先の欄の行をクリックすると、今の欄を確定してその欄に移る。
#[test]
fn test_click_later_field_commits_and_moves() {
    let tmp = vault("nnclick2");
    let mut a = boot(&tmp, "[new_note]\nask = [\"priority\", \"due\"]\n");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    click_label(&mut a, "  due");
    assert_eq!(current_field(&a), "due");
    let s = screen(&a);
    let name_row = s.lines().find(|l| l.contains("名前 *")).unwrap();
    assert!(name_row.contains("メモ"), "{s}");
    assert_eq!(notes_in(&tmp), vec!["a.md"]);
}

// [utest -> req~cell-edit.new-note-form-click-field~1]
/// 今の欄の値が読めなければ、クリックしても移らず理由が出る。
#[test]
fn test_click_field_with_bad_value_stays() {
    let tmp = vault("nnclick3");
    let mut a = boot(&tmp, "[new_note]\nask = [\"priority\", \"due\"]\n");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "12a");
    click_label(&mut a, "名前 *");
    assert_eq!(current_field(&a), "priority");
    assert_eq!(input_text(&a), "12a");
    assert!(a.message.is_some(), "理由が出る");
}

// [utest -> req~cell-edit.new-note-form-click-keys~1]
/// キーの行の「作る」のクリックで、ノートが作られる。
#[test]
fn test_click_create_key_creates() {
    let tmp = vault("nnkeys1");
    let mut a = boot(&tmp, "");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    click_label(&mut a, "^S 作る");
    assert_eq!(notes_in(&tmp), vec!["a.md", "メモ.md"]);
    assert_eq!(a.mode, Mode::Table);
}

// [utest -> req~cell-edit.new-note-form-click-keys~1]
/// キーの行の「やめる」のクリックで、作らずに窓が閉じる。
#[test]
fn test_click_stop_key_cancels() {
    let tmp = vault("nnkeys2");
    let mut a = boot(&tmp, "");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    click_label(&mut a, "Esc やめる");
    assert!(a.note.flow.is_none());
    assert_eq!(notes_in(&tmp), vec!["a.md"]);
}

// [utest -> req~cell-edit.new-note-form-click-keys~1]
/// キーの行の「前の項目」のクリックで、前の欄が前の答えで開く。
#[test]
fn test_click_prev_key_goes_back() {
    let tmp = vault("nnkeys3");
    let mut a = boot(&tmp, "[new_note]\nask = [\"priority\"]\n");
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    click_label(&mut a, "Shift+Tab 前の項目");
    assert_eq!(current_field(&a), "名前");
    assert_eq!(input_text(&a), "メモ");
}

// [utest -> req~cell-edit.new-note-form-click-list~1]
/// 候補のリストの項目のクリックで、その値に決まって次の欄へ移る。
#[test]
fn test_click_list_item_chooses_and_moves() {
    let tmp = vault("nnlist1");
    let mut a = boot(
        &tmp,
        "[rules.status]\nvalues = [\"todo\", \"doing\", \"done\"]\n[new_note]\nask = [\"status\", \"priority\"]\n",
    );
    ch(&mut a, 'a');
    typing(&mut a, "メモ");
    press(&mut a, KeyCode::Enter);
    assert_eq!(current_field(&a), "status");
    click_label(&mut a, "doing");
    assert_eq!(current_field(&a), "priority");
    let s = screen(&a);
    let status_row = s.lines().find(|l| l.contains("  status")).unwrap();
    assert!(status_row.contains("doing"), "{s}");
}
