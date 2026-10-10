use super::*;
use crate::expr::{self, Env, Val};
use crate::settings;
use crate::source::FileInfo;

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

/// 条件の式を、列 `col` の値 v で評価して真か(式を作らない条件は真)。
fn eval_cond(c: &Cond, kind: Option<Kind>, v: Option<&Value>) -> bool {
    let src = cond_expr(c, kind).0.unwrap_or_else(|| "true".to_string());
    let e = expr::parse(&src).unwrap_or_else(|e| panic!("parse {src}: {e:?}"));
    let f = file();
    let col = c.col.clone();
    let prop = |k: &str| if k == col { v.cloned() } else { None };
    let formula = |_: &str| None;
    let env = Env {
        prop: &prop,
        file: &f,
        formula: &formula,
        today: 0,
        now: 0,
    };
    matches!(expr::eval(&e, &env), Val::Bool(true))
}

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

fn list(xs: &[Value]) -> Value {
    Value::List(xs.to_vec())
}

/// 式と settings が同じになるはずの材料: 空・文字列・数・真偽・日付・リスト。
fn clean() -> Vec<Option<Value>> {
    vec![
        None,
        Some(Value::Null),
        Some(s("")),
        Some(s("done")),
        Some(s("todo")),
        Some(s("Delta REPORT")),
        Some(s("2026-02-01")),
        Some(s("2026-03-20")),
        Some(s("someday")),
        Some(s(" ")),
        Some(s(", ")),
        Some(Value::Int(3)),
        Some(Value::Int(10)),
        Some(Value::Float(2.5)),
        Some(Value::Bool(true)),
        Some(Value::Bool(false)),
        Some(list(&[])),
        Some(list(&[Value::Null])),
        Some(list(&[Value::Null, Value::Null])),
        Some(list(&[s(""), s("")])),
        Some(list(&[s(""), s(" ")])),
        Some(list(&[s("done")])),
        Some(list(&[s("a"), s("done")])),
        Some(list(&[s("会議"), s("todo")])),
        Some(list(&[Value::Int(3), s("x")])),
        Some(list(&[Value::Bool(true), Value::Null])),
        Some(list(&[s("2026-02-01"), s("x")])),
    ]
}

/// 式で同じにできない形(日時の書き方・前後の空白・マップ・入れ子のリスト)を含む材料。
fn odd() -> Vec<Option<Value>> {
    vec![
        Some(s("2026-02-01T10:00")),
        Some(s("2026-02-01T10:00:00")),
        Some(s("2026-02-01 10:00")),
        Some(s(" 2026-02-01 ")),
        Some(Value::Other),
        Some(list(&[list(&[])])),
        Some(list(&[Value::Other, s("a")])),
        Some(list(&[s("2026-02-01T10:00")])),
    ]
}

fn cond(col: &str, op: Op) -> Cond {
    Cond {
        col: col.to_string(),
        op,
    }
}

fn some(v: &[&str]) -> Vec<Option<String>> {
    v.iter().map(|s| Some(s.to_string())).collect()
}

/// 式で評価した結果が settings::matches と同じ値の数と、違った値(と注意が出ているか)。
fn compare(c: &Cond, kind: Kind, typed: bool, vals: &[Option<Value>]) -> Vec<Option<Value>> {
    let k = typed.then_some(kind);
    vals.iter()
        .filter(|v| settings::matches(c, v.as_ref(), kind) != eval_cond(c, k, v.as_ref()))
        .cloned()
        .collect()
}

/// strict の材料では同じ(近似の注意が出る条件は、リストでない値で同じ)。odd の材料では、違うなら注意が出ている。
fn assert_same(c: &Cond, kind: Kind, typed: bool, strict: &[Option<Value>]) {
    let k = typed.then_some(kind);
    let (e, notes) = cond_expr(c, k);
    let strict: Vec<Option<Value>> = strict
        .iter()
        .filter(|v| notes.is_empty() || !matches!(v, Some(Value::List(_))))
        .cloned()
        .collect();
    let bad = compare(c, kind, typed, &strict);
    assert!(
        bad.is_empty(),
        "{c:?}({kind:?}, 型 {typed}): 値 {bad:?} で行がずれる。式 {e:?}"
    );
    let bad = compare(c, kind, typed, &odd());
    assert!(
        bad.is_empty() || !notes.is_empty(),
        "{c:?}({kind:?}, 型 {typed}): 値 {bad:?} で行がずれるのに注意が無い。式 {e:?}"
    );
}

const OPS: [CmpOp; 6] = [
    CmpOp::Eq,
    CmpOp::Ne,
    CmpOp::Lt,
    CmpOp::Le,
    CmpOp::Gt,
    CmpOp::Ge,
];

#[test]
fn test_bv_19_keep_drop_match_settings() {
    // [BV-19] [NV-14] [NV-19] Keep・Drop(「(空)」・リストの要素・数・真偽・日付)を式にしても settings と同じ行。
    let all = clean();
    for kind in [Kind::Text, Kind::List, Kind::Checkbox, Kind::DateTime] {
        for keys in [
            some(&["done"]),
            some(&["todo", "done"]),
            vec![None],
            vec![None, Some("done".to_string())],
            some(&["3"]),
            some(&["2.5", "true"]),
            some(&["false"]),
            some(&["会議"]),
            some(&["a, done"]),
            some(&["2026-02-01"]),
            some(&["2026-02-01T10:00"]),
            some(&["{…}"]),
            Vec::new(),
        ] {
            for typed in [false, true] {
                assert_same(&cond("status", Op::Keep(keys.clone())), kind, typed, &all);
                assert_same(&cond("status", Op::Drop(keys.clone())), kind, typed, &all);
            }
        }
    }
}

#[test]
fn test_bv_19_contains_and_empty_match_settings() {
    // [BV-19] [NV-19] 含む・含まない(大文字小文字を区別しない)と、空である・空でない。
    // 入れ子のリストに中身がある値([["a"]])の Contains は式で同じにできない(試験の材料に入れない)。
    let all = clean();
    for needle in [
        "report", "REPORT", "do", "会", "", "2026", "t10:00", "a, d", " do", "{…}",
    ] {
        for kind in [Kind::Text, Kind::List, Kind::DateTime] {
            assert_same(
                &cond("title", Op::Contains(needle.to_string())),
                kind,
                true,
                &all,
            );
            assert_same(
                &cond("title", Op::NotContains(needle.to_string())),
                kind,
                true,
                &all,
            );
        }
    }
    for kind in [Kind::Text, Kind::List, Kind::Number] {
        assert_same(&cond("title", Op::Empty), kind, true, &all);
        assert_same(&cond("title", Op::NotEmpty), kind, true, &all);
    }
}

#[test]
fn test_bv_19_cmp_match_settings() {
    // [BV-19] [NV-19] 比較: 列の型を渡せば、数・日付・日時・テキスト・真偽で settings と同じ。
    // 型を渡さないときは、比べる値の形と列の型が合う(数の列に数、日付の列に日付)なら同じ。
    let scalars: Vec<Option<Value>> = clean()
        .into_iter()
        .filter(|v| !matches!(v, Some(Value::List(_))))
        .collect();
    let all = clean();
    for op in OPS {
        let cases = [
            ("3", Kind::Number),
            (" 2.5 ", Kind::Number),
            ("inf", Kind::Number),
            ("-inf", Kind::Number),
            ("abc", Kind::Number),
            ("2026-02-01", Kind::Date),
            ("2026-02-01T10:00", Kind::Date),
            ("2026-02-01T09:00", Kind::DateTime),
            ("2026-02-01", Kind::DateTime),
            ("m", Kind::Text),
            ("3", Kind::Text),
            ("2026", Kind::Text),
            ("true", Kind::Checkbox),
            ("m", Kind::List),
        ];
        for (rhs, kind) in cases {
            let c = cond("x", Op::Cmp(op, rhs.to_string()));
            assert_same(&c, kind, true, &scalars);
            // リストの値: 違うなら注意が出ている。
            let bad = compare(&c, kind, true, &all);
            assert!(
                bad.is_empty() || !cond_expr(&c, Some(kind)).1.is_empty(),
                "{c:?} {kind:?}: {bad:?}"
            );
        }
        for (rhs, kind) in [
            ("3", Kind::Number),
            ("2026-02-01", Kind::Date),
            ("2026-02-01T09:00", Kind::DateTime),
        ] {
            let c = cond("x", Op::Cmp(op, rhs.to_string()));
            let bad = compare(&c, kind, false, &scalars);
            assert!(bad.is_empty(), "型なし {c:?} {kind:?}: {bad:?}");
        }
        // 型なしでテキストの値の比較は、型が分からないと注意する。
        let (_, notes) = cond_expr(&cond("x", Op::Cmp(op, "m".to_string())), None);
        assert!(notes.iter().any(|n| n.contains("型")), "{notes:?}");
    }
}

#[test]
fn test_bv_19_review_cases_are_fixed_or_noted() {
    // [BV-19] 日時の列の Keep は同じ行になる。Contains の日時の書き方と、マップ・入れ子のリストの空は注意が出る。
    let keep = cond("at", Op::Keep(some(&["2026-02-01T10:00"])));
    let v = s("2026-02-01T10:00");
    assert!(settings::matches(&keep, Some(&v), Kind::DateTime));
    assert!(eval_cond(&keep, Some(Kind::DateTime), Some(&v)));
    assert!(!cond_expr(&keep, None).1.is_empty());

    let c = cond("at", Op::Contains("t10:00".to_string()));
    assert!(!cond_expr(&c, None).1.is_empty(), "日時の書き方の注意");

    let empty = cond("x", Op::Empty);
    let not_empty = cond("x", Op::NotEmpty);
    let nn = list(&[Value::Null, Value::Null]);
    let ee = list(&[s(""), s("")]);
    assert!(eval_cond(&empty, None, Some(&nn)));
    assert!(!eval_cond(&not_empty, None, Some(&ee)));
    // マップと [[]] は式で同じにできないので注意が出る。
    assert!(settings::matches(
        &not_empty,
        Some(&Value::Other),
        Kind::Text
    ));
    assert!(!eval_cond(&not_empty, None, Some(&Value::Other)));
    assert!(!cond_expr(&not_empty, None).1.is_empty());
    assert!(!cond_expr(&empty, None).1.is_empty());
}

#[test]
fn test_bv_19_to_base_typed_and_notes() {
    // [BV-19] 型を渡すと日付の列の比較は日付だけ。近似の注意と formula の参照は落とした説明に出る。
    let view = NativeView {
        name: "v".to_string(),
        order: vec!["file.name".to_string(), "formula.f".to_string()],
        hidden: Vec::new(),
        filters_expr: vec!["formula.g > 1".to_string()],
        settings: Settings {
            filters: vec![
                cond("due", Op::Cmp(CmpOp::Lt, "2026-03-01".to_string())),
                cond("title", Op::Empty),
            ],
            sorts: vec![("formula.h".to_string(), Dir::Asc)],
            group: Group::Inherit,
            ..Default::default()
        },
        new_note: None,
    };
    let (yaml, dropped) = to_base_typed(&view, &|c| (c == "due").then_some(Kind::Date));
    base::parse(&yaml).unwrap_or_else(|e| panic!("{e}\n{yaml}"));
    let all = dropped.join("\n");
    assert!(all.contains("formula.f"), "{all}");
    assert!(all.contains("formula.g"), "{all}");
    assert!(all.contains("formula.h"), "{all}");
    assert!(all.contains("title"), "{all}");
    assert!(yaml.contains("length == 10"), "{yaml}");
    let (_, plain) = to_base(&NativeView {
        filters_expr: Vec::new(),
        order: Vec::new(),
        settings: Settings {
            sorts: Vec::new(),
            ..view.settings.clone()
        },
        ..view.clone()
    });
    assert!(!plain.join("\n").contains("formula"), "{plain:?}");
}

#[test]
fn test_bv_19_column_names_are_quoted() {
    // [BV-19] 記号・日本語・予約語の列の名前は note["…"] で参照する。
    assert_eq!(col_ref("status"), "status");
    assert_eq!(col_ref("file.name"), "file.name");
    assert_eq!(col_ref("期限"), r#"note["期限"]"#);
    assert_eq!(col_ref("my key"), r#"note["my key"]"#);
    assert_eq!(col_ref("date"), r#"note["date"]"#);
    assert_eq!(col_ref(r#"a"b"#), r#"note["a\"b"]"#);
    let c = cond("my \"key\"", Op::Keep(some(&["x\\y"])));
    assert!(eval_cond(&c, None, Some(&s("x\\y"))));
    assert!(!eval_cond(&c, None, Some(&s("x"))));
}

#[test]
fn test_bv_19_to_base_quotes_strings_for_yaml() {
    // [BV-19] 名前・式・列の引用符・改行・コロンは YAML の引用符で囲み、base::parse で同じに戻る。
    let view = NativeView {
        name: "it's: \"x\"\nnext".to_string(),
        order: vec!["file.name".to_string(), "a: b".to_string()],
        hidden: Vec::new(),
        filters_expr: vec![r#"status != "it's""#.to_string()],
        settings: Settings {
            filters: vec![cond("title", Op::Contains("#x: 'y'".to_string()))],
            sorts: vec![("a: b".to_string(), Dir::Desc)],
            group: Group::Off,
            ..Default::default()
        },
        new_note: None,
    };
    let (yaml, dropped) = to_base(&view);
    assert!(dropped.is_empty(), "{dropped:?}");
    let b = base::parse(&yaml).unwrap_or_else(|e| panic!("{e}\n{yaml}"));
    assert_eq!(b.views[0].name, view.name, "{yaml}");
    assert_eq!(b.views[0].order, view.order);
    assert_eq!(b.views[0].sort, vec![("a: b".to_string(), base::Dir::Desc)]);
    let mut d = Vec::new();
    let exprs = b.views[0].filter_exprs(&mut d);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(exprs.len(), 2, "{exprs:?}");
    assert_eq!(exprs[0], view.filters_expr[0]);
    assert!(expr::parse(&exprs[1]).is_ok(), "{}", exprs[1]);
}

#[test]
fn test_bv_19_to_base_hidden_without_order_is_dropped() {
    // [BV-19] order が空で隠す列があるビューは、隠す列を落としたと知らせる。
    let view = NativeView {
        name: "x".to_string(),
        hidden: vec!["due".to_string()],
        ..NativeView::default()
    };
    let (yaml, dropped) = to_base(&view);
    assert_eq!(dropped.len(), 1, "{dropped:?}");
    assert!(dropped[0].contains("due"));
    assert!(base::parse(&yaml).is_ok());
}

#[test]
fn test_bv_19_from_base_nested_filters_and_extras() {
    // [BV-19] not・or の入れ子は式に、formulas・displayName・読めない形は落とした説明に出る。
    let text = r#"filters:
  not:
    - 'status == "done"'
    - 'status == "x"'
formulas:
  f: 'priority * 2'
properties:
  status:
    displayName: 状態
views:
  - type: cards
    name: c
    filters:
      or:
        - 'priority > 1'
        - and:
            - 'priority < 5'
            - 'file.hasTag("a")'
        - weird: 1
"#;
    let b = base::parse(text).expect("parse");
    let (nv, dropped) = from_base(&b, 0);
    assert_eq!(
        nv.filters_expr,
        vec![
            r#"!((status == "done") || (status == "x"))"#.to_string(),
            r#"(priority > 1) || ((priority < 5) && (file.hasTag("a")))"#.to_string(),
        ]
    );
    for e in &nv.filters_expr {
        assert!(expr::parse(e).is_ok(), "{e}");
    }
    let all = dropped.join("\n");
    assert!(all.contains("cards"), "{all}");
    assert!(all.contains("formulas"), "{all}");
    assert!(all.contains("displayName"), "{all}");
    assert!(all.contains("weird"), "{all}");
    let (_, d) = from_base(&b, 5);
    assert!(!d.is_empty());
}

struct Tmp(std::path::PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "mdgrid-views-unit-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn named(name: &str) -> NativeView {
    NativeView {
        name: name.to_string(),
        ..NativeView::default()
    }
}

#[test]
fn test_bv_20_save_refuses_broken_file() {
    // [BV-20] 壊れた views.toml には書かず、中身はそのまま。
    let t = Tmp::new("broken");
    let p = t.0.join(FILE_NAME);
    std::fs::write(&p, "[[target]\n").unwrap();
    assert!(save_views(&t.0, &t.0, &[named("x")]).is_err());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "[[target]\n");
}

#[test]
fn test_bv_20_save_keeps_other_targets_and_empty_removes_target() {
    // [BV-20] ほかの対象と最上位の知らない項目はそのまま。空のビューで書くと対象の表ごと消える。
    // 前の版の [[target]] は [[table]] として読み、書くときは [[table]](CLI-20)。
    let t = Tmp::new("keep");
    let notes = t.0.join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(
        t.0.join(FILE_NAME),
        "extra = 1\n[[target]]\npath = \"/somewhere/else\"\nnote = \"keep\"\n",
    )
    .unwrap();
    let v = named("x");
    save_views(&t.0, &notes, std::slice::from_ref(&v)).unwrap();
    let (back, warns) = load_views(&t.0, &notes);
    assert_eq!(back, vec![v]);
    assert!(warns.iter().any(|w| w.contains("extra")), "{warns:?}");
    assert!(warns.iter().any(|w| w.contains("table.note")), "{warns:?}");
    let text = std::fs::read_to_string(t.0.join(FILE_NAME)).unwrap();
    assert!(
        text.contains("[[table]]") && !text.contains("[[target]]"),
        "{text}"
    );
    save_views(&t.0, &notes, &[]).unwrap();
    let (back, _) = load_views(&t.0, &notes);
    assert!(back.is_empty());
    let text = std::fs::read_to_string(t.0.join(FILE_NAME)).unwrap();
    assert!(text.contains("/somewhere/else") && text.contains("keep") && text.contains("extra"));
    assert!(!text.contains(&config::target_key(&notes)), "{text}");
}

#[test]
fn test_bv_20_save_keeps_unknown_items_and_unreadable_views_of_own_target() {
    // [BV-20] 自分の対象の知らない項目(memo)、同じ名前のビューの知らない項目(color・settings.width)、
    // 読めなかったビュー(order が文字列)は保存しても残る。読めなかったビューは同じ名前で上書きされたら消える。
    let t = Tmp::new("own");
    let key = config::target_key(&t.0);
    let text = format!(
        r#"[[target]]
path = {}
memo = "対象のメモ"

[[target.view]]
name = "a"
color = 1
[target.view.settings]
width = 3

[[target.view]]
name = "壊れた"
order = "status"

[[target.view]]
name = "消す"
"#,
        str_lit(&key)
    );
    std::fs::write(t.0.join(FILE_NAME), text).unwrap();
    let (views, warns) = load_views(&t.0, &t.0);
    assert_eq!(
        views.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(),
        ["a", "消す"],
        "{warns:?}"
    );

    let mut a = named("a");
    a.order = vec!["title".to_string()];
    save_views(&t.0, &t.0, &[a.clone(), named("b")]).unwrap();
    let text = std::fs::read_to_string(t.0.join(FILE_NAME)).unwrap();
    assert!(text.contains("memo"), "{text}");
    assert!(text.contains("color"), "{text}");
    assert!(text.contains("width"), "{text}");
    assert!(text.contains("壊れた"), "{text}");
    assert!(!text.contains("消す"), "{text}");
    let (views, warns) = load_views(&t.0, &t.0);
    assert_eq!(views, vec![a, named("b")]);
    assert!(warns.iter().any(|w| w.contains("読めない")), "{warns:?}");
    assert!(warns.iter().any(|w| w.contains("memo")), "{warns:?}");

    // 同じ名前で上書きすると、読めなかったビューは消える。
    save_views(&t.0, &t.0, &[named("壊れた")]).unwrap();
    let (views, warns) = load_views(&t.0, &t.0);
    assert_eq!(views, vec![named("壊れた")]);
    assert!(!warns.iter().any(|w| w.contains("読めない")), "{warns:?}");
}

#[test]
fn test_bv_20_unknown_nested_settings_key_warns() {
    // [BV-20] [CLI-3] settings の中の知らない項目も警告にして、ビューは読む。
    let t = Tmp::new("nested");
    let key = config::target_key(&t.0);
    let text = format!(
        "[[target]]\npath = {}\n[[target.view]]\nname = \"v\"\n[target.view.settings]\nsorts = [[\"due\", \"Asc\"]]\nwidth = 3\n",
        str_lit(&key)
    );
    std::fs::write(t.0.join(FILE_NAME), text).unwrap();
    let (views, warns) = load_views(&t.0, &t.0);
    assert_eq!(views.len(), 1, "{warns:?}");
    assert_eq!(views[0].settings.sorts, vec![("due".to_string(), Dir::Asc)]);
    assert!(warns.iter().any(|w| w.contains("width")), "{warns:?}");
}
