//! cell-view.url-full-target: 切って見せた URL に、端末が開く行き先(OSC 8 のハイパーリンク)を付ける。
//!
//! 端末は画面の文字から URL を見つけて開くので、`…` で切った文字や折り返した文字だけだと、途中までの
//! URL が開く。描く側(表・木の右の欄・詳細)が「この桁のこの文字は、この URL」を `Links` に覚えさせ、
//! 描いたあとに、その桁の字を OSC 8 で包んでもう一度書く(ratatui の描画には混ぜない。docs/design.md)。

use super::app::App;
use super::width::width;
use mdgrid::folder_tree;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::buffer::{Buffer, Cell};
use ratatui::crossterm::cursor::{RestorePosition, SavePosition};
use ratatui::crossterm::queue;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::io::{self, Write};

/// 描いた行の中の、URL を見せている所(描く範囲の左上からの桁と行)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Spot {
    pub x: usize,
    pub y: usize,
    /// そこに描いた文字(切ったあと・寄せたあと)。画面の字と合わなければ、窓が重なっているので付けない。
    pub text: String,
    pub url: String,
}

/// 1回の描画で覚えた所。描く関数は `&App` しか持たないので、中で書き換えられる形にする。
#[derive(Debug, Default)]
pub(crate) struct Links(RefCell<Vec<Spot>>);

impl Links {
    pub(crate) fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub(crate) fn push(&self, x: usize, y: usize, text: String, url: &str) {
        self.0.borrow_mut().push(Spot {
            x,
            y,
            text,
            url: url.to_string(),
        });
    }

    /// `from` 番目より後ろに覚えた所を、(dx, dy) だけずらす(窓や区画の中の桁を、画面の桁に直す)。
    pub(crate) fn shift(&self, from: usize, dx: usize, dy: usize) {
        for s in self.0.borrow_mut().iter_mut().skip(from) {
            s.x += dx;
            s.y += dy;
        }
    }

    pub(crate) fn take(&self) -> Vec<Spot> {
        std::mem::take(&mut self.0.borrow_mut())
    }

    pub(crate) fn extend(&self, spots: Vec<Spot>) {
        self.0.borrow_mut().extend(spots);
    }

    pub(crate) fn spots(&self) -> Vec<Spot> {
        self.0.borrow().clone()
    }
}

impl App {
    /// 値 `raw` の全体が、行き先を付けてよい URL ならその URL。`TERM=dumb` では付けない。
    /// 開く操作と同じ決まり(tree.open-url-scheme・tree.open-url-disguised)で、開かない URL には付けない。
    pub(crate) fn link_url(&self, raw: &str) -> Option<String> {
        if self.no_emoji || !raw.contains("://") {
            return None;
        }
        let url = folder_tree::web_url(raw)?;
        folder_tree::url_refusal(url)
            .is_none()
            .then(|| url.to_string())
    }
}

/// 行き先ごとの、包んで書き直す字(画面の桁・行・字)。
pub type Linked = Vec<(String, Vec<(u16, u16, Cell)>)>;

fn solid(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 覚えた所のうち、描き終えた画面 `buf` に同じ文字が出ている所の字を、行き先ごとに集める。
/// 窓が重なって別の文字になった所と、画面の外の所は外す。空白の字は包まない。
pub fn linked(app: &App, buf: &Buffer, area: Rect) -> Linked {
    let mut out: Linked = Vec::new();
    for s in app.url_spots.spots() {
        let w = width(&s.text);
        let (Ok(x0), Ok(y)) = (u16::try_from(s.x), u16::try_from(s.y)) else {
            continue;
        };
        let (x0, y) = (area.x.saturating_add(x0), area.y.saturating_add(y));
        let Some(x1) = u16::try_from(w).ok().and_then(|w| x0.checked_add(w)) else {
            continue;
        };
        if w == 0 || y >= area.bottom() || x1 > area.right() {
            continue;
        }
        let cells: Vec<(u16, u16, Cell)> = (x0..x1)
            .filter_map(|x| buf.cell((x, y)).map(|c| (x, y, c.clone())))
            .collect();
        let on: String = cells.iter().map(|(_, _, c)| c.symbol()).collect();
        if solid(&on) != solid(&s.text) {
            continue;
        }
        let cells: Vec<_> = cells
            .into_iter()
            .filter(|(_, _, c)| !c.symbol().trim().is_empty())
            .collect();
        match out.iter_mut().find(|(u, _)| *u == s.url) {
            Some((_, v)) => v.extend(cells),
            None => out.push((s.url, cells)),
        }
    }
    out
}

/// `linked` の字を、OSC 8 で包んで書き直す。カーソルの位置は元に戻す。
/// URL に制御文字があれば書かない(`web_url` が通さないが、端末への列を壊さないためにここでも見る)。
pub fn write<W: Write>(backend: &mut CrosstermBackend<W>, links: &Linked) -> io::Result<()> {
    if links.is_empty() {
        return Ok(());
    }
    queue!(backend, SavePosition)?;
    for (url, cells) in links {
        if cells.is_empty() || url.chars().any(char::is_control) {
            continue;
        }
        write!(backend, "\x1b]8;;{url}\x1b\\")?;
        backend.draw(cells.iter().map(|(x, y, c)| (*x, *y, c)))?;
        write!(backend, "\x1b]8;;\x1b\\")?;
    }
    queue!(backend, RestorePosition)?;
    Write::flush(backend)
}
