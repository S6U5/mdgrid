//! 名前を付けたセッション(cli.session-*)の画面の側: パレットの「セッションを保存」、同じ名前の置き換えの確かめ、
//! 開いたときの当て方(列の並びと隠す列を見た目の状態より先に・絞り込み・並び)。main と同じ道筋
//! (`App::new` → `start` → `apply_session` → 読み込み)で開く。期待値は要求の受け入れの例から取った。

use super::keymap::Mode;
use super::startup::Startup;
use super::test_grid::labels;
use super::test_screen::{ch, col_named, press, screen, typing, Tmp};
use super::*;
use mdgrid::config::{self, ViewState};
use mdgrid::session::{self, Filter, Session};
use mdgrid::settings::Dir;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const CONF: &str = "\
[conditions.soon]
expr = 'priority <= 2'
key = 1
[conditions.old]
expr = 'status == \"済\"'
hide = true
key = 9
";

/// 会議のメモ(a・進行中・10-05)・買い物(b・完了・09-01)・会議の準備(c・進行中・10-01)・読書(d・進行中・11-01)・
/// 片付け(e・済。いつも隠す条件 old に当たる)。
fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    tmp.write(
        "a.md",
        "---\ntitle: 会議のメモ\nstatus: 進行中\ndue: 2026-10-05\npriority: 1\n---\n",
    );
    tmp.write(
        "b.md",
        "---\ntitle: 買い物\nstatus: 完了\ndue: 2026-09-01\npriority: 2\n---\n",
    );
    tmp.write(
        "c.md",
        "---\ntitle: 会議の準備\nstatus: 進行中\ndue: 2026-10-01\npriority: 2\n---\n",
    );
    tmp.write(
        "d.md",
        "---\ntitle: 読書\nstatus: 進行中\ndue: 2026-11-01\npriority: 3\n---\n",
    );
    tmp.write(
        "e.md",
        "---\ntitle: 片付けの会議\nstatus: 済\ndue: 2026-08-01\npriority: 1\n---\n",
    );
    std::fs::create_dir_all(tmp.0.join("config")).unwrap();
    tmp
}

fn cfg(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

/// main と同じ道筋で開く(`session` があれば、開いてから当てる。`readonly` は `--readonly`)。
fn boot_with(tmp: &Tmp, session: Option<&Session>, readonly: bool) -> App {
    let (config, warnings) = config::parse(CONF).expect("読める設定");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config,
        warnings,
        readonly: readonly || session.is_some_and(|s| s.readonly),
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(cfg(tmp)),
        target: tmp.notes(),
        base: None,
    });
    if let Some(s) = session {
        app.apply_session(s, Vec::new());
    }
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

fn boot(tmp: &Tmp) -> App {
    boot_with(tmp, None, false)
}

fn palette(a: &mut App, cmd: &str) {
    ch(a, ':');
    typing(a, cmd);
    press(a, KeyCode::Enter);
}

/// 名前の欄に名前を打って決める(前もって入った文字は消す)。
fn name_it(a: &mut App, name: &str) {
    if let Some(p) = &mut a.palette {
        p.query.clear();
    }
    typing(a, name);
    press(a, KeyCode::Enter);
}

/// ノート(ファイルの名前)の行を選ぶ。
fn select(a: &mut App, tmp: &Tmp, file: &str) {
    assert!(a.select_note(&tmp.notes().join(file)), "{file} の行が無い");
}

/// 見本のセッション: status が進行中・「会議」を含む・due の昇順・title・status・due の3列(priority を隠す)。
fn sample(tmp: &Tmp) -> Session {
    Session {
        name: "進行中".into(),
        folders: vec![tmp.notes().canonicalize().unwrap()],
        columns: vec!["title".into(), "status".into(), "due".into()],
        hidden: vec!["priority".into()],
        sort: vec![("due".into(), Dir::Asc)],
        filters: vec![
            Filter::Value {
                key: "status".into(),
                value: "進行中".into(),
                by_keys: false,
                list: false,
            },
            Filter::Text("会議".into()),
        ],
        ..Session::default()
    }
}

/// 画面で、status の「進行中」で絞り、`\` で「会議」と打ち、due の昇順にし、priority を隠す(受け入れの例の操作)。
fn shape(a: &mut App, tmp: &Tmp) {
    select(a, tmp, "a.md");
    col_named(a, "status");
    ch(a, ',');
    ch(a, '\\');
    typing(a, "会議");
    press(a, KeyCode::Enter);
    col_named(a, "due");
    ch(a, 's');
    col_named(a, "priority");
    ch(a, '-');
    assert_eq!(a.cols, ["title", "status", "due"], "{}", screen(a));
}

#[test]
fn test_session_save_from_palette_and_open_again() {
    // [utest -> req~cli.session-save~1] 絞り込み・並び・列を決めて名前 `進行中` で保存 → 開き直すと同じ表。
    // [utest -> req~cli.session-screen-save~1] パレットの「セッションを保存」→ 名前 → sessions.toml に `[sessions.進行中]`。
    let tmp = vault("ses_save");
    let mut a = boot(&tmp);
    shape(&mut a, &tmp);
    let before_rows = labels(&a);
    let before_cols = a.cols.clone();
    assert_eq!(before_rows, ["c.md", "a.md"], "{}", screen(&a));
    palette(&mut a, "セッションを保存");
    assert_eq!(a.mode, Mode::Palette, "{}", screen(&a));
    assert!(screen(&a).contains("セッションの名前"), "{}", screen(&a));
    name_it(&mut a, "進行中");
    assert_eq!(a.mode, Mode::Table);
    assert!(
        a.message
            .as_deref()
            .unwrap_or("")
            .contains("セッション「進行中」を保存した"),
        "{:?}",
        a.message
    );
    let text = std::fs::read_to_string(cfg(&tmp).join(session::FILE_NAME)).unwrap();
    let t: toml::Table = text.parse().unwrap();
    assert!(
        t["sessions"].as_table().unwrap().contains_key("進行中"),
        "{text}"
    );
    let (list, warns) = session::load(&cfg(&tmp)).unwrap();
    assert!(warns.is_empty(), "{warns:?}");
    let s = list.into_iter().find(|s| s.name == "進行中").unwrap();
    let b = boot_with(&tmp, Some(&s), false);
    assert_eq!(labels(&b), before_rows, "{}", screen(&b));
    assert_eq!(b.cols, before_cols);
    assert_eq!(b.filter.as_deref(), Some("会議"));
    assert_eq!(b.same.len(), 1);
    assert_eq!(b.settings.sorts, vec![("due".to_string(), Dir::Asc)]);
}

#[test]
fn test_session_keeps_every_screen_filter() {
    // [utest -> req~cli.session-content-fields~2] 検索・同じ値・式の絞り込み・数字のキーの条件・止めたいつも隠す条件と、
    // 並び・列・読むだけを持つ(値は持たない)。
    let tmp = vault("ses_fields");
    let mut a = boot_with(&tmp, None, true);
    a.filter = Some("会議".into());
    a.run_expr_filter("due >= \"2026-08-01\"");
    ch(&mut a, '1');
    ch(&mut a, '9');
    let s = a.current_session("全部".into());
    assert_eq!(s.folders, [tmp.notes().canonicalize().unwrap()]);
    assert_eq!(s.base, None);
    assert!(s.readonly);
    assert_eq!(
        s.filters,
        vec![
            Filter::Text("会議".into()),
            Filter::Expr("due >= \"2026-08-01\"".into()),
            Filter::Cond("soon".into()),
            Filter::Show("old".into()),
        ]
    );
    let text = session::block(&s);
    assert!(!text.contains("会議のメモ"), "値を持たない: {text}");
    // 開き直すと同じ絞り込み: 会議を含み、priority が 2 以下で、いつも隠す old の行(片付けの会議)も出る。
    let b = boot_with(&tmp, Some(&s), false);
    let mut got = labels(&b);
    got.sort();
    assert_eq!(got, ["a.md", "c.md", "e.md"], "{}", screen(&b));
    assert!(b.readonly, "読むだけで開く");
    assert_eq!(b.cond.on, ["soon"]);
    assert!(b.cond.shown.contains("old"));
    assert!(b.cond.expr.is_some());
}

#[test]
fn test_session_columns_before_view_state() {
    // [utest -> req~cli.session-content-columns-first~1] 見た目の状態で due を隠した表を、title・status・due の
    // 3列のセッションで開く → 3列が出る。
    let tmp = vault("ses_cols");
    let state = tmp.0.join("state");
    let hide_due = ViewState {
        order: vec![
            "due".into(),
            "priority".into(),
            "status".into(),
            "title".into(),
        ],
        hidden: vec!["due".into()],
        ..ViewState::default()
    };
    config::save_state(&state, &tmp.notes(), "", &hide_due).unwrap();
    let plain = boot(&tmp);
    assert!(
        !plain.cols.contains(&"due".to_string()),
        "状態で due を隠した"
    );
    drop(plain);
    let s = Session {
        filters: Vec::new(),
        sort: Vec::new(),
        ..sample(&tmp)
    };
    let b = boot_with(&tmp, Some(&s), false);
    assert_eq!(b.cols, ["title", "status", "due"], "{}", screen(&b));
    assert!(b
        .message
        .as_deref()
        .is_none_or(|m| !m.contains("今の表に無い")));
}

#[test]
fn test_session_missing_column_is_skipped_with_a_warning() {
    // [utest -> req~cli.session-content-columns-first~1] セッションにあって今の表に無い列 → その列だけ飛ばして警告。
    let tmp = vault("ses_missing");
    let s = Session {
        columns: vec!["title".into(), "owner".into(), "due".into()],
        filters: Vec::new(),
        sort: Vec::new(),
        ..sample(&tmp)
    };
    let b = boot_with(&tmp, Some(&s), false);
    assert_eq!(b.cols.first().map(String::as_str), Some("title"));
    assert!(!b.cols.contains(&"owner".to_string()));
    assert!(b.cols.contains(&"due".to_string()));
    assert!(
        b.message
            .as_deref()
            .unwrap_or("")
            .contains("セッションの列 owner は今の表に無いので飛ばした"),
        "{:?}",
        b.message
    );
}

#[test]
fn test_session_values_come_from_the_notes() {
    // [utest -> req~cli.session-content-fresh-values~1] 保存したあとノートの値を直して開く → 直した値が出る。
    let tmp = vault("ses_fresh");
    let s = sample(&tmp);
    tmp.write(
        "a.md",
        "---\ntitle: 会議のメモ(改)\nstatus: 進行中\ndue: 2026-10-05\npriority: 1\n---\n",
    );
    let b = boot_with(&tmp, Some(&s), false);
    assert!(screen(&b).contains("会議のメモ(改)"), "{}", screen(&b));
}

#[test]
fn test_session_overwrite_asks_and_no_keeps_the_old_one() {
    // [utest -> req~cli.session-screen-overwrite~1] 同じ名前で2回保存 → 2回目に置き換えの確かめ、断ると元のまま、
    // y で置き換える。
    let tmp = vault("ses_over");
    let mut a = boot(&tmp);
    palette(&mut a, "save_session");
    name_it(&mut a, "進行中");
    let first = std::fs::read(cfg(&tmp).join(session::FILE_NAME)).unwrap();
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    palette(&mut a, "save_session");
    name_it(&mut a, "進行中");
    assert_eq!(a.mode, Mode::Palette);
    assert!(
        screen(&a).contains("セッション「進行中」はもうある。置き換える?"),
        "{}",
        screen(&a)
    );
    typing(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(
        std::fs::read(cfg(&tmp).join(session::FILE_NAME)).unwrap(),
        first,
        "断ると元のまま"
    );
    palette(&mut a, "save_session");
    name_it(&mut a, "進行中");
    typing(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    let (list, _) = session::load(&cfg(&tmp)).unwrap();
    assert_eq!(list[0].filters, vec![Filter::Text("会議".into())]);
}

#[test]
fn test_session_store_writes_no_note_file() {
    // [utest -> req~cli.session-store~1] 保存しても、ノートのフォルダに新しいファイルは無い(読むだけの起動でも保存できる)。
    let tmp = vault("ses_store");
    let before: Vec<_> = std::fs::read_dir(tmp.notes())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let mut a = boot_with(&tmp, None, true);
    palette(&mut a, "save_session");
    name_it(&mut a, "見るだけ");
    let mut after: Vec<_> = std::fs::read_dir(tmp.notes())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let mut before = before;
    before.sort();
    after.sort();
    assert_eq!(before, after);
    let (list, _) = session::load(&cfg(&tmp)).unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].readonly);
}

#[test]
fn test_session_name_prefilled_and_empty_name_refused() {
    // [utest -> req~cli.session-screen-save~1] 開いたセッションの名前が名前の欄に入って始まり、空の名前は保存しない。
    let tmp = vault("ses_name");
    let s = sample(&tmp);
    let mut a = boot_with(&tmp, Some(&s), false);
    palette(&mut a, "save_session");
    assert_eq!(a.palette.as_ref().unwrap().query, "進行中");
    name_it(&mut a, "  ");
    assert_eq!(a.mode, Mode::Palette, "空の名前では閉じない");
    assert!(a
        .message
        .as_deref()
        .unwrap_or("")
        .contains("セッションの名前が空"));
    assert!(!cfg(&tmp).join(session::FILE_NAME).exists());
}
