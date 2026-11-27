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
    /// 選んでいる項目(本文の選べる項目を上から数えた番号。screen.help-run)。
    pub sel: usize,
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
    /// `wq`・`x`: 保存して終わる(差分の確認 → 確定で書いて終わる)。
    SaveQuit,
    /// `q!`: ためた変更を確かめずに捨てて終わる(screen.quit-discard)。
    DiscardQuit,
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

/// ヘルプの1つの項目。
struct Item {
    /// キーの欄(空白区切り)。
    keys: String,
    label: String,
    /// 下の帯の順位。
    rank: u8,
    /// Enter でする動作(screen.help-run)。数字のキーをまとめた項目は先頭のキーの動作。
    action: Action,
}

/// 1つのモードのキーを、動作ごとに(キーを空白でつないで)まとめる。表の順。番号の区画へ移る動作(1〜9)は
/// 1つにまとめ、続いた数字のキーは `1-9` と縮める(キーの欄に入るように)。
/// condition.digit-hints: 数字のキーは条件の見せる名前にし、条件の無い数字は出さない。
fn entries(app: &App, keys: &[Binding], pick: impl Fn(&Binding) -> bool) -> Vec<Item> {
    let group = |a: Action| match a {
        Action::GoToSection(_) => Action::GoToSection(0),
        // workspace.switch-origin-key: 0 は「最初に開いた所」で、1〜9 とは別の行。
        Action::Choose(n) if n > 0 => Action::Choose(1),
        a => a,
    };
    let mut out: Vec<(Mode, Item)> = Vec::new();
    for b in keys.iter().filter(|b| pick(b)) {
        let label = match app.cond_label(b.action) {
            Some(l) => l,
            None if b.action.cond_digit().is_some() => continue,
            None => b.text().to_string(),
        };
        if let Some((_, e)) = out
            .iter_mut()
            .find(|(m, e)| *m == b.mode && group(e.action) == group(b.action))
        {
            e.keys.push(' ');
            e.keys.push_str(&keymap::display(b.key));
        } else {
            out.push((
                b.mode,
                Item {
                    keys: keymap::display(b.key),
                    label,
                    rank: b.rank,
                    action: b.action,
                },
            ));
        }
    }
    out.into_iter()
        .map(|(_, mut e)| {
            if let Some(k) = digit_run(&e.keys) {
                e.keys = k;
            }
            e
        })
        .collect()
}

/// 3つ以上続いた数字のキーだけの並び(`1 2 3 … 9`)を `1-9` にする。ほかは None。
fn digit_run(keys: &str) -> Option<String> {
    let d: Vec<u32> = keys
        .split(' ')
        .map(|k| {
            let mut c = k.chars();
            match (c.next(), c.next()) {
                (Some(x), None) => x.to_digit(10),
                _ => None,
            }
        })
        .collect::<Option<_>>()?;
    let (first, last) = (*d.first()?, *d.last()?);
    let run = d.len() >= 3 && d.windows(2).all(|w| w[1] == w[0] + 1);
    run.then(|| format!("{first}-{last}"))
}

/// 読むだけ(WB-15)で出さない、パレットのコマンド(書く動作)か。キーの表にあっても出さない。
fn write_command(app: &App, a: Action) -> bool {
    app.readonly && keymap::COMMANDS.iter().any(|c| c.action == a)
}

/// 項目を2列(狭ければ1列)に並べる。`run` なら項目の Enter でその動作をする(「今押せるキー」の節。screen.help-run)。
fn columns(items: &[Item], w: usize, run: bool) -> Vec<Row> {
    let text = |it: &Item, cw: usize| {
        fit(
            &format!(
                "{} {}",
                fit(&sanitize(&it.keys), KEY_W, Align::Left),
                it.label
            ),
            cw,
            Align::Left,
        )
    };
    let cell = |it: &Item, x: usize, w: usize| Cell {
        x,
        w,
        run: run.then_some(it.action),
    };
    if w < 44 {
        let cw = w.saturating_sub(2);
        return items
            .iter()
            .map(|it| Row {
                text: format!("  {}", text(it, cw)),
                kind: Kind::Keys(key_spans(w)),
                cells: vec![cell(it, 2, cw)],
            })
            .collect();
    }
    let cw = (w - 4) / 2;
    items
        .chunks(2)
        .map(|pair| {
            let mut line = format!("  {}", text(&pair[0], cw));
            let mut cells = vec![cell(&pair[0], 2, cw)];
            if let Some(second) = pair.get(1) {
                line.push_str("  ");
                line.push_str(&text(second, cw));
                cells.push(cell(second, 4 + cw, cw));
            }
            Row {
                text: line,
                kind: Kind::Keys(key_spans(w)),
                cells,
            }
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

/// ヘルプの本文の選べる項目(screen.help-run): 始まりの桁と幅、Enter でする動作。
/// 動作を持つのは先頭の「今押せるキー」の節の項目だけ(下の節の項目は選べるが、Enter で何もしない)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Cell {
    x: usize,
    w: usize,
    run: Option<Action>,
}

/// ヘルプの本文の1行。
struct Row {
    text: String,
    kind: Kind,
    /// 行の中の選べる項目(左から)。見出しと空行は無い。
    cells: Vec<Cell>,
}

impl Row {
    fn plain(text: String, kind: Kind) -> Row {
        Row {
            text,
            kind,
            cells: Vec::new(),
        }
    }
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
/// その下に全部の節(SR-5)。試験が本文の文字を見るのに使う。
#[cfg(test)]
pub(crate) fn help_lines(app: &App, from: Mode, w: usize) -> Vec<(String, bool)> {
    help_rows(app, from, w)
        .into_iter()
        .map(|r| (r.text, r.kind == Kind::Head))
        .collect()
}

/// ヘルプの本文の行。
fn help_rows(app: &App, from: Mode, w: usize) -> Vec<Row> {
    let mut out = Vec::new();
    let mut now = entries(app, &app.keys, |b| {
        b.mode == from && !write_command(app, b.action)
    });
    now.sort_by_key(|e| if e.rank == 0 { u8::MAX } else { e.rank });
    out.push(Row::plain(Msg::HelpNow.fill(&[&from.label()]), Kind::Head));
    // screen.help-run: この節の項目だけ、Enter でヘルプを閉じてその動作をする。
    out.extend(columns(&now, w, true));
    // 節は日本語の名前でまとめ、今の言語の名前で見せる。
    let mut sections: Vec<(&'static str, &'static str)> = Vec::new();
    for b in &app.keys {
        if !sections.iter().any(|(s, _)| *s == b.section) {
            sections.push((b.section, b.section_text()));
        }
    }
    for (s, shown) in sections {
        let items = entries(app, &app.keys, |b| {
            b.section == s && !write_command(app, b.action)
        });
        if items.is_empty() {
            continue;
        }
        out.push(Row::plain(String::new(), Kind::Plain));
        out.push(Row::plain(format!(" {shown}"), Kind::Head));
        out.extend(columns(&items, w, false));
    }
    // BV-19: キーの無いパレットのコマンド(キーの欄は `:` のあとに打つ名前)。
    let mut cmd_sections: Vec<(&'static str, &'static str)> = Vec::new();
    // 読むだけ(WB-15)では書くコマンドなので出さない(パレットと同じ。ノートを書かない登録した表のは出す)。
    for c in keymap::commands(app.readonly).filter(|c| app.command_shown(c.action)) {
        if !cmd_sections.iter().any(|(s, _)| *s == c.section) {
            cmd_sections.push((c.section, c.section_text()));
        }
    }
    for (s, shown) in cmd_sections {
        out.push(Row::plain(String::new(), Kind::Plain));
        out.push(Row::plain(format!(" {shown}"), Kind::Head));
        let items: Vec<Item> = keymap::commands(app.readonly)
            .filter(|c| c.section == s && app.command_shown(c.action))
            .map(|c| Item {
                keys: format!(":{}", c.action.name()),
                label: c.text().to_string(),
                rank: 0,
                action: c.action,
            })
            .collect();
        // 名前が長いので1列に並べる(2列では切れる)。
        out.extend(columns(&items, w.min(43), false));
    }
    // SR-5: 最後に、色の代わりの印(SR-15)の意味。印の文字は画面と同じ定数から。
    out.push(Row::plain(String::new(), Kind::Plain));
    out.push(Row::plain(Msg::HelpMarks.text().to_string(), Kind::Head));
    let mark = |m: &str, t: Msg| {
        let text = format!("    {}  {}", fit(m, 7, Align::Left), t.text());
        let cw = width(&text).saturating_sub(4);
        Row {
            text,
            kind: Kind::Keys(vec![(4, 7)]),
            cells: vec![Cell {
                x: 4,
                w: cw,
                run: None,
            }],
        }
    };
    out.push(Row::plain(
        Msg::HelpMarksCell.text().to_string(),
        Kind::Plain,
    ));
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
    out.push(Row::plain(
        Msg::HelpMarksRow.text().to_string(),
        Kind::Plain,
    ));
    let held = super::view::HELD_MARK.to_string();
    for (m, t) in [
        (">", Msg::MarkRowCurrent),
        ("+", Msg::MarkRowMarked),
        ("!", Msg::MarkRowExternal),
        (super::view::GHOST_MARK, Msg::MarkRowContext),
        (held.as_str(), Msg::MarkRowHeld),
    ] {
        out.push(mark(m, t));
    }
    out
}

/// 本文の選べる項目を上から並べる: (行, 項目)。`HelpState::sel` はこの並びの番号。
fn picks(rows: &[Row]) -> Vec<(usize, Cell)> {
    rows.iter()
        .enumerate()
        .flat_map(|(i, r)| r.cells.iter().map(move |c| (i, *c)))
        .collect()
}

/// `sel` の項目から本文の `page` 行だけ下(`down`)か上の行の項目の番号。行の中の何番目かは保つ(無ければその行の
/// 最後)。その向きに項目の行が無ければ、端の項目。
fn page_pick(picks: &[(usize, Cell)], sel: usize, page: usize, down: bool) -> usize {
    let Some(&(line, _)) = picks.get(sel) else {
        return 0;
    };
    let nth = picks[..sel]
        .iter()
        .rev()
        .take_while(|p| p.0 == line)
        .count();
    let target = if down {
        picks.iter().map(|p| p.0).find(|&l| l >= line + page)
    } else {
        picks.iter().map(|p| p.0).rev().find(|&l| l + page <= line)
    };
    let Some(t) = target else {
        return if down {
            picks.len().saturating_sub(1)
        } else {
            0
        };
    };
    let first = picks.iter().position(|p| p.0 == t).unwrap_or(0);
    let n = picks[first..].iter().take_while(|p| p.0 == t).count();
    first + nth.min(n.saturating_sub(1))
}

/// 選んでいる項目の行が見えるように流す。上へ出たら、その上の見出しと空行も見せる(節の先頭の項目を選んだとき)。
fn follow(st: &mut HelpState, rows: &[Row], picks: &[(usize, Cell)], body_h: usize, max: usize) {
    let Some(&(line, _)) = picks.get(st.sel) else {
        return;
    };
    if line < st.top {
        let mut t = line;
        while t > 0 && rows[t - 1].cells.is_empty() && line - (t - 1) < body_h {
            t -= 1;
        }
        st.top = t;
    } else if line >= st.top + body_h {
        st.top = line + 1 - body_h.max(1);
    }
    st.top = st.top.min(max);
}

/// ヘルプの画面: 見出し・本文・下の帯・メッセージ行。
pub(crate) fn render_help(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(st) = &app.help else {
        return Vec::new();
    };
    let body = help_rows(app, st.from, w);
    // screen.help-run: 選んでいる項目(行と項目)。
    let chosen = picks(&body).get(st.sel).copied();
    let look = super::look::look(app);
    let body_h = h.saturating_sub(3);
    let top = st.top.min(body.len().saturating_sub(body_h));
    let end = (top + body_h).min(body.len());
    let head = Msg::HelpHead.fill(&[&(top + 1), &end, &body.len()]);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let mut lines = vec![Line::from(Span::styled(fit(&head, w, Align::Left), bold))];
    for (i, row) in body.iter().enumerate().take(end).skip(top) {
        let mut text = fit(&row.text, w, Align::Left);
        // SR-33: モダンな見た目では、節の見出しとキーの欄をアクセントの色に(文字は同じ)。
        let base = match (&row.kind, &look) {
            (Kind::Head, Some(l)) => l.key(),
            (Kind::Head, None) => bold,
            _ => Style::default(),
        };
        let mut ranges: Vec<(usize, usize, Style)> = match (&row.kind, &look) {
            (Kind::Keys(keys), Some(l)) => keys.iter().map(|&(x, kw)| (x, kw, l.key())).collect(),
            _ => Vec::new(),
        };
        // screen.help-run: 選んでいる項目は反転し(モダンな見た目では選びの地)、左に印 `>` を付ける
        // (色だけで伝えない。cell-view.not-color-only)。
        if let Some((_, c)) = chosen.filter(|(l, _)| *l == i) {
            text = mark_at(&text, c.x.saturating_sub(2));
            ranges.push((c.x, c.w, Style::default().add_modifier(Modifier::REVERSED)));
        }
        lines.push(Line::from(styled(&text, base, &ranges)));
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

/// `text` の桁 `col` の空白を選びの印 `>` にする(桁は表示の幅で数える)。
fn mark_at(text: &str, col: usize) -> String {
    let mut out = String::with_capacity(text.len());
    let mut x = 0;
    for c in text.chars() {
        out.push(if x == col && c == ' ' { '>' } else { c });
        x += width(&c.to_string());
    }
    out
}

/// 行 `text` を span に分ける。地の見た目は `base`、(始まりの桁, 幅, 見た目)の範囲はその見た目を重ねる
/// (あとの範囲ほど上。桁は表示の幅で数える)。
fn styled(text: &str, base: Style, ranges: &[(usize, usize, Style)]) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cur = String::new();
    let mut cur_st = base;
    let mut x = 0;
    for c in text.chars() {
        let st = ranges
            .iter()
            .filter(|&&(s, w, _)| x >= s && x < s + w)
            .fold(base, |acc, &(_, _, r)| acc.patch(r));
        if st != cur_st && !cur.is_empty() {
            spans.push(Span::styled(std::mem::take(&mut cur), cur_st));
        }
        cur_st = st;
        cur.push(c);
        x += width(&c.to_string());
    }
    if !cur.is_empty() || spans.is_empty() {
        spans.push(Span::styled(cur, cur_st));
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
            Target::Goto(_) | Target::SaveQuit | Target::DiscardQuit => String::new(),
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
    let label_of = |a: Action| -> String {
        if let Some(l) = app.cond_label(a) {
            return l;
        }
        app.keys
            .iter()
            .find(|b| b.mode == Mode::Table && b.action == a)
            .map_or(a.name(), super::keymap::Binding::text)
            .to_string()
    };
    match q {
        "w" if !app.readonly => {
            out.push(cand(Target::Run(Action::Save), label_of(Action::Save), "w"));
        }
        "q" => out.push(cand(Target::Run(Action::Quit), label_of(Action::Quit), "q")),
        "wq" | "x" if !app.readonly => out.push(cand(
            Target::SaveQuit,
            Msg::PaletteSaveQuit.text().into(),
            q,
        )),
        // screen.quit-discard: vim と同じ打ち方で、ためた変更を書かずにすぐ終わる(vim モードによらない)。
        "q!" => out.push(cand(
            Target::DiscardQuit,
            Msg::PaletteDiscardQuit.text().into(),
            q,
        )),
        _ => {}
    }
    let mut scored: Vec<(i32, usize, Action)> = Vec::new();
    for b in app.keys.iter().filter(|b| b.mode == Mode::Table) {
        let a = b.action;
        if a == Action::Palette
            || (a.cond_digit().is_some() && app.cond_label(a).is_none())
            || write_command(app, a)
            || (!app.src.notes() && Action::NOTE_ONLY.contains(&a))
            || scored.iter().any(|s| s.2 == a)
            || out.iter().any(|c| c.target == Target::Run(a))
        {
            continue;
        }
        let best = [score(q, a.name()), score(q, &label_of(a))]
            .into_iter()
            .flatten()
            .max();
        if let Some(p) = best {
            scored.push((p, scored.len(), a));
        }
    }
    // BV-19: キーの無いコマンド(読むだけでは書くので出さない。ノートを書かない登録した表のは出す)。
    let mut commands: Vec<&keymap::Command> = Vec::new();
    for c in keymap::commands(app.readonly)
        .filter(|c| app.src.notes() || !Action::NOTE_ONLY.contains(&c.action))
        .filter(|c| app.command_shown(c.action))
    {
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
            .map_or_else(|| label_of(a), |c| c.text().to_string());
        out.push(cand(Target::Run(a), label, a.name()));
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
    // ヘッダーのすぐ下(SR-1 の上の余白の分だけ下がる)。
    let top = super::bands::head_y(app) as usize + 1;
    for (k, l) in out.into_iter().enumerate() {
        if let Some(slot) = lines.get_mut(top + k) {
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
    (x < w).then_some((x as u16, super::bands::head_y(app) + 1))
}

impl App {
    /// `?`: ヘルプを重ねる(SR-5)。先頭の項目を選んで開く(screen.help-run)。
    pub(crate) fn open_help(&mut self) {
        self.help = Some(HelpState {
            from: self.mode,
            top: 0,
            sel: 0,
        });
        self.set_mode(Mode::Help);
    }

    /// ヘルプの本文と、本文の高さ(描くときと同じ。`ui::draw` は右の1桁と最下行を空ける)。
    fn help_body(&self) -> Option<(Mode, Vec<Row>, usize)> {
        let from = self.help.as_ref()?.from;
        let w = self.size.0.saturating_sub(1) as usize;
        let body_h = (self.size.1.saturating_sub(1) as usize).saturating_sub(3);
        Some((from, help_rows(self, from, w), body_h))
    }

    /// ヘルプの中の動作: 閉じる・項目を選ぶ・選んだ「今押せるキー」の項目をする(screen.help-run)。
    pub(crate) fn help_action(&mut self, action: Action) {
        let Some((from, rows, body_h)) = self.help_body() else {
            return self.set_mode(Mode::Table);
        };
        let picks = picks(&rows);
        let last = picks.len().saturating_sub(1);
        let max = rows.len().saturating_sub(body_h);
        let Some(st) = &mut self.help else {
            return;
        };
        match action {
            Action::Close => {
                self.help = None;
                return self.set_mode(from);
            }
            // screen.help-run: ヘルプを閉じ、開いた元のモードでそのキーを押したのと同じ道(`apply`)で動作をする。
            // 下の節の項目(動作を持たない)では何もせず、ヘルプは開いたまま。
            Action::Run => {
                let Some(a) = picks.get(st.sel).and_then(|p| p.1.run) else {
                    return;
                };
                self.help = None;
                self.set_mode(from);
                return self.apply(a);
            }
            Action::Down => st.sel = (st.sel + 1).min(last),
            Action::Up => st.sel = st.sel.saturating_sub(1),
            Action::PageDown => {
                st.top = (st.top + body_h.max(1)).min(max);
                st.sel = page_pick(&picks, st.sel, body_h.max(1), true);
            }
            Action::PageUp => {
                st.top = st.top.saturating_sub(body_h.max(1));
                st.sel = page_pick(&picks, st.sel, body_h.max(1), false);
            }
            Action::Top => {
                st.sel = 0;
                st.top = 0;
            }
            Action::Bottom => {
                st.sel = last;
                st.top = max;
            }
            _ => return,
        }
        follow(st, &rows, &picks, body_h, max);
    }

    /// ヘルプのホイール(SR-6): 本文を1行流す。選んでいる項目が見えなくなったら、見えている一番近い項目へ移す。
    pub(crate) fn help_wheel(&mut self, down: bool) {
        let Some((_, rows, body_h)) = self.help_body() else {
            return;
        };
        let picks = picks(&rows);
        let max = rows.len().saturating_sub(body_h);
        let Some(st) = &mut self.help else {
            return;
        };
        st.top = if down {
            (st.top.min(max) + 1).min(max)
        } else {
            st.top.min(max).saturating_sub(1)
        };
        let Some(&(line, _)) = picks.get(st.sel) else {
            return;
        };
        let (first, end) = (st.top, st.top + body_h);
        if line < first {
            if let Some(i) = picks.iter().position(|p| p.0 >= first && p.0 < end) {
                st.sel = i;
            }
        } else if line >= end {
            if let Some(i) = picks.iter().rposition(|p| p.0 >= first && p.0 < end) {
                st.sel = i;
            }
        }
    }

    /// ヘルプのクリック(screen.help-run): 項目のクリックでその項目を選ぶ。選んでいる項目のクリックは Enter と同じ
    /// (表のセルの2段階のクリックと同じ)。項目の左の印の桁も項目に数える。
    pub(crate) fn help_click(&mut self, x: u16, y: u16) {
        let Some((_, rows, body_h)) = self.help_body() else {
            return;
        };
        let picks = picks(&rows);
        let max = rows.len().saturating_sub(body_h);
        let Some(st) = &mut self.help else {
            return;
        };
        // 画面の1行目は見出し、本文はその下の body_h 行。
        let (x, y) = (usize::from(x), usize::from(y));
        if y == 0 || y > body_h {
            return;
        }
        let line = st.top.min(max) + y - 1;
        let Some(i) = picks
            .iter()
            .position(|(l, c)| *l == line && x + 2 >= c.x && x < c.x + c.w)
        else {
            return;
        };
        if i == st.sel {
            return self.help_action(Action::Run);
        }
        st.sel = i;
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
        // navigation.across-progress: 横断の検索の入力を閉じたら(Esc・行を開く)、読んでいる途中でも止める。
        if matches!(self.nv.ask, Some(super::native_views::Ask::Across)) {
            self.across_stop();
        }
        // routine.screen-create: 作るノートの確かめを閉じたら(Esc)、やめたのと同じ。
        if matches!(self.nv.ask, Some(super::native_views::Ask::RoutineRuns(_))) {
            self.nv.ask = None;
            self.routine_dismiss();
        }
        // cell-edit.children-reopen: 子も戻すかの窓を閉じたら(Esc)、待っている直しをやめる。詳細の表示で直したなら
        // 詳細の表示に戻る。
        let reopen = matches!(
            self.nv.ask,
            Some(super::native_views::Ask::ChildrenReopen { .. })
        );
        if reopen {
            self.children_ask = None;
        }
        self.palette = None;
        self.nv.ask = None;
        // TR-5: 木(ほかの表でない画面の型)から開いたパレットは、閉じるとその画面に戻る。
        let back = self.screen().mode();
        self.set_mode(if reopen && back == Mode::Table && self.detail.is_some() {
            Mode::Detail
        } else {
            back
        });
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
                        // NV-7: 行へ跳ぶのは表の行なので、表でない画面の型からは表の画面に戻ってから。
                        Target::Goto(n) => {
                            self.close_screen();
                            self.goto_line(n);
                        }
                        // 終了の確認で「保存する」を選んだのと同じ道(差分の確認 → 確定で書いて終わる)。
                        // ためた変更が無ければそのまま終わる。`q` と同じく、表を移る予約は捨てる。
                        Target::SaveQuit => {
                            self.close_screen();
                            self.drop_switch();
                            self.review_action(Action::QuitSave);
                        }
                        // screen.quit-discard: 確かめずに終わる(ノートは書かない)。表を移る予約も捨てる。
                        Target::DiscardQuit => {
                            self.close_screen();
                            self.drop_switch();
                            self.quit = true;
                        }
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
