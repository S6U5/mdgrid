//! mdgrid のビュー(native-views)の画面の受け入れ試験(タスク 2)。BV-17〜BV-20
//! (と NV-13〜NV-22・WB-15・BV-3・BV-13・SR-12)。
//! 仕様: specs/base-view/spec.md、決定 specs/_decisions/2026-10-02-native-views.md と -details.md、
//! 設計 docs/design.md の「mdgrid のビュー(BV-17〜BV-20)」。実装を見ずに、画面の文字とキーとファイルで確かめる。
//!
//! 仮定(実装役に渡す):
//! - `Startup` に `config_dir: Option<PathBuf>` を足す。mdgrid のビューの置き場(`views.toml` を置くフォルダ。
//!   main は `$XDG_CONFIG_HOME/mdgrid` を渡す)。None なら mdgrid のビューを読まず書かない。
//!   試験は `<一時>/config` を渡し、利用者の `~/.config` には触らない。
//! - タブは2行目(0 から数えて1行目)。選んでいるタブは `[名前]`。タブの切り替えは既存の `[` `]`。
//! - ビューの設定の画面(`o`)に「名前を付けて保存」「上書き」「名前の変更」「削除」のボタン(文字をクリックで押せる)。
//!   名前は入力の欄で打って Enter。削除に確かめを出すなら Enter で決まる。
//! - パレット(`:`)の「.base に書き出す」(続けてファイルの名前を打って Enter)と
//!   「.base のビューを取り込む」(続けて .base のファイル → ビューを、打って絞り Enter で選ぶ)。
//! - 書き出しの `.base` は開いたフォルダ(ノートのフォルダ)の下に書く。

use super::keymap::Mode;
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{ch, press, screen, typing, Tmp};
use super::*;
use mdgrid::base;
use mdgrid::settings::{Cond, Dir, Group, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::views::{load_views, save_views, NativeView};
use ratatui::crossterm::event::KeyCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---- 材料 ----

/// 6つのノート(status は todo・done・doing・無し、種別 は 会議・本・メモ)と types.json。
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
        "---\ntitle: 会議の準備\nstatus: todo\n種別: 会議\npriority: 3\ndue: 2026-09-20\n---\n",
    );
    tmp.write(
        "b.md",
        "---\ntitle: 読んだ本\nstatus: done\n種別: 本\npriority: 1\n---\n",
    );
    tmp.write(
        "c.md",
        "---\ntitle: 次の本\nstatus: todo\n種別: 本\npriority: 5\ndue: 2026-10-10\n---\n",
    );
    tmp.write("d.md", "---\ntitle: メモ\nstatus: done\n種別: メモ\n---\n");
    tmp.write(
        "e.md",
        "---\ntitle: 会議の記録\nstatus: doing\n種別: 会議\npriority: 4\n---\n",
    );
    tmp.write("f.md", "---\ntitle: 本棚\n種別: 本\n---\n");
    tmp
}

/// ビューが3つの `.base`。
const BASE: &str = r#"views:
  - type: table
    name: 全部
    order: [title, status, 種別, priority]
  - type: table
    name: 進行
    filters: 'status != "doing"'
    order: [title, status, 種別, priority, due]
  - type: table
    name: 名前だけ
    order: [title]
"#;

fn base_path(tmp: &Tmp) -> PathBuf {
    let p = tmp.notes().join("tasks.base");
    if !p.exists() {
        std::fs::write(&p, BASE).unwrap();
    }
    p
}

/// 設定の置き場(views.toml を置くフォルダ)。
fn conf_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("config")
}

fn state_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("state")
}

fn views_toml(tmp: &Tmp) -> PathBuf {
    conf_dir(tmp).join("views.toml")
}

/// main と同じ道筋(`App::new` → `start` → 読み込み)で、`.base` なしのフォルダを開く。
fn boot_folder(tmp: &Tmp, readonly: bool) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
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
    app
}

/// main と同じ道筋で `.base` を開く。
fn boot_base(tmp: &Tmp, readonly: bool) -> App {
    let p = base_path(tmp);
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(state_dir(tmp)),
        config_dir: Some(conf_dir(tmp)),
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// 核の save_views で、対象の mdgrid のビューを用意する(画面を通さない準備)。
fn prepare(tmp: &Tmp, target: &Path, views: &[NativeView]) {
    std::fs::create_dir_all(conf_dir(tmp)).unwrap();
    save_views(&conf_dir(tmp), target, views).unwrap();
}

fn view(name: &str, settings: Settings) -> NativeView {
    NativeView {
        name: name.into(),
        settings,
        ..Default::default()
    }
}

fn drop_done() -> Settings {
    Settings {
        filters: vec![Cond {
            col: "status".into(),
            op: Op::Drop(vec![Some("done".into())]),
        }],
        ..Default::default()
    }
}

fn keep_book() -> Settings {
    Settings {
        filters: vec![Cond {
            col: "種別".into(),
            op: Op::Keep(vec![Some("本".into())]),
        }],
        ..Default::default()
    }
}

fn names_in_config(tmp: &Tmp) -> Vec<String> {
    let (v, _) = load_views(&conf_dir(tmp), &tmp.notes());
    v.into_iter().map(|v| v.name).collect()
}

fn labels(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
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

// ---- 画面の読み取りと操作(画面の文字とキーだけ) ----

/// タブの行(2行目)。
fn tabs(a: &App) -> String {
    screen(a).lines().nth(1).unwrap_or("").to_string()
}

/// `]` で名前のタブまで動かす(選んだタブは `[名前]`)。
fn goto_tab(a: &mut App, name: &str) {
    let want = format!("[{name}]");
    for _ in 0..12 {
        if tabs(a).contains(&want) {
            return;
        }
        ch(a, ']');
    }
    panic!("タブ「{name}」に移れない: {}", screen(a));
}

/// 画面の文字 `label` をクリックする(ボタン・項目)。
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

/// 入力の欄を空にして打ち、Enter で決める。
fn enter_text(a: &mut App, text: &str) {
    for _ in 0..40 {
        press(a, KeyCode::Backspace);
    }
    typing(a, text);
    press(a, KeyCode::Enter);
}

/// 確かめを出すなら Enter で決める。
fn confirm_if_asked(a: &mut App) {
    if a.mode != Mode::Table && a.mode != Mode::Settings {
        press(a, KeyCode::Enter);
    }
}

/// 表に戻る。
fn settle(a: &mut App) {
    for _ in 0..4 {
        if a.mode == Mode::Table {
            return;
        }
        press(a, KeyCode::Esc);
    }
}

/// パレットのコマンドを、画面に出る名前で実行する。
fn palette(a: &mut App, label: &str) {
    ch(a, ':');
    typing(a, label);
    press(a, KeyCode::Enter);
}

fn message(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

fn quit(a: &mut App) {
    settle(a);
    ch(a, 'q');
    assert!(a.quit, "終了できる: {}", screen(a));
}

// ---- ビューの設定の画面の操作(既存の設定の画面の試験と同じ道筋) ----

fn draft(a: &App) -> &super::settings::Draft {
    a.draft.as_ref().expect("設定の画面が開いている")
}

fn open_settings(a: &mut App) {
    ch(a, 'o');
    assert_eq!(a.mode, Mode::Settings);
}

fn sel_to(a: &mut App, to: usize) {
    loop {
        let d = draft(a);
        let now = match &d.pick {
            Some(p) => p.sel(),
            None => d.at(d.sec),
        };
        if now == to {
            return;
        }
        press(a, if now < to { KeyCode::Down } else { KeyCode::Up });
        let d = draft(a);
        let after = match &d.pick {
            Some(p) => p.sel(),
            None => d.at(d.sec),
        };
        assert_ne!(after, now, "{to} まで動かせない");
    }
}

fn section(a: &mut App, sec: Sec) {
    for _ in 0..6 {
        if draft(a).sec == sec {
            return;
        }
        press(a, KeyCode::Tab);
    }
    panic!("区画 {sec:?} に移れない");
}

fn pick_column(a: &mut App, col: &str) {
    let i = draft(a).keys.iter().position(|c| c == col).unwrap();
    sel_to(a, i);
    press(a, KeyCode::Enter);
}

/// フィルターに「値の一覧」の条件を足す(`keep` なら チェックした値だけ残す)。
fn add_values(a: &mut App, col: &str, values: &[&str], keep: bool) {
    section(a, Sec::Filters);
    let last = draft(a).len(Sec::Filters) - 1;
    sel_to(a, last);
    press(a, KeyCode::Enter);
    pick_column(a, col);
    sel_to(a, 0);
    press(a, KeyCode::Enter);
    if keep {
        sel_to(a, 0);
        ch(a, ' ');
    }
    for v in values {
        let Some(Pick::Values { counts, .. }) = &draft(a).pick else {
            panic!("値の一覧が開いていない");
        };
        let i = counts
            .iter()
            .position(|(k, _)| k.as_deref() == Some(*v))
            .unwrap_or_else(|| panic!("値 {v:?} が一覧に無い: {counts:?}"));
        sel_to(a, i + 1);
        ch(a, ' ');
    }
    press(a, KeyCode::Enter);
    assert!(draft(a).pick.is_none());
}

/// 設定の画面で「名前を付けて保存」。
fn save_as(a: &mut App, name: &str) {
    click_label(a, "名前を付けて保存");
    enter_text(a, name);
    settle(a);
}

// ---- BV-17: タブで切り替え・開き直しても同じ・手で書いた views.toml・ノートのフォルダに書かない ----

#[test]
fn test_bv_17_tabs_list_default_and_saved_views() {
    // [BV-17] `.base` の無いフォルダで、画面からビューを2つ作る → タブに「既定の表」と2つの名前が並び、
    // 切り替えると絞り込みが変わる。終了して開き直すと同じ。ノートのフォルダに新しいファイルは無い。
    let tmp = vault("bv17tabs");
    let notes_before = snapshot(&tmp.notes());
    let mut a = boot_folder(&tmp, false);

    open_settings(&mut a);
    add_values(&mut a, "status", &["done"], false);
    save_as(&mut a, "未完了");

    open_settings(&mut a);
    click_label(&mut a, "既定に戻す");
    add_values(&mut a, "種別", &["本"], true);
    save_as(&mut a, "本");

    let t = tabs(&a);
    let (i0, i1, i2) = (t.find("既定の表"), t.find("未完了"), t.find("本"));
    assert!(
        i0.is_some() && i1.is_some() && i2.is_some(),
        "タブに3つ並ぶ: {t}"
    );
    assert!(i0 < i1 && i1 < i2, "既定の表 → 保存した順: {t}");

    goto_tab(&mut a, "未完了");
    let rows_undone = sorted(labels(&a));
    assert_eq!(rows_undone, strs(&["a.md", "c.md", "e.md", "f.md"]));
    goto_tab(&mut a, "本");
    let rows_book = sorted(labels(&a));
    assert_eq!(rows_book, strs(&["b.md", "c.md", "f.md"]));
    assert_ne!(rows_undone, rows_book, "切り替えると絞り込みが変わる");
    quit(&mut a);

    // 開き直しても同じ。
    let mut b = boot_folder(&tmp, false);
    let t = tabs(&b);
    assert!(
        t.contains("既定の表") && t.contains("未完了") && t.contains("本"),
        "開き直しても同じタブ: {t}"
    );
    goto_tab(&mut b, "未完了");
    assert_eq!(sorted(labels(&b)), rows_undone);
    goto_tab(&mut b, "本");
    assert_eq!(sorted(labels(&b)), rows_book);
    quit(&mut b);

    // 定義は設定の置き場の views.toml にあり、ノートのフォルダには書かない。
    assert_eq!(names_in_config(&tmp), strs(&["未完了", "本"]));
    assert_eq!(
        snapshot(&tmp.notes()),
        notes_before,
        "ノートのフォルダに新しいファイルが無い"
    );
}

#[test]
fn test_bv_17_hand_written_views_toml_becomes_tabs() {
    // [BV-17] 設定の置き場の views.toml を手で書く → タブになり、選ぶとその絞り込みと列で表を組む。
    // 書き足して開き直す → タブが増える。ノートのフォルダには何も書かない。
    let tmp = vault("bv17hand");
    let notes_before = snapshot(&tmp.notes());
    std::fs::create_dir_all(conf_dir(&tmp)).unwrap();
    let real = std::fs::canonicalize(tmp.notes()).unwrap();
    let path = toml::Value::String(real.to_string_lossy().into_owned()).to_string();
    let first = format!(
        r#"[[target]]
path = {path}

  [[target.view]]
  name = "未完了"
  order = ["title", "status"]
  hidden = []
  filters_expr = ['status != "done"']
"#
    );
    std::fs::write(views_toml(&tmp), &first).unwrap();

    let mut a = boot_folder(&tmp, false);
    let t = tabs(&a);
    assert!(
        t.contains("既定の表") && t.contains("未完了"),
        "手で書いたビューがタブになる: {t}"
    );
    assert!(t.contains("[既定の表]"), "起動は先頭のタブ: {t}");
    goto_tab(&mut a, "未完了");
    assert_eq!(a.cols, strs(&["title", "status"]), "ビューの列の並び");
    let rows = sorted(labels(&a));
    assert!(
        !rows.contains(&"b.md".to_string()) && !rows.contains(&"d.md".to_string()),
        "式の絞り込みが効く: {rows:?}"
    );
    quit(&mut a);

    // 書き足す。
    let more = format!(
        r#"{first}
  [[target.view]]
  name = "会議"
  order = ["title"]
  hidden = []
  filters_expr = ['種別 == "会議"']
"#
    );
    std::fs::write(views_toml(&tmp), more).unwrap();
    let mut b = boot_folder(&tmp, false);
    let t = tabs(&b);
    assert!(
        t.contains("未完了") && t.contains("会議"),
        "書き足したビューのタブが増える: {t}"
    );
    goto_tab(&mut b, "会議");
    assert_eq!(sorted(labels(&b)), strs(&["a.md", "e.md"]));
    quit(&mut b);
    assert_eq!(snapshot(&tmp.notes()), notes_before);
}

#[test]
fn test_bv_17_no_new_files_in_notes_folder() {
    // [BV-17] 保存・上書き・名前の変更・削除をひととおりしても、ノートのフォルダに新しいファイルは無く
    // (バイトも同じ)、書くのは設定の置き場の views.toml だけ。
    let tmp = vault("bv17nofile");
    let notes_before = snapshot(&tmp.notes());
    let mut a = boot_folder(&tmp, false);
    open_settings(&mut a);
    add_values(&mut a, "status", &["done"], false);
    save_as(&mut a, "未完了");
    goto_tab(&mut a, "未完了");
    open_settings(&mut a);
    add_values(&mut a, "種別", &["メモ"], false);
    click_label(&mut a, "上書き");
    settle(&mut a);
    open_settings(&mut a);
    click_label(&mut a, "名前の変更");
    enter_text(&mut a, "残り");
    settle(&mut a);
    goto_tab(&mut a, "残り");
    open_settings(&mut a);
    click_label(&mut a, "削除");
    confirm_if_asked(&mut a);
    settle(&mut a);
    quit(&mut a);
    assert!(views_toml(&tmp).exists(), "views.toml に書いた");
    assert_eq!(snapshot(&tmp.notes()), notes_before);
}

// ---- BV-18: 名前を付けて保存・上書き・名前の変更・削除 ----

#[test]
fn test_bv_18_save_as_adds_tab() {
    // [BV-18][NV-13] `o` で done を外し「名前を付けて保存」で「未完了」→ タブに「未完了」が増え、
    // views.toml にその設定で残る。
    let tmp = vault("bv18save");
    let mut a = boot_folder(&tmp, false);
    assert!(!tabs(&a).contains("未完了"));
    open_settings(&mut a);
    add_values(&mut a, "status", &["done"], false);
    save_as(&mut a, "未完了");
    assert!(tabs(&a).contains("未完了"), "タブが増える: {}", screen(&a));
    goto_tab(&mut a, "未完了");
    assert_eq!(
        sorted(labels(&a)),
        strs(&["a.md", "c.md", "e.md", "f.md"]),
        "保存した見せ方"
    );
    let (v, _) = load_views(&conf_dir(&tmp), &tmp.notes());
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].name, "未完了");
    assert_eq!(v[0].settings.filters, drop_done().filters);
}

#[test]
fn test_bv_18_overwrite_survives_restart() {
    // [BV-18] 保存したビュー「未完了」で、別の絞り込みにして「上書き」→ 開き直しても上書きした見せ方。
    let tmp = vault("bv18over");
    prepare(&tmp, &tmp.notes(), &[view("未完了", drop_done())]);
    let mut a = boot_folder(&tmp, false);
    goto_tab(&mut a, "未完了");
    assert_eq!(sorted(labels(&a)), strs(&["a.md", "c.md", "e.md", "f.md"]));
    open_settings(&mut a);
    add_values(&mut a, "種別", &["本"], true);
    click_label(&mut a, "上書き");
    confirm_if_asked(&mut a);
    settle(&mut a);
    let want = strs(&["c.md", "f.md"]);
    assert_eq!(sorted(labels(&a)), want, "上書きした見せ方");
    assert_eq!(names_in_config(&tmp), strs(&["未完了"]), "増えずに上書き");
    quit(&mut a);

    let mut b = boot_folder(&tmp, false);
    goto_tab(&mut b, "未完了");
    assert_eq!(sorted(labels(&b)), want, "開き直しても上書きした見せ方");
    let (v, _) = load_views(&conf_dir(&tmp), &tmp.notes());
    assert_eq!(v[0].settings.filters.len(), 2, "{v:?}");
}

#[test]
fn test_bv_18_rename_changes_tab() {
    // [BV-18] 「名前の変更」で「未完了」を「残り」に → タブの名前が変わり、開き直しても同じ。
    let tmp = vault("bv18rename");
    prepare(&tmp, &tmp.notes(), &[view("未完了", drop_done())]);
    let mut a = boot_folder(&tmp, false);
    goto_tab(&mut a, "未完了");
    open_settings(&mut a);
    click_label(&mut a, "名前の変更");
    enter_text(&mut a, "残り");
    settle(&mut a);
    let t = tabs(&a);
    assert!(t.contains("残り") && !t.contains("未完了"), "{t}");
    assert_eq!(names_in_config(&tmp), strs(&["残り"]));
    quit(&mut a);

    let mut b = boot_folder(&tmp, false);
    let t = tabs(&b);
    assert!(t.contains("残り") && !t.contains("未完了"), "{t}");
    goto_tab(&mut b, "残り");
    assert_eq!(
        sorted(labels(&b)),
        strs(&["a.md", "c.md", "e.md", "f.md"]),
        "名前を変えても見せ方は同じ"
    );
}

#[test]
fn test_bv_18_delete_removes_tab() {
    // [BV-18] 「未完了」と「本」があるとき、「未完了」を「削除」→ タブから消え、「本」は残る。開き直しても同じ。
    let tmp = vault("bv18del");
    prepare(
        &tmp,
        &tmp.notes(),
        &[view("未完了", drop_done()), view("本", keep_book())],
    );
    let mut a = boot_folder(&tmp, false);
    goto_tab(&mut a, "未完了");
    open_settings(&mut a);
    click_label(&mut a, "削除");
    confirm_if_asked(&mut a);
    settle(&mut a);
    let t = tabs(&a);
    assert!(
        !t.contains("未完了") && t.contains("本") && t.contains("既定の表"),
        "{t}"
    );
    assert_eq!(names_in_config(&tmp), strs(&["本"]));
    quit(&mut a);

    let b = boot_folder(&tmp, false);
    let t = tabs(&b);
    assert!(!t.contains("未完了") && t.contains("本"), "{t}");
}

// ---- BV-19: .base への書き出しと取り込み ----

#[test]
fn test_bv_19_export_writes_new_base_and_reports_dropped() {
    // [BV-19] 絞り込み・並べ替え・グループ(空を隠す)のあるビューをパレットの「.base に書き出す」で
    // 新しい名前に書き出す → `.base` が1つ増え、読めて、開くと同じ行が出る。表せない「空を隠す」は
    // 落としたと知らせる。[BV-3] 既存の `.base` とノートのバイトは同じ。
    let tmp = vault("bv19export");
    base_path(&tmp);
    let settings = Settings {
        filters: drop_done().filters,
        sorts: vec![("priority".into(), Dir::Desc)],
        group: Group::By {
            col: "種別".into(),
            dir: Dir::Asc,
            hide_empty: true,
        },
        ..Default::default()
    };
    prepare(&tmp, &tmp.notes(), &[view("未完了", settings)]);
    let before = snapshot(&tmp.notes());
    let mut a = boot_folder(&tmp, false);
    goto_tab(&mut a, "未完了");
    let rows = sorted(labels(&a));
    assert_eq!(rows, strs(&["a.md", "c.md", "e.md", "f.md"]));
    palette(&mut a, ".base に書き出す");
    enter_text(&mut a, "export.base");
    settle(&mut a);

    let after = snapshot(&tmp.notes());
    for (p, bytes) in &before {
        assert_eq!(
            after.get(p),
            Some(bytes),
            "既存のファイルは同じ: {}",
            p.display()
        );
    }
    let new: Vec<&PathBuf> = after.keys().filter(|p| !before.contains_key(*p)).collect();
    assert_eq!(new.len(), 1, "新しいファイルは1つ: {new:?}");
    let out = new[0];
    assert_eq!(out.extension().and_then(|e| e.to_str()), Some("base"));
    assert!(
        out.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("export"),
        "利用者が決めた名前: {}",
        out.display()
    );
    let parsed = base::parse(&std::fs::read_to_string(out).unwrap()).expect("読める .base");
    assert!(
        parsed.views.iter().any(|v| v.name == "未完了"),
        "{parsed:?}"
    );
    // 帯の「(空を隠す)」でなく、書き出しの知らせに落とした項目が出る。
    assert!(
        message(&a).contains("空を隠す"),
        "落とした項目(空を隠す)を知らせる: {:?}",
        message(&a)
    );

    // 書き出した `.base` を開くと同じ行。
    let t = crate::open_target(std::slice::from_ref(out), None).unwrap();
    let mut b = App::new(Box::new(t.src), ColorMode::None);
    b.resize(80, 24);
    b.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: true,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: out.clone(),
        base: t.base,
    });
    while !b.loaded() {
        b.load_step(100);
    }
    assert_eq!(sorted(labels(&b)), rows, "書き出した .base でも同じ行");
}

#[test]
fn test_bv_19_export_refuses_existing_base() {
    // [BV-19][BV-3] 書き出しで既存の `.base` の名前(tasks.base)を選ぶ → 書かずに理由。
    // ノートのフォルダのどのファイルも同じで、新しいファイルも無い。
    let tmp = vault("bv19exist");
    base_path(&tmp);
    prepare(&tmp, &tmp.notes(), &[view("未完了", drop_done())]);
    let before = snapshot(&tmp.notes());
    let mut a = boot_folder(&tmp, false);
    goto_tab(&mut a, "未完了");
    palette(&mut a, ".base に書き出す");
    enter_text(&mut a, "tasks.base");
    let shown = format!("{}\n{}", message(&a), screen(&a));
    settle(&mut a);
    assert_eq!(
        snapshot(&tmp.notes()),
        before,
        "既存の .base を書き換えない"
    );
    assert!(shown.contains("tasks.base"), "書かない理由を出す: {shown}");
}

#[test]
fn test_bv_19_import_base_view_adds_tab() {
    // [BV-19] パレットの「.base のビューを取り込む」で tasks.base の「進行」を取り込む → mdgrid のビューの
    // タブ「進行」が増え、`.base` の filters(doing を外す)が効く。views.toml に入り、`.base` は同じ。
    let tmp = vault("bv19import");
    let bp = base_path(&tmp);
    let base_before = std::fs::read(&bp).unwrap();
    let mut a = boot_folder(&tmp, false);
    assert!(!tabs(&a).contains("進行"));
    palette(&mut a, ".base のビューを取り込む");
    enter_text(&mut a, "tasks.base");
    enter_text(&mut a, "進行");
    settle(&mut a);
    assert!(
        tabs(&a).contains("進行"),
        "取り込んだビューのタブ: {}",
        screen(&a)
    );
    goto_tab(&mut a, "進行");
    let rows = labels(&a);
    assert!(
        !rows.contains(&"e.md".to_string()),
        "doing を外す: {rows:?}"
    );
    assert_eq!(rows.len(), 5, "{rows:?}");
    let (v, _) = load_views(&conf_dir(&tmp), &tmp.notes());
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].name, "進行");
    assert!(
        v[0].filters_expr.iter().any(|e| e.contains("doing")),
        "{v:?}"
    );
    assert_eq!(std::fs::read(&bp).unwrap(), base_before);
}

// ---- BV-20: 壊れた views.toml・知らない項目・.base のビューが先・読むだけ ----

#[test]
fn test_bv_20_broken_views_toml_warns_and_starts_without_tabs() {
    // [BV-20] views.toml を手で壊す → 起動して警告、mdgrid のビューのタブは出ず、既定の表で開く。
    // 壊れたファイルは書き換えない。
    let tmp = vault("bv20broken");
    std::fs::create_dir_all(conf_dir(&tmp)).unwrap();
    let broken = "[[target]\npath = \"/x\n  [[target.view]]\n  name = \n";
    std::fs::write(views_toml(&tmp), broken).unwrap();
    let mut a = boot_folder(&tmp, false);
    assert!(
        message(&a).contains("views.toml"),
        "警告を出す: {:?}",
        a.message
    );
    let t = tabs(&a);
    assert!(t.contains("既定の表"), "{t}");
    assert!(!t.contains('['), "mdgrid のビューのタブは出ない: {t}");
    assert_eq!(labels(&a).len(), 6, "表は開く");
    quit(&mut a);
    assert_eq!(std::fs::read_to_string(views_toml(&tmp)).unwrap(), broken);
}

#[test]
fn test_bv_20_unknown_key_warns_and_reads() {
    // [BV-20][CLI-3] 知らない項目 `color = 1` → 警告して、そのビューはタブに出る。
    let tmp = vault("bv20unknown");
    std::fs::create_dir_all(conf_dir(&tmp)).unwrap();
    let real = std::fs::canonicalize(tmp.notes()).unwrap();
    let path = toml::Value::String(real.to_string_lossy().into_owned()).to_string();
    std::fs::write(
        views_toml(&tmp),
        format!(
            "[[target]]\npath = {path}\n\n  [[target.view]]\n  name = \"色つき\"\n  order = [\"title\"]\n  hidden = []\n  filters_expr = []\n  color = 1\n"
        ),
    )
    .unwrap();
    let a = boot_folder(&tmp, false);
    assert!(message(&a).contains("color"), "警告: {:?}", a.message);
    assert!(tabs(&a).contains("色つき"), "{}", screen(&a));
}

#[test]
fn test_bv_20_base_views_come_first() {
    // [BV-20][BV-13] `.base` と mdgrid のビューの両方がある対象 → タブは `.base` のビューが先、
    // 起動は先頭(`.base` の最初のビュー)。mdgrid のビューに切り替えるとその見せ方。
    let tmp = vault("bv20order");
    let bp = base_path(&tmp);
    let base_before = std::fs::read(&bp).unwrap();
    prepare(&tmp, &bp, &[view("自前", keep_book())]);
    let mut a = boot_base(&tmp, false);
    let t = tabs(&a);
    let pos: Vec<Option<usize>> = ["全部", "進行", "名前だけ", "自前"]
        .iter()
        .map(|n| t.find(n))
        .collect();
    assert!(pos.iter().all(|p| p.is_some()), "4つのタブ: {t}");
    assert!(
        pos.windows(2).all(|w| w[0] < w[1]),
        ".base のビューが先: {t}"
    );
    assert!(t.contains("[全部]"), "起動は先頭のビュー: {t}");
    goto_tab(&mut a, "自前");
    assert_eq!(sorted(labels(&a)), strs(&["b.md", "c.md", "f.md"]));
    quit(&mut a);
    assert_eq!(std::fs::read(&bp).unwrap(), base_before);
}

#[test]
fn test_bv_20_readonly_writes_nothing() {
    // [BV-20][WB-15] `--readonly` で開く → mdgrid のビューのタブは読むが、保存・上書き・書き出しをしても
    // 何も書かれず、「読むだけ」と出る(保存の操作を出さないのでもよい)。views.toml・ノート・状態の置き場は同じ。
    let tmp = vault("bv20ro");
    base_path(&tmp);
    prepare(&tmp, &tmp.notes(), &[view("未完了", drop_done())]);
    let conf_before = snapshot(&conf_dir(&tmp));
    let notes_before = snapshot(&tmp.notes());
    let mut a = boot_folder(&tmp, true);
    assert!(
        tabs(&a).contains("未完了"),
        "読むだけでもタブは出る: {}",
        screen(&a)
    );
    goto_tab(&mut a, "未完了");
    open_settings(&mut a);
    add_values(&mut a, "種別", &["本"], true);
    for label in ["名前を付けて保存", "上書き", "名前の変更", "削除"] {
        if !screen(&a).contains(label) {
            continue; // 保存の操作を出さない(WB-15)
        }
        click_label(&mut a, label);
        if a.mode != Mode::Settings {
            enter_text(&mut a, "別名");
        }
        assert!(
            message(&a).contains("読むだけ"),
            "{label} で「読むだけ」: {:?}",
            a.message
        );
        if a.mode != Mode::Settings {
            settle(&mut a);
            open_settings(&mut a);
        }
    }
    settle(&mut a);
    palette(&mut a, ".base に書き出す");
    if a.mode != Mode::Table {
        enter_text(&mut a, "export.base");
    }
    settle(&mut a);
    quit(&mut a);
    assert_eq!(
        snapshot(&conf_dir(&tmp)),
        conf_before,
        "views.toml を書かない"
    );
    assert_eq!(
        snapshot(&tmp.notes()),
        notes_before,
        "ノートと .base を書かない"
    );
    assert!(snapshot(&state_dir(&tmp)).is_empty(), "状態も書かない");
}
