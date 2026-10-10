//! 並べ替えの窓(NV-24): 検索の欄の右の「並べ替え」のボタンで開く小さな窓。今のビューの並べ替えの決まり
//! (列と向き)を並べ、足す(列を選ぶ)・外す・向きを変える・順を入れ替える。変えるとすぐ表に当て、
//! ビューの設定の並べ替えとして見た目の状態に残す(set_sorts。NV-3 の見出しの並べ替えと同じ保存)。
//!
//! 窓は検索の欄の行の下に、ボタンの右の端にそろえて重ねる(欄が無ければヘッダーの下)。窓の外のクリックと
//! Esc で閉じる。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::settings::Dir;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 開いている並べ替えの窓。
#[derive(Debug, Clone, Default)]
pub(crate) struct SortWin {
    /// 選んでいる行(決まりの並びのあとに「+ 並べ替えを足す」)。
    pub sel: usize,
    /// 足す列を選んでいる間の、選べる列と選んでいる行。
    pub picking: Option<(Vec<String>, usize)>,
}

/// 検索の欄の右の端のボタンの文字(NV-24): 決まりが無ければ「並べ替え」、あれば先頭の決まり(と残りの数)。
pub(crate) fn button(app: &App) -> String {
    match app.settings.sorts.first() {
        None => Msg::SortButton.text().to_string(),
        Some((c, d)) => {
            let arrow = if *d == Dir::Desc { "↓" } else { "↑" };
            let more = app.settings.sorts.len() - 1;
            let name = sanitize(&app.title(c));
            if more == 0 {
                format!(" {arrow} {name} ")
            } else {
                format!(" {arrow} {name} +{more} ")
            }
        }
    }
}

/// 窓の1行の文字(決まり・足す行・列の候補)。
fn rows(app: &App, w: &SortWin) -> (String, Vec<String>) {
    match &w.picking {
        Some((cols, _)) => (
            Msg::SortPickTitle.text().to_string(),
            cols.iter().map(|c| sanitize(&app.title(c))).collect(),
        ),
        None => {
            let mut out: Vec<String> = app
                .settings
                .sorts
                .iter()
                .map(|(c, d)| {
                    let arrow = if *d == Dir::Desc { "↓" } else { "↑" };
                    let dir = if *d == Dir::Desc {
                        Msg::SortDesc.text()
                    } else {
                        Msg::SortAsc.text()
                    };
                    format!("{arrow} {}  {dir}", sanitize(&app.title(c)))
                })
                .collect();
            out.push(Msg::SortAdd.text().to_string());
            (Msg::SortTitle.text().to_string(), out)
        }
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    start: usize,
    vis: usize,
}

fn window(app: &App) -> Option<Geom> {
    let w = app.sorts_win.as_ref()?;
    let (title, items) = rows(app, w);
    let (sw, limit) = popup::screen(app);
    let inner = items
        .iter()
        .map(|t| width(t) + 5)
        .chain(std::iter::once(width(&title) + 2))
        .max()
        .unwrap_or(10);
    let iw = (inner + 2).min(sw);
    if iw < 8 {
        return None;
    }
    // 検索の欄の行(無ければヘッダー)の下に、右の端にそろえて置く。
    let ay = super::bands::sort_anchor_y(app);
    let popup::Place { top, vis } = popup::place(ay, items.len(), limit)?;
    let sel = match &w.picking {
        Some((_, s)) => *s,
        None => w.sel,
    }
    .min(items.len() - 1);
    Some(Geom {
        x: sw - iw,
        top,
        iw,
        start: (sel + 1).saturating_sub(vis),
        vis,
    })
}

impl App {
    /// NV-24: 並べ替えの窓を開く(表のモードから)。
    pub(crate) fn open_sorts(&mut self) {
        if self.mode != Mode::Table {
            self.set_mode(Mode::Table);
        }
        self.sorts_win = Some(SortWin::default());
        if window(self).is_none() {
            self.sorts_win = None;
            self.message = Some(Msg::SortNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::Sorts);
    }

    pub(crate) fn close_sorts(&mut self) {
        self.sorts_win = None;
        self.set_mode(Mode::Table);
    }

    /// 足せる列(今の決まりに無い列。表示している列のあとに隠した列)。
    fn sortable_columns(&self) -> Vec<String> {
        let used: Vec<&String> = self.settings.sorts.iter().map(|(c, _)| c).collect();
        self.cols
            .iter()
            .cloned()
            .chain(self.hidden.iter().map(|h| h.0.clone()))
            .filter(|c| !used.contains(&c))
            .collect()
    }

    /// 並べ替えの窓のモードの動作。
    pub(crate) fn sorts_action(&mut self, action: Action) {
        let Some(mut w) = self.sorts_win.take() else {
            return self.close_sorts();
        };
        let n = self.settings.sorts.len();
        match (&mut w.picking, action) {
            (Some((cols, s)), Action::Up | Action::Down | Action::Top | Action::Bottom) => {
                *s = popup::step_sel(*s, cols.len(), action, 0).unwrap_or(*s);
            }
            (Some((cols, s)), Action::Run) => {
                if let Some(c) = cols.get(*s).cloned() {
                    let mut sorts = self.settings.sorts.clone();
                    sorts.push((c, Dir::Asc));
                    w.sel = sorts.len() - 1;
                    w.picking = None;
                    self.set_sorts(sorts);
                }
            }
            // 列を選んでいる間の Esc は、決まりの並びに戻る。
            (Some(_), Action::Close) => w.picking = None,
            (Some(_), _) => {}
            (None, Action::Up | Action::Down | Action::Top | Action::Bottom) => {
                w.sel = popup::step_sel(w.sel, n + 1, action, 0).unwrap_or(w.sel);
            }
            (None, Action::Run) => {
                if w.sel < n {
                    let mut sorts = self.settings.sorts.clone();
                    let d = &mut sorts[w.sel].1;
                    *d = if *d == Dir::Asc { Dir::Desc } else { Dir::Asc };
                    self.set_sorts(sorts);
                } else {
                    let cols = self.sortable_columns();
                    if cols.is_empty() {
                        self.message = Some(Msg::SortNoColumn.text().into());
                    } else {
                        w.picking = Some((cols, 0));
                    }
                }
            }
            (None, Action::RemoveItem) if w.sel < n => {
                let mut sorts = self.settings.sorts.clone();
                sorts.remove(w.sel);
                w.sel = w.sel.min(sorts.len());
                self.set_sorts(sorts);
            }
            (None, Action::MoveItemUp) if w.sel > 0 && w.sel < n => {
                let mut sorts = self.settings.sorts.clone();
                sorts.swap(w.sel, w.sel - 1);
                w.sel -= 1;
                self.set_sorts(sorts);
            }
            (None, Action::MoveItemDown) if w.sel + 1 < n => {
                let mut sorts = self.settings.sorts.clone();
                sorts.swap(w.sel, w.sel + 1);
                w.sel += 1;
                self.set_sorts(sorts);
            }
            (None, Action::Close) => return self.close_sorts(),
            _ => {}
        }
        self.sorts_win = Some(w);
        self.set_mode(Mode::Sorts);
    }

    /// 並べ替えの窓の上のクリック: 行なら選んで Enter と同じ、窓の外なら閉じる。縁では何もしない。
    pub(crate) fn sorts_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_sorts();
        };
        match popup::hit(x, y, g.x, g.top, g.iw, g.vis) {
            None => self.close_sorts(),
            Some(None) => {}
            Some(Some(k)) => {
                let i = g.start + k;
                // 決まりの行の右の端の「×」なら外す(ほかの所は Enter と同じ)。
                let on_x = (x as usize) + 3 >= g.x + g.iw && (x as usize) < g.x + g.iw - 1;
                let rule = self
                    .sorts_win
                    .as_ref()
                    .is_some_and(|w| w.picking.is_none() && i < self.settings.sorts.len());
                if let Some(w) = self.sorts_win.as_mut() {
                    match &mut w.picking {
                        Some((_, s)) => *s = i,
                        None => w.sel = i,
                    }
                }
                if rule && on_x {
                    self.sorts_action(Action::RemoveItem);
                } else {
                    self.sorts_action(Action::Run);
                }
            }
        }
    }
}

/// 窓を重ねる(下の帯より上の行に)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Sorts {
        return;
    }
    let (Some(win), Some(g)) = (app.sorts_win.as_ref(), window(app)) else {
        return;
    };
    let (title, items) = rows(app, win);
    let sel = match &win.picking {
        Some((_, s)) => *s,
        None => win.sel,
    };
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let mut out: Vec<(String, Style)> = vec![(popup::top_edge(fr, &title, tw), Style::default())];
    for (i, t) in items.iter().enumerate().skip(g.start).take(g.vis) {
        let st = if i == sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let text = format!("{}{t}", if i == sel { "> " } else { "  " });
        // 決まりの行は右の端に「×」(押すと外す。NV-24)。
        let rule = win.picking.is_none() && i < app.settings.sorts.len();
        let line = if rule {
            format!("{} ×", fit(&text, tw.saturating_sub(2), Align::Left))
        } else {
            fit(&text, tw, Align::Left)
        };
        out.push((fr.side(&line), st));
    }
    out.push((fr.bottom(tw), Style::default()));
    popup::blit(app, lines, g.x, g.top, g.iw, w, out);
}
