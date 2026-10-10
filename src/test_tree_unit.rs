//! 親子の並べ方(NV-27)の核。

use crate::tree::{order, Node};

fn run(
    rows: &[&'static str],
    pairs: &[(&'static str, &'static str)],
) -> Vec<(&'static str, usize, bool)> {
    let p = |r: &&'static str| pairs.iter().find(|(c, _)| c == r).map(|(_, p)| *p);
    order(rows, p)
        .into_iter()
        .map(
            |Node {
                 row,
                 depth,
                 has_kids,
                 ..
             }| (row, depth, has_kids),
        )
        .collect()
}

#[test]
fn test_nv_27_order_preorder_keeps_sibling_order() {
    // [NV-27] 設計の下に画面(その下に部品)と API。兄弟の順は元の並び(API が先なら API が先)。
    let rows = ["API", "部品", "設計", "画面", "メモ"];
    let pairs = [("画面", "設計"), ("API", "設計"), ("部品", "画面")];
    assert_eq!(
        run(&rows, &pairs),
        [
            ("設計", 0, true),
            ("API", 1, false),
            ("画面", 1, true),
            ("部品", 2, false),
            ("メモ", 0, false)
        ]
    );
}

#[test]
fn test_nv_27_order_missing_parent_and_cycles_go_top() {
    // [NV-27] 親が並びに無い行・自分が親・輪(A↔B、C→D→E→C)は一番上の段。行の数は変わらない。
    let rows = ["子", "A", "B", "自分", "C", "D", "E"];
    let pairs = [
        ("子", "無い親"),
        ("A", "B"),
        ("B", "A"),
        ("自分", "自分"),
        ("C", "D"),
        ("D", "E"),
        ("E", "C"),
    ];
    let got = run(&rows, &pairs);
    assert_eq!(got.len(), rows.len());
    assert!(got.contains(&("子", 0, false)));
    assert!(got.contains(&("自分", 0, false)));
    // 輪の行はどれも出て、どれかは一番上の段。
    for r in ["A", "B", "C", "D", "E"] {
        assert!(got.iter().any(|g| g.0 == r), "{r}");
    }
    assert!(got.iter().filter(|g| g.1 == 0).count() >= 4, "{got:?}");
}

#[test]
fn test_nv_27_order_empty_and_flat() {
    // [NV-27] 空の並び・親の無い並びはそのまま。
    assert!(run(&[], &[]).is_empty());
    assert_eq!(run(&["a", "b"], &[]), [("a", 0, false), ("b", 0, false)]);
}
