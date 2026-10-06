//! チェックボックスの列の切り替えの受け入れ試験(CE-4・CE-9・CE-10・WB-17)。
//! 切り替えは Enter。真と偽を切り替え、空・キーの無いノートは真に(決定 2026-10-06-checkbox-two-state)。
//! 材料は test_screen の Tmp・make を使い、ためる変更(pending)と保存したファイルのバイトを見る。

use super::keymap::Mode;
use super::test_screen::{ch, col_named, ctrl, make, press, read, screen};
use super::*;
use mdgrid::source::{NewValue, RowId};
use ratatui::crossterm::event::KeyCode;

fn pending(a: &App, row: usize, col: &str) -> Option<NewValue> {
    let r: &RowId = &a.rows[row];
    a.changes.pending(r, col).cloned()
}

fn row_of(a: &App, name: &str) -> usize {
    a.rows.iter().position(|r| r.0.ends_with(name)).unwrap()
}

fn at(a: &App, name: &str) -> Option<NewValue> {
    pending(a, row_of(a, name), "done")
}

/// 今の行で Enter(切り替え)。入力は開かず表に留まる。
fn toggle(a: &mut App) {
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "切り替えは入力を開かない");
}

/// Ctrl+S → 差分 → Enter で確定。
fn save(a: &mut App) {
    ctrl(a, 's');
    press(a, KeyCode::Enter);
}

#[test]
fn test_ce_4_false_toggles_to_true() {
    // [CE-4] [CE-9] `done: false` のセルで Enter → true のためる変更。空にするのは BS(保存すると `done:`)。
    let (t, mut a) = make(
        "ce4false",
        &[
            ("a.md", "---\ndone: false\ntitle: a\n---\n"),
            ("b.md", "---\ndone: true\ntitle: b\n---\n"),
        ],
    );
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    assert_eq!(at(&a, "b.md"), None);
    assert_eq!(a.changes.count(), 1);
    press(&mut a, KeyCode::Backspace);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Null));
    save(&mut a);
    assert_eq!(read(&t, "a.md"), "---\ndone:\ntitle: a\n---\n");
    assert_eq!(read(&t, "b.md"), "---\ndone: true\ntitle: b\n---\n");
    assert_eq!(a.changes.count(), 0);
    // 保存したあとの空のセルは、続けて Enter で true になる。
    a.row = row_of(&a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
}

#[test]
fn test_ce_4_false_full_cycle_returns_to_original() {
    // [CE-4] [WB-17] `done: false` で Enter 2回 → true → false。元の false に戻ればためる変更から外れる。
    let notes = [
        ("a.md", "---\ndone: false\n---\n"),
        ("b.md", "---\ndone: true\n---\n"),
    ];
    let (t, mut a) = make("ce4falsecycle", &notes);
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), None);
    assert_eq!(a.changes.count(), 0);
    ctrl(&mut a, 's');
    for (n, text) in notes {
        assert_eq!(read(&t, n), text);
    }
}

#[test]
fn test_ce_4_keyless_note_toggles_true_false() {
    // [CE-4] [WB-17] キーの無いノートで Enter → true、もう一度 → false、もう一度 → true(空には戻らない)。
    let notes = [
        ("a.md", "---\ndone: true\ntitle: a\n---\n"),
        ("b.md", "---\ntitle: b\n---\n"),
    ];
    let (t, mut a) = make("ce4keyless", &notes);
    col_named(&mut a, "done");
    a.row = row_of(&a, "b.md");
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(true)));
    assert_eq!(a.changes.count(), 1);
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(true)));
    assert_eq!(a.changes.count(), 1);
    // 1回ずつの手: `u` で1つ前(false)に戻る。
    ch(&mut a, 'u');
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
    // `u` を重ねてためる変更が無くなれば、ファイルは変わらない。
    ch(&mut a, 'u');
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table);
    for (n, text) in notes {
        assert_eq!(read(&t, n), text);
    }
}

#[test]
fn test_ce_4_keyless_note_saves_false() {
    // [CE-4] キーの無いノートで Enter 2回 → false。保存すると `done: false` が足される。
    let (t, mut a) = make(
        "ce4keylesssave",
        &[
            ("a.md", "---\ndone: true\ntitle: a\n---\n"),
            ("b.md", "---\ntitle: b\n---\n"),
        ],
    );
    col_named(&mut a, "done");
    a.row = row_of(&a, "b.md");
    toggle(&mut a);
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
    save(&mut a);
    assert_eq!(read(&t, "b.md"), "---\ntitle: b\ndone: false\n---\n");
    assert_eq!(read(&t, "a.md"), "---\ndone: true\ntitle: a\n---\n");
}

#[test]
fn test_ce_4_true_toggles_to_false() {
    // [CE-4] `done: true` で Enter → false。さらに → true(元に戻って外れる)。
    let (t, mut a) = make(
        "ce4true",
        &[
            ("a.md", "---\ndone: true\n---\n"),
            ("b.md", "---\ndone: false\n---\n"),
        ],
    );
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(false)));
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), None);
    assert_eq!(a.changes.count(), 0);
    // true → false を保存 → `done: false`。
    toggle(&mut a);
    save(&mut a);
    assert_eq!(read(&t, "a.md"), "---\ndone: false\n---\n");
}

#[test]
fn test_ce_4_empty_value_toggles_to_true() {
    // [CE-4] [WB-17] `done:` の空のセルで Enter → true、→ false(空には戻らない)。
    let notes = [
        ("a.md", "---\ndone: true\n---\n"),
        ("c.md", "---\ndone:\ntitle: c\n---\n"),
    ];
    let (t, mut a) = make("ce4empty", &notes);
    col_named(&mut a, "done");
    a.row = row_of(&a, "c.md");
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(false)));
    // 空に戻すのは BS(元が `key:` なので変更なし)。
    press(&mut a, KeyCode::Backspace);
    assert_eq!(at(&a, "c.md"), None);
    assert_eq!(a.changes.count(), 0);
    ctrl(&mut a, 's');
    for (n, text) in notes {
        assert_eq!(read(&t, n), text);
    }
    // 1回で true を保存 → `done: true`。
    toggle(&mut a);
    save(&mut a);
    assert_eq!(read(&t, "c.md"), "---\ndone: true\ntitle: c\n---\n");
}

const BULK: [(&str, &str); 4] = [
    ("a.md", "---\ndone: false\n---\n"),
    ("b.md", "---\ndone: true\n---\n"),
    ("c.md", "---\nt: c\n---\n"),
    ("d.md", "---\ndone:\n---\n"),
];

#[test]
fn test_ce_4_ce_10_bulk_toggle_follows_current_row() {
    // [CE-4] [CE-10] [WB-17] 全部を選んで切り替え → 今の行(a: false)の次の状態が全部の行に入る。
    // Enter ごとに true と false を切り替える。
    let (t, mut a) = make("ce4bulk", &BULK);
    ctrl(&mut a, 'a');
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    // 1回目: false の次 → true(b は元の true のままで外れる)。
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    assert_eq!(at(&a, "b.md"), None);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(true)));
    assert_eq!(at(&a, "d.md"), Some(NewValue::Bool(true)));
    assert_eq!(a.changes.count(), 3);
    // 2回目: true の次 → false(a は元の false に戻って外れる)。
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), None);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(false)));
    assert_eq!(at(&a, "d.md"), Some(NewValue::Bool(false)));
    assert_eq!(a.changes.count(), 3);
    // 一括は1手: `u` で1回目の状態に戻る。
    ch(&mut a, 'u');
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    assert_eq!(at(&a, "b.md"), None);
    ctrl(&mut a, 'r');
    save(&mut a);
    assert_eq!(read(&t, "a.md"), BULK[0].1);
    assert_eq!(read(&t, "b.md"), "---\ndone: false\n---\n");
    assert_eq!(read(&t, "c.md"), "---\nt: c\ndone: false\n---\n");
    assert_eq!(read(&t, "d.md"), "---\ndone: false\n---\n");
}

#[test]
fn test_ce_4_ce_10_bulk_toggle_from_empty_current_row() {
    // [CE-4] [CE-10] [WB-17] 今の行がキーなし(空)なら、次の状態 true が全部の行に入る。
    // 今の行が true(b)なら false が、`done:`(d)なら true が全部の行に入る。
    for (cur, next) in [
        ("c.md", NewValue::Bool(true)),
        ("d.md", NewValue::Bool(true)),
        ("b.md", NewValue::Bool(false)),
    ] {
        let (t, mut a) = make("ce4bulkcur", &BULK);
        ctrl(&mut a, 'a');
        col_named(&mut a, "done");
        a.row = row_of(&a, cur);
        toggle(&mut a);
        for (n, orig) in [
            ("a.md", Some(false)),
            ("b.md", Some(true)),
            ("c.md", None),
            ("d.md", None),
        ] {
            // 元と同じ値になる行は外れる(WB-17)。
            let want = match (orig, &next) {
                (Some(o), NewValue::Bool(v)) if o == *v => None,
                _ => Some(next.clone()),
            };
            assert_eq!(at(&a, n), want, "今の行 {cur}・{n}");
        }
        save(&mut a);
        let want_c = match next {
            NewValue::Bool(true) => "---\nt: c\ndone: true\n---\n",
            _ => "---\nt: c\ndone: false\n---\n",
        };
        assert_eq!(read(&t, "c.md"), want_c, "今の行 {cur}");
    }
}

#[test]
fn test_ce_4_empty_string_toggles_to_true() {
    // [CE-4] [WB-17] `done: ""`(空の文字列)は空として扱う: Enter → true、→ false。1回で true を保存 → `done: true`。
    let notes = [
        ("a.md", "---\ndone: true\n---\n"),
        ("c.md", "---\ndone: \"\"\ntitle: c\n---\n"),
    ];
    let (t, mut a) = make("ce4emptystr", &notes);
    col_named(&mut a, "done");
    a.row = row_of(&a, "c.md");
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    assert_eq!(at(&a, "c.md"), Some(NewValue::Bool(false)));
    toggle(&mut a);
    save(&mut a);
    assert_eq!(read(&t, "c.md"), "---\ndone: true\ntitle: c\n---\n");
}

#[test]
fn test_ce_4_ce_10_bulk_empty_string_row_is_empty() {
    // [CE-4] [CE-10] 一括でも空の文字列の行は空として扱う: 今の行が `done: ""` なら true が全部に入る。
    let notes = [
        ("a.md", "---\ndone: false\n---\n"),
        ("b.md", "---\ndone: \"\"\n---\n"),
    ];
    let (_t, mut a) = make("ce4bulkemptystr", &notes);
    ctrl(&mut a, 'a');
    col_named(&mut a, "done");
    a.row = row_of(&a, "b.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(true)));
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(true)));
    toggle(&mut a);
    // true の次 → false: a は元の false に戻って外れ、b は false。
    assert_eq!(at(&a, "a.md"), None);
    assert_eq!(at(&a, "b.md"), Some(NewValue::Bool(false)));
}

#[test]
fn test_ce_4_ce_10_bulk_skips_mismatched_rows() {
    // [CE-4] [CE-10] 一括で型の合わない行(`done: yes`・`done: "true"`)は上書きせずに飛ばし、数と理由を出す。
    let notes = [
        ("a.md", "---\ndone: true\n---\n"),
        ("b.md", "---\ndone: yes\n---\n"),
        ("c.md", "---\ndone: \"true\"\n---\n"),
        ("d.md", "---\ndone: true\n---\n"),
    ];
    let (t, mut a) = make("ce4bulkmismatch", &notes);
    ctrl(&mut a, 'a');
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    toggle(&mut a);
    assert_eq!(at(&a, "a.md"), Some(NewValue::Bool(false)));
    assert_eq!(at(&a, "b.md"), None);
    assert_eq!(at(&a, "c.md"), None);
    assert_eq!(at(&a, "d.md"), Some(NewValue::Bool(false)));
    let m = a.message.clone().unwrap();
    assert!(m.contains("2行を飛ばした(型が合わない)"), "{m}");
    // 次の切り替えでも飛ばす。
    toggle(&mut a);
    assert_eq!(at(&a, "b.md"), None);
    assert_eq!(at(&a, "c.md"), None);
    toggle(&mut a);
    save(&mut a);
    assert_eq!(read(&t, "b.md"), notes[1].1);
    assert_eq!(read(&t, "c.md"), notes[2].1);
    assert_eq!(read(&t, "a.md"), "---\ndone: false\n---\n");
}

#[test]
fn test_ce_4_detail_view_cycles() {
    // [CE-4] [NV-6] 詳細の表示からの切り替えも true と false を切り替える(入力は開かず詳細の表示に留まる)。
    let (_t, mut a) = make(
        "ce4detail",
        &[
            ("a.md", "---\ndone: false\n---\n"),
            ("b.md", "---\ndone: true\n---\n"),
        ],
    );
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    for want in [Some(NewValue::Bool(true)), None] {
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Detail);
        assert_eq!(at(&a, "a.md"), want);
    }
    assert_eq!(a.changes.count(), 0);
}
