//! 関係マップ(REL-7): ワークスペースの表の形とつながりを読み、箱と矢印を文字の盤に並べる純関数。
//!
//! - 読み取り(`read`): 表ごとのノートの数とフロントマターの列(多い順)、表どうしのつながり(REL-6 と同じ
//!   見つけ方)と、つながりごとの実際のリンクの組(元のノート, 行き先のノート)。
//! - 並べ方(`layout`): 左から右の層(つながりの元が左、行き先が右)。箱は「題(名前と行の数)・区切り・列」。
//!   つながりの列は先に並べ `→ 行き先` を添える。矢印はつながりの列の行から出て、行き先の箱の題の行に入る
//!   (`▶`)。元の側に `N`、行き先の側に `1`(多対一)か `N`(多対多)。行き先が同じか左の層なら、元の箱の
//!   左を回る。盤の各セルは役割(枠・題・列・つながりの列・線・矢印・数の札)と、どの表・どのつながりかを持つ。

use crate::frontmatter::{self, Value};
use crate::relations::{file_stem, parse, resolve, strings, table_of, Notes, Table};
use std::path::{Path, PathBuf};

/// 1つのつながりで覚えるリンクの組の上限。
const MAX_PAIRS: usize = 500;
/// 箱に出す列の数の上限(超えたら `+N`)。
const MAX_FIELDS: usize = 6;
/// 箱の幅の上限。
const MAX_BOX_W: usize = 34;

/// 表の形。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableInfo {
    pub name: String,
    pub dir: PathBuf,
    /// その表のノートの数。
    pub rows: usize,
    /// フロントマターの列(多い順、同じ数は最初に出た順)。
    pub columns: Vec<String>,
}

/// つながり(REL-6)と、実際のリンクの組。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub from: String,
    pub column: String,
    pub to: String,
    /// リストの列(多対多)。
    pub many: bool,
    /// (元のノート, 行き先のノート)。最大 MAX_PAIRS。
    pub pairs: Vec<(PathBuf, PathBuf)>,
}

fn entries(path: &Path) -> Vec<(String, Value)> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    match frontmatter::parse(&bytes) {
        Ok(fm) => fm.entries.into_iter().map(|e| (e.key, e.value)).collect(),
        Err(_) => Vec::new(),
    }
}

/// ワークスペースの表の形とつながりを読む。表のフォルダが入れ子なら、ノートはいちばん深い表のもの。
pub fn read(tables: &[Table]) -> (Vec<TableInfo>, Vec<Link>) {
    let scanned: Vec<Notes> = tables.iter().map(|t| Notes::scan(&t.dir)).collect();
    let all: Vec<&Notes> = scanned.iter().collect();
    let mut infos: Vec<TableInfo> = Vec::new();
    let mut links: Vec<Link> = Vec::new();
    for (i, t) in tables.iter().enumerate() {
        let n = &scanned[i];
        let mut rows = 0usize;
        let mut cols: Vec<(String, usize)> = Vec::new();
        for rel in &n.rels {
            let note = n.root.join(rel);
            if table_of(&note, tables).map(|x| &x.name) != Some(&t.name) {
                continue;
            }
            rows += 1;
            let mut order: Vec<&Notes> = vec![n];
            order.extend(all.iter().filter(|x| !std::ptr::eq(**x, n)));
            for (col, v) in entries(&note) {
                match cols.iter_mut().find(|(c, _)| *c == col) {
                    Some((_, k)) => *k += 1,
                    None => cols.push((col.clone(), 1)),
                }
                let many = matches!(v, Value::List(_));
                for s in strings(&v) {
                    let Some(target) = parse(s).and_then(|l| resolve(&l, &note, &order)) else {
                        continue;
                    };
                    let Some(to) = table_of(&target, tables) else {
                        continue;
                    };
                    let k = match links
                        .iter()
                        .position(|e| e.from == t.name && e.column == col && e.to == to.name)
                    {
                        Some(k) => k,
                        None => {
                            links.push(Link {
                                from: t.name.clone(),
                                column: col.clone(),
                                to: to.name.clone(),
                                many: false,
                                pairs: Vec::new(),
                            });
                            links.len() - 1
                        }
                    };
                    let e = &mut links[k];
                    e.many |= many;
                    if e.pairs.len() < MAX_PAIRS {
                        e.pairs.push((note.clone(), target));
                    }
                }
            }
        }
        // 多い順(安定な並べ替えなので、同じ数は最初に出た順)。
        cols.sort_by_key(|c| std::cmp::Reverse(c.1));
        infos.push(TableInfo {
            name: t.name.clone(),
            dir: t.dir.clone(),
            rows,
            columns: cols.into_iter().map(|(c, _)| c).collect(),
        });
    }
    (infos, links)
}

/// 盤のセルの役割(色を付けるのに使う)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Empty,
    /// 箱の枠。
    Frame,
    /// 箱の題(名前と行の数)。
    Title,
    /// 列。
    Field,
    /// つながりの列(`→ 行き先` を含む)。
    Link,
    /// 矢印の線。
    Line,
    /// 矢印の先。
    Arrow,
    /// 数の札(`N`・`1`)。
    Label,
}

/// 盤の1セル。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub role: Role,
    /// どの表の箱か(箱のセル)。
    pub table: Option<usize>,
    /// どのつながりか(線・矢印・札・つながりの列)。
    pub link: Option<usize>,
}

const EMPTY: Cell = Cell {
    ch: ' ',
    role: Role::Empty,
    table: None,
    link: None,
};

/// 箱の置き場所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

/// 並べた盤。
#[derive(Debug, Clone)]
pub struct Map {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Vec<Cell>>,
    /// 表ごとの箱(infos と同じ添字)。
    pub boxes: Vec<Placed>,
}

impl Map {
    /// 行 y の文字。
    pub fn line(&self, y: usize) -> String {
        self.cells[y].iter().map(|c| c.ch).collect()
    }

    fn get(&self, x: usize, y: usize) -> Cell {
        self.cells
            .get(y)
            .and_then(|r| r.get(x))
            .copied()
            .unwrap_or(EMPTY)
    }

    fn set(&mut self, x: usize, y: usize, c: Cell) {
        if let Some(cell) = self.cells.get_mut(y).and_then(|r| r.get_mut(x)) {
            *cell = c;
        }
    }
}

/// 盤の線の文字。
#[derive(Debug, Clone, Copy)]
struct Glyphs {
    tl: char,
    tr: char,
    bl: char,
    br: char,
    h: char,
    v: char,
    lt: char,
    rt: char,
    cross: char,
    /// 曲がり角: 右から下(┐)・右から上(┘)・下から右(└)・上から右(┌)。
    rd: char,
    ru: char,
    dr: char,
    ur: char,
    right: char,
    left: char,
    arrow_r: &'static str,
}

const ROUND: Glyphs = Glyphs {
    tl: '╭',
    tr: '╮',
    bl: '╰',
    br: '╯',
    h: '─',
    v: '│',
    lt: '├',
    rt: '┤',
    cross: '┼',
    rd: '┐',
    ru: '┘',
    dr: '└',
    ur: '┌',
    right: '▶',
    left: '◀',
    arrow_r: "→",
};

const PLAIN: Glyphs = Glyphs {
    tl: '+',
    tr: '+',
    bl: '+',
    br: '+',
    h: '-',
    v: '|',
    lt: '+',
    rt: '+',
    cross: '+',
    rd: '+',
    ru: '+',
    dr: '+',
    ur: '+',
    right: '>',
    left: '<',
    arrow_r: "->",
};

/// 文字の幅(East Asian Wide/Fullwidth は2)。
fn cw(c: char) -> usize {
    use unicode_width::UnicodeWidthChar;
    c.width().unwrap_or(0)
}

fn sw(s: &str) -> usize {
    s.chars().map(cw).sum()
}

/// 制御文字(C0・DEL・C1)を `?` にする(ノートの列の名前などが端末を操作しないように)。
fn clean(s: &str) -> String {
    // 制御文字と、文字の向きを変える書式の文字(U+200E・U+200F・U+202A〜U+202E・U+2066〜U+2069・U+2028・U+2029)は
    // `?`(表示の行を並べ替えさせない)。
    s.chars()
        .map(|c| {
            let bidi = matches!(c, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{2028}' | '\u{2029}');
            if c.is_control() || bidi { '?' } else { c }
        })
        .collect()
}

/// 幅 `w` に収める(はみ出したら `…`)。制御文字は `?`。
fn clip(s: &str, w: usize) -> String {
    let s = &clean(s);
    if sw(s) <= w {
        return s.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        if used + cw(c) + 1 > w {
            break;
        }
        out.push(c);
        used += cw(c);
    }
    out.push('…');
    out
}

/// 箱の中の行: (文字, 役割, つながりの添字)。
fn box_lines(
    info: &TableInfo,
    links: &[Link],
    idx: usize,
    g: &Glyphs,
) -> Vec<(String, Role, Option<usize>)> {
    let _ = idx;
    let mut out = Vec::new();
    let mut shown: Vec<String> = Vec::new();
    for (k, l) in links.iter().enumerate() {
        if l.from == info.name && !shown.contains(&l.column) {
            shown.push(l.column.clone());
            out.push((
                format!("{} {} {}", l.column, g.arrow_r, l.to),
                Role::Link,
                Some(k),
            ));
        }
    }
    let rest: Vec<&String> = info.columns.iter().filter(|c| !shown.contains(c)).collect();
    let room = MAX_FIELDS.saturating_sub(out.len());
    for c in rest.iter().take(room) {
        out.push(((*c).clone(), Role::Field, None));
    }
    if rest.len() > room {
        out.push((format!("+{}", rest.len() - room), Role::Field, None));
    }
    out
}

/// 箱と矢印を盤に並べる(REL-7)。`ascii` なら ASCII の線(SR-32)。
pub fn layout(infos: &[TableInfo], links: &[Link], ascii: bool) -> Map {
    let g = if ascii { PLAIN } else { ROUND };
    let n = infos.len();
    let index = |name: &str| infos.iter().position(|t| t.name == name);
    // 層: つながりの元より1つ右(輪は n 回で打ち切る)。
    let mut layer = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for l in links {
            let (Some(a), Some(b)) = (index(&l.from), index(&l.to)) else {
                continue;
            };
            if a != b && layer[b] < layer[a] + 1 && layer[a] + 1 < n {
                layer[b] = layer[a] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let ncol = layer.iter().copied().max().map_or(0, |m| m + 1);
    // 箱の中身と大きさ。
    let contents: Vec<Vec<(String, Role, Option<usize>)>> = infos
        .iter()
        .enumerate()
        .map(|(i, t)| box_lines(t, links, i, &g))
        .collect();
    let titles: Vec<String> = infos
        .iter()
        .map(|t| format!("{} ({})", t.name, t.rows))
        .collect();
    let sizes: Vec<(usize, usize)> = (0..n)
        .map(|i| {
            let inner = contents[i]
                .iter()
                .map(|(s, _, _)| sw(s))
                .chain(std::iter::once(sw(&titles[i])))
                .max()
                .unwrap_or(0);
            let w = (inner + 4).clamp(12, MAX_BOX_W);
            // 上の縁・題・区切り・列・下の縁。
            let h = 3 + contents[i].len().max(1) + 1;
            (w, h)
        })
        .collect();
    // 矢印の通り道(隙間の縦の道)。隙間 g(0..=ncol)は列 g の左(0 は左の余白、ncol は右の余白)。
    // 隣の層へは行き先の左の隙間。離れた層と、左か同じ層へは、箱の下の横の道(下の道)を通る。
    #[derive(Clone, Copy)]
    enum Kind {
        /// 隣の右の層へ: (隙間, 道)。
        Next(usize, usize),
        /// 同じ層へ: 元の左の隙間の道。
        Same(usize, usize),
        /// 1つ左の層へ: 元の左の隙間(行き先の右)の道。
        Prev(usize, usize),
        /// 離れた層へ: (出る隙間, 道, 入る隙間, 道, 下の道)。
        Far(usize, usize, usize, usize, usize),
    }
    let mut gap_edges: Vec<usize> = vec![0; ncol + 1];
    let take = |gi: usize, gap_edges: &mut Vec<usize>| {
        let k = gap_edges[gi];
        gap_edges[gi] += 1;
        k
    };
    let mut lanes = 0usize;
    let mut kinds: Vec<Option<Kind>> = vec![None; links.len()];
    for (k, l) in links.iter().enumerate() {
        let (Some(a), Some(b)) = (index(&l.from), index(&l.to)) else {
            continue;
        };
        if a == b {
            continue;
        }
        let (la, lb) = (layer[a], layer[b]);
        kinds[k] = Some(if lb == la + 1 {
            Kind::Next(lb, take(lb, &mut gap_edges))
        } else if lb == la {
            Kind::Same(la, take(la, &mut gap_edges))
        } else if lb + 1 == la {
            Kind::Prev(la, take(la, &mut gap_edges))
        } else {
            // 出る隙間: 右へなら元の右、左へなら元の左。入る隙間: 右へなら行き先の左、左へなら行き先の右。
            let (go, gin) = if lb > la { (la + 1, lb) } else { (la, lb + 1) };
            let so = take(go, &mut gap_edges);
            let si = take(gin, &mut gap_edges);
            lanes += 1;
            Kind::Far(go, so, gin, si, lanes - 1)
        });
    }
    let gap_w = |gi: usize| -> usize {
        let k = gap_edges[gi];
        if gi == 0 {
            if k == 0 {
                0
            } else {
                3 + 2 * k
            }
        } else {
            // 道(2桁ずつ)と、行き先の箱の前の `1▶` の2桁。
            3 + 2 * k.max(1)
        }
    };
    // 列ごとの幅と x。
    let mut col_w = vec![0usize; ncol];
    for i in 0..n {
        col_w[layer[i]] = col_w[layer[i]].max(sizes[i].0);
    }
    let mut col_x = vec![0usize; ncol];
    let mut x = gap_w(0);
    for c in 0..ncol {
        col_x[c] = x;
        x += col_w[c] + if c + 1 < ncol { gap_w(c + 1) } else { 0 };
    }
    // 右の余白の隙間(右端の列から出る道があるとき)。
    let width =
        x + if ncol > 0 && gap_edges[ncol] > 0 {
            gap_w(ncol)
        } else {
            0
        } + 1;
    // 列の中で上から積む(1行空ける)。
    let mut boxes = vec![
        Placed {
            x: 0,
            y: 0,
            w: 0,
            h: 0
        };
        n
    ];
    let mut col_y = vec![0usize; ncol];
    for i in 0..n {
        let c = layer[i];
        boxes[i] = Placed {
            x: col_x[c],
            y: col_y[c],
            w: sizes[i].0,
            h: sizes[i].1,
        };
        col_y[c] += sizes[i].1 + 1;
    }
    let boxes_h = col_y.iter().copied().max().unwrap_or(1).max(1);
    // 下の道は箱の下に1行ずつ。
    let height = boxes_h + lanes + usize::from(lanes > 0);
    let mut map = Map {
        w: width,
        h: height,
        cells: vec![vec![EMPTY; width]; height],
        boxes: boxes.clone(),
    };
    // 箱を描く。
    for i in 0..n {
        let b = boxes[i];
        let put = |map: &mut Map, x: usize, y: usize, ch: char, role: Role, link: Option<usize>| {
            map.set(
                x,
                y,
                Cell {
                    ch,
                    role,
                    table: Some(i),
                    link,
                },
            )
        };
        let frame_row = |map: &mut Map, y: usize, l: char, r: char| {
            put(map, b.x, y, l, Role::Frame, None);
            for xx in b.x + 1..b.x + b.w - 1 {
                put(map, xx, y, g.h, Role::Frame, None);
            }
            put(map, b.x + b.w - 1, y, r, Role::Frame, None);
        };
        let text_row = |map: &mut Map, y: usize, text: &str, role: Role, link: Option<usize>| {
            put(map, b.x, y, g.v, Role::Frame, None);
            let t = clip(text, b.w - 4);
            let mut xx = b.x + 2;
            for xs in b.x + 1..b.x + b.w - 1 {
                put(map, xs, y, ' ', role, link);
            }
            for c in t.chars() {
                put(map, xx, y, c, role, link);
                // 幅2の文字の右のセルは空の印(描くときに飛ばす)。
                if cw(c) == 2 {
                    put(map, xx + 1, y, '\u{0}', role, link);
                }
                xx += cw(c);
            }
            put(map, b.x + b.w - 1, y, g.v, Role::Frame, None);
        };
        frame_row(&mut map, b.y, g.tl, g.tr);
        text_row(&mut map, b.y + 1, &titles[i], Role::Title, None);
        frame_row(&mut map, b.y + 2, g.lt, g.rt);
        let rows = &contents[i];
        if rows.is_empty() {
            text_row(&mut map, b.y + 3, "", Role::Field, None);
        }
        for (k, (t, role, link)) in rows.iter().enumerate() {
            text_row(&mut map, b.y + 3 + k, t, *role, *link);
        }
        frame_row(&mut map, b.y + b.h - 1, g.bl, g.br);
    }
    // 矢印を描く。入る行は、行き先の箱ごとに題の行から順に、列の行へ散らす。
    let mut incoming = vec![0usize; n];
    let gap_x = |gi: usize, slot: usize| -> usize {
        let left = if gi == 0 {
            0
        } else {
            col_x[gi - 1] + col_w[gi - 1]
        };
        left + 2 + 2 * slot
    };
    for (k, l) in links.iter().enumerate() {
        let (Some(a), Some(b)) = (index(&l.from), index(&l.to)) else {
            continue;
        };
        let Some(kind) = kinds[k] else {
            continue;
        };
        let src = boxes[a];
        let dst = boxes[b];
        let row = contents[a]
            .iter()
            .position(|(_, _, lk)| lk.is_some_and(|x| links[x].column == l.column))
            .unwrap_or(0);
        let sy = src.y + 3 + row;
        // 入る行: 題の行、続けて列の行(区切りの行は飛ばす)。
        let entries: Vec<usize> = std::iter::once(dst.y + 1)
            .chain(dst.y + 3..dst.y + dst.h - 1)
            .collect();
        let ey = entries[incoming[b] % entries.len()];
        incoming[b] += 1;
        let lane_y = |i: usize| boxes_h + 1 + i;
        let right_in = dst.x.saturating_sub(1); // 行き先の左の縁のすぐ左(右向きの矢印)
        let left_in = dst.x + dst.w; // 行き先の右の縁のすぐ右(左向きの矢印)
        let src_r = src.x + src.w; // 元の右の縁のすぐ右
        let src_l = src.x.saturating_sub(1); // 元の左の縁のすぐ左
        let (pts, arrow, start): Route = match kind {
            Kind::Next(gi, slot) => {
                let cx = gap_x(gi, slot);
                (
                    vec![(src_r, sy), (cx, sy), (cx, ey), (right_in, ey)],
                    (right_in, ey, g.right),
                    (src_r, sy),
                )
            }
            Kind::Same(gi, slot) => {
                let cx = gap_x(gi, slot);
                (
                    vec![(src_l, sy), (cx, sy), (cx, ey), (right_in, ey)],
                    (right_in, ey, g.right),
                    (src_l, sy),
                )
            }
            Kind::Prev(gi, slot) => {
                let cx = gap_x(gi, slot);
                (
                    vec![(src_l, sy), (cx, sy), (cx, ey), (left_in, ey)],
                    (left_in, ey, g.left),
                    (src_l, sy),
                )
            }
            Kind::Far(go, so, gin, si, lane) => {
                let ox = gap_x(go, so);
                let ix = gap_x(gin, si);
                let ly = lane_y(lane);
                let right = layer[b] > layer[a];
                let (sx, tx, ch) = if right {
                    (src_r, right_in, g.right)
                } else {
                    (src_l, left_in, g.left)
                };
                (
                    vec![(sx, sy), (ox, sy), (ox, ly), (ix, ly), (ix, ey), (tx, ey)],
                    (tx, ey, ch),
                    (sx, sy),
                )
            }
        };
        draw_path(&mut map, &pts, k, &g);
        map.set(
            arrow.0,
            arrow.1,
            Cell {
                ch: arrow.2,
                role: Role::Arrow,
                table: None,
                link: Some(k),
            },
        );
        // 数の札: 元の側に N、行き先の側(矢印の手前)に 1 か N。
        let put_label = |map: &mut Map, x: usize, y: usize, ch: char| {
            if map.get(x, y).table.is_none() {
                map.set(
                    x,
                    y,
                    Cell {
                        ch,
                        role: Role::Label,
                        table: None,
                        link: Some(k),
                    },
                );
            }
        };
        put_label(&mut map, start.0, start.1, 'N');
        let before = if arrow.2 == g.right {
            arrow.0.checked_sub(1)
        } else {
            Some(arrow.0 + 1)
        };
        if let Some(bx) = before {
            if map.get(bx, arrow.1).role == Role::Line {
                put_label(&mut map, bx, arrow.1, if l.many { 'N' } else { '1' });
            }
        }
    }
    map
}

/// 矢印の道: (折れ線の点, 矢印の先 (x, y, 文字), 元の端)。
type Route = (Vec<(usize, usize)>, (usize, usize, char), (usize, usize));

/// 向き。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dir {
    L,
    R,
    U,
    D,
}

fn dir(a: (usize, usize), b: (usize, usize)) -> Option<Dir> {
    use std::cmp::Ordering::*;
    match (b.0.cmp(&a.0), b.1.cmp(&a.1)) {
        (Greater, Equal) => Some(Dir::R),
        (Less, Equal) => Some(Dir::L),
        (Equal, Greater) => Some(Dir::D),
        (Equal, Less) => Some(Dir::U),
        _ => None,
    }
}

/// 曲がり角の文字(来た向き → 行く向き)。
fn corner(g: &Glyphs, from: Dir, to: Dir) -> char {
    use Dir::*;
    match (from, to) {
        (R, D) | (U, L) => g.rd,
        (R, U) | (D, L) => g.ru,
        (L, D) | (U, R) => g.ur,
        (L, U) | (D, R) => g.dr,
        (L, L) | (R, R) => g.h,
        _ => g.v,
    }
}

/// 折れ線(縦と横の線分の並び)を描く。箱のセルは上書きしない。線と線の交わりは十字。
fn draw_path(map: &mut Map, pts: &[(usize, usize)], k: usize, g: &Glyphs) {
    let put = |map: &mut Map, x: usize, y: usize, ch: char| {
        let cur = map.get(x, y);
        if cur.table.is_some() {
            return;
        }
        let ch = if cur.role == Role::Line && cur.link != Some(k) && cur.ch != ch {
            g.cross
        } else {
            ch
        };
        map.set(
            x,
            y,
            Cell {
                ch,
                role: Role::Line,
                table: None,
                link: Some(k),
            },
        );
    };
    let mut prev: Option<Dir> = None;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let Some(d) = dir(a, b) else {
            continue;
        };
        // 線分の始まりの点(最初の点か曲がり角)。
        let start_ch = match prev {
            Some(p) => corner(g, p, d),
            None => match d {
                Dir::L | Dir::R => g.h,
                _ => g.v,
            },
        };
        put(map, a.0, a.1, start_ch);
        let (ch, steps): (char, Vec<(usize, usize)>) = match d {
            Dir::R => (g.h, (a.0 + 1..b.0).map(|x| (x, a.1)).collect()),
            Dir::L => (g.h, (b.0 + 1..a.0).rev().map(|x| (x, a.1)).collect()),
            Dir::D => (g.v, (a.1 + 1..b.1).map(|y| (a.0, y)).collect()),
            Dir::U => (g.v, (b.1 + 1..a.1).rev().map(|y| (a.0, y)).collect()),
        };
        for (x, y) in steps {
            put(map, x, y, ch);
        }
        prev = Some(d);
    }
}

/// つながりの1行の文(狭い画面の一覧・詳細): `Tasks.project → Projects (N:1)`。
pub fn link_text(l: &Link) -> String {
    clean(&format!(
        "{}.{} → {} ({})",
        l.from,
        l.column,
        l.to,
        if l.many { "N:N" } else { "N:1" }
    ))
}

/// つながりの組のうち、行き先ごとの数の多いもの(詳細の見本): (行き先の名前, 元の数)。
pub fn top_targets(l: &Link, n: usize) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = Vec::new();
    for (_, t) in &l.pairs {
        let name = file_stem(t);
        match v.iter_mut().find(|(x, _)| *x == name) {
            Some((_, k)) => *k += 1,
            None => v.push((name, 1)),
        }
    }
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

#[cfg(test)]
#[path = "test_relmap_unit.rs"]
mod tests;
