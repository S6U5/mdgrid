//! 画面の型の一覧(SC-9・REL-8・TR-1・GA-7・KB-2): どの型の画面が開いているか、型を移るときにほかの型の状態を
//! 閉じること、型ごとの名前・記号・切り替えのキーを1か所に集める。型を足すときは、ここの `Screen` と各 `match`
//! に1行ずつ足す(画面の状態を閉じ忘れて2つの型が残らないように)。
//!
//! 画面の状態(`relmap`・`folder_tree`・`gantt`・`kanban`・`chart`)は、開いている型の1つだけが持つ。表の画面はどれも
//! 持たない。

use super::app::App;
use super::glyphs::Glyph;
use super::keymap::{Action, Mode};
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;

/// 画面の型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Table,
    /// REL-7: 関係マップ(つながる表があるときだけ)。
    Relations,
    /// TR-1: 木(表に木の設定があるときだけ)。
    Tree,
    /// GA-2: ガント(設定か推定で列が決まるときだけ)。
    Gantt,
    /// KB-2: カンバン(設定か推定で列が決まるときだけ)。
    Kanban,
    /// CH-2: グラフ(グラフにする列があるときだけ)。
    Chart,
}

/// 表でない画面の型(メニューの並びの順)。
pub(crate) const OTHERS: [Screen; 5] = [
    Screen::Relations,
    Screen::Tree,
    Screen::Gantt,
    Screen::Kanban,
    Screen::Chart,
];

impl Screen {
    /// その型の画面のモード。
    pub(crate) fn mode(self) -> Mode {
        match self {
            Screen::Table => Mode::Table,
            Screen::Relations => Mode::Relations,
            Screen::Tree => Mode::Tree,
            Screen::Gantt => Mode::Gantt,
            Screen::Kanban => Mode::Kanban,
            Screen::Chart => Mode::Chart,
        }
    }

    /// モードがその型の画面のモードなら、その型。
    pub(crate) fn of_mode(m: Mode) -> Option<Screen> {
        std::iter::once(Screen::Table)
            .chain(OTHERS)
            .find(|s| s.mode() == m)
    }

    /// 名前と記号(画面の型のメニューとヘッダーのボタン。SR-45)。
    pub(crate) fn label(self) -> (Glyph, &'static str) {
        match self {
            Screen::Table => (Glyph::Table, Msg::TabScreenTable.text()),
            Screen::Relations => (Glyph::RelMap, Msg::ScreenRelMap.text()),
            Screen::Tree => (Glyph::Tree, Msg::ScreenTree.text()),
            Screen::Gantt => (Glyph::Gantt, Msg::ScreenGantt.text()),
            Screen::Kanban => (Glyph::Kanban, Msg::ScreenKanban.text()),
            Screen::Chart => (Glyph::Chart, Msg::ScreenChart.text()),
        }
    }

    /// 表のモードで、窓を開かずにその型へ切り替える動作。
    pub(crate) fn switch_action(self) -> Option<Action> {
        match self {
            Screen::Table => None,
            Screen::Relations => Some(Action::RelationMap),
            Screen::Tree => Some(Action::TreeView),
            Screen::Gantt => Some(Action::GanttView),
            Screen::Kanban => Some(Action::KanbanView),
            Screen::Chart => Some(Action::ChartView),
        }
    }
}

impl App {
    /// 開いている画面の型(状態を持つ型。無ければ表)。
    pub(crate) fn screen(&self) -> Screen {
        if self.relmap.is_some() {
            Screen::Relations
        } else if self.folder_tree.is_some() {
            Screen::Tree
        } else if self.gantt.is_some() {
            Screen::Gantt
        } else if self.kanban.is_some() {
            Screen::Kanban
        } else if self.chart.is_some() {
            Screen::Chart
        } else {
            Screen::Table
        }
    }

    /// 表でない画面の型の状態を全部閉じる(型を移る前に。モードは変えない)。
    pub(crate) fn leave_screens(&mut self) {
        self.relmap = None;
        self.folder_tree = None;
        self.gantt = None;
        self.kanban = None;
        self.chart = None;
    }

    /// 今の画面の型を閉じて表の画面に戻る。
    pub(crate) fn close_screen(&mut self) {
        self.leave_screens();
        self.set_mode(Mode::Table);
    }

    /// 画面の型の設定の列を確かめてよいか(読み終えて、行が1つ以上ある)。列はノートのキーから作るので、読み込みの
    /// 途中と行の無い表(見本なしで作った台帳など)では分からない。分からないあいだは印の設定をそのまま使い、
    /// 列が無いと警告しない(空の台帳を開くたびに警告が出ないように)。
    /// 列が表にあるか: ノートのキーの列か、設定の `[types]`・`[rules.<列>]` に書いた列
    /// (gantt・kanban・chart・tree の config-declared-column: まだどのノートにも無いキーでも、設定に書いた列は
    /// 表にあると数える)。計算の列・ファイルの列は呼ぶ側が先に見る。
    pub(crate) fn has_column(&self, col: &str) -> bool {
        self.src.columns().iter().any(|x| x == col)
            || self.prof.resolved.types.contains_key(col)
            || self.prof.resolved.rules.contains_key(col)
    }

    pub(crate) fn columns_settled(&self) -> bool {
        self.progress.done && !self.src.rows().is_empty()
    }

    /// 表の画面で行 `row` のある位置(スロット)。畳んだ親の子・畳んだまとまりの行で見えなければ None。
    pub(crate) fn slot_of(&self, row: &RowId) -> Option<usize> {
        let r = self.rows.iter().position(|x| x == row)?;
        self.slots
            .iter()
            .position(|s| *s == super::grid::Slot::Row(r))
    }

    /// 表でない画面の型から、行 `row` を表の画面で選んで表に戻る。見えなければ表に戻らず理由を出し、false
    /// (パレットから選んだ動作を、表の別の行に効かせないため)。
    pub(crate) fn select_in_table(&mut self, row: Option<RowId>) -> bool {
        let at = match &row {
            Some(r) => self.slot_of(r).ok_or(Msg::TreeRowHidden),
            None => Err(Msg::KanbanNoCard),
        };
        match at {
            Ok(i) => {
                self.close_screen();
                self.row = i;
                self.scroll_into_view();
                true
            }
            Err(m) => {
                self.message = Some(m.text().into());
                false
            }
        }
    }

    /// 表でない画面の型で、パレットなどから選んだその画面に無い動作 `a` を、表の画面に戻って効かせる。選んでいる
    /// 行 `row` を表で選べれば選んでから。選べない(行が無い・表で見えない)ときは、行やセルに効く動作
    /// (`Action::ROW_TARGET`)だけその画面のまま理由を出してしない(表の別の行に効かせない)。ほかの動作は表の
    /// 今の行のまま効かせる(パレットで終わる・探す・画面の型を切り替えるなどが止まらないように。SR-14)。
    /// `no_row` は行が無いときの理由。
    pub(crate) fn act_from_screen(&mut self, row: Option<RowId>, a: Action, no_row: Msg) {
        match row.as_ref().and_then(|r| self.slot_of(r)) {
            Some(i) => {
                self.close_screen();
                self.row = i;
                self.scroll_into_view();
                self.apply(a);
            }
            None if Action::ROW_TARGET.contains(&a) => {
                let m = if row.is_some() {
                    Msg::TreeRowHidden
                } else {
                    no_row
                };
                self.message = Some(m.text().into());
            }
            None => {
                self.close_screen();
                self.apply(a);
            }
        }
    }

    /// 画面の型の設定の列を、列が出そろったときに1回だけ確かめて、無い列を警告する(読み終えたとき、行の無かった
    /// 表に行ができたとき)。設定は外さず、出せるかはそのたび列があるかで決める(あとで列ができれば出す)。今の列で
    /// 出せない型の画面が開いていれば(読み込みの途中で開いた画面)閉じる。
    pub(crate) fn check_screen_columns(&mut self) {
        if self.columns_checked || !self.columns_settled() {
            return;
        }
        self.columns_checked = true;
        // TR-2: 階層の列が表に無ければ警告して木を出さない。
        self.check_tree_column();
        // GA-1: ガントの設定の列が表に無ければ警告して設定を外す。
        self.check_gantt_column();
        // KB-1: カンバンの設定の列が表に無いかリストなら警告して設定を外す。
        self.check_kanban_column();
        // CH-1: 印のグラフの列が表に無ければ警告する(そのグラフは出さない)。
        self.check_chart_columns();
        // navigation.parent-child-config: 印の children の列が表に無ければ警告する(key なら親子にしない)。
        self.check_children_columns();
        // base-view.table-sort-bad: 印の表の既定の並べ替えの列が表に無ければ警告する。
        self.check_sort_columns();
        let off = match self.screen() {
            Screen::Tree => !self.tree_ready(),
            Screen::Gantt => self.gantt_spec().is_none(),
            Screen::Kanban => self.kanban_spec().is_none(),
            Screen::Chart => !self.chart_ready(),
            Screen::Table | Screen::Relations => false,
        };
        if off {
            self.close_screen();
        }
    }

    /// その型の画面を見ているか(画面の型のメニュー・パレット・切り替えの一覧をその型から開いている間も)。
    pub(crate) fn on_screen(&self, s: Screen) -> bool {
        self.mode == s.mode()
            || (matches!(self.mode, Mode::ScreenMenu | Mode::Palette | Mode::Switch)
                && self.screen() == s)
    }
}
