//! 名前を付けた条件(specs/requirements/condition.md)と画面の式の絞り込み(navigation.expr-filter)の画面の試験。
//! 期待値は要求の受け入れの例から取った。条件は config.toml の文から読み、App に当て、式の読み取りにも置く。

use super::keymap::Action;
use super::test_grid::{labels, open};
use super::test_screen::{ch, make, screen, typing, Tmp};
use super::*;
use mdgrid::condition;
use mdgrid::config;
use mdgrid::types::parse_date;
use ratatui::crossterm::event::KeyCode;

/// due と status の5行(今日 2026-11-21 から: a は3日後・b は7日後・c は8日後・d は60日後・e は値なし)。
const NOTES: &[(&str, &str)] = &[
    ("a.md", "---\ndue: 2026-11-24\nstatus: todo\n---\n"),
    ("b.md", "---\ndue: 2026-11-28\nstatus: done\n---\n"),
    ("c.md", "---\ndue: 2026-11-29\nstatus: todo\n---\n"),
    ("d.md", "---\ndue: 2027-01-20\nstatus: todo\n---\n"),
    ("e.md", "---\nstatus: todo\n---\n"),
];

const CONF: &str = "\
[conditions.soon]
expr = 'due <= today() + \"7d\"'
label = \"期限が近い\"
key = 1
[conditions.mine]
expr = 'status == \"todo\"'
key = 2
";

/// 設定 `conf` を当てた App(今日は 2026-11-21)と、式の読み取りの条件の置き換え。
fn with_conf(a: &mut App, conf: &str) -> condition::Scoped {
    let (c, w) = config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let g = condition::scoped(c.conditions.clone());
    let warns = a.configure(&c);
    assert!(warns.is_empty(), "{warns:?}");
    a.today = parse_date("2026-11-21").unwrap();
    a.now = a.today * 86_400;
    a.refresh();
    g
}

fn names(a: &App) -> Vec<String> {
    let mut v: Vec<String> = a
        .rows
        .iter()
        .map(|r| a.src.label(r).trim_end_matches(".md").to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn test_condition_digit_key_toggles_the_rows() {
    // [condition.digit-keys] `1` で soon の行だけ、もう一度で全部。条件の無い `3` は何もしない。
    let (_t, mut a) = make("cond-digit", NOTES);
    let _g = with_conf(&mut a, CONF);
    ch(&mut a, '1');
    assert_eq!(names(&a), ["a", "b"]);
    ch(&mut a, '1');
    assert_eq!(names(&a).len(), 5);
    ch(&mut a, '3');
    assert_eq!(names(&a).len(), 5);
}

#[test]
fn test_condition_digit_stack_shows_items_and_narrows_both() {
    // [utest -> req~condition.digit-stack~2] 2つ入れると帯に「1 期限が近い」「2 mine」が出て、両方に当たる行だけ。
    let (_t, mut a) = make("cond-stack", NOTES);
    let _g = with_conf(&mut a, CONF);
    ch(&mut a, '1');
    ch(&mut a, '2');
    let chips = a.bar_chips();
    assert!(chips.contains(&"1 期限が近い".to_string()), "{chips:?}");
    assert!(chips.contains(&"2 mine".to_string()), "{chips:?}");
    assert_eq!(names(&a), ["a"]);
    assert!(screen(&a).contains("[1 期限が近い]"), "{}", screen(&a));
    ch(&mut a, '1');
    assert!(!a.bar_chips().contains(&"1 期限が近い".to_string()));
    assert_eq!(names(&a), ["a", "c", "d", "e"]);
}

#[test]
fn test_condition_digit_lifetime_keeps_across_views() {
    // [utest -> req~condition.digit-lifetime~2] ビューを移っても数字のキーの条件は入れたまま。
    let tmp = Tmp::new("cond-views");
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let base = "views:\n  - type: table\n    name: 進行中\n  - type: table\n    name: 全部\n";
    let mut a = open(&tmp, base, None);
    let _g = with_conf(&mut a, CONF);
    ch(&mut a, '1');
    assert_eq!(labels(&a).len(), 2);
    a.apply(Action::NextView);
    assert_eq!(a.view_index(), 1);
    assert_eq!(labels(&a).len(), 2, "全部のビューでも soon で絞ったまま");
}

#[test]
fn test_condition_digit_hints_in_help() {
    // [utest -> req~condition.digit-hints~2] ヘルプに数字と見せる名前。条件の無い数字は出さない。
    let (_t, mut a) = make("cond-help", NOTES);
    let _g = with_conf(&mut a, CONF);
    let help: Vec<String> = super::help::help_lines(&a, super::keymap::Mode::Table, 120)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    let text = help.join("\n");
    assert!(
        help.iter()
            .any(|l| l.contains("1 ") && l.contains("期限が近い")),
        "{text}"
    );
    assert!(
        !help.iter().any(|l| l.trim_start().starts_with("3 ")),
        "{text}"
    );
    assert!(!text.contains("名前の条件の入れ切り"), "{text}");
}

#[test]
fn test_condition_digit_remap_wins_and_warns() {
    // [utest -> req~condition.digit-remap~1] `1` を割り当て直すと、その動作が効き、soon の key を警告する。
    let (_t, mut a) = make("cond-remap", NOTES);
    let conf = format!("{CONF}[keys.table]\n\"1\" = \"search\"\n");
    let (c, _) = config::parse(&conf).unwrap();
    let _g = condition::scoped(c.conditions.clone());
    let warns = a.configure(&c);
    assert!(
        warns.iter().any(|w| w.contains("conditions.soon.key")),
        "{warns:?}"
    );
    ch(&mut a, '1');
    assert_eq!(a.mode, super::keymap::Mode::Search);
    assert_eq!(names(&a).len(), 5);
}

#[test]
fn test_condition_hide_always_hides_and_counts() {
    // [condition.hide-always] hide の条件に当たる行は初めから出ず、帯に隠した数。
    let (_t, mut a) = make("cond-hide", NOTES);
    let conf = "[conditions.far]\nexpr = 'due > today() + \"30d\"'\nhide = true\nkey = 9\n";
    let _g = with_conf(&mut a, conf);
    assert_eq!(names(&a), ["a", "b", "c", "e"]);
    assert!(
        a.bar_chips().contains(&"far を隠す 1".to_string()),
        "{:?}",
        a.bar_chips()
    );
}

#[test]
fn test_condition_hide_always_respects_the_exception() {
    // [condition.hide-always] 始まりが先でも期限が近い行は出る(受け入れの例の式)。
    let notes = [
        ("x.md", "---\nstart: 2027-01-20\n---\n"),
        ("y.md", "---\nstart: 2027-01-20\ndue: 2026-11-24\n---\n"),
    ];
    let (_t, mut a) = make("cond-hide-ex", &notes);
    let conf = "[conditions.far]\nexpr = 'start > today() + \"30d\" && !(due <= today() + \"7d\")'\nhide = true\n";
    let _g = with_conf(&mut a, conf);
    assert_eq!(names(&a), ["y"]);
}

#[test]
fn test_condition_hide_toggle_shows_and_hides_again() {
    // [utest -> req~condition.hide-toggle~2] `9` で隠した行が出て帯が「far を出す」、もう一度で隠れる。
    let (_t, mut a) = make("cond-hide-key", NOTES);
    let conf = "[conditions.far]\nexpr = 'due > today() + \"30d\"'\nhide = true\nkey = 9\n";
    let _g = with_conf(&mut a, conf);
    ch(&mut a, '9');
    assert_eq!(names(&a).len(), 5);
    assert!(
        a.bar_chips().contains(&"far を出す".to_string()),
        "{:?}",
        a.bar_chips()
    );
    ch(&mut a, '9');
    assert_eq!(names(&a).len(), 4);
}

#[test]
fn test_condition_hide_everywhere_on_kanban() {
    // [utest -> req~condition.hide-everywhere~1] カンバンの札にも hide の条件の行は無い。
    let (_t, mut a) = make("cond-hide-kanban", NOTES);
    let conf = "[conditions.far]\nexpr = 'due > today() + \"30d\"'\nhide = true\n";
    let _g = with_conf(&mut a, conf);
    a.apply(Action::KanbanView);
    assert_eq!(a.mode, super::keymap::Mode::Kanban, "{:?}", a.message);
    let s = screen(&a);
    assert!(!s.contains("d.md") && !s.contains(" d "), "{s}");
    assert!(names(&a).iter().all(|n| n != "d"));
}

#[test]
fn test_condition_any_table_without_the_column() {
    // [utest -> req~condition.any-table~1] due の無い表で soon → 誤りも警告も無く、どの行も当たらない。
    let notes = [
        ("p.md", "---\ntitle: p\n---\n"),
        ("q.md", "---\ntitle: q\n---\n"),
    ];
    let (_t, mut a) = make("cond-any", &notes);
    let _g = with_conf(&mut a, CONF);
    ch(&mut a, '1');
    assert!(names(&a).is_empty());
    assert!(a.view_error.is_none());
    assert!(a.message.is_none(), "{:?}", a.message);
}

#[test]
fn test_condition_view_settings_hide_named() {
    // [utest -> req~condition.view-settings~1] ビューの設定のフィルターの区画で mine を「隠す」→ mine の行が消える。
    let (_t, mut a) = make("cond-view-settings", NOTES);
    let _g = with_conf(&mut a, CONF);
    a.apply(Action::ViewSettings);
    let d = a.draft.as_mut().unwrap();
    assert_eq!(
        d.conds.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        ["soon", "mine"]
    );
    d.cycle_named("mine");
    d.cycle_named("mine");
    a.draft.as_mut().unwrap().sec = super::settings::Sec::Buttons;
    a.draft.as_mut().unwrap().sel[super::settings::Sec::Buttons as usize] = 0;
    a.apply(Action::Run);
    assert_eq!(names(&a), ["b"]);
    assert!(
        a.bar_chips().contains(&"mine を隠す".to_string()),
        "{:?}",
        a.bar_chips()
    );
}

#[test]
fn test_expr_filter_from_the_palette() {
    // [navigation.expr-filter] パレットの「式で絞り込む」で式を打つと、当たる行だけになり、帯に式。
    let (t, mut a) = make("expr-filter", NOTES);
    let _g = with_conf(&mut a, CONF);
    let before = std::fs::read_dir(t.notes()).unwrap().count();
    a.apply(Action::ExprFilter);
    typing(&mut a, "status != \"done\" && cond(\"soon\")");
    super::test_screen::press(&mut a, KeyCode::Enter);
    assert_eq!(names(&a), ["a"]);
    assert!(
        a.bar_chips()
            .contains(&"式: status != \"done\" && cond(\"soon\")".to_string()),
        "{:?}",
        a.bar_chips()
    );
    assert_eq!(std::fs::read_dir(t.notes()).unwrap().count(), before);
}

#[test]
fn test_expr_filter_error_keeps_rows_and_text() {
    // [utest -> req~navigation.expr-filter-error~2] 読めない式は行を変えず、理由を出し、入力の欄に式を残す。
    let (_t, mut a) = make("expr-filter-bad", NOTES);
    let _g = with_conf(&mut a, CONF);
    a.apply(Action::ExprFilter);
    typing(&mut a, "due <=");
    super::test_screen::press(&mut a, KeyCode::Enter);
    assert_eq!(names(&a).len(), 5);
    assert!(a.message.is_some());
    assert_eq!(a.palette.as_ref().map(|p| p.query.as_str()), Some("due <="));
    // cond の無い名前は condition.unknown-name と同じ理由。
    super::test_screen::press(&mut a, KeyCode::Esc);
    a.apply(Action::ExprFilter);
    typing(&mut a, "cond(\"nosuch\")");
    super::test_screen::press(&mut a, KeyCode::Enter);
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains("条件「nosuch」はありません(ある名前: mine, soon)"),
        "{m}"
    );
}

#[test]
fn test_expr_filter_lifetime_chip_removal() {
    // [utest -> req~navigation.expr-filter-lifetime~2] 帯の式の項目を外すと元の行。
    let (_t, mut a) = make("expr-filter-chip", NOTES);
    let _g = with_conf(&mut a, CONF);
    a.apply(Action::ExprFilter);
    typing(&mut a, "status == \"done\"");
    super::test_screen::press(&mut a, KeyCode::Enter);
    assert_eq!(names(&a), ["b"]);
    let i = a
        .bar_chips()
        .iter()
        .position(|c| c.starts_with("式:"))
        .unwrap();
    a.remove_chip(i);
    a.refresh();
    assert_eq!(names(&a).len(), 5);
}
