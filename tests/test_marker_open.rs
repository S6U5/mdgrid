#![cfg(unix)]
//! タスク 1(marker-open)の受け入れ: 印 `.mdgrid/workspace.toml` の最上位の `open`(WS-7)と、
//! 印の根のフォルダを開いたときにその表を開くこと・`--raw`(CLI-1)。
//! 仕様: specs/cli/spec.md の CLI-1、specs/workspace/spec.md の WS-7、
//! specs/_changes/2026-10-15-marker-open.md、specs/_decisions/2026-10-15-marker-open.md。
//! 実装を見ずに書いた。本物の実行ファイルを動かす。画面を出さない起動(`--print`)も `open` を同じに
//! 使う(CLI-1)ので、開いた表の行は主に `--print --with-path` の標準出力(パイプ)で確かめ、
//! 一覧(CLI-19)を重ねないことと `--raw` の画面は疑似端末(tests/pty/mod.rs)で確かめる。
//!
//! 材料の台帳(作った値):
//!
//! | ファイル                 | 中身                                                   |
//! |--------------------------|--------------------------------------------------------|
//! | tasks.base               | `file.inFolder("todo")`、ビュー「全部」と「済だけ」    |
//! | todo/a.md                | status: 着手                                           |
//! | todo/b.md                | status: 済                                             |
//! | AGENTS.md                | status: 案内(台帳でないファイル。既定の表にだけ入る) |
//! | memo/m.md                | status: 覚え(自動の表の試験だけ)                     |
//!
//! 仮定(仕様に書かれていない細部): `--with-path` のパスの頭(`./` やフォルダの名前)は問わず、
//! 末尾の相対のパスで行を見分ける。行の並びは問わない。

mod pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::process::{Command, Output};

/// 台帳の .base(行の頭の字下げを残すため生の文字列で書く)。
const BASE: &str = r#"filters: file.inFolder("todo")
views:
  - type: table
    name: 全部
    order:
      - file.name
      - status
  - type: table
    name: 済だけ
    filters: status == "済"
    order:
      - file.name
      - status
"#;

/// 書いた表 `台帳`(tasks.base)を開く印。
const MARKER_WRITTEN: &str =
    "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\npath = \"tasks.base\"\n";

/// 書いた表 `台帳` にビュー「済だけ」を付けた印。
const MARKER_WITH_VIEW: &str =
    "open = \"台帳\"\n\n[[table]]\nname = \"台帳\"\npath = \"tasks.base\"\nview = \"済だけ\"\n";

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

/// `root` に台帳を置く(印は書かない)。
fn ledger(root: &Path) {
    std::fs::create_dir_all(root.join("todo")).unwrap();
    std::fs::write(root.join("tasks.base"), BASE).unwrap();
    std::fs::write(root.join("todo/a.md"), "---\nstatus: 着手\n---\n本文 a\n").unwrap();
    std::fs::write(root.join("todo/b.md"), "---\nstatus: 済\n---\n本文 b\n").unwrap();
    std::fs::write(root.join("AGENTS.md"), "---\nstatus: 案内\n---\n# 案内\n").unwrap();
}

/// `root` に印を書く。
fn marker(root: &Path, text: &str) {
    std::fs::create_dir_all(root.join(".mdgrid")).unwrap();
    std::fs::write(root.join(".mdgrid/workspace.toml"), text).unwrap();
}

/// HOME(設定の置き場つき)を作る。
fn home(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    std::fs::create_dir_all(t.path().join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(t.path().join("state")).unwrap();
    t
}

/// `--with-path` の出力の行のうち、`rel` で終わるパスの行があるか(1列目で見る)。
fn has_row(s: &str, rel: &str) -> bool {
    s.lines().skip(1).any(|l| {
        let p = l.split(',').next().unwrap_or("");
        p == rel || p.ends_with(&format!("/{rel}"))
    })
}

/// 表の行の数(見出しの1行を除く。空の行は数えない)。
fn rows(s: &str) -> usize {
    s.lines().skip(1).filter(|l| !l.trim().is_empty()).count()
}

/// tasks.base の最初のビュー(「全部」)の行だけ: a と b があり、AGENTS.md の行は無い。
fn assert_open_table(s: &str, what: &str) {
    assert!(has_row(s, "todo/a.md"), "{what}: a の行が無い\n{s}");
    assert!(has_row(s, "todo/b.md"), "{what}: b の行が無い\n{s}");
    assert!(
        !s.contains("AGENTS.md"),
        "{what}: AGENTS.md の行がある\n{s}"
    );
    assert_eq!(rows(s), 2, "{what}\n{s}");
}

/// 根の既定の表: AGENTS.md の行もある。
fn assert_default_table(s: &str, what: &str) {
    assert!(has_row(s, "AGENTS.md"), "{what}: AGENTS.md の行が無い\n{s}");
    assert!(has_row(s, "todo/a.md"), "{what}: a の行が無い\n{s}");
    assert!(has_row(s, "todo/b.md"), "{what}: b の行が無い\n{s}");
}

// ---- CLI-1: 根のフォルダを開くと印の表 ----

#[test]
fn test_cli_1_marker_open_no_args_in_root() {
    // [CLI-1] 根で引数なし(`--print --with-path`)→ tasks.base の表。AGENTS.md の行は無い。
    let h = home("mo-noargs");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let o = run(h.path(), &root, &["--print", "--with-path"]);
    let e = err(&o);
    let s = ok(o, "引数なし");
    assert_open_table(&s, "引数なし");
    // 印の表を指す `open` は知っている項目なので、警告は出さない。
    assert!(!e.contains("workspace.toml: open:"), "要らない警告: {e}");
}

#[test]
fn test_cli_1_marker_open_dot() {
    // [CLI-1] 根で `mdgrid .` → tasks.base の表。
    let h = home("mo-dot");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let s = ok(
        run(h.path(), &root, &[".", "--print", "--with-path"]),
        "`.`",
    );
    assert_open_table(&s, "`.`");
}

#[test]
fn test_cli_1_marker_open_root_path_from_elsewhere() {
    // [CLI-1] ほかのフォルダから根のパスを渡す(相対と絶対)→ tasks.base の表。
    let h = home("mo-path");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let s = ok(
        run(h.path(), h.path(), &["led", "--print", "--with-path"]),
        "相対",
    );
    assert_open_table(&s, "相対のパス");
    let abs = root.to_str().unwrap();
    let s = ok(
        run(h.path(), h.path(), &[abs, "--print", "--with-path"]),
        "絶対",
    );
    assert_open_table(&s, "絶対のパス");
}

#[test]
fn test_cli_1_marker_open_print_first_view_rows() {
    // [CLI-1] 根で `mdgrid --print` → tasks.base の最初のビュー(「全部」)の行。案内の行は無い。
    let h = home("mo-print");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let s = ok(run(h.path(), &root, &["--print"]), "--print");
    assert!(s.contains("着手") && s.contains("済"), "{s}");
    assert!(!s.contains("案内"), "AGENTS.md の行がある\n{s}");
    assert_eq!(rows(&s), 2, "{s}");
}

#[test]
fn test_cli_1_marker_open_raw_gives_default_table() {
    // [CLI-1] 根で `mdgrid --raw` と `mdgrid . --raw` → AGENTS.md の行もある既定の表。
    // `mdgrid --raw --print` → 既定の表の行(案内もある)。
    let h = home("mo-raw");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let s = ok(
        run(h.path(), &root, &["--raw", "--print", "--with-path"]),
        "--raw",
    );
    assert_default_table(&s, "--raw");
    let s = ok(
        run(h.path(), &root, &[".", "--raw", "--print", "--with-path"]),
        ". --raw",
    );
    assert_default_table(&s, ". --raw");
    let s = ok(run(h.path(), &root, &["--raw", "--print"]), "--raw --print");
    assert!(s.contains("案内"), "{s}");
}

#[test]
fn test_cli_1_marker_open_raw_without_marker_changes_nothing() {
    // [CLI-1] 印の無いフォルダでの `--raw` は何も変えない(出力が `--raw` 無しと同じ)。
    let h = home("mo-raw-none");
    let root = h.path().join("led");
    ledger(&root);
    let plain = ok(run(h.path(), &root, &["--print", "--with-path"]), "旗なし");
    let raw = ok(
        run(h.path(), &root, &["--raw", "--print", "--with-path"]),
        "--raw",
    );
    assert_eq!(raw, plain);
    assert_default_table(&raw, "印なしの --raw");
}

#[test]
fn test_cli_1_marker_open_subfolder_does_not_search_up() {
    // [CLI-1] 下の todo/ を開く(引数で渡す・todo で引数なし)→ todo の既定の表。印は上へたどらない。
    // 印の表はビュー「済だけ」(b だけ)なので、上へたどれば a の行が消えて見分けられる。
    let h = home("mo-sub");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WITH_VIEW);
    let s = ok(
        run(h.path(), &root, &["todo", "--print", "--with-path"]),
        "todo",
    );
    assert!(has_row(&s, "a.md") && has_row(&s, "b.md"), "{s}");
    let s = ok(
        run(h.path(), &root.join("todo"), &["--print", "--with-path"]),
        "todo で引数なし",
    );
    assert!(has_row(&s, "a.md") && has_row(&s, "b.md"), "{s}");
}

#[test]
fn test_cli_1_marker_open_two_folders_ignore_marker() {
    // [CLI-1] 根と別のフォルダの2つを渡す → 2つの既定の表(AGENTS.md の行もある)。
    let h = home("mo-two");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    std::fs::create_dir_all(h.path().join("other")).unwrap();
    std::fs::write(h.path().join("other/o.md"), "---\nstatus: 別\n---\n").unwrap();
    let s = ok(
        run(
            h.path(),
            h.path(),
            &["led", "other", "--print", "--with-path"],
        ),
        "2つ",
    );
    assert_default_table(&s, "2つのフォルダ");
    assert!(has_row(&s, "other/o.md"), "{s}");
}

#[test]
fn test_cli_1_marker_open_md_file_ignores_marker() {
    // [CLI-1・CLI-15] `mdgrid AGENTS.md` → 根の既定の表の AGENTS.md の行(印の表には無い行)。
    let h = home("mo-md");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WRITTEN);
    let s = ok(
        run(h.path(), &root, &["AGENTS.md", "--print", "--with-path"]),
        "AGENTS.md",
    );
    assert!(has_row(&s, "AGENTS.md"), "{s}");
    assert_eq!(rows(&s), 1, "{s}");
}

#[test]
fn test_cli_1_marker_open_screen_skips_places_list() {
    // [CLI-1] places.toml に登録があっても、根で引数なしの起動は一覧(CLI-19)を出さずに印の表を開く。
    let h = home("mo-places");
    let root = h.path();
    ledger(root);
    marker(root, MARKER_WRITTEN);
    std::fs::create_dir_all(root.join("shelf")).unwrap();
    std::fs::write(root.join("shelf/novel.md"), "---\nauthor: 誰か\n---\n").unwrap();
    std::fs::write(
        root.join("config/mdgrid/places.toml"),
        "[[place]]\nname = \"本棚\"\npath = \"~/shelf\"\n",
    )
    .unwrap();
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("一覧を重ねない印の表", |s| {
        s.contains("着手")
            && s.contains("済")
            && !s.contains("案内")
            && !s.contains("AGENTS")
            && !s.contains("今のフォルダ")
            && !s.contains("本棚")
            && !s.contains("workspace.toml: open:")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_cli_1_marker_open_screen_raw_shows_default_table() {
    // [CLI-1] 根で `mdgrid --raw`(画面)→ AGENTS.md の行もある既定の表。
    let h = home("mo-raw-screen");
    let root = h.path();
    ledger(root);
    marker(root, MARKER_WRITTEN);
    let mut tui = Tui::spawn(&["--raw"], root, &[]);
    tui.wait_until("既定の表", |s| {
        s.contains("案内") && s.contains("着手") && s.contains("済")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

// ---- WS-7: 印の `open` ----

#[test]
fn test_ws_7_marker_open_written_table_view() {
    // [WS-7] `open = "台帳"` と `[[table]] name = "台帳"`・`path = "tasks.base"`・`view = "済だけ"`
    // → `mdgrid led` が tasks.base の「済だけ」のビュー(b だけ)。`--view 全部` を添えればそちら。
    let h = home("mo-view");
    let root = h.path().join("led");
    ledger(&root);
    marker(&root, MARKER_WITH_VIEW);
    let s = ok(
        run(h.path(), h.path(), &["led", "--print", "--with-path"]),
        "view",
    );
    assert!(has_row(&s, "todo/b.md"), "{s}");
    assert!(!has_row(&s, "todo/a.md"), "「済だけ」に a がある\n{s}");
    assert!(!s.contains("AGENTS.md"), "{s}");
    assert_eq!(rows(&s), 1, "{s}");
    let s = ok(
        run(
            h.path(),
            h.path(),
            &["led", "--view", "全部", "--print", "--with-path"],
        ),
        "--view",
    );
    assert_open_table(&s, "--view 全部");
}

#[test]
fn test_ws_7_marker_open_auto_table_by_folder_name() {
    // [WS-7] 表を書かない印に `open = "todo"` → 自動の表 todo を開く(memo と AGENTS.md の行は無い)。
    let h = home("mo-auto");
    let root = h.path().join("led");
    ledger(&root);
    std::fs::create_dir_all(root.join("memo")).unwrap();
    std::fs::write(root.join("memo/m.md"), "---\nstatus: 覚え\n---\n").unwrap();
    marker(&root, "open = \"todo\"\n");
    let s = ok(
        run(h.path(), &root, &["--print", "--with-path"]),
        "自動の表",
    );
    assert!(has_row(&s, "a.md") && has_row(&s, "b.md"), "{s}");
    assert!(!s.contains("m.md"), "memo の行がある\n{s}");
    assert!(!s.contains("AGENTS.md"), "AGENTS.md の行がある\n{s}");
    assert_eq!(rows(&s), 2, "{s}");
}

#[test]
fn test_ws_7_marker_open_unknown_name_warns_and_opens_default() {
    // [WS-7] `open = "無い"` → `.mdgrid/workspace.toml: open:` で始まる警告(標準エラー)と、根の既定の表。
    let h = home("mo-unknown");
    let root = h.path().join("led");
    ledger(&root);
    marker(
        &root,
        "open = \"無い\"\n\n[[table]]\nname = \"台帳\"\npath = \"tasks.base\"\n",
    );
    let o = run(h.path(), &root, &["--print", "--with-path"]);
    let e = err(&o);
    let s = ok(o, "無い名前");
    assert_default_table(&s, "無い名前");
    assert!(
        e.lines().any(|l| l
            .strip_prefix("mdgrid: ")
            .unwrap_or(l)
            .starts_with(".mdgrid/workspace.toml: open:")),
        "警告が無い: {e}"
    );
}

#[test]
fn test_ws_7_marker_open_screen_uses_view() {
    // [WS-7] 画面でも、印の表の `view`(「済だけ」)で開く(`--print` と同じ行)。
    let h = home("mo-view-screen");
    let root = h.path();
    ledger(root);
    marker(root, MARKER_WITH_VIEW);
    let mut tui = Tui::spawn(&[], root, &[]);
    tui.wait_until("印の表のビュー", |s| {
        s.contains("済だけ") && s.matches('済').count() >= 2
    });
    std::thread::sleep(std::time::Duration::from_millis(500));
    let s = tui.screen();
    assert!(!s.contains("着手"), "「済だけ」に a の行がある\n{s}");
    assert!(!s.contains("案内"), "{s}");
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}
