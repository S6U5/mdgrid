//! 新しいノートの画面の追加の試験(レビューの差し戻し 1)。CE-25・CE-26・CE-27(関係 SR-1・SR-4・SR-14)。
//! 聞く項目を空にしたら前もって入れる値も書かない、キーを外してもパレットから始められる、Shift+Tab で
//! 前の項目へ戻る、下の帯とメッセージ行の案内、低い端末・2つのフォルダでは始めない。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, ctrl, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::settings::{Cond, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use mdgrid::views::{save_views, NativeView};
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const TODAY: &str = "2026-10-02";

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"priority": "number", "done": "checkbox"}}"#,
    )
    .unwrap();
    tmp.write("a.md", "---\nstatus: todo\npriority: 3\n---\n");
    tmp.write("b.md", "---\nstatus: done\npriority: 1\n---\n");
    tmp
}

fn config(toml: &str) -> Config {
    let (c, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    c
}

fn boot_with(tmp: &Tmp, folders: &[PathBuf], target: PathBuf, cfg: Config) -> App {
    let src = Markdown::open(folders).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app.today = types::parse_date(TODAY).unwrap();
    app
}

fn boot(tmp: &Tmp, cfg: Config) -> App {
    boot_with(tmp, &[tmp.notes()], tmp.notes(), cfg)
}

fn message(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

/// 下の帯(「未保存」で始まる行)とメッセージ行(その次の行)。
fn bottom(a: &App) -> (String, String) {
    let s = screen(a);
    let lines: Vec<&str> = s.lines().collect();
    let i = lines
        .iter()
        .position(|l| l.trim_start().starts_with("未保存"))
        .unwrap_or_else(|| panic!("下の帯が無い:\n{s}"));
    let msg = lines.get(i + 1).copied().unwrap_or("");
    (lines[i].to_string(), msg.to_string())
}

#[test]
fn test_ce_26_cleared_filter_value_is_not_written() {
    // [CE-26][CE-25] 「status が todo」のビューで status を聞く → 前もって todo が入って出る。空にして進める
    // → status は書かない。作った行はビューから外れるので「見えない」と知らせる。
    let tmp = vault("nnm_clear");
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    let view = NativeView {
        name: "やること".into(),
        settings: Settings {
            filters: vec![Cond {
                col: "status".into(),
                op: Op::Keep(vec![Some("todo".into())]),
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    save_views(&tmp.0.join("config"), &tmp.notes(), &[view]).unwrap();
    let cfg = config("[new_note]\nask = [\"status\"]\n[new_note.set]\nkind = \"memo\"\n");
    let mut a = boot(&tmp, cfg);
    for _ in 0..5 {
        if screen(&a)
            .lines()
            .nth(1)
            .unwrap_or("")
            .contains("[やること]")
        {
            break;
        }
        ch(&mut a, ']');
    }
    ch(&mut a, 'a');
    typing(&mut a, "空にする");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(
        s.lines()
            .any(|l| l.contains("> status") && l.contains("todo")),
        "前もって入れる値:\n{s}"
    );
    for _ in 0..4 {
        press(&mut a, KeyCode::Backspace);
    }
    press(&mut a, KeyCode::Enter);
    let text = std::fs::read_to_string(tmp.notes().join("空にする.md")).unwrap();
    assert_eq!(text, "---\nkind: memo\n---\n", "status は書かない");
    assert_eq!(a.mode, Mode::Table);
    assert!(message(&a).contains("見えない"), "{:?}", message(&a));
}

#[test]
fn test_ce_25_palette_after_key_removed() {
    // [CE-25][SR-14] `a` を外しても、パレットの「新しいノート」から始められる。
    let tmp = vault("nnm_pal");
    let mut a = boot(&tmp, config("[keys.table]\n\"a\" = \"none\"\n"));
    ch(&mut a, 'a');
    assert_eq!(a.mode, Mode::Table, "`a` は外した");
    ch(&mut a, ':');
    typing(&mut a, "新しいノート");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "パレットから名前の欄:\n{}", screen(&a));
    typing(&mut a, "パレット");
    ctrl(&mut a, 's'); // CE-26: 窓のどの欄からでも作る
    assert!(tmp.notes().join("パレット.md").is_file());
    // 候補は1つだけ(キーの表とコマンドの表で重ならない)。
    let n = help::candidates(&a, "新しいノート")
        .iter()
        .filter(|c| c.label == "新しいノート")
        .count();
    assert_eq!(n, 1);
}

#[test]
fn test_ce_26_shift_tab_goes_back_to_name() {
    // [CE-26] 聞く項目で Shift+Tab → 名前の欄に戻る(打った名前のまま)。直して進めると、その名前で作る。
    let tmp = vault("nnm_back");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"priority\"]\n"));
    ch(&mut a, 'a');
    typing(&mut a, "戻る");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "5");
    press(&mut a, KeyCode::BackTab);
    assert!(
        screen(&a)
            .lines()
            .any(|l| l.contains("> 名前") && l.contains("戻る")),
        "{}",
        screen(&a)
    );
    typing(&mut a, "2");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "7");
    press(&mut a, KeyCode::Enter);
    let text = std::fs::read_to_string(tmp.notes().join("戻る2.md")).unwrap();
    assert_eq!(text, "---\npriority: 7\n---\n");
}

#[test]
fn test_ce_25_bands_show_new_note_keys() {
    // [CE-25][SR-1] 下の帯は新しいノートの入力のキー(次へ・やめる)。メッセージ行にセルの案内は出さない。
    // チェックの項目の案内に「↑↓ でリスト」は出さない。
    let tmp = vault("nnm_bands");
    let mut a = boot(&tmp, config("[new_note]\nask = [\"done\"]\n"));
    ch(&mut a, 'a');
    let (foot, msg) = bottom(&a);
    assert!(
        foot.contains("Enter 次へ") && foot.contains("Esc やめる"),
        "{foot}"
    );
    assert!(
        msg.trim().is_empty(),
        "名前の欄の間はメッセージ行は空: {msg:?}"
    );
    typing(&mut a, "チェック");
    press(&mut a, KeyCode::Enter);
    let (foot, msg) = bottom(&a);
    assert!(
        foot.contains("Enter 作る") && foot.contains("前の項目"),
        "{foot}"
    );
    assert!(
        msg.contains("true か false") && !msg.contains("リスト"),
        "{msg}"
    );
}

#[test]
fn test_ce_25_low_terminal_does_not_start() {
    // [CE-25][SR-9] 欄を出せない低い端末では始めず、理由を出す。
    let tmp = vault("nnm_low");
    let mut a = boot(&tmp, Config::default());
    a.resize(80, 6);
    ch(&mut a, 'a');
    assert_eq!(a.mode, Mode::Table);
    assert!(message(&a).contains("端末が低くて"), "{:?}", message(&a));
}

#[test]
fn test_ce_25_two_folders_have_no_button() {
    // [CE-25] フォルダを2つ開いても「+ 新規」を出し、`a` は名前の欄の前に作る場所を選ぶ欄を出す
    // (new-note-folders・new-note-folders-order。前の振る舞い「出さず理由を出す」から変えた)。
    let tmp = vault("nnm_two");
    let other = tmp.0.join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("c.md"), "---\nx: 1\n---\n").unwrap();
    let target = PathBuf::from(format!("{}\n{}", tmp.notes().display(), other.display()));
    let mut a = boot_with(&tmp, &[tmp.notes(), other], target, Config::default());
    assert!(screen(&a).lines().next().unwrap().contains("+ 新規"));
    ch(&mut a, 'a');
    assert_ne!(a.mode, Mode::Table, "作る場所を選ぶ欄:\n{}", screen(&a));
    assert!(screen(&a).contains("作る場所"), "{}", screen(&a));
}

#[test]
fn test_ce_25_guide_shows_real_folder() {
    // [CE-25] 案内の行には実際のフォルダ(相対パスで開いても実際の名前)を出す。
    let tmp = vault("nnm_guide");
    let mut a = boot(&tmp, Config::default());
    ch(&mut a, 'a');
    let real = tmp.notes().canonicalize().unwrap();
    let name = real.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        screen(&a)
            .lines()
            .any(|l| l.contains(&name) && l.contains("に作る")),
        "{}",
        screen(&a)
    );
}
