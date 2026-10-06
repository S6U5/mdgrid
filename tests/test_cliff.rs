//! コミットの書き出しを git-cliff で CHANGELOG の節に写す設定(cliff.toml)の試験。
//! 変更の記録: specs/_changes/2026-10-03-cliff.md(タスク 1)。関係する要件: なし(開発の道具)。
//! git-cliff はこの環境に無いので、cliff.toml を TOML として読み、commit_parsers の
//! message の頭(`^<書き出し>` に半角と全角のコロンの文字の組が続く形)と `git log` の書き出しを突き合わせる。
//! 正規表現の crate は使わない。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 書き出しの後ろの区切り(半角の `:` と全角の `:` の両方を受ける文字の組)。
const SEP_CLASS: &str = "[:\u{ff1a}]";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_cliff() -> toml::Table {
    let path = repo_root().join("cliff.toml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()));
    text.parse::<toml::Table>()
        .unwrap_or_else(|e| panic!("{} が TOML として読めない: {e}", path.display()))
}

fn table<'a>(t: &'a toml::Table, key: &str) -> &'a toml::Table {
    t.get(key)
        .unwrap_or_else(|| panic!("cliff.toml に [{key}] が無い"))
        .as_table()
        .unwrap_or_else(|| panic!("cliff.toml の {key} が表でない"))
}

fn parsers(cliff: &toml::Table) -> Vec<toml::Table> {
    let git = table(cliff, "git");
    git.get("commit_parsers")
        .expect("[git] に commit_parsers が無い")
        .as_array()
        .expect("[git] の commit_parsers が配列でない")
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.as_table()
                .unwrap_or_else(|| panic!("commit_parsers[{i}] が表でない"))
                .clone()
        })
        .collect()
}

/// message の `^<書き出し>` + SEP_CLASS から書き出しを取り出す。形が違えば None。
fn prefix_of(message: &str) -> Option<&str> {
    let word = message.strip_prefix('^')?.strip_suffix(SEP_CLASS)?;
    let meta = |c: char| "\\^$.|?*+()[]{}:\u{ff1a}".contains(c) || c.is_whitespace();
    if word.is_empty() || word.chars().any(meta) {
        return None;
    }
    Some(word)
}

/// 各 parser の書き出しと、その行き先(節の名前か skip)。
#[derive(Debug, PartialEq)]
enum Dest {
    Group(String),
    Skip,
}

fn parser_entries(cliff: &toml::Table) -> Vec<(String, Dest)> {
    parsers(cliff)
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let message = p
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("commit_parsers[{i}] に文字列の message が無い"));
            let prefix = prefix_of(message).unwrap_or_else(|| {
                panic!("commit_parsers[{i}] の message {message:?} が `^<書き出し>[:\u{ff1a}]` の形でない")
            });
            let dest = match (p.get("group"), p.get("skip")) {
                (Some(g), None) => Dest::Group(
                    g.as_str()
                        .unwrap_or_else(|| panic!("commit_parsers[{i}] の group が文字列でない"))
                        .to_string(),
                ),
                (None, Some(s)) if s.as_bool() == Some(true) => Dest::Skip,
                _ => panic!(
                    "commit_parsers[{i}] ({message:?}) は group(文字列)か skip = true のどちらか一方を持つこと"
                ),
            };
            (prefix.to_string(), dest)
        })
        .collect()
}

/// 節の名前から、並びのための頭の HTML の注釈(`<!-- 0 -->` など)を除く。
fn group_name(g: &str) -> &str {
    let g = g.trim();
    if let Some(rest) = g.strip_prefix("<!--") {
        if let Some(end) = rest.find("-->") {
            return rest[end + 3..].trim();
        }
    }
    g
}

/// コミットの題の書き出し(最初の `:` か `:` の前)。
fn subject_prefix(subject: &str) -> &str {
    match subject.find([':', '\u{ff1a}']) {
        Some(i) => &subject[..i],
        None => subject,
    }
}

fn git_subjects(root: &Path) -> Option<Vec<String>> {
    let out = match Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%s"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln!("git を動かせないので履歴の突き合わせを飛ばす: {e}");
            return None;
        }
    };
    if !out.status.success() {
        eprintln!(
            "git log が失敗した(リポでない?)ので履歴の突き合わせを飛ばす: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
    )
}

#[test]
fn test_cliff_toml_parses_with_commit_parsers() {
    let cliff = load_cliff();
    let git = table(&cliff, "git");
    assert_eq!(
        git.get("conventional_commits").and_then(|v| v.as_bool()),
        Some(false),
        "[git] の conventional_commits は false であること"
    );
    let entries = parser_entries(&cliff);
    assert!(!entries.is_empty(), "commit_parsers が空");
}

#[test]
fn test_cliff_parser_messages_are_prefix_form() {
    let cliff = load_cliff();
    let entries = parser_entries(&cliff);
    let mut seen = BTreeSet::new();
    let dups: Vec<&str> = entries
        .iter()
        .filter(|(p, _)| !seen.insert(p.as_str()))
        .map(|(p, _)| p.as_str())
        .collect();
    assert!(dups.is_empty(), "同じ書き出しの parser が重なる: {dups:?}");
}

#[test]
fn test_cliff_history_prefixes_all_covered() {
    let cliff = load_cliff();
    let known: BTreeSet<String> = parser_entries(&cliff).into_iter().map(|(p, _)| p).collect();
    let Some(subjects) = git_subjects(&repo_root()) else {
        return;
    };
    let missing: Vec<&String> = subjects
        .iter()
        .filter(|s| !known.contains(subject_prefix(s)))
        .collect();
    assert!(
        missing.is_empty(),
        "commit_parsers のどれにも当たらないコミットが {} 件(書き出し: {:?}):\n{}",
        missing.len(),
        missing
            .iter()
            .map(|s| subject_prefix(s))
            .collect::<BTreeSet<_>>(),
        missing
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn test_cliff_group_assignment() {
    let cliff = load_cliff();
    let entries = parser_entries(&cliff);
    let expected: &[(&str, Option<&str>)] = &[
        ("実装", Some("Features")),
        ("追加", Some("Features")),
        ("修正", Some("Fixes")),
        ("文書", Some("Documentation")),
        ("specs", Some("Specification")),
        ("テスト", None),
        ("照合", None),
        ("照合の直し", None),
        ("設計", None),
        ("記録", None),
        ("整形", None),
    ];
    let mut wrong = Vec::new();
    for (prefix, want) in expected {
        let got = entries.iter().find(|(p, _)| p == prefix).map(|(_, d)| d);
        let ok = match (got, want) {
            (Some(Dest::Group(g)), Some(w)) => group_name(g) == *w,
            (Some(Dest::Skip), None) => true,
            _ => false,
        };
        if !ok {
            wrong.push(format!(
                "{prefix}: 期待 {}, 実際 {got:?}",
                want.map_or("skip".to_string(), |w| format!("group {w:?}"))
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "節の割り当てが違う:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn test_cliff_changelog_has_header_and_body() {
    let cliff = load_cliff();
    let changelog = table(&cliff, "changelog");
    for key in ["header", "body"] {
        let v = changelog
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("[changelog] に文字列の {key} が無い"));
        assert!(!v.trim().is_empty(), "[changelog] の {key} が空");
    }
    let body = changelog["body"].as_str().unwrap();
    assert!(
        body.contains("{{") || body.contains("{%"),
        "[changelog] の body がテンプレートに見えない: {body:?}"
    );
}

#[test]
fn test_cliff_prefix_helpers() {
    // 試験の道具そのものの確かめ(cliff.toml に依らない)。
    assert_eq!(prefix_of("^実装[:\u{ff1a}]"), Some("実装"));
    assert_eq!(prefix_of("^照合の直し[:\u{ff1a}]"), Some("照合の直し"));
    assert_eq!(prefix_of("^実装:"), None);
    assert_eq!(prefix_of("^実装[:]"), None);
    assert_eq!(prefix_of("^(実装|追加)[:\u{ff1a}]"), None);
    assert_eq!(prefix_of("実装[:\u{ff1a}]"), None);
    assert_eq!(subject_prefix("照合の直し: 何か"), "照合の直し");
    assert_eq!(subject_prefix("文書\u{ff1a}何か: x"), "文書");
    assert_eq!(subject_prefix("specs:採択: x"), "specs");
    assert_eq!(subject_prefix("区切り無し"), "区切り無し");
    assert_eq!(group_name("<!-- 0 -->Features"), "Features");
    assert_eq!(group_name("Fixes"), "Fixes");
}
