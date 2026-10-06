//! ビューの設定の画面と、表の上の設定の帯の試験(NV-13〜NV-22 の画面の側)。
//! 材料は test_screen の Tmp。状態の置き場は一時フォルダ(`<一時>/state`)を `Startup` で渡す。

use super::keymap::{self, Mode};
use super::settings::{Pick, Sec};
use super::startup::Startup;
use super::test_screen::{app_of, col_named, golden, press, screen, typing, Tmp};
use super::*;
use mdgrid::settings::{Dir, Group, Op};
use ratatui::crossterm::event::KeyCode;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ---- 材料 ----

/// 6つのノート(status は todo・done・doing・無し、種別 は 会議・本・メモ・無し)と types.json。
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

/// ビューが2つの `.base`: status でまとめた「全部」と、filters で doing を外した「進行」。
const BASE: &str = r#"views:
  - type: table
    name: 全部
    groupBy:
      property: status
      direction: ASC
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

fn state_dir(tmp: &Tmp) -> PathBuf {
    tmp.0.join("state")
}

/// main と同じ道筋で `.base` を開く(状態の置き場あり)。
fn boot(tmp: &Tmp, view: Option<&str>, readonly: bool) -> App {
    let p = base_path(tmp);
    let t = crate::open_target(std::slice::from_ref(&p), view).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly,
        no_color: false,
        state_dir: Some(state_dir(tmp)),
        config_dir: None,
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

/// `.base` なしの既定の表(状態の置き場なし)。
pub(super) fn folder(name: &str) -> (Tmp, App) {
    let tmp = vault(name);
    let app = app_of(&tmp, ColorMode::None);
    (tmp, app)
}

fn labels(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
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

fn state_files(tmp: &Tmp) -> usize {
    std::fs::read_dir(state_dir(tmp))
        .map(|d| d.flatten().count())
        .unwrap_or(0)
}

/// SR-9: どの行も幅以下。
fn assert_fits(s: &str, w: usize) {
    for l in s.lines() {
        assert!(width::width(l) <= w, "幅を超える行: {l}");
    }
}

// ---- 設定の画面の操作 ----

pub(super) fn draft(a: &App) -> &super::settings::Draft {
    a.draft.as_ref().expect("設定の画面が開いている")
}

fn ch(a: &mut App, c: char) {
    super::test_screen::ch(a, c);
}

pub(super) fn open(a: &mut App) {
    ch(a, 'o');
    assert_eq!(a.mode, Mode::Settings);
}

/// 今の選び(選び手が開いていればその中、無ければ区画の中)を `to` に動かす(↑↓ で)。
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

pub(super) fn section(a: &mut App, sec: Sec) {
    for _ in 0..super::settings::SECS.len() {
        if draft(a).sec == sec {
            return;
        }
        press(a, KeyCode::Tab);
    }
    panic!("区画 {sec:?} に移れない");
}

/// 選び手の列を選ぶ。
fn pick_column(a: &mut App, col: &str) {
    let i = draft(a).keys.iter().position(|c| c == col).unwrap();
    sel_to(a, i);
    press(a, KeyCode::Enter);
}

/// フィルターに「値の一覧」の条件を足す(`keep` なら チェックした値だけ残す)。
pub(super) fn add_values(a: &mut App, col: &str, values: &[Option<&str>], keep: bool) {
    section(a, Sec::Filters);
    let last = draft(a).len(Sec::Filters) - 1;
    sel_to(a, last);
    press(a, KeyCode::Enter);
    pick_column(a, col);
    // 種類: 値の一覧(先頭)
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
            .position(|(k, _)| k.as_deref() == *v)
            .unwrap_or_else(|| panic!("値 {v:?} が一覧に無い: {counts:?}"));
        sel_to(a, i + 1);
        ch(a, ' ');
    }
    press(a, KeyCode::Enter);
    assert!(draft(a).pick.is_none());
}

fn button(a: &mut App, i: usize) {
    section(a, Sec::Buttons);
    sel_to(a, i);
    press(a, KeyCode::Enter);
}

pub(super) fn apply_btn(a: &mut App) {
    button(a, 0);
    assert_eq!(a.mode, Mode::Table);
}

/// 設定で status の done を隠して反映する。
fn hide_done(a: &mut App) {
    open(a);
    add_values(a, "status", &[Some("done")], false);
    apply_btn(a);
}

// ---- NV-13: 反映するまで表は変わらず、取り消せば元のまま ----

#[test]
fn test_nv_13_apply_and_cancel() {
    // [NV-13] 設定の画面で status の done を隠す → 反映の前は表に done の行が残り、反映で消える。
    // 取り消し(Esc・ボタン)で閉じる → 表は開く前のまま。
    let (_t, mut a) = folder("nv13");
    let before = labels(&a);
    assert!(before.contains(&"b.md".to_string()) && before.contains(&"d.md".to_string()));
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    assert_eq!(
        draft(&a).s.filters[0].op,
        Op::Drop(vec![Some("done".into())])
    );
    assert_eq!(labels(&a), before, "反映の前は表を変えない");
    assert!(a.settings.is_default());
    apply_btn(&mut a);
    let after = labels(&a);
    assert!(!after.contains(&"b.md".to_string()) && !after.contains(&"d.md".to_string()));
    assert_eq!(after.len(), before.len() - 2);

    // 取り消し: 写しで条件を消しても、Esc で閉じれば表は開く前のまま。
    open(&mut a);
    section(&mut a, Sec::Filters);
    sel_to(&mut a, 0);
    ch(&mut a, 'd');
    assert!(draft(&a).s.filters.is_empty());
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    assert!(a.draft.is_none());
    assert_eq!(labels(&a), after, "Esc は写しを捨てる");
    assert!(a.message.as_deref().unwrap().contains("取り消した"));
    // 取り消しのボタンも同じ。
    open(&mut a);
    add_values(&mut a, "種別", &[Some("本")], true);
    button(&mut a, 1);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(labels(&a), after);
}

#[test]
fn test_nv_13_palette_opens_settings() {
    // [NV-13][SR-14] パレットの動作 `view_settings` からも開ける。キーは表のモードの `o`。
    let (_t, mut a) = folder("nv13pal");
    assert_eq!(
        keymap::lookup(&a.keys, Mode::Table, "o"),
        Some(keymap::Action::ViewSettings)
    );
    ch(&mut a, ':');
    typing(&mut a, "view_settings");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    assert!(a.draft.is_some());
}

// ---- NV-16: 表の上の帯 ----

#[test]
fn test_nv_16_band_lists_settings() {
    // [NV-16] status の done を外して反映 → 表の上に「status」と「done」を含む項目。表は1行下がる。
    // 何も効いていなければ帯を出さない。
    let (_t, mut a) = folder("nv16");
    let s = screen(&a);
    // 行は 0 から: ヘッダー・タブ・検索の欄(NV-23)のあとが帯。
    assert!(s.lines().nth(3).unwrap().contains("ノート"), "帯なし: {s}");
    hide_done(&mut a);
    let s = screen(&a);
    let band = s.lines().nth(3).unwrap();
    assert!(band.contains("status") && band.contains("done"), "{s}");
    assert!(band.contains("[status: done を除く]"), "{band}");
    assert!(
        s.lines().nth(4).unwrap().contains("ノート"),
        "表は1行下がる: {s}"
    );
    assert_fits(&s, 80);
    golden("nv_16", &s);
    // 表のクリックも1行ずれる(5行目が最初のノートの行)。
    a.click(0, 6);
    assert_eq!(a.row, 1);
    a.click(0, 5);
    assert_eq!(a.row, 0);
    assert_eq!(a.mode, Mode::Table);
}

// ---- NV-18: 設定の画面 ----

#[test]
fn test_nv_18_settings_screen_golden() {
    // [NV-18] 1つの画面に 列(表示・非表示と順)・フィルター・並べ替え・グループ の4つの区画と、
    // 反映・取り消し・既定に戻す のボタン。[SR-9] 全行が幅以下。
    let tmp = vault("nv18");
    let mut a = boot(&tmp, None, false);
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    // 並べ替え: priority を足して降順に。
    section(&mut a, Sec::Sorts);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "priority");
    ch(&mut a, ' ');
    // グループ: 種別で分け、空を隠す。
    section(&mut a, Sec::Group);
    sel_to(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "種別");
    sel_to(&mut a, 3);
    ch(&mut a, ' ');
    // 列: priority を隠す。
    section(&mut a, Sec::Columns);
    let i = draft(&a)
        .cols
        .iter()
        .position(|(c, _)| c == "priority")
        .unwrap();
    sel_to(&mut a, i);
    ch(&mut a, ' ');
    let s = screen(&a);
    for want in [
        "列(表示と順)",
        "フィルター",
        "並べ替え",
        "グループ",
        "status: done を除く",
        "priority ↓ 降順",
        "(*) 列で分ける: 種別",
        "[x] 空のまとまりを隠す",
        "[ ] priority",
        "[ 反映 ]",
        "[ 取り消し ]",
        "[ 既定に戻す ]",
    ] {
        assert!(s.contains(want), "{want} が無い: {s}");
    }
    assert_fits(&s, 80);
    golden("nv_18", &s);
    // 反映すると全部が表に効く。
    apply_btn(&mut a);
    assert!(!a.cols.contains(&"priority".to_string()));
    assert!(a.hidden.iter().any(|(c, _, _)| c == "priority"));
    let heads: Vec<&str> = a.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["会議", "本"], "done を隠し、種別でまとめ直す");
    assert_eq!(
        labels(&a),
        ["e.md", "a.md", "c.md", "f.md"],
        "中は priority の降順"
    );
}

#[test]
fn test_nv_18_values_picker_golden() {
    // [NV-18][NV-19] 値の一覧は件数つき、キーの無いノートは「(空)」。方式の行で 残す・隠す を切り替える。
    let (_t, mut a) = folder("nv18v");
    open(&mut a);
    section(&mut a, Sec::Filters);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    for want in [
        "status の値(件数)",
        "方式: チェックした値の行を隠す",
        "[ ] done  2件",
        "[ ] todo  2件",
        "[ ] doing  1件",
        "[ ] (空)  1件",
    ] {
        assert!(s.contains(want), "{want} が無い: {s}");
    }
    assert_fits(&s, 80);
    golden("nv_18_values", &s);
}

// ---- NV-14・NV-19: 画面からの条件 ----

#[test]
fn test_nv_14_keep_only_book_from_screen() {
    // [NV-14] 種別の列の 本 だけを残す → 本 の行だけ。
    let (_t, mut a) = folder("nv14");
    open(&mut a);
    add_values(&mut a, "種別", &[Some("本")], true);
    assert_eq!(draft(&a).s.filters[0].op, Op::Keep(vec![Some("本".into())]));
    apply_btn(&mut a);
    assert_eq!(labels(&a), ["b.md", "c.md", "f.md"]);
    // [NV-14] キーの無いノートは「(空)」として選べる。
    open(&mut a);
    section(&mut a, Sec::Filters);
    sel_to(&mut a, 0);
    ch(&mut a, 'd');
    add_values(&mut a, "status", &[None], true);
    apply_btn(&mut a);
    assert_eq!(labels(&a), ["f.md"]);
}

#[test]
fn test_nv_14_filter_by_key_not_in_view() {
    // [NV-14] ビューの列に無いノートのキー(status)でも、列を選んで隠せる。
    let tmp = vault("nv14key");
    let mut a = boot(&tmp, Some("名前だけ"), false);
    assert_eq!(a.cols, ["title"]);
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    apply_btn(&mut a);
    assert_eq!(a.cols, ["title"], "列は増えない");
    assert_eq!(labels(&a), ["a.md", "c.md", "e.md", "f.md"]);
}

#[test]
fn test_nv_19_compare_and_contains_from_screen() {
    // [NV-19] priority `≥ 3`(値を打つ)と title の「会議」を含む を並べる → 両方を満たす行だけ。
    // 日付の列に読めない値を打つと閉じずに理由。
    let (_t, mut a) = folder("nv19");
    open(&mut a);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "priority");
    sel_to(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    sel_to(&mut a, 5);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText);
    typing(&mut a, "3");
    assert!(screen(&a)
        .lines()
        .nth(22)
        .unwrap()
        .contains("priority ≥: 3"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings);
    let last = draft(&a).len(Sec::Filters) - 1;
    sel_to(&mut a, last);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "title");
    sel_to(&mut a, 1);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    // 日付の比較に読めない値。
    let last = draft(&a).len(Sec::Filters) - 1;
    sel_to(&mut a, last);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "due");
    sel_to(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    sel_to(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "someday");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::SettingsText, "読めない値では閉じない");
    assert!(a.message.as_deref().unwrap().contains("YYYY-MM-DD"));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Settings);
    assert_eq!(draft(&a).s.filters.len(), 2);
    apply_btn(&mut a);
    assert_eq!(labels(&a), ["a.md", "e.md"]);
}

// ---- NV-15・NV-21: グループ ----

#[test]
fn test_nv_15_group_off_and_by_column() {
    // [NV-15] groupBy のある `.base` のビューで「しない」→ 見出しの行が消える。種別を選ぶ → 種別ごとの見出し。
    // `.base` のバイトは同じ。
    let tmp = vault("nv15");
    let mut a = boot(&tmp, None, false);
    let base_before = std::fs::read(base_path(&tmp)).unwrap();
    assert!(!a.groups.is_empty());
    open(&mut a);
    section(&mut a, Sec::Group);
    sel_to(&mut a, 1);
    press(&mut a, KeyCode::Enter);
    apply_btn(&mut a);
    assert_eq!(a.settings.group, Group::Off);
    assert!(a.groups.is_empty());
    assert!(a.slots.iter().all(|s| matches!(s, grid::Slot::Row(_))));
    assert!(!screen(&a).contains('▾'));
    open(&mut a);
    section(&mut a, Sec::Group);
    sel_to(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "種別");
    apply_btn(&mut a);
    let heads: Vec<&str> = a.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["メモ", "会議", "本"]);
    assert!(screen(&a).contains("▾ 種別: 本(3件)"), "{}", screen(&a));
    // [NV-21] 並びを降順に。
    open(&mut a);
    section(&mut a, Sec::Group);
    sel_to(&mut a, 4);
    press(&mut a, KeyCode::Enter);
    apply_btn(&mut a);
    let heads: Vec<&str> = a.groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(heads, ["本", "会議", "メモ"]);
    assert_eq!(std::fs::read(base_path(&tmp)).unwrap(), base_before);
}

#[test]
fn test_nv_21_hide_empty_group_from_screen() {
    // [NV-21] status でまとめ、空を隠す → `(空)` の見出しと行が出ない。
    let (_t, mut a) = folder("nv21");
    open(&mut a);
    section(&mut a, Sec::Group);
    sel_to(&mut a, 3);
    ch(&mut a, ' ');
    assert!(a.message.as_deref().unwrap().contains("列で分けるとき"));
    sel_to(&mut a, 2);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "status");
    apply_btn(&mut a);
    assert!(a.groups.iter().any(|(h, _)| h == "(空)"));
    open(&mut a);
    section(&mut a, Sec::Group);
    sel_to(&mut a, 3);
    ch(&mut a, ' ');
    apply_btn(&mut a);
    assert!(a.groups.iter().all(|(h, _)| h != "(空)"));
    assert!(!labels(&a).contains(&"f.md".to_string()));
}

// ---- NV-20: 重なり方 ----

#[test]
fn test_nv_20_layers() {
    // [NV-20] `.base` の filters(doing を外す)の上に、設定で done を外す → done でも doing でもない行だけ。
    // `\` の簡易の絞り込みがさらに絞る。設定で並べた列と別の列を `s` で並べると `s` が勝ち、外すと設定の並び。
    let tmp = vault("nv20");
    let mut a = boot(&tmp, Some("進行"), false);
    let base_rows = labels(&a);
    assert!(!base_rows.contains(&"e.md".to_string()), "{base_rows:?}");
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    section(&mut a, Sec::Sorts);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "title");
    apply_btn(&mut a);
    let rows = labels(&a);
    let want: Vec<String> = ["a.md", "f.md", "c.md"].map(String::from).to_vec();
    assert_eq!(
        rows, want,
        "設定の並び(title の昇順: 会議の準備・本棚・次の本)"
    );
    // `\` でさらに絞る。
    ch(&mut a, '\\');
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    assert_eq!(labels(&a), ["a.md"]);
    press(&mut a, KeyCode::Esc);
    assert_eq!(labels(&a), want);
    // `s` の並びが勝つ。
    col_named(&mut a, "priority");
    ch(&mut a, 's');
    assert_eq!(labels(&a), ["a.md", "c.md", "f.md"]);
    ch(&mut a, 's');
    ch(&mut a, 's');
    assert!(a.sort.is_none());
    assert_eq!(labels(&a), want, "`s` を外すと設定の並び");
}

#[test]
fn test_nv_20_edited_row_stays_until_move() {
    // [NV-20][NV-12] done を外した表で done に直した行は、保存か移動まで残る。
    let (_t, mut a) = folder("nv20stay");
    hide_done(&mut a);
    assert_eq!(labels(&a)[0], "a.md");
    a.row = 0;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    for _ in 0..8 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 1);
    assert!(labels(&a).contains(&"a.md".to_string()), "直した行は残る");
    assert!(a.held.iter().any(|r| a.src.label(r) == "a.md"));
    ch(&mut a, 'j');
    assert!(!labels(&a).contains(&"a.md".to_string()), "移動で消える");
}

#[test]
fn test_nv_20_edited_row_released_on_save() {
    // [NV-20][NV-12] done を外した表で done に直した行は、保存すると留めが外れて表から消える。
    let (_t, mut a) = folder("nv20save");
    hide_done(&mut a);
    a.row = 0;
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    for _ in 0..8 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert!(labels(&a).contains(&"a.md".to_string()), "保存の前は残る");
    super::test_screen::ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert!(
        !labels(&a).contains(&"a.md".to_string()),
        "保存で消える: {:?}",
        labels(&a)
    );
    assert!(a.held.is_empty());
}

// ---- NV-22: 既定に戻す・帯から外す・読むだけ ----

#[test]
fn test_nv_22_reset_to_default() {
    // [NV-22] 設定を戻す → 帯が消え、`.base` のままの表(列も `.base` の順で全部)。
    let tmp = vault("nv22reset");
    let mut a = boot(&tmp, None, false);
    let rows0 = labels(&a);
    let cols0 = a.cols.clone();
    let groups0 = a.groups.clone();
    hide_done(&mut a);
    open(&mut a);
    section(&mut a, Sec::Columns);
    sel_to(&mut a, 0);
    ch(&mut a, ' ');
    ch(&mut a, 'J');
    apply_btn(&mut a);
    assert_ne!(labels(&a), rows0);
    assert_ne!(a.cols, cols0);
    open(&mut a);
    button(&mut a, 2);
    assert_eq!(a.mode, Mode::Settings, "既定に戻すは写しだけ");
    assert!(draft(&a).s.is_default());
    assert!(a.message.as_deref().unwrap().contains("既定に戻した"));
    apply_btn(&mut a);
    assert!(a.settings.is_default());
    assert_eq!(labels(&a), rows0);
    assert_eq!(a.cols, cols0);
    assert!(a.hidden.is_empty());
    assert_eq!(a.groups, groups0);
    assert!(!screen(&a).lines().nth(3).unwrap().contains("設定:"));
}

#[test]
fn test_nv_22_remove_one_from_band() {
    // [NV-22] 帯の「status: done を除く」を外す → done の行が戻る。キー(`f` → ←→ → Backspace)とクリック。
    let (_t, mut a) = folder("nv22band");
    let all = labels(&a);
    open(&mut a);
    add_values(&mut a, "status", &[Some("done")], false);
    add_values(&mut a, "種別", &[Some("メモ")], false);
    apply_btn(&mut a);
    assert_eq!(a.settings.chips().len(), 2);
    ch(&mut a, 'f');
    assert_eq!(a.mode, Mode::Chips);
    let s = screen(&a);
    assert!(
        s.lines().nth(3).unwrap().contains(">[status: done を除く]"),
        "{s}"
    );
    assert_fits(&s, 80);
    press(&mut a, KeyCode::Right);
    assert_eq!(a.chip, 1);
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.settings.chips(), ["status: done を除く"]);
    assert_eq!(a.mode, Mode::Chips);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    // クリックで外す。
    let x = (0..80u16)
        .find(|&x| view::chip_at(&a, x, 3) == Some(0))
        .unwrap();
    a.click(x, 3);
    assert!(a.settings.is_default());
    assert_eq!(labels(&a), all, "done の行が戻る");
    assert!(a.message.as_deref().unwrap().contains("外した"));
    // 帯が無いときの `f` は理由を出す。
    ch(&mut a, 'f');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.as_deref().unwrap().contains("項目が無い"));
}

#[test]
fn test_nv_22_readonly_applies_but_writes_nothing() {
    // [NV-22][WB-15] `--readonly` で設定を反映 → 表は変わり、ノート・`.base`・状態の置き場のどれも書かれない。
    let tmp = vault("nv22ro");
    base_path(&tmp);
    let before = snapshot(&tmp.notes());
    let mut a = boot(&tmp, None, true);
    let rows0 = labels(&a);
    hide_done(&mut a);
    assert_ne!(labels(&a), rows0, "表は変わる");
    assert!(a.message.as_deref().unwrap().contains("覚えない"));
    ch(&mut a, 'f');
    press(&mut a, KeyCode::Backspace);
    ch(&mut a, ']');
    ch(&mut a, 'q');
    assert!(a.quit);
    assert_eq!(snapshot(&tmp.notes()), before);
    assert_eq!(state_files(&tmp), 0);
}

// ---- NV-17: ビューごとに覚える ----

#[test]
fn test_nv_17_settings_survive_restart_per_view() {
    // [NV-17] 設定を反映して終了 → 同じ `.base` の同じビューで開き直すと同じ設定。ほかのビューは既定。
    // `.base` とノートのフォルダは変わらない。
    let tmp = vault("nv17");
    base_path(&tmp);
    let before = snapshot(&tmp.notes());
    let mut a = boot(&tmp, None, false);
    hide_done(&mut a);
    let s = a.settings.clone();
    let rows = labels(&a);
    ch(&mut a, ']');
    assert!(a.settings.is_default(), "ほかのビューは既定");
    ch(&mut a, '[');
    assert_eq!(a.settings, s, "ビューを戻すとそのビューの設定");
    ch(&mut a, 'q');
    assert!(a.quit);

    let b = boot(&tmp, None, false);
    assert_eq!(b.settings, s);
    assert_eq!(labels(&b), rows);
    assert!(screen(&b)
        .lines()
        .nth(3)
        .unwrap()
        .contains("status: done を除く"));
    let c = boot(&tmp, Some("進行"), false);
    assert!(c.settings.is_default());
    assert_eq!(snapshot(&tmp.notes()), before);
}

#[test]
fn test_nv_19_formula_kind_stable_across_restart() {
    // [NV-19][NV-17] formula の列の型は `.base` の結果の行の全部から決める: `dbl ≥ 3` と昇順を設定して開き直しても、
    // 操作のあとも、数で比べた同じ結果(文字で比べない)。
    let tmp = vault("nv19formula");
    std::fs::write(
        tmp.notes().join("tasks.base"),
        "formulas:\n  dbl: 'priority + priority'\nviews:\n  - type: table\n    name: 倍\n    order: [title, formula.dbl]\n",
    )
    .unwrap();
    let mut a = boot(&tmp, None, false);
    open(&mut a);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "formula.dbl");
    sel_to(&mut a, 3);
    press(&mut a, KeyCode::Enter);
    sel_to(&mut a, 5);
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Settings, "数として読める");
    section(&mut a, Sec::Sorts);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "formula.dbl");
    apply_btn(&mut a);
    let want = ["a.md", "e.md", "c.md"];
    assert_eq!(labels(&a), want);
    ch(&mut a, 'q');
    assert!(a.quit);

    let mut b = boot(&tmp, None, false);
    assert_eq!(labels(&b), want, "開き直しても同じ");
    ch(&mut b, 'j');
    ch(&mut b, 'l');
    assert_eq!(labels(&b), want, "操作のあとも同じ");
}

#[test]
fn test_nv_18_apply_keeps_selected_column_by_name() {
    // [NV-18] status を選んだまま左の title を隠して反映 → 選んだ列は status のまま。
    // 選んでいた列を隠したら、最寄りの(右の)列。
    let (_t, mut a) = folder("nv18sel");
    col_named(&mut a, "status");
    open(&mut a);
    section(&mut a, Sec::Columns);
    sel_to(&mut a, 0);
    ch(&mut a, ' ');
    apply_btn(&mut a);
    assert_eq!(a.cols[a.col], "status");
    open(&mut a);
    section(&mut a, Sec::Columns);
    let i = draft(&a)
        .cols
        .iter()
        .position(|(c, _)| c == "status")
        .unwrap();
    sel_to(&mut a, i);
    ch(&mut a, ' ');
    apply_btn(&mut a);
    assert_eq!(a.cols[a.col], "種別");
}

// ---- 見た目と操作の守り ----

#[test]
fn test_sr_9_settings_screen_fits_narrow() {
    // [SR-9] 狭い端末でも設定の画面・帯・値の入力の行は幅以下。
    let (_t, mut a) = folder("sr9set");
    hide_done(&mut a);
    open(&mut a);
    for (w, h) in [(40u16, 12u16), (20, 8), (80, 24)] {
        a.resize(w, h);
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw(f, &a)).unwrap();
        let s = super::test_screen::text(term.backend().buffer());
        assert_fits(&s, w as usize);
    }
    a.resize(80, 24);
    press(&mut a, KeyCode::Esc);
    let s = screen(&a);
    assert_fits(&s, 80);
}

#[test]
fn test_nv_18_click_items_and_buttons() {
    // [NV-18] 項目のクリックで選んで決め、ボタンのクリックで押す(反映)。
    let (_t, mut a) = folder("nv18click");
    open(&mut a);
    // 右の欄の「+ 条件を足す」(フィルターの見出しの下)をクリック → 列の選び手。
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("+ 条件を足す"))
        .unwrap();
    let x = width::width(&line[..line.find("+ 条件を足す").unwrap()]);
    a.click(x as u16, y as u16);
    assert!(matches!(draft(&a).pick, Some(Pick::Column { .. })));
    press(&mut a, KeyCode::Esc);
    assert!(draft(&a).pick.is_none());
    assert_eq!(a.mode, Mode::Settings, "選び手の Esc は画面を閉じない");
    // 並べ替えに priority を足し(キー)、反映のボタンをクリック。
    section(&mut a, Sec::Sorts);
    press(&mut a, KeyCode::Enter);
    pick_column(&mut a, "priority");
    assert_eq!(draft(&a).s.sorts, [("priority".to_string(), Dir::Asc)]);
    let s = screen(&a);
    let line = s.lines().nth(20).unwrap();
    let x = width::width(&line[..line.find("[ 反映 ]").unwrap()]);
    a.click(x as u16 + 1, 20);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.settings.sorts, [("priority".to_string(), Dir::Asc)]);
    assert_eq!(labels(&a)[0], "b.md");
}

#[test]
fn test_sr_16_settings_keys() {
    // [SR-16] 設定の値の入力のモードには文字1つのキーを割り当てない(打った文字は値に入る)。
    // [SR-4] 設定の画面の下の帯のキーは表から。
    for b in keymap::BINDINGS
        .iter()
        .filter(|b| b.mode == Mode::SettingsText)
    {
        assert!(b.key.chars().count() > 1, "文字のキー: {}", b.key);
    }
    let (_t, mut a) = folder("sr16set");
    open(&mut a);
    let s = screen(&a);
    let band = s.lines().nth(21).unwrap();
    assert!(
        band.contains("ビューの設定") && band.contains("Enter 決める"),
        "{band}"
    );
    // `?` のヘルプから戻ると設定の画面のまま。
    ch(&mut a, '?');
    assert_eq!(a.mode, Mode::Help);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Settings);
    assert!(a.draft.is_some());
}
