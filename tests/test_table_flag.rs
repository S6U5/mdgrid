#![cfg(unix)]
//! 印の表を名前で選ぶ旗 `--table <名前>`(cli.table-*)と、表のフォルダを直に開いたらその表を開くこと
//! (cli.launch-table-folder)を、本物の実行ファイルで確かめる。画面を出さない起動は標準出力のパイプで、画面は
//! 疑似端末(tests/pty/mod.rs)で見る。今日は MDGRID_TODAY=2026-11-25 に固定する。
//!
//! 材料(作った値):
//!
//! 調べ物の台帳 `r`(印 RESEARCH。開くのは「問い」):
//!
//! | ファイル                    | 中身                                   |
//! |-----------------------------|----------------------------------------|
//! | News/20261101_前の.md       | date: 2026-11-01、pick: take           |
//! | News/20261120_次の.md       | date: 2026-11-20(pick なし = 未判定) |
//! | Questions/問い一.md         | status: open                           |
//! | Other/o.md                  | kind: memo(どの表のフォルダでもない) |
//!
//! お金の台帳 `m`(印 MONEY。開くのは「支出」):
//!
//! | ファイル            | 中身                                   |
//! |---------------------|----------------------------------------|
//! | Expenses/e1.md      | paid: 2026-11-20                       |
//! | Recurring/r1.md     | status: active、paid: 2026-11-01       |
//! | Recurring/r2.md     | status: stopped、paid: 2026-10-01      |
//! | Recurring/r3.md     | status: active、paid: someday(型の違反)|

use crate::pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::process::{Command, Output};

const TODAY: &str = "2026-11-25";

/// 調べ物の台帳の印: 表「問い」(Questions/)と表「ニュース」(News/。ビューは「未判定」「全部」の順)。
const RESEARCH: &str = "open = \"問い\"\n\n[display]\nnote_name = '^\\d{8}_'\n\n\
[[table]]\nname = \"問い\"\n\n[table.new_note]\nfolder = \"Questions\"\n\n\
[table.base]\nfilters = { and = ['file.inFolder(\"Questions\")'] }\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"全部の問い\"\norder = [\"status\"]\n\n\
[[table]]\nname = \"ニュース\"\n\n[table.new_note]\nfolder = \"News\"\nname = \"{date:YYYYMMDD}_\"\n\
name_head = \"hide\"\nbody = \".mdgrid/templates/news.md\"\n\n[table.new_note.set]\ndate = \"{date}\"\n\n\
[table.base]\nfilters = { and = ['file.inFolder(\"News\")'] }\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"未判定\"\nfilters = { and = ['pick.isEmpty()'] }\norder = [\"date\", \"pick\"]\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"date\", \"pick\"]\n";

/// お金の台帳の印: 表「支出」(Expenses/)と表「定期の支払い」(Recurring/。ビューは「全部」「次の支払いの順」の順、
/// 表だけの status の値の候補)。
const MONEY: &str = "open = \"支出\"\n\n[types]\npaid = \"date\"\n\n\
[[table]]\nname = \"支出\"\n\n[table.base]\nfilters = { and = ['file.inFolder(\"Expenses\")'] }\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"今月\"\norder = [\"paid\"]\n\n\
[[table]]\nname = \"定期の支払い\"\n\n[table.rules.status]\nvalues = [\"active\", \"stopped\"]\n\n\
[table.base]\nfilters = { and = ['file.inFolder(\"Recurring\")'] }\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"全部\"\norder = [\"status\", \"paid\"]\n\n\
[[table.base.views]]\ntype = \"table\"\nname = \"次の支払いの順\"\nwhere = { status = [\"active\"] }\norder = [\"status\", \"paid\"]\n";

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// HOME(設定と状態の置き場つき)に、調べ物の台帳 `r` とお金の台帳 `m` を置く。
fn home(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    let h = t.path();
    std::fs::create_dir_all(h.join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(h.join("state")).unwrap();
    let r = h.join("r");
    write(&r, ".mdgrid/workspace.toml", RESEARCH);
    write(&r, ".mdgrid/templates/news.md", "# {name}\n\n## 要点\n");
    write(
        &r,
        "News/20261101_前の.md",
        "---\ndate: 2026-11-01\npick: take\n---\n",
    );
    write(&r, "News/20261120_次の.md", "---\ndate: 2026-11-20\n---\n");
    write(&r, "Questions/問い一.md", "---\nstatus: open\n---\n");
    write(&r, "Other/o.md", "---\nkind: memo\n---\n");
    let m = h.join("m");
    write(&m, ".mdgrid/workspace.toml", MONEY);
    write(&m, "Expenses/e1.md", "---\npaid: 2026-11-20\n---\n");
    write(
        &m,
        "Recurring/r1.md",
        "---\nstatus: active\npaid: 2026-11-01\n---\n",
    );
    write(
        &m,
        "Recurring/r2.md",
        "---\nstatus: stopped\npaid: 2026-10-01\n---\n",
    );
    write(
        &m,
        "Recurring/r3.md",
        "---\nstatus: active\npaid: someday\n---\n",
    );
    t
}

/// `home` を HOME と設定の置き場に、`cwd` を作業のフォルダにして起動する(標準出力はパイプ)。
fn run_in(home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
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
    run_in(home, home, args)
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 終了コード 0 を確かめて標準出力を返す。
fn ok(o: &Output, what: &str) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{what}: stderr={} stdout={}",
        err(o),
        out(o)
    );
    out(o)
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

/// `--with-path` の出力の、見出しを除いた各行の1列目(パス)。
fn paths(s: &str) -> Vec<String> {
    s.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').next().unwrap_or("").to_string())
        .collect()
}

fn read(p: &Path) -> String {
    String::from_utf8(std::fs::read(p).unwrap()).unwrap()
}

/// `dir` の下の全部のファイルの名前。
fn all_files(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(e.file_name().to_string_lossy().into_owned());
            }
        }
    }
    out
}

// ---- cli.table-open ----

#[test]
fn test_table_flag_prints_the_named_table_of_the_marker() {
    // [stest -> req~cli.table-open~1] 印の open が「問い」の台帳で `r --table ニュース --print --with-path` は、
    // ニュースの表(News/)の行だけを出す。
    let h = home("tf-open");
    let s = ok(
        &run(
            h.path(),
            &["r", "--table", "ニュース", "--print", "--with-path"],
        ),
        "--table ニュース",
    );
    let p = paths(&s);
    assert!(!p.is_empty(), "行が無い\n{s}");
    assert!(p.iter().all(|x| x.contains("/News/")), "News の外の行\n{s}");
    assert!(!s.contains("問い一"), "問いの行がある\n{s}");
}

#[test]
fn test_table_flag_opens_the_named_table_on_the_screen_from_a_subfolder() {
    // [stest -> req~cli.table-open~1] 今のフォルダが r/Questions で `mdgrid --table ニュース`(画面)→ 表「ニュース」の
    // タブ「未判定」「全部」で開き、News のノートの行(次の)が出る。
    let h = home("tf-screen");
    let cwd = h.path().join("r/Questions");
    let cfg = h.path().join("config");
    let state = h.path().join("state");
    let env = [
        ("HOME", h.path().to_str().unwrap()),
        ("XDG_CONFIG_HOME", cfg.to_str().unwrap()),
        ("XDG_STATE_HOME", state.to_str().unwrap()),
        ("MDGRID_TODAY", TODAY),
    ];
    let mut tui = Tui::spawn(&["--table", "ニュース"], &cwd, &env);
    tui.wait_until("ニュースの表", |s| {
        s.contains("未判定")
            && s.contains("全部")
            && s.contains("次の")
            && !s.contains("全部の問い")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

// ---- cli.table-commands ----

#[test]
fn test_table_flag_print_finds_the_view_of_the_named_table() {
    // [stest -> req~cli.table-commands~1] `m --table 定期の支払い --print --view 次の支払いの順` → 定期の支払いの active の行
    // (r1・r3)だけが出て、終了コード 0。
    let h = home("tf-view");
    let s = ok(
        &run(
            h.path(),
            &[
                "m",
                "--table",
                "定期の支払い",
                "--print",
                "--with-path",
                "--view",
                "次の支払いの順",
            ],
        ),
        "--view 次の支払いの順",
    );
    let mut p = paths(&s);
    p.sort();
    assert_eq!(p.len(), 2, "{s}");
    assert!(p[0].ends_with("Recurring/r1.md"), "{s}");
    assert!(p[1].ends_with("Recurring/r3.md"), "{s}");
}

#[test]
fn test_table_flag_count_counts_the_first_view_of_the_named_table() {
    // [stest -> req~cli.table-commands~1] `m --table 定期の支払い --count` → 最初のビュー「全部」の行の数 3(支出の表は1行)。
    let h = home("tf-count");
    let s = ok(
        &run(h.path(), &["m", "--table", "定期の支払い", "--count"]),
        "--count",
    );
    assert_eq!(s, "3\n");
}

#[test]
fn test_table_flag_check_checks_the_named_table() {
    // [stest -> req~cli.table-commands~1] `m --table 定期の支払い --check` → r3 の paid の問題の行が出て、終了コード 1。
    // 前提: 印の open の表(支出)には問題が無い(--table が無ければ終了コード 0)。
    let h = home("tf-check");
    let o = run(h.path(), &["m", "--check"]);
    assert_eq!(o.status.code(), Some(0), "{} {}", out(&o), err(&o));
    let o = run(h.path(), &["m", "--table", "定期の支払い", "--check"]);
    assert_eq!(o.status.code(), Some(1), "{} {}", out(&o), err(&o));
    let s = out(&o);
    assert_eq!(s.lines().count(), 1, "{s}");
    assert!(
        s.trim_end()
            .ends_with("Recurring/r3.md:3: paid: 型が合わない(日付ではない)"),
        "{s}"
    );
}

#[test]
fn test_table_flag_resolved_config_is_the_named_tables() {
    // [stest -> req~cli.table-commands~1] `m --table 定期の支払い --print-config --resolved` → 表「定期の支払い」にだけ書いた
    // status の values(stopped)が出る。前提: --table が無ければ(支出の表)出ない。
    let h = home("tf-resolved");
    let s = ok(
        &run(h.path(), &["m", "--print-config", "--resolved"]),
        "支出",
    );
    assert!(!s.contains("\"stopped\""), "{s}");
    let s = ok(
        &run(
            h.path(),
            &[
                "m",
                "--table",
                "定期の支払い",
                "--print-config",
                "--resolved",
            ],
        ),
        "定期の支払い",
    );
    assert!(s.contains("\"stopped\""), "{s}");
}

// ---- cli.table-new-note ----

#[test]
fn test_table_flag_new_note_uses_the_named_tables_rules() {
    // [stest -> req~cli.table-new-note~1] `r --table ニュース --new-note 見出し` → r/News/20261125_見出し.md を、ニュースの表の
    // 決まり(名前の頭・set の date・雛形)で作り、そのパスを1行出す。
    let h = home("tf-nn");
    let o = run(
        h.path(),
        &["r", "--table", "ニュース", "--new-note", "見出し"],
    );
    assert_eq!(ok(&o, "--new-note"), "r/News/20261125_見出し.md\n");
    let text = read(&h.path().join("r/News/20261125_見出し.md"));
    assert_eq!(text, "---\ndate: 2026-11-25\n---\n# 見出し\n\n## 要点\n");
}

#[test]
fn test_table_flag_new_note_from_another_table_folder() {
    // [stest -> req~cli.table-new-note~1] 今のフォルダが r/Questions で `--table ニュース --new-note 別` → r/News/20261125_別.md
    // ができ、標準出力は今のフォルダから見た `../News/20261125_別.md` の1行。
    let h = home("tf-nn-sub");
    let o = run_in(
        h.path(),
        &h.path().join("r/Questions"),
        &["--table", "ニュース", "--new-note", "別"],
    );
    assert_eq!(ok(&o, "--new-note"), "../News/20261125_別.md\n");
    let text = read(&h.path().join("r/News/20261125_別.md"));
    assert_eq!(text, "---\ndate: 2026-11-25\n---\n# 別\n\n## 要点\n");
}

#[test]
fn test_new_note_inside_a_table_folder_creates_in_that_folder() {
    // [stest -> req~cli.new-note-table-folder~1] 今のフォルダが表のフォルダ r/News で、フォルダを渡さずに `--new-note 中` →
    // r/News/20261125_中.md を作り(r/News/News/ の下には作らない)、出したパスは今のフォルダからそのファイルを指す。
    let h = home("tf-nn-here");
    let cwd = h.path().join("r/News");
    let o = run_in(h.path(), &cwd, &["--new-note", "中"]);
    let s = ok(&o, "--new-note");
    let made = h.path().join("r/News/20261125_中.md");
    assert!(made.is_file(), "{s}");
    assert!(!h.path().join("r/News/News").exists(), "{s}");
    assert_eq!(
        cwd.join(s.trim()).canonicalize().unwrap(),
        made.canonicalize().unwrap()
    );
    assert_eq!(read(&made), "---\ndate: 2026-11-25\n---\n# 中\n\n## 要点\n");
}

// ---- cli.table-unknown ----

#[test]
fn test_table_flag_unknown_name_lists_the_tables() {
    // [stest -> req~cli.table-unknown~1] `r --table 無い --print` → 標準出力は空で、理由1行に「問い」と「ニュース」、終了コード 2。
    let h = home("tf-unknown");
    let line = fails_one_line(&run(h.path(), &["r", "--table", "無い", "--print"]), "無い");
    assert!(
        line.contains("無い") && line.contains("問い") && line.contains("ニュース"),
        "{line}"
    );
}

#[test]
fn test_table_flag_unknown_name_creates_nothing() {
    // [stest -> req~cli.table-unknown~1] `r --table 無い --new-note x` → 理由1行と終了コード 2 で、r の下のどこにも x.md が無い。
    let h = home("tf-unknown-nn");
    fails_one_line(
        &run(h.path(), &["r", "--table", "無い", "--new-note", "x"]),
        "無い --new-note",
    );
    let files = all_files(&h.path().join("r"));
    assert!(
        !files.iter().any(|f| f.ends_with("x.md")),
        "作った: {files:?}"
    );
}

#[test]
fn test_table_flag_without_a_marker_fails() {
    // [stest -> req~cli.table-unknown~1] 上へたどっても印の無いフォルダ plain で `--table ニュース --print` → 印が無いことを
    // 言う理由1行(印のファイルの名前を言う)と終了コード 2。
    let h = home("tf-nomarker");
    write(h.path(), "plain/p.md", "---\nkind: x\n---\n");
    let line = fails_one_line(
        &run(h.path(), &["plain", "--table", "ニュース", "--print"]),
        "plain",
    );
    assert!(line.contains(".mdgrid/workspace.toml"), "{line}");
}

// ---- cli.table-bad-args ----

#[test]
fn test_table_flag_refuses_conflicting_arguments() {
    // [stest -> req~cli.table-bad-args~1] `--raw`・2つのパス・`-s`・`--continue`・`-w` と開くパスの組 → どれも --table を
    // 言う理由1行と終了コード 2。
    let h = home("tf-bad");
    std::fs::write(
        h.path().join("config/mdgrid/sessions.toml"),
        format!(
            "[sessions.\"朝\"]\nfolders = [\"{}\"]\n",
            h.path().join("r").display()
        ),
    )
    .unwrap();
    std::fs::write(
        h.path().join("config/mdgrid/workspaces.toml"),
        format!(
            "[[workspace]]\nname = \"仕事\"\n\n[[workspace.table]]\nname = \"ニュース\"\npath = \"{}\"\n",
            h.path().join("r/News").display()
        ),
    )
    .unwrap();
    let cases: [&[&str]; 5] = [
        &["r", "--table", "ニュース", "--raw", "--print"],
        &["r", "r/News", "--table", "ニュース", "--print"],
        &["-s", "朝", "--table", "ニュース"],
        &["--continue", "--table", "ニュース"],
        &["-w", "仕事", "r", "--table", "ニュース", "--print"],
    ];
    for args in cases {
        let line = fails_one_line(&run(h.path(), args), &format!("{args:?}"));
        assert!(line.contains("--table"), "{args:?}: {line}");
    }
}

// ---- cli.table-workspace ----

/// workspaces.toml にワークスペース「仕事」(表 a・b。どちらもフォルダ)を書く。
fn work_space(h: &Path) {
    write(h, "a/a1.md", "---\nkind: a\n---\n");
    write(h, "b/b1.md", "---\nkind: b\n---\n");
    std::fs::write(
        h.join("config/mdgrid/workspaces.toml"),
        format!(
            "[[workspace]]\nname = \"仕事\"\n\n[[workspace.table]]\nname = \"a\"\npath = \"{}\"\n\n\
             [[workspace.table]]\nname = \"b\"\npath = \"{}\"\n",
            h.join("a").display(),
            h.join("b").display()
        ),
    )
    .unwrap();
}

#[test]
fn test_table_flag_with_workspace_picks_its_table() {
    // [stest -> req~cli.table-workspace~1] `-w 仕事 --table b --print --with-path` → b のノートの行だけ。
    let h = home("tf-ws");
    work_space(h.path());
    let s = ok(
        &run(
            h.path(),
            &["-w", "仕事", "--table", "b", "--print", "--with-path"],
        ),
        "-w 仕事 --table b",
    );
    let p = paths(&s);
    assert_eq!(p.len(), 1, "{s}");
    assert!(p[0].ends_with("b1.md"), "{s}");
}

#[test]
fn test_table_flag_with_workspace_unknown_name_lists_its_tables() {
    // [stest -> req~cli.table-workspace~1] `-w 仕事 --table c --print` → a と b を並べた理由1行と終了コード 2。
    let h = home("tf-ws-unknown");
    work_space(h.path());
    let line = fails_one_line(
        &run(h.path(), &["-w", "仕事", "--table", "c", "--print"]),
        "c",
    );
    assert!(line.contains("a, b"), "{line}");
}

// ---- cli.launch-table-folder ----

#[test]
fn test_opening_a_table_folder_opens_that_table_on_the_screen() {
    // [stest -> req~cli.launch-table-folder~1] `mdgrid r/News`(画面)→ 表「ニュース」のタブ「未判定」「全部」で開く。
    let h = home("tf-folder-screen");
    let mut tui = Tui::spawn(&["r/News"], h.path(), &[("MDGRID_TODAY", TODAY)]);
    tui.wait_until("ニュースの表のタブ", |s| {
        s.contains("未判定") && s.contains("全部") && s.contains("次の")
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

#[test]
fn test_opening_a_table_folder_prints_that_table() {
    // [stest -> req~cli.launch-table-folder~1] `r/News --print --with-path` → `r --table ニュース --print --with-path` と同じ行
    // (最初のビュー「未判定」の、次の だけ)。
    let h = home("tf-folder-print");
    let by_folder = ok(
        &run(h.path(), &["r/News", "--print", "--with-path"]),
        "r/News",
    );
    let by_name = ok(
        &run(
            h.path(),
            &["r", "--table", "ニュース", "--print", "--with-path"],
        ),
        "--table ニュース",
    );
    assert_eq!(by_folder, by_name);
    let p = paths(&by_folder);
    assert_eq!(p.len(), 1, "{by_folder}");
    assert!(p[0].ends_with("News/20261120_次の.md"), "{by_folder}");
}

#[test]
fn test_opening_a_plain_folder_of_a_ledger_stays_plain() {
    // [stest -> req~cli.launch-table-folder~1] どの表のフォルダでもない r/Other → その既定の表(o.md の1行)。
    let h = home("tf-folder-other");
    let s = ok(
        &run(h.path(), &["r/Other", "--print", "--with-path"]),
        "r/Other",
    );
    let p = paths(&s);
    assert_eq!(p.len(), 1, "{s}");
    assert!(p[0].ends_with("Other/o.md"), "{s}");
}

#[test]
fn test_opening_a_table_folder_raw_is_the_plain_folder() {
    // [stest -> req~cli.launch-table-folder~1] `r/News --raw --print --with-path` → News の既定の表(前の・次の の2行。
    // 表の最初のビュー「未判定」の絞り込みを当てない)。
    let h = home("tf-folder-raw");
    let s = ok(
        &run(h.path(), &["r/News", "--raw", "--print", "--with-path"]),
        "--raw",
    );
    let mut p = paths(&s);
    p.sort();
    assert_eq!(p.len(), 2, "{s}");
    assert!(p[0].ends_with("20261101_前の.md"), "{s}");
    assert!(p[1].ends_with("20261120_次の.md"), "{s}");
}

// ---- cli.check-problems(印の表の view を当てない) ----

#[test]
fn test_check_without_view_ignores_the_marker_tables_view() {
    // [stest -> req~cli.check-problems~2] 印の open の表に `view = "済だけ"`(済 の行だけ)があっても、`--check` は表の全部の
    // 行を確かめ、そのビューの外の着手の行の問題も出す(終了コード 1)。
    let h = home("tf-check-view");
    let led = h.path().join("led");
    write(
        &led,
        ".mdgrid/workspace.toml",
        "open = \"台帳\"\n\n[types]\ndue = \"date\"\n\n[[table]]\nname = \"台帳\"\nview = \"済だけ\"\n\n\
         [table.base]\nfilters = { and = ['file.inFolder(\"todo\")'] }\n\n\
         [[table.base.views]]\ntype = \"table\"\nname = \"全部\"\n\n\
         [[table.base.views]]\ntype = \"table\"\nname = \"済だけ\"\nwhere = { status = [\"済\"] }\n",
    );
    write(&led, "todo/a.md", "---\nstatus: 着手\ndue: someday\n---\n");
    write(&led, "todo/b.md", "---\nstatus: 済\ndue: 2026-11-01\n---\n");
    let o = run(h.path(), &["led", "--print", "--with-path"]);
    assert!(
        !out(&o).contains("a.md"),
        "前提: 済だけのビュー\n{}",
        out(&o)
    );
    let o = run(h.path(), &["led", "--check"]);
    assert_eq!(o.status.code(), Some(1), "{} {}", out(&o), err(&o));
    let s = out(&o);
    assert_eq!(s.lines().count(), 1, "{s}");
    assert!(
        s.trim_end()
            .ends_with("todo/a.md:3: due: 型が合わない(日付ではない)"),
        "{s}"
    );
}
