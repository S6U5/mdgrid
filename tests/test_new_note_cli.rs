//! コマンドから新しいノートを作る(`[<フォルダ>] --new-note <名前> [--set <列>=<値>]…`)を、本物の実行ファイルで確かめる。
//! 画面を出さない起動なので、標準出力はパイプで受ける(疑似端末は使わない)。今日は MDGRID_TODAY=2026-11-18 に固定する。
//!
//! 材料(作った値): 台帳 `t/` に印 `.mdgrid/workspace.toml`(下の MARKER)と本文の雛形 `.mdgrid/templates/task.md`。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 試験ごとの一時フォルダ。Drop で消す。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let p = std::env::temp_dir().join(format!("mdgrid-nn-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        TempDir(p.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TODAY: &str = "2026-11-18";

const MARKER: &str =
    "open = \"Tasks\"\n\n[types]\ndue = \"date\"\npoints = \"number\"\ntags = \"list\"\n\n\
[rules.priority]\nvalues = [\"p1\", \"p2\", \"p3\"]\n\n\
[new_note]\nfolder = \"Tasks\"\nask = [\"due\", \"priority\"]\nhidden = [\"created\"]\n\
body = \".mdgrid/templates/task.md\"\n\n\
[new_note.set]\nstatus = \"todo\"\ncreated = \"{date}\"\n\n\
[[table]]\nname = \"Tasks\"\n\n[table.base]\nfilters = { and = ['file.inFolder(\"Tasks\")'] }\n";

const TEMPLATE: &str = "# {name}\n\nMade on {date}\n";

fn run_in(home: &Path, cwd: &Path, lang: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .env("MDGRID_TODAY", TODAY)
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn run(home: &Path, args: &[&str]) -> Output {
    run_in(home, home, "ja_JP.UTF-8", args)
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 終了コード 2・理由1行(`mdgrid: ` で始まる)・標準出力は空、を確かめて理由を返す。
fn fails_one_line(o: &Output, what: &str) -> String {
    assert_eq!(o.status.code(), Some(2), "{what}: {} {}", out(o), err(o));
    let e = err(o);
    let lines: Vec<&str> = e.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "{what}: 理由は1行: {lines:?}");
    assert!(lines[0].starts_with("mdgrid: "), "{what}: {}", lines[0]);
    assert!(out(o).is_empty(), "{what}: 標準出力は空: {}", out(o));
    lines[0].to_string()
}

/// 台帳 `<home>/t` を作る(印と雛形)。`marker` は印の中身。
fn ledger(tag: &str, marker: &str) -> TempDir {
    let t = TempDir::new(tag);
    let root = t.path().join("t");
    std::fs::create_dir_all(root.join(".mdgrid/templates")).unwrap();
    std::fs::write(root.join(".mdgrid/workspace.toml"), marker).unwrap();
    std::fs::write(root.join(".mdgrid/templates/task.md"), TEMPLATE).unwrap();
    t
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// フロントマターの行(`---` の間)。
fn front(text: &str) -> Vec<String> {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("---"), "フロントマターで始まる:\n{text}");
    lines
        .take_while(|l| *l != "---")
        .map(String::from)
        .collect()
}

#[test]
fn test_new_note_uses_the_new_note_settings_and_template() {
    // [stest -> req~cli.new-note-create~2] 印に folder・set・hidden・body のある台帳で --new-note "Buy milk" → Tasks/Buy milk.md ができ、set の値(status・今日の created)と、変数を埋めた雛形の本文が入る。
    let t = ledger("nn-create", MARKER);
    let o = run(t.path(), &["t", "--new-note", "Buy milk"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let text = read(&t.path().join("t/Tasks/Buy milk.md"));
    let fm = front(&text);
    assert!(fm.contains(&"status: todo".to_string()), "{text}");
    assert!(fm.contains(&format!("created: {TODAY}")), "{text}");
    // 聞く列(due・priority)は、答えが無いので書かない。
    assert!(
        !fm.iter()
            .any(|l| l.starts_with("due") || l.starts_with("priority")),
        "{text}"
    );
    assert!(
        text.ends_with(&format!("---\n# Buy milk\n\nMade on {TODAY}\n")),
        "本文は雛形:\n{text}"
    );
}

#[test]
fn test_new_note_uses_the_settings_of_the_marker_open_table() {
    // [stest -> req~cli.new-note-create~2] 印の open の表に書いた [table.new_note](folder = "Tasks")は、台帳を相対のパスで渡しても今のフォルダから起動しても使う。
    let marker = "open = \"Tasks\"\n\n[[table]]\nname = \"Tasks\"\n\n\
[table.new_note]\nfolder = \"Tasks\"\n\n[table.new_note.set]\nstatus = \"todo\"\n\n\
[table.base]\nfilters = { and = ['file.inFolder(\"Tasks\")'] }\n";
    let t = ledger("nn-table", marker);
    let o = run(t.path(), &["t", "--new-note", "a"]);
    assert_eq!(out(&o), "t/Tasks/a.md\n", "{}", err(&o));
    assert_eq!(
        read(&t.path().join("t/Tasks/a.md")),
        "---\nstatus: todo\n---\n"
    );
    let o = run_in(
        t.path(),
        &t.path().join("t"),
        "ja_JP.UTF-8",
        &["--new-note", "b"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(t.path().join("t/Tasks/b.md").is_file(), "{}", out(&o));
}

#[test]
fn test_new_note_without_settings_makes_an_empty_note_in_the_folder() {
    // [stest -> req~cli.new-note-create~2] 境界: 設定も印も無いフォルダで --new-note memo → そのフォルダに空の memo.md。フォルダを渡さなければ今のフォルダ。
    let t = TempDir::new("nn-plain");
    std::fs::create_dir_all(t.path().join("notes")).unwrap();
    let o = run(t.path(), &["notes", "--new-note", "memo"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(read(&t.path().join("notes/memo.md")), "");
    let o = run_in(
        t.path(),
        &t.path().join("notes"),
        "ja_JP.UTF-8",
        &["--new-note", "here"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(t.path().join("notes/here.md").is_file());
}

#[test]
fn test_new_note_set_values_are_read_by_column_type() {
    // [stest -> req~cli.new-note-set~2] --set due=2026-12-01(日付)・priority=p1・points=3(数)・tags=home,errand(リスト)・status=(空)→ 型どおりに書き、空の status は書かない。
    let t = ledger("nn-set", MARKER);
    let o = run(
        t.path(),
        &[
            "t",
            "--new-note",
            "Buy milk",
            "--set",
            "due=2026-12-01",
            "--set",
            "priority=p1",
            "--set",
            "points=3",
            "--set",
            "tags=home,errand",
            "--set",
            "status=",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let text = read(&t.path().join("t/Tasks/Buy milk.md"));
    let fm = front(&text);
    for want in ["due: 2026-12-01", "priority: p1", "points: 3"] {
        assert!(fm.contains(&want.to_string()), "{want}:\n{text}");
    }
    assert!(
        !fm.iter().any(|l| l.starts_with("status")),
        "空の値は書かない:\n{text}"
    );
    // リストは2つの要素として読める(--print の json で確かめる)。
    let p = run(t.path(), &["t", "--raw", "--print", "--format", "json"]);
    assert_eq!(p.status.code(), Some(0), "{}", err(&p));
    let json: String = out(&p).split_whitespace().collect();
    assert!(json.contains(r#""tags":["home","errand"]"#), "{}", out(&p));
    // 型と決まりに合う(--check が 0)。
    let c = run(t.path(), &["t", "--raw", "--check"]);
    assert_eq!(c.status.code(), Some(0), "{} {}", out(&c), err(&c));
}

#[test]
fn test_new_note_set_refuses_unreadable_values_and_writes_nothing() {
    // [stest -> req~cli.new-note-set~2] 失敗: 数の列に points=many・`=` の無い --set due → 理由1行と終了コード 2 で、ノートを作らない。
    let t = ledger("nn-set-bad", MARKER);
    for (args, word) in [
        (
            vec!["t", "--new-note", "x", "--set", "points=many"],
            "points",
        ),
        (vec!["t", "--new-note", "x", "--set", "due"], "--set"),
        (vec!["t", "--new-note", "x", "--set", "due=someday"], "due"),
    ] {
        let o = run(t.path(), &args);
        let why = fails_one_line(&o, &format!("{args:?}"));
        assert!(why.contains(word), "{why}");
        assert!(!t.path().join("t/Tasks").exists(), "{args:?}: 作らない");
    }
}

#[test]
fn test_new_note_prints_the_created_path() {
    // [stest -> req~cli.new-note-report~1] 作ったノートのパス(渡したフォルダにつないだ形)を1行だけ標準出力に出し、終了コード 0。標準エラーは空。LANG=en_US.UTF-8 でも同じ。
    let t = ledger("nn-report", MARKER);
    let o = run(t.path(), &["t", "--new-note", "Buy milk"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(out(&o), "t/Tasks/Buy milk.md\n");
    assert_eq!(err(&o), "");
    let o = run_in(
        t.path(),
        t.path(),
        "en_US.UTF-8",
        &["t", "--new-note", "Call"],
    );
    assert_eq!(out(&o), "t/Tasks/Call.md\n");
}

#[test]
fn test_new_note_never_overwrites_an_existing_note() {
    // [stest -> req~cli.new-note-no-overwrite~1] --new-note で作ったノートと同じ名前でもう一度(別の値を添えて)→ 理由1行と終了コード 2 で、既にあるファイルのバイトは変わらない。
    // 観点: 1回目が作れることを先に確かめる(何も作れない道具でも「上書きしない」は通ってしまうため)。
    let t = ledger("nn-exists", MARKER);
    let path = t.path().join("t/Tasks/Buy milk.md");
    let o = run(
        t.path(),
        &["t", "--new-note", "Buy milk", "--set", "priority=p2"],
    );
    assert_eq!(o.status.code(), Some(0), "1回目は作る: {}", err(&o));
    let first = read(&path);
    assert!(first.contains("priority: p2"), "{first}");
    let o = run(
        t.path(),
        &["t", "--new-note", "Buy milk", "--set", "priority=p1"],
    );
    let why = fails_one_line(&o, "既にある名前");
    assert!(why.contains("Buy milk.md"), "{why}");
    assert_eq!(read(&path), first, "2回目は1バイトも変えない");
    // 手で書いたノート(本文つき)も同じ。
    let hand = t.path().join("t/Tasks/Hand.md");
    std::fs::write(&hand, "---\nstatus: done\n---\nkeep me\n").unwrap();
    let o = run(t.path(), &["t", "--new-note", "Hand"]);
    fails_one_line(&o, "手で書いたノート");
    assert_eq!(read(&hand), "---\nstatus: done\n---\nkeep me\n");
}

#[test]
fn test_new_note_refuses_names_outside_the_folder() {
    // [stest -> req~cli.new-note-refuse-outside~2] `../x` と絶対パスの名前 → 理由1行と終了コード 2 で、フォルダの外に何も作らない。下のフォルダ `sub/x` は作る。
    let t = ledger("nn-outside", MARKER);
    let abs = t.path().join("abs");
    for name in ["../x", "../../x", abs.to_str().unwrap()] {
        let o = run(t.path(), &["t", "--new-note", name]);
        fails_one_line(&o, name);
    }
    assert!(!t.path().join("x.md").exists() && !t.path().join("t/x.md").exists());
    assert!(!t.path().join("abs.md").exists());
    let o = run(t.path(), &["t", "--new-note", "sub/x"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(t.path().join("t/Tasks/sub/x.md").is_file());
}

#[test]
fn test_new_note_refuses_when_a_required_column_is_empty() {
    // [stest -> req~cli.new-note-required~2] required = ["priority"] の台帳: --set priority=p2 があれば作り、無ければ理由1行(列の名前 priority を言う)と終了コード 2 で作らない。
    let marker = MARKER.replace("hidden = ", "required = [\"priority\"]\nhidden = ");
    let t = ledger("nn-required", &marker);
    let o = run(t.path(), &["t", "--new-note", "a"]);
    let why = fails_one_line(&o, "必須が空");
    assert!(why.contains("priority"), "{why}");
    assert!(!t.path().join("t/Tasks/a.md").exists());
    let o = run(t.path(), &["t", "--new-note", "a", "--set", "priority=p2"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(t.path().join("t/Tasks/a.md").is_file());
}

#[test]
fn test_new_note_flag_misuse_is_one_line_and_code_2() {
    // [stest -> req~cli.new-note-bad-args~2] --new-note の無い --set・フォルダ2つ・--print との組・空の名前 → 理由1行と終了コード 2 で何も作らない。
    let t = ledger("nn-args", MARKER);
    std::fs::create_dir_all(t.path().join("u")).unwrap();
    for args in [
        vec!["t", "--set", "priority=p1"],
        vec!["t", "u", "--new-note", "x"],
        vec!["t", "--new-note", "x", "--print"],
        vec!["t", "--new-note", ""],
    ] {
        let o = run(t.path(), &args);
        fails_one_line(&o, &format!("{args:?}"));
    }
    assert!(!t.path().join("t/Tasks").exists());
    assert!(!t.path().join("u/x.md").exists());
}

#[test]
fn test_help_lists_new_note_in_both_languages() {
    // [stest -> req~cli.new-note-help~1] --help に --new-note と --set が、日本語でも英語でも出る。
    let t = TempDir::new("nn-help");
    for lang in ["ja_JP.UTF-8", "en_US.UTF-8"] {
        let o = run_in(t.path(), t.path(), lang, &["--help"]);
        let h = out(&o);
        assert!(
            h.contains("--new-note") && h.contains("--set"),
            "{lang}:\n{h}"
        );
    }
}
