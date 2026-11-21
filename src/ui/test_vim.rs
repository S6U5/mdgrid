//! vim モード(設定の `[keys]` の `vim`)と、パレットの `q!`。
//! 期待値は specs/requirements/screen.md の「vim モード」の節(screen.vim-*・screen.quit-discard)の受け入れの例から。

use super::filters::{FilterWin, Step};
use super::keymap::{self, Action, Mode};
use super::settings::{TextEntry, TextKind};
use super::test_screen::{col_named, ctrl, edit_cell, make, press, screen, typing, Tmp};
use super::vim::{normal, CursorShape, Edited};
use super::width::{fit, Align};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::NewValue;
use ratatui::crossterm::event::KeyCode;

const NOTES: &[(&str, &str)] = &[
    (
        "a.md",
        "---\ntitle: 会議の見積もり\nstatus: done\ndue: 2026-11-01\n---\n",
    ),
    (
        "b.md",
        "---\ntitle: 買い物\nstatus: todo\ndue: 2026-11-02\n---\n",
    ),
    (
        "c.md",
        "---\ntitle: 掃除\nstatus: doing\ndue: 2026-11-03\n---\n",
    ),
];

/// 設定の `[keys]` の `vim` を当てた表。
fn with_vim(name: &str, vim: bool, keys: &[(&str, &str, &str)]) -> (Tmp, App) {
    let (t, mut a) = make(name, NOTES);
    let c = Config {
        vim,
        keys: keys
            .iter()
            .map(|(m, k, x)| ((*m).to_string(), (*k).to_string(), (*x).to_string()))
            .collect(),
        ..Config::default()
    };
    let w = a.configure(&c);
    assert!(w.is_empty(), "設定の警告: {w:?}");
    (t, a)
}

fn vim_app(name: &str) -> (Tmp, App) {
    with_vim(name, true, &[])
}

fn ch(a: &mut App, c: char) {
    super::test_screen::ch(a, c);
}

fn keys(a: &mut App, s: &str) {
    for c in s.chars() {
        ch(a, c);
    }
}

/// ノートの行(ファイルの名前)を選ぶ。
fn select(a: &mut App, label: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]) == label))
        .unwrap_or_else(|| panic!("{label} の行が無い"));
}

fn row_id(a: &App, label: &str) -> mdgrid::source::RowId {
    a.rows
        .iter()
        .find(|r| a.src.label(r) == label)
        .cloned()
        .unwrap_or_else(|| panic!("{label} の行が無い"))
}

fn labels(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| a.src.label(r)).collect()
}

/// 入力の欄の文字。
fn field(a: &App) -> String {
    a.vim_line().expect("入力の欄が開いている").0
}

fn pending(a: &App, label: &str, col: &str) -> Option<NewValue> {
    a.changes.pending(&row_id(a, label), col).cloned()
}

/// 下の帯の行(未保存の数で始まる行)。
fn footer(a: &App) -> String {
    screen(a)
        .lines()
        .find(|l| l.trim_start().starts_with("未保存"))
        .unwrap_or_default()
        .to_string()
}

/// title のセルの入力を開く(会議の見積もり)。
fn open_title(a: &mut App) {
    select(a, "a.md");
    col_named(a, "title");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit, "{:?}", a.message);
}

// ---- screen.vim-setting(本物の実行ファイルの確かめは tests/test_vim_e2e.rs) ----

#[test]
fn test_vim_setting_off_esc_cancels_on_esc_goes_normal() {
    // [utest -> req~screen.vim-input-modes~1] 設定が無い → セルの入力の欄で Esc を押すと、今どおり入力を取りやめて閉じる。
    // `vim = true` → 同じ Esc で入力の欄がノーマルのモードになり、閉じない。
    let (_t, mut a) = make("vimset-off", NOTES);
    open_title(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "vim が無ければ Esc で閉じる");
    let (_t, mut a) = vim_app("vimset-on");
    open_title(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Edit, "vim = true では閉じない");
    assert_eq!(a.key_mode(), Mode::InputNormal, "ノーマルのモード");
}

#[test]
fn test_vim_setting_is_read_from_keys_table() {
    // [screen.vim-setting] 設定の `[keys]` の `vim` を読む(既定は false、警告なし)。真偽でなければ警告して false のまま。
    let (c, w) = mdgrid::config::parse("").unwrap();
    assert!(!c.vim && w.is_empty());
    let (c, w) = mdgrid::config::parse("[keys]\nvim = true\n").unwrap();
    assert!(c.vim, "vim = true を読む");
    assert!(w.is_empty(), "警告なし: {w:?}");
    let (c, w) =
        mdgrid::config::parse("[keys]\nvim = true\n[keys.table]\n\"ctrl+f\" = \"search\"\n")
            .unwrap();
    assert!(c.vim && w.is_empty(), "割り当て直しと一緒に書ける: {w:?}");
    assert_eq!(c.keys.len(), 1);
    let (c, w) = mdgrid::config::parse("[keys]\nvim = \"yes\"\n").unwrap();
    assert!(!c.vim);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("keys.vim"), "{w:?}");
}

#[test]
fn test_vim_setting_off_adds_no_keys() {
    // [utest -> req~screen.vim-remap~1] `vim` が無い設定では、キーの表は既定の表のまま(vim モードのキーが無い)。
    let (_t, a) = make("vimset-keys", NOTES);
    assert_eq!(a.keys.len(), keymap::BINDINGS.len());
    assert!(a.keys.iter().all(|b| b.mode != Mode::InputNormal));
    let (_t, a) = vim_app("vimset-keys-on");
    assert_eq!(
        a.keys.len(),
        keymap::BINDINGS.len() + keymap::VIM_BINDINGS.len()
    );
}

// ---- screen.vim-input-modes ----

#[test]
fn test_vim_input_modes_insert_then_normal_then_cancel_or_commit() {
    // [utest -> req~screen.vim-input-modes~1] `vim = true` で title のセルを Enter → 打った字が入る。Esc → 欄は開いたまま。
    // もう一度 Esc → 取りやめて閉じ、値は前のまま。ノーマルのモードで Enter → 決まる。
    let (_t, mut a) = vim_app("vimin");
    open_title(&mut a);
    ch(&mut a, 'X');
    assert!(field(&a).contains('X'), "打った字が入る: {}", field(&a));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Edit, "欄は開いたまま");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "取りやめて閉じる");
    assert_eq!(pending(&a, "a.md", "title"), None, "値は前のまま");
    assert_eq!(a.changes.count(), 0);
    open_title(&mut a);
    ch(&mut a, 'Y');
    let typed = field(&a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.key_mode(), Mode::InputNormal);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "ノーマルのモードで Enter → 決まる");
    assert_eq!(pending(&a, "a.md", "title"), Some(NewValue::Str(typed)));
}

#[test]
fn test_vim_input_modes_every_input_box_starts_in_insert() {
    // [utest -> req~screen.vim-input-modes~1] 探す・検索・パレット・設定の値の入力・絞り込みの窓の値・新しいノートの名前の欄も、
    // 挿入のモードで始まり(打った字が入る)、Esc でノーマルのモード、ノーマルのモードの Esc で取りやめる。
    let (_t, mut a) = vim_app("vimin-all");
    let check = |a: &mut App, mode: Mode, what: &str| {
        assert_eq!(a.mode, mode, "{what} が開く");
        assert_eq!(a.key_mode(), mode, "{what} は挿入のモードで始まる");
        keys(a, "ab");
        assert!(
            field(a).ends_with("ab"),
            "{what}: 打った字が入る: {}",
            field(a)
        );
        press(a, KeyCode::Esc);
        assert_eq!(a.mode, mode, "{what}: Esc では閉じない");
        assert_eq!(a.key_mode(), Mode::InputNormal, "{what}: ノーマルのモード");
        press(a, KeyCode::Esc);
        assert_ne!(a.mode, mode, "{what}: ノーマルのモードの Esc で取りやめる");
    };
    ch(&mut a, '/');
    check(&mut a, Mode::Search, "探す");
    ch(&mut a, '\\');
    check(&mut a, Mode::Filter, "検索");
    ch(&mut a, ':');
    check(&mut a, Mode::Palette, "パレット");
    ch(&mut a, 'a');
    check(&mut a, Mode::Edit, "新しいノートの名前");
    assert!(a.note.flow.is_none(), "新しいノートは作らない");
    // 設定の値の入力(ビューの設定の画面の「含む」の値など)。
    a.open_settings();
    a.draft.as_mut().unwrap().text = Some(TextEntry {
        col: "status".into(),
        kind: TextKind::Contains,
        text: String::new(),
        edit: None,
    });
    a.set_mode(Mode::SettingsText);
    check(&mut a, Mode::SettingsText, "設定の値の入力");
    // 絞り込みの窓の値の入力。
    a.set_mode(Mode::Table);
    a.filter_win = Some(FilterWin {
        step: Some(Step::Text {
            col: "status".into(),
            kind: TextKind::Contains,
            text: String::new(),
            edit: None,
        }),
        ..FilterWin::default()
    });
    a.set_mode(Mode::FilterText);
    check(&mut a, Mode::FilterText, "絞り込みの窓の値");
}

#[test]
fn test_vim_input_modes_tab_opens_next_cell_in_insert() {
    // [utest -> req~screen.vim-input-modes~1] ノーマルのモードから Tab で隣のセルへ移ると、隣の欄も挿入のモードで始まる。
    let (_t, mut a) = vim_app("vimin-tab");
    open_title(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.key_mode(), Mode::InputNormal);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.key_mode(), Mode::Edit, "挿入のモードから");
}

// ---- screen.vim-mode-shown ----

#[test]
fn test_vim_mode_shown_in_bottom_band_and_cursor() {
    // [utest -> req~screen.vim-mode-shown~1] `vim = true` で入力の欄を開く → 下の帯に「-- 挿入 --」。Esc → 「-- ノーマル --」。
    // カーソルの形も変わる(挿入は棒、ノーマルは四角。欄の外は端末の既定)。
    let (_t, mut a) = vim_app("vimshown");
    assert_eq!(a.cursor_shape(), CursorShape::Default);
    open_title(&mut a);
    assert!(footer(&a).contains("-- 挿入 --"), "{}", footer(&a));
    assert_eq!(a.cursor_shape(), CursorShape::Bar);
    press(&mut a, KeyCode::Esc);
    assert!(footer(&a).contains("-- ノーマル --"), "{}", footer(&a));
    assert_eq!(a.cursor_shape(), CursorShape::Block);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.cursor_shape(), CursorShape::Default);
    assert!(!footer(&a).contains("--"), "{}", footer(&a));
}

#[test]
fn test_vim_mode_not_shown_without_vim() {
    // [utest -> req~screen.vim-mode-shown~1] vim が無い設定では、入力の欄でもモードを出さず、カーソルの形も変えない。
    let (_t, mut a) = make("vimshown-off", NOTES);
    open_title(&mut a);
    assert!(!footer(&a).contains("-- 挿入 --"), "{}", footer(&a));
    assert_eq!(a.cursor_shape(), CursorShape::Default);
}

// ---- screen.vim-normal-keys ----

#[test]
fn test_vim_normal_keys_acceptance() {
    // [utest -> req~screen.vim-normal-keys~1] `会議の見積もり` の欄で Esc・`0`・`dw` → `の見積もり`(語の切れ目は vim の `w` と同じ)。
    // `$`・`x` → 最後の1字が消える。`cc` → 欄が空になり挿入のモード。`u` → 1つ前の形に戻る。
    let (_t, mut a) = vim_app("vimnk");
    open_title(&mut a);
    assert_eq!(field(&a), "会議の見積もり");
    press(&mut a, KeyCode::Esc);
    keys(&mut a, "0dw");
    assert_eq!(field(&a), "の見積もり");
    keys(&mut a, "$x");
    assert_eq!(field(&a), "の見積も");
    keys(&mut a, "cc");
    assert_eq!(field(&a), "");
    assert_eq!(a.key_mode(), Mode::Edit, "挿入のモード");
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'u');
    assert_eq!(field(&a), "の見積も", "1つ前の形");
    ch(&mut a, 'u');
    assert_eq!(field(&a), "の見積もり");
    ch(&mut a, 'u');
    assert_eq!(field(&a), "会議の見積もり");
}

/// `normal` を `text` の `at` 番目の書記素で当てた結果(文字, カーソルの書記素の位置, 挿入か)。
fn run(action: Action, text: &str, at: usize) -> (String, usize, bool) {
    use unicode_segmentation::UnicodeSegmentation;
    let b: Vec<usize> = text.grapheme_indices(true).map(|(i, _)| i).collect();
    let cursor = b.get(at).copied().unwrap_or(text.len());
    let Edited {
        text,
        cursor,
        insert,
    } = normal(action, text, cursor).expect("扱う動作");
    let g = text
        .grapheme_indices(true)
        .position(|(i, _)| i == cursor)
        .unwrap_or_else(|| text.graphemes(true).count());
    (text, g, insert)
}

#[test]
fn test_vim_normal_keys_motions() {
    // [utest -> req~screen.vim-normal-keys~1] 動き(`h`・`l`・`0`・`$`・`w`・`b`・`e`)は vim と同じ(語は英数字・記号・空白・
    // ひらがな・カタカナ・漢字の種類の切れ目。カーソルは最後の字より後ろに出ない)。
    let t = "foo bar.baz 会議の";
    assert_eq!(run(Action::CursorLeft, t, 4).1, 3);
    assert_eq!(run(Action::CursorLeft, t, 0).1, 0);
    assert_eq!(run(Action::CursorRight, t, 0).1, 1);
    assert_eq!(run(Action::CursorRight, t, 14).1, 14, "最後の字で止まる");
    assert_eq!(run(Action::CursorHome, t, 9).1, 0);
    assert_eq!(run(Action::CursorEnd, t, 0).1, 14);
    // w: foo → bar → . → baz → 会議 → の → 最後の字で止まる。
    assert_eq!(run(Action::WordNext, t, 0).1, 4);
    assert_eq!(run(Action::WordNext, t, 4).1, 7);
    assert_eq!(run(Action::WordNext, t, 7).1, 8);
    assert_eq!(run(Action::WordNext, t, 8).1, 12);
    assert_eq!(run(Action::WordNext, t, 12).1, 14);
    assert_eq!(run(Action::WordNext, t, 14).1, 14);
    // b: 語の頭へ、頭なら前の語の頭へ。
    assert_eq!(run(Action::WordPrev, t, 5).1, 4);
    assert_eq!(run(Action::WordPrev, t, 4).1, 0);
    assert_eq!(run(Action::WordPrev, t, 14).1, 12);
    assert_eq!(run(Action::WordPrev, t, 0).1, 0);
    // e: 語の終わりへ、終わりなら次の語の終わりへ。
    assert_eq!(run(Action::WordEnd, t, 0).1, 2);
    assert_eq!(run(Action::WordEnd, t, 2).1, 6);
    assert_eq!(run(Action::WordEnd, t, 8).1, 10);
    assert_eq!(run(Action::WordEnd, t, 10).1, 13);
    assert_eq!(run(Action::WordEnd, "会議の見積もり", 0).1, 1);
    assert_eq!(run(Action::WordNext, "会議の見積もり", 0).1, 2);
    assert_eq!(run(Action::WordNext, "会議の見積もり", 2).1, 3);
    assert_eq!(run(Action::WordNext, "会議の見積もり", 3).1, 5);
}

#[test]
fn test_vim_normal_keys_insert_entries() {
    // [utest -> req~screen.vim-normal-keys~1] 挿入への移り(`i` は前、`a` は後ろ、`I` は頭(空白の後ろ)、`A` は末尾)。
    let t = "  abc";
    assert_eq!(run(Action::Insert, t, 3), (t.into(), 3, true));
    assert_eq!(run(Action::Append, t, 3), (t.into(), 4, true));
    assert_eq!(
        run(Action::Append, t, 4),
        (t.into(), 5, true),
        "最後の字の後ろ"
    );
    assert_eq!(run(Action::InsertStart, t, 4), (t.into(), 2, true));
    assert_eq!(run(Action::AppendEnd, t, 0), (t.into(), 5, true));
    assert_eq!(run(Action::Insert, "", 0), (String::new(), 0, true));
}

#[test]
fn test_vim_normal_keys_deletes() {
    // [utest -> req~screen.vim-normal-keys~1] 消す(`x`・`X`・`dw`・`db`・`d$`・`D`・`dd`)は vim と同じ。
    let t = "foo bar baz";
    assert_eq!(
        run(Action::DeleteForward, t, 0),
        ("oo bar baz".into(), 0, false)
    );
    assert_eq!(
        run(Action::DeleteForward, t, 10),
        ("foo bar ba".into(), 9, false),
        "最後の字を消すとカーソルは左へ"
    );
    assert_eq!(run(Action::DeleteForward, "", 0), (String::new(), 0, false));
    assert_eq!(
        run(Action::DeleteBack, t, 4),
        ("foobar baz".into(), 3, false)
    );
    assert_eq!(run(Action::DeleteBack, t, 0), (t.into(), 0, false));
    assert_eq!(
        run(Action::DeleteWord, t, 4),
        ("foo baz".into(), 4, false),
        "後ろの空白も消す"
    );
    assert_eq!(
        run(Action::DeleteWord, t, 8),
        ("foo bar ".into(), 7, false),
        "最後の語は末尾まで"
    );
    assert_eq!(
        run(Action::DeleteWordBack, t, 6),
        ("foo r baz".into(), 4, false)
    );
    assert_eq!(
        run(Action::DeleteWordBack, t, 4),
        ("bar baz".into(), 0, false)
    );
    assert_eq!(run(Action::DeleteToEnd, t, 4), ("foo ".into(), 3, false));
    assert_eq!(run(Action::ClearInput, t, 5), (String::new(), 0, false));
}

#[test]
fn test_vim_normal_keys_changes() {
    // [utest -> req~screen.vim-normal-keys~1] 替える(`cw` は語の終わりまでで後ろの空白を残す、`c$`・`C` は末尾まで、`cc` は全部)は
    // 消してから挿入のモード。
    let t = "foo bar baz";
    assert_eq!(run(Action::ChangeWord, t, 4), ("foo  baz".into(), 4, true));
    assert_eq!(run(Action::ChangeWord, t, 5), ("foo b baz".into(), 5, true));
    assert_eq!(
        run(Action::ChangeWord, t, 3),
        ("foobar baz".into(), 3, true),
        "空白の上は次の語の頭まで"
    );
    assert_eq!(run(Action::ChangeToEnd, t, 4), ("foo ".into(), 4, true));
    assert_eq!(run(Action::ChangeLine, t, 4), (String::new(), 0, true));
}

#[test]
fn test_vim_normal_keys_every_key_bound() {
    // [utest -> req~screen.vim-normal-keys~1] ノーマルのモードのキーの表に、動き・挿入への移り・消す・替える・取り消しのキーが全部ある。
    let (_t, a) = vim_app("vimnk-keys");
    let want = [
        ("h", Action::CursorLeft),
        ("l", Action::CursorRight),
        ("0", Action::CursorHome),
        ("$", Action::CursorEnd),
        ("w", Action::WordNext),
        ("b", Action::WordPrev),
        ("e", Action::WordEnd),
        ("i", Action::Insert),
        ("a", Action::Append),
        ("I", Action::InsertStart),
        ("A", Action::AppendEnd),
        ("x", Action::DeleteForward),
        ("X", Action::DeleteBack),
        ("d w", Action::DeleteWord),
        ("d b", Action::DeleteWordBack),
        ("d $", Action::DeleteToEnd),
        ("D", Action::DeleteToEnd),
        ("d d", Action::ClearInput),
        ("c w", Action::ChangeWord),
        ("c $", Action::ChangeToEnd),
        ("C", Action::ChangeToEnd),
        ("c c", Action::ChangeLine),
        ("u", Action::Undo),
    ];
    for (k, act) in want {
        assert_eq!(
            keymap::lookup(&a.keys, Mode::InputNormal, k),
            Some(act),
            "{k}"
        );
    }
}

#[test]
fn test_vim_normal_keys_in_search_box_without_cursor() {
    // [utest -> req~screen.vim-normal-keys~1, req~screen.vim-input-modes~1] カーソルを持たなかった欄(探す)でも、ノーマルのモードで
    // 直せて、`i` のあとはカーソルの所に打てる。打つたびに探し直す。
    let (_t, mut a) = vim_app("vimnk-search");
    ch(&mut a, '/');
    keys(&mut a, "abc");
    press(&mut a, KeyCode::Esc);
    keys(&mut a, "0x");
    assert_eq!(field(&a), "bc");
    assert_eq!(a.search.as_deref(), Some("bc"), "探し直す");
    keys(&mut a, "iZ");
    assert_eq!(field(&a), "Zbc");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(field(&a), "bc");
    keys(&mut a, "Y");
    assert_eq!(field(&a), "Ybc");
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'u');
    assert_eq!(field(&a), "bc", "挿入の間の変更は1手で戻る");
}

#[test]
fn test_vim_normal_keys_other_letters_do_nothing() {
    // [utest -> req~screen.vim-normal-keys~1] ノーマルのモードでは、表に無い文字は入れない(`d` のあとの表に無いキーは前置きを捨てて引き直す)。
    let (_t, mut a) = vim_app("vimnk-other");
    open_title(&mut a);
    press(&mut a, KeyCode::Esc);
    keys(&mut a, "zq");
    assert_eq!(field(&a), "会議の見積もり");
    ch(&mut a, 'd');
    press(&mut a, KeyCode::Esc);
    assert_eq!(
        a.mode,
        Mode::Edit,
        "前置きのあとの Esc は前置きを取り消すだけ"
    );
    assert_eq!(field(&a), "会議の見積もり");
}

// ---- screen.vim-dd ----

/// ごみ箱の中のノートの名前。
fn trashed(a: &App) -> Vec<String> {
    let mut v: Vec<String> = a.src.trashed().iter().map(|t| t.rel.clone()).collect();
    v.sort();
    v
}

#[test]
fn test_vim_dd_deletes_after_confirm_and_u_restores() {
    // [utest -> req~screen.vim-dd~1] `vim = true` で a.md の行で `dd` → 名前の確かめ → `y` → a.md が `.trash/` へ動き、表から消える。
    // `u` → 元の場所に戻る。
    let (t, mut a) = vim_app("vimdd");
    select(&mut a, "a.md");
    keys(&mut a, "dd");
    assert_eq!(a.mode, Mode::Palette, "名前の確かめ: {:?}", a.message);
    assert!(screen(&a).contains("消すノート: a.md"), "{}", screen(&a));
    keys(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert!(!t.notes().join("a.md").exists());
    assert_eq!(trashed(&a), ["a.md"]);
    assert_eq!(labels(&a), ["b.md", "c.md"]);
    ch(&mut a, 'u');
    assert!(t.notes().join("a.md").exists(), "元の場所に戻る");
    assert!(trashed(&a).is_empty());
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md"]);
}

#[test]
fn test_vim_dd_range_d_deletes_both_and_one_u_restores_both() {
    // [utest -> req~screen.vim-dd~1] `v` で2行選んで `d` → 2つともごみ箱へ、`u` 1回で2つとも戻る。
    let (t, mut a) = vim_app("vimdd-range");
    select(&mut a, "a.md");
    keys(&mut a, "vj");
    ch(&mut a, 'd');
    assert_eq!(a.mode, Mode::Palette, "名前の確かめ: {:?}", a.message);
    let s = screen(&a);
    assert!(s.contains("a.md") && s.contains("b.md"), "{s}");
    keys(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert!(!t.notes().join("a.md").exists() && !t.notes().join("b.md").exists());
    assert_eq!(trashed(&a), ["a.md", "b.md"]);
    assert_eq!(labels(&a), ["c.md"]);
    ch(&mut a, 'u');
    assert!(t.notes().join("a.md").exists() && t.notes().join("b.md").exists());
    assert!(trashed(&a).is_empty(), "1回で2つとも戻る");
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md"]);
}

#[test]
fn test_vim_dd_marked_rows() {
    // [utest -> req~screen.vim-dd~1] 印を付けた行があれば、`dd` は印を付けた行のノートを消す(選んでいる行ではなく)。
    let (t, mut a) = vim_app("vimdd-marked");
    select(&mut a, "a.md");
    press(&mut a, KeyCode::Char(' '));
    select(&mut a, "c.md");
    press(&mut a, KeyCode::Char(' '));
    select(&mut a, "b.md");
    keys(&mut a, "dd");
    keys(&mut a, "y");
    press(&mut a, KeyCode::Enter);
    assert_eq!(labels(&a), ["b.md"]);
    assert!(t.notes().join("b.md").exists());
}

#[test]
fn test_vim_dd_readonly_refuses_with_reason() {
    // [utest -> req~screen.vim-dd~1] 読むだけの起動(cli.readonly)→ 何もせず理由を出す。
    let (t, mut a) = vim_app("vimdd-ro");
    a.readonly = true;
    select(&mut a, "a.md");
    keys(&mut a, "dd");
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.message.as_deref(), Some(super::startup::READONLY.text()));
    assert!(t.notes().join("a.md").exists());
}

#[test]
fn test_vim_dd_without_vim_does_nothing() {
    // [utest -> req~screen.vim-dd~1] `vim` が無い設定で `dd` → 何も変わらない。
    let (t, mut a) = make("vimdd-off", NOTES);
    select(&mut a, "a.md");
    keys(&mut a, "dd");
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.is_none(), "{:?}", a.message);
    assert!(t.notes().join("a.md").exists());
    assert_eq!(labels(&a), ["a.md", "b.md", "c.md"]);
}

// ---- screen.vim-paste ----

#[test]
fn test_vim_paste_cell_into_selected_cell() {
    // [utest -> req~screen.vim-paste~1] `vim = true` で a.md の status(done)で `y`、b.md の status で `p` → b.md の status が done のためる変更。
    let (_t, mut a) = vim_app("vimp-cell");
    select(&mut a, "a.md");
    col_named(&mut a, "status");
    ch(&mut a, 'y');
    select(&mut a, "b.md");
    ch(&mut a, 'p');
    assert_eq!(
        pending(&a, "b.md", "status"),
        Some(NewValue::Str("done".into()))
    );
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_vim_paste_row_into_same_named_columns() {
    // [utest -> req~screen.vim-paste~1] a.md の行で `Y`、c.md の行で `p` → c.md の、a.md と同じ名前の書ける列が a.md の値になり、
    // `file.name` と計算の列は変わらない。
    let (t, mut a) = vim_app("vimp-row");
    select(&mut a, "a.md");
    ch(&mut a, 'Y');
    select(&mut a, "c.md");
    ch(&mut a, 'p');
    assert_eq!(
        pending(&a, "c.md", "title"),
        Some(NewValue::Str("会議の見積もり".into()))
    );
    assert_eq!(
        pending(&a, "c.md", "status"),
        Some(NewValue::Str("done".into()))
    );
    assert_eq!(
        pending(&a, "c.md", "due"),
        Some(NewValue::Date("2026-11-01".into()))
    );
    for col in a.cols.clone() {
        if col.starts_with("file.") || col.starts_with("formula.") {
            assert_eq!(pending(&a, "c.md", &col), None, "{col} は変わらない");
        }
    }
    assert!(t.notes().join("c.md").exists(), "ノートの名前は変わらない");
    assert_eq!(a.changes.count(), 3);
    // 1手で戻る。
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_vim_paste_nothing_copied_gives_reason() {
    // [utest -> req~screen.vim-paste~1] 何も写していない → 何もせず理由を出す。
    let (_t, mut a) = vim_app("vimp-none");
    select(&mut a, "b.md");
    col_named(&mut a, "status");
    ch(&mut a, 'p');
    assert_eq!(a.changes.count(), 0);
    assert_eq!(
        a.message.as_deref(),
        Some(mdgrid::i18n::Msg::PasteNothing.text())
    );
}

// ---- screen.vim-repeat ----

#[test]
fn test_vim_repeat_last_value_into_same_column() {
    // [utest -> req~screen.vim-repeat~1] `vim = true` で a.md の status を done にしたあと `j` で b.md へ移り、選んでいる列が due でも `.`
    // → b.md の status が done のためる変更。
    let (_t, mut a) = make(
        "vimrep",
        &[
            ("a.md", "---\nstatus: todo\ndue: 2026-11-01\n---\n"),
            ("b.md", "---\nstatus: todo\ndue: 2026-11-02\n---\n"),
        ],
    );
    a.configure(&Config {
        vim: true,
        ..Config::default()
    });
    edit_cell(&mut a, 0, "status", "done");
    assert_eq!(
        pending(&a, "a.md", "status"),
        Some(NewValue::Str("done".into()))
    );
    ch(&mut a, 'j');
    assert_eq!(a.src.label(&a.cur_row().unwrap()), "b.md");
    col_named(&mut a, "due");
    ch(&mut a, '.');
    assert_eq!(
        pending(&a, "b.md", "status"),
        Some(NewValue::Str("done".into()))
    );
    assert_eq!(pending(&a, "b.md", "due"), None, "選んでいる列は変えない");
}

#[test]
fn test_vim_repeat_nothing_changed_yet() {
    // [utest -> req~screen.vim-repeat~1] まだ何も直していない → 何も変わらない。
    let (_t, mut a) = vim_app("vimrep-none");
    select(&mut a, "b.md");
    ch(&mut a, '.');
    assert_eq!(a.changes.count(), 0);
}

// ---- screen.quit-discard ----

#[test]
fn test_quit_discard_with_two_pending() {
    // [utest -> req~screen.quit-discard~1] 未保存 2 でパレットの `q!` → 確かめずに終わり、ノートのファイルは前のまま。
    let (t, mut a) = make("qdis", NOTES);
    edit_cell(&mut a, 0, "status", "x");
    edit_cell(&mut a, 1, "status", "y");
    assert_eq!(a.changes.count(), 2);
    ch(&mut a, ':');
    typing(&mut a, "q!");
    press(&mut a, KeyCode::Enter);
    assert!(a.quit, "終わる");
    assert_ne!(a.mode, Mode::Quit, "確かめない");
    for (n, text) in NOTES {
        assert_eq!(
            std::fs::read_to_string(t.notes().join(n)).unwrap(),
            *text,
            "{n} は前のまま"
        );
    }
}

#[test]
fn test_quit_discard_with_nothing_pending() {
    // [utest -> req~screen.quit-discard~1] 未保存 0 → そのまま終わる。vim モードでも同じ(挿入のモードで打って Enter)。
    let (_t, mut a) = make("qdis0", NOTES);
    ch(&mut a, ':');
    typing(&mut a, "q!");
    press(&mut a, KeyCode::Enter);
    assert!(a.quit);
    let (_t, mut a) = vim_app("qdis-vim");
    ch(&mut a, ':');
    typing(&mut a, "q!");
    press(&mut a, KeyCode::Enter);
    assert!(a.quit);
}

#[test]
fn test_quit_plain_q_still_confirms() {
    // [utest -> req~screen.quit-discard~1] パレットの `q` はこれまでどおり確かめる(ためた変更があれば終了の確認)。
    let (_t, mut a) = make("qconf", NOTES);
    edit_cell(&mut a, 0, "status", "x");
    ch(&mut a, ':');
    typing(&mut a, "q");
    press(&mut a, KeyCode::Enter);
    assert!(!a.quit);
    assert_eq!(a.mode, Mode::Quit);
}

// ---- screen.vim-no-count ----

#[test]
fn test_vim_no_count_digit_then_j_moves_one_row() {
    // [utest -> req~screen.vim-no-count~1] `vim = true` で `5` `j` → `j` で1行だけ下へ(数字は回数にしない。5 の条件の切り替えは
    // condition.digit-keys の側で、vim モードは数字のキーを足さない)。
    let (_t, mut a) = vim_app("vimcount");
    select(&mut a, "a.md");
    ch(&mut a, '5');
    ch(&mut a, 'j');
    assert_eq!(a.src.label(&a.cur_row().unwrap()), "b.md", "1行だけ下へ");
    assert_eq!(a.changes.count(), 0);
    for d in '1'..='9' {
        let k = d.to_string();
        assert!(
            keymap::VIM_BINDINGS.iter().all(|b| b.key != k),
            "vim モードは数字 {k} を足さない"
        );
    }
}

// ---- screen.vim-remap ----

#[test]
fn test_vim_remap_removing_dd_disables_it() {
    // [utest -> req~screen.vim-remap~1] `vim = true` で `[keys.table]` の `delete_note`(`"d d" = "none"`)を外す → `dd` が何もしない。
    let (t, mut a) = with_vim("vimremap", true, &[("table", "d d", "none")]);
    select(&mut a, "a.md");
    keys(&mut a, "dd");
    assert_eq!(a.mode, Mode::Table);
    assert!(t.notes().join("a.md").exists());
    // 範囲の `d` も効かない。
    keys(&mut a, "vjd");
    assert_eq!(a.mode, Mode::Table);
    assert!(t.notes().join("a.md").exists());
}

#[test]
fn test_vim_remap_other_key_for_delete_and_normal_keys() {
    // [utest -> req~screen.vim-remap~1] vim モードのキーもほかのキーと同じく割り当て直せる(表の `delete_note`・入力の欄のノーマルのモード)。
    let (_t, mut a) = with_vim(
        "vimremap2",
        true,
        &[
            ("table", "ctrl+x", "delete_note"),
            ("input_normal", "q", "delete_forward"),
        ],
    );
    select(&mut a, "a.md");
    ctrl(&mut a, 'x');
    assert_eq!(a.mode, Mode::Palette, "{:?}", a.message);
    press(&mut a, KeyCode::Esc);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    open_title(&mut a);
    press(&mut a, KeyCode::Esc);
    keys(&mut a, "0q");
    assert_eq!(field(&a), "議の見積もり");
}

/// ヘルプの本文(80 桁)。
fn help_text(a: &App) -> Vec<String> {
    super::help::help_lines(a, Mode::Table, 79)
        .into_iter()
        .map(|(t, _)| t)
        .collect()
}

/// ヘルプのキーの欄の1項(`dd             ノートを消す`)。
fn help_item(key: &str, label: &str) -> String {
    format!("{} {label}", fit(key, 14, Align::Left))
}

#[test]
fn test_vim_remap_help_shows_vim_keys_only_with_vim() {
    // [utest -> req~screen.vim-remap~1] ヘルプに `dd`・`p`・`.` の行(`vim` が無い設定では出ない)。
    let items = [
        help_item("dd", "ノートを消す"),
        help_item("p", mdgrid::i18n::Msg::KeyPaste.text()),
        help_item(".", mdgrid::i18n::Msg::KeyRepeat.text()),
    ];
    let (_t, a) = vim_app("vimhelp");
    let h = help_text(&a);
    for it in &items {
        assert!(
            h.iter().any(|l| l.contains(it.as_str())),
            "{it:?} が無い:\n{}",
            h.join("\n")
        );
    }
    let (_t, a) = make("vimhelp-off", NOTES);
    let h = help_text(&a);
    for it in &items {
        assert!(
            !h.iter().any(|l| l.contains(it.as_str())),
            "{it:?} が出ている"
        );
    }
}

#[test]
fn test_vim_remap_bindings_do_not_clash() {
    // [utest -> req~screen.vim-remap~1, req~screen.key-modes-no-dup~1] 既定の表に vim モードのキーを足しても、同じモードの同じキーに
    // 2つの動作は無く、前置きのキーがそれだけで動作を持たない。動作の名前は動作ごとに違い、設定の名前で引ける。
    let all: Vec<keymap::Binding> = keymap::defaults().cloned().collect();
    let mut seen = std::collections::HashSet::new();
    for b in &all {
        assert!(seen.insert((b.mode, b.key)), "重複: {:?} {}", b.mode, b.key);
        assert!(
            !keymap::is_prefix(&all, b.mode, b.key),
            "前置きと重なる: {}",
            b.key
        );
    }
    let actions: std::collections::HashSet<Action> = all.iter().map(|b| b.action).collect();
    let names: std::collections::HashSet<&str> = actions.iter().map(|a| a.name()).collect();
    assert_eq!(actions.len(), names.len());
    for b in keymap::VIM_BINDINGS {
        assert_eq!(Action::by_name(b.action.name()), Some(b.action));
    }
    assert_eq!(Mode::by_name("input_normal"), Some(Mode::InputNormal));
}
