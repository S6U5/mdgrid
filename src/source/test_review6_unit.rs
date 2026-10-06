//! [CE-2] types.json の型を大文字小文字を外して引く表(specs/_changes/2026-10-06-review6-fixes.md)。
//! 大文字小文字だけ違う宣言の型が食い違えば、同じ名前の無い列はテキストで読むだけ(BV-12 と同じ)。

use super::*;

#[test]
fn test_ce_2_types_case_variants_conflict() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mdgrid-r6u-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join(".obsidian")).unwrap();
    std::fs::write(
        dir.join(".obsidian/types.json"),
        r#"{"types": {"Rating": "number", "rating": "text", "Score": "number"}}"#,
    )
    .unwrap();
    std::fs::write(dir.join("a.md"), "---\nRATING: 1\nscore: 2\n---\n").unwrap();
    let mut md = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    while !md.load(100).done {}
    let k = md.kind("RATING");
    assert_eq!(k.kind, Kind::Text);
    assert!(k.lock.is_some());
    assert_eq!(md.kind("score").kind, Kind::Number);
    assert_eq!(md.kind("Rating").kind, Kind::Number);
}
