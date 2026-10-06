//! [BV-7] 式で作る文字が上限を超えるときは null(資源を使い切らせない。specs/_changes/2026-10-06-expr-string-cap.md)。

use super::*;
use std::collections::HashMap;

fn run_with(src: &str, x: &str) -> Val {
    let mut map: HashMap<String, Value> = HashMap::new();
    map.insert("x".into(), Value::Str(x.into()));
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |_: &str| None;
    let f = FileInfo {
        name: "n.md".into(),
        basename: "n".into(),
        ext: "md".into(),
        path: "n.md".into(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    };
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today: 20727,
        now: 20727 * 86_400,
    };
    eval(&parse(src).expect(src), &env)
}

#[test]
fn test_bv_7_string_cap_replace_empty_pattern() {
    let big = "a".repeat(2000);
    // 2000 文字の間に 2000 文字ずつ入れると約 4 MB → null。
    assert_eq!(run_with("x.replace(\"\", x)", &big), Val::Null);
    // ふつうの長さはそのまま。
    assert_eq!(
        run_with("x.replace(\"b\", \"c\")", "abc"),
        Val::Str("acc".into())
    );
}

#[test]
fn test_bv_7_string_cap_concat() {
    let half = "a".repeat(600 * 1024);
    assert_eq!(run_with("x + x", &half), Val::Null);
    assert_eq!(run_with("x + \"!\"", "hi"), Val::Str("hi!".into()));
}
