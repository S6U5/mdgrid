//! `--print` の行の並びを、グループ分けの列の値の候補(`[rules.<列>] values`)の順にする(cli.print-group-order)。
//! 画面の navigation.group-values-order と同じ並び。本物の実行ファイルを動かし、標準出力はパイプ。値は作ったもの。
//!
//! 材料: alpha は status doing・owner carol、beta は todo・alice、gamma は waiting・bob(名前の順・status の文字の順・
//! 候補の順がどれも違う)。グループ分けしたビューは、印の `[table.base]` のビュー(groupBy)と、views.toml の
//! mdgrid のビュー(ビューの設定の group)の2つの形で作る。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-print-group-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 値の候補(受け入れの例の順)。
const RULES: &str = "[rules.status]\nvalues = [\"todo\", \"doing\", \"waiting\"]\n";

const NOTES: [(&str, &str); 3] = [
    ("alpha.md", "status: doing\nowner: carol\n"),
    ("beta.md", "status: todo\nowner: alice\n"),
    ("gamma.md", "status: waiting\nowner: bob\n"),
];

/// グループ分けしたビューの作り方。
#[derive(Clone, Copy, Debug)]
enum Shape {
    /// 印の `[table.base]` のビュー(groupBy)。
    Base,
    /// views.toml の mdgrid のビュー(設定の group)。
    Native,
}

/// `home` を HOME と設定・状態の置き場に、`cwd` を作業のフォルダにして動かす。
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
        .env("MDGRID_TODAY", "2026-11-25")
        .output()
        .unwrap()
}

/// 台帳(根 `<home>/led`、ノートは `notes/`)と、列 `status`・`owner` でまとめた2つのビュー「状態ごと」「担当ごと」を作る。
fn ledger(home: &Path, shape: Shape) -> PathBuf {
    let root = home.join("led");
    let notes = root.join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::create_dir_all(home.join("config/mdgrid")).unwrap();
    std::fs::create_dir_all(home.join("state")).unwrap();
    for (name, fm) in NOTES {
        std::fs::write(notes.join(name), format!("---\n{fm}---\n")).unwrap();
    }
    std::fs::create_dir_all(root.join(".mdgrid")).unwrap();
    let marker = match shape {
        Shape::Base => {
            let view = |name: &str, col: &str| {
                format!("\n[[table.base.views]]\ntype = \"table\"\nname = \"{name}\"\norder = [\"file.name\", \"status\", \"owner\"]\ngroupBy = {{ property = \"{col}\", direction = \"ASC\" }}\n")
            };
            format!(
                "open = \"T\"\n\n{RULES}\n[[table]]\nname = \"T\"\n\n[table.base]\nfilters = {{ or = ['file.inFolder(\"notes\")'] }}\n{}{}",
                view("状態ごと", "status"),
                view("担当ごと", "owner")
            )
        }
        Shape::Native => {
            let real = notes.canonicalize().unwrap();
            let p = real
                .to_str()
                .unwrap()
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            let view = |name: &str, col: &str| {
                format!("\n  [[target.view]]\n  name = \"{name}\"\n  order = [\"status\", \"owner\"]\n  hidden = []\n  filters_expr = []\n  [target.view.settings]\n  group = {{ By = {{ col = \"{col}\", dir = \"Asc\", hide_empty = false }} }}\n")
            };
            std::fs::write(
                home.join("config/mdgrid/views.toml"),
                format!(
                    "[[target]]\npath = \"{p}\"\n{}{}",
                    view("状態ごと", "status"),
                    view("担当ごと", "owner")
                ),
            )
            .unwrap();
            format!("open = \"T\"\n\n{RULES}\n[[table]]\nname = \"T\"\npath = \"notes\"\n")
        }
    };
    std::fs::write(root.join(".mdgrid/workspace.toml"), marker).unwrap();
    root
}

/// `--print --view <view>` の CSV の列 `col` の値(上から)。
fn column(home: &Path, root: &Path, view: &str, col: &str) -> Vec<String> {
    let o = run(home, root, &["--print", "--view", view]);
    let out = String::from_utf8_lossy(&o.stdout).into_owned();
    assert_eq!(
        o.status.code(),
        Some(0),
        "stderr={} stdout={out}",
        String::from_utf8_lossy(&o.stderr)
    );
    let mut lines = out.lines();
    let head: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
    let at = head
        .iter()
        .position(|h| *h == col)
        .unwrap_or_else(|| panic!("列 {col} が無い:\n{out}"));
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').nth(at).unwrap_or_default().to_string())
        .collect()
}

#[test]
fn test_print_group_order_follows_candidates() {
    // [stest -> req~cli.print-group-order~1] 印に [rules.status] values = ["todo", "doing", "waiting"] と、status で
    // グループ分けしたビュー、status が doing・todo・waiting の3行で `--print --view <そのビュー>` → 行は todo・doing・waiting の順。
    for shape in [Shape::Base, Shape::Native] {
        let home = TempDir::new(&format!("cand-{shape:?}"));
        let root = ledger(&home.0, shape);
        assert_eq!(
            column(&home.0, &root, "状態ごと", "status"),
            ["todo", "doing", "waiting"],
            "{shape:?}"
        );
    }
}

#[test]
fn test_print_group_order_without_candidates_keeps_text_order() {
    // [stest -> req~cli.print-group-order~1] 値の候補の無い列(owner)でグループ分けしたビューの `--print` は、今までどおり値の文字の順。
    for shape in [Shape::Base, Shape::Native] {
        let home = TempDir::new(&format!("text-{shape:?}"));
        let root = ledger(&home.0, shape);
        assert_eq!(
            column(&home.0, &root, "担当ごと", "owner"),
            ["alice", "bob", "carol"],
            "{shape:?}"
        );
    }
}
