//! `.base` の table ビュー(タスク 6。BV-3〜BV-8・BV-13・SC-8・CV-2)。
//!
//! `.base` は saphyr で読むだけで、書き戻さない(BV-3)。知らないキーは読まずに無視する(BV-8)。
//! filters と formulas の式は parse のときに1回だけ構文木にし、build・cell は評価だけをする。
//! 評価できない filters のビューは開かない。評価できない式の列は `Shown::Unsupported` と `Grid.notes`(BV-7)。
//! 形は docs/design.md の「核の公開のインターフェース」。

use crate::expr::{self, Env, Expr, ExprError, Links, Val};
use crate::i18n::Msg;
use crate::source::{Cell, FileInfo, RowId, Source, Value};
use crate::summary::Summary;
use crate::types::{self, Kind};
use saphyr::{LoadableYamlNode, Scalar, Yaml};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;

/// 並べ替えの向き。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Asc,
    Desc,
}

/// table などのビュー1つ。
#[derive(Clone, Debug)]
pub struct View {
    /// `type`(table・cards など)。無ければ空。
    pub kind: String,
    pub name: String,
    /// 列の id(`note.` は外した素の名前、`file.*`・`formula.*` はそのまま)。
    pub order: Vec<String>,
    pub sort: Vec<(String, Dir)>,
    pub group_by: Option<(String, Dir)>,
    pub limit: Option<usize>,
    /// `summaries`(列の id → 集計の名前。BV-14)。名前は build で読む。
    pub summaries: Vec<(String, String)>,
    filters: Filter,
}

/// `.base` を読んだもの。
#[derive(Clone, Debug)]
pub struct Base {
    pub views: Vec<View>,
    filters: Filter,
    formulas: HashMap<String, (String, Result<Expr, ExprError>)>,
    display: HashMap<String, String>,
    /// 最上位の `summaries`(式で書いた集計の名前 → 式)。解釈しない(BV-14)。
    summary_formulas: HashMap<String, String>,
    /// BV-22: `.base` を直接開いたときの this(その `.base` のファイル)。無ければ this を使う式は未対応(BV-7)。
    this: Option<FileInfo>,
}

/// 表の列。
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub id: String,
    pub title: String,
}

/// 組み立てた表。
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub columns: Vec<Column>,
    pub rows: Vec<RowId>,
    /// groupBy の見出し(値の文字列。空は `(空)`)と rows の範囲。groupBy なしなら空。
    pub groups: Vec<(String, Range<usize>)>,
    /// 未対応の列・並べ替え・集計の説明(BV-7)。
    pub notes: Vec<String>,
    /// `notes` の先頭のうち、列のセルに印(`?`)が付くもの(列の説明)の数。残りは画面に印の無い説明。
    pub marked: usize,
    /// 表の下の集計(列の id と組み込みの集計。BV-14)。無ければ集計の行を出さない。
    pub summaries: Vec<(String, Summary)>,
}

/// セルの見せ方。
#[derive(Clone, Debug, PartialEq)]
pub enum Shown {
    /// ノートのキー(値は prop の値、lock は Source の理由)。
    Prop(Cell),
    /// `file.*`・`formula.*` の値。
    Computed(Val),
    /// 評価できない式(理由)。
    Unsupported(String),
}

/// filters の木。
#[derive(Clone, Debug)]
enum Filter {
    And(Vec<Filter>),
    Or(Vec<Filter>),
    /// どれも真でない。
    Not(Vec<Filter>),
    Leaf(String, Result<Expr, ExprError>),
    /// 形が読めない(理由)。
    Bad(String),
}

/// 空のグループの見出し(日本語の文)。見出しには今の言語の `Msg::EmptyHeading.text()` を使う(SR-23)。
pub const EMPTY_HEADING: &str = Msg::EmptyHeading.ja();

// ---- 読み取り ----

/// `.base` を読む。YAML として読めない・先頭がマップでない → Err(理由)。知らないキーは無視(BV-8)。
pub fn parse(text: &str) -> Result<Base, String> {
    check_aliases(text)?;
    let docs = Yaml::load_from_str(text).map_err(|e| Msg::BaseYaml.fill(&[&e]))?;
    let mut base = Base {
        views: Vec::new(),
        filters: Filter::And(Vec::new()),
        formulas: HashMap::new(),
        display: HashMap::new(),
        summary_formulas: HashMap::new(),
        this: None,
    };
    let Some(root) = docs.first().map(untag) else {
        return Ok(base);
    };
    if root.is_null() {
        return Ok(base);
    }
    if root.as_mapping().is_none() {
        return Err(Msg::BaseNotMap.text().to_string());
    }
    if let Some(f) = get(root, "filters") {
        base.filters = filter_of(f);
    }
    if let Some(m) = get(root, "formulas").and_then(|y| y.as_mapping()) {
        for (k, v) in m {
            if let Some(name) = text_of(k) {
                let src = text_of(v).unwrap_or_default();
                let e = expr::parse(&src);
                base.formulas.insert(name, (src, e));
            }
        }
    }
    if let Some(m) = get(root, "properties").and_then(|y| y.as_mapping()) {
        for (k, v) in m {
            let title = get(v, "displayName").and_then(text_of);
            if let (Some(id), Some(title)) = (text_of(k), title) {
                base.display.insert(column_id(&id), title);
            }
        }
    }
    if let Some(m) = get(root, "summaries").and_then(|y| y.as_mapping()) {
        for (k, v) in m {
            if let Some(name) = text_of(k) {
                base.summary_formulas
                    .insert(name, text_of(v).unwrap_or_default());
            }
        }
    }
    if let Some(seq) = get(root, "views").and_then(|y| y.as_sequence()) {
        base.views = seq.iter().map(view_of).collect();
    }
    Ok(base)
}

/// 別名を展開して増えるノードの数の上限(saphyr は別名ごとにノードを複製するので、重ねた別名で膨らむ)。
const MAX_ALIAS_NODES: u64 = 10_000;
/// シーケンス・マップの入れ子の深さの上限(読み手の再帰がスタックを使い切らないように)。
const MAX_DEPTH: usize = 64;

/// 読む前に saphyr_parser のイベントを数え、別名を展開して増えるノードの数か、入れ子の深さが上限を
/// 超えたら Err(信用できない .base・ノートの中身を、再帰する読み手に渡さない)。
/// YAML の誤りはここでは見ない(load_from_str が理由を出す)。
pub(crate) fn check_aliases(text: &str) -> Result<(), String> {
    use saphyr_parser::{Event, Parser};
    // アンカーの id → そのノードを展開したノードの数。
    let mut sizes: HashMap<usize, u64> = HashMap::new();
    // 開いているシーケンス・マップ(アンカーの id, 中のノードの数)。
    let mut open: Vec<(usize, u64)> = Vec::new();
    let mut expanded: u64 = 0;
    for ev in Parser::new_from_str(text) {
        let Ok((ev, _)) = ev else {
            return Ok(());
        };
        let (n, anchor) = match ev {
            Event::Scalar(_, _, a, _) => (1, a),
            Event::Alias(id) => {
                let n = sizes.get(&id).copied().unwrap_or(1);
                expanded = expanded.saturating_add(n);
                if expanded > MAX_ALIAS_NODES {
                    return Err(Msg::BaseAliasTooBig.fill(&[&MAX_ALIAS_NODES]));
                }
                (n, 0)
            }
            Event::SequenceStart(a, _) | Event::MappingStart(a, _) => {
                open.push((a, 1));
                if open.len() > MAX_DEPTH {
                    return Err(Msg::BaseTooDeep.fill(&[&MAX_DEPTH]));
                }
                continue;
            }
            Event::SequenceEnd | Event::MappingEnd => match open.pop() {
                Some(x) => (x.1, x.0),
                None => continue,
            },
            _ => continue,
        };
        if anchor != 0 {
            sizes.insert(anchor, n);
        }
        if let Some(top) = open.last_mut() {
            top.1 = top.1.saturating_add(n);
        }
    }
    Ok(())
}

/// 式の文字列が参照する formula の名前(`formula.x`・`formula["x"]`)。文字列の中は見ない。
pub(crate) fn formula_refs(src: &str) -> Vec<String> {
    let cs: Vec<char> = src.chars().collect();
    let is_id = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = Vec::new();
    let mut i = 0;
    let mut prev_dot = false;
    while i < cs.len() {
        let c = cs[i];
        if c == '"' || c == '\'' {
            i += 1;
            while i < cs.len() && cs[i] != c {
                i += if cs[i] == '\\' { 2 } else { 1 };
            }
            i += 1;
            prev_dot = false;
            continue;
        }
        if is_id(c) {
            let start = i;
            while i < cs.len() && is_id(cs[i]) {
                i += 1;
            }
            let word: String = cs[start..i].iter().collect();
            if word == "formula" && !prev_dot {
                let mut j = i;
                while j < cs.len() && cs[j].is_whitespace() {
                    j += 1;
                }
                if j < cs.len() && cs[j] == '.' {
                    j += 1;
                    let s = j;
                    while j < cs.len() && is_id(cs[j]) {
                        j += 1;
                    }
                    out.push(cs[s..j].iter().collect());
                } else if j < cs.len() && cs[j] == '[' {
                    j += 1;
                    while j < cs.len() && cs[j].is_whitespace() {
                        j += 1;
                    }
                    if j < cs.len() && (cs[j] == '"' || cs[j] == '\'') {
                        let q = cs[j];
                        let s = j + 1;
                        let mut k = s;
                        while k < cs.len() && cs[k] != q {
                            k += 1;
                        }
                        out.push(cs[s..k.min(cs.len())].iter().collect());
                    }
                }
            }
            prev_dot = false;
            continue;
        }
        if !c.is_whitespace() {
            prev_dot = c == '.';
        }
        i += 1;
    }
    out
}

/// expr の if と同じ真偽の規則: Null・false・0・NaN・空の文字列は偽、ほかは真。
fn truthy(v: &Val) -> bool {
    match v {
        Val::Null => false,
        Val::Bool(b) => *b,
        Val::Num(f) => *f != 0.0 && !f.is_nan(),
        Val::Str(s) => !s.is_empty(),
        _ => true,
    }
}

fn untag<'a, 'b>(y: &'a Yaml<'b>) -> &'a Yaml<'b> {
    match y {
        Yaml::Tagged(_, inner) => untag(inner),
        _ => y,
    }
}

fn get<'a, 'b>(y: &'a Yaml<'b>, key: &str) -> Option<&'a Yaml<'b>> {
    untag(y).as_mapping_get(key).map(untag)
}

/// スカラーの文字列(数・真偽も文字にする)。null・マップ・リストは None。
fn text_of(y: &Yaml) -> Option<String> {
    match untag(y) {
        Yaml::Value(Scalar::String(s)) => Some(s.to_string()),
        Yaml::Value(Scalar::Integer(n)) => Some(n.to_string()),
        Yaml::Value(Scalar::FloatingPoint(f)) => Some(f.to_string()),
        Yaml::Value(Scalar::Boolean(b)) => Some(b.to_string()),
        Yaml::Representation(s, _, _) => Some(s.to_string()),
        _ => None,
    }
}

fn dir_of(y: Option<&Yaml>) -> Dir {
    match y.and_then(text_of) {
        Some(s) if s.eq_ignore_ascii_case("desc") => Dir::Desc,
        _ => Dir::Asc,
    }
}

fn view_of(y: &Yaml) -> View {
    let order = get(y, "order")
        .and_then(|o| o.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(text_of)
                .map(|t| column_id(&t))
                .collect()
        })
        .unwrap_or_default();
    let sort = get(y, "sort")
        .and_then(|o| o.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(|e| {
                    let p = get(e, "property").and_then(text_of)?;
                    Some((column_id(&p), dir_of(get(e, "direction"))))
                })
                .collect()
        })
        .unwrap_or_default();
    let group_by = get(y, "groupBy").and_then(|g| {
        let p = get(g, "property").and_then(text_of)?;
        Some((column_id(&p), dir_of(get(g, "direction"))))
    });
    let limit = get(y, "limit").and_then(|l| match l {
        Yaml::Value(Scalar::Integer(n)) => usize::try_from(*n).ok(),
        _ => None,
    });
    let summaries = get(y, "summaries")
        .and_then(|m| m.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((column_id(&text_of(k)?), text_of(v)?)))
                .collect()
        })
        .unwrap_or_default();
    View {
        kind: get(y, "type").and_then(text_of).unwrap_or_default(),
        name: get(y, "name").and_then(text_of).unwrap_or_default(),
        order,
        sort,
        group_by,
        limit,
        summaries,
        filters: get(y, "filters").map_or(Filter::And(Vec::new()), filter_of),
    }
}

/// filters の YAML → 木。文字列は式、マップは and・or・not(複数あれば AND でつなぐ)。
fn filter_of(y: &Yaml) -> Filter {
    let y = untag(y);
    if let Some(src) = text_of(y) {
        let e = expr::parse(&src);
        return Filter::Leaf(src, e);
    }
    if y.is_null() {
        return Filter::And(Vec::new());
    }
    let Some(m) = y.as_mapping() else {
        return Filter::Bad(Msg::BaseFilterShape.text().into());
    };
    let mut parts = Vec::new();
    for (k, v) in m {
        let list = |v: &Yaml| -> Vec<Filter> {
            match untag(v).as_sequence() {
                Some(s) => s.iter().map(filter_of).collect(),
                None => vec![filter_of(v)],
            }
        };
        match text_of(k).as_deref() {
            Some("and") => parts.push(Filter::And(list(v))),
            Some("or") => parts.push(Filter::Or(list(v))),
            Some("not") => parts.push(Filter::Not(list(v))),
            other => parts.push(Filter::Bad(
                Msg::BaseFilterUnknown.fill(&[&other.unwrap_or("?")]),
            )),
        }
    }
    if parts.len() == 1 {
        parts.pop().unwrap_or(Filter::And(Vec::new()))
    } else {
        Filter::And(parts)
    }
}

/// 列の id: `note.x` は素の `x`、ほかはそのまま。
/// BV-24: displayName の無い列の見出し(Obsidian の既定)。式の列は式の名前、ファイルの項目は `file name` など、
/// ノートのキーはキーの名前。
pub fn default_title(id: &str) -> String {
    if let Some(name) = id.strip_prefix("formula.") {
        return name.to_string();
    }
    match id.strip_prefix("file.") {
        Some("ctime") => "created time".to_string(),
        Some("mtime") => "modified time".to_string(),
        Some("ext") => "file extension".to_string(),
        Some(f) => format!("file {f}"),
        None => id.to_string(),
    }
}

fn column_id(s: &str) -> String {
    let s = s.trim();
    s.strip_prefix("note.").unwrap_or(s).to_string()
}

/// this の無いところ(`.base` を開いていない・mdgrid のビュー)で this を使う式(BV-22・BV-7)。
fn no_this() -> ExprError {
    ExprError::Unsupported("this".to_string())
}

fn reason(src: &str, e: &ExprError) -> String {
    match e {
        ExprError::Unsupported(name) => Msg::BaseUnsupported.fill(&[name, &src]),
        ExprError::Syntax(msg) => Msg::BaseExprSyntax.fill(&[msg, &src]),
    }
}

impl Filter {
    /// 評価できない葉・読めない形・使う formula の問題があれば理由(BV-7)。
    fn check(&self, base: &Base) -> Result<(), String> {
        match self {
            Filter::And(xs) | Filter::Or(xs) | Filter::Not(xs) => {
                xs.iter().try_for_each(|x| x.check(base))
            }
            Filter::Leaf(src, Err(e)) => Err(reason(src, e)),
            Filter::Leaf(src, Ok(e)) if e.uses_this() && base.this.is_none() => {
                Err(reason(src, &no_this()))
            }
            Filter::Leaf(src, Ok(_)) => formula_refs(src)
                .iter()
                .find_map(|r| base.formula_problem(r))
                .map_or(Ok(()), |p| Err(Msg::BaseInExpr.fill(&[&p, src]))),
            Filter::Bad(r) => Err(r.clone()),
        }
    }

    /// 一番外の and を分けた式の文字列(BV-19 の取り込み)。全部を満たす行だけ、の意味。
    fn to_exprs(&self, dropped: &mut Vec<String>) -> Vec<String> {
        match self {
            Filter::And(xs) => xs.iter().filter_map(|x| x.to_expr(dropped)).collect(),
            other => other.to_expr(dropped).into_iter().collect(),
        }
    }

    /// 1つの式の文字列。絞らないもの(空の and・or・not)は None。読めない形は落として dropped に説明。
    /// 評価できない葉はそのまま残し(Obsidian では評価できるかもしれない)、dropped に知らせる。
    fn to_expr(&self, dropped: &mut Vec<String>) -> Option<String> {
        let join = |xs: &[Filter], op: &str, dropped: &mut Vec<String>| -> Option<String> {
            let parts: Vec<String> = xs.iter().filter_map(|x| x.to_expr(dropped)).collect();
            match parts.len() {
                0 => None,
                1 => parts.into_iter().next(),
                _ => Some(
                    parts
                        .iter()
                        .map(|p| format!("({p})"))
                        .collect::<Vec<_>>()
                        .join(op),
                ),
            }
        };
        match self {
            Filter::Leaf(src, e) => {
                if let Err(e) = e {
                    dropped.push(Msg::BaseFilterKept.fill(&[&reason(src, e)]));
                }
                let s = src.trim();
                (!s.is_empty()).then(|| s.to_string())
            }
            Filter::Bad(r) => {
                dropped.push(Msg::BaseFilterDropped.fill(&[r]));
                None
            }
            Filter::And(xs) => join(xs, " && ", dropped),
            Filter::Or(xs) => join(xs, " || ", dropped),
            Filter::Not(xs) => join(xs, " || ", dropped).map(|e| format!("!({e})")),
        }
    }

    /// 葉は真偽の規則(if と同じ。Null・false・0・空の文字列は偽)で判定。空の and・or・not は真(絞らない)。
    fn pass(&self, ctx: &Ctx) -> bool {
        match self {
            Filter::And(xs) => xs.iter().all(|x| x.pass(ctx)),
            Filter::Or(xs) => xs.is_empty() || xs.iter().any(|x| x.pass(ctx)),
            Filter::Not(xs) => !xs.iter().any(|x| x.pass(ctx)),
            Filter::Leaf(_, Ok(e)) => truthy(&ctx.eval(e)),
            Filter::Leaf(_, Err(_)) | Filter::Bad(_) => false,
        }
    }
}

// ---- 評価の文脈(行1つ・問い1つ) ----

struct Ctx<'a> {
    base: &'a Base,
    prop: &'a dyn Fn(&str) -> Option<Value>,
    file: &'a FileInfo,
    links: &'a Links<'a>,
    today: i64,
    now: i64,
    /// 評価中の formula(循環は印を付けて止める)。
    stack: RefCell<Vec<String>>,
    memo: RefCell<HashMap<String, Val>>,
    /// 評価の途中で出会った、評価できない formula(BV-7)。
    bad: RefCell<Option<String>>,
}

impl<'a> Ctx<'a> {
    fn new(
        base: &'a Base,
        prop: &'a dyn Fn(&str) -> Option<Value>,
        file: &'a FileInfo,
        links: &'a Links<'a>,
        today: i64,
        now: i64,
    ) -> Ctx<'a> {
        Ctx {
            base,
            prop,
            file,
            links,
            today,
            now,
            stack: RefCell::new(Vec::new()),
            memo: RefCell::new(HashMap::new()),
            bad: RefCell::new(None),
        }
    }

    fn eval(&self, e: &Expr) -> Val {
        let formula = |n: &str| self.formula(n);
        let env = Env {
            prop: self.prop,
            file: self.file,
            formula: &formula,
            today: self.today,
            now: self.now,
        };
        expr::eval_with(e, &env, self.links)
    }

    fn mark_bad(&self, r: String) {
        let mut bad = self.bad.borrow_mut();
        if bad.is_none() {
            *bad = Some(r);
        }
    }

    fn formula(&self, name: &str) -> Option<Val> {
        if let Some(v) = self.memo.borrow().get(name) {
            return Some(v.clone());
        }
        if self.stack.borrow().iter().any(|n| n == name) {
            self.mark_bad(Msg::BaseFormulaCycle.fill(&[&name]));
            return None;
        }
        match self.base.formulas.get(name) {
            None => {
                self.mark_bad(Msg::BaseFormulaMissing.fill(&[&name]));
                None
            }
            Some((src, Err(e))) => {
                self.mark_bad(format!("formula.{name}: {}", reason(src, e)));
                None
            }
            Some((_, Ok(e))) => {
                self.stack.borrow_mut().push(name.to_string());
                let v = self.eval(e);
                self.stack.borrow_mut().pop();
                self.memo.borrow_mut().insert(name.to_string(), v.clone());
                Some(v)
            }
        }
    }
}

/// 読んでいない行の代わりの属性(表示名だけ)。
fn fallback_file(src: &dyn Source, row: &RowId) -> FileInfo {
    let path = src.label(row);
    let (folder, name) = match path.rsplit_once('/') {
        Some((f, n)) => (f.to_string(), n.to_string()),
        None => (String::new(), path.clone()),
    };
    let (basename, ext) = match name.rsplit_once('.') {
        Some((b, e)) if !b.is_empty() => (b.to_string(), e.to_string()),
        _ => (name.clone(), String::new()),
    };
    FileInfo {
        name,
        basename,
        ext,
        path,
        folder,
        size: 0,
        mtime: 0,
        ctime: 0,
        tags: Vec::new(),
    }
}

// ---- 並べ替えの鍵 ----

/// 並べ替えの鍵。型の合う値 → 型の合わない値(CV-2)→ 空 の順。向きは型の合う値の中だけに効く。
#[derive(Clone, Debug)]
enum Key {
    Val(Val),
    Bad(String),
    Empty,
}

/// 何を鍵にするか(列ごとに1回作る)。
enum Spec {
    Prop(String, Kind),
    Formula(String),
    Expr(Result<Expr, ExprError>),
}

impl Spec {
    fn new(id: &str, src: &dyn Source) -> Spec {
        if let Some(name) = id.strip_prefix("formula.") {
            Spec::Formula(name.to_string())
        } else if id.starts_with("file.") {
            Spec::Expr(expr::parse(id))
        } else {
            Spec::Prop(id.to_string(), src.kind(id).kind)
        }
    }

    /// 評価できなければ理由。
    fn problem(&self, base: &Base) -> Option<String> {
        match self {
            Spec::Prop(..) | Spec::Expr(Ok(_)) => None,
            Spec::Expr(Err(e)) => Some(match e {
                ExprError::Unsupported(n) => Msg::BaseUnsupportedField.fill(&[n]),
                ExprError::Syntax(m) => Msg::BaseBadColumn.fill(&[m]),
            }),
            Spec::Formula(name) => base.formula_problem(name),
        }
    }

    fn key(&self, ctx: &Ctx) -> Key {
        let v = match self {
            Spec::Prop(col, kind) => return prop_key((ctx.prop)(col).as_ref(), *kind),
            Spec::Formula(name) => ctx.formula(name).unwrap_or(Val::Null),
            Spec::Expr(Ok(e)) => ctx.eval(e),
            Spec::Expr(Err(_)) => Val::Null,
        };
        val_key(v)
    }
}

fn is_empty_val(v: &Val) -> bool {
    match v {
        Val::Null => true,
        Val::Str(s) => s.is_empty(),
        Val::List(xs) => xs.is_empty(),
        _ => false,
    }
}

fn val_key(v: Val) -> Key {
    if is_empty_val(&v) {
        Key::Empty
    } else {
        Key::Val(v)
    }
}

/// ノートのキーの値の鍵: 列の型(CE-2)で読み、合わない値は Bad(CV-2)。
fn prop_key(v: Option<&Value>, kind: Kind) -> Key {
    let Some(v) = v else {
        return Key::Empty;
    };
    if types::is_empty(v) {
        return Key::Empty;
    }
    if !types::fits(kind, v) {
        return Key::Bad(value_text(v));
    }
    let val = match (kind, v) {
        (Kind::Number, Value::Int(n)) => Val::Num(*n as f64),
        (Kind::Number, Value::Float(f)) => Val::Num(*f),
        (Kind::Checkbox, Value::Bool(b)) => Val::Bool(*b),
        (Kind::Date | Kind::DateTime, _) => expr::to_val(v),
        (Kind::List, Value::List(_)) => expr::to_val(v),
        (Kind::List, _) => Val::List(vec![Val::Str(value_text(v))]),
        _ => Val::Str(value_text(v)),
    };
    val_key(val)
}

/// フロントマターの値の文字列(並べ替えと見出し用)。
fn value_text(v: &Value) -> String {
    match v {
        Value::Null | Value::Other => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Str(s) => s.clone(),
        Value::List(xs) => xs.iter().map(value_text).collect::<Vec<_>>().join(", "),
    }
}

/// 式の値の文字列(見出し用)。
fn val_text(v: &Val) -> String {
    match v {
        Val::Null => String::new(),
        Val::Bool(b) => b.to_string(),
        Val::Num(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        Val::Num(f) => f.to_string(),
        Val::Str(s) => s.clone(),
        Val::Date(d) => types::format_date(*d),
        Val::DateTime(s) => {
            let t = s.rem_euclid(86_400);
            format!(
                "{}T{:02}:{:02}:{:02}",
                types::format_date(s.div_euclid(86_400)),
                t / 3600,
                t / 60 % 60,
                t % 60
            )
        }
        Val::Duration(ms) => ms.to_string(),
        Val::List(xs) => xs.iter().map(val_text).collect::<Vec<_>>().join(", "),
    }
}

fn heading(k: &Key) -> String {
    match k {
        Key::Val(v) => val_text(v),
        Key::Bad(s) => s.clone(),
        Key::Empty => Msg::EmptyHeading.text().to_string(),
    }
}

/// 型の違う値どうしの並び(真偽 < 数 < 期間 < 日付・日時 < 文字列 < リスト)。
fn rank(v: &Val) -> u8 {
    match v {
        Val::Null => 0,
        Val::Bool(_) => 1,
        Val::Num(_) => 2,
        Val::Duration(_) => 3,
        Val::Date(_) | Val::DateTime(_) => 4,
        Val::Str(_) => 5,
        Val::List(_) => 6,
    }
}

fn secs(v: &Val) -> i64 {
    match v {
        Val::Date(d) => d.saturating_mul(86_400),
        Val::DateTime(s) => *s,
        _ => 0,
    }
}

fn text_cmp(a: &str, b: &str) -> Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

fn val_cmp(a: &Val, b: &Val) -> Ordering {
    match (a, b) {
        (Val::Bool(x), Val::Bool(y)) => x.cmp(y),
        (Val::Num(x), Val::Num(y)) => x.total_cmp(y),
        (Val::Duration(x), Val::Duration(y)) => x.cmp(y),
        (Val::Date(_) | Val::DateTime(_), Val::Date(_) | Val::DateTime(_)) => secs(a).cmp(&secs(b)),
        (Val::Str(x), Val::Str(y)) => text_cmp(x, y),
        (Val::List(x), Val::List(y)) => {
            for (p, q) in x.iter().zip(y) {
                let o = val_cmp(p, q);
                if o != Ordering::Equal {
                    return o;
                }
            }
            x.len().cmp(&y.len())
        }
        _ => rank(a).cmp(&rank(b)),
    }
}

fn key_cmp(a: &Key, b: &Key, dir: Dir) -> Ordering {
    match (a, b) {
        (Key::Val(x), Key::Val(y)) => {
            let o = val_cmp(x, y);
            if dir == Dir::Desc {
                o.reverse()
            } else {
                o
            }
        }
        (Key::Bad(x), Key::Bad(y)) => text_cmp(x, y),
        (Key::Empty, Key::Empty) => Ordering::Equal,
        (Key::Val(_), _) | (Key::Bad(_), Key::Empty) => Ordering::Less,
        _ => Ordering::Greater,
    }
}

// ---- 組み立て ----

impl View {
    /// ビューの filters を式の文字列にしたもの(一番外の and は分ける。全部を満たす行だけ)。BV-19。
    /// 読めない部分・評価できない式は dropped に説明を足す。
    pub fn filter_exprs(&self, dropped: &mut Vec<String>) -> Vec<String> {
        self.filters.to_exprs(dropped)
    }
}

impl Base {
    /// 全体の filters を式の文字列にしたもの(View::filter_exprs と同じ形)。BV-19。
    pub fn filter_exprs(&self, dropped: &mut Vec<String>) -> Vec<String> {
        self.filters.to_exprs(dropped)
    }

    /// formulas の名前(文字の順)。BV-19。
    pub fn formula_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.formulas.keys().cloned().collect();
        v.sort();
        v
    }

    /// properties で displayName を付けた列の id(文字の順)。BV-19。
    pub fn display_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.display.keys().cloned().collect();
        v.sort();
        v
    }

    /// formula を評価できないなら理由。参照する formula を(循環に気をつけて)たどる(BV-7)。
    fn formula_problem(&self, name: &str) -> Option<String> {
        let mut path = Vec::new();
        let mut ok = std::collections::HashSet::new();
        self.formula_walk(name, &mut path, &mut ok)
    }

    fn formula_walk(
        &self,
        name: &str,
        path: &mut Vec<String>,
        ok: &mut std::collections::HashSet<String>,
    ) -> Option<String> {
        if ok.contains(name) {
            return None;
        }
        if path.iter().any(|n| n == name) {
            return Some(Msg::BaseFormulaCycle.fill(&[&name]));
        }
        match self.formulas.get(name) {
            None => Some(Msg::BaseFormulaMissing.fill(&[&name])),
            Some((src, Err(e))) => Some(format!("formula.{name}: {}", reason(src, e))),
            Some((src, Ok(e))) if e.uses_this() && self.this.is_none() => {
                Some(format!("formula.{name}: {}", reason(src, &no_this())))
            }
            Some((src, Ok(_))) => {
                path.push(name.to_string());
                let p = formula_refs(src)
                    .iter()
                    .find_map(|r| self.formula_walk(r, path, ok));
                path.pop();
                if p.is_none() {
                    ok.insert(name.to_string());
                }
                p
            }
        }
    }

    /// BV-22: `.base` を直接開いたときの this(その `.base` のファイル)を渡す。
    /// CLI-16: 列 `columns` の table のビュー1つだけの `.base`(フォルダの既定の表に --filter・--sort を当てる)。
    pub fn plain(columns: Vec<String>) -> Base {
        Base {
            views: vec![View {
                kind: "table".into(),
                name: "print".into(),
                order: columns,
                sort: Vec::new(),
                group_by: None,
                limit: None,
                summaries: Vec::new(),
                filters: Filter::And(Vec::new()),
            }],
            filters: Filter::And(Vec::new()),
            formulas: HashMap::new(),
            display: HashMap::new(),
            summary_formulas: HashMap::new(),
            this: None,
        }
    }

    /// CLI-16: ビュー `view` の絞り込みを「元の絞り込み AND `filters` の式」にし、`sorts` があれば並べ替えを置き換える。
    /// 式は `.base` の式と同じに読む(読めない式は評価のときに BV-7 の理由)。列の名前は `note.x` を `x` に。
    pub fn narrow(&mut self, view: usize, filters: &[String], sorts: &[(String, Dir)]) {
        let Some(v) = self.views.get_mut(view) else {
            return;
        };
        if !filters.is_empty() {
            let old = std::mem::replace(&mut v.filters, Filter::And(Vec::new()));
            let mut all = vec![old];
            all.extend(
                filters
                    .iter()
                    .map(|f| Filter::Leaf(f.clone(), expr::parse(f))),
            );
            v.filters = Filter::And(all);
        }
        if !sorts.is_empty() {
            v.sort = sorts.iter().map(|(c, d)| (column_id(c), *d)).collect();
        }
    }

    pub fn set_this(&mut self, this: Option<FileInfo>) {
        self.this = this;
    }

    fn title(&self, id: &str) -> String {
        self.display
            .get(id)
            .cloned()
            .unwrap_or_else(|| default_title(id))
    }

    /// ビューを組み立てる(BV-4・BV-5・BV-13)。table 以外(SC-8)・評価できない filters(BV-7)は Err(理由)。
    /// 並べ替えは安定(同じ鍵は src.rows() の順)。groupBy のまとまりで並べ、中は sort。limit は最後に全体の行数に効く。
    pub fn build(
        &self,
        view: usize,
        src: &dyn Source,
        prop: &dyn Fn(&RowId, &str) -> Option<Value>,
        today: i64,
        now: i64,
    ) -> Result<Grid, String> {
        let v = self
            .views
            .get(view)
            .ok_or_else(|| Msg::BaseNoViewIndex.fill(&[&view]))?;
        if v.kind.is_empty() {
            return Err(Msg::BaseViewNoType.fill(&[&v.name]));
        }
        if v.kind != "table" {
            return Err(Msg::BaseViewUnsupported.fill(&[&v.kind, &v.name]));
        }
        self.filters
            .check(self)
            .map_err(|e| Msg::BaseGlobalFilters.fill(&[&e]))?;
        v.filters
            .check(self)
            .map_err(|e| Msg::BaseViewFilters.fill(&[&v.name, &e]))?;

        let ids: Vec<String> = if v.order.is_empty() {
            std::iter::once("file.name".to_string())
                .chain(src.columns())
                .collect()
        } else {
            v.order.clone()
        };
        let columns: Vec<Column> = ids
            .iter()
            .map(|id| Column {
                id: id.clone(),
                title: self.title(id),
            })
            .collect();

        let mut notes = Vec::new();
        for id in &ids {
            if id.starts_with("formula.") || id.starts_with("file.") {
                if let Some(p) = Spec::new(id, src).problem(self) {
                    notes.push(format!("{id}: {p}"));
                }
            }
        }
        let marked = notes.len();
        let group = v
            .group_by
            .as_ref()
            .map(|(id, d)| (Spec::new(id, src), *d, id));
        let sorts: Vec<(Spec, Dir, &String)> = v
            .sort
            .iter()
            .map(|(id, d)| (Spec::new(id, src), *d, id))
            .collect();
        for (spec, _, id) in group.iter().chain(sorts.iter()) {
            if let Some(p) = spec.problem(self) {
                notes.push(Msg::BaseCannotSort.fill(&[id, &p]));
            }
        }
        // BV-14: 組み込みの集計だけを読む。式の集計・知らない名前・まとまりごとの集計は理由だけ。
        let mut summaries = Vec::new();
        for (id, name) in &v.summaries {
            match Summary::parse(name) {
                Some(s) => summaries.push((id.clone(), s)),
                None => notes.push(match self.summary_formulas.get(name.trim()) {
                    Some(src) => Msg::BaseSummaryFormula.fill(&[id, name, src]),
                    None => Msg::BaseSummaryUnknown.fill(&[id, name]),
                }),
            }
        }
        if v.group_by.is_some() && !v.summaries.is_empty() {
            notes.push(Msg::BaseSummaryGroups.text().to_string());
        }

        // 絞り込みと鍵(行ごとに1回だけ評価する)。
        struct Item {
            row: RowId,
            group: Option<Key>,
            keys: Vec<Key>,
        }
        let mut items: Vec<Item> = Vec::new();
        // リンクの索引は読み込み口が覚えている(行ごとに作らない。BV-22)。
        let index = src.link_index();
        let links = Links {
            index: index.as_deref(),
            this: self.this.as_ref(),
        };
        for row in src.rows() {
            let file = src.file(&row).unwrap_or_else(|| fallback_file(src, &row));
            let p = |k: &str| crate::source::expr_value(src, k, prop(&row, k));
            let ctx = Ctx::new(self, &p, &file, &links, today, now);
            let keep = self.filters.pass(&ctx) && v.filters.pass(&ctx);
            if let Some(bad) = ctx.bad.borrow_mut().take() {
                return Err(Msg::BaseViewFilters.fill(&[&v.name, &bad]));
            }
            if !keep {
                continue;
            }
            let group_key = group.as_ref().map(|(s, _, _)| s.key(&ctx));
            let keys = sorts.iter().map(|(s, _, _)| s.key(&ctx)).collect();
            drop(ctx);
            items.push(Item {
                row,
                group: group_key,
                keys,
            });
        }

        let group_dir = group.as_ref().map(|(_, d, _)| *d);
        items.sort_by(|a, b| {
            let g = match (&a.group, &b.group, group_dir) {
                (Some(x), Some(y), Some(d)) => key_cmp(x, y, d),
                _ => Ordering::Equal,
            };
            g.then_with(|| {
                a.keys
                    .iter()
                    .zip(&b.keys)
                    .zip(&sorts)
                    .map(|((x, y), (_, d, _))| key_cmp(x, y, *d))
                    .find(|o| *o != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
        });
        if let Some(n) = v.limit {
            items.truncate(n);
        }

        let mut groups: Vec<(String, Range<usize>)> = Vec::new();
        if let Some(d) = group_dir {
            let mut start = 0;
            for i in 1..=items.len() {
                let split = i == items.len()
                    || match (&items[i - 1].group, &items[i].group) {
                        (Some(x), Some(y)) => key_cmp(x, y, d) != Ordering::Equal,
                        _ => false,
                    };
                if split {
                    let h = items[start].group.as_ref().map(heading).unwrap_or_default();
                    groups.push((h, start..i));
                    start = i;
                }
            }
        }

        Ok(Grid {
            columns,
            rows: items.into_iter().map(|it| it.row).collect(),
            groups,
            notes,
            marked,
            summaries,
        })
    }

    /// セル1つ(BV-7)。ノートのキーは Prop、`file.*`・`formula.*` は Computed、評価できなければ Unsupported。
    pub fn cell(
        &self,
        src: &dyn Source,
        prop: &dyn Fn(&RowId, &str) -> Option<Value>,
        row: &RowId,
        col: &str,
        today: i64,
        now: i64,
    ) -> Shown {
        let id = column_id(col);
        if !(id.starts_with("formula.") || id.starts_with("file.")) {
            return Shown::Prop(Cell {
                value: prop(row, &id),
                lock: src.get(row, &id).lock,
            });
        }
        let spec = Spec::new(&id, src);
        if let Some(p) = spec.problem(self) {
            return Shown::Unsupported(p);
        }
        let file = src.file(row).unwrap_or_else(|| fallback_file(src, row));
        let p = |k: &str| crate::source::expr_value(src, k, prop(row, k));
        let index = src.link_index();
        let links = Links {
            index: index.as_deref(),
            this: self.this.as_ref(),
        };
        let ctx = Ctx::new(self, &p, &file, &links, today, now);
        let v = match &spec {
            Spec::Formula(name) => ctx.formula(name).unwrap_or(Val::Null),
            Spec::Expr(Ok(e)) => ctx.eval(e),
            _ => Val::Null,
        };
        let bad = ctx.bad.borrow_mut().take();
        match bad {
            Some(r) => Shown::Unsupported(r),
            None => Shown::Computed(v),
        }
    }
}

/// `.base` なしの既定の表(BV-1): 列は src.columns()、行は src.rows()、グループなし。
pub fn default_grid(src: &dyn Source) -> Grid {
    Grid {
        columns: src
            .columns()
            .into_iter()
            .map(|id| Column {
                title: id.clone(),
                id,
            })
            .collect(),
        rows: src.rows(),
        groups: Vec::new(),
        notes: Vec::new(),
        marked: 0,
        summaries: Vec::new(),
    }
}

#[cfg(test)]
#[path = "test_base_unit.rs"]
mod tests;
