//! 新しいノートの残り(new-note-gaps)の画面の受け入れ試験。CE-26・CE-27(関係: CE-2・CE-3・BV-17)。
//! 仕様: specs/cell-edit/spec.md、記録 specs/_changes/2026-10-03-new-note-gaps.md。
//! 画面の実装を見ずに、キー・画面の文字・ファイルで確かめる。今日は 2026-10-03 に固定する。
//!
//! 仮定(実装役に渡す):
//! - 起動は main と同じ道筋(`App::new` → `start(Startup)` → 読み込み)。設定は `Startup::config`、
//!   mdgrid のビューは設定の置き場(`Startup::config_dir`)の views.toml に手で書く。
//! - 聞いている項目は、画面に「新しいノート <列>」の形で出る(既存の new-note の試験と同じ)。
//! - テキストの列を聞くときの候補のリストは、セルの編集(CE-3)と同じ見せ方(枠 `|` の中に、選び `>`・
//!   今の値 `*` の印と値)。ここでは、セルの編集で同じ列を開いたときに画面に出る候補の並びと比べる。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const TODAY: &str = "2026-10-03";

// ---- 材料 ----

/// status は todo・done だけ(c.md には無い)。priority は数。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"priority": "number"}}"#,
    )
    .unwrap();
    tmp.write("a.md", "---\nstatus: todo\npriority: 3\n---\n");
    tmp.write("b.md", "---\nstatus: done\npriority: 1\n---\n");
    tmp.write("c.md", "---\ntitle: メモ\n---\n");
    tmp
}

fn conf_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

fn config(toml: &str) -> Config {
    let (c, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    c
}

/// views.toml を手で書く(この保管庫に、名前 `name` のビュー1つと、その new_note の表)。
fn write_view(tmp: &Tmp, name: &str, new_note: &str) {
    std::fs::create_dir_all(conf_dir(tmp)).unwrap();
    let real = std::fs::canonicalize(tmp.notes()).unwrap();
    let path = toml::Value::String(real.to_string_lossy().into_owned()).to_string();
    let text = format!(
        "[[target]]\npath = {path}\n\n[[target.view]]\nname = \"{name}\"\n\
         order = []\nhidden = []\nfilters_expr = []\n\n[target.view.new_note]\n{new_note}"
    );
    std::fs::write(conf_dir(tmp).join("views.toml"), text).unwrap();
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
        config_dir: Some(conf_dir(tmp)),
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
    let p = tmp.notes().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|_| panic!("{} が無い", p.display()))
}

fn labels(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// `]` で名前のタブまで動かす(選んだタブは `[名前]`)。
fn goto_tab(a: &mut App, name: &str) {
    let want = format!("[{name}]");
    for _ in 0..12 {
        if screen(a).lines().nth(1).unwrap_or("").contains(&want) {
            return;
        }
        ch(a, ']');
    }
    panic!("タブ「{name}」に移れない:\n{}", screen(a));
}

/// 新しいノートを始めて名前を打って Enter(聞く項目があればその入力に進む)。
fn start_and_name(a: &mut App, name: &str) {
    ch(a, 'a');
    assert_ne!(a.mode, Mode::Table, "`a` で名前の欄が開く:\n{}", screen(a));
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// 画面に出ている候補のリストの項目(枠 `|` の中の `>`・`*` の印のあとの値)と、選ばれているか。
/// セルの編集の見せ方(`|>*todo  |`・`|  done  |`)を読む。
fn list_on_screen(s: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for line in s.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 3 {
            continue;
        }
        for seg in &parts[1..parts.len() - 1] {
            let mut cs = seg.chars();
            let (Some(sel), Some(cur)) = (cs.next(), cs.next()) else {
                continue;
            };
            if !matches!(sel, '>' | ' ') || !matches!(cur, '*' | ' ') {
                continue;
            }
            let text = cs.as_str().trim();
            if text.is_empty() || text.starts_with('-') {
                continue;
            }
            out.push((text.to_string(), sel == '>'));
        }
    }
    out
}

fn texts(items: &[(String, bool)]) -> Vec<String> {
    items.iter().map(|(t, _)| t.clone()).collect()
}

/// セルの編集で、ノート `rel` の列 `col` のセルを開いたときに画面に出る候補(値だけ)。閉じて表に戻す。
fn cell_edit_list(a: &mut App, rel: &str, col: &str) -> Vec<String> {
    a.row = labels(a)
        .iter()
        .position(|l| l == rel)
        .unwrap_or_else(|| panic!("{rel} の行が無い: {:?}", labels(a)));
    col_named(a, col);
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "セルの編集が開く:\n{}", screen(a));
    let items = texts(&list_on_screen(&screen(a)));
    press(a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    items
}

/// 候補のリストで、値 `want` が選ばれるまで ↓(だめなら ↑)を押す。
fn select_in_list(a: &mut App, want: &str) {
    for key in [KeyCode::Down, KeyCode::Up] {
        for _ in 0..8 {
            let items = list_on_screen(&screen(a));
            if items.iter().any(|(t, sel)| *sel && t == want) {
                return;
            }
            press(a, key);
        }
    }
    panic!("候補の {want} を選べない:\n{}", screen(a));
}

// ---- 1. mdgrid のビューの new_note を使う(CE-26・CE-27・BV-17) ----

#[test]
fn test_ce_26_view_rule_overrides_config() {
    // [CE-26][CE-27][BV-17] 設定の [new_note] が ask = ["status"]、views.toml のビューの new_note が
    // ask = ["priority"]・set = { kind = "x" } → そのビューで始めると、名前のあとに priority を聞き
    // (status は聞かない)、作ったノートに kind: x と priority が入り、status は無い。
    let tmp = vault("nng_view");
    write_view(
        &tmp,
        "ビュー",
        "ask = [\"priority\"]\nset = { kind = \"x\" }\n",
    );
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    goto_tab(&mut a, "ビュー");
    start_and_name(&mut a, "ビューで");
    assert!(
        !tmp.notes().join("ビューで.md").exists(),
        "聞き終えるまで作らない"
    );
    let s = screen(&a);
    assert!(
        s.contains("新しいノート priority"),
        "ビューの決まりで priority を聞く:\n{s}"
    );
    assert!(
        !s.contains("新しいノート status"),
        "設定の status は聞かない:\n{s}"
    );
    typing(&mut a, "2");
    press(&mut a, KeyCode::Enter);
    let text = read(&tmp, "ビューで.md");
    assert!(text.contains("\nkind: x\n"), "ビューの set: {text:?}");
    assert!(text.contains("\npriority: 2\n"), "{text:?}");
    assert!(!text.contains("status"), "{text:?}");
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る:\n{}", screen(&a));
}

#[test]
fn test_ce_26_default_table_uses_config_rule() {
    // [CE-26][CE-27] 同じ設定・同じ views.toml で、既定の表(ビューを選ばない)では設定の status を聞く。
    // ビューの set(kind)は入らない。
    let tmp = vault("nng_default");
    write_view(
        &tmp,
        "ビュー",
        "ask = [\"priority\"]\nset = { kind = \"x\" }\n",
    );
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    goto_tab(&mut a, "既定の表");
    start_and_name(&mut a, "既定で");
    let s = screen(&a);
    assert!(
        s.contains("新しいノート status"),
        "設定の status を聞く:\n{s}"
    );
    assert!(!s.contains("新しいノート priority"), "{s}");
    typing(&mut a, "todo");
    press(&mut a, KeyCode::Enter);
    let text = read(&tmp, "既定で.md");
    assert!(text.contains("\nstatus: todo\n"), "{text:?}");
    assert!(!text.contains("kind"), "ビューの set は使わない: {text:?}");
    assert!(!text.contains("priority"), "{text:?}");
}

// ---- 2. テキストの列はセルの編集と同じ候補のリスト(CE-26・CE-2・CE-3) ----

#[test]
fn test_ce_26_text_ask_shows_same_list_as_cell_edit() {
    // [CE-26][CE-3] status が todo・done だけ → 名前のあとに、セルの編集で status のセルを開いたときと
    // 同じ候補(todo・done と「なし」)が同じ見せ方で出る。
    let tmp = vault("nng_list");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    let want = cell_edit_list(&mut a, "c.md", "status");
    let mut sorted = want.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        vec!["done".to_string(), "todo".to_string(), "なし".to_string()],
        "セルの編集の候補(前提)"
    );
    start_and_name(&mut a, "候補");
    assert!(
        !tmp.notes().join("候補.md").exists(),
        "聞き終えるまで作らない"
    );
    let s = screen(&a);
    assert!(s.contains("新しいノート status"), "{s}");
    assert_eq!(
        texts(&list_on_screen(&s)),
        want,
        "セルの編集と同じ候補のリスト:\n{s}"
    );
}

#[test]
fn test_ce_26_text_ask_pick_done_from_list() {
    // [CE-26][CE-3] 候補のリストで done を選んで確定 → 作ったノートが `status: done`。
    let tmp = vault("nng_pick");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    start_and_name(&mut a, "選ぶ");
    assert!(
        !list_on_screen(&screen(&a)).is_empty(),
        "候補のリストが出る:\n{}",
        screen(&a)
    );
    select_in_list(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&tmp, "選ぶ.md"), "---\nstatus: done\n---\n");
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る:\n{}", screen(&a));
}

#[test]
fn test_ce_26_text_ask_free_input_from_list() {
    // [CE-26][CE-3] 候補のリストから、打って自由入力に切り替える(リストは消える)→ `waiting` で作れる。
    let tmp = vault("nng_free");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    start_and_name(&mut a, "自由");
    assert!(
        !list_on_screen(&screen(&a)).is_empty(),
        "はじめは候補のリスト:\n{}",
        screen(&a)
    );
    typing(&mut a, "waiting");
    assert!(
        list_on_screen(&screen(&a)).is_empty(),
        "打つと自由入力に切り替わりリストは消える:\n{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&tmp, "自由.md"), "---\nstatus: waiting\n---\n");
    assert_eq!(a.mode, Mode::Table);
}

// ---- 3. 異なる値が多い列は1行の入力(セルの編集と同じ) ----

#[test]
fn test_ce_26_text_ask_too_many_values_is_one_line() {
    // [CE-26][CE-3] status の異なる値が 21 個(既定の 20 より多い)→ セルの編集と同じく候補のリストは出さず
    // 1行の入力。打った値で作る。
    let tmp = Tmp::new("nng_many");
    for i in 0..21 {
        tmp.write(&format!("n{i:02}.md"), &format!("---\nstatus: s{i}\n---\n"));
    }
    let mut a = boot(&tmp, config("[new_note]\nask = [\"status\"]\n"));
    let cell = cell_edit_list(&mut a, "n00.md", "status");
    assert!(
        cell.is_empty(),
        "セルの編集もリストを出さない(前提): {cell:?}"
    );
    start_and_name(&mut a, "多い");
    let s = screen(&a);
    assert!(s.contains("新しいノート status"), "{s}");
    assert!(
        list_on_screen(&s).is_empty(),
        "候補のリストは出さない:\n{s}"
    );
    typing(&mut a, "s99");
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&tmp, "多い.md"), "---\nstatus: s99\n---\n");
    assert_eq!(a.mode, Mode::Table);
}
