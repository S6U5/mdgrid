//! タスク 1(print-with-path)の受け入れ: `--print --with-path`(CLI-14。関係: CLI-5・CLI-4・OUT-3)。
//! 仕様: specs/cli/spec.md の CLI-14、specs/output/spec.md の OUT-3(`--pick path` のパスの形)、
//! specs/_changes/2026-10-06-print-with-path.md。
//! 実装を見ずに書いた。本物の実行ファイルを動かし、標準出力はパイプ(`Command::output`)。
//!
//! パスの形(OUT-3): 起動の引数のフォルダ(.base ならそのフォルダ)の文字に、ノートの相対のパスを
//! `/` でつないだもの。子の作業のフォルダを一時フォルダにし、引数を相対(`notes`・`notes/tasks.base`)
//! で渡すので、パスは `notes/a.md`・`notes/sub/d.md` の形になる。
//!
//! 材料の保管庫(一時フォルダの notes/、.base 無し):
//!
//! | note       | 中身                                   |
//! |------------|----------------------------------------|
//! | a.md       | フロントマターあり(title・status)     |
//! | b.md       | フロントマター無し(本文だけ)          |
//! | c.md       | 空のファイル                           |
//! | sub/d.md   | フロントマターあり(title・status)     |
//!
//! 仮定(仕様に書かれていない細部): 行の並びはここでは問わない(パスの集まりで比べる)。

use std::collections::BTreeSet;
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
            "mdgrid-withpath-{}-{}-{}",
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

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 保管庫のフォルダ(notes/)を作り、4件のノートを書く(.base は置かない)。
fn make_folder_vault(name: &str) -> TempDir {
    let t = TempDir::new(name);
    let notes = t.mkdir("notes");
    t.mkdir("notes/sub");
    let w = |rel: &str, text: &str| std::fs::write(notes.join(rel), text).unwrap();
    w("a.md", "---\ntitle: 会議\nstatus: todo\n---\n本文 a\n");
    w("b.md", "フロントマターの無いノート\n");
    w("c.md", "");
    w("sub/d.md", "---\ntitle: 下の\nstatus: doing\n---\n本文 d\n");
    t
}

/// 4件のノートのパス(引数 `notes` + 相対)。
fn folder_paths() -> BTreeSet<String> {
    ["notes/a.md", "notes/b.md", "notes/c.md", "notes/sub/d.md"]
        .iter()
        .map(|p| p.to_string())
        .collect()
}

const BASE: &str = r#"filters:
  and:
    - file.inFolder("Tasks")
views:
  - type: table
    name: 一覧
    order:
      - title
      - status
"#;

/// .base のある保管庫: notes/tasks.base(file.inFolder("Tasks"))、notes/Tasks/x.md・y.md、
/// 絞り込みで外れる notes/other.md。
fn make_base_vault(name: &str) -> TempDir {
    let t = TempDir::new(name);
    let notes = t.mkdir("notes");
    t.mkdir("notes/Tasks");
    let w = |rel: &str, text: &str| std::fs::write(notes.join(rel), text).unwrap();
    w("Tasks/x.md", "---\ntitle: 一つ目\nstatus: todo\n---\n");
    w("Tasks/y.md", "---\ntitle: 二つ目\nstatus: done\n---\n");
    w("other.md", "---\ntitle: 外\nstatus: todo\n---\n");
    std::fs::write(notes.join("tasks.base"), BASE).unwrap();
    t
}

/// 子の作業のフォルダを一時フォルダにし、設定と状態の置き場をその下に向けて動かす。標準出力はパイプ。
fn run_in(t: &TempDir, args: &[&str]) -> Output {
    let conf = t.path().join("xdg-config");
    let state = t.path().join("xdg-state");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(t.path())
        .env("XDG_CONFIG_HOME", &conf)
        .env("XDG_STATE_HOME", &state)
        .env_remove("MDGRID_CONFIG")
        .output()
        .expect("mdgrid を起動できる")
}

fn ok_stdout(out: &Output, what: &str) -> String {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{what}: 終了コード 0。stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).expect("出力は UTF-8")
}

fn stderr_lines(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

// ---- CSV の読み取り(RFC 4180) ----

/// RFC 4180 の CSV を読む。行の区切りは LF だけを受ける(CR があれば落とす)。
fn parse_csv(s: &str) -> Vec<Vec<String>> {
    assert!(!s.contains('\r'), "改行は LF だけ(CR が無い): {s:?}");
    assert!(s.ends_with('\n'), "最後の行も LF で終わる: {s:?}");
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = s.chars().peekable();
    let mut at_field_start = true;
    while let Some(c) = chars.next() {
        if at_field_start && c == '"' {
            // 括った値
            loop {
                match chars.next() {
                    Some('"') => {
                        if chars.peek() == Some(&'"') {
                            chars.next();
                            field.push('"');
                        } else {
                            break;
                        }
                    }
                    Some(ch) => field.push(ch),
                    None => panic!("閉じていない引用符: {s:?}"),
                }
            }
            at_field_start = false;
            match chars.peek() {
                Some(',') | Some('\n') | None => {}
                other => panic!("引用符のあとに区切りが無い: {other:?} in {s:?}"),
            }
            continue;
        }
        at_field_start = false;
        match c {
            ',' => {
                row.push(std::mem::take(&mut field));
                at_field_start = true;
            }
            '\n' => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
                at_field_start = true;
            }
            '"' => panic!("括っていない値の中に引用符: {s:?}"),
            ch => field.push(ch),
        }
    }
    rows
}

// ---- JSON の読み取り(小さなもの) ----

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// 鍵の並びを保つ。
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn keys(&self) -> Vec<String> {
        match self {
            Json::Obj(kv) => kv.iter().map(|(k, _)| k.clone()).collect(),
            _ => panic!("オブジェクトでない: {self:?}"),
        }
    }
}

struct JsonParser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> JsonParser<'a> {
    fn parse(text: &'a str) -> Json {
        let mut p = JsonParser {
            s: text.as_bytes(),
            i: 0,
        };
        let v = p.value();
        p.ws();
        assert_eq!(p.i, p.s.len(), "JSON のあとに余りがある: {text:?}");
        v
    }
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn eat(&mut self, b: u8) {
        self.ws();
        assert_eq!(
            self.s.get(self.i),
            Some(&b),
            "JSON: {:?} を期待 (位置 {})",
            b as char,
            self.i
        );
        self.i += 1;
    }
    fn lit(&mut self, word: &str) {
        assert!(
            self.s[self.i..].starts_with(word.as_bytes()),
            "JSON: {word} を期待 (位置 {})",
            self.i
        );
        self.i += word.len();
    }
    fn value(&mut self) -> Json {
        self.ws();
        match self.s.get(self.i).copied() {
            Some(b'n') => {
                self.lit("null");
                Json::Null
            }
            Some(b't') => {
                self.lit("true");
                Json::Bool(true)
            }
            Some(b'f') => {
                self.lit("false");
                Json::Bool(false)
            }
            Some(b'"') => Json::Str(self.string()),
            Some(b'[') => {
                self.i += 1;
                let mut v = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Json::Arr(v);
                }
                loop {
                    v.push(self.value());
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Json::Arr(v);
                        }
                        other => panic!("JSON: 配列の中に {other:?}"),
                    }
                }
            }
            Some(b'{') => {
                self.i += 1;
                let mut kv = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Json::Obj(kv);
                }
                loop {
                    self.ws();
                    let k = self.string();
                    self.eat(b':');
                    let v = self.value();
                    kv.push((k, v));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Json::Obj(kv);
                        }
                        other => panic!("JSON: オブジェクトの中に {other:?}"),
                    }
                }
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                while self.i < self.s.len()
                    && matches!(
                        self.s[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                let t = std::str::from_utf8(&self.s[start..self.i]).unwrap();
                Json::Num(t.parse().unwrap_or_else(|_| panic!("JSON: 数でない {t:?}")))
            }
            other => panic!("JSON: 値でない {other:?} (位置 {})", self.i),
        }
    }
    fn string(&mut self) -> String {
        assert_eq!(self.s.get(self.i), Some(&b'"'), "JSON: 文字列を期待");
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.s.get(self.i).expect("JSON: 閉じていない文字列");
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self.s[self.i];
                    self.i += 1;
                    match e {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let h = std::str::from_utf8(&self.s[self.i..self.i + 4]).unwrap();
                            self.i += 4;
                            let mut cp = u32::from_str_radix(h, 16).unwrap();
                            if (0xD800..0xDC00).contains(&cp) {
                                assert_eq!(&self.s[self.i..self.i + 2], b"\\u");
                                let h2 =
                                    std::str::from_utf8(&self.s[self.i + 2..self.i + 6]).unwrap();
                                self.i += 6;
                                let lo = u32::from_str_radix(h2, 16).unwrap();
                                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                            }
                            let ch = char::from_u32(cp).expect("JSON: 正しい文字");
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => panic!("JSON: 知らない逃がし \\{}", other as char),
                    }
                }
                c if c < 0x20 => panic!("JSON: 文字列の中に逃がしていない制御文字 {c}"),
                c => out.push(c),
            }
        }
        String::from_utf8(out).expect("JSON: UTF-8")
    }
}

// ---- Markdown の表の読み取り ----

/// 表の1行を、逃がしていない縦棒で切る。セルは前後の空白を落とし、逃がしはそのまま残す。
fn md_cells(line: &str) -> Vec<String> {
    let t = line.trim();
    assert!(
        t.starts_with('|') && t.ends_with('|') && !t.ends_with("\\|"),
        "md の行は縦棒で始まり縦棒で終わる: {line:?}"
    );
    let inner = &t[1..t.len() - 1];
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut prev_backslash = false;
    for ch in inner.chars() {
        if ch == '|' && !prev_backslash {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
        prev_backslash = ch == '\\' && !prev_backslash;
    }
    cells.push(cur.trim().to_string());
    cells
}

fn s(x: &str) -> String {
    x.to_string()
}

// ---- 1. csv ----

#[test]
fn test_cli_14_csv_with_path_first_column_is_note_path() {
    // [CLI-14] [OUT-3] フォルダ `--print --with-path` → 見出しの行の先頭が `path`、各行の先頭のセルが
    // 「引数のフォルダ + / + ノートの相対」、行の数はノートの数(4)、終了コード 0。
    let t = make_folder_vault("csv");
    let out = run_in(&t, &["notes", "--print", "--with-path"]);
    let text = ok_stdout(&out, "--print --with-path");
    let rows = parse_csv(&text);
    assert!(!rows.is_empty(), "見出しの行がある: {text:?}");
    assert_eq!(
        rows[0].first(),
        Some(&s("path")),
        "見出しの先頭は path: {text}"
    );
    let body = &rows[1..];
    assert_eq!(body.len(), 4, "行の数はノートの数: {text}");
    for r in body {
        assert_eq!(r.len(), rows[0].len(), "どの行も見出しと同じ列の数: {r:?}");
    }
    let paths: BTreeSet<String> = body.iter().map(|r| r[0].clone()).collect();
    assert_eq!(paths, folder_paths(), "各行の先頭はノートのパス: {text}");
}

#[test]
fn test_cli_14_rows_without_frontmatter_are_told_apart_by_path() {
    // [CLI-14] フロントマターの無いノート(b.md)と空のノート(c.md)の行は、パスの列以外が空でも、
    // パスで見分けられる(パスは行ごとに違い、それぞれの行がある)。
    let t = make_folder_vault("nofm");
    let out = run_in(&t, &["notes", "--print", "--with-path"]);
    let text = ok_stdout(&out, "--print --with-path");
    let rows = parse_csv(&text);
    let body: Vec<&Vec<String>> = rows.iter().skip(1).collect();
    let firsts: Vec<&str> = body.iter().map(|r| r[0].as_str()).collect();
    let unique: BTreeSet<&str> = firsts.iter().copied().collect();
    assert_eq!(unique.len(), firsts.len(), "パスは行ごとに違う: {text}");
    for p in ["notes/b.md", "notes/c.md"] {
        assert!(firsts.contains(&p), "{p} の行がある: {text}");
    }
}

// ---- 2. --with-path なしは今のまま ----

#[test]
fn test_cli_14_without_flag_output_has_no_path_column() {
    // [CLI-14] [CLI-5] `--with-path` を付けない → path の列は無い(今のまま)。
    // 付けたときの出力から先頭の列を除くと、付けないときの出力と同じ(行の中身と並びは変わらない)。
    let t = make_folder_vault("plain");
    let plain = ok_stdout(&run_in(&t, &["notes", "--print"]), "--print");
    let rows = parse_csv(&plain);
    assert!(!rows.is_empty(), "見出しの行がある: {plain:?}");
    assert!(
        !rows[0].contains(&s("path")),
        "--with-path なしの見出しに path は無い: {plain}"
    );
    for r in &rows {
        for p in folder_paths() {
            assert!(!r.contains(&p), "パスのセルは無い: {r:?}");
        }
    }
    let with = ok_stdout(
        &run_in(&t, &["notes", "--print", "--with-path"]),
        "--print --with-path",
    );
    let stripped: Vec<Vec<String>> = parse_csv(&with)
        .into_iter()
        .map(|mut r| {
            assert!(!r.is_empty());
            r.remove(0);
            r
        })
        .collect();
    assert_eq!(stripped, rows, "先頭の列を除けば今の出力と同じ");
}

// ---- 3. json ----

#[test]
fn test_cli_14_json_with_path_first_key_is_path() {
    // [CLI-14] `--format json --with-path` → 各オブジェクトの最初の鍵が `path`、値はノートのパス(文字列)。
    let t = make_folder_vault("json");
    let out = run_in(&t, &["notes", "--print", "--format", "json", "--with-path"]);
    let text = ok_stdout(&out, "--format json --with-path");
    let Json::Arr(items) = JsonParser::parse(&text) else {
        panic!("配列でない: {text}");
    };
    assert_eq!(items.len(), 4, "オブジェクトの数はノートの数: {text}");
    let mut paths = BTreeSet::new();
    for it in &items {
        let keys = it.keys();
        assert_eq!(keys.first(), Some(&s("path")), "最初の鍵は path: {it:?}");
        assert_eq!(
            keys.iter().filter(|k| *k == "path").count(),
            1,
            "鍵 path は1つ: {it:?}"
        );
        match it.get("path") {
            Some(Json::Str(p)) => {
                paths.insert(p.clone());
            }
            other => panic!("path の値は文字列: {other:?}"),
        }
    }
    assert_eq!(paths, folder_paths(), "path の値はノートのパス: {text}");
}

// ---- 4. md ----

#[test]
fn test_cli_14_md_with_path_first_header_column_is_path() {
    // [CLI-14] `--format md --with-path` → 見出しの行の最初の列が path、区切りの行のあとの各行の先頭はパス。
    let t = make_folder_vault("md");
    let out = run_in(&t, &["notes", "--print", "--format", "md", "--with-path"]);
    let text = ok_stdout(&out, "--format md --with-path");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "見出しと区切りの行がある: {text}");
    let header = md_cells(lines[0]);
    assert_eq!(
        header.first(),
        Some(&s("path")),
        "見出しの最初の列は path: {text}"
    );
    let body = &lines[2..];
    assert_eq!(body.len(), 4, "行の数はノートの数: {text}");
    let paths: BTreeSet<String> = body.iter().map(|l| md_cells(l)[0].clone()).collect();
    assert_eq!(paths, folder_paths(), "各行の先頭はノートのパス: {text}");
}

// ---- 5. .base ----

#[test]
fn test_cli_14_base_with_path_is_relative_to_base_folder() {
    // [CLI-14] [OUT-3] `.base` を渡す → パスは「.base のあるフォルダの文字 + / + ノートの相対」。
    // file.inFolder("Tasks") で notes/other.md は外れる。
    let t = make_base_vault("base");
    let out = run_in(&t, &["notes/tasks.base", "--print", "--with-path"]);
    let text = ok_stdout(&out, ".base --print --with-path");
    let rows = parse_csv(&text);
    assert!(!rows.is_empty(), "見出しの行がある: {text:?}");
    assert_eq!(
        rows[0],
        vec![s("path"), s("title"), s("status")],
        "見出しは path とビューの列: {text}"
    );
    let paths: BTreeSet<String> = rows[1..].iter().map(|r| r[0].clone()).collect();
    let want: BTreeSet<String> = ["notes/Tasks/x.md", "notes/Tasks/y.md"]
        .iter()
        .map(|p| p.to_string())
        .collect();
    assert_eq!(paths, want, "パスは .base のフォルダからの形: {text}");
    assert_eq!(rows.len(), 3, "見出しと2行: {text}");
}

// ---- 6. --print なし ----

#[test]
fn test_cli_14_with_path_without_print_fails_with_one_line_and_code_2() {
    // [CLI-14] [CLI-4] `--print` の無い `--with-path` → 理由1行・終了コード 2・標準出力は空。
    // 画面を出さずに終わる(標準出力はパイプ。疑似端末でないので、画面を出そうとしても待たない)。
    let t = make_folder_vault("noprint");
    let out = run_in(&t, &["notes", "--with-path"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "終了コード 2。stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lines = stderr_lines(&out);
    assert_eq!(lines.len(), 1, "標準エラーは理由1行: {:?}", lines);
    assert!(
        lines[0].contains("--with-path"),
        "理由は --with-path を名指す: {:?}",
        lines[0]
    );
    assert!(
        out.stdout.is_empty(),
        "標準出力は空: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}
