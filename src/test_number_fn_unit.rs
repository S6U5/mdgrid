//! [BV-6] 式の number()(Obsidian のヘルプの number(input))と、1秒に満たない期間の見せ方
//! (specs/_changes/2026-10-06-number-fn.md)。

use super::*;
use std::collections::HashMap;

fn run(src: &str) -> Val {
    let map: HashMap<String, Value> = HashMap::new();
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
fn test_bv_6_number_days_between_dates() {
    assert_eq!(
        run("number(date(\"2026-10-07\") - date(\"2026-10-06\")) / 86400000"),
        Val::Num(1.0)
    );
}

#[test]
fn test_bv_6_number_kinds() {
    assert_eq!(run("number(\"3.5\")"), Val::Num(3.5));
    assert_eq!(run("number(true)"), Val::Num(1.0));
    assert_eq!(run("number(false)"), Val::Num(0.0));
    assert_eq!(run("number(duration(\"1s\"))"), Val::Num(1000.0));
    assert_eq!(run("number(date(\"1970-01-02\"))"), Val::Num(86_400_000.0));
    assert_eq!(run("number(\"abc\")"), Val::Null);
}

#[test]
fn test_bv_6_number_sub_second_duration_shows_ms() {
    let v = run("(date(\"2026-10-07\") - date(\"2026-10-06\")) / 86400000");
    assert_eq!(crate::print::val_text_with(&v, &|s| s.to_string()), "1ms");
}
