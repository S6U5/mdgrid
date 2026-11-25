//! 名前を付けた条件(condition.define〜condition.cycle)。設定の `[conditions.<名前>]` を読み、
//! 式の中の `cond("名前")` を、その条件の式(真かどうか)に置き換える。置き換えは式を読むとき
//! (`expr::parse` が `lookup` を引く)に済ませるので、評価の側は条件を知らない。
//!
//! 今の条件は、起動のはじめに `install` で1度だけ置く(言語の `i18n::set_default` と同じ形)。
//! 試験は糸ごとの `scoped` で置き換える。形は docs/design.md の「名前を付けた条件」。

use crate::expr::{self, Expr, ExprError};
use crate::i18n::Msg;
use crate::profile::{not, warn};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, PoisonError, RwLock};

/// 1つの名前の条件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Condition {
    pub name: String,
    /// condition.label: 見せる名前(無ければ名前)。
    pub label: Option<String>,
    /// 書いた式。
    pub src: String,
    /// condition.digit-keys: 入れ切りの数字(1〜9)。重なった数字は外してある(condition.digit-dup)。
    pub key: Option<u8>,
    /// condition.hide-always: どの表でも当たる行を初めから隠す。
    pub hide: bool,
}

impl Condition {
    /// 見せる名前(condition.label)。
    pub fn shown(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

/// 使える条件の並び(書いた順。印の条件が先)と、置き換え済みの式。
#[derive(Clone, Debug, Default)]
pub struct Conditions {
    list: Vec<Condition>,
    exprs: HashMap<String, Expr>,
    /// condition.marker: 印(ワークスペースの印)から来た条件の名前。
    marker: HashSet<String>,
}

impl PartialEq for Conditions {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list && self.marker == other.marker
    }
}

impl Conditions {
    /// 使える条件(書いた順。読めない・輪になった条件は入れない)。
    pub fn list(&self) -> &[Condition] {
        &self.list
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Condition> {
        self.list.iter().find(|c| c.name == name)
    }

    /// 置き換え済みの式。
    pub fn expr(&self, name: &str) -> Option<&Expr> {
        self.exprs.get(name)
    }

    /// 数字のキー `d` の条件(condition.digit-keys)。
    pub fn by_key(&self, d: u8) -> Option<&Condition> {
        self.list.iter().find(|c| c.key == Some(d))
    }

    /// 名前を並べた文(condition.unknown-name の「ある名前」。名前の順)。
    pub fn names(&self) -> String {
        let mut names: Vec<&str> = self.list.iter().map(|c| c.name.as_str()).collect();
        names.sort_unstable();
        if names.is_empty() {
            Msg::CondNoNames.text().to_string()
        } else {
            names.join(", ")
        }
    }

    /// condition.marker: 印から来た条件か(`--print-config --resolved` の出どころ。cli.resolved-config-ledger)。
    pub fn from_marker(&self, name: &str) -> bool {
        self.marker.contains(name)
    }

    /// condition.digit-order: 数字のキーに振る順の条件(`key` を書いたもの)。印の条件が先、全体の設定の条件が後で、
    /// それぞれ `key` の小さい順(同じ数なら書いた順)。今の表で使えるかと、割り当て直した数字は見ない(画面が見る)。
    pub fn digit_order(&self) -> Vec<&Condition> {
        let mut keyed: Vec<(usize, &Condition)> = self
            .list
            .iter()
            .filter(|c| c.key.is_some())
            .map(|c| (usize::from(!self.marker.contains(&c.name)), c))
            .collect();
        keyed.sort_by_key(|(layer, c)| (*layer, c.key));
        keyed.into_iter().map(|(_, c)| c).collect()
    }

    /// condition.marker・condition.marker-layer: 印の条件 `marker`(読んだまま。書いた順)を、全体の設定の条件
    /// `global` の前に並べて組む。同じ名前は印の条件を使い、全体の設定の条件の式の中の `cond()` も印の条件を指す。
    /// 数字の重なり(condition.digit-dup)は、同じファイルの中だけを見る。警告は印の条件のものを `file` の名前で出す。
    pub fn layered(
        marker: Vec<Condition>,
        file: &str,
        global: &Conditions,
        w: &mut Vec<String>,
    ) -> Conditions {
        if marker.is_empty() {
            return global.clone();
        }
        let names: HashSet<String> = marker.iter().map(|c| c.name.clone()).collect();
        let mut specs: Vec<(Condition, usize)> = marker.into_iter().map(|c| (c, 0)).collect();
        specs.extend(
            global
                .list
                .iter()
                .filter(|c| !names.contains(&c.name))
                .map(|c| (c.clone(), 1)),
        );
        // 全体の設定の条件の問題は、設定を読むところで出してある(印に重ねて新しく起きる輪などは出す)。
        let mut out = Conditions::build_layers(specs, &[file, crate::config::CONFIG_FILE], w);
        out.marker = names
            .into_iter()
            .filter(|n| out.exprs.contains_key(n))
            .collect();
        out
    }

    /// 書いた条件(読んだまま)から、使える条件を組む。名前・式・輪・数字の重なりの問題は `w` に警告して外す。
    pub fn build(specs: Vec<Condition>, file: &str, w: &mut Vec<String>) -> Conditions {
        Conditions::build_layers(specs.into_iter().map(|c| (c, 0)).collect(), &[file], w)
    }

    /// `build` の中身。条件ごとに、書いたファイル(`files` の添字)を添える。
    fn build_layers(
        specs: Vec<(Condition, usize)>,
        files: &[&str],
        w: &mut Vec<String>,
    ) -> Conditions {
        let file_of: HashMap<String, String> = specs
            .iter()
            .map(|(c, k)| {
                let f = files.get(*k).copied().unwrap_or_default();
                (c.name.clone(), f.to_string())
            })
            .collect();
        let file_for = |name: &str| file_of.get(name).cloned().unwrap_or_default();
        let layer_of: HashMap<String, usize> =
            specs.iter().map(|(c, k)| (c.name.clone(), *k)).collect();
        let specs: Vec<Condition> = specs.into_iter().map(|(c, _)| c).collect();
        // 1. 式を読む(`cond("x")` は参照のまま)。読めない式は警告して外す。
        let mut open: HashMap<String, (Expr, Vec<String>)> = HashMap::new();
        let mut order = Vec::new();
        for c in &specs {
            match expr::parse_open(&c.src) {
                Ok((e, refs)) => {
                    open.insert(c.name.clone(), (e, refs));
                    order.push(c.name.clone());
                }
                Err(e) => warn(
                    w,
                    &file_for(&c.name),
                    &format!("conditions.{}.expr", c.name),
                    Msg::RsnCondExpr.fill(&[&expr_reason(&e)]),
                ),
            }
        }
        // 2. 輪(condition.cycle): 輪ごとに1つ警告し、輪の条件をどれも外す。
        let cycles = find_cycles(&order, &open);
        let in_cycle: HashSet<&String> = cycles.iter().flatten().collect();
        for cycle in &cycles {
            let mut path = cycle.clone();
            path.push(cycle[0].clone());
            warn(
                w,
                &file_for(&cycle[0]),
                &format!("conditions.{}", cycle[0]),
                Msg::RsnCondCycle.fill(&[&path.join(" → ")]),
            );
        }
        // 3. 参照を置き換える(参照の先から)。無い名前・外した名前を指す条件は警告して外す。
        let mut done: HashMap<String, Option<Expr>> = HashMap::new();
        for name in &order {
            if !in_cycle.contains(name) {
                resolve(name, &open, &in_cycle, &mut done, &file_for, w);
            }
        }
        // 4. 数字の重なり(condition.digit-dup): 同じファイルの中で、先に書いた条件に当てる。
        let mut list = Vec::new();
        let mut exprs = HashMap::new();
        let mut used: HashMap<(usize, u8), String> = HashMap::new();
        for mut c in specs {
            let Some(Some(e)) = done.remove(&c.name) else {
                continue;
            };
            if let Some(d) = c.key {
                let layer = layer_of.get(&c.name).copied().unwrap_or_default();
                if let Some(first) = used.get(&(layer, d)) {
                    warn(
                        w,
                        &file_for(&c.name),
                        &format!("conditions.{}.key", c.name),
                        Msg::RsnCondKeyDup.fill(&[&d, first]),
                    );
                    c.key = None;
                } else {
                    used.insert((layer, d), c.name.clone());
                }
            }
            exprs.insert(c.name.clone(), e);
            list.push(c);
        }
        Conditions {
            list,
            exprs,
            marker: HashSet::new(),
        }
    }
}

/// 名前 `name` の条件の、参照を置き換えた式(使えなければ None)。結果は `done` に残す。
fn resolve(
    name: &str,
    open: &HashMap<String, (Expr, Vec<String>)>,
    in_cycle: &HashSet<&String>,
    done: &mut HashMap<String, Option<Expr>>,
    file_for: &dyn Fn(&str) -> String,
    w: &mut Vec<String>,
) -> Option<Expr> {
    if let Some(r) = done.get(name) {
        return r.clone();
    }
    let (e, refs) = open.get(name)?;
    for r in refs {
        if open.contains_key(r) && !in_cycle.contains(r) {
            resolve(r, open, in_cycle, done, file_for, w);
        }
    }
    let get = |n: &str| done.get(n).cloned().flatten();
    let out = match expr::inline(e, &get) {
        Ok(e) => Some(e),
        Err(missing) => {
            let reason = if open.contains_key(&missing) || done.contains_key(&missing) {
                Msg::RsnCondUses.fill(&[&missing])
            } else {
                Msg::CondUnknown.fill(&[&missing, &known(open)])
            };
            warn(
                w,
                &file_for(name),
                &format!("conditions.{name}.expr"),
                reason,
            );
            None
        }
    };
    done.insert(name.to_string(), out.clone());
    out
}

fn known(open: &HashMap<String, (Expr, Vec<String>)>) -> String {
    let mut names: Vec<&str> = open.keys().map(String::as_str).collect();
    names.sort_unstable();
    names.join(", ")
}

/// 参照の輪(書いた順で最初に出会った条件から並べる)。
fn find_cycles(order: &[String], open: &HashMap<String, (Expr, Vec<String>)>) -> Vec<Vec<String>> {
    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for start in order {
        let mut stack: Vec<String> = Vec::new();
        walk(start, open, &mut stack, &mut seen, &mut cycles);
    }
    cycles
}

fn walk(
    name: &str,
    open: &HashMap<String, (Expr, Vec<String>)>,
    stack: &mut Vec<String>,
    seen: &mut HashSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if let Some(at) = stack.iter().position(|n| n == name) {
        let cycle = stack[at..].to_vec();
        let same = |c: &Vec<String>| c.len() == cycle.len() && cycle.iter().all(|n| c.contains(n));
        if !cycles.iter().any(same) {
            cycles.push(cycle);
        }
        return;
    }
    if seen.contains(name) {
        return;
    }
    let Some((_, refs)) = open.get(name) else {
        return;
    };
    stack.push(name.to_string());
    for r in refs {
        walk(r, open, stack, seen, cycles);
    }
    stack.pop();
    seen.insert(name.to_string());
}

/// 式の誤りの理由1行。
fn expr_reason(e: &ExprError) -> String {
    match e {
        ExprError::Syntax(m) => m.clone(),
        ExprError::Unsupported(n) => Msg::CondUnsupported.fill(&[n]),
    }
}

/// condition.invalid: 名前は英字で始まり、英数字・`_`・`-` だけ。
pub fn valid_name(name: &str) -> bool {
    let mut cs = name.chars();
    cs.next().is_some_and(|c| c.is_ascii_alphabetic())
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// 設定の文 `text` に条件の名前が出てくる順(`[conditions.名前]` の見出しと、`[conditions]` の下の
/// `名前 = { … }`)。toml の表は名前の順に並ぶので、書いた順(condition.digit-dup の「先に書いた条件」)はここで取る。
pub fn written_order(text: &str) -> Vec<String> {
    let unquote = |s: &str| s.trim().trim_matches('"').trim_matches('\'').to_string();
    let mut out: Vec<String> = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        let t = line.trim();
        if let Some(head) = t.strip_prefix('[') {
            let head = head.trim_start_matches('[');
            let head = head.split(']').next().unwrap_or_default().trim();
            in_table = head == "conditions";
            if let Some(rest) = head.strip_prefix("conditions.") {
                let name = unquote(rest.split('.').next().unwrap_or_default());
                if !out.contains(&name) {
                    out.push(name);
                }
            }
            continue;
        }
        if in_table {
            if let Some((k, _)) = t.split_once('=') {
                let name = unquote(k.split('.').next().unwrap_or_default());
                if !name.is_empty() && !name.starts_with('#') && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// `read` の結果を、設定の文に書いた順(`written_order`)に並べ直す。文に見つからない名前は後ろ(名前の順)。
pub fn in_written_order(mut specs: Vec<Condition>, text: &str) -> Vec<Condition> {
    let order = written_order(text);
    specs.sort_by_key(|c| {
        order
            .iter()
            .position(|n| *n == c.name)
            .unwrap_or(usize::MAX)
    });
    specs
}

/// config.toml の `[conditions]` を読む(名前の順。書いた順は `in_written_order`)。形の問題は警告して、
/// その条件か項目を外す。
pub fn read(v: &toml::Value, file: &str, w: &mut Vec<String>) -> Vec<Condition> {
    let Some(tab) = v.as_table() else {
        warn(w, file, "conditions", not(Msg::WantTable));
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, t) in tab {
        let path = format!("conditions.{name}");
        if !valid_name(name) {
            warn(w, file, &path, Msg::RsnCondName.text().to_string());
            continue;
        }
        let Some(t) = t.as_table() else {
            warn(w, file, &path, not(Msg::WantTable));
            continue;
        };
        let mut c = Condition {
            name: name.clone(),
            label: None,
            src: String::new(),
            key: None,
            hide: false,
        };
        let mut has_expr = false;
        for (k, v) in t {
            let p = format!("{path}.{k}");
            match k.as_str() {
                "expr" => match v.as_str() {
                    Some(s) => {
                        c.src = s.to_string();
                        has_expr = true;
                    }
                    None => warn(w, file, &p, not(Msg::WantString)),
                },
                "label" => match v.as_str() {
                    Some(s) if !s.trim().is_empty() => c.label = Some(s.to_string()),
                    _ => warn(w, file, &p, not(Msg::WantString)),
                },
                "key" => match v.as_integer().and_then(|n| u8::try_from(n).ok()) {
                    Some(d @ 1..=9) => c.key = Some(d),
                    _ => warn(w, file, &p, not(Msg::WantDigit)),
                },
                "hide" => match v.as_bool() {
                    Some(b) => c.hide = b,
                    None => warn(w, file, &p, not(Msg::WantBool)),
                },
                _ => warn(w, file, &p, Msg::RsnUnknown.text().to_string()),
            }
        }
        if has_expr {
            out.push(c);
        } else {
            warn(w, file, &path, Msg::RsnCondNeedExpr.text().to_string());
        }
    }
    out
}

// ---- 今の条件 ----

static GLOBAL: RwLock<Option<Arc<Conditions>>> = RwLock::new(None);

thread_local! {
    static LOCAL: RefCell<Option<Arc<Conditions>>> = const { RefCell::new(None) };
}

/// 起動のはじめに今の条件を置く(どこで式を読んでも `cond("名前")` が効くように)。
pub fn install(c: Arc<Conditions>) {
    *GLOBAL.write().unwrap_or_else(PoisonError::into_inner) = Some(c);
}

/// 今の条件(糸の置き換えが先。何も置いていなければ空)。
pub fn current() -> Arc<Conditions> {
    LOCAL
        .with(|l| l.borrow().clone())
        .or_else(|| {
            GLOBAL
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        })
        .unwrap_or_default()
}

/// 名前の条件の置き換え済みの式(expr::parse が引く)。
pub(crate) fn lookup(name: &str) -> Option<Expr> {
    current().expr(name).cloned()
}

/// この糸だけ今の条件を置き換える(試験)。落とすと元に戻る。
#[must_use]
pub fn scoped(c: Arc<Conditions>) -> Scoped {
    let prev = LOCAL.with(|l| l.replace(Some(c)));
    Scoped(prev)
}

pub struct Scoped(Option<Arc<Conditions>>);

impl Drop for Scoped {
    fn drop(&mut self) {
        let prev = self.0.take();
        LOCAL.with(|l| *l.borrow_mut() = prev);
    }
}

/// condition.columns-present: 式 `e` が使う列(`Expr::columns`)が全部、今の表にある(`has`)か。1つでも無ければ
/// 使えない(tbd~condition.columns-all)。列を使わない式はどの表でも使える。
pub fn usable(e: &Expr, has: &dyn Fn(&str) -> bool) -> bool {
    e.columns().iter().all(|c| has(c))
}

/// condition.unknown-name: 無い名前の理由(ある名前を添える)。
pub fn unknown_reason(name: &str) -> String {
    Msg::CondUnknown.fill(&[&name, &current().names()])
}

/// 設定の文から条件を読んで組む(起動のはじめ。警告は捨てる。設定を読み直すところで同じ警告が出る)。
pub fn peek(text: &str) -> Conditions {
    let Ok(table) = text.parse::<toml::Table>() else {
        return Conditions::default();
    };
    let mut w = Vec::new();
    match table.get("conditions") {
        Some(v) => Conditions::build(in_written_order(read(v, "", &mut w), text), "", &mut w),
        None => Conditions::default(),
    }
}
