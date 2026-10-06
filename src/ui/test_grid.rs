//! `.base` と表の組み立ての画面の試験(タスク 9 の残り)。材料は test_screen の Tmp を使う。

use super::grid::Slot;
use super::keymap::Action;
use super::test_screen::{buffer, ch, col_named, golden, press, screen, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::buffer::CellDiffOption;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use std::path::PathBuf;
use std::process::ExitCode;

/// 4つのノートと types.json(due は date)の保管庫(`<一時>/notes` が根)。
pub(super) fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date"}}"#,
    )
    .unwrap();
    tmp.write(
        "a.md",
        "---\ntitle: 会議のメモ\nstatus: 進行中\ndue: 2026-10-05\n---\n",
    );
    tmp.write(
        "b.md",
        "---\ntitle: 買い物\nstatus: 完了\ndue: someday\n---\n",
    );
    tmp.write(
        "c.md",
        "---\ntitle: 読書\nstatus: 進行中\ndue: 2026-09-01\n---\n",
    );
    tmp.write("d.md", "---\ntitle: 掃除\nstatus: 完了\n---\n");
    tmp
}

/// ビューが3つの `.base`(進行中・完了・全部)。
pub(super) const TASKS: &str = r#"formulas:
  odd: 'foo(status)'
properties:
  note.status:
    displayName: 状態
views:
  - type: table
    name: 進行中
    filters: 'status == "進行中"'
    order: [status, due, title]
  - type: table
    name: 完了
    filters: 'status == "完了"'
    order: [title, formula.odd]
  - type: table
    name: 全部
    groupBy:
      property: status
      direction: ASC
    order: [title, status]
"#;

pub(super) fn base_path(tmp: &Tmp, text: &str) -> PathBuf {
    let p = tmp.notes().join("tasks.base");
    std::fs::write(&p, text).unwrap();
    p
}

fn load(app: &mut App) {
    app.resize(80, 24);
    while !app.loaded() {
        app.load_step(100);
    }
}

/// main と同じ道筋(`open_target` → `App::new` → `set_base`)で `.base` を開く。
pub(super) fn open(tmp: &Tmp, text: &str, view: Option<&str>) -> App {
    let t = crate::open_target(&[base_path(tmp, text)], view).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    let (base, name, idx) = t.base.unwrap();
    app.set_base(base, name, idx.unwrap_or(0));
    load(&mut app);
    app
}

pub(super) fn labels(app: &App) -> Vec<String> {
    app.rows.iter().map(|r| app.src.label(r)).collect()
}

/// SR-9: どの行も幅以下。
fn assert_fits(s: &str) {
    for l in s.lines() {
        assert!(width::width(l) <= 80, "幅を超える行: {l}");
    }
}

#[test]
fn test_sr_1_base_header_and_tabs() {
    // [SR-1] ヘッダーは `.base` の名前と行数。[BV-13] タブに3つ、先頭のビューが選ばれている。
    let tmp = vault("sr1base");
    let app = open(&tmp, TASKS, None);
    let s = screen(&app);
    let mut lines = s.lines();
    assert!(lines.next().unwrap().contains("tasks.base  2行"), "{s}");
    let tabs = lines.next().unwrap();
    assert!(
        tabs.contains("[進行中]") && tabs.contains(" 完了 ") && tabs.contains(" 全部 "),
        "{s}"
    );
    assert!(s.contains("状態"), "displayName が見出しに出る(BV-5): {s}");
    assert_fits(&s);
    golden("sr_1_base", &s);
}

#[test]
fn test_bv_13_switch_view_changes_rows_and_cols() {
    // [BV-13] `]` `[` で切り替えると行と列が変わる。タブのクリックでも切り替わる。
    let tmp = vault("bv13");
    let mut app = open(&tmp, TASKS, None);
    assert_eq!(labels(&app), ["a.md", "c.md"]);
    assert_eq!(app.cols, ["status", "due", "title"]);
    ch(&mut app, ']');
    assert_eq!(app.view_index(), 1);
    assert_eq!(labels(&app), ["b.md", "d.md"]);
    assert_eq!(app.cols, ["title", "formula.odd"]);
    assert!(screen(&app).lines().nth(1).unwrap().contains("[完了]"));
    ch(&mut app, '[');
    assert_eq!(app.view_index(), 0);
    ch(&mut app, '[');
    assert_eq!(app.view_index(), 2, "端で回る");
    // タブの帯(2行目)の「完了」をクリック。
    let tabs = screen(&app).lines().nth(1).unwrap().to_string();
    let x = width::width(&tabs[..tabs.find("完了").unwrap()]) as u16;
    app.click(x, 1);
    assert_eq!(app.view_index(), 1);
    assert_eq!(labels(&app), ["b.md", "d.md"]);
}

#[test]
fn test_sr_2_group_heading_enter_folds() {
    // [SR-2] groupBy の見出しの行で Enter → そのまとまりの行が畳まれ、もう一度で開く。[BV-5] まとまりで並ぶ。
    let tmp = vault("sr2");
    let mut app = open(&tmp, TASKS, Some("全部"));
    assert_eq!(labels(&app), ["b.md", "d.md", "a.md", "c.md"]);
    assert_eq!(
        app.slots,
        [
            Slot::Head(0),
            Slot::Row(0),
            Slot::Row(1),
            Slot::Head(1),
            Slot::Row(2),
            Slot::Row(3)
        ]
    );
    let s = screen(&app);
    assert!(s.contains("▾ 状態: 完了(2件)"), "{s}");
    assert_eq!(app.row, 0);
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        app.slots,
        [Slot::Head(0), Slot::Head(1), Slot::Row(2), Slot::Row(3)]
    );
    assert_eq!(
        app.mode,
        keymap::Mode::Table,
        "見出しの Enter は編集を開かない"
    );
    let s = screen(&app);
    assert!(
        s.contains("▸ 状態: 完了(2件)") && !s.contains("b.md"),
        "{s}"
    );
    assert_fits(&s);
    golden("sr_2", &s);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.slots.len(), 6);
}

#[test]
fn test_bv_7_unsupported_formula_column() {
    // [BV-7] 評価できない式の列は全行に薄い `?`。選ぶと下の帯とメッセージ行に未対応の式が出る。
    // 値が `?` のテキストの列は普通の `?`(薄くない)。
    let tmp = vault("bv7col");
    tmp.write("e.md", "---\ntitle: \"?\"\nstatus: 完了\n---\n");
    let mut app = open(&tmp, TASKS, Some("完了"));
    assert_eq!(labels(&app), ["b.md", "d.md", "e.md"]);
    let buf = buffer(&app);
    let s = super::test_screen::text(&buf);
    // 見出しの行は4行目(検索の欄の下。NV-23)、ノートの行はその下。`?` のセルの薄さを見る。
    let dim_q = |y: u16| {
        (0..80u16)
            .filter(|&x| buf[(x, y)].symbol() == "?")
            .map(|x| buf[(x, y)].modifier.contains(Modifier::DIM))
            .collect::<Vec<_>>()
    };
    assert_eq!(dim_q(4), [true], "{s}");
    assert_eq!(dim_q(5), [true], "{s}");
    assert_eq!(dim_q(6), [false, true], "本当の値の ? は薄くない: {s}");
    col_named(&mut app, "formula.odd");
    let s = screen(&app);
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[21].contains("未対応"), "下の帯: {s}");
    assert!(lines[22].contains("foo"), "メッセージ行に式の名前: {s}");
    assert_fits(&s);
    golden("bv_7", &s);
    // 読むだけ: 編集は開かない。
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, keymap::Mode::Table);
}

#[test]
fn test_bv_7_unevaluable_filters_view_not_opened() {
    // [BV-7] filters が評価できないビューは開かず、関数名つきの理由を出す。ほかのビューへは移れる。
    let tmp = vault("bv7view");
    let text = "views:\n  - type: table\n    name: 壊れた\n    filters: 'bar(status)'\n  - type: table\n    name: 全部\n";
    let mut app = open(&tmp, text, None);
    assert!(app.rows.is_empty());
    assert!(app.view_error.as_deref().unwrap().contains("bar"));
    let s = screen(&app);
    assert!(
        s.contains("このビューは開けない") && s.contains("bar"),
        "{s}"
    );
    assert_fits(&s);
    ch(&mut app, ']');
    assert_eq!(app.rows.len(), 4);
    assert!(app.view_error.is_none());
}

#[test]
fn test_cv_2_misfit_date_marked() {
    // [CV-2] date の列の someday は値をそのまま見せて `!` を付ける。合う値には付けない。
    let tmp = vault("cv2");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    load(&mut app);
    let s = screen(&app);
    assert!(s.contains("!someday"), "{s}");
    assert!(s.contains("2026-10-05") && !s.contains("!2026"), "{s}");
}

#[test]
fn test_cv_4_types_json_number_right_aligned() {
    // [CV-4] 寄せは列の型(Source::kind)で決める。types.json で number の列は、最初の値が文字でも右寄せ。
    let tmp = Tmp::new("cv4types");
    tmp.write("a.md", "---\ncode: x\n---\n");
    tmp.write("b.md", "---\ncode: 5\n---\n");
    tmp.write("c.md", "---\ncode: 1200\n---\n");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    load(&mut app);
    let s0 = screen(&app);
    assert!(
        s0.lines().any(|l| l == " b      5"),
        "推定はテキスト(左寄せ): {s0}"
    );
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"code": "number"}}"#,
    )
    .unwrap();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    load(&mut app);
    let s = screen(&app);
    assert!(s.lines().any(|l| l == " b         5"), "{s}");
    assert!(s.contains("!x"), "型の合わない値に `!`(CV-2): {s}");
}

#[test]
fn test_cv_6_ambiguous_wide_column_aligned() {
    // [CV-6] ambiguous_wide=true で ○ を幅2と数え、○ の行と無い行で右の列の位置がそろう。
    // ratatui の数え方(幅1)とずれる分は、セルに ForcedWidth(2) を付けて後ろのセルを書かない。
    let tmp = Tmp::new("cv6");
    tmp.write("a.md", "---\nmark: ○○\nnext: Z\n---\n");
    tmp.write("b.md", "---\nmark: xxxx\nnext: Z\n---\n");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    load(&mut app);
    app.configure(&Config {
        ambiguous_wide: true,
        ..Config::default()
    });
    let buf = buffer(&app);
    let z_at = |y: u16| (0..80u16).find(|&x| buf[(x, y)].symbol() == "Z").unwrap();
    // ノートの行は検索の欄(NV-23)と列の見出しの下の4行目から。
    assert_eq!(z_at(4), z_at(5), "{}", super::test_screen::text(&buf));
    let o = (0..80u16).find(|&x| buf[(x, 4)].symbol() == "○").unwrap();
    assert!(matches!(
        buf[(o, 4)].diff_option,
        CellDiffOption::ForcedWidth(n) if n.get() == 2
    ));
    assert_eq!(buf[(o + 2, 4)].symbol(), "○");
    assert_fits(&super::test_screen::text(&buf));
    // 既定(幅1)では ○ の後ろにすぐ次の文字。
    app.configure(&Config::default());
    let buf = buffer(&app);
    let o = (0..80u16).find(|&x| buf[(x, 4)].symbol() == "○").unwrap();
    assert_eq!(buf[(o + 1, 4)].symbol(), "○");
}

#[test]
fn test_sr_3_resize_column_keys_and_drag() {
    // [SR-3] `<` `>` で選んだ列の幅が1桁ずつ変わる。列の見出しの境界のドラッグでも変わる。
    let tmp = vault("sr3w");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    load(&mut app);
    col_named(&mut app, "title");
    let j = app.col;
    let w0 = view::layout(&app).widths[j];
    ch(&mut app, '>');
    assert_eq!(view::layout(&app).widths[j], w0 + 1);
    ch(&mut app, '<');
    ch(&mut app, '<');
    assert_eq!(view::layout(&app).widths[j], w0 - 1);
    for _ in 0..100 {
        ch(&mut app, '<');
    }
    assert_eq!(view::layout(&app).widths[j], 1, "最小は1");
    assert_fits(&screen(&app));
    // 境界(列の右の区切り)を押して右へ動かす。
    let x = (0..80u16)
        .find(|&x| view::boundary_at(&app, x, 3).is_some_and(|d| d.col == j))
        .unwrap();
    app.click(x, 3);
    app.drag(x + 5);
    app.release();
    assert_eq!(view::layout(&app).widths[j], 6);
    assert_eq!(app.drag, None);
    // 動作の名前でも同じ(パレット)。
    app.apply(Action::Wider);
    assert_eq!(view::layout(&app).widths[j], 7);
}

#[test]
fn test_cli_2_view_option_opens_named_view() {
    // [CLI-2] `--view 完了` で2つ目のビューで開く。[CLI-1] `.base` のパスを受ける。
    let tmp = vault("cli2");
    let p = base_path(&tmp, TASKS);
    assert!(crate::check_paths(std::slice::from_ref(&p)).is_ok());
    let mut args: Vec<std::ffi::OsString> = vec![p.clone().into(), "--view".into(), "完了".into()];
    let o = crate::take_options(&mut args).unwrap();
    assert_eq!(o.view.as_deref(), Some("完了"));
    assert_eq!(args.len(), 1);
    let app = open(&tmp, TASKS, Some("完了"));
    assert_eq!(app.view_index(), 1);
    assert_eq!(labels(&app), ["b.md", "d.md"]);
    assert!(screen(&app).lines().nth(1).unwrap().contains("[完了]"));
}

#[test]
fn test_cli_4_unknown_view_one_line_exit_2() {
    // [CLI-4] 無いビューの名前は理由1行と終了コード 2。`.base` とフォルダを並べるのも理由1行。
    let tmp = vault("cli4");
    let p = base_path(&tmp, TASKS);
    let e = crate::open_target(std::slice::from_ref(&p), Some("無い"))
        .err()
        .unwrap();
    assert!(e.contains("無い") && e.contains("進行中"), "{e}");
    assert!(!e.contains('\n'));
    assert_eq!(crate::fail(&e), ExitCode::from(2));
    let e = crate::check_paths(&[p.clone(), tmp.notes()]).unwrap_err();
    assert!(!e.contains('\n'));
    let e = crate::open_target(&[tmp.notes()], Some("完了"))
        .err()
        .unwrap();
    assert!(!e.contains('\n'));
    let bad = tmp.notes().join("bad.base");
    std::fs::write(&bad, "views: [\n").unwrap();
    let e = crate::open_target(&[bad], None).err().unwrap();
    assert!(!e.contains('\n'), "{e}");
}

#[test]
fn test_ce_11_tab_after_row_leaves_view_does_not_write_other_row() {
    // [CE-11] [NV-12] 絞り込みのビューで値を直して Tab。直した行は元の位置に留まるので、同じ行の隣のセルが開く
    // (別の行に入力ボックスを重ねない)。行を移ると、直した行は表から外れる。
    let tmp = vault("ce11filter");
    let mut app = open(&tmp, TASKS, None);
    assert_eq!(labels(&app), ["a.md", "c.md"]);
    col_named(&mut app, "status");
    press(&mut app, KeyCode::Enter);
    for _ in 0..3 {
        press(&mut app, KeyCode::Backspace);
    }
    for c in "完了".chars() {
        ch(&mut app, c);
    }
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.mode, keymap::Mode::Edit);
    let i = app.input.as_ref().unwrap();
    assert_eq!(
        (app.src.label(&i.row).as_str(), i.col.as_str()),
        ("a.md", "due")
    );
    assert_eq!(labels(&app), ["a.md", "c.md"]);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.changes.count(), 1, "a.md の status だけ");
    ch(&mut app, 'j');
    assert_eq!(labels(&app), ["c.md"]);
    assert_eq!(app.src.label(&app.cur_row().unwrap()), "c.md");
}

#[test]
fn test_ce_1_input_closed_when_row_leaves_on_rebuild() {
    // [CE-1] 編集中の組み立て直し(読み込みの1歩など)で入力の行が表から外れたら、入力を閉じて理由を出す。
    let tmp = vault("ce1guard");
    let mut app = open(&tmp, TASKS, None);
    col_named(&mut app, "title");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, keymap::Mode::Edit);
    let row = app.cur_row().unwrap();
    tmp.write("a.md", "---\ntitle: 会議のメモ\nstatus: 完了\n---\n");
    app.src.reload(&row).unwrap();
    app.refresh();
    assert_eq!(app.mode, keymap::Mode::Table);
    assert!(app.input.is_none());
    assert!(app.message.as_deref().unwrap().contains("外れた"));
    // 行が残る組み立て直しでは閉じない。
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, keymap::Mode::Edit);
    app.refresh();
    assert_eq!(app.mode, keymap::Mode::Edit);
}

#[test]
fn test_cv_4_formula_number_and_bool_aligned() {
    // [CV-4] formula の列は見えている行の値で寄せる: 数は右寄せ、真偽は中央寄せ。
    let tmp = vault("cv4formula");
    let text = "formulas:\n  num: '1 + 2'\n  done: 'status == \"完了\"'\nviews:\n  - type: table\n    name: 式\n    order: [formula.num, formula.done, title]\n";
    let app = open(&tmp, text, None);
    assert_eq!(
        app.kinds,
        [
            mdgrid::types::Kind::Number,
            mdgrid::types::Kind::Checkbox,
            mdgrid::types::Kind::Text
        ]
    );
    let s = screen(&app);
    // 見出し「num」(式の名前。BV-24)の右端に `#3`、「done」の列に `#false`(`#` は読むだけの印)。
    assert!(
        s.lines().any(|l| l == ">a       #3 #false 会議のメモ"),
        "{s}"
    );
}
