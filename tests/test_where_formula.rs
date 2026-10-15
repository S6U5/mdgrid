#![cfg(unix)]
//! [WS-10] 印の表の `where` の鍵に計算の列(`formula.x`)を書ける。台帳の期限のタブ(計算の列で期限の区分を
//! 1回定義し、タブで区分を選ぶ)の境目もここで確かめる。

mod pty;

use pty::TempDir;
use std::path::Path;
use std::process::{Command, Output};

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

/// 今日から `d` 日ずらした日付。今日は地域の時刻の日付(mdgrid の today() と同じ)を `date +%F` で取る
/// (試験は多くの糸で動くので、time の地域の時差は取れない)。
fn day(d: i64) -> String {
    let out = Command::new("date").arg("+%F").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let p: Vec<i32> = s.split('-').map(|x| x.parse().unwrap()).collect();
    let month = time::Month::try_from(p[1] as u8).unwrap();
    let t =
        time::Date::from_calendar_date(p[0], month, p[2] as u8).unwrap() + time::Duration::days(d);
    format!("{:04}-{:02}-{:02}", t.year(), u8::from(t.month()), t.day())
}

/// 期限の区分を1回だけ計算の列で定義し、タブは区分を `where` で選ぶ印(台帳と同じ形)。
const MARKER: &str = r#"open = "台帳"

[types]
due = "date"

[[table]]
name = "台帳"

[table.base]
filters = { or = ['file.inFolder("todo")'] }

[table.base.formulas]
"期限" = 'if(due.isEmpty(), "なし", if(due < today(), "期限切れ", if(due == today(), "今日", if(due <= today() + "7d", "7日以内", "それより先"))))'

[[table.base.views]]
type = "table"
name = "全部"
order = ["title", "due", "formula.期限"]

[[table.base.views]]
type = "table"
name = "期限切れ"
where = { "formula.期限" = ["期限切れ"] }
order = ["title"]

[[table.base.views]]
type = "table"
name = "7日以内まで"
where = { "formula.期限" = ["期限切れ", "今日", "7日以内"], status = ["todo", "doing"] }
order = ["title"]
"#;

fn sample(root: &Path) {
    for d in ["todo", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    let notes = [
        ("past", Some(-1), "todo"),
        ("today", Some(0), "todo"),
        ("week", Some(7), "todo"),
        ("later", Some(8), "todo"),
        ("nodue", None, "todo"),
        ("pastdone", Some(-1), "done"),
    ];
    for (name, d, st) in notes {
        let due = d
            .map(|d| format!("due: \"{}\"\n", day(d)))
            .unwrap_or_default();
        std::fs::write(
            root.join(format!("todo/{name}.md")),
            format!("---\ntitle: {name}\nstatus: {st}\n{due}---\n"),
        )
        .unwrap();
    }
    std::fs::write(root.join(".mdgrid/workspace.toml"), MARKER).unwrap();
}

fn titles(o: &Output) -> Vec<String> {
    let mut v: Vec<String> = String::from_utf8_lossy(&o.stdout)
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty())
        .map(|l| l.split(',').next().unwrap_or("").to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn test_ws_10_where_formula_column() {
    // [WS-10] `where = { "formula.期限" = ["期限切れ"] }` → 計算の列の値で絞る(期限が昨日の行だけ)。
    let t = TempDir::new("where-formula");
    let root = t.path();
    sample(root);
    let o = run(root, &["--print", "--view", "期限切れ"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(titles(&o), vec!["past", "pastdone"]);
}

#[test]
fn test_ws_10_due_tabs_boundaries() {
    // [WS-10] 7日以内まで = 期限切れ・今日・7日以内 で、終えていないもの。今日から7日目は入り、8日目と期限の無い行は入らない。
    let t = TempDir::new("where-due");
    let root = t.path();
    sample(root);
    let o = run(root, &["--print", "--view", "7日以内まで"]);
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(titles(&o), vec!["past", "today", "week"]);
}
