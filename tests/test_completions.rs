//! CLI-13 の受け入れ: `--completions <シェル>` と `--man`。実装(引数の読み取り)を見ずに書いた。
//! 本物の実行ファイルを動かし、子の環境だけで設定・状態の置き場を存在しない所へ向ける。

use std::path::PathBuf;
use std::process::{Command, Output};

/// CLI-2 の受け付けるオプションのうち、補完と man に必ず載るべき名前。
const CLI2_OPTIONS: &[&str] = &[
    "--readonly",
    "--no-color",
    "--config",
    "--view",
    "--print-config",
];

const SHELLS: &[&str] = &["bash", "zsh", "fish", "elvish", "powershell"];

fn nowhere() -> PathBuf {
    std::env::temp_dir().join(format!("mdgrid-completions-none-{}", std::process::id()))
}

fn run_in(args: &[&str], dir: Option<&PathBuf>) -> Output {
    // 利用者の設定を読まないよう、子の環境だけで設定の置き場を存在しない所へ向ける。
    let nowhere = nowhere();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    cmd.args(args)
        .env("XDG_CONFIG_HOME", &nowhere)
        .env("XDG_STATE_HOME", &nowhere);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    cmd.output().expect("mdgrid を起動できる")
}

fn run(args: &[&str]) -> Output {
    run_in(args, None)
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("標準出力は UTF-8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// 終了コード 2、標準エラーが1行(改行は末尾の1つだけ)、標準出力は空。
fn assert_one_line_error(out: &Output, what: &str) {
    let err = stderr(out);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{what}: 終了コード。stderr: {err}"
    );
    assert!(
        err.ends_with('\n') && err.matches('\n').count() == 1 && !err.trim().is_empty(),
        "{what}: 標準エラーは理由1行のはず: {err:?}"
    );
    assert!(
        out.stdout.is_empty(),
        "{what}: 標準出力は空のはず: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

fn completions(shell: &str) -> String {
    let out = run(&["--completions", shell]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--completions {shell} の終了コード。stderr: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(!text.trim().is_empty(), "--completions {shell} の出力が空");
    for opt in CLI2_OPTIONS {
        // fish は `complete -c mdgrid -l view …` の形で書くので、`--view` か `-l view` のどちらかでよい。
        let found = text.contains(opt) || (shell == "fish" && fish_has_long(&text, opt));
        assert!(found, "--completions {shell} の出力に {opt} が無い");
    }
    text
}

/// fish の補完に、長いオプション `opt`(`--x`)が `-l x` の形で(後ろに続く字なしで)あるか。
fn fish_has_long(text: &str, opt: &str) -> bool {
    let name = opt.trim_start_matches("--");
    text.lines().any(|line| {
        let words: Vec<&str> = line.split_whitespace().collect();
        words.windows(2).any(|w| w[0] == "-l" && w[1] == name)
    })
}

#[test]
fn test_cli_13_completions_bash() {
    let text = completions("bash");
    assert!(text.contains("complete"), "bash の補完に complete が無い");
}

#[test]
fn test_cli_13_completions_zsh() {
    let text = completions("zsh");
    assert!(
        text.starts_with("#compdef mdgrid"),
        "zsh の補完は #compdef mdgrid で始まるはず: {:?}",
        text.lines().next()
    );
}

#[test]
fn test_cli_13_completions_fish() {
    completions("fish");
}

#[test]
fn test_cli_13_completions_elvish() {
    completions("elvish");
}

#[test]
fn test_cli_13_completions_powershell() {
    completions("powershell");
}

#[test]
fn test_cli_13_completions_all_shells_differ() {
    // 5つのシェルの出力はそれぞれのシェルの形で、同じものの使い回しではない。
    let outs: Vec<String> = SHELLS.iter().map(|s| completions(s)).collect();
    for i in 0..outs.len() {
        for j in i + 1..outs.len() {
            assert_ne!(
                outs[i], outs[j],
                "{} と {} の補完が同じ",
                SHELLS[i], SHELLS[j]
            );
        }
    }
}

#[test]
fn test_cli_13_completions_unknown_shell() {
    let out = run(&["--completions", "nushell"]);
    assert_one_line_error(&out, "--completions nushell");
}

#[test]
fn test_cli_13_completions_missing_value() {
    let out = run(&["--completions"]);
    assert_one_line_error(&out, "値の無い --completions");
}

#[test]
fn test_cli_13_man() {
    let out = run(&["--man"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--man の終了コード。stderr: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains(".TH"), "man ページに .TH が無い");
    for opt in CLI2_OPTIONS {
        // roff では `-` が `\-` と書かれることがある。
        let escaped = opt.replace('-', "\\-");
        assert!(
            text.contains(opt) || text.contains(&escaped),
            "man ページに {opt}({escaped})が無い"
        );
    }
}

#[test]
fn test_cli_13_help_lists_new_options() {
    let out = run(&["--help"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--help の終了コード。stderr: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("--completions"),
        "--help に --completions が無い"
    );
    assert!(text.contains("--man"), "--help に --man が無い");
}

#[test]
fn test_cli_13_man_after_double_dash_is_path() {
    // `--` の後ろの `--man` はパスとして扱う。`--man` という名前のものが無い空のフォルダで動かす。
    let dir = std::env::temp_dir().join(format!("mdgrid-completions-cwd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("一時のフォルダを作れる");
    assert!(!dir.join("--man").exists());
    let out = run_in(&["--", "--man"], Some(&dir));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        !stdout(&out).contains(".TH"),
        "`--` の後ろの --man で man ページが出た"
    );
    assert_one_line_error(&out, "-- --man(無いパス)");
}
