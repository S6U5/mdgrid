//! ビューの設定の受け入れ(NV-14・NV-15・NV-17・NV-19・NV-21)。
//! 実装を見ずに、docs/design.md の「ビューの設定(NV-13〜NV-22)」の `src/settings.rs` と
//! `config::ViewState.settings` の形だけを使う。
//!
//! 材料は手で作る小さな表(5行):
//!
//! | 行 | status | 種別 | tags         | priority | due        | title          |
//! |----|--------|------|--------------|----------|------------|----------------|
//! | r1 | todo   | 本   | [会議, 仕事] | 12       | 2026-09-01 | Alpha Report   |
//! | r2 | done   | 記事 | [私用]       | 1        | 2026-10-15 | beta memo      |
//! | r3 | (なし) | 本   | []           | 3        | someday    | Gamma          |
//! | r4 | doing  | (なし)| [会議]      | "high"   | (なし)     | delta REPORT   |
//! | r5 | done   | 本   | [仕事]       | 4        | 2026-09-30 | ""             |

use mdgrid::config::{load_state, save_state};
use mdgrid::settings::{
    apply, matches, value_counts, value_keys, CmpOp, Cond, Dir, Group, Op, Settings,
};
use mdgrid::source::{RowId, Value};
use mdgrid::types::Kind;
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ---- 材料 ----

struct Table {
    rows: Vec<RowId>,
    cells: HashMap<String, HashMap<String, Value>>,
}

fn str_v(s: &str) -> Value {
    Value::Str(s.to_string())
}

fn list_v(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| str_v(s)).collect())
}

fn table() -> Table {
    let mut cells: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut put = |row: &str, kv: Vec<(&str, Value)>| {
        let m = cells.entry(row.to_string()).or_default();
        for (k, v) in kv {
            m.insert(k.to_string(), v);
        }
    };
    put(
        "r1",
        vec![
            ("status", str_v("todo")),
            ("種別", str_v("本")),
            ("tags", list_v(&["会議", "仕事"])),
            ("priority", Value::Int(12)),
            ("due", str_v("2026-09-01")),
            ("title", str_v("Alpha Report")),
        ],
    );
    put(
        "r2",
        vec![
            ("status", str_v("done")),
            ("種別", str_v("記事")),
            ("tags", list_v(&["私用"])),
            ("priority", Value::Int(1)),
            ("due", str_v("2026-10-15")),
            ("title", str_v("beta memo")),
        ],
    );
    put(
        "r3",
        vec![
            ("種別", str_v("本")),
            ("tags", list_v(&[])),
            ("priority", Value::Int(3)),
            ("due", str_v("someday")),
            ("title", str_v("Gamma")),
        ],
    );
    put(
        "r4",
        vec![
            ("status", str_v("doing")),
            ("tags", list_v(&["会議"])),
            ("priority", str_v("high")),
            ("title", str_v("delta REPORT")),
        ],
    );
    put(
        "r5",
        vec![
            ("status", str_v("done")),
            ("種別", str_v("本")),
            ("tags", list_v(&["仕事"])),
            ("priority", Value::Int(4)),
            ("due", str_v("2026-09-30")),
            ("title", str_v("")),
        ],
    );
    Table {
        rows: ["r1", "r2", "r3", "r4", "r5"]
            .iter()
            .map(|r| RowId(r.to_string()))
            .collect(),
        cells,
    }
}

fn kind_of(col: &str) -> Kind {
    match col {
        "tags" => Kind::List,
        "priority" => Kind::Number,
        "due" => Kind::Date,
        _ => Kind::Text,
    }
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

fn ids(rows: &[RowId]) -> Vec<String> {
    rows.iter().map(|r| r.0.clone()).collect()
}

fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// 設定を表に当てる(渡す groups は空)。
fn run(s: &Settings) -> (Vec<RowId>, Vec<(String, Range<usize>)>) {
    run_with_groups(s, Vec::new())
}

fn run_with_groups(
    s: &Settings,
    groups: Vec<(String, Range<usize>)>,
) -> (Vec<RowId>, Vec<(String, Range<usize>)>) {
    let t = table();
    let cells = t.cells.clone();
    let get = move |row: &RowId, col: &str| -> Option<Value> {
        cells.get(&row.0).and_then(|m| m.get(col).cloned())
    };
    apply(t.rows.clone(), groups, s, &get, &kind_of)
}

fn filtered(filters: Vec<Cond>) -> Vec<String> {
    let s = Settings {
        filters,
        ..Settings::default()
    };
    ids(&run(&s).0)
}

fn sorted(sorts: Vec<(&str, Dir)>) -> Vec<String> {
    let s = Settings {
        sorts: sorts.into_iter().map(|(c, d)| (c.to_string(), d)).collect(),
        ..Settings::default()
    };
    ids(&run(&s).0)
}

/// まとまり → (見出し, その範囲の行)。範囲が rows を隙間なく覆うことも確かめる。
fn grouped(rows: &[RowId], groups: &[(String, Range<usize>)]) -> Vec<(String, Vec<String>)> {
    let mut next = 0;
    let mut out = Vec::new();
    for (h, r) in groups {
        assert_eq!(r.start, next, "まとまりの範囲が続いていない: {:?}", groups);
        assert!(r.end <= rows.len(), "範囲が行の外: {:?}", groups);
        next = r.end;
        out.push((h.clone(), ids(&rows[r.clone()])));
    }
    assert_eq!(
        next,
        rows.len(),
        "まとまりが全部の行を覆っていない: {:?}",
        groups
    );
    out
}

fn group_by(col: &str, dir: Dir, hide_empty: bool) -> Group {
    Group::By {
        col: col.to_string(),
        dir,
        hide_empty,
    }
}

// ---- NV-14: どの列でも、値を選んで隠す・残す ----

#[test]
fn test_nv_14_drop_done_hides_done_rows() {
    // [NV-14] status の done を外す → done の行が消える。
    assert_eq!(
        filtered(vec![cond("status", Op::Drop(some(&["done"])))]),
        names(&["r1", "r3", "r4"])
    );
}

#[test]
fn test_nv_14_keep_only_book_in_other_column() {
    // [NV-14] status 以外の列(種別)でも効く。本 だけを残す → 本 の行だけ。
    assert_eq!(
        filtered(vec![cond("種別", Op::Keep(some(&["本"])))]),
        names(&["r1", "r3", "r5"])
    );
}

#[test]
fn test_nv_14_empty_value_can_be_kept_or_dropped() {
    // [NV-14] キーの無い行は「(空)」(None)として選べる。
    assert_eq!(
        filtered(vec![cond("status", Op::Keep(vec![None]))]),
        names(&["r3"])
    );
    assert_eq!(
        filtered(vec![cond("種別", Op::Drop(vec![None]))]),
        names(&["r1", "r2", "r3", "r5"])
    );
    // 空と値を並べて残す。
    assert_eq!(
        filtered(vec![cond(
            "status",
            Op::Keep(vec![None, Some("todo".to_string())])
        )]),
        names(&["r1", "r3"])
    );
}

#[test]
fn test_nv_14_keep_works_on_number_column() {
    // [NV-14] 数の列でも、値(表示の文字列)を選んで残せる。
    assert_eq!(
        filtered(vec![cond("priority", Op::Keep(some(&["3"])))]),
        names(&["r3"])
    );
}

#[test]
fn test_nv_14_matches_single_value() {
    // [NV-14] matches は1つの値に条件を当てる。
    let drop_done = cond("status", Op::Drop(some(&["done"])));
    assert!(!matches(&drop_done, Some(&str_v("done")), Kind::Text));
    assert!(matches(&drop_done, Some(&str_v("todo")), Kind::Text));
    assert!(matches(&drop_done, None, Kind::Text));
    let keep_empty = cond("status", Op::Keep(vec![None]));
    assert!(matches(&keep_empty, None, Kind::Text));
    assert!(matches(&keep_empty, Some(&Value::Null), Kind::Text));
    assert!(matches(&keep_empty, Some(&str_v("")), Kind::Text));
    assert!(!matches(&keep_empty, Some(&str_v("todo")), Kind::Text));
}

#[test]
fn test_nv_14_no_filters_keeps_all_rows_in_order() {
    // [NV-14] 条件が無ければ行はそのまま。
    let (rows, groups) = run(&Settings::default());
    assert_eq!(ids(&rows), names(&["r1", "r2", "r3", "r4", "r5"]));
    assert!(groups.is_empty());
}

// ---- NV-19: 値の一覧と条件の種類 ----

#[test]
fn test_nv_19_value_keys() {
    // [NV-19] 値の鍵: 空は [None]、リストは要素ごと。
    assert_eq!(value_keys(None), vec![None]);
    assert_eq!(value_keys(Some(&Value::Null)), vec![None]);
    assert_eq!(value_keys(Some(&str_v(""))), vec![None]);
    assert_eq!(value_keys(Some(&list_v(&[]))), vec![None]);
    assert_eq!(value_keys(Some(&str_v("done"))), some(&["done"]));
    assert_eq!(
        value_keys(Some(&list_v(&["会議", "仕事"]))),
        some(&["会議", "仕事"])
    );
}

#[test]
fn test_nv_19_value_counts_status() {
    // [NV-19] 値の一覧は件数つき、空は None、件数の多い順。
    let t = table();
    let vals: Vec<Option<Value>> = t
        .rows
        .iter()
        .map(|r| t.cells[&r.0].get("status").cloned())
        .collect();
    let counts = value_counts(vals.iter().map(|v| v.as_ref()));
    assert_eq!(counts.len(), 4, "done・todo・doing・空");
    assert_eq!(counts[0], (Some("done".to_string()), 2));
    let rest: HashMap<Option<String>, usize> = counts[1..].iter().cloned().collect();
    assert_eq!(rest.get(&Some("todo".to_string())), Some(&1));
    assert_eq!(rest.get(&Some("doing".to_string())), Some(&1));
    assert_eq!(rest.get(&None), Some(&1));
}

#[test]
fn test_nv_19_value_counts_list_per_element() {
    // [NV-19] リストの値は要素ごとに数える。空のリストは「(空)」。
    let t = table();
    let vals: Vec<Option<Value>> = t
        .rows
        .iter()
        .map(|r| t.cells[&r.0].get("tags").cloned())
        .collect();
    let counts = value_counts(vals.iter().map(|v| v.as_ref()));
    let m: HashMap<Option<String>, usize> = counts.iter().cloned().collect();
    assert_eq!(m.len(), 4, "会議・仕事・私用・空: {:?}", counts);
    assert_eq!(m.get(&Some("会議".to_string())), Some(&2));
    assert_eq!(m.get(&Some("仕事".to_string())), Some(&2));
    assert_eq!(m.get(&Some("私用".to_string())), Some(&1));
    assert_eq!(m.get(&None), Some(&1));
    // 件数の多い順。
    for w in counts.windows(2) {
        assert!(w[0].1 >= w[1].1, "件数の多い順でない: {:?}", counts);
    }
}

#[test]
fn test_nv_19_keep_list_element() {
    // [NV-19] tags の値の一覧で 会議 だけにチェック → 会議 を要素に持つ行だけ。
    assert_eq!(
        filtered(vec![cond("tags", Op::Keep(some(&["会議"])))]),
        names(&["r1", "r4"])
    );
    // 隠すときは要素のどれかが入れば隠す。
    assert_eq!(
        filtered(vec![cond("tags", Op::Drop(some(&["仕事"])))]),
        names(&["r2", "r3", "r4"])
    );
}

#[test]
fn test_nv_19_contains_is_case_insensitive() {
    // [NV-19] テキストの「含む・含まない」は大文字小文字を区別しない。
    assert_eq!(
        filtered(vec![cond("title", Op::Contains("report".to_string()))]),
        names(&["r1", "r4"])
    );
    assert_eq!(
        filtered(vec![cond("title", Op::Contains("REPORT".to_string()))]),
        names(&["r1", "r4"])
    );
    assert_eq!(
        filtered(vec![cond("title", Op::NotContains("Report".to_string()))]),
        names(&["r2", "r3", "r5"])
    );
}

#[test]
fn test_nv_19_number_cmp() {
    // [NV-19] 数の比較: priority ≥ 3。型の合わない値("high")は残らない。
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Ge, "3".to_string()))]),
        names(&["r1", "r3", "r5"])
    );
    // 数として比べる(文字の順なら "12" < "3")。
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Gt, "10".to_string()))]),
        names(&["r1"])
    );
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Eq, "4".to_string()))]),
        names(&["r5"])
    );
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Ne, "4".to_string()))]),
        names(&["r1", "r2", "r3"])
    );
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Lt, "3".to_string()))]),
        names(&["r2"])
    );
    assert_eq!(
        filtered(vec![cond("priority", Op::Cmp(CmpOp::Le, "3".to_string()))]),
        names(&["r2", "r3"])
    );
}

#[test]
fn test_nv_19_date_cmp_drops_mismatched_and_empty() {
    // [NV-19] 日付の列の `< 2026-10-01` → それより前の行だけ。`someday` と空は残らない。
    assert_eq!(
        filtered(vec![cond(
            "due",
            Op::Cmp(CmpOp::Lt, "2026-10-01".to_string())
        )]),
        names(&["r1", "r5"])
    );
    assert_eq!(
        filtered(vec![cond(
            "due",
            Op::Cmp(CmpOp::Ge, "2026-09-30".to_string())
        )]),
        names(&["r2", "r5"])
    );
    let lt = cond("due", Op::Cmp(CmpOp::Lt, "2026-10-01".to_string()));
    assert!(!matches(&lt, Some(&str_v("someday")), Kind::Date));
    assert!(!matches(&lt, None, Kind::Date));
    assert!(matches(&lt, Some(&str_v("2026-09-01")), Kind::Date));
}

#[test]
fn test_nv_19_empty_and_not_empty() {
    // [NV-19] 「空である・空でない」。キーなし・空のリスト・空の文字列は空。
    assert_eq!(filtered(vec![cond("status", Op::Empty)]), names(&["r3"]));
    assert_eq!(
        filtered(vec![cond("status", Op::NotEmpty)]),
        names(&["r1", "r2", "r4", "r5"])
    );
    assert_eq!(filtered(vec![cond("tags", Op::Empty)]), names(&["r3"]));
    assert_eq!(filtered(vec![cond("title", Op::Empty)]), names(&["r5"]));
}

#[test]
fn test_nv_19_conditions_are_and() {
    // [NV-19] priority `≥ 3` と status の done を外す を並べる → 両方を満たす行だけ。
    assert_eq!(
        filtered(vec![
            cond("priority", Op::Cmp(CmpOp::Ge, "3".to_string())),
            cond("status", Op::Drop(some(&["done"]))),
        ]),
        names(&["r1", "r3"])
    );
    assert_eq!(
        filtered(vec![
            cond("tags", Op::Keep(some(&["会議"]))),
            cond("種別", Op::Keep(some(&["本"]))),
        ]),
        names(&["r1"])
    );
}

// ---- NV-15: グループ分けと並べ替え ----

#[test]
fn test_nv_15_inherit_keeps_given_groups() {
    // [NV-15] Inherit は渡された groups(`.base` の groupBy の結果)のまま。
    let given = vec![("A".to_string(), 0..2), ("B".to_string(), 2..5)];
    let (rows, groups) = run_with_groups(&Settings::default(), given.clone());
    assert_eq!(ids(&rows), names(&["r1", "r2", "r3", "r4", "r5"]));
    assert_eq!(groups, given);
}

#[test]
fn test_nv_15_off_removes_groups() {
    // [NV-15] 「しない」を選ぶ → 見出しが消える(.base の groupBy より優先)。
    let given = vec![("A".to_string(), 0..2), ("B".to_string(), 2..5)];
    let s = Settings {
        group: Group::Off,
        ..Settings::default()
    };
    let (rows, groups) = run_with_groups(&s, given);
    assert!(groups.is_empty(), "見出しが残っている: {:?}", groups);
    assert_eq!(ids(&rows), names(&["r1", "r2", "r3", "r4", "r5"]));
}

#[test]
fn test_nv_15_group_by_column_values() {
    // [NV-15] 種別の列を選ぶ → 種別ごとの見出しと範囲。まとまりの中は元の順(安定)。
    let given = vec![("A".to_string(), 0..5)];
    let s = Settings {
        group: group_by("種別", Dir::Asc, false),
        ..Settings::default()
    };
    let (rows, groups) = run_with_groups(&s, given);
    let g = grouped(&rows, &groups);
    assert_eq!(g.len(), 3, "本・記事・空: {:?}", g);
    let find = |name: &str| {
        g.iter()
            .position(|(h, _)| h.contains(name))
            .unwrap_or_else(|| panic!("{} の見出しが無い: {:?}", name, g))
    };
    let book = find("本");
    let article = find("記事");
    assert_eq!(g[book].1, names(&["r1", "r3", "r5"]));
    assert_eq!(g[article].1, names(&["r2"]));
    assert!(book < article, "昇順で 本 が 記事 の前: {:?}", g);
    // 残りの1つは空のまとまり。
    let empty: Vec<_> = g
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != book && *i != article)
        .collect();
    assert_eq!(empty[0].1 .1, names(&["r4"]));
}

#[test]
fn test_nv_15_group_by_list_column_does_not_lose_rows() {
    // [NV-15] まとまりは全部の行を覆う(範囲が隙間なく続く)。
    let s = Settings {
        group: group_by("status", Dir::Asc, false),
        ..Settings::default()
    };
    let (rows, groups) = run(&s);
    let g = grouped(&rows, &groups);
    let mut all: Vec<String> = g.iter().flat_map(|(_, r)| r.clone()).collect();
    all.sort();
    assert_eq!(all, names(&["r1", "r2", "r3", "r4", "r5"]));
    let done = g
        .iter()
        .find(|(h, _)| h.contains("done"))
        .expect("done の見出し");
    assert_eq!(done.1, names(&["r2", "r5"]));
}

#[test]
fn test_nv_15_sort_by_number_asc_desc() {
    // [NV-15] 並べ替え: 数として並べ、型の合わない値は Asc でも Desc でも後ろ。
    assert_eq!(
        sorted(vec![("priority", Dir::Asc)]),
        names(&["r2", "r3", "r5", "r1", "r4"])
    );
    assert_eq!(
        sorted(vec![("priority", Dir::Desc)]),
        names(&["r1", "r5", "r3", "r2", "r4"])
    );
}

#[test]
fn test_nv_15_sort_by_date_puts_mismatched_and_empty_last() {
    // [NV-15] 日付の並べ替え: `someday` と空は後ろ(Desc でも)。
    let asc = sorted(vec![("due", Dir::Asc)]);
    assert_eq!(asc[..3].to_vec(), names(&["r1", "r5", "r2"]));
    let mut tail = asc[3..].to_vec();
    tail.sort();
    assert_eq!(tail, names(&["r3", "r4"]));

    let desc = sorted(vec![("due", Dir::Desc)]);
    assert_eq!(desc[..3].to_vec(), names(&["r2", "r5", "r1"]));
    let mut tail = desc[3..].to_vec();
    tail.sort();
    assert_eq!(tail, names(&["r3", "r4"]));
}

#[test]
fn test_nv_15_sort_is_stable_and_empty_last() {
    // [NV-15] 並べ替えは安定(同じ値は元の順)。空は Desc でも後ろ。
    assert_eq!(
        sorted(vec![("種別", Dir::Asc)]),
        names(&["r1", "r3", "r5", "r2", "r4"])
    );
    assert_eq!(
        sorted(vec![("種別", Dir::Desc)]),
        names(&["r2", "r1", "r3", "r5", "r4"])
    );
}

#[test]
fn test_nv_15_multiple_sorts() {
    // [NV-15] 複数の列で並べる: 種別 ↑ → priority ↓。
    assert_eq!(
        sorted(vec![("種別", Dir::Asc), ("priority", Dir::Desc)]),
        names(&["r1", "r5", "r3", "r2", "r4"])
    );
}

#[test]
fn test_nv_15_group_by_with_sort_inside_groups() {
    // [NV-15] まとまりの中は sorts の順。
    let s = Settings {
        sorts: vec![("priority".to_string(), Dir::Desc)],
        group: group_by("種別", Dir::Asc, false),
        ..Settings::default()
    };
    let (rows, groups) = run(&s);
    let g = grouped(&rows, &groups);
    let book = g
        .iter()
        .find(|(h, _)| h.contains("本"))
        .expect("本 の見出し");
    assert_eq!(book.1, names(&["r1", "r5", "r3"]));
}

#[test]
fn test_nv_15_filter_then_group() {
    // [NV-15] 絞った行だけをまとめる。
    let s = Settings {
        filters: vec![cond("status", Op::Drop(some(&["done"])))],
        group: group_by("種別", Dir::Asc, false),
        ..Settings::default()
    };
    let (rows, groups) = run(&s);
    let g = grouped(&rows, &groups);
    let mut all: Vec<String> = g.iter().flat_map(|(_, r)| r.clone()).collect();
    all.sort();
    assert_eq!(all, names(&["r1", "r3", "r4"]));
    assert!(
        g.iter().all(|(h, _)| !h.contains("記事")),
        "記事 の行は消えた: {:?}",
        g
    );
}

// ---- NV-21: 空のまとまりを隠す・まとまりの並び ----

#[test]
fn test_nv_21_hide_empty_group() {
    // [NV-21] status でグループ分けし、空を隠す → (空) の見出しと行が出ない。
    let s = Settings {
        group: group_by("status", Dir::Asc, true),
        ..Settings::default()
    };
    let (rows, groups) = run(&s);
    let g = grouped(&rows, &groups);
    assert!(
        !ids(&rows).contains(&"r3".to_string()),
        "空の行が残っている"
    );
    assert_eq!(g.len(), 3, "doing・done・todo: {:?}", g);
    assert!(g.iter().all(|(h, _)| !h.contains("(空)")), "{:?}", g);

    // 隠さなければ空のまとまりも出る。
    let s = Settings {
        group: group_by("status", Dir::Asc, false),
        ..Settings::default()
    };
    let (rows, groups) = run(&s);
    let g = grouped(&rows, &groups);
    assert_eq!(g.len(), 4, "{:?}", g);
    assert!(g.iter().any(|(_, r)| r == &names(&["r3"])), "{:?}", g);
}

#[test]
fn test_nv_21_group_dir_desc_reverses_groups() {
    // [NV-21] まとまりの並び: Asc は doing → done → todo、Desc は逆。
    let order = |dir: Dir| -> Vec<String> {
        let s = Settings {
            group: group_by("status", dir, true),
            ..Settings::default()
        };
        let (rows, groups) = run(&s);
        grouped(&rows, &groups)
            .into_iter()
            .map(|(_, r)| r.join(","))
            .collect()
    };
    assert_eq!(order(Dir::Asc), names(&["r4", "r2,r5", "r1"]));
    assert_eq!(order(Dir::Desc), names(&["r1", "r2,r5", "r4"]));
}

// ---- chips(NV-16 の帯の項目。is_default) ----

#[test]
fn test_nv_16_settings_default_and_chips() {
    // [NV-16] [NV-17] 既定の設定は is_default、帯の項目は無い。
    let d = Settings::default();
    assert!(d.is_default());
    assert!(d.chips().is_empty());
    assert_eq!(d.group, Group::Inherit);

    let s = Settings {
        filters: vec![cond("status", Op::Drop(some(&["done"])))],
        sorts: vec![("due".to_string(), Dir::Asc)],
        group: group_by("種別", Dir::Asc, false),
        ..Default::default()
    };
    assert!(!s.is_default());
    let chips = s.chips();
    assert_eq!(chips.len(), 3, "1条件 = 1項目: {:?}", chips);
    assert!(
        chips
            .iter()
            .any(|c| c.contains("status") && c.contains("done")),
        "{:?}",
        chips
    );
    assert!(chips.iter().any(|c| c.contains("due")), "{:?}", chips);
    assert!(chips.iter().any(|c| c.contains("種別")), "{:?}", chips);

    // グループ分けを「しない」も1項目。
    let off = Settings {
        group: Group::Off,
        ..Settings::default()
    };
    assert!(!off.is_default());
    assert_eq!(off.chips().len(), 1, "{:?}", off.chips());
}

// ---- NV-17: 見た目の状態に覚える ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-settings-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn list_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn full_settings() -> Settings {
    Settings {
        filters: vec![
            cond("status", Op::Drop(some(&["done"]))),
            cond("tags", Op::Keep(some(&["会議", "仕事"]))),
            cond("title", Op::Contains("Report".to_string())),
            cond("title", Op::NotContains("memo".to_string())),
            cond("priority", Op::Cmp(CmpOp::Ge, "3".to_string())),
            cond("due", Op::Cmp(CmpOp::Lt, "2026-10-01".to_string())),
            cond("種別", Op::NotEmpty),
            cond("status", Op::Empty),
        ],
        sorts: vec![
            ("due".to_string(), Dir::Asc),
            ("priority".to_string(), Dir::Desc),
        ],
        group: group_by("種別", Dir::Desc, true),
        ..Default::default()
    }
}

#[test]
fn test_nv_17_settings_round_trip_in_view_state() {
    // [NV-17] 設定は見た目の状態に覚え、次に同じものを開いたときに同じ値に戻る。
    let t = TempDir::new("roundtrip");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let mut st = load_state(&dir, &notes, "表");
    assert!(st.settings.is_default(), "保存が無いときは既定");
    st.order = names(&["title", "status"]);
    st.settings = full_settings();
    save_state(&dir, &notes, "表", &st).expect("保存できる");
    let back = load_state(&dir, &notes, "表");
    assert_eq!(back.settings, full_settings());
    assert_eq!(back.order, names(&["title", "status"]));
    // ノートのフォルダには書かない。
    assert!(list_files(&notes).is_empty());

    // グループの別の形も戻る。
    for group in [Group::Off, Group::Inherit] {
        let mut st = load_state(&dir, &notes, "表");
        st.settings.group = group.clone();
        save_state(&dir, &notes, "表", &st).expect("保存できる");
        assert_eq!(load_state(&dir, &notes, "表").settings.group, group);
    }
}

#[test]
fn test_nv_17_round_trip_keeps_empty_value_choice() {
    // [NV-17] 「(空)」(None)を選んだ条件も覚えて戻る。
    let t = TempDir::new("emptychoice");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let s = Settings {
        filters: vec![
            cond("status", Op::Keep(vec![None, Some("todo".to_string())])),
            cond("種別", Op::Drop(vec![None])),
        ],
        ..Settings::default()
    };
    let mut st = load_state(&dir, &notes, "表");
    st.settings = s.clone();
    save_state(&dir, &notes, "表", &st).expect("保存できる");
    assert_eq!(load_state(&dir, &notes, "表").settings, s);
}

#[test]
fn test_nv_17_settings_are_per_view() {
    // [NV-17] ビューごとに覚える。
    let t = TempDir::new("perview");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let mut st = load_state(&dir, &notes, "表");
    st.settings = full_settings();
    save_state(&dir, &notes, "表", &st).expect("保存できる");
    assert!(load_state(&dir, &notes, "別の表").settings.is_default());
    assert_eq!(load_state(&dir, &notes, "表").settings, full_settings());
}

#[test]
fn test_nv_17_old_state_file_without_settings_loads() {
    // [NV-17] settings のキーが無い古い形の状態のファイルも読め、settings は既定。
    let t = TempDir::new("oldform");
    let dir = t.mkdir("state");
    let notes = t.mkdir("notes");
    let mut st = load_state(&dir, &notes, "表");
    st.order = names(&["title", "status", "due"]);
    st.hidden = names(&["due"]);
    st.settings = full_settings();
    save_state(&dir, &notes, "表", &st).expect("保存できる");

    let files = list_files(&dir);
    assert_eq!(files.len(), 1, "状態のファイルは1つ: {:?}", files);
    let text = std::fs::read_to_string(&files[0]).unwrap();
    let mut table: toml::Table = toml::from_str(&text).expect("状態のファイルは TOML");
    assert!(
        table.contains_key("settings"),
        "settings のキーで書く: {}",
        text
    );
    table.remove("settings");
    assert!(!table.contains_key("settings"));
    let old = toml::to_string(&table).unwrap();
    std::fs::write(&files[0], old).unwrap();

    let back = load_state(&dir, &notes, "表");
    assert!(back.settings.is_default(), "settings は既定");
    assert_eq!(back.order, names(&["title", "status", "due"]));
    assert_eq!(back.hidden, names(&["due"]));
}
