//! ルーチンの回を作る(specs/requirements/routine.md)の画面の試験: 開いたとき・日付が変わったときの知らせ、作る前の
//! 確かめ、やめたとき、読むだけの起動、印の `runs` の有無、画面と `--runs` が同じ記録を使うこと。期待値は要求の受け入れの
//! 例から取った(今日は 2026-11-24、火曜)。決まりの読みと記録の核は src/test_routine_unit.rs。
//!
//! 台帳は一時のフォルダ: 印 `.mdgrid/workspace.toml`、ルーチンは `notes/`、回は `Runs/`。画面は main と同じ道筋
//! (`App::new` → `start`)で `Runs/` を開く。記録は一時のフォルダの `state/` に置く。

use super::keymap::Action;
use super::startup::Startup;
use super::test_screen::{ch, press, screen, Tmp};
use super::*;
use mdgrid::config;
use mdgrid::source::markdown::Markdown;
use mdgrid::types::parse_date;
use mdgrid::writeback::NewValue;
use ratatui::crossterm::event::KeyCode;
use std::time::{Duration, Instant};

/// 表「ルーチン」(notes/)に `runs` を書いた印。回は Runs/ に作る。
const MARKER: &str = "name = \"暮らし\"\n\n[[table]]\nname = \"ルーチン\"\npath = \"notes\"\nruns = { folder = \"Runs\", rule = \"recurrence\" }\n\n[[table]]\nname = \"Runs\"\npath = \"Runs\"\n";
const REVIEW: (&str, &str) = ("週次レビュー.md", "---\nrecurrence: weekly mon\n---\n");
const DIARY: (&str, &str) = ("日記.md", "---\nrecurrence: daily\n---\n");
const OFFER: &str = "ルーチンの回が";

fn day(s: &str) -> i64 {
    parse_date(s).unwrap()
}

/// 台帳を作り、Runs/ を main と同じ道筋で開く(`readonly` は `--readonly`)。
fn boot(name: &str, marker: &str, routines: &[(&str, &str)], readonly: bool) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(tmp.0.join(".mdgrid/workspace.toml"), marker).unwrap();
    for (n, t) in routines {
        tmp.write(n, t);
    }
    let runs = tmp.0.join("Runs");
    std::fs::create_dir_all(&runs).unwrap();
    let (config, warnings) = config::parse("").unwrap();
    let src = Markdown::open(std::slice::from_ref(&runs)).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(80, 24);
    app.start(Startup {
        config,
        warnings,
        readonly,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: None,
        target: runs,
        base: None,
    });
    // main と同じく、記録の置き場を置く(試験は一時のフォルダ)。
    app.routine.dir = Some(tmp.0.join("state"));
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// 今日 `today` で確かめ、結果を拾うまで待つ(上限 10 秒)。
fn check(a: &mut App, today: &str) {
    let d = day(today);
    a.routine_poll_at(d, d * 86_400);
    let end = Instant::now() + Duration::from_secs(10);
    while a.routine_busy() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(20));
        a.routine_poll_at(d, d * 86_400);
    }
    assert!(!a.routine_busy(), "確かめが 10 秒で終わらない");
}

/// Runs/ のファイルの名前(並べたもの)。
fn runs(t: &Tmp) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(t.0.join("Runs"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// `--runs` と同じ道(表をファイルから読み、記録と比べて作る)で、今日 `today` に来ている回を作る。
fn runs_cli(a: &App, today: &str) -> mdgrid::routine::Report {
    a.routine_once(day(today), day(today) * 86_400)
}

/// 画面の最初の `what` をクリックする。
fn click(a: &mut App, what: &str) {
    let s = screen(a);
    let (x, y) = s
        .lines()
        .enumerate()
        .find_map(|(y, l)| l.find(what).map(|i| (width::width(&l[..i]), y)))
        .unwrap_or_else(|| panic!("画面に「{what}」が無い:\n{s}"));
    a.click(x as u16, y as u16);
}

// ---- routine.screen-offer ----

#[test]
fn test_routine_offer_on_open_without_writing() {
    // [utest -> req~routine.screen-offer~1] 作る回が2つある台帳を開くと、下の帯に「ルーチンの回が 2 件来ています」が出て、Runs/ は変わらない。
    let (t, mut a) = boot("routine-offer", MARKER, &[REVIEW, DIARY], false);
    check(&mut a, "2026-11-24");
    let s = screen(&a);
    assert!(s.contains("ルーチンの回が 2 件来ています"), "{s}");
    assert!(runs(&t).is_empty(), "{:?}", runs(&t));
}

#[test]
fn test_routine_offer_when_the_date_changes() {
    // [utest -> req~routine.screen-offer~1] 11-24 の回を作ってある `daily` のルーチンは、開いている間に日付が 11-25 に変わると
    // 「ルーチンの回が 1 件来ています」が出る。
    let (t, mut a) = boot("routine-midnight", MARKER, &[DIARY], false);
    runs_cli(&a, "2026-11-24");
    assert_eq!(runs(&t), ["日記 2026-11-24.md"]);
    check(&mut a, "2026-11-24");
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
    check(&mut a, "2026-11-25");
    let s = screen(&a);
    assert!(s.contains("ルーチンの回が 1 件来ています"), "{s}");
}

#[test]
fn test_routine_no_offer_when_nothing_is_due() {
    // [utest -> req~routine.screen-offer~1] 作る回の無い台帳を開いても、下の帯に回の知らせは出ない。
    let (_t, mut a) = boot("routine-none", MARKER, &[REVIEW], false);
    runs_cli(&a, "2026-11-24");
    check(&mut a, "2026-11-24");
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
}

// ---- routine.screen-create ----

#[test]
fn test_routine_create_after_confirming_the_names() {
    // [utest -> req~routine.screen-create~2] 作る操作で2つの名前が並び、確かめると2つのノートができて、表を読み直した Runs の表に2行が出る。
    let (t, mut a) = boot("routine-create", MARKER, &[REVIEW, DIARY], false);
    check(&mut a, "2026-11-24");
    a.apply(Action::RoutineRuns);
    let s = screen(&a);
    assert!(s.contains("Runs/週次レビュー 2026-11-23.md"), "{s}");
    assert!(s.contains("Runs/日記 2026-11-24.md"), "{s}");
    assert!(runs(&t).is_empty(), "確かめる前に書いた: {:?}", runs(&t));
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        runs(&t),
        ["日記 2026-11-24.md", "週次レビュー 2026-11-23.md"]
    );
    assert_eq!(a.rows.len(), 2, "表を読み直していない:\n{}", screen(&a));
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
}

#[test]
fn test_routine_offer_click_opens_the_names() {
    // [utest -> req~routine.screen-create~2] 下の帯の知らせのクリックで、作るノートの名前の確かめが開く(tbd~routine.offer-key)。
    let (t, mut a) = boot("routine-click", MARKER, &[REVIEW, DIARY], false);
    check(&mut a, "2026-11-24");
    click(&mut a, OFFER);
    let s = screen(&a);
    assert!(s.contains("Runs/日記 2026-11-24.md"), "{s}");
    assert!(runs(&t).is_empty());
}

#[test]
fn test_routine_palette_opens_the_names() {
    // [utest -> req~routine.screen-create~2, req~routine.run-words~1] パレットの「繰り返しの回を作る」(routine_runs)からも、作るノートの名前の確かめが開く(キーは割り当てない。tbd~routine.offer-key)。
    let (t, mut a) = boot("routine-palette", MARKER, &[REVIEW, DIARY], false);
    check(&mut a, "2026-11-24");
    let first = help::candidates(&a, "routine_runs").into_iter().next();
    assert_eq!(
        first.map(|c| (c.name, c.label, c.keys)),
        Some((
            "routine_runs".to_string(),
            "繰り返しの回を作る".to_string(),
            String::new()
        ))
    );
    ch(&mut a, ':');
    for c in "routine_runs".chars() {
        ch(&mut a, c);
    }
    press(&mut a, KeyCode::Enter);
    let s = screen(&a);
    assert!(s.contains("Runs/週次レビュー 2026-11-23.md"), "{s}");
    assert!(runs(&t).is_empty());
}

#[test]
fn test_routine_cancel_writes_nothing_and_stays_quiet_today() {
    // [utest -> req~routine.screen-create~2] 名前の一覧でやめると Runs/ は変わらず、その日のうちは知らせが出ない。次の日に来た回は、また知らせる。
    for cancel in [KeyCode::Esc, KeyCode::Enter] {
        let (t, mut a) = boot("routine-cancel", MARKER, &[REVIEW, DIARY], false);
        check(&mut a, "2026-11-24");
        a.apply(Action::RoutineRuns);
        if cancel == KeyCode::Enter {
            ch(&mut a, 'n');
        }
        press(&mut a, cancel);
        assert!(runs(&t).is_empty(), "{cancel:?}: {:?}", runs(&t));
        assert!(!screen(&a).contains(OFFER), "{cancel:?}: {}", screen(&a));
        // 開き直したのと同じく、同じ日にもう一度確かめても出ない。
        a.routine.day = None;
        check(&mut a, "2026-11-24");
        assert!(!screen(&a).contains(OFFER), "{cancel:?}: {}", screen(&a));
        // 次の日に来た回(日記の 11-25)は知らせる。
        check(&mut a, "2026-11-25");
        assert!(screen(&a).contains(OFFER), "{cancel:?}: {}", screen(&a));
        a.apply(Action::RoutineRuns);
        assert!(
            screen(&a).contains("Runs/日記 2026-11-25.md"),
            "{cancel:?}: {}",
            screen(&a)
        );
    }
}

/// パレットで `routine_runs`(「ルーチンの回を作る」)を選ぶ。
fn palette_runs(a: &mut App) {
    ch(a, ':');
    for c in "routine_runs".chars() {
        ch(a, c);
    }
    press(a, KeyCode::Enter);
}

#[test]
fn test_routine_palette_lists_dismissed_runs_and_creates_them() {
    // [utest -> req~routine.screen-create~2] その日にやめたあと、パレットの「ルーチンの回を作る」を選ぶと、やめた回の名前が並び、
    // 確かめると作る(やめたのは下の帯の知らせだけ)。
    let (t, mut a) = boot("routine-after-cancel", MARKER, &[REVIEW, DIARY], false);
    check(&mut a, "2026-11-24");
    a.apply(Action::RoutineRuns);
    press(&mut a, KeyCode::Esc);
    assert!(runs(&t).is_empty(), "{:?}", runs(&t));
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
    palette_runs(&mut a);
    let s = screen(&a);
    assert!(s.contains("Runs/週次レビュー 2026-11-23.md"), "{s}");
    assert!(s.contains("Runs/日記 2026-11-24.md"), "{s}");
    assert!(runs(&t).is_empty(), "確かめる前に書いた: {:?}", runs(&t));
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        runs(&t),
        ["日記 2026-11-24.md", "週次レビュー 2026-11-23.md"]
    );
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
}

#[test]
fn test_routine_palette_recounts_routines_added_after_opening() {
    // [utest -> req~routine.screen-create~2] 開いたあとに、ルーチンの表で `recurrence: daily` の「散歩」を足して保存し、パレットの
    // 「ルーチンの回を作る」を選ぶと、「散歩」の今日の回の名前が並ぶ。
    let (t, mut a) = boot("routine-recount", MARKER, &[REVIEW], false);
    check(&mut a, "2026-11-24");
    t.write("散歩.md", "---\nrecurrence: daily\n---\n");
    palette_runs(&mut a);
    let s = screen(&a);
    assert!(s.contains("Runs/散歩 2026-11-24.md"), "{s}");
    assert!(s.contains("Runs/週次レビュー 2026-11-23.md"), "{s}");
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    assert_eq!(
        runs(&t),
        ["散歩 2026-11-24.md", "週次レビュー 2026-11-23.md"]
    );
}

// ---- routine.state ----

#[test]
fn test_routine_screen_and_cli_share_the_record() {
    // [utest -> req~routine.state~1] 開いている画面で 2026-11-23 の回を作ったあと、続けて `--runs` してもその回を作らない。
    let (t, mut a) = boot("routine-shared", MARKER, &[REVIEW], false);
    check(&mut a, "2026-11-24");
    a.apply(Action::RoutineRuns);
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    assert_eq!(runs(&t), ["週次レビュー 2026-11-23.md"]);
    let r = runs_cli(&a, "2026-11-24");
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert_eq!(runs(&t), ["週次レビュー 2026-11-23.md"]);
}

// ---- routine.readonly ----

#[test]
fn test_routine_readonly_offers_nothing() {
    // [utest -> req~routine.readonly~1] `--readonly` で開くと、作る回があっても下の帯に知らせは出ず、パレットに作る操作が無い。
    let (t, mut a) = boot("routine-readonly", MARKER, &[REVIEW, DIARY], true);
    check(&mut a, "2026-11-24");
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
    let names: Vec<String> = help::candidates(&a, "routine")
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert!(!names.iter().any(|n| n == "routine_runs"), "{names:?}");
    a.apply(Action::RoutineRuns);
    assert!(runs(&t).is_empty());
    assert!(!t.0.join("state").join(mdgrid::routine::STATE_FILE).exists());
}

// ---- routine.opt-in ----

#[test]
fn test_routine_without_runs_offers_and_records_nothing() {
    // [utest -> req~routine.opt-in~1] `runs` の無い印と `recurrence: daily` のノートの表を開くと、知らせは出ず、ノートのフォルダは
    // 変わらず、状態の置き場に回の記録が無い。
    let marker = "[[table]]\nname = \"ルーチン\"\npath = \"notes\"\n";
    let (t, mut a) = boot("routine-optin", marker, &[DIARY], false);
    let before = std::fs::read(t.notes().join("日記.md")).unwrap();
    check(&mut a, "2026-11-24");
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
    assert!(runs(&t).is_empty());
    assert_eq!(std::fs::read(t.notes().join("日記.md")).unwrap(), before);
    assert_eq!(
        std::fs::read_dir(t.notes()).unwrap().count(),
        1,
        "notes/ にファイルが増えた"
    );
    assert!(!t.0.join("state").join(mdgrid::routine::STATE_FILE).exists());
    let names: Vec<String> = help::candidates(&a, "routine")
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert!(!names.iter().any(|n| n == "routine_runs"), "{names:?}");
}

#[test]
fn test_routine_marker_warning_at_start() {
    // [utest -> req~routine.opt-in~1] `runs` に `folder` の無い印は、起動で cli.config-file-warning-form の形の警告を出し、その表では回を作らない。
    let marker =
        "[[table]]\nname = \"ルーチン\"\npath = \"notes\"\nruns = { rule = \"recurrence\" }\n";
    let (t, mut a) = boot("routine-warn", marker, &[DIARY], false);
    let msg = a.message.clone().unwrap_or_default();
    assert!(
        msg.contains(".mdgrid/workspace.toml: table[1].runs.folder:"),
        "{msg}"
    );
    check(&mut a, "2026-11-24");
    assert!(!screen(&a).contains(OFFER), "{}", screen(&a));
    assert!(runs(&t).is_empty());
}

// ---- routine.run-note(表の読み) ----

#[test]
fn test_routine_rows_read_rule_start_and_copy_columns() {
    // [utest -> req~routine.run-note~1] ルーチンの表から、決まりの列・起点の列 start・copy の列の値(日付は日付のまま)を読む。
    let marker = "[[table]]\nname = \"ルーチン\"\npath = \"notes\"\nruns = { folder = \"Runs\", rule = \"recurrence\", copy = [\"actor\", \"due\"] }\n";
    let (_t, a) = boot(
        "routine-rows",
        marker,
        &[(
            "水やり.md",
            "---\nrecurrence: 3日ごと\nstart: 2026-11-20\nactor: both\ndue: 2026-12-01\n---\n",
        )],
        false,
    );
    assert_eq!(a.routine.tables.len(), 1);
    let d = day("2026-11-24");
    let got = routine::rows(&a.routine.tables[0], d, d * 86_400).unwrap();
    assert_eq!(got.len(), 1);
    let r = &got[0];
    assert_eq!(
        (r.name.as_str(), r.rule.as_str(), r.start.as_deref()),
        ("水やり", "3日ごと", Some("2026-11-20"))
    );
    assert_eq!(
        r.values,
        [
            ("actor".to_string(), NewValue::Str("both".into())),
            ("due".to_string(), NewValue::Date("2026-12-01".into())),
        ]
    );
}

// ---- routine.fail ----

#[cfg(unix)]
#[test]
fn test_routine_screen_failure_shows_reason_and_retries() {
    // [utest -> req~routine.fail~1] 開いている間に作れなかった回は、下の帯に理由を出し、記録せずに残して、書けるようにすると作れる。
    // 前提: Unix の許可(CI の ubuntu と macOS の test で回る。root では書けてしまう)。
    use std::os::unix::fs::PermissionsExt;
    let marker = MARKER.replace("folder = \"Runs\"", "folder = \"Locked\"");
    let (t, mut a) = boot("routine-fail", &marker, &[DIARY], false);
    let locked = t.0.join("Locked");
    std::fs::create_dir_all(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    check(&mut a, "2026-11-24");
    a.apply(Action::RoutineRuns);
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    let msg = a.message.clone().unwrap_or_default();
    assert!(msg.contains(&locked.display().to_string()), "{msg}");
    assert!(screen(&a).contains("ルーチンの回が 1 件来ています"));
    a.apply(Action::RoutineRuns);
    ch(&mut a, 'y');
    press(&mut a, KeyCode::Enter);
    assert!(locked.join("日記 2026-11-24.md").is_file());
}

// ---- routine.runs-where ----

/// 表「定期の支払い」(notes/)に、active のものだけ回を作る `runs` を書いた印。
const PAY: &str = "name = \"お金\"\n\n[[table]]\nname = \"定期の支払い\"\npath = \"notes\"\nruns = { folder = \"Runs\", rule = \"recurrence\", where = { status = [\"active\"] } }\n\n[[table]]\nname = \"Runs\"\npath = \"Runs\"\n";

#[test]
fn test_routine_runs_only_where_matches() {
    // [utest -> req~routine.runs-where~1] where = { status = ["active"] } → active の「家賃」の今日の回だけを作り、paused の「雑誌」は作らない
    // (雑誌の recurrence は残ったまま)。
    let rent = ("家賃.md", "---\nrecurrence: daily\nstatus: active\n---\n");
    let mag = ("雑誌.md", "---\nrecurrence: daily\nstatus: paused\n---\n");
    let (t, a) = boot("routine-where", PAY, &[rent, mag], false);
    assert!(a.routine.warnings.is_empty(), "{:?}", a.routine.warnings);
    let r = runs_cli(&a, "2026-11-24");
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(runs(&t), ["家賃 2026-11-24.md"], "{:?}", r.made);
    let mag = std::fs::read_to_string(t.notes().join("雑誌.md")).unwrap();
    assert!(mag.contains("recurrence: daily"), "{mag}");
}

#[test]
fn test_routine_runs_where_not_a_table_warns() {
    // [utest -> req~routine.runs-where~1] where = "status"(表でない)→ runs.where を言う警告1行を出し、where が無いときと同じに回を作る。
    let marker = PAY.replace("where = { status = [\"active\"] }", "where = \"status\"");
    let mag = ("雑誌.md", "---\nrecurrence: daily\nstatus: paused\n---\n");
    let (t, a) = boot("routine-where-bad", &marker, &[mag], false);
    let w = &a.routine.warnings;
    assert!(w.len() == 1 && w[0].contains("runs.where"), "{w:?}");
    runs_cli(&a, "2026-11-24");
    assert_eq!(runs(&t), ["雑誌 2026-11-24.md"]);
}

// ---- routine.run-words ----

#[test]
fn test_routine_words_use_the_table_name() {
    // [utest -> req~routine.run-words~1] 回を作る表が「定期の支払い」だけ → 下の帯に「定期の支払いの回が 2 件来ています」。
    let rent = ("家賃.md", "---\nrecurrence: daily\nstatus: active\n---\n");
    let gas = ("ガス.md", "---\nrecurrence: daily\nstatus: active\n---\n");
    let (_t, mut a) = boot("routine-words", PAY, &[rent, gas], false);
    check(&mut a, "2026-11-24");
    let s = screen(&a);
    assert!(s.contains("定期の支払いの回が 2 件来ています"), "{s}");
}

#[test]
fn test_routine_words_for_two_tables() {
    // [utest -> req~routine.run-words~1] 回を作る表が「ルーチン」と「定期の支払い」の2つ → 下の帯に「繰り返しの回が 2 件来ています」。
    let marker = "name = \"暮らし\"\n\n[[table]]\nname = \"ルーチン\"\npath = \"notes\"\nruns = { folder = \"Runs\", rule = \"recurrence\" }\n\n\
                  [[table]]\nname = \"定期の支払い\"\npath = \"pay\"\nruns = { folder = \"Runs\", rule = \"recurrence\" }\n\n[[table]]\nname = \"Runs\"\npath = \"Runs\"\n";
    let (t, mut a) = boot("routine-words2", marker, &[DIARY], false);
    std::fs::create_dir_all(t.0.join("pay")).unwrap();
    std::fs::write(t.0.join("pay/家賃.md"), "---\nrecurrence: daily\n---\n").unwrap();
    check(&mut a, "2026-11-24");
    let s = screen(&a);
    assert!(
        s.contains("繰り返しの回が 2 件来ています"),
        "{s}\n{:?}",
        a.routine.tables
    );
}
