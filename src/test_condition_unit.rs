//! 名前を付けた条件(specs/requirements/condition.md)の核の試験: 設定の読み・式の中の `cond()`・輪・数字の重なり。
//! 期待値は要求の受け入れの例から取った。

use crate::condition::{self, Conditions};
use crate::config;
use crate::expr::{self, Env, Val};
use crate::source::{FileInfo, Value};
use crate::types::parse_date;
use std::collections::HashMap;
use std::sync::Arc;

fn read(text: &str) -> (Arc<Conditions>, Vec<String>) {
    let (c, w) = config::parse(text).unwrap();
    (c.conditions, w)
}

fn file() -> FileInfo {
    FileInfo {
        name: "a.md".to_string(),
        basename: "a".to_string(),
        ext: "md".to_string(),
        path: "a.md".to_string(),
        folder: String::new(),
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    }
}

/// `src` を今の条件で読み、行(キーと値)ごとに評価した値が `true` か。
fn hits(src: &str, rows: &[&[(&str, &str)]], today: &str) -> Vec<bool> {
    let e = expr::parse(src).unwrap_or_else(|e| panic!("{src}: {e:?}"));
    let today = parse_date(today).unwrap();
    rows.iter()
        .map(|kv| {
            let m: HashMap<String, Value> = kv
                .iter()
                .map(|(k, v)| (k.to_string(), Value::Str(v.to_string())))
                .collect();
            let prop = |k: &str| m.get(k).cloned();
            let formula = |_: &str| None;
            let f = file();
            let env = Env {
                prop: &prop,
                file: &f,
                formula: &formula,
                today,
                now: today * 86_400,
            };
            matches!(expr::eval(&e, &env), Val::Bool(true))
        })
        .collect()
}

const SOON: &str = "[conditions.soon]\nexpr = 'due <= today() + \"7d\"'\n";

#[test]
fn test_condition_expr_call_combines_with_other_expressions() {
    // [utest -> req~condition.expr-call~1] cond("soon") はほかの式と組み、`!` で反転できる。
    let (c, w) = read(SOON);
    assert!(w.is_empty(), "{w:?}");
    let _g = condition::scoped(c);
    let rows: [&[(&str, &str)]; 3] = [
        &[("due", "2026-11-24"), ("status", "todo")],
        &[("due", "2026-11-24"), ("status", "done")],
        &[("due", "2026-12-11"), ("status", "todo")],
    ];
    assert_eq!(
        hits(r#"cond("soon") && status != "done""#, &rows, "2026-11-21"),
        [true, false, false]
    );
    assert_eq!(
        hits(r#"!cond("soon")"#, &rows, "2026-11-21"),
        [false, false, true]
    );
}

#[test]
fn test_condition_define_boundary_of_seven_days() {
    // [condition.define] 今日が 2026-11-21 なら 2026-11-28 は当たり、2026-11-29 は当たらない。
    let (c, _) = read(SOON);
    let _g = condition::scoped(c);
    let rows: [&[(&str, &str)]; 2] = [&[("due", "2026-11-28")], &[("due", "2026-11-29")]];
    assert_eq!(hits(r#"cond("soon")"#, &rows, "2026-11-21"), [true, false]);
}

#[test]
fn test_condition_invalid_name_and_expression_are_warned_and_skipped() {
    // [utest -> req~condition.invalid~1] 外れた名前と読めない式は警告して外し、ほかの条件は使える。
    let text = format!(
        "[conditions.\"期限 近い\"]\nexpr = 'true'\n[conditions.broken]\nexpr = 'due <='\n{SOON}"
    );
    let (c, w) = read(&text);
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(
        w.iter().any(|x| x.contains("conditions.期限 近い")),
        "{w:?}"
    );
    assert!(
        w.iter().any(|x| x.contains("conditions.broken.expr")),
        "{w:?}"
    );
    let names: Vec<&str> = c.list().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["soon"]);
}

#[test]
fn test_condition_cycle_warns_once_and_drops_the_loop() {
    // [utest -> req~condition.cycle~1] a と b の輪は警告1つで、どちらも無い名前の扱い。c は使える。
    let text = format!(
        "[conditions.a]\nexpr = 'cond(\"b\")'\n[conditions.b]\nexpr = 'cond(\"a\")'\n\
         [conditions.c]\nexpr = 'cond(\"soon\")'\n{SOON}"
    );
    let (c, w) = read(&text);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("a → b → a"), "{w:?}");
    let _g = condition::scoped(c);
    assert!(expr::parse(r#"cond("a")"#).is_err());
    let rows: [&[(&str, &str)]; 2] = [&[("due", "2026-11-22")], &[("due", "2026-12-30")]];
    assert_eq!(hits(r#"cond("c")"#, &rows, "2026-11-21"), [true, false]);
}

#[test]
fn test_condition_unknown_name_lists_the_known_names() {
    // [utest -> req~condition.unknown-name~1] 無い名前は読めない式になり、理由に無い名前とある名前が出る。
    let text = format!("{SOON}[conditions.far]\nexpr = 'due > today() + \"30d\"'\n");
    let (c, _) = read(&text);
    let _g = condition::scoped(c);
    let crate::i18n::Lang::Ja = crate::i18n::current() else {
        panic!("試験の環境は日本語のはず");
    };
    match expr::parse(r#"cond("sooon")"#) {
        Err(expr::ExprError::Syntax(m)) => {
            assert_eq!(m, "条件「sooon」はありません(ある名前: far, soon)");
        }
        other => panic!("読めない式のはず: {other:?}"),
    }
}

#[test]
fn test_condition_digit_dup_first_written_wins() {
    // [utest -> req~condition.digit-dup~2] 同じファイルの同じ数字は先に書いた条件だけを並びに入れ、あとの条件の key は警告して外す。
    let text =
        "[conditions.soon]\nexpr = 'true'\nkey = 1\n[conditions.mine]\nexpr = 'true'\nkey = 1\n";
    let (c, w) = read(text);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("conditions.mine.key"), "{w:?}");
    assert_eq!(c.by_key(1).map(|c| c.name.as_str()), Some("soon"));
    assert_eq!(c.get("mine").and_then(|c| c.key), None);
    assert!(c.expr("mine").is_some(), "mine は cond() では使える");
}

#[test]
fn test_condition_label_defaults_to_the_name() {
    // [utest -> req~condition.label~2] label があればそれを、無ければ名前を見せる。
    let text = format!("{SOON}label = \"期限が近い\"\n[conditions.far]\nexpr = 'true'\n");
    let (c, _) = read(&text);
    assert_eq!(
        c.get("soon").map(condition::Condition::shown),
        Some("期限が近い")
    );
    assert_eq!(c.get("far").map(condition::Condition::shown), Some("far"));
}
