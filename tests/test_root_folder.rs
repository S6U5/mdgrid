//! 「保管庫」をルートフォルダと言い換え、検知の値を目印のフォルダの名前にする(root-folder)の受け入れ試験。
//! WS-5: `[workspace] detect` の値は `".obsidian"`・`".git"`。旧い `"vault"`・`"git"` も同じ意味で読み、移ったと
//! 警告する。BV-2 ほか: `--help` は「ルートフォルダ」(英語 root folder)と言い、保管庫・vault と言わない。
//! 記録: specs/_changes/2026-10-13-root-folder.md(タスク #1)。環境変数は書き換えない(子の環境だけ)。

use mdgrid::config::parse;
use std::process::Command;

fn detect_debug(text: &str) -> (String, Vec<String>) {
    let (c, warnings) = parse(text).expect("読める");
    (format!("{:?}", c.workspace_detect), warnings)
}

#[test]
fn test_ws_5_detect_reads_marker_names() {
    // [WS-5] `detect = [".obsidian", ".git"]` → 両方、警告なし。
    let (d, w) = detect_debug("[workspace]\ndetect = [\".obsidian\", \".git\"]\n");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(d, "[Vault, Git]");
}

#[test]
fn test_ws_5_detect_old_names_read_with_warning() {
    // [WS-5] 旧い `"vault"`・`"git"` も同じ意味で読み、移ったと警告する(新しい値を示す)。
    let (d, w) = detect_debug("[workspace]\ndetect = [\"vault\", \"git\"]\n");
    assert_eq!(d, "[Vault, Git]");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("workspace.detect"), "{w:?}");
    assert!(w[0].contains(".obsidian"), "新しい値を示す: {w:?}");
}

#[test]
fn test_ws_5_detect_unknown_warns() {
    // [WS-5] 知らない値 → 警告して既定(`[".obsidian"]`)。警告は新しい値の名前を示す。
    let (d, w) = detect_debug("[workspace]\ndetect = [\"svn\"]\n");
    assert_eq!(d, "[Vault]");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].contains(".obsidian") && !w[0].contains("\"vault\""),
        "{w:?}"
    );
}

fn run(args: &[&str], lang: &str) -> String {
    let nowhere =
        std::env::temp_dir().join(format!("mdgrid-root-folder-none-{}", std::process::id()));
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .env("XDG_CONFIG_HOME", &nowhere)
        .env("XDG_STATE_HOME", &nowhere)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .output()
        .expect("mdgrid を起動できる");
    String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)
}

#[test]
fn test_ws_5_print_config_default_is_obsidian() {
    // [WS-5][CLI-11] --print-config の `[workspace] detect` の既定は `[".obsidian"]`。
    let text = run(&["--print-config"], "en_US.UTF-8");
    let table: toml::Table = text.parse().expect("TOML");
    let d = table["workspace"]["detect"].as_array().expect("detect");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].as_str(), Some(".obsidian"), "{text}");
    // 旧い値として `"vault"` と引用する所のほかは、説明にも vault と書かない。
    assert!(
        !text
            .replace("\"vault\"", "")
            .to_lowercase()
            .contains("vault"),
        "説明にも vault と書かない:\n{text}"
    );
}

#[test]
fn test_bv_2_help_says_root_folder() {
    // [BV-2] --help は「ルートフォルダ」(英語 root folder)と言い、保管庫・vault と言わない。
    let en = run(&["--help"], "en_US.UTF-8");
    assert!(en.contains("root folder"), "{en}");
    assert!(!en.to_lowercase().contains("vault"), "{en}");
    let ja = run(&["--help"], "ja_JP.UTF-8");
    assert!(ja.contains("ルートフォルダ"), "{ja}");
    assert!(!ja.contains("保管庫"), "{ja}");
}

#[test]
fn test_ws_5_migrate_config_rewrites_old_values() {
    // [WS-5][CLI-20] --migrate-config は検知の旧い値を目印のフォルダの名前に写す。読み直すと警告なしで同じ検知。
    for text in [
        "[workspace]\ndetect = [\"vault\", \"git\"]\n",
        "workspace_detect = [\"vault\"]\n",
    ] {
        let out = mdgrid::config::migrate(text).expect("写せる");
        assert!(!out.contains("\"vault\""), "{out}");
        assert!(out.contains("\".obsidian\""), "{out}");
        let (c, w) = parse(&out).expect("読める");
        assert!(w.is_empty(), "読み直して警告なし: {w:?}\n{out}");
        assert_eq!(format!("{:?}", c.workspace_detect).contains("Vault"), true);
    }
}
