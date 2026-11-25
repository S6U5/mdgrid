//! 見た目の状態の列の並びを、人が動かすか隠したときだけ覚え(screen.state-store-arranged-only)、印の `order` が
//! 覚えたときから変わったら `order` に合わせる(screen.state-store-follow-order)試験。値は作ったもの。
//!
//! 期待値は要求の受け入れの例から取った: ビューの `order` が a・b・c の `.base`。状態の置き場は一時フォルダ。
//! 終了(`q`)で状態を書き、同じ置き場で開き直す。

use super::app::{App, ColorMode};
use super::startup::Startup;
use super::test_screen::{ch, col_named, Tmp};
use std::path::PathBuf;

const NOTES: &[(&str, &str)] = &[
    ("n1.md", "---\na: 1\nb: 2\nc: 3\nd: 4\nx: 5\n---\n"),
    ("n2.md", "---\na: 6\nb: 7\nc: 8\nd: 9\nx: 0\n---\n"),
];

/// ビュー V の列 `order` の `.base` の中身。
fn base_text(order: &[&str]) -> String {
    format!(
        "views:\n  - type: table\n    name: V\n    order: [{}]\n",
        order.join(", ")
    )
}

/// ノートと `.base`(列 `order`)を置いたフォルダ。
fn ledger(tag: &str, order: &[&str]) -> Tmp {
    let tmp = Tmp::new(tag);
    std::fs::create_dir_all(tmp.notes().join(".obsidian")).unwrap();
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    tmp.write("t.base", &base_text(order));
    tmp
}

/// `.base` の列を `order` に書き換える(人が印を手で直す)。
fn rewrite(tmp: &Tmp, order: &[&str]) {
    tmp.write("t.base", &base_text(order));
}

/// main と同じ道筋で `.base` を開く(状態の置き場は `<一時>/state`)。
fn boot(tmp: &Tmp) -> App {
    let p: PathBuf = tmp.notes().join("t.base");
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut app = App::new(Box::new(t.src), ColorMode::None);
    app.resize(100, 24);
    app.start(Startup {
        config: Default::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: p,
        base: t.base,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

#[test]
fn test_state_open_only_does_not_pin_order() {
    // [utest -> req~screen.state-store-arranged-only~1] order が a・b・c の表を開いて何もせずに終了し、order を
    // a・b・d・c にして開き直すと、列は a・b・d・c。
    let tmp = ledger("so_open", &["a", "b", "c"]);
    let mut a = boot(&tmp);
    assert_eq!(a.cols, ["a", "b", "c"]);
    ch(&mut a, 'q');
    rewrite(&tmp, &["a", "b", "d", "c"]);
    let b = boot(&tmp);
    assert_eq!(b.cols, ["a", "b", "d", "c"]);
}

#[test]
fn test_state_width_only_does_not_pin_order() {
    // [utest -> req~screen.state-store-arranged-only~1] b の幅を変えて終了し、order を a・b・d・c にして開き直すと、
    // 列は a・b・d・c で、b は変えた幅のまま。
    let tmp = ledger("so_width", &["a", "b", "c"]);
    let mut a = boot(&tmp);
    a.set_width("b", 7);
    ch(&mut a, 'q');
    rewrite(&tmp, &["a", "b", "d", "c"]);
    let b = boot(&tmp);
    assert_eq!(b.cols, ["a", "b", "d", "c"]);
    assert_eq!(b.widths.get("b"), Some(&7));
}

/// 列 c を先頭に動かす(c・a・b)。
fn move_c_first(a: &mut App) {
    col_named(a, "c");
    ch(a, 'H');
    ch(a, 'H');
    assert_eq!(a.cols, ["c", "a", "b"]);
}

#[test]
fn test_state_follow_changed_order_keeps_user_move() {
    // [utest -> req~screen.state-store-follow-order~1] order a・b・c で c を先頭に動かして終了し、order を a・x・b・c に
    // して開き直すと、列は c・a・x・b(足した x は印の位置、先頭に動かした c は先頭のまま)。
    let tmp = ledger("so_follow", &["a", "b", "c"]);
    let mut a = boot(&tmp);
    move_c_first(&mut a);
    ch(&mut a, 'q');
    rewrite(&tmp, &["a", "x", "b", "c"]);
    let b = boot(&tmp);
    assert_eq!(b.cols, ["c", "a", "x", "b"]);
}

#[test]
fn test_state_same_order_keeps_arrangement() {
    // [utest -> req~screen.state-store-follow-order~1] c を先頭に動かして終了し、order を変えずに開き直すと c・a・b。
    let tmp = ledger("so_same", &["a", "b", "c"]);
    let mut a = boot(&tmp);
    move_c_first(&mut a);
    ch(&mut a, 'q');
    let b = boot(&tmp);
    assert_eq!(b.cols, ["c", "a", "b"]);
}

#[test]
fn test_state_follow_changed_order_keeps_hidden() {
    // [utest -> req~screen.state-store-follow-order~1] c を先頭に動かし、b を隠して終了し、order を b・a・c にして
    // 開き直すと、列は c・a で、b は隠したまま。
    let tmp = ledger("so_hidden", &["a", "b", "c"]);
    let mut a = boot(&tmp);
    move_c_first(&mut a);
    col_named(&mut a, "b");
    ch(&mut a, '-');
    assert_eq!(a.cols, ["c", "a"]);
    ch(&mut a, 'q');
    rewrite(&tmp, &["b", "a", "c"]);
    let b = boot(&tmp);
    assert_eq!(b.cols, ["c", "a"]);
    assert!(b.hidden.iter().any(|(c, _, _)| c == "b"), "{:?}", b.hidden);
}
