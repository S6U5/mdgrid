//! language(specs/_changes/2026-10-03-language.md)タスク1の受け入れ: 本物の実行ファイルの
//! `--help` と起動できない理由が、環境と設定の `language` で英語と日本語に切り替わる(SR-23)。
//! 実装を見ずに書いた。試験のプロセスの環境変数は書き換えない(子の環境だけ)。
//!
//! 子の環境: LC_ALL・LC_MESSAGES・MDGRID_CONFIG を外し、LANG は試験ごとに決める(外す試験もある)。
//! 設定と状態の置き場(XDG_CONFIG_HOME・XDG_STATE_HOME)は一時フォルダ。
//!
//! 仮定した形: 設定の最上位の項目 `language`(`"auto"`・`"en"`・`"ja"`。既定 `"auto"`)。
//! `--print-config` はそれを `language = "auto"` の行で出す。

use mdgrid::config::{parse, Config};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::SystemTime;

// ---- 一時フォルダ ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-language-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
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

const EN: Option<&str> = Some("en_US.UTF-8");
const JA: Option<&str> = Some("ja_JP.UTF-8");

/// 子の環境を決めて動かす。`lang` が None なら LANG も外す。`config` があれば一時フォルダに
/// 書いて `--config <パス>` を先頭に足す。
fn run(name: &str, lang: Option<&str>, config: Option<&str>, args: &[&str]) -> Output {
    let t = TempDir::new(name);
    let conf = t.path().join("xdg-config");
    let state = t.path().join("xdg-state");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    if let Some(text) = config {
        let p = t.path().join("config.toml");
        std::fs::write(&p, text).unwrap();
        cmd.arg("--config").arg(&p);
    }
    cmd.args(args)
        .current_dir(t.path())
        .env("XDG_CONFIG_HOME", &conf)
        .env("XDG_STATE_HOME", &state)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG");
    match lang {
        Some(v) => {
            cmd.env("LANG", v);
        }
        None => {
            cmd.env_remove("LANG");
        }
    }
    cmd.output().expect("mdgrid を起動できる")
}

/// ひらがな・カタカナ・漢字と、日本語の約物(・「」、。)。
fn is_japanese_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}'
        | '\u{30A0}'..='\u{30FF}'
        | '\u{31F0}'..='\u{31FF}'
        | '\u{FF66}'..='\u{FF9F}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '「' | '」' | '、' | '。')
}

fn japanese_chars(s: &str) -> String {
    s.chars().filter(|c| is_japanese_char(*c)).collect()
}

fn stdout_ok(out: &Output, what: &str) -> String {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{what}: 終了コード 0。stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).expect("出力は UTF-8")
}

/// 標準エラーの空でない行がちょうど1行・終了コード 2。その行を返す。
fn one_error_line(out: &Output, what: &str) -> String {
    assert_eq!(
        out.status.code(),
        Some(2),
        "{what}: 終了コード 2。stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8(out.stderr.clone()).expect("標準エラーは UTF-8");
    let lines: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "{what}: 標準エラーは理由1行: {lines:?}");
    lines[0].to_string()
}

const OPTIONS: [&str; 11] = [
    "--view",
    "--readonly",
    "--no-color",
    "--config",
    "--print",
    "--format",
    "--pick",
    "--print-config",
    "--completions",
    "--man",
    "--version",
];

fn assert_english_help(help: &str, what: &str) {
    let bad = japanese_chars(help);
    assert!(
        bad.is_empty(),
        "{what}: 英語の --help に日本語の文字 {bad:?} がある\n---\n{help}"
    );
    assert!(
        help.to_lowercase().contains("usage"),
        "{what}: 英語の --help に usage が無い\n---\n{help}"
    );
    for opt in OPTIONS {
        assert!(
            help.contains(opt),
            "{what}: 英語の --help に {opt} が無い\n---\n{help}"
        );
    }
}

fn assert_japanese_help(help: &str, what: &str) {
    assert!(
        help.contains("使い方"),
        "{what}: 日本語の --help に「使い方」が無い\n---\n{help}"
    );
    for opt in OPTIONS {
        assert!(
            help.contains(opt),
            "{what}: 日本語の --help に {opt} が無い\n---\n{help}"
        );
    }
}

// ---- --help ----

#[test]
fn test_sr_23_help_en_with_lang_en() {
    // [SR-23] LANG=en_US.UTF-8 → --help が英語。
    let out = run("help-en", EN, None, &["--help"]);
    let help = stdout_ok(&out, "LANG=en --help");
    assert_english_help(&help, "LANG=en");
}

#[test]
fn test_sr_23_help_ja_with_lang_ja() {
    // [SR-23] LANG=ja_JP.UTF-8 → 日本語。
    let out = run("help-ja", JA, None, &["--help"]);
    let help = stdout_ok(&out, "LANG=ja --help");
    assert_japanese_help(&help, "LANG=ja");
}

#[test]
fn test_sr_23_help_config_ja_wins_over_lang_en() {
    // [SR-23] LANG=en_US.UTF-8 で設定に language = "ja" → 日本語。
    let out = run("help-conf-ja", EN, Some("language = \"ja\"\n"), &["--help"]);
    let help = stdout_ok(&out, "LANG=en language=ja --help");
    assert_japanese_help(&help, "LANG=en language=ja");
}

#[test]
fn test_sr_23_help_config_en_wins_over_lang_ja() {
    // [SR-23] LANG=ja_JP.UTF-8 で設定に language = "en" → 英語。
    let out = run("help-conf-en", JA, Some("language = \"en\"\n"), &["--help"]);
    let help = stdout_ok(&out, "LANG=ja language=en --help");
    assert_english_help(&help, "LANG=ja language=en");
}

#[test]
fn test_sr_23_help_config_auto_follows_lang() {
    // [SR-23] language = "auto" は環境に従う。
    let out = run(
        "help-auto-ja",
        JA,
        Some("language = \"auto\"\n"),
        &["--help"],
    );
    assert_japanese_help(&stdout_ok(&out, "auto ja"), "LANG=ja language=auto");
    let out = run(
        "help-auto-en",
        EN,
        Some("language = \"auto\"\n"),
        &["--help"],
    );
    assert_english_help(&stdout_ok(&out, "auto en"), "LANG=en language=auto");
}

#[test]
fn test_sr_23_help_no_locale_env_is_english() {
    // [SR-23] LC_ALL・LC_MESSAGES・LANG のどれも無い → 英語。
    let out = run("help-none", None, None, &["--help"]);
    let help = stdout_ok(&out, "環境変数なし --help");
    assert_english_help(&help, "環境変数なし");
}

// ---- 起動できない理由 ----

#[test]
fn test_sr_23_missing_path_reason_en() {
    // [SR-23] LANG=en で無いパス → 理由の1行が英語、終了コード 2。
    let out = run("missing-en", EN, None, &["no-such-dir-xyz"]);
    let line = one_error_line(&out, "LANG=en 無いパス");
    let bad = japanese_chars(&line);
    assert!(bad.is_empty(), "英語の理由に日本語の文字 {bad:?}: {line}");
    assert!(
        line.contains("no-such-dir-xyz"),
        "理由がパスを名指さない: {line}"
    );
}

#[test]
fn test_sr_23_missing_path_reason_ja() {
    // [SR-23] LANG=ja で無いパス → 日本語(今と同じ文)、終了コード 2。
    let out = run("missing-ja", JA, None, &["no-such-dir-xyz"]);
    let line = one_error_line(&out, "LANG=ja 無いパス");
    assert!(
        line.contains("開けないパス"),
        "日本語の理由ではない: {line}"
    );
    assert!(
        line.contains("no-such-dir-xyz"),
        "理由がパスを名指さない: {line}"
    );
}

#[test]
fn test_sr_23_missing_path_reason_config_ja() {
    // [SR-23] LANG=en でも設定の language = "ja" なら起動の理由も日本語。
    let out = run(
        "missing-conf-ja",
        EN,
        Some("language = \"ja\"\n"),
        &["no-such-dir-xyz"],
    );
    let line = one_error_line(&out, "LANG=en language=ja 無いパス");
    assert!(
        line.contains("開けないパス"),
        "日本語の理由ではない: {line}"
    );
}

#[test]
fn test_sr_23_unknown_option_en() {
    // [SR-23] LANG=en で知らないオプション → 英語の1行、終了コード 2。
    let out = run("unknown-en", EN, None, &["--no-such-option-xyz"]);
    let line = one_error_line(&out, "LANG=en 知らないオプション");
    let bad = japanese_chars(&line);
    assert!(bad.is_empty(), "英語の理由に日本語の文字 {bad:?}: {line}");
    assert!(
        line.contains("--no-such-option-xyz"),
        "理由がオプションを名指さない: {line}"
    );
}

#[test]
fn test_sr_23_unknown_option_ja() {
    // [SR-23] LANG=ja で知らないオプション → 日本語の1行(今と同じ)、終了コード 2。
    let out = run("unknown-ja", JA, None, &["--no-such-option-xyz"]);
    let line = one_error_line(&out, "LANG=ja 知らないオプション");
    assert!(
        !japanese_chars(&line).is_empty(),
        "日本語の理由ではない: {line}"
    );
}

// ---- 設定の language ----

#[test]
fn test_sr_23_print_config_has_language_auto() {
    // [SR-23] [CLI-11] --print-config に language = "auto" の行があり、読み直すと既定と同じ。
    let out = run("print-config", EN, None, &["--print-config"]);
    let text = stdout_ok(&out, "--print-config");
    assert!(
        text.lines().any(|l| {
            let l = l.trim();
            l.starts_with("language")
                && l.split_once('=')
                    .map(|(k, v)| k.trim() == "language" && v.trim() == "\"auto\"")
                    .unwrap_or(false)
        }),
        "--print-config に language = \"auto\" の行が無い\n---\n{text}"
    );
    let (c, warnings) = parse(&text).expect("--print-config の出力は設定として読める");
    assert!(warnings.is_empty(), "読み直しで警告: {warnings:?}");
    assert_eq!(
        c.language,
        Config::default().language,
        "読み直した設定が既定と違う"
    );
    let (mut got, mut want) = (c.resolved(), Config::default().resolved());
    got.origins.clear();
    want.origins.clear();
    assert_eq!(got, want, "読み直した設定が既定と違う");
}

#[test]
fn test_sr_23_language_values_read_without_warning() {
    // [SR-23] language の auto・en・ja は警告なしで読める(--print-config と並べて確かめる)。
    for v in ["auto", "en", "ja"] {
        let text = format!("language = \"{v}\"\n");
        let (_c, warnings) = parse(&text).unwrap_or_else(|e| panic!("{text}: 読めない: {e:?}"));
        assert!(warnings.is_empty(), "{text}: 警告 {warnings:?}");
    }
}
