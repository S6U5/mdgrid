//! 画面の文言の言語(SR-23)の受け入れ試験(タスク4: 残りのリテラルの検査)。
//! 記録: specs/_changes/2026-10-03-language.md。
//!
//! src の下の .rs のうち、試験のファイル(名前が `test_` で始まるもの)と文言の表(src/i18n.rs)を除いたものを読み、
//! コメント(`//`・`///`・`//!`・`/* */`)を除いた文字列のリテラル(`"…"`・`r"…"`・`r#"…"#`。`b` の付くものも)に
//! 日本語の文字が無いことを確かめる。訳さないもの(記録の「不明点と仮定」)は下の許す表で許す。
//! リテラルの切り出しはここに小さく書く(完全な Rust の字句の解析ではない)。エスケープした `\"` と、
//! `'"'` の文字、ライフタイム(`'a`)は誤らない。

use std::path::{Path, PathBuf};

/// 許す表: (ファイルの src からの相対のパス, リテラルの始まる行の中身の部分文字列, 理由)。
/// 行の中身に部分文字列を含むリテラルを許す(その行の中のリテラルは全部)。
const ALLOWED: &[(&str, &str, &str)] = &[
    // 曜日の名前は日付の形の ddd の読み書きに使う値(ノートの値の形)なので訳さない(記録の仮定)。
    (
        "types.rs",
        "const WEEKDAY_NAMES",
        "日付の形 ddd の値(ノートの値の形)",
    ),
    // schema.rs の ja・ty_ja・書ける範囲の文は docs/config.ja.md と突き合わせる正本なので表に移さない(記録の仮定)。
    (
        "schema.rs",
        "ja: \"",
        "設定の項目の日本語の説明(文書と突き合わせる正本)",
    ),
    (
        "schema.rs",
        "ty_ja: \"",
        "設定の項目の型の日本語の名前(文書と突き合わせる正本)",
    ),
    (
        "schema.rs",
        "Scope::Global => \"",
        "書ける範囲の日本語の名前(文書と突き合わせる正本)",
    ),
    (
        "schema.rs",
        "Scope::Profile => \"",
        "書ける範囲の日本語の名前(文書と突き合わせる正本)",
    ),
    // man と補完の説明(clap の about・long_about)は日本語のまま(記録の仮定。英語にそろえるかは後で)。
    ("main.rs", "about = \"", "clap の about(man と補完の説明)"),
    (
        "main.rs",
        "long_about = \"",
        "clap の long_about(man と補完の説明)",
    ),
];

/// 日本語の文字(ひらがな・カタカナ・漢字・「・」「「」「」」「、」「。」などの和文の記号)。
fn is_ja(c: char) -> bool {
    matches!(c,
        '\u{3001}'..='\u{303F}' // 、。「」 など
        | '\u{3040}'..='\u{309F}' // ひらがな
        | '\u{30A0}'..='\u{30FF}' // カタカナ・「・」
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}' // 漢字
        | '\u{FF00}'..='\u{FFEF}' // 全角の英数と記号(「(」「)」「:」など)
    )
}

/// 文字列のリテラル(始まりの行は 1 から、中身はエスケープを解かない)。
#[derive(Debug, PartialEq)]
struct Lit {
    line: usize,
    text: String,
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Rust のソースから、コメントを除いた文字列のリテラルを切り出す。
fn literals(src: &str) -> Vec<Lit> {
    let cs: Vec<char> = src.chars().collect();
    let n = cs.len();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let at = |k: usize| -> char {
        if k < n {
            cs[k]
        } else {
            '\0'
        }
    };
    while i < n {
        let c = cs[i];
        // 行のコメント(`//`・`///`・`//!`)。
        if c == '/' && at(i + 1) == '/' {
            while i < n && cs[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // 塊のコメント(入れ子を許す)。
        if c == '/' && at(i + 1) == '*' {
            let mut depth = 1;
            i += 2;
            while i < n && depth > 0 {
                if cs[i] == '\n' {
                    line += 1;
                    i += 1;
                } else if cs[i] == '/' && at(i + 1) == '*' {
                    depth += 1;
                    i += 2;
                } else if cs[i] == '*' && at(i + 1) == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // 生の文字列: r"…"・r#"…"#・br"…"(前が識別子の文字でない r)。
        let prev_ident = i > 0 && is_ident(cs[i - 1]);
        let raw_start = if c == 'r' && !prev_ident {
            Some(i + 1)
        } else if c == 'b' && at(i + 1) == 'r' && !prev_ident {
            Some(i + 2)
        } else {
            None
        };
        if let Some(mut k) = raw_start {
            let mut hashes = 0;
            while at(k) == '#' {
                hashes += 1;
                k += 1;
            }
            if at(k) == '"' {
                let start_line = line;
                k += 1;
                let mut text = String::new();
                loop {
                    if k >= n {
                        break;
                    }
                    if cs[k] == '"' && (1..=hashes).all(|h| at(k + h) == '#') {
                        k += 1 + hashes;
                        break;
                    }
                    if cs[k] == '\n' {
                        line += 1;
                    }
                    text.push(cs[k]);
                    k += 1;
                }
                out.push(Lit {
                    line: start_line,
                    text,
                });
                i = k;
                continue;
            }
            // r で始まる識別子(raw でない)。識別子を読み飛ばす。
        }
        // 識別子は丸ごと飛ばす(途中の r や b を生の文字列の始まりと見ない)。
        if is_ident(c) {
            while i < n && is_ident(cs[i]) {
                i += 1;
            }
            // b"…"・b'…' は次の巡りで " や ' として読む。
            continue;
        }
        // 文字のリテラルとライフタイム。
        if c == '\'' {
            if at(i + 1) == '\\' {
                // '\n'・'\''・'\u{..}' など: 閉じの ' まで。
                let mut k = i + 2;
                k += 1; // エスケープされた1文字
                while k < n && cs[k] != '\'' && cs[k] != '\n' {
                    k += 1;
                }
                i = k + 1;
                continue;
            }
            if at(i + 2) == '\'' {
                // 'x'・'"'
                if at(i + 1) == '\n' {
                    line += 1;
                }
                i += 3;
                continue;
            }
            // ライフタイム・ラベル。
            i += 1;
            continue;
        }
        // ふつうの文字列。
        if c == '"' {
            let start_line = line;
            let mut k = i + 1;
            let mut text = String::new();
            while k < n && cs[k] != '"' {
                if cs[k] == '\\' && k + 1 < n {
                    if cs[k + 1] == '\n' {
                        line += 1;
                    }
                    text.push(cs[k]);
                    text.push(cs[k + 1]);
                    k += 2;
                    continue;
                }
                if cs[k] == '\n' {
                    line += 1;
                }
                text.push(cs[k]);
                k += 1;
            }
            out.push(Lit {
                line: start_line,
                text,
            });
            i = k + 1;
            continue;
        }
        if c == '\n' {
            line += 1;
        }
        i += 1;
    }
    out
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut es: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn test_sr_23_literal_scanner() {
    // 切り出しの道具の確かめ: コメントを除き、\" と '"' を誤らない。
    let src = concat!(
        "// \"コメント\"\n",
        "/// \"説明\"\n",
        "/* \"塊\" /* 入れ子 */ \"まだ塊\" */\n",
        "let a = \"one \\\" two\"; let q = '\"'; let e = '\\''; fn f<'a>(x: &'a str) {}\n",
        "let r = r#\"raw \"in\" raw\"#; let s = r\"x\"; let b = b\"y\"; let rr = br\"z\";\n",
        "let m = \"multi\nline\"; let after = \"z\";\n",
        "let id = for_r\"w\";\n",
    );
    let got: Vec<(usize, String)> = literals(src)
        .into_iter()
        .map(|l| (l.line, l.text))
        .collect();
    assert_eq!(
        got,
        vec![
            (4, "one \\\" two".to_string()),
            (5, "raw \"in\" raw".to_string()),
            (5, "x".to_string()),
            (5, "y".to_string()),
            (5, "z".to_string()),
            (6, "multi\nline".to_string()),
            (7, "z".to_string()),
            (8, "w".to_string()),
        ]
    );
}

#[test]
fn test_sr_23_no_japanese_literals() {
    // [SR-23] lib・ui・main の文字列のリテラルに日本語が残っていない(文言は src/i18n.rs の表に置く)。
    // 訳さないものは ALLOWED で許す。
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files(&root, &mut files);
    assert!(files.len() > 10, "src の .rs を読めている: {}", files.len());
    let mut found = Vec::new();
    for p in &files {
        let rel = p
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let name = p.file_name().unwrap().to_string_lossy();
        if name.starts_with("test_") || rel == "i18n.rs" {
            continue;
        }
        let src = std::fs::read_to_string(p).unwrap();
        let lines: Vec<&str> = src.lines().collect();
        for lit in literals(&src) {
            if !lit.text.chars().any(is_ja) {
                continue;
            }
            let row = lines.get(lit.line - 1).copied().unwrap_or("");
            let allowed = ALLOWED
                .iter()
                .any(|(f, part, _)| rel == *f && row.contains(part));
            if !allowed {
                found.push(format!("src/{rel}:{}: \"{}\"", lit.line, lit.text));
            }
        }
    }
    assert!(
        found.is_empty(),
        "日本語の文字列のリテラルが {} 件残っている(src/i18n.rs の表に移すか、訳さないものなら許す表に足す):\n{}",
        found.len(),
        found.join("\n")
    );
}
