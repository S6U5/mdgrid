//! 印の `[[table]]` の `children`(navigation.parent-child-config。`impl App` の続き)。書いた表は、どのビューも
//! `key` の列で親子に並べ(ビューの設定でオフにしたビューを除く)、親を完了の値に直したときに、まだ終わっていない
//! 子孫を `on_done` のとおりに扱う(cell-edit.children-done)。ガントで子孫の棒が親の期間からはみ出したときの扱い
//! (gantt.children-span-*)は gantt.rs。
//!
//! 子孫は、ビューの絞り込み・グループ分けによらず、表の全部の行の `key` の値(リストなら先頭)のリンクでたどる
//! (絞り込みで見えない子も終わっていない子に数え、完了にする)。親の完了で子孫を扱うのは印に `children` を書いた
//! 表だけで、ビューの設定で親子をオンにしただけの表では扱わない(tbd~cell-edit.children-done-scope の仮決め)。
//! `--apply` と `--new-note --set` は扱わない(画面の直しだけ。tbd~cell-edit.children-done-cli の仮決め)。

use super::app::App;
use super::input::new_value_text;
use mdgrid::changes::Skip;
use mdgrid::i18n::Msg;
use mdgrid::settings::Settings;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::tree::{descendants, ChildrenSpec, OnDone};
use mdgrid::types::Kind;
use std::collections::{HashMap, HashSet};

/// 状態の列を書いていないときの既定(印の kanban の column も無いとき)。
const DEFAULT_STATUS: &str = "status";

/// `stage_children` の結果。
pub(crate) struct Staged {
    /// 渡した行のうち、直せなかった行と理由(読むだけのセルなど)。
    pub skips: Vec<Skip>,
    /// 終わっていない子孫があるので直さなかった親と、その理由(on_done = "refuse")。
    pub refused: Vec<(RowId, String)>,
    /// 一緒に完了にした子孫(on_done = "complete")。
    pub extra: Vec<RowId>,
    /// 下の帯に出す知らせ(on_done = "keep" の残りの数、完了にできなかった子孫)。
    pub notice: Option<String>,
}

impl App {
    /// 行の親(`key` の列の値。リストなら先頭のリンクの行き先のノート。REL-1 と同じ解き方)。NV-27 の並びと同じ。
    pub(crate) fn tree_parent(&self, r: &RowId, key: &str) -> Option<RowId> {
        let v = self.prop(r, key)?;
        let s = match &v {
            Value::Str(s) => s.clone(),
            Value::List(items) => items.iter().find_map(|x| match x {
                Value::Str(s) => Some(s.clone()),
                _ => None,
            })?,
            _ => return None,
        };
        let (_, target) = self.link_target(r, &s)?;
        Some(RowId(target?.to_string_lossy().into_owned()))
    }

    /// 印の children の `key`。列が出そろって表に無ければ None(親子にしない)。読み込みの途中は列を待たない。
    pub(crate) fn children_key(&self) -> Option<String> {
        let (spec, _) = self.children_spec.as_ref()?;
        (!self.columns_settled() || self.has_column(&spec.key)).then(|| spec.key.clone())
    }

    /// NV-27・navigation.parent-child-config: 今のビューで親子に並べるキー。ビューの設定で選んだキー、無ければ
    /// (ビューの設定でオフにしていなければ)印の children の `key`。
    pub(crate) fn tree_key(&self) -> Option<String> {
        if let Some(k) = &self.settings.tree {
            return Some(k.clone());
        }
        if self.settings.tree_off {
            return None;
        }
        self.children_key()
    }

    /// ビューの設定の画面の写しの元: 今のビューの設定で、親子は今の並びのとおり(印の children に従う分も `tree`
    /// に入れる。区画の「親子」をオンと見せ、キーを選び直せるように)。
    pub(crate) fn draft_settings(&self) -> Settings {
        Settings {
            tree: self.tree_key(),
            ..self.settings.clone()
        }
    }

    /// ビューの設定の写し(親子は今の並びのとおりに `tree` を持つ)を、App に当てて残す形にする(反映のとき)。
    /// 印の children の `key` で並べるなら書かずに印に従い(`tree` は None)、オフなら印があるときだけ「オフにした」を
    /// 残す(印を書き換えても、オフにしたビューはオフのまま。navigation.parent-child-save)。
    pub(crate) fn settle_tree(&self, s: &mut Settings) {
        let Some((spec, _)) = &self.children_spec else {
            s.tree_off = false;
            return;
        };
        match &s.tree {
            Some(k) if *k == spec.key => {
                s.tree = None;
                s.tree_off = false;
            }
            Some(_) => s.tree_off = false,
            None => s.tree_off = true,
        }
    }

    /// navigation.parent-child-config: 列が出そろったときに、印の children の列が表に無ければ CLI-3 の形で1回警告する。
    /// `key` の列が無ければ親子にしない(`children_key`)。書いた `status` の列が無ければ子孫を扱えない。
    pub(crate) fn check_children_columns(&mut self) {
        let Some((spec, at)) = self.children_spec.clone() else {
            return;
        };
        let file = format!(
            "{}/{}",
            mdgrid::workspace::MARKER_DIR,
            mdgrid::workspace::MARKER_FILE
        );
        let mut warns = Vec::new();
        if !self.has_column(&spec.key) {
            warns.push(Msg::SettingWarn.fill(&[
                &file,
                &format!("{at}.key"),
                &Msg::RsnChildrenNoColumn.fill(&[&spec.key]),
            ]));
        }
        if let Some(s) = spec.status.as_deref().filter(|s| !self.has_column(s)) {
            warns.push(Msg::SettingWarn.fill(&[
                &file,
                &format!("{at}.status"),
                &Msg::RsnChildrenNoStatus.fill(&[&s]),
            ]));
        }
        for w in warns {
            self.message = Some(match self.message.take() {
                Some(m) => format!("{m} / {w}"),
                None => w,
            });
        }
    }

    /// cell-edit.children-done: 状態の列(children の `status`、書かなければ印の kanban の `column`、それも無ければ
    /// `status`)。
    fn children_status(&self, spec: &ChildrenSpec) -> String {
        spec.status
            .clone()
            .or_else(|| self.kanban_spec.as_ref().map(|(k, _)| k.column.clone()))
            .unwrap_or_else(|| DEFAULT_STATUS.to_string())
    }

    /// 表の全部の行の、親 → 子の並び(`key` の列で解いた親。自分を指す行は入れない)。
    pub(crate) fn kids_of(&self, key: &str) -> HashMap<RowId, Vec<RowId>> {
        let mut kids: HashMap<RowId, Vec<RowId>> = HashMap::new();
        for r in self.src.rows() {
            if let Some(p) = self.tree_parent(&r, key).filter(|p| *p != r) {
                kids.entry(p).or_default().push(r);
            }
        }
        kids
    }

    /// 行の `col` を値として直せるセルか(読むだけのセルとリストのセルには、完了の値を当てない。Changes の決まり)。
    fn takes_value(&self, r: &RowId, col: &str) -> bool {
        let cell = self.src.get(r, col);
        cell.lock.is_none()
            && !matches!(cell.value, Some(Value::List(_)))
            && self.col_type(col) != Kind::List
    }

    /// `col` を `items` の値に1手でためる(列の `then` も一緒に。cell-edit.then)。印に children を書いた表で、
    /// 状態の列を完了の値に直す親があれば、まだ終わっていない子孫を `on_done` のとおりに扱う
    /// (cell-edit.children-done・-complete・-refuse・-keep)。
    pub(crate) fn stage_children(
        &mut self,
        col: &str,
        items: Vec<(RowId, Option<NewValue>)>,
    ) -> Staged {
        let mut out = Staged {
            skips: Vec::new(),
            refused: Vec::new(),
            extra: Vec::new(),
            notice: None,
        };
        let mut items = items;
        let given: HashSet<RowId> = items.iter().map(|(r, _)| r.clone()).collect();
        let mut open_left: HashSet<RowId> = HashSet::new();
        if let Some(spec) = self
            .children_spec
            .as_ref()
            .map(|(s, _)| s.clone())
            .filter(|s| self.children_status(s) == col)
        {
            self.plan_children(&spec, col, &mut items, &mut out, &mut open_left);
        }
        let follow = self.follow(col);
        let skips = self
            .changes
            .set_each_then(self.src.as_ref(), col, &items, &follow);
        // 一緒に完了にできなかった子孫は、終わっていない子に数えて知らせる(親は完了にしたまま)。
        let mut reasons = Vec::new();
        for s in skips {
            if given.contains(&s.row) {
                out.skips.push(s);
            } else {
                out.extra.retain(|r| *r != s.row);
                open_left.insert(s.row);
                if !reasons.contains(&s.reason) {
                    reasons.push(s.reason);
                }
            }
        }
        if !open_left.is_empty() {
            let mut n = Msg::ChildrenOpenKeep.fill(&[&open_left.len()]);
            if !reasons.is_empty() {
                n.push_str(&format!(" ({})", reasons.join(" / ")));
            }
            out.notice = Some(n);
        }
        out
    }

    /// `stage_children` の子孫の扱い: 完了の値に直す親ごとに、まだ終わっていない子孫を数え、`complete` なら足し、
    /// `refuse` なら親を外し、`keep` なら数を覚える。同じ1手で直す値(子を一緒に done にするなど)は直したあとの値で
    /// 見る。断った親は直さないので、ほかの親から見て終わっていない子に数え直す。
    fn plan_children(
        &self,
        spec: &ChildrenSpec,
        col: &str,
        items: &mut Vec<(RowId, Option<NewValue>)>,
        out: &mut Staged,
        open_left: &mut HashSet<RowId>,
    ) {
        let text = |v: &NewValue| new_value_text(v).trim().to_string();
        let parents: Vec<(RowId, NewValue)> = items
            .iter()
            .filter_map(|(r, v)| {
                let v = v.as_ref()?;
                (text(v) == spec.done && self.takes_value(r, col)).then(|| (r.clone(), v.clone()))
            })
            .collect();
        if parents.is_empty() {
            return;
        }
        let kids = self.kids_of(&spec.key);
        // 同じ1手の値(重ねて渡した行は最初の値。Changes と同じ)。
        let mut batch: HashMap<RowId, Option<NewValue>> = HashMap::new();
        for (r, v) in items.iter() {
            batch.entry(r.clone()).or_insert_with(|| v.clone());
        }
        let mut refused: HashSet<RowId> = HashSet::new();
        let closed = |r: &RowId, refused: &HashSet<RowId>| -> bool {
            let now = match batch.get(r) {
                Some(Some(v)) if !refused.contains(r) => text(v),
                // 外す値(元に戻す)は元の値。
                Some(None) => self
                    .src
                    .get(r, col)
                    .value
                    .as_ref()
                    .map(mdgrid::print::value_plain)
                    .unwrap_or_default(),
                _ => self.plain(r, col),
            };
            spec.is_closed(&now)
        };
        let open = |p: &RowId, refused: &HashSet<RowId>| -> Vec<RowId> {
            descendants(p, &kids)
                .into_iter()
                .filter(|d| !closed(d, refused))
                .collect()
        };
        match spec.on_done {
            OnDone::Refuse => {
                // 断った親は直さないので、ほかの親の子孫の数え直しが要る(変わらなくなるまで)。
                loop {
                    let mut changed = false;
                    for (p, _) in &parents {
                        if refused.contains(p) {
                            continue;
                        }
                        let n = open(p, &refused).len();
                        if n > 0 {
                            refused.insert(p.clone());
                            out.refused
                                .push((p.clone(), Msg::ChildrenOpenRefuse.fill(&[&n])));
                            changed = true;
                        }
                    }
                    if !changed {
                        break;
                    }
                }
                items.retain(|(r, _)| !refused.contains(r));
            }
            OnDone::Keep => {
                for (p, _) in &parents {
                    open_left.extend(open(p, &refused));
                }
            }
            OnDone::Complete => {
                let mut added: HashSet<RowId> = HashSet::new();
                for (p, v) in &parents {
                    for d in open(p, &refused) {
                        if batch.contains_key(&d) || !added.insert(d.clone()) {
                            continue;
                        }
                        items.push((d.clone(), Some(v.clone())));
                        out.extra.push(d);
                    }
                }
            }
        }
    }
}
