//! vim モード(screen.vim-*。設定の `[keys]` の `vim`。`impl App` の続きと、1行の欄の直し方の純関数)。
//! 入力の欄(セルの編集・探す・検索・パレット・設定の値・絞り込みの窓の値。新しいノートの名前もセルの編集の欄)は
//! 挿入のモードで始まり、そのモードの抜けるキー(既定は Esc)でノーマルのモードに移る。ノーマルのモードのキーは
//! キーの表の `input_normal` のモード(`keymap::VIM_BINDINGS`)で先に引き、表に無い文字は入れない。
//! 表の画面には `dd`(ノートを消す)・`p`(写した値を入れる)・`.`(直前の値をもう一度)を足す。
//! 形と決めたことは docs/design.md の「vim モード」。

use super::app::App;
use super::input::{bulk_tail, new_value_text};
use super::keymap::{self, Action, Mode};
use super::startup::READONLY;
use super::width::{sanitize, width};
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::{self, Kind};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_segmentation::UnicodeSegmentation;

/// mdgrid の中で最後に写した値(screen.vim-paste)。OS のクリップボードは読まない。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Yank {
    /// `y` で写したセルの値。
    Cell(NewValue),
    /// `Y`(か、行を選んで `y`)で写した行の、ノートのキーごとの値(計算の列は除く)。
    Row(Vec<(String, NewValue)>),
}

/// vim モードの状態。
#[derive(Debug, Default)]
pub(crate) struct Vim {
    /// 設定の `[keys]` の `vim`(tbd~screen.vim-setting-name。既定は偽)。
    pub on: bool,
    /// 入力の欄がノーマルのモードか(偽は挿入のモード)。
    pub normal: bool,
    /// カーソルを持たない欄(探す・検索・パレット・設定の値・絞り込みの窓の値)のカーソル(文字のバイトの位置)。
    /// None は末尾(打った字はいつも末尾に入る、今までの欄と同じ)。
    pub cursor: Option<usize>,
    /// 入力の欄の中の取り消しの積み(文字とカーソル)。欄を開き直すと空にする。
    undo: Vec<(String, usize)>,
    /// 挿入のモードに入ったときの形。ノーマルのモードに戻るとき、変わっていれば取り消しの積みへ(挿入の間に
    /// 打った字を `u` 1回で戻す)。
    insert_start: Option<(String, usize)>,
    /// mdgrid の中で最後に写した値(`y`・`Y`。vim モードによらず覚える)。
    pub yank: Option<Yank>,
    /// 直前に1つのセルに入れた値(列, 値)。`.` で繰り返す(vim モードによらず覚える)。
    pub last_set: Option<(String, NewValue)>,
}

impl Vim {
    /// 入力の欄を開き直した: 挿入のモードから、取り消しの積みは空。
    pub(crate) fn reset_input(&mut self) {
        self.normal = false;
        self.cursor = None;
        self.undo.clear();
        self.insert_start = None;
    }
}

/// 端末のカーソルの形(screen.vim-mode-shown)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// 端末の既定のまま(vim モードの入力の欄の外)。
    Default,
    /// 挿入のモード(縦の棒)。
    Bar,
    /// ノーマルのモード(字を覆う四角)。
    Block,
}

/// vim モードの入力の欄になるモード。リストの選択(`list_select`)は1行の入力の欄でないので入れない。
pub(crate) fn field_mode(m: Mode) -> bool {
    matches!(
        m,
        Mode::Edit
            | Mode::Search
            | Mode::Filter
            | Mode::Palette
            | Mode::SettingsText
            | Mode::FilterText
    )
}

// ---- 1行の欄の直し方(純関数) ----

/// 文字の種類(語の切れ目。vim の `utf_class` と同じ分け方): 0 は空白、1 は記号、2 は英数字と `_`、
/// それより大きい値は文字の種類(ひらがな・カタカナ・漢字・ハングルなど。違う種類の間が語の切れ目)。
fn class(g: &str) -> u32 {
    let Some(c) = g.chars().next() else {
        return 0;
    };
    let u = c as u32;
    if c.is_whitespace() {
        return 0;
    }
    if u < 0x100 {
        return if c.is_ascii_alphanumeric() || c == '_' || u >= 0xc0 {
            2
        } else {
            1
        };
    }
    // vim の mbyte.c の表(範囲は重ならない形に直した)。表に無い文字は 2(語)。
    const CLASSES: &[(u32, u32, u32)] = &[
        (0x037e, 0x037e, 1),
        (0x0387, 0x0387, 1),
        (0x055a, 0x055f, 1),
        (0x0589, 0x0589, 1),
        (0x05be, 0x05be, 1),
        (0x05c0, 0x05c0, 1),
        (0x05c3, 0x05c3, 1),
        (0x05f3, 0x05f4, 1),
        (0x060c, 0x060c, 1),
        (0x061b, 0x061b, 1),
        (0x061f, 0x061f, 1),
        (0x066a, 0x066d, 1),
        (0x06d4, 0x06d4, 1),
        (0x0700, 0x070d, 1),
        (0x0964, 0x0965, 1),
        (0x0970, 0x0970, 1),
        (0x0df4, 0x0df4, 1),
        (0x0e4f, 0x0e4f, 1),
        (0x0e5a, 0x0e5b, 1),
        (0x0f04, 0x0f12, 1),
        (0x0f3a, 0x0f3d, 1),
        (0x0f85, 0x0f85, 1),
        (0x104a, 0x104f, 1),
        (0x10fb, 0x10fb, 1),
        (0x1361, 0x1368, 1),
        (0x166d, 0x166e, 1),
        (0x1680, 0x1680, 0),
        (0x169b, 0x169c, 1),
        (0x16eb, 0x16ed, 1),
        (0x1735, 0x1736, 1),
        (0x17d4, 0x17dc, 1),
        (0x1800, 0x180a, 1),
        (0x2000, 0x200b, 0),
        (0x200c, 0x2027, 1),
        (0x2028, 0x2029, 0),
        (0x202a, 0x202e, 1),
        (0x202f, 0x202f, 0),
        (0x2030, 0x205e, 1),
        (0x205f, 0x205f, 0),
        (0x2060, 0x206f, 1),
        (0x2070, 0x207f, 0x2070),
        (0x2080, 0x2094, 0x2080),
        (0x2095, 0x27ff, 1),
        (0x2800, 0x28ff, 0x2800),
        (0x2900, 0x2998, 1),
        (0x29d8, 0x29db, 1),
        (0x29fc, 0x29fd, 1),
        (0x2e00, 0x2e7f, 1),
        (0x3000, 0x3000, 0),
        (0x3001, 0x3020, 1),
        (0x3030, 0x3030, 1),
        (0x303d, 0x303d, 1),
        (0x3040, 0x309f, 0x3040),
        (0x30a0, 0x30ff, 0x30a0),
        (0x3300, 0x9fff, 0x4e00),
        (0xac00, 0xd7a3, 0xac00),
        (0xf900, 0xfaff, 0x4e00),
        (0xfd3e, 0xfd3f, 1),
        (0xfe30, 0xfe6b, 1),
        (0xff00, 0xff0f, 1),
        (0xff1a, 0xff20, 1),
        (0xff3b, 0xff40, 1),
        (0xff5b, 0xff65, 1),
        (0x1d000, 0x1d24f, 1),
        (0x1d400, 0x1d7ff, 1),
        (0x1f000, 0x1f2ff, 1),
        (0x1f300, 0x1f9ff, 1),
        (0x20000, 0x2a6df, 0x4e00),
        (0x2a700, 0x2b73f, 0x4e00),
        (0x2b740, 0x2b81f, 0x4e00),
        (0x2f800, 0x2fa1f, 0x4e00),
    ];
    CLASSES
        .iter()
        .find(|(a, b, _)| (*a..=*b).contains(&u))
        .map_or(2, |(_, _, k)| *k)
}

/// 書記素の頭のバイトの位置の並びと、末尾(`text.len()`)。
fn bounds(text: &str) -> Vec<usize> {
    let mut b: Vec<usize> = text.grapheme_indices(true).map(|(i, _)| i).collect();
    b.push(text.len());
    b
}

/// ノーマルのモードのカーソルの位置: 末尾(最後の字の後ろ)には置かず、最後の字の上にする。
pub(crate) fn clamp_normal(text: &str, at: usize) -> usize {
    if at >= text.len() {
        text.grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i)
    } else {
        at
    }
}

/// ノーマルのモードの操作の結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Edited {
    pub text: String,
    /// カーソル(バイトの位置)。
    pub cursor: usize,
    /// 挿入のモードに移る。
    pub insert: bool,
}

/// ノーマルのモードの動き・挿入への移り・消す・替えるを、1行の文字に当てる(screen.vim-normal-keys)。
/// `cursor` はバイトの位置。語の切れ目は vim の `w` と同じ(`class`)。取り消し(`u`)と、扱わない動作は None。
pub(crate) fn normal(action: Action, text: &str, cursor: usize) -> Option<Edited> {
    let b = bounds(text);
    let n = b.len() - 1;
    let c = b.iter().rposition(|&x| x <= cursor).unwrap_or(0).min(n);
    let cls = |i: usize| class(&text[b[i]..b[i + 1]]);
    let last = n.saturating_sub(1);
    // 次の語の頭(無ければ末尾 n)。
    let next_word = |i: usize| {
        let mut i = i;
        if i < n {
            let k = cls(i);
            if k != 0 {
                while i < n && cls(i) == k {
                    i += 1;
                }
            }
        }
        while i < n && cls(i) == 0 {
            i += 1;
        }
        i
    };
    // 語の頭(頭にいれば前の語の頭)。
    let prev_word = |i: usize| {
        if i == 0 {
            return 0;
        }
        let mut i = i - 1;
        while i > 0 && cls(i) == 0 {
            i -= 1;
        }
        let k = cls(i);
        while i > 0 && cls(i - 1) == k {
            i -= 1;
        }
        i
    };
    // 語の終わり(終わりにいれば次の語の終わり)。
    let word_end = |i: usize| {
        let mut i = i + 1;
        while i < n && cls(i) == 0 {
            i += 1;
        }
        if i >= n {
            return last;
        }
        let k = cls(i);
        while i + 1 < n && cls(i + 1) == k {
            i += 1;
        }
        i
    };
    let stay = |i: usize, insert: bool| Edited {
        text: text.to_string(),
        cursor: b[if insert { i.min(n) } else { i.min(last) }],
        insert,
    };
    // [from, to) の字を消す。カーソルは消した所(ノーマルのモードでは最後の字より後ろに置かない)。
    let cut = |from: usize, to: usize, insert: bool| {
        let mut t = text.to_string();
        t.replace_range(b[from]..b[to.max(from)], "");
        let at = b[from];
        Edited {
            cursor: if insert { at } else { clamp_normal(&t, at) },
            text: t,
            insert,
        }
    };
    Some(match action {
        Action::CursorLeft => stay(c.saturating_sub(1), false),
        Action::CursorRight => stay(c + 1, false),
        Action::CursorHome => stay(0, false),
        Action::CursorEnd => stay(last, false),
        Action::WordNext => stay(next_word(c), false),
        Action::WordPrev => stay(prev_word(c), false),
        Action::WordEnd => stay(word_end(c), false),
        Action::Insert => stay(c, true),
        Action::Append => stay((c + 1).min(n), true),
        Action::InsertStart => stay((0..n).find(|&i| cls(i) != 0).unwrap_or(n), true),
        Action::AppendEnd => stay(n, true),
        Action::DeleteForward if n > 0 => cut(c, c + 1, false),
        Action::DeleteBack if c > 0 => cut(c - 1, c, false),
        Action::DeleteForward | Action::DeleteBack => stay(c, false),
        Action::DeleteWord => cut(c, next_word(c), false),
        Action::DeleteWordBack => cut(prev_word(c), c, false),
        Action::DeleteToEnd => cut(c, n, false),
        Action::ClearInput => cut(0, n, false),
        // `cw` は語の中なら語の終わりまで(後ろの空白は残す。vim の `cw` の決まり)、空白の上なら次の語の頭まで。
        Action::ChangeWord if c < n && cls(c) != 0 => {
            let k = cls(c);
            let mut end = c;
            while end < n && cls(end) == k {
                end += 1;
            }
            cut(c, end, true)
        }
        Action::ChangeWord => cut(c, next_word(c), true),
        Action::ChangeToEnd => cut(c, n, true),
        Action::ChangeLine => cut(0, n, true),
        _ => return None,
    })
}

// ---- 画面と入力 ----

impl App {
    /// vim モードの入力の欄を開いているか(新しいノートの作る場所を選ぶ欄は、文字を受けないので入れない)。
    pub(crate) fn vim_field(&self) -> bool {
        self.vim.on
            && field_mode(self.mode)
            && !(self.mode == Mode::Edit && self.note_choosing())
            && self.vim_line().is_some()
    }

    /// 入力の欄がノーマルのモードか。
    fn vim_normal(&self) -> bool {
        self.vim_field() && self.vim.normal
    }

    /// キーを引くモード: 入力の欄がノーマルのモードなら `input_normal`、ほかは今のモード(前置きの案内にも使う)。
    pub(crate) fn key_mode(&self) -> Mode {
        if self.vim_normal() {
            Mode::InputNormal
        } else {
            self.mode
        }
    }

    /// 端末のカーソルの形: 挿入のモードは棒、ノーマルのモードは四角、ほかは端末の既定(screen.vim-mode-shown)。
    pub fn cursor_shape(&self) -> CursorShape {
        if !self.vim_field() {
            CursorShape::Default
        } else if self.vim.normal {
            CursorShape::Block
        } else {
            CursorShape::Bar
        }
    }

    /// 今の入力の欄の文字とカーソル(バイトの位置)。
    pub(crate) fn vim_line(&self) -> Option<(String, usize)> {
        let text = match self.mode {
            Mode::Edit => {
                let i = self.input.as_ref()?;
                return Some((i.text.clone(), i.cursor.min(i.text.len())));
            }
            Mode::Search | Mode::Filter => self.prompt.as_ref()?.text.clone(),
            Mode::Palette => self.palette.as_ref()?.query.clone(),
            Mode::SettingsText => self.draft.as_ref()?.text.as_ref()?.text.clone(),
            Mode::FilterText => match &self.filter_win.as_ref()?.step {
                Some(super::filters::Step::Text { text, .. }) => text.clone(),
                _ => return None,
            },
            _ => return None,
        };
        let at = self.vim.cursor.unwrap_or(text.len()).min(text.len());
        let at = bounds(&text).into_iter().rfind(|&x| x <= at).unwrap_or(0);
        Some((text, at))
    }

    /// 今の入力の欄の文字とカーソルを置き換え、欄ごとの続き(探す・検索の当て直し、候補の選び直しなど)をする。
    fn vim_put_line(&mut self, text: String, cursor: usize) {
        let changed = self.vim_line().is_none_or(|(t, _)| t != text);
        match self.mode {
            Mode::Edit => {
                // CE-3: 文字の編集とカーソルの操作は、候補のリストから自由入力に切り替える(←→ と同じ)。
                self.list_to_free();
                if let Some(i) = &mut self.input {
                    i.fresh = false;
                    i.touched |= changed;
                    i.text = text;
                    i.cursor = cursor;
                }
                if changed {
                    self.calendar_follow();
                }
                return;
            }
            Mode::Search | Mode::Filter => {
                if let Some(p) = &mut self.prompt {
                    p.text = text;
                }
                self.vim.cursor = Some(cursor);
                if changed {
                    self.prompt_changed();
                    self.refresh_if_needed();
                }
                return;
            }
            Mode::Palette => {
                if let Some(p) = &mut self.palette {
                    p.query = text;
                    if changed {
                        p.sel = 0;
                    }
                }
            }
            Mode::SettingsText => {
                if let Some(t) = self.draft.as_mut().and_then(|d| d.text.as_mut()) {
                    t.text = text;
                }
            }
            Mode::FilterText => {
                if let Some(super::filters::Step::Text { text: t, .. }) =
                    self.filter_win.as_mut().and_then(|w| w.step.as_mut())
                {
                    *t = text;
                }
            }
            _ => return,
        }
        self.vim.cursor = Some(cursor);
    }

    /// キーを受け取る前に、vim モードの入力の欄と表の範囲の `d` を扱う(screen.vim-*)。扱ったら true。
    pub(crate) fn vim_key(&mut self, ev: &KeyEvent, name: &str) -> bool {
        if !self.vim.on {
            return false;
        }
        if self.mode == Mode::Table {
            return self.vim_range_delete(name);
        }
        if !self.vim_field() {
            return false;
        }
        if self.vim.normal {
            self.vim_normal_key(ev, name)
        } else {
            self.vim_insert_key(ev, name)
        }
    }

    /// screen.vim-dd: 範囲を選んでいるときは、`d d` の前置きの `d` だけでノートを消す(vim の範囲の `d`)。
    /// `d d` を割り当て直せば、その前置きのキーで同じに効く。外せば効かない(screen.vim-remap)。
    fn vim_range_delete(&mut self, name: &str) -> bool {
        if self.prefix.is_some()
            || self.anchor.is_none()
            || !keymap::is_prefix(&self.keys, Mode::Table, name)
            || keymap::lookup(&self.keys, Mode::Table, &format!("{name} {name}"))
                != Some(Action::DeleteNote)
        {
            return false;
        }
        self.apply(Action::DeleteNote);
        true
    }

    /// 挿入のモードのキー: 抜けるキーでノーマルのモードへ。カーソルを持たない欄で、カーソルが途中にあれば、
    /// 打った字と BS をカーソルの所に当てる。ほかは今までの入力の欄と同じ(false を返す)。
    fn vim_insert_key(&mut self, ev: &KeyEvent, name: &str) -> bool {
        if self.vim.insert_start.is_none() {
            self.vim.insert_start = self.vim_line();
        }
        let found = keymap::lookup(&self.keys, self.mode, name).or_else(|| {
            keymap::unshifted(name).and_then(|n| keymap::lookup(&self.keys, self.mode, &n))
        });
        if found.is_some_and(|a| keymap::exits(self.mode).contains(&a)) {
            self.vim_to_normal();
            return true;
        }
        // カーソルを持たない欄で、カーソルが末尾にあれば今までの入力の欄と同じ(打った字は末尾に入る)。
        let Some(at) = self.vim.cursor.filter(|_| self.mode != Mode::Edit) else {
            return false;
        };
        let Some((text, _)) = self.vim_line() else {
            return false;
        };
        if at >= text.len() {
            self.vim.cursor = None;
            return false;
        }
        match (found, ev.code) {
            (Some(Action::DeleteBack), _) => {
                self.message = None;
                if let Some((k, _)) = text[..at].grapheme_indices(true).next_back() {
                    let mut t = text.clone();
                    t.replace_range(k..at, "");
                    self.vim_put_line(t, k);
                }
                true
            }
            (None, KeyCode::Char(c))
                if !ev
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.message = None;
                // SR-17: パレットは全角の英数字を半角に直す(`palette_insert` と同じ)。
                let c = if self.mode == Mode::Palette {
                    keymap::halfwidth(c)
                } else {
                    c
                };
                let mut t = text;
                t.insert(at, c);
                self.vim_put_line(t, at + c.len_utf8());
                true
            }
            _ => false,
        }
    }

    /// 挿入のモードからノーマルのモードへ。カーソルは1字左(vim と同じ)。挿入の間に変えた文字は、取り消しの1手。
    fn vim_to_normal(&mut self) {
        self.message = None;
        let Some((text, at)) = self.vim_line() else {
            return;
        };
        if let Some(start) = self.vim.insert_start.take() {
            if start.0 != text {
                self.vim.undo.push(start);
            }
        }
        let left = text[..at]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i);
        let at = clamp_normal(&text, left);
        self.vim.normal = true;
        match self.mode {
            // 候補のリストの選びは保つ(Esc だけでは自由入力に切り替えない)。
            Mode::Edit => {
                if let Some(i) = &mut self.input {
                    i.fresh = false;
                    i.cursor = at;
                }
            }
            _ => self.vim.cursor = Some(at),
        }
    }

    /// ノーマルのモードのキー。`input_normal` の表(前置きの `d`・`c` を含む)で先に引き、表に無い文字は入れない。
    /// Enter・Esc・矢印・Ctrl のキーなどは、欄のモードの表で今までどおり(false を返す)。
    fn vim_normal_key(&mut self, ev: &KeyEvent, name: &str) -> bool {
        let m = Mode::InputNormal;
        if let Some(p) = self.prefix.take() {
            if let Some(a) = keymap::lookup(&self.keys, m, &format!("{p} {name}")) {
                self.vim_normal_action(a);
                return true;
            }
            // SR-25: 前置きのあとの Esc は前置きを取り消すだけ。表に無いキーは、前置きを捨ててそのキーだけで引き直す。
            if name == "Esc" {
                return true;
            }
        }
        if keymap::is_prefix(&self.keys, m, name) {
            self.prefix = Some(name.to_string());
            return true;
        }
        if let Some(a) = keymap::lookup(&self.keys, m, name) {
            self.vim_normal_action(a);
            return true;
        }
        matches!(ev.code, KeyCode::Char(_))
            && !ev
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    }

    /// ノーマルのモードの動作(screen.vim-normal-keys)。
    fn vim_normal_action(&mut self, action: Action) {
        self.message = None;
        let Some((text, at)) = self.vim_line() else {
            return;
        };
        if action == Action::Undo {
            match self.vim.undo.pop() {
                Some((t, c)) => {
                    let c = clamp_normal(&t, c);
                    self.vim_put_line(t, c);
                }
                None => self.message = Some(Msg::NothingToUndo.text().into()),
            }
            return;
        }
        let Some(e) = normal(action, &text, at) else {
            return;
        };
        if e.insert {
            // 挿入のモードへ: 替える操作の消した分と、続けて打つ字を、取り消しの1手にする。
            self.vim.insert_start = Some((text.clone(), at));
            self.vim.normal = false;
        } else if e.text != text {
            self.vim.undo.push((text.clone(), at));
        }
        // 動きだけ(文字が同じ)でも、セルの編集では候補のリストから自由入力に切り替える(←→ と同じ)。
        self.vim_put_line(e.text, e.cursor);
    }

    /// 下の帯の「今押せるキー」(vim モードの入力の欄のとき)。挿入のモードでは抜けるキーを「ノーマルへ」と案内し、
    /// ノーマルのモードでは欄のキーのあとに `input_normal` のキー。
    pub(crate) fn vim_hints(&self) -> Option<Vec<String>> {
        if !self.vim_field() {
            return None;
        }
        let exits = keymap::exits(self.mode);
        let mut shown: Vec<(u8, Action, String)> = Vec::new();
        for b in self
            .keys
            .iter()
            .filter(|b| b.mode == self.mode && b.rank > 0)
        {
            if shown.iter().any(|(_, a, _)| *a == b.action) {
                continue;
            }
            let label = if !self.vim.normal && exits.contains(&b.action) {
                Msg::KeyVimToNormal.text()
            } else {
                b.text()
            };
            shown.push((
                b.rank,
                b.action,
                format!("{} {label}", keymap::display(b.key)),
            ));
        }
        shown.sort_by_key(|(r, _, _)| *r);
        let mut out: Vec<String> = shown.into_iter().map(|(_, _, s)| s).collect();
        if self.vim.normal {
            out.extend(keymap::hints(&self.keys, Mode::InputNormal));
        }
        Some(out)
    }

    /// 下の帯に出す今のモード(screen.vim-mode-shown)。vim モードの入力の欄でなければ None。
    pub(crate) fn vim_mode_text(&self) -> Option<&'static str> {
        self.vim_field().then(|| {
            if self.vim.normal {
                Msg::VimNormal.text()
            } else {
                Msg::VimInsert.text()
            }
        })
    }

    /// カーソルを持たない欄では、端末のカーソル(欄の末尾)を、vim のカーソルの所まで戻す。
    pub(crate) fn vim_cursor_shift(&self, (x, y): (u16, u16)) -> (u16, u16) {
        if !self.vim_field() || self.mode == Mode::Edit {
            return (x, y);
        }
        let Some((text, at)) = self.vim_line() else {
            return (x, y);
        };
        let back = width(&sanitize(&text[at..]));
        (x.saturating_sub(back as u16), y)
    }
}

// ---- 表の画面: 写す・入れる・繰り返す ----

/// セルの値を、写した値(`NewValue`)にする。値の形でないもの(リストの中の表など)は None。
fn yank_value(v: &Value) -> Option<NewValue> {
    Some(match v {
        Value::Null => NewValue::Null,
        Value::Str(s) => NewValue::Str(s.clone()),
        Value::Bool(b) => NewValue::Bool(*b),
        Value::Int(i) => NewValue::Int(*i),
        Value::Float(f) => NewValue::Float(*f),
        Value::List(items) => NewValue::List(
            items
                .iter()
                .map(|x| match x {
                    Value::Str(s) => Some(s.clone()),
                    Value::Bool(b) => Some(b.to_string()),
                    Value::Int(i) => Some(i.to_string()),
                    Value::Float(f) => Some(f.to_string()),
                    Value::Null | Value::List(_) | Value::Other => None,
                })
                .collect::<Option<Vec<_>>>()?,
        ),
        Value::Other => return None,
    })
}

impl App {
    /// `y`(セル)で写した値を覚える(screen.vim-paste)。
    pub(crate) fn remember_cell_yank(&mut self, row: &RowId, col: &str) {
        if let Some(v) = self.setting_value(row, col).as_ref().and_then(yank_value) {
            self.vim.yank = Some(Yank::Cell(v));
        }
    }

    /// `Y`(行)で写した行の、ノートのキーの値を覚える(screen.vim-paste)。計算の列(`file.*`・`formula.*`)は除く。
    pub(crate) fn remember_row_yank(&mut self, row: &RowId) {
        let vals = self
            .src
            .columns()
            .into_iter()
            .filter(|c| !c.starts_with("file.") && !c.starts_with("formula."))
            .filter_map(|c| {
                let v = self.prop(row, &c)?;
                Some((c, yank_value(&v)?))
            })
            .collect();
        self.vim.yank = Some(Yank::Row(vals));
    }

    /// 1つのセルに値を入れた(screen.vim-repeat の `.` で繰り返す)。
    pub(crate) fn remember_set(&mut self, col: &str, v: &NewValue) {
        self.vim.last_set = Some((col.to_string(), v.clone()));
    }

    /// 写した値を、入れる列の型の値にする(表の入力と同じ読み方)。型に合わなければ理由。
    fn paste_value(&self, col: &str, v: &NewValue) -> Result<NewValue, String> {
        let kind = self.col_type(col);
        match (kind, v) {
            (_, NewValue::Null) => Ok(NewValue::Null),
            // WB-18: 日付・日時の列は、打ち込みと同じく日付の値にして書く(囲まずに書く形を保つ)。
            (Kind::Date | Kind::DateTime, _) => {
                super::kanban::value_for(self, col, &new_value_text(v))
            }
            (Kind::List, NewValue::List(_)) => Ok(v.clone()),
            (Kind::List, _) => Ok(NewValue::List(vec![new_value_text(v)])),
            (_, NewValue::List(_)) => super::kanban::value_for(self, col, &new_value_text(v)),
            _ if types::fits(kind, &super::grid::value_of(v)) => Ok(v.clone()),
            _ => super::kanban::value_for(self, col, &new_value_text(v)),
        }
    }

    /// output.paste: 貼った文字を、入れる列の型の値にする(打ち込みと同じ読み方。空の文字は空にする)。
    pub(crate) fn paste_text_value(&self, col: &str, text: &str) -> Result<NewValue, String> {
        match self.col_type(col) {
            Kind::List if !text.is_empty() => Ok(NewValue::List(vec![text.to_string()])),
            _ => super::kanban::value_for(self, col, text),
        }
    }

    /// screen.vim-paste: `p`。セルを写したなら選んでいるセルに、行を写したなら印を付けた行(無ければ選んでいる行)の
    /// 同じ名前の書ける列に、ためる変更として入れる(行は1手)。
    pub(crate) fn vim_paste(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(yank) = self.vim.yank.clone() else {
            self.message = Some(Msg::PasteNothing.text().into());
            return;
        };
        match yank {
            Yank::Cell(v) => {
                let Some((row, col)) = self.selected() else {
                    self.message = Some(Msg::PasteNoCell.text().into());
                    return;
                };
                if !super::view::selected_drawn(self) {
                    self.message = Some(Msg::CellNotDrawn.into());
                    return;
                }
                let v = match self.paste_value(&col, &v) {
                    Ok(v) => v,
                    Err(e) => {
                        self.message = Some(e);
                        return;
                    }
                };
                self.vim_set(&row, &col, v);
            }
            Yank::Row(vals) => {
                let mut rows = self.selection();
                if rows.is_empty() {
                    rows.extend(self.cur_row());
                }
                if rows.is_empty() {
                    self.message = Some(Msg::PasteNoRow.text().into());
                    return;
                }
                let mut items = Vec::new();
                let mut reasons = Vec::new();
                for r in &rows {
                    for (c, v) in &vals {
                        match self.paste_value(c, v) {
                            Ok(nv) => items.push((r.clone(), c.clone(), nv)),
                            Err(e) => reasons.push(e),
                        }
                    }
                }
                let skips = self.changes.set_cells(self.src.as_ref(), &items);
                reasons.extend(skips.into_iter().map(|s| s.reason));
                self.stay.extend(rows.iter().cloned());
                self.regrid = true;
                let mut msg = Msg::PasteRows.fill(&[&rows.len()]);
                bulk_tail(&mut msg, 0, reasons);
                self.message = Some(msg);
            }
        }
    }

    /// screen.vim-repeat: `.`。直前に1つのセルに入れた値を、選んでいる行の同じ列に入れる(選んでいる列によらない)。
    pub(crate) fn vim_repeat(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some((col, v)) = self.vim.last_set.clone() else {
            self.message = Some(Msg::RepeatNothing.text().into());
            return;
        };
        let Some(row) = self.cur_row() else {
            return;
        };
        self.vim_set(&row, &col, v);
    }

    /// 1つのセルに値をためる(列の `then` と、印の children の子孫も一緒に。表の入力と同じ)。入れた値は `.` で
    /// 繰り返せる。
    fn vim_set(&mut self, row: &RowId, col: &str, v: NewValue) {
        let st = self.stage_children(col, vec![(row.clone(), Some(v.clone()))]);
        if let Some((_, why)) = st.refused.into_iter().next() {
            self.message = Some(why);
            return;
        }
        if let Some(s) = st.skips.first() {
            self.message = Some(Msg::CannotCommit.fill(&[&s.reason]));
            return;
        }
        // NV-12: 直した行は、保存か移動の操作まで元の位置に留める。
        self.stay.insert(row.clone());
        self.stay.extend(st.extra);
        self.regrid = true;
        self.remember_set(col, &v);
        if st.notice.is_some() {
            self.message = st.notice;
        }
    }
}
