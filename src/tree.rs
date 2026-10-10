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
        let ks = kids.get(r).map(Vec::as_slice).unwrap_or(&[]);
        out.push(Node {
            row: r.clone(),
            depth: d,
            has_kids: !ks.is_empty(),
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
