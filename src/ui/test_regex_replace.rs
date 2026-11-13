//! [CE-15] 今の列の値を正規表現で探して置き換える(パレットの「正規表現で置き換える」と操作の一覧から)。
//! 記録: specs/_changes/2026-11-13-regex-replace.md。
//!
//! 実装より先に、実装を見ずに書いた。実装の内側の名前には頼らず、キーの入力・パレットの入力・
//! 動作の名前 `regex_replace`・画面の文言(記録の「名前と文言」)だけで確かめる。
//! 試験の環境は日本語に固定されている(.cargo/config.toml)。英語は `scoped(Lang::En)` で確かめる。

use super::grid::Slot;
use super::keymap::Mode;
use super::test_action_menu::{has_section, menu_lines, menu_mode, shot};
use super::test_screen::{ch, col_named, ctrl, edit_cell, make, press, read, screen, typing, Tmp};
use super::*;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::source::{NewValue, RowId, Value};
use ratatui::crossterm::event::KeyCode;

/// 要件の例の3行(title は文字、estimate は数)。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: v1 draft\nestimate: 3\n---\n"),
    ("b.md", "---\ntitle: v1 final\nestimate: 5\n---\n"),
    ("c.md", "---\ntitle: notes\nestimate: 8\n---\n"),
];

const LABEL: &str = "正規表現で置き換える";
const ASK_FIND: &str = "探す正規表現:";
const ASK_WITH: &str = "置き換える文字:";

// ---- 道具 ----

fn s(text: &str) -> Option<Value> {
    Some(Value::Str(text.into()))
}

/// 名前でノートの行を引く(今のビューに出ている行から)。
fn row_of(a: &App, name: &str) -> RowId {
    a.rows
        .iter()
        .find(|r| a.src.label(r).contains(name))
        .unwrap_or_else(|| panic!("行 {name} が無い"))
        .clone()
}

/// ためた値を重ねた、今の値。
fn val(a: &App, name: &str, col: &str) -> Option<Value> {
    a.prop(&row_of(a, name), col)
}

fn pending(a: &App, name: &str, col: &str) -> Option<NewValue> {
    a.changes.pending(&row_of(a, name), col).cloned()
}

/// 名前の行に立つ。
fn go(a: &mut App, name: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], Slot::Row(r) if a.src.label(&a.rows[r]).contains(name)))
        .unwrap_or_else(|| panic!("行 {name} が無い"));
}

fn msg(a: &App) -> String {
    a.message.clone().unwrap_or_default()
}

/// パレットを開いて動作の名前を打つ(まだ実行しない)。
fn palette_query(a: &mut App, query: &str) {
    ch(a, ':');
    assert_eq!(a.mode, Mode::Palette, "`:` でパレットが開かない");
    typing(a, query);
}

/// パレットから `regex_replace` を選ぶ(候補に表示名が出ていることも見る)。
fn pick_command(a: &mut App) {
    palette_query(a, "regex_replace");
    let shown = screen(a);
    assert!(
        shown.contains(LABEL),
        "パレットに「{LABEL}」(regex_replace)が無い:\n{shown}"
    );
    press(a, KeyCode::Enter);
}

/// 探す正規表現を答える(前置きが出ていることも見る)。
fn answer_find(a: &mut App, pattern: &str) {
    let shown = screen(a);
    assert!(
        a.mode == Mode::Palette && shown.contains(ASK_FIND),
        "「{ASK_FIND}」と聞かれない({:?}):\n{shown}",
        a.message
    );
    typing(a, pattern);
    press(a, KeyCode::Enter);
}

/// 置き換える文字を答える(前置きが出ていることも見る)。
fn answer_with(a: &mut App, with: &str) {
    let shown = screen(a);
    assert!(
        a.mode == Mode::Palette && shown.contains(ASK_WITH),
        "「{ASK_WITH}」と聞かれない({:?}):\n{shown}",
        a.message
    );
    typing(a, with);
    press(a, KeyCode::Enter);
}

/// 今の列で、パレットから置き換えを最後まで行う。
fn replace(a: &mut App, pattern: &str, with: &str) {
    pick_command(a);
    answer_find(a, pattern);
    answer_with(a, with);
    assert_eq!(
        a.mode,
        Mode::Table,
        "置き換えのあと表に戻らない: {:?}",
        a.message
    );
}

/// 開いた入力が残っていれば閉じて、表のモードに戻す。
fn back_to_table(a: &mut App) {
    for _ in 0..3 {
        if a.mode == Mode::Table {
            break;
        }
        press(a, KeyCode::Esc);
    }
    assert_eq!(a.mode, Mode::Table, "Esc で表に戻らない");
}

fn assert_files_unchanged(t: &Tmp, notes: &[(&str, &str)]) {
    for (name, text) in notes {
        assert_eq!(&read(t, name), text, "{name} が書き換わった");
    }
}

// ---- 要件の例 ----

#[test]
fn test_ce_15_no_selection_replaces_all_rows_of_view() {
    // [CE-15] 行を選ばずに `v(\d)` → `version $1` → 2行が変わって「未保存 2」、notes は変わらない。
    // 変わったセルだけがたまり、結果は「2 件を置き換えた」で始まる。
    let (_t, mut a) = make("ce15all", NOTES);
    col_named(&mut a, "title");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(val(&a, "a.md", "title"), s("version 1 draft"));
    assert_eq!(val(&a, "b.md", "title"), s("version 1 final"));
    assert_eq!(val(&a, "c.md", "title"), s("notes"));
    assert_eq!(
        pending(&a, "a.md", "title"),
        Some(NewValue::Str("version 1 draft".into()))
    );
    assert_eq!(
        pending(&a, "c.md", "title"),
        None,
        "一致しないセルはためない"
    );
    assert_eq!(a.changes.count(), 2);
    let shown = screen(&a);
    assert!(shown.contains("未保存 2"), "{shown}");
    assert!(shown.contains("version 1 draft"), "{shown}");
    let m = msg(&a);
    assert!(m.trim_start().starts_with("2 件を置き換えた"), "{m:?}");
    // 今の列だけ。ほかの列はためない。
    for n in ["a.md", "b.md", "c.md"] {
        assert_eq!(pending(&a, n, "estimate"), None, "{n}");
    }
}

#[test]
fn test_ce_15_undo_once_restores_all() {
    // [CE-15][WB-10] ためる変更の1手。`u` を1回 → 両方戻って「未保存 0」。やり直しで両方また入る。
    let (_t, mut a) = make("ce15undo", NOTES);
    col_named(&mut a, "title");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(a.changes.count(), 2);
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0, "取り消し1回で全部戻らない");
    assert_eq!(val(&a, "a.md", "title"), s("v1 draft"));
    assert_eq!(val(&a, "b.md", "title"), s("v1 final"));
    assert!(screen(&a).contains("未保存 0"), "{}", screen(&a));
    ctrl(&mut a, 'r');
    assert_eq!(a.changes.count(), 2, "やり直し1回で全部入らない");
    assert_eq!(val(&a, "b.md", "title"), s("version 1 final"));
}

#[test]
fn test_ce_15_selected_rows_only() {
    // [CE-15][NV-5] `v1 draft` の行だけを選んでから同じ置き換え → その行だけが変わって「未保存 1」。
    // (Space は印を付けて下へ動くので、選んだ行へ戻ってから始める。)
    let (_t, mut a) = make("ce15sel", NOTES);
    col_named(&mut a, "title");
    go(&mut a, "a.md");
    press(&mut a, KeyCode::Char(' '));
    assert_eq!(a.marked.len(), 1, "材料: Space で印が付かない");
    go(&mut a, "a.md");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(val(&a, "a.md", "title"), s("version 1 draft"));
    assert_eq!(
        val(&a, "b.md", "title"),
        s("v1 final"),
        "選んでいない行が変わった"
    );
    assert_eq!(pending(&a, "b.md", "title"), None);
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("未保存 1"), "{}", screen(&a));
    assert!(msg(&a).trim_start().starts_with("1 件を置き換えた"));
}

#[test]
fn test_ce_15_bad_regex_changes_nothing() {
    // [CE-15] 探す文字に `(` → 「正規表現として読めない」と出て、何も変わらない。
    let (t, mut a) = make("ce15bad", NOTES);
    col_named(&mut a, "title");
    pick_command(&mut a);
    answer_find(&mut a, "(");
    let shown = screen(&a);
    assert!(
        msg(&a).contains("正規表現として読めない") || shown.contains("正規表現として読めない"),
        "理由が出ない({:?}):\n{shown}",
        a.message
    );
    assert!(
        !shown.contains(ASK_WITH),
        "読めない正規表現なのに置き換える文字を聞いた:\n{shown}"
    );
    assert_eq!(a.changes.count(), 0);
    back_to_table(&mut a);
    assert_eq!(a.changes.count(), 0);
    assert_eq!(val(&a, "a.md", "title"), s("v1 draft"));
    assert_eq!(val(&a, "b.md", "title"), s("v1 final"));
    assert_files_unchanged(&t, NOTES);
}

#[test]
fn test_ce_15_does_not_start_on_non_text_columns() {
    // [CE-15][CE-2] 数の列 estimate で始める → 始まらず理由。チェックボックス・日付・リストの列も同じ。
    let notes: &[(&str, &str)] = &[
        (
            "a.md",
            "---\ntitle: v1 draft\nestimate: 3\ndone: true\ndue: 2026-10-05\ntags: [v1, x]\n---\n",
        ),
        (
            "b.md",
            "---\ntitle: v1 final\nestimate: 5\ndone: false\ndue: 2026-10-06\ntags: [v1]\n---\n",
        ),
    ];
    let (t, mut a) = make("ce15type", notes);
    // 対照: テキストの列では始まる(パレットにコマンドがある)。
    col_named(&mut a, "title");
    pick_command(&mut a);
    assert!(screen(&a).contains(ASK_FIND), "{}", screen(&a));
    back_to_table(&mut a);
    for col in ["estimate", "done", "due", "tags"] {
        a.message = None;
        col_named(&mut a, col);
        pick_command(&mut a);
        let shown = screen(&a);
        assert!(
            !shown.contains(ASK_FIND),
            "{col}: テキストでない列で始まった:\n{shown}"
        );
        assert_eq!(a.mode, Mode::Table, "{col}");
        assert!(
            msg(&a).contains("テキストの列でだけ置き換えられる"),
            "{col}: 理由が出ない: {:?}",
            a.message
        );
        assert_eq!(a.changes.count(), 0, "{col}");
    }
    assert_files_unchanged(&t, notes);
}

#[test]
fn test_ce_15_does_not_start_on_formula_column() {
    // [CE-15] 計算の列(formula.*)では始めずに、理由を出す。
    let tmp = super::test_grid::vault("ce15formula");
    let mut a = super::test_grid::open(
        &tmp,
        "formulas:\n  num: '1 + 2'\nviews:\n  - type: table\n    name: 式\n    order: [title, formula.num]\n",
        None,
    );
    // 対照: 同じ表のテキストの列 title では始まる。
    col_named(&mut a, "title");
    pick_command(&mut a);
    assert!(screen(&a).contains(ASK_FIND), "{}", screen(&a));
    back_to_table(&mut a);
    a.message = None;
    col_named(&mut a, "formula.num");
    pick_command(&mut a);
    let shown = screen(&a);
    assert!(!shown.contains(ASK_FIND), "計算の列で始まった:\n{shown}");
    assert_eq!(a.mode, Mode::Table);
    assert!(a.message.is_some(), "計算の列で理由が出ない");
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_15_file_unchanged_until_save() {
    // [CE-15][WB-9] 保存するまでファイルは変わらない。Ctrl+S → 確認で確定 → 変わったノートだけ書かれる。
    let (t, mut a) = make("ce15save", NOTES);
    col_named(&mut a, "title");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(a.changes.count(), 2);
    assert_files_unchanged(&t, NOTES);
    ctrl(&mut a, 's');
    assert_files_unchanged(&t, NOTES);
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        read(&t, "a.md"),
        "---\ntitle: version 1 draft\nestimate: 3\n---\n"
    );
    assert_eq!(
        read(&t, "b.md"),
        "---\ntitle: version 1 final\nestimate: 5\n---\n"
    );
    assert_eq!(read(&t, "c.md"), NOTES[2].1);
    assert_eq!(a.changes.count(), 0);
}

// ---- 境界 ----

#[test]
fn test_ce_15_numbered_and_named_groups() {
    // [CE-15] 置き換える文字では `$1`・`${名前}` で正規表現の組を引ける。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: rel 1.2 beta\n---\n"),
        ("b.md", "---\ntitle: rel 10.4\n---\n"),
    ];
    let (_t, mut a) = make("ce15groups", notes);
    col_named(&mut a, "title");
    replace(
        &mut a,
        r"(?P<major>\d+)\.(?P<minor>\d+)",
        "${minor}/${major}",
    );
    assert_eq!(val(&a, "a.md", "title"), s("rel 2/1 beta"));
    assert_eq!(val(&a, "b.md", "title"), s("rel 4/10"));
    // 番号の組(ためた値にもう一度当てる)。
    replace(&mut a, r"(\d+)/(\d+)", "$2-$1");
    assert_eq!(val(&a, "a.md", "title"), s("rel 1-2 beta"));
    assert_eq!(val(&a, "b.md", "title"), s("rel 10-4"));
    assert_eq!(a.changes.count(), 2);
}

#[test]
fn test_ce_15_replaces_every_match_in_a_cell() {
    // [CE-15] 1つのセルの中の一致は全部置き換える。変えたセルの数は1。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: red-green-blue\n---\n"),
        ("b.md", "---\ntitle: plain\n---\n"),
    ];
    let (_t, mut a) = make("ce15every", notes);
    col_named(&mut a, "title");
    replace(&mut a, "-", "+");
    assert_eq!(val(&a, "a.md", "title"), s("red+green+blue"));
    assert_eq!(a.changes.count(), 1);
    assert!(msg(&a).trim_start().starts_with("1 件を置き換えた"));
}

#[test]
fn test_ce_15_no_match_stages_nothing() {
    // [CE-15] 一致するセルが無い → 何もためず「一致するセルは無い」。
    let (t, mut a) = make("ce15nomatch", NOTES);
    col_named(&mut a, "title");
    replace(&mut a, "zzz+", "y");
    assert_eq!(a.changes.count(), 0);
    assert!(msg(&a).contains("一致するセルは無い"), "{:?}", a.message);
    assert!(screen(&a).contains("未保存 0"), "{}", screen(&a));
    assert_files_unchanged(&t, NOTES);
}

#[test]
fn test_ce_15_same_value_cells_are_not_staged() {
    // [CE-15] 一致しても置き換えた値が元と同じセルはためない(変わったセルだけがたまり、数が合う)。
    // `v1|notes` → `v1`: `v1 draft`・`v1 final` は同じ値のまま、`notes` だけが `v1` になる。
    let (_t, mut a) = make("ce15same", NOTES);
    col_named(&mut a, "title");
    replace(&mut a, "v1|notes", "v1");
    assert_eq!(pending(&a, "a.md", "title"), None);
    assert_eq!(pending(&a, "b.md", "title"), None);
    assert_eq!(
        pending(&a, "c.md", "title"),
        Some(NewValue::Str("v1".into()))
    );
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("未保存 1"), "{}", screen(&a));
    assert!(msg(&a).trim_start().starts_with("1 件を置き換えた"));
}

#[test]
fn test_ce_15_non_string_cells_are_untouched() {
    // [CE-15] テキストの列(最初の値が文字)の中の、数・真偽・リスト・空(`key:`)・キーの無いセルは変えない。
    // 文字のセルだけが変わる(型と合わない値を黙って書かない)。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: v1 draft\n---\n"),
        ("b.md", "---\ntitle: 12\n---\n"),
        ("c.md", "---\ntitle: true\n---\n"),
        ("d.md", "---\ntitle: [v1, v2]\n---\n"),
        ("e.md", "---\ntitle:\nother: x\n---\n"),
        ("f.md", "---\nother: y\n---\n"),
        ("g.md", "---\ntitle: v2 final\n---\n"),
    ];
    let (t, mut a) = make("ce15kinds", notes);
    col_named(&mut a, "title");
    let before: Vec<Option<Value>> = ["b.md", "c.md", "d.md", "e.md", "f.md"]
        .iter()
        .map(|n| val(&a, n, "title"))
        .collect();
    assert_eq!(before[0], Some(Value::Int(12)), "材料: 数のセル");
    assert_eq!(before[1], Some(Value::Bool(true)), "材料: 真偽のセル");
    // どの値の字にも一致する正規表現。
    replace(&mut a, r"\w+", "X");
    assert_eq!(val(&a, "a.md", "title"), s("X X"));
    assert_eq!(val(&a, "g.md", "title"), s("X X"));
    for (i, n) in ["b.md", "c.md", "d.md", "e.md", "f.md"].iter().enumerate() {
        assert_eq!(pending(&a, n, "title"), None, "{n} をためた");
        assert_eq!(val(&a, n, "title"), before[i], "{n} が変わった");
    }
    assert_eq!(a.changes.count(), 2);
    assert!(screen(&a).contains("未保存 2"), "{}", screen(&a));
    assert!(msg(&a).trim_start().starts_with("2 件を置き換えた"));
    assert_files_unchanged(&t, notes);
}

#[test]
fn test_ce_15_read_only_note_is_skipped_with_reason() {
    // [CE-15][WB-5] 読むだけのノート(フロントマターの閉じの区切りが無い)のセルは変えず、飛ばした理由を知らせる
    // (CE-10 と同じ出し方)。
    let notes: &[(&str, &str)] = &[
        ("a.md", "---\ntitle: v1 draft\n---\n"),
        ("b.md", "---\ntitle: v1 final\n---\n"),
        ("c.md", "---\ntitle: v1 locked\n本文\n"),
    ];
    let (t, mut a) = make("ce15ro", notes);
    col_named(&mut a, "title");
    let reason = a
        .src
        .get(&row_of(&a, "c.md"), "title")
        .lock
        .expect("材料: 閉じの無いフロントマターのセルは読むだけ");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(val(&a, "a.md", "title"), s("version 1 draft"));
    assert_eq!(val(&a, "b.md", "title"), s("version 1 final"));
    assert_eq!(pending(&a, "c.md", "title"), None, "読むだけのセルをためた");
    assert_eq!(a.changes.count(), 2);
    let m = msg(&a);
    assert!(m.trim_start().starts_with("2 件を置き換えた"), "{m:?}");
    assert!(m.contains(&format!("1行を飛ばした({reason})")), "{m:?}");
    assert_files_unchanged(&t, notes);
}

#[test]
fn test_ce_15_applies_to_pending_value() {
    // [CE-15] ためた値のあるセルは、その値に当てる。取り消し1回で、置き換えの前のためた値に戻る。
    let (t, mut a) = make("ce15pending", NOTES);
    go(&mut a, "c.md");
    let row = a.row;
    edit_cell(&mut a, row, "title", "v2 memo");
    assert_eq!(a.changes.count(), 1, "材料: セルを直せない");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(val(&a, "c.md", "title"), s("version 2 memo"));
    assert_eq!(val(&a, "a.md", "title"), s("version 1 draft"));
    assert_eq!(a.changes.count(), 3);
    assert!(msg(&a).trim_start().starts_with("3 件を置き換えた"));
    ch(&mut a, 'u');
    assert_eq!(val(&a, "c.md", "title"), s("v2 memo"));
    assert_eq!(val(&a, "a.md", "title"), s("v1 draft"));
    assert_eq!(a.changes.count(), 1);
    assert_files_unchanged(&t, NOTES);
}

#[test]
fn test_ce_15_filtered_out_rows_are_not_targets() {
    // [CE-15] 行を選んでいなければ「今のビューに出ている」行が対象。絞り込みで隠れた行は変えない。
    let (_t, mut a) = make("ce15filter", NOTES);
    let hidden = row_of(&a, "b.md");
    a.filter = Some("draft".into());
    a.refresh();
    assert_eq!(a.rows.len(), 1, "材料: draft で1行に絞れない");
    col_named(&mut a, "title");
    replace(&mut a, r"v(\d)", "version $1");
    assert_eq!(a.changes.count(), 1);
    assert_eq!(a.changes.pending(&hidden, "title"), None);
    a.filter = None;
    a.refresh();
    assert_eq!(a.rows.len(), 3);
    assert_eq!(val(&a, "a.md", "title"), s("version 1 draft"));
    assert_eq!(
        val(&a, "b.md", "title"),
        s("v1 final"),
        "隠れた行が変わった"
    );
}

// ---- 入口 ----

#[test]
fn test_ce_15_readonly_start_hides_command() {
    // [CE-15][WB-15] 読むだけの起動(--readonly)では、パレットにコマンドが出ない(ためる変更を作るので)。
    let (_t, mut a) = make("ce15readonly", NOTES);
    col_named(&mut a, "title");
    // 対照: 読むだけでなければ、動作の名前でも表示名でも出る。
    for q in ["regex_replace", LABEL] {
        palette_query(&mut a, q);
        assert!(
            screen(&a).contains(LABEL),
            "{q}: パレットに無い:\n{}",
            screen(&a)
        );
        back_to_table(&mut a);
    }
    a.readonly = true;
    for q in ["regex_replace", LABEL, "正規表現"] {
        palette_query(&mut a, q);
        let shown = screen(&a);
        // 打った字(検索の欄)は除いて、候補の行に表示名が無いこと。
        let hits = shown
            .lines()
            .filter(|l| l.contains(LABEL) && !l.trim_start().starts_with(':'))
            .count();
        assert_eq!(hits, 0, "{q}: 読むだけなのにパレットに出た:\n{shown}");
        press(&mut a, KeyCode::Enter);
        assert!(
            !screen(&a).contains(ASK_FIND),
            "{q}: 読むだけなのに始まった"
        );
        back_to_table(&mut a);
    }
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_15_action_menu_lists_it_in_column_section() {
    // [CE-15][SR-24] テキストの列のセルで操作の一覧(`x`)→ 列の節に「正規表現で置き換える」がある。
    // 選ぶと(Enter)探す正規表現を聞く。
    let (_t, mut a) = make("ce15menu", NOTES);
    go(&mut a, "a.md");
    col_named(&mut a, "title");
    let before = shot(&mut a, 100, 50);
    ch(&mut a, 'x');
    let after = shot(&mut a, 100, 50);
    assert_eq!(a.mode, menu_mode(), "x で一覧が開かない");
    let lines = menu_lines(&before, &after);
    assert!(has_section(&lines, "列"), "節「列」が無い:\n{after}");
    let bare = |l: &str| {
        l.trim_matches(|c: char| c.is_whitespace() || "│┃|─━-+┌┐└┘╭╮╰╯├┤".contains(c))
            .to_string()
    };
    let at = lines
        .iter()
        .position(|l| l.contains(LABEL))
        .unwrap_or_else(|| panic!("一覧に「{LABEL}」が無い:\n{after}"));
    // その行より上でいちばん近い節の見出しが「列」。
    let head = lines[..at]
        .iter()
        .rev()
        .map(|l| bare(l))
        .find(|l| ["セル", "列", "行", "ビューとファイル"].contains(&l.as_str()));
    assert_eq!(head.as_deref(), Some("列"), "列の節に無い:\n{after}");
    // 一覧から選ぶ: 項目の順の数だけ ↓ で下りて Enter → 探す正規表現を聞く。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table, "Esc で一覧が閉じない");
    let n = super::menu::items(&a)
        .iter()
        .position(|i| i.label == LABEL)
        .unwrap_or_else(|| panic!("操作の一覧の項目に「{LABEL}」が無い"));
    ch(&mut a, 'x');
    for _ in 0..n {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    let shown = screen(&a);
    assert!(
        a.mode == Mode::Palette && shown.contains(ASK_FIND),
        "一覧から選んでも始まらない:\n{shown}"
    );
    back_to_table(&mut a);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_15_english_wording() {
    // [CE-15][SR-23] 英語の表示名・前置き・結果の文。
    let _g = scoped(Lang::En);
    let (_t, mut a) = make("ce15en", NOTES);
    col_named(&mut a, "title");
    palette_query(&mut a, "regex_replace");
    let shown = screen(&a);
    assert!(
        shown.contains("Replace with a regular expression"),
        "{shown}"
    );
    press(&mut a, KeyCode::Enter);
    let shown = screen(&a);
    assert!(shown.contains("Find (regular expression):"), "{shown}");
    typing(&mut a, r"v(\d)");
    press(&mut a, KeyCode::Enter);
    let shown = screen(&a);
    assert!(shown.contains("Replace with:"), "{shown}");
    typing(&mut a, "version $1");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 2);
    let m = msg(&a);
    assert!(m.trim_start().starts_with("replaced 2 cell"), "{m:?}");
    // 読めない正規表現・テキストでない列の理由も英語。
    pick_en(&mut a);
    typing(&mut a, "(");
    press(&mut a, KeyCode::Enter);
    let shown = screen(&a);
    assert!(
        msg(&a).contains("not a valid regular expression")
            || shown.contains("not a valid regular expression"),
        "{:?}\n{shown}",
        a.message
    );
    back_to_table(&mut a);
    col_named(&mut a, "estimate");
    pick_en(&mut a);
    assert!(
        msg(&a).contains("only text columns can be replaced"),
        "{:?}",
        a.message
    );
    assert_eq!(a.mode, Mode::Table);
}

/// 英語の画面で、パレットから `regex_replace` を選ぶ。
fn pick_en(a: &mut App) {
    palette_query(a, "regex_replace");
    assert!(
        screen(a).contains("Replace with a regular expression"),
        "{}",
        screen(a)
    );
    press(a, KeyCode::Enter);
}
