#![cfg(unix)]
//! タスク 2(print-pick)の受け入れ: `--pick path|<列の名前>`(OUT-3・SR-10・WB-15)。
//! 仕様: specs/output/spec.md の OUT-3、specs/cell-view/spec.md の SR-10、
//! specs/_changes/2026-10-03-print-pick.md の「不明点と仮定」。
//! 実装(src/main.rs・src/ui)を見ずに、要件と公開の振る舞い(USAGE・docs/keys.md・README)だけで書いた。
//!
//! - 画面は端末(`/dev/tty`)、結果は標準出力。疑似端末の道具(tests/pty/mod.rs)の上に
//!   `sh -c 'exec "$MDGRID" "$@" > "$OUT"'` を立て、標準出力だけを普通のファイルに向ける
//!   (標準入力・標準エラー・制御端末は疑似端末のまま)。パイプの場合は `| cat > "$OUT"` で、
//!   mdgrid の終了コードは `echo $? > "$RC"` で取る。
//! - 表のキー(docs/keys.md): `Space` = mark_row(印を付けて1行下へ)、`j` = down、`G` = bottom、
//!   `g g` = top、`l` = right、`Backspace` = clear、`Ctrl+S` = save、`q` = quit、`Esc` = clear_selection。
//! - 子は Drop で kill と wait をする(落ちても止まる)。画面と終了の待ちは 10 秒で落ちる。
//!
//! 材料の保管庫(`<home>/notes`。起動の引数は相対の `notes`、子の作業フォルダは `<home>`):
//!
//! | note       | status | tags   | memo         | due        |
//! |------------|--------|--------|--------------|------------|
//! | a.md       | todo   | [a, b] | `x⏎y`        | 2026-11-01 |
//! | b.md       | doing  | [c]    | plain        | (無し)     |
//! | c.md       | done   | []     | (無し)       | (無し)     |
//! | sub/d.md   | todo   | [z]    | (無し)       | (無し)     |
//!
//! 表の並び(`.base` の無いフォルダの既定)は a.md・b.md・c.md・sub/d.md と仮定した
//! (`--print` の出力の行の順と同じ)。

mod pty;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use pty::{assert_gone, TempDir, Tui};

// ---------------------------------------------------------------- 場

struct Env {
    home: TempDir,
}

impl Env {
    fn new(tag: &str) -> Env {
        let home = TempDir::new(&format!("pick-{tag}"));
        for d in ["notes/sub", "config", "state"] {
            std::fs::create_dir_all(home.path().join(d)).unwrap();
        }
        let v = home.path().join("notes");
        let w = |rel: &str, text: &str| std::fs::write(v.join(rel), text).unwrap();
        w(
            "a.md",
            "---\nstatus: todo\ntags: [a, b]\nmemo: \"x\\ny\"\ndue: 2026-11-01\n---\nA\n",
        );
        w(
            "b.md",
            "---\nstatus: doing\ntags: [c]\nmemo: plain\n---\nB\n",
        );
        w("c.md", "---\nstatus: done\ntags: []\n---\nC\n");
        w("sub/d.md", "---\nstatus: todo\ntags: [z]\n---\nD\n");
        Env { home }
    }

    fn home(&self) -> &Path {
        self.home.path()
    }

    fn vault(&self) -> PathBuf {
        self.home().join("notes")
    }

    fn out(&self) -> PathBuf {
        self.home().join("out.txt")
    }

    fn rc(&self) -> PathBuf {
        self.home().join("rc.txt")
    }

    /// 既定の置き場($XDG_CONFIG_HOME/mdgrid/config.toml)に設定を書く。
    fn write_default_config(&self, body: &str) {
        let d = self.home().join("config").join("mdgrid");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("config.toml"), body).unwrap();
    }

    /// 保管庫の全ファイルのバイト(相対のパスごと)。
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, m: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(root, &p, m);
                } else {
                    m.insert(
                        p.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(&p).unwrap(),
                    );
                }
            }
        }
        let mut m = BTreeMap::new();
        walk(&self.vault(), &self.vault(), &mut m);
        m
    }
}

/// 標準出力の向け先。
#[derive(Clone, Copy)]
enum Sink {
    /// 普通のファイル(`exec … > "$OUT"`)。終了コードは子(= mdgrid)のもの。
    File,
    /// パイプ(`… | cat > "$OUT"`)。終了コードは `$RC` のファイルから読む。
    Pipe,
}

/// 疑似端末の上で、標準出力だけを向け先に向けて `mdgrid <args>` を起動する。
fn spawn(env: &Env, args: &[&str], sink: Sink) -> Tui {
    let script = match sink {
        Sink::File => r#"exec "$MDGRID" "$@" > "$OUT""#,
        Sink::Pipe => r#"{ "$MDGRID" "$@"; echo $? > "$RC"; } | cat > "$OUT""#,
    };
    let mut sh_args: Vec<&str> = vec!["-c", script, "sh"];
    sh_args.extend_from_slice(args);
    let out = env.out();
    let rc = env.rc();
    Tui::spawn_program(
        "/bin/sh",
        &sh_args,
        env.home(),
        &[
            ("MDGRID", env!("CARGO_BIN_EXE_mdgrid")),
            ("OUT", out.to_str().unwrap()),
            ("RC", rc.to_str().unwrap()),
        ],
    )
}

/// 起動して表(4行)が出るまで待つ。
fn open(env: &Env, args: &[&str], sink: Sink) -> Tui {
    let t = spawn(env, args, sink);
    // SR-29: 左のノートの欄は `.md` を除いた名前。
    t.wait_for(">a ");
    t.wait_for("sub/d");
    std::thread::sleep(Duration::from_millis(200));
    t
}

/// 終わるのを待ち、(mdgrid の終了コード, 標準出力) を返す。
fn finish(env: &Env, t: &mut Tui, sink: Sink) -> (u32, String) {
    let code = t.wait_exit();
    let code = match sink {
        Sink::File => code,
        Sink::Pipe => {
            let s = std::fs::read_to_string(env.rc()).unwrap_or_default();
            s.trim()
                .parse()
                .unwrap_or_else(|_| panic!("mdgrid の終了コードが取れない: {s:?}"))
        }
    };
    let out = std::fs::read_to_string(env.out()).unwrap_or_default();
    (code, out)
}

/// 表の並びと逆の順に、b.md と sub/d.md に印を付けて Enter。
fn mark_b_and_d_then_enter(t: &mut Tui) {
    t.send_settle("G"); // sub/d.md
    t.send_settle(" ");
    t.send_settle("gg");
    t.send_settle("j"); // b.md
    t.send_settle(" ");
    t.send("\r");
}

/// 標準出力がファイルでもパイプでも同じ: 印の2行のパスが表の並びの順に出て、終了コード 0。
fn assert_two_marked_paths(sink: Sink, tag: &str) {
    let env = Env::new(tag);
    let mut t = open(&env, &["notes", "--pick", "path"], sink);
    mark_b_and_d_then_enter(&mut t);
    let (code, out) = finish(&env, &mut t, sink);
    assert_eq!(out, "notes/b.md\nnotes/sub/d.md\n", "画面:\n{}", t.screen());
    assert_eq!(code, 0);
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- path

#[test]
fn test_out_3_pick_path_prints_marked_rows_in_table_order() {
    // [OUT-3] 2行に印を付けて Enter → 2つのパスが表の並びの順に1行ずつ、終了コード 0。
    // 引数を相対(`notes`)で渡したので、パスは「引数の文字 + 相対」(下のフォルダは `sub/` つき)。
    assert_two_marked_paths(Sink::File, "path-marked");
}

#[test]
fn test_out_3_pick_path_without_marks_prints_selected_row() {
    // [OUT-3] 印なしで Enter → 選んでいる行の1行。
    let env = Env::new("path-cursor");
    let mut t = open(&env, &["notes", "--pick", "path"], Sink::File);
    t.send_settle("jj"); // c.md
    t.send("\r");
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, "notes/c.md\n", "画面:\n{}", t.screen());
    assert_eq!(code, 0);
    assert_gone(t.stop());
}

#[test]
fn test_out_3_pick_path_with_absolute_argument_prints_absolute_path() {
    // [OUT-3] 引数を絶対のパスで渡す → 出るパスは「引数の文字 + 相対」= 絶対。
    let env = Env::new("path-abs");
    let vault = env.vault();
    let arg = vault.to_str().unwrap();
    let mut t = open(&env, &[arg, "--pick", "path"], Sink::File);
    t.send("\r"); // 最初の行(a.md)
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, format!("{arg}/a.md\n"), "画面:\n{}", t.screen());
    assert_eq!(code, 0);
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- 列の値

/// a.md・b.md・sub/d.md に印を付けて Enter し、標準出力を返す(終了コード 0 を確かめる)。
fn pick_column_of_a_b_d(tag: &str, column: &str) -> String {
    let env = Env::new(tag);
    // 画面の日付の形を変えても、出る値は YYYY-MM-DD(change record の仮定)。
    env.write_default_config("[dates]\nformat = \"YYYY/MM/DD\"\n");
    let mut t = open(&env, &["notes", "--pick", column], Sink::File);
    t.send_settle(" "); // a.md に印、b.md へ
    t.send_settle(" "); // b.md に印、c.md へ
    t.send_settle("G"); // sub/d.md
    t.send_settle(" ");
    t.send("\r");
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(code, 0, "画面:\n{}", t.screen());
    assert_gone(t.stop());
    out
}

#[test]
fn test_out_3_pick_column_prints_values_one_per_line() {
    // [OUT-3] `--pick status` → 値が1行ずつ(表の並びの順)。
    assert_eq!(
        pick_column_of_a_b_d("col-status", "status"),
        "todo\ndoing\ntodo\n"
    );
}

#[test]
fn test_out_3_pick_list_column_joins_with_comma_space() {
    // [OUT-3] リストの列 → リストを `, ` でつなぐ(画面の印は付けない素の文字)。
    assert_eq!(pick_column_of_a_b_d("col-tags", "tags"), "a, b\nc\nz\n");
}

#[test]
fn test_out_3_pick_column_newline_becomes_space_and_missing_is_empty() {
    // [OUT-3] 改行を含む値は空白に。キーの無いセルは空の1行。
    assert_eq!(pick_column_of_a_b_d("col-memo", "memo"), "x y\nplain\n\n");
}

#[test]
fn test_out_3_pick_date_column_is_iso_regardless_of_date_format() {
    // [OUT-3] 日付は設定の date_format でなく YYYY-MM-DD。キーの無いセルは空の1行。
    assert_eq!(pick_column_of_a_b_d("col-due", "due"), "2026-11-01\n\n\n");
}

// ---------------------------------------------------------------- 取りやめ

#[test]
fn test_out_3_q_cancels_with_no_output_and_exit_1() {
    // [OUT-3] `q` → 何も出さず終了コード 1。
    let env = Env::new("cancel-q");
    let mut t = open(&env, &["notes", "--pick", "path"], Sink::File);
    t.send("q");
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, "", "画面:\n{}", t.screen());
    assert_eq!(code, 1);
    assert_gone(t.stop());
}

#[test]
fn test_out_3_esc_without_marks_cancels_with_exit_1() {
    // [OUT-3] 印も範囲も無いときの Esc → 何も出さず終了コード 1。
    let env = Env::new("cancel-esc");
    let mut t = open(&env, &["notes", "--pick", "path"], Sink::File);
    t.send("\x1b");
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, "", "画面:\n{}", t.screen());
    assert_eq!(code, 1);
    assert_gone(t.stop());
}

#[test]
fn test_out_3_esc_with_marks_only_clears_marks() {
    // [OUT-3] 印があるときの Esc は印を解くだけで終わらない。続けて Enter → 選んでいる行の1行。
    let env = Env::new("esc-marks");
    let mut t = open(&env, &["notes", "--pick", "path"], Sink::File);
    t.send_settle(" "); // a.md に印、b.md へ
    t.send_settle("\x1b");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        t.try_exit(),
        None,
        "印を解く Esc で終わった。画面:\n{}",
        t.screen()
    );
    t.send("\r");
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, "notes/b.md\n", "画面:\n{}", t.screen());
    assert_eq!(code, 0);
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- 無い列

#[test]
fn test_out_3_unknown_column_exits_2_with_one_line_reason_without_screen() {
    // [OUT-3] 表に無い列の名前 → 画面を出さずにすぐ、理由1行と終了コード 2(キーは何も送らない)。
    let env = Env::new("unknown-col");
    let mut t = spawn(&env, &["notes", "--pick", "無い列"], Sink::File);
    let code = t.wait_exit();
    std::thread::sleep(Duration::from_millis(200));
    let screen = t.screen();
    let lines: Vec<&str> = screen
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(code, 2, "画面:\n{screen}");
    assert_eq!(lines.len(), 1, "理由は1行のはず。画面:\n{screen}");
    assert!(
        lines[0].contains("無い列"),
        "理由に列の名前が無い: {:?}",
        lines[0]
    );
    assert!(!screen.contains("ノート"), "表が出た。画面:\n{screen}");
    let out = std::fs::read_to_string(env.out()).unwrap_or_default();
    assert_eq!(out, "", "標準出力に何か出た");
    assert_gone(t.stop());
}

// ---------------------------------------------------------------- パイプ

#[test]
fn test_out_3_pick_works_when_stdout_is_pipe() {
    // [OUT-3][SR-10] 標準出力がパイプ(`| cat`)でも、ファイルのときと同じ結果。
    assert_two_marked_paths(Sink::Pipe, "pipe");
}

// ---------------------------------------------------------------- 読むだけ

#[test]
fn test_out_3_pick_opens_readonly_and_writes_nothing() {
    // [OUT-3][WB-15] 読むだけで開く: セルを空にし(Backspace)、保存(Ctrl+S)を試みても、
    // 「読むだけ」と出て、終わったあと保管庫のファイルのバイトが同じ(増えも減りもしない)。
    let env = Env::new("readonly");
    let before = env.snapshot();
    let mut t = open(&env, &["notes", "--pick", "path"], Sink::File);
    t.send_settle("l");
    t.send_settle("\x7f"); // Backspace = clear
    t.send_settle("l");
    t.send_settle("\x7f");
    t.send_settle("\x13"); // Ctrl+S = save
    t.wait_for("読むだけ");
    t.send("q");
    // 万一ためる変更ができて終わりの確認が出たら、捨てて終える(書かないことを確かめるのが目的)。
    std::thread::sleep(Duration::from_millis(1500));
    if t.try_exit().is_none() {
        t.send("d");
    }
    let (code, out) = finish(&env, &mut t, Sink::File);
    assert_eq!(out, "", "画面:\n{}", t.screen());
    assert_eq!(code, 1);
    assert_gone(t.stop());
    assert_eq!(env.snapshot(), before, "保管庫のファイルが変わった");
}
