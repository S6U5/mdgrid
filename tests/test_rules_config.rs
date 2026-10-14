//! 列の決まりの設定 `[rules.<列>]`(CE-36)の読み取り・範囲の重ね・警告と、問題の判定 `problem`、
//! 理由の文(CV-9)の受け入れ試験。
//! 記録: specs/_changes/2026-10-14-type-check.md(タスク #1)。実装を見ずに、記録の「設計」と公開の口だけから書いた。

use mdgrid::config::parse;
use mdgrid::frontmatter::Value;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::profile::{self, Layer, Origin, Place, Profile};
use mdgrid::rules::{problem, Reason, Rule};
use mdgrid::types::Kind;

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

fn strings(xs: &[&str]) -> Option<Vec<String>> {
    Some(xs.iter().map(|x| x.to_string()).collect())
}

fn rule(required: bool, values: &[&str]) -> Rule {
    Rule {
        required,
        values: if values.is_empty() {
            None
        } else {
            strings(values)
        },
    }
}

#[test]
fn test_ce_36_rules_read_without_warnings() {
    // [CE-36] `required` と `values` を読み、決まった値の rules に列ごとに入る(警告なし)。
    let text = "[rules.status]\nrequired = true\nvalues = [\"todo\", \"doing\", \"done\"]\n\n[rules.owner]\nrequired = true\n";
    let (c, w) = parse(text).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let r = c.resolved();
    let st = r.rules.get("status").expect("status の決まり");
    assert!(st.required);
    assert_eq!(st.values, strings(&["todo", "doing", "done"]));
    let ow = r.rules.get("owner").expect("owner の決まり");
    assert!(ow.required);
    assert_eq!(ow.values, None);
}

#[test]
fn test_ce_36_narrow_scope_overrides_per_item() {
    // [CE-36] 範囲ごとに項目ごとに重ね、狭い範囲が勝つ(SR-44)。表の範囲の values が全体の values に代わり、
    // 表の範囲に書いていない required は全体のものが残る。表では doing も違反になる。
    let (c, w) =
        parse("[rules.status]\nrequired = true\nvalues = [\"todo\", \"doing\", \"done\"]\n")
            .expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let t: toml::Table = "[rules.status]\nvalues = [\"todo\", \"done\"]\n"
        .parse()
        .unwrap();
    let mut w = Vec::new();
    let table = Profile::read_scoped(&t, "views.toml", "", &[], &mut w);
    assert!(w.is_empty(), "{w:?}");
    let layers = [
        c.layer(),
        Layer {
            origin: Origin::new(Place::TableApp, "views.toml (notes)"),
            profile: table,
        },
    ];
    let r = profile::resolve(&layers, &c.templates, &mut w);
    assert!(w.is_empty(), "{w:?}");
    let st = r.rules.get("status").expect("status の決まり");
    assert!(st.required, "全体の required が残る");
    assert_eq!(st.values, strings(&["todo", "done"]));
    assert_eq!(
        problem(Kind::Text, Some(st), Some(&s("doing"))),
        Some(Reason::NotAllowed)
    );
    // 全体だけなら doing は違反でない。
    let wide = c.resolved();
    assert_eq!(
        problem(Kind::Text, wide.rules.get("status"), Some(&s("doing"))),
        None
    );
}

#[test]
fn test_ce_36_file_and_formula_columns_warn_and_are_unused() {
    // [CE-36] `file.*`・`formula.*` の列の決まりは警告して使わない。
    let (c, w) = parse(
        "[rules.\"file.name\"]\nrequired = true\n\n[rules.\"formula.x\"]\nvalues = [\"a\"]\n",
    )
    .expect("止めない");
    assert!(
        w.iter().any(|x| x.contains("rules.file.name")),
        "file.name の警告: {w:?}"
    );
    assert!(
        w.iter().any(|x| x.contains("rules.formula.x")),
        "formula.x の警告: {w:?}"
    );
    let r = c.resolved();
    assert!(!r.rules.contains_key("file.name"), "{:?}", r.rules.keys());
    assert!(!r.rules.contains_key("formula.x"), "{:?}", r.rules.keys());
}

#[test]
fn test_ce_36_wrong_value_types_warn_and_item_is_unused() {
    // [CE-36] 型の違う値(required が文字・values が配列でない)は警告し、その項目を使わない。
    // ほかの項目はそのまま使う。
    let (c, w) = parse(
        "[rules.status]\nrequired = \"yes\"\nvalues = [\"a\"]\n\n[rules.owner]\nrequired = true\nvalues = \"todo\"\n",
    )
    .expect("止めない");
    assert!(
        w.iter().any(|x| x.contains("rules.status.required")),
        "required の警告: {w:?}"
    );
    assert!(
        w.iter().any(|x| x.contains("rules.owner.values")),
        "values の警告: {w:?}"
    );
    let r = c.resolved();
    let st = r.rules.get("status").expect("values は使う");
    assert!(!st.required);
    assert_eq!(st.values, strings(&["a"]));
    let ow = r.rules.get("owner").expect("required は使う");
    assert!(ow.required);
    assert_eq!(ow.values, None);
}

#[test]
fn test_ce_36_unknown_item_and_non_string_values_warn() {
    // [CE-36] 知らない項目は警告する。values に文字でない要素があれば警告する。
    let (_, w) = parse("[rules.status]\nfoo = 1\n").expect("止めない");
    assert!(
        w.iter().any(|x| x.contains("rules.status.foo")),
        "知らない項目の警告: {w:?}"
    );
    let (_, w) = parse("[rules.status]\nvalues = [1, 2]\n").expect("止めない");
    assert!(
        w.iter().any(|x| x.contains("rules.status.values")),
        "文字でない要素の警告: {w:?}"
    );
}

#[test]
fn test_ce_36_required_violations() {
    // [CE-36] required: キーが無い・null・空の文字列・空のリストは違反、値があれば違反でない。
    let r = rule(true, &[]);
    assert_eq!(problem(Kind::Text, Some(&r), None), Some(Reason::Required));
    assert_eq!(
        problem(Kind::Text, Some(&r), Some(&Value::Null)),
        Some(Reason::Required)
    );
    assert_eq!(
        problem(Kind::Text, Some(&r), Some(&s(""))),
        Some(Reason::Required)
    );
    assert_eq!(
        problem(Kind::List, Some(&r), Some(&Value::List(vec![]))),
        Some(Reason::Required)
    );
    assert_eq!(problem(Kind::Text, Some(&r), Some(&s("x"))), None);
    // 必須でなければ、キーが無くても違反でない。
    let free = Rule::default();
    assert_eq!(problem(Kind::Text, Some(&free), None), None);
    assert_eq!(problem(Kind::Text, None, None), None);
}

#[test]
fn test_ce_36_values_compare_text_case_sensitive() {
    // [CE-36] values: 値の文字がどれかでなければ違反。大文字小文字を区別する。キーが無いのは values の違反でない。
    let r = rule(false, &["todo", "doing", "done"]);
    assert_eq!(problem(Kind::Text, Some(&r), Some(&s("done"))), None);
    assert_eq!(
        problem(Kind::Text, Some(&r), Some(&s("wip"))),
        Some(Reason::NotAllowed)
    );
    assert_eq!(
        problem(Kind::Text, Some(&r), Some(&s("Done"))),
        Some(Reason::NotAllowed)
    );
    assert_eq!(problem(Kind::Text, Some(&r), None), None);
}

#[test]
fn test_ce_36_values_check_each_list_item() {
    // [CE-36] リストの値は各要素の文字を values と比べ、1つでも外れれば違反。
    let r = rule(false, &["a", "b"]);
    let ok = Value::List(vec![s("a"), s("b")]);
    let bad = Value::List(vec![s("a"), s("c")]);
    assert_eq!(problem(Kind::List, Some(&r), Some(&ok)), None);
    assert_eq!(
        problem(Kind::List, Some(&r), Some(&bad)),
        Some(Reason::NotAllowed)
    );
}

#[test]
fn test_ce_36_values_compare_numbers_and_bools_as_text() {
    // [CE-36] 数は `3` のように、真偽は `true`・`false` の文字で values と比べる。
    let n = rule(false, &["1", "3"]);
    assert_eq!(problem(Kind::Number, Some(&n), Some(&Value::Int(3))), None);
    assert_eq!(
        problem(Kind::Number, Some(&n), Some(&Value::Int(4))),
        Some(Reason::NotAllowed)
    );
    let b = rule(false, &["true"]);
    assert_eq!(
        problem(Kind::Checkbox, Some(&b), Some(&Value::Bool(true))),
        None
    );
    assert_eq!(
        problem(Kind::Checkbox, Some(&b), Some(&Value::Bool(false))),
        Some(Reason::NotAllowed)
    );
}

#[test]
fn test_ce_36_type_mismatch_comes_first() {
    // [CE-36] 型が合わない値は、決まりにも外れていても理由は型(CV-2 の印と同じ判定)。決まりが無くても型を見る。
    let r = rule(true, &["todo"]);
    assert_eq!(
        problem(Kind::Date, Some(&r), Some(&s("someday"))),
        Some(Reason::Type(Kind::Date))
    );
    assert_eq!(
        problem(Kind::Number, Some(&r), Some(&s("abc"))),
        Some(Reason::Type(Kind::Number))
    );
    assert_eq!(
        problem(Kind::Date, None, Some(&s("someday"))),
        Some(Reason::Type(Kind::Date))
    );
    assert_eq!(problem(Kind::Date, None, Some(&s("2026-10-01"))), None);
}

#[test]
fn test_cv_9_reason_texts_in_japanese() {
    // [CV-9] 理由を「型が合わない(〇〇ではない)」「必須のキーが無い」「選べる値ではない」の3つで言い分ける。
    let _g = scoped(Lang::Ja);
    let names = [
        (Kind::Text, "テキスト"),
        (Kind::Number, "数"),
        (Kind::Checkbox, "チェック"),
        (Kind::Date, "日付"),
        (Kind::DateTime, "日時"),
        (Kind::List, "リスト"),
    ];
    for (k, name) in names {
        assert_eq!(
            Reason::Type(k).text(),
            format!("型が合わない({name}ではない)")
        );
    }
    assert_eq!(Reason::Required.text(), "必須のキーが無い");
    assert_eq!(Reason::NotAllowed.text(), "選べる値ではない");
}

#[test]
fn test_cv_9_reason_texts_follow_language() {
    // [CV-9] 理由の文は今の言語で出る(英語では日本語の字を含まず、3つが互いに違う)。
    let _g = scoped(Lang::En);
    let texts = [
        Reason::Type(Kind::Date).text(),
        Reason::Required.text(),
        Reason::NotAllowed.text(),
    ];
    for t in &texts {
        assert!(!t.is_empty());
        assert!(
            !t.chars().any(|c| ('\u{3000}'..='\u{9fff}').contains(&c)
                || ('\u{ff00}'..='\u{ffef}').contains(&c)),
            "日本語の字: {t}"
        );
    }
    assert_ne!(texts[0], texts[1]);
    assert_ne!(texts[1], texts[2]);
    assert_ne!(texts[0], texts[2]);
}
