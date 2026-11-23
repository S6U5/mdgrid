//! 開いている間に新しく当たった行を知らせる(specs/requirements/notify.md)の画面の試験。期待値は要求の受け入れの
//! 例から取った。出す先は試験の出す先(`TestOutlet`)に差し替え、記録は一時のフォルダに置く。
//! 記録と比べて鳴らす行を決める核は src/test_notify_unit.rs。

use super::notify::TestOutlet;
use super::test_screen::{edit_cell, make, press, Tmp};
use super::*;
use mdgrid::condition;
use mdgrid::config;
use mdgrid::notify::{self, To};
use mdgrid::types::parse_date;
use mdgrid::workspace::{Scope, Source, WsTable};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 今日 2026-11-22 の秒(地域の時計)。
fn at(hm: &str) -> (i64, i64) {
    let today = parse_date("2026-11-22").unwrap();
    let (h, m) = hm.split_once(':').unwrap();
    let secs = h.parse::<i64>().unwrap() * 3600 + m.parse::<i64>().unwrap() * 60;
    (today, today * 86_400 + secs)
}

const REMIND: &str = "[conditions.remind]\nexpr = 'remind <= now()'\nlabel = \"リマインダー\"\n";

/// 設定を当て、記録の置き場と試験の出す先を置く。
fn with_conf(a: &mut App, t: &Tmp, conf: &str) -> (condition::Scoped, Arc<Mutex<TestOutlet>>) {
    let (c, w) = config::parse(conf).unwrap();
    assert!(w.is_empty(), "{w:?}");
    let g = condition::scoped(c.conditions.clone());
    a.configure(&c);
    a.nv.target = t.notes();
    a.notify.dir = Some(t.notes().parent().unwrap().join("state"));
    let out = Arc::new(Mutex::new(TestOutlet::default()));
    a.notify.test = Some(out.clone());
    (g, out)
}

/// 確かめ始めて、結果を拾うまで待つ(上限 10 秒)。
fn check_at(a: &mut App, hm: &str) {
    let (today, now) = at(hm);
    a.notify_start(today, now);
    let end = Instant::now() + Duration::from_secs(10);
    while a.notify_busy() && Instant::now() < end {
        a.notify.want = false;
        a.notify.last = Some(Instant::now());
        a.notify_poll();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!a.notify_busy(), "確かめが 10 秒で終わらない");
}

fn texts(out: &Arc<Mutex<TestOutlet>>) -> Vec<(String, String)> {
    out.lock()
        .unwrap()
        .got
        .iter()
        .map(|(_, n)| (n.title.clone(), n.text.clone()))
        .collect()
}

const BILL: &str = "---\ntitle: 請求書を送る\ndue: 2026-11-25\nremind: 2026-11-22T10:00\n---\n";

#[test]
fn test_notify_opt_in_without_items() {
    // [utest -> req~notify.opt-in~1] [[notify]] の無い設定では、当たる行があっても確かめず、記録のファイルを作らない。
    let (t, mut a) = make("notify-none", &[("請求書.md", BILL)]);
    let (_g, out) = with_conf(&mut a, &t, REMIND);
    a.notify_poll();
    assert!(!a.notify_busy());
    assert!(!a.notify_due(Instant::now()));
    assert!(texts(&out).is_empty());
    let state = a.notify.dir.clone().unwrap().join(notify::STATE_FILE);
    assert!(!state.exists());
}

#[test]
fn test_notify_rings_when_time_passes() {
    // [utest -> req~notify.while-open~1, req~notify.text~1] 9:58 に開いて remind が 10:00 のノートは、10:00 を
    // 過ぎた確かめで、題「思い出す」・文「請求書を送る(2026-11-25)」の知らせが出る。
    let (t, mut a) = make("notify-time", &[("請求書.md", BILL)]);
    let conf = format!(
        "{REMIND}[[notify]]\ncond = \"remind\"\ntitle = \"思い出す\"\ntext = \"{{title}}({{due}})\"\n"
    );
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "09:58");
    assert!(texts(&out).is_empty());
    check_at(&mut a, "10:00");
    assert_eq!(
        texts(&out),
        [(
            "思い出す".to_string(),
            "請求書を送る(2026-11-25)".to_string()
        )]
    );
}

#[test]
fn test_notify_text_defaults() {
    // [utest -> req~notify.text~1] title と text を書かなければ、題は条件の見せる名前、文はノートの名前。
    let (t, mut a) = make("notify-default", &[("請求書.md", BILL)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "09:00");
    check_at(&mut a, "10:30");
    assert_eq!(
        texts(&out),
        [("リマインダー".to_string(), "請求書".to_string())]
    );
}

#[test]
fn test_notify_text_missing_column_is_empty() {
    // [utest -> req~notify.text~1] 無い列は空にし、エラーにしない。
    let (t, mut a) = make("notify-missing", &[("請求書.md", BILL)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\ntext = \"{{owner}}: {{title}}\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "09:00");
    check_at(&mut a, "10:30");
    assert_eq!(texts(&out)[0].1, ": 請求書を送る");
}

#[test]
fn test_notify_workspace_is_the_default_scope() {
    // [utest -> req~notify.item~1] scope を書かなければ、tasks を開いていても projects の行を鳴らす。
    let (t, mut a) = make("notify-ws", &[("請求書.md", BILL)]);
    let proj = t.notes().parent().unwrap().join("projects");
    std::fs::create_dir_all(&proj).unwrap();
    a.scope = Some(Scope {
        name: "Work".into(),
        source: Source::Chosen,
        tables: vec![
            WsTable {
                name: "tasks".into(),
                path: t.notes().canonicalize().unwrap(),
                ..WsTable::default()
            },
            WsTable {
                name: "projects".into(),
                path: proj.canonicalize().unwrap(),
                ..WsTable::default()
            },
        ],
        profile: Default::default(),
        warnings: Vec::new(),
    });
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\ntitle = \"{{title}}\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "11:00");
    assert!(texts(&out).is_empty(), "初めては黙って覚える");
    std::fs::write(
        proj.join("見積.md"),
        "---\ntitle: 見積を出す\nremind: 2026-11-22T11:30\n---\n",
    )
    .unwrap();
    check_at(&mut a, "12:00");
    assert_eq!(
        texts(&out),
        [("見積を出す".to_string(), "見積".to_string())]
    );
}

#[test]
fn test_notify_due_on_open_and_every_interval() {
    // [utest -> req~notify.while-open~1] 開いたときに確かめ、そのあとは間隔ごとに確かめる。
    let (t, mut a) = make("notify-due", &[("請求書.md", BILL)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\n");
    let (_g, _out) = with_conf(&mut a, &t, &conf);
    let now = Instant::now();
    assert!(a.notify_due(now), "開いたとき");
    check_at(&mut a, "09:00");
    let last = a.notify.last.unwrap();
    assert!(!a.notify_due(last + notify::INTERVAL.saturating_sub(Duration::from_secs(1))));
    assert!(a.notify_due(last + notify::INTERVAL));
}

#[test]
fn test_notify_after_save() {
    // [utest -> req~notify.while-open~1] 画面でためた変更で remind を過去にしても、保存する前は鳴らず、保存したあとに鳴る。
    let later = "---\ntitle: 請求書を送る\nremind: 2026-11-23T10:00\n---\n";
    let (t, mut a) = make("notify-save", &[("請求書.md", later)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "12:00");
    edit_cell(&mut a, 0, "remind", "2026-11-22T09:00");
    check_at(&mut a, "12:00");
    assert!(texts(&out).is_empty(), "保存する前は鳴らない");
    assert!(!a.notify.want);
    a.key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 0);
    assert!(a.notify.want, "保存したら次の機会に確かめる");
    check_at(&mut a, "12:00");
    assert_eq!(texts(&out).len(), 1);
}

#[test]
fn test_notify_failure_shown_once() {
    // [utest -> req~notify.fail~1] 届かなかった理由は下の帯に出し、同じ出す先の同じ理由は開いている間に1回だけ出す。
    let (t, mut a) = make("notify-fail", &[("請求書.md", BILL)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    out.lock().unwrap().fail = vec![To::Desktop];
    check_at(&mut a, "09:00");
    check_at(&mut a, "10:30");
    let msg = a.message.take().unwrap_or_default();
    assert!(msg.contains("desktop"), "{msg}");
    check_at(&mut a, "10:31");
    assert_eq!(texts(&out).len(), 2, "覚えずに次の確かめでまた鳴らす");
    assert_eq!(a.message, None, "同じ理由は2回出さない");
}

#[test]
fn test_notify_screen_answers_keys_while_checking() {
    // [utest -> req~notify.while-open~1] 確かめは裏の糸なので、始めてすぐにキーを受ける。
    let (t, mut a) = make(
        "notify-keys",
        &[("a.md", BILL), ("b.md", BILL), ("c.md", BILL)],
    );
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\n");
    let (_g, _out) = with_conf(&mut a, &t, &conf);
    let (today, now) = at("09:00");
    let start = Instant::now();
    a.notify_start(today, now);
    press(&mut a, KeyCode::Down);
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(a.row, 1);
}

#[test]
fn test_notify_text_computed_columns() {
    // [utest -> req~notify.text~1] `{file.name}` のような計算の列も、表のセルに出る文字で埋める。
    let (t, mut a) = make("notify-file", &[("請求書.md", BILL)]);
    let conf = format!("{REMIND}[[notify]]\ncond = \"remind\"\ntitle = \"{{file.name}}\"\n");
    let (_g, out) = with_conf(&mut a, &t, &conf);
    check_at(&mut a, "09:00");
    check_at(&mut a, "10:30");
    assert_eq!(texts(&out)[0].0, "請求書.md");
}
