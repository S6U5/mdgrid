//! 移動・検索・絞り込み・並べ替え・列・選択の試験(タスク 10。NV-1〜NV-5・NV-7・NV-8・NV-12・SR-6・SR-17・SR-18・OUT-1)。
use super::external::{Memory, MemoryLog};
use super::keymap::Mode;
use super::test_grid::{labels, open, vault, TASKS};
use super::test_screen::*;
use super::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::RefCell;
use std::rc::Rc;

fn names(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

fn shift(app: &mut App, code: KeyCode) {
    app.key(KeyEvent::new(code, KeyModifiers::SHIFT));
}

fn footer_line(app: &App) -> String {
    screen(app).lines().nth(21).unwrap().to_string()
}

/// 画面の列の見出しの行で `title` の始まりの桁。
fn header_x(app: &App, title: &str) -> u16 {
    let s = screen(app);
    let line = s.lines().nth(view::data_y(app) - 1).unwrap();
    width::width(&line[..line.find(title).unwrap()]) as u16
}

#[test]
fn test_nv_1_search_smartcase_and_n() {
    // [NV-1] `/Todo` は大文字小文字を区別、`/todo` は区別しない。`n` `N` で前後の一致へ。[SR-13] `/` `n` `N`。
    let (_t, mut a) = make(
        "nv1",
        &[
            ("a.md", "---\ntitle: Todo list\n---\n"),
            ("b.md", "---\ntitle: other\n---\n"),
            ("c.md", "---\ntitle: todo later\n---\n"),
        ],
    );
    ch(&mut a, '/');
    assert_eq!(a.mode, Mode::Search);
    typing(&mut a, "Todo");
    assert!(screen(&a).lines().nth(22).unwrap().starts_with("/Todo"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    let (ra, rc) = (a.rows[0].clone(), a.rows[2].clone());
    assert!(a.search_hit(&ra, Some("title")));
    assert!(!a.search_hit(&rc, Some("title")), "大文字があれば区別する");
    ch(&mut a, 'n');
    assert_eq!(a.row, 0, "一致は a だけ");
    ch(&mut a, '/');
    typing(&mut a, "todo");
    press(&mut a, KeyCode::Enter);
    assert!(a.search_hit(&ra, Some("title")) && a.search_hit(&rc, Some("title")));
    assert_eq!(a.row, 0);
    ch(&mut a, 'n');
    assert_eq!(a.row, 2);
    ch(&mut a, 'n');
    assert_eq!(a.row, 0, "端で回る");
    assert!(a.message.as_deref().unwrap().contains("回った"));
    ch(&mut a, 'N');
    assert_eq!(a.row, 2);
    // 一致の無い語は知らせる。Esc の取り消しで前の語と位置に戻る。
    ch(&mut a, '/');
    typing(&mut a, "zzz");
    assert!(screen(&a).contains("見つからない"));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.search.as_deref(), Some("todo"));
    assert_eq!(a.row, 2);
    assert_fits(&mut a);
}

#[test]
fn test_nv_2_quick_filter_and_esc() {
    // [NV-2] `\` で「会議」と打つ → 「会議」を含む行だけ。Esc で元に戻る。.base の絞り込みに重ねて効く。
    let (_t, mut a) = make(
        "nv2",
        &[
            ("a.md", "---\ntitle: 会議のメモ\n---\n"),
            ("b.md", "---\ntitle: 買い物\n---\n"),
            ("c.md", "---\ntitle: 定例会議\n---\n"),
        ],
    );
    ch(&mut a, '\\');
    assert_eq!(a.mode, Mode::Filter);
    ch(&mut a, '会');
    ch(&mut a, '議');
    assert_eq!(names(&a), ["a.md", "c.md"], "打つたびに絞る");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(names(&a), ["a.md", "c.md"]);
    // NV-23: 効いている語は表の上の検索の欄に出る(ヘッダーには出さない)。
    let s = screen(&a);
    assert!(s.lines().nth(2).unwrap().contains("検索: 会議"), "{s}");
    assert!(!s.lines().next().unwrap().contains("絞り込み"), "{s}");
    press(&mut a, KeyCode::Esc);
    assert_eq!(names(&a), ["a.md", "b.md", "c.md"]);
    // 入力の中の Esc でも解く。
    ch(&mut a, '\\');
    typing(&mut a, "買");
    assert_eq!(names(&a), ["b.md"]);
    press(&mut a, KeyCode::Esc);
    assert_eq!(names(&a), ["a.md", "b.md", "c.md"]);
    assert!(a.filter.is_none());

    // .base の絞り込み(status == "進行中")に重ねる。
    let tmp = vault("nv2base");
    let mut b = open(&tmp, TASKS, None);
    assert_eq!(labels(&b), ["a.md", "c.md"]);
    ch(&mut b, '\\');
    typing(&mut b, "読書");
    assert_eq!(labels(&b), ["c.md"]);
    typing(&mut b, "x");
    assert!(labels(&b).is_empty());
    assert_fits(&mut b);
}

const NUMS: &str = "views:\n  - type: table\n    name: 全部\n    order: [n, title]\n";

#[test]
fn test_nv_3_temporary_sort_keeps_base() {
    // [NV-3] 見出しを2回 → 降順、3回 → .base の並びに戻る。.base のバイトは変わらない。
    let tmp = Tmp::new("nv3");
    tmp.write("a.md", "---\nn: 2\ntitle: a\n---\n");
    tmp.write("b.md", "---\nn: 3\ntitle: b\n---\n");
    tmp.write("c.md", "---\nn: 1\ntitle: c\n---\n");
    tmp.write("d.md", "---\ntitle: d\n---\n");
    let mut a = open(&tmp, NUMS, None);
    let path = tmp.notes().join("tasks.base");
    let before = std::fs::read(&path).unwrap();
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md", "d.md"]);
    // 動作(`s`)。空は向きによらず末尾。
    col_named(&mut a, "n");
    ch(&mut a, 's');
    assert_eq!(labels(&a), ["c.md", "a.md", "b.md", "d.md"]);
    assert!(screen(&a).lines().nth(3).unwrap().contains("n↑"));
    ch(&mut a, 's');
    assert_eq!(labels(&a), ["b.md", "a.md", "c.md", "d.md"]);
    ch(&mut a, 's');
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md", "d.md"]);
    // 見出しのクリック(列の見出しは検索の欄の下の3行目。NV-23)。
    let x = header_x(&a, "title");
    a.click(x, 3);
    assert_eq!(a.sort, Some(("title".to_string(), false)));
    a.click(x, 3);
    assert_eq!(labels(&a), ["d.md", "c.md", "b.md", "a.md"]);
    a.click(x, 3);
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md", "d.md"]);
    assert_eq!(std::fs::read(&path).unwrap(), before, ".base は変えない");
}

#[test]
fn test_nv_4_hide_show_move_freeze() {
    // [NV-4] 列を隠す・戻す、並びを変える、左の列を固定する。
    let (_t, mut a) = make(
        "nv4",
        &[(
            "a.md",
            "---\nalpha: 1111111111\nbeta: 2222222222\ngamma: 3333333333\ndelta: 4444444444\nepsilon: 5555555555\nzeta: 6666666666\neta: 7777777777\n---\n",
        )],
    );
    assert_eq!(a.cols[..3], ["alpha", "beta", "gamma"]);
    col_named(&mut a, "beta");
    ch(&mut a, '-');
    assert!(!a.cols.contains(&"beta".to_string()));
    assert!(!screen(&a).lines().nth(3).unwrap().contains("beta"));
    assert!(screen(&a).lines().next().unwrap().contains("隠した列 1"));
    // 組み立て直しても隠したまま(見た目の状態として残すのは起動のタスク)。
    a.refresh();
    assert!(!a.cols.contains(&"beta".to_string()));
    ch(&mut a, '+');
    assert_eq!(a.cols[..3], ["alpha", "beta", "gamma"]);
    // 並びを変える。
    ch(&mut a, 'L');
    assert_eq!(a.cols[..3], ["alpha", "gamma", "beta"]);
    assert_eq!(a.cols[a.col], "beta");
    ch(&mut a, 'H');
    assert_eq!(a.cols[..3], ["alpha", "beta", "gamma"]);
    // 左の列(alpha)を固定して右端へ流しても、alpha は見える。
    ch(&mut a, '0');
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 1);
    ch(&mut a, '$');
    let head = screen(&a).lines().nth(3).unwrap().to_string();
    assert!(head.contains("alpha") && head.contains("eta"), "{head}");
    assert!(!head.contains("beta"), "{head}");
    assert_fits(&mut a);
    ch(&mut a, '0');
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 0);
}

#[test]
fn test_nv_5_marks_range_all_and_esc() {
    // [NV-5] Space を3行 → 下の帯に「選択 3」。`v` と Shift+矢印で範囲、Ctrl+A で全部、Esc で解く。
    let notes: Vec<(String, String)> = (0..5)
        .map(|i| (format!("n{i}.md"), format!("---\nn: {i}\n---\n")))
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, mut a) = make("nv5", &refs);
    for _ in 0..3 {
        press(&mut a, KeyCode::Char(' '));
    }
    assert!(footer_line(&a).contains("選択 3"), "{}", footer_line(&a));
    assert_eq!(a.row, 3);
    assert!(screen(&a).lines().nth(4).unwrap().starts_with('+'));
    press(&mut a, KeyCode::Esc);
    assert!(!footer_line(&a).contains("選択"));
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    ch(&mut a, 'v');
    ch(&mut a, 'j');
    ch(&mut a, 'j');
    assert_eq!(a.selection().len(), 3);
    ch(&mut a, 'v');
    ch(&mut a, 'j');
    assert_eq!(a.selection().len(), 3, "v で決めた範囲は印のまま");
    press(&mut a, KeyCode::Esc);
    shift(&mut a, KeyCode::Down);
    assert_eq!(a.selection().len(), 2);
    shift(&mut a, KeyCode::Up);
    shift(&mut a, KeyCode::Up);
    assert_eq!(a.selection().len(), 2);
    press(&mut a, KeyCode::Esc);
    ctrl(&mut a, 'a');
    assert!(footer_line(&a).contains("選択 5"));
    assert_fits(&mut a);
    press(&mut a, KeyCode::Esc);
    assert!(a.selection().is_empty());
}

#[test]
fn test_nv_7_column_ends_and_half_page() {
    // [NV-7] `0` `$` で列の先頭・末尾、Ctrl+D / Ctrl+U で半画面。
    let tmp = Tmp::new("nv7half");
    for i in 0..40 {
        tmp.write(
            &format!("n{i:02}.md"),
            &format!("---\na: {i}\nb: x\nc: y\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::None);
    ch(&mut a, '$');
    assert_eq!(a.col, 2);
    ch(&mut a, '0');
    assert_eq!(a.col, 0);
    let half = a.data_height() / 2;
    ctrl(&mut a, 'd');
    assert_eq!(a.row, half);
    ctrl(&mut a, 'd');
    assert_eq!(a.row, half * 2);
    ctrl(&mut a, 'u');
    assert_eq!(a.row, half);
}

const STATUS: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\nstatus: done\n---\n"),
    ("b.md", "---\ntitle: b\nstatus: todo\n---\n"),
    ("c.md", "---\ntitle: c\nstatus: done\n---\n"),
];

#[test]
fn test_nv_8_same_value_highlight_and_filter() {
    // [NV-8] status が done のセルで `,` → done の行だけ。`*` は同じ値の行を強調。[SR-18] `*`・`,`。
    let (_t, mut a) = make("nv8", STATUS);
    col_named(&mut a, "status");
    ch(&mut a, '*');
    assert_eq!(a.same_mark, Some(("status".into(), "done".into())));
    assert!(a.message.as_deref().unwrap().contains("2行"));
    assert_eq!(names(&a).len(), 3, "強調は絞らない");
    ch(&mut a, ',');
    assert_eq!(names(&a), ["a.md", "c.md"]);
    assert!(screen(&a)
        .lines()
        .next()
        .unwrap()
        .contains("同じ値 status=done"));
    ch(&mut a, ',');
    assert_eq!(names(&a).len(), 3, "同じ条件でもう一度押すと解く");
}

#[test]
fn test_sr_17_marks_act_as_comma_and_slash() {
    // [SR-17] 表で `、` → `,`(同じ値の行だけに絞る)と同じ動き。`・` → `/`(検索)と同じ動き。
    let (_t, mut a) = make("sr17nv", STATUS);
    col_named(&mut a, "status");
    ch(&mut a, '、');
    assert_eq!(names(&a), ["a.md", "c.md"]);
    ch(&mut a, '・');
    assert_eq!(a.mode, Mode::Search);
    // 入力の中では全角のまま入る。
    ch(&mut a, '・');
    assert_eq!(a.prompt.as_ref().unwrap().text, "・");
}

#[test]
fn test_sr_18_esc_clears_selection_then_filter() {
    // [SR-18] Esc は、選択があれば選択を、無ければ簡易の絞り込みを解く。
    let (_t, mut a) = make("sr18esc", STATUS);
    ch(&mut a, '\\');
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(names(&a), ["a.md", "c.md"]);
    ctrl(&mut a, 'a');
    assert_eq!(a.selection().len(), 2);
    press(&mut a, KeyCode::Esc);
    assert!(a.selection().is_empty());
    assert_eq!(names(&a), ["a.md", "c.md"], "1回目は選択だけ");
    press(&mut a, KeyCode::Esc);
    assert_eq!(names(&a).len(), 3, "2回目で絞り込みを解く");
}

#[test]
fn test_nv_12_edited_row_stays_until_save() {
    // [NV-12] status で絞ったビューで done(完了)に直す → 行は残り、印が付く。保存で消える。
    let tmp = vault("nv12");
    let mut a = open(&tmp, TASKS, None);
    assert_eq!(labels(&a), ["a.md", "c.md"]);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    for _ in 0..3 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "完了");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(labels(&a), ["a.md", "c.md"], "直した行は元の位置に留まる");
    let s = screen(&a);
    assert!(s.lines().nth(4).unwrap().contains("~a "), "{s}");
    assert!(
        s.lines()
            .nth(22)
            .unwrap()
            .contains("保存か行の移動で並び直す"),
        "{s}"
    );
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(labels(&a), ["c.md"], "保存で消える");
    assert!(a.held.is_empty());
}

#[test]
fn test_nv_12_move_releases_held_row() {
    // [NV-12] 移動の操作で、留めていた行は本来の位置へ(選んだ行は保つ)。並べ替えにもためた値が効く。
    let tmp = Tmp::new("nv12mv");
    tmp.write("a.md", "---\nn: 1\n---\n");
    tmp.write("b.md", "---\nn: 2\n---\n");
    tmp.write("c.md", "---\nn: 3\n---\n");
    let mut a = app_of(&tmp, ColorMode::None);
    col_named(&mut a, "n");
    ch(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "9");
    press(&mut a, KeyCode::Enter);
    assert_eq!(names(&a), ["a.md", "b.md", "c.md"]);
    assert!(a.held.contains(&a.rows[0]));
    ch(&mut a, 'j');
    assert_eq!(names(&a), ["b.md", "c.md", "a.md"]);
    assert_eq!(a.src.label(&a.cur_row().unwrap()), "b.md");
    // 取り消しも同じく留める。
    ch(&mut a, 'u');
    assert_eq!(names(&a), ["b.md", "c.md", "a.md"]);
    ch(&mut a, 'j');
    assert_eq!(names(&a), ["a.md", "b.md", "c.md"]);
}

#[test]
fn test_out_1_copy_selected_rows() {
    // [OUT-1] 選んだ2行を `Y` → 見出しつきのタブ区切り(見出しの行 + 2行)。
    let (_t, mut a) = make("out1sel", STATUS);
    let log = Rc::new(RefCell::new(MemoryLog::default()));
    a.clipboard = Box::new(Memory(log.clone()));
    press(&mut a, KeyCode::Char(' '));
    press(&mut a, KeyCode::Char(' '));
    ch(&mut a, 'Y');
    assert_eq!(
        log.borrow().system.last().unwrap(),
        "ノート\ttitle\tstatus\na.md\ta\tdone\nb.md\tb\ttodo"
    );
    assert!(a.message.as_deref().unwrap().contains("選んだ 2 行"));
}

#[test]
fn test_sr_6_click_selects_then_edits() {
    // [SR-6] 別の行のセルを1回クリック → 選ぶだけ。選んだ行のセルのクリック → 編集。ホイールで上下。
    let (_t, mut a) = make("sr6", STATUS);
    let x = header_x(&a, "status");
    a.click(x, 5);
    assert_eq!((a.row, a.mode), (1, Mode::Table));
    a.click(x, 5);
    assert_eq!(a.mode, Mode::Edit);
    press(&mut a, KeyCode::Esc);
    a.wheel(true);
    assert_eq!(a.row, 2);
    a.wheel(false);
    assert_eq!(a.row, 1);
}

/// 30字の値の列を5つ(a1 a2 a3 b4 b5)持つノート。
fn wide_cols(name: &str) -> (Tmp, App) {
    let v = "x".repeat(30);
    let note = format!("---\na1: {v}\na2: {v}\na3: {v}\nb4: {v}\nb5: {v}\n---\n");
    make(name, &[("a.md", note.as_str())])
}

#[test]
fn test_nv_4_frozen_columns_keep_selected_visible() {
    // [NV-4] [SR-3] 3列目で `F`、続けて `$` → 選んだ b5 が画面に出る(固定は一時的に減らす)。
    // 空にするのも入力ボックスも、見えている b5 に効く。
    let (_t, mut a) = wide_cols("nv4frz");
    col_named(&mut a, "a3");
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 3);
    ch(&mut a, '$');
    assert_eq!(a.cols[a.col], "b5");
    let head = screen(&a).lines().nth(3).unwrap().to_string();
    assert!(head.contains("b5") && head.contains("a1"), "{head}");
    assert!(view::selected_drawn(&a));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(view::cursor(&a).is_some(), "入力ボックスが見える");
    press(&mut a, KeyCode::Esc);
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).lines().nth(4).unwrap().contains("*∅"));
    assert_fits(&mut a);
}

#[test]
fn test_nv_4_freeze_refused_on_narrow_terminal() {
    // [NV-4] [SR-3] 狭い端末で、固定した列だけで画面を超える `F` は断って理由を出す。
    // 広い端末で固定してから狭くしても、選んだ列は見える。
    let (_t, mut a) = wide_cols("nv4narrow");
    a.resize(40, 24);
    col_named(&mut a, "a3");
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 0);
    assert!(a.message.as_deref().unwrap().contains("収まらない"));
    a.resize(80, 24);
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 3);
    a.resize(40, 24);
    for _ in 0..5 {
        assert!(view::selected_drawn(&a), "列 {}", a.cols[a.col]);
        ch(&mut a, 'l');
    }
    ch(&mut a, '0');
    assert!(view::selected_drawn(&a));
    assert_fits(&mut a);
}

#[test]
fn test_sr_3_hidden_cell_not_edited() {
    // [SR-3] 選んだセルが画面に描かれていなければ、空にする・編集を始めない(見えないセルに書かない)。
    let (_t, mut a) = make("sr3guard", STATUS);
    col_named(&mut a, "status");
    a.top = 2; // 選んだ行(0)を画面の外に置く(守りの試験のため直接)。
    a.clear();
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.as_deref().unwrap().contains("見えていない"));
    a.open_edit();
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_nv_4_show_column_restores_frozen() {
    // [NV-4] 固定した a, b のうち a を隠して戻すと、固定も2列に戻る。
    let (_t, mut a) = make("nv4show", &[("a.md", "---\na: 1\nb: 2\nc: 3\n---\n")]);
    col_named(&mut a, "b");
    ch(&mut a, 'F');
    assert_eq!(a.frozen, 2);
    ch(&mut a, '0');
    ch(&mut a, '-');
    assert_eq!(a.frozen, 1);
    ch(&mut a, '+');
    assert_eq!(a.cols[..2], ["a", "b"]);
    assert_eq!(a.frozen, 2);
    // 固定していない列を固定の境目に戻しても、固定は増えない。
    col_named(&mut a, "c");
    ch(&mut a, '-');
    ch(&mut a, '+');
    assert_eq!(a.frozen, 2);
}

#[test]
fn test_nv_12_goto_line_after_release() {
    // [NV-12] [NV-7] 留めた行があるときの `:120` は、先に並び直してから 120 行目へ移る。
    let tmp = Tmp::new("nv12goto");
    for i in 1..=150 {
        tmp.write(&format!("n{i:03}.md"), &format!("---\nn: {i}\n---\n"));
    }
    let mut a = app_of(&tmp, ColorMode::None);
    col_named(&mut a, "n");
    ch(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 'r');
    for _ in 0..3 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "999");
    press(&mut a, KeyCode::Enter);
    assert!(!a.held.is_empty());
    ch(&mut a, ':');
    typing(&mut a, "120");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.src.label(&a.cur_row().unwrap()), "n121.md");
    assert!(footer_line(&a).contains("120/150行"), "{}", footer_line(&a));
}
