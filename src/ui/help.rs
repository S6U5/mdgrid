//! ヘルプ(SR-5)とコマンドのパレット(SR-14・SR-16・NV-7 の `:<番号>`)。どちらもキーの表(SR-4)から作る。
//! `impl App` の続き(状態と意図の処理)と、描画の純関数を持つ。

use super::app::App;
use super::grid::Slot;
use super::keymap::{self, Action, Binding, Mode};
use super::view;
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// ヘルプの状態。
pub(crate) struct HelpState {
    /// 開いたときのモード(先頭にこのモードのキーを出し、閉じたらここへ戻る)。
    pub from: Mode,
    /// 本文の先頭の行。
    pub top: usize,
}

/// パレットの状態。
#[derive(Default)]
pub(crate) struct PaletteState {
    pub query: String,
    /// 選んだ候補。
    pub sel: usize,
}

/// パレットの候補の行き先。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Run(Action),
    /// 行番号(1 始まり。NV-7)。
    Goto(usize),
}

/// パレットの候補。
pub(crate) struct Cand {
    pub target: Target,
    pub label: String,
    pub name: String,
    /// 割り当てキー(表のモードの全部。空白区切り)。
    pub keys: String,
}

/// パレットに一度に見せる候補の数。
const PALETTE_ROWS: usize = 8;
/// ヘルプのキーの欄の幅。
const KEY_W: usize = 14;

/// 1つのモードのキーを、動作ごとに(キーを空白でつないで)まとめる。表の順。
fn entries(keys: &[Binding], pick: impl Fn(&Binding) -> bool) -> Vec<(String, &'static str, u8)> {
    let mut out: Vec<(Mode, Action, String, &'static str, u8)> = Vec::new();
    for b in keys.iter().filter(|b| pick(b)) {
        if let Some(e) = out.iter_mut().find(|e| e.0 == b.mode && e.1 == b.action) {
            e.2.push(' ');
            e.2.push_str(&keymap::display(b.key));
        } else {
            out.push((b.mode, b.action, keymap::display(b.key), b.text(), b.rank));
        }
    }
    out.into_iter().map(|(_, _, k, l, r)| (k, l, r)).collect()
}

/// 読むだけ(WB-15)で出さない、パレットのコマンド(書く動作)か。キーの表にあっても出さない。
fn write_command(app: &App, a: Action) -> bool {
    app.readonly && keymap::COMMANDS.iter().any(|c| c.action == a)
}

/// 項目を2列(狭ければ1列)に並べる。
fn columns(items: &[(String, &'static str, u8)], w: usize) -> Vec<String> {
    let cell = |(k, l, _): &(String, &'static str, u8), cw: usize| {
        fit(
            &format!("{} {}", fit(&sanitize(k), KEY_W, Align::Left), l),
            cw,
            Align::Left,
        )
    };
    if w < 44 {
        return items
            .iter()
            .map(|it| format!("  {}", cell(it, w.saturating_sub(2))))
            .collect();
    }
    let cw = (w - 4) / 2;
    items
        .chunks(2)
        .map(|pair| {
            let mut line = format!("  {}", cell(&pair[0], cw));
            if let Some(second) = pair.get(1) {
                line.push_str("  ");
                line.push_str(&cell(second, cw));
            }
            line
        })
        .collect()
}

/// ヘルプの本文の行の種類(描くときの色。文字は同じ)。
#[derive(Clone, PartialEq, Eq)]
enum Kind {
    /// 節の見出し。
    Head,
    /// キーの欄のある行: (始まりの桁, 幅) の並び。
    Keys(Vec<(usize, usize)>),
    /// ほか。
    Plain,
}

/// `columns` の行のキーの欄。
fn key_spans(w: usize) -> Vec<(usize, usize)> {
    if w < 44 {
        vec![(2, KEY_W)]
    } else {
        vec![(2, KEY_W), (4 + (w - 4) / 2, KEY_W)]
    }
}

/// ヘルプの本文(行の文字と、節の見出しか)。先頭に `from` のモードで押せるキー(下の帯の順位の順)、
/// その下に全部の節(SR-5)。
pub(crate) fn help_lines(app: &App, from: Mode, w: usize) -> Vec<(String, bool)> {
    help_rows(app, from, w)
        .into_iter()
        .map(|(t, k)| (t, k == Kind::Head))
        .collect()
}

/// ヘルプの本文と、行ごとの種類。
fn help_rows(app: &App, from: Mode, w: usize) -> Vec<(String, Kind)> {
    let mut out = Vec::new();
    let mut now = entries(&app.keys, |b| {
        b.mode == from && !write_command(app, b.action)
    });
    now.sort_by_key(|(_, _, r)| if *r == 0 { u8::MAX } else { *r });
    out.push((Msg::HelpNow.fill(&[&from.label()]), Kind::Head));
    out.extend(
        columns(&now, w)
            .into_iter()
            .map(|l| (l, Kind::Keys(key_spans(w)))),
    );
    // 節は日本語の名前でまとめ、今の言語の名前で見せる。
    let mut sections: Vec<(&'static str, &'static str)> = Vec::new();
    for b in &app.keys {
        if !sections.iter().any(|(s, _)| *s == b.section) {
            sections.push((b.section, b.section_text()));
        }
    }
    for (s, shown) in sections {
        out.push((String::new(), Kind::Plain));
        out.push((format!(" {shown}"), Kind::Head));
        let items = entries(&app.keys, |b| {
            b.section == s && !write_command(app, b.action)
        });
        if items.is_empty() {
            out.truncate(out.len() - 2);
            continue;
        }
        out.extend(
            columns(&items, w)
                .into_iter()
                .map(|l| (l, Kind::Keys(key_spans(w)))),
        );
    }
    // BV-19: キーの無いパレットのコマンド(キーの欄は `:` のあとに打つ名前)。
    let mut cmd_sections: Vec<(&'static str, &'static str)> = Vec::new();
    // 読むだけ(WB-15)では書くコマンドなので出さない(パレットと同じ。ノートを書かない登録した表のは出す)。
    for c in keymap::commands(app.readonly) {
        if !cmd_sections.iter().any(|(s, _)| *s == c.section) {
            cmd_sections.push((c.section, c.section_text()));
        }
    }
    for (s, shown) in cmd_sections {
        out.push((String::new(), Kind::Plain));
        out.push((format!(" {shown}"), Kind::Head));
        let items: Vec<(String, &'static str, u8)> = keymap::commands(app.readonly)
            .filter(|c| c.section == s)
            .map(|c| (format!(":{}", c.action.name()), c.text(), 0))
            .collect();
        // 名前が長いので1列に並べる(2列では切れる)。
        out.extend(
            columns(&items, w.min(43))
                .into_iter()
                .map(|l| (l, Kind::Keys(key_spans(w.min(43))))),
        );
    }
    // SR-5: 最後に、色の代わりの印(SR-15)の意味。印の文字は画面と同じ定数から。
    out.push((String::new(), Kind::Plain));
    out.push((Msg::HelpMarks.text().to_string(), Kind::Head));
    let mark = |m: &str, t: Msg| {
        (
            format!("    {}  {}", fit(m, 7, Align::Left), t.text()),
            Kind::Keys(vec![(4, 7)]),
        )
    };
    out.push((Msg::HelpMarksCell.text().to_string(), Kind::Plain));
    let misfit = super::cell::MISFIT_MARK.to_string();
    let lock = super::cell::LOCK_MARK.to_string();
    for (m, t) in [
        ("∅", Msg::MarkNull),
        ("\"\"", Msg::MarkEmptyStr),
        (Msg::MarkBlankSign.text(), Msg::MarkBlank),
        (misfit.as_str(), Msg::MarkMisfit),
        ("*", Msg::MarkPending),
        (lock.as_str(), Msg::MarkLock),
        (super::cell::UNSUPPORTED_MARK, Msg::MarkUnsupported),
        ("…⏎", Msg::MarkMultiline),
    ] {
        out.push(mark(m, t));
    }
    out.push((Msg::HelpMarksRow.text().to_string(), Kind::Plain));
    let held = super::view::HELD_MARK.to_string();
    for (m, t) in [
        (">", Msg::MarkRowCurrent),
        ("+", Msg::MarkRowMarked),
        ("!", Msg::MarkRowExternal),
        (held.as_str(), Msg::MarkRowHeld),
    ] {
        out.push(mark(m, t));
    }
    out
}

/// ヘルプの画面: 見出し・本文・下の帯・メッセージ行。
pub(crate) fn render_help(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(st) = &app.help else {
        return Vec::new();
    };
    let body = help_rows(app, st.from, w);
    let look = super::look::look(app);
    let body_h = h.saturating_sub(3);
    let top = st.top.min(body.len().saturating_sub(body_h));
    let end = (top + body_h).min(body.len());
    let head = Msg::HelpHead.fill(&[&(top + 1), &end, &body.len()]);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let mut lines = vec![Line::from(Span::styled(fit(&head, w, Align::Left), bold))];
    for (t, kind) in &body[top..end] {
        let text = fit(t, w, Align::Left);
        // SR-33: モダンな見た目では、節の見出しとキーの欄をアクセントの色に(文字は同じ)。
        let line = match (kind, &look) {
            (Kind::Head, Some(l)) => Line::from(Span::styled(text, l.key())),
            (Kind::Head, None) => Line::from(Span::styled(text, bold)),
            (Kind::Keys(keys), Some(l)) => Line::from(key_line(&text, keys, l.key())),
            _ => Line::from(Span::raw(text)),
        };
        lines.push(line);
    }
    while lines.len() < h.saturating_sub(2) {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(h.saturating_sub(2));
    lines.push(view::footer(app, w));
    lines.push(view::message(app, w));
    lines.truncate(h);
    lines
}

/// 行 `text` を、キーの欄(始まりの桁, 幅)だけ `key` の見た目にした span に分ける(桁は表示の幅で数える)。
fn key_line(text: &str, keys: &[(usize, usize)], key: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cur = String::new();
    let mut cur_key = false;
    let mut x = 0;
    for c in text.chars() {
        let in_key = keys.iter().any(|&(s, w)| x >= s && x < s + w);
        if in_key != cur_key && !cur.is_empty() {
            let st = if cur_key { key } else { Style::default() };
            spans.push(Span::styled(std::mem::take(&mut cur), st));
        }
        cur_key = in_key;
        cur.push(c);
        x += width(&c.to_string());
    }
    if !cur.is_empty() {
        spans.push(Span::styled(
            cur,
            if cur_key { key } else { Style::default() },
        ));
    }
    spans
}

/// あいまいな一致の点(大きいほど良い)。一致しなければ None。
/// 同じ文字列 > 先頭の一致 > 途中の一致 > 飛び飛びの一致(間が狭いほど良い)。
pub(crate) fn score(query: &str, text: &str) -> Option<i32> {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    if q.is_empty() {
        return Some(0);
    }
    if t == q {
        return Some(1000);
    }
    if t.starts_with(&q) {
        return Some(900 - t.chars().count() as i32);
    }
    if let Some(i) = t.find(&q) {
        return Some(700 - t[..i].chars().count() as i32);
    }
    let tc: Vec<char> = t.chars().collect();
    let mut i = 0;
    let mut first = None;
    for c in q.chars() {
        while i < tc.len() && tc[i] != c {
            i += 1;
        }
        if i == tc.len() {
            return None;
        }
        first.get_or_insert(i);
        i += 1;
    }
    Some(400 - (i - first.unwrap_or(0)) as i32)
}

/// パレットの候補(SR-14)。`120` は行へ移動(NV-7)、`w` は保存、`q` は終了(SR-16)を先頭に。
/// 残りは表のモードの動作を、名前と表示名のあいまいな一致の点の順に。
pub(crate) fn candidates(app: &App, query: &str) -> Vec<Cand> {
    let q = query.trim();
    let mut out = Vec::new();
    let cand = |target: Target, label: String, name: &str| {
        let keys = match target {
            Target::Run(a) => keymap::keys_of(&app.keys, Mode::Table, a).join(" "),
            Target::Goto(_) => String::new(),
        };
        Cand {
            target,
            label,
            name: name.to_string(),
            keys,
        }
    };
    if !q.is_empty() && q.chars().all(|c| c.is_ascii_digit()) {
        let n = q.parse().unwrap_or(usize::MAX);
        out.push(cand(Target::Goto(n), Msg::PaletteGoto.fill(&[&n]), q));
    }
    let label_of = |a: Action| {
        app.keys
            .iter()
            .find(|b| b.mode == Mode::Table && b.action == a)
            .map(|b| b.text())
            .unwrap_or(a.name())
    };
    match q {
        "w" if !app.readonly => out.push(cand(
            Target::Run(Action::Save),
            label_of(Action::Save).into(),
            "w",
        )),
        "q" => out.push(cand(
            Target::Run(Action::Quit),
            label_of(Action::Quit).into(),
            "q",
        )),
        _ => {}
    }
    let mut scored: Vec<(i32, usize, Action)> = Vec::new();
    for b in app.keys.iter().filter(|b| b.mode == Mode::Table) {
        let a = b.action;
        if a == Action::Palette
            || write_command(app, a)
            || scored.iter().any(|s| s.2 == a)
            || out.iter().any(|c| c.target == Target::Run(a))
        {
            continue;
        }
        let best = [score(q, a.name()), score(q, label_of(a))]
            .into_iter()
            .flatten()
            .max();
        if let Some(p) = best {
            scored.push((p, scored.len(), a));
        }
    }
    // BV-19: キーの無いコマンド(読むだけでは書くので出さない。ノートを書かない登録した表のは出す)。
    let mut commands: Vec<&keymap::Command> = Vec::new();
    for c in keymap::commands(app.readonly) {
        let best = [score(q, c.action.name()), score(q, c.text())]
            .into_iter()
            .flatten()
            .max();
        if let Some(p) = best {
            // キーの表にもある動作(新しいノート)は、点の高い方の1つにする。
            if let Some(s) = scored.iter_mut().find(|s| s.2 == c.action) {
                s.0 = s.0.max(p);
            } else if !out.iter().any(|x| x.target == Target::Run(c.action)) {
                scored.push((p, scored.len(), c.action));
            }
            commands.push(c);
        }
    }
    scored.sort_by_key(|&(p, i, _)| (-p, i));
    for (_, _, a) in scored {
        let label = commands
            .iter()
            .find(|c| c.action == a)
            .map(|c| c.text())
            .unwrap_or_else(|| label_of(a));
        out.push(cand(Target::Run(a), label.into(), a.name()));
    }
    out
}

/// パレットを表の上に重ねる: 2行目に入力、その下に候補。
pub(crate) fn palette_overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(p) = &app.palette else {
        return;
    };
    // BV-19: 書き出し・取り込みの続きの入力。
    if app.nv.ask.is_some() {
        return super::native_views::ask_overlay(app, lines, w);
    }
    let mut out = vec![view::band(&format!(": {}", p.query), w)];
    let cands = candidates(app, &p.query);
    let room = lines.len().saturating_sub(3).clamp(1, PALETTE_ROWS);
    if cands.is_empty() {
        out.push(Line::from(fit(Msg::PaletteEmpty.text(), w, Align::Left)));
    }
    let start = p.sel.saturating_sub(room - 1);
    for (i, c) in cands.iter().enumerate().skip(start).take(room) {
        let sel = i == p.sel;
        let t = format!(
            "{} {} {} {}",
            if sel { ">" } else { " " },
            fit(&sanitize(&c.label), 24, Align::Left),
            fit(&c.name, 16, Align::Left),
            sanitize(&c.keys)
        );
        let st = if sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
    }
    out.push(Line::from(" ".repeat(w)));
    for (k, l) in out.into_iter().enumerate() {
        if let Some(slot) = lines.get_mut(1 + k) {
            *slot = l;
        }
    }
}

/// パレットの入力の位置(SR-17: 変換の窓がそこに出る)。
pub(crate) fn palette_cursor(app: &App) -> Option<(u16, u16)> {
    let p = app.palette.as_ref()?;
    let lead = super::native_views::ask_lead(app);
    let x = 2 + width(&lead) + width(&sanitize(&p.query));
    let w = app.size.0.saturating_sub(1) as usize;
    (x < w).then_some((x as u16, 1))
}

impl App {
    /// `?`: ヘルプを重ねる(SR-5)。
    pub(crate) fn open_help(&mut self) {
        self.help = Some(HelpState {
            from: self.mode,
            top: 0,
        });
        self.set_mode(Mode::Help);
    }

    /// ヘルプの中の動作: 閉じる・流す。
    pub(crate) fn help_action(&mut self, action: Action) {
        let w = self.size.0.saturating_sub(1) as usize;
        let body_h = (self.size.1.saturating_sub(1) as usize).saturating_sub(3);
        let Some(from) = self.help.as_ref().map(|h| h.from) else {
            return self.set_mode(Mode::Table);
        };
        let max = help_lines(self, from, w).len().saturating_sub(body_h);
        let Some(st) = &mut self.help else {
            return;
        };
        match action {
            Action::Close => {
                self.help = None;
                self.set_mode(from);
            }
            Action::Down => st.top = (st.top + 1).min(max),
            Action::Up => st.top = st.top.saturating_sub(1),
            Action::PageDown => st.top = (st.top + body_h.max(1)).min(max),
            Action::PageUp => st.top = st.top.saturating_sub(body_h.max(1)),
            Action::Top => st.top = 0,
            Action::Bottom => st.top = max,
            _ => {}
        }
    }

    /// `:` と Ctrl+P: パレットを開く(SR-14)。
    pub(crate) fn open_palette(&mut self) {
        self.palette = Some(PaletteState::default());
        self.set_mode(Mode::Palette);
    }

    /// パレットに文字を足す。全角の英数字は半角に直す(SR-17)。
    pub(crate) fn palette_insert(&mut self, c: char) {
        self.message = None;
        if let Some(p) = &mut self.palette {
            p.query.push(keymap::halfwidth(c));
            p.sel = 0;
        }
    }

    pub(crate) fn close_palette(&mut self) {
        self.palette = None;
        self.nv.ask = None;
        self.set_mode(Mode::Table);
    }

    /// パレットの中の動作: 実行・閉じる・候補の選択・1字消す。
    pub(crate) fn palette_action(&mut self, action: Action) {
        if self.nv.ask.is_some() {
            return self.ask_action(action);
        }
        let Some(query) = self.palette.as_ref().map(|p| p.query.clone()) else {
            return self.close_palette();
        };
        let cands = candidates(self, &query);
        let last = cands.len().saturating_sub(1);
        let Some(p) = &mut self.palette else {
            return;
        };
        match action {
            Action::Close => self.close_palette(),
            Action::Down => p.sel = (p.sel + 1).min(last),
            Action::Up => p.sel = p.sel.saturating_sub(1),
            Action::DeleteBack => {
                p.query.pop();
                p.sel = 0;
            }
            Action::Run => match cands.get(p.sel) {
                None => self.message = Some(Msg::PaletteNoMatch.fill(&[&sanitize(&query)])),
                Some(c) => {
                    let target = c.target;
                    self.close_palette();
                    match target {
                        Target::Run(a) => self.apply(a),
                        Target::Goto(n) => self.goto_line(n),
                    }
                }
            },
            _ => {}
        }
    }

    /// `:<番号>`: その行へ移る(NV-7。1 始まりのノートの行)。行が足りなければ末尾へ移って知らせる。
    /// 畳んだグループの中の行なら、その見出しへ移って知らせる(SR-2)。
    pub(crate) fn goto_line(&mut self, n: usize) {
        // NV-12: 移動の操作なので、先に留めた行を本来の位置へ戻してから番号を引く。
        if !self.stay.is_empty() {
            self.stay.clear();
            self.refresh();
        }
        if self.rows.is_empty() {
            self.message = Some(Msg::GotoNoRows.text().into());
            return;
        }
        let len = self.rows.len();
        let i = n.max(1) - 1;
        if i >= len {
            self.message = Some(Msg::GotoPastEnd.fill(&[&n, &len]));
        }
        let i = i.min(len - 1);
        let pos = self.slots.iter().position(|s| *s == Slot::Row(i));
        let pos = pos.or_else(|| {
            let g = self.groups.iter().position(|(_, r)| r.contains(&i))?;
            self.message = Some(Msg::GotoFolded.fill(&[&(i + 1)]));
            self.slots.iter().position(|s| *s == Slot::Head(g))
        });
        if let Some(p) = pos {
            self.row = p;
        }
        self.scroll_into_view();
    }
}
