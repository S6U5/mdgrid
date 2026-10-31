//! 木の画面(specs/tree/spec.md の TR-1〜TR-7。`impl App` の続き): 階層を文字の列で持つ表を、左のフォルダの木と、
//! 右の選んだフォルダの直下の行で見せる。表の画面とは別の画面の型(SC-9)で、画面の型のメニュー(REL-8)と
//! キー `T` で切り替える。木を組むのは画面に依存しない `mdgrid::folder_tree`。
//!
//! 木は開いたときの表の行(今のビューを通った行。TR-1)で組み、外で変わったら組み直す(開閉と選びは鍵で保つ)。
//! 段組み(TR-6): 描ける幅 76 以上は左右に並べ、未満は1つずつにして上のタブの行と Tab で切り替える。窓は関係マップと
//! 同じ枠と題(relmap.rs の `panel`)。右の欄の行の操作(URL を開く・表でこの行を見る)は `x` の小さな窓(TR-5)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::relmap::{panel, side_by_side};
use super::width::{fit, sanitize, width, Align};
use mdgrid::base::Shown;
use mdgrid::folder_tree::{self, Entry, Tree, TreeSpec};
use mdgrid::i18n::Msg;
use mdgrid::source::{RowId, Value};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashSet;

/// 選んでいる欄。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pane {
    Folders,
    Rows,
}

/// 木の画面の状態(開いている間だけ)。
pub(crate) struct FolderTree {
    pub tree: Tree,
    /// 木に入れた行(`Folder::rows` の添字の先)。
    pub rows: Vec<RowId>,
    /// 閉じたフォルダの鍵。
    pub folded: HashSet<String>,
    /// 選んだフォルダ(`tree.folders` の添字)。
    pub sel: usize,
    /// 右の欄で選んだ行(選んだフォルダの直下の行の並びの添字)。
    pub item: usize,
    pub focus: Pane,
    /// 行の操作の窓(TR-5)で選んでいる項目。開いていなければ None。
    pub menu: Option<usize>,
}

impl FolderTree {
    /// 見せるフォルダ(開いているものだけ、前順)。
    pub(crate) fn shown(&self) -> Vec<usize> {
        self.tree.shown(&self.folded)
    }

    /// 選んだフォルダの直下の行(`rows` の添字。order の並び)。
    fn items(&self) -> &[usize] {
        self.tree
            .folders
            .get(self.sel)
            .map_or(&[][..], |f| f.rows.as_slice())
    }

    /// 右の欄で選んでいる行。
    pub(crate) fn row(&self) -> Option<&RowId> {
        self.items().get(self.item).map(|&i| &self.rows[i])
    }

    /// 選んだフォルダの道(根から、`sep` でつなぐ)。
    fn path_of(&self, f: usize, sep: &str) -> String {
        let mut parts = Vec::new();
        let mut up = Some(f);
        while let Some(k) = up {
            parts.push(self.tree.folders[k].name.clone());
            up = self.tree.folders[k].parent;
        }
        parts.reverse();
        parts.join(sep)
    }
}

/// 段組みの寸法(描画とクリックで同じものを使う)。窓の (左の桁, 幅)。体の行は上から。
struct Geo {
    /// 左右に並べるか(狭ければ1つずつ)。
    two: bool,
    left: (usize, usize),
    right: (usize, usize),
    /// 1つずつのときのタブの行の数(0 か 1)。
    tabs: usize,
    /// 窓の高さ。
    ph: usize,
}

/// 幅 `w`・体の高さ `h` の段組み(TR-6)。
fn geo(w: usize, h: usize) -> Geo {
    // 80 桁の端末(描ける幅 79)でも並べる。
    if w >= 76 {
        let lw = (w * 2 / 5).clamp(20, 44);
        Geo {
            two: true,
            left: (0, lw),
            right: (lw, w - lw),
            tabs: 0,
            ph: h,
        }
    } else {
        Geo {
            two: false,
            left: (0, w),
            right: (0, w),
            tabs: 1,
            ph: h.saturating_sub(1),
        }
    }
}

/// 見せる行の数 `vis` の窓で、`pos` 番目が見えるようにするずらし。
fn scroll(pos: usize, vis: usize) -> usize {
    (pos + 1).saturating_sub(vis.max(1))
}

/// 体の始まりの画面の行(ヘッダーの上の余白・ヘッダー・帯の上の余白の下)。
fn body_y(app: &App) -> usize {
    super::bands::top_rows(app) + 1 + super::bands::bar_padding(app)
}

/// 体の高さ(下の帯とメッセージ行を除く)。`h` は描ける行の数。
fn body_h(app: &App, h: usize) -> usize {
    h.saturating_sub(body_y(app) + 2)
}

/// 行の操作の窓の項目(TR-5)。
fn menu_items(app: &App, ft: &FolderTree) -> Vec<(Action, &'static str)> {
    let Some(row) = ft.row() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if app.row_url(row).is_some() {
        out.push((Action::OpenUrl, Msg::KeyOpenUrl.text()));
    }
    out.push((Action::Run, Msg::TreeShowInTable.text()));
    out
}

/// 行の操作の窓の置き場所: (桁, 上の縁の行, 幅, 中の行の数)。
fn menu_geom(app: &App, ft: &FolderTree) -> Option<(usize, usize, usize, usize)> {
    ft.menu?;
    let items = menu_items(app, ft);
    let (w, limit) = popup::screen(app);
    let inner = items
        .iter()
        .map(|(_, l)| width(l) + 2)
        .chain([width(Msg::KeyTreeMenu.text())])
        .max()
        .unwrap_or(10)
        + 1;
    let iw = (inner + 2).min(w);
    let g = geo(w, body_h(app, limit + 2));
    let x = (g.right.0 + 2).min(w.saturating_sub(iw));
    let top = body_y(app) + g.tabs + 2;
    (top + items.len() + popup::FRAME <= limit).then_some((x, top, iw, items.len()))
}

impl App {
    /// TR-1・TR-2: この表で木を出せるか(印に木の設定があり、階層の列が表にある)。読み込みの途中は列を待たない。
    pub(crate) fn tree_ready(&self) -> bool {
        match &self.tree_spec {
            Some((spec, _)) => !self.progress.done || self.tree_column_ok(spec),
            None => false,
        }
    }

    /// 階層の列が表にあるか(ノートのキーの列か、計算の列・ファイルの列)。
    fn tree_column_ok(&self, spec: &TreeSpec) -> bool {
        spec.path.starts_with("formula.")
            || spec.path.starts_with("file.")
            || self.src.columns().contains(&spec.path)
    }

    /// TR-2: 読み終えたときに、階層の列が表に無ければ CLI-3 の形で警告し、木を出さない。
    pub(crate) fn check_tree_column(&mut self) {
        let Some((spec, at)) = self.tree_spec.clone() else {
            return;
        };
        if self.tree_column_ok(&spec) {
            return;
        }
        let w = Msg::SettingWarn.fill(&[
            &format!(
                "{}/{}",
                mdgrid::workspace::MARKER_DIR,
                mdgrid::workspace::MARKER_FILE
            ),
            &format!("{at}.path"),
            &Msg::RsnTreeNoColumn.fill(&[&spec.path]),
        ]);
        self.tree_spec = None;
        self.message = Some(match self.message.take() {
            Some(m) => format!("{m} / {w}"),
            None => w,
        });
    }

    /// 階層の値(リストなら最初の要素。前提の「版1 は最初の要素だけ」)。
    fn tree_text(&self, row: &RowId, col: &str) -> String {
        if self.changes.pending(row, col).is_none() {
            if let Shown::Prop(c) = self.cell(row, col) {
                if let Some(Value::List(items)) = &c.value {
                    return items
                        .first()
                        .map(mdgrid::print::value_plain)
                        .unwrap_or_default();
                }
            }
        }
        self.plain(row, col)
    }

    /// 今のビューを通った行で木を組む。
    fn build_tree(&self, spec: &TreeSpec) -> (Tree, Vec<RowId>) {
        let rows = self.rows.clone();
        let entries: Vec<Entry> = rows
            .iter()
            .map(|r| Entry {
                path: folder_tree::split(&self.tree_text(r, &spec.path), &spec.separator),
                order: spec
                    .order
                    .as_ref()
                    .map(|o| self.plain(r, o))
                    .unwrap_or_default(),
                name: self.src.label(r),
            })
            .collect();
        (folder_tree::build(&entries, Msg::TreeNone.text()), rows)
    }

    /// TR-1: 木の画面を開く。今の表の行があれば、そのフォルダとその行を選ぶ。
    pub(crate) fn open_tree(&mut self) {
        let Some((spec, _)) = self.tree_spec.clone().filter(|_| self.tree_ready()) else {
            self.message = Some(Msg::TreeNotSet.text().into());
            return;
        };
        let (tree, rows) = self.build_tree(&spec);
        if tree.folders.is_empty() {
            self.message = Some(Msg::TreeNoRows.text().into());
            return;
        }
        let mut ft = FolderTree {
            tree,
            rows,
            folded: HashSet::new(),
            sel: 0,
            item: 0,
            focus: Pane::Folders,
            menu: None,
        };
        ft.sel = ft.tree.roots.first().copied().unwrap_or(0);
        if let Some(cur) = self.cur_row() {
            select_row(&mut ft, &cur);
        }
        self.relmap = None;
        self.gantt = None;
        self.folder_tree = Some(ft);
        self.set_mode(Mode::Tree);
    }

    /// 外で変わったら組み直す(開閉・選んだフォルダと行は鍵で保つ)。
    pub(crate) fn rebuild_tree(&mut self) {
        let Some(old) = self.folder_tree.as_ref() else {
            return;
        };
        let Some((spec, _)) = self.tree_spec.clone() else {
            return;
        };
        let key = old.tree.folders.get(old.sel).map(|f| f.key.clone());
        let row = old.row().cloned();
        let (folded, focus) = (old.folded.clone(), old.focus);
        let (tree, rows) = self.build_tree(&spec);
        let mut ft = FolderTree {
            tree,
            rows,
            folded,
            sel: 0,
            item: 0,
            focus,
            menu: None,
        };
        ft.sel = key
            .and_then(|k| ft.tree.find(&k))
            .or_else(|| ft.tree.roots.first().copied())
            .unwrap_or(0);
        if let Some(r) = row {
            if let Some(i) = ft.items().iter().position(|&i| ft.rows[i] == r) {
                ft.item = i;
            }
        }
        self.folder_tree = Some(ft);
    }

    /// 木の画面を閉じて表の画面に戻る。
    pub(crate) fn close_tree(&mut self) {
        self.folder_tree = None;
        self.set_mode(Mode::Table);
    }

    /// TR-4: 右の欄で選んだ行を、表の画面で選ぶ。
    fn tree_show_in_table(&mut self) {
        let Some(row) = self.folder_tree.as_ref().and_then(|f| f.row().cloned()) else {
            return;
        };
        self.close_tree();
        if !self.select_row_id(&row) {
            self.message = Some(Msg::TreeRowHidden.text().into());
        }
    }

    /// 表の行(RowId)を選ぶ。見えなければ偽。
    fn select_row_id(&mut self, id: &RowId) -> bool {
        let at = self.rows.iter().position(|r| r == id).and_then(|r| {
            self.slots
                .iter()
                .position(|s| *s == super::grid::Slot::Row(r))
        });
        match at {
            Some(i) => {
                self.row = i;
                self.scroll_into_view();
                true
            }
            None => false,
        }
    }

    /// TR-5: 行の開いてよい URL(今のビューの列の順に見て、無ければ表の列の順)。
    pub(crate) fn row_url(&self, row: &RowId) -> Option<String> {
        let mut cols: Vec<String> = self.cols.clone();
        for c in self.src.columns() {
            if !cols.contains(&c) {
                cols.push(c);
            }
        }
        cols.iter()
            .find_map(|c| folder_tree::web_url(&self.plain(row, c)).map(str::to_string))
    }

    /// TR-5: 行の URL を開く。開く前に下の帯(メッセージ行)に URL を出す。http・https 以外は開かない。
    pub(crate) fn open_row_url(&mut self, row: &RowId) {
        let Some(url) = self.row_url(row) else {
            self.message = Some(Msg::TreeNoUrl.text().into());
            return;
        };
        self.message = Some(Msg::UrlOpening.fill(&[&sanitize(&url)]));
        if let Err(e) = self.opener.open(&url) {
            self.message = Some(Msg::UrlOpenFailed.fill(&[&sanitize(&url), &e]));
        }
    }

    /// TR-5: 表の画面の `open_url`(パレット・操作の一覧): 今の行の URL を開く。
    pub(crate) fn open_url_here(&mut self) {
        match self.cur_row() {
            Some(r) => self.open_row_url(&r),
            None => self.message = Some(Msg::TreeNoUrl.text().into()),
        }
    }

    /// 木の画面の動作。
    pub(crate) fn tree_action(&mut self, action: Action) {
        let Some(mut ft) = self.folder_tree.take() else {
            return self.set_mode(Mode::Table);
        };
        // TR-5: 行の操作の窓が開いていれば、窓の動作。
        if let Some(sel) = ft.menu {
            let items = menu_items(self, &ft);
            match action {
                Action::Up | Action::Down => {
                    ft.menu = popup::step_sel(sel, items.len(), action, 0);
                    self.folder_tree = Some(ft);
                }
                Action::Run => {
                    ft.menu = None;
                    self.folder_tree = Some(ft);
                    if let Some((a, _)) = items.get(sel) {
                        self.tree_action(*a);
                    }
                }
                _ => {
                    ft.menu = None;
                    self.folder_tree = Some(ft);
                }
            }
            return;
        }
        let shown = ft.shown();
        let pos = shown.iter().position(|&f| f == ft.sel).unwrap_or(0);
        let n_items = ft.items().len();
        let has_kids = !ft.tree.folders[ft.sel].kids.is_empty();
        let key = ft.tree.folders[ft.sel].key.clone();
        let folded = ft.folded.contains(&key);
        match (action, ft.focus) {
            (Action::Up | Action::Down, Pane::Folders) => {
                let next = if action == Action::Up {
                    pos.saturating_sub(1)
                } else {
                    (pos + 1).min(shown.len().saturating_sub(1))
                };
                if let Some(&f) = shown.get(next) {
                    if f != ft.sel {
                        ft.sel = f;
                        ft.item = 0;
                    }
                }
            }
            (Action::Up | Action::Down, Pane::Rows) => {
                ft.item = if action == Action::Up {
                    ft.item.saturating_sub(1)
                } else {
                    (ft.item + 1).min(n_items.saturating_sub(1))
                };
            }
            (Action::Run, Pane::Folders) if !has_kids => {
                if n_items > 0 {
                    ft.focus = Pane::Rows;
                }
            }
            (Action::Toggle, _) | (Action::Run, Pane::Folders) => toggle(&mut ft, &key, has_kids),
            (Action::Right, Pane::Folders) => {
                if has_kids && folded {
                    ft.folded.remove(&key);
                } else if n_items > 0 {
                    ft.focus = Pane::Rows;
                }
            }
            (Action::Left, Pane::Folders) => {
                if has_kids && !folded {
                    ft.folded.insert(key);
                } else if let Some(p) = ft.tree.folders[ft.sel].parent {
                    ft.sel = p;
                    ft.item = 0;
                }
            }
            (Action::Left, Pane::Rows) => ft.focus = Pane::Folders,
            (Action::Right, Pane::Rows) => {}
            (Action::NextSection | Action::PrevSection, _) => {
                ft.focus = match ft.focus {
                    Pane::Folders => Pane::Rows,
                    Pane::Rows => Pane::Folders,
                };
            }
            (Action::Run, Pane::Rows) => {
                self.folder_tree = Some(ft);
                return self.tree_show_in_table();
            }
            (Action::OpenUrl, _) => {
                let row = ft.row().cloned();
                self.folder_tree = Some(ft);
                match row {
                    Some(r) => self.open_row_url(&r),
                    None => self.message = Some(Msg::TreeNoUrl.text().into()),
                }
                return;
            }
            (Action::Menu, _) => {
                if ft.row().is_some() {
                    ft.menu = Some(0);
                    self.folder_tree = Some(ft);
                    if menu_geom(self, self.folder_tree.as_ref().expect("tree")).is_none() {
                        self.folder_tree.as_mut().expect("tree").menu = None;
                        self.message = Some(Msg::MenuNoRoom.text().into());
                    }
                    return;
                }
            }
            (Action::Close | Action::TreeView, _) => return self.close_tree(),
            (Action::Help, _) => {
                self.folder_tree = Some(ft);
                return self.open_help();
            }
            (Action::Palette, _) => {
                self.folder_tree = Some(ft);
                return self.open_palette();
            }
            // パレットから選んだ木に無い動作は、表の画面に戻ってからする。
            (a, _) => {
                self.close_tree();
                return self.apply(a);
            }
        }
        self.folder_tree = Some(ft);
    }

    /// TR-7: 木の画面のクリック。フォルダは選び、開閉の印で開く・閉じる。右の行は選び、選んでいる行をもう一度で
    /// 表のその行へ。狭い画面のタブの行は欄の切り替え。行の操作の窓の項目は実行し、窓の外は閉じる。
    pub(crate) fn tree_click(&mut self, x: u16, y: u16) {
        let Some(ft) = self.folder_tree.as_ref() else {
            return;
        };
        if ft.menu.is_some() {
            let hit = menu_geom(self, ft)
                .and_then(|(gx, top, iw, vis)| popup::hit(x, y, gx, top, iw, vis));
            match hit {
                Some(Some(k)) => {
                    if let Some(t) = self.folder_tree.as_mut() {
                        t.menu = Some(k);
                    }
                    self.tree_action(Action::Run);
                }
                Some(None) => {}
                None => {
                    if let Some(t) = self.folder_tree.as_mut() {
                        t.menu = None;
                    }
                }
            }
            return;
        }
        let (w, limit) = popup::screen(self);
        let by = body_y(self);
        let Some(ry) = (y as usize).checked_sub(by) else {
            return;
        };
        let g = geo(w, body_h(self, limit + 2));
        let x = x as usize;
        if g.tabs == 1 && ry == 0 {
            let (t1, _) = tab_texts(ft);
            let pane = if x < width(&t1) {
                Pane::Folders
            } else {
                Pane::Rows
            };
            if let Some(t) = self.folder_tree.as_mut() {
                t.focus = pane;
            }
            return;
        }
        let ry = ry - g.tabs;
        // 窓の中の (桁, 行)。枠の上なら None。
        let inner = |(left, pw): (usize, usize)| -> Option<(usize, usize)> {
            (x > left && x + 1 < left + pw && ry > 0 && ry + 1 < g.ph)
                .then(|| (x - left - 1, ry - 1))
        };
        let show_left = g.two || ft.focus == Pane::Folders;
        let show_right = g.two || ft.focus == Pane::Rows;
        if let Some((cx, r)) = inner(g.left).filter(|_| show_left) {
            let shown = ft.shown();
            let vis = g.ph.saturating_sub(2);
            let pos = shown.iter().position(|&f| f == ft.sel).unwrap_or(0);
            let Some(&f) = shown.get(r + scroll(pos, vis)) else {
                return;
            };
            let folder = &ft.tree.folders[f];
            // 開閉の印の桁: 選びの印(1桁)と字下げの後ろの2桁。
            let mark = 1 + 2 * folder.depth;
            let on_mark = !folder.kids.is_empty() && (mark..mark + 2).contains(&cx);
            let key = folder.key.clone();
            let t = self.folder_tree.as_mut().expect("tree");
            if on_mark {
                toggle(t, &key, true);
            }
            if t.sel != f {
                t.sel = f;
                t.item = 0;
            }
            t.focus = Pane::Folders;
        } else if let Some((_, r)) = inner(g.right).filter(|_| show_right) {
            // 1行目は列の見出し。
            let Some(r) = r.checked_sub(1) else {
                return;
            };
            let vis = g.ph.saturating_sub(3);
            let k = r + scroll(ft.item, vis);
            if k >= ft.items().len() {
                return;
            }
            let again = ft.item == k && ft.focus == Pane::Rows;
            let t = self.folder_tree.as_mut().expect("tree");
            t.item = k;
            t.focus = Pane::Rows;
            if again {
                self.tree_show_in_table();
            }
        }
    }
}

/// フォルダを開く・閉じる(子のあるフォルダだけ)。
fn toggle(ft: &mut FolderTree, key: &str, has_kids: bool) {
    if !has_kids {
        return;
    }
    if !ft.folded.remove(key) {
        ft.folded.insert(key.to_string());
    }
}

/// 木で、表の行 `cur` のフォルダとその行を選ぶ。
fn select_row(ft: &mut FolderTree, cur: &RowId) {
    let Some(i) = ft.rows.iter().position(|r| r == cur) else {
        return;
    };
    if let Some(f) = ft.tree.folders.iter().position(|f| f.rows.contains(&i)) {
        ft.sel = f;
        ft.item = ft.tree.folders[f]
            .rows
            .iter()
            .position(|&k| k == i)
            .unwrap_or(0);
        // 閉じた親があれば開く(木を開いた直後は全部開いているので何もしない)。
        let mut up = ft.tree.folders[f].parent;
        while let Some(p) = up {
            ft.folded.remove(&ft.tree.folders[p].key);
            up = ft.tree.folders[p].parent;
        }
    }
}

/// 狭い画面のタブの行の2つの文字(選んでいる欄に `>`)。
fn tab_texts(ft: &FolderTree) -> (String, String) {
    let mark = |p: Pane| if ft.focus == p { ">" } else { " " };
    (
        format!(" {}{} ", mark(Pane::Folders), Msg::TreeFolders.text()),
        format!(" {}{} ", mark(Pane::Rows), Msg::TreeRows.text()),
    )
}

/// 選んだものの見た目(色なしでは反転。色に頼らず `>` も付ける。SR-15)。
fn sel_style(app: &App, focused: bool) -> Style {
    match (super::look::look(app), focused) {
        (Some(l), true) => l.selected(),
        (None, true) => Style::default().add_modifier(Modifier::REVERSED),
        (_, false) => Style::default().add_modifier(Modifier::BOLD),
    }
}

/// 左の欄の中身: 見せるフォルダ(字下げ・開閉の印・名前・下までの行の数)。
fn folder_lines(app: &App, ft: &FolderTree, w: usize, vis: usize) -> Vec<Line<'static>> {
    let shown = ft.shown();
    let pos = shown.iter().position(|&f| f == ft.sel).unwrap_or(0);
    shown
        .into_iter()
        .skip(scroll(pos, vis))
        .take(vis)
        .map(|f| {
            let d = &ft.tree.folders[f];
            let sel = f == ft.sel;
            let mark = if d.kids.is_empty() {
                "  "
            } else if ft.folded.contains(&d.key) {
                "▸ "
            } else {
                "▾ "
            };
            let head = format!(
                "{}{}{mark}",
                if sel { ">" } else { " " },
                "  ".repeat(d.depth)
            );
            let count = d.count.to_string();
            let name_w = w.saturating_sub(width(&head) + width(&count) + 1);
            let text = format!(
                "{head}{} {count}",
                fit(&sanitize(&d.name), name_w, Align::Left)
            );
            let st = if sel {
                sel_style(app, ft.focus == Pane::Folders)
            } else {
                Style::default()
            };
            Line::from(Span::styled(fit(&text, w, Align::Left), st))
        })
        .collect()
}

/// 右の欄の中身: 列の見出しと、選んだフォルダの直下の行(ノートの名前と今のビューの列)。
fn row_lines(app: &App, ft: &FolderTree, w: usize, vis: usize) -> Vec<Line<'static>> {
    let items: Vec<&RowId> = ft.items().iter().map(|&i| &ft.rows[i]).collect();
    if items.is_empty() {
        return vec![Line::from(fit(Msg::TreeEmptyFolder.text(), w, Align::Left))];
    }
    let one_line = |s: String| sanitize(s.lines().next().unwrap_or_default());
    let names: Vec<String> = items.iter().map(|r| one_line(app.src.label(r))).collect();
    let head = super::view::LABEL_HEADER.text();
    // 名前の欄はいちばん長い名前まで(幅の半分まで)、列の欄はいちばん長い値まで(24 桁まで)。入る列だけ。
    let nw = names
        .iter()
        .map(|n| width(n))
        .chain([width(head)])
        .max()
        .unwrap_or(4)
        .min((w.saturating_sub(1) / 2).max(8));
    let mut cols: Vec<(String, usize, Vec<String>)> = Vec::new();
    let mut used = 1 + nw;
    for c in &app.cols {
        let vals: Vec<String> = items.iter().map(|r| one_line(app.plain(r, c))).collect();
        let cw = vals
            .iter()
            .map(|v| width(v))
            .chain([width(c)])
            .max()
            .unwrap_or(1)
            .clamp(1, 24);
        if used + 2 + cw > w {
            break;
        }
        used += 2 + cw;
        cols.push((c.clone(), cw, vals));
    }
    let faint = super::look::look(app)
        .map(|l| l.faint())
        .unwrap_or_else(|| Style::default().add_modifier(Modifier::BOLD));
    let mut out = Vec::new();
    let mut h = format!(" {}", fit(head, nw, Align::Left));
    for (c, cw, _) in &cols {
        h.push_str(&format!("  {}", fit(&sanitize(c), *cw, Align::Left)));
    }
    out.push(Line::from(Span::styled(fit(&h, w, Align::Left), faint)));
    let vis = vis.saturating_sub(1);
    let skip = scroll(ft.item, vis);
    for (k, name) in names.iter().enumerate().skip(skip).take(vis) {
        let sel = k == ft.item;
        let mut t = format!(
            "{}{}",
            if sel { ">" } else { " " },
            fit(name, nw, Align::Left)
        );
        for (_, cw, vals) in &cols {
            t.push_str(&format!("  {}", fit(&vals[k], *cw, Align::Left)));
        }
        let st = if sel {
            sel_style(app, ft.focus == Pane::Rows)
        } else {
            Style::default()
        };
        out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
    }
    out
}

/// 木の画面の全部(ヘッダー・窓・下の帯・メッセージ行)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(ft) = app.folder_tree.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let sep = app
        .tree_spec
        .as_ref()
        .map_or(folder_tree::DEFAULT_SEPARATOR, |(s, _)| {
            s.separator.as_str()
        });
    let mut lines = super::bands::top_lines(app, w);
    lines.push(super::bands::header(app, w));
    for _ in 0..super::bands::bar_padding(app) {
        lines.push(Line::from(" ".repeat(w)));
    }
    let bh = h.saturating_sub(lines.len() + 2);
    let g = geo(w, bh);
    let left = |pw: usize| {
        panel(
            app,
            Msg::TreeFolders.text(),
            folder_lines(app, ft, pw.saturating_sub(2), g.ph.saturating_sub(2)),
            pw,
            g.ph,
            ft.focus == Pane::Folders,
        )
    };
    let right = |pw: usize| {
        let title = format!("{}: {}", Msg::TreeRows.text(), ft.path_of(ft.sel, sep));
        panel(
            app,
            &title,
            row_lines(app, ft, pw.saturating_sub(2), g.ph.saturating_sub(2)),
            pw,
            g.ph,
            ft.focus == Pane::Rows,
        )
    };
    if g.two {
        lines.extend(side_by_side(vec![left(g.left.1), right(g.right.1)]));
    } else {
        let (t1, t2) = tab_texts(ft);
        let st = |p: Pane| {
            if ft.focus == p {
                sel_style(app, true)
            } else {
                Style::default()
            }
        };
        let tabs = vec![
            Span::styled(t1, st(Pane::Folders)),
            Span::raw(" "),
            Span::styled(t2, st(Pane::Rows)),
        ];
        let used: usize = tabs.iter().map(|s| width(&s.content)).sum();
        let mut tabs = tabs;
        if used < w {
            tabs.push(Span::raw(" ".repeat(w - used)));
        }
        lines.push(Line::from(tabs));
        lines.extend(match ft.focus {
            Pane::Folders => left(w),
            Pane::Rows => right(w),
        });
    }
    let body = h.saturating_sub(2);
    while lines.len() < body {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body);
    // TR-5: 行の操作の窓。
    if let (Some(sel), Some((x, top, iw, vis))) = (ft.menu, menu_geom(app, ft)) {
        let fr = popup::frame(app);
        let tw = iw - 2;
        let mut out = vec![(
            popup::top_edge(fr, Msg::KeyTreeMenu.text(), tw),
            Style::default(),
        )];
        for (i, (_, label)) in menu_items(app, ft).into_iter().enumerate().take(vis) {
            let st = if i == sel {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let t = format!("{}{label}", if i == sel { "> " } else { "  " });
            out.push((fr.side(&fit(&t, tw, Align::Left)), st));
        }
        out.push((fr.bottom(tw), Style::default()));
        popup::blit(app, &mut lines, x, top, iw, w, out);
    }
    // REL-8: 画面の型のメニューの窓(木から開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // パレット(木から開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
