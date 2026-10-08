//! 登録した表(CLI-18・CLI-19): パレットの「この表を登録」と「登録した表を開く」、開き直す前の確かめ。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::places::{self, Place};
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write("a.md", "---\nstatus: todo\n---\n");
    tmp.write("b.md", "---\nstatus: done\n---\n");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn boot(tmp: &Tmp) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config: Config::default(),
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
    app
}

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

fn clear_query(a: &mut App) {
    if let Some(p) = &mut a.palette {
        p.query.clear();
    }
}

fn place(name: &str, group: &str, path: PathBuf) -> Place {
    Place {
        name: name.into(),
        group: group.into(),
        path,
        view: None,
    }
}

#[test]
fn test_cli_18_register_from_palette() {
    // [CLI-18] 「この表を登録」→ 名前の欄に今のフォルダの名前 → 名前を打つ → 分類を新しく打つ → places.toml。
    let tmp = vault("pl_reg");
    let mut a = boot(&tmp);
    palette(&mut a, "この表を登録");
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    assert_eq!(
        a.palette.as_ref().unwrap().query,
        "notes",
        "前もって入る名前"
    );
    assert!(screen(&a).contains("この表の名前"), "{}", screen(&a));
    clear_query(&mut a);
    typing(&mut a, "タスク");
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("分類"), "{}", screen(&a));
    typing(&mut a, "仕事");
    assert!(screen(&a).contains("+ 新しい分類: 仕事"), "{}", screen(&a));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains("タスク を 仕事 に登録した"),
        "{:?}",
        a.message
    );
    let (got, _) = places::load(&tmp.0.join("config"));
    let real = tmp.notes().canonicalize().unwrap();
    assert_eq!(got, vec![place("タスク", "仕事", real)]);
    assert_eq!(a.registered.len(), 1, "登録したらすぐ一覧に出る");
}

#[test]
fn test_cli_18_pick_existing_group_and_overwrite() {
    // [CLI-18] 分類は今の分類から選べる(空のまま ↓ で選ぶ)。同じ名前は y で置き換え、ほかの答えでやめる。
    let tmp = vault("pl_over");
    let cfg = tmp.0.join("config");
    places::save(&cfg, place("タスク", "仕事", PathBuf::from("/old"))).unwrap();
    places::save(&cfg, place("本", "趣味", PathBuf::from("/b"))).unwrap();
    let mut a = boot(&tmp);
    palette(&mut a, "この表を登録");
    clear_query(&mut a);
    typing(&mut a, "タスク");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(
        s.contains("仕事") && s.contains("趣味") && s.contains("(分類なし)"),
        "{s}"
    );
    press(&mut a, KeyCode::Down); // 仕事 → 趣味
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("もう登録してある"), "{}", screen(&a));
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        places::load(&cfg).0[0].path,
        PathBuf::from("/old"),
        "やめたら書かない"
    );
    palette(&mut a, "この表を登録");
    clear_query(&mut a);
    typing(&mut a, "タスク");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    let got = places::load(&cfg).0;
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].name, "タスク");
    assert_eq!(got[0].group, "趣味");
    assert_eq!(got[0].path, tmp.notes().canonicalize().unwrap());
}

#[test]
fn test_cli_18_empty_name_stays() {
    // [CLI-18] 名前が空なら理由を出して名前の欄に残る。
    let tmp = vault("pl_empty");
    let mut a = boot(&tmp);
    palette(&mut a, "この表を登録");
    clear_query(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette);
    assert!(a.message.as_deref().unwrap_or("").contains("名前が空"));
}

#[test]
fn test_cli_19_open_list_and_switch() {
    // [CLI-19] 一覧は分類ごと(places.toml の順で分類が出た順)、先頭に今のフォルダ。打つと絞る(大文字小文字を
    // 問わない)。無いパスは理由を出して一覧に残る。在る表を選ぶと終わって開き直す(switch_to)。
    let tmp = vault("pl_open");
    let cfg = tmp.0.join("config");
    let other = tmp.0.join("Other");
    std::fs::create_dir_all(&other).unwrap();
    places::save(&cfg, place("Books", "趣味", other.clone())).unwrap();
    places::save(&cfg, place("無い", "仕事", tmp.0.join("gone"))).unwrap();
    places::save(&cfg, place("Tasks", "趣味", tmp.notes())).unwrap();
    let mut a = boot(&tmp);
    a.start_open_places(true);
    let s = screen(&a);
    let pos = |t: &str| s.find(t).unwrap_or_else(|| panic!("{t} が無い:\n{s}"));
    assert!(pos("今のフォルダ") < pos("趣味 › Books"));
    assert!(
        pos("趣味 › Books") < pos("趣味 › Tasks"),
        "同じ分類はまとめる"
    );
    assert!(pos("趣味 › Tasks") < pos("仕事 › 無い"));
    typing(&mut a, "無い");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Palette, "無いパスでは一覧に残る");
    assert!(a.message.as_deref().unwrap_or("").contains("開けない"));
    clear_query(&mut a);
    typing(&mut a, "books");
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "変更が無ければすぐ終わる");
    assert_eq!(a.switch_to.as_ref().map(|p| p.path.clone()), Some(other));
}

#[test]
fn test_cli_19_here_closes_list() {
    // [CLI-19] 先頭の「今のフォルダ」は一覧を閉じるだけ。
    let tmp = vault("pl_here");
    places::save(&tmp.0.join("config"), place("x", "", tmp.notes())).unwrap();
    let mut a = boot(&tmp);
    a.start_open_places(true);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(!a.quit && a.switch_to.is_none());
}

#[test]
fn test_cli_19_pending_changes_ask_first() {
    // [CLI-19][WB-11] ためた変更があれば、保存する・捨てる・戻るを確かめる。戻ると移らない。
    let tmp = vault("pl_pending");
    let other = tmp.0.join("Other");
    std::fs::create_dir_all(&other).unwrap();
    places::save(&tmp.0.join("config"), place("O", "", other)).unwrap();
    let mut a = boot(&tmp);
    let row = a.src.rows()[0].clone();
    a.changes
        .set(
            a.src.as_ref(),
            &row,
            "status",
            mdgrid::source::NewValue::Str("x".into()),
        )
        .unwrap();
    palette(&mut a, "登録した表を開く");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.switch_to.is_none(), "戻ったら移らない");
    // そのあと普通に q で終わっても、前に選んだ表へは移らない。
    a.changes = Default::default();
    ch(&mut a, 'q');
    assert!(a.quit && a.switch_to.is_none());
}

#[test]
fn test_cli_19_no_places_message() {
    // [CLI-19] 登録が無ければ一覧を出さず、登録のしかたを出す。
    let tmp = vault("pl_none");
    let mut a = boot(&tmp);
    palette(&mut a, "登録した表を開く");
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.as_deref().unwrap_or("").contains("この表を登録"));
}
