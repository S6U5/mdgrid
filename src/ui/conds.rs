//! 名前を付けた条件の画面の側(`impl App` の続き。condition.digit-keys〜condition.hide-toggle)と、
//! 画面から足す式の絞り込み(navigation.expr-filter)。
//!
//! 行の判定は表の組み立ての重ね(grid.rs の `overlay`)で当てる: いつも隠す条件で隠し、そのあと
//! 数字のキーの条件・式の絞り込みで絞る(検索や同じ値の絞り込みと同じ段)。ガント・カンバン・グラフ・木は
//! その行を使うので、同じく効く(condition.hide-everywhere)。表を移ると App ごと作り直すので、
//! 数字のキーの条件と式の絞り込みは解ける(condition.digit-lifetime・navigation.expr-filter-lifetime)。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::native_views::Ask;
use mdgrid::base;
use mdgrid::condition::{Condition, Conditions};
use mdgrid::expr::{self, Expr};
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// condition.columns-present: 列の有無を覚えた時の表の形(読み込み・外の変更・ためた変更の数)。
type PresentKey = (u64, usize, usize, usize);

/// 条件の画面の状態。
#[derive(Default)]
pub(crate) struct CondState {
    /// 設定の名前の条件(印の条件を重ねたもの。condition.marker-layer)。
    pub all: Arc<Conditions>,
    /// 全体の設定の条件(ほかの表を数えるとき、その表の印と重ね直す。notice.columns-present・notify.columns-present)。
    pub global: Arc<Conditions>,
    /// condition.digit-remap: 別の動作に割り当て直した数字(条件には振らない。condition.digit-order)。
    pub remapped: Vec<u8>,
    /// condition.digit-remap: 書いた数字を割り当て直したので、数字のキーに振らない条件。
    pub no_digit: HashSet<String>,
    /// condition.columns-present: 今の表の行に列があるか(列ごと。表の形が変わったら数え直す)。
    present: RefCell<Option<(PresentKey, HashMap<String, bool>)>>,
    /// 数字のキーで入れた条件(入れた順)。
    pub on: Vec<String>,
    /// いつも隠す条件のうち、数字のキーで止めたもの(condition.hide-toggle)。
    pub shown: HashSet<String>,
    /// condition.show-hidden-cli: `--pick --show-hidden` で、いつも隠す条件を当てない。
    pub show_hidden: bool,
    /// 画面で足した式の絞り込み(打った式と読んだ式)。
    pub expr: Option<(String, Expr)>,
    /// いつも隠す条件ごとの、最後の組み立てで隠した行の数(当たる行のある条件だけ。書いた順)。
    pub hidden: Vec<(String, usize)>,
}

/// 設定の帯の項目のうち、名前の条件と式の絞り込みのもの(ビューの設定の項目のあとに並べる)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CondChip {
    /// 数字のキーで入れた条件。
    On(String),
    /// いつも隠す条件(隠した行のあるもの)。
    Hide(String),
    /// いつも隠す条件を止めたもの。
    Shown(String),
    /// 画面で足した式の絞り込み。
    Expr,
}

impl App {
    /// 設定の名前の条件を当てる(App::configure から)。数字を割り当て直したキーの条件を警告にして返す。
    /// condition.digit-order: 割り当て直した数字は条件に振らずに飛ばし、その数字を書いた条件は数字のキーに入れない。
    pub(crate) fn configure_conditions(&mut self, all: Arc<Conditions>) -> Vec<String> {
        let mut warnings = Vec::new();
        self.cond.remapped = (1..=9u8)
            .filter(|d| {
                keymap::lookup(&self.keys, Mode::Table, &d.to_string()).and_then(Action::cond_digit)
                    != Some(*d)
            })
            .collect();
        self.cond.no_digit.clear();
        for c in all.list() {
            let Some(d) = c.key else {
                continue;
            };
            if self.cond.remapped.contains(&d) {
                let file = if all.from_marker(&c.name) {
                    format!(
                        "{}/{}",
                        mdgrid::workspace::MARKER_DIR,
                        mdgrid::workspace::MARKER_FILE
                    )
                } else {
                    mdgrid::config::CONFIG_FILE.to_string()
                };
                mdgrid::profile::warn(
                    &mut warnings,
                    &file,
                    &format!("conditions.{}.key", c.name),
                    Msg::RsnCondKeyTaken.fill(&[&d]),
                );
                self.cond.no_digit.insert(c.name.clone());
            }
        }
        self.cond.all = all;
        *self.cond.present.borrow_mut() = None;
        warnings
    }

    /// condition.columns-present: 今の表の行(`.base` の最上位の絞り込みのあと)に列 `col` があるか。ノートのキーは
    /// 値を持つ(空の値も含む)行が1つでもあれば、`formula.x` は表の式に x があれば、ある。表の形が変わるまで覚える。
    pub(crate) fn has_table_column(&self, col: &str) -> bool {
        let key: PresentKey = (
            self.data_gen,
            self.progress.loaded,
            usize::from(self.progress.done),
            self.changes.count(),
        );
        if let Some((k, m)) = self.cond.present.borrow().as_ref() {
            if *k == key {
                if let Some(b) = m.get(col) {
                    return *b;
                }
            }
        }
        let found = if let Some(f) = col.strip_prefix("formula.") {
            self.native_synth()
                .or_else(|| self.base.as_ref().map(|b| &b.base))
                .is_some_and(|b| b.formula_names().iter().any(|n| n == f))
        } else {
            let prop = |r: &RowId, k: &str| self.prop(r, k);
            self.src.columns().iter().any(|c| c == col)
                && self.src.rows().iter().any(|r| {
                    self.prop(r, col).is_some()
                        && self.base.as_ref().is_none_or(|b| {
                            b.base
                                .keeps(self.src.as_ref(), &prop, r, self.today, self.now)
                        })
                })
        };
        let mut cell = self.cond.present.borrow_mut();
        match cell.as_mut() {
            Some((k, m)) if *k == key => {
                m.insert(col.to_string(), found);
            }
            _ => *cell = Some((key, HashMap::from([(col.to_string(), found)]))),
        }
        found
    }

    /// condition.columns-present: 名前 `name` の条件が今の表で使えるか(式が使う列が全部ある)。無い名前は使えない。
    pub(crate) fn cond_usable(&self, name: &str) -> bool {
        self.cond
            .all
            .expr(name)
            .is_some_and(|e| mdgrid::condition::usable(e, &|c| self.has_table_column(c)))
    }

    /// condition.digit-order: 今の表で数字のキーに振った条件(数字, 名前)。印の条件が先、全体の設定の条件が後で、
    /// 今の表で使えない条件と書いた数字を割り当て直した条件は飛ばし、割り当て直した数字も飛ばして 1 から詰める。
    pub(crate) fn digits(&self) -> Vec<(u8, String)> {
        let mut out = Vec::new();
        let mut next = 1u8;
        for c in self.cond.all.digit_order() {
            if self.cond.no_digit.contains(&c.name) || !self.cond_usable(&c.name) {
                continue;
            }
            while self.cond.remapped.contains(&next) {
                next += 1;
            }
            if next > 9 {
                break;
            }
            out.push((next, c.name.clone()));
            next += 1;
        }
        out
    }

    /// condition.digit-order: 数字 `d` に振った条件。
    pub(crate) fn digit_cond(&self, d: u8) -> Option<&Condition> {
        let name = self
            .digits()
            .into_iter()
            .find(|(x, _)| *x == d)
            .map(|(_, n)| n)?;
        self.cond.all.get(&name)
    }

    /// condition.digit-order: 名前 `name` の条件に振った数字。
    pub(crate) fn digit_of(&self, name: &str) -> Option<u8> {
        self.digits()
            .into_iter()
            .find(|(_, n)| n == name)
            .map(|(d, _)| d)
    }

    /// condition.show-hidden-cli: いつも隠す名前の条件を当てない(`--pick --show-hidden`)。
    pub fn show_hidden_conditions(&mut self) {
        self.cond.show_hidden = true;
        self.refresh();
    }

    /// 行で式を評価して、行を残す値(`true`)か。ためた値を重ね、ビューの formula を引ける。
    pub(crate) fn eval_true(&self, row: &RowId, e: &Expr) -> bool {
        let prop = |r: &RowId, k: &str| self.prop(r, k);
        let base = self
            .native_synth()
            .or_else(|| self.base.as_ref().map(|b| &b.base));
        let v = match base {
            Some(b) => b.eval_row(self.src.as_ref(), &prop, row, e, self.today, self.now),
            None => base::eval_plain(self.src.as_ref(), &prop, row, e, self.today, self.now),
        };
        expr::passes(&v)
    }

    /// 名前 `name` の条件に行が当たるか(無い名前は当たらない)。
    pub(crate) fn hits(&self, row: &RowId, name: &str) -> bool {
        self.cond
            .all
            .expr(name)
            .is_some_and(|e| self.eval_true(row, e))
    }

    /// 今いつも隠している条件(止めたものと `--show-hidden` を除く)。使う列の無い表でも当てる
    /// (tbd~condition.columns-hide。数字のキーと違い、condition.columns-present を見ない)。
    pub(crate) fn hiding(&self) -> Vec<&Condition> {
        if self.cond.show_hidden {
            return Vec::new();
        }
        self.cond
            .all
            .list()
            .iter()
            .filter(|c| c.hide && !self.cond.shown.contains(&c.name))
            .collect()
    }

    /// condition.hide-always: いつも隠す条件に当たる行を除き、条件ごとに隠した数を覚える。
    pub(crate) fn drop_hidden(&mut self, segs: &mut [(String, Vec<RowId>)]) {
        let hiding: Vec<String> = self.hiding().iter().map(|c| c.name.clone()).collect();
        let mut counts: Vec<(String, usize)> = hiding.iter().map(|n| (n.clone(), 0)).collect();
        if !hiding.is_empty() {
            for (_, rs) in segs.iter_mut() {
                rs.retain(|r| {
                    let mut keep = true;
                    for (k, n) in hiding.iter().enumerate() {
                        if self.hits(r, n) {
                            counts[k].1 += 1;
                            keep = false;
                        }
                    }
                    keep
                });
            }
        }
        counts.retain(|(_, n)| *n > 0);
        self.cond.hidden = counts;
    }

    /// 数字のキーの条件・式の絞り込み・ビューの設定の名前の条件が効いているか(grid.rs の絞り込みの要否)。
    pub(crate) fn cond_narrowing(&self) -> bool {
        !self.cond.on.is_empty() || self.cond.expr.is_some() || !self.settings.named.is_empty()
    }

    /// 行が、数字のキーの条件の全部・式の絞り込み・ビューの設定の名前の条件を満たすか(nav.rs の `passes`)。
    pub(crate) fn cond_passes(&self, row: &RowId) -> bool {
        self.cond.on.iter().all(|n| self.hits(row, n))
            && self
                .cond
                .expr
                .as_ref()
                .is_none_or(|(_, e)| self.eval_true(row, e))
            && self
                .settings
                .named
                .iter()
                .all(|n| self.hits(row, &n.name) != n.hide)
    }

    /// condition.digit-keys・condition.hide-toggle: 数字 `d` の条件を入れ切りする。条件の無い数字は何もしない。
    pub(crate) fn toggle_digit(&mut self, d: u8) {
        let Some(c) = self.digit_cond(d) else {
            return;
        };
        let name = c.name.clone();
        if c.hide {
            if !self.cond.shown.remove(&name) {
                self.cond.shown.insert(name);
            }
        } else if let Some(i) = self.cond.on.iter().position(|n| *n == name) {
            self.cond.on.remove(i);
        } else {
            self.cond.on.push(name);
        }
        self.stay.clear();
        self.regrid = true;
    }

    /// 設定の帯の名前の条件と式の項目(文字, 何の項目か)。ビューの設定の項目のあとに並べる。
    pub(crate) fn cond_chips(&self) -> Vec<(String, CondChip)> {
        let mut out = Vec::new();
        for n in &self.cond.on {
            if let Some(c) = self.cond.all.get(n) {
                let d = self.digit_of(n).map(|d| d.to_string()).unwrap_or_default();
                out.push((
                    Msg::ChipCondOn.fill(&[&d, &c.shown()]),
                    CondChip::On(n.clone()),
                ));
            }
        }
        for (n, count) in &self.cond.hidden {
            if let Some(c) = self.cond.all.get(n) {
                out.push((
                    Msg::ChipCondHide.fill(&[&c.shown(), count]),
                    CondChip::Hide(n.clone()),
                ));
            }
        }
        for c in self.cond.all.list() {
            if c.hide && self.cond.shown.contains(&c.name) {
                out.push((
                    Msg::ChipCondShown.fill(&[&c.shown()]),
                    CondChip::Shown(c.name.clone()),
                ));
            }
        }
        if let Some((src, _)) = &self.cond.expr {
            out.push((Msg::ChipExpr.fill(&[src]), CondChip::Expr));
        }
        out
    }

    /// 設定の帯の項目の全部(ビューの設定・名前の条件・式。NV-16)。
    /// 列の名前は列の見出しと同じ displayName(base-view.display-name-everywhere)。
    pub(crate) fn bar_chips(&self) -> Vec<String> {
        let mut out = self.settings.chips_titled(&|c| self.title(c));
        out.extend(self.cond_chips().into_iter().map(|(t, _)| t));
        out
    }

    /// 設定の帯に出す項目が無いか(navigation.settings-bar-empty)。
    pub(crate) fn bar_empty(&self) -> bool {
        self.settings.no_conditions() && self.cond_chips().is_empty()
    }

    /// 設定の帯の名前の条件か式の項目を外す(`i` はビューの設定の項目のあとからの添字)。
    /// 外すのは、数字のキーの条件を解く・いつも隠す条件を止める・止めたのを戻す・式を解く。
    pub(crate) fn remove_cond_chip(&mut self, i: usize) {
        let Some((_, chip)) = self.cond_chips().into_iter().nth(i) else {
            return;
        };
        match chip {
            CondChip::On(n) => self.cond.on.retain(|x| *x != n),
            CondChip::Hide(n) => {
                self.cond.shown.insert(n);
            }
            CondChip::Shown(n) => {
                self.cond.shown.remove(&n);
            }
            CondChip::Expr => self.cond.expr = None,
        }
        self.stay.clear();
        self.regrid = true;
    }

    /// condition.digit-hints: ヘルプとパレットの数字のキーの名前(条件の見せる名前)。条件の無い数字は None。
    pub(crate) fn cond_label(&self, a: Action) -> Option<String> {
        let d = a.cond_digit()?;
        self.digit_cond(d).map(|c| c.shown().to_string())
    }

    /// navigation.expr-filter: パレットから式の入力を開く。
    pub(crate) fn open_expr_filter(&mut self) {
        self.open_ask(Ask::ExprFilter);
        if let (Some(p), Some((src, _))) = (&mut self.palette, &self.cond.expr) {
            p.query.clone_from(src);
        }
    }

    /// navigation.expr-filter: 絞り込む式を読む(読めない・この表で評価できない式は理由)。セッション(sessions.rs)も使う。
    pub(crate) fn filter_expr(&self, src: &str) -> Result<Expr, String> {
        let parsed = expr::parse(src).map_err(|e| match e {
            expr::ExprError::Syntax(m) => m,
            expr::ExprError::Unsupported(n) => Msg::CondUnsupported.fill(&[&n]),
        });
        parsed.and_then(|e| {
            if e.uses_this() && self.base.as_ref().is_none_or(|b| !b.base.has_this()) {
                Err(Msg::CondUnsupported.fill(&[&"this"]))
            } else {
                Ok(e)
            }
        })
    }

    /// navigation.expr-filter・navigation.expr-filter-error: 打った式で絞る。読めないか評価できない式は、
    /// 絞り込みを変えずに理由を出し、入力の欄に式を残す。空の式は式の絞り込みを解く。
    pub(crate) fn run_expr_filter(&mut self, query: &str) {
        let src = query.trim();
        if src.is_empty() {
            self.close_palette();
            if self.cond.expr.take().is_some() {
                self.stay.clear();
                self.refresh();
            }
            return;
        }
        match self.filter_expr(src) {
            Ok(e) => {
                self.close_palette();
                self.cond.expr = Some((src.to_string(), e));
                self.stay.clear();
                // パレットの動作は組み直しの合図(`regrid`)を見ずに戻るので、ここで組み直す。
                self.refresh();
            }
            Err(reason) => self.message = Some(Msg::ExprFilterBad.fill(&[&reason])),
        }
    }
}
