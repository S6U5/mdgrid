//! 印の `[[table]]` の `children`(navigation.parent-child-config。`impl App` の続き)。書いた表は、どのビューも
//! `key` の列で親子に並べ(ビューの設定でオフにしたビューを除く)、親を完了の値に直したときに、まだ終わっていない
//! 子孫を `on_done` のとおりに扱う(cell-edit.children-done)。ガントで子孫の棒が親の期間からはみ出したときの扱い
//! (gantt.children-span-*)は gantt.rs。
//!
//! 子孫は、ビューの絞り込み・グループ分けによらず、表の全部の行の `key` の値(リストなら先頭)のリンクでたどる
//! (絞り込みで見えない子も終わっていない子に数え、完了にする)。親の完了で子孫を扱うのは印に `children` を書いた
//! 表だけで、ビューの設定で親子をオンにしただけの表では扱わない(tbd~cell-edit.children-done-scope の仮決め)。
//! `--apply` と `--new-note --set` は扱わない(画面の直しだけ。tbd~cell-edit.children-done-cli の仮決め)。
//!
//! `closed_by` を書いた表では、親の完了で一緒に完了にした子孫に、その列へ親へのリンクを同じ1手でためる
//! (cell-edit.children-closed-by)。完了の親を終わっていない値に戻したときは、`on_reopen` のとおりに、`closed_by` が
//! その親を指す子孫を親と同じ値に戻して印を空にする(cell-edit.children-reopen)。`"ask"` は何もためずに
//! 「子も戻す」「親だけ」を選ぶ窓を開き、選んだ答えで同じ直しをためる。

use super::app::App;
use super::input::new_value_text;
use super::native_io::Ask;
use mdgrid::changes::Skip;
use mdgrid::i18n::Msg;
use mdgrid::relations::Form;
use mdgrid::settings::Settings;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::tree::{descendants, ChildrenSpec, OnDone, OnReopen};
use mdgrid::types::Kind;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// 状態の列を書いていないときの既定(印の kanban の column も無いとき)。
const DEFAULT_STATUS: &str = "status";

/// `stage_children` の結果。
pub(crate) struct Staged {
    /// 渡した行のうち、直せなかった行と理由(読むだけのセルなど)。
    pub skips: Vec<Skip>,
    /// 終わっていない子孫があるので直さなかった親と、その理由(on_done = "refuse")。
    pub refused: Vec<(RowId, String)>,
    /// 一緒に完了にした子孫(on_done = "complete")と、一緒に戻した子孫(on_reopen)。
    pub extra: Vec<RowId>,
    /// 下の帯に出す知らせ(on_done = "keep" の残りの数、完了にできなかった・戻せなかった子孫)。
    pub notice: Option<String>,
    /// on_reopen = "ask" で、子も戻すかを選ぶ窓を出すことにした(まだ何もためていない。`App::children_ask` に
    /// 覚えたので、呼んだ側は直しを終えたあとに `open_children_ask` で窓を開く)。
    pub asked: bool,
}

/// cell-edit.children-reopen: on_reopen = "ask" で、選ぶのを待っている直し(列と、渡された行と値)。
pub(crate) struct ReopenAsk {
    pub col: String,
    pub items: Vec<(RowId, Option<NewValue>)>,
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

    /// relations.link-input-no-cycle: 印の children の表で、親を指す列(`key`)の候補から外すノート。直す行の
    /// それぞれと、その子孫(選ぶと親子が輪になるもの)。ほかの列と、children を書いていない表では空。
    pub(crate) fn not_parents(&self, col: &str, rows: &[RowId]) -> HashSet<PathBuf> {
        let Some((spec, _)) = &self.children_spec else {
            return HashSet::new();
        };
        if spec.key != col || rows.is_empty() {
            return HashSet::new();
        }
        let kids = self.kids_of(&spec.key);
        rows.iter()
            .flat_map(|r| std::iter::once(r.clone()).chain(descendants(r, &kids)))
            .map(|r| PathBuf::from(r.0))
            .collect()
    }

    /// 行の `col` を値として直せるセルか(読むだけのセルとリストのセルには、完了の値を当てない。Changes の決まり)。
    fn takes_value(&self, r: &RowId, col: &str) -> bool {
        let cell = self.src.get(r, col);
        cell.lock.is_none()
            && !matches!(cell.value, Some(Value::List(_)))
            && self.col_type(col) != Kind::List
    }

    /// `col` を `items` の値に1手でためる(列の `then` も一緒に。cell-edit.then)。印に children を書いた表で、
    /// 状態の列を完了の値に直す親があれば、まだ終わっていない子孫を `on_done` のとおりに扱い
    /// (cell-edit.children-done・-complete・-refuse・-keep。`closed_by` があれば閉じた印も。-closed-by)、完了の親を
    /// 戻すなら、親の完了で閉じた子孫を `on_reopen` のとおりに扱う(cell-edit.children-reopen)。`"ask"` で戻す子孫が
    /// あれば何もためずに `asked` を立てる。
    pub(crate) fn stage_children(
        &mut self,
        col: &str,
        items: Vec<(RowId, Option<NewValue>)>,
    ) -> Staged {
        self.stage_children_with(col, items, None)
    }

    /// `stage_children` の中身。`reopen` は on_reopen = "ask" の窓で選んだ答え(子も戻すなら true。まだ聞いて
    /// いなければ None)。
    fn stage_children_with(
        &mut self,
        col: &str,
        items: Vec<(RowId, Option<NewValue>)>,
        reopen: Option<bool>,
    ) -> Staged {
        let mut out = Staged {
            skips: Vec::new(),
            refused: Vec::new(),
            extra: Vec::new(),
            notice: None,
            asked: false,
        };
        let mut items = items;
        let given: HashSet<RowId> = items.iter().map(|(r, _)| r.clone()).collect();
        let mut open_left: HashSet<RowId> = HashSet::new();
        // 同じ1手でためる閉じた印のセルと、一緒に戻す子孫。
        let mut also: Vec<(RowId, String, NewValue)> = Vec::new();
        let mut back: HashSet<RowId> = HashSet::new();
        if let Some(spec) = self
            .children_spec
            .as_ref()
            .map(|(s, _)| s.clone())
            .filter(|s| self.children_status(s) == col)
        {
            let closed_by = self.children_reopen.closed_by.clone();
            if let Some(cb) = &closed_by {
                let kids = self.plan_reopen(&spec, col, &items, cb);
                let go = match (kids.is_empty(), reopen, self.children_reopen.on_reopen) {
                    (true, _, _) | (false, None, OnReopen::Keep) => false,
                    (false, Some(b), _) => b,
                    (false, None, OnReopen::Reopen) => true,
                    (false, None, OnReopen::Ask) => {
                        self.children_ask = Some(ReopenAsk {
                            col: col.to_string(),
                            items,
                        });
                        out.asked = true;
                        return out;
                    }
                };
                if go {
                    for (d, v) in kids {
                        items.push((d.clone(), Some(v)));
                        also.push((d.clone(), cb.clone(), NewValue::Null));
                        out.extra.push(d.clone());
                        back.insert(d);
                    }
                }
            }
            let closed = self.plan_children(&spec, col, &mut items, &mut out, &mut open_left);
            if let Some(cb) = &closed_by {
                for (d, p) in closed {
                    let link = self.link_value(&d, Form::Wiki, Path::new(&p.0));
                    also.push((d, cb.clone(), NewValue::Str(link)));
                }
            }
        }
        let follow = self.follow(col);
        let skips = self
            .changes
            .set_each_then_with(self.src.as_ref(), col, &items, &follow, &also);
        // 一緒に完了にできなかった子孫は、終わっていない子に数えて知らせる(親は完了にしたまま)。一緒に戻せなかった
        // 子孫は、戻せなかったと知らせる(親は戻したまま)。
        let mut reasons = Vec::new();
        let mut back_reasons = Vec::new();
        for s in skips {
            if given.contains(&s.row) {
                out.skips.push(s);
                continue;
            }
            out.extra.retain(|r| *r != s.row);
            if back.contains(&s.row) {
                if !back_reasons.contains(&s.reason) {
                    back_reasons.push(s.reason);
                }
            } else {
                open_left.insert(s.row);
                if !reasons.contains(&s.reason) {
                    reasons.push(s.reason);
                }
            }
        }
        let mut notes = Vec::new();
        if !open_left.is_empty() {
            let mut n = Msg::ChildrenOpenKeep.fill(&[&open_left.len()]);
            if !reasons.is_empty() {
                n.push_str(&format!(" ({})", reasons.join(" / ")));
            }
            notes.push(n);
        }
        if !back_reasons.is_empty() {
            notes.push(format!(
                "{} ({})",
                Msg::ChildrenReopenSkipped.text(),
                back_reasons.join(" / ")
            ));
        }
        if !notes.is_empty() {
            out.notice = Some(notes.join(" / "));
        }
        out
    }

    /// cell-edit.children-reopen: 状態の列を、終わったとみなす値から終わっていない値に直す親ごとに、`closed_by`
    /// (`cb`)がその親を指す子孫と、戻す値(親と同じ値)。同じ1手で値を渡された行は除く。
    fn plan_reopen(
        &self,
        spec: &ChildrenSpec,
        col: &str,
        items: &[(RowId, Option<NewValue>)],
        cb: &str,
    ) -> Vec<(RowId, NewValue)> {
        let text = |v: &NewValue| new_value_text(v).trim().to_string();
        let parents: Vec<(RowId, NewValue)> = items
            .iter()
            .filter_map(|(r, v)| {
                let v = v.as_ref()?;
                (spec.is_closed(&self.plain(r, col))
                    && !spec.is_closed(&text(v))
                    && self.takes_value(r, col))
                .then(|| (r.clone(), v.clone()))
            })
            .collect();
        if parents.is_empty() {
            return Vec::new();
        }
        let given: HashSet<&RowId> = items.iter().map(|(r, _)| r).collect();
        let kids = self.kids_of(&spec.key);
        let mut seen: HashSet<RowId> = HashSet::new();
        let mut out = Vec::new();
        for (p, v) in &parents {
            for d in descendants(p, &kids) {
                if given.contains(&d) || seen.contains(&d) {
                    continue;
                }
                // 閉じた印の値は、親を指す列と同じ解き方(リストなら先頭のリンク)で見る。
                if self.tree_parent(&d, cb).as_ref() == Some(p) {
                    seen.insert(d.clone());
                    out.push((d, v.clone()));
                }
            }
        }
        out
    }

    /// cell-edit.children-reopen: 選ぶのを待っている直しがあれば、「子も戻す」「親だけ」を選ぶ窓を開く。開いたら true。
    pub(crate) fn open_children_ask(&mut self) -> bool {
        if self.children_ask.is_none() || self.nv.ask.is_some() {
            return false;
        }
        self.open_ask(Ask::ChildrenReopen {
            items: vec![
                Msg::ChildrenReopenAll.text().to_string(),
                Msg::ChildrenReopenParent.text().to_string(),
            ],
        });
        true
    }

    /// cell-edit.children-reopen: 窓で選んだ答えで直す(`all` なら子も戻す)。表の直しと同じく、直した行を留め、
    /// 直せなかった理由か知らせを出す。
    pub(crate) fn children_reopen_chosen(&mut self, all: bool) {
        let Some(p) = self.children_ask.take() else {
            return self.close_palette();
        };
        self.close_palette();
        let rows: Vec<RowId> = p.items.iter().map(|(r, _)| r.clone()).collect();
        let st = self.stage_children_with(&p.col, p.items, Some(all));
        let not: HashSet<RowId> = st
            .refused
            .iter()
            .map(|(r, _)| r.clone())
            .chain(st.skips.iter().map(|s| s.row.clone()))
            .collect();
        self.stay
            .extend(rows.into_iter().filter(|r| !not.contains(r)));
        self.stay.extend(st.extra);
        self.regrid = true;
        let why = st
            .refused
            .into_iter()
            .map(|(_, w)| w)
            .chain(
                st.skips
                    .into_iter()
                    .map(|s| Msg::CannotCommit.fill(&[&s.reason])),
            )
            .next();
        self.message = why.or(st.notice);
    }

    /// `stage_children` の子孫の扱い: 完了の値に直す親ごとに、まだ終わっていない子孫を数え、`complete` なら足し、
    /// `refuse` なら親を外し、`keep` なら数を覚える。同じ1手で直す値(子を一緒に done にするなど)は直したあとの値で
    /// 見る。断った親は直さないので、ほかの親から見て終わっていない子に数え直す。返すのは、一緒に完了にした子孫と、
    /// その完了のもとの親(cell-edit.children-closed-by の印に書く)。
    fn plan_children(
        &self,
        spec: &ChildrenSpec,
        col: &str,
        items: &mut Vec<(RowId, Option<NewValue>)>,
        out: &mut Staged,
        open_left: &mut HashSet<RowId>,
    ) -> Vec<(RowId, RowId)> {
        let text = |v: &NewValue| new_value_text(v).trim().to_string();
        let parents: Vec<(RowId, NewValue)> = items
            .iter()
            .filter_map(|(r, v)| {
                let v = v.as_ref()?;
                (text(v) == spec.done && self.takes_value(r, col)).then(|| (r.clone(), v.clone()))
            })
            .collect();
        let mut closed_with = Vec::new();
        if parents.is_empty() {
            return closed_with;
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
                        out.extra.push(d.clone());
                        closed_with.push((d, p.clone()));
                    }
                }
            }
        }
        closed_with
    }
}
