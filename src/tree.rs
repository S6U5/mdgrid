//! 親子の並べ方(NV-27)。画面に依存しない。行の並びと、行ごとの親(解けた親の行)から、親の下に子を
//! 前順(親 → 子 → 孫)に並べ直し、行ごとの深さと子の有無を返す。兄弟の順は元の並びのまま(並べ替えは
//! 兄弟の中で効く)。親が並びに無い行と、親子が輪になった行は一番上の段に置く(行を落とさない)。

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// 並べ直した1行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub row: T,
    /// 一番上の段は 0。
    pub depth: usize,
    /// 並びの中に子を持つか。
    pub has_kids: bool,
    /// 並びの中の直の子の数(navigation.parent-child-count。孫は数えない)。
    pub kids: usize,
    /// 並びの中の親(一番上の段は None)。
    pub parent: Option<T>,
}

/// `rows` を親子の前順に並べ直す。`parent` は行の親(並びに無い行・自分を返してもよい)。
pub fn order<T: Clone + Eq + Hash>(rows: &[T], parent: impl Fn(&T) -> Option<T>) -> Vec<Node<T>> {
    let set: HashSet<&T> = rows.iter().collect();
    // 並びの中の親だけを使う。
    let mut par: HashMap<T, T> = HashMap::new();
    for r in rows {
        if let Some(p) = parent(r) {
            if p != *r && set.contains(&p) {
                par.insert(r.clone(), p);
            }
        }
    }
    // 輪を切る: 祖先をたどって自分に戻る行は、親を外して一番上の段にする。
    let mut cut: Vec<T> = Vec::new();
    for r in rows {
        let mut seen: HashSet<&T> = HashSet::new();
        let mut cur = r;
        while let Some(p) = par.get(cur) {
            if p == r {
                cut.push(r.clone());
                break;
            }
            if !seen.insert(p) {
                break;
            }
            cur = p;
        }
    }
    for r in &cut {
        par.remove(r);
    }
    let mut kids: HashMap<&T, Vec<&T>> = HashMap::new();
    for r in rows {
        if let Some(p) = par.get(r) {
            kids.entry(p).or_default().push(r);
        }
    }
    let mut out = Vec::with_capacity(rows.len());
    let mut stack: Vec<(&T, usize)> = rows
        .iter()
        .filter(|r| !par.contains_key(*r))
        .rev()
        .map(|r| (r, 0))
        .collect();
    while let Some((r, d)) = stack.pop() {
        let ks = kids.get(r).map_or(&[][..], Vec::as_slice);
        out.push(Node {
            row: r.clone(),
            depth: d,
            has_kids: !ks.is_empty(),
            kids: ks.len(),
            parent: par.get(r).cloned(),
        });
        for k in ks.iter().rev() {
            stack.push((k, d + 1));
        }
    }
    out
}

/// NV-28: 前順に並べた行の WBS の番号(一番上の段は 1 から、子は親の番号に `.` と兄弟の中の順)。
pub fn numbers<T: Clone + Eq + Hash>(nodes: &[Node<T>]) -> HashMap<T, String> {
    let mut out: HashMap<T, String> = HashMap::new();
    let mut count: HashMap<Option<T>, usize> = HashMap::new();
    for n in nodes {
        let k = count.entry(n.parent.clone()).or_insert(0);
        *k += 1;
        let num = match n.parent.as_ref().and_then(|p| out.get(p)) {
            Some(p) => format!("{p}.{k}"),
            None => k.to_string(),
        };
        out.insert(n.row.clone(), num);
    }
    out
}

/// NV-28: 子のある行ごとに、子孫の子の無い行の割合(`leaf`)の平均(小数は切り捨て)。
pub fn progress<T: Clone + Eq + Hash>(
    nodes: &[Node<T>],
    leaf: impl Fn(&T) -> u8,
) -> HashMap<T, u8> {
    // 子の無い行の割合を、祖先のすべてに足す。
    let parent: HashMap<&T, &T> = nodes
        .iter()
        .filter_map(|n| n.parent.as_ref().map(|p| (&n.row, p)))
        .collect();
    let mut sum: HashMap<&T, (u32, u32)> = HashMap::new();
    for n in nodes.iter().filter(|n| !n.has_kids) {
        let pct = u32::from(leaf(&n.row).min(100));
        let mut cur = parent.get(&n.row);
        let mut steps = 0;
        while let Some(p) = cur {
            let e = sum.entry(p).or_insert((0, 0));
            e.0 += pct;
            e.1 += 1;
            steps += 1;
            if steps > nodes.len() {
                break;
            }
            cur = parent.get(p);
        }
    }
    sum.into_iter()
        .map(|(r, (s, c))| (r.clone(), (s / c.max(1)) as u8))
        .collect()
}

/// navigation.parent-child-config: 印の `[[table]]` の `children = { key, status, done, closed, on_done, gantt }`。
/// 書いた表は、どのビューも `key` の列で親子に並べ(ビューの設定でオフにしたビューを除く)、親を完了にしたときの子孫
/// (cell-edit.children-done)と、ガントで子が親の期間をはみ出したとき(gantt.children-span-*)をここで決める。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildrenSpec {
    /// 親のノートへのリンクを書く列。
    pub key: String,
    /// 状態の列。None は書いていない(画面が、印の kanban の column か `status` を使う)。
    pub status: Option<String>,
    /// 完了の値(既定 `done`)。
    pub done: String,
    /// 終わったとみなす値(既定 `["done"]`)。
    pub closed: Vec<String>,
    /// 親を完了にしたときの子孫の扱い。
    pub on_done: OnDone,
    /// ガントで子孫の棒が親の期間からはみ出したときの扱い。
    pub gantt: SpanRule,
}

/// cell-edit.children-done: 親を完了の値に直したときの、まだ終わっていない子孫の扱い。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnDone {
    /// 子孫も完了の値にする(cell-edit.children-done-complete)。
    #[default]
    Complete,
    /// 親を直さない(cell-edit.children-done-refuse)。
    Refuse,
    /// 親だけ直して、残りの数を知らせる(cell-edit.children-done-keep)。
    Keep,
}

impl OnDone {
    pub const NAMES: [&'static str; 3] = ["complete", "refuse", "keep"];

    pub fn parse(s: &str) -> Option<OnDone> {
        match s {
            "complete" => Some(OnDone::Complete),
            "refuse" => Some(OnDone::Refuse),
            "keep" => Some(OnDone::Keep),
            _ => None,
        }
    }
}

/// cell-edit.children-reopen: 完了の親を完了でない値に戻したときの、親の完了で一緒に閉じた子孫の扱い。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnReopen {
    /// 子孫を変えない(既定)。
    #[default]
    Keep,
    /// `closed_by` がその親を指す子孫を、親と同じ値に戻して `closed_by` を空にする。
    Reopen,
    /// その場で「子も戻す」「親だけ」を選ぶ。
    Ask,
}

impl OnReopen {
    pub const NAMES: [&'static str; 3] = ["keep", "reopen", "ask"];

    pub fn parse(s: &str) -> Option<OnReopen> {
        match s {
            "keep" => Some(OnReopen::Keep),
            "reopen" => Some(OnReopen::Reopen),
            "ask" => Some(OnReopen::Ask),
            _ => None,
        }
    }
}

/// cell-edit.children-closed-by・cell-edit.children-reopen: 印の `children` の `closed_by` と `on_reopen`。
/// `ChildrenSpec` に項目を足すと、項目を並べて組み立てる前からの試験が組み立てられなくなるので、別に持つ。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReopenSpec {
    /// 親の完了で一緒に完了にした子孫に、親へのリンクを書く列。既定の名前は無い(書いた表だけが書く。
    /// tbd~cell-edit.children-closed-by-name の仮決め)。
    pub closed_by: Option<String>,
    /// 完了の親を戻したときの子孫の扱い。`closed_by` が無ければ `Keep`(読むときに警告して直す)。
    pub on_reopen: OnReopen,
}

/// gantt.children-span-mark・gantt.children-span-extend: 子孫の棒が親の期間からはみ出したときの扱い。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpanRule {
    /// 印を付ける(既定。tbd~gantt.children-span-default の仮決め)。
    #[default]
    Mark,
    /// 子の棒を動かす・伸ばすとき、親を子孫の範囲まで広げる。
    Extend,
    /// 何もしない。
    None,
}

impl SpanRule {
    pub const NAMES: [&'static str; 3] = ["mark", "extend", "none"];

    pub fn parse(s: &str) -> Option<SpanRule> {
        match s {
            "mark" => Some(SpanRule::Mark),
            "extend" => Some(SpanRule::Extend),
            "none" => Some(SpanRule::None),
            _ => None,
        }
    }
}

/// 完了の値の既定。
pub const DEFAULT_DONE: &str = "done";

impl ChildrenSpec {
    /// `key` だけを書いたときの設定(ほかは既定)。
    pub fn new(key: &str) -> ChildrenSpec {
        ChildrenSpec {
            key: key.to_string(),
            status: None,
            done: DEFAULT_DONE.to_string(),
            closed: vec![DEFAULT_DONE.to_string()],
            on_done: OnDone::default(),
            gantt: SpanRule::default(),
        }
    }

    /// 値が終わったとみなす値か(`closed` に入るか、完了の値そのもの)。前後の空白は見ない。
    /// 完了の値は `closed` に書かなくても終わったとみなす(完了の値の子を「終わっていない」と数えると、`refuse` で
    /// 親をいつまでも完了にできないため)。
    pub fn is_closed(&self, value: &str) -> bool {
        let v = value.trim();
        v == self.done || self.closed.iter().any(|c| c == v)
    }
}

/// `root` の子孫(`kids` は行 → 子の並び)を前順に。輪になっていても1行は1回だけで、`root` は入れない。
pub fn descendants<T: Clone + Eq + Hash>(root: &T, kids: &HashMap<T, Vec<T>>) -> Vec<T> {
    let mut out = Vec::new();
    let mut seen: HashSet<&T> = HashSet::from([root]);
    let mut stack: Vec<&T> = kids
        .get(root)
        .map(|k| k.iter().rev().collect())
        .unwrap_or_default();
    while let Some(r) = stack.pop() {
        if !seen.insert(r) {
            continue;
        }
        out.push(r.clone());
        if let Some(k) = kids.get(r) {
            stack.extend(k.iter().rev());
        }
    }
    out
}
