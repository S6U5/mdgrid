#![cfg(unix)]
//! タスク 3(marker-base)の受け入れ: 印 `.mdgrid/workspace.toml` の `[[table]]` に `.base` と同じ中身を
//! `[table.base]` で書くこと(WS-9)と、その中の `where`(WS-10)。
//! 仕様: specs/workspace/spec.md の WS-9・WS-10、specs/_changes/2026-10-15-marker-base.md、
//! specs/_decisions/2026-10-15-marker-base.md・2026-10-15-marker-where.md。
//! 実装のあとに独立の試験役が書いた。本物の実行ファイルを動かす。行は主に `--print --with-path` の標準出力
//! (パイプ)で確かめ、ビューのタブと警告は疑似端末(tests/pty/mod.rs)の画面で確かめる。
//!
//! 材料の台帳(作った値。根に `.base` のファイルは置かない):
//!
//! | ファイル   | status  | actor | title | repo  | kind | tags   | ほか     |
//! |------------|---------|-------|-------|-------|------|--------|----------|
//! | todo/a.md  | todo    | ai    | 甲    | tasks | todo | [a, b] | prio: 1  |
//! | todo/b.md  | doing   | human | 乙    | other | bug  | [c]    | prio: 2  |
//! | todo/c.md  | done    | ai    | 丙    | tasks | todo |        | flag: true |
//! | todo/d.md  | done    | human | 丁    | other | todo |        |          |
//! | todo/e.md  | blocked | ai    | 戊    | tasks | todo |        |          |
//! | AGENTS.md  | 案内    |       |       |       |      |        | 台帳でない |
//!
//! 仮定(仕様に書かれていない細部): `--with-path` のパスの頭は問わず、末尾の相対のパスで行を見分ける。
//! 行の並びは問わない。印の警告(CLI-3 の形)は画面のメッセージ行で確かめる(`--print` の標準エラーに
//! 出るかは仕様が決めていない)。

mod pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::process::{Command, Output};

/// 決定 marker-base の確かめ方の印(絞り込み・計算の列・表示名・ビュー2つ・グループ分け)。
const MARKER_LEDGER: &str = r#"open = "台帳"

[[table]]
name = "台帳"

[table.base]
filters = { or = ['file.inFolder("todo")'] }

[table.base.formulas]
"担当" = 'if(actor == "ai", "AI", "人")'

[table.base.properties."note.title"]
displayName = "題名"

[[table.base.views]]
type = "table"
name = "未完了"
filters = 'status != "done"'
order = ["note.title", "status", "formula.担当"]
groupBy = { property = "formula.担当", direction = "ASC" }

[[table.base.views]]
type = "table"
name = "終わった"
filters = 'status == "done"'
order = ["note.title", "status", "formula.担当"]
groupBy = { property = "formula.担当", direction = "ASC" }
"#;

/// `home` を HOME と設定の置き場に、`cwd` を作業のフォルダにして起動する(標準出力はパイプ)。
fn run(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 成功を確かめて標準出力を返す。
fn ok(o: Output, what: &str) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{what}: stderr={} stdout={}",
        err(&o),
        out(&o)
    );
    out(&o)
}

/// HOME(設定の置き場つき)を作る。
fn home(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    std::fs::create_dir_all(t.path().join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(t.path().join("state")).unwrap();
    t
}

/// `root` に台帳のノートを置く(印と `.base` は置かない)。
fn ledger(root: &Path) {
    std::fs::create_dir_all(root.join("todo")).unwrap();
    let notes = [
        (
            "todo/a.md",
            "status: todo\nactor: ai\ntitle: 甲\nrepo: tasks\nkind: todo\ntags: [a, b]\nprio: 1\n",
        ),
        (
            "todo/b.md",
            "status: doing\nactor: human\ntitle: 乙\nrepo: other\nkind: bug\ntags: [c]\nprio: 2\n",
        ),
        (
            "todo/c.md",
            "status: done\nactor: ai\ntitle: 丙\nrepo: tasks\nkind: todo\nflag: true\n",
        ),
        (
            "todo/d.md",
            "status: done\nactor: human\ntitle: 丁\nrepo: other\nkind: todo\n",
        ),
        (
            "todo/e.md",
            "status: blocked\nactor: ai\ntitle: 戊\nrepo: tasks\nkind: todo\n",
        ),
        ("AGENTS.md", "status: 案内\n"),
    ];
    for (rel, fm) in notes {
        std::fs::write(root.join(rel), format!("---\n{fm}---\n本文\n")).unwrap();
    }
}

/// `root` に印を書く。
fn marker(root: &Path, text: &str) {
    std::fs::create_dir_all(root.join(".mdgrid")).unwrap();
    std::fs::write(root.join(".mdgrid/workspace.toml"), text).unwrap();
}

/// 印のバイト。
fn marker_bytes(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".mdgrid/workspace.toml")).unwrap()
}

/// `--with-path` の出力のうち、パスが `todo/<x>.md` の行の `x` を並べ替えて返す(AGENTS.md は "AGENTS")。
fn names(s: &str) -> Vec<String> {
    let mut v: Vec<String> = s
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let p = l.split(',').next().unwrap_or("");
            let f = p.rsplit('/').next().unwrap_or(p);
            f.trim_end_matches(".md").to_string()
        })
        .collect();
    v.sort();
    v
}

/// 根で `--print --with-path`(足す引数つき)を動かし、残った行の名前を返す。
fn rows_of(h: &Path, root: &Path, extra: &[&str]) -> Vec<String> {
    let mut args = vec!["--print", "--with-path"];
    args.extend_from_slice(extra);
    let s = ok(run(h, root, &args), &format!("{extra:?}"));
    names(&s)
}

/// 期待の名前の並び。
fn want(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|x| x.to_string()).collect()
}

/// `where` の試験の印: `[table.base]` に `top`(最上位の中身)と、`views`(`[[table.base.views]]` の中身の並び)。
/// どのビューも `type = "table"` と名前 v1, v2, … を付ける。
fn where_marker(top: &str, views: &[&str]) -> String {
    let mut s = String::from("open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\n\n[table.base]\n");
    s.push_str(top);
    s.push('\n');
    for (i, v) in views.iter().enumerate() {
        s.push_str(&format!(
            "\n[[table.base.views]]\ntype = \"table\"\nname = \"v{}\"\norder = [\"title\", \"status\"]\n{v}\n",
            i + 1
        ));
    }
    s
}

/// 根で画面を出し、メッセージ行(画面のどこか)に `prefix` で始まる警告が出るのを待つ。
fn screen_warns(root: &Path, prefix: &str) {
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("印の警告", |s| s.contains(prefix));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

// ---- WS-9: 印の表に .base と同じ中身 ----

#[test]
fn test_ws_9_marker_base_print_first_view() {
    // [WS-9] 根に `.base` を置かず、印に中身を書いた表 → `--print` が最初のビュー(未完了)の行で、
    // 見出しに表示名「題名」と計算の列「担当」。AGENTS.md の行は無い。
    let h = home("mb-print");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_LEDGER);
    assert!(!root.join("台帳.base").exists());
    let s = ok(run(h.path(), &root, &["--print"]), "--print");
    let head = s.lines().next().unwrap_or("");
    assert!(head.contains("題名"), "見出しに表示名が無い\n{s}");
    assert!(head.contains("担当"), "見出しに計算の列が無い\n{s}");
    assert!(
        s.lines().any(|l| l.contains("甲") && l.contains("AI")),
        "{s}"
    );
    assert!(
        s.lines().any(|l| l.contains("乙") && l.contains("人")),
        "{s}"
    );
    assert!(
        s.lines().any(|l| l.contains("戊") && l.contains("AI")),
        "{s}"
    );
    assert!(
        !s.contains("丙") && !s.contains("丁"),
        "done の行がある\n{s}"
    );
    assert!(!s.contains("案内"), "AGENTS.md の行がある\n{s}");
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a", "b", "e"]));
}

#[test]
fn test_ws_9_marker_base_view_option() {
    // [WS-9] `--view 終わった` → done の行だけ(計算の列も効く)。無いビューは理由と 0 以外。
    let h = home("mb-view");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_LEDGER);
    assert_eq!(
        rows_of(h.path(), &root, &["--view", "終わった"]),
        want(&["c", "d"])
    );
    let s = ok(
        run(h.path(), &root, &["--print", "--view", "終わった"]),
        "--view",
    );
    assert!(
        s.lines().any(|l| l.contains("丙") && l.contains("AI")),
        "{s}"
    );
    assert!(
        s.lines().any(|l| l.contains("丁") && l.contains("人")),
        "{s}"
    );
    let o = run(h.path(), &root, &["--print", "--view", "無い"]);
    assert_ne!(o.status.code(), Some(0), "{}", out(&o));
    assert!(err(&o).contains("無い"), "{}", err(&o));
}

#[test]
fn test_ws_9_marker_base_check_runs_on_base_rows() {
    // [WS-9・CLI-23] `--check` も印の中身の表で動く: 最初のビューの行(todo/ の a)の問題は出し、
    // 範囲の外(memo/)のノートの問題は出さない。
    let h = home("mb-check");
    let root = h.path().join("led");
    ledger(&root);
    std::fs::create_dir_all(root.join("memo")).unwrap();
    std::fs::write(
        root.join("todo/a.md"),
        "---\nstatus: todo\nactor: ai\ntitle: 甲\ndue: someday\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join("memo/m.md"),
        "---\nstatus: todo\ndue: never\n---\n",
    )
    .unwrap();
    std::fs::write(
        h.path().join("config/mdgrid/config.toml"),
        "[types]\ndue = \"date\"\n",
    )
    .unwrap();
    marker(&root, MARKER_LEDGER);
    let o = run(h.path(), &root, &["--check"]);
    let s = out(&o);
    assert_eq!(o.status.code(), Some(1), "stdout={s} stderr={}", err(&o));
    assert!(
        s.lines().any(|l| l.contains("a.md") && l.contains("due")),
        "a の問題が無い\n{s}"
    );
    assert!(!s.contains("m.md"), "範囲の外の行を確かめた\n{s}");
}

#[test]
fn test_ws_9_marker_base_bytes_unchanged() {
    // [WS-9] 書いた中身は mdgrid が書き換えない: --print・--view・--check・画面の前後で印のバイトが同じ。
    // 印の根に `台帳.base` のファイルも作らない。
    let h = home("mb-bytes");
    let root = h.path();
    ledger(root);
    marker(root, MARKER_LEDGER);
    let before = marker_bytes(root);
    ok(run(root, root, &["--print"]), "--print");
    ok(
        run(root, root, &["--print", "--view", "終わった"]),
        "--view",
    );
    let _ = run(root, root, &["--check"]);
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("印の表", |s| s.contains("未完了") && s.contains("甲"));
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
    assert_eq!(marker_bytes(root), before, "印が書き換わった");
    assert!(!root.join("台帳.base").exists(), "台帳.base ができた");
}

#[test]
fn test_ws_9_marker_base_screen_tabs_and_groups() {
    // [WS-9] 画面: タブに「未完了」「終わった」があり、未完了の行(甲・乙・戊)が出て、AGENTS.md の行は無い。
    // 担当(AI・人)でまとまりに分かれる(まとまりの名前が画面に出る)。
    let h = home("mb-screen");
    let root = h.path();
    ledger(root);
    marker(root, MARKER_LEDGER);
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("タブと行", |s| {
        s.contains("未完了")
            && s.contains("終わった")
            && s.contains("甲")
            && s.contains("乙")
            && s.contains("戊")
    });
    std::thread::sleep(std::time::Duration::from_millis(300));
    let s = tui.screen();
    assert!(!s.contains("案内"), "AGENTS.md の行がある\n{s}");
    assert!(
        !s.contains("丙") && !s.contains("丁"),
        "done の行がある\n{s}"
    );
    assert!(s.contains("AI") && s.contains("人"), "まとまりが無い\n{s}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_ws_9_marker_base_and_path_warns_uses_base() {
    // [WS-9] `base` と `path = "x.base"` を両方書く → `.mdgrid/workspace.toml: table[1].path:` で始まる警告で、
    // `base` を使う(x.base は無いが、未完了のビューの行が出る)。
    let h = home("mb-both");
    let root = h.path();
    ledger(root);
    let text = MARKER_LEDGER.replace(
        "[[table]]\nname = \"台帳\"\n",
        "[[table]]\nname = \"台帳\"\npath = \"x.base\"\n",
    );
    assert!(text.contains("path = \"x.base\""));
    marker(root, &text);
    assert_eq!(rows_of(root, root, &[]), want(&["a", "b", "e"]));
    screen_warns(root, ".mdgrid/workspace.toml: table[1].path:");
}

#[test]
fn test_ws_9_marker_base_unreadable_views_fails() {
    // [WS-9・BV-1] `[table.base] views = "x"` のような読めない中身 → 理由1行と 0 以外の終了コード。
    // 根の既定の表に落ちない(標準出力に行が無い)。
    let h = home("mb-bad-views");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\n\n[table.base]\nviews = \"x\"\n",
    );
    for args in [&["--print"][..], &["--check"][..]] {
        let o = run(h.path(), &root, args);
        assert_ne!(o.status.code(), Some(0), "{args:?}: {}", out(&o));
        assert_eq!(
            err(&o).lines().filter(|l| !l.trim().is_empty()).count(),
            1,
            "{args:?}: 理由が1行でない: {}",
            err(&o)
        );
        assert!(out(&o).trim().is_empty(), "{args:?}: {}", out(&o));
    }
}

#[test]
fn test_ws_9_marker_base_not_a_table_fails() {
    // [WS-9・BV-1] `base = "x"`(表でない中身)も `.base` として読めない中身なので、理由を出して開かない
    // (0 以外の終了コード)。根の既定の表(AGENTS.md の行)を開かない。
    let h = home("mb-bad-base");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\nbase = \"x\"\n",
    );
    let o = run(h.path(), &root, &["--print"]);
    assert!(
        !out(&o).contains("案内"),
        "根の既定の表を開いた\n{}",
        out(&o)
    );
    assert_ne!(o.status.code(), Some(0), "stderr={}", err(&o));
    assert!(!err(&o).trim().is_empty(), "理由が無い");
}

// ---- WS-10: where ----

/// 最上位の絞り込み(todo/ だけ)。
const IN_TODO: &str = "filters = 'file.inFolder(\"todo\")'";

#[test]
fn test_ws_10_where_view_values_any() {
    // [WS-10] ビューに `where = { status = ["todo", "doing"] }` → todo と doing の行だけ。
    let h = home("mw-any");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(IN_TODO, &["where = { status = [\"todo\", \"doing\"] }"]),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a", "b"]));
}

#[test]
fn test_ws_10_where_two_keys_all() {
    // [WS-10] `where = { status = ["done"], repo = ["tasks"] }` → done で repo が tasks の行だけ。
    let h = home("mw-two");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(
            IN_TODO,
            &["where = { status = [\"done\"], repo = [\"tasks\"] }"],
        ),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["c"]));
}

#[test]
fn test_ws_10_where_top_level_every_view() {
    // [WS-10] 最上位の `where = { kind = ["todo"] }` → どのビューも kind が todo の行だけ
    // (ビューの where と重なれば両方)。
    let h = home("mw-top");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(
            &format!("{IN_TODO}\nwhere = {{ kind = [\"todo\"] }}"),
            &["", "where = { status = [\"todo\", \"doing\"] }"],
        ),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a", "c", "d", "e"]));
    assert_eq!(rows_of(h.path(), &root, &["--view", "v2"]), want(&["a"]));
}

#[test]
fn test_ws_10_where_list_column_by_element() {
    // [WS-10] tags が `[a, b]` のノートに `where = { tags = ["b"] }` → 残る(要素のどれかが当たれば残す)。
    let h = home("mw-list");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(
            IN_TODO,
            &[
                "where = { tags = [\"b\"] }",
                "where = { tags = [\"c\", \"z\"] }",
            ],
        ),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a"]));
    assert_eq!(rows_of(h.path(), &root, &["--view", "v2"]), want(&["b"]));
}

#[test]
fn test_ws_10_where_number_and_bool_values() {
    // [WS-10] 値の配列の要素は数と真偽でもよい: `prio = [1]` → a、`flag = [true]` → c。
    let h = home("mw-num");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(
            IN_TODO,
            &["where = { prio = [1] }", "where = { flag = [true] }"],
        ),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a"]));
    assert_eq!(rows_of(h.path(), &root, &["--view", "v2"]), want(&["c"]));
}

#[test]
fn test_ws_10_where_and_filters_both() {
    // [WS-10] 同じ所の `filters` と `where` → 両方を満たす行だけ。ビュー: `filters = 'repo == "tasks"'` と
    // `where = { status = ["todo", "done"] }` → a と c。最上位: inFolder("todo") と where の両方で AGENTS.md は無い。
    let h = home("mw-both");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        &where_marker(
            &format!("{IN_TODO}\nwhere = {{ status = [\"todo\", \"done\", \"案内\"] }}"),
            &[
                "filters = 'repo == \"tasks\"'\nwhere = { status = [\"todo\", \"done\"] }",
                "",
            ],
        ),
    );
    assert_eq!(rows_of(h.path(), &root, &[]), want(&["a", "c"]));
    assert_eq!(
        rows_of(h.path(), &root, &["--view", "v2"]),
        want(&["a", "c", "d"])
    );
}

#[test]
fn test_ws_10_where_empty_array_warns_and_does_not_narrow() {
    // [WS-10] ビューの `where = { status = [] }` → `.mdgrid/workspace.toml: table[1].base.views[1].where.status:`
    // で始まる警告と、その鍵で絞らない表(todo/ の5行)。
    let h = home("mw-empty");
    let root = h.path();
    ledger(root);
    marker(root, &where_marker(IN_TODO, &["where = { status = [] }"]));
    assert_eq!(rows_of(root, root, &[]), want(&["a", "b", "c", "d", "e"]));
    screen_warns(
        root,
        ".mdgrid/workspace.toml: table[1].base.views[1].where.status:",
    );
}

#[test]
fn test_ws_10_where_not_array_warns_and_does_not_narrow() {
    // [WS-10] 最上位の `where = { status = "todo" }`(配列でない値)→ `.mdgrid/workspace.toml:
    // table[1].base.where.status:` で始まる警告と、その鍵で絞らない表。ほかの鍵(repo)は効く。
    let h = home("mw-scalar");
    let root = h.path();
    ledger(root);
    marker(
        root,
        &where_marker(
            &format!("{IN_TODO}\nwhere = {{ status = \"todo\", repo = [\"other\"] }}"),
            &[""],
        ),
    );
    assert_eq!(rows_of(root, root, &[]), want(&["b", "d"]));
    screen_warns(root, ".mdgrid/workspace.toml: table[1].base.where.status:");
}
