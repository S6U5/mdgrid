//! 表に重ねる窓(操作の一覧・頻度表・リストの選択・候補)の共通の部品: 画面の大きさ、セルの下か上への置き方、
//! 枠の行、クリックの当たり、表の行への重ね方。窓ごとの中身(行の文字)は各モジュールが作る。

use super::app::App;
use super::keymap::Action;
use super::list::splice;
use super::width::{take, width};
use ratatui::style::Style;
use ratatui::text::{Line, Span};

/// 窓の上と下の縁の行の数。
pub(crate) const FRAME: usize = 2;
/// 窓の中に少なくとも見せる行の数(これより低い端末では窓を開かない)。
pub(crate) const MIN_VIS: usize = 3;

/// 描ける幅(右端の1桁を除く)と、下の帯より上の行の数(最下行と下の帯・メッセージ行を除く。mod.rs の draw)。
pub(crate) fn screen(app: &App) -> (usize, usize) {
    let w = app.size.0.saturating_sub(1) as usize;
    let limit = (app.size.1.saturating_sub(1) as usize).saturating_sub(2);
    (w, limit)
}

/// 窓の縦の置き場所。
pub(crate) struct Place {
    /// 窓の上の縁の行。
    pub top: usize,
    /// 中に見せる行の数。
    pub vis: usize,
}

/// `n` 行の中身の窓を、行 `ay` のセルの下(入らなければ上)に置く。`limit` は下の帯より上の行の数。
/// 少なくとも MIN_VIS 行(中身が少なければその数)見せられなければ None(SR-9)。
pub(crate) fn place(ay: usize, n: usize, limit: usize) -> Option<Place> {
    place_with(ay, n, limit, FRAME, MIN_VIS)
}

/// `place` の縁の行の数(`frame`。中身でない行を含む)と、少なくとも見せる行の数(`min_vis`)を変えたもの。
pub(crate) fn place_with(
    ay: usize,
    n: usize,
    limit: usize,
    frame: usize,
    min_vis: usize,
) -> Option<Place> {
    let below_room = limit.saturating_sub(ay + 1);
    let above_room = ay.min(limit);
    let below = below_room >= n + frame || below_room >= above_room;
    let room = if below { below_room } else { above_room };
    if room < frame + min_vis.min(n) {
        return None;
    }
    let vis = n.min(room - frame);
    let top = if below { ay + 1 } else { ay - (vis + frame) };
    Some(Place { top, vis })
}

/// 一覧の選びを動かす動作(↑↓・先頭・末尾・1画面)なら、`len` 行の一覧の新しい選び。ほかの動作なら None。
pub(crate) fn step_sel(sel: usize, len: usize, action: Action, page: usize) -> Option<usize> {
    let last = len.saturating_sub(1);
    Some(match action {
        Action::Up => sel.saturating_sub(1),
        Action::Down => (sel + 1).min(last),
        Action::PageUp => sel.saturating_sub(page),
        Action::PageDown => (sel + page).min(last),
        Action::Top => 0,
        Action::Bottom => last,
        _ => return None,
    })
}

/// 上の縁の行: `+題----+`(幅 `tw` に収める)。
pub(crate) fn top_edge(title: &str, tw: usize) -> String {
    let t = take(title, tw);
    format!("+{t}{}+", "-".repeat(tw - width(&t)))
}

/// (x, y) が窓のどこか: 窓の外なら None、窓の中なら中身の行の位置(0 から。縁なら None の中身)。
pub(crate) fn hit(
    x: u16,
    y: u16,
    gx: usize,
    top: usize,
    iw: usize,
    vis: usize,
) -> Option<Option<usize>> {
    let (x, y) = (x as usize, y as usize);
    if x < gx || x >= gx + iw || y < top || y >= top + vis + FRAME {
        return None;
    }
    let k = y - top;
    if k == 0 || k > vis {
        return Some(None);
    }
    Some(Some(k - 1))
}

/// 窓の行を、表の行(`lines`)の桁 [x, x + iw) に上から重ねる。
pub(crate) fn blit(
    lines: &mut [Line<'static>],
    x: usize,
    top: usize,
    iw: usize,
    w: usize,
    rows: Vec<(String, Style)>,
) {
    for (k, (t, st)) in rows.into_iter().enumerate() {
        if let Some(line) = lines.get_mut(top + k) {
            *line = splice(line, x, Span::styled(t, st), iw, w);
        }
    }
}
