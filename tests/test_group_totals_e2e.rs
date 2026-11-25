#![cfg(unix)]
//! まとまりごとの集計(base-view.summaries-per-group)と数の見せ方(cell-view.number-format)の、本物の実行ファイルでの
//! 確かめ(端末 80×24)。値は作ったもの。お金の台帳の形: 支出を分類でまとめたビューに金額の合計、金額は `¥` と桁の区切り。

use crate::pty;

use pty::{TempDir, Tui};
use std::path::Path;
use std::process::{Command, Output};

/// 台帳「お金」: food の 1000 と 2500、daily の 300。ビュー「分類ごと」は分類でまとめ、金額の合計を出す。
fn root(tag: &str) -> TempDir {
    let t = TempDir::new(tag);
    let h = t.path();
    for d in ["t", ".mdgrid", "config/mdgrid", "state"] {
        std::fs::create_dir_all(h.join(d)).unwrap();
    }
    for (name, amount, category) in [
        ("lunch", 1000, "food"),
        ("dinner", 2500, "food"),
        ("soap", 300, "daily"),
    ] {
        std::fs::write(
            h.join(format!("t/{name}.md")),
            format!("---\namount: {amount}\ncategory: {category}\n---\n"),
        )
        .unwrap();
    }
    std::fs::write(
        h.join(".mdgrid/workspace.toml"),
        r#"open = "お金"

[types]
amount = "number"

[format.amount]
thousands = true
prefix = "¥"

[[table]]
name = "お金"

[table.base]
filters = { or = ['file.inFolder("t")'] }

[[table.base.views]]
type = "table"
name = "分類ごと"
order = ["amount"]
groupBy = { property = "category", direction = "ASC" }
summaries = { amount = "Sum" }
"#,
    )
    .unwrap();
    t
}

/// まとまり `word` の見出しの行(畳んだ見出しも)。
fn heading(s: &str, word: &str) -> Option<String> {
    s.lines()
        .find(|l| (l.contains('▾') || l.contains('▸')) && l.contains(word))
        .map(str::to_string)
}

#[test]
fn test_group_subtotals_with_number_format_e2e() {
    // [stest -> req~base-view.summaries-per-group~1, req~cell-view.number-format~1] 開くと daily の見出しに「合計 ¥300」、food の見出しに
    // 「合計 ¥3,500」、表の下の集計の行に「合計 ¥3,800」、dinner のセルは `¥2,500`。下の帯に未対応は出ない。
    // 先頭(daily)の見出しで Enter → 畳んでも「合計 ¥300」が出たまま。
    let home = root("group-totals-e2e");
    let mut tui = Tui::spawn(&[], home.path(), &[]);
    tui.wait_until("まとまりの小計", |s| {
        heading(s, "food").is_some_and(|l| l.contains("合計 ¥3,500"))
            && heading(s, "daily").is_some_and(|l| l.contains("合計 ¥300"))
    });
    let s = tui.screen();
    assert!(s.contains("合計 ¥3,800"), "{s}");
    assert!(
        s.lines()
            .any(|l| l.contains("dinner") && l.contains("¥2,500")),
        "{s}"
    );
    assert!(!s.contains("未対応"), "{s}");
    tui.send("\r");
    tui.wait_until("畳んだ daily の見出しにも小計", |s| {
        s.contains('▸') && heading(s, "daily").is_some_and(|l| l.contains("合計 ¥300"))
    });
    tui.send("q");
    assert_eq!(tui.wait_exit(), 0);
}

fn run(home: &Path, args: &[&str]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.args(args)
        .current_dir(home)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"));
    c.output().unwrap()
}

#[test]
fn test_number_format_print_keeps_numbers_e2e() {
    // [stest -> req~cell-view.number-format-display-only~1] `--print` の amount は元の数(1000・2500・300)で、`¥` も `,` も付けない。
    let home = root("number-format-print-e2e");
    let o = run(home.path(), &["--print"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(!out.contains('¥'), "{out}");
    for n in ["1000", "2500", "300"] {
        assert!(
            out.lines().any(|l| l.split(',').any(|c| c == n)),
            "{n}: {out}"
        );
    }
}
