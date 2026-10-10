//! 関係マップの画面(REL-7・REL-8・REL-9。`impl App` の続き): ワークスペースの表(登録した表と今の表)の箱と、
//! つながりの矢印。表の画面とは別の画面の型(SC-9)で、上の端のタブとキー `R` で切り替える。
//!
//! 段組み(REL-9): 幅 120 以上は 左の表の一覧・真ん中のマップ・右の詳細、80 以上はマップと右の詳細、80 未満は
//! 表とつながりの一覧の文字だけ。高さ 30 以上なら下につながった行。窓はどれも枠と題を持つ(SR-32)。
//! 色はモダンな見た目(SR-33)のときだけ。classic と色なしでは、選んだものを `>` と太字で示す(SR-15)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::look::Look;
use super::popup::{self, Frame};
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::relations::{self, Table};
use mdgrid::relmap::{self, Cell, Link, Map, Role, TableInfo};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// 関係マップの状態(開いたときに読んだ表とつながり)。
pub(crate) struct RelMap {
    pub infos: Vec<TableInfo>,
    pub links: Vec<Link>,
    pub map: Map,
    /// 選んだ表(infos の添字)。
    pub sel: usize,
    /// 選んだつながり(links の添字。選んだ表に関わるものだけを ←→ で回る)。
    pub link: Option<usize>,
}

impl RelMap {
    /// 選んだ表に関わるつながり(元か行き先)。
    fn links_of(&self, t: usize) -> Vec<usize> {
        let name = &self.infos[t].name;
        (0..self.links.len())
            .filter(|&k| &self.links[k].from == name || &self.links[k].to == name)
            .collect()
    }
}

/// 段組みの形(REL-9)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Panes {
    /// 左の一覧・マップ・右の詳細。
    Three,
    /// マップと右の詳細。
    Two,
    /// 一覧の文字だけ。
    One,
}

/// 幅で段組みを決める。
pub(crate) fn panes(w: usize) -> Panes {
    if w >= 120 {
        Panes::Three
    } else if w >= 80 {
        Panes::Two
    } else {
        Panes::One
    }
}

/// 段組みの寸法(描画とクリックの判定で同じものを使う)。窓の (左の桁, 幅)。上の窓は1行目から top_h 行。
struct Geo {
    panes: Panes,
    list: Option<(usize, usize)>,
    map: (usize, usize),
    detail: Option<(usize, usize)>,
    top_h: usize,
    linked_h: usize,
}

fn geo(w: usize, h: usize) -> Geo {
    let body_h = h.saturating_sub(3);
    let panes = panes(w);
    // 下につながった行(高さがあれば)。
    let linked_h = if body_h >= 27 && panes != Panes::One {
        8
    } else {
        0
    };
    let top_h = body_h - linked_h;
    let (list, map, detail) = match panes {
        Panes::Three => {
            let (lw, dw) = (26, 34);
            (Some((0, lw)), (lw, w - lw - dw), Some((w - dw, dw)))
        }
        Panes::Two => {
            let dw = 26;
            (None, (0, w - dw), Some((w - dw, dw)))
        }
        Panes::One => (None, (0, w), None),
    };
    Geo {
        panes,
        list,
        map,
        detail,
        top_h,
        linked_h,
    }
}

/// 1つの段組みの一覧のずらし(行)。選んでいるつながり(無ければ表)の行が、高さ `h` の窓に入るように。
/// 一覧は表の行・空きの行・つながりの行の順(one_body)。
fn one_scroll(rm: &RelMap, h: usize) -> usize {
    let target = match rm.link {
        Some(k) => rm.infos.len() + 1 + k,
        None => rm.sel,
    };
    (target + 1).saturating_sub(h.max(1))
}

/// 盤のずらし(選んだ表の箱が見えるように)。`w`・`h` は盤の窓の中の大きさ。
fn map_offset(rm: &RelMap, w: usize, h: usize) -> (usize, usize) {
    let b = rm.map.boxes.get(rm.sel).copied();
    let ox = match b {
        Some(b) if b.x + b.w > w => (b.x + b.w).saturating_sub(w).min(b.x.saturating_sub(1)),
        _ => 0,
    };
    let oy = match b {
        Some(b) if b.y + b.h > h => (b.y + b.h).saturating_sub(h).min(b.y),
        _ => 0,
    };
    (ox, oy)
}

impl App {
    /// REL-12: 関係マップのクリック。表は1回目で選び、選んだ表をもう一度で開く。つながりは選ぶ。
    /// つながった行はその元のノートを開く。枠や空きは何もしない。
    pub(crate) fn relmap_click(&mut self, x: u16, y: u16) {
        let (w, h) = (
            self.size.0.saturating_sub(1) as usize,
            self.size.1.saturating_sub(1) as usize,
        );
        let Some(rm) = self.relmap.as_ref() else {
            return;
        };
        let g = geo(w, h);
        let (x, y) = (x as usize, y as usize);
        // 窓の中の (桁, 行)。枠の上なら None。
        let inner = |left: usize, pw: usize, top: usize, ph: usize| -> Option<(usize, usize)> {
            (x > left && x + 1 < left + pw && y > top && y + 1 < top + ph)
                .then(|| (x - left - 1, y - top - 1))
        };
        enum Hit {
            Table(usize),
            Link(usize),
            Note(std::path::PathBuf),
        }
        let mut hit = None;
        if let Some((_, r)) = g.list.and_then(|(l, pw)| inner(l, pw, 1, g.top_h)) {
            hit = (r < rm.infos.len()).then_some(Hit::Table(r));
        } else if let Some((cx, r)) = inner(g.map.0, g.map.1, 1, g.top_h) {
            if g.panes == Panes::One {
                let r = r + one_scroll(rm, g.top_h.saturating_sub(2));
                let n = rm.infos.len();
                hit = if r < n {
                    Some(Hit::Table(r))
                } else if r > n && r - n - 1 < rm.links.len() {
                    Some(Hit::Link(r - n - 1))
                } else {
                    None
                };
            } else {
                let (ox, oy) = map_offset(rm, g.map.1.saturating_sub(2), g.top_h.saturating_sub(2));
                let (mx, my) = (ox + cx, oy + r);
                let row = rm.map.cells.get(my);
                // 幅2の文字の右半分は、左のセル。
                let c = row.and_then(|row| match row.get(mx) {
                    Some(c) if c.ch == '\u{0}' && mx > 0 => row.get(mx - 1),
                    c => c,
                });
                hit = match c {
                    Some(c) if c.table.is_some() => c.table.map(Hit::Table),
                    Some(c) => c.link.map(Hit::Link),
                    None => None,
                };
            }
        } else if g.linked_h > 0 {
            if let Some((_, r)) = inner(0, w, 1 + g.top_h, g.linked_h) {
                hit = rm
                    .link
                    .and_then(|k| rm.links[k].pairs.get(r))
                    .map(|(a, _)| Hit::Note(a.clone()));
            }
        }
        match hit {
            Some(Hit::Table(t)) if t == rm.sel => self.relmap_open_table(),
            Some(Hit::Table(t)) => {
                let rm = self.relmap.as_mut().expect("relmap");
                rm.sel = t;
                rm.link = rm.links_of(t).first().copied();
            }
            Some(Hit::Link(k)) => {
                let rm = self.relmap.as_mut().expect("relmap");
                if !rm.links_of(rm.sel).contains(&k) {
                    let from = &rm.links[k].from;
                    if let Some(t) = rm.infos.iter().position(|i| &i.name == from) {
                        rm.sel = t;
                    }
                }
                rm.link = Some(k);
            }
            Some(Hit::Note(note)) => {
                self.relmap = None;
                self.set_mode(Mode::Table);
                self.open_note(&note);
            }
            None => {}
        }
    }

    /// REL-7: 関係マップを開く(ワークスペースの表を読んで並べる)。今の表を選ぶ。
    pub(crate) fn open_relmap(&mut self) {
        let tables = self.workspace_tables_now();
        if tables.is_empty() {
            self.message = Some(Msg::RelMapNoTables.into());
            return;
        }
        let (infos, links) = relmap::read(&tables);
        let ascii = popup::frame(self) == popup::ASCII;
        let map = relmap::layout(&infos, &links, ascii);
        let here = self.current_table_dir();
        let sel = here
            .and_then(|d| infos.iter().position(|t| t.dir == d))
            .unwrap_or(0);
        let mut rm = RelMap {
            infos,
            links,
            map,
            sel,
            link: None,
        };
        rm.link = rm.links_of(sel).first().copied();
        self.relmap = Some(rm);
        self.set_mode(Mode::Relations);
    }

    /// 今の表のフォルダ(実体のパス)。
    fn current_table_dir(&self) -> Option<std::path::PathBuf> {
        let row = self.src.rows().into_iter().next()?;
        let info = self.src.file(&row)?;
        let mut root = std::path::PathBuf::from(&row.0);
        for _ in std::path::Path::new(&info.path).components() {
            root.pop();
        }
        Some(root)
    }

    /// ワークスペースの表(登録した表と今の表)を今読む。
    fn workspace_tables_now(&self) -> Vec<Table> {
        let mut out: Vec<Table> = self
            .scope_places()
            .iter()
            .map(|p| Table::new(&p.name, &p.path))
            .collect();
        if let Some(root) = self.current_table_dir() {
            if !out.iter().any(|t| t.dir == root) {
                out.push(Table::new(&self.src.name(), &root));
            }
        }
        // 箱とつながりは名前で結ぶので、同じ名前(フォルダの Tasks と Tasks.base など)は番号で分ける。
        for i in 1..out.len() {
            let base = out[i].name.clone();
            let mut k = 2;
            while out[..i].iter().any(|t| t.name == out[i].name) {
                out[i].name = format!("{base} ({k})");
                k += 1;
            }
        }
        out
    }

    /// 関係マップの動作。
    pub(crate) fn relmap_action(&mut self, action: Action) {
        let Some(rm) = self.relmap.as_mut() else {
            return self.set_mode(Mode::Table);
        };
        let n = rm.infos.len();
        match action {
            Action::Up | Action::Down => {
                if n > 0 {
                    rm.sel = if action == Action::Up {
                        (rm.sel + n - 1) % n
                    } else {
                        (rm.sel + 1) % n
                    };
                    rm.link = rm.links_of(rm.sel).first().copied();
                }
            }
            Action::Left | Action::Right => {
                let ls = rm.links_of(rm.sel);
                if !ls.is_empty() {
                    let at = rm
                        .link
                        .and_then(|k| ls.iter().position(|x| *x == k))
                        .unwrap_or(0);
                    let m = ls.len();
                    let next = if action == Action::Left {
                        (at + m - 1) % m
                    } else {
                        (at + 1) % m
                    };
                    rm.link = Some(ls[next]);
                }
            }
            Action::Run => self.relmap_open_table(),
            Action::NewNote => self.relmap_new_note(),
            Action::Close | Action::RelationMap => {
                self.relmap = None;
                self.set_mode(Mode::Table);
            }
            Action::Help => self.open_help(),
            _ => {}
        }
    }

    /// CE-25: 関係マップの「+ 新規」: 選んでいる表を開いて、その表で名前の欄を出す。今の表ならすぐ、
    /// ほかの表なら開き直したあと(src/main.rs)。
    fn relmap_new_note(&mut self) {
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        self.relmap_open_table();
        if self.switch_to.is_some() {
            self.switch_new_note = true;
        } else if self.mode == Mode::Table {
            self.start_new_note();
        }
    }

    /// 選んだ表を開く: 今の表なら表の画面に戻る。ほかの表は、登録した表(無ければそのフォルダ)へ移る。
    fn relmap_open_table(&mut self) {
        let Some(rm) = self.relmap.as_ref() else {
            return;
        };
        let t = rm.infos[rm.sel].clone();
        if self.current_table_dir().as_ref() == Some(&t.dir) {
            self.relmap = None;
            self.set_mode(Mode::Table);
            return;
        }
        let place = self
            .scope_places()
            .into_iter()
            .find(|p| Table::new(&p.name, &p.path).dir == t.dir)
            .unwrap_or_else(|| Place {
                name: t.name.clone(),
                group: String::new(),
                path: t.dir.clone(),
                view: None,
            });
        self.relmap = None;
        self.set_mode(Mode::Table);
        self.switch_to = Some(place);
        self.switch_select = None;
        self.begin_quit();
    }
}

/// 窓: 枠と題の付いた w×h の行。`focused` なら枠はアクセントの色(モダン)か太字の題(ほか)。
fn panel(
    app: &App,
    title: &str,
    body: Vec<Line<'static>>,
    w: usize,
    h: usize,
    focused: bool,
) -> Vec<Line<'static>> {
    let f: Frame = popup::frame(app);
    let look = super::look::look(app);
    let edge = match &look {
        Some(l) if focused => l.border(),
        Some(l) => l.faint(),
        None => Style::default(),
    };
    let title_st = match &look {
        Some(l) if focused => l.key(),
        Some(_) => Style::default().add_modifier(Modifier::BOLD),
        None => Style::default().add_modifier(Modifier::BOLD),
    };
    if w < 4 || h < 2 {
        return vec![Line::from(" ".repeat(w)); h];
    }
    let inner = w - 2;
    let t = fit(
        &format!(" {} ", sanitize(title)),
        inner.saturating_sub(1).min(width(title) + 2),
        Align::Left,
    );
    let mut out = Vec::new();
    out.push(Line::from(vec![
        Span::styled(format!("{}{}", f.tl, f.h), edge),
        Span::styled(t.clone(), title_st),
        Span::styled(
            format!("{}{}", f.line(inner.saturating_sub(1 + width(&t))), f.tr),
            edge,
        ),
    ]));
    let mut body = body.into_iter();
    for _ in 0..h.saturating_sub(2) {
        let mut spans = vec![Span::styled(f.v.to_string(), edge)];
        match body.next() {
            Some(line) => {
                let used: usize = line.spans.iter().map(|s| width(&s.content)).sum();
                spans.extend(line.spans);
                if used < inner {
                    spans.push(Span::raw(" ".repeat(inner - used)));
                }
            }
            None => spans.push(Span::raw(" ".repeat(inner))),
        }
        spans.push(Span::styled(f.v.to_string(), edge));
        out.push(Line::from(spans));
    }
    out.push(Line::from(Span::styled(f.bottom(inner), edge)));
    out
}

/// 盤のセルの見た目。
fn cell_style(c: &Cell, rm: &RelMap, look: Option<&Look>) -> Style {
    let sel_table = c.table == Some(rm.sel);
    let sel_link = c.link.is_some() && c.link == rm.link;
    match look {
        Some(l) => match c.role {
            Role::Frame if sel_table => l.border(),
            Role::Frame => l.faint(),
            Role::Title if sel_table => l.key(),
            Role::Title => Style::default().add_modifier(Modifier::BOLD),
            Role::Link if sel_link => l.selected(),
            Role::Link => Style::default().fg(l.accent),
            Role::Line | Role::Arrow | Role::Label if sel_link => l.key(),
            Role::Line | Role::Arrow | Role::Label => l.faint(),
            _ => Style::default(),
        },
        None => match c.role {
            Role::Title if sel_table => {
                Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
            }
            Role::Title => Style::default().add_modifier(Modifier::BOLD),
            Role::Link if sel_link => {
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            }
            Role::Line | Role::Arrow | Role::Label if sel_link => {
                Style::default().add_modifier(Modifier::BOLD)
            }
            _ => Style::default(),
        },
    }
}

/// マップの窓の中身: 選んだ表の箱が見えるようにずらした盤の一部。
fn map_body(app: &App, rm: &RelMap, w: usize, h: usize) -> Vec<Line<'static>> {
    let look = super::look::look(app);
    let m = &rm.map;
    let b = m.boxes.get(rm.sel).copied();
    let (ox, oy) = map_offset(rm, w, h);
    let mut out = Vec::new();
    for y in oy..(oy + h).min(m.h) {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut used = 0usize;
        let mut x = ox;
        while x < m.w && used < w {
            let c = m.cells[y][x];
            if c.ch == '\u{0}' {
                // 窓の左の端が幅2の文字の右半分なら、空白で埋めて桁を保つ(クリックの判定と同じ桁。REL-12)。
                if x == ox {
                    spans.push(Span::raw(" "));
                    used += 1;
                }
                x += 1;
                continue;
            }
            let cw = width(&c.ch.to_string());
            if used + cw > w {
                break;
            }
            // classic と色なしでは、選んだ表の題の左に `>`(色に頼らない。SR-15)。
            let ch = if look.is_none()
                && c.table == Some(rm.sel)
                && c.role == Role::Title
                && b.is_some_and(|bb| x == bb.x + 1 && y == bb.y + 1)
            {
                '>'
            } else {
                c.ch
            };
            spans.push(Span::styled(
                ch.to_string(),
                cell_style(&c, rm, look.as_ref()),
            ));
            used += cw;
            x += 1;
        }
        if used < w {
            spans.push(Span::raw(" ".repeat(w - used)));
        }
        out.push(Line::from(spans));
    }
    out
}

/// 左の表の一覧の中身。
fn list_body(app: &App, rm: &RelMap, w: usize) -> Vec<Line<'static>> {
    let look = super::look::look(app);
    rm.infos
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let sel = i == rm.sel;
            let count = t.rows.to_string();
            let name_w = w.saturating_sub(width(&count) + 3);
            let text = format!(
                "{}{} {}",
                if sel { ">" } else { " " },
                fit(&sanitize(&t.name), name_w, Align::Left),
                count
            );
            let st = match (&look, sel) {
                (Some(l), true) => l.selected(),
                (None, true) => Style::default().add_modifier(Modifier::REVERSED),
                _ => Style::default(),
            };
            Line::from(Span::styled(fit(&text, w, Align::Left), st))
        })
        .collect()
}

/// 右の詳細の中身: 選んだつながり(無ければ選んだ表)。
fn detail_body(app: &App, rm: &RelMap, w: usize) -> Vec<Line<'static>> {
    let look = super::look::look(app);
    let key_st = look.as_ref().map(|l| l.faint()).unwrap_or_default();
    let strong = match &look {
        Some(l) => l.key(),
        None => Style::default().add_modifier(Modifier::BOLD),
    };
    // 項目の名前の欄: いちばん長い名前と空き2桁(値の欄をなるべく広く)。
    let kw = [
        Msg::RelMapKind,
        Msg::RelMapColumn,
        Msg::RelMapTarget,
        Msg::RelMapLinks,
        Msg::RelMapReverse,
        Msg::RelMapRows,
    ]
    .iter()
    .map(|m| width(m.text()))
    .max()
    .unwrap_or(7)
        + 2;
    let row = |k: &str, v: String| -> Line<'static> {
        let kw = kw.min(w);
        Line::from(vec![
            Span::styled(fit(k, kw, Align::Left), key_st),
            Span::raw(fit(&sanitize(&v), w.saturating_sub(kw), Align::Left)),
        ])
    };
    let mut out = Vec::new();
    match rm.link.map(|k| &rm.links[k]) {
        Some(l) => {
            out.push(Line::from(Span::styled(
                fit(
                    &sanitize(&format!("{}.{} → {}", l.from, l.column, l.to)),
                    w,
                    Align::Left,
                ),
                strong,
            )));
            out.push(Line::from(" ".repeat(w)));
            out.push(row(
                Msg::RelMapKind.text(),
                if l.many {
                    Msg::RelMapManyToMany.text().to_string()
                } else {
                    Msg::RelMapManyToOne.text().to_string()
                },
            ));
            out.push(row(Msg::RelMapColumn.text(), l.column.clone()));
            out.push(row(Msg::RelMapTarget.text(), l.to.clone()));
            out.push(row(Msg::RelMapLinks.text(), l.pairs.len().to_string()));
            // 逆向き: 行き先から元へ戻るつながり。
            let back: Vec<String> = rm
                .links
                .iter()
                .filter(|x| x.from == l.to && x.to == l.from)
                .map(|x| x.column.clone())
                .collect();
            if !back.is_empty() {
                out.push(row(Msg::RelMapReverse.text(), back.join(", ")));
            }
            out.push(Line::from(" ".repeat(w)));
            out.push(Line::from(Span::styled(
                fit(Msg::RelMapTop.text(), w, Align::Left),
                strong,
            )));
            for (name, k) in relmap::top_targets(l, 5) {
                let count = k.to_string();
                let nw = w.saturating_sub(width(&count) + 2);
                out.push(Line::from(format!(
                    " {} {count}",
                    fit(&sanitize(&name), nw, Align::Left)
                )));
            }
        }
        None => {
            let t = &rm.infos[rm.sel];
            out.push(Line::from(Span::styled(
                fit(&sanitize(&t.name), w, Align::Left),
                strong,
            )));
            out.push(row(Msg::RelMapRows.text(), t.rows.to_string()));
            out.push(Line::from(fit(Msg::RelMapNoLinks.text(), w, Align::Left)));
            for c in t.columns.iter().take(12) {
                out.push(Line::from(fit(
                    &format!(" {}", sanitize(c)),
                    w,
                    Align::Left,
                )));
            }
        }
    }
    out
}

/// 下のつながった行の中身: 選んだつながりのリンクの組。
fn linked_body(rm: &RelMap, w: usize) -> Vec<Line<'static>> {
    let Some(l) = rm.link.map(|k| &rm.links[k]) else {
        return Vec::new();
    };
    l.pairs
        .iter()
        .map(|(a, b)| {
            Line::from(fit(
                &format!(
                    " {}  {}  {}",
                    sanitize(&relations::file_stem(a)),
                    "→",
                    sanitize(&relations::file_stem(b))
                ),
                w,
                Align::Left,
            ))
        })
        .collect()
}

/// 1つの段組み(狭い画面): 表とつながりの一覧の文字。
fn one_body(app: &App, rm: &RelMap, w: usize) -> Vec<Line<'static>> {
    let mut out = list_body(app, rm, w);
    out.push(Line::from(" ".repeat(w)));
    let look = super::look::look(app);
    for (k, l) in rm.links.iter().enumerate() {
        let sel = rm.link == Some(k);
        let text = format!("{}{}", if sel { ">" } else { " " }, relmap::link_text(l));
        let st = match (&look, sel) {
            (Some(lk), true) => lk.selected(),
            (None, true) => Style::default().add_modifier(Modifier::REVERSED),
            _ => Style::default(),
        };
        out.push(Line::from(Span::styled(
            fit(&sanitize(&text), w, Align::Left),
            st,
        )));
    }
    out
}

/// 横に並べる(各窓は同じ高さの行の並び)。
fn side_by_side(cols: Vec<Vec<Line<'static>>>) -> Vec<Line<'static>> {
    let h = cols.iter().map(Vec::len).max().unwrap_or(0);
    (0..h)
        .map(|y| {
            let mut spans = Vec::new();
            for c in &cols {
                if let Some(l) = c.get(y) {
                    spans.extend(l.spans.clone());
                }
            }
            Line::from(spans)
        })
        .collect()
}

/// 関係マップの画面の全部(上の端・窓・下の帯・メッセージ行)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(rm) = app.relmap.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let mut lines = vec![super::bands::header(app, w)];
    let g = geo(w, h);
    let (top_h, linked_h) = (g.top_h, g.linked_h);
    let title_map = Msg::RelMapTitle.text();
    let (_, mw) = g.map;
    let map_panel = |body: Vec<Line<'static>>| panel(app, title_map, body, mw, top_h, true);
    let detail_panel = |dw: usize| {
        panel(
            app,
            Msg::RelMapDetail.text(),
            detail_body(app, rm, dw.saturating_sub(2)),
            dw,
            top_h,
            false,
        )
    };
    let mut body = match (g.panes, g.list, g.detail) {
        (Panes::One, _, _) => {
            let skip = one_scroll(rm, top_h.saturating_sub(2));
            map_panel(
                one_body(app, rm, mw.saturating_sub(2))
                    .into_iter()
                    .skip(skip)
                    .collect(),
            )
        }
        (_, list, Some((_, dw))) => {
            let mut cols = Vec::new();
            if let Some((_, lw)) = list {
                cols.push(panel(
                    app,
                    Msg::RelMapTables.text(),
                    list_body(app, rm, lw.saturating_sub(2)),
                    lw,
                    top_h,
                    false,
                ));
            }
            cols.push(map_panel(map_body(
                app,
                rm,
                mw.saturating_sub(2),
                top_h.saturating_sub(2),
            )));
            cols.push(detail_panel(dw));
            side_by_side(cols)
        }
        // 2つ・3つの段組みには必ず詳細がある。
        (_, _, None) => map_panel(map_body(
            app,
            rm,
            mw.saturating_sub(2),
            top_h.saturating_sub(2),
        )),
    };
    if linked_h > 0 {
        body.extend(panel(
            app,
            Msg::RelMapLinked.text(),
            linked_body(rm, w.saturating_sub(2)),
            w,
            linked_h,
            false,
        ));
    }
    lines.extend(body);
    while lines.len() < h.saturating_sub(2) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(h.saturating_sub(2));
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
