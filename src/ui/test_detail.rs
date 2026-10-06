//! 詳細の表示の試験(NV-6・SR-16)。
use super::keymap::{lookup, Action, Mode, BINDINGS};
use super::test_screen::*;
use super::view;
use ratatui::crossterm::event::KeyCode;

const LONG: &str = "長い説明の値を折り返して全文を見せる。表のセルでは切れるが、詳細の表示では最後の一文字まで読める。終わり。";

fn note() -> String {
    format!("---\ntitle: 会議のメモ\ndescription: {LONG}\nstatus: 進行中\n---\n")
}

#[test]
fn test_nv_6_detail_wraps_long_value() {
    // [NV-6] `K` で全プロパティを縦に。長い値は折り返して全文。
    let n = note();
    let (_t, mut a) = make(
        "nv6",
        &[
            ("a.md", n.as_str()),
            ("b.md", "---\ntitle: b\nextra: x\n---\n"),
        ],
    );
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    let s = screen(&a);
    // 本文の行をつなげる(空白を除く)と全文が出る。
    let joined: String = s
        .lines()
        .skip(1)
        .take(20)
        .flat_map(|l| l.split_whitespace())
        .collect();
    assert!(joined.contains(LONG), "全文が出る: {s}");
    assert!(s
        .lines()
        .any(|l| l.contains("status") && l.contains("進行中")));
    golden("nv_6", &s);
    // 表の列はキーの無いノートでも出す(空欄。CV-1)。
    assert!(s.contains("extra"));
    assert_fits(&mut a);
}

#[test]
fn test_sr_16_detail_keys_edit_and_close() {
    // [SR-16] 詳細の表示で Enter = 選んだプロパティの編集、Esc と `K` = 閉じる。表で `K` = 詳細、Space = 行の印。
    assert_eq!(lookup(BINDINGS, Mode::Table, "K"), Some(Action::Detail));
    assert_eq!(lookup(BINDINGS, Mode::Table, "Space"), Some(Action::Mark));
    assert_eq!(lookup(BINDINGS, Mode::Table, "Esc"), Some(Action::Escape));
    assert_eq!(lookup(BINDINGS, Mode::Detail, "Enter"), Some(Action::Edit));
    assert_eq!(lookup(BINDINGS, Mode::Detail, "Esc"), Some(Action::Close));
    assert_eq!(lookup(BINDINGS, Mode::Detail, "K"), Some(Action::Close));
    assert_eq!(Mode::Detail.label(), "詳細の表示");
    let n = note();
    let (_t, mut a) = make("sr16d", &[("a.md", n.as_str())]);
    ch(&mut a, 'K');
    // 並びは表の列の順(description, status, title)。status を選んで編集する。
    let d = a.detail.as_ref().unwrap();
    let props = a.detail_props(&d.row);
    let k = props.iter().position(|p| p == "status").unwrap();
    for _ in 0..k {
        ch(&mut a, 'j');
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    assert!(view::cursor(&a).is_some(), "入力の位置にカーソル(SR-17)");
    ctrl(&mut a, 'r');
    for _ in 0..3 {
        press(&mut a, KeyCode::Backspace);
    }
    typing(&mut a, "完了");
    assert!(screen(&a).contains("完了"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Detail, "確定したら詳細の表示に戻る");
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("*完了"));
    assert_fits(&mut a);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
    ch(&mut a, 'K');
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Table);
    assert!(a.detail.is_none());
}

#[test]
fn test_ce_1_click_inside_detail_input_keeps_editing() {
    // [CE-1] [NV-6] 詳細の表示からの編集で、入力ボックスの中のクリックは確定しない。外のクリックは確定する。
    let n = note();
    let (_t, mut a) = make("ce1detail", &[("a.md", n.as_str())]);
    ch(&mut a, 'K');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Edit);
    let (x, y) = view::cursor(&a).unwrap();
    a.click(x, y);
    assert_eq!(a.mode, Mode::Edit);
    a.click(0, 20);
    assert_eq!(a.mode, Mode::Detail);
}
