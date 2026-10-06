//! Shift+矢印の試験(CE-23・CE-24 の「カレンダーの外」。関係: SR-4・NV-5)。
//! Shift+←→ を割り当てていないモードでは ←→ と同じ結果になり、表の Shift+↑↓ は範囲の選択のまま。
//! 日付の列でもカレンダーが出ていなければ、Shift+←→ は ←→、Shift+↑↓ は ↑↓ と同じ。
//! 同じ材料の App を2つ作り、片方に ←/→(↑/↓)、もう片方に Shift 付きを送って、画面と状態を比べる。

use super::keymap::Mode;
use super::settings::Sec;
use super::test_screen::{ch, col_named, make, press, screen, typing, Tmp};
use super::test_settings_screen as st;
use super::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn shift(a: &mut App, code: KeyCode) {
    a.key(KeyEvent::new(code, KeyModifiers::SHIFT));
}

/// `setup` で同じ App を2つ作り、`code` を Shift 無しと有りで送って、モード・セル・画面が同じことを確かめる。
/// Shift 付きの側を返す。
fn twin(name: &str, setup: impl Fn(&str) -> (Tmp, App), code: KeyCode) -> ((Tmp, App), (Tmp, App)) {
    let (tp, mut p) = setup(&format!("{name}p"));
    let (ts, mut s) = setup(&format!("{name}s"));
    press(&mut p, code);
    shift(&mut s, code);
    assert_eq!(p.mode, s.mode, "{name} {code:?}");
    assert_eq!((p.row, p.col), (s.row, s.col), "{name} {code:?}");
    assert_eq!(
        p.input.as_ref().map(|i| (i.text.clone(), i.cursor)),
        s.input.as_ref().map(|i| (i.text.clone(), i.cursor)),
        "{name} {code:?}"
    );
    assert_eq!(screen(&p), screen(&s), "{name} {code:?}");
    ((tp, p), (ts, s))
}

const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: 会議\nstatus: todo\n---\n"),
    ("b.md", "---\ntitle: 買い物\nstatus: done\n---\n"),
    ("c.md", "---\ntitle: 旅行\nstatus: todo\n---\n"),
];

#[test]
fn test_ce_23_table_shift_left_right_is_left_right() {
    // [CE-23][SR-4] 表では Shift+←→ も左右の列へ動く(表に Shift+←→ の割り当てが無い)。
    let setup = |n: &str| {
        let (t, mut a) = make(n, NOTES);
        col_named(&mut a, "title");
        (t, a)
    };
    let col = setup("sa_table_0").1.col;
    let ((_t, p), _) = twin("sa_table_r", setup, KeyCode::Right);
    assert_eq!(p.col, col + 1, "→ で右の列へ動く");
    let setup_r = |n: &str| {
        let (t, mut a) = setup(n);
        press(&mut a, KeyCode::Right);
        (t, a)
    };
    let ((_t2, p), _) = twin("sa_table_l", setup_r, KeyCode::Left);
    assert_eq!(p.col, col, "← で左の列へ動く");
}

#[test]
fn test_ce_24_table_shift_up_down_still_selects_range() {
    // [CE-24][NV-5] 表の Shift+↑↓ は範囲の選択のまま(編集のモードの年のキーは表に効かない)。
    let (_t, mut a) = make("sa_range", NOTES);
    shift(&mut a, KeyCode::Down);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.selection().len(), 2);
    shift(&mut a, KeyCode::Down);
    assert_eq!(a.selection().len(), 3);
    shift(&mut a, KeyCode::Up);
    assert_eq!(a.selection().len(), 2);
}

#[test]
fn test_ce_23_settings_buttons_shift_left_right() {
    // [CE-23] ビューの設定の画面のボタンの行で、Shift+←→ も ←→ と同じにボタンを動く。
    let setup = |n: &str| {
        let (t, mut a) = st::folder(n);
        st::open(&mut a);
        st::section(&mut a, Sec::Buttons);
        (t, a)
    };
    let btn = |a: &App| st::draft(a).sel[Sec::Buttons as usize];
    let ((_t, p), (_t2, s)) = twin("sa_set_r", setup, KeyCode::Right);
    assert_eq!((btn(&p), btn(&s)), (1, 1), "→ で次のボタン");
    let setup_r = |n: &str| {
        let (t, mut a) = setup(n);
        press(&mut a, KeyCode::Right);
        (t, a)
    };
    let ((_t, p), (_t2, s)) = twin("sa_set_l", setup_r, KeyCode::Left);
    assert_eq!((btn(&p), btn(&s)), (0, 0), "← で前のボタン");
}

#[test]
fn test_ce_23_chips_shift_left_right() {
    // [CE-23] 設定の帯の中で、Shift+←→ も ←→ と同じに項目を動く。
    let setup = |n: &str| {
        let (t, mut a) = st::folder(n);
        st::open(&mut a);
        st::add_values(&mut a, "status", &[Some("done")], false);
        st::add_values(&mut a, "種別", &[Some("メモ")], false);
        st::apply_btn(&mut a);
        ch(&mut a, 'f');
        assert_eq!(a.mode, Mode::Chips);
        (t, a)
    };
    let ((_t, p), (_t2, s)) = twin("sa_chip_r", setup, KeyCode::Right);
    assert_eq!((p.chip, s.chip), (1, 1), "→ で右の項目");
    let setup_r = |n: &str| {
        let (t, mut a) = setup(n);
        press(&mut a, KeyCode::Right);
        (t, a)
    };
    let ((_t, p), (_t2, s)) = twin("sa_chip_l", setup_r, KeyCode::Left);
    assert_eq!((p.chip, s.chip), (0, 0), "← で左の項目");
}

#[test]
fn test_ce_23_search_and_palette_shift_left_right() {
    // [CE-23] 検索の欄・パレットでも、Shift+←→ は ←→ と同じ(どちらも語を変えない)。続けて打った語も同じ。
    for (open, name) in [('/', "sa_search"), (':', "sa_palette")] {
        for code in [KeyCode::Left, KeyCode::Right] {
            let setup = |n: &str| {
                let (t, mut a) = make(n, NOTES);
                ch(&mut a, open);
                assert_ne!(a.mode, Mode::Table);
                typing(&mut a, "会");
                (t, a)
            };
            let ((_t, mut p), (_t2, mut s)) = twin(name, setup, code);
            typing(&mut p, "議");
            typing(&mut s, "議");
            assert_eq!(screen(&p), screen(&s), "{name} {code:?}");
            assert!(screen(&s).contains("会議"), "{name} {code:?}");
        }
    }
}

#[test]
fn test_ce_23_list_pick_shift_left_right() {
    // [CE-23][CE-16] リストの選択(tags の付け外し)でも、Shift+←→ は ←→ と同じ。続けて付け外しても同じ。
    let setup = |n: &str| {
        let (t, mut a) = make(
            n,
            &[
                ("a.md", "---\ntitle: 一\ntags: [会議, 本]\n---\n"),
                ("b.md", "---\ntitle: 二\ntags: [会議]\n---\n"),
            ],
        );
        col_named(&mut a, "tags");
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::ListPick, "{:?}", a.message);
        press(&mut a, KeyCode::Down);
        (t, a)
    };
    for code in [KeyCode::Left, KeyCode::Right] {
        let ((_t, mut p), (_t2, mut s)) = twin("sa_pick", setup, code);
        ch(&mut p, ' ');
        ch(&mut s, ' ');
        assert_eq!(screen(&p), screen(&s), "{code:?}");
        press(&mut p, KeyCode::Tab);
        press(&mut s, KeyCode::Tab);
        assert_eq!(p.mode, Mode::Table);
        assert_eq!(
            p.changes.pending(&p.rows[0], "tags"),
            s.changes.pending(&s.rows[0], "tags"),
            "{code:?}"
        );
    }
}

#[test]
fn test_ce_23_text_input_shift_left_right_moves_cursor() {
    // [CE-23] 日付でない列の入力では、Shift+←→ は文字のカーソルを ←→ と同じに動かす。
    let setup = |n: &str| {
        let (t, mut a) = make(n, NOTES);
        col_named(&mut a, "title");
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Edit);
        press(&mut a, KeyCode::Home);
        press(&mut a, KeyCode::Right);
        (t, a)
    };
    let ((_t, p), _) = twin("sa_text_r", setup, KeyCode::Right);
    assert_eq!(p.input.as_ref().unwrap().cursor, "会議".len());
    let ((_t, p), _) = twin("sa_text_l", setup, KeyCode::Left);
    assert_eq!(p.input.as_ref().unwrap().cursor, 0);
}

#[test]
fn test_ce_24_no_calendar_shift_up_down_is_up_down() {
    // [CE-24][SR-9] 日付の列でもカレンダーが出ていない(幅 30)ときは、Shift+↑↓ は ↑↓ と同じで、日付は変わらない。
    let setup = |n: &str| {
        let tmp = Tmp::new(n);
        std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
        std::fs::write(
            tmp.notes().join(".obsidian/types.json"),
            r#"{"types": {"due": "date"}}"#,
        )
        .unwrap();
        tmp.write("a.md", "---\ndue: 2026-10-05\n---\n");
        let mut a = super::test_screen::app_of(&tmp, ColorMode::None);
        a.resize(30, 24);
        col_named(&mut a, "due");
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, Mode::Edit);
        assert!(!calendar::shown(&a));
        (tmp, a)
    };
    for code in [KeyCode::Up, KeyCode::Down] {
        let ((_t, p), (_t2, mut s)) = twin("sa_date", setup, code);
        assert_eq!(p.input.as_ref().unwrap().text, "2026-10-05", "{code:?}");
        press(&mut s, KeyCode::Enter);
        assert_eq!(s.changes.count(), 0, "{code:?}");
    }
}
