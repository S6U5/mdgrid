//! タスク 1(bases-docs)の受け入れ: `.base` の対応範囲の文書(BV-21)と、式の評価が受け付ける
//! 関数・メソッド・`file.*` の一覧の突き合わせ。仕様: specs/base-view/spec.md の BV-21(と BV-7)。
//! 設計: specs/_changes/2026-10-03-bases-docs.md。実装(src/expr.rs の中身)を見ずに書いた。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/expr.rs
//! pub const FUNCTIONS: &[&str];   // トップの関数の名前(`if`・`date`・`now`・`today`・`duration` など)
//! pub const METHODS: &[&str];     // `x.contains(...)` などのメソッドの名前(`contains` など。`.` も `()` も付けない)
//! pub const FILE_FIELDS: &[&str]; // `file.name` などの名前(`name` など。`file.` を付けない)
//! pub fn parse(src: &str) -> Result<Expr, ExprError>; // 未対応の名前は Err(ExprError::Unsupported(名前を含む))
//! ```
//!
//! `file.hasTag(...)` のように `file` に付く呼び出しを METHODS と FILE_FIELDS のどちらに置くか、
//! `.length` のように括弧の無いものを METHODS に置くかは決めない(どちらに置いても通るように確かめる)。
//!
//! 仮定した文書の形: docs/obsidian-bases.md(英語)と docs/obsidian-bases.ja.md(日本語)。
//!
//! - 一覧の節(見出しは次の文字列とちょうど同じ `## ` の行):
//!   - 英: `## Functions`・`## Methods`・`## File properties`
//!   - 日: `## 関数`・`## メソッド`・`## file のプロパティ`
//!
//!   節は見出しの次の行から、次の `# `・`## ` の見出しの手前まで(`### ` は節の中)。節の中で、
//!   箇条書きの行(`- `・`* ` で始まる)と表の行(`|` で始まる)のうち、コードスパン(`` `...` ``)を持つ
//!   行を1つの項目とし、その行の最初のコードスパンを名前とする。コードブロック(```)の中と、
//!   ふつうの文の段落は数えない。名前は `(` より前を取り、最後の `.` より後を取る
//!   (`if`・`if()`・`if(c, a, b)`・`.contains()`・`x.contains(v)`・`file.name`・`name` はどれも受ける)。
//! - ほかに次の見出しの節がある(中身は試験しない):
//!   - 英: `## Keys read`・`## View types`・`## Operators`・`## Not supported`
//!   - 日: `## 読む項目`・`## ビューの型`・`## 演算子`・`## 解釈しないもの`
//! - 英の文書は `](obsidian-bases.ja.md)` で日本語へ、日の文書は `](obsidian-bases.md)` で英語へリンクする
//!   (`./` を前に付けてもよい)。

use mdgrid::expr::{parse, ExprError, FILE_FIELDS, FUNCTIONS, METHODS};
use std::collections::BTreeSet;
use std::path::PathBuf;

const EN: &str = "docs/obsidian-bases.md";
const JA: &str = "docs/obsidian-bases.ja.md";

/// 一覧の節の見出し(英, 日)。
const SEC_FUNCTIONS: (&str, &str) = ("Functions", "関数");
const SEC_METHODS: (&str, &str) = ("Methods", "メソッド");
const SEC_FILE: (&str, &str) = ("File properties", "file のプロパティ");

/// 中身は試験しない節の見出し(英, 日)。
const OTHER_SECTIONS: &[(&str, &str)] = &[
    ("Keys read", "読む項目"),
    ("View types", "ビューの型"),
    ("Operators", "演算子"),
    ("Not supported", "解釈しないもの"),
];

// ---- 小さな helper ----

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} を読めない: {e}", p.display()))
}

fn headings(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim_end().strip_prefix("## "))
        .map(|h| h.trim().to_string())
        .collect()
}

/// 見出し `## title` の節の行(次の `# `・`## ` の手前まで)。無ければ panic。
fn section_lines<'a>(text: &'a str, rel: &str, title: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut inside = false;
    let mut found = false;
    for line in text.lines() {
        let t = line.trim_end();
        if t.starts_with("# ") || t.starts_with("## ") {
            if inside {
                break;
            }
            if t.strip_prefix("## ").map(str::trim) == Some(title) {
                inside = true;
                found = true;
            }
            continue;
        }
        if inside {
            out.push(line);
        }
    }
    assert!(
        found,
        "{rel} に見出し `## {title}` が無い。ある見出し: {:?}",
        headings(text)
    );
    out
}

/// 行の最初のコードスパンの中身。
fn first_code_span(line: &str) -> Option<&str> {
    let start = line.find('`')? + 1;
    let len = line[start..].find('`')?;
    Some(line[start..start + len].trim())
}

/// コードスパンの中身 → 名前(`(` より前、最後の `.` より後)。
fn normalize(span: &str) -> String {
    let before_paren = span.split('(').next().unwrap_or("").trim();
    before_paren
        .rsplit('.')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// 文書の節の名前の一覧。節が空なら panic。
fn doc_names(rel: &str, title: &str) -> BTreeSet<String> {
    let text = repo_file(rel);
    let mut names = BTreeSet::new();
    let mut in_fence = false;
    for line in section_lines(&text, rel, title) {
        let t = line.trim_start();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let is_item = t.starts_with("- ") || t.starts_with("* ") || t.starts_with('|');
        if !is_item {
            continue;
        }
        if let Some(span) = first_code_span(t) {
            let name = normalize(span);
            assert!(
                !name.is_empty(),
                "{rel} の `## {title}` の行 {line:?} から名前を取れない"
            );
            names.insert(name);
        }
    }
    assert!(
        !names.is_empty(),
        "{rel} の `## {title}` に名前の行(箇条書きか表の行の最初のコードスパン)が無い"
    );
    names
}

fn const_set(label: &str, list: &[&str]) -> BTreeSet<String> {
    let set: BTreeSet<String> = list.iter().map(|s| s.to_string()).collect();
    assert_eq!(set.len(), list.len(), "{label} に重複がある: {list:?}");
    assert!(!set.is_empty(), "{label} が空");
    for n in &set {
        assert!(
            !n.is_empty() && !n.contains(['(', ')', '.', ' ', '`']),
            "{label} の名前 {n:?} に `(`・`.`・空白などが入っている(名前だけを置く)"
        );
    }
    set
}

fn assert_same(rel: &str, title: &str, label: &str, list: &[&str]) {
    let doc = doc_names(rel, title);
    let imp = const_set(label, list);
    let only_doc: Vec<_> = doc.difference(&imp).collect();
    let only_imp: Vec<_> = imp.difference(&doc).collect();
    assert!(
        only_doc.is_empty() && only_imp.is_empty(),
        "{rel} の `## {title}` と {label} が違う。文書だけ: {only_doc:?} {label} だけ: {only_imp:?}"
    );
}

// ---- 式の読み取りが受け付けるか ----

#[derive(Debug)]
#[allow(dead_code)] // 中の文字列は失敗の表示(Debug)にだけ使う
enum Outcome {
    Ok,
    Unsupported(String),
    Syntax(String),
}

fn outcome(src: &str) -> Outcome {
    match parse(src) {
        Ok(_) => Outcome::Ok,
        Err(ExprError::Unsupported(m)) => Outcome::Unsupported(m),
        Err(ExprError::Syntax(m)) => Outcome::Syntax(m),
    }
}

/// 引数の並びの候補。`n` 個の引数。中身は文字列(日付・期間・タグ・フォルダとして読める形)と真偽。
fn args(n: usize) -> String {
    const POOL: &[&str] = &["\"2026-09-30\"", "\"1d\"", "\"x\""];
    (0..n)
        .map(|i| POOL[i % POOL.len()])
        .collect::<Vec<_>>()
        .join(", ")
}

/// 関数ごとに分かっている形。表に無い関数は 0〜3 個の引数を順に試す。
fn function_forms(name: &str) -> Vec<String> {
    let known: &[(&str, &str)] = &[
        ("if", "if(true, 1, 2)"),
        ("date", "date(\"2026-09-30\")"),
        ("now", "now()"),
        ("today", "today()"),
        ("duration", "duration(\"1d\")"),
    ];
    let mut forms: Vec<String> = known
        .iter()
        .filter(|(n, _)| *n == name)
        .map(|(_, s)| s.to_string())
        .collect();
    forms.extend((0..=3).map(|n| format!("{name}({})", args(n))));
    forms
}

/// メソッド(または `file` に付く名前)を使う形。受け手はリスト・文字列・`file`、引数は 0〜3 個、
/// 括弧の無い形(`.length` など)も試す。
fn member_forms(name: &str) -> Vec<String> {
    let mut forms = Vec::new();
    for recv in ["note.list", "status", "file", "file.tags", "file.name"] {
        for n in 0..=3 {
            forms.push(format!("{recv}.{name}({})", args(n)));
        }
        forms.push(format!("{recv}.{name}"));
    }
    forms
}

/// 形のどれかが Ok なら Ok。全部だめなら、試した形と結果を返す。
fn accepted_by_any(forms: &[String]) -> Result<(), Vec<(String, Outcome)>> {
    let mut tried = Vec::new();
    for f in forms {
        match outcome(f) {
            Outcome::Ok => return Ok(()),
            o => tried.push((f.clone(), o)),
        }
    }
    Err(tried)
}

fn assert_function_accepted(name: &str, from: &str) {
    if let Err(tried) = accepted_by_any(&function_forms(name)) {
        panic!(
            "{from} の関数 {name:?} を使う式がどれも読めない(BV-7 の未対応か文法の誤り): {tried:?}"
        );
    }
}

fn assert_method_accepted(name: &str, from: &str) {
    if let Err(tried) = accepted_by_any(&member_forms(name)) {
        panic!("{from} のメソッド {name:?} を使う式がどれも読めない(BV-7 の未対応か文法の誤り): {tried:?}");
    }
}

fn assert_file_field_accepted(name: &str, from: &str) {
    let mut forms = vec![format!("file.{name}")];
    forms.extend((0..=3).map(|n| format!("file.{name}({})", args(n))));
    if let Err(tried) = accepted_by_any(&forms) {
        panic!(
            "{from} の file の名前 {name:?} を使う式がどれも読めない(BV-7 の未対応か文法の誤り): {tried:?}"
        );
    }
}

/// 形がどれも Unsupported(名前を含む)か Syntax で、少なくとも1つは Unsupported(Ok になる形が1つも無い)。
/// 違えば理由を返す。
fn check_unsupported(name: &str, forms: &[String]) -> Result<(), String> {
    let mut any_unsupported = false;
    for f in forms {
        match outcome(f) {
            Outcome::Unsupported(m) => {
                if !m.contains(name) {
                    return Err(format!("{f:?}: Unsupported({m:?}) に {name:?} が無い"));
                }
                any_unsupported = true;
            }
            Outcome::Ok => {
                return Err(format!(
                    "{f:?} が読めてしまった({name:?} を受け付けるなら FUNCTIONS・METHODS・FILE_FIELDS のどれかに置く)"
                ))
            }
            // 引数の数の誤りなどを Syntax にするのは許す(受け付けていないことに変わりはない)。
            Outcome::Syntax(_) => {}
        }
    }
    // 少なくとも1つは Unsupported として名前が出る(黙って Syntax にしない)。
    if !any_unsupported {
        return Err(format!(
            "{name:?} を使う式がどれも Unsupported にならない: {forms:?}"
        ));
    }
    Ok(())
}

fn assert_all_unsupported(name: &str, forms: &[String]) {
    if let Err(e) = check_unsupported(name, forms) {
        panic!("{e}");
    }
}

/// 名前ごとの check_unsupported の誤りをまとめて出す(受け付けているのに定数に無い名前を全部並べる)。
fn assert_none_accepted(label: &str, failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{label}: 定数に無いのに未対応にならない名前がある:\n{}",
        failures.join("\n")
    );
}

fn all_known() -> BTreeSet<&'static str> {
    FUNCTIONS
        .iter()
        .chain(METHODS.iter())
        .chain(FILE_FIELDS.iter())
        .copied()
        .collect()
}

// ---- BV-21: 文書の形 ----

#[test]
fn test_bv_21_english_doc_has_sections() {
    // [BV-21] 英語の文書に、読む項目・ビューの型・演算子・関数・メソッド・file・解釈しないものの節がある
    let text = repo_file(EN);
    for (en, _) in OTHER_SECTIONS
        .iter()
        .chain([SEC_FUNCTIONS, SEC_METHODS, SEC_FILE].iter())
    {
        section_lines(&text, EN, en);
    }
}

#[test]
fn test_bv_21_japanese_doc_has_sections() {
    // [BV-21] 日本語の文書に、同じ節がある
    let text = repo_file(JA);
    for (_, ja) in OTHER_SECTIONS
        .iter()
        .chain([SEC_FUNCTIONS, SEC_METHODS, SEC_FILE].iter())
    {
        section_lines(&text, JA, ja);
    }
}

#[test]
fn test_bv_21_docs_link_each_other() {
    // [BV-21] 英と日の文書は互いにリンクする
    let en = repo_file(EN);
    let ja = repo_file(JA);
    assert!(
        en.contains("](obsidian-bases.ja.md)") || en.contains("](./obsidian-bases.ja.md)"),
        "{EN} に日本語の文書(obsidian-bases.ja.md)へのリンクが無い"
    );
    assert!(
        ja.contains("](obsidian-bases.md)") || ja.contains("](./obsidian-bases.md)"),
        "{JA} に英語の文書(obsidian-bases.md)へのリンクが無い"
    );
}

// ---- BV-21: 文書の一覧 = 定数 ----

#[test]
fn test_bv_21_english_functions_match() {
    // [BV-21] docs/obsidian-bases.md の関数の一覧 = FUNCTIONS
    assert_same(EN, SEC_FUNCTIONS.0, "FUNCTIONS", FUNCTIONS);
}

#[test]
fn test_bv_21_english_methods_match() {
    // [BV-21] docs/obsidian-bases.md のメソッドの一覧 = METHODS
    assert_same(EN, SEC_METHODS.0, "METHODS", METHODS);
}

#[test]
fn test_bv_21_english_file_properties_match() {
    // [BV-21] docs/obsidian-bases.md の file の一覧 = FILE_FIELDS
    assert_same(EN, SEC_FILE.0, "FILE_FIELDS", FILE_FIELDS);
}

#[test]
fn test_bv_21_japanese_functions_match() {
    // [BV-21] docs/obsidian-bases.ja.md の関数の一覧 = FUNCTIONS
    assert_same(JA, SEC_FUNCTIONS.1, "FUNCTIONS", FUNCTIONS);
}

#[test]
fn test_bv_21_japanese_methods_match() {
    // [BV-21] docs/obsidian-bases.ja.md のメソッドの一覧 = METHODS
    assert_same(JA, SEC_METHODS.1, "METHODS", METHODS);
}

#[test]
fn test_bv_21_japanese_file_properties_match() {
    // [BV-21] docs/obsidian-bases.ja.md の file の一覧 = FILE_FIELDS
    assert_same(JA, SEC_FILE.1, "FILE_FIELDS", FILE_FIELDS);
}

// ---- BV-21: 定数・文書の名前は式の読み取りが本当に受け付ける(BV-7 の未対応にならない) ----

#[test]
fn test_bv_21_constants_hold_current_names() {
    // [BV-21] 今の式の評価が受け付ける名前(tests/test_expr.rs が使うもの)は、どれかの定数にある
    let known = all_known();
    for f in ["if", "date", "now", "today", "duration"] {
        assert!(FUNCTIONS.contains(&f), "FUNCTIONS に {f:?} が無い");
    }
    for m in [
        "contains",
        "containsAll",
        "containsAny",
        "isEmpty",
        "toString",
    ] {
        assert!(METHODS.contains(&m), "METHODS に {m:?} が無い");
    }
    for n in [
        "name", "basename", "ext", "path", "folder", "size", "mtime", "ctime", "tags",
    ] {
        assert!(FILE_FIELDS.contains(&n), "FILE_FIELDS に {n:?} が無い");
    }
    for n in ["length", "hasTag", "inFolder", "hasProperty"] {
        assert!(
            known.contains(n),
            "{n:?} が FUNCTIONS・METHODS・FILE_FIELDS のどれにも無い"
        );
    }
}

#[test]
fn test_bv_21_functions_are_accepted() {
    // [BV-21] FUNCTIONS の各関数を使った式は、parse で未対応にならない
    for name in FUNCTIONS {
        assert_function_accepted(name, "FUNCTIONS");
    }
}

#[test]
fn test_bv_21_methods_are_accepted() {
    // [BV-21] METHODS の各メソッドを使った式は、parse で未対応にならない
    for name in METHODS {
        assert_method_accepted(name, "METHODS");
    }
}

#[test]
fn test_bv_21_file_fields_are_accepted() {
    // [BV-21] FILE_FIELDS の各名前を使った式は、parse で未対応にならない
    for name in FILE_FIELDS {
        assert_file_field_accepted(name, "FILE_FIELDS");
    }
}

#[test]
fn test_bv_21_doc_names_are_accepted() {
    // [BV-21] 文書(英・日)に書いた関数・メソッド・file の名前を使った絞り込みの式は、BV-7 の未対応にならない
    for rel in [EN, JA] {
        let (f, m, file) = if rel == EN {
            (SEC_FUNCTIONS.0, SEC_METHODS.0, SEC_FILE.0)
        } else {
            (SEC_FUNCTIONS.1, SEC_METHODS.1, SEC_FILE.1)
        };
        for name in doc_names(rel, f) {
            assert_function_accepted(&name, rel);
        }
        for name in doc_names(rel, m) {
            assert_method_accepted(&name, rel);
        }
        for name in doc_names(rel, file) {
            assert_file_field_accepted(&name, rel);
        }
    }
}

#[test]
fn test_bv_21_filter_with_supported_names_is_not_unsupported() {
    // [BV-21] 対応する関数・メソッド・file を組み合わせた絞り込みの式は、まとめて読める
    let src = "file.inFolder(\"notes\") && file.hasTag(\"proj\") && note.list.contains(\"x\") \
               && if(status == \"done\", true, false) && date(\"2026-09-30\") < today() \
               && file.name.contains(\"a\") && file.mtime < now()";
    match outcome(src) {
        Outcome::Ok => {}
        o => panic!("{src:?} が読めない: {o:?}"),
    }
}

// ---- BV-21: 定数に無い名前は未対応(定数が受け付けるものを言い当てていることの裏) ----

#[test]
fn test_bv_21_fake_names_are_unsupported() {
    // [BV-21][BV-7] 定数に無い作りものの名前は未対応になる
    assert_all_unsupported("fakefn", &function_forms("fakefn"));
    assert_all_unsupported(
        "fakeMethod",
        &(0..=2)
            .map(|n| format!("note.list.fakeMethod({})", args(n)))
            .collect::<Vec<_>>(),
    );
    assert_all_unsupported("fakeField", &["file.fakeField".to_string()]);
}

/// Obsidian の Bases にある(またはありそうな)トップの関数の名前。定数に無いものは未対応のはず。
const OBSIDIAN_FUNCTIONS: &[&str] = &[
    "if",
    "date",
    "now",
    "today",
    "duration",
    "max",
    "min",
    "link",
    "list",
    "number",
    "image",
    "icon",
    "escapeHTML",
    "html",
    "sum",
    "average",
    "round",
    "abs",
    "string",
    "boolean",
];

/// Obsidian の Bases にある(またはありそうな)メソッドの名前。定数に無いものは未対応のはず。
const OBSIDIAN_METHODS: &[&str] = &[
    "contains",
    "containsAll",
    "containsAny",
    "isEmpty",
    "toString",
    "length",
    "startsWith",
    "endsWith",
    "lower",
    "upper",
    "title",
    "trim",
    "replace",
    "repeat",
    "reverse",
    "slice",
    "split",
    "join",
    "map",
    "filter",
    "reduce",
    "sort",
    "unique",
    "flat",
    "format",
    "relative",
    "year",
    "month",
    "day",
    "hour",
    "minute",
    "second",
    "time",
    "round",
    "floor",
    "ceil",
    "abs",
    "toFixed",
    "isTruthy",
    "isType",
    "matches",
    "asFile",
    "asLink",
    "linksTo",
    "keys",
    "values",
    "hasTag",
    "hasLink",
    "hasProperty",
    "inFolder",
];

/// Obsidian の Bases にある `file.*` の名前。定数に無いものは未対応のはず。
const OBSIDIAN_FILE_FIELDS: &[&str] = &[
    "name",
    "basename",
    "ext",
    "path",
    "folder",
    "size",
    "mtime",
    "ctime",
    "tags",
    "links",
    "backlinks",
    "embeds",
    "properties",
    "file",
];

#[test]
fn test_bv_21_unlisted_obsidian_functions_are_unsupported() {
    // [BV-21][BV-7] Obsidian の関数で FUNCTIONS(ほかの定数も含む)に無いものは、どの引数の数でも読めない
    let known = all_known();
    let failures: Vec<String> = OBSIDIAN_FUNCTIONS
        .iter()
        .filter(|n| !known.contains(*n))
        .filter_map(|n| check_unsupported(n, &function_forms(n)).err())
        .collect();
    assert_none_accepted("FUNCTIONS", failures);
}

#[test]
fn test_bv_21_unlisted_obsidian_methods_are_unsupported() {
    // [BV-21][BV-7] Obsidian のメソッドで定数に無いものは、リスト・文字列に付けても読めない
    let known = all_known();
    let failures: Vec<String> = OBSIDIAN_METHODS
        .iter()
        .filter(|n| !known.contains(*n))
        .filter_map(|name| {
            let mut forms = Vec::new();
            for recv in ["note.list", "status", "file.name"] {
                for n in 0..=2 {
                    forms.push(format!("{recv}.{name}({})", args(n)));
                }
            }
            check_unsupported(name, &forms).err()
        })
        .collect();
    assert_none_accepted("METHODS", failures);
}

#[test]
fn test_bv_21_unlisted_obsidian_file_fields_are_unsupported() {
    // [BV-21][BV-7] Obsidian の file の名前で定数に無いもの(backlinks など)は、file.X で読めない
    let known = all_known();
    let failures: Vec<String> = OBSIDIAN_FILE_FIELDS
        .iter()
        .filter(|n| !known.contains(*n))
        .filter_map(|n| check_unsupported(n, &[format!("file.{n}")]).err())
        .collect();
    assert_none_accepted("FILE_FIELDS", failures);
}
