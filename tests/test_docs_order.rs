//! 説明書の目次とサイトの説明書の章の並びが読む人の順で、入口と説明書の文に大げさな語が無い(SR-50)。
//! 読むのはリポの実物(docs/manual/<言語>/index.md・scripts/pages.py・README)。fixture は作らず、
//! 行番号や字の位置は決め打ちせず、中身で探す。

use std::path::{Path, PathBuf};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// 読む人の順(index の次から)。themes.md はこの並びに入れない(見た目の章のどこにあってもよい)。
const ORDER: [&str; 6] = [
    "views.md",
    "getting-started.md",
    "tasks.md",
    "concepts.md",
    "appearance.md",
    "faq.md",
];

/// 日本語の大げさな語(docs/writing.md)。部分一致で探す。
const JA_WORDS: [&str; 6] = ["強力", "シームレス", "直感的", "画期的", "最適", "豊富"];

/// 英語の大げさな語(docs/writing.md)。語の頭で区切り、大文字小文字は問わない。
const EN_WORDS: [&str; 9] = [
    "seamless",
    "robust",
    "powerful",
    "intuitive",
    "leverage",
    "empower",
    "delve",
    "crucial",
    "pivotal",
];

/// `## <heading>` の節(次の `## ` まで)にある `- [題](ファイル.md)` のリンク先を、書いてある順に。
/// `#` 以降の断片は落とす。
fn toc_targets(text: &str, heading: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            inside = h.trim() == heading;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(rest) = line.trim_start().strip_prefix("- [") else {
            continue;
        };
        let Some(i) = rest.find("](") else { continue };
        let after = &rest[i + 2..];
        let Some(j) = after.find(')') else { continue };
        let target = after[..j].split('#').next().unwrap_or("").trim();
        out.push(target.to_string());
    }
    out
}

/// `names` が `seq` にすべて在り、その相対の順が守られているか。守られていなければ理由。
fn check_order(seq: &[String], names: &[&str]) -> Result<(), String> {
    let mut last: Option<(usize, &str)> = None;
    for name in names {
        let Some(pos) = seq.iter().position(|s| s == name) else {
            return Err(format!("{name} が無い"));
        };
        if let Some((last_pos, last_name)) = last {
            if pos < last_pos {
                return Err(format!("{name} が {last_name} より前にある"));
            }
        }
        last = Some((pos, name));
    }
    Ok(())
}

/// `scripts/pages.py` の `PAGES = [...]` の中の文字列を、書いてある順に(複数行でもよい)。
fn pages_list(py: &str) -> Vec<String> {
    let head = "PAGES = [";
    let start = py
        .find(head)
        .expect("scripts/pages.py に `PAGES = [` が無い");
    let body = &py[start + head.len()..];
    let end = body
        .find(']')
        .expect("scripts/pages.py の PAGES を閉じる `]` が無い");
    body[..end]
        .split(',')
        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\''))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// `docs/manual/<言語>/` の Markdown を名前の順に。
fn manual_pages(lang: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join("docs/manual").join(lang))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    v.sort();
    v
}

/// 英語の語を、語の頭で区切って(前が英字でない)、大文字小文字を問わず探す。
/// 後ろは区切らないので、powerfully・seamlessly・leverages のような活用も拾う。
fn has_en_word(line: &str, word: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(i) = lower[from..].find(word) {
        let at = from + i;
        if at == 0 || !bytes[at - 1].is_ascii_alphabetic() {
            return true;
        }
        // word は ASCII なので at の字は 1 バイト。at + 1 は字の境。
        from = at + 1;
    }
    false
}

/// 各ファイルの各行を `found` で調べ、当たった「ファイル:行: 語: 行の中身」を全部集める。
fn hits(files: &[PathBuf], words: &[&str], found: fn(&str, &str) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    for p in files {
        let text = std::fs::read_to_string(p).unwrap();
        for (n, line) in text.lines().enumerate() {
            for w in words {
                if found(line, w) {
                    out.push(format!("{}:{}: {w}: {}", rel(p), n + 1, line.trim()));
                }
            }
        }
    }
    out
}

#[test]
fn test_sr_50_manual_index_toc_in_reading_order() {
    // [SR-50] 説明書の目次(ja の `## 目次`・en の `## Contents`)のリンクは
    // views → getting-started → tasks → concepts → appearance → faq の順(間にほかの行があってもよい)。
    for (lang, heading) in [("ja", "目次"), ("en", "Contents")] {
        let path = root().join("docs/manual").join(lang).join("index.md");
        let text = std::fs::read_to_string(&path).unwrap();
        let toc = toc_targets(&text, heading);
        assert!(
            !toc.is_empty(),
            "{} に `## {heading}` の節のリンク(- [題](ファイル.md))が無い",
            rel(&path)
        );
        if let Err(why) = check_order(&toc, &ORDER) {
            panic!(
                "{} の目次: {why}(目次の並び: {})",
                rel(&path),
                toc.join(" → ")
            );
        }
    }
}

#[test]
fn test_sr_50_pages_py_pages_in_reading_order() {
    // [SR-50] サイトの説明書の章(scripts/pages.py の PAGES)は index.md の次が views.md で、
    // そのあと getting-started → tasks → concepts → appearance → faq の順。themes.md はどこでもよい。
    let py = std::fs::read_to_string(root().join("scripts/pages.py")).unwrap();
    let pages = pages_list(&py);
    let shown = pages.join(" → ");
    assert_eq!(
        pages.first().map(String::as_str),
        Some("index.md"),
        "PAGES の先頭は index.md({shown})"
    );
    assert_eq!(
        pages.get(1).map(String::as_str),
        Some("views.md"),
        "PAGES の index.md の次は views.md({shown})"
    );
    if let Err(why) = check_order(&pages, &ORDER) {
        panic!("PAGES: {why}({shown})");
    }
}

#[test]
fn test_sr_50_no_hype_words_ja() {
    // [SR-50] 日本語の説明書(docs/manual/ja/*.md)・README.ja.md・scripts/pages.py の日本語の文言に、
    // 大げさな語(強力・シームレス・直感的・画期的・最適・豊富)が無い。見つけた行を全部出す。
    let mut files = manual_pages("ja");
    files.push(root().join("README.ja.md"));
    files.push(root().join("scripts/pages.py"));
    let found = hits(&files, &JA_WORDS, |line, w| line.contains(w));
    assert!(
        found.is_empty(),
        "日本語の大げさな語が {} 件ある:\n{}",
        found.len(),
        found.join("\n")
    );
}

#[test]
fn test_sr_50_no_hype_words_en() {
    // [SR-50] 英語の説明書(docs/manual/en/*.md)・README.md・scripts/pages.py に、大げさな語
    // (seamless・robust・powerful・intuitive・leverage・empower・delve・crucial・pivotal。
    // 単語として、大文字小文字は問わず、活用も含む)が無い。見つけた行を全部出す。
    let mut files = manual_pages("en");
    files.push(root().join("README.md"));
    files.push(root().join("scripts/pages.py"));
    let found = hits(&files, &EN_WORDS, has_en_word);
    assert!(
        found.is_empty(),
        "英語の大げさな語が {} 件ある:\n{}",
        found.len(),
        found.join("\n")
    );
}
