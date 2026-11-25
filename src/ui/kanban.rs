//! カンバンの画面(specs/_history/decidespec/kanban/spec.md の KB-1〜KB-6。`impl App` の続き): 今のビューの行を、状態の列の値ごとの
//! レーン(縦の列)に札として並べる。表の画面とは別の画面の型(SC-9)で、画面の型のメニュー(REL-8)とキー `B` で
//! 切り替える。列の設定は印の `kanban = { column }`(KB-1)、無ければ列の名前から推定する(GA-7)。レーンの順は
//! 画面に依存しない `mdgrid::kanban::lane_keys`(列の決まり CE-36 の `values` から)。
//!
//! 札を左右のレーンへ動かすと、その行の列の値を表のセルの直しと同じためる変更(未保存)で書き換える(KB-4)。
//! 保存は表の保存の確かめ(差分)を通る。

use super::app::App;
use super::entry::{parse, Entry};
use super::keymap::{Action, Mode};
use super::relmap::panel;
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::kanban::{self, KanbanSpec, LaneKey};
use mdgrid::source::RowId;
use mdgrid::types::Kind;
use mdgrid::writeback::NewValue;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::{HashMap, HashSet};

/// レーン1つ。
pub(crate) struct Lane {
    pub key: LaneKey,
    /// 札(表の行の順)。
    pub rows: Vec<RowId>,
    /// このレーンで選んでいる札(`rows` の添字。レーンを行き来しても覚えておく)。
    pub sel: usize,
}

/// カンバンの画面の状態(開いている間だけ)。
pub(crate) struct Kanban {
    pub lanes: Vec<Lane>,
    /// 選んでいるレーン(`lanes` の添字)。
    pub lane: usize,
    /// 左端に見せるレーン(幅に収まらないとき)。
    pub left: usize,
    /// 使っている列。
    pub spec: KanbanSpec,
    /// 設定でなく推定した列か。
    pub inferred: bool,
}

impl Kanban {
    /// 選んでいる札の行。
    fn row(&self) -> Option<&RowId> {
        let l = self.lanes.get(self.lane)?;
        l.rows.get(l.sel)
    }
}

/// レーンの最小の幅(区切りを除く)。
const MIN_LANE: usize = 14;

/// 体の始まりの画面の行(ヘッダーの上の余白・ヘッダー・帯の上の余白の下)。
fn body_y(app: &App) -> usize {
    super::bands::top_rows(app) + 1 + super::bands::bar_padding(app)
}

/// 体の高さ(下の帯とメッセージ行を除く)。`h` は描ける行の数。
fn body_h(app: &App, h: usize) -> usize {
    h.saturating_sub(body_y(app) + 2)
}

/// 今の画面の幅から、窓の中の幅(描画 `ui::draw` と同じく右の1列を除き、枠の2列を除く)。
fn inner_now(app: &App) -> usize {
    super::popup::screen(app).0.saturating_sub(2)
}

/// 見せる行の数 `vis` の窓で、`pos` 番目が見えるようにするずらし。
fn scroll(pos: usize, vis: usize) -> usize {
    (pos + 1).saturating_sub(vis.max(1))
}

/// レーンの並べ方: (見せるレーンの数, レーンの幅)。中の幅 `iw` の両端の1字ずつは隠れたレーンの印(`‹` `›`)に取る。
fn fit_lanes(n: usize, iw: usize) -> (usize, usize) {
    let area = iw.saturating_sub(2);
    let n = n.max(1);
    // 区切り(1字)ごとに最小の幅が入る数。
    let fits = ((area + 1) / (MIN_LANE + 1)).max(1);
    let k = n.min(fits);
    let lw = (area.saturating_sub(k - 1) / k).max(1);
    (k, lw)
}

/// 選んでいるレーン `lane` が見えるように、左端のレーンを決める。
fn left_for(lane: usize, left: usize, shown: usize, n: usize) -> usize {
    let shown = shown.max(1);
    let mut left = left.min(n.saturating_sub(shown));
    if lane < left {
        left = lane;
    } else if lane >= left + shown {
        left = lane + 1 - shown;
    }
    left
}

/// 選んだものの見た目(色なしでは反転。SR-15)。
fn sel_style(app: &App) -> Style {
    match super::look::look(app) {
        Some(l) => l.selected(),
        None => Style::default().add_modifier(Modifier::REVERSED),
    }
}

/// レーンの名前(値なしは「(なし)」)。列 `col` の決まりに字と名前があれば「字 名前」(navigation.group-values-icon。
/// 札を動かして書く値は `key` の値のまま)。
fn lane_name(app: &App, col: &str, key: &LaneKey) -> String {
    match key {
        LaneKey::Value(v) if v.is_empty() => Msg::KanbanNoValue.text().to_string(),
        LaneKey::Value(v) => sanitize(
            &app.prof
                .resolved
                .value_shown(col, v)
                .unwrap_or_else(|| v.clone()),
        ),
        LaneKey::NoValue => Msg::KanbanNoValue.text().to_string(),
    }
}

/// レーンの値を、表の入力と同じ読み方(`entry::parse`)で列の型の値にする。型に合わなければ理由(表の入力と同じ文)。
pub(super) fn value_for(app: &App, col: &str, v: &str) -> Result<NewValue, String> {
    let entry = match app.col_type(col) {
        Kind::Number => Entry::Number,
        Kind::Checkbox => Entry::Checkbox,
        Kind::Date => Entry::Date,
        Kind::DateTime => Entry::DateTime,
        Kind::Text | Kind::List => Entry::Text,
    };
    parse(entry, v, app.today, app.date_input())
}

impl App {
    /// KB-1・GA-7: この表のカンバンの列(印の設定か、無ければ列の名前からの推定)と、推定したかどうか。
    /// 読み込みの途中は印の設定を列を待たずに返す。
    pub(crate) fn kanban_spec(&self) -> Option<(KanbanSpec, bool)> {
        // 印の設定があれば、列が出そろうまでは設定のまま、出そろって列が使えなければ出さない(設定を書いた人の
        // 意図に反して推定で開かない。読み終えたあとに外で列が消えたときも)。
        if let Some((spec, _)) = &self.kanban_spec {
            if !self.columns_settled() || self.kanban_column_ok(&spec.column).is_none() {
                return Some((spec.clone(), false));
            }
            return None;
        }
        let cols: Vec<(String, bool)> = self
            .src
            .columns()
            .into_iter()
            .map(|c| {
                let is_list = self.col_type(&c) == Kind::List;
                (c, is_list)
            })
            .collect();
        kanban::infer(&cols).map(|s| (s, true))
    }

    /// 設定の列が使えないなら理由(表に無い・リストの列)。使えれば None。
    fn kanban_column_ok(&self, col: &str) -> Option<String> {
        let computed = col.starts_with("formula.") || col.starts_with("file.");
        if !computed && !self.has_column(col) {
            return Some(Msg::RsnKanbanNoColumn.fill(&[&col]));
        }
        if self.col_type(col) == Kind::List {
            return Some(Msg::RsnKanbanListColumn.fill(&[&col]));
        }
        None
    }

    /// KB-1: この表でカンバンを出せるか。
    pub(crate) fn kanban_ready(&self) -> bool {
        self.kanban_spec().is_some()
    }

    /// KB-1: 列が出そろったときに、設定の列が表に無いかリストなら CLI-3 の形で1回警告する。設定は外さない(あとで
    /// 列ができれば出す。使えないあいだは `kanban_spec` が推定に落ちずに出さない)。
    pub(crate) fn check_kanban_column(&mut self) {
        let Some((spec, at)) = self.kanban_spec.clone() else {
            return;
        };
        let Some(why) = self.kanban_column_ok(&spec.column) else {
            return;
        };
        let w = Msg::SettingWarn.fill(&[
            &format!(
                "{}/{}",
                mdgrid::workspace::MARKER_DIR,
                mdgrid::workspace::MARKER_FILE
            ),
            &format!("{at}.column"),
            &why,
        ]);
        self.message = Some(match self.message.take() {
            Some(m) => format!("{m} / {w}"),
            None => w,
        });
    }

    /// 行のレーンの値(ためた値を重ねた文字の前後の空白を除いたもの)。キーの名前の変更か削除(CE-29)をためた
    /// 行は、元の値で分ける(見せ方の「→ 新しい名前」をレーンの値にして、それを書かないように)。
    fn kanban_value(&self, row: &RowId, col: &str) -> String {
        if matches!(
            self.changes.pending(row, col),
            Some(NewValue::RenameKey(_) | NewValue::DeleteKey)
        ) {
            return self
                .src
                .get(row, col)
                .value
                .as_ref()
                .map(mdgrid::print::value_plain)
                .unwrap_or_default()
                .trim()
                .to_string();
        }
        self.plain(row, col).trim().to_string()
    }

    /// KB-2: 今のビューを通った行をレーンに分ける(レーンの順は列の決まり → 現れた順 → 値なし)。
    fn kanban_lanes(&self, spec: &KanbanSpec) -> Vec<Lane> {
        let values: Vec<String> = self
            .rows
            .iter()
            .map(|r| self.kanban_value(r, &spec.column))
            .collect();
        let rules = self
            .prof
            .resolved
            .rules
            .get(&spec.column)
            .and_then(|r| r.values.clone());
        let keys = kanban::lane_keys(rules.as_deref(), &values);
        let mut lanes: Vec<Lane> = keys
            .into_iter()
            .map(|key| Lane {
                key,
                rows: Vec::new(),
                sel: 0,
            })
            .collect();
        let at: HashMap<LaneKey, usize> = lanes
            .iter()
            .enumerate()
            .map(|(i, l)| (l.key.clone(), i))
            .collect();
        for (r, v) in self.rows.iter().zip(&values) {
            if let Some(i) = at.get(&kanban::key_of(v)) {
                lanes[*i].rows.push(r.clone());
            }
        }
        lanes
    }

    /// KB-2: カンバンの画面を開く。設定も推定も無ければ理由、行が無ければ理由。今の表の行があればその札を選ぶ。
    pub(crate) fn open_kanban(&mut self) {
        let Some((spec, inferred)) = self.kanban_spec() else {
            // 設定はあって列が使えないなら、その理由を言う(「設定が無い」と言わない)。
            let why = self
                .kanban_spec
                .clone()
                .and_then(|(s, _)| self.kanban_column_ok(&s.column));
            self.message = Some(why.unwrap_or_else(|| Msg::KanbanNotSet.text().into()));
            return;
        };
        let mut lanes = self.kanban_lanes(&spec);
        // kanban.open-empty-lanes: 行が無くても、列の決まりに値があれば空のレーンで開く(レーンが1つも無ければ理由)。
        if self.rows.is_empty() && lanes.is_empty() {
            self.message = Some(Msg::KanbanNoRows.text().into());
            return;
        }
        let mut lane = 0;
        if let Some(cur) = self.cur_row() {
            for (i, l) in lanes.iter_mut().enumerate() {
                if let Some(k) = l.rows.iter().position(|r| *r == cur) {
                    l.sel = k;
                    lane = i;
                }
            }
        }
        self.leave_screens();
        self.kanban = Some(Kanban {
            lanes,
            lane,
            left: 0,
            spec,
            inferred,
        });
        self.kanban_follow();
        self.set_mode(Mode::Kanban);
    }

    /// 表の画面に戻る。
    pub(crate) fn close_kanban(&mut self) {
        self.kanban = None;
        self.set_mode(Mode::Table);
    }

    /// KB-4・KB-5: 外で変わった・札を動かした・取り消したときに、今のビューを通った行で組み直す。選んでいた札の行を
    /// レーンを問わず探して選び直し(別のレーンへ移っていればそのレーンで)、ほかのレーンの選びは行で保つ。
    pub(crate) fn rebuild_kanban(&mut self) {
        if self.kanban.is_none() {
            return;
        }
        // 列が出そろって、設定(か推定)の列が使えなくなった(外で消えた・リストになった)ら、古い列で組み直さずに
        // 閉じて理由を出す(札を動かして、無い列やリストの列に書かないように。KB-1)。
        // ヘルプ・パレット・画面の型のメニューを上に開いているあいだは閉じない(窓が消えて、打ったキーが表に効かない
        // ように)。窓を閉じてカンバンに戻った最初のキーで `kanban_action` が閉じる。
        if self.kanban_spec().is_none() && self.columns_settled() && self.mode == Mode::Kanban {
            let why = self
                .kanban_spec
                .clone()
                .and_then(|(s, _)| self.kanban_column_ok(&s.column))
                .unwrap_or_else(|| Msg::KanbanNotSet.text().into());
            self.close_kanban();
            self.message = Some(why);
            return;
        }
        let Some(k) = self.kanban.as_ref() else {
            return;
        };
        let keep = k.row().cloned();
        let keep_lane = k.lanes.get(k.lane).map(|l| l.key.clone());
        let per_lane: HashMap<LaneKey, (Option<RowId>, usize)> = k
            .lanes
            .iter()
            .map(|l| (l.key.clone(), (l.rows.get(l.sel).cloned(), l.sel)))
            .collect();
        let (spec, inferred) = match self.kanban_spec() {
            Some(s) => s,
            None => (k.spec.clone(), k.inferred),
        };
        let mut lanes = self.kanban_lanes(&spec);
        // ほかのレーンの選びは行で保ち、その行がレーンから出た(動かした・外で変わった)ら同じ位置にする。
        for l in &mut lanes {
            if let Some((r, at)) = per_lane.get(&l.key) {
                l.sel = r
                    .as_ref()
                    .and_then(|r| l.rows.iter().position(|x| x == r))
                    .unwrap_or(*at);
            }
            l.sel = l.sel.min(l.rows.len().saturating_sub(1));
        }
        // 選んでいた札のあるレーン、無ければ同じ鍵のレーン、それも無ければ前の位置に近いレーン。
        let found = keep.as_ref().and_then(|r| {
            lanes
                .iter()
                .enumerate()
                .find_map(|(i, l)| l.rows.iter().position(|x| x == r).map(|p| (i, p)))
        });
        let lane = match found {
            Some((i, p)) => {
                lanes[i].sel = p;
                i
            }
            None => keep_lane
                .and_then(|key| lanes.iter().position(|l| l.key == key))
                .unwrap_or(k.lane)
                .min(lanes.len().saturating_sub(1)),
        };
        if let Some(k) = self.kanban.as_mut() {
            k.lanes = lanes;
            k.lane = lane;
            k.spec = spec;
            k.inferred = inferred;
        }
        self.kanban_follow();
    }

    /// 選んでいるレーンが見えるように左端のレーンを直す(今の画面の幅で)。
    fn kanban_follow(&mut self) {
        let iw = inner_now(self);
        if let Some(k) = self.kanban.as_mut() {
            let (shown, _) = fit_lanes(k.lanes.len(), iw);
            k.left = left_for(k.lane, k.left, shown, k.lanes.len());
        }
    }

    /// KB-3: 選んでいる札の行を表の画面で選んで表に戻る。札が無い(空のレーン)か表で見えない(畳んだ親の子・
    /// 畳んだまとまり)なら、カンバンのまま理由を出して false。
    fn kanban_show_in_table(&mut self) -> bool {
        let row = self.kanban.as_ref().and_then(|k| k.row().cloned());
        self.select_in_table(row)
    }

    /// KB-4: 選んでいる札を隣のレーンへ動かし、その行の列の値を隣のレーンの値に書き換える(ためる変更)。
    fn kanban_move(&mut self, right: bool) {
        let Some(k) = self.kanban.as_ref() else {
            return;
        };
        let Some(row) = k.row().cloned() else {
            self.message = Some(Msg::KanbanNoCard.text().into());
            return;
        };
        let to = if right {
            (k.lane + 1 < k.lanes.len()).then(|| k.lane + 1)
        } else {
            k.lane.checked_sub(1)
        };
        let Some(to) = to else {
            self.message = Some(
                if right {
                    Msg::KanbanNoLaneRight
                } else {
                    Msg::KanbanNoLaneLeft
                }
                .text()
                .into(),
            );
            return;
        };
        let LaneKey::Value(v) = k.lanes[to].key.clone() else {
            self.message = Some(Msg::KanbanToNoValue.text().into());
            return;
        };
        let col = k.spec.column.clone();
        // WB-15: 読むだけで開いたら書かない。
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        // 書くのは表にある列だけ。読み込みの途中は列が出そろう前で設定の列を確かめずに開くので、ここでも確かめる
        // (設定の `note.status` のような表に無い名前で、新しいキーに値をためない。リストの列にも書かない)。
        if let Some(why) = self.kanban_column_ok(&col) {
            self.message = Some(why);
            return;
        }
        // 読むだけのセル(計算の列・ファイルの列・型の食い違う列など。BV-7・BV-12)は理由を出す。
        if let Err(m) = self.editable(&row, &col) {
            self.message = Some(m);
            return;
        }
        // CE-29: キーの名前の変更か削除をためた列は、値の書き換えで黙って上書きしない。
        if matches!(
            self.changes.pending(&row, &col),
            Some(NewValue::RenameKey(_) | NewValue::DeleteKey)
        ) {
            self.message = Some(Msg::KanbanKeyPending.fill(&[&sanitize(&col)]));
            return;
        }
        let old = self.kanban_value(&row, &col);
        // 書く値は表の入力と同じ読み方で列の型に合わせる(型に合わない値は理由を出して書かない)。
        let nv = match value_for(self, &col, &v) {
            Ok(nv) => nv,
            Err(m) => {
                self.message = Some(m);
                return;
            }
        };
        // cell-edit.children-done: 親の札を完了のレーンへ動かしたら、印の children のとおりに子孫も扱う。
        let st = self.stage_children(&col, vec![(row.clone(), Some(nv))]);
        // cell-edit.children-reopen: 子も戻すかを選んでから動かす(選んだら札は表の直しと同じく組み直す)。
        if st.asked {
            self.open_children_ask();
            return;
        }
        if let Some((_, why)) = st.refused.into_iter().next() {
            self.message = Some(why);
            return;
        }
        if let Some(s) = st.skips.first() {
            self.message = Some(Msg::CannotCommit.fill(&[&s.reason]));
            return;
        }
        // NV-12: 直した行は、保存か移動の操作まで表の元の位置に留める。
        self.stay.insert(row.clone());
        self.stay.extend(st.extra);
        self.regrid = true;
        let name = card_name(self, &row);
        // navigation.value-labels-cells: 知らせの値もレーンの題と同じ名前で(書いた値は値のまま)。
        let old = lane_name(self, &col, &LaneKey::Value(old));
        let new = lane_name(self, &col, &LaneKey::Value(v));
        let mut msg = Msg::KanbanMoved.fill(&[&name, &old, &new]);
        if let Some(n) = st.notice {
            msg.push_str(" / ");
            msg.push_str(&n);
        }
        self.message = Some(msg);
    }

    /// KB-4: 取り消し・やり直し(カンバンのまま。読むだけでは何もしない)。
    pub(crate) fn kanban_undo(&mut self, redo: bool) {
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        let rows = self.changes.rows();
        let done = if redo {
            self.changes.redo()
        } else {
            self.changes.undo()
        };
        if done {
            self.keep_in_place(rows);
        } else {
            self.message = Some(
                if redo {
                    Msg::NothingToRedo
                } else {
                    Msg::NothingToUndo
                }
                .text()
                .into(),
            );
        }
    }

    /// KB-3・KB-4: カンバンの画面の動作。
    pub(crate) fn kanban_action(&mut self, action: Action) {
        if self.kanban.is_none() {
            return self.set_mode(Mode::Table);
        }
        // 窓を開いているあいだに設定の列が使えなくなっていたら(外の変化)、ここで閉じて理由を出す(動作はしない。
        // 無い列やリストの列に書かないため)。
        if self.kanban_spec().is_none() && self.columns_settled() {
            return self.rebuild_kanban();
        }
        match action {
            Action::Up | Action::Down => {
                if let Some(k) = self.kanban.as_mut() {
                    if let Some(l) = k.lanes.get_mut(k.lane) {
                        l.sel = if action == Action::Up {
                            l.sel.saturating_sub(1)
                        } else {
                            (l.sel + 1).min(l.rows.len().saturating_sub(1))
                        };
                    }
                }
            }
            Action::Left | Action::Right => {
                if let Some(k) = self.kanban.as_mut() {
                    k.lane = if action == Action::Left {
                        k.lane.saturating_sub(1)
                    } else {
                        (k.lane + 1).min(k.lanes.len().saturating_sub(1))
                    };
                }
                self.kanban_follow();
            }
            Action::MoveCardLeft => self.kanban_move(false),
            Action::MoveCardRight => self.kanban_move(true),
            Action::Undo => self.kanban_undo(false),
            Action::Redo => self.kanban_undo(true),
            // KB-4: 保存は表の保存の確かめ(差分)を通る。
            Action::Save => {
                self.close_kanban();
                self.apply(Action::Save);
            }
            Action::Run => {
                self.kanban_show_in_table();
            }
            Action::Close | Action::KanbanView => self.close_kanban(),
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            // パレットから選んだカンバンに無い動作は、表に戻ってする。選んでいる札の行を表で選べればそこで。
            // 選べなければ(空のレーン・表で見えない行)、行やセルに効く動作だけしない(表の別の行に書かない)。
            a => {
                let row = self.kanban.as_ref().and_then(|k| k.row().cloned());
                self.act_from_screen(row, a, Msg::KanbanNoCard);
            }
        }
    }

    /// 画面の (x, y) が指すレーンと、札の行の番号(見出しなら None)。
    fn kanban_hit(&self, x: u16, y: u16) -> Option<(usize, Option<usize>)> {
        let k = self.kanban.as_ref()?;
        let h = super::popup::screen(self).1 + 2;
        let top = body_y(self) + 1; // 枠の上の縁
        let vis = body_h(self, h).saturating_sub(2);
        let y = usize::from(y);
        if y < top || y >= top + vis {
            return None;
        }
        let (shown, lw) = fit_lanes(k.lanes.len(), inner_now(self));
        // 枠の左の縁と、隠れたレーンの印の1字。
        let x = usize::from(x).checked_sub(2)?;
        let j = x / (lw + 1);
        if j >= shown || x % (lw + 1) == lw {
            return None;
        }
        // 描画(`body_lines`)と同じく今の幅で左端を出す(端末の大きさを変えたあとも見えているレーンに当てる)。
        let lane = left_for(k.lane, k.left, shown, k.lanes.len()) + j;
        let l = k.lanes.get(lane)?;
        match y - top {
            0 => Some((lane, None)),
            1 => None,
            d => {
                let first = scroll(l.sel, vis.saturating_sub(2));
                let i = first + d - 2;
                (i < l.rows.len()).then_some((lane, Some(i)))
            }
        }
    }

    /// KB-3: カンバンの画面のクリック。札のクリックで選び、選んでいる札をもう一度で表のその行へ。見出しのクリックで
    /// そのレーンを選ぶ。
    pub(crate) fn kanban_click(&mut self, x: u16, y: u16) {
        let Some((lane, card)) = self.kanban_hit(x, y) else {
            return;
        };
        let same = self
            .kanban
            .as_ref()
            .is_some_and(|k| k.lane == lane && Some(k.lanes[lane].sel) == card);
        if same {
            self.kanban_show_in_table();
            return;
        }
        if let Some(k) = self.kanban.as_mut() {
            k.lane = lane;
            if let (Some(i), Some(l)) = (card, k.lanes.get_mut(lane)) {
                l.sel = i;
            }
        }
        self.kanban_follow();
    }

    /// KB-3: 横のホイールでレーンを移る。
    pub(crate) fn kanban_hwheel(&mut self, right: bool) {
        self.kanban_action(if right { Action::Right } else { Action::Left });
    }
}

/// 札の名前(表のノートの欄と同じ名前の見せ方。`.md` を除き、フォルダは note_path に従う)。字下げ・印は付けない。
fn card_name(app: &App, row: &RowId) -> String {
    let name = super::view::note_name(app, row);
    if name.trim().is_empty() {
        sanitize(&app.src.label(row))
    } else {
        name
    }
}

/// 窓の題: 「カンバン: status」(推定なら「(推定)」を添える)。列の名前は列の見出しと同じ displayName
/// (base-view.display-name-everywhere)。
fn title(app: &App, k: &Kanban) -> String {
    let col = sanitize(&app.title(&k.spec.column));
    if k.inferred {
        Msg::KanbanTitleInferred.fill(&[&col])
    } else {
        Msg::KanbanTitle.fill(&[&col])
    }
}

/// 体の中身(見出し・区切り・札)。
fn body_lines(app: &App, k: &Kanban, iw: usize, vis: usize) -> Vec<Line<'static>> {
    let n = k.lanes.len();
    let (shown, lw) = fit_lanes(n, iw);
    let left = left_for(k.lane, k.left, shown, n);
    let lanes: Vec<(usize, &Lane)> = k.lanes.iter().enumerate().skip(left).take(shown).collect();
    let hidden_left = left > 0;
    let hidden_right = left + shown < n;
    let sel_st = sel_style(app);
    let pending_st = super::theme::meaning(app, |p| p.pending)
        .or(super::view::pending_color(app.color))
        .map_or(Style::default(), |c| Style::default().fg(c));
    let changed: HashSet<RowId> = app.changes.rows().into_iter().collect();
    let edge = |on: bool, mark: &'static str| Span::raw(if on { mark } else { " " });
    let mut lines = Vec::new();
    // 見出し: 「>値 (数)」(選んでいるレーンは `>` と選びの見た目)。隠れたレーンのある側の端に ‹ ›。
    let mut head = vec![edge(hidden_left, "‹")];
    for (j, (i, l)) in lanes.iter().enumerate() {
        if j > 0 {
            head.push(Span::raw("│"));
        }
        let on = *i == k.lane;
        let text = format!(
            "{}{} ({})",
            if on { ">" } else { " " },
            lane_name(app, &k.spec.column, &l.key),
            l.rows.len()
        );
        let text = fit(&text, lw, Align::Left);
        head.push(if on {
            Span::styled(text, sel_st)
        } else {
            Span::styled(text, Style::default().add_modifier(Modifier::BOLD))
        });
    }
    head.push(edge(hidden_right, "›"));
    lines.push(Line::from(head));
    // 区切りの線。
    let mut rule = String::from(" ");
    for j in 0..lanes.len() {
        if j > 0 {
            rule.push('┼');
        }
        rule.push_str(&"─".repeat(lw));
    }
    lines.push(Line::from(fit(&rule, iw, Align::Left)));
    // 札: 「>*名前」(`>` は選んでいる札、`*` はためた変更のある札)。
    let rows_vis = vis.saturating_sub(2);
    let firsts: Vec<usize> = lanes.iter().map(|(_, l)| scroll(l.sel, rows_vis)).collect();
    for d in 0..rows_vis {
        let mut spans = vec![Span::raw(" ")];
        for (j, (i, l)) in lanes.iter().enumerate() {
            if j > 0 {
                spans.push(Span::raw("│"));
            }
            let Some(row) = l.rows.get(firsts[j] + d) else {
                spans.push(Span::raw(" ".repeat(lw)));
                continue;
            };
            let on = *i == k.lane && firsts[j] + d == l.sel;
            let dirty = changed.contains(row);
            let head = format!(
                "{}{}",
                if on { ">" } else { " " },
                if dirty { "*" } else { " " }
            );
            let name = card_name(app, row);
            let text = fit(
                &format!(
                    "{head}{}",
                    fit(&name, lw.saturating_sub(width(&head)), Align::Left)
                ),
                lw,
                Align::Left,
            );
            spans.push(if on {
                Span::styled(text, sel_st)
            } else if dirty {
                Span::styled(text, pending_st)
            } else {
                Span::raw(text)
            });
        }
        spans.push(Span::raw(" "));
        lines.push(Line::from(spans));
    }
    lines
}

/// 画面の全部の行(幅 `w`、高さ `h`)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(k) = app.kanban.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let mut lines = super::bands::top_lines(app, w);
    lines.push(super::bands::header(app, w));
    for _ in 0..super::bands::bar_padding(app) {
        lines.push(Line::from(" ".repeat(w)));
    }
    let bh = h.saturating_sub(lines.len() + 2);
    let iw = w.saturating_sub(2);
    let body = body_lines(app, k, iw, bh.saturating_sub(2));
    lines.extend(panel(app, &title(app, k), body, w, bh, true));
    let body_end = h.saturating_sub(2);
    while lines.len() < body_end {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body_end);
    // REL-8: 画面の型のメニューの窓(カンバンから開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // workspace.switch-open: お気に入りのワークスペースの切り替えの一覧。
    super::switch::overlay(app, &mut lines, w);
    // パレット(カンバンから開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
