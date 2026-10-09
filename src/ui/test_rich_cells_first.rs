//! 札の並びの1つ目(SR-35): 単独で入るなら、+N の幅が取れなくても丸ごと見せる。

use super::test_screen::{app_of, screen, Tmp};
use super::*;

#[test]
fn test_sr_35_first_chip_whole_before_count() {
    // [SR-35] 幅 9 に launch・writing → ` launch ` を丸ごと(+1 は入らないので出さない)。
    let tmp = Tmp::new("sr35w_first");
    tmp.write("a.md", "---\ntags: [launch, writing]\n---\n");
    tmp.write("b.md", "---\ntags: [x]\n---\n");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    a.widths.insert("tags".into(), 9);
    a.row = 1;
    let r = screen(&a)
        .lines()
        .find(|l| l.get(1..).is_some_and(|r| r.starts_with("a ")))
        .unwrap()
        .to_string();
    assert!(r.contains(" launch") && !r.contains('…'), "{r}");
}
