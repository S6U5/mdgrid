//! 台帳の印に書く名前の条件・通知の欄・知らせ(specs/requirements/condition.md の condition.marker など)を、本物の
//! 実行ファイルで確かめる。`--print`・`--notices`・`--notify`・`--print-config --resolved` は子の標準出力と標準エラーで、
//! 数字のキーは疑似端末(80×24)で見る。今日は `MDGRID_TODAY=2026-11-21`。期待値は要求の受け入れの例から取った。
//!
//! ホームの形: people(印あり。名前は「人と会議」)の People に人のノート、tasks(印なし)にタスクのノート。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TODAY: &str = "2026-11-21";

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mdgrid-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(std::fs::canonicalize(&dir).unwrap_or(dir))
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 全体の設定 `conf` と people の印の中身 `marker`(頭に `name = "人と会議"` を足す)のホーム。ノートは
/// `people/People/<名前>.md` と `tasks/<名前>.md` に、フロントマターの中身だけを渡す。
fn home(tag: &str, conf: &str, marker: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "out", "tasks", "people/People"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    h.write("config/mdgrid/config.toml", conf);
    h.write(
        "people/.mdgrid/workspace.toml",
        &format!("name = \"人と会議\"\n{marker}"),
    );
    h
}

fn note(h: &TempDir, rel: &str, fm: &str) {
    h.write(rel, &format!("---\n{fm}\n---\n"));
}

fn command(home: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("MDGRID_TODAY", TODAY);
    // Windows の子は SystemRoot が無いと動かないものがある(env_clear で消えるので戻す)。
    for k in ["SystemRoot", "SYSTEMROOT", "TEMP", "TMP"] {
        if let Some(v) = std::env::var_os(k) {
            c.env(k, v);
        }
    }
    c
}

fn run(home: &Path, args: &[&str]) -> Output {
    command(home, args).output().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// `--print --with-path --filter <式>` の行のノートの名前(見出しを除く。名前の順)。
fn printed(home: &Path, table: &str, filter: &str) -> Vec<String> {
    let o = run(home, &[table, "--print", "--with-path", "--filter", filter]);
    assert!(o.status.success(), "{}", err(&o));
    let mut v: Vec<String> = out(&o)
        .lines()
        .skip(1)
        .filter_map(|l| l.split(',').next())
        .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
        .collect();
    v.sort();
    v
}

const CONTACT: &str = "[conditions.contact]\nexpr = 'next <= today()'\n";

#[test]
fn test_condition_marker_only_in_its_ledger() {
    // [stest -> req~condition.marker~1] 印の条件は、その印の台帳の表でだけ使える。印の無いフォルダでは無い名前。
    let h = home("ledger-marker", "", CONTACT);
    note(&h, "people/People/x.md", "next: 2026-11-21");
    note(&h, "people/People/y.md", "next: 2026-11-30");
    note(&h, "tasks/t.md", "next: 2026-11-01");
    assert_eq!(
        printed(h.path(), "people/People", "cond(\"contact\")"),
        ["x.md"]
    );
    let o = run(
        h.path(),
        &["tasks", "--print", "--filter", "cond(\"contact\")"],
    );
    assert_eq!(o.status.code(), Some(2), "{}", out(&o));
    assert!(
        err(&o).contains("条件「contact」はありません"),
        "{}",
        err(&o)
    );
    assert_eq!(err(&o).lines().count(), 1, "{}", err(&o));
}

#[test]
fn test_condition_marker_bad_condition_warns() {
    // [stest -> req~condition.marker~1] 印の読めない条件は、印の名前で警告して、表は出す。
    let marker = format!("{CONTACT}[conditions.broken]\nexpr = 'next <='\n");
    let h = home("ledger-marker-bad", "", &marker);
    note(&h, "people/People/x.md", "next: 2026-11-21");
    let o = run(h.path(), &["people/People", "--print"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        err(&o).contains(".mdgrid/workspace.toml: conditions.broken.expr"),
        "{}",
        err(&o)
    );
    assert!(out(&o).contains("2026-11-21"), "{}", out(&o));
}

#[test]
fn test_condition_marker_layer_marker_wins() {
    // [stest -> req~condition.marker-layer~1] 同じ名前は印の soon(3日以内)。全体の near の cond("soon") も印の soon を
    // 指し、全体だけの open も使える。
    let conf = "\
[conditions.soon]
expr = 'due <= today() + \"7d\"'
[conditions.near]
expr = 'cond(\"soon\")'
[conditions.open]
expr = 'status != \"done\"'
";
    let marker = "[conditions.soon]\nexpr = 'due <= today() + \"3d\"'\n";
    let h = home("ledger-layer", conf, marker);
    note(&h, "people/People/p.md", "due: 2026-11-24\nstatus: todo");
    note(&h, "people/People/q.md", "due: 2026-11-26\nstatus: todo");
    note(&h, "people/People/r.md", "due: 2026-11-24\nstatus: done");
    let t = "people/People";
    assert_eq!(printed(h.path(), t, "cond(\"soon\")"), ["p.md", "r.md"]);
    assert_eq!(printed(h.path(), t, "cond(\"near\")"), ["p.md", "r.md"]);
    assert_eq!(printed(h.path(), t, "cond(\"open\")"), ["p.md", "q.md"]);
    // 印の無いフォルダでは、全体の soon(7日以内)のまま。
    note(&h, "tasks/a.md", "due: 2026-11-26\nstatus: todo");
    assert_eq!(printed(h.path(), "tasks", "cond(\"soon\")"), ["a.md"]);
}

#[test]
fn test_notice_marker_items_first_and_only_in_the_ledger() {
    // [stest -> req~notice.marker~1] 印の項目が全体の項目より先。印の無いフォルダには出ない。印の連絡の相対の note は
    // 印の根から読む。
    let conf = "\
[conditions.open]
expr = 'status != \"done\"'
[[notices]]
cond = \"open\"
text = \"未完了 {count} 件\"
";
    let marker = format!(
        "[[notices]]\ncond = \"contact\"\ntext = \"連絡する人 {{count}} 人\"\n\n[[notices]]\nnote = \"inbox.md\"\n\n{CONTACT}"
    );
    let h = home("ledger-notice", conf, &marker);
    note(&h, "people/People/x.md", "next: 2026-11-21\nstatus: todo");
    note(&h, "people/People/y.md", "next: 2026-11-20\nstatus: todo");
    note(&h, "people/People/z.md", "next: 2026-11-30\nstatus: todo");
    note(&h, "tasks/t.md", "status: todo");
    h.write("people/inbox.md", "- 鍵を返す\n");
    let o = run(h.path(), &["people/People", "--notices"]);
    assert!(o.status.success(), "{}", err(&o));
    let lines: Vec<String> = out(&o).lines().map(str::to_string).collect();
    assert_eq!(
        lines.first().map(String::as_str),
        Some("連絡する人 2 人"),
        "{lines:?}"
    );
    assert!(lines.contains(&"- 鍵を返す".to_string()), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("未完了")), "{lines:?}");
    let o = run(h.path(), &["tasks", "--notices"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(!out(&o).contains("連絡する人"), "{}", out(&o));
    assert!(out(&o).contains("未完了 1 件"), "{}", out(&o));
}

/// 鳴った知らせの `{part}` を名前にしたノートを out に作るコマンド(TOML のリテラルの文字列)。
fn record(home: &Path, part: &str) -> String {
    format!(
        "['{}', '{}', \"--new-note\", \"{{{part}}}\"]",
        env!("CARGO_BIN_EXE_mdgrid"),
        home.join("out").display()
    )
}

/// out に作られたノートの名前(鳴った知らせ)。
fn rung(home: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(home.join("out"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn test_notify_marker_never_runs_commands() {
    // [stest -> req~notify.marker-no-command~1] 印の [[notify]] に to = ["command"] を書いても、--notify で新しく
    // 当たった行があってもコマンドは動かず、印の知らせにはコマンドを書けないという警告が標準エラーに出る。
    let h = home("ledger-notify-nocmd", "", "");
    let marker = format!(
        "name = \"人と会議\"\n{CONTACT}[[notify]]\ncond = \"contact\"\nto = [\"command\"]\ncommand = {}\n",
        record(h.path(), "text")
    );
    h.write("people/.mdgrid/workspace.toml", &marker);
    note(&h, "people/People/x.md", "next: 2026-11-21");
    let o = run(h.path(), &["people/People", "--notify"]);
    assert!(o.status.success(), "{}", err(&o));
    note(&h, "people/People/w.md", "next: 2026-11-01");
    let o = run(h.path(), &["people/People", "--notify"]);
    assert!(
        rung(h.path()).is_empty(),
        "コマンドが動いた: {:?}",
        rung(h.path())
    );
    assert!(err(&o).contains("config.toml"), "{}", err(&o));
}

#[test]
fn test_notify_marker_replaces_the_same_global_item() {
    // [stest -> req~notify.marker~1] 全体の設定と印に同じ cond の項目があれば印の項目を使い、全体の項目
    // (コマンドに出す)は動かず、警告は無い。
    let h = home("ledger-notify-same", "", "");
    let conf = format!(
        "{CONTACT}[[notify]]\ncond = \"contact\"\ntitle = \"全体\"\nto = [\"command\"]\ncommand = {}\n",
        record(h.path(), "title")
    );
    h.write("config/mdgrid/config.toml", &conf);
    let marker = "name = \"人と会議\"\n[[notify]]\ncond = \"contact\"\ntitle = \"台帳\"\nto = [\"terminal\"]\n";
    h.write("people/.mdgrid/workspace.toml", marker);
    note(&h, "people/People/x.md", "next: 2026-11-30");
    let o = run(h.path(), &["people/People", "--notify"]);
    assert!(o.status.success(), "{}", err(&o));
    note(&h, "people/People/w.md", "next: 2026-11-01");
    let o = run(h.path(), &["people/People", "--notify"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), "", "警告は無い");
    assert!(
        rung(h.path()).is_empty(),
        "全体の項目が動いた: {:?}",
        rung(h.path())
    );
    // 印の無いフォルダでは、全体の項目が動く(全体の項目そのものは効いている)。
    note(&h, "tasks/t.md", "next: 2026-11-30");
    let o = run(h.path(), &["tasks", "--notify"]);
    assert!(o.status.success(), "{}", err(&o));
    note(&h, "tasks/u.md", "next: 2026-11-01");
    let o = run(h.path(), &["tasks", "--notify"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(rung(h.path()), ["全体.md"]);
}

#[test]
fn test_starter_marker_conditions_work_in_made_and_layered_ledgers() {
    // [stest -> req~starter.marker-conditions~1] スターターの印の条件は、スターターから作った台帳でも、印に
    // starter = "kb" と書いた台帳でも効く。
    let h = home("ledger-starter", "", "");
    h.write(
        "config/mdgrid/starters/kb/starter.toml",
        "[name]\nen = \"kb\"\n\n[description]\nen = \"board\"\n",
    );
    h.write(
        "config/mdgrid/starters/kb/ledger/.mdgrid/workspace.toml",
        "name = \"kb\"\n\n[conditions.open]\nexpr = 'status != \"done\"'\nkey = 1\n",
    );
    let o = run(h.path(), &["made", "--init-workspace", "--starter", "kb"]);
    assert!(o.status.success(), "{}", err(&o));
    for dir in ["made", "layered"] {
        note(&h, &format!("{dir}/Board/a.md"), "status: todo");
        note(&h, &format!("{dir}/Board/b.md"), "status: done");
    }
    h.write("layered/.mdgrid/workspace.toml", "starter = \"kb\"\n");
    for dir in ["made", "layered"] {
        let table = format!("{dir}/Board");
        assert_eq!(
            printed(h.path(), &table, "cond(\"open\")"),
            ["a.md"],
            "{dir}"
        );
    }
}

#[test]
fn test_resolved_config_shows_where_conditions_came_from() {
    // [stest -> req~cli.resolved-config-ledger~1] 印から来た条件と通知の欄の上に印の出どころ、全体の条件の上に
    // config.toml。出した文は --config で警告なしに読める。
    let conf = "[conditions.due]\nexpr = 'due <= today()'\nkey = 1\n";
    let marker = format!("{CONTACT}[[notices]]\ncond = \"contact\"\n");
    let h = home("ledger-resolved", conf, &marker);
    note(&h, "people/People/x.md", "next: 2026-11-21");
    let o = run(h.path(), &["people/People", "--print-config", "--resolved"]);
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    assert!(
        text.contains("# .mdgrid/workspace.toml (人と会議)\n[conditions.contact]"),
        "{text}"
    );
    assert!(
        text.contains("# .mdgrid/workspace.toml (人と会議)\n[[notices]]"),
        "{text}"
    );
    assert!(text.contains("# config.toml\n[conditions.due]"), "{text}");
    h.write("back.toml", &text);
    let back = h.path().join("back.toml");
    let o = run(
        h.path(),
        &[
            "tasks",
            "--config",
            back.to_str().unwrap(),
            "--print-config",
            "--resolved",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(err(&o), "", "読み戻しても警告は無い");
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, note, CONTACT, TODAY};
    use crate::pty::Tui;

    /// 起動の直後の1打は、端末の地の色の問い合わせに吸われることがあるので、`done` まで押し直す。
    fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done(&t.screen()) && std::time::Instant::now() < end {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    #[test]
    fn test_condition_digit_order_marker_first_on_screen() {
        // [stest -> req~condition.digit-order~1] 印の contact(key = 5)が 1、全体の due(key = 1)が 2。
        let conf = "\
[conditions.due]
expr = 'due <= today()'
key = 1
[conditions.week]
expr = 'due <= today() + \"7d\"'
key = 2
";
        let marker = format!("{CONTACT}key = 5\n");
        let h = home("ledger-digit-screen", conf, &marker);
        note(
            &h,
            "people/People/x.md",
            "next: 2026-11-21\ndue: 2026-12-20",
        );
        note(
            &h,
            "people/People/y.md",
            "next: 2026-11-30\ndue: 2026-11-01",
        );
        let mut t = Tui::spawn(&["people/People"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        t.wait_for("2026-11-30");
        press_until(&mut t, "1", |s| s.contains("1 contact"));
        t.wait_until("contact で絞る", |s| {
            s.contains("1 contact") && !s.contains("2026-11-30")
        });
        t.send("1");
        t.wait_until("戻る", |s| s.contains("2026-11-30"));
        t.send("2");
        t.wait_until("due で絞る", |s| {
            s.contains("2 due") && !s.contains("2026-12-20")
        });
        let s = t.screen();
        assert!(s.contains("2 due") && !s.contains("contact"), "{s}");
        t.send("q");
        t.wait_exit();
    }

    #[test]
    fn test_condition_columns_present_on_screen() {
        // [stest -> req~condition.columns-present~1] actor の無い表では mine に数字を振らず、open が 1。
        let conf = "\
[conditions.mine]
expr = 'actor != \"ai\" && status != \"done\"'
key = 1
[conditions.open]
expr = 'status != \"done\"'
key = 2
";
        let h = home("ledger-columns-screen", conf, "");
        note(&h, "tasks/a.md", "status: todo\ndue: 2026-11-28");
        note(&h, "tasks/b.md", "status: done\ndue: 2026-11-29");
        let mut t = Tui::spawn(&["tasks"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        t.wait_for("2026-11-29");
        press_until(&mut t, "1", |s| s.contains("1 open"));
        t.wait_until("open で絞る", |s| {
            s.contains("1 open") && !s.contains("2026-11-29")
        });
        assert!(!t.screen().contains("mine"), "{}", t.screen());
        t.send("q");
        t.wait_exit();
    }
}
