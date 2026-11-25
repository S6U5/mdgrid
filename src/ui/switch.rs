//! お気に入りのワークスペースの切り替え(`impl App` の続き。workspace.switch-*・workspace.favorite-mark)。
//!
//! - 一覧: 表・関係マップ・木・ガント・カンバン・グラフのどの画面からでも、キー(既定 `W`)かパレットの
//!   「ワークスペースを切り替える」で、画面の左の端、ヘッダーの下に重ねて開く。お気に入り(`workspaces.toml` の
//!   最上位の `favorites`。核の mdgrid::favorites)を入れた順に番号つきで並べ、起動したときに開いた所から今は
//!   離れていれば、その所を「最初に開いた所」として先頭に置く。数字のキーでその番号へ、Enter で選んだ所へ移り、
//!   Esc で何もせずに閉じる(開いた元の画面に戻る)。
//! - 移る先: この起動のあいだにそのワークスペースで最後に見ていた表とビュー。まだ見ていなければ、印の `open` か
//!   最初の表。「最初に開いた所」は起動したときの表で、そこで最後に見ていたビュー。
//! - 移るのは登録した表と同じ道(終わる手順を通してから main が開き直す。ためた変更があれば確かめる)。App は表ごとに
//!   作り直すので、見た表の記録(`Visits`)は main が次の App に渡す。
//! - お気に入りに入れる・外すのは、引き出しのワークスペースの見出し(drawer.rs)と、パレットの
//!   「今のワークスペースをお気に入りにする」(今のワークスペース。もう入っていれば外す)。
//! - 引き出し(workspace.drawer-favorites)とパレットの「ワークスペースを開く」も、最初に開いた所とお気に入りを
//!   並べ、選んだら同じ道(`go_to`)で移る。今いる所を選んだら、閉じて今いる所だと知らせる(workspace.switch-hint)。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::popup;
use super::width::{fit, sanitize, width, Align};
use mdgrid::favorites::{self, Favorite};
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::workspace::{self, Source};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use std::path::{Path, PathBuf};

/// 窓の幅(枠を含む)の下限。
const MIN_W: usize = 24;

/// 今いる所・移る所の目印(比べるのに使う)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Spot {
    /// `workspaces.toml` のワークスペース(`-w` で選んだものも)。
    Named(String),
    /// 印か検知の根(実体のパス)。
    Root(PathBuf),
    /// ワークスペースでない表(実体のパス)。
    Table(PathBuf),
}

/// 見た表(移る前の表と、そこで見ていたビュー)。
#[derive(Debug, Clone)]
struct Visit {
    spot: Spot,
    path: PathBuf,
    view: Option<String>,
}

/// 起動したときに開いた所(workspace.switch-origin)。
#[derive(Debug, Clone)]
struct Origin {
    spot: Spot,
    /// 起動したときの表。
    path: PathBuf,
    /// 一覧に見せる名前(ワークスペースか根か表の名前)。
    label: String,
    /// 起動の `-w`(戻るときに同じ範囲で開く)。
    workspace: Option<String>,
}

/// 表を開き直してもこの起動のあいだ持つ記録(main が次の App に渡す)。
#[derive(Debug, Clone, Default)]
pub struct Visits {
    origin: Option<Origin>,
    /// 見た表(古い順)。
    seen: Vec<Visit>,
    /// 起動の一覧(引数なしの起動の登録した表の一覧)で選んで移った: 移った先を最初に開いた所にする。
    pub(crate) retarget: bool,
}

impl Visits {
    /// その所で最後に見た表とビュー。
    fn last_in(&self, spot: &Spot) -> Option<&Visit> {
        self.seen.iter().rev().find(|v| v.spot == *spot)
    }

    /// その表で最後に見ていたビュー(見ていなければ None)。
    fn last_view(&self, path: &Path) -> Option<Option<String>> {
        self.seen
            .iter()
            .rev()
            .find(|v| workspace::same_path(&v.path, path))
            .map(|v| v.view.clone())
    }
}

/// 一覧の1行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    /// 最初に開いた所。
    Origin,
    /// お気に入り(添字)。
    Fav(usize),
    /// お気に入りが無いときの、入れ方の案内。
    Hint,
}

/// 移る先(切り替えの一覧・引き出し・パレットの「ワークスペースを開く」で同じ道)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Dest {
    /// 最初に開いた所(workspace.switch-origin)。
    Origin,
    /// お気に入りのワークスペース。
    Fav(Favorite),
}

/// 移る先を選んだ窓(移るとき・今いる所のときに閉じる)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Picker {
    Switch,
    Drawer,
    Palette,
}

/// 開いている切り替えの一覧。
#[derive(Debug, Clone)]
pub(crate) struct Switcher {
    sel: usize,
    /// お気に入り(開いたときに読み解いたもの)。
    favs: Vec<Favorite>,
    /// 最初に開いた所を先頭に出す。
    origin: bool,
    /// 今いる所(開いたときに決めたもの。今のお気に入りの印と、選んだ所が今いる所かに使う)。
    here: Option<Spot>,
}

impl Switcher {
    fn rows(&self) -> Vec<Row> {
        let mut out = Vec::new();
        if self.origin {
            out.push(Row::Origin);
        }
        if self.favs.is_empty() {
            out.push(Row::Hint);
        }
        out.extend((0..self.favs.len()).map(Row::Fav));
        out
    }
}

/// お気に入りの目印。
pub(crate) fn spot_of(f: &Favorite) -> Spot {
    match f {
        Favorite::App(n) => Spot::Named(n.clone()),
        Favorite::Marker { root, .. } => Spot::Root(root.clone()),
    }
}

/// 範囲を探しはじめるフォルダ(`.base` ならそのフォルダ。workspace::resolve と同じ)。
fn start_of(p: &Path) -> PathBuf {
    if p.extension().is_some_and(|x| x == "base") {
        p.parent().unwrap_or(p).to_path_buf()
    } else {
        p.to_path_buf()
    }
}

/// 行の文字(選びの印・今の所の印の後ろ)。
fn label(app: &App, s: &Switcher, r: Row) -> String {
    match r {
        // workspace.switch-origin-key: 番号 0 を添える(`0` で選べる)。
        Row::Origin => format!("0 {}", app.origin_label()),
        Row::Fav(k) => format!("{} {}", k + 1, sanitize(s.favs[k].name())),
        Row::Hint => {
            let key = |m: Mode, a: Action| {
                keymap::key_for(&app.keys, m, a).unwrap_or_else(|| format!(":{}", a.name()))
            };
            Msg::SwitchHint.fill(&[
                &key(Mode::Table, Action::WsDrawer),
                &key(Mode::Drawer, Action::WsFavorite),
            ])
        }
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    iw: usize,
    top: usize,
    vis: usize,
    start: usize,
}

/// 下の縁のキー(screen.window-key-hints): 移ると閉じる。キーと名前はキーの表から。
fn edge_hints(app: &App) -> Vec<popup::KeyHint> {
    popup::key_hints(
        app,
        Mode::Switch,
        &[(Action::Run, None), (Action::Close, None)],
    )
}

fn window(app: &App) -> Option<Geom> {
    let s = app.switcher.as_ref()?;
    let rows = s.rows();
    let (sw, limit) = popup::screen(app);
    let inner = rows
        .iter()
        .map(|r| width(&label(app, s, *r)) + 5)
        .chain(std::iter::once(width(Msg::SwitchTitle.text())))
        .max()
        .unwrap_or(0);
    // screen.window-key-hints: 下の縁のキーが入るまで広げる(窓は左の端に置く)。
    let edge = popup::hints_width(&edge_hints(app));
    let iw = popup::widen_for_hints((inner + 2).max(MIN_W), edge, sw);
    let ay = super::bands::head_y(app) as usize;
    let popup::Place { top, vis } = popup::place(ay, rows.len(), limit)?;
    if iw < 12 {
        return None;
    }
    let sel = s.sel.min(rows.len().saturating_sub(1));
    Some(Geom {
        iw,
        top,
        vis,
        start: (sel + 1).saturating_sub(vis),
    })
}

impl App {
    /// 今いる所の目印。フォルダを複数開いていれば None。
    pub(crate) fn here_spot(&self) -> Option<Spot> {
        let here = super::drawer::here(self)?;
        let start = start_of(&here);
        let spot = match &self.scope {
            Some(s) => match s.source {
                Source::App | Source::Chosen => Some(Spot::Named(s.name.clone())),
                Source::Marker => workspace::find_marker(&start).map(Spot::Root),
                Source::Detected(_) => {
                    workspace::detect(&start, &self.workspace_detect).map(|(r, _)| Spot::Root(r))
                }
            },
            None => None,
        };
        Some(spot.unwrap_or(Spot::Table(here)))
    }

    /// 起動したときに開いた所を覚える(main が起動のときと、起動の一覧で選んで移ったときに呼ぶ)。`workspace` は
    /// 起動の `-w`。フォルダを複数開いていれば覚えない(1つの表として開き直せないため)。
    pub fn mark_origin(&mut self, workspace: Option<String>) {
        let (Some(spot), Some(path)) = (self.here_spot(), super::drawer::here(self)) else {
            return;
        };
        let label = match (&spot, &self.scope) {
            (Spot::Table(p), _) => workspace::stem_of(p),
            (_, Some(s)) => s.name.clone(),
            (Spot::Named(n), None) => n.clone(),
            (Spot::Root(r), None) => workspace::stem_of(r),
        };
        self.visits.origin = Some(Origin {
            spot,
            path,
            label,
            workspace,
        });
    }

    /// 今の表とビューを、見た表として覚える(main が別の表へ移る前に呼ぶ)。
    pub fn leave_visit(&mut self) {
        let (Some(spot), Some(path)) = (self.here_spot(), super::drawer::here(self)) else {
            return;
        };
        let view = self.resume_view();
        self.visits.seen.push(Visit { spot, path, view });
    }

    /// workspace.switch-button: ヘッダーに切り替えのボタンを出すか。お気に入りが1つ以上あるか、起動したときに
    /// 開いた所から今は離れているとき。
    pub(crate) fn switch_button_shown(&self) -> bool {
        !self.favorites.is_empty() || self.origin_away()
    }

    /// workspace.switch-open: 切り替えの一覧を開く(今の画面の上に重ねる)。お気に入りの読めない項目は警告を出す。
    pub(crate) fn open_switcher(&mut self) {
        self.message = None;
        let (favs, warns) = self.resolved_favorites();
        let here = self.here_spot();
        let origin = self.origin_away();
        let s = Switcher {
            sel: 0,
            favs,
            origin,
            here,
        };
        self.switcher = Some(s);
        if window(self).is_none() {
            self.switcher = None;
            self.message = Some(Msg::SwitchNoRoom.text().into());
            return;
        }
        if !warns.is_empty() {
            self.message = Some(warns.join(" / "));
        }
        self.set_mode(Mode::Switch);
    }

    /// 一覧を閉じて、開いた元の画面(表か、表でない型の画面)に戻る。
    pub(crate) fn close_switcher(&mut self) {
        self.switcher = None;
        let back = self.screen().mode();
        self.set_mode(back);
    }

    /// 切り替えの一覧のモードの動作。
    pub(crate) fn switcher_action(&mut self, action: Action) {
        let Some(mut s) = self.switcher.take() else {
            return self.close_switcher();
        };
        let rows = s.rows();
        s.sel = s.sel.min(rows.len().saturating_sub(1));
        match action {
            Action::Up | Action::Down | Action::Top | Action::Bottom => {
                s.sel = popup::step_sel(s.sel, rows.len(), action, 0).unwrap_or(s.sel);
            }
            Action::Run => {
                let row = rows.get(s.sel).copied();
                self.switcher = Some(s);
                return self.switch_row(row);
            }
            // workspace.switch-origin-key: `0` は「最初に開いた所」(その行が無ければ何もしない)。
            Action::Choose(0) => {
                if s.origin {
                    self.switcher = Some(s);
                    return self.switch_row(Some(Row::Origin));
                }
            }
            Action::Choose(n) => {
                let k = usize::from(n).saturating_sub(1);
                if k < s.favs.len() {
                    self.switcher = Some(s);
                    return self.switch_row(Some(Row::Fav(k)));
                }
            }
            Action::Close | Action::WsSwitch => return self.close_switcher(),
            _ => {}
        }
        self.switcher = Some(s);
        self.set_mode(Mode::Switch);
    }

    /// 行を選んだ: その所へ移る(今いる所なら閉じて、今いる所だと知らせる)。案内の行では何もしない。
    fn switch_row(&mut self, row: Option<Row>) {
        let Some(s) = self.switcher.clone() else {
            return;
        };
        let dest = match row {
            Some(Row::Origin) => Dest::Origin,
            Some(Row::Fav(k)) => Dest::Fav(s.favs[k].clone()),
            Some(Row::Hint) | None => return,
        };
        self.go_to(dest, Picker::Switch);
    }

    /// 最初に開いた所を並べるか(起動したときに開いた所から今は離れているとき)。
    pub(crate) fn origin_away(&self) -> bool {
        self.visits
            .origin
            .as_ref()
            .is_some_and(|o| Some(&o.spot) != self.here_spot().as_ref())
    }

    /// 「最初に開いた所: <名前>」の文字(引き出し・パレット)。
    pub(crate) fn origin_label(&self) -> String {
        Msg::SwitchOrigin.fill(&[&sanitize(
            self.visits.origin.as_ref().map_or("", |o| o.label.as_str()),
        )])
    }

    /// お気に入りを読み直して読み解く(読めない項目の警告も)。設定のフォルダが無ければ空。
    pub(crate) fn resolved_favorites(&mut self) -> (Vec<Favorite>, Vec<String>) {
        match self.nv.dir.clone() {
            Some(dir) => {
                self.favorites = favorites::entries(&dir);
                favorites::resolve(&dir, &self.workspaces)
            }
            None => (Vec::new(), Vec::new()),
        }
    }

    /// 選んだ窓を閉じる。
    fn close_picker(&mut self, from: Picker) {
        match from {
            Picker::Switch => self.close_switcher(),
            Picker::Drawer => self.close_drawer(),
            Picker::Palette => self.close_palette(),
        }
    }

    /// その所へ移る(切り替えの一覧・引き出し・パレットの同じ道)。今いる所なら、窓を閉じて今いる所だと知らせる
    /// (workspace.switch-hint)。開けない所は理由を出して窓に残る。
    pub(crate) fn go_to(&mut self, dest: Dest, from: Picker) {
        let target = match &dest {
            Dest::Origin => {
                let Some(o) = self.visits.origin.clone() else {
                    return self.close_picker(from);
                };
                let view = self.visits.last_view(&o.path).flatten();
                let place = Place {
                    name: o.label.clone(),
                    group: String::new(),
                    path: o.path.clone(),
                    view,
                };
                Ok((place, o.workspace.clone()))
            }
            Dest::Fav(f) => {
                if self.here_spot().as_ref() == Some(&spot_of(f)) {
                    self.close_picker(from);
                    self.message = Some(Msg::SwitchAlreadyHere.fill(&[&sanitize(f.name())]));
                    return;
                }
                self.favorite_target(f)
            }
        };
        match target {
            Err(e) => self.message = Some(e),
            Ok((place, ws)) => {
                // WS-9: 印に中身を書いた表は、ファイルが無くても開ける。
                if !workspace::openable(&place.path) {
                    self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
                    return;
                }
                self.close_picker(from);
                self.switch_to = Some(place);
                self.switch_select = None;
                // workspaces.toml のワークスペースはその名前の範囲で開き、印・ワークスペースでない所は `-w` を外す。
                self.switch_leave_workspace = ws.is_none();
                self.switch_workspace = ws;
                self.begin_quit();
            }
        }
    }

    /// workspace.switch-target: お気に入りのワークスペースで開く表とビュー(と `-w` にする名前)。この起動のあいだに
    /// そこで最後に見ていた表、無ければ印の `open` か最初の表。
    fn favorite_target(&self, f: &Favorite) -> Result<(Place, Option<String>), String> {
        let ws = match f {
            Favorite::App(n) => Some(n.clone()),
            Favorite::Marker { .. } => None,
        };
        if let Some(v) = self.visits.last_in(&spot_of(f)) {
            if workspace::openable(&v.path) {
                let place = Place {
                    name: workspace::stem_of(&v.path),
                    group: String::new(),
                    path: v.path.clone(),
                    view: v.view.clone(),
                };
                return Ok((place, ws));
            }
        }
        let first = match f {
            Favorite::App(n) => self
                .workspaces
                .iter()
                .find(|w| w.name == *n)
                .and_then(|w| w.tables.first().cloned()),
            Favorite::Marker { root, .. } => workspace::marker_open(root)
                .0
                .or_else(|| workspace::read_marker(root).0.tables.first().cloned()),
        };
        let t = first.ok_or_else(|| Msg::WsEmpty.fill(&[&f.name()]))?;
        // workspaces.toml の相対のパス(`../notes`)は実体のパスにして開く(印に中身を書いた表は実体が無いのでそのまま)。
        let path = std::fs::canonicalize(&t.path).unwrap_or(t.path);
        let place = Place {
            name: t.name,
            group: String::new(),
            path,
            view: t.view,
        };
        Ok((place, ws))
    }

    /// 窓の上のクリック: 行なら選んで Enter と同じ、窓の外なら閉じる。縁では何もしない。
    pub(crate) fn switcher_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_switcher();
        };
        // screen.window-key-hints: 下の縁のキーは、そのキーと同じ。
        if let Some(dx) = popup::on_bottom(x, y, 0, g.iw, g.top + g.vis + 1) {
            if let Some(a) = popup::hint_at(&edge_hints(self), g.iw - 2, "", dx) {
                self.switcher_action(a);
            }
            return;
        }
        match popup::hit(x, y, 0, g.top, g.iw, g.vis) {
            None => self.close_switcher(),
            Some(None) => {}
            Some(Some(k)) => {
                let i = g.start + k;
                let n = self.switcher.as_ref().map_or(0, |s| s.rows().len());
                if i < n {
                    if let Some(s) = self.switcher.as_mut() {
                        s.sel = i;
                    }
                    self.switcher_action(Action::Run);
                }
            }
        }
    }

    /// お気に入りに入れるか外す(もう入っていれば外す)。`entry` は書く形(名前か根のパス)、`label` は見せる名前。
    pub(crate) fn toggle_favorite(&mut self, entry: &str, label: &str) {
        let Some(dir) = self.nv.dir.clone() else {
            self.message = Some(Msg::WsNoConfigDir.into());
            return;
        };
        let on = !favorites::contains(&favorites::entries(&dir), entry);
        match favorites::set(&dir, entry, on) {
            Ok(()) => {
                self.favorites = favorites::entries(&dir);
                let m = if on {
                    Msg::WsFavoriteAdded
                } else {
                    Msg::WsFavoriteRemoved
                };
                self.message = Some(m.fill(&[&label]));
            }
            Err(e) => self.message = Some(Msg::WsSaveError.fill(&[&e])),
        }
    }

    /// workspace.favorite-mark: パレットの「今のワークスペースをお気に入りにする」。`workspaces.toml` のワークスペースは
    /// 名前、印のワークスペースは根のパスで入れる(もう入っていれば外す)。ワークスペースでなければ理由。
    pub(crate) fn favorite_current(&mut self) {
        let scope = self.scope.as_ref().map(|s| (s.source, s.name.clone()));
        let entry = match (scope.as_ref(), self.here_spot()) {
            (Some((Source::App | Source::Chosen, _)), Some(Spot::Named(n))) => n,
            (Some((Source::Marker, _)), Some(Spot::Root(r))) => favorites::marker_entry(&r),
            _ => {
                self.message = Some(Msg::WsFavoriteNoWorkspace.into());
                return;
            }
        };
        let label = scope.map(|(_, n)| n).unwrap_or_default();
        self.toggle_favorite(&entry, &label);
    }

    /// 引き出しの見出し(workspaces.toml のワークスペース)がお気に入りか。
    pub(crate) fn is_favorite_name(&self, name: &str) -> bool {
        favorites::contains(&self.favorites, name)
    }
}

/// workspace.switch-hint: 切り替えの一覧を開いているあいだの下の帯のキー。今までのキー(順位の順)の Enter の
/// 後ろに、数字のキーで番号の所へ移れることを足す。数字のキーは表から引く(`0`〜`9`。割り当てを外したキーは
/// 除き、2つ以上なら最初と最後を範囲にする)。ほかのモードでは None。
pub(crate) fn hints(app: &App) -> Option<Vec<String>> {
    if app.mode != Mode::Switch {
        return None;
    }
    let mut out = keymap::hints(&app.keys, Mode::Switch);
    let keys: Vec<String> = (0..=9)
        .filter_map(|n| keymap::key_for(&app.keys, Mode::Switch, Action::Choose(n)))
        .collect();
    let digits = match keys.as_slice() {
        [] => return Some(out),
        [one] => one.clone(),
        [first, .., last] => Msg::KeyDigitRange.fill(&[first, last]),
    };
    let at = keymap::key_for(&app.keys, Mode::Switch, Action::Run)
        .and_then(|k| out.iter().position(|h| h.starts_with(&format!("{k} "))))
        .map_or(0, |i| i + 1);
    out.insert(at, format!("{digits} {}", Msg::KeySwitchDigits.text()));
    Some(out)
}

/// 窓を重ねる(左の端、ヘッダーの下)。どの画面の型の描画からも呼ぶ。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Switch {
        return;
    }
    let (Some(s), Some(g)) = (app.switcher.as_ref(), window(app)) else {
        return;
    };
    let rows = s.rows();
    let plain = app.ambiguous_wide;
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let mut out: Vec<(String, Style)> = vec![(
        popup::top_edge(fr, Msg::SwitchTitle.text(), tw),
        Style::default(),
    )];
    for k in 0..g.vis {
        let i = g.start + k;
        let Some(r) = rows.get(i) else {
            out.push((fr.side(&" ".repeat(tw)), Style::default()));
            continue;
        };
        let current = matches!(r, Row::Fav(j) if s.here.as_ref() == Some(&spot_of(&s.favs[*j])));
        let mark = match (current, plain) {
            (false, _) => "  ",
            (true, false) => "● ",
            (true, true) => "* ",
        };
        let sel = i == s.sel;
        let t = format!(
            "{}{mark}{}",
            if sel { "> " } else { "  " },
            label(app, s, *r)
        );
        let st = if sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        out.push((fr.side(&fit(&t, tw, Align::Left)), st));
    }
    out.push((
        popup::bottom_hints(fr, tw, "", &edge_hints(app)),
        Style::default(),
    ));
    popup::blit(app, lines, 0, g.top, g.iw, w, out);
}
