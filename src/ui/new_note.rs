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
    /// 0 は名前、k は `fields[k - 1]`。
    pub(crate) step: usize,
    /// CE-26: 窓の欄の列(設定の ask があればその列、無ければ表で見えている列。hidden は除く)。
    fields: Vec<String>,
    /// 欄ごとの答え(空は Null。書かない)。同じ列は置き換える。
    answers: Vec<Edit>,
    /// CE-26: 今の欄を確かめたら、次の欄へ進まずに作る(Ctrl+S)。
    finish: bool,
    /// CE-26: 作れなかった理由と、その欄(0 は名前)。
    error: Option<(usize, String)>,
    /// CE-33: 作ったらエディタで開く(mode = "editor" か Ctrl+E)。
    open_editor: bool,
}

impl Flow {
    /// 今聞いている列(名前の間は None)。
    fn col(&self) -> Option<&str> {
        self.step
            .checked_sub(1)
            .and_then(|k| self.fields.get(k))
            .map(String::as_str)
    }

    /// 欄の答え。
    fn answer_of(&self, col: &str) -> Option<&NewValue> {
        self.answers.iter().find(|e| e.key == col).map(|e| &e.value)
    }

    /// 前もって入れる値の出どころ(絞り込みか設定か)。
    fn preset_from_filter(&self, col: &str) -> bool {
        self.prefill.iter().any(|e| e.key == col)
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

/// CE-32: 雛形を埋めた値が日付・日時の形なら、日付として(囲まずに)書く: 型の決まらない列(WB-18)と、
/// 日付・日時の列。
fn as_date(src: &dyn Source, col: &str, v: NewValue) -> NewValue {
    match v {
        NewValue::Str(s)
            if types::date_or_datetime(&s)
                && matches!(src.kind(col).kind, Kind::Date | Kind::DateTime) =>
        {
            NewValue::Date(s)
        }
        v => date_if_untyped(src, col, v),
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
        // CE-26: 窓の欄は、設定の ask があればその列、無ければ表で見えている列(書けない列と hidden は除く)。
        let source: Vec<String> = if rule.ask.is_empty() {
            self.cols.clone()
        } else {
            rule.ask.clone()
        };
        // CE-33: エディタで作るときは名前だけを聞く。
        let fields: Vec<String> = source
            .into_iter()
            .filter(|c| !newnote::not_a_key(c) && !rule.hidden.contains(c))
            .filter(|_| !rule.editor())
            .collect();
        let open_editor = rule.editor();
        self.note.flow = Some(Flow {
            root,
            places,
            choosing,
            rule,
            prefill,
            name: String::new(),
            step: 0,
            fields,
            answers: Vec::new(),
            finish: false,
            error: None,
            open_editor,
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
            return self.open_note_input(Entry::Text, name, None);
        }
        self.open_step(None);
    }

    /// 今の聞く項目の欄を開く。`value` があればその値で(戻ったとき)、無ければ前もって入れる値で始める。
    fn open_step(&mut self, value: Option<NewValue>) {
        let Some(f) = &self.note.flow else {
            return;
        };
        let Some(col) = f.col().map(String::from) else {
            return;
        };
        // 答えた欄はその答えで、まだなら前もって入れる値(変数を埋めて)で始める。
        let vars = self.note_vars(f);
        let preset = value
            .or_else(|| f.answer_of(&col).cloned())
            .or_else(|| f.preset(&col).map(|v| newnote::expand_value(v, &vars)));
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
            Action::CreateNote => self.note_create_now(),
            Action::CreateNoteEdit => self.note_create_and_edit(),
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
            Action::CreateNote => self.note_create_now(),
            Action::CreateNoteEdit => self.note_create_and_edit(),
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
            let done = f.finish || f.fields.is_empty();
            let name = i.text.clone();
            if let Some(f) = &mut self.note.flow {
                f.name = name;
            }
            if done {
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
        f.answers.retain(|e| e.key != col);
        f.answers.push(Edit { key: col, value });
        if !f.finish && f.step < f.fields.len() {
            self.ask_next();
            return true;
        }
        self.finish_note()
    }

    /// 描画から使う雛形の変数(CE-32)。
    pub(crate) fn note_vars_for(&self, f: &Flow) -> newnote::Vars {
        self.note_vars(f)
    }

    /// CE-32: 雛形の変数(今日・今の時刻・決めた名前・作る場所)。
    fn note_vars(&self, f: &Flow) -> newnote::Vars {
        let name = f.name.trim();
        let name = name.strip_suffix(".md").unwrap_or(name);
        newnote::Vars {
            today: self.today,
            // now は地域の時計の秒(today_now。時差は足し済み)。
            minutes: self.now.rem_euclid(86_400) / 60,
            name: name.rsplit('/').next().unwrap_or(name).to_string(),
            folder: f.rule.folder.trim().trim_matches('/').to_string(),
        }
    }

    /// CE-33: Ctrl+E。Ctrl+S と同じく作り、作れたらエディタで開く。
    fn note_create_and_edit(&mut self) {
        let before = match &mut self.note.flow {
            Some(f) => std::mem::replace(&mut f.open_editor, true),
            None => return,
        };
        self.note_create_now();
        if let Some(f) = &mut self.note.flow {
            f.open_editor = before;
        }
    }

    /// CE-26: Ctrl+S。今の欄を確かめて(読めなければ理由を出して欄は開いたまま)、次の欄へ進まずに作る。
    fn note_create_now(&mut self) {
        let Some(f) = &mut self.note.flow else {
            return;
        };
        if f.choosing {
            return self.note_commit();
        }
        f.finish = true;
        if self.mode == Mode::ListPick {
            self.note_pick_commit();
        } else {
            self.note_commit();
        }
        if let Some(f) = &mut self.note.flow {
            f.finish = false;
        }
    }

    /// 作る(CE-25・CE-26)。作れたら表に戻ってその行を選び true。作れなければ理由を出して false。
    /// 作れたら、または作れない理由の欄へ移したら true。今の欄で作れない(理由はその欄に出す)なら false。
    fn finish_note(&mut self) -> bool {
        let Some(f) = &self.note.flow else {
            return false;
        };
        let cur = f.step;
        // CE-27: 必須の欄が空なら作らずにその欄へ。名前が作れないなら名前の欄へ。
        let empty = |v: Option<&NewValue>| match v {
            None | Some(NewValue::Null) => true,
            Some(NewValue::Str(s)) => s.trim().is_empty(),
            Some(NewValue::List(items)) => items.is_empty(),
            _ => false,
        };
        let missing = f.rule.required.iter().find(|c| {
            let v = f.answer_of(c).or_else(|| f.preset(c));
            empty(v)
        });
        let problem = match newnote::note_path(&f.root, &f.rule.folder, &f.name) {
            Err(e) => Some((0, e)),
            Ok(_) => missing.map(|c| {
                let at = f.fields.iter().position(|x| x == c).map_or(0, |k| k + 1);
                (at, Msg::NoteRequiredEmpty.text().to_string())
            }),
        };
        if let Some((at, e)) = problem {
            return self.note_problem(cur, at, e);
        }
        let vars = self.note_vars(f);
        let src = self.src.as_ref();
        let mut rule = f.rule.clone();
        for (k, v) in &mut rule.set {
            *v = as_date(src, k, newnote::expand_value(v, &vars));
        }
        let prefill: Vec<Edit> = f
            .prefill
            .iter()
            .map(|e| Edit {
                key: e.key.clone(),
                value: as_date(src, &e.key, newnote::expand_value(&e.value, &vars)),
            })
            .collect();
        let body = if rule.body.trim().is_empty() {
            None
        } else {
            let p = f.root.join(rule.body.trim());
            match std::fs::read_to_string(&p) {
                Ok(t) => Some(t),
                Err(e) => {
                    let msg = Msg::NoteBodyUnreadable.fill(&[&p.display(), &e]);
                    return self.note_problem(cur, cur, msg);
                }
            }
        };
        let made = newnote::note_path(&f.root, &f.rule.folder, &f.name).and_then(|path| {
            let bytes = newnote::build_with(&rule, &prefill, &f.answers, &vars, body.as_deref())?;
            newnote::create(&path, &bytes)
                .map_err(|e| Msg::NoteCannotCreate.fill(&[&path.display(), &e]))?;
            Ok(path)
        });
        let path = match made {
            Ok(p) => p,
            Err(e) => return self.note_problem(cur, 0, e),
        };
        let edit = f.open_editor;
        self.note.flow = None;
        self.input = None;
        self.pick = None;
        self.set_mode(Mode::Table);
        self.select_new_note(&path);
        // CE-33: `e` と同じ道でエディタで開く(端末を持つ main が開き、戻ったら読み直す)。
        if edit {
            let real = path.canonicalize().unwrap_or(path);
            self.editor_request = Some(RowId(real.to_string_lossy().into_owned()));
        }
        true
    }

    /// CE-26: 作れない理由を `at` の欄に出す。今の欄なら false(呼び手が欄を開いたままにする)、
    /// ほかの欄ならその欄を開いて true。
    fn note_problem(&mut self, cur: usize, at: usize, e: String) -> bool {
        self.message = Some(e.clone());
        let Some(f) = &mut self.note.flow else {
            return false;
        };
        f.error = Some((at, e));
        if at == cur {
            return false;
        }
        f.step = at;
        if at == 0 {
            let name = f.name.clone();
            self.pick = None;
            self.open_note_input(Entry::Text, name, None);
        } else {
            self.open_step(None);
        }
        true
    }

    /// 作ったノートを読み直して表に入れ、その行を選ぶ。見えない(絞り込み・畳んだまとまり)なら知らせる。
    fn select_new_note(&mut self, path: &std::path::Path) {
        let real = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let id = RowId(real.to_string_lossy().into_owned());
        self.data_gen += 1;
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

/// 窓を出せる高さがあるか(窓の上の端の行。表の見出しの行から)。低い端末では None。
fn panel_y(app: &App) -> Option<usize> {
    let h = app.size.1.saturating_sub(1) as usize;
    let limit = h.saturating_sub(2);
    let top = super::view::data_y(app).saturating_sub(1);
    (limit >= top + 4).then_some(top)
}

/// 窓の欄(CE-26)。`None` は作る場所、`Some(0)` は名前、`Some(k)` は `fields[k - 1]`。
fn rows(f: &Flow) -> Vec<Option<usize>> {
    let mut out = Vec::new();
    if f.places.len() > 1 {
        out.push(None);
    }
    out.extend((0..=f.fields.len()).map(Some));
    out
}

/// 欄の見出し(必須なら ` *`)。
fn label(f: &Flow, row: Option<usize>) -> String {
    match row {
        None => Msg::NoteFieldPlace.text().to_string(),
        Some(0) => format!("{} *", Msg::NoteFieldName.text()),
        Some(k) => {
            let c = &f.fields[k - 1];
            if f.rule.required.contains(c) {
                format!("{} *", sanitize(c))
            } else {
                sanitize(c)
            }
        }
    }
}

/// 今の欄(行の並びでの位置)。
fn current(f: &Flow) -> usize {
    let off = usize::from(f.places.len() > 1);
    if f.choosing {
        0
    } else {
        f.step + off
    }
}

/// 窓の形: 上の縁の行・左の桁・幅・見出しの幅・出す欄の最初と数(SR-31: 枠のある窓)。
struct Geom {
    top: usize,
    left: usize,
    /// 窓の幅(縁を含む)。
    bw: usize,
    label_w: usize,
    first: usize,
    count: usize,
}

/// 窓の縁と区切りと縦の線を除いた行の数(上の縁・区切り・キーの行・下の縁)。
const BOX_FRAME: usize = 4;

fn geom(app: &App) -> Option<Geom> {
    let f = app.note.flow.as_ref()?;
    let top = panel_y(app)?;
    let h = app.size.1.saturating_sub(1) as usize;
    let w = app.size.0.saturating_sub(1) as usize;
    let limit = h.saturating_sub(2);
    let rs = rows(f);
    // 縁と区切りとキーの行を除いた欄の数。入らなければ今の欄の周りを出す。
    let room = limit.saturating_sub(top + BOX_FRAME).max(1);
    let count = rs.len().min(room);
    let cur = current(f);
    let first = cur.saturating_sub(count - 1).min(rs.len() - count);
    let label_w = rs.iter().map(|r| width(&label(f, *r))).max().unwrap_or(4);
    // 幅: 見出し + 40 桁(値の欄)を目安に、画面の左右に2桁ずつ空ける。真ん中に置く。
    let bw = (label_w + 46)
        .max(48)
        .min(w.saturating_sub(4))
        .max(24.min(w));
    let left = (w - bw) / 2;
    Some(Geom {
        top,
        left,
        bw,
        label_w,
        first,
        count,
    })
}

/// 欄の行の値の欄の左の桁(縦の線・空白・印2桁・見出し・空白2桁の後ろ)。
fn value_x(g: &Geom) -> usize {
    g.left + 1 + 1 + 2 + g.label_w + 2
}

/// 新しいノートの入力の欄(入力ボックスの位置。今の欄の行。カーソル・カレンダー・リストの選択の窓の置き場)。
pub(crate) fn note_box(app: &App, w: usize) -> Option<InputBox> {
    let f = app.note.flow.as_ref()?;
    let g = geom(app)?;
    let y = g.top + 1 + current(f) - g.first;
    let x = value_x(&g);
    let right = (g.left + g.bw).min(w).saturating_sub(2);
    if x + 2 > right {
        return None;
    }
    let room = right - x;
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
    let last = f.step >= f.fields.len();
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
        if !last {
            push(
                key(Mode::ListPick, Action::CreateNote),
                Msg::NoteCreate.text(),
            );
        }
        push(key(Mode::ListPick, Action::Cancel), Msg::NoteStop.text());
    } else {
        push(key(Mode::Edit, Action::Commit), next);
        if !last && !f.choosing {
            push(key(Mode::Edit, Action::CreateNote), Msg::NoteCreate.text());
        }
        if f.step > 0 || (!f.choosing && f.places.len() > 1) {
            push(
                key(Mode::Edit, Action::CommitPrev),
                Msg::NotePrevField.text(),
            );
        }
        push(key(Mode::Edit, Action::Cancel), Msg::NoteStop.text());
        // CE-33: 作ってエディタで開く(帯が狭いので最後に短く)。
        if !f.choosing && !f.rule.editor() {
            push(
                key(Mode::Edit, Action::CreateNoteEdit),
                Msg::NoteEditShort.text(),
            );
        }
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

/// 窓の題(上の縁に出す): 「新しいノート · <フォルダ> に作る」。作る場所を選んでいる間は選んでいるフォルダ。
fn title(app: &App, f: &Flow) -> String {
    let name = |p: &Path| {
        p.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.display().to_string())
    };
    let place = if f.choosing {
        let sel = app
            .input
            .as_ref()
            .and_then(|i| i.list.as_ref())
            .map_or(0, |l| l.sel);
        f.places.get(sel).map(|p| name(p)).unwrap_or_default()
    } else {
        let mut place = name(&f.root);
        if !f.rule.folder.trim().is_empty() {
            place.push('/');
            place.push_str(f.rule.folder.trim().trim_matches('/'));
        }
        place
    };
    sanitize(&format!(
        "{} · {}",
        Msg::NoteFormTitle.text().trim(),
        Msg::NoteFormWhere.fill(&[&place])
    ))
}

/// 今の欄でない欄の値の見せ方: 答え、無ければ前もって入れる値と出どころ、それも無ければ型の手がかり(SR-31。
/// 2つ目が true なら手がかりで、薄く出す)。
fn value_text(app: &App, f: &Flow, row: Option<usize>) -> (String, bool) {
    let Some(k) = row else {
        let name = f
            .root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return (name, false);
    };
    if k == 0 {
        return (f.name.clone(), false);
    }
    let c = &f.fields[k - 1];
    if let Some(v) = f.answer_of(c) {
        return (new_value_text(v), false);
    }
    match f.preset(c) {
        Some(v) => {
            let vars = app.note_vars_for(f);
            let from = if f.preset_from_filter(c) {
                Msg::NoteFromFilter.text()
            } else {
                Msg::NoteFromConfig.text()
            };
            let text = format!(
                "{}  {from}",
                new_value_text(&newnote::expand_value(v, &vars))
            );
            (text, false)
        }
        None => (type_hint(app, c), true),
    }
}

/// 型の手がかり(SR-31)。
fn type_hint(app: &App, col: &str) -> String {
    match app.src.kind(col).kind {
        Kind::Date => "YYYY-MM-DD".to_string(),
        Kind::DateTime => "YYYY-MM-DDTHH:MM".to_string(),
        Kind::Number => Msg::NoteHintKindNumber.text().to_string(),
        Kind::List => Msg::NoteHintKindList.text().to_string(),
        Kind::Checkbox => "[ ]".to_string(),
        Kind::Text => String::new(),
    }
}

/// 枠の線の文字(SR-31): (左上・右上・左下・右下・横・縦・左の区切り・右の区切り)。あいまいな幅を2と数える
/// 設定(CV-6)では ASCII にする(罫線は東アジアのあいまいな幅の文字)。
fn box_chars(app: &App) -> [char; 8] {
    // SR-32: 枠の文字は窓どうしでそろえる(popup::frame)。
    let f = super::popup::frame(app);
    [f.tl, f.tr, f.bl, f.br, f.h, f.v, f.lt, f.rt]
}

/// 窓の行を、表の行(`line`)の桁 x から、span を並べて重ねる。
fn put(lines: &mut [Line<'static>], y: usize, x: usize, spans: Vec<Span<'static>>, w: usize) {
    let Some(line) = lines.get_mut(y) else {
        return;
    };
    let mut at = x;
    for sp in spans {
        let sw = width(&sp.content);
        if sw == 0 {
            continue;
        }
        *line = super::list::splice(line, at, sp, sw, w);
        at += sw;
    }
}

/// 表の上に窓を重ねる(CE-26・SR-31)。カレンダーとリストの選択の窓はこの上に重なる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(f) = app.note.flow.as_ref() else {
        return;
    };
    let Some(g) = geom(app) else {
        return;
    };
    let [tl, tr, bl, br, hz, vt, ml, mr] = box_chars(app);
    let inner = g.bw - 2;
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let dim = Style::default().add_modifier(Modifier::DIM);
    // SR-33: モダンな見た目では、縁はアクセントの色、今の欄の印もアクセントの色。
    let look = super::look::look(app);
    let bst = look.as_ref().map(|l| l.border()).unwrap_or_default();
    let mark_st = look.as_ref().map(|l| l.key()).unwrap_or(bold);
    let edge = |l: char, r: char, text: &str| {
        let t = super::width::take(text, inner);
        format!("{l}{t}{}{r}", hz.to_string().repeat(inner - width(&t)))
    };
    // 上の縁と題。
    let head = format!("{hz} {} ", title(app, f));
    put(
        lines,
        g.top,
        g.left,
        vec![Span::styled(
            edge(tl, tr, &head),
            bst.add_modifier(Modifier::BOLD),
        )],
        w,
    );
    let rs = rows(f);
    let cur = current(f);
    let editing = app.mode == Mode::Edit;
    let vx = value_x(&g);
    for (k, row) in rs.iter().enumerate().skip(g.first).take(g.count) {
        let y = g.top + 1 + k - g.first;
        let here = k == cur;
        let mut spans = vec![
            Span::styled(format!("{vt} "), bst),
            Span::styled(if here { "> " } else { "  " }, mark_st),
            Span::styled(fit(&label(f, *row), g.label_w, Align::Left), bold),
            Span::raw("  "),
        ];
        let room = (g.left + g.bw).saturating_sub(vx + 1);
        if here && editing {
            if let Some(b) = note_box(app, w) {
                // SR-31: 入力欄は反転で欄と分かるように。
                let st = super::view::input_style(app);
                spans.push(Span::styled(fit(&b.text, b.w, Align::Left), st));
                spans.push(Span::raw(" ".repeat(room.saturating_sub(b.w))));
            }
        } else {
            let (mut t, hint) = value_text(app, f, *row);
            let mut t2 = sanitize(&t);
            let step = row.unwrap_or(usize::MAX);
            if let Some((at, e)) = &f.error {
                if *at == step {
                    t2 = format!("{t2}  ! {}", sanitize(e));
                }
            }
            t = t2;
            spans.push(Span::styled(
                fit(&t, room, Align::Left),
                if hint { dim } else { Style::default() },
            ));
        }
        spans.push(Span::styled(vt.to_string(), bst));
        put(lines, y, g.left, spans, w);
    }
    // 区切りと、押せるキー(下の帯と同じ。キーの表から)と、下の縁。
    let y = g.top + 1 + g.count;
    put(
        lines,
        y,
        g.left,
        vec![Span::styled(edge(ml, mr, ""), bst)],
        w,
    );
    let keys = hints(app).unwrap_or_default().join("  ");
    let keys = fit(&format!(" {}", sanitize(&keys)), inner, Align::Left);
    put(
        lines,
        y + 1,
        g.left,
        vec![
            Span::styled(vt.to_string(), bst),
            Span::styled(keys, look.as_ref().map(|l| l.faint()).unwrap_or(dim)),
            Span::styled(vt.to_string(), bst),
        ],
        w,
    );
    put(
        lines,
        y + 2,
        g.left,
        vec![Span::styled(edge(bl, br, ""), bst)],
        w,
    );
    // 窓の左右に1桁ずつ空ける(表の文字が窓の縁に付いて見えないように)。
    for yy in g.top..=y + 2 {
        if g.left > 0 {
            put(lines, yy, g.left - 1, vec![Span::raw(" ")], w);
        }
        if g.left + g.bw < w {
            put(lines, yy, g.left + g.bw, vec![Span::raw(" ")], w);
        }
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
