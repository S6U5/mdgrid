//! 名前を付けた条件(specs/requirements/condition.md)と画面の式の絞り込み(navigation.expr-filter)を、
//! 本物の実行ファイルで確かめる。`--print`・`--count` は子の標準出力で、画面は疑似端末(80×24)で見る。
//! 今日は `MDGRID_TODAY=2026-11-21` に固定する。期待値は要求の受け入れの例から取った。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 一時フォルダ(落とすと消す)。コマンドの試験は疑似端末を使わないので、Windows でも回す。
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
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const TODAY: &str = "2026-11-21";

const CONF: &str = "\
[conditions.soon]
expr = 'due <= today() + \"7d\"'
label = \"期限が近い\"
key = 1
[conditions.far]
expr = 'due > today() + \"30d\"'
hide = true
key = 9
";

/// notes に5つのノート(a は 11-28、b は 11-29、c は 11-24 で done、d は 2027-01-20、e は due なし)と設定。
fn home(tag: &str, conf: &str) -> TempDir {
    let h = TempDir::new(tag);
    for d in ["config/mdgrid", "state", "notes"] {
        std::fs::create_dir_all(h.path().join(d)).unwrap();
    }
    let notes = [
        ("a.md", "due: 2026-11-28\nstatus: todo"),
        ("b.md", "due: 2026-11-29\nstatus: todo"),
        ("c.md", "due: 2026-11-24\nstatus: done"),
        ("d.md", "due: 2027-01-20\nstatus: todo"),
        ("e.md", "status: todo"),
    ];
    for (n, fm) in notes {
        std::fs::write(h.path().join("notes").join(n), format!("---\n{fm}\n---\n")).unwrap();
    }
    std::fs::write(h.path().join("config/mdgrid/config.toml"), conf).unwrap();
    h
}

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("MDGRID_TODAY", TODAY)
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// `--print --with-path` の行のノートの名前(見出しを除く)。
fn printed(o: &Output) -> Vec<String> {
    let mut v: Vec<String> = out(o)
        .lines()
        .skip(1)
        .filter_map(|l| l.split(',').next())
        .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn test_condition_define_print_filter() {
    // [stest -> req~condition.define~1] cond("soon") は 2026-11-28 の行を出し、2026-11-29 の行を出さない。
    let h = home("cond-define", CONF);
    let o = run(
        h.path(),
        &[
            "notes",
            "--print",
            "--with-path",
            "--filter",
            "cond(\"soon\")",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let rows = printed(&o);
    assert!(rows.contains(&"a.md".to_string()), "{rows:?}");
    assert!(!rows.contains(&"b.md".to_string()), "{rows:?}");
}

#[test]
fn test_condition_unknown_name_on_the_command_line() {
    // [stest -> req~condition.unknown-name~1] 無い名前は理由1行と終了コード 2。
    let h = home("cond-unknown", CONF);
    let o = run(
        h.path(),
        &["notes", "--print", "--filter", "cond(\"sooon\")"],
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(
        err(&o).contains("条件「sooon」はありません(ある名前: far, soon)"),
        "{}",
        err(&o)
    );
    assert_eq!(err(&o).lines().count(), 1, "{}", err(&o));
}

#[test]
fn test_condition_hide_everywhere_in_print() {
    // [stest -> req~condition.hide-everywhere~1] hide の far の行は --print に出ない。
    let h = home("cond-hide-print", CONF);
    let o = run(h.path(), &["notes", "--print", "--with-path"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(printed(&o), ["a.md", "b.md", "c.md", "e.md"]);
}

#[test]
fn test_condition_show_hidden_cli() {
    // [stest -> req~condition.show-hidden-cli~2] --show-hidden で隠す条件を当てず、--print などが無ければ理由と 2。
    let h = home("cond-show-hidden", CONF);
    let o = run(
        h.path(),
        &["notes", "--print", "--with-path", "--show-hidden"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(printed(&o).len(), 5);
    let o = run(h.path(), &["notes", "--show-hidden"]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(err(&o).lines().count(), 1, "{}", err(&o));
}

#[test]
fn test_condition_count_prints_only_the_number() {
    // [stest -> req~condition.count~2] 当たる行の数だけの1行と終了コード 0。当たらなければ 0。
    let h = home("cond-count", CONF);
    let o = run(
        h.path(),
        &["notes", "--count", "--filter", "cond(\"soon\")"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "2\n");
    let o = run(
        h.path(),
        &["notes", "--count", "--filter", "status == \"nothing\""],
    );
    assert_eq!(out(&o), "0\n");
    // いつも隠す far の行は数えない(--show-hidden で数える)。
    assert_eq!(out(&run(h.path(), &["notes", "--count"])), "4\n");
    assert_eq!(
        out(&run(h.path(), &["notes", "--count", "--show-hidden"])),
        "5\n"
    );
}

#[test]
fn test_condition_count_with_view() {
    // [stest -> req~condition.count~2] --view を添えると、そのビューの絞り込みのあとで数える。
    let h = home("cond-count-view", CONF);
    let base = "views:\n  - type: table\n    name: 進行中\n    filters: 'status == \"todo\"'\n";
    std::fs::write(h.path().join("notes/tasks.base"), base).unwrap();
    let o = run(
        h.path(),
        &["notes/tasks.base", "--view", "進行中", "--count"],
    );
    assert!(o.status.success(), "{}", err(&o));
    // todo は a・b・d・e の4つで、far の d は隠す。
    assert_eq!(out(&o), "3\n");
}

#[test]
fn test_condition_count_like_print() {
    // [stest -> req~condition.count-like-print~2] 読めない表・--print と一緒は理由1行と 2。
    let h = home("cond-count-like", CONF);
    let o = run(h.path(), &["missing", "--count"]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(err(&o).lines().count(), 1, "{}", err(&o));
    let o = run(h.path(), &["notes", "--count", "--print"]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(err(&o).lines().count(), 1, "{}", err(&o));
    let o = run(h.path(), &["notes", "--count", "--filter", "due <="]);
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(out(&o), "");
}

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, TempDir, CONF, TODAY};
    use crate::pty::Tui;

    /// 起動の直後の1打は、端末の地の色の問い合わせ(テーマの auto)に吸われることがあるので、`done` まで押し直す。
    fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done(&t.screen()) && std::time::Instant::now() < end {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    fn spawn(h: &TempDir) -> Tui {
        let t = Tui::spawn(&["notes"], h.path(), &[("MDGRID_TODAY", TODAY)]);
        // 行の名前は拡張子なしで出るので、行は due の値で見分ける(a は 11-28・b は 11-29・c は 11-24・d は 2027-01-20)。
        t.wait_for("2026-11-28");
        t
    }

    #[test]
    fn test_condition_digit_keys_on_screen() {
        // [stest -> req~condition.digit-keys~3] soon に振った `1` で soon の行だけ、もう一度で戻る。
        let h = home("cond-digit-screen", CONF);
        let mut t = spawn(&h);
        press_until(&mut t, "1", |s| s.contains("1 期限が近い"));
        t.wait_until("soon で絞る", |s| {
            s.contains("1 期限が近い") && !s.contains("2026-11-29")
        });
        let s = t.screen();
        assert!(s.contains("2026-11-28") && s.contains("2026-11-24"), "{s}");
        t.send("1");
        t.wait_until("戻る", |s| s.contains("2026-11-29"));
        t.send("q");
        t.wait_exit();
    }

    #[test]
    fn test_condition_hide_always_on_screen() {
        // [stest -> req~condition.hide-always~2] far の行は初めから出ず、帯に far と隠した数。
        let h = home("cond-hide-screen", CONF);
        let t = spawn(&h);
        t.wait_until("隠す帯", |s| s.contains("far を隠す 1"));
        assert!(!t.screen().contains("2027-01-20"), "{}", t.screen());
        drop(t);
    }

    #[test]
    fn test_expr_filter_on_screen() {
        // [stest -> req~navigation.expr-filter~2] パレットの式で絞ると当たる行だけになり、帯に式が出る。
        let h = home("cond-expr-screen", CONF);
        let mut t = spawn(&h);
        press_until(&mut t, ":", |s| {
            s.lines().nth(1).is_some_and(|l| l.contains(':')) || s.contains("\n: ")
        });
        t.send("expr_filter");
        t.wait_for("式で絞り込む");
        t.send("\r");
        t.wait_for("式:");
        t.send("status == \"done\"\r");
        t.wait_until("式で絞る", |s| {
            s.contains("式: status == \"done\"") && !s.contains("2026-11-28")
        });
        assert!(t.screen().contains("2026-11-24"), "{}", t.screen());
        drop(t);
    }
}
