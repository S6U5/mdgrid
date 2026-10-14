//! 表の行の高さと画面の位置(CV-8)。設定 `display.row_extra_lines` で、ノートの行を最大で「1 + その数」の
//! 行の高さまで伸ばす。行の高さは、見えている列のセルのうち一番多くの行が要るものに合わせ、行ごとに変わる。
//! まとまりの見出しと空き行は1行。`app.top` は slot の添字のまま(行の途中から描かない)。
//!
//! 行の位置を出す所(描く・クリック・入力の欄と窓・スクロール・ページ送り)は、ここの関数を通す。
//! 0 では、どの関数も今までの算術(1 slot = 画面の1行)をそのまま返す。

use super::app::App;
use super::cell::{cell_lines, shown};
use super::grid::Slot;
use super::view::visible_layout;

/// 行を伸ばす数(CV-8。ビューの設定の上書きを重ねた値)。
pub(crate) fn extra(app: &App) -> usize {
    usize::from(app.settings.display.over(&app.display).row_extra_lines)
}

/// slot `i` の行の数(1〜1 + extra)。ノートの行だけが伸びる。
pub(crate) fn height(app: &App, cols: &[(usize, usize)], i: usize) -> usize {
    let n = extra(app);
    if n == 0 {
        return 1;
    }
    let Some(Slot::Row(r)) = app.slots.get(i) else {
        return 1;
    };
    let Some(row) = app.rows.get(*r) else {
        return 1;
    };
    cols.iter()
        .map(|&(j, cw)| {
            let col = &app.cols[j];
            cell_lines(app, &shown(app, row, col), row, col, cw, n + 1).len()
        })
        .max()
        .unwrap_or(1)
        .clamp(1, n + 1)
}

/// 見えている slot の (添字, 表の中の y, 描く行の数)。表の高さ `h` まで並べ、一番下の行は入る分だけ描く。
pub(crate) fn frame_in(app: &App, cols: &[(usize, usize)], h: usize) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    let mut y = 0;
    for i in app.top..app.slots.len() {
        if y >= h {
            break;
        }
        let k = height(app, cols, i).min(h - y);
        out.push((i, y, k));
        y += k;
    }
    out
}

/// 見えている列と、見えている slot(`frame_in` の戻り)。
type Frame = (Vec<(usize, usize)>, Vec<(usize, usize, usize)>);

/// 今の画面の、見えている slot(`frame_in` を今の列と表の高さで)。
fn frame(app: &App) -> Frame {
    let (_, cols) = visible_layout(app);
    let f = frame_in(app, &cols, app.data_height());
    (cols, f)
}

/// slot `i` の1行目の、表の中の y。見えていなければ None。
pub(crate) fn slot_y(app: &App, i: usize) -> Option<usize> {
    if extra(app) == 0 {
        let h = app.data_height();
        return (i >= app.top && i < app.top + h).then(|| i - app.top);
    }
    let (_, f) = frame(app);
    f.iter().find(|&&(s, _, _)| s == i).map(|&(_, y, _)| y)
}

/// 表の中の y `dy` にある slot の添字(表の高さの中であることは呼ぶ側が確かめる)。行の無い所なら None
/// (0 では slot の数を超えた添字も返す。今までどおり呼ぶ側が数と比べる)。
pub(crate) fn slot_at(app: &App, dy: usize) -> Option<usize> {
    if extra(app) == 0 {
        return Some(app.top + dy);
    }
    let (_, f) = frame(app);
    f.iter()
        .find(|&&(_, y, k)| dy >= y && dy < y + k)
        .map(|&(i, _, _)| i)
}

/// ページ送り(PageUp・PageDown)で動く行の数: 画面に高さ全部が入っている slot の数(少なくとも1)。
/// 表の下の端より下は1行の slot が続くとして数える(0 では表の高さ)。
pub(crate) fn page(app: &App) -> usize {
    let h = app.data_height().max(1);
    if extra(app) == 0 {
        return h;
    }
    let (cols, f) = frame(app);
    let mut n = 0;
    let mut used = 0;
    for &(i, _, k) in &f {
        if k < height(app, &cols, i) {
            break;
        }
        n += 1;
        used += k;
    }
    if f.last().is_some_and(|&(i, _, _)| i + 1 >= app.slots.len()) {
        n += h.saturating_sub(used);
    }
    n.max(1)
}

/// 選んだ行(`app.row`)の高さ全部が見える `top`(SR-3 の縦の送り)。もう見えていれば今の `top`。
/// 画面より高い行は、その行を一番上に置く(上から入る分だけ見せる)。
pub(crate) fn top_for(app: &App) -> usize {
    if app.row < app.top {
        return app.row;
    }
    let (cols, f) = frame(app);
    let h = app.data_height().max(1);
    if f.iter()
        .any(|&(i, _, k)| i == app.row && k == height(app, &cols, i))
    {
        return app.top;
    }
    let mut t = app.row;
    let mut used = height(app, &cols, t);
    while t > 0 {
        let up = height(app, &cols, t - 1);
        if used + up > h {
            break;
        }
        t -= 1;
        used += up;
    }
    t
}
