//! 式(src/expr.rs)の受け入れテスト(タスク 6)。
//! BV-6・BV-7。仕様: specs/base-view/spec.md。形: docs/design.md の src/expr.rs と FileInfo。
//! 各例は Obsidian のヘルプ(bases/syntax・bases/functions)の意味を写した小さな例。
//!
//! 今日は 2026-10-01(1970-01-01 から 20727 日)に固定する。

use mdgrid::expr::{eval, parse, to_val, Env, Expr, ExprError, Val};
use mdgrid::source::{FileInfo, Value};
use std::collections::HashMap;

const TODAY: i64 = 20727; // 2026-10-01
const NOW: i64 = TODAY * 86_400 + 12 * 3600; // 2026-10-01 12:00:00 UTC
const DAY_MS: i64 = 86_400_000;

// ---- 小さな helper ----

fn file() -> FileInfo {
    FileInfo {
        name: "a b.md".to_string(),
        basename: "a b".to_string(),
        ext: "md".to_string(),
        path: "notes/sub/a b.md".to_string(),
        folder: "notes/sub".to_string(),
        size: 123,
        mtime: TODAY * 86_400 - 3600,
        ctime: TODAY * 86_400 - 10 * 86_400,
        tags: vec!["a/b".to_string(), "proj".to_string()],
    }
}

fn props() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    m.insert("status".to_string(), Value::Str("done".to_string()));
    m.insert("a b".to_string(), Value::Str("spaced".to_string()));
    m.insert("count".to_string(), Value::Int(3));
    m.insert("price".to_string(), Value::Float(2.5));
    m.insert("flag".to_string(), Value::Bool(true));
    m.insert("due".to_string(), Value::Str("2026-09-30".to_string()));
    m.insert(
        "list".to_string(),
        Value::List(vec![
            Value::Str("x".to_string()),
            Value::Str("y".to_string()),
            Value::Str("z".to_string()),
        ]),
    );
    m.insert("empty".to_string(), Value::Str(String::new()));
    m.insert("nothing".to_string(), Value::Null);
    m
}

fn try_parse(src: &str) -> Result<Expr, ExprError> {
    parse(src)
}

fn must_parse(src: &str) -> Expr {
    match parse(src) {
        Ok(e) => e,
        Err(ExprError::Syntax(m)) => panic!("{src:?} が Syntax({m:?}) になった"),
        Err(ExprError::Unsupported(m)) => panic!("{src:?} が Unsupported({m:?}) になった"),
    }
}

/// 固定の Env で式を評価する。
fn ev(src: &str) -> Val {
    let map = props();
    let f = file();
    let prop = move |k: &str| map.get(k).cloned();
    let formula = |k: &str| match k {
        "x" => Some(Val::Num(42.0)),
        "label" => Some(Val::Str("hi".to_string())),
        _ => None,
    };
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today: TODAY,
        now: NOW,
    };
    eval(&must_parse(src), &env)
}

fn t(src: &str) {
    assert_eq!(ev(src), Val::Bool(true), "{src}");
}

fn f(src: &str) {
    assert_eq!(ev(src), Val::Bool(false), "{src}");
}

fn s(v: &str) -> Val {
    Val::Str(v.to_string())
}

// ---- BV-6: 演算子 ----

#[test]
fn test_bv_6_comparison() {
    // [BV-6] 比較 == != < <= > >=。== と != はどの型にも使える。
    t("1 == 1");
    t("1 == 1.0");
    f("1 == 2");
    t("1 != 2");
    t("1 < 2");
    t("2 <= 2");
    f("3 > 4");
    t("4 >= 3");
    t("\"a\" == 'a'");
    t("\"a\" != \"b\"");
    t("true == true");
}

#[test]
fn test_bv_6_logic() {
    // [BV-6] 論理 && || !
    t("true && true");
    f("true && false");
    t("false || true");
    f("!true");
    t("!false");
    t("1 < 2 && 2 < 3");
}

#[test]
fn test_bv_6_arithmetic() {
    // [BV-6] 算術 + - * / %
    assert_eq!(ev("1 + 2"), Val::Num(3.0));
    assert_eq!(ev("5 - 7"), Val::Num(-2.0));
    assert_eq!(ev("3 * 4"), Val::Num(12.0));
    assert_eq!(ev("10 / 4"), Val::Num(2.5));
    assert_eq!(ev("7 % 3"), Val::Num(1.0));
    assert_eq!(ev("2 * (1 + 2) * 2.5"), Val::Num(2.0 * 3.0 * 2.5));
}

#[test]
fn test_bv_6_precedence_and_parens() {
    // [BV-6] 優先順位と括弧。* は + より強く、&& は || より強い。- は左から。
    assert_eq!(ev("1 + 2 * 3"), Val::Num(7.0));
    assert_eq!(ev("(1 + 2) * 3"), Val::Num(9.0));
    assert_eq!(ev("1 - 2 - 3"), Val::Num(-4.0));
    assert_eq!(ev("8 / 2 / 2"), Val::Num(2.0));
    t("true || false && false");
    f("(true || false) && false");
    t("1 + 1 == 2");
    f("!(1 < 2)");
}

#[test]
fn test_bv_6_literals() {
    // [BV-6] 文字列・数・true・false・null・リスト
    assert_eq!(ev("'abc'"), s("abc"));
    assert_eq!(ev("\"abc\""), s("abc"));
    assert_eq!(ev("12.5"), Val::Num(12.5));
    assert_eq!(ev("true"), Val::Bool(true));
    assert_eq!(ev("null"), Val::Null);
    assert_eq!(ev("[1, \"a\"]"), Val::List(vec![Val::Num(1.0), s("a")]));
}

// ---- BV-6: 参照 ----

#[test]
fn test_bv_6_note_references() {
    // [BV-6] `status`・`note.status`・`note["a b"]`
    assert_eq!(ev("status"), s("done"));
    assert_eq!(ev("note.status"), s("done"));
    assert_eq!(ev("note[\"a b\"]"), s("spaced"));
    assert_eq!(ev("note[\"status\"]"), s("done"));
    t("status == \"done\"");
    assert_eq!(ev("count + 1"), Val::Num(4.0));
    assert_eq!(ev("price * 2"), Val::Num(5.0));
    t("flag");
    assert_eq!(ev("due"), Val::Date(TODAY - 1));
}

#[test]
fn test_bv_6_missing_key_is_null() {
    // [BV-6] 存在しないキー・formula は Null
    assert_eq!(ev("nokey"), Val::Null);
    assert_eq!(ev("note.nokey"), Val::Null);
    assert_eq!(ev("note[\"no key\"]"), Val::Null);
    assert_eq!(ev("formula.nope"), Val::Null);
    assert_eq!(ev("nothing"), Val::Null);
}

#[test]
fn test_bv_6_file_references() {
    // [BV-6] file.name ほか FileInfo の項目
    assert_eq!(ev("file.name"), s("a b.md"));
    assert_eq!(ev("file.basename"), s("a b"));
    assert_eq!(ev("file.ext"), s("md"));
    assert_eq!(ev("file.path"), s("notes/sub/a b.md"));
    assert_eq!(ev("file.folder"), s("notes/sub"));
    assert_eq!(ev("file.size"), Val::Num(123.0));
    t("file.tags.contains(\"proj\")");
    // mtime・ctime は日時。今より前で、mtime は ctime より後。
    t("file.mtime < now()");
    t("file.ctime < file.mtime");
}

#[test]
fn test_bv_6_formula_reference() {
    // [BV-6] formula.x
    assert_eq!(ev("formula.x"), Val::Num(42.0));
    assert_eq!(ev("formula.x + 1"), Val::Num(43.0));
    assert_eq!(ev("formula.label"), s("hi"));
}

// ---- BV-6: 関数 ----

#[test]
fn test_bv_6_if() {
    // [BV-6] if(c, a, b?): 真なら a、偽なら b、b が無ければ Null
    assert_eq!(ev("if(1 > 2, \"a\", \"b\")"), s("b"));
    assert_eq!(ev("if(status == \"done\", \"yes\", \"no\")"), s("yes"));
    assert_eq!(ev("if(false, \"a\")"), Val::Null);
    assert_eq!(ev("if(true, 1)"), Val::Num(1.0));
}

#[test]
fn test_bv_6_date_today_now() {
    // [BV-6] date("2026-09-30")・today()・now()
    assert_eq!(ev("date(\"2026-09-30\")"), Val::Date(TODAY - 1));
    assert_eq!(ev("today()"), Val::Date(TODAY));
    assert_eq!(ev("now()"), Val::DateTime(NOW));
}

#[test]
fn test_bv_6_date_plus_duration() {
    // [BV-6] 日付 + 期間の文字列("1d"・"2 weeks")と、日付 − 期間
    assert_eq!(ev("date(\"2026-09-30\") + \"1d\""), Val::Date(TODAY));
    assert_eq!(ev("today() + \"2 weeks\""), Val::Date(TODAY + 14));
    assert_eq!(ev("today() - \"1d\""), Val::Date(TODAY - 1));
    assert_eq!(ev("today() + \"1w\""), Val::Date(TODAY + 7));
    assert_eq!(ev("today() + \"3 days\""), Val::Date(TODAY + 3));
    // 日時 + 時間
    assert_eq!(ev("now() + \"1h\""), Val::DateTime(NOW + 3600));
    // due は `YYYY-MM-DD` の文字列なので日付として足せる
    assert_eq!(ev("due + \"1d\""), Val::Date(TODAY));
    // duration(s)
    assert_eq!(ev("duration(\"1d\")"), Val::Duration(DAY_MS));
}

#[test]
fn test_bv_6_date_minus_date_is_millis() {
    // [BV-6] 日付 − 日付 = 期間(ミリ秒)
    assert_eq!(
        ev("date(\"2026-10-01\") - date(\"2026-09-30\")"),
        Val::Duration(DAY_MS)
    );
    assert_eq!(ev("today() - due"), Val::Duration(DAY_MS));
    assert_eq!(ev("date(\"2026-09-30\") - today()"), Val::Duration(-DAY_MS));
}

#[test]
fn test_bv_6_date_comparison() {
    // [BV-6] 日付の比較
    t("date(\"2026-09-30\") < today()");
    t("due < today()");
    f("due > today()");
    t("due == date(\"2026-09-30\")");
    t("today() + \"1d\" > today()");
    t("today() <= today()");
}

#[test]
fn test_bv_6_has_tag() {
    // [BV-6] file.hasTag: いずれかのタグがあれば true。入れ子 `a/b` は親 `a` でも当たる(Obsidian のヘルプ)。
    t("file.hasTag(\"proj\")");
    t("file.hasTag(\"a\")");
    t("file.hasTag(\"a/b\")");
    f("file.hasTag(\"b\")");
    f("file.hasTag(\"pro\")");
    t("file.hasTag(\"nope\", \"proj\")");
    f("file.hasTag(\"nope\", \"other\")");
}

#[test]
fn test_bv_6_in_folder() {
    // [BV-6] file.inFolder: そのフォルダか、その下のフォルダにあれば true
    t("file.inFolder(\"notes\")");
    t("file.inFolder(\"notes/sub\")");
    f("file.inFolder(\"note\")");
    f("file.inFolder(\"other\")");
}

#[test]
fn test_bv_6_has_property() {
    // [BV-6] file.hasProperty
    t("file.hasProperty(\"status\")");
    t("file.hasProperty(\"a b\")");
    f("file.hasProperty(\"nokey\")");
}

#[test]
fn test_bv_6_string_methods() {
    // [BV-6] 文字列の .contains・.isEmpty・.length
    t("\"hello\".contains(\"ell\")");
    f("\"hello\".contains(\"xyz\")");
    t("status.contains(\"on\")");
    t("\"\".isEmpty()");
    f("\"a\".isEmpty()");
    t("empty.isEmpty()");
    assert_eq!(ev("\"hello\".length"), Val::Num(5.0));
    assert_eq!(ev("status.length"), Val::Num(4.0));
    assert_eq!(ev("(1 + 2).toString()"), s("3"));
}

#[test]
fn test_bv_6_list_methods() {
    // [BV-6] リストの .contains・.containsAll・.containsAny・.isEmpty・.length
    t("list.contains(\"x\")");
    f("list.contains(\"w\")");
    t("list.containsAll(\"x\", \"z\")");
    f("list.containsAll(\"x\", \"w\")");
    t("list.containsAny(\"w\", \"y\")");
    f("list.containsAny(\"v\", \"w\")");
    f("list.isEmpty()");
    t("[].isEmpty()");
    assert_eq!(ev("list.length"), Val::Num(3.0));
    assert_eq!(ev("[1, 2].length"), Val::Num(2.0));
    t("[1, 2, 3].contains(2)");
}

#[test]
fn test_bv_6_type_mismatch_is_null() {
    // [BV-6] 型の合わない演算は Null
    assert_eq!(ev("\"abc\" * 2"), Val::Null);
    assert_eq!(ev("\"abc\" - 1"), Val::Null);
    assert_eq!(ev("[1] / 2"), Val::Null);
    assert_eq!(ev("nokey + 1"), Val::Null);
    assert_eq!(ev("true * 3"), Val::Null);
}

#[test]
fn test_bv_6_to_val() {
    // [BV-6] フロントマターの値 → Val。`YYYY-MM-DD` の文字列は Date。
    assert_eq!(
        to_val(&Value::Str("2026-10-01".to_string())),
        Val::Date(TODAY)
    );
    assert_eq!(to_val(&Value::Str("1970-01-01".to_string())), Val::Date(0));
    assert_eq!(to_val(&Value::Str("done".to_string())), s("done"));
    assert_eq!(to_val(&Value::Int(3)), Val::Num(3.0));
    assert_eq!(to_val(&Value::Float(2.5)), Val::Num(2.5));
    assert_eq!(to_val(&Value::Bool(false)), Val::Bool(false));
    assert_eq!(to_val(&Value::Null), Val::Null);
    assert_eq!(
        to_val(&Value::List(vec![
            Value::Str("a".to_string()),
            Value::Int(1)
        ])),
        Val::List(vec![s("a"), Val::Num(1.0)])
    );
}

// ---- BV-7: 未対応と文法の誤り ----

fn unsupported_with(src: &str, name: &str) {
    match try_parse(src) {
        Err(ExprError::Unsupported(m)) => {
            assert!(
                m.contains(name),
                "{src:?}: Unsupported({m:?}) に {name:?} が無い"
            )
        }
        Err(ExprError::Syntax(m)) => panic!("{src:?} が Unsupported でなく Syntax({m:?})"),
        Ok(_) => panic!("{src:?} が parse できてしまった(Unsupported のはず)"),
    }
}

fn syntax_error(src: &str) {
    match try_parse(src) {
        Err(ExprError::Syntax(_)) => {}
        Err(ExprError::Unsupported(m)) => panic!("{src:?} が Syntax でなく Unsupported({m:?})"),
        Ok(_) => panic!("{src:?} が parse できてしまった(Syntax のはず)"),
    }
}

#[test]
fn test_bv_7_unknown_function_is_unsupported() {
    // [BV-7] 知らない関数・メソッドは parse で Unsupported、名前を含む
    unsupported_with("foo(1)", "foo");
    unsupported_with("list.map(value + 1)", "map");
    unsupported_with("status == \"a\" && bar(status)", "bar");
    unsupported_with("file.name.frobnicate()", "frobnicate");
}

#[test]
fn test_bv_7_uncomputable_file_property_is_unsupported() {
    // [BV-7] Obsidian にはあるが mdgrid が計算できない file の項目(バックリンク)は、推測で値を作らず未対応と出す
    unsupported_with("file.embeds.length", "embeds");
}

#[test]
fn test_bv_7_syntax_errors() {
    // [BV-7] 文法の誤りは Syntax
    syntax_error("1 +");
    syntax_error("(1 + 2");
    syntax_error("\"abc");
    syntax_error("status ==");
    syntax_error("1 2");
    syntax_error("if(true, ");
}
