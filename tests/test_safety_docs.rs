//! 書き戻しの安全の文書(safety-docs)の受け入れ: WB-19(と WB-1〜WB-8・WB-12・WB-16 を書くこと)。
//! 仕様: specs/write-back/spec.md の WB-19。記録: specs/_changes/2026-10-03-safety-docs.md。
//! 実装(src/frontmatter.rs・src/writeback.rs の中身)を見ずに、公開の型と関数だけから書いた。
//!
//! 使う公開の口: `mdgrid::frontmatter::{parse, ReadOnly}`、`mdgrid::source::markdown::Markdown`
//! (`open`・`set_add_frontmatter`)、`mdgrid::source::Source`(`load`・`rows`・`label`・`get(..).lock`)。
//!
//! 仮定した文書の形: docs/safety.md(英語)と docs/safety.ja.md(日本語)。
//!
//! - 節(見出しは次の文字列とちょうど同じ `## ` の行):
//!   - 英: `## What mdgrid changes`・`## Read-only notes`・`## External changes`・`## How a file is written`
//!   - 日: `## mdgrid が変えるもの`・`## 読むだけのノート`・`## 外の変更`・`## ファイルの書き方`
//!
//!   節は見出しの次の行から、次の `# `・`## ` の見出しの手前まで。
//! - `## Read-only notes` / `## 読むだけのノート` の節の中に、読むだけの理由の種類(`ReadOnly` の腕)ごとに
//!   小見出し `` ### `<名前>` `` を1つ置く。名前は腕の名前を snake_case にしたもの:
//!   `no_frontmatter`・`empty_frontmatter`・`bom`・`not_utf8`・`mixed_newlines`・`duplicate_key`・
//!   `unclosed`・`invalid_yaml`・`hard_link`。この節の `### ` の小見出しは、これらの名前のどれかでなければならない。
//!   小見出しの範囲は次の `### `・`## `・`# ` の見出しの手前まで。
//! - `hard_link` 以外の小見出しの下には、例のノートを ```` ```text ```` のコードブロックでちょうど1つ置く。
//!   `hard_link` は読み取りでなく保存で見つかるので例を置かず、説明だけ(例のブロックは見ない)。
//! - `no_frontmatter`・`empty_frontmatter` の小見出しの本文には `add_frontmatter` の語を書く
//!   (既定では書け、`add_frontmatter = false` のときだけ読むだけになるため。WB-3)。
//! - `## How a file is written` / `## ファイルの書き方` の節には `fsync` の語を書く。
//! - 英の文書は `](safety.ja.md)` で日本語へ、日の文書は `](safety.md)` で英語へリンクする(`./` を前に付けてもよい)。
//!
//! 例のノートのバイトへの直し方:
//!
//! - コードブロックの中の各行の後ろに `\n`(LF)を付けて並べる(最後の行にも付く)。
//! - そのうえで、次の印をバイトに置き換える(印は大文字。ほかの `<...>` はそのまま):
//!   - `<BOM>` → EF BB BF(UTF-8 の BOM)
//!   - `<CRLF>` → 0D 0A。行の終わりに置いたときは、その行の `\n` を付けない(`<CRLF>` が改行の代わりになる)
//!   - `<CR>` → 0D。行の終わりに置いたときは同じく、その行の `\n` を付けない
//!   - `<XX>`(X は 16 進の数字 `0-9A-F` の2文字。例: `<FF>`・`<FE>`)→ その1バイト
//!
//! 確かめること:
//!
//! 1. 英・日それぞれで、`ReadOnly` のすべての腕に小見出しがある(腕の一覧は `name` の網羅の match。腕が
//!    足されるとコンパイルが通らない)。
//! 2. 各例(hard_link を除く)を `parse` に通すと、その名前の `ReadOnly` で Err になる。さらに Source で開くと:
//!    `no_frontmatter`・`empty_frontmatter` は既定(add_frontmatter = true)で書け、false で読むだけになる。
//!    ほかの形は既定でも false でも読むだけになる。
//! 3. 必要な節の見出しがあり、英・日が互いにリンクする。

use mdgrid::frontmatter::{parse, ReadOnly};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source};
use std::path::{Path, PathBuf};

const EN: &str = "docs/safety.md";
const JA: &str = "docs/safety.ja.md";

/// 節の見出し(英, 日)。
const SEC_CHANGES: (&str, &str) = ("What mdgrid changes", "mdgrid が変えるもの");
const SEC_READ_ONLY: (&str, &str) = ("Read-only notes", "読むだけのノート");
const SEC_EXTERNAL: (&str, &str) = ("External changes", "外の変更");
const SEC_WRITE: (&str, &str) = ("How a file is written", "ファイルの書き方");

// ---- 読むだけの理由の種類 ----

/// 腕の名前(snake_case)。網羅の match なので、ReadOnly に腕が足されるとここでコンパイルが落ちる。
/// そのときは名前をここに足し、`all_reasons` にも足し、文書に小見出しと例を足す。
fn name(r: &ReadOnly) -> &'static str {
    match r {
        ReadOnly::NoFrontmatter => "no_frontmatter",
        ReadOnly::EmptyFrontmatter => "empty_frontmatter",
        ReadOnly::Bom => "bom",
        ReadOnly::NotUtf8 => "not_utf8",
        ReadOnly::MixedNewlines => "mixed_newlines",
        ReadOnly::DuplicateKey(_) => "duplicate_key",
        ReadOnly::Unclosed => "unclosed",
        ReadOnly::InvalidYaml => "invalid_yaml",
        ReadOnly::HardLink => "hard_link",
    }
}

/// すべての腕(中身は捨てる)。
fn all_reasons() -> Vec<ReadOnly> {
    vec![
        ReadOnly::NoFrontmatter,
        ReadOnly::EmptyFrontmatter,
        ReadOnly::Bom,
        ReadOnly::NotUtf8,
        ReadOnly::MixedNewlines,
        ReadOnly::DuplicateKey(String::new()),
        ReadOnly::Unclosed,
        ReadOnly::InvalidYaml,
        ReadOnly::HardLink,
    ]
}

fn all_names() -> Vec<&'static str> {
    all_reasons().iter().map(name).collect()
}

/// 既定では書け、add_frontmatter = false のときだけ読むだけになる形(WB-3)。
fn depends_on_setting(n: &str) -> bool {
    n == "no_frontmatter" || n == "empty_frontmatter"
}

// ---- 文書を読む道具 ----

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("[WB-19] {} を読めない: {e}", p.display()))
}

/// `## <title>` の節の中身(見出しの次の行から、次の `# `・`## ` の手前まで)。無ければ None。
fn section<'a>(doc: &'a str, title: &str) -> Option<Vec<&'a str>> {
    let heading = format!("## {title}");
    let mut lines = doc.lines();
    lines.by_ref().find(|l| l.trim_end() == heading)?;
    Some(
        lines
            .take_while(|l| !(l.starts_with("# ") || l.starts_with("## ")))
            .collect(),
    )
}

/// 節の中の `` ### `<名前>` `` の小見出しごとに、(名前, 中身の行)。`### ` の行が名前の形でなければ名前は None。
fn subsections<'a>(sec: &[&'a str]) -> Vec<(Option<String>, Vec<&'a str>)> {
    let mut out: Vec<(Option<String>, Vec<&'a str>)> = Vec::new();
    let mut cur: Option<(Option<String>, Vec<&'a str>)> = None;
    for l in sec {
        if let Some(rest) = l.strip_prefix("### ") {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            let rest = rest.trim();
            let n = rest
                .strip_prefix('`')
                .and_then(|r| r.strip_suffix('`'))
                .filter(|n| !n.is_empty() && !n.contains('`'))
                .map(str::to_string);
            cur = Some((n, Vec::new()));
        } else if let Some((_, body)) = cur.as_mut() {
            body.push(l);
        }
    }
    if let Some(c) = cur.take() {
        out.push(c);
    }
    out
}

/// 小見出しの中の ```` ```text ```` のコードブロックの中身(行の並び)をすべて。
fn text_blocks(body: &[&str]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for l in body {
        let t = l.trim();
        match cur.as_mut() {
            None => {
                if t.starts_with("```") && t.trim_start_matches('`').trim() == "text" {
                    cur = Some(Vec::new());
                }
            }
            Some(lines) => {
                if t.starts_with("```") && t.trim_start_matches('`').trim().is_empty() {
                    out.push(cur.take().unwrap());
                } else {
                    lines.push(l.to_string());
                }
            }
        }
    }
    assert!(cur.is_none(), "[WB-19] 閉じない ```text ブロックがある");
    out
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// 1行の印をバイトに置き換える。行の終わりが `<CRLF>`・`<CR>` だったら true(その行の `\n` を付けない)。
fn decode_line(line: &str, out: &mut Vec<u8>) -> bool {
    let b = line.as_bytes();
    let mut i = 0;
    let mut ends_with_newline_marker = false;
    while i < b.len() {
        ends_with_newline_marker = false;
        let rest = &b[i..];
        if rest.starts_with(b"<BOM>") {
            out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            i += 5;
        } else if rest.starts_with(b"<CRLF>") {
            out.extend_from_slice(b"\r\n");
            i += 6;
            ends_with_newline_marker = true;
        } else if rest.starts_with(b"<CR>") {
            out.push(b'\r');
            i += 4;
            ends_with_newline_marker = true;
        } else if rest.len() >= 4 && rest[0] == b'<' && rest[3] == b'>' {
            match (hex_digit(rest[1]), hex_digit(rest[2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 4;
                }
                _ => {
                    out.push(b[i]);
                    i += 1;
                }
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    ends_with_newline_marker
}

/// 例のノートの行を、印を置き換えたバイトにする。
fn example_bytes(lines: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    for l in lines {
        if !decode_line(l, &mut out) {
            out.push(b'\n');
        }
    }
    out
}

/// 文書の読むだけの節から、名前ごとの例(hard_link は None)。小見出しの形の誤りはここで落とす。
fn examples(doc_path: &str, sec_title: &str) -> Vec<(String, Option<Vec<u8>>, String)> {
    let doc = repo_file(doc_path);
    let sec = section(&doc, sec_title)
        .unwrap_or_else(|| panic!("[WB-19] {doc_path}: `## {sec_title}` の節が無い"));
    let known = all_names();
    let mut out = Vec::new();
    for (n, body) in subsections(&sec) {
        let n = n.unwrap_or_else(|| {
            panic!(
                "[WB-19] {doc_path}: `## {sec_title}` の `### ` は `` ### `<名前>` `` の形にする"
            )
        });
        assert!(
            known.contains(&n.as_str()),
            "[WB-19] {doc_path}: 知らない読むだけの理由の名前 `{n}`(知っている名前: {known:?})"
        );
        assert!(
            !out.iter().any(|(m, _, _): &(String, _, _)| *m == n),
            "[WB-19] {doc_path}: `{n}` の小見出しが2つある"
        );
        let text = body.join("\n");
        if n == "hard_link" {
            out.push((n, None, text));
            continue;
        }
        let blocks = text_blocks(&body);
        assert_eq!(
            blocks.len(),
            1,
            "[WB-19] {doc_path}: `{n}` の下に ```text の例のノートはちょうど1つ(今は {})",
            blocks.len()
        );
        out.push((n, Some(example_bytes(&blocks[0])), text));
    }
    out
}

fn docs() -> [(&'static str, &'static str); 2] {
    [(EN, SEC_READ_ONLY.0), (JA, SEC_READ_ONLY.1)]
}

// ---- Source で開く道具(tests/test_empty_frontmatter.rs と同じ口) ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-{}-{}-{}",
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

    fn write(&self, name: &str, bytes: &[u8]) {
        std::fs::write(self.0.join(name), bytes).unwrap();
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open_with(folder: &Path, add_frontmatter: bool) -> Markdown {
    let mut md = Markdown::open(&[folder.to_path_buf()]).expect("Markdown::open");
    md.set_add_frontmatter(add_frontmatter);
    md
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| {
            let labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
            panic!("row {label} not found in {labels:?}")
        })
}

fn lock_of(src: &dyn Source, label: &str, col: &str) -> Option<String> {
    src.get(&row_of(src, label), col).lock
}

// ---- 試験 ----

#[test]
fn test_wb_19_reason_names_are_distinct() {
    // [WB-19] 試験の側の前提: 腕の名前はすべて違う。
    let names = all_names();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "{names:?}");
}

#[test]
fn test_wb_19_every_read_only_reason_is_documented() {
    // [WB-19] 英・日それぞれで、ReadOnly のすべての腕に `` ### `<名前>` `` の小見出しがある。
    for (path, title) in docs() {
        let got: Vec<String> = examples(path, title)
            .into_iter()
            .map(|(n, _, _)| n)
            .collect();
        let missing: Vec<&str> = all_names()
            .into_iter()
            .filter(|n| !got.iter().any(|g| g == n))
            .collect();
        assert!(
            missing.is_empty(),
            "[WB-19] {path}: 読むだけの理由の小見出しが無い: {missing:?}"
        );
    }
}

#[test]
fn test_wb_19_examples_parse_to_their_reason() {
    // [WB-19] 各例(hard_link を除く)を parse に通すと、その名前の ReadOnly で Err になる。
    for (path, title) in docs() {
        for (n, bytes, _) in examples(path, title) {
            let Some(bytes) = bytes else { continue };
            match parse(&bytes) {
                Ok(_) => panic!(
                    "[WB-19] {path}: `{n}` の例が読むだけにならない(parse が Ok)。例のバイト: {:?}",
                    String::from_utf8_lossy(&bytes)
                ),
                Err(r) => assert_eq!(
                    name(&r),
                    n,
                    "[WB-19] {path}: `{n}` の例が別の理由 {r:?} になる。例のバイト: {:?}",
                    String::from_utf8_lossy(&bytes)
                ),
            }
        }
    }
}

#[test]
fn test_wb_19_examples_are_read_only_when_opened() {
    // [WB-19] [WB-3] [WB-5] 各例を Source で開く。no_frontmatter・empty_frontmatter は既定で書け、
    // add_frontmatter = false で読むだけ(理由つき)。ほかの形は既定でも false でも読むだけ(理由つき)。
    for (path, title) in docs() {
        let exs: Vec<(String, Vec<u8>)> = examples(path, title)
            .into_iter()
            .filter_map(|(n, b, _)| b.map(|b| (n, b)))
            .collect();
        for on in [true, false] {
            let dir = TempDir::new(&format!("safety-docs-{on}"));
            for (n, b) in &exs {
                dir.write(&format!("{n}.md"), b);
            }
            dir.write("plain.md", b"---\nstatus: done\n---\nbody\n");
            let mut md = open_with(dir.path(), on);
            load_all(&mut md);
            for (n, _) in &exs {
                let lock = lock_of(&md, &format!("{n}.md"), "status");
                if on && depends_on_setting(n) {
                    assert_eq!(
                        lock, None,
                        "[WB-3] {path}: `{n}` の例は既定(add_frontmatter = true)では書ける"
                    );
                } else {
                    assert!(
                        lock.as_deref().is_some_and(|r| !r.trim().is_empty()),
                        "[WB-19] {path}: `{n}` の例が読むだけにならない(add_frontmatter = {on}): {lock:?}"
                    );
                }
            }
            assert_eq!(
                lock_of(&md, "plain.md", "status"),
                None,
                "plain writable ({on})"
            );
        }
    }
}

#[test]
fn test_wb_19_setting_dependent_forms_mention_add_frontmatter() {
    // [WB-19] [WB-3] no_frontmatter・empty_frontmatter の説明は add_frontmatter の設定に触れる。
    for (path, title) in docs() {
        for (n, _, text) in examples(path, title) {
            if depends_on_setting(&n) {
                assert!(
                    text.contains("add_frontmatter"),
                    "[WB-19] {path}: `{n}` の説明に `add_frontmatter` が無い"
                );
            }
        }
    }
}

#[test]
fn test_wb_19_sections_present() {
    // [WB-19] 変える範囲・読むだけのノート・外の変更・書き込みの手順の節がある。書き込みの手順は fsync に触れる。
    for (i, path) in [EN, JA].into_iter().enumerate() {
        let doc = repo_file(path);
        for sec in [SEC_CHANGES, SEC_READ_ONLY, SEC_EXTERNAL, SEC_WRITE] {
            let t = if i == 0 { sec.0 } else { sec.1 };
            assert!(
                section(&doc, t).is_some(),
                "[WB-19] {path}: `## {t}` の節が無い"
            );
        }
        let t = if i == 0 { SEC_WRITE.0 } else { SEC_WRITE.1 };
        let w = section(&doc, t).unwrap_or_default().join("\n");
        assert!(
            w.contains("fsync"),
            "[WB-19] {path}: `## {t}` に fsync が無い"
        );
    }
}

#[test]
fn test_wb_19_docs_link_each_other() {
    // [WB-19] 英は日本語へ、日は英語へリンクする。
    let en = repo_file(EN);
    let ja = repo_file(JA);
    assert!(
        en.contains("](safety.ja.md)") || en.contains("](./safety.ja.md)"),
        "[WB-19] {EN} から {JA} へのリンクが無い"
    );
    assert!(
        ja.contains("](safety.md)") || ja.contains("](./safety.md)"),
        "[WB-19] {JA} から {EN} へのリンクが無い"
    );
}

#[test]
fn test_wb_19_marker_decoding() {
    // 試験の側の道具の確かめ: 印の置き換え。
    let lines: Vec<String> = ["<BOM>---", "a: <FF><FE>x<CRLF>", "b: <zz> <C>", "---"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut want = vec![0xEF, 0xBB, 0xBF];
    want.extend_from_slice(b"---\na: ");
    want.extend_from_slice(&[0xFF, 0xFE]);
    want.extend_from_slice(b"x\r\nb: <zz> <C>\n---\n");
    assert_eq!(example_bytes(&lines), want);
}
