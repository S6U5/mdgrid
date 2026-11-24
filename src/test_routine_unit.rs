//! ルーチンの回を作る(specs/requirements/routine.md)の核の試験: 決まりの読み・回の日・回のノートの名前と中身・
//! 一番新しい回だけ・1回だけ・記録・作れないとき・印の `runs` の読み。期待値は要求の受け入れの例から取った
//! (今日は 2026-11-24、火曜)。表を開いてルーチンの行を集めるのと、画面の知らせは src/ui/test_routine.rs、
//! 本物の実行ファイルの `--runs` は tests/test_routine_e2e.rs。

use crate::recurrence::Rule;
use crate::routine::{self, Loaded, Report, Routine, Spec, Table};
use crate::types::{format_date, parse_date};
use crate::writeback::NewValue;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const TODAY: &str = "2026-11-24";

fn day(s: &str) -> i64 {
    parse_date(s).unwrap()
}

/// 今日 `TODAY` の、決まり `rule`(起点 `start`)の一番新しい回の日。
fn latest(rule: &str, start: Option<&str>) -> Option<String> {
    Rule::parse(rule)
        .unwrap_or_else(|| panic!("決まり {rule} が読めない"))
        .latest(start.map(day), day(TODAY))
        .map(format_date)
}

/// 試験ごとの台帳(印の根)。記録は `<根>/state` に置く。
struct Ledger(PathBuf);

impl Ledger {
    fn new(name: &str) -> Ledger {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "mdgrid-routine-{name}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join("ルーチン")).unwrap();
        Ledger(p)
    }

    fn root(&self) -> &Path {
        &self.0
    }

    fn state(&self) -> PathBuf {
        self.0.join("state")
    }

    /// `Runs/` のファイルの名前(並べたもの。無ければ空)。
    fn runs(&self) -> Vec<String> {
        let Ok(rd) = std::fs::read_dir(self.0.join("Runs")) else {
            return Vec::new();
        };
        let mut v: Vec<String> = rd
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    fn run_file(&self, name: &str) -> PathBuf {
        self.0.join("Runs").join(name)
    }
}

impl Drop for Ledger {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn spec() -> Spec {
    Spec {
        folder: "Runs".into(),
        rule: "recurrence".into(),
        ..Spec::default()
    }
}

fn table(l: &Ledger, spec: Spec) -> Table {
    Table {
        name: "ルーチン".into(),
        path: l.root().join("ルーチン"),
        root: l.root().to_path_buf(),
        spec,
    }
}

fn routine(l: &Ledger, name: &str, rule: &str) -> Routine {
    Routine {
        name: name.into(),
        path: l.root().join("ルーチン").join(format!("{name}.md")),
        rule: rule.into(),
        ..Routine::default()
    }
}

fn loaded(l: &Ledger, spec: Spec, routines: Vec<Routine>) -> Vec<Loaded> {
    vec![(table(l, spec), Ok(routines))]
}

/// `--runs` と同じ道で、今日 `today` に来ている回を作る。
fn run_at(l: &Ledger, tables: &[Loaded], today: &str) -> Report {
    routine::run_all(&l.state(), tables, day(today), day(today) * 86_400)
}

// ---- routine.rule ----

#[test]
fn test_routine_rule_weekly_mon() {
    // [utest -> req~routine.rule~1] 今日 2026-11-24(火)の `weekly mon` の回は 2026-11-23(月)。
    assert_eq!(latest("weekly mon", None).as_deref(), Some("2026-11-23"));
    // 観点: 境界。今日がその曜日なら今日、明日の曜日なら先週。
    assert_eq!(latest("weekly tue", None).as_deref(), Some("2026-11-24"));
    assert_eq!(latest("weekly wed", None).as_deref(), Some("2026-11-18"));
    assert_eq!(
        latest("weekly mon thu", None).as_deref(),
        Some("2026-11-23")
    );
}

#[test]
fn test_routine_rule_month_end_not_come_yet() {
    // [utest -> req~routine.rule~1] 今日 2026-11-24 の `毎月 末日` の回は 2026-10-31(11-30 はまだ来ていない)。
    assert_eq!(latest("毎月 末日", None).as_deref(), Some("2026-10-31"));
    assert_eq!(latest("monthly last", None).as_deref(), Some("2026-10-31"));
}

#[test]
fn test_routine_rule_monthly_31_skips_short_months() {
    // [utest -> req~routine.rule~1] 今日 2026-11-24 の `monthly 31` の回は 2026-10-31(31 日の無い月には回が無い)。
    assert_eq!(latest("monthly 31", None).as_deref(), Some("2026-10-31"));
    // 観点: 境界。今日の日・明日の日・昨日の日。
    assert_eq!(latest("monthly 24", None).as_deref(), Some("2026-11-24"));
    assert_eq!(latest("monthly 25", None).as_deref(), Some("2026-10-25"));
    assert_eq!(latest("monthly 1 15", None).as_deref(), Some("2026-11-15"));
}

#[test]
fn test_routine_rule_every_counts_from_start() {
    // [utest -> req~routine.rule~1] 今日 2026-11-24 で `start: 2026-11-20`・`3日ごと` の回は 2026-11-23(11-20・11-23・11-26)。
    assert_eq!(
        latest("3日ごと", Some("2026-11-20")).as_deref(),
        Some("2026-11-23")
    );
    assert_eq!(
        latest("every 3d", Some("2026-11-20")).as_deref(),
        Some("2026-11-23")
    );
    // 観点: 境界。起点が今日なら今日、明日ならまだ回が無い、3日前なら今日。
    assert_eq!(
        latest("every 3d", Some("2026-11-24")).as_deref(),
        Some("2026-11-24")
    );
    assert_eq!(latest("every 3d", Some("2026-11-25")), None);
    assert_eq!(
        latest("every 3d", Some("2026-11-21")).as_deref(),
        Some("2026-11-24")
    );
}

#[test]
fn test_routine_rule_daily_is_today() {
    // [utest -> req~routine.rule~1] 今日 2026-11-24 の `daily` の回は 2026-11-24。
    assert_eq!(latest("daily", None).as_deref(), Some("2026-11-24"));
    assert_eq!(latest("毎日", None).as_deref(), Some("2026-11-24"));
}

#[test]
fn test_routine_rule_japanese_forms_read_the_same() {
    // [utest -> req~routine.rule~1] 日本語の `毎日`・`毎週 月 木`・`毎月 1日 15日`・`毎月 末日`・`N日ごと` は英語の形と同じに読む。
    for (ja, en) in [
        ("毎日", "daily"),
        ("毎週 月 木", "weekly mon thu"),
        ("毎月 1日 15日", "monthly 1 15"),
        ("毎月 末日", "monthly last"),
        ("3日ごと", "every 3d"),
    ] {
        let (a, b) = (Rule::parse(ja), Rule::parse(en));
        assert!(a.is_some(), "{ja} が読めない");
        assert_eq!(a, b, "{ja} と {en}");
    }
}

#[test]
fn test_routine_rule_unknown_forms_are_unreadable() {
    // [utest -> req~routine.rule-unreadable~1] 知らない形と、曜日・日・間隔の無い形・範囲の外の数は読めない。
    for bad in [
        "毎週かも",
        "weekly",
        "weekly someday",
        "monthly",
        "monthly 0",
        "monthly 32",
        "every 0d",
        "every d",
        "hourly",
    ] {
        assert_eq!(Rule::parse(bad), None, "{bad} を読んだ");
    }
}

// ---- routine.rule-unreadable ----

#[test]
fn test_routine_unreadable_rule_is_reported_and_others_run() {
    // [utest -> req~routine.rule-unreadable~1] `毎週かも` の「掃除」は「掃除」と `毎週かも` を含む1行を出し、`daily` の「日記」の回は作る。
    let l = Ledger::new("unreadable");
    let t = loaded(
        &l,
        spec(),
        vec![
            routine(&l, "掃除", "毎週かも"),
            routine(&l, "日記", "daily"),
        ],
    );
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
    assert!(
        r.failures[0].contains("掃除") && r.failures[0].contains("毎週かも"),
        "{:?}",
        r.failures
    );
    assert_eq!(l.runs(), ["日記 2026-11-24.md"]);
}

#[test]
fn test_routine_empty_rule_is_not_a_routine() {
    // [utest -> req~routine.rule-unreadable~1] 決まりの列が空のルーチンは回を作らず、理由も出さない。
    let l = Ledger::new("empty-rule");
    let t = loaded(&l, spec(), vec![routine(&l, "メモ", "")]);
    let r = run_at(&l, &t, TODAY);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert!(r.made.is_empty());
    assert!(l.runs().is_empty());
}

#[test]
fn test_routine_every_without_start_is_reported() {
    // [utest -> req~routine.rule-unreadable~1] `every` で起点の無いルーチンは回を作らず、名前と値を添えた1行を出す。
    let l = Ledger::new("no-start");
    let t = loaded(&l, spec(), vec![routine(&l, "水やり", "every 3d")]);
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
    assert!(
        r.failures[0].contains("水やり") && r.failures[0].contains("every 3d"),
        "{:?}",
        r.failures
    );
    assert!(l.runs().is_empty());
}

// ---- routine.run-note ----

/// `runs = { folder = "Runs", rule = "recurrence", set = { status = "todo" }, copy = ["actor"] }` と、
/// `recurrence: weekly mon`・`actor: both` の「週次レビュー」。
fn review(l: &Ledger, name: &str) -> Vec<Loaded> {
    let s = Spec {
        name: name.into(),
        set: vec![("status".into(), NewValue::Str("todo".into()))],
        copy: vec!["actor".into()],
        ..spec()
    };
    let mut r = routine(l, "週次レビュー", "weekly mon");
    r.values = vec![("actor".into(), NewValue::Str("both".into()))];
    loaded(l, s, vec![r])
}

#[test]
fn test_routine_run_note_contents() {
    // [utest -> req~routine.run-note~1] 回のノートは `Runs/週次レビュー 2026-11-23.md` で、フロントマターは routine・date・set・copy の
    // 順の値、本文は見出し1行。
    let l = Ledger::new("note");
    let r = run_at(&l, &review(&l, ""), TODAY);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    let path = l.run_file("週次レビュー 2026-11-23.md");
    assert_eq!(r.made, std::slice::from_ref(&path));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "---\nroutine: \"[[週次レビュー]]\"\ndate: 2026-11-23\nstatus: todo\nactor: both\n---\n# 週次レビュー 2026-11-23\n"
    );
}

#[test]
fn test_routine_run_note_name_template() {
    // [utest -> req~routine.run-note~1] `name = "{date:YYYYMMDD}_{routine}"` の回の名前は `20261123_週次レビュー.md`。
    let l = Ledger::new("name");
    let r = run_at(&l, &review(&l, "{date:YYYYMMDD}_{routine}"), TODAY);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(l.runs(), ["20261123_週次レビュー.md"]);
}

#[test]
fn test_routine_run_note_makes_the_folder() {
    // [utest -> req~routine.run-note~1] `folder` のフォルダがまだ無ければ、作ってからその中に回のノートを作る。
    let l = Ledger::new("folder");
    let t = loaded(
        &l,
        Spec {
            folder: "記録/Runs".into(),
            ..spec()
        },
        vec![routine(&l, "日記", "daily")],
    );
    assert!(!l.root().join("記録").exists());
    let r = run_at(&l, &t, TODAY);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert!(l.root().join("記録/Runs/日記 2026-11-24.md").is_file());
}

#[test]
fn test_routine_run_note_body_template() {
    // [utest -> req~routine.run-note~1] `body` の雛形があれば、本文はその雛形で始まり、変数は新しいノートと同じに埋める。
    let l = Ledger::new("body");
    std::fs::create_dir_all(l.root().join("tpl")).unwrap();
    std::fs::write(
        l.root().join("tpl/run.md"),
        "## {routine}({date:YYYY/MM/DD})\n- [ ] {name}\n",
    )
    .unwrap();
    let t = loaded(
        &l,
        Spec {
            body: "tpl/run.md".into(),
            ..spec()
        },
        vec![routine(&l, "週次レビュー", "weekly mon")],
    );
    let r = run_at(&l, &t, TODAY);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    let text = std::fs::read_to_string(l.run_file("週次レビュー 2026-11-23.md")).unwrap();
    assert!(
        text.ends_with("---\n## 週次レビュー(2026/11/23)\n- [ ] 週次レビュー 2026-11-23\n"),
        "{text}"
    );
}

// ---- routine.latest-only ----

#[test]
fn test_routine_latest_only_after_a_gap() {
    // [utest -> req~routine.latest-only~1] `daily` で最後に作った回が 2026-11-20、今日が 2026-11-24 なら、11-24 の回だけができる。
    let l = Ledger::new("gap");
    let t = loaded(&l, spec(), vec![routine(&l, "日記", "daily")]);
    run_at(&l, &t, "2026-11-20");
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.made, [l.run_file("日記 2026-11-24.md")]);
    assert_eq!(l.runs(), ["日記 2026-11-20.md", "日記 2026-11-24.md"]);
}

#[test]
fn test_routine_first_run_makes_the_latest() {
    // [utest -> req~routine.latest-only~1] 回を作ったことの無い `weekly mon` のルーチンは、今日 2026-11-24 に 11-23 の回を1つ作る。
    let l = Ledger::new("first");
    let t = loaded(&l, spec(), vec![routine(&l, "週次レビュー", "weekly mon")]);
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.made, [l.run_file("週次レビュー 2026-11-23.md")]);
}

#[test]
fn test_routine_no_future_run() {
    // [utest -> req~routine.latest-only~1] `weekly fri` で 2026-11-20 の回を作ってあれば、今日 2026-11-24 には何も作らない(次の 11-27 はまだ)。
    let l = Ledger::new("future");
    let t = loaded(&l, spec(), vec![routine(&l, "週報", "weekly fri")]);
    run_at(&l, &t, "2026-11-20");
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert_eq!(l.runs(), ["週報 2026-11-20.md"]);
}

// ---- routine.once ----

/// フォルダの中のファイルの名前とバイト。
fn bytes(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.map(|e| {
                let p = e.unwrap().path();
                (
                    p.file_name().unwrap().to_string_lossy().into_owned(),
                    std::fs::read(&p).unwrap(),
                )
            })
            .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn test_routine_once_per_day() {
    // [utest -> req~routine.once~1] 2026-11-23 の「週次レビュー」の回を作ったあと、もう一度回を作っても何も作らず、`Runs/` は変わらない。
    let l = Ledger::new("once");
    let t = loaded(&l, spec(), vec![routine(&l, "週次レビュー", "weekly mon")]);
    run_at(&l, &t, TODAY);
    let before = bytes(&l.root().join("Runs"));
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert_eq!(bytes(&l.root().join("Runs")), before);
}

#[test]
fn test_routine_hand_made_note_is_kept_and_recorded() {
    // [utest -> req~routine.once~1] 人が先に作った `Runs/週次レビュー 2026-11-23.md` のバイトは変わらず、作った回と記録して次から作らない。
    let l = Ledger::new("hand");
    std::fs::create_dir_all(l.root().join("Runs")).unwrap();
    let hand = l.run_file("週次レビュー 2026-11-23.md");
    std::fs::write(&hand, "手で書いた\n").unwrap();
    let t = loaded(&l, spec(), vec![routine(&l, "週次レビュー", "weekly mon")]);
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(std::fs::read(&hand).unwrap(), "手で書いた\n".as_bytes());
    // 記録したので、ノートを消しても作り直さない。
    std::fs::remove_file(&hand).unwrap();
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert!(l.runs().is_empty());
}

#[test]
fn test_routine_deleted_run_is_not_made_again() {
    // [utest -> req~routine.once~1] 作った回のノートを人が消したら、同じ日のうちにもう一度回を作っても作り直さない(tbd~routine.deleted-run)。
    let l = Ledger::new("deleted");
    let t = loaded(&l, spec(), vec![routine(&l, "日記", "daily")]);
    run_at(&l, &t, TODAY);
    std::fs::remove_file(l.run_file("日記 2026-11-24.md")).unwrap();
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert!(l.runs().is_empty());
}

// ---- routine.state ----

#[test]
fn test_routine_state_leaves_routine_notes_alone() {
    // [utest -> req~routine.state~1] 作る回の無い台帳で回を作っても、ルーチンのノートのバイトは変わらない。
    let l = Ledger::new("untouched");
    let note = l.root().join("ルーチン/週報.md");
    std::fs::write(&note, "---\nrecurrence: weekly fri\n---\n").unwrap();
    let t = loaded(&l, spec(), vec![routine(&l, "週報", "weekly fri")]);
    run_at(&l, &t, "2026-11-20");
    let before = bytes(&l.root().join("ルーチン"));
    let r = run_at(&l, &t, TODAY);
    assert!(r.made.is_empty());
    assert_eq!(bytes(&l.root().join("ルーチン")), before);
    // 記録はノートの外(状態の置き場)にある。
    assert!(l.state().join(routine::STATE_FILE).is_file());
}

#[test]
fn test_routine_broken_state_is_rebuilt() {
    // [utest -> req~routine.state~1] 記録が壊れていたら、作り直した旨の1行を出し、同じ名前のノートがある回は作らず、ほかの回は作る。
    let l = Ledger::new("broken");
    let t = loaded(
        &l,
        spec(),
        vec![
            routine(&l, "週次レビュー", "weekly mon"),
            routine(&l, "日記", "daily"),
        ],
    );
    std::fs::create_dir_all(l.root().join("Runs")).unwrap();
    let made = l.run_file("週次レビュー 2026-11-23.md");
    std::fs::write(&made, "前に作った回\n").unwrap();
    std::fs::create_dir_all(l.state()).unwrap();
    std::fs::write(l.state().join(routine::STATE_FILE), "{ 壊れた").unwrap();
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
    assert!(r.failures.is_empty(), "{:?}", r.failures);
    assert_eq!(std::fs::read(&made).unwrap(), "前に作った回\n".as_bytes());
    assert_eq!(r.made, [l.run_file("日記 2026-11-24.md")]);
    // 作り直した記録は読めるので、次は何も言わない。
    let r = run_at(&l, &t, TODAY);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

// ---- routine.fail ----

#[cfg(unix)]
fn set_mode(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
#[test]
fn test_routine_unwritable_folder_fails_and_retries() {
    // [utest -> req~routine.fail~1] 書けない `Runs/` の回は、フォルダを含む1行を出して記録せず、書けるようにすると次に作る。
    // 前提: Unix の許可(CI の ubuntu と macOS の test で回る。root では書けてしまう)。
    let l = Ledger::new("unwritable");
    let runs = l.root().join("Runs");
    std::fs::create_dir_all(&runs).unwrap();
    set_mode(&runs, 0o555);
    let t = loaded(&l, spec(), vec![routine(&l, "日記", "daily")]);
    let r = run_at(&l, &t, TODAY);
    set_mode(&runs, 0o755);
    assert!(r.made.is_empty(), "{:?}", r.made);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
    assert!(
        r.failures[0].contains(&runs.display().to_string()),
        "{:?}",
        r.failures
    );
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.made, [l.run_file("日記 2026-11-24.md")]);
}

#[cfg(unix)]
#[test]
fn test_routine_one_unwritable_table_does_not_stop_the_other() {
    // [utest -> req~routine.fail~1] 2つのルーチンの表のうち1つの `folder` だけが書けなければ、書ける方の回は作り、理由を出す。
    // 前提: Unix の許可(CI の ubuntu と macOS の test で回る)。
    let l = Ledger::new("two-tables");
    let locked = l.root().join("Locked");
    std::fs::create_dir_all(&locked).unwrap();
    set_mode(&locked, 0o555);
    let mut t = loaded(&l, spec(), vec![routine(&l, "日記", "daily")]);
    let mut other = table(
        &l,
        Spec {
            folder: "Locked".into(),
            ..spec()
        },
    );
    other.path = l.root().join("仕事");
    t.push((
        other,
        Ok(vec![Routine {
            path: l.root().join("仕事/週報.md"),
            ..routine(&l, "週報", "daily")
        }]),
    ));
    let r = run_at(&l, &t, TODAY);
    set_mode(&locked, 0o755);
    assert_eq!(r.made, [l.run_file("日記 2026-11-24.md")]);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
}

#[test]
fn test_routine_bad_name_fails_and_others_run() {
    // [utest -> req~routine.fail~1] 名前に使えない字が入る回は作れない理由を出して記録せず、ほかの回は作る。
    let l = Ledger::new("bad-name");
    let t = loaded(
        &l,
        spec(),
        vec![routine(&l, "a:b", "daily"), routine(&l, "日記", "daily")],
    );
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
    assert!(r.failures[0].contains("a:b"), "{:?}", r.failures);
    assert_eq!(r.made, [l.run_file("日記 2026-11-24.md")]);
    // 記録していないので、次もまた作ろうとする。
    let r = run_at(&l, &t, TODAY);
    assert_eq!(r.failures.len(), 1, "{:?}", r.failures);
}

// ---- routine.opt-in(印の runs の読み) ----

fn marker(l: &Ledger, text: &str) {
    std::fs::create_dir_all(l.root().join(".mdgrid")).unwrap();
    std::fs::write(l.root().join(".mdgrid/workspace.toml"), text).unwrap();
}

#[test]
fn test_routine_marker_runs_makes_runs() {
    // [utest -> req~routine.opt-in~1] 表「ルーチン」に `runs = { folder = "Runs", rule = "recurrence" }` を書いた印なら、`Runs/` に回を作る。
    let l = Ledger::new("marker");
    marker(
        &l,
        "[[table]]\nname = \"ルーチン\"\npath = \"ルーチン\"\nruns = { folder = \"Runs\", rule = \"recurrence\" }\n",
    );
    let (got, warns) = crate::workspace::marker_runs(&l.root().join("ルーチン"));
    assert!(warns.is_empty(), "{warns:?}");
    let (s, root) = got.expect("runs を読む");
    assert_eq!((s.folder.as_str(), s.rule.as_str()), ("Runs", "recurrence"));
    assert_eq!(
        root.canonicalize().unwrap(),
        l.root().canonicalize().unwrap()
    );
    let t = loaded(&l, s, vec![routine(&l, "日記", "daily")]);
    run_at(&l, &t, TODAY);
    assert_eq!(l.runs(), ["日記 2026-11-24.md"]);
}

#[test]
fn test_routine_marker_without_runs_is_not_a_routine_table() {
    // [utest -> req~routine.opt-in~1] `runs` の無い印の表は、ルーチンの表でない(警告もない)。
    let l = Ledger::new("no-runs");
    marker(&l, "[[table]]\nname = \"ルーチン\"\npath = \"ルーチン\"\n");
    let (got, warns) = crate::workspace::marker_runs(&l.root().join("ルーチン"));
    assert_eq!(got, None);
    assert!(warns.is_empty(), "{warns:?}");
}

#[test]
fn test_routine_marker_runs_needs_folder_and_rule() {
    // [utest -> req~routine.opt-in~1] `runs` に `folder` か `rule` が無ければ cli.config-file-warning-form の形で警告し、その表では回を作らない。
    for (runs, key) in [
        ("{ rule = \"recurrence\" }", "folder"),
        ("{ folder = \"Runs\" }", "rule"),
    ] {
        let l = Ledger::new("need");
        marker(
            &l,
            &format!("[[table]]\nname = \"ルーチン\"\npath = \"ルーチン\"\nruns = {runs}\n"),
        );
        let (got, warns) = crate::workspace::marker_runs(&l.root().join("ルーチン"));
        assert_eq!(got, None, "{runs}");
        assert_eq!(warns.len(), 1, "{warns:?}");
        assert!(
            warns[0].starts_with(&format!(".mdgrid/workspace.toml: table[1].runs.{key}:")),
            "{warns:?}"
        );
    }
}
