//! 保存の前の差分(WB-9)。行単位の簡単な差分: 前と後で違う行だけを `-` / `+` で出し、前後1行の文脈を付ける。
//! 書き戻しは値の範囲(とキーを足す1行)だけを変えるので、共通の先頭と末尾を除いた残りは小さい。
//! 残りが大きいときだけ、LCS をあきらめて全部を `-` と `+` で出す。

/// 差分の1行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    Same(String),
    Del(String),
    Add(String),
    /// 省いた行がある。
    Gap,
}

/// LCS を使う上限(前の行数 × 後の行数)。
const LCS_LIMIT: usize = 250_000;
/// 変わった行の前後に付ける文脈の行数。
const CONTEXT: usize = 1;

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Same(usize),
    Del(usize),
    Add(usize),
}

fn lines(b: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(b);
    let mut out: Vec<String> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
        .collect();
    // 末尾の改行のあとの空の1行は除く。
    if out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out
}

/// 中ほど(共通の先頭と末尾を除いた部分)の並び。
fn middle(a: &[String], b: &[String], pa: usize, pb: usize) -> Vec<Op> {
    let (n, m) = (a.len(), b.len());
    if n == 0 || m == 0 || n * m > LCS_LIMIT {
        let mut ops: Vec<Op> = (0..n).map(|i| Op::Del(pa + i)).collect();
        ops.extend((0..m).map(|j| Op::Add(pb + j)));
        return ops;
    }
    // dp[i][j] = a[i..] と b[j..] の LCS の長さ。
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut ops = Vec::new();
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            ops.push(Op::Same(pa + i));
            i += 1;
            j += 1;
        } else if j < m && (i == n || dp[i][j + 1] >= dp[i + 1][j]) {
            ops.push(Op::Add(pb + j));
            j += 1;
        } else {
            ops.push(Op::Del(pa + i));
            i += 1;
        }
    }
    // 同じ場所の `-` を `+` より先に並べる。
    let mut k = 0;
    while k < ops.len() {
        let start = k;
        while k < ops.len() && !matches!(ops[k], Op::Same(_)) {
            k += 1;
        }
        ops[start..k].sort_by_key(|o| matches!(o, Op::Add(_)));
        k += 1;
    }
    ops
}

/// 前と後のバイト列の差分。変わった行が無ければ空。
pub fn diff(before: &[u8], after: &[u8]) -> Vec<DiffLine> {
    let a = lines(before);
    let b = lines(after);
    let mut p = 0;
    while p < a.len() && p < b.len() && a[p] == b[p] {
        p += 1;
    }
    let mut s = 0;
    while s < a.len() - p && s < b.len() - p && a[a.len() - 1 - s] == b[b.len() - 1 - s] {
        s += 1;
    }
    let mut ops: Vec<Op> = (0..p).map(Op::Same).collect();
    ops.extend(middle(&a[p..a.len() - s], &b[p..b.len() - s], p, p));
    ops.extend((a.len() - s..a.len()).map(Op::Same));

    // 変わった行から CONTEXT 行以内の Same だけを残す。
    let changed: Vec<usize> = ops
        .iter()
        .enumerate()
        .filter(|(_, o)| !matches!(o, Op::Same(_)))
        .map(|(i, _)| i)
        .collect();
    if changed.is_empty() {
        return Vec::new();
    }
    let keep = |i: usize| {
        changed
            .iter()
            .any(|&c| i + CONTEXT >= c && i <= c + CONTEXT)
    };
    let mut out = Vec::new();
    let mut skipped = false;
    for (i, o) in ops.iter().enumerate() {
        if !keep(i) {
            skipped = true;
            continue;
        }
        if skipped && !out.is_empty() {
            out.push(DiffLine::Gap);
        }
        skipped = false;
        out.push(match *o {
            Op::Same(k) => DiffLine::Same(a[k].clone()),
            Op::Del(k) => DiffLine::Del(a[k].clone()),
            Op::Add(k) => DiffLine::Add(b[k].clone()),
        });
    }
    out
}

#[cfg(test)]
#[path = "ui/test_diff.rs"]
mod tests;
