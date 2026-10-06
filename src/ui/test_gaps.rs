//! 照合で見つかった試験の抜けを埋める画面の試験(OUT-1・SR-10・BV-10・BV-12・BV-16・SC-8・WB-13・CV-1・CV-3・
//! CV-5・NV-1・NV-8・SR-17)。
use super::external::{Memory, MemoryLog};
use super::keymap::{Action, Mode};
use super::test_grid::{open, vault};
use super::test_screen::*;
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use ratatui::Terminal;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn memory(a: &mut App) -> Rc<RefCell<MemoryLog>> {
    let log = Rc::new(RefCell::new(MemoryLog::default()));
    a.clipboard = Box::new(Memory(log.clone()));
    log
}

/// base64(標準の字母、`=` の埋め)を戻す。
fn unbase64(s: &str) -> Vec<u8> {
    const ABC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for c in s.bytes().filter(|&c| c != b'=') {
        let v = ABC.iter().position(|&x| x == c).expect("base64 の字") as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

/// OSC 52 の列(`ESC ] 52 ; c ; <base64> BEL`)の中身を戻す。
fn osc52_text(seq: &[u8]) -> String {
    let s = std::str::from_utf8(seq).unwrap();
    let b64 = s
        .strip_prefix("\x1b]52;c;")
        .and_then(|r| r.strip_suffix('\x07'))
        .expect("OSC 52 の形");
    String::from_utf8(unbase64(b64)).unwrap()
}

fn label_of(a: &App, i: usize) -> String {
    a.src.label(&a.rows[i])
}

fn cur_label(a: &App) -> String {
    a.src.label(&a.cur_row().unwrap())
}

/// 画面の上の、列 `name` の左端の桁と幅(見えている列)。
fn col_x(a: &App, name: &str) -> (u16, usize) {
    let (lay, cols) = view::visible_layout(a);
    let j = a.cols.iter().position(|c| c == name).unwrap();
    let mut x = lay.data_x();
    for &(k, w) in &cols {
        if k == j {
            return (x as u16, w);
        }
        x += w + 1;
    }
    panic!("列 {name} が見えていない");
}

/// 表の行 `i`(slots の添字)の列 `name` のセルの範囲の書式。
fn cell_mods(buf: &Buffer, a: &App, i: usize, name: &str) -> Vec<Modifier> {
    let (x, w) = col_x(a, name);
    let y = (view::data_y(a) + i - a.top) as u16;
    (x..x + w as u16).map(|cx| buf[(cx, y)].modifier).collect()
}

fn poll_until(a: &mut App, n: usize) {
    for _ in 0..400 {
        a.poll();
        if a.rows.len() == n {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("行が {n} にならない: {}", a.rows.len());
}

// ---- OUT-1 ----

const BASE_COPY: &str = "formulas:\n  num: '1 + 2'\nviews:\n  - type: table\n    name: 全部\n    order: [title, file.name, formula.num]\n";

#[test]
fn test_out_1_y_with_selection_copies_rows_with_header() {
    // [OUT-1] 2行を選んで(Space×2)`y` → 見出しつきのタブ区切り2行が OSC 52 で出る(Ctrl+C も同じ)。
    // 計算の列(file.*・formula.*)は画面と同じ値で写す。
    let tmp = vault("out1y");
    let mut a = open(&tmp, BASE_COPY, None);
    let log = memory(&mut a);
    press(&mut a, KeyCode::Char(' '));
    press(&mut a, KeyCode::Char(' '));
    ch(&mut a, 'y');
    let want = format!(
        "ノート\ttitle\tfile.name\tformula.num\n{}\t会議のメモ\ta.md\t3\n{}\t買い物\tb.md\t3",
        label_of(&a, 0),
        label_of(&a, 1)
    );
    {
        let l = log.borrow();
        assert_eq!(l.osc.len(), 1);
        assert_eq!(osc52_text(&l.osc[0]), want);
        assert_eq!(l.system, vec![want.clone()]);
    }
    assert!(a.message.as_deref().unwrap().contains("選んだ 2 行"));
    ctrl(&mut a, 'c');
    assert_eq!(osc52_text(&log.borrow().osc[1]), want, "Ctrl+C も選んだ行");
}

#[test]
fn test_out_1_y_copies_computed_column_value() {
    // [OUT-1] 選択が無ければ `y` は今のセル1つ。計算の列のセルは画面と同じ値(空にしない)。
    let tmp = vault("out1f");
    let mut a = open(&tmp, BASE_COPY, None);
    let log = memory(&mut a);
    col_named(&mut a, "formula.num");
    ch(&mut a, 'y');
    col_named(&mut a, "file.name");
    ch(&mut a, 'y');
    let l = log.borrow();
    assert_eq!(osc52_text(&l.osc[0]), "formula.num\t3");
    assert_eq!(osc52_text(&l.osc[1]), "file.name\ta.md");
}

// ---- SR-10 ----

#[test]
fn test_sr_10_dumb_term_draws_no_emoji() {
    // [SR-10] `TERM=dumb` では色に加えて絵文字も出さない(値の中の絵文字は同じ幅の `?` に)。
    // `NO_COLOR` だけなら絵文字はそのまま。
    let notes = &[
        ("a.md", "---\ntitle: 🍎りんご\nok: ✅\n---\n"),
        ("b.md", "---\ntitle: 国旗🇯🇵と☀\u{fe0f}\nok: x\n---\n"),
    ];
    let dumb = |k: &str| (k == "TERM").then(|| "dumb".to_string());
    let (_t, mut a) = make("sr10dumb", notes);
    a.color = ColorMode::detect(dumb);
    a.no_emoji = dumb_terminal(dumb);
    assert_eq!(a.color, ColorMode::None);
    assert!(a.no_emoji);
    let s = screen(&a);
    for e in ["🍎", "✅", "🇯", "🇵", "☀"] {
        assert!(!s.contains(e), "{e} が出ている: {s}");
    }
    assert!(s.contains("? りんご"), "{s}");
    // 桁はずれない: 列の見出しの位置と列の幅は、絵文字を出すときと同じ。
    let no_color = |k: &str| (k == "NO_COLOR").then(|| "1".to_string());
    let (_t2, mut b) = make("sr10nc", notes);
    b.color = ColorMode::detect(no_color);
    b.no_emoji = dumb_terminal(no_color);
    assert!(!b.no_emoji);
    let s2 = screen(&b);
    assert!(s2.contains("🍎りんご"), "{s2}");
    assert_eq!(s.lines().nth(3), s2.lines().nth(3));
    assert_eq!(view::layout(&a).widths, view::layout(&b).widths);
}

// ---- BV-10 ----

#[test]
fn test_bv_10_deleted_selected_note_picks_nearest_row() {
    // [BV-10] 画面の経路(poll → refresh)で、選んだノートを消すと下の行、末尾なら前の行が選ばれる。
    // 核の vault::reselect と同じ決まり(SC-14 のため画面は鍵で探し、無ければ同じ添字を末尾で抑える)。
    let (t, mut a) = make(
        "bv10near",
        &[
            ("a.md", "---\nx: 1\n---\n"),
            ("b.md", "---\nx: 2\n---\n"),
            ("c.md", "---\nx: 3\n---\n"),
        ],
    );
    let paths = |a: &App| -> Vec<PathBuf> { a.rows.iter().map(|r| PathBuf::from(&r.0)).collect() };
    ch(&mut a, 'j');
    assert_eq!(cur_label(&a), "b.md");
    let old = paths(&a);
    std::fs::remove_file(t.notes().join("b.md")).unwrap();
    poll_until(&mut a, 2);
    assert_eq!(cur_label(&a), "c.md", "下の行");
    assert_eq!(
        mdgrid::vault::reselect(&old, 1, &paths(&a)),
        Some(a.row),
        "reselect と同じ"
    );
    // 末尾の行を消す → 前の行。
    assert_eq!(a.row, 1);
    let old = paths(&a);
    std::fs::remove_file(t.notes().join("c.md")).unwrap();
    poll_until(&mut a, 1);
    assert_eq!(cur_label(&a), "a.md", "前の行");
    assert_eq!(mdgrid::vault::reselect(&old, 1, &paths(&a)), Some(a.row));
}

// ---- BV-12 ----

#[test]
fn test_bv_12_conflicting_type_cell_enter_is_read_only() {
    // [BV-12] 2つの根で types.json の型が食い違う列 → テキストで表示し、Enter は入力を開かず読むだけの理由を出す。
    let tmp = Tmp::new("bv12ui");
    for (root, ty, due) in [("A", "date", "2026-10-01"), ("B", "text", "later")] {
        let r = tmp.notes().join(root);
        std::fs::create_dir_all(r.join(".obsidian")).unwrap();
        std::fs::write(
            r.join(".obsidian/types.json"),
            format!(r#"{{"types": {{"due": "{ty}"}}}}"#),
        )
        .unwrap();
        std::fs::write(r.join("n.md"), format!("---\ndue: {due}\nt: x\n---\n")).unwrap();
    }
    let src = Markdown::open(&[tmp.notes().join("A"), tmp.notes().join("B")]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    while !a.loaded() {
        a.load_step(100);
    }
    assert_eq!(a.kind_of("due"), mdgrid::types::Kind::Text);
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "入力を開かない");
    assert!(a.input.is_none());
    let m = a.message.clone().unwrap();
    assert!(m.contains("読むだけ") && m.contains("types.json"), "{m}");
    let s = screen(&a);
    assert!(s.lines().nth(22).unwrap().contains("読むだけ"), "{s}");
    // 食い違わない列は直せる。
    col_named(&mut a, "t");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
}

// ---- BV-16 ----

/// ヘッダーの「読み込み中 N/M」の N。
fn loaded_on_screen(s: &str) -> usize {
    let top = s.lines().next().unwrap();
    let rest = &top[top.find("読み込み中 ").expect("読み込み中の表示") + "読み込み中 ".len()..];
    rest[..rest.find('/').unwrap()].parse().unwrap()
}

#[test]
fn test_bv_16_ten_thousand_notes_first_screen_fast() {
    // [BV-16] 1万ノート → 最初の読み込みの1歩のあと1秒以内に画面が出て、進捗(読み込み中 N/1万)が増えて画面に出る。
    let tmp = Tmp::new("bv16big");
    for i in 0..10_000 {
        std::fs::write(
            tmp.notes().join(format!("n{i:05}.md")),
            format!("---\nn: {i}\n---\n"),
        )
        .unwrap();
    }
    let t0 = Instant::now();
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(80, 24);
    a.load_step(crate::LOAD_BUDGET);
    let s = screen(&a);
    let took = t0.elapsed();
    assert!(took < Duration::from_secs(1), "最初の画面まで {took:?}");
    assert!(!a.loaded());
    // 読んだ分の行が出る(読む順はフォルダの並びなので、どのノートかは問わない)。
    assert!(s.lines().nth(4).unwrap()[1..].starts_with('n'), "{s}");
    let first = loaded_on_screen(&s);
    assert!(first > 0 && first < 10_000, "{first}");
    assert_eq!(a.rows.len(), first, "{s}");
    a.load_step(crate::LOAD_BUDGET);
    let second = loaded_on_screen(&screen(&a));
    assert!(second > first, "進捗が増える: {first} → {second}");
}

// ---- SC-8 ----

#[test]
fn test_sc_8_other_view_type_shows_unsupported_reason() {
    // [SC-8] table 以外のビュー(list・map・kanban・cards)を選ぶと開かず、画面に未対応の理由が出る。
    let tmp = vault("sc8ui");
    let text = "views:\n  - type: table\n    name: 表\n  - type: list\n    name: L\n  - type: map\n    name: M\n  - type: kanban\n    name: K\n  - type: cards\n    name: C\n";
    let mut a = open(&tmp, text, None);
    assert!(a.view_error.is_none());
    for kind in ["list", "map", "kanban", "cards"] {
        ch(&mut a, ']');
        let s = screen(&a);
        assert!(a.rows.is_empty());
        assert!(
            s.contains("このビューは開けない") && s.contains(kind) && s.contains("未対応"),
            "{kind}: {s}"
        );
    }
    ch(&mut a, ']');
    assert!(a.view_error.is_none());
    assert_eq!(a.rows.len(), 4);
}

// ---- WB-13 ----

#[test]
fn test_wb_13_sync_conflict_row_marked_and_explained() {
    // [WB-13] `note.sync-conflict-20260930-1.md` のある表 → その行に印 `!`、選ぶとメッセージ行に案内。
    let (_t, mut a) = make(
        "wb13ui",
        &[
            ("note.md", "---\ns: 1\n---\n"),
            ("note.sync-conflict-20260930-1.md", "---\ns: 2\n---\n"),
        ],
    );
    let i = (0..a.rows.len())
        .find(|&i| label_of(&a, i).contains("sync-conflict"))
        .unwrap();
    let s = screen(&a);
    assert!(s.contains("!note.sync-conflict-2"), "{s}");
    let plain = s.lines().find(|l| l[1..].starts_with("note ")).unwrap();
    assert!(!plain.contains("!note "), "普通の行に印は無い: {s}");
    a.row = i;
    a.scroll_into_view();
    let s = screen(&a);
    assert!(
        s.lines().nth(22).unwrap().contains("同期の競合ファイル"),
        "{s}"
    );
    a.row = 1 - i;
    let s = screen(&a);
    assert!(!s.contains("同期の競合ファイル"), "{s}");
}

// ---- CV-1 ----

#[test]
fn test_cv_1_null_is_dim_but_empty_string_and_missing_are_not() {
    // [CV-1] null の `∅` は薄い表示(DIM)、空の文字列の `""` とキーの無い空欄は薄くしない。
    let (_t, mut a) = make(
        "cv1dim",
        &[
            ("a.md", "---\ns: null\nt: 1\n---\n"),
            ("b.md", "---\ns: \"\"\nt: 2\n---\n"),
            ("c.md", "---\nt: 3\n---\n"),
        ],
    );
    col_named(&mut a, "t");
    let buf = buffer(&a);
    let s = text(&buf);
    assert!(s.lines().nth(4).unwrap().contains('∅'), "{s}");
    assert!(s.lines().nth(5).unwrap().contains("\"\""), "{s}");
    let null = cell_mods(&buf, &a, 0, "s");
    assert!(null.iter().all(|m| m.contains(Modifier::DIM)), "{null:?}");
    for i in [1, 2] {
        let m = cell_mods(&buf, &a, i, "s");
        assert!(
            m.iter().all(|m| !m.contains(Modifier::DIM)),
            "行 {i}: {m:?}"
        );
    }
}

// ---- CV-3 ----

#[test]
fn test_cv_3_detail_shows_every_line_of_multiline_value() {
    // [CV-3] 改行を含む値は表では1行目と `…⏎`、`K` の詳細の表示では1行目も2行目も出る。
    let (_t, mut a) = make(
        "cv3detail",
        &[(
            "a.md",
            "---\nmemo: |\n  一行目の文\n  二行目の文\ntitle: t\n---\n",
        )],
    );
    let s = screen(&a);
    assert!(
        s.contains("一行目の文…⏎") && !s.contains("二行目の文"),
        "{s}"
    );
    col_named(&mut a, "memo");
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    let s = screen(&a);
    assert!(s.contains("一行目の文"), "{s}");
    assert!(s.contains("二行目の文"), "{s}");
}

// ---- CV-5 ----

#[test]
fn test_cv_5_width_from_visible_rows_changes_on_scroll() {
    // [CV-5] スクロールの外(24行より下)の行にだけ長い値 → 最初の列の幅は見えている行の短い値から決まり、
    // スクロールしてその行が見えると広がる。
    let tmp = Tmp::new("cv5scroll");
    for i in 0..30 {
        let v = if i == 29 { "x".repeat(20) } else { "ab".into() };
        tmp.write(&format!("n{i:02}.md"), &format!("---\nv: {v}\n---\n"));
    }
    let mut a = app_of(&tmp, ColorMode::None);
    assert!(a.data_height() < 29);
    assert_eq!(view::layout(&a).widths, [2], "見えている行は ab だけ");
    assert!(!screen(&a).contains("xxxx"));
    a.apply(Action::Bottom);
    assert!(a.top > 0);
    assert_eq!(view::layout(&a).widths, [20], "長い値が見えると広がる");
    assert!(screen(&a).contains(&"x".repeat(20)));
    a.apply(Action::Top);
    assert_eq!(view::layout(&a).widths, [2]);
}

// ---- NV-1・NV-8(強調の描画)----

#[test]
fn test_nv_1_nv_8_highlight_is_drawn_on_cells() {
    // [NV-1] 検索の一致のセルに下線。[NV-8] `*` で同じ値の行のそのセルに下線。一致しないセルには付かない。
    let (_t, mut a) = make(
        "nv1hl",
        &[
            ("a.md", "---\ntitle: 会議\nstatus: done\n---\n"),
            ("b.md", "---\ntitle: 買い物\nstatus: todo\n---\n"),
            ("c.md", "---\ntitle: 会議の続き\nstatus: done\n---\n"),
        ],
    );
    let under = |buf: &Buffer, a: &App, i: usize, c: &str| {
        cell_mods(buf, a, i, c)
            .iter()
            .any(|m| m.contains(Modifier::UNDERLINED))
    };
    ch(&mut a, '/');
    typing(&mut a, "会議");
    press(&mut a, KeyCode::Enter);
    let buf = buffer(&a);
    assert!(under(&buf, &a, 0, "title") && under(&buf, &a, 2, "title"));
    assert!(!under(&buf, &a, 1, "title"));
    assert!(!under(&buf, &a, 1, "status"));
    a.search = None;
    a.row = 0;
    col_named(&mut a, "status");
    ch(&mut a, '*');
    let buf = buffer(&a);
    assert!(under(&buf, &a, 0, "status") && under(&buf, &a, 2, "status"));
    assert!(!under(&buf, &a, 1, "status"));
    assert!(!under(&buf, &a, 2, "title"), "ほかの列には付かない");
}

// ---- SR-17(端末のカーソル)----

#[test]
fn test_sr_17_terminal_cursor_after_draw_is_at_input() {
    // [SR-17] 入力ボックスでは、draw のあと端末のカーソルが入力の位置にある(IME の変換の窓がそこに出る)。
    let (_t, mut a) = make("sr17cur", &[("a.md", "---\ntitle: abc\n---\n")]);
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| draw(f, &a)).unwrap();
    let want = view::cursor(&a).expect("入力の位置");
    let got = term.get_cursor_position().unwrap();
    assert_eq!((got.x, got.y), want);
    let (x, _) = col_x(&a, "title");
    assert_eq!(want.1 as usize, view::data_y(&a));
    assert!(want.0 >= x, "入力ボックスの中: {want:?}");
    // 1字打つとカーソルも右へ(End で続きを直す形にしてから。リストの直後に打つと置き換わる。CE-3)。
    press(&mut a, KeyCode::End);
    ch(&mut a, 'd');
    term.draw(|f| draw(f, &a)).unwrap();
    let got2 = term.get_cursor_position().unwrap();
    assert_eq!(got2.x, got.x + 1);
}
