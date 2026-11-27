//! 表から AI に頼む(ai.ask-command・ai.ask-run・ai.ask-proposal)。`ask.command` に、標準入力を写して決まった答えを返す
//! `sh` を置く(Windows には `sh` が無いので unix だけ。ubuntu と macOS の CI で走る)。

use super::startup::Startup;
use super::test_screen::{ch, press, Tmp};
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::crossterm::event::KeyCode;
use std::path::PathBuf;

const A: &str = "---\nstatus: todo\n---\n本文A\n";
const B: &str = "---\nstatus: todo\n---\n本文B\n";
const C: &str = "---\nstatus: doing\n---\n本文C\n";

/// 台帳(印あり)の表 tasks に a・b・c。
fn ledger(tag: &str) -> Tmp {
    let tmp = Tmp::new(tag);
    for (rel, text) in [
        (".mdgrid/workspace.toml", ""),
        ("tasks/a.md", A),
        ("tasks/b.md", B),
        ("tasks/c.md", C),
    ] {
        let p = tmp.notes().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    tmp
}

/// 標準入力を `capture` に写し、`reply` を答えるコマンド。
fn echo_command(capture: &std::path::Path, reply: &str) -> Vec<String> {
    vec![
        "sh".into(),
        "-c".into(),
        "cat > \"$0\"; printf '%s' \"$1\"".into(),
        capture.to_string_lossy().into_owned(),
        reply.into(),
    ]
}

fn open(tmp: &Tmp, command: Vec<String>) -> App {
    let dir = tmp.notes().join("tasks");
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: Config {
            ask_command: command,
            ..Config::default()
        },
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a
}

fn footer(a: &App) -> String {
    super::bands::footer(a, 200)
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect()
}

fn typing(a: &mut App, s: &str) {
    for c in s.chars() {
        ch(a, c);
    }
}

/// パレットの「AI に頼む」で `text` を頼む。
fn ask(a: &mut App, text: &str) {
    ch(a, ':');
    typing(a, "ai_ask");
    press(a, KeyCode::Enter);
    typing(a, text);
    press(a, KeyCode::Enter);
}

/// 答えが来るまで見回る(5 秒まで)。
fn wait(a: &mut App) {
    for _ in 0..500 {
        a.poll();
        if !footer(a).contains("AI に頼んでいる") {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn proposals(tmp: &Tmp) -> Vec<PathBuf> {
    std::fs::read_dir(tmp.notes().join(".mdgrid/proposals"))
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default()
}

fn row_index(a: &App, name: &str) -> usize {
    (0..a.slots.len())
        .find(|&i| matches!(a.slots[i], super::grid::Slot::Row(r) if a.src.label(&a.rows[r]).ends_with(name)))
        .unwrap_or_else(|| panic!("{name} の行が無い"))
}

#[test]
fn test_ask_command_only_in_global_config() {
    // [utest -> req~ai.ask-command~1] 印に `[ask] command` → 全体にだけ書けるという警告。`ask.command` が無ければパレットに
    // 「AI に頼む」は無く、あれば並ぶ。
    let tmp = ledger("ask_cmd");
    std::fs::write(
        tmp.notes().join(".mdgrid/workspace.toml"),
        "[ask]\ncommand = [\"sh\"]\n",
    )
    .unwrap();
    let (_, warns) = mdgrid::workspace::read_marker(&tmp.notes());
    assert!(warns.iter().any(|w| w.contains("ask")), "{warns:?}");
    let a = open(&tmp, Vec::new());
    assert!(!a.command_shown(super::keymap::Action::AiAsk));
    let a = open(&tmp, vec!["sh".into()]);
    assert!(a.command_shown(super::keymap::Action::AiAsk));
}

#[test]
fn test_ask_sends_selected_rows_and_waits() {
    // [utest -> req~ai.ask-run~1] 3行のうち a と b を選んで「期限を見直す」→ 写したファイルに頼みの文と a・b のパスと値、c のパスと本文は
    // 無い。待つあいだ下の帯に「AI に頼んでいる」。
    let tmp = ledger("ask_run");
    let capture = tmp.0.join("capture.txt");
    let mut a = open(&tmp, echo_command(&capture, "[]"));
    for name in ["a.md", "b.md"] {
        a.row = row_index(&a, name);
        press(&mut a, KeyCode::Char(' '));
    }
    ask(&mut a, "期限を見直す");
    assert!(footer(&a).contains("AI に頼んでいる"), "{}", footer(&a));
    wait(&mut a);
    let sent = std::fs::read_to_string(&capture).unwrap_or_default();
    assert!(sent.contains("期限を見直す"), "{sent}");
    assert!(
        sent.contains("tasks/a.md") && sent.contains("tasks/b.md"),
        "{sent}"
    );
    assert!(!sent.contains("tasks/c.md"), "{sent}");
    assert!(!sent.contains("本文A"), "{sent}");
    assert!(sent.contains("status"), "{sent}");
}

#[test]
fn test_ask_reply_becomes_one_proposal() {
    // [utest -> req~ai.ask-proposal~1] a の status を done にする直しと、渡していない行を指す直し → 案が1つ(理由は頼みの文、a の
    // status だけ、was は今の値)。案の一覧が開き、ノートは変わらない。
    let tmp = ledger("ask_reply");
    let capture = tmp.0.join("capture.txt");
    let reply = r#"Here you go: [{"path": "tasks/a.md", "set": {"status": "done"}}, {"path": "tasks/zzz.md", "set": {"status": "done"}}]"#;
    let mut a = open(&tmp, echo_command(&capture, reply));
    a.row = row_index(&a, "a.md");
    press(&mut a, KeyCode::Char(' '));
    ask(&mut a, "終わったものを閉じる");
    wait(&mut a);
    let ps = proposals(&tmp);
    assert_eq!(ps.len(), 1, "{:?}", a.message);
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&ps[0]).unwrap()).unwrap();
    assert_eq!(v["note"], "終わったものを閉じる");
    assert_eq!(v["changes"].as_array().map(Vec::len), Some(1), "{v}");
    assert_eq!(v["changes"][0]["path"], "tasks/a.md");
    assert_eq!(v["changes"][0]["set"]["status"], "done");
    assert_eq!(v["changes"][0]["was"]["status"], "todo");
    assert_eq!(
        std::fs::read_to_string(tmp.notes().join("tasks/a.md")).unwrap(),
        A
    );
    let s = super::test_screen::screen(&a);
    assert!(s.contains("終わったものを閉じる"), "{s}");
}

#[test]
fn test_ask_failure_says_why() {
    // [utest -> req~ai.ask-proposal~1] 0 でない終了コードで標準エラーに理由を書くコマンド → 案を作らず、知らせに標準エラーの最初の行。
    let tmp = ledger("ask_fail");
    let cmd = vec![
        "sh".into(),
        "-c".into(),
        "cat >/dev/null; echo 'API の鍵が無い' >&2; exit 3".into(),
    ];
    let mut a = open(&tmp, cmd);
    ask(&mut a, "見直す");
    wait(&mut a);
    assert!(proposals(&tmp).is_empty());
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("API の鍵が無い"), "{m}");
}

#[test]
fn test_ask_reply_without_json() {
    // [utest -> req~ai.ask-proposal~1] JSON の無い文を答えるコマンド → 案を作らず、答えに直しが無いと知らせる。
    let tmp = ledger("ask_nojson");
    let capture = tmp.0.join("capture.txt");
    let mut a = open(&tmp, echo_command(&capture, "直すものはありません"));
    ask(&mut a, "見直す");
    wait(&mut a);
    assert!(proposals(&tmp).is_empty());
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("直し"), "{m}");
}
