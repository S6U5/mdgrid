//! [CE-2] types.json の型は、同じ名前が無ければ大文字小文字を問わずに引く(Obsidian のプロパティの名前と同じ)。
//! specs/_changes/2026-10-06-types-case.md。

use super::*;

#[test]
fn test_ce_2_types_json_any_case() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mdgrid-tc-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join(".obsidian")).unwrap();
    std::fs::write(
        dir.join(".obsidian/types.json"),
        r#"{"types": {"rating": "number", "Due": "date", "same": "text", "Same": "number"}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("a.md"),
        "---\nRating: high\ndue: 2026-10-01\nSame: 1\n---\n",
    )
    .unwrap();
    let mut md = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    while !md.load(100).done {}
    assert_eq!(md.kind("Rating").kind, Kind::Number);
    assert_eq!(md.kind("due").kind, Kind::Date);
    // 同じ名前があればそれ(大文字小文字の違う別の宣言より先)。
    assert_eq!(md.kind("Same").kind, Kind::Number);
    assert_eq!(md.kind("same").kind, Kind::Text);
    assert!(md.typed("Rating"));
}
