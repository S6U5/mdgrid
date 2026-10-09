//! 札が列に入りきらないとき(SR-35): 札を途中で切らず、入る札だけを見せて残りの数を `+N` で示す。

use super::test_screen::{app_of, screen, Tmp};
use super::*;

fn app(name: &str, tags: &str, w: usize) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    tmp.write("a.md", &format!("---\ntags: {tags}\n---\n"));
    tmp.write("b.md", "---\ntags: [x]\n---\n");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    a.widths.insert("tags".into(), w);
    a.col = 0;
    a.row = 1; // a の行のセルを選ばない(選んだセルは札にしない)
    (tmp, a)
}

fn row_a(a: &App) -> String {
    screen(a)
        .lines()
        .find(|l| l.get(1..).is_some_and(|r| r.starts_with("a ")))
        .unwrap()
        .to_string()
}

#[test]
fn test_sr_35_chips_overflow_counts_the_rest() {
    // [SR-35] 幅 10 に ui・perf・web → ` ui ` のあとに `+2`。perf は途中で切らない。
    let (_t, a) = app("sr35f_more", "[ui, perf, web]", 10);
    let r = row_a(&a);
    assert!(r.contains(" ui ") && r.contains("+2"), "{r}");
    assert!(!r.contains("pe") && !r.contains('…'), "{r}");
}

#[test]
fn test_sr_35_chips_all_fit() {
    // [SR-35] 入りきれば全部の札(+N は無い)。
    let (_t, a) = app("sr35f_all", "[ui, perf]", 20);
    let r = row_a(&a);
    assert!(
        r.contains(" ui ") && r.contains(" perf") && !r.contains('+'),
        "{r}"
    );
}

#[test]
fn test_sr_35_first_chip_too_wide_is_cut() {
    // [SR-35] 1つ目の札が入らなければ、その札を `…` で切って見せる。
    let (_t, a) = app("sr35f_cut", "[documentation]", 8);
    let r = row_a(&a);
    assert!(r.contains("docu") && r.contains('…'), "{r}");
}
