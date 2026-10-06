//! フロントマターの読み取りと、第1階層のキーの値の範囲の特定(タスク 2)。形は docs/design.md。
//!
//! YAML の一般の解析器は作らない。第1階層のキーの行と、その値の形(素のスカラー・引用符つき・
//! フローのリスト・ブロックのリスト・ブロックスカラー・ネスト・アンカー)を見分けるだけの、
//! 最小の読み取り。値は文字列のまま読み、型は YAML 1.2 の core の規則で判定する。
//! 分からない形は推測せず、読むだけ(span なし)か InvalidYaml にする。

use std::collections::HashSet;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Other,
}

/// 読むだけにする理由(CE-8・WB-5)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnly {
    NoFrontmatter,
    EmptyFrontmatter,
    Bom,
    NotUtf8,
    MixedNewlines,
    DuplicateKey(String),
    Unclosed,
    InvalidYaml,
    /// ハードリンクがある(WB-5)。parse は返さず、save が返す。
    HardLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    FlowList,
    BlockList,
    BlockScalar,
    Nested,
    Anchor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    pub shape: Shape,
    /// 値のバイトの範囲。書き換えられない形は None。BlockList は書ける形(CE-18)のときだけ、要素の行の範囲。
    pub span: Option<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
    pub entries: Vec<Entry>,
    /// 閉じの区切りの行の先頭のバイト位置。
    pub end: usize,
}

/// 1行。`content` は改行を除いた中身。
struct Line<'a> {
    start: usize,
    content: &'a str,
    newline: &'a str,
}

fn split_lines(text: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    while start < bytes.len() {
        match text[start..].find('\n') {
            Some(i) => {
                let nl_at = start + i;
                let (content_end, newline) = if nl_at > start && bytes[nl_at - 1] == b'\r' {
                    (nl_at - 1, "\r\n")
                } else {
                    (nl_at, "\n")
                };
                lines.push(Line {
                    start,
                    content: &text[start..content_end],
                    newline,
                });
                start = nl_at + 1;
            }
            None => {
                lines.push(Line {
                    start,
                    content: &text[start..],
                    newline: "",
                });
                start = bytes.len();
            }
        }
    }
    lines
}

fn is_delimiter(content: &str) -> bool {
    content.trim_end_matches([' ', '\t']) == "---"
}

pub fn parse(bytes: &[u8]) -> Result<Frontmatter, ReadOnly> {
    parse_full(bytes).map(|(fm, _)| fm)
}

/// ブロックのリストを書けない形にしている理由(CE-8・CE-19)。理由の文言は読み込み口が付ける。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListIssue {
    /// 要素の間に空行がある。
    BlankBetween,
    /// 要素の間にコメントの行がある。
    CommentBetween,
    /// 要素の行末にコメントがある。
    TrailingComment,
    /// 字下げの違う要素の行がある。
    Indent,
    /// `-` のあとがタブ。
    TabAfterDash,
    /// 要素が次の行に続く(複数行の値・入れ子の map)。
    MultiLine,
    /// 入れ子の要素(フローのリスト・map など)。
    Nested,
    /// 文字列でない要素(数・真偽値・空の要素など)。
    NonString,
}

/// 第1階層の `key` がブロックのリストで、書けない形なら、その理由。書ける形・ブロックのリストでなければ None。
pub(crate) fn list_issue(bytes: &[u8], key: &str) -> Option<ListIssue> {
    let (fm, issues) = parse_full(bytes).ok()?;
    let i = fm.entries.iter().position(|e| e.key == key)?;
    issues[i]
}

/// parse と、エントリーごとのブロックのリストの書けない理由(並びは entries と同じ)。
fn parse_full(bytes: &[u8]) -> Result<(Frontmatter, Vec<Option<ListIssue>>), ReadOnly> {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ReadOnly::Bom);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ReadOnly::NotUtf8)?;
    let lines = split_lines(text);
    let first = match lines.first() {
        Some(l) if is_delimiter(l.content) => l,
        _ => return Err(ReadOnly::NoFrontmatter),
    };
    if first.newline.is_empty() {
        return Err(ReadOnly::Unclosed);
    }
    let newline = first.newline;
    // 開きの行から閉じの行まで、改行コードがそろっていること(本文は見ない)。
    let mut close = None;
    for (i, l) in lines.iter().enumerate().skip(1) {
        if l.content.contains('\r') || (!l.newline.is_empty() && l.newline != newline) {
            return Err(ReadOnly::MixedNewlines);
        }
        if is_delimiter(l.content) {
            close = Some(i);
            break;
        }
    }
    let close = close.ok_or(ReadOnly::Unclosed)?;
    if close == 1 {
        return Err(ReadOnly::EmptyFrontmatter);
    }
    let (entries, issues) = parse_entries(&lines[1..close])?.into_iter().unzip();
    Ok((
        Frontmatter {
            entries,
            end: lines[close].start,
        },
        issues,
    ))
}

fn indent_of(s: &str) -> usize {
    s.len() - s.trim_start_matches([' ', '\t']).len()
}

fn is_blank_or_comment(s: &str) -> bool {
    let t = s.trim_start_matches([' ', '\t']);
    t.is_empty() || t.starts_with('#')
}

/// `-` のあとが空白か行末(ブロックのリストの要素)。
fn is_dash_item(s: &str) -> bool {
    s == "-" || s.starts_with("- ") || s.starts_with("-\t")
}

/// キーの行の値が、次の行からのブロック(リスト・map)を開くか(値が空、またはアンカー・タグだけ)。
fn opens_block(header: &str) -> bool {
    match read_inline(header) {
        Inline::Empty => true,
        Inline::Anchor | Inline::Tag => {
            only_comment_after(header.trim_start_matches(|c: char| !c.is_whitespace()))
        }
        _ => false,
    }
}

fn parse_entries(lines: &[Line<'_>]) -> Result<Vec<(Entry, Option<ListIssue>)>, ReadOnly> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        let c = line.content;
        if is_blank_or_comment(c) {
            i += 1;
            continue;
        }
        if indent_of(c) > 0 {
            // 第1階層のキーより前に字下げの行がある。
            return Err(ReadOnly::InvalidYaml);
        }
        let (key, value_at) = parse_key(c)?;
        let header = &c[value_at..];
        let block = opens_block(header);
        // 続きの行: 空行・コメント・字下げの行、値が空なら行頭の `- ` も。
        let mut j = i + 1;
        while j < lines.len() {
            let n = lines[j].content;
            let cont =
                is_blank_or_comment(n) || n.starts_with([' ', '\t']) || (block && is_dash_item(n));
            if !cont {
                break;
            }
            if !is_blank_or_comment(n) && n[..indent_of(n)].contains('\t') {
                // YAML の字下げにタブは使えない(WB-5)。
                return Err(ReadOnly::InvalidYaml);
            }
            j += 1;
        }
        let cont: Vec<&str> = lines[i + 1..j].iter().map(|l| l.content).collect();
        let mut entry = classify(key.clone(), header, line.start + value_at, &cont)?;
        let mut issue = None;
        if entry.shape == Shape::BlockList {
            match block_list_check(&lines[i + 1..j], &entry.value) {
                Ok(span) => entry.span = Some(span),
                Err(e) => issue = Some(e),
            }
        }
        if !seen.insert(key.clone()) {
            return Err(ReadOnly::DuplicateKey(key));
        }
        entries.push((entry, issue));
        i = j;
    }
    Ok(entries)
}

/// 第1階層のキーの行の並び(CE-29 の名前の変更と削除)。`token` はキーの文字(引用符を含む)のバイトの範囲、
/// `lines` はキーの行から値の最後の続きの行までの範囲(後ろの空行とコメントの行は、次のキーのものとして含めない)。
pub(crate) struct KeyLines {
    pub key: String,
    pub token: Range<usize>,
    pub lines: Range<usize>,
}

/// 読めるフロントマター(parse が通る形)のキーの行の並び。読めなければ None。
pub(crate) fn key_lines(bytes: &[u8]) -> Option<Vec<KeyLines>> {
    parse(bytes).ok()?;
    let text = std::str::from_utf8(bytes).ok()?;
    let lines = split_lines(text);
    let close = lines.iter().skip(1).position(|l| is_delimiter(l.content))? + 1;
    let body = &lines[1..close];
    let mut out = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let c = body[i].content;
        if is_blank_or_comment(c) {
            i += 1;
            continue;
        }
        let (key, value_at) = parse_key(c).ok()?;
        let token_end = key_token_end(c)?;
        let block = opens_block(&c[value_at..]);
        // ブロックの文字(`|`・`>`)の字下げした行は、`#` で始まっても値の中身。
        let scalar = c[value_at..].trim_start().starts_with(['|', '>']);
        let mut j = i + 1;
        let mut last = i;
        while j < body.len() {
            let n = body[j].content;
            let cont =
                is_blank_or_comment(n) || n.starts_with([' ', '\t']) || (block && is_dash_item(n));
            if !cont {
                break;
            }
            let content = !n.trim().is_empty() && scalar && n.starts_with([' ', '\t']);
            if !is_blank_or_comment(n) || content {
                last = j;
            }
            j += 1;
        }
        let start = body[i].start;
        let l = &body[last];
        out.push(KeyLines {
            key,
            token: start..start + token_end,
            lines: start..l.start + l.content.len() + l.newline.len(),
        });
        i = j;
    }
    Some(out)
}

/// キーの行のキーの文字の終わり(引用符つきは閉じの引用符の後ろ、素のキーは `:` の前の空白を除いた終わり)。
fn key_token_end(c: &str) -> Option<usize> {
    if c.starts_with(['"', '\'']) {
        return Some(read_quoted(c)?.end);
    }
    let b = c.as_bytes();
    let colon = (0..b.len())
        .find(|&i| b[i] == b':' && (i + 1 == b.len() || matches!(b[i + 1], b' ' | b'\t')))?;
    Some(c[..colon].trim_end_matches([' ', '\t']).len())
}

/// キーの行を読み、(キー, 値の始まりの行内の位置) を返す。
fn parse_key(c: &str) -> Result<(String, usize), ReadOnly> {
    let (key, after_colon) = if c.starts_with(['"', '\'']) {
        let q = read_quoted(c).ok_or(ReadOnly::InvalidYaml)?;
        let value = q.value.ok_or(ReadOnly::InvalidYaml)?;
        let pad = indent_of(&c[q.end..]);
        if !c[q.end + pad..].starts_with(':') {
            return Err(ReadOnly::InvalidYaml);
        }
        (value, q.end + pad + 1)
    } else {
        let first = c.chars().next().ok_or(ReadOnly::InvalidYaml)?;
        if "[]{},&*!|>%@`".contains(first) || (matches!(first, '-' | '?' | ':') && is_dash_like(c))
        {
            return Err(ReadOnly::InvalidYaml);
        }
        let b = c.as_bytes();
        let mut colon = None;
        for (i, &ch) in b.iter().enumerate() {
            if ch == b'#' && i > 0 && matches!(b[i - 1], b' ' | b'\t') {
                break;
            }
            if ch == b':' && (i + 1 == b.len() || matches!(b[i + 1], b' ' | b'\t')) {
                colon = Some(i);
                break;
            }
        }
        let colon = colon.ok_or(ReadOnly::InvalidYaml)?;
        let key = c[..colon].trim_end_matches([' ', '\t']);
        if key.is_empty() {
            return Err(ReadOnly::InvalidYaml);
        }
        (key.to_string(), colon + 1)
    };
    let rest = &c[after_colon..];
    if !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
        return Err(ReadOnly::InvalidYaml);
    }
    Ok((key, after_colon + indent_of(rest)))
}

struct Quoted {
    shape: Shape,
    /// 閉じの引用符が同じ行に無ければ None。
    value: Option<String>,
    /// 閉じの引用符の直後の位置。
    end: usize,
}

/// `s` の先頭の引用符つきのスカラーを読む。エスケープが不正なら None。
fn read_quoted(s: &str) -> Option<Quoted> {
    let mut chars = s.char_indices();
    let (_, q) = chars.next()?;
    let mut out = String::new();
    if q == '\'' {
        let mut it = chars.peekable();
        while let Some((i, ch)) = it.next() {
            if ch != '\'' {
                out.push(ch);
            } else if let Some(&(_, '\'')) = it.peek() {
                it.next();
                out.push('\'');
            } else {
                return Some(Quoted {
                    shape: Shape::SingleQuoted,
                    value: Some(out),
                    end: i + 1,
                });
            }
        }
        return Some(Quoted {
            shape: Shape::SingleQuoted,
            value: None,
            end: s.len(),
        });
    }
    fn hex(n: usize, chars: &mut std::str::CharIndices<'_>) -> Option<char> {
        let mut v = 0u32;
        for _ in 0..n {
            v = v * 16 + chars.next()?.1.to_digit(16)?;
        }
        char::from_u32(v)
    }
    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => {
                return Some(Quoted {
                    shape: Shape::DoubleQuoted,
                    value: Some(out),
                    end: i + 1,
                })
            }
            '\\' => {
                let c = match chars.next()?.1 {
                    '0' => '\0',
                    'a' => '\x07',
                    'b' => '\x08',
                    't' | '\t' => '\t',
                    'n' => '\n',
                    'v' => '\x0B',
                    'f' => '\x0C',
                    'r' => '\r',
                    'e' => '\x1B',
                    ' ' => ' ',
                    '"' => '"',
                    '/' => '/',
                    '\\' => '\\',
                    'N' => '\u{85}',
                    '_' => '\u{A0}',
                    'L' => '\u{2028}',
                    'P' => '\u{2029}',
                    'x' => hex(2, &mut chars)?,
                    'u' => hex(4, &mut chars)?,
                    'U' => hex(8, &mut chars)?,
                    _ => return None,
                };
                out.push(c);
            }
            _ => out.push(ch),
        }
    }
    Some(Quoted {
        shape: Shape::DoubleQuoted,
        value: None,
        end: s.len(),
    })
}

/// 1行の中の値の読み取りの結果。位置は読んだ文字列の中のバイト位置。
enum Inline {
    Empty,
    Quoted(Quoted),
    /// `closed` は同じ行で閉じたときの閉じの直後の位置。
    Flow {
        closed: Option<usize>,
        map: bool,
    },
    BlockScalar,
    Anchor,
    Tag,
    Plain {
        end: usize,
    },
    Invalid,
}

/// 閉じた値のあとに残る文字列が、空白と行末のコメントだけか。
fn only_comment_after(rest: &str) -> bool {
    let t = rest.trim_start_matches([' ', '\t']);
    t.is_empty() || (t.starts_with('#') && t.len() < rest.len())
}

/// 先頭の記号のあとが空白か行末。
fn is_dash_like(s: &str) -> bool {
    s[1..].chars().next().is_none_or(|c| c == ' ' || c == '\t')
}

fn read_inline(s: &str) -> Inline {
    let Some(first) = s.chars().next() else {
        return Inline::Empty;
    };
    match first {
        '#' => Inline::Empty,
        '"' | '\'' => read_quoted(s).map_or(Inline::Invalid, Inline::Quoted),
        '[' | '{' => Inline::Flow {
            closed: find_flow_end(s),
            map: first == '{',
        },
        '|' | '>' => {
            let rest =
                s[1..].trim_start_matches(|c: char| c == '+' || c == '-' || c.is_ascii_digit());
            if only_comment_after(rest) {
                Inline::BlockScalar
            } else {
                Inline::Invalid
            }
        }
        '&' | '*' => Inline::Anchor,
        '!' => Inline::Tag,
        '@' | '`' | '%' | ',' | ']' | '}' => Inline::Invalid,
        '-' | '?' | ':' if is_dash_like(s) => Inline::Invalid,
        _ => {
            let end = plain_end(s);
            let v = &s[..end];
            if v.ends_with(':') || v.contains(": ") || v.contains(":\t") {
                Inline::Invalid
            } else {
                Inline::Plain { end }
            }
        }
    }
}

/// 素のスカラーの終わり(行末のコメントと末尾の空白を除く)。
fn plain_end(s: &str) -> usize {
    let b = s.as_bytes();
    let mut end = b.len();
    for i in 1..b.len() {
        if b[i] == b'#' && matches!(b[i - 1], b' ' | b'\t') {
            end = i;
            break;
        }
    }
    s[..end].trim_end_matches([' ', '\t']).len()
}

/// フローの `[`・`{` に対応する閉じの直後の位置。同じ行で閉じなければ None。
fn find_flow_end(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            b'"' | b'\'' if matches!(b[i - 1], b'[' | b'{' | b',' | b' ' | b'\t') => {
                let q = read_quoted(&s[i..])?;
                q.value.as_ref()?;
                i += q.end;
                continue;
            }
            b'#' if matches!(b[i - 1], b' ' | b'\t') => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// フローのリストの中身(`[` と `]` を除いた文字列)を要素に分ける。
/// 別名・アンカーを含めば Err(true)、形が不正なら Err(false)。
fn flow_items(inner: &str) -> Result<Vec<Value>, bool> {
    let mut items = Vec::new();
    for t in flow_parts(inner)? {
        items.push(match t.chars().next() {
            None => return Err(false),
            Some('&' | '*') => return Err(true),
            Some('"' | '\'') => match read_quoted(t) {
                Some(Quoted {
                    value: Some(v),
                    end,
                    ..
                }) if end == t.len() => Value::Str(v),
                _ => return Err(false),
            },
            Some('[' | '{' | '!') => Value::Other,
            _ if t.contains(": ") || t.ends_with(':') => Value::Other,
            _ => resolve_plain(t),
        });
    }
    Ok(items)
}

/// フローのリストの中身を、要素ごとの書き方(前後の空白を除く)に分ける。末尾のカンマの後ろの空は除く。
/// 括弧の対応が不正なら Err(false)。
fn flow_parts(inner: &str) -> Result<Vec<&str>, bool> {
    let mut parts = Vec::new();
    let b = inner.as_bytes();
    let mut depth = 0usize;
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'[' | b'{' => depth += 1,
            b']' | b'}' => depth = depth.checked_sub(1).ok_or(false)?,
            b'"' | b'\'' if inner[start..i].trim().is_empty() || depth > 0 => {
                let q = read_quoted(&inner[i..]).ok_or(false)?;
                i += q.end;
                continue;
            }
            b',' if depth == 0 => {
                parts.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&inner[start..]);
    let n = parts.len();
    let mut out = Vec::new();
    for (k, p) in parts.into_iter().enumerate() {
        let t = p.trim_matches([' ', '\t']);
        if t.is_empty() && k == n - 1 {
            continue; // 空のリストと末尾のカンマ
        }
        // 途中の空の要素は、呼ぶ側が不正とする(アンカーの検出の順を保つため、ここでは残す)。
        out.push(t);
    }
    Ok(out)
}

/// フローの形のフロントマターを読む中身の大きさと、括弧の入れ子の深さの上限。
const FLOW_MAX_BYTES: usize = 64 * 1024;
const FLOW_MAX_DEPTH: usize = 32;

/// `{` と `[` の入れ子の最も深い段(引用符の中は数えない。粗く数えて、多めにずれても読まないだけ)。
fn flow_depth(s: &str) -> usize {
    let (mut depth, mut max) = (0usize, 0usize);
    let mut quote: Option<char> = None;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some('"'), '\\') => {
                chars.next();
            }
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '{' | '[') => {
                depth += 1;
                max = max.max(depth);
            }
            (None, '}' | ']') => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    max
}

/// 区切りの間の中身が `{` で始まる(JSON・フローの形のマップ)フロントマターを、見せる値のために読む(C-4)。
/// 第1階層のキーと値だけ(入れ子は Other)。書き換えの範囲は持たない(span は None。書かない。WB-5)。
/// 別名の展開は .base と同じ上限で止める。読めなければ None。
pub fn flow_frontmatter(bytes: &[u8]) -> Option<Frontmatter> {
    use saphyr::{LoadableYamlNode, Scalar, Yaml};
    let text = std::str::from_utf8(bytes).ok()?;
    let lines = split_lines(text);
    if !is_delimiter(lines.first()?.content) {
        return None;
    }
    let close = lines.iter().skip(1).find(|l| is_delimiter(l.content))?;
    let first = lines.first()?;
    let inner = &text[first.start + first.content.len() + first.newline.len()..close.start];
    if !inner.trim_start().starts_with('{') {
        return None;
    }
    // 信用できないノートの中身を、再帰する読み手に渡す前に止める(深い入れ子でスタックを使い切らない。
    // 大きな中身で遅くならない)。
    if inner.len() > FLOW_MAX_BYTES || flow_depth(inner) > FLOW_MAX_DEPTH {
        return None;
    }
    crate::base::check_aliases(inner).ok()?;
    let docs = Yaml::load_from_str(inner).ok()?;
    fn scalar(y: &Yaml) -> Value {
        match y {
            Yaml::Tagged(_, inner) => scalar(inner),
            Yaml::Value(Scalar::Null) => Value::Null,
            Yaml::Value(Scalar::Boolean(b)) => Value::Bool(*b),
            Yaml::Value(Scalar::Integer(n)) => Value::Int(*n),
            Yaml::Value(Scalar::FloatingPoint(f)) => Value::Float(f.into_inner()),
            Yaml::Value(Scalar::String(s)) => Value::Str(s.to_string()),
            Yaml::Representation(s, _, _) => resolve_plain(s),
            _ => Value::Other,
        }
    }
    let map = docs.first()?.as_mapping()?;
    let mut entries = Vec::new();
    for (k, v) in map {
        let key = match scalar(k) {
            Value::Str(s) => s,
            Value::Int(n) => n.to_string(),
            _ => continue,
        };
        let value = match v {
            Yaml::Sequence(items) => Value::List(
                items
                    .iter()
                    .map(|x| match scalar(x) {
                        Value::List(_) => Value::Other,
                        other => other,
                    })
                    .collect(),
            ),
            other => scalar(other),
        };
        entries.push(Entry {
            key,
            value,
            shape: Shape::Plain,
            span: None,
        });
    }
    Some(Frontmatter {
        entries,
        end: close.start,
    })
}

/// 1行目が `+++`(Hugo・Zola の TOML のフロントマター)か(WB-20)。parse はこれを「フロントマターが無い」と
/// 返すので、読み込み口と書き戻しがこれで読むだけにする(先頭に `---` を足すと、生成器が読めないページになる)。
pub fn is_toml(bytes: &[u8]) -> bool {
    let first = bytes.split(|&b| b == b'\n').next().unwrap_or(&[]);
    first.trim_ascii_end() == b"+++"
}

/// YAML 1.2 の core の規則で素のスカラーの型を決める。i64 に収まらない整数は元の文字のまま
/// (丸めた数やマップの印で見せない。CV-2)。
pub(crate) fn resolve_plain(s: &str) -> Value {
    match s {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => {
            return Value::Float(f64::INFINITY)
        }
        "-.inf" | "-.Inf" | "-.INF" => return Value::Float(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => return Value::Float(f64::NAN),
        _ => {}
    }
    let digits = |t: &str, radix: u32| !t.is_empty() && t.chars().all(|c| c.is_digit(radix));
    if let Some(h) = s.strip_prefix("0x").filter(|h| digits(h, 16)) {
        return i64::from_str_radix(h, 16).map_or_else(|_| Value::Str(s.to_string()), Value::Int);
    }
    if let Some(o) = s.strip_prefix("0o").filter(|o| digits(o, 8)) {
        return i64::from_str_radix(o, 8).map_or_else(|_| Value::Str(s.to_string()), Value::Int);
    }
    let unsigned = s.strip_prefix(['-', '+']).unwrap_or(s);
    if digits(unsigned, 10) {
        return s
            .parse::<i64>()
            .map_or_else(|_| Value::Str(s.to_string()), Value::Int);
    }
    if is_core_float(unsigned) {
        return s.parse::<f64>().map_or(Value::Other, Value::Float);
    }
    Value::Str(s.to_string())
}

/// 整数の形(10進・`0x`・`0o`)か。i64 に収まらなくても真(書き戻しで引用符を外さない。CV-2)。
pub(crate) fn is_int_form(s: &str) -> bool {
    let digits = |t: &str, radix: u32| !t.is_empty() && t.chars().all(|c| c.is_digit(radix));
    s.strip_prefix("0x").is_some_and(|h| digits(h, 16))
        || s.strip_prefix("0o").is_some_and(|o| digits(o, 8))
        || digits(s.strip_prefix(['-', '+']).unwrap_or(s), 10)
}

/// `(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?`(符号を除いた形)。
fn is_core_float(s: &str) -> bool {
    let all_digits = |t: &str| t.chars().all(|c| c.is_ascii_digit());
    let (mant, exp) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let mant_ok = match mant.split_once('.') {
        Some((a, b)) => all_digits(a) && all_digits(b) && !(a.is_empty() && b.is_empty()),
        None => !mant.is_empty() && all_digits(mant),
    };
    let exp_ok = exp.is_none_or(|e| {
        let e = e.strip_prefix(['-', '+']).unwrap_or(e);
        !e.is_empty() && all_digits(e)
    });
    mant_ok && exp_ok
}

fn entry(key: String, value: Value, shape: Shape, span: Option<Range<usize>>) -> Entry {
    Entry {
        key,
        value,
        shape,
        span,
    }
}

/// 第1階層のキーの値を分類する。`at` は値の始まりの絶対位置、`cont` は続きの行。
fn classify(key: String, header: &str, at: usize, cont: &[&str]) -> Result<Entry, ReadOnly> {
    let has_cont = cont.iter().any(|l| !is_blank_or_comment(l));
    let invalid = Err(ReadOnly::InvalidYaml);
    Ok(match read_inline(header) {
        Inline::Invalid => return invalid,
        Inline::Anchor => entry(key, Value::Other, Shape::Anchor, None),
        // タグつきの値は推測しない(読むだけ)。
        Inline::Tag => entry(key, Value::Other, Shape::Plain, None),
        Inline::BlockScalar => {
            let value = block_scalar_value(header, cont);
            entry(key, value, Shape::BlockScalar, None)
        }
        // `key:` だけ。値の位置に空の範囲を置き、書くときに空白を補う。
        Inline::Empty if !has_cont => entry(key, Value::Null, Shape::Plain, Some(at..at)),
        Inline::Empty => match block_list(cont) {
            Some(items) => entry(key, Value::List(items?), Shape::BlockList, None),
            None => entry(key, Value::Other, Shape::Nested, None),
        },
        Inline::Quoted(q) => match q.value {
            Some(v) if only_comment_after(&header[q.end..]) && !has_cont => {
                entry(key, Value::Str(v), q.shape, Some(at..at + q.end))
            }
            Some(_) => return invalid,
            // 複数行にまたがる引用符つきの値は読むだけ。
            None if has_cont => entry(key, Value::Other, q.shape, None),
            None => return invalid,
        },
        Inline::Flow {
            closed: Some(end),
            map,
        } => {
            if !only_comment_after(&header[end..]) || has_cont {
                return invalid;
            }
            if map {
                entry(key, Value::Other, Shape::Nested, None)
            } else {
                match flow_items(&header[1..end - 1]) {
                    Ok(items) => {
                        entry(key, Value::List(items), Shape::FlowList, Some(at..at + end))
                    }
                    Err(true) => entry(key, Value::Other, Shape::Anchor, None),
                    Err(false) => return invalid,
                }
            }
        }
        // 複数行のフローは読むだけ。
        Inline::Flow { closed: None, map } if has_cont => {
            let shape = if map { Shape::Nested } else { Shape::FlowList };
            entry(key, Value::Other, shape, None)
        }
        Inline::Flow { closed: None, .. } => return invalid,
        // 複数行の素のスカラーは読むだけ。
        Inline::Plain { .. } if has_cont => entry(key, Value::Other, Shape::Plain, None),
        Inline::Plain { end } => {
            let v = resolve_plain(&header[..end]);
            entry(key, v, Shape::Plain, Some(at..at + end))
        }
    })
}

/// 続きの行がブロックのリストなら要素を返す(不正なら Some(Err))。リストでなければ None。
fn block_list(cont: &[&str]) -> Option<Result<Vec<Value>, ReadOnly>> {
    let lines: Vec<&str> = cont
        .iter()
        .copied()
        .filter(|l| !is_blank_or_comment(l))
        .collect();
    let first = lines.first()?;
    let n = indent_of(first);
    if !is_dash_item(&first[n..]) {
        return None;
    }
    let mut items: Vec<Value> = Vec::new();
    for l in lines {
        let ind = indent_of(l);
        if ind > n {
            // 前の要素の続き(複数行の要素やネスト)。
            if let Some(last) = items.last_mut() {
                *last = Value::Other;
            }
            continue;
        }
        if ind < n || !is_dash_item(&l[n..]) {
            return Some(Err(ReadOnly::InvalidYaml));
        }
        let text = l[n + 1..].trim_start_matches([' ', '\t']);
        items.push(match read_inline(text) {
            Inline::Empty => Value::Null,
            Inline::Plain { end } => resolve_plain(&text[..end]),
            Inline::Quoted(Quoted {
                value: Some(v),
                end,
                ..
            }) if only_comment_after(&text[end..]) => Value::Str(v),
            Inline::Flow {
                closed: Some(end),
                map: false,
            } if only_comment_after(&text[end..]) => {
                flow_items(&text[1..end - 1]).map_or(Value::Other, Value::List)
            }
            _ => Value::Other,
        });
    }
    Some(Ok(items))
}

/// ブロックのリストが書ける形(CE-18)なら、要素の行の範囲(最初の要素の行の先頭から、最後の要素の行の改行の後ろまで)。
/// 書ける形: 要素が全部文字列で、要素の行が間に空行・コメント・続きの行を挟まずに並び、各要素が1行の素のスカラーか
/// 引用符つきの値で、要素の後ろに行末のコメントが無い。ほかは書けない理由(読むだけ。CE-8)。
/// 最後の要素の後ろの空行・コメントは範囲の外なので、書けない理由にしない。
fn block_list_check(cont: &[Line<'_>], value: &Value) -> Result<Range<usize>, ListIssue> {
    let Value::List(items) = value else {
        return Err(ListIssue::Nested);
    };
    let is_item = |l: &Line<'_>| !is_blank_or_comment(l.content);
    let first = cont.iter().position(is_item).ok_or(ListIssue::NonString)?;
    let last = cont.iter().rposition(is_item).ok_or(ListIssue::NonString)?;
    let item_lines = &cont[first..=last];
    let n = indent_of(item_lines[0].content);
    for l in item_lines {
        let c = l.content;
        if c.trim_matches([' ', '\t']).is_empty() {
            return Err(ListIssue::BlankBetween);
        }
        if is_blank_or_comment(c) {
            return Err(ListIssue::CommentBetween);
        }
        let ind = indent_of(c);
        if ind != n {
            return Err(if is_dash_item(&c[ind..]) {
                ListIssue::Indent
            } else {
                ListIssue::MultiLine
            });
        }
        if c[n..].starts_with("-\t") {
            return Err(ListIssue::TabAfterDash);
        }
    }
    if items
        .iter()
        .any(|v| matches!(v, Value::Other | Value::List(_)))
    {
        return Err(ListIssue::Nested);
    }
    if !items.iter().all(|v| matches!(v, Value::Str(_))) {
        return Err(ListIssue::NonString);
    }
    if item_lines
        .iter()
        .any(|l| item_raw(l.content).is_none_or(str::is_empty))
    {
        return Err(ListIssue::TrailingComment);
    }
    let last = &cont[last];
    Ok(item_lines[0].start..last.start + last.content.len() + last.newline.len())
}

/// ブロックのリストの要素の行(`  - a`)の、要素の書き方(`a`・`"a"`)。行末のコメントがあれば None。
fn item_raw(line: &str) -> Option<&str> {
    let n = indent_of(line);
    let text = line[n..].strip_prefix('-')?.trim_matches([' ', '\t']);
    let end = match read_inline(text) {
        Inline::Plain { end } => end,
        Inline::Quoted(Quoted {
            value: Some(_),
            end,
            ..
        }) => end,
        _ => return None,
    };
    (end == text.len()).then_some(text)
}

/// 書ける形のリスト(span のある FlowList・BlockList)の値の範囲から、要素ごとの元の書き方を返す(書き戻しで
/// 変わらない要素の書き方を保つため)。並びは Entry.value の要素と同じ。読めなければ None。
pub(crate) fn list_item_sources(shape: Shape, span_text: &str) -> Option<Vec<&str>> {
    match shape {
        Shape::FlowList => {
            let inner = span_text.strip_prefix('[')?.strip_suffix(']')?;
            flow_parts(inner)
                .ok()
                .filter(|parts| parts.iter().all(|p| !p.is_empty()))
        }
        Shape::BlockList => span_text
            .lines()
            .map(|l| item_raw(l.strip_suffix('\r').unwrap_or(l)))
            .collect(),
        _ => None,
    }
}

/// ブロックスカラー(`|`・`>`)の値(表示用のおおまかな読み取り。より深い字下げの折り返しの規則は省く)。
fn block_scalar_value(header: &str, cont: &[&str]) -> Value {
    let literal = header.starts_with('|');
    let ind: String = header[1..]
        .chars()
        .take_while(|c| *c == '+' || *c == '-' || c.is_ascii_digit())
        .collect();
    let explicit = ind.chars().find_map(|c| c.to_digit(10));
    let indent = match explicit {
        Some(d) => d as usize,
        None => cont
            .iter()
            .find(|l| !l.trim().is_empty())
            .map_or(0, |l| indent_of(l)),
    };
    let mut body: Vec<&str> = cont.iter().map(|l| l.get(indent..).unwrap_or("")).collect();
    let mut trailing = 0;
    while body.last().is_some_and(|l| l.trim().is_empty()) {
        body.pop();
        trailing += 1;
    }
    let mut out = String::new();
    if literal {
        out = body.join("\n");
    } else {
        for (i, l) in body.iter().enumerate() {
            if i > 0 {
                out.push(if l.is_empty() || body[i - 1].is_empty() {
                    '\n'
                } else {
                    ' '
                });
            }
            out.push_str(l);
        }
    }
    if !body.is_empty() && !ind.contains('-') {
        out.push('\n');
        if ind.contains('+') {
            out.push_str(&"\n".repeat(trailing));
        }
    }
    Value::Str(out)
}

/// YAML として読めないフロントマターの、最初の誤りのファイルの行(1始まり)と誤りの文(WB-5 の理由を詳しくする)。
/// フロントマター(先頭の `---` の行から次の `---` の行の前まで)を saphyr_parser で読み直す。フロントマターが無い・
/// YAML としては読める(mdgrid の独自の検査で読むだけにした)ときは None。
pub fn yaml_error(bytes: &[u8]) -> Option<(usize, String)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let end = if rest.starts_with("---") {
        0
    } else {
        rest.find("\n---").map(|i| i + 1).unwrap_or(rest.len())
    };
    for ev in saphyr_parser::Parser::new_from_str(&rest[..end]) {
        if let Err(e) = ev {
            // 開きの `---` の1行の分を足す。
            return Some((e.marker().line() + 1, e.info().to_string()));
        }
    }
    None
}

#[cfg(test)]
#[path = "test_yaml_error_unit.rs"]
mod test_yaml_error_unit;

#[cfg(test)]
#[path = "test_big_int_unit.rs"]
mod test_big_int_unit;

#[cfg(test)]
#[path = "test_flow_limits_unit.rs"]
mod test_flow_limits_unit;
