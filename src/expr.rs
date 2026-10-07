//! 式(タスク 6。BV-6・BV-7)。字句解析 → 構文木 → 評価の3段。
//! 未対応の関数・メソッド・項目は構文木の段(`parse`)で `Unsupported` になる。
//! 評価は止まらない: 型の合わない演算・あふれ・0 での割り算は Null。構文木の深さは parse で上限を持つ。
//! 形は docs/design.md の「核の公開のインターフェース」。

use crate::i18n::Msg;
use crate::source::{FileInfo, Value};
use crate::types::{format_date, parse_date};
use std::cmp::Ordering;

/// 式の値。日付は 1970-01-01 からの日数、日時は地域の時計の秒(UNIX 秒 + 地域の時差。docs/design.md)、期間はミリ秒。
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Date(i64),
    DateTime(i64),
    Duration(i64),
    List(Vec<Val>),
}

/// parse の誤り。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprError {
    /// 文法の誤り(理由)。
    Syntax(String),
    /// 未対応の関数・メソッド・項目の名前(BV-7)。
    Unsupported(String),
}

/// 構文木。parse でだけ作れる(深さは MAX_DEPTH まで)。
#[derive(Clone, Debug)]
pub struct Expr {
    node: Node,
}

/// 評価の環境。
pub struct Env<'a> {
    /// ノートのキー(ためた値を重ねたもの。NV-12)。
    pub prop: &'a dyn Fn(&str) -> Option<Value>,
    pub file: &'a FileInfo,
    /// formula.x の値(循環は呼ぶ側で Null)。
    pub formula: &'a dyn Fn(&str) -> Option<Val>,
    pub today: i64,
    pub now: i64,
}

/// BV-22: リンクの索引と、`.base` を直接開いたときの this(その `.base` のファイル)。無ければ None。
#[derive(Default, Clone, Copy)]
pub struct Links<'a> {
    pub index: Option<&'a crate::links::Index>,
    pub this: Option<&'a FileInfo>,
}

/// 式の読み取りが受け付けるトップの関数の名前(BV-21)。`if(...)` の `if` だけを置く。
/// 読み取りはこの一覧に無い名前を未対応(BV-7)にする。docs/obsidian-bases.md の「Functions」と同じ。
pub const FUNCTIONS: &[&str] = &[
    "if", "date", "now", "today", "duration", "number", "max", "min", "list",
];

/// 値に付けるメソッドの名前(BV-21)。`x.contains(...)` の `contains` だけを置く。
/// `length` と日付・日時の `year`・`month`・`day`・`hour`・`minute`・`second` は括弧の無い形(`x.length`)。
/// docs/obsidian-bases.md の「Methods」と同じ。
pub const METHODS: &[&str] = &[
    "contains",
    "containsAll",
    "containsAny",
    "isEmpty",
    "toString",
    "lower",
    "upper",
    "title",
    "trim",
    "startsWith",
    "endsWith",
    "slice",
    "replace",
    "split",
    "reverse",
    "round",
    "floor",
    "ceil",
    "abs",
    "date",
    "time",
    "format",
    "join",
    "unique",
    "sort",
    "flat",
    "length",
    "year",
    "month",
    "day",
    "hour",
    "minute",
    "second",
];

/// `file.` の後ろに置ける名前(BV-21)。値(`file.name`)と、`file` の関数(`file.hasTag(...)`)。
/// docs/obsidian-bases.md の「File properties」と同じ。
pub const FILE_FIELDS: &[&str] = &[
    "name",
    "basename",
    "ext",
    "path",
    "folder",
    "size",
    "mtime",
    "ctime",
    "tags",
    "links",
    "backlinks",
    "hasTag",
    "hasLink",
    "inFolder",
    "hasProperty",
];

/// 構文木の深さと、parse の入れ子の上限。
const MAX_DEPTH: usize = 128;
/// 式で作る文字の長さの上限(バイト)。超えるなら作らずに null(共有された .base の式でメモリを使い切らせない。BV-7)。
const MAX_STR: usize = 1 << 20;

/// 値の重さ(文字は長さ、どの値にも1つあたり 16 を足す)。式で作るリストがこれを超えるなら null
/// (リストの書き方・flat・split で大きさを倍々にさせない。BV-7)。
fn weight(v: &Val) -> usize {
    16 + match v {
        Val::Str(s) => s.len(),
        Val::List(xs) => xs.iter().map(weight).fold(0usize, usize::saturating_add),
        _ => 0,
    }
}

const DAY_SECS: i64 = 86_400;
const DAY_MS: i64 = 86_400_000;
/// 扱う日付の範囲(0000-01-01 〜 9999-12-31)。外は Null。
const MIN_DAY: i64 = -719_528;
const MAX_DAY: i64 = 2_932_896;

// ---- 字句解析 ----

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Str(String),
    Ident(String),
    LParen,
    RParen,
    LBrack,
    RBrack,
    Comma,
    Dot,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphanumeric()
}

fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let cs: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        let next = cs.get(i + 1).copied();
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && next.is_some_and(|n| n.is_ascii_digit())) {
            let start = i;
            while i < cs.len() && cs[i].is_ascii_digit() {
                i += 1;
            }
            if i < cs.len() && cs[i] == '.' && cs.get(i + 1).is_some_and(|n| n.is_ascii_digit()) {
                i += 1;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') {
                let mut j = i + 1;
                if j < cs.len() && (cs[j] == '+' || cs[j] == '-') {
                    j += 1;
                }
                if j < cs.len() && cs[j].is_ascii_digit() {
                    while j < cs.len() && cs[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            let text: String = cs[start..i].iter().collect();
            let n: f64 = text
                .parse()
                .map_err(|_| Msg::ExprBadNumber.fill(&[&text]))?;
            out.push(Tok::Num(n));
            if i < cs.len() && is_ident_start(cs[i]) {
                return Err(Msg::ExprNameAfterNumber.fill(&[&text, &cs[i]]));
            }
            continue;
        }
        if c == '"' || c == '\'' {
            let q = c;
            let mut s = String::new();
            i += 1;
            loop {
                let Some(&ch) = cs.get(i) else {
                    return Err(Msg::ExprUnclosedString.text().to_string());
                };
                i += 1;
                if ch == q {
                    break;
                }
                if ch == '\\' {
                    let Some(&e) = cs.get(i) else {
                        return Err(Msg::ExprUnclosedString.text().to_string());
                    };
                    i += 1;
                    s.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        other => other,
                    });
                } else {
                    s.push(ch);
                }
            }
            out.push(Tok::Str(s));
            continue;
        }
        if is_ident_start(c) {
            let start = i;
            while i < cs.len() && is_ident_char(cs[i]) {
                i += 1;
            }
            out.push(Tok::Ident(cs[start..i].iter().collect()));
            continue;
        }
        let two = |a: char, b: char| c == a && next == Some(b);
        let (tok, len) = if two('=', '=') {
            (Tok::Eq, 2)
        } else if two('!', '=') {
            (Tok::Ne, 2)
        } else if two('<', '=') {
            (Tok::Le, 2)
        } else if two('>', '=') {
            (Tok::Ge, 2)
        } else if two('&', '&') {
            (Tok::And, 2)
        } else if two('|', '|') {
            (Tok::Or, 2)
        } else {
            let t = match c {
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '[' => Tok::LBrack,
                ']' => Tok::RBrack,
                ',' => Tok::Comma,
                '.' => Tok::Dot,
                '!' => Tok::Not,
                '<' => Tok::Lt,
                '>' => Tok::Gt,
                '+' => Tok::Plus,
                '-' => Tok::Minus,
                '*' => Tok::Star,
                '/' => Tok::Slash,
                '%' => Tok::Percent,
                other => return Err(Msg::ExprBadChar.fill(&[&other])),
            };
            (t, 1)
        };
        // `==` の後ろにもう1つ `=`(`===`)は JavaScript の書き方。同じ意味として読む。
        let extra = usize::from(
            matches!(tok, Tok::Eq | Tok::Ne) && cs.get(i + 2).is_some_and(|&x| x == '='),
        );
        out.push(tok);
        i += len + extra;
    }
    Ok(out)
}

// ---- 構文木 ----

#[derive(Clone, Copy, Debug, PartialEq)]
enum BinOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Func {
    If,
    Date,
    Now,
    Today,
    Duration,
    Number,
    Max,
    Min,
    List,
    HasTag,
    /// `file.hasLink(x)`(BV-22)。
    HasLink,
    InFolder,
    HasProperty,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Meth {
    Contains,
    ContainsAll,
    ContainsAny,
    IsEmpty,
    ToString,
    /// 文字列を小文字にする(Obsidian の `lower()`)。文字列でなければ Null。
    Lower,
    Upper,
    Title,
    Trim,
    StartsWith,
    EndsWith,
    /// 文字列とリスト。
    Slice,
    /// pattern は文字だけ(正規表現は無い)。全部を置き換える。
    Replace,
    Split,
    /// 文字列とリスト。
    Reverse,
    Round,
    Floor,
    Ceil,
    Abs,
    /// 日付・日時の時刻を落とす。
    Date,
    /// 日付・日時の時刻の文字 `HH:mm:ss`。
    Time,
    Format,
    Join,
    Unique,
    Sort,
    Flat,
}

/// 括弧の無いメソッド(`x.length`・`d.year` など)。
#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Length,
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FileField {
    Name,
    Basename,
    Ext,
    Path,
    Folder,
    Size,
    Mtime,
    Ctime,
    Tags,
    /// BV-22: 行き先のリスト(索引から)。
    Links,
    /// BV-22: 被リンクのリスト(索引から)。
    Backlinks,
}

#[derive(Clone, Debug)]
enum Node {
    Lit(Val),
    List(Vec<Node>),
    Prop(String),
    File(FileField),
    Formula(String),
    /// BV-22: `this.file`(None。値は this のパス)と `this.file.x`。
    This(Option<FileField>),
    Not(Box<Node>),
    Neg(Box<Node>),
    Bin(BinOp, Box<Node>, Box<Node>),
    Call(Func, Vec<Node>),
    Method(Box<Node>, Meth, Vec<Node>),
    Field(Box<Node>, Field),
    Index(Box<Node>, Box<Node>),
    /// 未対応の場所(parse は Unsupported を返すので評価されない)。
    Bad,
}

/// (節, 深さ)。
type Pr = Result<(Node, usize), String>;

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    nest: usize,
    unsupported: Option<String>,
}

// 名前の判定は、まず一覧の定数(FUNCTIONS・METHODS・FILE_FIELDS)で絞る。定数に無い名前は
// `match` に腕があっても未対応。定数にあって腕が無い名前も未対応になり、tests/test_bases_docs.rs が落とす。

/// トップの関数(種類, 引数の最少, 最多)。
fn func_spec(name: &str) -> Option<(Func, usize, usize)> {
    if !FUNCTIONS.contains(&name) {
        return None;
    }
    Some(match name {
        "if" => (Func::If, 2, 3),
        "date" => (Func::Date, 1, 1),
        "now" => (Func::Now, 0, 0),
        "today" => (Func::Today, 0, 0),
        "duration" => (Func::Duration, 1, 1),
        "number" => (Func::Number, 1, 1),
        "max" => (Func::Max, 1, usize::MAX),
        "min" => (Func::Min, 1, usize::MAX),
        "list" => (Func::List, 1, 1),
        _ => return None,
    })
}

/// 括弧の付くメソッド(種類, 引数の最少, 最多)。
fn meth_spec(name: &str) -> Option<(Meth, usize, usize)> {
    if !METHODS.contains(&name) {
        return None;
    }
    Some(match name {
        "contains" => (Meth::Contains, 1, 1),
        "containsAll" => (Meth::ContainsAll, 1, usize::MAX),
        "containsAny" => (Meth::ContainsAny, 1, usize::MAX),
        "isEmpty" => (Meth::IsEmpty, 0, 0),
        "toString" => (Meth::ToString, 0, 0),
        "lower" => (Meth::Lower, 0, 0),
        "upper" => (Meth::Upper, 0, 0),
        "title" => (Meth::Title, 0, 0),
        "trim" => (Meth::Trim, 0, 0),
        "startsWith" => (Meth::StartsWith, 1, 1),
        "endsWith" => (Meth::EndsWith, 1, 1),
        "slice" => (Meth::Slice, 1, 2),
        "replace" => (Meth::Replace, 2, 2),
        "split" => (Meth::Split, 1, 2),
        "reverse" => (Meth::Reverse, 0, 0),
        "round" => (Meth::Round, 0, 1),
        "floor" => (Meth::Floor, 0, 0),
        "ceil" => (Meth::Ceil, 0, 0),
        "abs" => (Meth::Abs, 0, 0),
        "date" => (Meth::Date, 0, 0),
        "time" => (Meth::Time, 0, 0),
        "format" => (Meth::Format, 1, 1),
        "join" => (Meth::Join, 1, 1),
        "unique" => (Meth::Unique, 0, 0),
        "sort" => (Meth::Sort, 0, 0),
        "flat" => (Meth::Flat, 0, 0),
        _ => return None,
    })
}

/// 括弧の無いメソッド。
fn field_spec(name: &str) -> Option<Field> {
    if !METHODS.contains(&name) {
        return None;
    }
    Some(match name {
        "length" => Field::Length,
        "year" => Field::Year,
        "month" => Field::Month,
        "day" => Field::Day,
        "hour" => Field::Hour,
        "minute" => Field::Minute,
        "second" => Field::Second,
        _ => return None,
    })
}

/// `file` の関数(種類, 引数の最少, 最多)。
fn file_func_spec(name: &str) -> Option<(Func, usize, usize)> {
    if !FILE_FIELDS.contains(&name) {
        return None;
    }
    Some(match name {
        "hasTag" => (Func::HasTag, 1, usize::MAX),
        "hasLink" => (Func::HasLink, 1, 1),
        "inFolder" => (Func::InFolder, 1, 1),
        "hasProperty" => (Func::HasProperty, 1, 1),
        _ => return None,
    })
}

fn file_field(name: &str) -> Option<FileField> {
    if !FILE_FIELDS.contains(&name) {
        return None;
    }
    Some(match name {
        "name" => FileField::Name,
        "basename" => FileField::Basename,
        "ext" => FileField::Ext,
        "path" => FileField::Path,
        "folder" => FileField::Folder,
        "size" => FileField::Size,
        "mtime" => FileField::Mtime,
        "ctime" => FileField::Ctime,
        "tags" => FileField::Tags,
        "links" => FileField::Links,
        "backlinks" => FileField::Backlinks,
        _ => return None,
    })
}

fn deeper(d: usize) -> Result<usize, String> {
    if d >= MAX_DEPTH {
        Err(Msg::ExprTooDeep.text().to_string())
    } else {
        Ok(d + 1)
    }
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn peek_at(&self, k: usize) -> Option<&Tok> {
        self.toks.get(self.pos + k)
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &Tok, what: &str) -> Result<(), String> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(Msg::ExprMissing.fill(&[&what]))
        }
    }

    fn unsupported(&mut self, name: String) -> (Node, usize) {
        if self.unsupported.is_none() {
            self.unsupported = Some(name);
        }
        (Node::Bad, 1)
    }

    fn enter(&mut self) -> Result<(), String> {
        self.nest += 1;
        if self.nest > MAX_DEPTH {
            Err(Msg::ExprTooDeep.text().to_string())
        } else {
            Ok(())
        }
    }

    fn expr(&mut self) -> Pr {
        self.enter()?;
        let r = self.binary(0);
        self.nest -= 1;
        r
    }

    /// 優先順位で結ぶ(0 = ||、5 = * / %)。左から結ぶ。段ごとに関数を重ねず、スタックを浅く保つ。
    fn binary(&mut self, min: u8) -> Pr {
        let (mut l, mut dl) = self.unary()?;
        loop {
            let (op, prec) = match self.peek() {
                Some(Tok::Or) => (BinOp::Or, 0),
                Some(Tok::And) => (BinOp::And, 1),
                Some(Tok::Eq) => (BinOp::Eq, 2),
                Some(Tok::Ne) => (BinOp::Ne, 2),
                Some(Tok::Lt) => (BinOp::Lt, 3),
                Some(Tok::Le) => (BinOp::Le, 3),
                Some(Tok::Gt) => (BinOp::Gt, 3),
                Some(Tok::Ge) => (BinOp::Ge, 3),
                Some(Tok::Plus) => (BinOp::Add, 4),
                Some(Tok::Minus) => (BinOp::Sub, 4),
                Some(Tok::Star) => (BinOp::Mul, 5),
                Some(Tok::Slash) => (BinOp::Div, 5),
                Some(Tok::Percent) => (BinOp::Rem, 5),
                _ => return Ok((l, dl)),
            };
            if prec < min {
                return Ok((l, dl));
            }
            self.pos += 1;
            // 右は1段強い演算子だけを取る(左から結ぶ)。右の入れ子は高々5段。
            let (r, dr) = self.binary(prec + 1)?;
            dl = deeper(dl.max(dr))?;
            l = Node::Bin(op, Box::new(l), Box::new(r));
        }
    }

    fn unary(&mut self) -> Pr {
        let neg = match self.peek() {
            Some(Tok::Not) => false,
            Some(Tok::Minus) => true,
            Some(Tok::Plus) => {
                self.pos += 1;
                self.enter()?;
                let r = self.unary();
                self.nest -= 1;
                return r;
            }
            _ => return self.postfix(),
        };
        self.pos += 1;
        self.enter()?;
        let r = self.unary();
        self.nest -= 1;
        let (n, d) = r?;
        let d = deeper(d)?;
        Ok((
            if neg {
                Node::Neg(Box::new(n))
            } else {
                Node::Not(Box::new(n))
            },
            d,
        ))
    }

    /// `(` の後ろの引数の並び(`)` まで)。深さは引数の最大。
    fn args(&mut self) -> Result<(Vec<Node>, usize), String> {
        let mut v = Vec::new();
        let mut d = 0;
        if self.eat(&Tok::RParen) {
            return Ok((v, d));
        }
        loop {
            let (n, dn) = self.expr()?;
            v.push(n);
            d = d.max(dn);
            if self.eat(&Tok::RParen) {
                return Ok((v, d));
            }
            self.expect(&Tok::Comma, Msg::ExprCommaOrRParen.text())?;
        }
    }

    fn postfix(&mut self) -> Pr {
        let (mut n, mut d) = self.primary()?;
        loop {
            if self.eat(&Tok::Dot) {
                let Some(Tok::Ident(name)) = self.peek().cloned() else {
                    return Err(Msg::ExprNoNameAfterDot.text().to_string());
                };
                self.pos += 1;
                if self.eat(&Tok::LParen) {
                    let (args, da) = self.args()?;
                    match meth_spec(&name) {
                        Some((m, lo, hi)) => {
                            arity(&name, args.len(), lo, hi)?;
                            d = deeper(d.max(da))?;
                            n = Node::Method(Box::new(n), m, args);
                        }
                        None => (n, d) = self.unsupported(format!(".{name}()")),
                    }
                } else if let Some(f) = field_spec(&name) {
                    d = deeper(d)?;
                    n = Node::Field(Box::new(n), f);
                } else {
                    (n, d) = self.unsupported(format!(".{name}"));
                }
            } else if self.eat(&Tok::LBrack) {
                let (i, di) = self.expr()?;
                self.expect(&Tok::RBrack, "`]`")?;
                d = deeper(d.max(di))?;
                n = Node::Index(Box::new(n), Box::new(i));
            } else {
                return Ok((n, d));
            }
        }
    }

    /// `note`・`file`・`formula` の後ろの `.name` か `["name"]`。無ければ None。
    fn member(&mut self) -> Result<Option<String>, String> {
        if self.eat(&Tok::Dot) {
            match self.peek().cloned() {
                Some(Tok::Ident(name)) => {
                    self.pos += 1;
                    Ok(Some(name))
                }
                _ => Err(Msg::ExprNoNameAfterDot.text().to_string()),
            }
        } else if self.peek() == Some(&Tok::LBrack) {
            if let (Some(Tok::Str(s)), Some(Tok::RBrack)) = (self.peek_at(1), self.peek_at(2)) {
                let s = s.clone();
                self.pos += 3;
                Ok(Some(s))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn primary(&mut self) -> Pr {
        let Some(tok) = self.peek().cloned() else {
            return Err(Msg::ExprUnexpectedEnd.text().to_string());
        };
        self.pos += 1;
        match tok {
            Tok::Num(n) => Ok((Node::Lit(num(n)), 1)),
            Tok::Str(s) => Ok((Node::Lit(Val::Str(s)), 1)),
            Tok::LParen => {
                let r = self.expr()?;
                self.expect(&Tok::RParen, "`)`")?;
                Ok(r)
            }
            Tok::LBrack => {
                let mut v = Vec::new();
                let mut d = 0;
                if !self.eat(&Tok::RBrack) {
                    loop {
                        let (n, dn) = self.expr()?;
                        v.push(n);
                        d = d.max(dn);
                        if self.eat(&Tok::RBrack) {
                            break;
                        }
                        self.expect(&Tok::Comma, Msg::ExprCommaOrRBrack.text())?;
                    }
                }
                Ok((Node::List(v), deeper(d)?))
            }
            Tok::Ident(name) => self.ident(name),
            other => Err(Msg::ExprUnexpectedToken.fill(&[&format!("{other:?}")])),
        }
    }

    fn ident(&mut self, name: String) -> Pr {
        match name.as_str() {
            "true" => return Ok((Node::Lit(Val::Bool(true)), 1)),
            "false" => return Ok((Node::Lit(Val::Bool(false)), 1)),
            "null" => return Ok((Node::Lit(Val::Null), 1)),
            _ => {}
        }
        if self.eat(&Tok::LParen) {
            let (args, da) = self.args()?;
            return match func_spec(&name) {
                Some((f, lo, hi)) => {
                    arity(&name, args.len(), lo, hi)?;
                    Ok((Node::Call(f, args), deeper(da)?))
                }
                None => Ok(self.unsupported(format!("{name}()"))),
            };
        }
        match name.as_str() {
            "note" => match self.member()? {
                Some(k) => Ok((Node::Prop(k), 1)),
                None => Ok(self.unsupported("note".to_string())),
            },
            "formula" => match self.member()? {
                Some(k) => Ok((Node::Formula(k), 1)),
                None => Ok(self.unsupported("formula".to_string())),
            },
            "file" => {
                let Some(k) = self.member()? else {
                    return Ok(self.unsupported("file".to_string()));
                };
                if self.eat(&Tok::LParen) {
                    let (args, da) = self.args()?;
                    return match file_func_spec(&k) {
                        Some((f, lo, hi)) => {
                            arity(&format!("file.{k}"), args.len(), lo, hi)?;
                            Ok((Node::Call(f, args), deeper(da)?))
                        }
                        None => Ok(self.unsupported(format!("file.{k}()"))),
                    };
                }
                match file_field(&k) {
                    Some(f) => Ok((Node::File(f), 1)),
                    // embeds・properties・file など、計算できない項目(BV-7)
                    None => Ok(self.unsupported(format!("file.{k}"))),
                }
            }
            // BV-22: `.base` を直接開いたときの this は、その `.base` のファイル。`this.file` と `this.file.x` だけ
            // (リンクは .base に無い)。ほかの `this.x` は未対応。
            "this" => {
                if self.member()?.as_deref() != Some("file") {
                    return Ok(self.unsupported("this".to_string()));
                }
                let Some(k) = self.member()? else {
                    return Ok((Node::This(None), 1));
                };
                if self.peek() == Some(&Tok::LParen) {
                    return Ok(self.unsupported(format!("this.file.{k}()")));
                }
                match file_field(&k) {
                    Some(FileField::Links | FileField::Backlinks) | None => {
                        Ok(self.unsupported(format!("this.file.{k}")))
                    }
                    Some(f) => Ok((Node::This(Some(f)), 1)),
                }
            }
            _ => Ok((Node::Prop(name), 1)),
        }
    }
}

fn arity(name: &str, n: usize, lo: usize, hi: usize) -> Result<(), String> {
    if n < lo || n > hi {
        let want = if lo == hi {
            format!("{lo}")
        } else if hi == usize::MAX {
            Msg::ExprArityAtLeast.fill(&[&lo])
        } else {
            Msg::ExprArityRange.fill(&[&lo, &hi])
        };
        Err(Msg::ExprArity.fill(&[&name, &n, &want]))
    } else {
        Ok(())
    }
}

/// 字句解析 → 構文木。未対応の関数・メソッド・項目は Unsupported(BV-7)。文法の誤りが先。
pub fn parse(src: &str) -> Result<Expr, ExprError> {
    let toks = lex(src).map_err(ExprError::Syntax)?;
    let mut p = Parser {
        toks,
        pos: 0,
        nest: 0,
        unsupported: None,
    };
    let (node, _) = p.expr().map_err(ExprError::Syntax)?;
    if p.pos != p.toks.len() {
        return Err(ExprError::Syntax(
            Msg::ExprTrailing.fill(&[&format!("{:?}", p.toks[p.pos])]),
        ));
    }
    if let Some(name) = p.unsupported {
        return Err(ExprError::Unsupported(name));
    }
    Ok(Expr { node })
}

// ---- 日付と期間 ----

fn day_ok(d: i64) -> Option<i64> {
    (MIN_DAY..=MAX_DAY).contains(&d).then_some(d)
}

fn date_val(d: i64) -> Val {
    day_ok(d).map_or(Val::Null, Val::Date)
}

fn datetime_val(s: i64) -> Val {
    match day_ok(s.div_euclid(DAY_SECS)) {
        Some(_) => Val::DateTime(s),
        None => Val::Null,
    }
}

/// 日数 → (年, 月, 日)。範囲の中だけ。
fn civil(d: i64) -> Option<(i64, u32, u32)> {
    let d = day_ok(d)?;
    let s = format_date(d);
    let y = s.get(..4)?.parse().ok()?;
    let m = s.get(5..7)?.parse().ok()?;
    let dd = s.get(8..10)?.parse().ok()?;
    Some((y, m, dd))
}

fn add_months(d: i64, months: i64) -> Option<i64> {
    if months == 0 {
        return day_ok(d);
    }
    let (y, m, dd) = civil(d)?;
    let total = y
        .checked_mul(12)?
        .checked_add(i64::from(m) - 1)?
        .checked_add(months)?;
    let ny = total.div_euclid(12);
    let nm = total.rem_euclid(12) + 1;
    if !(0..=9999).contains(&ny) {
        return None;
    }
    // 月の終わりを越える日は、その月の最後の日にする。
    (28..=dd.max(28))
        .rev()
        .find_map(|day| parse_date(&format!("{ny:04}-{nm:02}-{:02}", day.min(dd))))
}

/// `YYYY-MM-DD` → Date、`YYYY-MM-DDTHH:MM(:SS(.fff))(Z)`(T の代わりに空白も可)→ DateTime。
fn parse_when(s: &str) -> Option<Val> {
    let s = s.trim();
    if let Some(d) = parse_date(s) {
        return Some(Val::Date(day_ok(d)?));
    }
    // C-1・C-2: 時差(`Z`・`±HH:MM`)つきは地域の時刻に直す(types と同じ読み方)。
    if let Some(t) = crate::types::parse_datetime(s) {
        day_ok(t.div_euclid(DAY_SECS))?;
        return Some(Val::DateTime(t));
    }
    let day = parse_date(s.get(..10)?)?;
    let rest = s.get(10..)?;
    let rest = rest.strip_prefix('T').or_else(|| rest.strip_prefix(' '))?;
    let rest = rest.strip_suffix('Z').unwrap_or(rest);
    let b = rest.as_bytes();
    if b.len() < 5 || b[2] != b':' {
        return None;
    }
    let num = |x: &str| -> Option<i64> {
        (x.len() == 2 && x.bytes().all(|c| c.is_ascii_digit()))
            .then(|| x.parse().ok())
            .flatten()
    };
    let h = num(rest.get(..2)?)?;
    let mi = num(rest.get(3..5)?)?;
    let sec = match rest.get(5..)? {
        "" => 0,
        r => {
            let r = r.strip_prefix(':')?;
            let (w, f) = r.split_once('.').unwrap_or((r, "0"));
            if f.is_empty() || !f.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
            num(w)?
        }
    };
    if h > 23 || mi > 59 || sec > 59 {
        return None;
    }
    Some(Val::DateTime(
        day_ok(day)? * DAY_SECS + h * 3600 + mi * 60 + sec,
    ))
}

/// 期間の文字列("1d"・"2 weeks"・"1d 3h"・"-1 day")。月と年は暦で足すので別に持つ。
#[derive(Clone, Copy, Debug, PartialEq)]
struct Dur {
    months: i64,
    ms: i64,
}

fn parse_dur(s: &str) -> Option<Dur> {
    let s = s.trim();
    let (neg, mut rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let mut dur = Dur { months: 0, ms: 0 };
    let mut any = false;
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let nd = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        if nd == 0 {
            return None;
        }
        let n: i64 = rest[..nd].parse().ok()?;
        rest = rest[nd..].trim_start();
        let nu = rest
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(rest.len());
        if nu == 0 {
            return None;
        }
        let unit = &rest[..nu];
        rest = &rest[nu..];
        let lower = unit.to_ascii_lowercase();
        let (months, ms) = match (unit, lower.as_str()) {
            ("M", _) => (1, 0),
            ("m", _) => (0, 60_000),
            (_, "y" | "yr" | "yrs" | "year" | "years") => (12, 0),
            (_, "month" | "months") => (1, 0),
            (_, "w" | "week" | "weeks") => (0, 7 * DAY_MS),
            (_, "d" | "day" | "days") => (0, DAY_MS),
            (_, "h" | "hr" | "hrs" | "hour" | "hours") => (0, 3_600_000),
            (_, "min" | "mins" | "minute" | "minutes") => (0, 60_000),
            (_, "s" | "sec" | "secs" | "second" | "seconds") => (0, 1000),
            _ => return None,
        };
        dur.months = dur.months.checked_add(n.checked_mul(months)?)?;
        dur.ms = dur.ms.checked_add(n.checked_mul(ms)?)?;
        any = true;
    }
    if !any {
        return None;
    }
    if neg {
        dur.months = dur.months.checked_neg()?;
        dur.ms = dur.ms.checked_neg()?;
    }
    Some(dur)
}

/// 期間の文字列をミリ秒に(月は 30 日、年は 365 日として。duration() と Duration の値のため)。
fn dur_ms(d: Dur) -> Option<i64> {
    let years = d.months / 12;
    let months = d.months % 12;
    years
        .checked_mul(365 * DAY_MS)?
        .checked_add(months.checked_mul(30 * DAY_MS)?)?
        .checked_add(d.ms)
}

/// 日付・日時 + 期間。
fn shift(base: &Val, d: Dur) -> Val {
    let r = match *base {
        Val::Date(day) => shift_date(day, d),
        Val::DateTime(s) => shift_time(s, d),
        _ => None,
    };
    r.unwrap_or(Val::Null)
}

fn shift_date(day: i64, d: Dur) -> Option<Val> {
    let day = add_months(day, d.months)?;
    if d.ms % DAY_MS == 0 {
        Some(date_val(day.checked_add(d.ms / DAY_MS)?))
    } else {
        let s = day.checked_mul(DAY_SECS)?.checked_add(d.ms / 1000)?;
        Some(datetime_val(s))
    }
}

fn shift_time(s: i64, d: Dur) -> Option<Val> {
    let day = add_months(s.div_euclid(DAY_SECS), d.months)?;
    let s = day
        .checked_mul(DAY_SECS)?
        .checked_add(s.rem_euclid(DAY_SECS))?
        .checked_add(d.ms / 1000)?;
    Some(datetime_val(s))
}

/// 日時 − 日時 のミリ秒。
fn diff_ms(x: &Val, y: &Val) -> Option<i64> {
    secs_of(x)?.checked_sub(secs_of(y)?)?.checked_mul(1000)
}

fn secs_of(v: &Val) -> Option<i64> {
    match *v {
        Val::Date(d) => d.checked_mul(DAY_SECS),
        Val::DateTime(s) => Some(s),
        _ => None,
    }
}

fn is_when(v: &Val) -> bool {
    matches!(v, Val::Date(_) | Val::DateTime(_))
}

/// 片方が日付・日時なら、もう片方の文字列を日付として読む。
fn coerce_pair(a: Val, b: Val) -> (Val, Val) {
    match (&a, &b) {
        (x, Val::Str(s)) if is_when(x) => {
            let b2 = parse_when(s).unwrap_or(b);
            (a, b2)
        }
        (Val::Str(s), y) if is_when(y) => {
            let a2 = parse_when(s).unwrap_or(a);
            (a2, b)
        }
        _ => (a, b),
    }
}

// ---- 値の変換 ----

/// フロントマターの値 → Val。`YYYY-MM-DD` の文字列は Date、`YYYY-MM-DDTHH:MM(:SS)` は DateTime。
pub fn to_val(v: &Value) -> Val {
    match v {
        Value::Null | Value::Other => Val::Null,
        Value::Bool(b) => Val::Bool(*b),
        Value::Int(n) => Val::Num(*n as f64),
        Value::Float(f) => num(*f),
        Value::Str(s) => parse_when(s).unwrap_or_else(|| Val::Str(s.clone())),
        Value::List(xs) => Val::List(xs.iter().map(to_val).collect()),
    }
}

/// 有限でない数(0 での割り算・あふれ)は Null。
fn num(f: f64) -> Val {
    if f.is_finite() {
        Val::Num(f)
    } else {
        Val::Null
    }
}

fn fmt_num(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

fn fmt_time(s: i64) -> String {
    let day = s.div_euclid(DAY_SECS);
    let t = s.rem_euclid(DAY_SECS);
    format!(
        "{}T{:02}:{:02}:{:02}",
        format_date(day),
        t / 3600,
        t / 60 % 60,
        t % 60
    )
}

fn to_text(v: &Val) -> String {
    match v {
        Val::Null => String::new(),
        Val::Bool(b) => b.to_string(),
        Val::Num(f) => fmt_num(*f),
        Val::Str(s) => s.clone(),
        Val::Date(d) => match day_ok(*d) {
            Some(d) => format_date(d),
            None => String::new(),
        },
        Val::DateTime(s) => match day_ok(s.div_euclid(DAY_SECS)) {
            Some(_) => fmt_time(*s),
            None => String::new(),
        },
        Val::Duration(ms) => ms.to_string(),
        Val::List(xs) => xs.iter().map(to_text).collect::<Vec<_>>().join(", "),
    }
}

fn truthy(v: &Val) -> bool {
    match v {
        Val::Null => false,
        Val::Bool(b) => *b,
        Val::Num(f) => *f != 0.0 && !f.is_nan(),
        Val::Str(s) => !s.is_empty(),
        _ => true,
    }
}

// ---- 比較 ----

fn equal(a: &Val, b: &Val) -> bool {
    match (a, b) {
        (Val::Null, Val::Null) => true,
        (Val::Bool(x), Val::Bool(y)) => x == y,
        (Val::Num(x), Val::Num(y)) => x == y,
        (Val::Str(x), Val::Str(y)) => x == y,
        (Val::Duration(x), Val::Duration(y)) => x == y,
        (Val::List(x), Val::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| equal(p, q))
        }
        (Val::Date(x), Val::Date(y)) => x == y,
        (x, y) if is_when(x) && is_when(y) => secs_of(x).is_some() && secs_of(x) == secs_of(y),
        (x, Val::Str(s)) | (Val::Str(s), x) if is_when(x) => {
            parse_when(s).is_some_and(|w| equal(x, &w))
        }
        _ => false,
    }
}

fn order(a: &Val, b: &Val) -> Option<Ordering> {
    match (a, b) {
        (Val::Num(x), Val::Num(y)) => x.partial_cmp(y),
        (Val::Str(x), Val::Str(y)) => Some(x.cmp(y)),
        (Val::Bool(x), Val::Bool(y)) => Some(x.cmp(y)),
        (Val::Duration(x), Val::Duration(y)) => Some(x.cmp(y)),
        (Val::Duration(x), Val::Num(y)) => (*x as f64).partial_cmp(y),
        (Val::Num(x), Val::Duration(y)) => x.partial_cmp(&(*y as f64)),
        (Val::Date(x), Val::Date(y)) => Some(x.cmp(y)),
        (x, y) if is_when(x) && is_when(y) => Some(secs_of(x)?.cmp(&secs_of(y)?)),
        _ => None,
    }
}

// ---- 算術 ----

fn arith(op: BinOp, a: Val, b: Val) -> Val {
    let (a, b) = coerce_pair(a, b);
    match (op, &a, &b) {
        (_, Val::Num(x), Val::Num(y)) => num(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            _ => x % y,
        }),
        // 期間を左にした * / は期間(Obsidian のヘルプ: duration('1d') * 2)。ミリ秒は丸める。
        (BinOp::Mul | BinOp::Div, Val::Duration(x), Val::Num(y)) => {
            let ms = if op == BinOp::Mul {
                *x as f64 * y
            } else {
                *x as f64 / y
            }
            .round();
            // i64 に収まる有限の値だけ(2^63 は f64 で正確に表せる)。
            if ms.is_finite() && ms.abs() < 9_223_372_036_854_775_808.0 {
                Val::Duration(ms as i64)
            } else {
                Val::Null
            }
        }
        // ほかは期間をミリ秒の数として使う(Obsidian のヘルプ: 日付の差はミリ秒)。
        (_, Val::Duration(x), Val::Num(y)) => arith(op, Val::Num(*x as f64), Val::Num(*y)),
        (_, Val::Num(x), Val::Duration(y)) => arith(op, Val::Num(*x), Val::Num(*y as f64)),
        (BinOp::Add, Val::Duration(x), Val::Duration(y)) => {
            x.checked_add(*y).map_or(Val::Null, Val::Duration)
        }
        (BinOp::Sub, Val::Duration(x), Val::Duration(y)) => {
            x.checked_sub(*y).map_or(Val::Null, Val::Duration)
        }
        (BinOp::Add | BinOp::Sub, w, Val::Str(s)) if is_when(w) => match parse_dur(s) {
            Some(d) if op == BinOp::Add => shift(w, d),
            Some(d) => match (d.months.checked_neg(), d.ms.checked_neg()) {
                (Some(months), Some(ms)) => shift(w, Dur { months, ms }),
                _ => Val::Null,
            },
            None => Val::Null,
        },
        (BinOp::Add | BinOp::Sub, w, Val::Duration(ms)) if is_when(w) => {
            let ms = if op == BinOp::Add {
                Some(*ms)
            } else {
                ms.checked_neg()
            };
            ms.map_or(Val::Null, |ms| shift(w, Dur { months: 0, ms }))
        }
        (BinOp::Add, Val::Duration(_), w) if is_when(w) => arith(op, b, a),
        (BinOp::Sub, Val::Date(x), Val::Date(y)) => x
            .checked_sub(*y)
            .and_then(|d| d.checked_mul(DAY_MS))
            .map_or(Val::Null, Val::Duration),
        (BinOp::Sub, x, y) if is_when(x) && is_when(y) => {
            diff_ms(x, y).map_or(Val::Null, Val::Duration)
        }
        (BinOp::Add, Val::Str(x), Val::Str(y)) if x.len() + y.len() > MAX_STR => Val::Null,
        (BinOp::Add, Val::Str(x), Val::Str(y)) => Val::Str(format!("{x}{y}")),
        (BinOp::Add, Val::Str(x), Val::Num(y)) => Val::Str(format!("{x}{}", fmt_num(*y))),
        (BinOp::Add, Val::Num(x), Val::Str(y)) => Val::Str(format!("{}{y}", fmt_num(*x))),
        _ => Val::Null,
    }
}

/// 値が1つに決まる簡単な形の項(new-note。CE-25)。
#[derive(Clone, Debug, PartialEq)]
pub enum Fixed {
    /// `列 == 値`(値は文字列・数・真偽)。
    Eq(String, Val),
    /// `列.contains("値")`。
    Contains(String, String),
    /// `file.hasTag("x")`(引数1つ。先頭の `#` は除く)。Obsidian と同じく tags の列のタグ。
    Tag(String),
    /// `file.inFolder("x")`(保管庫の根からのフォルダ。前後の `/` は除く。空は入れない)。
    Folder(String),
}

impl Expr {
    /// 式を `&&` で分けた項のうち、`列 == 値`(`値 == 列` も)・`列.contains("値")`・`file.hasTag("x")` の形のもの。
    /// 列はノートのキー(`note.x`・`note["x"]` も)だけで、`file.*`・`formula.*` は入れない。
    /// ほかの形の項(`||`・`!=`・比較・関数など)は入れない(式全体がそれだけなら空)。
    /// `this` を使うか(BV-22。this の無いところでは未対応にする)。
    pub fn uses_this(&self) -> bool {
        uses_this(&self.node)
    }

    pub fn fixed_values(&self) -> Vec<Fixed> {
        let mut out = Vec::new();
        collect_fixed(&self.node, &mut out);
        out
    }
}

fn uses_this(n: &Node) -> bool {
    match n {
        Node::This(_) => true,
        Node::Lit(_) | Node::Prop(_) | Node::File(_) | Node::Formula(_) | Node::Bad => false,
        Node::Not(x) | Node::Neg(x) | Node::Field(x, _) => uses_this(x),
        Node::Bin(_, a, b) | Node::Index(a, b) => uses_this(a) || uses_this(b),
        Node::List(xs) | Node::Call(_, xs) => xs.iter().any(uses_this),
        Node::Method(r, _, xs) => uses_this(r) || xs.iter().any(uses_this),
    }
}

fn collect_fixed(n: &Node, out: &mut Vec<Fixed>) {
    let lit = |n: &Node| match n {
        Node::Lit(v @ (Val::Str(_) | Val::Num(_) | Val::Bool(_))) => Some(v.clone()),
        _ => None,
    };
    match n {
        Node::Bin(BinOp::And, a, b) => {
            collect_fixed(a, out);
            collect_fixed(b, out);
        }
        Node::Bin(BinOp::Eq, a, b) => match (&**a, &**b) {
            (Node::Prop(k), v) | (v, Node::Prop(k)) => {
                if let Some(v) = lit(v) {
                    out.push(Fixed::Eq(k.clone(), v));
                }
            }
            _ => {}
        },
        Node::Method(recv, Meth::Contains, args) => {
            if let (Node::Prop(k), [Node::Lit(Val::Str(s))]) = (&**recv, args.as_slice()) {
                out.push(Fixed::Contains(k.clone(), s.clone()));
            }
        }
        Node::Call(Func::HasTag, args) => {
            if let [Node::Lit(Val::Str(s))] = args.as_slice() {
                let t = s.trim().trim_start_matches('#');
                if !t.is_empty() {
                    out.push(Fixed::Tag(t.to_string()));
                }
            }
        }
        Node::Call(Func::InFolder, args) => {
            if let [Node::Lit(Val::Str(s))] = args.as_slice() {
                let f = s.trim().trim_matches('/');
                if !f.is_empty() {
                    out.push(Fixed::Folder(f.to_string()));
                }
            }
        }
        _ => {}
    }
}

// ---- 評価 ----

/// 評価。型の合わない演算は Null。filters は Val::Bool(true) のときだけ行を残す。
pub fn eval(e: &Expr, env: &Env) -> Val {
    eval_with(e, env, &Links::default())
}

/// リンクの索引と this を添えた評価(BV-22)。`eval` はどちらも無いもの(`file.links` などは Null)。
pub fn eval_with(e: &Expr, env: &Env, links: &Links) -> Val {
    ev(&e.node, &Cx { env, links })
}

/// 評価の中で持ち回る環境。
struct Cx<'a> {
    env: &'a Env<'a>,
    links: &'a Links<'a>,
}

/// 文字の要素のリスト。重さが上限(MAX_STR)を超えるなら Null(BV-7)。
fn str_list(items: &[String]) -> Val {
    let w = items
        .iter()
        .map(|s| 16 + s.len())
        .fold(16usize, usize::saturating_add);
    if w > MAX_STR {
        return Val::Null;
    }
    Val::List(items.iter().map(|s| Val::Str(s.clone())).collect())
}

fn link_val(f: FileField, fi: &FileInfo, cx: &Cx) -> Val {
    match (f, cx.links.index) {
        (FileField::Links, Some(ix)) => match cx.links.this {
            // `file.links.contains(this.file)`: .base を指すリンクは this.file と同じ値にする(BV-22)。
            Some(t) => str_list(&ix.links_toward(&fi.path, &t.path)),
            None => str_list(ix.links(&fi.path)),
        },
        (FileField::Backlinks, Some(ix)) => str_list(ix.backlinks(&fi.path)),
        (FileField::Links | FileField::Backlinks, None) => Val::Null,
        _ => file_val(f, fi),
    }
}

fn ev(n: &Node, cx: &Cx) -> Val {
    match n {
        Node::Lit(v) => v.clone(),
        // 重さを足しながら作り、上限を超えたところでやめる(全部を作ってから測らない)。
        Node::List(xs) => {
            let mut out = Vec::with_capacity(xs.len());
            let mut w = 16usize;
            for x in xs {
                let v = ev(x, cx);
                w = w.saturating_add(weight(&v));
                if w > MAX_STR {
                    return Val::Null;
                }
                out.push(v);
            }
            Val::List(out)
        }
        Node::Prop(k) => (cx.env.prop)(k).map_or(Val::Null, |v| to_val(&v)),
        Node::Formula(k) => (cx.env.formula)(k).unwrap_or(Val::Null),
        Node::File(f) => link_val(*f, cx.env.file, cx),
        Node::This(f) => match (cx.links.this, f) {
            (Some(t), None) => Val::Str(t.path.clone()),
            (Some(t), Some(f)) => file_val(*f, t),
            (None, _) => Val::Null,
        },
        Node::Not(x) => Val::Bool(!truthy(&ev(x, cx))),
        Node::Neg(x) => match ev(x, cx) {
            Val::Num(f) => Val::Num(-f),
            Val::Duration(ms) => ms.checked_neg().map_or(Val::Null, Val::Duration),
            _ => Val::Null,
        },
        Node::Bin(BinOp::And, a, b) => Val::Bool(truthy(&ev(a, cx)) && truthy(&ev(b, cx))),
        Node::Bin(BinOp::Or, a, b) => Val::Bool(truthy(&ev(a, cx)) || truthy(&ev(b, cx))),
        Node::Bin(op, a, b) => {
            let (a, b) = (ev(a, cx), ev(b, cx));
            match op {
                BinOp::Eq | BinOp::Ne => {
                    let eq = equal(&a, &b);
                    Val::Bool(if *op == BinOp::Eq { eq } else { !eq })
                }
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    let (a, b) = coerce_pair(a, b);
                    match order(&a, &b) {
                        Some(o) => Val::Bool(match op {
                            BinOp::Lt => o.is_lt(),
                            BinOp::Le => o.is_le(),
                            BinOp::Gt => o.is_gt(),
                            _ => o.is_ge(),
                        }),
                        None => Val::Null,
                    }
                }
                _ => arith(*op, a, b),
            }
        }
        Node::Call(f, args) => call(*f, args, cx),
        Node::Method(recv, m, args) => {
            let r = ev(recv, cx);
            let args: Vec<Val> = args.iter().map(|a| ev(a, cx)).collect();
            method(&r, *m, &args)
        }
        Node::Field(x, f) => field(&ev(x, cx), *f),
        Node::Index(x, i) => match (ev(x, cx), ev(i, cx)) {
            (Val::List(xs), Val::Num(k)) if k >= 0.0 && k.fract() == 0.0 => {
                xs.get(k as usize).cloned().unwrap_or(Val::Null)
            }
            _ => Val::Null,
        },
        Node::Bad => Val::Null,
    }
}

fn file_val(f: FileField, fi: &FileInfo) -> Val {
    match f {
        FileField::Name => Val::Str(fi.name.clone()),
        FileField::Basename => Val::Str(fi.basename.clone()),
        FileField::Ext => Val::Str(fi.ext.clone()),
        FileField::Path => Val::Str(fi.path.clone()),
        FileField::Folder => Val::Str(fi.folder.clone()),
        FileField::Size => Val::Num(fi.size as f64),
        // 式の中の日時は地域の時計の秒(now()・today()・フロントマターの日時と同じ物差し。local-today)。
        FileField::Mtime => datetime_val(fi.mtime + crate::clock::local_offset()),
        FileField::Ctime => datetime_val(fi.ctime + crate::clock::local_offset()),
        FileField::Tags => Val::List(fi.tags.iter().map(|t| Val::Str(t.clone())).collect()),
        // 索引が要る(link_val)。
        FileField::Links | FileField::Backlinks => Val::Null,
    }
}

fn call(f: Func, args: &[Node], cx: &Cx) -> Val {
    let arg = |i: usize| args.get(i).map_or(Val::Null, |a| ev(a, cx));
    match f {
        Func::If => {
            if truthy(&arg(0)) {
                arg(1)
            } else {
                arg(2)
            }
        }
        Func::Now => datetime_val(cx.env.now),
        Func::Today => date_val(cx.env.today),
        Func::Date => match arg(0) {
            Val::Str(s) => parse_when(&s).unwrap_or(Val::Null),
            v @ (Val::Date(_) | Val::DateTime(_)) => v,
            _ => Val::Null,
        },
        // Obsidian の number(input): 期間はミリ秒、日付・日時は 1970 からのミリ秒、真偽は 1・0、数として読める文字は数。
        Func::Number => match arg(0) {
            Val::Num(n) => Val::Num(n),
            Val::Bool(b) => Val::Num(if b { 1.0 } else { 0.0 }),
            Val::Duration(ms) => Val::Num(ms as f64),
            Val::Date(d) => Val::Num(d as f64 * DAY_MS as f64),
            Val::DateTime(t) => Val::Num(t as f64 * 1000.0),
            Val::Str(t) => t
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|n| n.is_finite())
                .map_or(Val::Null, Val::Num),
            _ => Val::Null,
        },
        Func::Max | Func::Min => {
            let mut best: Option<f64> = None;
            for a in args {
                let Val::Num(x) = ev(a, cx) else {
                    return Val::Null;
                };
                best = Some(match best {
                    Some(b) if f == Func::Max => b.max(x),
                    Some(b) => b.min(x),
                    None => x,
                });
            }
            best.map_or(Val::Null, num)
        }
        Func::List => match arg(0) {
            v @ Val::List(_) => v,
            v => Val::List(vec![v]),
        },
        Func::Duration => match arg(0) {
            Val::Str(s) => parse_dur(&s)
                .and_then(dur_ms)
                .map_or(Val::Null, Val::Duration),
            v @ Val::Duration(_) => v,
            _ => Val::Null,
        },
        Func::HasTag => {
            let tags: Vec<String> = cx.env.file.tags.iter().map(|t| t.to_lowercase()).collect();
            let hit = args.iter().any(|a| match ev(a, cx) {
                Val::Str(t) => {
                    let t = t.trim_start_matches('#').to_lowercase();
                    !t.is_empty()
                        && tags.iter().any(|x| {
                            x == &t || x.strip_prefix(&t).is_some_and(|r| r.starts_with('/'))
                        })
                }
                _ => false,
            });
            Val::Bool(hit)
        }
        Func::HasLink => match (arg(0), cx.links.index) {
            (Val::Str(x), Some(ix)) => Val::Bool(ix.has_link(&cx.env.file.path, &x)),
            (Val::Str(_), None) => Val::Null,
            _ => Val::Null,
        },
        Func::InFolder => match arg(0) {
            Val::Str(f) => {
                let f = f.trim_matches('/');
                let folder = cx.env.file.folder.as_str();
                Val::Bool(
                    f.is_empty()
                        || folder == f
                        || folder.strip_prefix(f).is_some_and(|r| r.starts_with('/')),
                )
            }
            _ => Val::Null,
        },
        Func::HasProperty => match arg(0) {
            Val::Str(p) => Val::Bool((cx.env.prop)(&p).is_some()),
            _ => Val::Null,
        },
    }
}

fn field(v: &Val, f: Field) -> Val {
    match (f, v) {
        (Field::Length, Val::Str(s)) => Val::Num(s.chars().count() as f64),
        (Field::Length, Val::List(xs)) => Val::Num(xs.len() as f64),
        (Field::Length, _) => Val::Null,
        (_, w) => {
            let Some(secs) = secs_of(w) else {
                return Val::Null;
            };
            let Some((y, mo, d)) = civil(secs.div_euclid(DAY_SECS)) else {
                return Val::Null;
            };
            let t = secs.rem_euclid(DAY_SECS);
            Val::Num(match f {
                Field::Year => y as f64,
                Field::Month => f64::from(mo),
                Field::Day => f64::from(d),
                Field::Hour => (t / 3600) as f64,
                Field::Minute => (t / 60 % 60) as f64,
                _ => (t % 60) as f64,
            })
        }
    }
}

/// 引数 i の文字。
fn str_arg(args: &[Val], i: usize) -> Option<&str> {
    match args.get(i) {
        Some(Val::Str(s)) => Some(s),
        _ => None,
    }
}

/// 引数 i の数を整数に(0 の方へ切り捨てる)。数でなければ None。
fn int_arg(args: &[Val], i: usize) -> Option<i64> {
    match args.get(i) {
        Some(Val::Num(x)) => Some(x.trunc().clamp(-9e15, 9e15) as i64),
        _ => None,
    }
}

/// JavaScript の slice の範囲(負は後ろから数える)。
fn slice_range(len: usize, args: &[Val]) -> Option<(usize, usize)> {
    let len_i = len as i64;
    let at = |k: i64| -> usize {
        let k = if k < 0 {
            (len_i + k).max(0)
        } else {
            k.min(len_i)
        };
        k as usize
    };
    let start = at(int_arg(args, 0)?);
    let end = if args.len() > 1 {
        at(int_arg(args, 1)?)
    } else {
        len
    };
    Some((start, end.max(start)))
}

/// 文字列の新しいメソッド。当てはまらなければ None(呼ぶ側で Null)。
fn str_method(s: &str, m: Meth, args: &[Val]) -> Option<Val> {
    Some(match m {
        Meth::Upper => Val::Str(s.to_uppercase()),
        Meth::Title => Val::Str(title_case(s)),
        Meth::Trim => Val::Str(s.trim().to_string()),
        Meth::StartsWith => Val::Bool(s.starts_with(str_arg(args, 0)?)),
        Meth::EndsWith => Val::Bool(s.ends_with(str_arg(args, 0)?)),
        Meth::Slice => {
            let cs: Vec<char> = s.chars().collect();
            let (a, b) = slice_range(cs.len(), args)?;
            Val::Str(cs[a..b].iter().collect())
        }
        Meth::Replace => {
            let (pat, rep) = (str_arg(args, 0)?, str_arg(args, 1)?);
            // 作る前に長さを見積もる(空の pattern は各文字の間と両端に入る)。
            let hits = if pat.is_empty() {
                s.chars().count() + 1
            } else {
                s.matches(pat).count()
            };
            let len = hits
                .checked_mul(rep.len())
                .and_then(|n| n.checked_add(s.len()))?;
            if len > MAX_STR {
                return None;
            }
            Val::Str(s.replace(pat, rep))
        }
        Meth::Split => {
            let sep = str_arg(args, 0)?;
            let limit = if args.len() > 1 {
                usize::try_from(int_arg(args, 1)?.max(0)).ok()?
            } else {
                usize::MAX
            };
            // 作る前に重さを見積もる(要素の数 × 16 + 文字の長さ)。
            let count = if sep.is_empty() {
                s.chars().count()
            } else {
                s.matches(sep).count() + 1
            }
            .min(limit);
            if count.saturating_mul(32).saturating_add(s.len()) > MAX_STR {
                return None;
            }
            let parts: Vec<Val> = if sep.is_empty() {
                s.chars().map(|c| Val::Str(c.to_string())).collect()
            } else {
                s.split(sep).map(|p| Val::Str(p.to_string())).collect()
            };
            Val::List(parts.into_iter().take(limit).collect())
        }
        Meth::Reverse => Val::Str(s.chars().rev().collect()),
        _ => return None,
    })
}

/// 語ごとに先頭を大文字、残りを小文字にする(Obsidian の `title()`)。
fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut start = true;
    for c in s.chars() {
        if c.is_whitespace() {
            start = true;
            out.push(c);
        } else if start {
            out.extend(c.to_uppercase());
            start = false;
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// 数の新しいメソッド。
fn num_method(x: f64, m: Meth, args: &[Val]) -> Option<Val> {
    // JavaScript の Math.round(0.5 は上へ)。
    let js_round = |v: f64| (v + 0.5).floor();
    Some(match m {
        Meth::Round => match args.first() {
            None => num(js_round(x)),
            Some(Val::Num(d)) if d.fract() == 0.0 && (0.0..=15.0).contains(d) => {
                let p = 10f64.powi(*d as i32);
                num(js_round(x * p) / p)
            }
            Some(_) => return None,
        },
        Meth::Floor => num(x.floor()),
        Meth::Ceil => num(x.ceil()),
        Meth::Abs => num(x.abs()),
        _ => return None,
    })
}

/// 日付・日時の新しいメソッド。
fn when_method(w: &Val, m: Meth, args: &[Val]) -> Option<Val> {
    let secs = secs_of(w)?;
    let day = secs.div_euclid(DAY_SECS);
    let t = secs.rem_euclid(DAY_SECS);
    Some(match m {
        Meth::Date => date_val(day),
        Meth::Time => Val::Str(format!("{:02}:{:02}:{:02}", t / 3600, t / 60 % 60, t % 60)),
        Meth::Format => Val::Str(format_when(day, t, str_arg(args, 0)?)?),
        _ => return None,
    })
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Moment の書き方(英語の既定のロケール)。YYYY・YY・Q・MMMM・MMM・MM・M・DDDD・DDD・DD・Do・D・dddd・ddd・
/// dd・d・e・E・HH・H・hh・h・A・a・mm・ss・SSS・X・x・GGGG・WW・W(ISO の週)・gggg・ww・w(日曜始まりで、
/// 1月1日を含む週が第1週)。ほかの文字と `[...]` の中はそのまま(Moment と同じ)。曜日と月の名前は英語。
fn format_when(day: i64, t: i64, fmt: &str) -> Option<String> {
    let (y, mo, d) = civil(day)?;
    let month = MONTHS[(mo - 1) as usize];
    // 1970-01-01 は木曜日(0 = 日曜日)。
    let wd_sun = (day + 4).rem_euclid(7);
    let weekday = WEEKDAYS[wd_sun as usize];
    let iso_wd = (day + 3).rem_euclid(7) + 1;
    let (h, mi, s) = (t / 3600, t / 60 % 60, t % 60);
    let jan1 = |year: i64| parse_date(&format!("{year:04}-01-01"));
    // 週の年と週: `anchor`(ISO はその週の木曜日、日曜始まりは土曜日)の年の1月1日からの週。年 0 と
    // 9999 の端で暦の外に出たら、その日の年と第1週にする(週の記号を使わない書式まで null にしない)。
    let week_of = |anchor: i64| {
        civil(anchor)
            .and_then(|(wy, _, _)| Some((wy, (anchor - jan1(wy)?) / 7 + 1)))
            .unwrap_or((y, 1))
    };
    let (iso_year, iso_week) = week_of(day - iso_wd + 4);
    let (loc_year, loc_week) = week_of(day + 6 - wd_sun);
    let doy = jan1(y).map_or(1, |j| day - j + 1);
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    let ordinal = |n: i64| {
        let suffix = match (n % 100, n % 10) {
            (11..=13, _) => "th",
            (_, 1) => "st",
            (_, 2) => "nd",
            (_, 3) => "rd",
            _ => "th",
        };
        format!("{n}{suffix}")
    };
    let unix = day * DAY_SECS + t - crate::clock::local_offset();
    // 長い記号を先に(`DDDD` は `DD` より先、`Do` は `D` より先)。
    let tokens: [(&str, String); 35] = [
        ("YYYY", format!("{y:04}")),
        ("GGGG", format!("{iso_year:04}")),
        ("gggg", format!("{loc_year:04}")),
        ("MMMM", month.to_string()),
        ("dddd", weekday.to_string()),
        ("DDDD", format!("{doy:03}")),
        ("MMM", month[..3].to_string()),
        ("ddd", weekday[..3].to_string()),
        ("DDD", doy.to_string()),
        ("SSS", "000".to_string()),
        ("YY", format!("{:02}", y % 100)),
        ("MM", format!("{mo:02}")),
        ("DD", format!("{d:02}")),
        ("Do", ordinal(i64::from(d))),
        ("dd", weekday[..2].to_string()),
        ("HH", format!("{h:02}")),
        ("hh", format!("{h12:02}")),
        ("mm", format!("{mi:02}")),
        ("ss", format!("{s:02}")),
        ("WW", format!("{iso_week:02}")),
        ("ww", format!("{loc_week:02}")),
        ("Q", ((mo - 1) / 3 + 1).to_string()),
        ("M", mo.to_string()),
        ("D", d.to_string()),
        ("d", wd_sun.to_string()),
        ("e", wd_sun.to_string()),
        ("E", iso_wd.to_string()),
        ("H", h.to_string()),
        ("h", h12.to_string()),
        ("A", if h < 12 { "AM" } else { "PM" }.to_string()),
        ("a", if h < 12 { "am" } else { "pm" }.to_string()),
        ("W", iso_week.to_string()),
        ("w", loc_week.to_string()),
        ("X", unix.to_string()),
        ("x", (unix * 1000).to_string()),
    ];
    let cs: Vec<char> = fmt.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '[' {
            if let Some(k) = cs[i + 1..].iter().position(|&c| c == ']') {
                out.extend(&cs[i + 1..i + 1 + k]);
                i += k + 2;
                continue;
            }
        }
        let rest: String = cs[i..cs.len().min(i + 4)].iter().collect();
        match tokens.iter().find(|(k, _)| rest.starts_with(k)) {
            Some((k, v)) => {
                out.push_str(v);
                i += k.len();
            }
            None => {
                out.push(cs[i]);
                i += 1;
            }
        }
    }
    Some(out)
}

/// 型の違う値どうしも並べられる比べ方(同じ型どうしは order、ほかは型の順)。
fn sort_cmp(a: &Val, b: &Val) -> Ordering {
    let rank = |v: &Val| match v {
        Val::Null => 0,
        Val::Bool(_) => 1,
        Val::Num(_) => 2,
        Val::Duration(_) => 3,
        Val::Date(_) | Val::DateTime(_) => 4,
        Val::Str(_) => 5,
        Val::List(_) => 6,
    };
    order(a, b)
        .filter(|_| rank(a) == rank(b))
        .unwrap_or_else(|| rank(a).cmp(&rank(b)))
}

/// リストの新しいメソッド。
fn list_method(xs: &[Val], m: Meth, args: &[Val]) -> Option<Val> {
    Some(match m {
        Meth::Join => {
            let sep = str_arg(args, 0)?;
            let parts: Vec<String> = xs.iter().map(to_text).collect();
            let len: usize = parts.iter().map(String::len).sum::<usize>()
                + sep.len() * parts.len().saturating_sub(1);
            if len > MAX_STR {
                return None;
            }
            Val::Str(parts.join(sep))
        }
        Meth::Unique => {
            let mut out: Vec<Val> = Vec::new();
            for x in xs {
                if !out.iter().any(|y| equal(x, y)) {
                    out.push(x.clone());
                }
            }
            Val::List(out)
        }
        Meth::Sort => {
            let mut v = xs.to_vec();
            v.sort_by(sort_cmp);
            Val::List(v)
        }
        Meth::Reverse => Val::List(xs.iter().rev().cloned().collect()),
        // 1段だけ平らにする(JavaScript の flat() と同じ)。
        // 平らにしても重さは元のリストを超えないが、作る前に元の重さで確かめる。
        Meth::Flat
            if weight(&Val::List(Vec::new()))
                + xs.iter().map(weight).fold(0usize, usize::saturating_add)
                > MAX_STR =>
        {
            return None
        }
        Meth::Flat => Val::List(
            xs.iter()
                .flat_map(|x| match x {
                    Val::List(ys) => ys.clone(),
                    other => vec![other.clone()],
                })
                .collect(),
        ),
        Meth::Slice => {
            let (a, b) = slice_range(xs.len(), args)?;
            Val::List(xs[a..b].to_vec())
        }
        _ => return None,
    })
}

fn method(r: &Val, m: Meth, args: &[Val]) -> Val {
    if !matches!(
        m,
        Meth::Contains
            | Meth::ContainsAll
            | Meth::ContainsAny
            | Meth::IsEmpty
            | Meth::ToString
            | Meth::Lower
    ) {
        let v = match r {
            Val::Str(s) => str_method(s, m, args),
            Val::Num(x) => num_method(*x, m, args),
            Val::List(xs) => list_method(xs, m, args),
            w if is_when(w) => when_method(w, m, args),
            _ => None,
        };
        return v.unwrap_or(Val::Null);
    }
    match (m, r) {
        (Meth::ToString, v) => Val::Str(to_text(v)),
        (Meth::IsEmpty, Val::Null) => Val::Bool(true),
        (Meth::IsEmpty, Val::Str(s)) => Val::Bool(s.is_empty()),
        (Meth::IsEmpty, Val::List(xs)) => Val::Bool(xs.is_empty()),
        (Meth::IsEmpty, _) => Val::Bool(false),
        (Meth::Lower, Val::Str(s)) => Val::Str(s.to_lowercase()),
        (Meth::Lower, _) => Val::Null,
        (_, Val::Str(s)) => {
            let has = |a: &Val| match a {
                Val::Str(x) => Some(s.contains(x.as_str())),
                Val::Null => None,
                other => Some(s.contains(to_text(other).as_str())),
            };
            let hits: Option<Vec<bool>> = args.iter().map(has).collect();
            match hits {
                Some(h) => Val::Bool(if m == Meth::ContainsAny {
                    h.iter().any(|&x| x)
                } else {
                    h.iter().all(|&x| x)
                }),
                None => Val::Null,
            }
        }
        (_, Val::List(xs)) => {
            let has = |a: &Val| xs.iter().any(|x| equal(x, a));
            Val::Bool(if m == Meth::ContainsAny {
                args.iter().any(has)
            } else {
                args.iter().all(has)
            })
        }
        _ => Val::Null,
    }
}

#[cfg(test)]
#[path = "test_expr_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_number_fn_unit.rs"]
mod test_number_fn_unit;

#[cfg(test)]
#[path = "test_methods_unit.rs"]
mod test_methods_unit;

#[cfg(test)]
#[path = "test_expr_cap_unit.rs"]
mod test_expr_cap_unit;

#[cfg(test)]
#[path = "test_expr_list_cap_unit.rs"]
mod test_expr_list_cap_unit;

#[cfg(test)]
#[path = "test_moment_tokens_unit.rs"]
mod test_moment_tokens_unit;

#[cfg(test)]
#[path = "test_review6_unit.rs"]
mod test_review6_unit;
