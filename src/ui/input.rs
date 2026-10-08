//! 入力ボックス(`impl App` の続き。CE-1〜CE-5・CE-7・CE-9〜CE-12・CV-2・SR-18)。
//! 列の型(`Source::kind`。CE-2)で入り方を変える: テキストは候補のリスト(CE-3。list.rs)か自由入力、
//! チェックボックスは Enter で切り替え(CE-4)、日付は `YYYY-MM-DD`・設定の形・`+3`・`-2`・空(CE-5・CE-22)と
//! カレンダー(CE-20・CE-21。calendar.rs)、数は数だけ(CE-7)。
//! 不正なら閉じずに理由を出す。型の合わない値(CV-2)も列の型の入力で直し、合わない値のままは書かない。
//! 行を選んでいれば(NV-5)、確定は選んだ行の同じ列への一括の設定(CE-10。`Changes::set_many` の1手)。

use super::app::App;
use super::calendar::Choice;
pub(crate) use super::entry::{hint, parse, Entry};
use super::keymap::{Action, Mode};
use super::list::{to_new, List};
use super::view::{data_y, visible_layout, Layout};
use super::width::{sanitize, take, width};
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::{self, Kind};
use unicode_segmentation::UnicodeSegmentation;

/// その場の入力ボックス(CE-1)。行と列は鍵で持つ(読み直しで添字が動いても同じセルに確定する)。
pub(crate) struct Input {
    pub row: RowId,
    pub col: String,
    pub text: String,
    /// カーソルの位置(text のバイトの位置。書記素の境目)。
    pub cursor: usize,
    /// 編集前の値(CE-11 の Ctrl+R で戻す)。
    pub initial: String,
    pub entry: Entry,
    /// 候補のリスト(CE-3)。None なら自由入力だけ。
    pub list: Option<List>,
    /// 一括の設定(CE-10)の行。None なら1行。
    pub bulk: Option<Vec<RowId>>,
    /// 一括から外した、畳んだまとまりの中の行の数(CE-10)。
    pub folded_out: usize,
    /// 文字の入力・BS・Del・リストの移動かクリックをした。一括はこれが真のときだけ書く(CE-10)。
    pub touched: bool,
    /// リストを開いた直後で、まだ文字も ↑↓ 以外の操作も無い。このとき打った文字は今の値を置き換える(CE-3)。
    pub fresh: bool,
    /// 今の値が列の型に合わない(CV-2)。
    pub mismatch: bool,
    /// 日付・日時の列のカレンダー(CE-20・CE-21。calendar.rs)。
    pub cal: Option<super::calendar::Cal>,
}

/// 値を入力ボックスの文字にする。改行を含む値・リストなどは None(1行の入力では直せない)。
fn input_text(v: Option<&Value>) -> Option<String> {
    Some(match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::Str(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Int(i)) => i.to_string(),
        Some(Value::Float(f)) => format!("{f:?}"),
        Some(Value::List(_)) | Some(Value::Other) => return None,
    })
}

pub(crate) fn new_value_text(v: &NewValue) -> String {
    match v {
        NewValue::Null => String::new(),
        NewValue::Str(s) | NewValue::Date(s) => s.clone(),
        NewValue::Bool(b) => b.to_string(),
        NewValue::Int(i) => i.to_string(),
        NewValue::Float(f) => format!("{f:?}"),
        NewValue::List(items) => items.join(", "),
        NewValue::RenameKey(to) => format!("→ {to}"),
        NewValue::DeleteKey => String::new(),
    }
}

/// 今の値と書く値が同じか。日付(WB-18 の Date)は、読んだ文字列(Str)と文字で比べる。
fn same_new(cur: Option<&NewValue>, v: &NewValue) -> bool {
    match (cur, v) {
        (Some(NewValue::Str(a) | NewValue::Date(a)), NewValue::Date(b)) => a == b,
        (Some(NewValue::Date(a)), NewValue::Str(b)) => a == b,
        _ => cur == Some(v),
    }
}

/// 選んだセルが画面に描かれていないときの理由(見えないセルは直さない。SR-3)。
const NOT_DRAWN: Msg = Msg::CellNotDrawn;

/// 読むだけの理由の前置き(一括で飛ばした理由では外す)。今の言語の文。
pub(crate) fn read_only_lead() -> &'static str {
    Msg::ReadOnlyLead.text()
}

/// 一括の設定(CE-10)のメッセージの後ろ: 畳んだまとまりで外した行の数と、飛ばした行の数と理由(重ねない)。
pub(crate) fn bulk_tail(msg: &mut String, folded_out: usize, reasons: Vec<String>) {
    if folded_out > 0 {
        msg.push_str(&Msg::BulkFoldedOut.fill(&[&folded_out]));
    }
    if !reasons.is_empty() {
        let n = reasons.len();
        let mut uniq: Vec<String> = Vec::new();
        for r in reasons {
            if !uniq.contains(&r) {
                uniq.push(r);
            }
        }
        msg.push_str(&Msg::BulkSkipped.fill(&[&n, &uniq.join(" / ")]));
    }
}

fn has_newline(s: &str) -> bool {
    s.contains(['\n', '\r'])
}

impl App {
    /// 編集のモードの動作(確定・取り消し・カーソル・候補のリスト)。
    pub(crate) fn input_action(&mut self, action: Action) {
        // CE-3: リストの ↑↓ のほかの操作(カーソル・BS・Ctrl+R など)のあとは、打つと続きを直す。
        // カレンダーの ↑↓(リストが出ていないとき)は日を動かして値を変えるので、そのあとは続きを直す。
        if !matches!(action, Action::ListUp | Action::ListDown) || self.active_list().is_none() {
            self.with_input(|i| i.fresh = false);
        }
        // 文字の編集とカーソルの操作は、リストから自由入力に切り替える(CE-3)。
        if matches!(
            action,
            Action::CursorLeft
                | Action::CursorRight
                | Action::CursorHome
                | Action::CursorEnd
                | Action::DeleteBack
                | Action::DeleteForward
        ) {
            self.list_to_free();
        }
        if matches!(action, Action::DeleteBack | Action::DeleteForward) {
            self.with_input(|i| i.touched = true);
        }
        match action {
            Action::Commit => {
                self.commit();
            }
            Action::Cancel => {
                self.input = None;
                self.close_input();
            }
            // 入力の文字を全部消す(端末の行の編集の Ctrl+U)。リストは自由入力に切り替える。
            Action::ClearInput => {
                self.list_to_free();
                self.with_input(|i| {
                    i.text.clear();
                    i.cursor = 0;
                    i.touched = true;
                });
                self.calendar_follow();
            }
            Action::Revert => self.with_input(|i| {
                i.text = i.initial.clone();
                i.cursor = i.text.len();
                i.touched = false;
                if let Some(l) = &mut i.list {
                    l.free = false;
                    l.filter = None;
                    l.sel = if l.initial_sel < l.items.len() {
                        l.initial_sel
                    } else {
                        0
                    };
                    l.moved = false;
                }
            }),
            Action::ListUp => self.list_move(false),
            Action::ListDown => self.list_move(true),
            Action::CommitNext | Action::CommitPrev => {
                let row = self.input.as_ref().map(|i| i.row.clone());
                if self.commit() {
                    // 隣を開くのは組み立て直しのあと(直した値で行が絞り込みから外れたら、別の行に開かない)。
                    if std::mem::take(&mut self.regrid) {
                        self.refresh();
                    }
                    if row.is_some() && self.cur_row() != row {
                        let label = row.map(|r| self.src.label(&r)).unwrap_or_default();
                        self.message = Some(Msg::NeighborRowGone.fill(&[&label]));
                    } else {
                        self.edit_neighbor(action == Action::CommitNext);
                    }
                }
            }
            Action::CursorLeft => self.with_input(|i| {
                if let Some((k, _)) = i.text[..i.cursor].grapheme_indices(true).next_back() {
                    i.cursor = k;
                }
            }),
            Action::CursorRight => self.with_input(|i| {
                if let Some(g) = i.text[i.cursor..].graphemes(true).next() {
                    i.cursor += g.len();
                }
            }),
            Action::CursorHome => self.with_input(|i| i.cursor = 0),
            Action::CursorEnd => self.with_input(|i| i.cursor = i.text.len()),
            Action::DeleteBack => self.with_input(|i| {
                if let Some((k, _)) = i.text[..i.cursor].grapheme_indices(true).next_back() {
                    i.text.replace_range(k..i.cursor, "");
                    i.cursor = k;
                }
            }),
            Action::DeleteForward => self.with_input(|i| {
                if let Some(g) = i.text[i.cursor..].graphemes(true).next() {
                    let end = i.cursor + g.len();
                    i.text.replace_range(i.cursor..end, "");
                }
            }),
            _ => {}
        }
    }

    fn with_input(&mut self, f: impl FnOnce(&mut Input)) {
        if let Some(i) = &mut self.input {
            f(i);
        }
    }

    /// 入力ボックスに文字を入れる(CE-1)。改行は入れない(CE-12)。リストは自由入力に切り替える(CE-3)。
    pub(crate) fn insert(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            return;
        }
        self.message = None;
        // CE-3: リストを開いた直後に打った文字は、今の値を置き換える。
        let replace = self.input.as_ref().is_some_and(|i| {
            i.fresh
                && (self.active_list().is_some()
                    || matches!(i.entry, Entry::Date | Entry::DateTime | Entry::Number))
        });
        self.list_to_free();
        self.with_input(|i| {
            if replace {
                i.text.clear();
                i.cursor = 0;
            }
            i.fresh = false;
            i.touched = true;
            i.text.insert(i.cursor, c);
            i.cursor += c.len_utf8();
        });
        self.calendar_follow();
    }

    /// 入力を閉じて、開いたところ(表か詳細の表示)へ戻る。
    pub(crate) fn close_input(&mut self) {
        let back = if self.detail.is_some() {
            Mode::Detail
        } else {
            Mode::Table
        };
        self.set_mode(back);
    }

    /// 選んだセルの入力を開く(CE-1)。読むだけ(CE-6・CE-8・WB-5)と
    /// 改行を含む値(CE-12)では開かず、メッセージ行に理由を出す。チェックボックスは切り替える(CE-4)。
    pub(crate) fn open_edit(&mut self) {
        self.open_selected(true);
    }

    /// `toggle` が偽なのは Tab で隣へ移ったとき(CE-11): チェックボックスは切り替えずにリスト、一括にしない。
    fn open_selected(&mut self, toggle: bool) {
        let Some((row, col)) = self.selected() else {
            return;
        };
        if !super::view::selected_drawn(self) {
            self.message = Some(NOT_DRAWN.into());
            return;
        }
        self.open_at(row, col, toggle);
    }

    /// 行と列を決めて入力を開く(詳細の表示からも。NV-6)。
    pub(crate) fn open_edit_at(&mut self, row: RowId, col: String) {
        self.open_at(row, col, true);
    }

    /// 一括の設定(CE-10)の行と、外した畳んだまとまりの中の行の数: 表で今の行を含む行を選んでいるときだけ。
    /// 今の行が選択の外なら1行の編集(入力ボックスの出る行と書く行を揃える)。
    pub(crate) fn bulk_rows(&self, row: &RowId) -> Option<(Vec<RowId>, usize)> {
        if self.detail.is_some() {
            return None;
        }
        let s = self.selection();
        if !s.contains(row) {
            return None;
        }
        let hidden: std::collections::HashSet<&RowId> = self
            .groups
            .iter()
            .filter(|(h, _)| self.folded.contains(h))
            .flat_map(|(_, r)| self.rows[r.clone()].iter())
            .collect();
        let n = s.len();
        let shown: Vec<RowId> = s.into_iter().filter(|r| !hidden.contains(r)).collect();
        let out = n - shown.len();
        Some((shown, out))
    }

    /// 入力を開く。`toggle` ならチェックボックスは開かずに切り替える(CE-4。Tab で着いたときはリスト)。
    /// 印を付けた行の外で開いたら、この行だけを直すことと入らない印の行の数を知らせる(CE-10)。
    fn open_at(&mut self, row: RowId, col: String, toggle: bool) {
        let marked = if self.detail.is_none() {
            self.selection()
        } else {
            Vec::new()
        };
        let outside = !marked.is_empty() && !marked.contains(&row);
        self.open_at_inner(row, col, toggle);
        if outside && matches!(self.mode, Mode::Edit | Mode::ListPick) && self.message.is_none() {
            self.message = Some(Msg::EditOnlyThisRow.fill(&[&marked.len()]));
        }
    }

    fn open_at_inner(&mut self, row: RowId, col: String, toggle: bool) {
        let (bulk, folded_out) = match self.bulk_rows(&row).filter(|_| toggle) {
            Some((b, n)) => (Some(b), n),
            None => (None, 0),
        };
        // CE-16: リストのセル(列の型がリストか、値がリスト)は、どの道(Tab の移動・一括)でも
        // 1行の入力を開かず、リストの選択(listpick.rs)で開く。開けない行は飛ばし、無ければ理由を出す。
        if self.is_list_cell(&row, &col) {
            return self.open_pick(row, col, bulk.map(|b| (b, folded_out)));
        }
        // 最初の文字と今の値は今の行から(今の行が直せなければ、選んだ行の直せる最初の行から)。
        let probe: Vec<RowId> = match &bulk {
            Some(b) => std::iter::once(row.clone()).chain(b.clone()).collect(),
            None => vec![row.clone()],
        };
        let mut first_err = None;
        let mut found = None;
        for r in &probe {
            match self.editable(r, &col) {
                Ok(t) => {
                    found = Some((r.clone(), t));
                    break;
                }
                Err(e) => {
                    first_err.get_or_insert(e);
                }
            }
        }
        let Some((base, initial)) = found else {
            let e = first_err.unwrap_or_default();
            self.message = Some(match &bulk {
                Some(_) => Msg::BulkNoEditable.fill(&[&e]),
                None => e,
            });
            return;
        };
        let kind = self.src.kind(&col).kind;
        let cur = self.current(&base, &col);
        let mismatch =
            self.src.get(&base, &col).value.is_some_and(|v| {
                self.changes.pending(&base, &col).is_none() && !types::fits(kind, &v)
            });
        if kind == Kind::Checkbox && toggle && !mismatch {
            // CE-4: 1回の操作で真と偽を切り替える(空・キーの無いノートは真に。空にするのは BS・Delete)。
            // 次の状態は今の行の表示の値で決める。
            let next = match &cur {
                Some(NewValue::Bool(true)) => Some(false),
                _ => Some(true),
            };
            self.toggle_checkbox(&row, &col, bulk.as_deref(), folded_out, next);
            return;
        }
        let entry = match kind {
            Kind::Number => Entry::Number,
            Kind::Checkbox => Entry::Checkbox,
            Kind::Date => Entry::Date,
            Kind::DateTime => Entry::DateTime,
            Kind::Text | Kind::List => Entry::Text,
        };
        let list = match kind {
            // REL-3: リンクの列は、行き先のフォルダのノートを候補にする(見せるのは名前、書くのはリンク)。
            Kind::Text => self.link_list(&base, &col, cur.as_ref()).or_else(|| {
                self.src
                    .candidates(&col, self.candidates)
                    .map(|c| List::new(&c, cur.as_ref(), !mismatch))
            }),
            Kind::Checkbox => Some(List::new(
                &[Value::Bool(true), Value::Bool(false)],
                cur.as_ref(),
                !mismatch,
            )),
            _ => None,
        };
        // CE-22: 日付は設定の形で見せる(4桁の年が無い形なら `YYYY-MM-DD`。型の合わない値はそのまま)。
        let raw = initial;
        let initial = match types::parse_date(raw.trim()).filter(|_| entry == Entry::Date) {
            Some(d) => super::calendar::input_date(&self.date_format, d),
            None => raw.clone(),
        };
        // カレンダーの日は元の値(raw)の日数から(入力の文字を読み直さない)。
        let cal = super::calendar::Cal::open(entry, &raw, &initial, self.today);
        // CE-3・CE-5・CE-7: リスト・日付・日時・数の入力は、開いた直後に打つと今の値を置き換える。
        let fresh =
            list.is_some() || matches!(entry, Entry::Date | Entry::DateTime | Entry::Number);
        self.input = Some(Input {
            row,
            col,
            cursor: initial.len(),
            text: initial.clone(),
            initial,
            entry,
            list,
            bulk,
            folded_out,
            touched: false,
            fresh,
            mismatch,
            cal,
        });
        self.set_mode(Mode::Edit);
    }

    /// セルの今の値(ためた値があればそれ)。キーが無ければ None。
    fn current(&self, row: &RowId, col: &str) -> Option<NewValue> {
        if let Some(v) = self.changes.pending(row, col) {
            return Some(v.clone());
        }
        self.src.get(row, col).value.as_ref().and_then(to_new)
    }

    /// セルを1行の入力で直せるなら入力の最初の文字、直せなければ理由(メッセージ行の文)。
    fn editable(&self, row: &RowId, col: &str) -> Result<String, String> {
        if let Some(v) = self.changes.pending(row, col) {
            return Ok(new_value_text(v));
        }
        let cell = self.src.get(row, col);
        if let Some(Value::Str(s)) = &cell.value {
            if has_newline(s) {
                return Err(Msg::EditMultiLine.into());
            }
        }
        if let Some(r) = &cell.lock {
            return Err(format!("{}{r}", read_only_lead()));
        }
        input_text(cell.value.as_ref()).ok_or_else(|| Msg::EditNotOneLine.into())
    }

    /// CE-11: 確定したあと、同じ行の右(左)の直せるセルの入力を開く。読むだけのセルは飛ばす。
    /// 直せるセルが無ければ表に戻り、飛ばした理由を出す。チェックボックスは切り替えずにリストを開く。
    fn edit_neighbor(&mut self, right: bool) {
        let Some(row) = self.cur_row() else {
            return;
        };
        let mut j = self.col;
        let mut last = None;
        loop {
            let next = if right {
                (j + 1 < self.cols.len()).then(|| j + 1)
            } else {
                j.checked_sub(1)
            };
            let Some(n) = next else { break };
            j = n;
            // CE-16: リストのセルは、書けるならリストの選択を開き、書けなければ飛ばす。
            let found = if self.is_list_cell(&row, &self.cols[j]) {
                self.list_of(&row, &self.cols[j]).map(|_| String::new())
            } else {
                self.editable(&row, &self.cols[j])
            };
            match found {
                Ok(_) => {
                    self.col = j;
                    self.scroll_into_view();
                    self.open_selected(false);
                    return;
                }
                Err(m) => last = Some(m),
            }
        }
        self.message = Some(match (last, right) {
            (Some(m), true) => Msg::NoEditableRight.fill(&[&m]),
            (Some(m), false) => Msg::NoEditableLeft.fill(&[&m]),
            (None, true) => Msg::NoCellRight.into(),
            (None, false) => Msg::NoCellLeft.into(),
        });
    }

    /// 入力を確定してためる変更にする(CE-1・CE-9)。閉じたら true。値が不正なら閉じずに理由を出す。
    /// 1行で編集前と同じなら何もしない(数や真偽の値を、開いて閉じただけで書き直さない)。
    fn commit(&mut self) -> bool {
        let Some(i) = self.input.take() else {
            return false;
        };
        // 表の入力ボックスが描かれていなければ確定しない(見えないセルに書かない)。
        if self.detail.is_none() && !super::view::selected_drawn(self) {
            self.message = Some(Msg::CellNotDrawnEsc.into());
            self.input = Some(i);
            return false;
        }
        // 一括(CE-10)は、触った(打った・消した・リストを動かした)ときだけ書く。値が今の行と同じでも選んだ行に揃える。
        // 触らずに閉じたら書かない(今の行の値を広げない)。1行は、文字が同じかリストの選びが今の値なら書かない。
        let list = i.list.as_ref().filter(|l| !l.free);
        let bulk = i.bulk.is_some();
        let v = match list {
            Some(_) if bulk && !i.touched => None,
            Some(l) if bulk => Some(Choice::Value(l.chosen())),
            Some(l) if !l.moved || l.sel == l.initial_sel => None,
            Some(l) => Some(Choice::Value(l.chosen())),
            None if bulk && !i.touched => None,
            None if !bulk && i.text == i.initial => None,
            // CE-20・CE-21: カレンダーで選んだ日・日時の日だけの打ち込みは calendar.rs が決める。
            None => match super::calendar::choice(&i, self.today, &self.date_format) {
                Some(c) => Some(c),
                None => match parse(i.entry, &i.text, self.today, &self.date_format) {
                    Ok(v) => Some(Choice::Value(v)),
                    Err(e) => {
                        self.message = Some(e);
                        self.input = Some(i);
                        return false;
                    }
                },
            },
        };
        let v = v.filter(|c| {
            i.bulk.is_some()
                || !same_new(
                    self.current(&i.row, &i.col).as_ref(),
                    &c.value(self, &i.row, &i.col),
                )
        });
        if let Some(v) = v {
            if let Err(m) = self.put(&i.row, &i.col, i.bulk.as_deref(), i.folded_out, v) {
                self.message = Some(m);
                self.input = Some(i);
                return false;
            }
        }
        self.close_input();
        true
    }

    /// 値をためる。`bulk` なら選んだ行に1手で(CE-10)、飛ばした数と理由と、畳んで外した数をメッセージ行に出す。
    /// 値は行ごとに `Choice::value` で決める(日時の日だけの変更は、行ごとの元の時刻を保つ。CE-21)。
    fn put(
        &mut self,
        row: &RowId,
        col: &str,
        bulk: Option<&[RowId]>,
        folded_out: usize,
        v: Choice,
    ) -> Result<(), String> {
        let Some(rows) = bulk else {
            let v = v.value(self, row, col);
            self.changes
                .set(self.src.as_ref(), row, col, v)
                .map_err(|s| Msg::CannotCommit.fill(&[&s.reason]))?;
            // NV-12: 直した行は、保存か移動の操作まで元の位置に留める。
            self.stay.insert(row.clone());
            self.regrid = true;
            return Ok(());
        };
        let mut reasons: Vec<String> = Vec::new();
        let mut items = Vec::new();
        let mut same = 0;
        for r in rows {
            if let Err(e) = self.editable(r, col) {
                reasons.push(e.trim_start_matches(read_only_lead()).to_string());
                continue;
            }
            let rv = v.value(self, r, col);
            if same_new(self.current(r, col).as_ref(), &rv) {
                same += 1;
            } else {
                items.push((r.clone(), Some(rv)));
            }
        }
        let skips = self.changes.set_each(self.src.as_ref(), col, &items);
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let mut msg = Msg::BulkSet.fill(&[&done]);
        if same > 0 {
            msg.push_str(&Msg::BulkSame.fill(&[&same]));
        }
        bulk_tail(&mut msg, folded_out, reasons);
        self.message = Some(msg);
        Ok(())
    }

    /// チェックボックスを切り替える(CE-4・CE-9・CE-10・WB-17)。`next` は今の行で決めた次の状態
    /// (`Some(真偽)` か、`None` で空)。選んだ行の全部に同じ状態を当て、1手にまとめる。
    /// 真偽: その値(元と同じ行は WB-17 で外れる)。
    /// 空: `key:`(NewValue::Null。CE-9)。ただし元が空(キーが無いか Null か空の文字列)の行はためた値を外す(変更なし)。
    /// 一括では、型の合わない値の行(1行なら入力を開く行)は上書きせずに飛ばす(CE-10)。
    fn toggle_checkbox(
        &mut self,
        row: &RowId,
        col: &str,
        bulk: Option<&[RowId]>,
        folded_out: usize,
        next: Option<bool>,
    ) {
        let value_for = |app: &App, r: &RowId| -> Option<NewValue> {
            let empty = match &app.src.get(r, col).value {
                None | Some(Value::Null) => true,
                Some(Value::Str(s)) => s.is_empty(),
                Some(_) => false,
            };
            match (next, empty) {
                (Some(b), _) => Some(NewValue::Bool(b)),
                (None, true) => None,
                (None, false) => Some(NewValue::Null),
            }
        };
        let Some(rows) = bulk else {
            let items = [(row.clone(), value_for(self, row))];
            if let Some(s) = self
                .changes
                .set_each(self.src.as_ref(), col, &items)
                .first()
            {
                self.message = Some(Msg::CannotCommit.fill(&[&s.reason]));
                return;
            }
            // NV-12: 直した行は、保存か移動の操作まで元の位置に留める。
            self.stay.insert(row.clone());
            self.regrid = true;
            return;
        };
        let mut reasons: Vec<String> = Vec::new();
        let mut items = Vec::new();
        let mut same = 0;
        for r in rows {
            if let Err(e) = self.editable(r, col) {
                reasons.push(e.trim_start_matches(read_only_lead()).to_string());
                continue;
            }
            let mismatch = self.changes.pending(r, col).is_none()
                && self
                    .src
                    .get(r, col)
                    .value
                    .is_some_and(|v| !types::fits(Kind::Checkbox, &v));
            if mismatch {
                reasons.push(Msg::BulkTypeMismatch.into());
                continue;
            }
            let v = value_for(self, r);
            // 表示の値がもう目当てと同じ行(None はためた値が無い行)は数えるだけ。
            let already = match &v {
                Some(v) => self.current(r, col).as_ref() == Some(v),
                None => self.changes.pending(r, col).is_none(),
            };
            if already {
                same += 1;
            } else {
                items.push((r.clone(), v));
            }
        }
        let skips = self.changes.set_each(self.src.as_ref(), col, &items);
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let state = match next {
            Some(true) => "true",
            Some(false) => "false",
            None => Msg::BulkStateEmpty.text(),
        };
        let mut msg = Msg::BulkSetTo.fill(&[&done, &state]);
        if same > 0 {
            msg.push_str(&Msg::BulkSame.fill(&[&same]));
        }
        bulk_tail(&mut msg, folded_out, reasons);
        self.message = Some(msg);
    }

    /// 選んだセルを空にする(SR-18・CE-9)。読むだけなら理由を出して何もしない。
    pub(crate) fn clear(&mut self) {
        let Some((row, col)) = self.selected() else {
            return;
        };
        if !super::view::selected_drawn(self) {
            self.message = Some(NOT_DRAWN.into());
            return;
        }
        match self
            .changes
            .set(self.src.as_ref(), &row, &col, NewValue::Null)
        {
            Ok(()) => {
                self.stay.insert(row);
                self.regrid = true;
            }
            Err(skip) => self.message = Some(Msg::CannotClear.fill(&[&skip.reason])),
        }
    }
}

// ---- 描画(純関数) ----

/// 入力ボックスの位置と見せる文字(CE-1・SR-17)。
pub(crate) struct InputBox {
    /// 描く範囲の中の桁と行。
    pub x: usize,
    pub y: usize,
    pub w: usize,
    /// 見せる文字(幅 w 以下。カーソルが見えるように左を落とす)。
    pub text: String,
    /// カーソルの桁。
    pub cursor_x: usize,
}

/// 入力ボックスの最小の幅。
const INPUT_MIN: usize = 16;

pub(crate) fn input_box(app: &App, lay: &Layout, cols: &[(usize, usize)]) -> Option<InputBox> {
    // CE-25: 新しいノートの入力は表の下の端の欄(new_note.rs)。
    if app.note.flow.is_some() {
        return super::new_note::note_box(app, lay.width);
    }
    let input = app.input.as_ref()?;
    if app.mode != Mode::Edit || app.row < app.top || app.row >= app.top + app.data_height() {
        return None;
    }
    let mut x = lay.data_x();
    let mut cw = None;
    for &(j, w) in cols {
        if j == app.col {
            cw = Some(w);
            break;
        }
        x += w + 1;
    }
    let cw = cw?;
    if x >= lay.width {
        return None;
    }
    let want = width(&sanitize(&input.text)) + 1;
    let w = cw.max(want).max(INPUT_MIN).min(lay.width - x);
    // カーソルの左は、カーソルの1桁が収まるまで先頭から落とす。
    let mut before = sanitize(&input.text[..input.cursor]);
    while width(&before) + 1 > w && !before.is_empty() {
        let first = before.graphemes(true).next().map(str::len).unwrap_or(0);
        before.drain(..first);
    }
    let after = sanitize(&input.text[input.cursor..]);
    let cursor_x = x + width(&before).min(w.saturating_sub(1));
    let text = take(&format!("{before}{after}"), w);
    Some(InputBox {
        x,
        y: data_y(app) + app.row - app.top,
        w,
        text,
        cursor_x,
    })
}

/// 端末のカーソルを置く位置(SR-17: 入力ボックスでは入力の位置。変換の窓がそこに出る)。
pub fn cursor(app: &App) -> Option<(u16, u16)> {
    match app.mode {
        Mode::Palette => return super::help::palette_cursor(app),
        Mode::Search | Mode::Filter => return super::view::prompt_cursor(app),
        Mode::SettingsText => return super::settings_view::text_cursor(app),
        Mode::Settings | Mode::Chips => return None,
        Mode::ListPick => return super::listpick::pick_cursor(app),
        Mode::Edit if app.detail.is_some() => return super::detail::detail_cursor(app),
        _ => {}
    }
    let (lay, cols) = visible_layout(app);
    let b = input_box(app, &lay, &cols)?;
    Some((b.cursor_x as u16, b.y as u16))
}

/// (x, y) が入力ボックスの中か(CE-1)。
pub fn in_input(app: &App, x: u16, y: u16) -> bool {
    let (lay, cols) = visible_layout(app);
    input_box(app, &lay, &cols)
        .is_some_and(|b| y as usize == b.y && (x as usize) >= b.x && (x as usize) < b.x + b.w)
}
