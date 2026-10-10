//! ためた真偽とリストは、決まった値と同じ部品の形で見せる(CE-8 のためる印 `*` は残す。SR-35)。

use super::cell::shown;
use super::test_screen::{app_of, Tmp};
use super::ColorMode;
use mdgrid::source::NewValue;

#[test]
fn test_sr_35_pending_bool_and_list_use_parts() {
    // [SR-35] done を true にためる → `*✓`(生の `*true` でない)。tags に足す → `*a · b`(`*[a, b]` でない)。
    let tmp = Tmp::new("sr35pending");
    tmp.write("a.md", "---\ndone: false\ntags: [a]\n---\n");
    tmp.write("b.md", "---\ndone: true\ntags: [b]\n---\n");
    let mut a = app_of(&tmp, ColorMode::Rgb);
    let r = a.rows[0].clone();
    let src: &dyn mdgrid::source::Source = &*a.src;
    let mut ch = std::mem::take(&mut a.changes);
    ch.set(src, &r, "done", NewValue::Bool(true)).unwrap();
    ch.set(
        src,
        &r,
        "tags",
        NewValue::List(vec!["a".into(), "b".into()]),
    )
    .unwrap();
    a.changes = ch;
    let d = shown(&a, &r, "done").text;
    let t = shown(&a, &r, "tags").text;
    assert!(d.starts_with('*') && !d.contains("true"), "{d}");
    assert_eq!(t, "*a · b");
}
