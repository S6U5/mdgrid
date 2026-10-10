//! ビューの設定の画面の選び手と値の入力(`impl App` の続き。NV-14・NV-19)。
//! フィルターは 列 → 種類 → 値の一覧(件数つき・「(空)」あり・残すか隠すか)か、値の入力(含む・含まない・
//! 比べる)。並べ替えとグループは列を選ぶ。画面の状態と区画の操作は settings.rs。

use super::app::App;
use super::keymap::{Action, Mode};
use super::settings::{Pick, Purpose, Sec, TextEntry, TextKind, CMPS};
use mdgrid::i18n::Msg;
use mdgrid::settings::{self as core, CmpOp, Cond, Dir, Group, Op};
use mdgrid::source::Value;
use mdgrid::types::{self, Kind};

impl App {
    /// 足したフィルターを直す: 値の一覧はチェックを付けたまま開き、値の条件は値を打ち直す。
    pub(crate) fn edit_cond(&mut self, i: usize) {
        let Some(c) = self
            .draft
            .as_ref()
            .and_then(|d| d.s.filters.get(i))
            .cloned()
        else {
            return;
        };
        match c.op {
            Op::Keep(k) => self.open_values(c.col, true, k, Some(i)),
            Op::Drop(k) => self.open_values(c.col, false, k, Some(i)),
            Op::Contains(s) => self.open_text(c.col, TextKind::Contains, s, Some(i)),
            Op::NotContains(s) => self.open_text(c.col, TextKind::NotContains, s, Some(i)),
            Op::Cmp(op, s) => self.open_text(c.col, TextKind::Cmp(op), s, Some(i)),
            Op::Empty | Op::NotEmpty => {
                let sel = if c.op == Op::Empty { 4 } else { 5 };
                if let Some(d) = self.draft.as_mut() {
                    d.pick = Some(Pick::Kind {
                        col: c.col,
                        sel,
                        edit: Some(i),
                    });
                }
            }
        }
    }

    /// 値の一覧を開く(NV-19): `.base` の結果の行の値ごとの件数(リストは要素ごと、空は「(空)」)。
    fn open_values(
        &mut self,
        col: String,
        keep: bool,
        checked: Vec<Option<String>>,
        edit: Option<usize>,
    ) {
        let rows = self.build_grid().map(|g| g.rows).unwrap_or_default();
        let vals: Vec<Option<Value>> = rows.iter().map(|r| self.setting_value(r, &col)).collect();
        let mut counts = core::value_counts(vals.iter().map(|v| v.as_ref()));
        // 今は無い値にチェックが付いていれば、件数 0 で残す(外せるように)。
        for k in &checked {
            if !counts.iter().any(|(c, _)| c == k) {
                counts.push((k.clone(), 0));
            }
        }
        let sel = usize::from(!counts.is_empty());
        if let Some(d) = self.draft.as_mut() {
            d.pick = Some(Pick::Values {
                col,
                keep,
                checked,
                counts,
                sel,
                edit,
            });
        }
    }

    pub(crate) fn open_text(
        &mut self,
        col: String,
        kind: TextKind,
        text: String,
        edit: Option<usize>,
    ) {
        if let Some(d) = self.draft.as_mut() {
            d.pick = None;
            d.text = Some(TextEntry {
                col,
                kind,
                text,
                edit,
            });
        }
        self.set_mode(Mode::SettingsText);
    }

    /// 条件を足す(`edit` があれば置き換える)。
    fn put_cond(&mut self, col: String, op: Op, edit: Option<usize>) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let cond = Cond { col, op };
        let i = match edit {
            Some(i) if i < d.s.filters.len() => {
                d.s.filters[i] = cond;
                i
            }
            _ => {
                d.s.filters.push(cond);
                d.s.filters.len() - 1
            }
        };
        d.pick = None;
        d.select(Sec::Filters, i);
    }

    /// 選び手の中の動作。
    pub(crate) fn pick_action(&mut self, action: Action) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let len = d.pick_len();
        let Some(p) = d.pick.as_mut() else {
            return;
        };
        match action {
            Action::Cancel => d.pick = None,
            Action::Up => {
                let s = p.sel();
                *p.sel_mut() = s.saturating_sub(1);
            }
            Action::Down => {
                let s = p.sel();
                *p.sel_mut() = (s + 1).min(len.saturating_sub(1));
            }
            Action::Toggle => match p {
                Pick::Values {
                    keep,
                    checked,
                    counts,
                    sel,
                    ..
                } => {
                    if *sel == 0 {
                        *keep = !*keep;
                    } else if let Some((k, _)) = counts.get(*sel - 1) {
                        match checked.iter().position(|c| c == k) {
                            Some(at) => {
                                checked.remove(at);
                            }
                            None => checked.push(k.clone()),
                        }
                    }
                }
                _ => self.pick_run(),
            },
            Action::Run => self.pick_run(),
            Action::NextSection | Action::PrevSection => {
                let esc = self.key_of(Mode::Settings, Action::Cancel);
                self.message = Some(Msg::PickInProgress.fill(&[&esc]));
            }
            _ => {}
        }
    }

    /// 選び手で選んだ項目を決める。
    fn pick_run(&mut self) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        let Some(p) = d.pick.clone() else {
            return;
        };
        match p {
            Pick::TabMenu { tab, sel } => self.tab_menu_run(tab, sel),
            Pick::Look { field, sel } => self.look_pick_run(field, sel),
            Pick::Column { purpose, sel } => {
                let Some(col) = d.keys.get(sel).cloned() else {
                    return;
                };
                match purpose {
                    Purpose::Filter => {
                        d.pick = Some(Pick::Kind {
                            col,
                            sel: 0,
                            edit: None,
                        })
                    }
                    Purpose::Sort => {
                        if d.s.sorts.iter().any(|(c, _)| *c == col) {
                            let t = self.title(&col);
                            self.message = Some(Msg::SortAlready.fill(&[&t]));
                            return;
                        }
                        d.s.sorts.push((col, Dir::Asc));
                        d.pick = None;
                        let n = d.s.sorts.len() - 1;
                        d.select(Sec::Sorts, n);
                    }
                    Purpose::Group => {
                        let (dir, hide_empty) = match &d.s.group {
                            Group::By {
                                dir, hide_empty, ..
                            } => (*dir, *hide_empty),
                            _ => (Dir::Asc, false),
                        };
                        d.s.group = Group::By {
                            col,
                            dir,
                            hide_empty,
                        };
                        d.pick = None;
                        d.select(Sec::Group, 2);
                    }
                    Purpose::WbsKey => {
                        d.wbs.key = col;
                        d.sync_wbs();
                        let w = d.wbs.clone();
                        let vals = self.wbs_values(&w);
                        if let Some(d) = self.draft.as_mut() {
                            d.wbs_vals = vals;
                            d.pick = None;
                            d.select(Sec::Tree, 3);
                        }
                    }
                    Purpose::Tree => {
                        if d.s.tree.is_some() {
                            d.s.tree = Some(col.clone());
                        }
                        d.tree_key = col;
                        d.pick = None;
                        d.select(Sec::Tree, 1);
                    }
                }
            }
            Pick::Kind { col, sel, edit } => match sel {
                0 => {
                    let (keep, checked) = match edit.and_then(|i| d.s.filters.get(i)).map(|c| &c.op)
                    {
                        Some(Op::Keep(k)) => (true, k.clone()),
                        Some(Op::Drop(k)) => (false, k.clone()),
                        _ => (false, Vec::new()),
                    };
                    self.open_values(col, keep, checked, edit);
                }
                1 => self.open_text(col, TextKind::Contains, String::new(), edit),
                2 => self.open_text(col, TextKind::NotContains, String::new(), edit),
                3 => d.pick = Some(Pick::Cmp { col, sel: 0, edit }),
                4 => self.put_cond(col, Op::Empty, edit),
                _ => self.put_cond(col, Op::NotEmpty, edit),
            },
            Pick::Cmp { col, sel, edit } => {
                let op = CMPS.get(sel).map(|c| c.0).unwrap_or(CmpOp::Eq);
                self.open_text(col, TextKind::Cmp(op), String::new(), edit);
            }
            Pick::Values {
                col,
                keep,
                checked,
                counts,
                edit,
                ..
            } => {
                if checked.is_empty() {
                    let sp = self.key_of(Mode::Settings, Action::Toggle);
                    self.message = Some(Msg::NoValueChecked.fill(&[&sp]));
                    return;
                }
                // チェックした値は一覧の順にそろえる。
                let keys: Vec<Option<String>> = counts
                    .into_iter()
                    .map(|(k, _)| k)
                    .filter(|k| checked.contains(k))
                    .collect();
                let op = if keep { Op::Keep(keys) } else { Op::Drop(keys) };
                self.put_cond(col, op, edit);
            }
        }
    }

    /// 値の入力に文字を足す。
    pub(crate) fn settings_insert(&mut self, c: char) {
        self.message = None;
        if let Some(t) = self.draft.as_mut().and_then(|d| d.text.as_mut()) {
            t.text.push(c);
        }
    }

    /// 値の入力の中の動作: 決める・戻る・1字消す。
    pub(crate) fn text_action(&mut self, action: Action) {
        let Some(t) = self.draft.as_ref().and_then(|d| d.text.clone()) else {
            return self.set_mode(Mode::Settings);
        };
        match action {
            Action::Cancel => {
                if let Some(d) = self.draft.as_mut() {
                    d.text = None;
                }
                self.set_mode(Mode::Settings);
            }
            Action::DeleteBack => {
                if let Some(t) = self.draft.as_mut().and_then(|d| d.text.as_mut()) {
                    t.text.pop();
                }
            }
            Action::Commit => {
                let v = t.text.trim().to_string();
                if v.is_empty() {
                    let esc = self.key_of(Mode::SettingsText, Action::Cancel);
                    self.message = Some(Msg::ValueEmpty.fill(&[&esc]));
                    return;
                }
                let op = match t.kind {
                    // BV-18: mdgrid のビューの名前(native_views.rs)。
                    TextKind::SaveAs | TextKind::Rename => {
                        return self.commit_view_name(t.kind, v);
                    }
                    // NV-26: ビューの区画からの名前の変更。
                    TextKind::RenameTab => return self.rename_tab(t.col, v),
                    TextKind::LookTemplate => return self.save_look_template(v),
                    TextKind::WbsValue => return self.put_wbs_value(t.col, v),
                    TextKind::DeleteView => return self.confirm_delete_view(t.col, v == "y"),
                    TextKind::Contains => Op::Contains(v),
                    TextKind::NotContains => Op::NotContains(v),
                    TextKind::Cmp(o) => {
                        if let Err(e) = self.check_operand(&t.col, &v) {
                            self.message = Some(e);
                            return;
                        }
                        Op::Cmp(o, v)
                    }
                };
                if let Some(d) = self.draft.as_mut() {
                    d.text = None;
                }
                self.set_mode(Mode::Settings);
                self.put_cond(t.col, op, t.edit);
            }
            _ => {}
        }
    }

    /// NV-28: 対応表の1行を決める(「割合 ラベル」。割合に `-` を打つと対応から外す)。
    fn put_wbs_value(&mut self, value: String, text: String) {
        let text = text.trim();
        let (pct, label) = match text.split_once(char::is_whitespace) {
            Some((p, l)) => (p, l.trim()),
            None => (text, ""),
        };
        let remove = pct == "-";
        let percent = match pct.parse::<u8>() {
            Ok(p) if p <= 100 => p,
            _ if remove => 0,
            _ => {
                self.message = Some(Msg::WbsBadPercent.fill(&[&text]));
                return;
            }
        };
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        d.wbs.map.retain(|m| m.value != value);
        if !remove {
            d.wbs.map.push(mdgrid::settings::WbsValue {
                value,
                percent,
                label: label.to_string(),
            });
        }
        d.sync_wbs();
        d.text = None;
        self.set_mode(Mode::Settings);
    }

    /// 比べる値が列の型で読めるか(NV-19)。読めなければ理由。
    fn check_operand(&self, col: &str, v: &str) -> Result<(), String> {
        match self.column_kind(col) {
            Kind::Number if v.parse::<f64>().is_err() => Err(Msg::OperandNumber.fill(&[&v])),
            Kind::Date if types::parse_date(v).is_none() => Err(Msg::OperandDate.fill(&[&v])),
            Kind::DateTime
                if types::parse_date(v).is_none()
                    && !types::fits(Kind::DateTime, &Value::Str(v.to_string())) =>
            {
                Err(Msg::OperandDateTime.fill(&[&v]))
            }
            _ => Ok(()),
        }
    }
}
