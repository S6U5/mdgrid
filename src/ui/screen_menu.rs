//! 画面の型のメニュー(REL-8): ヘッダーの右の端の「▦ 表 ▾」(関係マップでは「◇ 関係マップ ▾」)のボタンで開く
//! 小さな窓。画面の型(表・関係マップ。関係マップは、つながる表があるときだけ)を並べ、今の型に印を付け、
//! 切り替えのキーがあれば右に添える。選んだ型に切り替える。
//!
//! 窓はボタンのすぐ下に、ボタンの左の端にそろえて重ねる(並べ替えの窓 sorts.rs と同じ作り)。窓の外の
//! クリックと Esc で閉じる。切り替えのキー(関係マップの `R`)は窓を開かずに効く。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::popup;
use super::width::{fit, width, Align};
use mdgrid::i18n::Msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 開いている画面の型のメニュー。
#[derive(Debug, Clone, Default)]
pub(crate) struct ScreenMenu {
    /// 選んでいる行。
    pub sel: usize,
}

/// 画面の型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Table,
    Relations,
}

/// 窓に並べる型(表と、つながる表があれば関係マップ。関係マップの画面ではいつも)。
fn screens(app: &App) -> Vec<Screen> {
    let rel = app.src.notes() && (!app.link_places().is_empty() || app.relmap.is_some());
    let mut out = vec![Screen::Table];
    if rel {
        out.push(Screen::Relations);
    }
    out
}

/// 今の型。
fn current(app: &App) -> Screen {
    if app.relmap.is_some() {
        Screen::Relations
    } else {
        Screen::Table
    }
}

/// 型の (名前, 切り替えのキー)。SR-45: アイコンを使うときは名前の頭に型の記号。
fn entry(app: &App, s: Screen) -> (String, Option<String>) {
    use super::glyphs::{glyph, nerd_on, Glyph};
    let (g, name, key) = match s {
        Screen::Table => (Glyph::Table, Msg::TabScreenTable.text(), None),
        Screen::Relations => (
            Glyph::RelMap,
            Msg::ScreenRelMap.text(),
            keymap::key_for(&app.keys, Mode::Table, Action::RelationMap),
        ),
    };
    let name = if nerd_on(app) {
        format!("{} {name}", glyph(app, g))
    } else {
        name.to_string()
    };
    (name, key)
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    vis: usize,
}

/// 行の中身の幅(印・名前・キー)。
fn inner_width(app: &App) -> usize {
    screens(app)
        .into_iter()
        .map(|s| {
            let (name, key) = entry(app, s);
            // 「> ● 名前  キー」
            4 + width(&name) + key.map_or(0, |k| 2 + width(&k))
        })
        .chain(std::iter::once(width(Msg::ScreenMenuTitle.text())))
        .max()
        .unwrap_or(10)
        + 1
}

fn window(app: &App) -> Option<Geom> {
    app.screen_menu.as_ref()?;
    let n = screens(app).len();
    let (sw, limit) = popup::screen(app);
    let iw = (inner_width(app) + 2).min(sw);
    if iw < 8 {
        return None;
    }
    // ボタンのすぐ下に、ボタンの左の端にそろえて置く(右へはみ出すなら左へずらす)。
    let bx = super::bands::menu_anchor(app);
    // SR-1: ヘッダーの行(上の余白の分だけ下がる)のすぐ下に開く。
    let ay = super::bands::head_y(app) as usize;
    let popup::Place { top, vis } = popup::place(ay, n, limit)?;
    Some(Geom {
        x: bx.map_or(sw - iw, |x| x.min(sw - iw)),
        top,
        iw,
        vis,
    })
}

impl App {
    /// REL-8: 画面の型のメニューを開く(表の画面か関係マップから)。
    pub(crate) fn open_screen_menu(&mut self) {
        let sel = screens(self)
            .iter()
            .position(|s| *s == current(self))
            .unwrap_or(0);
        self.screen_menu = Some(ScreenMenu { sel });
        if window(self).is_none() {
            self.screen_menu = None;
            self.message = Some(Msg::ScreenMenuNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::ScreenMenu);
    }

    /// 窓を閉じて、開く前の画面(表の画面か関係マップ)に戻る。
    pub(crate) fn close_screen_menu(&mut self) {
        self.screen_menu = None;
        let back = if self.relmap.is_some() {
            Mode::Relations
        } else {
            Mode::Table
        };
        self.set_mode(back);
    }

    /// 画面の型のメニューのモードの動作。
    pub(crate) fn screen_menu_action(&mut self, action: Action) {
        let Some(mut m) = self.screen_menu.take() else {
            return self.close_screen_menu();
        };
        let items = screens(self);
        match action {
            Action::Up | Action::Down | Action::Top | Action::Bottom => {
                m.sel = popup::step_sel(m.sel, items.len(), action, 0).unwrap_or(m.sel);
            }
            Action::Run => {
                let pick = items.get(m.sel).copied().unwrap_or(Screen::Table);
                self.close_screen_menu();
                return self.switch_screen(pick);
            }
            Action::Close => return self.close_screen_menu(),
            _ => {}
        }
        self.screen_menu = Some(m);
        self.set_mode(Mode::ScreenMenu);
    }

    /// 選んだ型に切り替える(今の型なら何もしない)。
    fn switch_screen(&mut self, to: Screen) {
        if to == current(self) {
            return;
        }
        match to {
            Screen::Table => {
                self.relmap = None;
                self.set_mode(Mode::Table);
            }
            Screen::Relations => self.open_relmap(),
        }
    }

    /// 窓の上のクリック: 行なら選んで Enter と同じ、窓の外なら閉じる。縁では何もしない。
    pub(crate) fn screen_menu_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_screen_menu();
        };
        match popup::hit(x, y, g.x, g.top, g.iw, g.vis) {
            None => self.close_screen_menu(),
            Some(None) => {}
            Some(Some(k)) => {
                if let Some(m) = self.screen_menu.as_mut() {
                    m.sel = k;
                }
                self.screen_menu_action(Action::Run);
            }
        }
    }
}

/// 窓を重ねる(下の帯より上の行に)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::ScreenMenu {
        return;
    }
    let (Some(m), Some(g)) = (app.screen_menu.as_ref(), window(app)) else {
        return;
    };
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let now = current(app);
    let mut out: Vec<(String, Style)> = vec![(
        popup::top_edge(fr, Msg::ScreenMenuTitle.text(), tw),
        Style::default(),
    )];
    for (i, s) in screens(app).into_iter().enumerate().take(g.vis) {
        let (name, key) = entry(app, s);
        let st = if i == m.sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let sel = if i == m.sel { "> " } else { "  " };
        let mark = if s == now { "● " } else { "  " };
        let head = format!("{sel}{mark}{name}");
        let line = match key {
            Some(k) => {
                let kw = width(&k);
                format!("{}{k} ", fit(&head, tw.saturating_sub(kw + 1), Align::Left))
            }
            None => fit(&head, tw, Align::Left),
        };
        out.push((fr.side(&line), st));
    }
    out.push((fr.bottom(tw), Style::default()));
    popup::blit(app, lines, g.x, g.top, g.iw, w, out);
}
