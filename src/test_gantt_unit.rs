//! ガント(GA-1・GA-2・GA-3・GA-7)の、画面に依存しない部分。値は作ったもの。
//! 日付は 1970-01-01 からの日数(`types::parse_date`)。2026-10-01 は木曜、2026-10-05 は月曜。

use super::*;

fn d(s: &str) -> i64 {
    crate::types::parse_date(s).unwrap()
}

/// `infer` に渡す列の並び(名前, 日付の列か)。
fn cols(names: &[(&str, bool)]) -> Vec<(String, bool)> {
    names
        .iter()
        .map(|(n, date)| (n.to_string(), *date))
        .collect()
}

fn spec(start: Option<&str>, end: Option<&str>, days: Option<&str>) -> GanttSpec {
    GanttSpec {
        start: start.map(str::to_string),
        end: end.map(str::to_string),
        days: days.map(str::to_string),
    }
}

#[test]
fn test_ga_1_infer_start_and_end_from_column_names() {
    // [utest -> req~gantt.config-infer~1] 設定が無いときは日付の列の名前から推定する: start と due → 両方。大文字小文字を問わず、列の名前はそのまま。
    assert_eq!(
        infer(&cols(&[("title", false), ("start", true), ("due", true)])),
        Some(spec(Some("start"), Some("due"), None))
    );
    assert_eq!(
        infer(&cols(&[("Start", true), ("DUE", true)])),
        Some(spec(Some("Start"), Some("DUE"), None))
    );
    // 日本語の名前(開始・期限・終了)と、ほかの英語の名前(begin・from / end・finish・until・to)。
    assert_eq!(
        infer(&cols(&[("開始", true), ("期限", true)])),
        Some(spec(Some("開始"), Some("期限"), None))
    );
    assert_eq!(
        infer(&cols(&[("begin", true), ("finish", true)])),
        Some(spec(Some("begin"), Some("finish"), None))
    );
    assert_eq!(
        infer(&cols(&[("from", true), ("to", true)])),
        Some(spec(Some("from"), Some("to"), None))
    );
}

#[test]
fn test_ga_1_infer_end_only_and_none() {
    // [utest -> req~gantt.config-infer~1, req~gantt.config-infer-fail~1, req~gantt.config-missing-column~1, req~gantt.config-empty-table~1] Due だけ → 始まりは None(終わりの日に ◆ を置く行だけの画面)。日付でない列は無視。
    // 始まりも終わりも無ければ None(理由を出してガントを出さない)。
    assert_eq!(
        infer(&cols(&[("title", false), ("Due", true)])),
        Some(spec(None, Some("Due"), None))
    );
    assert_eq!(
        infer(&cols(&[("start", true), ("due", false)])),
        Some(spec(Some("start"), None, None))
    );
    assert_eq!(infer(&cols(&[("start", false), ("due", false)])), None);
    assert_eq!(infer(&cols(&[("created", true), ("title", false)])), None);
    assert_eq!(infer(&[]), None);
}

#[test]
fn test_ga_1_bar_from_values_end_or_days() {
    // [utest -> req~gantt.config-days~1] end があればそれ、無ければ days(始まりからの日数)で start+days-1。days だけでは棒にならない。
    let (s, e) = (d("2026-10-01"), d("2026-10-07"));
    let bar = Bar::from_values(Some(s), Some(e), None);
    assert_eq!((bar.start, bar.end), (Some(s), Some(e)));
    assert!(!bar.is_point());
    let bar = Bar::from_values(Some(s), None, Some(3));
    assert_eq!((bar.start, bar.end), (Some(s), Some(s + 2)));
    // end と days の両方があれば end。
    let bar = Bar::from_values(Some(s), Some(e), Some(3));
    assert_eq!((bar.start, bar.end), (Some(s), Some(e)));
    // days だけ(始まりが無い)。
    let bar = Bar::from_values(None, None, Some(3));
    assert_eq!((bar.start, bar.end), (None, None));
    assert!(!bar.is_point());
    // 片方だけは印(◆)。
    assert!(Bar::from_values(None, Some(e), None).is_point());
    assert!(Bar::from_values(Some(s), None, None).is_point());
    assert!(!Bar::from_values(None, None, None).is_point());
}

#[test]
fn test_ga_3_bar_span_of_children() {
    // [utest -> req~gantt.tree-wbs-parent-range~1] 親の棒は子孫の始まりの最小から終わりの最大まで。片方だけの子はその日。子が無ければ棒なし。
    let b = |s: &str, e: &str| Bar::from_values(Some(d(s)), Some(d(e)), None);
    let span = Bar::span(&[b("2026-10-05", "2026-10-07"), b("2026-10-01", "2026-10-03")]);
    assert_eq!(
        (span.start, span.end),
        (Some(d("2026-10-01")), Some(d("2026-10-07")))
    );
    let span = Bar::span(&[
        b("2026-10-01", "2026-10-03"),
        Bar::from_values(None, Some(d("2026-10-07")), None),
    ]);
    assert_eq!(
        (span.start, span.end),
        (Some(d("2026-10-01")), Some(d("2026-10-07")))
    );
    let span = Bar::span(&[]);
    assert_eq!((span.start, span.end), (None, None));
    let span = Bar::span(&[Bar::from_values(None, None, None)]);
    assert_eq!((span.start, span.end), (None, None));
}

#[test]
fn test_ga_2_fit_unit_finest_that_fits() {
    // [utest -> req~gantt.open-unit~1] 開くときは範囲が幅に収まる最も細かい単位: 7日・幅 40 → 日。400 日・幅 100 → 週か月(日ではない)。
    // 200 日・幅 20 → 月(29 週は入らない)。400 日でも幅 1000 なら日。
    let o = d("2026-10-01");
    assert_eq!(fit_unit(o, o + 6, 40), Unit::Day);
    assert!(matches!(
        fit_unit(o, o + 399, 100),
        Unit::Week | Unit::Month
    ));
    assert_eq!(fit_unit(o, o + 199, 20), Unit::Month);
    assert_eq!(fit_unit(o, o + 399, 1000), Unit::Day);
    // 単位の細かく・粗く(端で止まる)。
    assert_eq!(Unit::Day.coarser(), Unit::Week);
    assert_eq!(Unit::Week.coarser(), Unit::Month);
    assert_eq!(Unit::Month.coarser(), Unit::Month);
    assert_eq!(Unit::Month.finer(), Unit::Week);
    assert_eq!(Unit::Week.finer(), Unit::Day);
    assert_eq!(Unit::Day.finer(), Unit::Day);
}

#[test]
fn test_ga_2_floor_unit_week_monday_month_first() {
    // [utest -> req~gantt.open-bars~1, req~gantt.open-today~1] 週のマスは月曜から、月のマスは1日から。日はそのまま。
    assert_eq!(floor_unit(d("2026-10-06"), Unit::Day), d("2026-10-06"));
    assert_eq!(floor_unit(d("2026-10-06"), Unit::Week), d("2026-10-05"));
    assert_eq!(floor_unit(d("2026-10-05"), Unit::Week), d("2026-10-05"));
    assert_eq!(floor_unit(d("2026-10-04"), Unit::Week), d("2026-09-28"));
    assert_eq!(floor_unit(d("2026-10-06"), Unit::Month), d("2026-10-01"));
    assert_eq!(floor_unit(d("2026-10-01"), Unit::Month), d("2026-10-01"));
    assert_eq!(floor_unit(d("2026-12-31"), Unit::Month), d("2026-12-01"));
}

#[test]
fn test_ga_2_cell_of_and_cell_start_round_trip() {
    // [utest -> req~gantt.open-bars~1] マスと日の往復: 日は1日1マス、週は月曜ごと、月は1日ごと。原点より前は負のマス。
    let o = d("2026-10-05");
    assert_eq!(cell_of(o + 5, o, Unit::Day), 5);
    assert_eq!(cell_start(5, o, Unit::Day), o + 5);
    assert_eq!(cell_of(d("2026-10-11"), o, Unit::Week), 0);
    assert_eq!(cell_of(d("2026-10-12"), o, Unit::Week), 1);
    assert_eq!(cell_start(1, o, Unit::Week), d("2026-10-12"));
    assert_eq!(cell_of(d("2026-10-04"), o, Unit::Week), -1);
    let m = d("2026-10-01");
    assert_eq!(cell_of(d("2026-10-31"), m, Unit::Month), 0);
    assert_eq!(cell_of(d("2026-11-15"), m, Unit::Month), 1);
    assert_eq!(cell_start(2, m, Unit::Month), d("2026-12-01"));
    assert_eq!(cell_start(3, m, Unit::Month), d("2027-01-01"));
    assert_eq!(cell_of(d("2026-09-30"), m, Unit::Month), -1);
    for unit in [Unit::Day, Unit::Week, Unit::Month] {
        let origin = floor_unit(o, unit);
        for k in -3..=3 {
            assert_eq!(
                cell_of(cell_start(k, origin, unit), origin, unit),
                k,
                "{unit:?} {k}"
            );
        }
    }
}

/// 印のある一時フォルダ(根)と、その下の表のフォルダ `notes`。
struct Root(std::path::PathBuf);

impl Root {
    fn new(tag: &str, marker: &str) -> Root {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-gantt-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(dir.join("notes")).unwrap();
        std::fs::create_dir_all(dir.join(".mdgrid")).unwrap();
        std::fs::write(dir.join(".mdgrid/workspace.toml"), marker).unwrap();
        Root(dir)
    }

    fn notes(&self) -> std::path::PathBuf {
        self.0.join("notes")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const MARKER_HEAD: &str = "[[table]]\nname = \"T\"\npath = \"notes\"\n";

#[test]
fn test_ga_1_marker_gantt_reads_start_and_end() {
    // [utest -> req~gantt.config-start-end~1, req~gantt.config-infer-fail~1, req~gantt.config-empty-table~1] 印の [[table]] の gantt = { start, end } を読む。警告は無く、範囲の読みでも知らない項目にならない。
    let r = Root::new(
        "read",
        &format!("{MARKER_HEAD}gantt = {{ start = \"start\", end = \"due\" }}\n"),
    );
    let (got, warns) = crate::workspace::marker_gantt(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (got, at) = got.expect("gantt");
    assert_eq!(got, spec(Some("start"), Some("due"), None));
    assert_eq!(at, "table[1].gantt");
    let (_, warns) = crate::workspace::read_marker(&r.0);
    assert!(warns.is_empty(), "{warns:?}");
    // 別の表(gantt の無い表)には設定が無い。
    std::fs::create_dir_all(r.0.join("other")).unwrap();
    let (got, warns) = crate::workspace::marker_gantt(&r.0.join("other"));
    assert!(got.is_none());
    assert!(warns.is_empty(), "{warns:?}");
}

#[test]
fn test_ga_1_marker_gantt_days_instead_of_end() {
    // [utest -> req~gantt.config-days~1] end の代わりに days(始まりからの日数の列)でもよい。
    let r = Root::new(
        "days",
        &format!("{MARKER_HEAD}gantt = {{ start = \"start\", days = \"len\" }}\n"),
    );
    let (got, warns) = crate::workspace::marker_gantt(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (got, _) = got.expect("gantt");
    assert_eq!(got, spec(Some("start"), None, Some("len")));
}

#[test]
fn test_ga_1_marker_gantt_warns_and_none_when_malformed() {
    // [utest -> req~gantt.config-days~1, req~gantt.config-missing-column~1, req~gantt.config-empty-table~1] start が無い・end も days も無い・表でない → CLI-3 の形の警告1行で、設定は None。
    use crate::i18n::Msg;
    for (bad, at, reason) in [
        (
            "gantt = { end = \"due\" }",
            "table[1].gantt",
            Msg::RsnGanttNeedStart.text(),
        ),
        (
            "gantt = { start = \"start\" }",
            "table[1].gantt",
            Msg::RsnGanttNeedEnd.text(),
        ),
        (
            "gantt = \"x\"",
            "table[1].gantt: ",
            Msg::RsnGanttNotTable.text(),
        ),
    ] {
        let r = Root::new("bad", &format!("{MARKER_HEAD}{bad}\n"));
        let (got, warns) = crate::workspace::marker_gantt(&r.notes());
        assert!(got.is_none(), "{bad}");
        assert_eq!(warns.len(), 1, "{bad}: {warns:?}");
        assert!(
            warns[0].starts_with(".mdgrid/workspace.toml: ") && warns[0].contains(at),
            "{bad}: {warns:?}"
        );
        assert!(warns[0].ends_with(reason), "{bad}: {warns:?}");
    }
    // 知らないキーは警告して、設定は読む。
    let r = Root::new(
        "unknown",
        &format!("{MARKER_HEAD}gantt = {{ start = \"start\", end = \"due\", foo = 1 }}\n"),
    );
    let (got, warns) = crate::workspace::marker_gantt(&r.notes());
    assert!(got.is_some());
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(
        warns[0].contains("table[1].gantt.foo: ")
            && warns[0].ends_with(crate::i18n::Msg::RsnUnknown.text()),
        "{warns:?}"
    );
}

#[test]
fn test_ga_7_tree_and_gantt_read_in_the_same_shape() {
    // [utest -> req~gantt.screen-config~2] 画面の型ごとの設定は [[table]] に型の名前のキーで同じ形。tree と gantt の両方がある表は、
    // marker_tree と marker_gantt の両方が読め、警告は無い。
    let r = Root::new(
        "both",
        &format!(
            "{MARKER_HEAD}tree = {{ path = \"folder\" }}\ngantt = {{ start = \"start\", end = \"due\" }}\n"
        ),
    );
    let (tree, warns) = crate::workspace::marker_tree(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (tree, at) = tree.expect("tree");
    assert_eq!(tree.path, "folder");
    assert_eq!(at, "table[1].tree");
    let (gantt, warns) = crate::workspace::marker_gantt(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (gantt, at) = gantt.expect("gantt");
    assert_eq!(gantt, spec(Some("start"), Some("due"), None));
    assert_eq!(at, "table[1].gantt");
    let (_, warns) = crate::workspace::read_marker(&r.0);
    assert!(warns.is_empty(), "{warns:?}");
    // 2つ目の表の設定は table[2]。1つ目の表の設定は2つ目に漏れない。
    let r = Root::new(
        "second",
        &format!(
            "[[table]]\nname = \"A\"\npath = \"other\"\ngantt = {{ start = \"a\", end = \"b\" }}\n\n{MARKER_HEAD}gantt = {{ start = \"start\", end = \"due\" }}\n"
        ),
    );
    std::fs::create_dir_all(r.0.join("other")).unwrap();
    let (gantt, _) = crate::workspace::marker_gantt(&r.notes());
    let (gantt, at) = gantt.expect("gantt");
    assert_eq!(gantt.start.as_deref(), Some("start"));
    assert_eq!(at, "table[2].gantt");
}

#[test]
fn test_overrun_counts_days_before_the_start_and_after_the_end() {
    // 子が親の期間からはみ出す日数は、始まりより前の日数と終わりより後の日数の和で、中に収まれば 0。
    // [utest -> req~gantt.children-span-mark~1]
    let parent = (d("2026-11-01"), d("2026-11-10"));
    assert_eq!(overrun(parent, (d("2026-11-05"), d("2026-11-12"))), 2);
    assert_eq!(overrun(parent, (d("2026-11-05"), d("2026-11-11"))), 1);
    assert_eq!(overrun(parent, (d("2026-11-05"), d("2026-11-10"))), 0);
    assert_eq!(overrun(parent, (d("2026-11-01"), d("2026-11-01"))), 0);
    assert_eq!(overrun(parent, (d("2026-10-31"), d("2026-11-11"))), 2);
}

#[test]
fn test_marker_children_reads_the_keys_and_defaults() {
    // 印の children を読む。key だけなら既定(done・["done"]・complete・mark)で、警告は無く、範囲の読みでも知らない項目にならない。
    // [utest -> req~navigation.parent-child-config~1, req~cell-edit.children-done~1, req~gantt.children-span-mark~1]
    use crate::tree::{ChildrenSpec, OnDone, SpanRule};
    let r = Root::new(
        "children",
        &format!("{MARKER_HEAD}children = {{ key = \"parent\" }}\n"),
    );
    let (got, warns) = crate::workspace::marker_children(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (got, at) = got.expect("children");
    assert_eq!(got, ChildrenSpec::new("parent"));
    assert_eq!(at, "table[1].children");
    let (_, warns) = crate::workspace::read_marker(&r.0);
    assert!(warns.is_empty(), "{warns:?}");
    let r = Root::new(
        "children-all",
        &format!(
            "{MARKER_HEAD}children = {{ key = \"up\", status = \"state\", done = \"完了\", closed = [\"完了\", \"中止\"], on_done = \"refuse\", gantt = \"extend\" }}\n"
        ),
    );
    let (got, warns) = crate::workspace::marker_children(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    let (got, _) = got.expect("children");
    assert_eq!(
        got,
        ChildrenSpec {
            key: "up".into(),
            status: Some("state".into()),
            done: "完了".into(),
            closed: vec!["完了".into(), "中止".into()],
            on_done: OnDone::Refuse,
            gantt: SpanRule::Extend,
        }
    );
}

#[test]
fn test_marker_children_warns_in_place_and_falls_back() {
    // 書き方の違う children は場所を言う警告1行。key が無いか表でなければ設定を使わず、値の違う項目は既定にする。
    // [utest -> req~cell-edit.children-done~1, req~gantt.children-span-mark~1]
    use crate::tree::{OnDone, SpanRule};
    for (bad, at) in [
        ("children = \"parent\"", "table[1].children: "),
        ("children = { status = \"s\" }", "table[1].children.key: "),
        ("children = { key = 1 }", "table[1].children.key: "),
    ] {
        let r = Root::new("children-bad", &format!("{MARKER_HEAD}{bad}\n"));
        let (got, warns) = crate::workspace::marker_children(&r.notes());
        assert!(got.is_none(), "{bad}");
        assert_eq!(warns.len(), 1, "{bad}: {warns:?}");
        assert!(
            warns[0].starts_with(&format!(".mdgrid/workspace.toml: {at}")),
            "{bad}: {warns:?}"
        );
    }
    for (bad, at) in [
        ("on_done = \"nope\"", "table[1].children.on_done: "),
        ("gantt = \"wide\"", "table[1].children.gantt: "),
        ("closed = \"done\"", "table[1].children.closed: "),
        ("done = 1", "table[1].children.done: "),
        ("color = \"red\"", "table[1].children.color: "),
    ] {
        let r = Root::new(
            "children-val",
            &format!("{MARKER_HEAD}children = {{ key = \"parent\", {bad} }}\n"),
        );
        let (got, warns) = crate::workspace::marker_children(&r.notes());
        let (got, _) = got.expect(bad);
        assert_eq!(warns.len(), 1, "{bad}: {warns:?}");
        assert!(
            warns[0].starts_with(&format!(".mdgrid/workspace.toml: {at}")),
            "{bad}: {warns:?}"
        );
        assert_eq!(got.on_done, OnDone::Complete, "{bad}");
        assert_eq!(got.gantt, SpanRule::Mark, "{bad}");
        assert_eq!(got.done, "done", "{bad}");
        assert_eq!(got.closed, ["done"], "{bad}");
    }
}

#[test]
fn test_marker_children_reopen_reads_closed_by_and_on_reopen() {
    // 印の children の closed_by と on_reopen を読む。書かなければ列なし(既定の名前は無い)・keep で、警告は無い。
    // [utest -> req~cell-edit.children-closed-by~1, req~cell-edit.children-reopen~1]
    use crate::tree::{OnReopen, ReopenSpec};
    let r = Root::new(
        "reopen-none",
        &format!("{MARKER_HEAD}children = {{ key = \"parent\" }}\n"),
    );
    let (got, warns) = crate::workspace::marker_children_reopen(&r.notes());
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got.expect("children").0, ReopenSpec::default());
    assert_eq!(ReopenSpec::default().closed_by, None);
    assert_eq!(ReopenSpec::default().on_reopen, OnReopen::Keep);
    for (how, want) in [
        ("keep", OnReopen::Keep),
        ("reopen", OnReopen::Reopen),
        ("ask", OnReopen::Ask),
    ] {
        let r = Root::new(
            "reopen-all",
            &format!(
                "{MARKER_HEAD}children = {{ key = \"parent\", closed_by = \"closed_by\", on_reopen = \"{how}\" }}\n"
            ),
        );
        let (got, warns) = crate::workspace::marker_children_reopen(&r.notes());
        assert!(warns.is_empty(), "{how}: {warns:?}");
        let got = got.expect(how).0;
        assert_eq!(got.closed_by.as_deref(), Some("closed_by"), "{how}");
        assert_eq!(got.on_reopen, want, "{how}");
        // 親子の設定の読みでも、知らない項目にならない。
        let (_, warns) = crate::workspace::marker_children(&r.notes());
        assert!(warns.is_empty(), "{how}: {warns:?}");
    }
}

#[test]
fn test_marker_children_reopen_warns_in_place_and_keeps() {
    // 書き方の違う closed_by・on_reopen と、closed_by の無い reopen・ask は、場所を言う警告1行で keep にする。
    // [utest -> req~cell-edit.children-reopen~1]
    use crate::tree::OnReopen;
    for (bad, at) in [
        ("on_reopen = \"nope\"", "table[1].children.on_reopen: "),
        ("on_reopen = 1", "table[1].children.on_reopen: "),
        ("on_reopen = \"reopen\"", "table[1].children.on_reopen: "),
        ("on_reopen = \"ask\"", "table[1].children.on_reopen: "),
        (
            "closed_by = 1, on_reopen = \"keep\"",
            "table[1].children.closed_by: ",
        ),
    ] {
        let r = Root::new(
            "reopen-bad",
            &format!("{MARKER_HEAD}children = {{ key = \"parent\", {bad} }}\n"),
        );
        let (got, warns) = crate::workspace::marker_children_reopen(&r.notes());
        let (got, _) = got.expect(bad);
        assert_eq!(warns.len(), 1, "{bad}: {warns:?}");
        assert!(
            warns[0].starts_with(&format!(".mdgrid/workspace.toml: {at}")),
            "{bad}: {warns:?}"
        );
        assert_eq!(got.on_reopen, OnReopen::Keep, "{bad}");
    }
}
