//! [WB-5][BV-1] フローの形のフロントマターは、大きさと入れ子の深さの上限を超えたら読まない(落ちない)。
//! specs/_changes/2026-10-06-flow-frontmatter-limits.md。

use super::flow_frontmatter;

fn note(inner: &str) -> Vec<u8> {
    format!("---\n{inner}\n---\nbody\n").into_bytes()
}

#[test]
fn test_wb_5_flow_limits_deep_nesting_is_refused() {
    // 10 万段の入れ子: 読み手の再帰に渡さず None(スタックを使い切らない)。
    let deep = format!("{{a: {}{}}}", "[".repeat(100_000), "]".repeat(100_000));
    assert!(flow_frontmatter(&note(&deep)).is_none());
    let deep_map = format!("{}{}", "{a: ".repeat(40), "}".repeat(40));
    assert!(flow_frontmatter(&note(&deep_map)).is_none());
}

#[test]
fn test_wb_5_flow_limits_large_is_refused() {
    let big = format!("{{a: \"{}\"}}", "x".repeat(70 * 1024));
    assert!(flow_frontmatter(&note(&big)).is_none());
}

#[test]
fn test_wb_5_flow_limits_normal_still_read() {
    let fm = flow_frontmatter(&note("{tags: [a, \"[[x]]\"], s: \"{{not nesting}}\"}")).unwrap();
    assert_eq!(fm.entries.len(), 2);
}

#[test]
fn test_wb_5_flow_limits_deep_base_is_refused() {
    // .base の読み取りも同じ上限(ブロックの入れ子も数える)。
    let mut text = String::from("views:\n");
    for i in 0..100 {
        text.push_str(&" ".repeat(2 * i + 2));
        text.push_str("a:\n");
    }
    assert!(crate::base::parse(&text).is_err());
    assert!(crate::base::parse("views:\n  - type: table\n    name: v\n").is_ok());
}
