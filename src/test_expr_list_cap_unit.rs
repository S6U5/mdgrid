//! [BV-7] 式で作るリストの重さが上限を超えるときは null(specs/_changes/2026-10-06-expr-list-cap.md)。

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
fn test_bv_7_list_cap_literal_doubling() {
    let half = "a".repeat(600 * 1024);
    // 600 KiB の文字を2つ持つリストは上限(1 MiB)を超える → null。
    assert_eq!(run_with("[x, x]", &half), Val::Null);
    // ふつうの大きさはそのまま。
    assert_eq!(
        run_with("[x, x]", "ab"),
        Val::List(vec![Val::Str("ab".into()), Val::Str("ab".into())])
    );
}

#[test]
fn test_bv_7_list_cap_split_and_flat() {
    let big = "a".repeat(200 * 1024);
    // 1文字ずつに分けると要素が 20 万 → 重さが上限を超える → null。
    assert_eq!(run_with("x.split(\"\")", &big), Val::Null);
    assert_eq!(
        run_with("x.split(\",\")", "a,b"),
        Val::List(vec![Val::Str("a".into()), Val::Str("b".into())])
    );
    assert_eq!(
        run_with("[[x], [x]].flat()", "z"),
        Val::List(vec![Val::Str("z".into()), Val::Str("z".into())])
    );
}
