//! 表から新しいノートを作る画面(`impl App` の続きと描画の純関数。CE-25・CE-26・CE-27)。
//!
//! 始め方は3つ: ヘッダーの右の端の「+ 新規」のクリック、表のモードの `new_note`(既定 `a`)、パレットの
//! 「新しいノート」。表の下の端に欄を出し、名前(雛形 `expand_name` で始まる)を打って Enter →
//! 決まり(`rule_for`: ビューの new_note か設定の `[new_note]`)の `ask` の列を1つずつ、列の型の入力で聞く
//! (数・チェックは1行の入力、テキストはセルの編集と同じ CE-3 の候補のリスト付き(list.rs)、
//! 日付・日時は CE-20 のカレンダー付き、リストは CE-16 の選択)→
//! 最後の Enter で作る。聞く項目を空で Enter ならその項目は書かない。Esc でどこからでも取りやめる。
//! 入力は編集のモード(`Mode::Edit`・`Mode::ListPick`)の入力ボックス(input.rs)とリストの選択(listpick.rs)を
//! そのまま使い、確定と取り消しだけをここで受ける(書き先の行は無い)。
//! 作る場所は開いたフォルダ(`.base` ならそのフォルダ)の下の決まりの `folder`。フォルダを2つ以上開いたときは、
//! 名前の欄の前に開いたフォルダの一覧(起動の引数の順。`places`)を候補のリスト(list.rs)の見せ方で出し、
//! ↑↓ で選んで Enter で決めたフォルダを根にする(既定は最初。Shift+Tab で名前から戻れる)。前もって入れる値は今のビューの
//! 設定と式の絞り込み(`prefill`)。作るのはためる変更を経ず、その場で `create`(create_new)で作る。
//! 作れないとき(核の理由)はメッセージ行に出して欄は開いたまま。作ったら読み直して、その行を選ぶ。
//! 読むだけ(WB-15)では始めない。

use super::app::App;
use super::calendar::{self, Cal, Choice};
use super::entry::{parse, Entry};
use super::grid::Slot;
use super::input::{new_value_text, Input, InputBox};
use super::keymap::{self, Action, Mode};
use super::list::{to_new, Item, List};
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::newnote::{self, NewNote};
use mdgrid::source::{date_if_untyped, Edit, NewValue, RowId, Source};
use mdgrid::types::{self, Kind};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// ヘッダーの右の端のボタン(CE-25)の今の言語の文字。クリックの当たり(`button_at`)もこの幅から。
pub(crate) fn button() -> &'static str {
    Msg::NoteButton.text()
}

/// 新しいノートの設定と、作っている途中の状態。
#[derive(Default)]
pub(crate) struct NoteMaker {
    /// 設定の `[new_note]`(ビューに new_note が無いときの決まり)。
    pub rule: NewNote,
    /// 作っている途中(作る場所・名前か聞く項目の入力が開いている)。
    pub flow: Option<Flow>,
    /// 開いたフォルダの一覧(起動の引数の順。`.base` はそのフォルダ。同じフォルダは1つ。`places`)。
    pub places: Vec<PathBuf>,
}

/// 作っている途中の状態。
pub(crate) struct Flow {
    /// 開いたフォルダ(作る場所の根。2つ以上なら選んだもの)。
    root: PathBuf,
    /// 開いたフォルダの一覧(2つ以上なら名前の前に選ぶ。CE-25)。
    places: Vec<PathBuf>,
    /// 作る場所を選んでいる(名前の欄の前)。
    choosing: bool,
    rule: NewNote,
    /// 今のビューの絞り込みから前もって入れる値。
    prefill: Vec<Edit>,
    /// 決めた名前(名前の Enter のあと)。
    name: String,
    /// 0 は名前、k は `rule.ask[k - 1]`。
    step: usize,
    /// 聞いた値(空は Null。書かない)。
    answers: Vec<Edit>,
}

impl Flow {
    /// 今聞いている列(名前の間は None)。
    fn col(&self) -> Option<&str> {
        self.step
            .checked_sub(1)
            .and_then(|k| self.rule.ask.get(k))
            .map(String::as_str)
    }

    /// 前もって入れる値(絞り込みが先、無ければ設定の set)。
    fn preset(&self, col: &str) -> Option<&NewValue> {
        self.prefill
            .iter()
            .find(|e| e.key == col)
            .map(|e| &e.value)
            .or_else(|| self.rule.set.iter().find(|(k, _)| k == col).map(|(_, v)| v))
    }
}

/// 書き先の行の無い入力の鍵(入力ボックスとリストの選択は行を持つ形なので、空の鍵を入れる)。
fn no_row() -> RowId {
    RowId(String::new())
}

impl App {
    /// 「+ 新規」・`a`・パレット: 名前の欄を開く(CE-25)。フォルダを2つ以上開いたときは、その前に
    /// 作る場所を選ぶ欄を開く(既定は最初)。読むだけでは開かない(WB-15)。
    pub(crate) fn start_new_note(&mut self) {
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        let places: Vec<PathBuf> = self
            .note
            .places
            .iter()
            .filter(|p| p.is_dir())
            .cloned()
            .collect();
        let Some(root) = places.first().cloned() else {
            self.message = Some(Msg::NoteNoPlace.text().into());
            return;
        };
        if panel_y(self).is_none() {
            self.message = Some(Msg::NoteTooLow.text().into());
            return;
        }
        let choosing = places.len() > 1;
        let mut rule = self
            .native_view()
            .and_then(|v| v.new_note.clone())
            .unwrap_or_else(|| self.note.rule.clone());
        // WB-18: 型の決まらない列に前もって入れる日付・日時の形だけの文字は、日付として(囲まずに)書く。
        let src = self.src.as_ref();
        for (k, v) in &mut rule.set {
            *v = date_if_untyped(src, k, std::mem::replace(v, NewValue::Null));
        }
        // CE-25: ビューの絞り込みの file.inFolder のフォルダに作る(作った行がビューに残る)。
        // 設定のフォルダがその中ならそのまま。作る場所の根の外なら今の決まり(外には作らない)のまま。
        if let Some(dir) = newnote::view_folder(&self.view_filter_exprs()) {
            if let Some(rel) = folder_under_root(&root, &dir) {
                let current = root.join(rule.folder.trim().trim_matches('/'));
                let target = root.join(&rel);
                if !current.starts_with(&target) {
                    rule.folder = rel;
                }
            }
        }
        let mut prefill = self.view_prefill();
        for e in &mut prefill {
            e.value = date_if_untyped(src, &e.key, std::mem::replace(&mut e.value, NewValue::Null));
        }
        let text = newnote::expand_name(&rule.name, self.today);
        self.note.flow = Some(Flow {
            root,
            places,
            choosing,
            rule,
            prefill,
            name: String::new(),
            step: 0,
            answers: Vec::new(),
        });
        if choosing {
            self.open_place(0);
        } else {
            self.open_note_input(Entry::Text, text, None);
        }
    }

    /// 作る場所を選ぶ欄が開いているか(CE-25。文字の入力を受けない)。
    pub(crate) fn note_choosing(&self) -> bool {
        self.note.flow.as_ref().is_some_and(|f| f.choosing)
    }

    /// 作る場所を選ぶ欄を開く(CE-25)。セルの編集と同じ候補のリスト(list.rs)の見せ方で、`sel` を選んで始める。
    fn open_place(&mut self, sel: usize) {
        let Some(f) = &self.note.flow else {
            return;
        };
        let items: Vec<Item> = f
            .places
            .iter()
            .enumerate()
            .map(|(k, p)| Item {
                value: NewValue::Int(k as i64),
                text: place_name(p),
                current: false,
            })
            .collect();
        let sel = sel.min(items.len().saturating_sub(1));
        let list = List {
            items,
            sel,
            initial_sel: sel,
            moved: false,
            free: false,
            filter: None,
            narrow: false,
        };
        self.open_note_input(Entry::Text, String::new(), None);
        if let Some(i) = &mut self.input {
            i.list = Some(list);
        }
    }

    /// 作る場所の確定: 選んだフォルダを根にして名前の欄へ(打った文字は使わず、リストの選びで決める)。
    fn place_commit(&mut self, sel: usize) {
        let today = self.today;
        let Some(f) = &mut self.note.flow else {
            return;
        };
        if let Some(p) = f.places.get(sel) {
            f.root = p.clone();
        }
        f.choosing = false;
        let text = if f.name.is_empty() {
            newnote::expand_name(&f.rule.name, today)
        } else {
            f.name.clone()
        };
        self.open_note_input(Entry::Text, text, None);
    }

    /// 今のビュー(`.base` のビュー・mdgrid のビュー)の式の絞り込み。
    fn view_filter_exprs(&self) -> Vec<String> {
        let mut exprs: Vec<String> = Vec::new();
        let mut dropped = Vec::new();
        if let Some(nv) = self.native_view() {
            exprs.extend(nv.filters_expr.iter().cloned());
        } else if let Some(b) = &self.base {
            exprs.extend(b.base.filter_exprs(&mut dropped));
            if let Some(v) = b.base.views.get(b.view) {
                exprs.extend(v.filter_exprs(&mut dropped));
            }
        }
        exprs
    }

    /// 今のビュー(既定の表・`.base` のビュー・mdgrid のビュー)の設定と式の絞り込みから、前もって入れる値。
    fn view_prefill(&self) -> Vec<Edit> {
        let exprs = self.view_filter_exprs();
        let mut cols = self.src.columns();
        cols.extend(self.settings.filters.iter().map(|c| c.col.clone()));
        let kinds: HashMap<String, Kind> = cols
            .into_iter()
            .map(|c| {
                let k = self.column_kind(&c);
                (c, k)
            })
            .collect();
        newnote::prefill(&self.settings, &exprs, &kinds)
    }

    /// 1行の入力を開く(名前と、リストでない聞く項目)。
    fn open_note_input(&mut self, entry: Entry, text: String, cal: Option<Cal>) {
        self.input = Some(Input {
            row: no_row(),
            col: String::new(),
            cursor: text.len(),
            initial: text.clone(),
            text,
            entry,
            list: None,
            bulk: None,
            folded_out: 0,
            touched: false,
            fresh: false,
            mismatch: false,
            cal,
        });
        self.set_mode(Mode::Edit);
    }

    /// 次の聞く項目を開く。
    fn ask_next(&mut self) {
        let Some(f) = &mut self.note.flow else {
            return;
        };
        f.step += 1;
        self.open_step(None);
    }

    /// Shift+Tab: 前の項目へ戻る(名前まで戻れる)。前に答えた値で欄を開き直す。
    fn note_back(&mut self) {
        self.pick = None;
        let Some(f) = &mut self.note.flow else {
            return;
        };
        if f.step == 0 {
            // CE-25: 作る場所を選ぶ欄があれば、名前からそこへ戻る(打っていた名前は覚えておく)。
            if !f.choosing && f.places.len() > 1 {
                f.name = self
                    .input
                    .as_ref()
                    .map(|i| i.text.clone())
                    .unwrap_or_default();
                f.choosing = true;
                let sel = f.places.iter().position(|p| *p == f.root).unwrap_or(0);
                return self.open_place(sel);
            }
            self.message = Some(if f.choosing {
                Msg::NoteFirstPlace.text().into()
            } else {
                Msg::NoteFirstName.text().into()
            });
            return;
        }
        f.step -= 1;
        if f.step == 0 {
            let name = f.name.clone();
            f.answers.clear();
            return self.open_note_input(Entry::Text, name, None);
        }
        let prev = f.answers.pop().map(|e| e.value);
        self.open_step(Some(prev.unwrap_or(NewValue::Null)));
    }

    /// 今の聞く項目の欄を開く。`value` があればその値で(戻ったとき)、無ければ前もって入れる値で始める。
    fn open_step(&mut self, value: Option<NewValue>) {
        let Some(f) = &self.note.flow else {
            return;
        };
        let Some(col) = f.col().map(String::from) else {
            return;
        };
        let preset = value.or_else(|| f.preset(&col).cloned());
        let kind = self.src.kind(&col).kind;
        if kind == Kind::List {
            // CE-16: リストの列はリストの選択で(前もって入れる値は付いた要素にする)。
            let items = match preset {
                Some(NewValue::List(items)) => items,
                Some(NewValue::Str(s)) if !s.is_empty() => vec![s],
                _ => Vec::new(),
            };
            self.input = None;
            let pick = self.new_pick(col, vec![(no_row(), items)]);
            self.pick = Some(pick);
            self.set_mode(Mode::ListPick);
            return;
        }
        let entry = match kind {
            Kind::Number => Entry::Number,
            Kind::Checkbox => Entry::Checkbox,
            Kind::Date => Entry::Date,
            Kind::DateTime => Entry::DateTime,
            Kind::Text | Kind::List => Entry::Text,
        };
        let raw = preset.as_ref().map(new_value_text).unwrap_or_default();
        // CE-22: 日付は設定の形で見せる(セルの編集と同じ)。
        let text = match types::parse_date(raw.trim()).filter(|_| entry == Entry::Date) {
            Some(d) => calendar::input_date(&self.date_format, d),
            None => raw.clone(),
        };
        let cal = Cal::open(entry, &raw, &text, self.today);
        // CE-3: テキストの列は、セルの編集と同じ候補のリスト(list.rs)。今の値は前もって入れる値か前の答え。
        // 候補が0件(どのノートにもまだ無い列)なら「なし」だけのリストにせず1行の入力。
        let list = match kind {
            Kind::Text => self
                .src
                .candidates(&col, self.candidates)
                .filter(|c| {
                    c.iter()
                        .filter_map(to_new)
                        .any(|v| !matches!(v, NewValue::Null))
                })
                .map(|c| List::new(&c, preset.as_ref(), true)),
            _ => None,
        };
        self.open_note_input(entry, text, cal);
        if let Some(i) = &mut self.input {
            i.list = list;
        }
    }

    /// 編集のモードの動作のうち、新しいノートの入力で受けるもの。扱ったら true。
    /// 確定(Enter・Tab)で次へ、Shift+Tab で前の項目へ、Esc で取りやめ。日付の「空にして確定」は空の答え。ほかは入力ボックスのまま。
    pub(crate) fn new_note_input_action(&mut self, action: Action) -> bool {
        if self.note.flow.is_none() {
            return false;
        }
        // CE-25: 作る場所を選ぶ欄は ↑↓・確定・取りやめ・前の項目だけを受け、文字の編集とカーソルの操作は
        // 捨てる(自由入力に切り替えない。打った文字が確定で捨てられないように)。
        if self.note_choosing() {
            match action {
                Action::Commit | Action::CommitNext => self.note_commit(),
                Action::CommitPrev => self.note_back(),
                Action::Cancel => self.cancel_note(),
                Action::ListUp => self.list_move(false),
                Action::ListDown => self.list_move(true),
                _ => {}
            }
            return true;
        }
        match action {
            Action::Commit | Action::CommitNext => self.note_commit(),
            Action::CommitPrev => self.note_back(),
            Action::Cancel => self.cancel_note(),
            Action::Clear if self.input.as_ref().is_some_and(|i| i.cal.is_some()) => {
                if let Some(i) = &mut self.input {
                    i.text.clear();
                    i.cursor = 0;
                }
                self.note_commit();
            }
            _ => return false,
        }
        true
    }

    /// リストの選択の動作のうち、新しいノートの聞く項目で受けるもの(確定・取りやめ)。扱ったら true。
    pub(crate) fn new_note_pick_action(&mut self, action: Action) -> bool {
        if self.note.flow.is_none() {
            return false;
        }
        let searching = self.pick.as_ref().is_some_and(|p| p.searching());
        match action {
            Action::Cancel => self.cancel_note(),
            Action::Commit => self.note_pick_commit(),
            Action::Run if !searching => self.note_pick_commit(),
            _ => return false,
        }
        true
    }

    fn note_pick_commit(&mut self) {
        let Some(p) = self.pick.take() else {
            return;
        };
        let items = p.result(&p.targets[0].1);
        let value = if items.is_empty() {
            NewValue::Null
        } else {
            NewValue::List(items)
        };
        if !self.answer(value) {
            self.pick = Some(p);
        }
    }

    /// 名前か聞く項目の入力の確定。読めない値・作れない名前なら、理由を出して欄は開いたまま。
    fn note_commit(&mut self) {
        let Some(i) = self.input.take() else {
            return;
        };
        let Some(f) = &self.note.flow else {
            return;
        };
        if f.choosing {
            let sel = i.list.as_ref().map_or(0, |l| l.sel);
            return self.place_commit(sel);
        }
        if f.step == 0 {
            // 名前は、ここで場所と名前を確かめる(聞き終えてから断らない)。
            if let Err(e) = newnote::note_path(&f.root, &f.rule.folder, &i.text) {
                self.message = Some(e);
                self.input = Some(i);
                return;
            }
            let asks = !f.rule.ask.is_empty();
            let name = i.text.clone();
            if let Some(f) = &mut self.note.flow {
                f.name = name;
            }
            if !asks {
                if !self.finish_note() {
                    self.input = Some(i);
                }
                return;
            }
            return self.ask_next();
        }
        // CE-3: 候補のリストが出ていれば選んだ値(「なし」は空の答え)。
        // CE-20・CE-21: カレンダーで選んだ日はその日。ほかは入力の文字を列の型で読む(CE-2)。
        let picked = i.list.as_ref().filter(|l| !l.free).map(|l| l.chosen());
        let value = match picked
            .map(Choice::Value)
            .or_else(|| calendar::choice(&i, self.today, &self.date_format))
        {
            Some(Choice::Value(v)) => v,
            Some(Choice::Day(d)) => NewValue::Date(types::format_date(d)),
            None => match parse(i.entry, &i.text, self.today, &self.date_format) {
                Ok(v) => v,
                Err(e) => {
                    self.message = Some(e);
                    self.input = Some(i);
                    return;
                }
            },
        };
        if !self.answer(value) {
            self.input = Some(i);
        }
    }

    /// 聞いた値を足して次へ(最後なら作る)。作れなければ値を戻して false(欄は開いたまま)。
    fn answer(&mut self, value: NewValue) -> bool {
        let Some(col) = self
            .note
            .flow
            .as_ref()
            .and_then(|f| f.col().map(String::from))
        else {
            return false;
        };
        // WB-18: 型の決まらない列(どのノートにも値が無い列など)の日付・日時の形だけの答えは囲まずに書く。
        let value = date_if_untyped(self.src.as_ref(), &col, value);
        let Some(f) = &mut self.note.flow else {
            return false;
        };
        f.answers.push(Edit { key: col, value });
        if f.step < f.rule.ask.len() {
            self.ask_next();
            return true;
        }
        if self.finish_note() {
            return true;
        }
        if let Some(f) = &mut self.note.flow {
            f.answers.pop();
        }
        false
    }

    /// 作る(CE-25・CE-26)。作れたら表に戻ってその行を選び true。作れなければ理由を出して false。
    fn finish_note(&mut self) -> bool {
        let Some(f) = &self.note.flow else {
            return false;
        };
        let made = newnote::note_path(&f.root, &f.rule.folder, &f.name).and_then(|path| {
            let bytes = newnote::build(&f.rule, &f.prefill, &f.answers)?;
            newnote::create(&path, &bytes)
                .map_err(|e| Msg::NoteCannotCreate.fill(&[&path.display(), &e]))?;
            Ok(path)
        });
        let path = match made {
            Ok(p) => p,
            Err(e) => {
                self.message = Some(e);
                return false;
            }
        };
        self.note.flow = None;
        self.input = None;
        self.pick = None;
        self.set_mode(Mode::Table);
        self.select_new_note(&path);
        true
    }

    /// 作ったノートを読み直して表に入れ、その行を選ぶ。見えない(絞り込み・畳んだまとまり)なら知らせる。
    fn select_new_note(&mut self, path: &std::path::Path) {
        let real = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let id = RowId(real.to_string_lossy().into_owned());
        if let Err(e) = self.src.reload(&id) {
            self.message = Some(Msg::NoteUnreadable.fill(&[&e]));
            return;
        }
        // 移動の操作なので、留めていた行は本来の位置へ(NV-12)。
        self.stay.clear();
        self.refresh();
        let label = sanitize(&self.src.label(&id));
        let at = self
            .rows
            .iter()
            .position(|r| *r == id)
            .and_then(|r| self.slots.iter().position(|s| *s == Slot::Row(r)));
        match at {
            Some(i) => {
                self.row = i;
                self.scroll_into_view();
                self.message = Some(Msg::NoteCreated.fill(&[&label]));
            }
            None => {
                self.message = Some(Msg::NoteCreatedHidden.fill(&[&label]));
            }
        }
    }

    /// Esc: どこからでも取りやめる(何も作らない)。
    fn cancel_note(&mut self) {
        self.note.flow = None;
        self.input = None;
        self.pick = None;
        self.set_mode(Mode::Table);
        self.message = Some(Msg::NoteCancelled.text().into());
    }
}

// ---- 描画(純関数) ----

/// 欄の行(入力の行。その下に案内の行)。表の下の端(下の帯の上の2行)。低い端末では None。
fn panel_y(app: &App) -> Option<usize> {
    let h = app.size.1.saturating_sub(1) as usize;
    let limit = h.saturating_sub(2);
    (limit >= 4).then(|| limit - 2)
}

/// 入力の前置き。
fn lead(f: &Flow) -> String {
    if f.choosing {
        return Msg::NoteLeadPlace.text().to_string();
    }
    match f.col() {
        None => Msg::NoteLeadName.text().to_string(),
        Some(c) => Msg::NoteLeadCol.fill(&[&sanitize(c)]),
    }
}

/// 新しいノートの入力の欄(入力ボックスの位置。カーソル・カレンダー・リストの選択の窓の置き場)。
pub(crate) fn note_box(app: &App, w: usize) -> Option<InputBox> {
    let f = app.note.flow.as_ref()?;
    let y = panel_y(app)?;
    let x = width(&lead(f));
    if x + 2 > w {
        return None;
    }
    let room = w - x;
    let (text, cx) = super::detail::input_view(app, room).unwrap_or_default();
    Some(InputBox {
        x,
        y,
        w: room,
        text,
        cursor_x: x + cx,
    })
}

/// 下の帯の「今押せるキー」(新しいノートの入力のとき。キーの表から。SR-4)。そうでなければ None。
pub(crate) fn hints(app: &App) -> Option<Vec<String>> {
    let f = app.note.flow.as_ref()?;
    let key = |m: Mode, a: Action| keymap::key_for(&app.keys, m, a);
    let last = f.step >= f.rule.ask.len();
    let next = if f.choosing {
        Msg::KeyDecide.text()
    } else if last {
        Msg::NoteCreate.text()
    } else {
        Msg::NoteNext.text()
    };
    let mut out = Vec::new();
    let mut push = |k: Option<String>, label: &str| {
        if let Some(k) = k {
            out.push(format!("{k} {label}"));
        }
    };
    if app.mode == Mode::ListPick {
        push(key(Mode::ListPick, Action::Toggle), Msg::NoteToggle.text());
        push(key(Mode::ListPick, Action::Run), next);
        push(key(Mode::ListPick, Action::Cancel), Msg::NoteStop.text());
    } else {
        push(key(Mode::Edit, Action::Commit), next);
        if f.step > 0 || (!f.choosing && f.places.len() > 1) {
            push(
                key(Mode::Edit, Action::CommitPrev),
                Msg::NotePrevField.text(),
            );
        }
        push(key(Mode::Edit, Action::Cancel), Msg::NoteStop.text());
    }
    Some(out)
}

/// メッセージ行の案内(新しいノートの入力のとき): 聞く項目の入り方。テキストの列で候補のリスト(CE-3)が
/// あれば、セルの編集と同じリストの案内に「なし = 書かない」を添える(自由入力のときは「↑↓ でリストに戻る」)。
/// 名前の欄・リストの無いテキストでは出さない(表のセルの案内も出さない)。新しいノートの入力でなければ None。
pub(crate) fn hint(app: &App) -> Option<Option<String>> {
    let f = app.note.flow.as_ref()?;
    if app.mode == Mode::Edit && f.choosing {
        return Some(Some(Msg::NoteHintPlace.text().to_string()));
    }
    if app.mode != Mode::Edit || f.step == 0 {
        return Some(None);
    }
    if let Some(l) = app.input.as_ref().and_then(|i| i.list.as_ref()) {
        let t = if l.free {
            Msg::NoteHintFree.text().to_string()
        } else {
            Msg::NoteHintList.text().to_string()
        };
        return Some(Some(t));
    }
    let forms = super::entry::date_forms(&app.date_format);
    let t = match app.input.as_ref().map(|i| i.entry) {
        Some(Entry::Number) => Msg::NoteHintNumber.text().to_string(),
        Some(Entry::Checkbox) => Msg::NoteHintCheckbox.text().to_string(),
        Some(Entry::Date) => Msg::NoteHintDate.fill(&[&forms]),
        Some(Entry::DateTime) => Msg::NoteHintDateTime.fill(&[&forms]),
        _ => return Some(None),
    };
    Some(Some(t))
}

/// 案内の行: 作る場所と名前、何番目か。作る場所を選んでいる間は、選んでいるフォルダの実際のパス。
fn guide(app: &App, f: &Flow) -> String {
    if f.choosing {
        let sel = app
            .input
            .as_ref()
            .and_then(|i| i.list.as_ref())
            .map_or(0, |l| l.sel);
        let path = f
            .places
            .get(sel)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        return sanitize(&Msg::NoteGuidePlace.fill(&[&(sel + 1), &f.places.len(), &path]));
    }
    let total = f.rule.ask.len();
    // フォルダの名前を先に出す(長いパスは幅で切れるので後ろに)。
    let mut place = f
        .root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| f.root.display().to_string());
    if !f.rule.folder.trim().is_empty() {
        place.push('/');
        place.push_str(f.rule.folder.trim().trim_matches('/'));
    }
    let what = if f.step == 0 {
        Msg::NoteGuideWhere.fill(&[&place, &f.root.display()])
    } else {
        let name = f.name.trim();
        Msg::NoteGuideField.fill(&[&name, &f.step.min(total), &total])
    };
    sanitize(&format!("   {what}"))
}

/// 表の下の端に欄を重ねる(CE-25)。カレンダーとリストの選択の窓はこの上に重なる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(f) = app.note.flow.as_ref() else {
        return;
    };
    let Some(y) = panel_y(app) else {
        return;
    };
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let head = lead(f);
    let mut spans = vec![Span::styled(head.clone(), bold)];
    if let Some(b) = note_box(app, w) {
        let st = Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        spans.push(Span::styled(fit(&b.text, b.w, Align::Left), st));
    }
    if let Some(line) = lines.get_mut(y) {
        *line = super::view::pad(spans, w, Style::default());
    }
    if let Some(line) = lines.get_mut(y + 1) {
        *line = Line::from(fit(&guide(app, f), w, Align::Left));
    }
}

/// 開いたフォルダの一覧(作る場所の根の候補。CE-25)。`.base` で開いたときは、その `.base` のフォルダ1つ
/// (起動の対象 `target` のフォルダ)。フォルダで開いたときは、読み込み口が返す開いたフォルダ(`Source::folders`。
/// 起動の引数の順で、同じフォルダは1つ)。対象の文字(複数なら改行つなぎ)は分けない(改行を含む名前で壊れず、
/// 開いていないフォルダを候補にしないため)。読み込み口がフォルダを返さず対象が1つのパスなら、そのフォルダ1つ。
/// フォルダでないものは除き、相対パスで開いても実際のパスにする。
pub(crate) fn places(src: &dyn Source, target: &Path, base: bool) -> Vec<PathBuf> {
    let one = || {
        super::native_io::export_dir_of(target)
            .filter(|d| d.is_dir())
            .map(|d| d.canonicalize().unwrap_or(d))
            .into_iter()
            .collect::<Vec<_>>()
    };
    if base {
        return one();
    }
    let folders: Vec<PathBuf> = src.folders().into_iter().filter(|d| d.is_dir()).collect();
    if folders.is_empty() && !target.to_string_lossy().contains('\n') {
        return one();
    }
    folders
}

/// 一覧の1行: フォルダの名前(最後の部分)。
fn place_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

/// ヘッダーにボタンを出すか(読むだけ(WB-15)と、作る場所(開いたフォルダ)が無いときは出さない)。
pub(crate) fn button_shown(app: &App) -> bool {
    !app.readonly && app.note.places.iter().any(|p| p.is_dir())
}

/// (x, y) がヘッダーの「+ 新規」か(CE-25)。
pub(crate) fn button_at(app: &App, x: u16, y: u16) -> bool {
    let w = app.size.0.saturating_sub(1) as usize;
    let bw = width(button());
    y == 0 && button_shown(app) && w >= bw + 2 && (w - bw..w).contains(&(x as usize))
}

/// 保管庫の根(`.obsidian` のある最寄りの上。BV-2)から `dir` のフォルダが、作る場所の根 `root` の中なら、
/// `root` からの相対(`/` でつなぐ)。外か、根が決まらなければ None。
fn folder_under_root(root: &Path, dir: &str) -> Option<String> {
    let real = root.canonicalize().ok()?;
    let vault = mdgrid::vault::roots(std::slice::from_ref(&real))
        .ok()?
        .into_iter()
        .next()?;
    let rel = vault.join(dir).strip_prefix(&real).ok()?.to_path_buf();
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}
