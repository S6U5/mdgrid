//! タスク 1(print-pick)の受け入れ: `--print [--format csv|json|md] [--view <名前>]`(CLI-5・SR-10)。
//! 仕様: specs/cli/spec.md の CLI-5、specs/_changes/2026-10-03-print-pick.md の「不明点と仮定」。
//! 実装を見ずに書いた。本物の実行ファイルを動かし、標準出力はパイプ(`Command::output`)。
//!
//! 材料の保管庫(一時フォルダ)のノート:
//!
//! | note | title          | priority | done  | tags   | due        | memo          | status |
//! |------|----------------|----------|-------|--------|------------|---------------|--------|
//! | a.md | 会議           | 2        | false | [a, b] | 2026-11-01 | `x, y`        | todo   |
//! | b.md | He said "hi"   | 1        | false | [c]    | 2026-10-05 | `line1⏎line2` | doing  |
//! | c.md | 縦\|棒         | 3        | false | []     | 2026-12-01 | null          | todo   |
//! | d.md | 済んだ         | 0        | true  | [z]    | 2026-09-01 | `done`        | done   |
//! | e.md | キー無し       | 5        | false | (無し) | (無し)     | (無し)        | doing  |
//!
//! 仮定(仕様に書かれていない細部): 列の名前は displayName の無い列の id(ノートのキーは素の名前)。
//! csv・md の真偽は `true` / `false`。キーの無いセルは json で null。

use std::collections::BTreeMap;
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
            "mdgrid-print-{}-{}-{}",
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

const BASE: &str = r#"formulas:
  謎: 'nosuchfunc(priority)'
views:
  - type: table
    name: 一覧
    filters:
      and:
        - done == false
    order:
      - title
      - priority
      - done
      - tags
      - due
      - memo
    sort:
      - property: priority
        direction: DESC
  - type: table
    name: 期限順
    filters:
      and:
        - priority < 3
    order:
      - due
      - title
    sort:
      - property: due
        direction: ASC
  - type: table
    name: 状態ごと
    groupBy:
      property: status
      direction: ASC
    order:
      - title
      - status
    sort:
      - property: priority
        direction: ASC
  - type: table
    name: 未対応
    order:
      - title
      - formula.謎
    sort:
      - property: priority
        direction: DESC
"#;

/// 保管庫のフォルダ(notes/)を作り、ノートと tasks.base を書く。(一時フォルダ, 保管庫, .base のパス)
fn make_vault(name: &str) -> (TempDir, PathBuf, PathBuf) {
    let t = TempDir::new(name);
    let notes = t.mkdir("notes");
    let w = |rel: &str, text: &str| std::fs::write(notes.join(rel), text).unwrap();
    w(
        "a.md",
        "---\ntitle: 会議\npriority: 2\ndone: false\ntags: [a, b]\ndue: 2026-11-01\nmemo: \"x, y\"\nstatus: todo\n---\n本文 a\n",
    );
    w(
        "b.md",
        "---\ntitle: 'He said \"hi\"'\npriority: 1\ndone: false\ntags: [c]\ndue: 2026-10-05\nmemo: \"line1\\nline2\"\nstatus: doing\n---\n本文 b\n",
    );
    w(
        "c.md",
        "---\ntitle: 縦|棒\npriority: 3\ndone: false\ntags: []\ndue: 2026-12-01\nmemo:\nstatus: todo\n---\n本文 c\n",
    );
    w(
        "d.md",
        "---\ntitle: 済んだ\npriority: 0\ndone: true\ntags: [z]\ndue: 2026-09-01\nmemo: done\nstatus: done\n---\n本文 d\n",
    );
    w(
        "e.md",
        "---\ntitle: キー無し\npriority: 5\ndone: false\nstatus: doing\n---\n本文 e\n",
    );
    let base = notes.join("tasks.base");
    std::fs::write(&base, BASE).unwrap();
    (t, notes, base)
}

/// 子の環境の設定と状態の置き場を一時フォルダの下に向けて動かす。標準出力はパイプ。
fn run_in(t: &TempDir, args: &[&str]) -> Output {
    let conf = t.path().join("xdg-config");
    let state = t.path().join("xdg-state");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
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

/// 理由1行・終了コード 2・標準出力は空。理由の行は `mentions` を含む(何が悪いかを名指す)。
fn assert_fails_with_one_line(out: &Output, what: &str, mentions: &str) {
    assert_eq!(
        out.status.code(),
        Some(2),
        "{what}: 終了コード 2。stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let lines = stderr_lines(out);
    assert_eq!(lines.len(), 1, "{what}: 標準エラーは理由1行: {:?}", lines);
    assert!(
        lines[0].contains(mentions),
        "{what}: 理由は {mentions:?} を名指す: {:?}",
        lines[0]
    );
    assert!(
        out.stdout.is_empty(),
        "{what}: 標準出力は空: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
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

// ---- 保管庫の写し(書き換えの検査) ----

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    let mut m = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            let md = std::fs::symlink_metadata(&p).unwrap();
            if md.is_dir() {
                m.insert(p.clone(), (Vec::new(), md.modified().unwrap()));
                stack.push(p);
            } else {
                m.insert(
                    p.clone(),
                    (std::fs::read(&p).unwrap(), md.modified().unwrap()),
                );
            }
        }
    }
    m
}

fn s(x: &str) -> String {
    x.to_string()
}

// ---- 1. 既定の形(csv) ----

#[test]
fn test_cli_5_default_csv_header_rows_and_rfc4180() {
    // [CLI-5] [SR-10] 形を指定しない → csv。見出しの行は order の並び、行は絞り込み(done == false)と
    // 並べ替え(priority DESC)のあと。カンマ・引用符・改行を含む値は RFC 4180 で括る。改行は LF。
    let (t, _notes, base) = make_vault("csv");
    let out = run_in(&t, &[base.to_str().unwrap(), "--print"]);
    let text = ok_stdout(&out, "--print");
    let rows = parse_csv(&text);
    assert_eq!(
        rows,
        vec![
            vec![
                s("title"),
                s("priority"),
                s("done"),
                s("tags"),
                s("due"),
                s("memo")
            ],
            vec![s("キー無し"), s("5"), s("false"), s(""), s(""), s("")],
            vec![
                s("縦|棒"),
                s("3"),
                s("false"),
                s(""),
                s("2026-12-01"),
                s("")
            ],
            vec![
                s("会議"),
                s("2"),
                s("false"),
                s("a, b"),
                s("2026-11-01"),
                s("x, y")
            ],
            vec![
                s("He said \"hi\""),
                s("1"),
                s("false"),
                s("c"),
                s("2026-10-05"),
                s("line1\nline2")
            ],
        ],
        "csv の全体: {text}"
    );
    // 括りの形を生の文字でも確かめる(引用符は2つ重ね、改行は括りの中の LF)。
    assert!(text.contains("\"He said \"\"hi\"\"\""), "{text}");
    assert!(text.contains("\"x, y\""), "{text}");
    assert!(text.contains("\"line1\nline2\""), "{text}");
    assert!(text.contains("\"a, b\""), "{text}");
    // --format csv を明示しても同じ。
    let out2 = run_in(&t, &[base.to_str().unwrap(), "--print", "--format", "csv"]);
    assert_eq!(ok_stdout(&out2, "--format csv"), text);
}

#[test]
fn test_cli_5_group_by_only_orders_rows_without_heading_rows() {
    // [CLI-5] グループ分けは並びだけに効き、見出しの行は出さない。
    // status ASC のまとまり(doing・done・todo)、まとまりの中は priority ASC。
    let (t, _notes, base) = make_vault("group");
    let out = run_in(
        &t,
        &[base.to_str().unwrap(), "--print", "--view", "状態ごと"],
    );
    let rows = parse_csv(&ok_stdout(&out, "--view 状態ごと"));
    assert_eq!(
        rows,
        vec![
            vec![s("title"), s("status")],
            vec![s("He said \"hi\""), s("doing")],
            vec![s("キー無し"), s("doing")],
            vec![s("済んだ"), s("done")],
            vec![s("会議"), s("todo")],
            vec![s("縦|棒"), s("todo")],
        ]
    );
}

// ---- 2. json ----

#[test]
fn test_cli_5_json_array_of_objects_with_json_types() {
    // [CLI-5] json は列の名前を鍵にしたオブジェクトの配列。数・真偽・null・リストは JSON の型、日付は文字列。
    let (t, _notes, base) = make_vault("json");
    let out = run_in(&t, &[base.to_str().unwrap(), "--print", "--format", "json"]);
    let text = ok_stdout(&out, "--format json");
    let v = JsonParser::parse(&text);
    let Json::Arr(items) = v else {
        panic!("配列でない: {text}");
    };
    assert_eq!(items.len(), 4, "絞り込みのあとの4行: {text}");
    let order = vec![
        s("title"),
        s("priority"),
        s("done"),
        s("tags"),
        s("due"),
        s("memo"),
    ];
    for it in &items {
        assert_eq!(it.keys(), order, "鍵はビューの列の並び: {it:?}");
    }
    let str_ = |x: &str| Json::Str(x.to_string());
    // 並べ替えのあと(priority DESC): e, c, a, b
    let titles: Vec<Json> = items
        .iter()
        .map(|it| it.get("title").unwrap().clone())
        .collect();
    assert_eq!(
        titles,
        vec![
            str_("キー無し"),
            str_("縦|棒"),
            str_("会議"),
            str_("He said \"hi\"")
        ]
    );
    // e: キーの無いセルは null
    let e = &items[0];
    assert_eq!(e.get("priority"), Some(&Json::Num(5.0)));
    assert_eq!(e.get("done"), Some(&Json::Bool(false)));
    assert_eq!(e.get("tags"), Some(&Json::Null));
    assert_eq!(e.get("due"), Some(&Json::Null));
    assert_eq!(e.get("memo"), Some(&Json::Null));
    // c: 空のリストは空の配列、null は null
    let c = &items[1];
    assert_eq!(c.get("priority"), Some(&Json::Num(3.0)));
    assert_eq!(c.get("tags"), Some(&Json::Arr(vec![])));
    assert_eq!(c.get("due"), Some(&str_("2026-12-01")));
    assert_eq!(c.get("memo"), Some(&Json::Null));
    // a: リストは配列、日付は "YYYY-MM-DD"
    let a = &items[2];
    assert_eq!(a.get("priority"), Some(&Json::Num(2.0)));
    assert_eq!(a.get("done"), Some(&Json::Bool(false)));
    assert_eq!(a.get("tags"), Some(&Json::Arr(vec![str_("a"), str_("b")])));
    assert_eq!(a.get("due"), Some(&str_("2026-11-01")));
    assert_eq!(a.get("memo"), Some(&str_("x, y")));
    // b: 引用符と改行は JSON の逃がしで保たれる
    let b = &items[3];
    assert_eq!(b.get("priority"), Some(&Json::Num(1.0)));
    assert_eq!(b.get("tags"), Some(&Json::Arr(vec![str_("c")])));
    assert_eq!(b.get("memo"), Some(&str_("line1\nline2")));
    // 生の文字でも型を確かめる(数・真偽を文字列にしていない)。
    assert!(!text.contains("\"5\""), "数は文字列にしない: {text}");
    assert!(!text.contains("\"false\""), "真偽は文字列にしない: {text}");
    assert!(text.contains("\"2026-11-01\""), "{text}");
}

// ---- 3. md ----

#[test]
fn test_cli_5_md_table_with_separator_escaped_pipe_and_spaces_for_newlines() {
    // [CLI-5] md は見出しの行と区切りの行のある表。セルの中の縦棒は `\|`、改行は空白。
    let (t, _notes, base) = make_vault("md");
    let out = run_in(&t, &[base.to_str().unwrap(), "--print", "--format", "md"]);
    let text = ok_stdout(&out, "--format md");
    assert!(!text.contains('\r'), "改行は LF: {text:?}");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2 + 4, "見出し・区切り・4行: {text}");
    assert_eq!(
        md_cells(lines[0]),
        vec!["title", "priority", "done", "tags", "due", "memo"],
        "{text}"
    );
    let sep = md_cells(lines[1]);
    assert_eq!(sep.len(), 6, "{text}");
    for c in &sep {
        let core = c.trim_start_matches(':').trim_end_matches(':');
        assert!(
            !core.is_empty() && core.chars().all(|ch| ch == '-'),
            "区切りのセルは `---` の形: {c:?} in {text}"
        );
    }
    let body: Vec<Vec<String>> = lines[2..].iter().map(|l| md_cells(l)).collect();
    for r in &body {
        assert_eq!(r.len(), 6, "列の数が揃う: {r:?} in {text}");
    }
    assert_eq!(body[0][0], "キー無し");
    assert_eq!(body[0][1], "5");
    assert_eq!(body[0][3], "", "キーが無ければ空");
    assert_eq!(
        body[1][0], "縦\\|棒",
        "縦棒はバックスラッシュで逃がす: {text}"
    );
    assert_eq!(body[1][5], "", "null は空");
    assert_eq!(body[2][0], "会議");
    assert_eq!(body[2][3], "a, b", "リストは `, ` でつなぐ");
    assert_eq!(body[2][4], "2026-11-01");
    assert_eq!(body[3][0], "He said \"hi\"");
    assert_eq!(body[3][5], "line1 line2", "改行は空白: {text}");
}

// ---- 4. 評価できない列 ----

#[test]
fn test_cli_5_unsupported_formula_column_is_empty_or_null_with_one_warning() {
    // [CLI-5] [BV-7] 評価できない列は csv で空、json で null、md で空。標準エラーに1行。終了コード 0。
    let (t, _notes, base) = make_vault("unsupported");
    let b = base.to_str().unwrap();

    let out = run_in(&t, &[b, "--print", "--view", "未対応"]);
    let rows = parse_csv(&ok_stdout(&out, "csv 未対応"));
    assert_eq!(rows.len(), 1 + 5, "見出し + 全5行: {rows:?}");
    assert_eq!(rows[0], vec![s("title"), s("謎")]);
    assert_eq!(rows[1][0], "キー無し", "priority DESC の先頭");
    for r in &rows[1..] {
        assert_eq!(r.len(), 2, "{r:?}");
        assert_eq!(r[1], "", "評価できない列は空(`?` の印も出さない): {r:?}");
    }
    let warn = stderr_lines(&out);
    assert_eq!(warn.len(), 1, "標準エラーに1行: {warn:?}");

    let out = run_in(&t, &[b, "--print", "--view", "未対応", "--format", "json"]);
    let text = ok_stdout(&out, "json 未対応");
    let Json::Arr(items) = JsonParser::parse(&text) else {
        panic!("配列でない: {text}");
    };
    assert_eq!(items.len(), 5, "{text}");
    for it in &items {
        assert_eq!(it.get("謎"), Some(&Json::Null), "{it:?}");
    }
    assert_eq!(stderr_lines(&out).len(), 1, "json でも標準エラーに1行");

    let out = run_in(&t, &[b, "--print", "--view", "未対応", "--format", "md"]);
    let text = ok_stdout(&out, "md 未対応");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2 + 5, "{text}");
    for l in &lines[2..] {
        let c = md_cells(l);
        assert_eq!(c.len(), 2, "{l}");
        assert_eq!(c[1], "", "md でも空: {l}");
    }
    assert_eq!(stderr_lines(&out).len(), 1, "md でも標準エラーに1行");
}

// ---- 5. 知らない形 ----

#[test]
fn test_cli_5_unknown_format_fails_with_one_line_and_code_2() {
    // [CLI-5] [CLI-4] `--format xml` → 理由1行・終了コード 2・標準出力は空。
    let (t, _notes, base) = make_vault("xml");
    let out = run_in(&t, &[base.to_str().unwrap(), "--print", "--format", "xml"]);
    assert_fails_with_one_line(&out, "--format xml", "xml");
}

#[test]
fn test_cli_5_missing_path_fails_with_one_line_and_code_2() {
    // [CLI-5] [CLI-4] 起動できない(無いパス)→ 理由1行・終了コード 2。
    let t = TempDir::new("nopath");
    let missing = t.path().join("無いフォルダ");
    let out = run_in(&t, &[missing.to_str().unwrap(), "--print"]);
    assert_fails_with_one_line(&out, "無いパス", "無いフォルダ");
}

// ---- 6. --view ----

#[test]
fn test_cli_5_view_option_selects_another_base_view() {
    // [CLI-5] `--view 期限順` → そのビューの絞り込み(priority < 3)・並べ替え(due ASC)・列(due, title)。
    let (t, _notes, base) = make_vault("view");
    let out = run_in(&t, &[base.to_str().unwrap(), "--print", "--view", "期限順"]);
    let rows = parse_csv(&ok_stdout(&out, "--view 期限順"));
    assert_eq!(
        rows,
        vec![
            vec![s("due"), s("title")],
            vec![s("2026-09-01"), s("済んだ")],
            vec![s("2026-10-05"), s("He said \"hi\"")],
            vec![s("2026-11-01"), s("会議")],
        ]
    );
}

#[test]
fn test_cli_5_unknown_view_fails_with_one_line_and_code_2() {
    // [CLI-5] 無いビューの名前 → 理由1行・終了コード 2・標準出力は空。
    let (t, _notes, base) = make_vault("noview");
    let out = run_in(
        &t,
        &[base.to_str().unwrap(), "--print", "--view", "無いビュー"],
    );
    assert_fails_with_one_line(&out, "--view 無いビュー", "無いビュー");
}

#[test]
fn test_cli_5_view_option_falls_back_to_mdgrid_view_in_views_toml() {
    // [CLI-5] [BV-17] `.base` を使わずフォルダを開き、`--view` に views.toml の mdgrid のビューの名前。
    // その order と filters_expr で出す(並べ替えの指定は無いので、行は集合で比べる)。
    let (t, notes, base) = make_vault("native");
    std::fs::remove_file(&base).unwrap();
    let conf = t.mkdir("xdg-config/mdgrid");
    let real = std::fs::canonicalize(&notes).unwrap();
    let p = real
        .to_str()
        .unwrap()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let toml = format!(
        r#"[[target]]
path = "{p}"

  [[target.view]]
  name = "高い"
  order = ["title", "priority"]
  hidden = []
  filters_expr = ['priority >= 3']
"#
    );
    std::fs::write(conf.join("views.toml"), toml).unwrap();
    let out = run_in(&t, &[notes.to_str().unwrap(), "--print", "--view", "高い"]);
    let mut rows = parse_csv(&ok_stdout(&out, "--view 高い(mdgrid のビュー)"));
    assert_eq!(rows[0], vec![s("title"), s("priority")]);
    let mut body = rows.split_off(1);
    body.sort();
    assert_eq!(
        body,
        vec![vec![s("キー無し"), s("5")], vec![s("縦|棒"), s("3")]]
    );
}

// ---- 7. ノートを書き換えない ----

#[test]
fn test_cli_5_print_does_not_modify_or_create_files_in_vault() {
    // [CLI-5] [WB-15] 3つの形で出したあとも、保管庫のファイルのバイトと更新時刻は同じで、新しいファイルが無い。
    let (t, notes, base) = make_vault("readonly");
    // 更新時刻の比べを確かにするため、書いてから少し間を空ける。
    std::thread::sleep(std::time::Duration::from_millis(50));
    let before = snapshot(&notes);
    let b = base.to_str().unwrap();
    for args in [
        vec![b, "--print"],
        vec![b, "--print", "--format", "json"],
        vec![b, "--print", "--format", "md"],
        vec![b, "--print", "--view", "未対応"],
    ] {
        let out = run_in(&t, &args);
        ok_stdout(&out, &format!("{args:?}"));
    }
    let after = snapshot(&notes);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "保管庫に新しいファイルができない・消えない"
    );
    for (p, (bytes, mtime)) in &before {
        let (bytes2, mtime2) = &after[p];
        assert_eq!(bytes, bytes2, "中身が同じ: {}", p.display());
        assert_eq!(mtime, mtime2, "更新時刻が同じ: {}", p.display());
    }
}
