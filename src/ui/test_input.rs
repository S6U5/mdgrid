//! 型ごとの入り方と一括の設定の試験(タスク 11 の残り)。材料は test_screen の Tmp を使う。

use super::input::Entry;
use super::keymap::Mode;
use super::test_grid::open;
use super::test_screen::{
    assert_fits, ch, col_named, ctrl, golden, make, press, read, screen, typing, Tmp,
};
use super::*;
use mdgrid::source::{NewValue, RowId};
use mdgrid::types;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn pending(a: &App, row: usize, col: &str) -> Option<NewValue> {
    let r: &RowId = &a.rows[row];
    a.changes.pending(r, col).cloned()
}

/// 入力の文字を全部消す(リストなら自由入力に切り替わる)。
fn wipe(a: &mut App) {
    press(a, KeyCode::End);
    let n = a.input.as_ref().unwrap().text.chars().count();
    for _ in 0..n {
        press(a, KeyCode::Backspace);
    }
}

fn list_texts(a: &App) -> Vec<(String, bool)> {
    a.active_list()
        .unwrap()
        .items
        .iter()
        .map(|i| (i.text.clone(), i.current))
        .collect()
}

/// types.json(due は date)のある保管庫。
fn typed(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    std::fs::write(
        tmp.notes().join(".obsidian/types.json"),
        r#"{"types": {"due": "date"}}"#,
    )
    .unwrap();
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
    a.today = types::parse_date("2026-10-01").unwrap();
    (tmp, a)
}

const STATUS_BASE: &str = r#"views:
  - type: table
    name: 未完了
    filters: 'status != "done"'
    order: [title, status]
"#;

#[test]
fn test_ce_3_list_of_candidates() {
    // [CE-3] status が todo・doing・done だけ → 3つと「なし」のリスト、今の値に印。
    // ビューの絞り込みで見えていないノート(done)の値も候補に入る。
    let t = Tmp::new("ce3");
    t.write("a.md", "---\ntitle: 会議\nstatus: todo\n---\n");
    t.write("b.md", "---\ntitle: 買い物\nstatus: doing\n---\n");
    t.write("c.md", "---\ntitle: 掃除\nstatus: done\n---\n");
    let mut a = open(&t, STATUS_BASE, None);
    assert_eq!(a.rows.len(), 2);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(
        list_texts(&a),
        vec![
            ("todo".into(), true),
            ("doing".into(), false),
            ("done".into(), false),
            ("なし".into(), false),
        ]
    );
    let s = screen(&a);
    golden("ce_3", &s);
    assert!(s.contains("|>*todo") && s.contains("|  done"), "{s}");
    assert_fits(&mut a);
    // ↓↓ で done を選んで Enter → ためる変更。
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "a.md"), "---\ntitle: 会議\nstatus: todo\n---\n");
}

#[test]
fn test_ce_3_none_and_free_input() {
    // [CE-3] 「なし」を選ぶと Null。打つと自由入力に切り替わり、↑↓ でリストに戻る。
    let (_t, mut a) = make(
        "ce3free",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: doing\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    for _ in 0..5 {
        press(&mut a, KeyCode::Down);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "status"), Some(NewValue::Null));
    // 自由入力: 文字を打つとリストが消え、打った文字で確定する。
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().text, "doing");
    typing(&mut a, "!");
    assert!(a.active_list().is_none());
    assert!(!screen(&a).contains("|>"));
    // ↑ でリストに戻り、Ctrl+R で編集前に戻す [CE-11]。
    press(&mut a, KeyCode::Up);
    assert!(a.active_list().is_some());
    typing(&mut a, "?");
    ctrl(&mut a, 'r');
    assert_eq!(a.input.as_ref().unwrap().text, "doing");
    assert!(a.active_list().is_some());
    typing(&mut a, "-x");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a, 1, "status"),
        Some(NewValue::Str("doing-x".into()))
    );
}

#[test]
fn test_ce_3_too_many_candidates_is_free_input() {
    // [CE-3] 候補が上限(既定 20)を超える 21 個 → リストを出さず自由入力。
    let notes: Vec<(String, String)> = (0..21)
        .map(|i| (format!("n{i:02}.md"), format!("---\nstatus: s{i}\n---\n")))
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, mut a) = make("ce3many", &refs);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.input.as_ref().unwrap().list.is_none());
    press(&mut a, KeyCode::Esc);
    // 上限は設定で変えられる(CLI-3 の candidates)。
    a.candidates = 30;
    press(&mut a, KeyCode::Enter);
    assert_eq!(list_texts(&a).len(), 22);
    assert_fits(&mut a);
}

#[test]
fn test_ce_4_enter_toggles_checkbox() {
    // [CE-4] [SR-16] `done: false` のセルで Enter → true のためる変更(入力は開かない)。true と false を切り替える。
    let (t, mut a) = make(
        "ce4",
        &[
            ("a.md", "---\ndone: false\n---\n"),
            ("b.md", "---\ndone: true\n---\n"),
        ],
    );
    col_named(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 0, "done"), Some(NewValue::Bool(true)));
    // もう一度で元の false に戻る。元の値に戻ったので、ためる変更から外れる([WB-17])。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(pending(&a, 0, "done"), None);
    assert_eq!(a.changes.count(), 0);
    // 空にするのは BS。保存すると `done:`(キーは残す。[CE-9])。
    press(&mut a, KeyCode::Backspace);
    assert_eq!(pending(&a, 0, "done"), Some(NewValue::Null));
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), "---\ndone:\n---\n");
    assert_eq!(read(&t, "b.md"), "---\ndone: true\n---\n");
}

#[test]
fn test_wb_17_ce_4_empty_checkbox_toggles_back() {
    // [WB-17] [CE-4] 元が空(キーが無い・`done:`)のチェックボックスで Enter 2回 → true → false。`u` で戻せば未保存 0、
    // 保存してもファイルは変わらない。
    let notes = [
        ("a.md", "---\ndone: true\n---\n"),
        ("b.md", "---\ntitle: b\n---\n"),
        ("c.md", "---\ndone:\n---\n"),
    ];
    let (t, mut a) = make("wb17ce4", &notes);
    col_named(&mut a, "done");
    for name in ["b.md", "c.md"] {
        a.row = a.rows.iter().position(|r| r.0.ends_with(name)).unwrap();
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Table);
        assert_eq!(pending(&a, a.row, "done"), Some(NewValue::Bool(true)));
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Table);
        assert_eq!(
            pending(&a, a.row, "done"),
            Some(NewValue::Bool(false)),
            "{name}"
        );
        ch(&mut a, 'u');
        ch(&mut a, 'u');
        assert_eq!(pending(&a, a.row, "done"), None, "{name}");
    }
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));
    ctrl(&mut a, 's');
    assert_eq!(a.mode, Mode::Table);
    for (n, text) in notes {
        assert_eq!(read(&t, n), text);
    }
    // 元が true のセルは true → false → true と切り替わり、元に戻れば外れる。
    a.row = a.rows.iter().position(|r| r.0.ends_with("a.md")).unwrap();
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, a.row, "done"), Some(NewValue::Bool(false)));
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, a.row, "done"), None);
}

fn row_of(a: &App, name: &str) -> usize {
    a.rows.iter().position(|r| r.0.ends_with(name)).unwrap()
}

#[test]
fn test_wb_17_ce_4_ce_10_bulk_toggle_per_row() {
    // [WB-17] [CE-4] [CE-10] 一括の切り替えは、今の行の次の状態(true の次は false、ほかは true)を選んだ行の全部に入れる。
    // 元の値と同じになる行は外れる(行ごとに決める)。
    // 例1: a `done: false`・b キー無し を全部選んで Enter 2回。
    let notes = [
        ("a.md", "---\ndone: false\n---\n"),
        ("b.md", "---\nt: b\n---\n"),
    ];
    // (今の行, 2回それぞれの後の (a, b) のためる変更, 保存した a・b のバイト)
    let cases = [
        (
            "a.md",
            [
                (Some(NewValue::Bool(true)), Some(NewValue::Bool(true))),
                (None, Some(NewValue::Bool(false))),
            ],
            ["---\ndone: false\n---\n", "---\nt: b\ndone: false\n---\n"],
        ),
        (
            "b.md",
            [
                (Some(NewValue::Bool(true)), Some(NewValue::Bool(true))),
                (None, Some(NewValue::Bool(false))),
            ],
            ["---\ndone: false\n---\n", "---\nt: b\ndone: false\n---\n"],
        ),
    ];
    for (at, steps, saved) in cases {
        let (t, mut a) = make("wb17bulk1", &notes);
        ctrl(&mut a, 'a');
        col_named(&mut a, "done");
        a.row = row_of(&a, at);
        for (i, (wa, wb)) in steps.iter().enumerate() {
            press(&mut a, KeyCode::Enter);
            assert_eq!(a.mode, Mode::Table);
            assert_eq!(
                &pending(&a, row_of(&a, "a.md"), "done"),
                wa,
                "カーソル {at} {i}"
            );
            assert_eq!(
                &pending(&a, row_of(&a, "b.md"), "done"),
                wb,
                "カーソル {at} {i}"
            );
        }
        assert_eq!(a.changes.count(), 1, "カーソル {at}");
        // 1手なので `u` で1回目の後に、`U` で2回目の後に戻る。
        ch(&mut a, 'u');
        let (wa, wb) = &steps[0];
        assert_eq!(
            &pending(&a, row_of(&a, "a.md"), "done"),
            wa,
            "カーソル {at}"
        );
        assert_eq!(
            &pending(&a, row_of(&a, "b.md"), "done"),
            wb,
            "カーソル {at}"
        );
        ch(&mut a, 'U');
        assert_eq!(a.changes.count(), 1, "カーソル {at}");
        ctrl(&mut a, 's');
        press(&mut a, KeyCode::Enter);
        assert_eq!(read(&t, "a.md"), saved[0], "カーソル {at}");
        assert_eq!(read(&t, "b.md"), saved[1], "カーソル {at}");
    }
    // 例2: a `done: true` に false、b キー無し に true をためて、全部選んで Enter。
    // 今の行 b(表示 true)なら次の false が両方に、今の行 a(表示 false)なら次の true が両方に入る。
    let notes = [
        ("a.md", "---\ndone: true\n---\n"),
        ("b.md", "---\nt: b\n---\n"),
    ];
    for at in ["a.md", "b.md"] {
        let (t, mut a) = make("wb17bulk2", &notes);
        col_named(&mut a, "done");
        a.row = row_of(&a, "a.md");
        press(&mut a, KeyCode::Enter);
        a.row = row_of(&a, "b.md");
        press(&mut a, KeyCode::Enter);
        let (ra, rb) = (row_of(&a, "a.md"), row_of(&a, "b.md"));
        assert_eq!(pending(&a, ra, "done"), Some(NewValue::Bool(false)));
        assert_eq!(pending(&a, rb, "done"), Some(NewValue::Bool(true)));
        ctrl(&mut a, 'a');
        a.row = row_of(&a, at);
        press(&mut a, KeyCode::Enter);
        let (ra, rb) = (row_of(&a, "a.md"), row_of(&a, "b.md"));
        let (want_a, want_b) = if at == "b.md" {
            // 今の行 b は true の表示 → 次は false: a は false のまま、b は false。
            assert_eq!(pending(&a, ra, "done"), Some(NewValue::Bool(false)));
            assert_eq!(pending(&a, rb, "done"), Some(NewValue::Bool(false)));
            ("---\ndone: false\n---\n", "---\nt: b\ndone: false\n---\n")
        } else {
            // 今の行 a は false の表示 → 次は true: a は元の true に戻って外れ、b は true のまま。
            assert_eq!(pending(&a, ra, "done"), None);
            assert_eq!(pending(&a, rb, "done"), Some(NewValue::Bool(true)));
            ("---\ndone: true\n---\n", "---\nt: b\ndone: true\n---\n")
        };
        ctrl(&mut a, 's');
        press(&mut a, KeyCode::Enter);
        assert_eq!(read(&t, "a.md"), want_a, "カーソル {at}");
        assert_eq!(read(&t, "b.md"), want_b, "カーソル {at}");
    }
}

#[test]
fn test_ce_5_date_input() {
    // [CE-5] 実在しない日付は閉じずに不正。`+3`・`-2` は今日から。空で確定 → 保存で `due:` の行が残る。
    let (t, mut a) = typed(
        "ce5",
        &[
            ("a.md", "---\ndue: 2026-10-05\ntitle: a\n---\n"),
            ("b.md", "---\ndue: 2026-09-01\ntitle: b\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().entry, Entry::Date);
    assert!(screen(&a).contains("YYYY-MM-DD"));
    wipe(&mut a);
    typing(&mut a, "2026-02-30");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let s = screen(&a);
    assert!(
        // 理由は types::parse_date_input の文(CE-22 で寄せた)。
        s.lines().nth(22).unwrap().contains("実在しない日付"),
        "{s}"
    );
    assert_eq!(a.changes.count(), 0);
    wipe(&mut a);
    typing(&mut a, "+3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(
        pending(&a, 0, "due"),
        Some(NewValue::Date("2026-10-04".into()))
    );
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "-2");
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        pending(&a, 0, "due"),
        Some(NewValue::Date("2026-09-29".into()))
    );
    // 空で確定 → Null(CE-9)。
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "due"), Some(NewValue::Null));
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "b.md"), "---\ndue:\ntitle: b\n---\n");
}

#[test]
fn test_ce_7_number_input() {
    // [CE-7] 数の列は数として読める値だけ。`12a` → 閉じずに不正。
    let (_t, mut a) = make(
        "ce7",
        &[("a.md", "---\nn: 5\n---\n"), ("b.md", "---\nn: 7\n---\n")],
    );
    col_named(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().entry, Entry::Number);
    wipe(&mut a);
    typing(&mut a, "12a");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.message.as_deref().unwrap().contains("数として読めない"));
    wipe(&mut a);
    typing(&mut a, "12");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 0, "n"), Some(NewValue::Int(12)));
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "1.5");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "n"), Some(NewValue::Float(1.5)));
}

const THREE: &[(&str, &str)] = &[
    ("a.md", "---\nstatus: todo\n---\n"),
    ("b.md", "---\nstatus: doing\n---\n"),
    ("c.md", "---\nstatus: todo\n---\n"),
];

/// 選んだ行の status に done を入れる(Enter → 自由入力で done → Enter)。
fn set_done(a: &mut App) {
    col_named(a, "status");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    wipe(a);
    typing(a, "done");
    press(a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table, "{:?}", a.message);
}

#[test]
fn test_ce_10_bulk_set() {
    // [CE-10] 3行を選んで status に done → 3つのためる変更(1手)、保存前の差分に3ファイル。
    let (_t, mut a) = make("ce10", THREE);
    ctrl(&mut a, 'a');
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert!(screen(&a).contains("一括 3行"));
    assert_fits(&mut a);
    press(&mut a, KeyCode::Esc);
    set_done(&mut a);
    assert_eq!(a.changes.count(), 3);
    for r in 0..3 {
        assert_eq!(pending(&a, r, "status"), Some(NewValue::Str("done".into())));
    }
    assert_eq!(a.changes.previews(a.src.as_ref()).len(), 3);
    // 1手なので、取り消しで3つとも戻る。
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_10_bulk_skips_read_only_rows() {
    // [CE-10] 3行のうち1行が読めないフロントマターのノート(WB-5。閉じの区切りが無い) → 2つのためる変更と
    // 「1行を飛ばした(理由)」。フロントマターの無いノート(b.md)は飛ばさず、ためる変更になる(WB-3)。
    let (_t, mut a) = make(
        "ce10skip",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "本文だけ\n"),
            ("c.md", "---\nstatus: doing\n本文\n"),
        ],
    );
    let idx = |a: &App, label: &str| {
        a.rows
            .iter()
            .position(|r| a.src.label(r) == label)
            .unwrap_or_else(|| panic!("row {label}"))
    };
    let (ra, rb, rc) = (idx(&a, "a.md"), idx(&a, "b.md"), idx(&a, "c.md"));
    // 飛ばす理由は、読めないノートのセルの lock の理由。
    let reason = a
        .src
        .get(&a.rows[rc].clone(), "status")
        .lock
        .expect("[WB-5] unclosed frontmatter is locked");
    // Space で3行に印(印を付けると1行下へ)。今の行は選んだ行の外でも一括になる。
    for _ in 0..3 {
        press(&mut a, KeyCode::Char(' '));
    }
    set_done(&mut a);
    assert_eq!(a.changes.count(), 2);
    assert_eq!(
        pending(&a, ra, "status"),
        Some(NewValue::Str("done".into()))
    );
    assert_eq!(
        pending(&a, rb, "status"),
        Some(NewValue::Str("done".into())),
        "[WB-3] no-frontmatter note is not skipped"
    );
    assert_eq!(pending(&a, rc, "status"), None);
    let msg = a.message.clone().unwrap();
    assert!(msg.contains(&format!("1行を飛ばした({reason})")), "{msg}");
    assert!(!msg.contains("フロントマターが無い"), "{msg}");
    assert!(screen(&a).contains("1行を飛ばした"));
}

#[test]
fn test_ce_10_bulk_toggle_checkbox() {
    // [CE-10] [CE-4] チェックボックスの列は、選んだ行を今の行の次の状態(true の次は false)にそろえる。
    let (_t, mut a) = make(
        "ce10cb",
        &[
            ("a.md", "---\ndone: true\n---\n"),
            ("b.md", "---\ndone: false\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "done");
    a.row = row_of(&a, "a.md");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(
        pending(&a, row_of(&a, "a.md"), "done"),
        Some(NewValue::Bool(false))
    );
    assert_eq!(pending(&a, row_of(&a, "b.md"), "done"), None);
    assert_eq!(a.changes.count(), 1);
    assert!(a.message.as_deref().unwrap().contains("1行は同じ値"));
}

#[test]
fn test_cv_2_mismatched_value_gets_column_input() {
    // [CV-2] date の列の `someday` を編集 → 日付の入力。合わない値のままは書かない。
    let (_t, mut a) = typed(
        "cv2",
        &[
            ("a.md", "---\ndue: someday\n---\n"),
            ("b.md", "---\ndue: 2026-10-05\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    let i = a.input.as_ref().unwrap();
    assert_eq!(i.entry, Entry::Date);
    assert!(i.mismatch);
    let msg = screen(&a).lines().nth(22).unwrap().to_string();
    assert!(msg.contains("合わない") && msg.contains("日付"), "{msg}");
    typing(&mut a, "!");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.changes.count(), 0);
    // 編集前のまま Enter は何も書かずに閉じる。
    ctrl(&mut a, 'r');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
    // チェックボックスの列の合わない値は、切り替えずに true・false・なし のリスト。
    let (_t2, mut b) = make(
        "cv2cb",
        &[
            ("a.md", "---\ndone: false\n---\n"),
            ("b.md", "---\ndone: yes\n---\n"),
        ],
    );
    col_named(&mut b, "done");
    ch(&mut b, 'j');
    press(&mut b, KeyCode::Enter);
    assert_eq!(b.mode, Mode::Edit);
    assert_eq!(list_texts(&b).len(), 3);
    press(&mut b, KeyCode::Up);
    press(&mut b, KeyCode::Up);
    press(&mut b, KeyCode::Up);
    press(&mut b, KeyCode::Enter);
    assert_eq!(pending(&b, 1, "done"), Some(NewValue::Bool(true)));
}

#[test]
fn test_ce_2_types_json_gives_date_input() {
    // [CE-2] types.json で date の列は、値から推定できなくても(全部空)日付の入力になる。
    let (_t, mut a) = typed(
        "ce2",
        &[
            ("a.md", "---\ndue:\ntitle: a\n---\n"),
            ("b.md", "---\ntitle: b\n---\n"),
        ],
    );
    col_named(&mut a, "due");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().entry, Entry::Date);
    assert!(a.active_list().is_none());
}

#[test]
fn test_ce_1_invalid_value_keeps_input_on_click() {
    // [CE-1] 値が不正なら、外のクリックで閉じずにそのクリックを捨てる(行も移らない)。
    let (_t, mut a) = make(
        "ce1bad",
        &[("a.md", "---\nn: 1\n---\n"), ("b.md", "---\nn: 2\n---\n")],
    );
    col_named(&mut a, "n");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "x");
    let (cx, cy) = view::cursor(&a).unwrap();
    a.click(cx - 1, cy + 1);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.row, 0);
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.as_deref().unwrap().contains("数として読めない"));
}

#[test]
fn test_ce_1_click_list_item() {
    // [CE-1] [CE-3] リストの項目のクリックで、その候補に決める。
    let (_t, mut a) = make(
        "ce1list",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: doing\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("|  doing"))
        .unwrap();
    let x = width::width(&line[..line.find("|  doing").unwrap()]) + 3;
    a.click(x as u16, y as u16);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(
        pending(&a, 0, "status"),
        Some(NewValue::Str("doing".into()))
    );
}

#[test]
fn test_ce_11_tab_in_typed_inputs() {
    // [CE-11] リストでも Tab で確定して右へ。チェックボックスへは切り替えずにリストで着く。
    let (_t, mut a) = make(
        "ce11typed",
        &[
            ("a.md", "---\nstatus: todo\ndone: false\nn: 3\n---\n"),
            ("b.md", "---\nstatus: doing\ndone: true\nn: 4\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Tab);
    assert_eq!(
        pending(&a, 0, "status"),
        Some(NewValue::Str("doing".into()))
    );
    assert_eq!(a.cols[a.col], "done");
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.active_list().is_some());
    assert_eq!(pending(&a, 0, "done"), None);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.cols[a.col], "n");
    typing(&mut a, "z");
    a.key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.cols[a.col], "n");
    ctrl(&mut a, 'r');
    assert_eq!(a.input.as_ref().unwrap().text, "3");
}

#[test]
fn test_nv_6_detail_edit_uses_list() {
    // [NV-6] [CE-3] 詳細の表示からの編集でも候補のリストを出す。
    let (_t, mut a) = make(
        "nv6list",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: doing\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    ch(&mut a, 'K');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(screen(&a).contains("|>*todo"), "{}", screen(&a));
    assert_fits(&mut a);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Detail);
    assert_eq!(
        pending(&a, 0, "status"),
        Some(NewValue::Str("doing".into()))
    );
}

#[test]
fn test_ce_10_untouched_input_writes_nothing() {
    // [CE-10] [CE-11] 一括で確定したあと Tab で隣へ移ると1行の入力になり、何も打たずに Tab しても書かない。
    // 一括の入力をすぐ Enter・外のクリックで閉じても、今の行の値を広げない。
    let (_t, mut a) = make(
        "ce10tab",
        &[
            ("a.md", "---\nstatus: todo\nprio: high\n---\n"),
            ("b.md", "---\nstatus: doing\nprio: low\n---\n"),
            ("c.md", "---\nstatus: todo\nprio: low\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    wipe(&mut a);
    typing(&mut a, "done");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.changes.count(), 3);
    assert_eq!(a.cols[a.col], "prio");
    assert_eq!(a.mode, Mode::Edit);
    assert!(a.input.as_ref().unwrap().bulk.is_none());
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.changes.count(), 3);
    // 右に直せるセルが無いので表に戻る。表で Enter → すぐ Enter(リストを動かしていない)。
    assert_eq!(a.mode, Mode::Table);
    col_named(&mut a, "prio");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_some());
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 3);
    // カーソルを動かして自由入力にしただけ(文字を打っていない)で外のクリック。
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Left);
    assert!(a.active_list().is_none());
    let (cx, cy) = view::cursor(&a).unwrap();
    a.click(cx, cy + 5);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 3);
}

#[test]
fn test_cv_2_bulk_list_never_offers_mismatched_value() {
    // [CV-2] [CE-10] チェックボックスの列の合わない値(yes)の行で開いたリストに yes は無く、
    // 動かさずに Enter しても何も書かない。
    let (_t, mut a) = make(
        "cv2bulk",
        &[
            ("a.md", "---\ndone: false\n---\n"),
            ("b.md", "---\ndone: yes\n---\n"),
            ("c.md", "---\ndone: true\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "done");
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let items = list_texts(&a);
    assert_eq!(
        items,
        vec![
            ("true".into(), false),
            ("false".into(), false),
            ("なし".into(), false)
        ]
    );
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0);
    // 自由入力でも yes は書けない。
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::End);
    typing(&mut a, "!");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_10_outside_selection_and_folded_rows() {
    // [CE-10] 今の行が選択の外なら1行の編集。畳んだまとまりの中の行は一括から外し、外した数を出す。
    let (_t, mut a) = make(
        "ce10out",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: doing\n---\n"),
            ("c.md", "---\nstatus: todo\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Char(' '));
    assert_eq!(a.row, 1);
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_none());
    assert_eq!(a.input.as_ref().unwrap().text, "doing");
    wipe(&mut a);
    typing(&mut a, "x");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(pending(&a, 1, "status"), Some(NewValue::Str("x".into())));

    let t = Tmp::new("ce10fold");
    t.write("a.md", "---\nstatus: todo\nprio: high\n---\n");
    t.write("b.md", "---\nstatus: todo\nprio: low\n---\n");
    t.write("c.md", "---\nstatus: doing\nprio: low\n---\n");
    let base = "views:\n  - type: table\n    name: g\n    groupBy:\n      property: status\n      direction: ASC\n    order: [prio, status]\n";
    let mut a = open(&t, base, None);
    // 先頭の見出し(doing)を畳む。
    a.row = 0;
    press(&mut a, KeyCode::Enter);
    ctrl(&mut a, 'a');
    assert_eq!(a.selection().len(), 3);
    a.row = a
        .slots
        .iter()
        .position(|s| matches!(s, super::grid::Slot::Row(_)))
        .unwrap();
    col_named(&mut a, "prio");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.input.as_ref().unwrap().text, "high");
    wipe(&mut a);
    typing(&mut a, "mid");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 2);
    let msg = a.message.clone().unwrap();
    assert!(
        msg.contains("1行は畳んだまとまりの中なので入れない"),
        "{msg}"
    );
}

/// a の status が done・b が todo。Ctrl+A で2行を選び、a の status で入力を開く。
fn done_todo(name: &str) -> (Tmp, App) {
    let (t, mut a) = make(
        name,
        &[
            ("a.md", "---\nstatus: done\n---\n"),
            ("b.md", "---\nstatus: todo\n---\n"),
        ],
    );
    ctrl(&mut a, 'a');
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert!(a.input.as_ref().unwrap().bulk.is_some());
    (t, a)
}

#[test]
fn test_ce_10_bulk_same_as_current_row() {
    // [CE-10] 今の行と同じ値でも、触れば(打ち直し・リストで動かして戻す)選んだ全部の行に揃える。
    // 何も触らずに Enter なら書かない。
    let (_t, mut a) = done_todo("ce10same1");
    wipe(&mut a);
    typing(&mut a, "done");
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "status"), Some(NewValue::Str("done".into())));
    assert_eq!(a.changes.count(), 1);
    let msg = a.message.clone().unwrap();
    assert!(
        msg.contains("1行に入れた") && msg.contains("1行は同じ値"),
        "{msg}"
    );

    let (_t, mut a) = done_todo("ce10same2");
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Up);
    press(&mut a, KeyCode::Enter);
    assert_eq!(pending(&a, 1, "status"), Some(NewValue::Str("done".into())));
    assert!(a.message.as_deref().unwrap().contains("1行に入れた"));

    let (_t, mut a) = done_todo("ce10same3");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.changes.count(), 0);
}
