//! 札の列の判定(SR-35)は、同じ表なら数え直さない。行が増えるなど表が変われば数え直す。

use super::test_screen::{app_of, Tmp};
use super::*;

#[test]
fn test_sr_35_selects_reused_and_renewed() {
    // [SR-35] 絞り込み(表の行は同じ)では前の判定を使い、ノートが増えて行の数が変われば数え直す。
    let tmp = Tmp::new("sr35_cache");
    for (n, s) in [("a", "todo"), ("b", "doing"), ("c", "todo")] {
        tmp.write(
            &format!("{n}.md"),
            &format!("---\nstatus: {s}\nmemo: m{n}\n---\n"),
        );
    }
    let mut a = app_of(&tmp, ColorMode::Rgb);
    assert!(a.is_select("status") && !a.is_select("memo"));
    let key = a.built.selects_key.clone();
    assert!(key.is_some());
    a.filter = Some("todo".into());
    a.refresh();
    assert_eq!(a.built.selects_key, key, "絞り込みでは数え直さない");
    assert!(a.is_select("status"));
    // memo にくり返しの値を足したノートが増える → 表が変わったので数え直し、memo も札の列になる。
    tmp.write("d.md", "---\nstatus: done\nmemo: same\n---\n");
    tmp.write("e.md", "---\nstatus: done\nmemo: same\n---\n");
    let mut b = app_of(&tmp, ColorMode::Rgb);
    b.refresh();
    assert!(b.is_select("memo"), "行が変われば数え直す");
}
