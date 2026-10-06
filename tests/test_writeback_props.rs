//! 書き戻しの性質の試験(Q-8。WB-1・WB-5・WB-6・WB-7)。仕様: specs/write-back/spec.md。
//! 変更の記録: specs/_changes/2026-10-03-writeback-props.md。
//!
//! ランダムに作ったノートと値で、公開の口(`frontmatter::parse`・`writeback::apply`)だけを使って確かめる。
//! ノートは試験の側で組み立てるので、各キーの行のバイトの位置と閉じの区切りの位置は試験の側で分かっている。

use mdgrid::frontmatter::{parse, Frontmatter, ReadOnly, Value};
use mdgrid::writeback::{apply, Edit, EditError, NewValue};
use proptest::prelude::*;
use proptest::sample::Index;
use std::ops::Range;

/// 1つの性質のケースの数(既定の 256 以下。全体が数秒で終わるように)。
const CASES: u32 = 256;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: CASES,
        // 作業ツリーに proptest-regressions を書かない(落ちた入力は試験の出力に出る)。
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

// ---- ノートの組み立て ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Plain,
    Single,
    Double,
}

/// 第1階層のキーの値の書き方。
#[derive(Debug, Clone)]
enum ValueSpec {
    /// 1行のスカラー。`text` は書いたままの字面(引用符を含む。空なら `key:`)。`comment` は行末のコメント(空か `  # …`)。
    Scalar {
        text: String,
        quote: Quote,
        comment: String,
    },
    /// フローのリスト(字面)。
    Flow(String),
    /// ブロックのリストの要素。
    Block(Vec<String>),
    /// ネストした map の (キー, 値)。
    Nested(Vec<(String, String)>),
}

/// キーの前に置くもの。
#[derive(Debug, Clone, Copy)]
enum Sep {
    None,
    Blank,
    CommentLine,
}

#[derive(Debug, Clone)]
struct NoteSpec {
    entries: Vec<(ValueSpec, Sep)>,
    key_prefix: &'static str,
    crlf: bool,
    body: &'static str,
}

#[derive(Clone)]
struct Note {
    bytes: Vec<u8>,
    nl: &'static str,
    keys: Vec<String>,
    values: Vec<ValueSpec>,
    /// 各キーの行(複数行の値はその全部)のバイトの範囲。改行を含む。
    ranges: Vec<Range<usize>>,
    /// 閉じの区切りの行の先頭のバイト位置。
    end: usize,
}

/// 縮めた反例を読みやすくするため、バイト列は文字として出す。
impl std::fmt::Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Note")
            .field("text", &String::from_utf8_lossy(&self.bytes))
            .field("ranges", &self.ranges)
            .field("end", &self.end)
            .finish()
    }
}

impl Note {
    fn text(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap()
    }

    fn scalar_indices(&self) -> Vec<usize> {
        (0..self.values.len())
            .filter(|&i| matches!(self.values[i], ValueSpec::Scalar { .. }))
            .collect()
    }
}

fn build(spec: NoteSpec) -> Note {
    let nl = if spec.crlf { "\r\n" } else { "\n" };
    let mut s = String::new();
    s.push_str("---");
    s.push_str(nl);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut ranges = Vec::new();
    for (i, (value, sep)) in spec.entries.into_iter().enumerate() {
        match sep {
            Sep::None => {}
            Sep::Blank => s.push_str(nl),
            Sep::CommentLine => {
                s.push_str("# 区切りのコメント");
                s.push_str(nl);
            }
        }
        let key = format!("{}{}", spec.key_prefix, i);
        let start = s.len();
        match &value {
            ValueSpec::Scalar { text, comment, .. } => {
                if text.is_empty() {
                    s.push_str(&format!("{key}:{comment}"));
                } else {
                    s.push_str(&format!("{key}: {text}{comment}"));
                }
                s.push_str(nl);
            }
            ValueSpec::Flow(text) => {
                s.push_str(&format!("{key}: {text}"));
                s.push_str(nl);
            }
            ValueSpec::Block(items) => {
                s.push_str(&format!("{key}:"));
                s.push_str(nl);
                for item in items {
                    s.push_str(&format!("  - {item}"));
                    s.push_str(nl);
                }
            }
            ValueSpec::Nested(pairs) => {
                s.push_str(&format!("{key}:"));
                s.push_str(nl);
                for (k, v) in pairs {
                    s.push_str(&format!("  {k}: {v}"));
                    s.push_str(nl);
                }
            }
        }
        ranges.push(start..s.len());
        keys.push(key);
        values.push(value);
    }
    let end = s.len();
    s.push_str("---");
    s.push_str(nl);
    s.push_str(&spec.body.replace('\n', nl));
    Note {
        bytes: s.into_bytes(),
        nl,
        keys,
        values,
        ranges,
        end,
    }
}

fn plain_word() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z][a-z0-9_]{0,7}",
        "[a-z]{1,5}( [a-z]{1,5}){1,2}",
        "[ぁ-ゖ]{1,4}",
        Just("進行中".to_string()),
    ]
}

fn comment() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => Just(String::new()),
        1 => "[a-zコメント ]{0,8}".prop_map(|c| format!("  # {c}")),
    ]
}

fn scalar_text() -> impl Strategy<Value = (String, Quote)> {
    prop_oneof![
        plain_word().prop_map(|w| (w, Quote::Plain)),
        any::<i32>().prop_map(|n| (n.to_string(), Quote::Plain)),
        "-?[0-9]{1,3}\\.[0-9]{1,3}".prop_map(|f| (f, Quote::Plain)),
        prop::sample::select(vec!["true", "false", "~", "null", ""])
            .prop_map(|t| (t.to_string(), Quote::Plain)),
        "[a-zA-Z0-9 :#\"'\\\\あ-お-]{0,8}"
            .prop_map(|c| (format!("'{}'", c.replace('\'', "''")), Quote::Single)),
        "[a-zA-Z0-9 :#\"'\\\\あ-お-]{0,8}".prop_map(|c| {
            (
                format!("\"{}\"", c.replace('\\', "\\\\").replace('"', "\\\"")),
                Quote::Double,
            )
        }),
    ]
}

fn value_spec() -> impl Strategy<Value = ValueSpec> {
    prop_oneof![
        6 => (scalar_text(), comment()).prop_map(|((text, quote), comment)| ValueSpec::Scalar {
            text,
            quote,
            comment
        }),
        1 => prop::collection::vec(plain_word(), 0..4)
            .prop_map(|items| ValueSpec::Flow(format!("[{}]", items.join(", ")))),
        1 => prop::collection::vec(plain_word(), 1..4).prop_map(ValueSpec::Block),
        1 => prop::collection::vec(("[a-z]{1,4}", plain_word()), 1..3).prop_map(|pairs| {
            // ネストした map の中のキーも重ならないようにする。
            ValueSpec::Nested(
                pairs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (k, v))| (format!("{k}{i}"), v))
                    .collect(),
            )
        }),
    ]
}

fn sep() -> impl Strategy<Value = Sep> {
    prop_oneof![
        4 => Just(Sep::None),
        1 => Just(Sep::Blank),
        1 => Just(Sep::CommentLine),
    ]
}

fn note() -> impl Strategy<Value = Note> {
    (
        prop::collection::vec((value_spec(), sep()), 1..7),
        prop::sample::select(vec!["k", "key_", "状態", "my-key"]),
        any::<bool>(),
        prop::sample::select(vec![
            "",
            "本文\n",
            "# 見出し\n\nstatus: 本文の中の行\n",
            "改行の無い本文",
            "---\nafter: x\n",
        ]),
    )
        .prop_map(|(entries, key_prefix, crlf, body)| {
            build(NoteSpec {
                entries,
                key_prefix,
                crlf,
                body,
            })
        })
}

/// スカラーの値を1つ以上持つノートと、そのうち1つの位置。
fn note_with_scalar() -> impl Strategy<Value = (Note, usize)> {
    (note(), any::<Index>()).prop_filter_map("no scalar key", |(n, ix)| {
        let scalars = n.scalar_indices();
        if scalars.is_empty() {
            None
        } else {
            let i = scalars[ix.index(scalars.len())];
            Some((n, i))
        }
    })
}

// ---- 書く値 ----

/// 書く文字列の文字。改行(`\n`・`\r`)と、YAML で扱いの難しい制御文字や改行に見える文字をまれに混ぜる。
fn value_char() -> impl Strategy<Value = char> {
    prop_oneof![
        12 => proptest::char::range(' ', '~'),
        3 => proptest::char::range('ぁ', 'ゖ'),
        1 => prop::sample::select(vec!['日', '本', '😀', '\t', 'é']),
        1 => prop::sample::select(vec![
            '\n', '\r', '\u{0}', '\u{7}', '\u{1b}', '\u{7f}', '\u{85}', '\u{a0}', '\u{2028}',
            '\u{2029}', '\u{feff}',
        ]),
    ]
}

fn value_string() -> impl Strategy<Value = String> {
    prop::collection::vec(value_char(), 0..12).prop_map(|cs| cs.into_iter().collect())
}

/// WB-7 の、型が変わりうる・クオートの要る文字列。
const RISKY: &[&str] = &[
    "no",
    "yes",
    "on",
    "off",
    "No",
    "Yes",
    "ON",
    "OFF",
    "y",
    "n",
    "Y",
    "N",
    "null",
    "Null",
    "NULL",
    "~",
    "true",
    "True",
    "FALSE",
    "1e3",
    "1E3",
    "0x10",
    "0xFF",
    "0o17",
    "017",
    "0b101",
    ".5",
    "+.5",
    "-.5",
    "1.",
    "1.0",
    "1_000",
    "+1",
    "-1",
    "0",
    "1:20",
    "190:20:30",
    ".inf",
    "-.Inf",
    ".NaN",
    "a: b",
    ": ",
    ":",
    "a #b",
    " #",
    "#x",
    "-",
    "- x",
    "-x",
    "[",
    "[x]",
    "{",
    "{a: b}",
    "&a",
    "*a",
    "!t",
    "!!str",
    "|",
    ">",
    "%x",
    "@x",
    "`x",
    "?",
    "? x",
    ",x",
    "'x",
    "\"x",
    "<<",
    "=",
    " lead",
    "trail ",
    " ",
    "  both  ",
    "\tx",
    "x\t",
    "",
    "a\\b",
    "\\",
    "2026-10-03",
    "2026-1-3",
    "2026-10-03T12:00",
    "2026-10-03 12:00:00",
    "2001-12-14t21:59:43.10-05:00",
];

#[derive(Debug, Clone)]
enum Written {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl Written {
    fn new_value(&self) -> NewValue {
        match self {
            Written::Str(s) => NewValue::Str(s.clone()),
            Written::Int(n) => NewValue::Int(*n),
            Written::Float(f) => NewValue::Float(*f),
            Written::Bool(b) => NewValue::Bool(*b),
            Written::Null => NewValue::Null,
        }
    }

    fn expected(&self) -> Value {
        match self {
            Written::Str(s) => Value::Str(s.clone()),
            Written::Int(n) => Value::Int(*n),
            Written::Float(f) => Value::Float(*f),
            Written::Bool(b) => Value::Bool(*b),
            Written::Null => Value::Null,
        }
    }

    fn has_newline(&self) -> bool {
        matches!(self, Written::Str(s) if s.contains(['\n', '\r']))
    }
}

fn written() -> impl Strategy<Value = Written> {
    prop_oneof![
        8 => value_string().prop_map(Written::Str),
        2 => prop::sample::select(RISKY).prop_map(|s| Written::Str(s.to_string())),
        1 => any::<i64>().prop_map(Written::Int),
        1 => any::<f64>()
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(Written::Float),
        1 => prop::sample::select(vec![0.5, 1.0, -2.0, 1e3, 1e-7, 3.0e20])
            .prop_map(Written::Float),
        1 => any::<bool>().prop_map(Written::Bool),
        1 => Just(Written::Null),
    ]
}

// ---- 補助 ----

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn value_of(fm: &Frontmatter, key: &str) -> Option<Value> {
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .map(|e| e.value.clone())
}

fn span_of(fm: &Frontmatter, key: &str) -> Option<Range<usize>> {
    fm.entries
        .iter()
        .find(|e| e.key == key)
        .and_then(|e| e.span.clone())
}

/// 前後の共通の先頭と末尾を除いた、元のバイト列の中の変わった範囲。
fn changed_range(before: &[u8], after: &[u8]) -> Range<usize> {
    let max = before.len().min(after.len());
    let mut p = 0;
    while p < max && before[p] == after[p] {
        p += 1;
    }
    let mut q = 0;
    while q < max - p && before[before.len() - 1 - q] == after[after.len() - 1 - q] {
        q += 1;
    }
    p..before.len() - q
}

/// 対象のキーのほかの値が前と同じで、キーの並びも同じこと(WB-6)。
fn assert_others_unchanged(
    before: &Frontmatter,
    after: &Frontmatter,
    target: &str,
) -> Result<(), TestCaseError> {
    let keys_before: Vec<&str> = before.entries.iter().map(|e| e.key.as_str()).collect();
    let keys_after: Vec<&str> = after.entries.iter().map(|e| e.key.as_str()).collect();
    let mut expected_keys = keys_before.clone();
    if !expected_keys.contains(&target) {
        expected_keys.push(target);
    }
    prop_assert_eq!(keys_after, expected_keys, "[WB-1] key order");
    for e in &before.entries {
        if e.key != target {
            prop_assert_eq!(
                value_of(after, &e.key),
                Some(e.value.clone()),
                "[WB-6] {} changed",
                e.key
            );
        }
    }
    Ok(())
}

/// YAML 1.1 の暗黙の型(bool・null・int・float・timestamp・merge・value)に読まれうる素の字面か。
/// YAML 1.2 の core は parse で読み直して確かめる。ここは保守的に広めに当てる。
fn yaml11_typed(s: &str) -> bool {
    const WORDS: &[&str] = &[
        "y", "Y", "yes", "Yes", "YES", "n", "N", "no", "No", "NO", "true", "True", "TRUE", "false",
        "False", "FALSE", "on", "On", "ON", "off", "Off", "OFF", "~", "null", "Null", "NULL", "",
        "<<", "=",
    ];
    if WORDS.contains(&s) {
        return true;
    }
    let body = s.strip_prefix(['-', '+']).unwrap_or(s);
    if matches!(body, ".inf" | ".Inf" | ".INF") || matches!(s, ".nan" | ".NaN" | ".NAN") {
        return true;
    }
    let digits_or = |t: &str, extra: &str| {
        !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || extra.contains(c))
    };
    // int: 2進・16進・8進・10進・60進。float: `.` を含む数字の並び(指数つき)・60進。
    if let Some(b) = body.strip_prefix("0b") {
        if digits_or(b, "_") && b.chars().all(|c| matches!(c, '0' | '1' | '_')) {
            return true;
        }
    }
    if let Some(h) = body.strip_prefix("0x") {
        if !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit() || c == '_') {
            return true;
        }
    }
    if body.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
        let mantissa = match body.find(['e', 'E']) {
            Some(i) => {
                let exp = &body[i + 1..];
                let exp = exp.strip_prefix(['-', '+']).unwrap_or(exp);
                if !digits_or(exp, "") {
                    return false;
                }
                &body[..i]
            }
            None => body,
        };
        if digits_or(mantissa, "_.:") && mantissa.chars().any(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    // timestamp: YYYY-M-D で始まる。
    let b = s.as_bytes();
    if b.len() >= 8
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5].is_ascii_digit()
    {
        return true;
    }
    // そのほか、Rust が数として読めるもの(数字を含むものだけ。`inf` のような語は文字列)。
    let t = s.replace('_', "");
    s.chars().any(|c| c.is_ascii_digit()) && (t.parse::<f64>().is_ok() || t.parse::<i64>().is_ok())
}

// ---- WB-1・WB-6: 既にあるスカラーのキーを書く ----

proptest! {
    #![proptest_config(config())]

    #[test]
    fn test_wb_1_props_existing_scalar_key((note, i) in note_with_scalar(), w in written()) {
        // [WB-1] [WB-6]
        let before = &note.bytes;
        let key = &note.keys[i];
        let fm_before = parse(before);
        prop_assert!(fm_before.is_ok(), "generated note must parse: {:?}\n{}", fm_before, note.text());
        let fm_before = fm_before.unwrap();
        prop_assert_eq!(fm_before.end, note.end, "end of frontmatter");
        let span = span_of(&fm_before, key);
        prop_assert!(span.is_some(), "scalar {} must have a span", key);
        let span = span.unwrap();

        let r = apply(before, &[edit(key, w.new_value())]);
        let after = match r {
            Ok(after) => after,
            Err(e) => {
                // apply は純関数で、Err ならファイルは書かれない。想定の Err は改行を含む値だけ(WB-7)。
                prop_assert!(
                    w.has_newline() && e == EditError::Newline,
                    "unexpected error {:?} for {:?} on {:?}\n{}", e, w, key, note.text()
                );
                return Ok(());
            }
        };
        prop_assert!(!w.has_newline(), "value with a newline must be rejected: {:?}", w);

        // (a) 読み直すと、対象のキーは書いた値(型も)で、ほかのキーの値は前と同じ(WB-6)。
        let fm_after = parse(&after);
        prop_assert!(
            fm_after.is_ok(),
            "reparse failed: {:?}\n{}", fm_after, String::from_utf8_lossy(&after)
        );
        let fm_after = fm_after.unwrap();
        prop_assert_eq!(
            value_of(&fm_after, key),
            Some(w.expected()),
            "[WB-6] {} after writing {:?}:\n{}", key, w, String::from_utf8_lossy(&after)
        );
        assert_others_unchanged(&fm_before, &fm_after, key)?;

        // (b) 対象のキーの行の外のバイトは前と同じ(WB-1)。
        let line = note.ranges[i].clone();
        let (pre, post) = (&before[..line.start], &before[line.end..]);
        prop_assert!(after.len() >= pre.len() + post.len());
        prop_assert!(after.starts_with(pre), "[WB-1] bytes before the line changed");
        prop_assert!(after.ends_with(post), "[WB-1] bytes after the line changed");
        let new_line = String::from_utf8_lossy(&after[line.start..after.len() - post.len()]).into_owned();
        let ValueSpec::Scalar { comment, .. } = &note.values[i] else { unreachable!() };
        prop_assert!(new_line.starts_with(&format!("{key}:")), "[WB-1] key kept: {:?}", new_line);
        // 行末のコメントは `#` から後ろが同じで、前に空白が残る(空の値の位置は空白の中なので、空白の数は問わない)。
        let tail = format!("{}{}", comment.trim_start(), note.nl);
        prop_assert!(new_line.ends_with(&tail), "[WB-1] comment and newline kept: {:?}", new_line);
        if !comment.is_empty() {
            let before_hash = &new_line[..new_line.len() - tail.len()];
            prop_assert!(before_hash.ends_with([' ', '\t']), "[WB-1] comment separated: {:?}", new_line);
        }
        prop_assert_eq!(new_line.matches('\n').count(), 1, "[WB-1] one line: {:?}", new_line);
        prop_assert_eq!(
            new_line.matches('\r').count(),
            usize::from(note.nl == "\r\n"),
            "[WB-1] newline style: {:?}", new_line
        );
        // さらに、変わったバイトは元の値の範囲の中だけ(WB-1)。
        // (同じ値を書いて1バイトも変わらないときは、範囲は無い。)
        let changed = changed_range(before, &after);
        // 人の判断待ち(2026-10-03): WB-1 と CE-9 の `key:` のぶつかり。決まったらここを直す
        // それまでは、Null を書くときだけ区切りの空白(`:` の後ろから値の前まで)も範囲に含めて許す。
        // 消すと前後の空白が同じなので、差分の位置ではなく「`:` までと値の後ろが元のまま、間は空白だけ」で見る。
        if matches!(w, Written::Null) {
            let colon_end = line.start + key.len() + 1;
            let (head, tail) = (&before[..colon_end], &before[span.end..]);
            let fits = after.len() >= head.len() + tail.len()
                && after.starts_with(head)
                && after.ends_with(tail)
                && after[head.len()..after.len() - tail.len()]
                    .iter()
                    .all(|&c| c == b' ' || c == b'\t');
            prop_assert!(
                fits,
                "[WB-1] Null changed {:?} outside the separator and value span {:?}: {:?}",
                changed, span, new_line
            );
        } else {
            prop_assert!(
                after == *before || (changed.start >= span.start && changed.end <= span.end),
                "[WB-1] changed {:?} is outside the value span {:?}: {:?}", changed, span, new_line
            );
        }
    }

    #[test]
    fn test_wb_6_props_written_value_reads_back((note, i) in note_with_scalar(), w in written()) {
        // [WB-6] 2つのキーを1度に書いても、両方が意図した値になり、ほかは変わらない。
        let scalars = note.scalar_indices();
        let j = scalars[(scalars.iter().position(|&x| x == i).unwrap() + 1) % scalars.len()];
        prop_assume!(!w.has_newline());
        let before = &note.bytes;
        let fm_before = parse(before).unwrap();
        let mut edits = vec![edit(&note.keys[i], w.new_value())];
        if j != i {
            edits.push(edit(&note.keys[j], NewValue::Str("二つ目".into())));
        }
        let after = apply(before, &edits);
        prop_assert!(after.is_ok(), "apply failed: {:?} for {:?}\n{}", after, w, note.text());
        let after = after.unwrap();
        let fm_after = parse(&after).unwrap();
        prop_assert_eq!(value_of(&fm_after, &note.keys[i]), Some(w.expected()));
        if j != i {
            prop_assert_eq!(value_of(&fm_after, &note.keys[j]), Some(Value::Str("二つ目".into())));
        }
        for (k, key) in note.keys.iter().enumerate() {
            if k != i && k != j {
                prop_assert_eq!(value_of(&fm_after, key), value_of(&fm_before, key), "[WB-6] {} changed", key);
            }
        }
        // 書いた行の外(本文を含む)は前と同じ。
        let first = note.ranges[i.min(j)].start;
        let last = note.ranges[i.max(j)].end;
        prop_assert!(after.starts_with(&before[..first]));
        prop_assert!(after.ends_with(&before[last..]));
    }

    // ---- WB-1: 無いキーを足す ----

    #[test]
    fn test_wb_1_props_added_key_is_one_line(note in note(), w in written()) {
        // [WB-1] [WB-3] 元のバイトはそのまま残り、閉じの区切りの前に1行だけ増える。
        prop_assume!(!w.has_newline());
        let before = &note.bytes;
        let fm_before = parse(before).unwrap();
        let key = "zz_追加";
        let after = apply(before, &[edit(key, w.new_value())]);
        prop_assert!(after.is_ok(), "apply failed: {:?} for {:?}\n{}", after, w, note.text());
        let after = after.unwrap();
        let (pre, post) = (&before[..note.end], &before[note.end..]);
        prop_assert!(after.starts_with(pre), "[WB-1] bytes before the closing delimiter changed");
        prop_assert!(after.ends_with(post), "[WB-1] closing delimiter and body changed");
        prop_assert!(after.len() > before.len());
        let added = String::from_utf8_lossy(&after[pre.len()..after.len() - post.len()]).into_owned();
        prop_assert!(added.starts_with(&format!("{key}:")), "[WB-1] added line: {:?}", added);
        prop_assert!(added.ends_with(note.nl), "[WB-1] newline of the file: {:?}", added);
        prop_assert_eq!(added.matches('\n').count(), 1, "[WB-1] one line: {:?}", added);
        prop_assert_eq!(added.matches('\r').count(), usize::from(note.nl == "\r\n"));
        let fm_after = parse(&after).unwrap();
        prop_assert_eq!(value_of(&fm_after, key), Some(w.expected()), "[WB-6] added {:?}", added);
        assert_others_unchanged(&fm_before, &fm_after, key)?;
    }

    // ---- WB-7: 文字列は文字列として戻る ----

    #[test]
    fn test_wb_7_props_text_reads_back_as_same_string(
        (note, i) in note_with_scalar(),
        random in value_string(),
    ) {
        // [WB-7]
        let key = &note.keys[i];
        let ValueSpec::Scalar { quote, .. } = &note.values[i] else { unreachable!() };
        let mut values: Vec<String> = RISKY.iter().map(|s| s.to_string()).collect();
        if !random.contains(['\n', '\r']) {
            values.push(random);
        }
        for v in values {
            let after = apply(&note.bytes, &[edit(key, NewValue::Str(v.clone()))]);
            prop_assert!(after.is_ok(), "apply failed for {:?}: {:?}\n{}", v, after, note.text());
            let after = after.unwrap();
            let fm = parse(&after);
            prop_assert!(fm.is_ok(), "reparse failed for {:?}: {:?}\n{}", v, fm, String::from_utf8_lossy(&after));
            let fm = fm.unwrap();
            prop_assert_eq!(
                value_of(&fm, key),
                Some(Value::Str(v.clone())),
                "[WB-7] {:?} read back:\n{}", v, String::from_utf8_lossy(&after)
            );
            let span = span_of(&fm, key);
            prop_assert!(span.is_some());
            let text = String::from_utf8_lossy(&after[span.unwrap()]).into_owned();
            match quote {
                // 元のクオートを保つ。一重で表せない値は二重(WB-7)。
                Quote::Double => prop_assert!(text.starts_with('"'), "[WB-7] keep \"\": {:?}", text),
                Quote::Single => prop_assert!(
                    text.starts_with('\'') || text.starts_with('"'),
                    "[WB-7] keep '': {:?}", text
                ),
                Quote::Plain => {}
            }
            if !text.starts_with(['"', '\'']) {
                // 囲まずに書いたなら、YAML 1.1 で読んでも文字列のまま(1.2 は上の parse で確かめた)。
                prop_assert!(!yaml11_typed(&text), "[WB-7] plain {:?} is typed in YAML 1.1", text);
                prop_assert_eq!(&text, &v, "[WB-7] plain text is the value itself");
            }
        }
    }

    // ---- WB-5: 任意のバイト列で panic しない ----

    #[test]
    fn test_wb_5_props_arbitrary_bytes_do_not_panic(
        bytes in prop_oneof![
            prop::collection::vec(any::<u8>(), 0..300),
            prop::collection::vec(any::<u8>(), 0..300).prop_map(|mut b| {
                let mut v = b"---\n".to_vec();
                v.append(&mut b);
                v
            }),
            prop::collection::vec(
                prop::sample::select(b"---\n\r:# '\"[]{}-|>&*!?ab\t\xef\xbb\xbf\xff".to_vec()),
                0..200,
            ).prop_map(|mut b| {
                let mut v = b"---\n".to_vec();
                v.append(&mut b);
                v
            }),
        ],
        w in written(),
    ) {
        // [WB-5] 読み取りと書き戻しは、どんなバイト列でも panic せずに結果か Err を返す。
        let _ = parse(&bytes);
        let _ = apply(&bytes, &[edit("a", w.new_value())]);
        let _ = apply(&bytes, &[edit("status", NewValue::Str("x".into())), edit("b", NewValue::List(vec!["y".into()]))]);
    }

    #[test]
    fn test_wb_5_props_mutated_notes_do_not_panic(
        note in note(),
        pos in any::<Index>(),
        byte in any::<u8>(),
        kind in 0u8..4,
        w in written(),
    ) {
        // [WB-5] 正しいノートを1か所壊しても panic しない。読めて書けたなら、書いたキーは読み直せる。
        let mut b = note.bytes.clone();
        let p = pos.index(b.len());
        match kind {
            0 => b[p] = byte,
            1 => b.insert(p, byte),
            2 => { b.remove(p); }
            _ => b.truncate(p),
        }
        let _ = parse(&b);
        let key = note.keys[0].clone();
        if let Ok(after) = apply(&b, &[edit(&key, w.new_value())]) {
            let fm = parse(&after);
            prop_assert!(fm.is_ok(), "[WB-6] written note must parse: {:?}", fm);
            prop_assert_eq!(value_of(&fm.unwrap(), &key), Some(w.expected()), "[WB-6]");
        }
    }

    // ---- WB-5: 読めない形は読むだけ ----

    #[test]
    fn test_wb_5_props_broken_notes_are_read_only(
        (note, i) in note_with_scalar(),
        kind in 0u8..5,
    ) {
        // [WB-5]
        let b = &note.bytes;
        let (broken, reason) = match kind {
            0 => {
                let mut v = b"\xef\xbb\xbf".to_vec();
                v.extend_from_slice(b);
                (v, ReadOnly::Bom)
            }
            1 => (b[..note.end].to_vec(), ReadOnly::Unclosed),
            2 => {
                // 同じキーをもう1行、閉じの区切りの前に足す。
                let mut v = b[..note.end].to_vec();
                v.extend_from_slice(format!("{}: dup{}", note.keys[i], note.nl).as_bytes());
                v.extend_from_slice(&b[note.end..]);
                (v, ReadOnly::DuplicateKey(note.keys[i].clone()))
            }
            3 => {
                // 開きの区切りの行だけ改行コードを変える。
                let other = if note.nl == "\n" { "\r\n" } else { "\n" };
                let mut v = format!("---{other}").into_bytes();
                v.extend_from_slice(&b[3 + note.nl.len()..]);
                (v, ReadOnly::MixedNewlines)
            }
            _ => {
                // 対象のキーの行に UTF-8 でないバイトを入れる。
                let mut v = b.clone();
                v.insert(note.ranges[i].start + note.keys[i].len() + 1, 0xff);
                (v, ReadOnly::NotUtf8)
            }
        };
        prop_assert_eq!(parse(&broken), Err(reason.clone()), "{}", String::from_utf8_lossy(&broken));
        prop_assert_eq!(
            apply(&broken, &[edit(&note.keys[i], NewValue::Str("done".into()))]),
            Err(EditError::ReadOnly(reason.clone()))
        );
        prop_assert_eq!(
            apply(&broken, &[edit("zz_追加", NewValue::Str("done".into()))]),
            Err(EditError::ReadOnly(reason))
        );
    }
}
