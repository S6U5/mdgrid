//! 表に重ねる窓(操作の一覧・頻度表・リストの選択・候補)の共通の部品: 画面の大きさ、セルの下か上への置き方、
//! 枠の行、クリックの当たり、表の行への重ね方。窓ごとの中身(行の文字)は各モジュールが作る。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::list::splice;
use super::width::{take, width};
use mdgrid::i18n::Msg;
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

/// 窓の枠の線の文字(SR-32)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Frame {
    /// 左上・右上・左下・右下。
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    /// 横・縦。
    pub h: char,
    pub v: char,
    /// 左右の区切り(枠の途中の横の線)。
    pub lt: char,
    pub rt: char,
}

/// 角の丸い、つながった罫線(既定)。
pub(crate) const ROUNDED: Frame = Frame {
    tl: '╭',
    tr: '╮',
    bl: '╰',
    br: '╯',
    h: '─',
    v: '│',
    lt: '├',
    rt: '┤',
};

/// ASCII(設定の `borders = "ascii"` と、East Asian Ambiguous を幅2とする設定 CV-6)。
pub(crate) const ASCII: Frame = Frame {
    tl: '+',
    tr: '+',
    bl: '+',
    br: '+',
    h: '-',
    v: '|',
    lt: '+',
    rt: '+',
};

/// 今の設定の枠の文字(SR-32): 既定は罫線。ambiguous_wide なら ASCII(罫線は
/// East Asian Ambiguous なので、幅2で描く端末では列がずれる)。
pub(crate) fn frame(app: &App) -> Frame {
    use mdgrid::style::Frames;
    if app.ambiguous_wide {
        return ASCII;
    }
    // SR-36 の `frames`。線の無い枠も、枠の幅は変えない(中身の位置とクリックの位置を変えない)。
    let (tl, tr, bl, br, h, v, lt, rt) = match app.style.frames {
        Frames::Rounded => return ROUNDED,
        Frames::Ascii => return ASCII,
        Frames::Square => ('┌', '┐', '└', '┘', '─', '│', '├', '┤'),
        Frames::Heavy => ('┏', '┓', '┗', '┛', '━', '┃', '┣', '┫'),
        Frames::None_ => (' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '),
    };
    Frame {
        tl,
        tr,
        bl,
        br,
        h,
        v,
        lt,
        rt,
    }
}

impl Frame {
    /// 横の線を n 個。
    pub(crate) fn line(&self, n: usize) -> String {
        std::iter::repeat_n(self.h, n).collect()
    }

    /// 中身の行(左右に縦の線)。
    pub(crate) fn side(&self, inner: &str) -> String {
        format!("{}{inner}{}", self.v, self.v)
    }

    /// 下の縁(幅 `tw` の中身の下)。
    pub(crate) fn bottom(&self, tw: usize) -> String {
        format!("{}{}{}", self.bl, self.line(tw), self.br)
    }
}

/// 上の縁の行: 左上・題・横の線・右上(幅 `tw` に収める)。
pub(crate) fn top_edge(f: Frame, title: &str, tw: usize) -> String {
    let t = take(title, tw);
    format!("{}{t}{}{}", f.tl, f.line(tw - width(&t)), f.tr)
}

/// チェックの窓の一番下の「[ 決める ]」の行の字(screen.checklist-keys)。動作の名前は決めるキーの表示名と同じ文言。
pub(crate) fn decide_label() -> String {
    format!("[ {} ]", Msg::KeyDecide.text())
}

// ---- 下の縁のキーの案内(screen.window-key-hints) ----

/// 窓の下の縁に並べるキーの案内の1つ: 「キー 動作」の字と、押した(クリックした)ときの動作。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KeyHint {
    pub text: String,
    pub action: Action,
}

/// モード `mode` の動作の案内を、並べたい順に作る。キーはキーの表(割り当て直したあと)の最初のキー、動作の名前は
/// 添えた文言(窓の段で意味が変わるとき)か、無ければ表の表示名。キーを外した動作は出さない(SR-4・SR-13)。
pub(crate) fn key_hints(app: &App, mode: Mode, items: &[(Action, Option<Msg>)]) -> Vec<KeyHint> {
    items
        .iter()
        .filter_map(|&(action, label)| {
            let b = app
                .keys
                .iter()
                .find(|b| b.mode == mode && b.action == action)?;
            let name = label.map_or_else(|| b.text(), Msg::text);
            Some(KeyHint {
                text: format!("{} {name}", keymap::display(b.key)),
                action,
            })
        })
        .collect()
}

/// 案内を全部入れるのに要る縁の中(角の間)の幅: 線1・空白1・案内(間は2桁)・空白1・線1。案内が無ければ 0。
pub(crate) fn hints_width(hints: &[KeyHint]) -> usize {
    if hints.is_empty() {
        return 0;
    }
    hints.iter().map(|h| width(&h.text) + 2).sum::<usize>() + 2
}

/// 窓の幅(縁を含む): 中身の幅 `content` を、下の縁の案内が全部入る幅(縁の中の幅 `edge` と縁2桁)まで広げ、画面の
/// 幅 `w` に収める。下の縁も窓の中身なので、右へはみ出すなら中身と同じく左へずらす(置き場は各窓のまま)。画面が狭くて
/// 入らない案内は、描くときに後ろから省く。
pub(crate) fn widen_for_hints(content: usize, edge: usize, w: usize) -> usize {
    if edge == 0 {
        return content.min(w);
    }
    content.max(edge + 2).min(w)
}

/// 縁の中の幅 `tw` に入る案内の置き場(案内の添字と、縁の中の左の端からの桁)。`lead` 桁(下の縁の数など)のあと、
/// 線1・空白1を空けて前から並べ、右に空白1・線1が残らない案内からあとは省く(縁の線と字を重ねない)。
fn hint_spots(hints: &[KeyHint], tw: usize, lead: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut at = lead + 2;
    for (i, h) in hints.iter().enumerate() {
        let w = width(&h.text);
        if at + w + 2 > tw {
            break;
        }
        out.push((i, at));
        at += w + 2;
    }
    out
}

/// 下の縁の行(角を含む): 左に `lead`(選んでいる番号/数など)、その後ろに入るだけの案内。案内が1つも入らなければ
/// 今までどおり `lead` と線だけ。
pub(crate) fn bottom_hints(f: Frame, tw: usize, lead: &str, hints: &[KeyHint]) -> String {
    let lead = take(lead, tw);
    let lw = width(&lead);
    let spots = hint_spots(hints, tw, lw);
    let mut s = format!("{}{lead}", f.bl);
    let mut x = lw;
    for (i, at) in spots {
        // 最初の案内の前は線1・空白1、案内の間は空白2。
        if x == lw {
            s.push(f.h);
            s.push(' ');
        } else {
            s.push_str("  ");
        }
        s.push_str(&hints[i].text);
        x = at + width(&hints[i].text);
    }
    if x > lw {
        s.push(' ');
        x += 1;
    }
    s.push_str(&f.line(tw - x));
    s.push(f.br);
    s
}

/// 下の縁の桁 `dx`(窓の左の端からの桁。左下の角が 0)にある案内の動作。案内の上でなければ None。
pub(crate) fn hint_at(hints: &[KeyHint], tw: usize, lead: &str, dx: usize) -> Option<Action> {
    let c = dx.checked_sub(1)?;
    let lw = width(&take(lead, tw));
    hint_spots(hints, tw, lw)
        .into_iter()
        .find(|&(i, at)| (at..at + width(&hints[i].text)).contains(&c))
        .map(|(i, _)| hints[i].action)
}

/// (x, y) が、左の端 `gx`・幅 `iw` の窓の下の縁の行 `bottom` の上なら、窓の左の端からの桁。
pub(crate) fn on_bottom(x: u16, y: u16, gx: usize, iw: usize, bottom: usize) -> Option<usize> {
    let (x, y) = (x as usize, y as usize);
    (y == bottom && (gx..gx + iw).contains(&x)).then(|| x - gx)
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
    app: &App,
    lines: &mut [Line<'static>],
    x: usize,
    top: usize,
    iw: usize,
    w: usize,
    rows: Vec<(String, Style)>,
) {
    let look = super::look::look(app);
    for (k, (t, st)) in rows.into_iter().enumerate() {
        let Some(line) = lines.get_mut(top + k) else {
            continue;
        };
        *line = match &look {
            None => splice(line, x, Span::styled(t, st), iw, w),
            Some(l) => modern_row(line, x, &t, st, iw, w, l),
        };
    }
}

/// モダンな見た目の窓の1行(SR-33): 縁の線はアクセントの色、縁の上の題は太字、中身の反転は背景の色。
fn modern_row(
    line: &Line<'static>,
    x: usize,
    t: &str,
    st: Style,
    iw: usize,
    w: usize,
    l: &super::look::Look,
) -> Line<'static> {
    let chars: Vec<char> = t.chars().collect();
    // 縁の文字は frames(SR-36)のどの形でも(丸・角・太い・ASCII)。
    let edge = |c: char| "╭╮╰╯┌┐└┘┏┓┗┛├┤┣┫+".contains(c);
    let line_char = |c: char| "─━-".contains(c);
    let (Some(&first), Some(&last)) = (chars.first(), chars.last()) else {
        return splice(line, x, Span::styled(t.to_string(), st), iw, w);
    };
    // 上と下の縁(角で始まる行): 線はアクセントの色、線でない文字(題・数)は太字のアクセント。
    if edge(first) && edge(last) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut buf = String::new();
        let mut on_line = true;
        for (i, c) in chars.iter().enumerate() {
            let is_line = i == 0 || i == chars.len() - 1 || line_char(*c);
            if is_line != on_line && !buf.is_empty() {
                let s = if on_line {
                    l.border()
                } else {
                    l.border().add_modifier(ratatui::style::Modifier::BOLD)
                };
                spans.push(Span::styled(std::mem::take(&mut buf), s));
            }
            on_line = is_line;
            buf.push(*c);
        }
        if !buf.is_empty() {
            let s = if on_line {
                l.border()
            } else {
                l.border().add_modifier(ratatui::style::Modifier::BOLD)
            };
            spans.push(Span::styled(buf, s));
        }
        return splice_spans(line, x, spans, iw, w);
    }
    // 中身の行: 左右の縦の線と、中身。
    if "│┃|".contains(first) && "│┃|".contains(last) && chars.len() >= 2 {
        let body: String = chars[1..chars.len() - 1].iter().collect();
        let spans = vec![
            Span::styled(first.to_string(), l.border()),
            Span::styled(body, l.swap_reverse(st)),
            Span::styled(last.to_string(), l.border()),
        ];
        return splice_spans(line, x, spans, iw, w);
    }
    splice(
        line,
        x,
        Span::styled(t.to_string(), l.swap_reverse(st)),
        iw,
        w,
    )
}

/// 行の桁 [x, x + iw) を、幅を足して iw になる span の並びで置き換える。
fn splice_spans(
    line: &Line<'static>,
    x: usize,
    spans: Vec<Span<'static>>,
    iw: usize,
    w: usize,
) -> Line<'static> {
    let mut out = line.clone();
    let mut at = x;
    let n = spans.len();
    for (i, s) in spans.into_iter().enumerate() {
        let sw = if i + 1 == n {
            (x + iw).saturating_sub(at)
        } else {
            width(&s.content)
        };
        out = splice(&out, at, s, sw, w);
        at += sw;
    }
    out
}
