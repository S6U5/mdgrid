//! 台帳の印に書く名前の条件・通知の欄・知らせと、数字のキーの振り方・使う列の無い条件の画面の試験
//! (specs/requirements/condition.md の condition.digit-order・condition.columns-present、notice.md の
//! notice.columns-present、notify.md の notify.marker・notify.columns-present)。期待値は要求の受け入れの例から取った。
//! 印はノートのフォルダに置き、App::start と同じく `config::for_table` で全体の設定に重ねて当てる。

use super::help::help_lines;
use super::keymap::{Action, Mode};
use super::notify::TestOutlet;
use super::test_screen::{ch, make, Tmp};
use super::*;
use mdgrid::condition;
use mdgrid::config;
use mdgrid::types::parse_date;
use mdgrid::workspace::{Scope, Source, WsTable};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 全体の設定 `conf` と、ノートのフォルダに置いた印 `marker`(空なら置かない)を重ねて当てる(今日は 2026-11-21)。
/// 重ねたときと App に当てたときの警告を返す。
fn with_ledger(a: &mut App, t: &Tmp, conf: &str, marker: &str) -> (condition::Scoped, Vec<String>) {
    let (c, w) = config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    if !marker.is_empty() {
        std::fs::create_dir_all(t.notes().join(".mdgrid")).unwrap();
        std::fs::write(t.notes().join(".mdgrid/workspace.toml"), marker).unwrap();
    }
    let (c, mut warns) = config::for_table(&c, &t.notes(), None);
    let g = condition::scoped(c.conditions.clone());
    a.today = parse_date("2026-11-21").unwrap();
    a.now = a.today * 86_400;
    warns.extend(a.configure(&c));
    a.refresh();
    (g, warns)
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

const DUE_WEEK: &str = "\
[conditions.due]
expr = 'due <= today()'
key = 1
[conditions.week]
expr = 'due <= today() + \"7d\"'
key = 2
";

const CONTACT5: &str = "[conditions.contact]\nexpr = 'next <= today()'\nkey = 5\n";

#[test]
fn test_digit_order_marker_first_then_global() {
    // [utest -> req~condition.digit-order~1] 印の contact(key = 5)が 1、全体の due が 2、week が 3。帯に「1 contact」。
    let (t, mut a) = make(
        "ledger-digit-order",
        &[
            ("x.md", "---\nnext: 2026-11-21\ndue: 2026-12-20\n---\n"),
            ("y.md", "---\nnext: 2026-11-30\ndue: 2026-11-01\n---\n"),
            ("z.md", "---\nnext: 2026-11-30\ndue: 2026-11-25\n---\n"),
        ],
    );
    let (_g, w) = with_ledger(&mut a, &t, DUE_WEEK, CONTACT5);
    assert!(w.is_empty(), "{w:?}");
    ch(&mut a, '1');
    assert_eq!(names(&a), ["x"]);
    assert!(
        a.bar_chips().contains(&"1 contact".to_string()),
        "{:?}",
        a.bar_chips()
    );
    ch(&mut a, '1');
    ch(&mut a, '2');
    assert_eq!(names(&a), ["y"]);
    ch(&mut a, '2');
    ch(&mut a, '3');
    assert_eq!(names(&a), ["y", "z"]);
}

#[test]
fn test_digit_order_skips_conditions_without_columns() {
    // [utest -> req~condition.digit-order~1] due の列の無い表では contact が 1 で、2 と 3 は行を変えない。
    let (t, mut a) = make(
        "ledger-digit-skip",
        &[
            ("x.md", "---\nnext: 2026-11-21\n---\n"),
            ("y.md", "---\nnext: 2026-11-30\n---\n"),
        ],
    );
    let (_g, _) = with_ledger(&mut a, &t, DUE_WEEK, CONTACT5);
    ch(&mut a, '2');
    ch(&mut a, '3');
    assert_eq!(names(&a), ["x", "y"]);
    ch(&mut a, '1');
    assert_eq!(names(&a), ["x"]);
}

#[test]
fn test_digit_order_packs_the_global_keys() {
    // [utest -> req~condition.digit-order~1] 全体の soon(key = 1)と far(key = 9)だけなら、1 が soon、2 が far。
    let (t, mut a) = make(
        "ledger-digit-pack",
        &[
            ("a.md", "---\ndue: 2026-11-24\n---\n"),
            ("d.md", "---\ndue: 2027-01-20\n---\n"),
        ],
    );
    let conf = "\
[conditions.soon]
expr = 'due <= today() + \"7d\"'
key = 1
[conditions.far]
expr = 'due > today() + \"30d\"'
key = 9
";
    let (_g, _) = with_ledger(&mut a, &t, conf, "");
    ch(&mut a, '9');
    assert_eq!(names(&a), ["a", "d"], "書いた数の 9 は振っていない");
    ch(&mut a, '2');
    assert_eq!(names(&a), ["d"]);
    ch(&mut a, '2');
    ch(&mut a, '1');
    assert_eq!(names(&a), ["a"]);
}

const MINE_OPEN: &str = "\
[conditions.mine]
expr = 'actor != \"ai\" && status != \"done\"'
key = 1
[conditions.open]
expr = 'status != \"done\"'
key = 2
";

#[test]
fn test_columns_present_help_keys_and_view_settings() {
    // [utest -> req~condition.columns-present~1] actor の無い表では、ヘルプに「1 open」があって mine が無く、`1` は
    // open で絞り、ビューの設定のフィルターの区画の条件は open だけ。
    let (t, mut a) = make(
        "ledger-columns",
        &[
            ("a.md", "---\nstatus: todo\n---\n"),
            ("b.md", "---\nstatus: done\n---\n"),
        ],
    );
    let (_g, _) = with_ledger(&mut a, &t, MINE_OPEN, "");
    let help: Vec<String> = help_lines(&a, Mode::Table, 120)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    let text = help.join("\n");
    assert!(
        help.iter().any(|l| l.contains("1 ") && l.contains("open")),
        "{text}"
    );
    assert!(!text.contains("mine"), "{text}");
    ch(&mut a, '1');
    assert_eq!(names(&a), ["a"]);
    ch(&mut a, '1');
    a.apply(Action::ViewSettings);
    let d = a.draft.as_ref().unwrap();
    assert_eq!(
        d.conds.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        ["open"]
    );
}

#[test]
fn test_notice_table_item_without_columns_is_not_shown() {
    // [utest -> req~notice.columns-present~1] due の無い表では、show_zero = true の overdue の項目も出さない。
    let (t, mut a) = make("ledger-notice-off", &[("a.md", "---\nstatus: todo\n---\n")]);
    let conf = "\
[conditions.overdue]
expr = 'due < today()'
[[notices]]
cond = \"overdue\"
show_zero = true
";
    let (_g, _) = with_ledger(&mut a, &t, conf, "");
    a.notice_refresh();
    assert!(a.notice_entries().is_empty(), "{:?}", a.notice_lines());
}

/// tasks(ノートのフォルダの外)に actor が me の2行を置き、people(今の表)と tasks を範囲にする。
fn people_and_tasks(a: &mut App, t: &Tmp) -> std::path::PathBuf {
    let tasks = t.notes().parent().unwrap().join("tasks");
    std::fs::create_dir_all(&tasks).unwrap();
    for n in ["t1", "t2"] {
        std::fs::write(
            tasks.join(format!("{n}.md")),
            "---\nactor: me\nstatus: todo\n---\n",
        )
        .unwrap();
    }
    a.scope = Some(Scope {
        name: "Home".into(),
        source: Source::Chosen,
        tables: vec![
            WsTable {
                name: "people".into(),
                path: t.notes().canonicalize().unwrap(),
                ..WsTable::default()
            },
            WsTable {
                name: "tasks".into(),
                path: tasks.canonicalize().unwrap(),
                ..WsTable::default()
            },
        ],
        profile: Default::default(),
        warnings: Vec::new(),
    });
    tasks
}

const PEOPLE: &[(&str, &str)] = &[
    ("p1.md", "---\nstatus: todo\n---\n"),
    ("p2.md", "---\nstatus: todo\n---\n"),
    ("p3.md", "---\nstatus: todo\n---\n"),
];

#[test]
fn test_notice_workspace_item_skips_tables_without_columns() {
    // [utest -> req~notice.columns-present~1] actor の無い people の3行は数えず、tasks の2行だけで「担当 2 件」。
    let (t, mut a) = make("ledger-notice-ws", PEOPLE);
    people_and_tasks(&mut a, &t);
    let conf = "\
[conditions.mine]
expr = 'actor != \"ai\"'
[[notices]]
cond = \"mine\"
scope = \"workspace\"
text = \"担当 {count} 件\"
";
    let (_g, _) = with_ledger(&mut a, &t, conf, "");
    a.notice_refresh();
    let end = Instant::now() + Duration::from_secs(10);
    while a.notice.counts.iter().any(Option::is_none) && Instant::now() < end {
        a.notice_poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    a.notice_poll();
    assert_eq!(a.notice_lines(), ["担当 2 件"]);
}

/// 記録の置き場と試験の出す先を置く。
fn outlet(a: &mut App, t: &Tmp) -> Arc<Mutex<TestOutlet>> {
    a.nv.target = t.notes();
    a.notify.dir = Some(t.notes().parent().unwrap().join("state"));
    let out = Arc::new(Mutex::new(TestOutlet::default()));
    a.notify.test = Some(out.clone());
    out
}

/// 確かめ始めて、結果を拾うまで待つ(上限 10 秒)。
fn check(a: &mut App) {
    a.notify_start(a.today, a.now);
    let end = Instant::now() + Duration::from_secs(10);
    while a.notify_busy() && Instant::now() < end {
        a.notify.want = false;
        a.notify.last = Some(Instant::now());
        a.notify_poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!a.notify_busy(), "確かめが 10 秒で終わらない");
}

fn rung(out: &Arc<Mutex<TestOutlet>>) -> Vec<(String, String)> {
    out.lock()
        .unwrap()
        .got
        .iter()
        .map(|(_, n)| (n.title.clone(), n.text.clone()))
        .collect()
}

#[test]
fn test_notify_skips_tables_without_columns() {
    // [utest -> req~notify.columns-present~1] actor の無い people に足したノートは鳴らず、tasks に足したノートは鳴る。
    let (t, mut a) = make("ledger-notify-cols", PEOPLE);
    let tasks = people_and_tasks(&mut a, &t);
    let conf = "\
[conditions.mine]
expr = 'actor != \"ai\"'
[[notify]]
cond = \"mine\"
scope = \"workspace\"
title = \"担当\"
";
    let (_g, _) = with_ledger(&mut a, &t, conf, "");
    let out = outlet(&mut a, &t);
    check(&mut a);
    assert!(rung(&out).is_empty(), "初めては黙って覚える");
    t.write("p4.md", "---\nstatus: todo\n---\n");
    check(&mut a);
    assert!(rung(&out).is_empty(), "{:?}", rung(&out));
    std::fs::write(tasks.join("t3.md"), "---\nactor: me\n---\n").unwrap();
    check(&mut a);
    assert_eq!(rung(&out), [("担当".to_string(), "t3".to_string())]);
}

#[test]
fn test_notify_marker_item_replaces_the_global_one() {
    // [utest -> req~notify.marker~1] 全体の設定と印に同じ cond の項目があれば、印の項目(題「台帳」)だけで確かめ、
    // 警告は無い。
    let (t, mut a) = make(
        "ledger-notify-marker",
        &[("x.md", "---\nnext: 2026-11-30\n---\n")],
    );
    let conf = "\
[conditions.contact]
expr = 'next <= today()'
[[notify]]
cond = \"contact\"
title = \"全体\"
";
    let marker = "[[notify]]\ncond = \"contact\"\ntitle = \"台帳\"\n";
    let (_g, w) = with_ledger(&mut a, &t, conf, marker);
    assert!(w.is_empty(), "{w:?}");
    let out = outlet(&mut a, &t);
    check(&mut a);
    t.write("w.md", "---\nnext: 2026-11-01\n---\n");
    check(&mut a);
    assert_eq!(rung(&out), [("台帳".to_string(), "w".to_string())]);
}

#[test]
fn test_notify_marker_item_rings_in_its_ledger_only() {
    // [utest -> req~notify.marker~1] 印の [[notify]](題「台帳」)は、その印の台帳の表で確かめると1回出て、
    // 印の無いフォルダの表で確かめると出ない。
    let conf = "[conditions.contact]\nexpr = 'next <= today()'\n";
    let marker = "[[notify]]\ncond = \"contact\"\ntitle = \"台帳\"\n";
    let (t, mut a) = make(
        "ledger-notify-own",
        &[("x.md", "---\nnext: 2026-11-30\n---\n")],
    );
    let (_g, w) = with_ledger(&mut a, &t, conf, marker);
    assert!(w.is_empty(), "{w:?}");
    let out = outlet(&mut a, &t);
    check(&mut a);
    t.write("w.md", "---\nnext: 2026-11-01\n---\n");
    check(&mut a);
    assert_eq!(rung(&out), [("台帳".to_string(), "w".to_string())]);
    // 印の無いフォルダ(同じ全体の設定)では、印の項目で確かめない。
    let (t2, mut b) = make(
        "ledger-notify-none",
        &[("x.md", "---\nnext: 2026-11-30\n---\n")],
    );
    let (_g2, w2) = with_ledger(&mut b, &t2, conf, "");
    assert!(w2.is_empty(), "{w2:?}");
    let out2 = outlet(&mut b, &t2);
    check(&mut b);
    t2.write("w.md", "---\nnext: 2026-11-01\n---\n");
    check(&mut b);
    assert!(rung(&out2).is_empty(), "{:?}", rung(&out2));
}

#[test]
fn test_notify_marker_drops_command_outlets() {
    // [utest -> req~notify.marker-no-command~1] 印の項目の to = ["terminal", "command"] は terminal だけになり、
    // command の並びは空になり、理由の警告が出る。to = ["command"] だけの項目は使わない。
    let conf = "[conditions.contact]\nexpr = 'next <= today()'\n[conditions.soon]\nexpr = 'next <= today() + \"3d\"'\n";
    let marker = "\
[[notify]]
cond = \"contact\"
to = [\"terminal\", \"command\"]
command = [\"touch\", \"/tmp/mdgrid-should-not-run\"]

[[notify]]
cond = \"soon\"
to = [\"command\"]
command = [\"touch\", \"/tmp/mdgrid-should-not-run\"]
";
    let t = Tmp::new("ledger-notify-nocmd");
    std::fs::create_dir_all(t.notes().join(".mdgrid")).unwrap();
    std::fs::write(t.notes().join(".mdgrid/workspace.toml"), marker).unwrap();
    let (c, w) = config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let (c, warns) = config::for_table(&c, &t.notes(), None);
    let mine: Vec<_> = c.notify.iter().filter(|i| i.cond == "contact").collect();
    assert_eq!(mine.len(), 1, "{:?}", c.notify);
    assert_eq!(mine[0].to, [mdgrid::notify::To::Terminal]);
    assert!(mine[0].command.is_empty(), "{:?}", mine[0].command);
    assert!(
        !c.notify.iter().any(|i| i.cond == "soon"),
        "出し先が残らない項目は使わない: {:?}",
        c.notify
    );
    let reason = mdgrid::i18n::Msg::RsnNotifyCommandInMarker.text();
    assert_eq!(
        warns.iter().filter(|x| x.contains(reason)).count(),
        2,
        "{warns:?}"
    );
}

#[test]
fn test_notice_marker_notes_stay_inside_the_ledger() {
    // [utest -> req~notice.marker-inside~1] 印の連絡の項目の note は台帳の根の中だけ。~・..・絶対のパス・
    // 根の外を指すリンクは、警告して使わない。
    let t = Tmp::new("ledger-notice-inside");
    let root = t.notes();
    std::fs::create_dir_all(root.join(".mdgrid")).unwrap();
    std::fs::write(root.join("inbox.md"), "- 鍵を返す\n").unwrap();
    let outside = root.parent().unwrap().join("outside.md");
    std::fs::write(&outside, "- 外のメモ\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, root.join("link.md")).unwrap();
    let marker = "\
[[notices]]
note = \"inbox.md\"
[[notices]]
note = \"~/secret.md\"
[[notices]]
note = \"../outside.md\"
[[notices]]
note = \"/tmp/x.md\"
";
    // 外を指すリンクは Unix でだけ作れるので、その項目も Unix でだけ書く。
    let marker = if cfg!(unix) {
        format!("{marker}[[notices]]\nnote = \"link.md\"\n")
    } else {
        marker.to_string()
    };
    std::fs::write(root.join(".mdgrid/workspace.toml"), marker).unwrap();
    let (c, w) = config::parse("").unwrap();
    assert!(w.is_empty(), "{w:?}");
    let home = root.parent().unwrap().to_path_buf();
    let (c, warns) = config::for_table(&c, &root, Some(&home));
    let notes: Vec<_> = c
        .notices
        .iter()
        .filter_map(|i| match i {
            mdgrid::notice::Item::Note { path, .. } => Some(path.clone()),
            mdgrid::notice::Item::Count { .. } => None,
        })
        .collect();
    assert_eq!(
        notes,
        [root.canonicalize().unwrap().join("inbox.md")],
        "{warns:?}"
    );
    let reason = mdgrid::i18n::Msg::RsnNoticeNoteOutside.text();
    let reason = reason.split('{').next().unwrap();
    let n = if cfg!(unix) { 4 } else { 3 };
    assert_eq!(
        warns.iter().filter(|x| x.contains(reason)).count(),
        n,
        "{warns:?}"
    );
}
