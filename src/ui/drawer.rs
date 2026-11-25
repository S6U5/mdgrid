//! 表の一覧の引き出し(WS-8): 画面の左から表の上に重ねて出し、今の範囲(WS-6)の表と、ほかのワークスペースの
//! 見出し(畳んで出し、開くとその表)を並べる。選んだ表へ移る(終わる手順を通してから main が開き直す。CLI-19 と
//! 同じ保存の確かめ)。今の範囲の表の下には、「最初に開いた所」(離れているとき)とお気に入りのワークスペースを
//! 並べ、選んだら切り替えの一覧と同じ道で移る(workspace.drawer-favorites。switch.rs の `go_to`)。
//!
//! 普段は出さない(左に自分の一覧を持つ端末の道具の中で、左に帯が2本並ばないように)。ヘッダーのボタン
//! 「ワークスペース」(SR-42)・`w`・パレットで開き、表を選ぶ・Esc・窓の外のクリック・同じボタンかキーで閉じる。
//! 窓はヘッダーの下の行から下の帯の上まで(ヘッダーのボタンは押せるまま)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::switch::{spot_of, Dest, Picker, Spot};
use super::width::{fit, sanitize, width, Align};
use mdgrid::favorites::Favorite;
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::workspace::{self, Source};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 窓の幅(枠を含む)の下限と上限。
const MIN_W: usize = 24;
const MAX_W: usize = 40;
/// 窓の上の縁の行(ヘッダーの下)。
/// 引き出しの上の縁の行(ヘッダーのすぐ下。SR-1 の上の余白の分だけ下がる)。
fn top(app: &App) -> usize {
    super::bands::head_y(app) as usize + 1
}

/// 開いている引き出し。
#[derive(Debug, Clone, Default)]
pub(crate) struct Drawer {
    /// 選んでいる行。
    pub sel: usize,
    /// 開いた見出し(ワークスペースの名前)。
    pub open: Vec<String>,
    /// workspace.drawer-favorites: お気に入りのワークスペース(開いたときに読み解いたもの)。
    favs: Vec<Favorite>,
    /// 「最初に開いた所」を並べる(起動したときに開いた所から今は離れている)。
    origin: bool,
    /// 今いる所(開いたときに決めたもの。お気に入りの今の印に使う)。
    here: Option<Spot>,
}

/// 引き出しの1行。
#[derive(Debug, Clone)]
enum Row {
    /// 表。`ws` は移るときの範囲(ワークスペースの名前。None なら今の起動の選び方のまま)。`child` は見出しの下。
    Table {
        place: Place,
        ws: Option<String>,
        current: bool,
        child: bool,
    },
    /// ほかのワークスペースの見出し。
    Head { name: String, open: bool },
    /// stack.page-open: 今の範囲のページ(表の上に並べる)。`current` は開いているページ。
    Page { name: String, current: bool },
    /// workspace.drawer-favorites: 最初に開いた所。
    Origin,
    /// workspace.drawer-favorites: お気に入りのワークスペース。`current` は今いる所。
    Fav { fav: Favorite, current: bool },
}

/// 並べる行: 今の範囲のページと表、最初に開いた所とお気に入りのワークスペース、そのあとにほかのワークスペースの
/// 見出し(開いていればその表)。
fn rows(app: &App, d: &Drawer) -> Vec<Row> {
    let mut out = Vec::new();
    // workspaces.toml のワークスペースの範囲だけ、その名前で開き直す(印・検知の範囲は workspaces.toml に名前が
    // 無く、`-w` の範囲と登録した表は今の起動の選び方のままで同じ範囲になる)。
    let ws = match &app.scope {
        Some(s) if matches!(s.source, Source::App) => Some(s.name.clone()),
        _ => None,
    };
    // 今の範囲と同じ名前のワークスペースは見出しに出さない(印・検知の範囲も名前で比べる)。
    let own = app.scope.as_ref().map(|s| s.name.as_str());
    // stack.page-open: 今の範囲のページを表の上に。
    let page = app.stack.as_ref().and_then(|s| s.page.as_deref());
    for p in &app.pages {
        out.push(Row::Page {
            name: p.name.clone(),
            current: page == Some(p.name.as_str()),
        });
    }
    let places = app.scope_places();
    // stack.page-drawer-leave: ページを開いているあいだは、今の印はページの見出しだけ(表の行には付けない)。
    let cur = if page.is_some() {
        None
    } else {
        current_index(app, &places)
    };
    for (i, place) in places.into_iter().enumerate() {
        out.push(Row::Table {
            place,
            ws: ws.clone(),
            current: cur == Some(i),
            child: false,
        });
    }
    // workspace.drawer-favorites: 今の範囲の表の下に、最初に開いた所とお気に入り。ページを開いているあいだは、今の
    // 印をページの見出しだけにする(stack.page-drawer-leave)。
    if d.origin {
        out.push(Row::Origin);
    }
    for f in &d.favs {
        out.push(Row::Fav {
            fav: f.clone(),
            current: page.is_none() && d.here.as_ref() == Some(&spot_of(f)),
        });
    }
    for w in app
        .workspaces
        .iter()
        .filter(|w| Some(w.name.as_str()) != own)
    {
        let open = d.open.contains(&w.name);
        out.push(Row::Head {
            name: w.name.clone(),
            open,
        });
        if open {
            for t in &w.tables {
                out.push(Row::Table {
                    place: Place {
                        name: t.name.clone(),
                        group: String::new(),
                        path: t.path.clone(),
                        view: t.view.clone(),
                    },
                    ws: Some(w.name.clone()),
                    current: false,
                    child: true,
                });
            }
        }
    }
    out
}

/// 今の表の行: パスが同じ行(いくつもあれば、今のビューの名前の行を先に)。フォルダを複数開いていれば無し。
fn current_index(app: &App, places: &[Place]) -> Option<usize> {
    let here = here(app)?;
    let same: Vec<usize> = (0..places.len())
        .filter(|&i| workspace::same_path(&places[i].path, &here))
        .collect();
    let view = app.current_view_name();
    same.iter()
        .copied()
        .find(|&i| places[i].view.is_some() && places[i].view == view)
        .or_else(|| same.first().copied())
}

/// 今の表の実体のパス。フォルダを複数開いていれば無し。
pub(crate) fn here(app: &App) -> Option<std::path::PathBuf> {
    let target = app.nv.target.clone();
    if target.to_string_lossy().contains('\n') {
        return None;
    }
    Some(std::fs::canonicalize(&target).unwrap_or(target))
}

/// 行の文字(選びの印の後ろ)。今の表に印、見出しに開け閉めの印、見出しの下の表は字下げ。
fn label(app: &App, r: &Row) -> String {
    let plain = app.ambiguous_wide;
    match r {
        Row::Table {
            place,
            current,
            child,
            ..
        } => {
            let mark = match (*current, plain) {
                (false, _) => "  ",
                (true, false) => "● ",
                (true, true) => "* ",
            };
            let indent = if *child { "  " } else { "" };
            format!("{mark}{indent}{}", sanitize(&place.name))
        }
        Row::Head { name, open } => {
            let arrow = match (*open, plain) {
                (true, false) => "▾",
                (false, false) => "▸",
                (true, true) => "-",
                (false, true) => "+",
            };
            // workspace.favorite-mark: お気に入りのワークスペースの見出しに印。
            let star = match (app.is_favorite_name(name), plain) {
                (false, _) => "",
                (true, false) => " ★",
                (true, true) => " *",
            };
            format!("{arrow} {}{star}", sanitize(name))
        }
        Row::Page { name, current } => {
            let mark = match (*current, plain) {
                (false, _) => "  ",
                (true, false) => "● ",
                (true, true) => "* ",
            };
            format!("{mark}{} · {}", sanitize(name), Msg::DrawerPage.text())
        }
        Row::Origin => format!("  {}", app.origin_label()),
        // お気に入りの行は、見出しと同じく名前の後ろに ★。
        Row::Fav { fav, current } => {
            let mark = match (*current, plain) {
                (false, _) => "  ",
                (true, false) => "● ",
                (true, true) => "* ",
            };
            let star = if plain { " *" } else { " ★" };
            format!("{mark}{}{star}", sanitize(fav.name()))
        }
    }
}

/// 窓の題: 範囲の名前(範囲が無ければ「登録した表」)。
fn title(app: &App) -> String {
    match &app.scope {
        Some(s) => format!(" {} ", sanitize(&s.name)),
        None => Msg::DrawerRegistered.text().to_string(),
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    /// 枠を含む幅。
    iw: usize,
    /// 中身の行の数(窓は下の帯の上まで伸ばす)。
    vis: usize,
    /// 見せる最初の行。
    start: usize,
}

fn window(app: &App) -> Option<Geom> {
    let d = app.drawer.as_ref()?;
    let items: Vec<String> = rows(app, d).iter().map(|r| label(app, r)).collect();
    let (sw, limit) = popup::screen(app);
    let inner = items
        .iter()
        .map(|t| width(t) + 2)
        .chain(std::iter::once(width(&title(app))))
        .max()
        .unwrap_or(0);
    let iw = (inner + 2).clamp(MIN_W, MAX_W).min(sw);
    let vis = limit.saturating_sub(top(app) + popup::FRAME);
    if iw < 12 || vis < popup::MIN_VIS.min(items.len().max(1)) {
        return None;
    }
    let sel = d.sel.min(items.len().saturating_sub(1));
    Some(Geom {
        iw,
        vis,
        start: (sel + 1).saturating_sub(vis),
    })
}

impl App {
    /// WS-8: 引き出しを開く(関係マップからは表の画面に戻ってから)。並べるものが無い・端末が小さければ理由。
    pub(crate) fn open_drawer(&mut self) {
        self.leave_screens();
        // workspace.favorite-mark・workspace.drawer-favorites: 見出しの印とお気に入りの行のために、お気に入りを
        // 読み直す(ほかの窓で変えたかもしれない)。読めない項目の警告は切り替えの一覧と同じくメッセージ行に出す。
        let (favs, warns) = self.resolved_favorites();
        if self.mode != Mode::Table {
            self.set_mode(Mode::Table);
        }
        let mut d = Drawer {
            favs,
            origin: self.origin_away(),
            here: self.here_spot(),
            ..Drawer::default()
        };
        let list = rows(self, &d);
        if list.is_empty() {
            self.message = Some(Msg::DrawerEmpty.text().into());
            return;
        }
        d.sel = list
            .iter()
            .position(|r| matches!(r, Row::Page { current: true, .. }))
            .or_else(|| {
                list.iter()
                    .position(|r| matches!(r, Row::Table { current: true, .. }))
            })
            .unwrap_or(0);
        self.drawer = Some(d);
        // stack.*: 縦に並べた表では、引き出しは選んでいる表を画面いっぱいに出して重ねる。
        self.fit_stack_size();
        if window(self).is_none() {
            self.drawer = None;
            self.message = Some(Msg::DrawerNoRoom.text().into());
            return;
        }
        if !warns.is_empty() {
            self.message = Some(warns.join(" / "));
        }
        self.set_mode(Mode::Drawer);
    }

    pub(crate) fn close_drawer(&mut self) {
        self.drawer = None;
        self.set_mode(Mode::Table);
    }

    /// 引き出しのモードの動作。
    pub(crate) fn drawer_action(&mut self, action: Action) {
        // 端末を小さくして窓が出せなくなったら、見えない選びに効かせずに閉じる。
        if window(self).is_none() {
            self.close_drawer();
            self.message = Some(Msg::DrawerNoRoom.text().into());
            return;
        }
        let Some(mut d) = self.drawer.take() else {
            return self.close_drawer();
        };
        let list = rows(self, &d);
        let n = list.len();
        d.sel = d.sel.min(n.saturating_sub(1));
        let at = list.get(d.sel).cloned();
        match (action, at) {
            (Action::Up | Action::Down | Action::Top | Action::Bottom, _) => {
                d.sel = popup::step_sel(d.sel, n, action, 0).unwrap_or(d.sel);
            }
            (
                Action::Run,
                Some(Row::Table {
                    place, ws, child, ..
                }),
            ) => {
                self.drawer = Some(d);
                return self.drawer_choose(place, ws, child);
            }
            (Action::Run, Some(Row::Head { name, open })) => toggle(&mut d, name, !open),
            // workspace.drawer-favorites: 切り替えの一覧と同じ道で移る(今いる所なら閉じて知らせる)。
            (Action::Run, Some(Row::Origin)) => {
                self.drawer = Some(d);
                return self.go_to(Dest::Origin, Picker::Drawer);
            }
            (Action::Run, Some(Row::Fav { fav, .. })) => {
                self.drawer = Some(d);
                return self.go_to(Dest::Fav(fav), Picker::Drawer);
            }
            // stack.page-open: ページを選ぶと、その並べた形で開く(開いているページなら閉じるだけ)。
            (Action::Run, Some(Row::Page { name, .. })) => {
                self.close_drawer();
                return self.open_page_request(name);
            }
            (Action::Right, Some(Row::Head { name, .. })) => toggle(&mut d, name, true),
            (Action::Left, Some(Row::Head { name, .. })) => toggle(&mut d, name, false),
            // 見出しの下の表で ← → その見出しを畳み、見出しを選ぶ。
            (
                Action::Left,
                Some(Row::Table {
                    child: true,
                    ws: Some(name),
                    ..
                }),
            ) => {
                if let Some(h) = list
                    .iter()
                    .position(|r| matches!(r, Row::Head { name: h, .. } if *h == name))
                {
                    d.sel = h;
                }
                toggle(&mut d, name, false);
            }
            (Action::Close | Action::WsDrawer, _) => return self.close_drawer(),
            // workspace.favorite-mark: 見出しのワークスペースをお気に入りに入れる・外す(見出しでないところでは案内)。
            (Action::WsFavorite, Some(Row::Head { name, .. })) => {
                self.toggle_favorite(&name, &name);
                // お気に入りの行を読み直し、選びは同じ見出しに置く。
                d.favs = self.resolved_favorites().0;
                if let Some(h) = rows(self, &d)
                    .iter()
                    .position(|r| matches!(r, Row::Head { name: h, .. } if *h == name))
                {
                    d.sel = h;
                }
            }
            (Action::WsFavorite, _) => {
                self.message = Some(Msg::WsFavoriteHeadOnly.text().into());
            }
            _ => {}
        }
        self.drawer = Some(d);
        self.set_mode(Mode::Drawer);
    }

    /// 表を選んだ: 今の範囲の行で今の表と同じ実体なら閉じるだけ(行のビューが今と違えばそのビューにする)。
    /// ほかの表と、ほかのワークスペースの行(範囲が変わる)へは終わる手順を通してから移る。無いパスは理由を
    /// 出して引き出しに残る。ページを開いているあいだは、選んでいる表と同じでもページを閉じて、その表を1つで
    /// 開く(stack.page-drawer-leave)。
    fn drawer_choose(&mut self, place: Place, ws: Option<String>, child: bool) {
        let in_page = self.stack.as_ref().is_some_and(|s| s.page.is_some());
        let same =
            !child && !in_page && here(self).is_some_and(|h| workspace::same_path(&place.path, &h));
        if same {
            self.close_drawer();
            if let Some(v) = &place.view {
                if self.current_view_name().as_ref() != Some(v) {
                    self.select_view_named(v);
                }
            }
            return;
        }
        // WS-9: 印に中身を書いた表は、ファイルが無くても開ける。
        if !mdgrid::workspace::openable(&place.path) {
            self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
            return;
        }
        self.close_drawer();
        self.switch_to = Some(place);
        self.switch_select = None;
        self.switch_workspace = ws;
        // 前に取りやめた道の「範囲を外す」を拾わない(`-w` の範囲は今の起動の選び方のまま)。
        self.switch_leave_workspace = false;
        self.begin_quit();
    }

    /// 引き出しの上のクリック: 行なら選んで Enter と同じ、窓の外(ヘッダーのボタンを含む)なら閉じる。
    pub(crate) fn drawer_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_drawer();
        };
        match popup::hit(x, y, 0, top(self), g.iw, g.vis) {
            None => self.close_drawer(),
            Some(None) => {}
            Some(Some(k)) => {
                let i = g.start + k;
                let n = self.drawer.as_ref().map_or(0, |d| rows(self, d).len());
                if i < n {
                    if let Some(d) = self.drawer.as_mut() {
                        d.sel = i;
                    }
                    self.drawer_action(Action::Run);
                }
            }
        }
    }
}

/// 見出しを開く(`open`)か畳む。
fn toggle(d: &mut Drawer, name: String, open: bool) {
    d.open.retain(|n| *n != name);
    if open {
        d.open.push(name);
    }
}

/// 窓を重ねる(左の端、ヘッダーの下から下の帯の上まで)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Drawer {
        return;
    }
    let (Some(d), Some(g)) = (app.drawer.as_ref(), window(app)) else {
        return;
    };
    let items: Vec<String> = rows(app, d).iter().map(|r| label(app, r)).collect();
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let mut out: Vec<(String, Style)> =
        vec![(popup::top_edge(fr, &title(app), tw), Style::default())];
    for k in 0..g.vis {
        let i = g.start + k;
        let (text, st) = match items.get(i) {
            Some(t) if i == d.sel => (
                format!("> {t}"),
                Style::default().add_modifier(Modifier::REVERSED),
            ),
            Some(t) => (format!("  {t}"), Style::default()),
            None => (String::new(), Style::default()),
        };
        out.push((fr.side(&fit(&text, tw, Align::Left)), st));
    }
    out.push((fr.bottom(tw), Style::default()));
    popup::blit(app, lines, 0, top(app), g.iw, w, out);
}
