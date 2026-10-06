//! 表から新しいノートを作る(new-note)の画面の受け入れ試験(タスク 2)。CE-25・CE-26・CE-27
//! (関係: CE-2・CE-20・CE-16・WB-15・SR-4・SR-14)。
//! 仕様: specs/cell-edit/spec.md、記録 specs/_changes/2026-10-02-new-note.md。
//! 画面の実装を見ずに、キー・クリック・画面の文字・ファイルで確かめる。今日は 2026-10-02 に固定する。
//!
//! 仮定(実装役に渡す):
//! - 起動は main と同じ道筋(`App::new` → `start(Startup)` → 読み込み)。作る場所の既定は `Startup::target`
//!   (`.base` なしなら開いたフォルダ)。設定(`[new_note]`・`[keys.table]`)は `Startup::config` で渡す。
//! - ヘッダー(1行目)の右の端に「+ 新規」の文字。クリックで名前の欄が開く。
//! - 名前の欄・聞く項目の入力の間は、モードが表(`Mode::Table`)でない。作り終えるか Esc で表に戻る。
//! - 名前の欄は雛形(無ければ空)で始まり、打った文字は画面に出る。
//! - パレット(`:`)のコマンドの名前は「新しいノート」。ヘルプにも「新しいノート」の行がある。
//! - 作れないときの理由はメッセージ行(`App::message`)に出て、核の理由の「作らない」を含む。
//! - 読むだけのときのメッセージは「読むだけ」を含む(既存の READONLY の文)。
//! - 日付の列を聞くときは、セルの編集(CE-20)と同じく入力の下にカレンダー(`2026年10月` の見出しと曜日)が出る。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::settings::{Cond, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::types;
use mdgrid::views::{save_views, NativeView};
use ratatui::crossterm::event::KeyCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const TODAY: &str = "2026-10-02";

// ---- 材料 ----

/// 3つのノートと types.json(priority は数、due は日付)。
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
    tmp.write("c.md", "---\ntitle: メモ\nstatus: todo\n---\n");
    tmp
}

fn conf_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

fn state_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("state")
}

/// 設定の文を読む(警告なし)。
fn config(toml: &str) -> Config {
    let (c, warnings) = mdgrid::config::parse(toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    c
}

/// main と同じ道筋で、`.base` なしのフォルダを開く。利用者の ~/.config には触らない。
fn boot(tmp: &Tmp, cfg: Config, readonly: bool) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.today = types::parse_date(TODAY).unwrap();
    app.start(Startup {
        config: cfg,
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(state_dir(tmp)),
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

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.clone(), std::fs::read(&p).unwrap());
            }
        }
    }
    out
}

fn read(tmp: &Tmp, rel: &str) -> String {
    let p = tmp.notes().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|_| panic!("{} が無い", p.display()))
}

fn labels(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

fn selected_label(a: &App) -> String {
    a.src.label(&a.cur_row().expect("行が選ばれている"))
}

fn message(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

fn header(a: &App) -> String {
    screen(a).lines().next().unwrap_or("").to_string()
}

/// 画面の文字 `label` をクリックする。
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

/// パレットのコマンドを、画面に出る名前で実行する。
fn palette(a: &mut App, label: &str) {
    ch(a, ':');
    typing(a, label);
    press(a, KeyCode::Enter);
}

/// 名前の欄が開いている(表のモードでない)。
fn assert_opened(a: &App, how: &str) {
    assert_ne!(
        a.mode,
        Mode::Table,
        "{how} で名前の欄が開く:\n{}",
        screen(a)
    );
}

/// 名前を打って Enter。
fn name_enter(a: &mut App, name: &str) {
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// 作ったあと: 表に戻り、その行が表にあって選ばれている。未保存は増えない。
fn assert_created_and_selected(a: &App, rel: &str) {
    assert_eq!(a.mode, Mode::Table, "作ったら表に戻る:\n{}", screen(a));
    assert!(
        labels(a).iter().any(|l| l == rel),
        "表に {rel} の行がある: {:?}",
        labels(a)
    );
    assert_eq!(selected_label(a), rel, "作った行が選ばれている");
    assert_eq!(a.changes.count(), 0, "その場で作るので、ためる変更は無い");
    assert!(screen(a).contains("未保存 0"), "{}", screen(a));
}

fn status_todo_view() -> NativeView {
    NativeView {
        name: "やること".into(),
        settings: Settings {
            filters: vec![Cond {
                col: "status".into(),
                op: Op::Keep(vec![Some("todo".into())]),
            }],
            ..Default::default()
        },
        ..Default::default()
    }
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

// ---- CE-25: 始め方(「+ 新規」・a・パレット)と作る ----

#[test]
fn test_ce_25_header_button_click_creates_note() {
    // [CE-25] ヘッダーの行の右の端に「+ 新規」。クリックで名前の欄 → 名前を打って Enter →
    // 開いたフォルダに `<名前>.md` ができ、表にその行が出て選ばれている。未保存は増えない。
    let tmp = vault("ce25click");
    let mut a = boot(&tmp, Config::default(), false);
    let h = header(&a);
    assert!(
        h.trim_end().ends_with("+ 新規"),
        "ヘッダーの右の端に「+ 新規」: {h:?}"
    );
    click_label(&mut a, "+ 新規");
    assert_opened(&a, "「+ 新規」のクリック");
    name_enter(&mut a, "会議の準備");
    assert!(tmp.notes().join("会議の準備.md").is_file());
    assert_created_and_selected(&a, "会議の準備.md");
}

#[test]
fn test_ce_25_key_a_creates_note() {
    // [CE-25][CE-27] 表のモードの `a` で名前の欄。作るのは新しいファイルだけで、中身は空(値が無いので)。
    let tmp = vault("ce25a");
    let mut a = boot(&tmp, Config::default(), false);
    let before = snapshot(&tmp.notes());
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    name_enter(&mut a, "買い物");
    assert_created_and_selected(&a, "買い物.md");
    let after = snapshot(&tmp.notes());
    for (p, bytes) in &before {
        assert_eq!(
            after.get(p),
            Some(bytes),
            "既にあるファイルは変えない: {p:?}"
        );
    }
    assert_eq!(after.len(), before.len() + 1, "増えたのは1つだけ");
    assert!(
        !read(&tmp, "買い物.md").contains("status"),
        "絞り込みの無いビューでは値を入れない"
    );
}

#[test]
fn test_ce_25_palette_creates_note() {
    // [CE-25][SR-14] パレット(`:`)の「新しいノート」でも名前の欄が開く。
    let tmp = vault("ce25pal");
    let mut a = boot(&tmp, Config::default(), false);
    palette(&mut a, "新しいノート");
    assert_opened(&a, "パレットの「新しいノート」");
    name_enter(&mut a, "パレットから");
    assert!(tmp.notes().join("パレットから.md").is_file());
    assert_created_and_selected(&a, "パレットから.md");
}

#[test]
fn test_ce_25_view_filter_value_is_prefilled_and_row_stays() {
    // [CE-25] 「status が todo」のビュー(Keep の値1つ)で作る → `---\nstatus: todo\n---\n` で、
    // 行がそのビューに残って選ばれる。
    let tmp = vault("ce25view");
    std::fs::create_dir_all(conf_dir(&tmp)).unwrap();
    save_views(&conf_dir(&tmp), &tmp.notes(), &[status_todo_view()]).unwrap();
    let mut a = boot(&tmp, Config::default(), false);
    goto_tab(&mut a, "やること");
    assert!(!labels(&a).contains(&"b.md".to_string()), "done は見えない");
    click_label(&mut a, "+ 新規");
    assert_opened(&a, "「+ 新規」のクリック");
    name_enter(&mut a, "会議の準備");
    assert_eq!(read(&tmp, "会議の準備.md"), "---\nstatus: todo\n---\n");
    assert_created_and_selected(&a, "会議の準備.md");
    assert!(
        screen(&a)
            .lines()
            .nth(1)
            .unwrap_or("")
            .contains("[やること]"),
        "ビューはそのまま:\n{}",
        screen(&a)
    );
}

#[test]
fn test_ce_25_subfolder_name_creates_folder() {
    // [CE-25] `プロジェクト/新しい` → 下のフォルダ(無ければ作る)に作る。
    let tmp = vault("ce25sub");
    let mut a = boot(&tmp, Config::default(), false);
    ch(&mut a, 'a');
    name_enter(&mut a, "プロジェクト/新しい");
    assert!(tmp.notes().join("プロジェクト/新しい.md").is_file());
    assert_created_and_selected(&a, "プロジェクト/新しい.md");
}

#[test]
fn test_ce_25_existing_name_refused_and_retype() {
    // [CE-25][WB-2] 既にある名前 → 作らず理由をメッセージに出し、欄は開いたまま。打ち直して作れる。
    let tmp = vault("ce25dup");
    let mut a = boot(&tmp, Config::default(), false);
    let before = snapshot(&tmp.notes());
    ch(&mut a, 'a');
    name_enter(&mut a, "a");
    assert_eq!(
        snapshot(&tmp.notes()),
        before,
        "既にあるファイルは変えず、何も作らない"
    );
    assert!(message(&a).contains("作らない"), "理由: {:?}", message(&a));
    assert_opened(&a, "作れなかったあと");
    // 打ち直す: `a` → `a2`。
    name_enter(&mut a, "2");
    assert!(tmp.notes().join("a2.md").is_file(), "打ち直した名前で作る");
    assert_created_and_selected(&a, "a2.md");
}

#[test]
fn test_ce_25_outside_folder_refused() {
    // [CE-25] `../外` → 開いたフォルダの外には作らず、理由を出し、欄は開いたまま。
    let tmp = vault("ce25out");
    let mut a = boot(&tmp, Config::default(), false);
    let before = snapshot(&tmp.0);
    ch(&mut a, 'a');
    name_enter(&mut a, "../外");
    assert!(!tmp.0.join("外.md").exists(), "外に作らない");
    assert_eq!(snapshot(&tmp.0), before, "何も作らない");
    assert!(message(&a).contains("作らない"), "理由: {:?}", message(&a));
    assert_opened(&a, "作れなかったあと");
    // Esc で取りやめ。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(snapshot(&tmp.0), before);
}

#[test]
fn test_ce_25_esc_creates_nothing() {
    // [CE-25] 名前を打って Esc → 何も作らず表に戻る。
    let tmp = vault("ce25esc");
    let mut a = boot(&tmp, Config::default(), false);
    let before = snapshot(&tmp.notes());
    let rows = labels(&a);
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    typing(&mut a, "取りやめ");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(snapshot(&tmp.notes()), before);
    assert_eq!(labels(&a), rows);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_25_readonly_creates_nothing() {
    // [CE-25][WB-15] `--readonly` では `a` もクリックも作らず「読むだけ」と出す。
    let tmp = vault("ce25ro");
    let mut a = boot(&tmp, Config::default(), true);
    let before = snapshot(&tmp.notes());
    ch(&mut a, 'a');
    assert_eq!(a.mode, Mode::Table, "名前の欄は開かない:\n{}", screen(&a));
    assert!(message(&a).contains("読むだけ"), "{:?}", message(&a));
    a.message = None;
    if header(&a).contains("+ 新規") {
        click_label(&mut a, "+ 新規");
        assert_eq!(a.mode, Mode::Table, "クリックでも開かない:\n{}", screen(&a));
        assert!(message(&a).contains("読むだけ"), "{:?}", message(&a));
    }
    // 開いていたとしても、打って Enter で作らない。
    typing(&mut a, "x");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Esc);
    assert_eq!(snapshot(&tmp.notes()), before, "何も作らない");
}

// ---- CE-26: 設定で聞く項目を1つずつ、型の入力で ----

#[test]
fn test_ce_26_ask_number_then_date_with_calendar() {
    // [CE-26][CE-2][CE-20] `ask = ["priority","due"]` → 名前の Enter のあと priority の入力、
    // Enter で due の入力(カレンダーが出る)、Enter で作る。priority と due(囲まない日付)が入る。
    let tmp = vault("ce26ask");
    let cfg = config("[new_note]\nask = [\"priority\", \"due\"]\n");
    let mut a = boot(&tmp, cfg, false);
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    name_enter(&mut a, "聞く");
    assert!(
        !tmp.notes().join("聞く.md").exists(),
        "聞き終えるまで作らない"
    );
    assert_opened(&a, "priority の入力");
    assert!(screen(&a).contains("priority"), "{}", screen(&a));
    typing(&mut a, "2");
    press(&mut a, KeyCode::Enter);
    assert!(
        !tmp.notes().join("聞く.md").exists(),
        "due を聞き終えるまで作らない"
    );
    assert_opened(&a, "due の入力");
    let s = screen(&a);
    assert!(s.contains("2026年10月"), "due の入力の下にカレンダー:\n{s}");
    assert!(s.contains("日  月  火  水  木  金  土"), "{s}");
    typing(&mut a, "2026-10-05");
    press(&mut a, KeyCode::Enter);
    let text = read(&tmp, "聞く.md");
    assert!(
        text.starts_with("---\n") && text.ends_with("---\n"),
        "{text:?}"
    );
    assert!(text.contains("\npriority: 2\n"), "{text:?}");
    assert!(
        text.contains("\ndue: 2026-10-05\n"),
        "囲まない日付: {text:?}"
    );
    assert_created_and_selected(&a, "聞く.md");
}

#[test]
fn test_ce_26_empty_answer_is_not_written() {
    // [CE-26] priority を空で Enter → priority の行は無い。
    let tmp = vault("ce26empty");
    let cfg = config("[new_note]\nask = [\"priority\", \"due\"]\n");
    let mut a = boot(&tmp, cfg, false);
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    name_enter(&mut a, "空");
    press(&mut a, KeyCode::Enter); // priority は空
    assert!(!tmp.notes().join("空.md").exists());
    typing(&mut a, "2026-10-05");
    press(&mut a, KeyCode::Enter);
    let text = read(&tmp, "空.md");
    assert!(!text.contains("priority"), "{text:?}");
    assert_eq!(text, "---\ndue: 2026-10-05\n---\n");
    assert_created_and_selected(&a, "空.md");
}

#[test]
fn test_ce_26_esc_while_asking_creates_nothing() {
    // [CE-26] 聞いている途中の Esc → 何も作らない。
    let tmp = vault("ce26esc");
    let cfg = config("[new_note]\nask = [\"priority\"]\n");
    let mut a = boot(&tmp, cfg, false);
    let before = snapshot(&tmp.notes());
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    name_enter(&mut a, "途中");
    assert_opened(&a, "priority の入力");
    typing(&mut a, "4");
    press(&mut a, KeyCode::Esc);
    for _ in 0..3 {
        if a.mode == Mode::Table {
            break;
        }
        press(&mut a, KeyCode::Esc);
    }
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(snapshot(&tmp.notes()), before);
}

// ---- CE-27: 名前の雛形・割り当て直し・ヘルプ ----

#[test]
fn test_ce_27_name_template_has_today() {
    // [CE-27] `name = "{date} "` → 名前の欄が今日の日付と空白で始まる。続けて打った名前で作る。
    let tmp = vault("ce27name");
    let cfg = config("[new_note]\nname = \"{date} \"\n");
    let mut a = boot(&tmp, cfg, false);
    ch(&mut a, 'a');
    assert_opened(&a, "`a`");
    typing(&mut a, "日報");
    assert!(
        screen(&a).contains(&format!("{TODAY} 日報")),
        "名前の欄は `{TODAY} ` で始まる:\n{}",
        screen(&a)
    );
    press(&mut a, KeyCode::Enter);
    let rel = format!("{TODAY} 日報.md");
    assert!(tmp.notes().join(&rel).is_file());
    assert_created_and_selected(&a, &rel);
}

#[test]
fn test_ce_27_rebind_n_to_new_note() {
    // [CE-27][SR-4] `[keys.table] "n" = "new_note"` → `n` でも名前の欄が開く。
    let tmp = vault("ce27key");
    let cfg = config("[keys.table]\n\"n\" = \"new_note\"\n");
    let mut a = boot(&tmp, cfg, false);
    assert!(
        message(&a).is_empty(),
        "割り当ての警告は無い: {:?}",
        message(&a)
    );
    ch(&mut a, 'n');
    assert_opened(&a, "割り当て直した `n`");
    name_enter(&mut a, "n で");
    assert_created_and_selected(&a, "n で.md");
}

#[test]
fn test_ce_27_help_lists_new_note() {
    // [CE-27][SR-5] ヘルプに新しいノートの行(`a` と「新しいノート」)がある。
    let tmp = vault("ce27help");
    let mut a = boot(&tmp, Config::default(), false);
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    let mut seen = String::new();
    let mut last = String::new();
    for _ in 0..400 {
        let s = screen(&a);
        if s == last {
            break;
        }
        seen.push_str(&s);
        last = s;
        press(&mut a, KeyCode::Down);
    }
    let line = seen
        .lines()
        .find(|l| l.contains("新しいノート"))
        .expect("ヘルプに「新しいノート」の行がある");
    assert!(
        line.split_whitespace().any(|w| w == "a"),
        "その行に既定のキー `a`: {line:?}"
    );
}
