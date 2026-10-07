//! その場の操作の一覧(SR-24): 選んだセルで今使える操作を、節(セル・列・行・ビューとファイル)に分けて
//! 表示名と今のキーを添えて並べ、選んで実行する。キー(既定 `x`)とセルの右クリックで開く。
//!
//! 一覧の中身は `items` が今の `App` の状態から作る(開いている間に外の変更で状態が変わっても、
//! 描くときと実行するときに作り直す)。実行は一覧を閉じてから、キーで直接行ったときと同じ道
//! (`App::apply` → 表のモードの動作)に渡す。一覧の上で押したキーは、一覧のモードのキー(↑↓・`j` `k`・
//! Enter・Esc)でなければ表のモードのキーとして読み、その動作が一覧にあれば実行する。

use super::app::App;
use super::cell::shown;
use super::grid::Slot;
use super::keymap::{self, Action, Mode};
use super::popup;
use super::view::{data_y, visible_layout};
use super::width::{fit, width, Align};
use mdgrid::i18n::Msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 一覧の節(SR-24 の順)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Cell,
    Column,
    Row,
    View,
}

impl Section {
    fn msg(self) -> Msg {
        match self {
            Section::Cell => Msg::MenuSecCell,
            Section::Column => Msg::MenuSecColumn,
            Section::Row => Msg::MenuSecRow,
            Section::View => Msg::MenuSecView,
        }
    }
}

/// 一覧の1項目。
#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub section: Section,
    /// 今の言語の表示名(キーの表の表示名。割り当てが無ければ既定の表かパレットのコマンドから)。
    pub label: &'static str,
    pub action: Action,
    /// 表のモードの今のキーの見せ方(割り当て直したあと)。無ければ空。
    pub key: String,
    /// 選んだ行にまとめて入れる(CE-10)。今の行が選択の外なら、選択の最初の行へ移ってから編集を始める。
    pub bulk: bool,
}

/// 動作の今の言語の表示名。
fn label_of(app: &App, action: Action) -> &'static str {
    let table = |b: &&keymap::Binding| b.mode == Mode::Table && b.action == action;
    if let Some(b) = app.keys.iter().find(table) {
        return b.text();
    }
    if let Some(b) = keymap::BINDINGS.iter().find(table) {
        return b.text();
    }
    keymap::COMMANDS
        .iter()
        .find(|c| c.action == action)
        .map_or(action.name(), |c| c.text())
}

/// セルを直せるか(読むだけのセル・評価できない式でない。WB-5・CE-8・BV-7)。
fn cell_writable(app: &App, row: &mdgrid::source::RowId, col: &str) -> bool {
    let s = shown(app, row, col);
    s.locked.is_none() && s.unsupported.is_none() && app.src.get(row, col).lock.is_none()
}

/// 今使える操作の一覧(SR-24)。使えない操作(読むだけのセルの編集と空にする、印の無いときや
/// 選んだ行に今の列を直せる行が無いときのまとめて入れる、読むだけの起動での保存・新しいノート・書き出し)は出さない。
pub(crate) fn items(app: &App) -> Vec<Item> {
    let mut want: Vec<(Section, Action, bool)> = Vec::new();
    let cell = app.selected();
    let writable = !app.readonly;
    if let Some((row, col)) = &cell {
        if writable && cell_writable(app, row, col) {
            want.push((Section::Cell, Action::Edit, false));
            want.push((Section::Cell, Action::Clear, false));
        }
        for a in [Action::Copy, Action::OpenEditor, Action::Detail] {
            want.push((Section::Cell, a, false));
        }
    }
    if !app.cols.is_empty() {
        want.push((Section::Column, Action::Sort, false));
        if cell.is_some() {
            want.push((Section::Column, Action::FilterSame, false));
            want.push((Section::Column, Action::HighlightSame, false));
        }
        // NV-9: 列の値の頻度表(数える行があるとき)。
        if !app.rows.is_empty() {
            want.push((Section::Column, Action::Freq, false));
        }
        // CE-28: 列を足す(読むだけの起動では出さない)。
        if !app.readonly {
            want.push((Section::Column, Action::AddColumn, false));
            // CE-29: ノートのキーの列なら、名前の変更と削除。
            let col = &app.cols[app.col.min(app.cols.len() - 1)];
            if !col.starts_with("file.") && !col.starts_with("formula.") {
                want.push((Section::Column, Action::RenameKey, false));
                want.push((Section::Column, Action::DeleteKey, false));
            }
        }
        for a in [
            Action::HideColumn,
            Action::Narrower,
            Action::Wider,
            Action::MoveColumnLeft,
            Action::MoveColumnRight,
        ] {
            want.push((Section::Column, a, false));
        }
    }
    if app.cur_row().is_some() {
        want.push((Section::Row, Action::Mark, false));
        if let Some((_, col)) = cell.as_ref().filter(|_| writable) {
            if app.selection().iter().any(|r| cell_writable(app, r, col)) {
                want.push((Section::Row, Action::Edit, true));
            }
        }
        want.push((Section::Row, Action::CopyRow, false));
    }
    if writable {
        want.push((Section::View, Action::NewNote, false));
    }
    want.push((Section::View, Action::ViewSettings, false));
    if writable {
        want.push((Section::View, Action::ExportBase, false));
        want.push((Section::View, Action::Save, false));
    }
    want.into_iter()
        .map(|(section, action, bulk)| Item {
            section,
            label: if bulk {
                Msg::MenuBulk.text()
            } else {
                label_of(app, action)
            },
            action,
            key: keymap::key_for(&app.keys, Mode::Table, action).unwrap_or_default(),
            bulk,
        })
        .collect()
}

/// 一覧の項目の動作(前置きの続きを一覧の項目に絞る。SR-25)。
pub(crate) fn item_actions(app: &App) -> Vec<Action> {
    items(app).into_iter().map(|it| it.action).collect()
}

/// 一覧の行: 節の見出しか項目(`items` の添字)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    Head(Section),
    Item(usize),
}

fn entries(items: &[Item]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut last = None;
    for (i, it) in items.iter().enumerate() {
        if last != Some(it.section) {
            out.push(Entry::Head(it.section));
            last = Some(it.section);
        }
        out.push(Entry::Item(i));
    }
    out
}

impl App {
    /// 一覧を開く(表のモードから)。最初は先頭の項目を選ぶ。項目が無ければ開かない。
    pub(crate) fn open_menu(&mut self) {
        if self.mode != Mode::Table || items(self).is_empty() {
            return;
        }
        self.menu = Some(0);
        // 窓が入らない低い端末では開かず、理由を出す(見えない一覧にキーを取られないように)。
        if !fits(self) {
            self.menu = None;
            self.message = Some(Msg::MenuNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::Menu);
    }

    /// 一覧を開いたまま端末が変わって窓が入らなくなったら閉じ、開くときと同じ理由を出す
    /// (見えない一覧にキーを取られないように)。
    pub(crate) fn close_menu_if_no_room(&mut self) {
        if self.mode == Mode::Menu && !fits(self) {
            self.close_menu();
            self.message = Some(Msg::MenuNoRoom.text().into());
        }
    }

    /// 一覧を閉じて表に戻る(何もしない)。
    pub(crate) fn close_menu(&mut self) {
        self.menu = None;
        self.set_mode(Mode::Table);
    }

    /// 一覧のモードの動作(↑↓ で選び、Enter で実行、Esc で閉じる)。
    pub(crate) fn menu_action(&mut self, action: Action) {
        let n = items(self).len();
        let sel = self.menu.unwrap_or(0).min(n.saturating_sub(1));
        match action {
            Action::Up | Action::Down | Action::Top | Action::Bottom => {
                self.menu = popup::step_sel(sel, n, action, 0);
            }
            Action::Run => self.run_menu_item(sel),
            Action::Close => self.close_menu(),
            _ => {}
        }
    }

    /// 一覧の上で、一覧のモードに無いキーを表のモードのキーとして読む。続きに一覧の項目の動作がある
    /// 前置きなら待ち(SR-25)、一覧にある動作なら実行する。どちらかなら true。
    pub(crate) fn menu_key(&mut self, name: &str) -> bool {
        let actions = item_actions(self);
        let leads = keymap::continuations(&self.keys, Mode::Table, name)
            .iter()
            .any(|(a, _)| actions.contains(a));
        if leads {
            self.prefix = Some(name.to_string());
            return true;
        }
        let Some(a) = keymap::lookup(&self.keys, Mode::Table, name) else {
            return false;
        };
        match items(self).iter().position(|it| it.action == a) {
            Some(i) => {
                self.message = None;
                self.run_menu_item(i);
                true
            }
            None => false,
        }
    }

    /// 項目を実行する: 一覧を閉じ、キーで直接行ったときと同じ道に渡す。
    fn run_menu_item(&mut self, i: usize) {
        let Some(it) = items(self).into_iter().nth(i) else {
            return self.close_menu();
        };
        self.close_menu();
        if it.bulk {
            self.go_to_selection();
        }
        self.apply(it.action);
    }

    /// 今の行が選択の外なら、選択の最初の行へ移る(まとめて入れる編集が、選んだ行に効くように)。
    fn go_to_selection(&mut self) {
        let sel = self.selection();
        let Some(cur) = self.cur_row() else {
            return;
        };
        if sel.contains(&cur) {
            return;
        }
        let Some(first) = sel.first() else {
            return;
        };
        let at = self
            .slots
            .iter()
            .position(|s| matches!(s, Slot::Row(r) if self.rows.get(*r) == Some(first)));
        if let Some(i) = at {
            self.row = i;
            self.moved();
            self.refresh_if_needed();
        }
    }

    /// セルの右クリック(SR-24): そのセルを選んで一覧を開く。表のセルでなければ何もしない。
    /// 一覧を開いている間の右クリックは、開き直す。
    pub fn right_click(&mut self, x: u16, y: u16) {
        match self.mode {
            Mode::Table => {}
            Mode::Menu => self.close_menu(),
            _ => return,
        }
        let Some((i, j)) = super::view::hit(self, x, y) else {
            return;
        };
        let moved = i != self.row;
        self.row = i;
        self.col = j;
        self.message = None;
        if moved {
            self.moved();
            self.refresh_if_needed();
        }
        self.scroll_into_view();
        self.apply(Action::Menu);
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    /// 見せる最初の行(`entries` の添字)と行の数。
    start: usize,
    vis: usize,
}

/// 選んだセルの位置(桁と行)。見えていなければ None。
pub(crate) fn anchor(app: &App) -> Option<(usize, usize)> {
    if app.row < app.top || app.row >= app.top + app.data_height() {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let mut x = lay.data_x();
    for &(j, cw) in &cols {
        if j == app.col {
            return Some((x, data_y(app) + app.row - app.top));
        }
        x += cw + 1;
    }
    Some((lay.label_x(), data_y(app) + app.row - app.top))
}

/// 項目の行の文字(選んだ印・表示名・キー)。
fn item_text(it: &Item, sel: bool, lw: usize, kw: usize) -> String {
    format!(
        "{}{}  {} ",
        if sel { ">" } else { " " },
        fit(it.label, lw, Align::Left),
        fit(&it.key, kw, Align::Left)
    )
}

/// `limit` は下の帯より上の行の数。窓はセルの下(入らなければ上)に、画面の幅と高さに収める(SR-9)。
fn geometry(
    app: &App,
    items: &[Item],
    ents: &[Entry],
    sel: usize,
    w: usize,
    limit: usize,
) -> Option<Geom> {
    let (ax, ay) = anchor(app).unwrap_or((0, data_y(app)));
    // 見出しと項目が少なくとも MIN_VIS 行見えなければ、窓は入らない(開かない)。
    if w < 8 {
        return None;
    }
    let popup::Place { top, vis } = popup::place(ay, ents.len(), limit)?;
    let lw = items.iter().map(|it| width(it.label)).max().unwrap_or(0);
    let kw = items.iter().map(|it| width(&it.key)).max().unwrap_or(0);
    let inner = ents
        .iter()
        .map(|e| match e {
            Entry::Head(s) => 1 + width(s.msg().text()),
            Entry::Item(_) => 1 + lw + 2 + kw + 1,
        })
        .chain([width(Mode::Menu.label())])
        .max()
        .unwrap_or(0);
    let iw = (inner + 2).min(w);
    // 選んだ項目が見えるように流す。節の最初の項目なら、その見出しも見せる。
    let at = ents
        .iter()
        .position(|e| *e == Entry::Item(sel))
        .unwrap_or(0);
    let head = if at > 0 && matches!(ents[at - 1], Entry::Head(_)) {
        at - 1
    } else {
        at
    };
    let mut start = (at + 1).saturating_sub(vis);
    if start > head && at - head < vis {
        start = head;
    }
    Some(Geom {
        x: ax.min(w - iw),
        top,
        iw,
        start,
        vis,
    })
}

/// 画面の大きさから、窓の置き場所と一覧(項目・行・選んだ項目)。窓が入らなければ None。
/// 描く(`overlay`)・開く・クリックの判定で同じものを使う。
fn window(app: &App) -> Option<(Geom, Vec<Item>, Vec<Entry>, usize)> {
    let items = items(app);
    if items.is_empty() {
        return None;
    }
    let sel = app.menu.unwrap_or(0).min(items.len() - 1);
    let ents = entries(&items);
    let (w, limit) = popup::screen(app);
    let g = geometry(app, &items, &ents, sel, w, limit)?;
    Some((g, items, ents, sel))
}

/// 窓が今の端末に入るか(入らなければ一覧を開かない。SR-24)。
pub(crate) fn fits(app: &App) -> bool {
    window(app).is_some()
}

/// (x, y) にある窓の中身: 窓の外なら None、窓の中なら項目の添字(見出し・縁なら None の中身)。
fn hit(app: &App, x: u16, y: u16) -> Option<Option<usize>> {
    let (g, _, ents, _) = window(app)?;
    let k = popup::hit(x, y, g.x, g.top, g.iw, g.vis)?;
    Some(match k.and_then(|k| ents.get(g.start + k)) {
        Some(Entry::Item(i)) => Some(*i),
        _ => None,
    })
}

impl App {
    /// 一覧の上の左クリック(SR-24): 項目ならその操作を実行して閉じ、窓の外なら閉じるだけ。
    /// 窓の見出しと縁では何もしない。
    pub(crate) fn menu_click(&mut self, x: u16, y: u16) {
        self.message = None;
        match hit(self, x, y) {
            None => self.close_menu(),
            Some(Some(i)) => self.run_menu_item(i),
            Some(None) => {}
        }
    }
}

/// 下の帯より上の行(`lines`)に一覧の窓を重ねる(リストの選択と同じ見せ方)。窓の外の表はそのまま見せる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Menu {
        return;
    }
    let Some((g, items, ents, sel)) = window(app) else {
        return;
    };
    let tw = g.iw - 2;
    let lw = items.iter().map(|it| width(it.label)).max().unwrap_or(0);
    let kw = items.iter().map(|it| width(&it.key)).max().unwrap_or(0);
    let mut out: Vec<(String, Style)> = Vec::new();
    out.push((popup::top_edge(Mode::Menu.label(), tw), Style::default()));
    for e in ents.iter().skip(g.start).take(g.vis) {
        match *e {
            Entry::Head(s) => out.push((
                format!(
                    "|{}|",
                    fit(&format!(" {}", s.msg().text()), tw, Align::Left)
                ),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Entry::Item(i) => {
                let st = if i == sel {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                let text = item_text(&items[i], i == sel, lw, kw);
                out.push((format!("|{}|", fit(&text, tw, Align::Left)), st));
            }
        }
    }
    // 全部が窓に入らず流れるときは、下の縁に「選んでいる番号/項目の数」(リストの選択と同じ印)。
    let edge = if g.vis < ents.len() {
        let count = format!("{}/{}", sel + 1, items.len());
        fit(&count, tw, Align::Left).replace(' ', "-")
    } else {
        "-".repeat(tw)
    };
    out.push((format!("+{edge}+"), Style::default()));
    popup::blit(lines, g.x, g.top, g.iw, w, out);
}
