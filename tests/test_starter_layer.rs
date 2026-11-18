//! スターターを元に重ねる台帳(印に `starter` と `options` だけを書く)を、本物の実行ファイルで確かめる。
//! 画面を出さない起動なので、標準出力はパイプで受ける。今日は MDGRID_TODAY=2026-11-18 に固定する。
//!
//! 材料(作った値): ユーザーのスターター `kb` を設定の置き場 `config/mdgrid/starters/kb/` に書く。
//!
//! | ファイル                          | 中身                                                                 |
//! |-----------------------------------|----------------------------------------------------------------------|
//! | ledger/.mdgrid/workspace.toml     | open = "Board"、status は todo・done、priority は p1・p2、ビュー All |
//! | ledger/.mdgrid/templates/card.md  | `# {name}`                                                           |
//! | options/who.toml                  | actor は human・ai、All に actor の列、ビュー AI(actor が ai)      |
//!
//! 台帳 `L/` の印は `starter = "kb"` と上書きだけ。ノートは cards/a.md(todo・high・ai)と cards/b.md(zzz・p1・human)。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const STARTER: &str = "open = \"Board\"\n\n[rules.status]\nvalues = [\"todo\", \"done\"]\n\n\
[rules.priority]\nvalues = [\"p1\", \"p2\"]\n\n\
[new_note]\nfolder = \"cards\"\nbody = \".mdgrid/templates/card.md\"\n\n\
[[table]]\nname = \"Board\"\n\n[table.base]\nfilters = { and = ['file.inFolder(\"cards\")'] }\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"All\"\norder = [\"status\", \"priority\"]\n";

const OPTION: &str = "[rules.actor]\nvalues = [\"human\", \"ai\"]\n\n[[table]]\nname = \"Board\"\n\n\
[[table.base.views]]\nname = \"All\"\norder = [\"status\", \"priority\", \"actor\"]\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"AI\"\nwhere = { actor = [\"ai\"] }\norder = [\"status\"]\n";

/// 台帳の印: kb に who を付け、priority の値を上書きし、自分のビュー Mine を足す。
const OWN: &str = "starter = \"kb\"\noptions = [\"who\"]\n\n[rules.priority]\nvalues = [\"high\", \"low\"]\n\n\
[[table]]\nname = \"Board\"\n\n[[table.base.views]]\ntype = \"table\"\nname = \"Mine\"\norder = [\"priority\"]\n";

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let p = std::env::temp_dir().join(format!("mdgrid-layer-{tag}-{}", std::process::id()));
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

fn put(p: &Path, text: &str) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn run_lang(home: &Path, lang: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .env("MDGRID_TODAY", "2026-11-18")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn run(home: &Path, args: &[&str]) -> Output {
    run_lang(home, "en_US.UTF-8", args)
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err_lines(o: &Output) -> Vec<String> {
    String::from_utf8_lossy(&o.stderr)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect()
}

/// 成功を確かめて標準出力の行を返す。
fn lines(o: Output, what: &str) -> Vec<String> {
    assert_eq!(o.status.code(), Some(0), "{what}: {:?}", err_lines(&o));
    out(&o).lines().map(String::from).collect()
}

fn starter_dir(home: &Path) -> PathBuf {
    home.join("config/mdgrid/starters/kb")
}

fn marker(home: &Path) -> PathBuf {
    home.join("L/.mdgrid/workspace.toml")
}

/// スターター kb と、印 `own` の台帳 L(ノート a・b)を作る。
fn setup(tag: &str, own: &str) -> TempDir {
    let t = TempDir::new(tag);
    let h = t.path();
    let s = starter_dir(h);
    put(&s.join("ledger/.mdgrid/workspace.toml"), STARTER);
    put(&s.join("ledger/.mdgrid/templates/card.md"), "# {name}\n");
    put(&s.join("options/who.toml"), OPTION);
    put(&marker(h), own);
    put(
        &h.join("L/cards/a.md"),
        "---\nstatus: todo\npriority: high\nactor: ai\n---\n",
    );
    put(
        &h.join("L/cards/b.md"),
        "---\nstatus: zzz\npriority: p1\nactor: human\n---\n",
    );
    t
}

#[test]
fn test_marker_with_starter_and_options_opens_their_tables_and_views() {
    // [stest -> req~starter.layer-marker~1, req~starter.option-shape~1] 印が starter = "kb" と options = ["who"] だけ → スターターの表 Board の All(オプションが足した actor の列つき)で開き、オプションのビュー AI は actor が ai の行だけ。
    let t = setup("marker", "starter = \"kb\"\noptions = [\"who\"]\n");
    let h = t.path();
    let all = lines(run(h, &["L", "--print"]), "All");
    assert_eq!(all[0], "status,priority,actor");
    assert_eq!(all.len(), 3, "見出しと2行: {all:?}");
    let ai = lines(run(h, &["L", "--print", "--view", "AI"]), "AI");
    assert_eq!(ai, ["status", "todo"]);
    // オプションを付けない印では、actor の列もビュー AI も無い。
    put(&marker(h), "starter = \"kb\"\n");
    assert_eq!(lines(run(h, &["L", "--print"]), "素")[0], "status,priority");
    assert_eq!(
        run(h, &["L", "--print", "--view", "AI"]).status.code(),
        Some(2)
    );
}

#[test]
fn test_own_items_win_over_the_starter() {
    // [stest -> req~starter.layer-own-wins~1] スターターの priority は p1・p2、印の上書きは high・low → b.md の p1 が違反で、a.md の high は違反でない。印に書いていない status はスターターの値(todo・done)のままで、b.md の zzz が違反。
    let t = setup("own", OWN);
    let o = run(t.path(), &["L", "--check"]);
    assert_eq!(o.status.code(), Some(1), "{:?}", err_lines(&o));
    let found = out(&o);
    let rows: Vec<&str> = found.lines().collect();
    assert_eq!(rows.len(), 2, "{found}");
    assert!(
        rows.iter().all(|l| l.starts_with("L/cards/b.md:")),
        "{found}"
    );
    assert!(rows.iter().any(|l| l.contains(": priority: ")), "{found}");
    assert!(rows.iter().any(|l| l.contains(": status: ")), "{found}");
}

#[test]
fn test_own_views_are_added_after_the_starter_views() {
    // [stest -> req~starter.layer-tables~1] 印の表 Board に自分のビュー Mine だけを書く → スターターの All とオプションの AI はそのまま使え、Mine も使える(列は priority)。
    let t = setup("tables", OWN);
    let h = t.path();
    assert_eq!(
        lines(run(h, &["L", "--print"]), "All")[0],
        "status,priority,actor"
    );
    assert_eq!(
        lines(run(h, &["L", "--print", "--view", "AI"]), "AI")[0],
        "status"
    );
    assert_eq!(
        lines(run(h, &["L", "--print", "--view", "Mine"]), "Mine")[0],
        "priority"
    );
}

#[test]
fn test_a_newer_starter_takes_effect_without_rewriting_the_marker() {
    // [stest -> req~starter.layer-update~1] スターターにビュー New が増える → 台帳の印のバイトは前と同じまま、New で開ける。増える前は New は無い。
    let t = setup("update", OWN);
    let h = t.path();
    assert_eq!(
        run(h, &["L", "--print", "--view", "New"]).status.code(),
        Some(2)
    );
    let newer = format!(
        "{STARTER}\n[[table.base.views]]\ntype = \"table\"\nname = \"New\"\norder = [\"actor\"]\n"
    );
    put(
        &starter_dir(h).join("ledger/.mdgrid/workspace.toml"),
        &newer,
    );
    assert_eq!(
        lines(run(h, &["L", "--print", "--view", "New"]), "New")[0],
        "actor"
    );
    // 画面を出さないコマンドを何度か走らせても、印は書き換わらない。
    lines(run(h, &["L", "--print", "--with-path"]), "print");
    assert_eq!(std::fs::read_to_string(marker(h)).unwrap(), OWN);
}

#[test]
fn test_unknown_option_warns_and_only_that_option_is_skipped() {
    // [stest -> req~starter.option-unknown~1] options = ["who", "nope"] → options の nope を言う警告1行で、who は効く(ビュー AI が使える)。
    let t = setup(
        "option",
        "starter = \"kb\"\noptions = [\"who\", \"nope\"]\n",
    );
    let h = t.path();
    let o = run(h, &["L", "--print-config", "--resolved"]);
    let w = err_lines(&o);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].contains("options") && w[0].contains("nope"),
        "{}",
        w[0]
    );
    assert_eq!(
        lines(run(h, &["L", "--print", "--view", "AI"]), "AI"),
        ["status", "todo"]
    );
}

#[test]
fn test_missing_starter_warns_and_opens_with_the_marker_alone() {
    // [stest -> req~starter.retire-missing~1] starter = "gone" と自分の表 T → starter の gone を言う警告1行で、印に書いた表 T の行が出る。ノートと印のバイトは変わらない。
    let own = "starter = \"gone\"\nopen = \"T\"\n\n[[table]]\nname = \"T\"\npath = \"cards\"\n";
    let t = setup("gone", own);
    let h = t.path();
    let o = run(h, &["L", "--print-config", "--resolved"]);
    let w = err_lines(&o);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].contains("starter") && w[0].contains("gone"),
        "{}",
        w[0]
    );
    let rows = lines(run(h, &["L", "--print", "--with-path"]), "T");
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert_eq!(std::fs::read_to_string(marker(h)).unwrap(), own);
    assert_eq!(
        std::fs::read_to_string(h.join("L/cards/a.md")).unwrap(),
        "---\nstatus: todo\npriority: high\nactor: ai\n---\n"
    );
}

#[test]
fn test_body_template_comes_from_the_starter_unless_the_ledger_has_its_own() {
    // [stest -> req~starter.layer-files~1] 台帳に雛形のファイルが無い → 新しいノートの本文はスターターの雛形(`# x`)。台帳に同じパスの雛形を置く → そちらを使う(`mine y`)。
    let t = setup("files", "starter = \"kb\"\n");
    let h = t.path();
    lines(run(h, &["L", "--new-note", "x"]), "x");
    assert_eq!(
        std::fs::read_to_string(h.join("L/cards/x.md")).unwrap(),
        "# x\n"
    );
    put(&h.join("L/.mdgrid/templates/card.md"), "mine {name}\n");
    lines(run(h, &["L", "--new-note", "y"]), "y");
    assert_eq!(
        std::fs::read_to_string(h.join("L/cards/y.md")).unwrap(),
        "mine y\n"
    );
}

#[test]
fn test_starter_lang_fixes_the_language_variant() {
    // [stest -> req~starter.layer-lang~1] スターターに ledger.ja/(All の列は priority だけ)がある。LANG が ja で starter_lang が無い → ja の変種。starter_lang = "en" → LANG が ja でも英語の変種。
    let t = setup("lang", "starter = \"kb\"\n");
    let h = t.path();
    let ja = STARTER.replace(
        "order = [\"status\", \"priority\"]",
        "order = [\"priority\"]",
    );
    put(
        &starter_dir(h).join("ledger.ja/.mdgrid/workspace.toml"),
        &ja,
    );
    assert_eq!(
        lines(run_lang(h, "ja_JP.UTF-8", &["L", "--print"]), "ja")[0],
        "priority"
    );
    assert_eq!(lines(run(h, &["L", "--print"]), "en")[0], "status,priority");
    put(&marker(h), "starter = \"kb\"\nstarter_lang = \"en\"\n");
    assert_eq!(
        lines(run_lang(h, "ja_JP.UTF-8", &["L", "--print"]), "fixed")[0],
        "status,priority"
    );
}

#[test]
fn test_builtin_tasks_options_read_without_warnings() {
    // [stest -> req~starter.option-shape~1] 組み込みの tasks に ai と priority-words を付けた印(2行だけ)は、ja でも en でも警告なしに読め、優先度の値が high・medium・low になる。
    for lang in ["ja_JP.UTF-8", "en_US.UTF-8"] {
        let t = TempDir::new(&format!("builtin-{}", &lang[..2]));
        let h = t.path();
        put(
            &marker(h),
            "starter = \"tasks\"\noptions = [\"ai\", \"priority-words\"]\n",
        );
        put(
            &h.join("L/Tasks/a.md"),
            "---\nstatus: todo\nactor: ai\npriority: p1\n---\n",
        );
        let o = run_lang(h, lang, &["L", "--print-config", "--resolved"]);
        assert_eq!(err_lines(&o), Vec::<String>::new(), "{lang}");
        let c = run_lang(h, lang, &["L", "--check"]);
        assert_eq!(c.status.code(), Some(1), "{lang}: {}", out(&c));
        let found = out(&c);
        assert_eq!(found.lines().count(), 1, "{lang}: {found}");
        assert!(found.contains(": priority: "), "{lang}: {found}");
    }
}
