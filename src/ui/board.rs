//! ボードの画面(specs/board/spec.md の BD-1〜BD-6。`impl App` の続き): 表の行を、切り替えたときにカーソルのあった
//! 列の値ごとのレーンに分け、1行を1枚のカードとして並べる。表の画面とは別の画面の型(SC-9)で、画面の型の
//! メニュー(REL-8)とキー `B` で切り替える。レーンに分けるのは画面に依存しない `mdgrid::board`。
//!
//! カードを隣のレーンへ移すと、その行のまとめる列の値を書き換える変更をためる(BD-4。`Changes::set` の1手)。
//! ファイルは保存の流れ(WB-9)でだけ書く。段組み(BD-5): レーンの幅は描ける幅をレーンの数で分けた幅(下限 18 桁)で、
//! 入りきらないレーンは選んだレーンが見えるように横に送る。窓は関係マップと同じ枠と題(relmap.rs の `panel`)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::relmap::{panel, side_by_side};
use super::startup::READONLY;
use super::width::{fit, sanitize, width, Align};
use mdgrid::board::{self, Lane};
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::Kind;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashSet;

/// レーンの幅の下限(枠を含む)。
const LANE_MIN: usize = 18;
/// カード1枚の行の数(名前・値・空き)。
const CARD_H: usize = 3;

/// ボードの画面の状態(開いている間だけ)。
pub(crate) struct Board {
    /// まとめる列。
    pub col: String,
    pub lanes: Vec<Lane>,
    /// 選んでいるレーン。
    pub lane: usize,
    /// 選んでいるカード(レーンの中の添字)。
    pub card: usize,
}

impl Board {
    /// 選んでいるカードの行。
    pub(crate) fn row(&self) -> Option<&RowId> {
        self.lanes.get(self.lane)?.rows.get(self.card)
    }

    /// 行を選ぶ(その行のあるレーンとカード)。無ければ偽。
    fn select(&mut self, row: &RowId) -> bool {
        for (i, l) in self.lanes.iter().enumerate() {
            if let Some(k) = l.rows.iter().position(|r| r == row) {
                self.lane = i;
                self.card = k;
                return true;
            }
        }
        false
    }

    fn clamp(&mut self) {
        self.lane = self.lane.min(self.lanes.len().saturating_sub(1));
        let n = self.lanes.get(self.lane).map_or(0, |l| l.rows.len());
        self.card = self.card.min(n.saturating_sub(1));
    }
}

/// 段組み: 見せるレーンの始まりと数、レーンの幅、窓の高さ、窓の始まりの画面の行。
struct Geo {
    first: usize,
    n: usize,
    lw: usize,
    ph: usize,
    top: usize,
}

/// 末尾を見せる送り: 位置 `pos` が見える数 `vis` に入るだけ送る。
fn scroll(pos: usize, vis: usize) -> usize {
    (pos + 1).saturating_sub(vis.max(1))
}

/// 窓の始まりの画面の行(ヘッダーの上の余白・ヘッダー・帯の上の余白・ボードの見出しの行の下)。
fn panel_y(app: &App) -> usize {
    super::bands::top_rows(app) + 1 + super::bands::bar_padding(app) + 1
}

/// `w`・`h` は描ける幅と行の数。
fn geo(app: &App, b: &Board, w: usize, h: usize) -> Geo {
    let top = panel_y(app);
    let ph = h.saturating_sub(top + 2);
    let n = (w / LANE_MIN).clamp(1, b.lanes.len().max(1));
    Geo {
        first: scroll(b.lane, n),
        n,
        lw: (w / n).max(1),
        ph,
        top,
    }
}

/// 窓の中に見えるカードの数。
fn cards_vis(ph: usize) -> usize {
    (ph.saturating_sub(2) + 1) / CARD_H
}

/// レーンの見出しの値(空のレーンは「(なし)」)。
fn lane_name(l: &Lane) -> String {
    match &l.value {
        Some(v) => sanitize(v),
        None => Msg::TreeNone.text().to_string(),
    }
}

impl App {
    /// BD-1: その列でボードにまとめられるか(ノートのキーの列で、列の型がテキスト)。
    fn board_column_ok(&self, col: &str) -> bool {
        !col.starts_with("file.") && !col.starts_with("formula.") && self.kind_of(col) == Kind::Text
    }

    /// BD-2: 行のまとめる列の値の文字(ためた値があればその値。空は None)。
    fn board_value(&self, row: &RowId, col: &str) -> Option<String> {
        let text = match self.changes.pending(row, col) {
            Some(NewValue::Str(s)) | Some(NewValue::Date(s)) => s.clone(),
            Some(NewValue::Null) => return None,
            Some(_) => self.plain(row, col),
            None => return self.board_read_value(row, col),
        };
        let one = text.lines().next().unwrap_or_default().to_string();
        (!one.trim().is_empty()).then_some(one)
    }

    /// 行のまとめる列の、読んだ値(ためた値を見ない)のレーンの文字。
    fn board_read_value(&self, row: &RowId, col: &str) -> Option<String> {
        let text = match self.src.get(row, col).value {
            Some(Value::Str(s)) => s,
            Some(Value::Null) | None => return None,
            Some(v) => super::cell::value_plain(&v),
        };
        let one = text.lines().next().unwrap_or_default().to_string();
        (!one.trim().is_empty()).then_some(one)
    }

    /// 今のビューを通った行(BD-1)を、列 `col` の値でレーンに分ける(BD-2)。
    fn build_board(&self, col: &str) -> Vec<Lane> {
        let rows: Vec<(RowId, Option<String>)> = self
            .rows
            .iter()
            .map(|r| (r.clone(), self.board_value(r, col)))
            .collect();
        let order = self
            .prof
            .resolved
            .rules
            .get(col)
            .and_then(|r| r.values.clone())
            .unwrap_or_default();
        board::lanes(&rows, &order)
    }

    /// BD-1: ボードの画面を開く。カーソルのある列でまとめ、今の表の行のカードを選ぶ。
    pub(crate) fn open_board(&mut self) {
        let Some(col) = self.cols.get(self.col).cloned() else {
            self.message = Some(Msg::BoardNotText.text().into());
            return;
        };
        if !self.board_column_ok(&col) {
            self.message = Some(Msg::BoardNotText.text().into());
            return;
        }
        let lanes = self.build_board(&col);
        if lanes.is_empty() {
            self.message = Some(Msg::BoardNoRows.text().into());
            return;
        }
        let mut b = Board {
            col,
            lanes,
            lane: 0,
            card: 0,
        };
        if let Some(cur) = self.cur_row() {
            b.select(&cur);
        }
        self.relmap = None;
        self.folder_tree = None;
        self.board = Some(b);
        self.set_mode(Mode::Board);
    }

    /// ためた変更・外の変化のあとに組み直す(選んだカードは行で保つ)。
    pub(crate) fn rebuild_board(&mut self) {
        let Some(old) = self.board.as_ref() else {
            return;
        };
        let (col, lane, card) = (old.col.clone(), old.lane, old.card);
        let row = old.row().cloned();
        // 開いている間は、レーンを消さない(カードを移して空になったレーンへ、戻せるように)。前からあるレーンは
        // 前の並びのまま、新しく出た値のレーンは「(なし)」の前に足す。
        let mut fresh = self.build_board(&col);
        let mut lanes: Vec<Lane> = Vec::new();
        for l in &old.lanes {
            let rows = match fresh.iter().position(|f| f.value == l.value) {
                Some(i) => fresh.remove(i).rows,
                None => Vec::new(),
            };
            lanes.push(Lane {
                value: l.value.clone(),
                rows,
            });
        }
        for f in fresh {
            let at = lanes
                .iter()
                .position(|l| l.value.is_none())
                .filter(|_| f.value.is_some())
                .unwrap_or(lanes.len());
            lanes.insert(at, f);
        }
        let mut b = Board {
            lanes,
            col,
            lane,
            card,
        };
        if !row.is_some_and(|r| b.select(&r)) {
            b.clamp();
        }
        self.board = Some(b);
    }

    /// ボードの画面を閉じて表の画面に戻る。
    pub(crate) fn close_board(&mut self) {
        self.board = None;
        self.set_mode(Mode::Table);
    }

    /// BD-3: 選んだカードの行を、表の画面で選ぶ。
    fn board_show_in_table(&mut self) {
        let Some(row) = self.board.as_ref().and_then(|b| b.row().cloned()) else {
            return;
        };
        self.close_board();
        if !self.select_row_id(&row) {
            self.message = Some(Msg::TreeRowHidden.text().into());
        }
    }

    /// BD-4: 選んだカードを隣のレーンへ移す(その行のまとめる列の値を、移した先のレーンの値にする変更をためる)。
    fn board_move(&mut self, right: bool) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(b) = self.board.as_ref() else {
            return;
        };
        let Some(row) = b.row().cloned() else {
            return;
        };
        let to = if right {
            b.lane + 1
        } else {
            match b.lane.checked_sub(1) {
                Some(t) => t,
                None => return,
            }
        };
        let Some(target) = b.lanes.get(to) else {
            return;
        };
        let (col, name) = (b.col.clone(), lane_name(target));
        let v = match &target.value {
            Some(v) => NewValue::Str(v.clone()),
            None => NewValue::Null,
        };
        // 読んだ値のレーンへ戻すときは、ためた値を外す(元の値の書き方・型のまま。未保存に数えない)。
        let back = self.changes.pending(&row, &col).is_some()
            && self.board_read_value(&row, &col) == target.value;
        let res = if back {
            match self
                .changes
                .set_each(self.src.as_ref(), &col, &[(row.clone(), None)])
                .pop()
            {
                Some(skip) => Err(skip),
                None => Ok(()),
            }
        } else {
            self.changes.set(self.src.as_ref(), &row, &col, v)
        };
        match res {
            Err(skip) => {
                self.message = Some(format!("{}{}", super::input::read_only_lead(), skip.reason));
            }
            Ok(()) => {
                // NV-12: 値が変わっても、保存までは表の行を元の位置に留める(絞り込みで消えないように)。
                self.stay.insert(row);
                self.refresh();
                self.rebuild_board();
                self.message = Some(Msg::BoardMoved.fill(&[&name]));
            }
        }
    }

    /// ボードの画面のモードの動作。
    pub(crate) fn board_action(&mut self, action: Action) {
        let Some(b) = self.board.as_mut() else {
            return self.set_mode(Mode::Table);
        };
        let lanes = b.lanes.len();
        let cards = b.lanes.get(b.lane).map_or(0, |l| l.rows.len());
        match action {
            Action::Left => {
                b.lane = b.lane.saturating_sub(1);
                b.clamp();
            }
            Action::Right => {
                b.lane = (b.lane + 1).min(lanes.saturating_sub(1));
                b.clamp();
            }
            Action::Up => b.card = b.card.saturating_sub(1),
            Action::Down => b.card = (b.card + 1).min(cards.saturating_sub(1)),
            Action::Top => b.card = 0,
            Action::Bottom => b.card = cards.saturating_sub(1),
            Action::Run => self.board_show_in_table(),
            Action::CardLeft => self.board_move(false),
            Action::CardRight => self.board_move(true),
            Action::Undo | Action::Redo => {
                if self.readonly {
                    self.message = Some(READONLY.into());
                    return;
                }
                let before = self.changes.rows();
                let done = if action == Action::Undo {
                    self.changes.undo()
                } else {
                    self.changes.redo()
                };
                if done {
                    self.stay.extend(before);
                    self.stay.extend(self.changes.rows());
                    self.refresh();
                    self.rebuild_board();
                } else {
                    self.message = Some(
                        if action == Action::Undo {
                            Msg::NothingToUndo
                        } else {
                            Msg::NothingToRedo
                        }
                        .text()
                        .into(),
                    );
                }
            }
            Action::Close | Action::BoardView => self.close_board(),
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            // 保存と、パレットから選んだボードに無い動作は、表の画面に戻ってからする(保存の確認は表の上に開く)。
            a => {
                self.close_board();
                self.apply(a);
            }
        }
    }

    /// BD-6: カードのクリックで選び、選んでいるカードをもう一度クリックしたら表のその行へ。レーンの見出し(窓の上の縁)
    /// のクリックでそのレーンを選ぶ。
    pub(crate) fn board_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let (w, h) = (
            self.size.0.saturating_sub(1) as usize,
            self.size.1.saturating_sub(1) as usize,
        );
        let Some(b) = self.board.as_ref() else {
            return;
        };
        let g = geo(self, b, w, h);
        let (x, y) = (x as usize, y as usize);
        if y < g.top || y >= g.top + g.ph || x >= g.lw * g.n {
            return;
        }
        let lane = g.first + x / g.lw;
        if lane >= b.lanes.len() {
            return;
        }
        let vis = cards_vis(g.ph);
        let first_card = if lane == b.lane {
            scroll(b.card, vis)
        } else {
            0
        };
        let inner = y - g.top;
        let hit_card = (inner >= 1 && inner + 1 < g.ph)
            .then(|| first_card + (inner - 1) / CARD_H)
            .filter(|&k| {
                (inner - 1) % CARD_H < CARD_H - 1
                    && k < first_card + vis
                    && k < b.lanes[lane].rows.len()
            });
        let same = hit_card.is_some_and(|k| lane == b.lane && k == b.card);
        let Some(b) = self.board.as_mut() else {
            return;
        };
        match hit_card {
            Some(_) if same => self.board_show_in_table(),
            Some(k) => {
                b.lane = lane;
                b.card = k;
            }
            None => {
                b.lane = lane;
                b.clamp();
            }
        }
    }
}

/// 選んだものの見た目(色なしでは反転。色に頼らず `>` も付ける。SR-15)。
fn sel_style(app: &App, focused: bool) -> Style {
    match (super::look::look(app), focused) {
        (Some(l), true) => l.selected(),
        (None, true) => Style::default().add_modifier(Modifier::REVERSED),
        (_, false) => Style::default().add_modifier(Modifier::BOLD),
    }
}

/// レーンの中身: カード(1行目に名前、2行目に先頭の2つの列の値、空きの行)。
fn lane_lines(
    app: &App,
    b: &Board,
    i: usize,
    pending: &HashSet<RowId>,
    w: usize,
    vis: usize,
) -> Vec<Line<'static>> {
    let lane = &b.lanes[i];
    let focused = i == b.lane;
    let first = if focused { scroll(b.card, vis) } else { 0 };
    let faint = super::look::look(app)
        .map(|l| l.faint())
        .unwrap_or_default();
    let one_line = |s: String| sanitize(s.lines().next().unwrap_or_default());
    let detail_cols: Vec<&String> = app.cols.iter().filter(|c| **c != b.col).take(2).collect();
    let mut out = Vec::new();
    for (k, row) in lane.rows.iter().enumerate().skip(first).take(vis) {
        let sel = focused && k == b.card;
        let name = card_name(app, row);
        let t = format!(
            "{}{}{}",
            if sel { ">" } else { " " },
            if pending.contains(row) { "*" } else { "" },
            name.trim_start()
        );
        let st = if sel {
            sel_style(app, true)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };
        out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
        let vals: Vec<String> = detail_cols
            .iter()
            .map(|c| one_line(app.plain(row, c)))
            .filter(|v| !v.is_empty())
            .collect();
        let d = format!("  {}", vals.join(" · "));
        out.push(Line::from(Span::styled(fit(&d, w, Align::Left), faint)));
        out.push(Line::from(" ".repeat(w)));
    }
    out
}

/// カードの名前(BD-3): ノートの名前。表の左のノートの欄と同じく、共通のフォルダと `.md` を除き、名前の正規表現
/// (SR-47)を当てる。ノートの欄を出さない表(`file.name` の列がある・`note_path = off`・CSV)でも名前は出し、
/// 表の行の印(親子・WBS・留めた行)は付けない。
fn card_name(app: &App, row: &RowId) -> String {
    let full = app.src.label(row);
    let s = full
        .strip_prefix(app.built.label_prefix.as_str())
        .unwrap_or(&full);
    let s = s.strip_suffix(".md").unwrap_or(s);
    let s = s.lines().next().unwrap_or_default();
    match app.settings.display.over(&app.display).note_name {
        Some(re) => {
            let (dir, base) = match s.rsplit_once('/') {
                Some((d, b)) => (format!("{d}/"), b),
                None => (String::new(), s),
            };
            sanitize(&format!("{dir}{}", re.apply(base)))
        }
        None => sanitize(s),
    }
}

/// ボードの見出しの行: まとめる列と、選んでいるレーンの位置。
fn head_line(app: &App, b: &Board, g: &Geo, w: usize) -> Line<'static> {
    let text = format!(
        " {}: {}  {}/{}",
        Msg::ScreenBoard.text(),
        sanitize(&app.title(&b.col)),
        b.lane + 1,
        b.lanes.len()
    );
    let _ = g;
    Line::from(Span::styled(
        fit(&text, w, Align::Left),
        Style::default().add_modifier(Modifier::BOLD),
    ))
}

/// ボードの画面の全部(ヘッダー・見出しの行・レーンの窓・下の帯・メッセージ行)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(b) = app.board.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let g = geo(app, b, w, h);
    let mut lines = super::bands::top_lines(app, w);
    lines.push(super::bands::header(app, w));
    for _ in 0..super::bands::bar_padding(app) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.push(head_line(app, b, &g, w));
    let pending: HashSet<RowId> = app.changes.rows().into_iter().collect();
    let vis = cards_vis(g.ph);
    let mut cols = Vec::new();
    for i in g.first..(g.first + g.n).min(b.lanes.len()) {
        let lane = &b.lanes[i];
        let first = if i == b.lane { scroll(b.card, vis) } else { 0 };
        // 題は「値 数」と、上下に隠れたカードの印。幅が足りなければ値の側を切り、数と印は残す(BD-2・BD-5)。
        let mut tail = format!(" {}", lane.rows.len());
        if first > 0 {
            tail.push_str(" ^");
        }
        if first + vis < lane.rows.len() {
            tail.push_str(" v");
        }
        // 窓の題に使える幅(枠・角の横の線・前後の空白を除く)。
        let room = g.lw.saturating_sub(5).saturating_sub(width(&tail));
        let name = lane_name(lane);
        let name = if width(&name) > room {
            fit(&name, room, Align::Left)
        } else {
            name
        };
        let title = format!("{name}{tail}");
        cols.push(panel(
            app,
            &title,
            lane_lines(app, b, i, &pending, g.lw.saturating_sub(2), vis),
            g.lw,
            g.ph,
            i == b.lane,
        ));
    }
    if g.ph > 0 {
        let mut rows = side_by_side(cols);
        // BD-5: 左右に隠れたレーンがあれば、レーンの題の行の左端に `<`、右端に `>`。
        if let Some(top) = rows.first_mut() {
            let hidden_left = g.first > 0;
            let hidden_right = g.first + g.n < b.lanes.len();
            mark_ends(top, hidden_left, hidden_right);
        }
        lines.extend(rows.into_iter().map(|l| pad(l, w)));
    }
    let body = h.saturating_sub(2);
    while lines.len() < body {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body);
    // REL-8: 画面の型のメニューの窓(ボードから開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // パレット(ボードから開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}

/// 行の最初の字を `<`、最後の字を `>` に置き換える(幅は変えない。枠の角は幅1の字)。
fn mark_ends(line: &mut Line<'static>, left: bool, right: bool) {
    if left {
        if let Some(s) = line.spans.first_mut() {
            let mut cs: Vec<char> = s.content.chars().collect();
            if let Some(c) = cs.first_mut() {
                *c = '<';
                s.content = cs.into_iter().collect::<String>().into();
            }
        }
    }
    if right {
        if let Some(s) = line.spans.iter_mut().rev().find(|s| !s.content.is_empty()) {
            let mut cs: Vec<char> = s.content.chars().collect();
            if let Some(c) = cs.last_mut() {
                *c = '>';
                s.content = cs.into_iter().collect::<String>().into();
            }
        }
    }
}

/// 行を幅 `w` まで空白で埋める(レーンの幅の端数の分)。
fn pad(mut line: Line<'static>, w: usize) -> Line<'static> {
    let used: usize = line.spans.iter().map(|s| width(&s.content)).sum();
    if used < w {
        line.spans.push(Span::raw(" ".repeat(w - used)));
    }
    line
}
