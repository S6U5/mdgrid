//! ボードの画面のレーンに分ける核 `mdgrid::board::lanes`(BD-2)の受け入れ試験。
//! 記録: specs/_changes/2026-11-13-board-view.md(タスク #1)。実装を見ずに、記録の「設計」の口の形
//! (`Lane { value, rows }` と `lanes(rows, order)`)と、仕様(specs/board/spec.md)の BD-2 だけから書いた。
//! 値はどれも作ったもの。

use mdgrid::board::{lanes, Lane};
use mdgrid::source::RowId;

fn id(name: &str) -> RowId {
    RowId(name.to_string())
}

/// 行(名前と値。値の "" でない None は「値が空」)。
fn rows(items: &[(&str, Option<&str>)]) -> Vec<(RowId, Option<String>)> {
    items
        .iter()
        .map(|(n, v)| (id(n), v.map(|s| s.to_string())))
        .collect()
}

fn order(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}

/// レーンを「値(None は (なし))」と行の名前の並びにする。
fn outline(ls: &[Lane]) -> Vec<(String, Vec<String>)> {
    ls.iter()
        .map(|l| {
            (
                l.value.clone().unwrap_or_else(|| "(なし)".to_string()),
                l.rows.iter().map(|r| r.0.clone()).collect(),
            )
        })
        .collect()
}

fn want(items: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
    items
        .iter()
        .map(|(v, rs)| (v.to_string(), rs.iter().map(|r| r.to_string()).collect()))
        .collect()
}

#[test]
fn test_bd_2_rule_order_first_then_rest_then_none() {
    // [BD-2] 決まりが todo・doing・done で、行が done・todo・review・空 → todo(1)・doing(0)・done(1)・review(1)・(なし)(1)。
    let rs = rows(&[
        ("a.md", Some("done")),
        ("b.md", Some("todo")),
        ("c.md", Some("review")),
        ("d.md", None),
    ]);
    let ls = lanes(&rs, &order(&["todo", "doing", "done"]));
    assert_eq!(
        outline(&ls),
        want(&[
            ("todo", &["b.md"]),
            ("doing", &[]),
            ("done", &["a.md"]),
            ("review", &["c.md"]),
            ("(なし)", &["d.md"]),
        ])
    );
    // 「(なし)」のレーンは value が None、ほかは Some。
    assert_eq!(ls.last().unwrap().value, None);
    assert!(ls[..ls.len() - 1].iter().all(|l| l.value.is_some()));
}

#[test]
fn test_bd_2_no_rule_sorts_by_text() {
    // [BD-2] 決まりが無い → doing・done・todo の文字の順(行の出てきた順ではない)。
    let rs = rows(&[
        ("a.md", Some("todo")),
        ("b.md", Some("done")),
        ("c.md", Some("doing")),
    ]);
    let ls = lanes(&rs, &[]);
    assert_eq!(
        outline(&ls),
        want(&[
            ("doing", &["c.md"]),
            ("done", &["b.md"]),
            ("todo", &["a.md"]),
        ])
    );
}

#[test]
fn test_bd_2_rule_lanes_shown_without_rows() {
    // [BD-2] 決まりにある値は、その値の行が無くてもレーンを出す(行が1つも無くても)。
    let ls = lanes(&[], &order(&["todo", "doing", "done"]));
    assert_eq!(
        outline(&ls),
        want(&[("todo", &[]), ("doing", &[]), ("done", &[])])
    );
    // 決まりの順は文字の順に直さない。
    let rs = rows(&[("a.md", Some("b")), ("b.md", Some("z"))]);
    let ls = lanes(&rs, &order(&["z", "m", "b"]));
    assert_eq!(
        outline(&ls),
        want(&[("z", &["b.md"]), ("m", &[]), ("b", &["a.md"])])
    );
}

#[test]
fn test_bd_2_none_lane_only_when_empty_rows() {
    // [BD-2] 空の行が無い → 「(なし)」は出ない。決まりがあっても同じ。
    let rs = rows(&[("a.md", Some("todo")), ("b.md", Some("done"))]);
    for ord in [order(&[]), order(&["todo", "doing", "done"])] {
        let ls = lanes(&rs, &ord);
        assert!(ls.iter().all(|l| l.value.is_some()), "{:?}", outline(&ls));
    }
    // 行が1つも無く、決まりも無い → レーンは無い。
    assert!(lanes(&[], &[]).is_empty());
}

#[test]
fn test_bd_2_none_lane_is_last_even_if_first_row_is_empty() {
    // [BD-2] 空の行が先頭にあっても、「(なし)」は最後(決まりの値・残りの値のあと)。空の行が2つでも1つのレーン。
    let rs = rows(&[
        ("a.md", None),
        ("b.md", Some("zzz")),
        ("c.md", None),
        ("d.md", Some("todo")),
    ]);
    let ls = lanes(&rs, &order(&["todo"]));
    assert_eq!(
        outline(&ls),
        want(&[
            ("todo", &["d.md"]),
            ("zzz", &["b.md"]),
            ("(なし)", &["a.md", "c.md"]),
        ])
    );
    // 決まりが無くても最後。
    let ls = lanes(&rs, &[]);
    assert_eq!(
        outline(&ls),
        want(&[
            ("todo", &["d.md"]),
            ("zzz", &["b.md"]),
            ("(なし)", &["a.md", "c.md"]),
        ])
    );
}

#[test]
fn test_bd_2_keeps_given_row_order_in_each_lane() {
    // [BD-2][BD-3] レーンの中の行は、渡した行の並びのまま(名前の順などに並べ直さない)。
    let rs = rows(&[
        ("z.md", Some("todo")),
        ("m.md", Some("done")),
        ("a.md", Some("todo")),
        ("k.md", Some("done")),
        ("b.md", Some("todo")),
    ]);
    let ls = lanes(&rs, &order(&["todo", "done"]));
    assert_eq!(
        outline(&ls),
        want(&[
            ("todo", &["z.md", "a.md", "b.md"]),
            ("done", &["m.md", "k.md"]),
        ])
    );
}

#[test]
fn test_bd_2_same_value_makes_one_lane_and_every_row_once() {
    // [BD-2] 同じ値の行は1つのレーン(決まりにある値も、無い値も)。どの行もちょうど1回だけ出る。
    let rs = rows(&[
        ("a.md", Some("todo")),
        ("b.md", Some("review")),
        ("c.md", Some("todo")),
        ("d.md", Some("review")),
        ("e.md", None),
        ("f.md", Some("alpha")),
    ]);
    let ls = lanes(&rs, &order(&["todo", "doing"]));
    assert_eq!(
        outline(&ls),
        want(&[
            ("todo", &["a.md", "c.md"]),
            ("doing", &[]),
            ("alpha", &["f.md"]),
            ("review", &["b.md", "d.md"]),
            ("(なし)", &["e.md"]),
        ])
    );
    let mut all: Vec<String> = ls
        .iter()
        .flat_map(|l| l.rows.iter().map(|r| r.0.clone()))
        .collect();
    all.sort();
    assert_eq!(all, ["a.md", "b.md", "c.md", "d.md", "e.md", "f.md"]);
}

#[test]
fn test_bd_2_values_differ_by_exact_text() {
    // [BD-2] レーンは値の文字ごと: 大文字と小文字の違う値は別のレーン。
    let rs = rows(&[
        ("a.md", Some("Todo")),
        ("b.md", Some("todo")),
        ("c.md", Some("todo")),
    ]);
    let ls = lanes(&rs, &order(&["todo"]));
    assert_eq!(
        outline(&ls),
        want(&[("todo", &["b.md", "c.md"]), ("Todo", &["a.md"])])
    );
}
