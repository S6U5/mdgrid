//! 通知の欄(notice.*。`impl App` の続き)。設定の `[[notices]]` の項目を数えて読み、ヘッダーの上に出す。
//!
//! 今の表だけを数える項目は、その場で今の表の行(`.base` の最上位の絞り込みといつも隠す条件のあと)を数える。
//! ほかの表を含む項目は、裏の糸で表をファイルから開いて数え(`crate::count_tables`)、数え終わるまで「…」を出す
//! (notice.background)。数え直すのは、開いたとき・保存したあと・外の変更を読んだとき・エディタから戻ったとき・
//! 日付が変わったとき(notice.refresh)。

use super::app::App;
use super::keymap::Mode;
use super::native_views::Ask;
use mdgrid::expr::Expr;
use mdgrid::i18n::Msg;
use mdgrid::notice::{self, Item, Scope};
use mdgrid::places::Place;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

/// 表ごとの数(表の名前, パス, 数か読めなかった理由)。
pub(crate) type PerTable = Vec<(String, PathBuf, Result<usize, String>)>;

/// 裏の糸で数える項目(項目の添字, 数える表(名前, パス), 条件の式)。
type Job = (usize, Vec<(String, PathBuf)>, Expr);

/// 通知の欄の状態。
#[derive(Default)]
pub(crate) struct NoticeState {
    pub items: Vec<Item>,
    /// 件数の項目ごとの表ごとの数(None は数えている途中。件数の項目でなければ空)。
    pub counts: Vec<Option<PerTable>>,
    /// 連絡の項目ごとの行(読めなければ知らせの1行)。
    pub notes: Vec<Vec<String>>,
    /// 裏の糸の結果(数えた世代, 項目, 表ごとの数)。
    rx: Option<Receiver<(usize, usize, PerTable)>>,
    generation: usize,
    /// 欄の中で選んでいる項目(`entries` の添字)。
    pub sel: usize,
    /// 数え直しが要る(外の変更を読んだ。描く前の `notice_poll` で数え直す)。
    pub dirty: bool,
}

/// 欄に出す項目の1つ(項目の添字と、出す文の行)。
pub(crate) struct Entry {
    pub item: usize,
    pub lines: Vec<String>,
    pub count: bool,
}

impl App {
    /// 設定の `[[notices]]` を当てて数える(App::configure から)。
    pub(crate) fn configure_notices(&mut self, items: Vec<Item>) {
        self.notice.items = items;
        self.notice_refresh();
    }

    /// notice.refresh: 全部の項目を数え直し、読み直す。
    pub(crate) fn notice_refresh(&mut self) {
        if self.notice.items.is_empty() {
            return;
        }
        self.notice.generation += 1;
        let generation = self.notice.generation;
        let n = self.notice.items.len();
        self.notice.counts = vec![None; n];
        self.notice.notes = vec![Vec::new(); n];
        let mut jobs: Vec<Job> = Vec::new();
        for k in 0..n {
            match self.notice.items[k].clone() {
                Item::Note { path, lines } => {
                    self.notice.notes[k] =
                        notice::note_lines(&path, lines).unwrap_or_else(|e| vec![e]);
                }
                Item::Count { cond, scope, .. } => {
                    let Some(e) = self.cond.all.expr(&cond).cloned() else {
                        self.notice.counts[k] = Some(Vec::new());
                        continue;
                    };
                    match scope {
                        Scope::Table => {
                            let here = self.notice_here_count(&e);
                            self.notice.counts[k] =
                                Some(vec![(self.table_name(), PathBuf::new(), Ok(here))]);
                        }
                        Scope::Workspace => jobs.push((k, self.notice_places(None), e)),
                        Scope::Tables(names) => jobs.push((k, self.notice_places(Some(&names)), e)),
                    }
                }
            }
        }
        if jobs.is_empty() {
            self.notice.rx = None;
            return;
        }
        let (tx, rx) = mpsc::channel();
        let hides: Vec<Expr> = self
            .hiding()
            .iter()
            .filter_map(|c| self.cond.all.expr(&c.name).cloned())
            .collect();
        let (today, now) = (self.today, self.now);
        std::thread::spawn(move || {
            for (k, places, e) in jobs {
                let per = crate::count_tables(&places, &e, &hides, today, now);
                if tx.send((generation, k, per)).is_err() {
                    return;
                }
            }
        });
        self.notice.rx = Some(rx);
    }

    /// 裏の糸の結果を拾う(描く前に呼ぶ)。古い世代の結果は捨てる。
    pub(crate) fn notice_poll(&mut self) {
        if std::mem::take(&mut self.notice.dirty) {
            self.notice_refresh();
        }
        let Some(rx) = &self.notice.rx else {
            return;
        };
        while let Ok((generation, k, per)) = rx.try_recv() {
            if generation == self.notice.generation {
                if let Some(slot) = self.notice.counts.get_mut(k) {
                    *slot = Some(per);
                }
            }
        }
    }

    /// notice.cli・notice.cli-plain: `--notices` の行(1項目1行、連絡は1行ずつ。飾りなし)。ほかの表を数える
    /// 項目は数え終わるまで待つ。
    pub(crate) fn notice_report(&mut self) -> Vec<String> {
        self.notice_refresh();
        if let Some(rx) = self.notice.rx.take() {
            while self.notice.counts.iter().any(Option::is_none) {
                let Ok((generation, k, per)) = rx.recv() else {
                    break;
                };
                if generation == self.notice.generation {
                    if let Some(slot) = self.notice.counts.get_mut(k) {
                        *slot = Some(per);
                    }
                }
            }
        }
        // 制御の文字(ESC など)は見える図記号に置き換え、端末の帯にそのまま流さない(画面の欄と同じ)。
        self.notice_entries()
            .into_iter()
            .flat_map(|e| e.lines)
            .map(|l| super::width::sanitize(&l))
            .collect()
    }

    /// 今の表の行のうち、`.base` の最上位の絞り込みといつも隠す条件を通り、`e` に当たる行の数(notice.count-rows)。
    fn notice_here_count(&self, e: &Expr) -> usize {
        let hiding: Vec<String> = self.hiding().iter().map(|c| c.name.clone()).collect();
        let prop = |r: &mdgrid::source::RowId, k: &str| self.prop(r, k);
        self.src
            .rows()
            .iter()
            .filter(|r| {
                self.base.as_ref().is_none_or(|b| {
                    b.base
                        .keeps(self.src.as_ref(), &prop, r, self.today, self.now)
                })
            })
            .filter(|r| !hiding.iter().any(|n| self.hits(r, n)))
            .filter(|r| self.eval_true(r, e))
            .count()
    }

    /// 数える表(範囲の表の全部か、名前を挙げた表)。
    fn notice_places(&self, names: Option<&[String]>) -> Vec<(String, PathBuf)> {
        self.scope_places()
            .into_iter()
            .filter(|p| names.is_none_or(|ns| ns.contains(&p.name)))
            .map(|p| (p.name, p.path))
            .collect()
    }

    /// 今の表の名前(ヘッダーのパンくずの表の名前と同じ)。
    fn table_name(&self) -> String {
        match &self.base {
            Some(b) => b.name.clone(),
            None => self.src.name(),
        }
    }

    /// 欄に出す項目(書いた順。0 件で `show_zero` でない件数の項目と、行の無い連絡の項目は出さない)。
    pub(crate) fn notice_entries(&self) -> Vec<Entry> {
        let mut out = Vec::new();
        for (k, item) in self.notice.items.iter().enumerate() {
            match item {
                Item::Count {
                    cond,
                    text,
                    show_zero,
                    ..
                } => {
                    let label = self
                        .cond
                        .all
                        .get(cond)
                        .map_or_else(|| cond.clone(), |c| c.shown().to_string());
                    let per = self.notice.counts.get(k).and_then(Option::as_ref);
                    let (count, partial) = match per {
                        None => (None, false),
                        Some(per) => (
                            Some(
                                per.iter()
                                    .filter_map(|(_, _, r)| r.as_ref().ok())
                                    .sum::<usize>(),
                            ),
                            per.iter().any(|(_, _, r)| r.is_err()),
                        ),
                    };
                    if count == Some(0) && !show_zero && !partial {
                        continue;
                    }
                    out.push(Entry {
                        item: k,
                        lines: vec![notice::count_text(text.as_deref(), &label, count, partial)],
                        count: true,
                    });
                }
                Item::Note { .. } => {
                    let lines = self.notice.notes.get(k).cloned().unwrap_or_default();
                    if !lines.is_empty() {
                        out.push(Entry {
                            item: k,
                            lines,
                            count: false,
                        });
                    }
                }
            }
        }
        out
    }

    /// notice.layout: 欄の行(件数の項目を1行目に ` · ` でつなぎ、連絡の行を1行ずつ。3行まで)。
    pub(crate) fn notice_lines(&self) -> Vec<String> {
        let entries = self.notice_entries();
        let counts: Vec<String> = entries
            .iter()
            .filter(|e| e.count)
            .flat_map(|e| e.lines.clone())
            .collect();
        let notes: Vec<String> = entries
            .iter()
            .filter(|e| !e.count)
            .flat_map(|e| e.lines.clone())
            .collect();
        notice::layout(&counts, &notes)
    }

    /// 欄の行の数(出す項目が無ければ 0)。
    pub(crate) fn notice_rows(&self) -> usize {
        // stack.title: 縦に並べた表の区画は、通知の欄を持たない(選んでいる表を画面いっぱいに出すと出る)。
        if self.notice.items.is_empty() || super::stack::in_region(self) {
            return 0;
        }
        self.notice_lines().len()
    }

    /// notice.focus: 欄へ移る。欄が無ければ何もしない。
    pub(crate) fn focus_notice(&mut self) {
        if self.notice_entries().is_empty() {
            return;
        }
        self.notice.sel = 0;
        self.set_mode(Mode::Notice);
    }

    /// 欄の中の動作(←→・↑↓ で項目、Enter で移る、Esc で表へ)。
    pub(crate) fn notice_action(&mut self, action: super::keymap::Action) {
        use super::keymap::Action;
        let n = self.notice_entries().len();
        match action {
            Action::Left | Action::Up => self.notice.sel = self.notice.sel.saturating_sub(1),
            Action::Right | Action::Down => {
                self.notice.sel = (self.notice.sel + 1).min(n.saturating_sub(1));
            }
            Action::Run => self.notice_enter(),
            Action::Close => self.set_mode(Mode::Table),
            Action::Help => self.open_help(),
            _ => {}
        }
        if n == 0 && self.mode == Mode::Notice {
            self.set_mode(Mode::Table);
        }
    }

    /// notice.focus: 欄の行 `y`・桁 `x` のクリック。件数の行は桁で項目を選ぶ。選んでいた項目をもう一度押すと移る。
    pub(crate) fn notice_click(&mut self, x: u16, y: u16) {
        let entries = self.notice_entries();
        let counts: Vec<usize> = (0..entries.len()).filter(|&i| entries[i].count).collect();
        let x = usize::from(x);
        let y = usize::from(y);
        let hit = if !counts.is_empty() && y == 0 {
            // 1行目は「 項目 · 項目」。桁で項目を探す。
            let mut at = 1;
            let mut found = counts[0];
            for (k, &i) in counts.iter().enumerate() {
                if k > 0 {
                    at += 3;
                }
                let w = super::width::width(&entries[i].lines.concat());
                if x < at + w {
                    found = i;
                    break;
                }
                at += w;
                found = i;
            }
            Some(found)
        } else {
            // 連絡の行は、行ごとに項目を数える。
            let mut line = usize::from(!counts.is_empty());
            let mut found = None;
            for (i, e) in entries.iter().enumerate().filter(|(_, e)| !e.count) {
                if y < line + e.lines.len() {
                    found = Some(i);
                    break;
                }
                line += e.lines.len();
            }
            found
        };
        let Some(i) = hit else {
            return;
        };
        if self.mode == Mode::Notice && self.notice.sel == i {
            return self.notice_enter();
        }
        self.notice.sel = i;
        self.set_mode(Mode::Notice);
    }

    /// notice.jump・notice.open-note: 選んだ項目へ移る。
    fn notice_enter(&mut self) {
        let entries = self.notice_entries();
        let Some(e) = entries.get(self.notice.sel) else {
            return;
        };
        match self.notice.items[e.item].clone() {
            Item::Note { path, .. } => {
                self.set_mode(Mode::Table);
                self.editor_note = Some(path);
            }
            Item::Count { cond, scope, .. } => {
                self.set_mode(Mode::Table);
                if scope == Scope::Table {
                    self.narrow_by_condition(&cond);
                    return;
                }
                let per = self
                    .notice
                    .counts
                    .get(e.item)
                    .cloned()
                    .flatten()
                    .unwrap_or_default();
                let hits: Vec<(String, PathBuf, usize)> = per
                    .into_iter()
                    .filter_map(|(name, path, r)| {
                        r.ok().filter(|n| *n > 0).map(|n| (name, path, n))
                    })
                    .collect();
                if hits.is_empty() {
                    return;
                }
                let items: Vec<String> = hits.iter().map(|(n, _, c)| format!("{n} {c}")).collect();
                let places: Vec<Place> = hits
                    .into_iter()
                    .map(|(name, path, _)| Place {
                        name,
                        group: String::new(),
                        path,
                        view: None,
                    })
                    .collect();
                self.open_ask(Ask::NoticeTables {
                    items,
                    places,
                    cond,
                });
            }
        }
    }

    /// notice.jump: 今の表を名前の条件で絞る(数字のキーで入れたときと同じく、ほかの表へ移ると解ける)。
    pub(crate) fn narrow_by_condition(&mut self, name: &str) {
        if !self.cond.on.iter().any(|n| n == name) {
            self.cond.on.push(name.to_string());
        }
        self.stay.clear();
        self.refresh();
    }

    /// notice.jump: 一覧で選んだ表を、その条件で絞って開く(ためた変更があれば終わる手順で確かめる)。
    pub(crate) fn notice_open_table(&mut self, place: Place, cond: String) {
        self.close_palette();
        if self.notice_is_here(&place.path) {
            return self.narrow_by_condition(&cond);
        }
        if !mdgrid::workspace::openable(&place.path) {
            self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
            return;
        }
        self.switch_to = Some(place);
        self.switch_select = None;
        self.switch_cond = Some(cond);
        self.begin_quit();
    }

    fn notice_is_here(&self, path: &std::path::Path) -> bool {
        super::drawer::here(self).is_some_and(|h| mdgrid::workspace::same_path(path, &h))
    }
}

/// notice.layout・notice.focus: 欄の行。件数の項目は1行目に ` · ` でつなぐ。欄を選んでいるときは、選んだ項目を
/// 反転し、頭に `>` を付ける(色だけに頼らない)。3行を超える分は `notice::layout` と同じく「ほか N 行」。
pub(crate) fn notice_band(app: &App, w: usize) -> Vec<ratatui::text::Line<'static>> {
    use super::width::{fit, sanitize, Align};
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    if app.notice.items.is_empty() {
        return Vec::new();
    }
    let entries = app.notice_entries();
    if entries.is_empty() {
        return Vec::new();
    }
    let focused = app.mode == Mode::Notice;
    let dim = Style::default().add_modifier(Modifier::DIM);
    let pick = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let shown = app.notice_lines();
    let mut out: Vec<Line<'static>> = Vec::new();
    // 1行目: 件数の項目。
    let counts: Vec<(usize, &Entry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.count)
        .collect();
    if !counts.is_empty() {
        let mut spans = vec![Span::raw(" ")];
        for (k, (i, e)) in counts.iter().enumerate() {
            if k > 0 {
                spans.push(Span::styled(" · ", dim));
            }
            let sel = focused && *i == app.notice.sel;
            let text = sanitize(&e.lines.concat());
            spans.push(Span::styled(
                if sel { format!(">{text}") } else { text },
                if sel { pick } else { Style::default() },
            ));
        }
        out.push(fit_line(Line::from(spans), w));
    }
    // 連絡の行(項目ごと)。
    for (i, e) in entries.iter().enumerate().filter(|(_, e)| !e.count) {
        let sel = focused && i == app.notice.sel;
        for (k, l) in e.lines.iter().enumerate() {
            let lead = if sel && k == 0 { ">" } else { " " };
            let st = if sel { pick } else { Style::default() };
            out.push(Line::from(Span::styled(
                fit(&format!("{lead}{}", sanitize(l)), w, Align::Left),
                st,
            )));
        }
    }
    // 3行に収める(最後の行は「ほか N 行」。文は notice::layout と同じ)。
    if out.len() > shown.len() {
        out.truncate(shown.len().saturating_sub(1));
        if let Some(last) = shown.last() {
            out.push(Line::from(Span::styled(
                fit(&format!(" {last}"), w, Align::Left),
                dim,
            )));
        }
    }
    out
}

/// 1行を幅 `w` に切る(切った所に `…`)。
fn fit_line(line: ratatui::text::Line<'static>, w: usize) -> ratatui::text::Line<'static> {
    use super::width::width;
    let total: usize = line.spans.iter().map(|s| width(&s.content)).sum();
    if total <= w {
        return line;
    }
    let mut used = 0;
    let mut spans = Vec::new();
    for s in line.spans {
        let sw = width(&s.content);
        if used + sw < w {
            used += sw;
            spans.push(s);
        } else {
            let room = w.saturating_sub(used);
            let cut = super::width::fit(&s.content, room, super::width::Align::Left);
            spans.push(ratatui::text::Span::styled(cut, s.style));
            break;
        }
    }
    ratatui::text::Line::from(spans)
}
