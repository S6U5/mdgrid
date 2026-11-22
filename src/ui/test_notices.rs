//! 通知の欄(specs/requirements/notice.md)の画面の試験。期待値は要求の受け入れの例から取った。
//! ほかの表を数える項目は裏の糸で数え、`notice_poll` で拾うまでは「…」(数えている途中)。

use super::keymap::{Action, Mode};
use super::test_screen::{ch, make, press, screen, typing, Tmp};
use super::*;
use mdgrid::condition;
use mdgrid::config;
use mdgrid::types::parse_date;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::crossterm::event::KeyCode;
use std::time::{Duration, Instant};

/// tasks の表: 期限切れ(今日 2026-11-21 より前で done でない)が a・b の2行、c は先、d は done。
const TASKS: &[(&str, &str)] = &[
    ("a.md", "---\ndue: 2026-11-10\nstatus: todo\n---\n"),
    ("b.md", "---\ndue: 2026-11-11\nstatus: doing\n---\n"),
    ("c.md", "---\ndue: 2026-12-11\nstatus: todo\n---\n"),
    ("d.md", "---\ndue: 2026-11-01\nstatus: done\n---\n"),
];

const OVERDUE: &str = "[conditions.overdue]\nexpr = 'due < today() && status != \"done\"'\n";

fn with_conf(a: &mut App, conf: &str) -> condition::Scoped {
    let (c, w) = config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let g = condition::scoped(c.conditions.clone());
    a.today = parse_date("2026-11-21").unwrap();
    a.now = a.today * 86_400;
    a.configure(&c);
    a.refresh();
    a.notice_refresh();
    g
}

/// 裏の糸の数が出そろうまで拾う(上限 10 秒)。
fn settle(a: &mut App) {
    let end = Instant::now() + Duration::from_secs(10);
    while a.notice.counts.iter().any(Option::is_none) && Instant::now() < end {
        a.notice_poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    a.notice_poll();
}

/// projects の表を tmp の中に足し、tasks(tmp のノートのフォルダ)と projects を範囲にする。
fn workspace(a: &mut App, tmp: &Tmp, missing: bool) {
    // tasks のフォルダの外に置く(中に置くと、tasks の表にも p.md が入る)。
    let proj = tmp.notes().parent().unwrap().join("projects");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(
        proj.join("p.md"),
        "---\ndue: 2026-11-01\nstatus: todo\n---\n",
    )
    .unwrap();
    let mut tables = vec![
        WsTable {
            name: "tasks".into(),
            path: tmp.notes().canonicalize().unwrap(),
            ..WsTable::default()
        },
        WsTable {
            name: "projects".into(),
            path: proj.canonicalize().unwrap(),
            ..WsTable::default()
        },
    ];
    if missing {
        tables.push(WsTable {
            name: "gone".into(),
            path: tmp.notes().join("gone"),
            ..WsTable::default()
        });
    }
    a.scope = Some(Scope {
        name: "Work".into(),
        source: Source::Chosen,
        tables,
        profile: Default::default(),
        warnings: Vec::new(),
    });
}

#[test]
fn test_notice_opt_in_without_items() {
    // [notice.opt-in] [[notices]] の無い設定 → 欄が無く、表の位置は前と同じ。
    let (_t, mut a) = make("notice-none", TASKS);
    let before = super::view::data_y(&a);
    let _g = with_conf(&mut a, OVERDUE);
    assert_eq!(a.notice_rows(), 0);
    assert_eq!(super::view::data_y(&a), before);
}

#[test]
fn test_notice_count_item_text_and_default() {
    // [utest -> req~notice.count-item~1] {count} を数に置き換える。text が無ければ条件の名前と数。
    let (_t, mut a) = make("notice-text", TASKS);
    let conf = format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n[[notices]]\ncond = \"overdue\"\n");
    let _g = with_conf(&mut a, &conf);
    assert_eq!(a.notice_lines(), ["期限切れ 2 件 · overdue 2"]);
    assert!(screen(&a)
        .lines()
        .next()
        .unwrap_or_default()
        .contains("期限切れ 2 件"));
}

#[test]
fn test_notice_count_item_unknown_condition_is_warned() {
    // [utest -> req~notice.count-item~1] cond に書いていない名前は警告して項目を出さない。
    let conf = "[[notices]]\ncond = \"nosuch\"\n";
    let (c, w) = config::parse(conf).unwrap();
    assert!(c.notices.is_empty());
    assert!(w.iter().any(|x| x.contains("notices[1].cond")), "{w:?}");
}

#[test]
fn test_notice_zero_is_hidden_unless_show_zero() {
    // [utest -> req~notice.zero~1] 0 件は出さず、ほかに無ければ欄ごと出ない。show_zero = true なら出す。
    let notes = [("a.md", "---\ndue: 2026-12-11\nstatus: todo\n---\n")];
    let (_t, mut a) = make("notice-zero", &notes);
    let conf =
        format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n");
    let g = with_conf(&mut a, &conf);
    assert_eq!(a.notice_rows(), 0);
    drop(g);
    let conf = format!("{conf}show_zero = true\n");
    let _g = with_conf(&mut a, &conf);
    assert_eq!(a.notice_lines(), ["期限切れ 0 件"]);
}

#[test]
fn test_notice_scope_table_workspace_and_names() {
    // [notice.scope] table は今の表、workspace は範囲の全部、名前の並びはその表だけ。
    let (t, mut a) = make("notice-scope", TASKS);
    workspace(&mut a, &t, false);
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"t{{count}}\"\n\
         [[notices]]\ncond = \"overdue\"\ntext = \"w{{count}}\"\nscope = \"workspace\"\n\
         [[notices]]\ncond = \"overdue\"\ntext = \"p{{count}}\"\nscope = [\"projects\"]\n"
    );
    let _g = with_conf(&mut a, &conf);
    settle(&mut a);
    assert_eq!(a.notice_lines(), ["t2 · w3 · p1"]);
}

#[test]
fn test_notice_count_rows_ignore_view_filters() {
    // [utest -> req~notice.count-rows~1] 検索で1行に絞っても、欄の数は変わらない。
    let (_t, mut a) = make("notice-rows", TASKS);
    let conf =
        format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n");
    let _g = with_conf(&mut a, &conf);
    ch(&mut a, '\\');
    typing(&mut a, "2026-12");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.rows.len(), 1);
    a.notice_refresh();
    assert_eq!(a.notice_lines(), ["期限切れ 2 件"]);
}

#[test]
fn test_notice_note_item_lines() {
    // [notice.note-item] 本文の空でない行を頭から lines 行。無いノートは知らせの1行。
    let (t, mut a) = make("notice-note", TASKS);
    let inbox = t.notes().join("inbox.md");
    std::fs::write(
        &inbox,
        "---\ntitle: inbox\n---\n\n- 金曜に請求書\n- 鍵を返す\n- 歯医者\n",
    )
    .unwrap();
    let conf = format!(
        "{OVERDUE}[[notices]]\nnote = '{}'\nlines = 2\n[[notices]]\nnote = '{}'\n",
        inbox.display(),
        t.notes().join("none.md").display()
    );
    let _g = with_conf(&mut a, &conf);
    assert_eq!(
        a.notice_lines(),
        ["- 金曜に請求書", "- 鍵を返す", "none.md がありません"]
    );
}

#[test]
fn test_notice_layout_counts_line_and_more() {
    // [notice.layout] 件数は1行目に · でつなぎ、連絡は1行ずつ。3行を超えたら「ほか N 行」。
    let (t, mut a) = make("notice-layout", TASKS);
    let inbox = t.notes().join("inbox.md");
    std::fs::write(&inbox, "a\nb\nc\nd\ne\n").unwrap();
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n\
         [[notices]]\ncond = \"overdue\"\ntext = \"今日まで {{count}} 件\"\n\
         [[notices]]\nnote = '{}'\nlines = 5\n",
        inbox.display()
    );
    let _g = with_conf(&mut a, &conf);
    assert_eq!(
        a.notice_lines(),
        ["期限切れ 2 件 · 今日まで 2 件", "a", "ほか 4 行"]
    );
    assert_eq!(a.notice_rows(), 3);
}

#[test]
fn test_notice_refresh_after_outside_change() {
    // [utest -> req~notice.refresh~1] 期限切れの1行の due を直して読み直すと、1件減る。
    let (t, mut a) = make("notice-refresh", TASKS);
    let conf =
        format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n");
    let _g = with_conf(&mut a, &conf);
    assert_eq!(a.notice_lines(), ["期限切れ 2 件"]);
    super::test_screen::edit_outside(&t, "a.md", "---\ndue: 2026-12-30\nstatus: todo\n---\n");
    let today = a.today;
    a.poll();
    a.today = today;
    a.notice_refresh();
    assert_eq!(a.notice_lines(), ["期限切れ 1 件"]);
}

#[test]
fn test_notice_background_shows_ellipsis_until_counted() {
    // [utest -> req~notice.background~1] ほかの表を数える項目は、拾うまで「…」で、表はすぐ使える。
    let (t, mut a) = make("notice-bg", TASKS);
    workspace(&mut a, &t, false);
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\nscope = \"workspace\"\n"
    );
    let _g = with_conf(&mut a, &conf);
    assert_eq!(a.notice_lines(), ["期限切れ … 件"]);
    assert_eq!(a.rows.len(), 4, "表は数え終わるのを待たずに使える");
    settle(&mut a);
    assert_eq!(a.notice_lines(), ["期限切れ 3 件"]);
}

#[test]
fn test_notice_unreadable_table_adds_question_mark() {
    // [utest -> req~notice.unreadable~1] 読めない表は飛ばして数え、数のあとに「?」。
    let (t, mut a) = make("notice-gone", TASKS);
    workspace(&mut a, &t, true);
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\nscope = \"workspace\"\n"
    );
    let _g = with_conf(&mut a, &conf);
    settle(&mut a);
    assert_eq!(a.notice_lines(), ["期限切れ 3? 件"]);
}

#[test]
fn test_notice_focus_key_and_esc() {
    // [utest -> req~notice.focus~1] 欄へ移るキーで1つ目を選び、Esc で表へ。欄が無ければ何もしない。
    let (_t, mut a) = make("notice-focus", TASKS);
    let g = with_conf(&mut a, OVERDUE);
    ch(&mut a, 'm');
    assert_eq!(a.mode, Mode::Table);
    drop(g);
    let conf = format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\n");
    let _g = with_conf(&mut a, &conf);
    ch(&mut a, 'm');
    assert_eq!(a.mode, Mode::Notice);
    assert_eq!(a.notice.sel, 0);
    assert!(screen(&a)
        .lines()
        .next()
        .unwrap_or_default()
        .contains(">overdue 2"));
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Table);
}

#[test]
fn test_notice_jump_in_this_table() {
    // [notice.jump] 今の表の項目で Enter → その条件の行だけになり、帯に条件。
    let (_t, mut a) = make("notice-jump", TASKS);
    let conf =
        format!("{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\n");
    let _g = with_conf(&mut a, &conf);
    ch(&mut a, 'm');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(a.rows.len(), 2);
    assert!(
        a.bar_chips().iter().any(|c| c.contains("overdue")),
        "{:?}",
        a.bar_chips()
    );
}

#[test]
fn test_notice_jump_lists_tables() {
    // [notice.jump] ほかの表を含む項目で Enter → 当たる行のある表を数つきで並べる。
    let (t, mut a) = make("notice-jump-ws", TASKS);
    workspace(&mut a, &t, false);
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\nscope = \"workspace\"\n"
    );
    let _g = with_conf(&mut a, &conf);
    settle(&mut a);
    ch(&mut a, 'm');
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("tasks 2") && s.contains("projects 1"), "{s}");
    typing(&mut a, "projects");
    press(&mut a, KeyCode::Enter);
    assert!(a.switch_to.as_ref().is_some_and(|p| p.name == "projects"));
    assert_eq!(a.switch_cond.as_deref(), Some("overdue"));
}

#[test]
fn test_notice_open_note_asks_editor() {
    // [utest -> req~notice.open-note~1] 連絡の項目で Enter → そのノートをエディタで開くよう頼む。
    let (t, mut a) = make("notice-open", TASKS);
    let inbox = t.notes().join("inbox.md");
    std::fs::write(&inbox, "- 金曜に請求書\n").unwrap();
    let conf = format!("{OVERDUE}[[notices]]\nnote = '{}'\n", inbox.display());
    let _g = with_conf(&mut a, &conf);
    a.apply(Action::FocusNotice);
    press(&mut a, KeyCode::Enter);
    assert!(a.wants_editor());
    assert_eq!(a.editor_note.as_deref(), Some(inbox.as_path()));
}

#[test]
fn test_notice_jump_cancelled_drops_the_condition() {
    // [notice.jump] ためた変更があって終わる確かめから戻ったら、移る先も入れる条件も残らない。
    let (t, mut a) = make("notice-jump-back", TASKS);
    workspace(&mut a, &t, false);
    let conf = format!(
        "{OVERDUE}[[notices]]\ncond = \"overdue\"\ntext = \"期限切れ {{count}} 件\"\nscope = \"workspace\"\n"
    );
    let _g = with_conf(&mut a, &conf);
    settle(&mut a);
    super::test_screen::edit_cell(&mut a, 0, "status", "done");
    assert_eq!(a.changes.count(), 1);
    ch(&mut a, 'm');
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "projects");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Quit);
    assert_eq!(a.switch_cond.as_deref(), Some("overdue"));
    a.apply(Action::Back);
    assert!(a.switch_to.is_none());
    assert!(a.switch_cond.is_none());
}
