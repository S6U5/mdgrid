//! 縦に並べる(stack.*): 別の表を1つの画面に上下に並べ、選んでいる表にだけキーを効かせる。形は
//! docs/design.md の「縦に並べる」。
//!
//! - 表ごとの状態は、今までどおり1つの `App`。選んでいる表の App が外側(画面を回す側が持つ App)で、この
//!   `Stack` を持ち、ほかの表の App を並びの中に持つ。選ぶ表を移るときは App の中身を入れ替える
//!   (`std::mem::swap`)。だから検索・絞り込み・並べ替え・ビュー・セルの編集・新しいノートは、どれも今までの道の
//!   まま選んでいる表にだけ効く(stack.each-table)。
//! - ためた変更(`Changes`)は1つで、選んでいる表の App が持つ。ほかの表を組み直す・描く・読み直すときだけ、その
//!   表の App に一時的に渡して返す(`with_table`)。ほかの表は、ためた変更の世代(`Changes::generation`)が
//!   変わったときに組み直す(stack.shared-pending)。
//! - 保存は、全部の表の読み込み口を1つにまとめた `AllSources`(行ごとに、その行を読んだ表へ渡す)で、1つの
//!   差分の確認に表ごとの見出しを付けて出す(stack.save-all)。書き戻しの約束(外の変更で止める)は今までの
//!   `Changes::save` のまま。
//! - 画面は、表ごとに「見出しの1行 + 列の見出し + 行」の区画を上から並べ、下の帯とメッセージ行は選んでいる表の
//!   もの(`render`)。表の区画は、その表の App を区画の高さの画面として描く(`App::size` を区画の大きさにする)。
//!   表でない画面の型と、画面いっぱいの窓(ヘルプ・パレット・保存の確認・設定・引き出しなど)は、選んでいる表
//!   だけを画面いっぱいに出す(`full_screen`)。

use super::app::App;
use super::keymap::{self, Action, Mode};
use super::native_io::Ask;
use super::screens::Screen;
use super::view;
use super::width::{fit, sanitize, Align};
use mdgrid::changes::Changes;
use mdgrid::i18n::Msg;
use mdgrid::page::{Page, PageTable};
use mdgrid::places::Place;
use mdgrid::source::{
    Cell, ColumnKind, Edit, EditError, FileInfo, RowId, SaveError, Source, Stamp, Value,
};
use mdgrid::workspace::{self, PageHome};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashSet;
use std::path::PathBuf;

/// 並べられる表の数(stack.limit)。
pub(crate) const MAX_TABLES: usize = 4;
/// 表1つに要る高さ: 見出しの1行と、その下の3行(stack.limit・stack.resize)。
pub(crate) const MIN_H: usize = 4;
/// 区画の外の行(下の帯とメッセージ行)。
const CHROME: usize = 2;

/// 画面を回す側(main)に頼むこと。表を開くのは main の `open_app` なので、画面を一度抜けて開いてから戻る
/// (登録した表へ移るとき(CLI-19)と同じ道)。
#[derive(Debug, Clone, PartialEq)]
pub enum StackRequest {
    /// 今の表の下にこの表を並べる(stack.add)。
    Add(Place),
    /// このページを開く(stack.page-open)。
    Page(String),
}

/// 並べた表。選んでいる表の App が持つ。
pub(crate) struct Stack {
    /// 並べた表(上から)。選んでいる表の場所は None(その App は外側で、この Stack を持つ)。
    pub(crate) tables: Vec<Option<App>>,
    /// 選んでいる表の位置。
    pub(crate) focus: usize,
    /// 高さの比(stack.resize・stack.page-save)。`tables` と同じ並び。
    pub(crate) weights: Vec<u32>,
    /// 開いているページ(stack.page-open・stack.page-resume)。表を足したり閉じたりしたら外す。
    pub(crate) page: Option<String>,
    /// 閉じようとしている表の、ほかの表に無いためた変更の行(stack.close-pending の確かめの間)。
    pub(crate) closing: Option<Vec<RowId>>,
    /// 端末の大きさ(最下行と右端の1桁を含む)。
    pub(crate) screen: (u16, u16),
    /// 最後に大きさを合わせたとき、選んでいる表を画面いっぱいに出していたか。
    pub(crate) full: bool,
    /// ほかの表を組み直したときの、ためた変更の世代(`tables` と同じ並び)。
    pub(crate) seen: Vec<u64>,
    /// 画面のいちばん上の通知の欄(notice.layout。選んでいる表のもの)の行の数。区画はその下から。
    pub(crate) top: usize,
}

impl Stack {
    fn new(screen: (u16, u16)) -> Stack {
        Stack {
            tables: vec![None],
            focus: 0,
            weights: vec![1],
            page: None,
            closing: None,
            screen,
            full: false,
            seen: vec![u64::MAX],
            top: 0,
        }
    }

    /// 区画の高さ(見出しを含む。上から)。
    pub(crate) fn heights(&self) -> Vec<usize> {
        let avail = usize::from(self.screen.1).saturating_sub(1 + CHROME + self.top);
        heights(&self.weights, self.focus, avail)
    }

    /// 区画の始まりの行と高さ。
    fn regions(&self) -> Vec<(usize, usize)> {
        let mut y = self.top;
        self.heights()
            .into_iter()
            .map(|h| {
                let r = (y, h);
                y += h;
                r
            })
            .collect()
    }
}

/// stack.resize・stack.limit: 高さ `avail` の行を、比 `weights` で分ける(見出しを含む)。どの表にも見出しと3行が
/// 入らなければ、下の表から順に(選んでいる表 `focus` は残して)見出しの1行だけにする。それでも入らなければ
/// 上から1行ずつ。比で分けた端数は上の表から1行ずつ足し、見出しと3行に満たない表は、余っている表から1行ずつ
/// もらう。
pub(crate) fn heights(weights: &[u32], focus: usize, avail: usize) -> Vec<usize> {
    let n = weights.len();
    if n == 0 {
        return Vec::new();
    }
    let mut open: Vec<usize> = (0..n).collect();
    while open.len() * MIN_H + (n - open.len()) > avail {
        let Some(at) = open.iter().rposition(|&k| k != focus) else {
            break;
        };
        open.remove(at);
    }
    let rest = avail.saturating_sub(n - open.len());
    if open.len() * MIN_H > rest {
        // 選んでいる表の見出しと3行も入らない: 上から1行ずつ。
        return (0..n).map(|k| usize::from(k < avail)).collect();
    }
    let sum: u64 = open.iter().map(|&k| u64::from(weights[k].max(1))).sum();
    let mut hs: Vec<usize> = open
        .iter()
        .map(|&k| usize::try_from(rest as u64 * u64::from(weights[k].max(1)) / sum).unwrap_or(0))
        .collect();
    let mut left = rest.saturating_sub(hs.iter().sum());
    let mut i = 0;
    while left > 0 {
        hs[i % open.len()] += 1;
        left -= 1;
        i += 1;
    }
    while let Some(lo) = hs.iter().position(|&h| h < MIN_H) {
        let Some(hi) = (0..hs.len())
            .filter(|&j| hs[j] > MIN_H)
            .max_by_key(|&j| (hs[j], std::cmp::Reverse(j)))
        else {
            break;
        };
        hs[hi] -= 1;
        hs[lo] += 1;
    }
    let mut out = vec![1; n];
    for (i, &k) in open.iter().enumerate() {
        out[k] = hs[i];
    }
    out
}

/// 区画の中で描く画面か(並べた表で、選んでいる表を画面いっぱいに出していない)。
pub(crate) fn in_region(app: &App) -> bool {
    app.stacked && !full_screen(app)
}

/// 選んでいる表を画面いっぱいに出す状態か: 表でない画面の型と、画面いっぱいの窓(ヘルプ・パレット・保存と
/// 終了の確認・詳細・設定・引き出し・絞り込みと並べ替えの窓・新しいノートの窓など)。区画の中で描くのは、表と、
/// セルに重ねる入力(編集・候補・リストの選択・操作の一覧・頻度表・列の型)と、メッセージ行で打つ検索と、
/// 画面の上の通知の欄だけ。
pub(crate) fn full_screen(app: &App) -> bool {
    !matches!(
        app.mode,
        Mode::Table
            | Mode::Edit
            | Mode::Search
            | Mode::Filter
            | Mode::ListPick
            | Mode::Menu
            | Mode::Freq
            | Mode::ColumnType
            | Mode::Notice
    ) || app.drawer.is_some()
        || app.detail.is_some()
        || app.draft.is_some()
        || app.review.is_some()
        || app.note.flow.is_some()
        || app.screen() != Screen::Table
}

/// 表の開いたパス(`.base` か、フォルダの並び)。
pub(crate) fn target_paths(app: &App) -> Vec<PathBuf> {
    app.nv
        .target
        .to_string_lossy()
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// 範囲(ワークスペース)の表としての名前: 並べるときに選んだ名前、無ければ開いたパスとビューで範囲の表を引く
/// (resume.herdr-title-form と同じ引き方)。範囲に無ければ None。
pub(crate) fn scope_name(app: &App) -> Option<String> {
    if let Some(n) = &app.place_name {
        return Some(n.clone());
    }
    let tables = &app.scope.as_ref()?.tables;
    let paths = target_paths(app);
    let name = mdgrid::resume::table_name(&paths, tables, app.current_view_name().as_deref());
    tables.iter().any(|t| t.name == name).then_some(name)
}

/// 見出しの表の名前(stack.title): 範囲の表の名前、無ければ `.base` のファイル名かフォルダの名前。
pub(crate) fn table_label(app: &App) -> String {
    if let Some(n) = scope_name(app) {
        return n;
    }
    let paths = target_paths(app);
    if paths.is_empty() {
        return app.src.name();
    }
    mdgrid::resume::table_name(&paths, &[], None)
}

/// 見出しのビューの名前(ビューの無い表では None)。
fn view_label(app: &App) -> Option<String> {
    let names = app.view_names();
    names.get(app.view_index()).cloned()
}

/// stack.title: 並べた表の見出しの1行「▶ projects · 進行中 · 4」。選んでいる表には頭に選びの印と太字、
/// ほかの表は印の代わりに空白で薄く。読み込みと画面の中の絞り込みは、ヘッダーと同じ形で後ろに添える。
pub(crate) fn title_line(app: &App, w: usize) -> Line<'static> {
    let focused = app.stack.is_some();
    let mark = match (focused, app.ambiguous_wide) {
        (true, false) => "▶ ",
        (true, true) => "> ",
        (false, _) => "  ",
    };
    let mut t = format!("{mark}{}", table_label(app));
    if let Some(v) = view_label(app) {
        t.push_str(&format!(" · {v}"));
    }
    t.push_str(&format!(" · {}", app.rows.len()));
    t.push_str(&super::bands::header_extra(app));
    let st = match (focused, super::look::look(app)) {
        (true, Some(l)) => Style::default().fg(l.accent).add_modifier(Modifier::BOLD),
        (true, None) => Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
        (false, _) => Style::default().add_modifier(Modifier::DIM),
    };
    Line::from(Span::styled(fit(&sanitize(&t), w, Align::Left), st))
}

/// 表1つの区画の行(高さ `h`)。1行なら見出しだけ。
fn region(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let mut lines = if h <= 1 {
        vec![title_line(app, w)]
    } else {
        view::render(app, w, h)
    };
    lines.truncate(h);
    lines
}

/// 並べた表の画面(高さ `h` = 描く行の数): いちばん上に通知の欄(notice.layout)、その下に区画を上から並べ、
/// 下の帯とメッセージ行は選んでいる表のもの。入力のカーソルの位置も返す(検索はメッセージ行、セルの入力は
/// 選んでいる表の区画の中)。
pub(crate) fn render(
    app: &mut App,
    w: usize,
    h: usize,
) -> (Vec<Line<'static>>, Option<(u16, u16)>) {
    let Some((weights, focus)) = app.stack.as_ref().map(|s| (s.weights.clone(), s.focus)) else {
        return (view::render(app, w, h), view::cursor(app));
    };
    let mut lines: Vec<Line<'static>> = super::notices::notice_band(app, w);
    let body = h.saturating_sub(CHROME);
    let hs = heights(&weights, focus, body.saturating_sub(lines.len()));
    let blank = || Line::from(" ".repeat(w));
    let mut cursor = None;
    for (k, &hk) in hs.iter().enumerate() {
        let y0 = lines.len();
        if hk == 0 {
            continue;
        }
        let part = if k == focus {
            if let Some((cx, cy)) = view::cursor(app) {
                cursor = if matches!(app.mode, Mode::Search | Mode::Filter) {
                    u16::try_from(h.saturating_sub(1)).ok().map(|y| (cx, y))
                } else if usize::from(cy) < hk {
                    u16::try_from(y0 + usize::from(cy)).ok().map(|y| (cx, y))
                } else {
                    None
                };
            }
            // cell-view.url-full-target: 区画の中で覚えた URL の所を、区画の行の分だけ下へずらす。
            let mark = app.url_spots.len();
            let part = region(app, w, hk);
            app.url_spots.shift(mark, 0, y0);
            part
        } else {
            let (part, mut spots) = app
                .with_table(k, |o| {
                    o.url_spots.clear();
                    (region(o, w, hk), o.url_spots.take())
                })
                .unwrap_or_default();
            for s in &mut spots {
                s.y += y0;
            }
            app.url_spots.extend(spots);
            part
        };
        lines.extend(part);
        while lines.len() < y0 + hk {
            lines.push(blank());
        }
    }
    while lines.len() < body {
        lines.push(blank());
    }
    lines.truncate(body);
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines.truncate(h);
    (lines, cursor)
}

/// 終える前の確かめ(stack.close-pending)の下の帯のキーの案内。閉じる確かめの間だけ。
pub(crate) fn close_hints(app: &App) -> Option<Vec<String>> {
    app.stack.as_ref()?.closing.as_ref()?;
    if app.mode != Mode::Quit {
        return None;
    }
    let key = |a| keymap::key_for(&app.keys, Mode::Quit, a).map(|k| keymap::display(&k));
    let mut out = Vec::new();
    for (a, m) in [
        (Action::QuitSave, Msg::KeyStackCloseSave),
        (Action::QuitDiscard, Msg::KeyStackCloseDiscard),
        (Action::Back, Msg::KeyBack),
    ] {
        if let Some(k) = key(a) {
            out.push(format!("{k} {}", m.text()));
        }
    }
    Some(out)
}

/// 閉じる確かめの帯の文(stack.close-pending)。閉じる確かめの間だけ。
pub(crate) fn close_band(app: &App) -> Option<String> {
    let rows = app.stack.as_ref()?.closing.as_ref()?;
    Some(Msg::StackCloseBand.fill(&[&table_label(app), &rows.len()]))
}

impl App {
    /// 並べた表の数(並べていなければ 1)。
    pub(crate) fn stack_len(&self) -> usize {
        self.stack.as_ref().map_or(1, |s| s.tables.len())
    }

    /// 並べた表の `k` 番目(選んでいる表でないもの)に、ためた変更を渡して `f` を当てる。選んでいる表や無い位置は
    /// None。
    pub(crate) fn with_table<R>(&mut self, k: usize, f: impl FnOnce(&mut App) -> R) -> Option<R> {
        let mut st = self.stack.take()?;
        let out = match st.tables.get_mut(k).and_then(Option::as_mut) {
            Some(o) => {
                std::mem::swap(&mut self.changes, &mut o.changes);
                let r = f(o);
                std::mem::swap(&mut self.changes, &mut o.changes);
                Some(r)
            }
            None => None,
        };
        self.stack = Some(st);
        out
    }

    /// ほかの表の位置(選んでいる表を除く)。
    fn other_tables(&self) -> Vec<usize> {
        match &self.stack {
            Some(st) => (0..st.tables.len()).filter(|&k| k != st.focus).collect(),
            None => Vec::new(),
        }
    }

    /// stack.add: 開いた表 `next` を、選んでいる表の下に並べて選ぶ(main が開いて渡す)。
    pub fn stack_push(&mut self, mut next: App) {
        let screen = self.stack.as_ref().map_or(self.size, |s| s.screen);
        next.stacked = true;
        next.term_bg = self.term_bg;
        self.stacked = true;
        let mut st = self
            .stack
            .take()
            .unwrap_or_else(|| Box::new(Stack::new(screen)));
        let at = st.focus + 1;
        st.tables.insert(at, Some(next));
        st.seen.insert(at, u64::MAX);
        st.weights = vec![1; st.tables.len()];
        st.page = None;
        st.full = !full_screen(self);
        self.stack = Some(st);
        self.stack_focus(at);
    }

    /// stack.page-open・stack.page-cli: 開いた表を上から並べた App(最初の表を選ぶ)。表が1つなら並べない。
    pub fn stack_of(mut apps: Vec<App>, weights: Vec<u32>, page: Option<String>) -> Option<App> {
        if apps.is_empty() {
            return None;
        }
        let mut first = apps.remove(0);
        if apps.is_empty() {
            return Some(first);
        }
        let mut st = Stack::new(first.size);
        first.stacked = true;
        for mut a in apps {
            a.stacked = true;
            st.tables.push(Some(a));
            st.seen.push(u64::MAX);
        }
        st.weights = (0..st.tables.len())
            .map(|k| weights.get(k).copied().unwrap_or(1).max(1))
            .collect();
        st.page = page;
        st.full = !full_screen(&first);
        first.stack = Some(Box::new(st));
        Some(first)
    }

    /// stack.focus: `j` 番目の表を選ぶ。App の中身を入れ替え、ためた変更と git に渡していないノートを渡す。
    pub(crate) fn stack_focus(&mut self, j: usize) {
        let Some(mut st) = self.stack.take() else {
            return;
        };
        if j == st.focus || j >= st.tables.len() {
            self.stack = Some(st);
            return;
        }
        let Some(mut other) = st.tables[j].take() else {
            self.stack = Some(st);
            return;
        };
        std::mem::swap(&mut self.changes, &mut other.changes);
        other.git_pending.append(&mut self.git_pending);
        other.term_bg = self.term_bg;
        self.prefix = None;
        let gen = other.changes.generation();
        let stale = st.seen.get(j) != Some(&gen);
        std::mem::swap(self, &mut other);
        // other は前に選んでいた表(今のためた変更で組んである)。
        other.message = None;
        st.tables[st.focus] = Some(other);
        if let Some(s) = st.seen.get_mut(st.focus) {
            *s = gen;
        }
        st.focus = j;
        st.full = !full_screen(self);
        self.stack = Some(st);
        self.message = None;
        if stale {
            self.refresh();
        }
        self.fit_stack_size();
        self.scroll_into_view();
    }

    /// 並べた表のキーとパレットの動作(stack.*)。動作が並べる動作なら true。
    pub(crate) fn stack_action(&mut self, action: Action) -> bool {
        match action {
            Action::StackDown => self.stack_step(true),
            Action::StackUp => self.stack_step(false),
            Action::StackClose => self.stack_close(),
            Action::StackGrow => self.stack_grow(true),
            Action::StackShrink => self.stack_grow(false),
            Action::StackAdd => self.start_stack_add(),
            Action::PageSave => self.start_page_save(),
            _ => return false,
        }
        true
    }

    /// stack.focus: 下(`down`)か上の表へ。端では動かない。並べていなければ何もしない。
    fn stack_step(&mut self, down: bool) {
        let Some(st) = &self.stack else {
            return;
        };
        let j = if down {
            st.focus + 1
        } else if st.focus == 0 {
            return;
        } else {
            st.focus - 1
        };
        if j < st.tables.len() {
            self.stack_focus(j);
        }
    }

    /// stack.resize: 選んでいる表を1行高く(`up`)か低く。相手は下の表(いちばん下なら上の表)。どの表も見出しと
    /// 3行は残す。
    fn stack_grow(&mut self, up: bool) {
        let Some(st) = self.stack.as_mut() else {
            self.message = Some(Msg::StackSingle.text().into());
            return;
        };
        let mut hs = st.heights();
        if hs.iter().any(|&h| h < MIN_H) {
            self.message = Some(Msg::StackCollapsed.text().into());
            return;
        }
        let (f, n) = (st.focus, hs.len());
        let order: Vec<usize> = (f + 1..n).chain((0..f).rev()).collect();
        if up {
            let Some(&j) = order.iter().find(|&&j| hs[j] > MIN_H) else {
                self.message = Some(Msg::StackGrowLimit.text().into());
                return;
            };
            hs[f] += 1;
            hs[j] -= 1;
        } else {
            let (Some(&j), true) = (order.first(), hs[f] > MIN_H) else {
                self.message = Some(Msg::StackShrinkLimit.text().into());
                return;
            };
            hs[f] -= 1;
            hs[j] += 1;
        }
        st.weights = hs
            .iter()
            .map(|&h| u32::try_from(h).unwrap_or(u32::MAX))
            .collect();
        let (w, h) = st.screen;
        self.stack_resize(w, h);
    }

    /// 大きさを合わせる(端末の大きさが変わったとき・選ぶ表を移ったとき・画面いっぱいの窓を開け閉めしたとき)。
    /// ほかの表は区画の大きさ、選んでいる表は区画か画面いっぱい。
    pub(crate) fn stack_resize(&mut self, w: u16, h: u16) {
        let full = full_screen(self);
        // notice.layout: 通知の欄は画面のいちばん上に1つ(選んでいる表のもの)。区画はその下から。
        let top = super::notices::notice_band(self, usize::from(w.saturating_sub(1))).len();
        let Some(st) = self.stack.as_mut() else {
            return self.resize_to(w, h);
        };
        st.screen = (w, h);
        st.full = full;
        st.top = top;
        let (hs, focus) = (st.heights(), st.focus);
        let bg = self.term_bg;
        for k in self.other_tables() {
            let hk = u16::try_from(hs[k] + 1).unwrap_or(u16::MAX);
            self.with_table(k, |o| {
                o.term_bg = bg;
                o.resize_to(w, hk);
            });
        }
        let mine = if full {
            h
        } else {
            u16::try_from(hs[focus] + 1).unwrap_or(u16::MAX)
        };
        self.resize_to(w, mine);
    }

    /// モードを替えたあと(`set_mode`)と引き出しを開いたときに、区画と画面いっぱいの間を行き来したなら大きさを
    /// 合わせ直す。
    pub(crate) fn fit_stack_size(&mut self) {
        let Some(st) = &self.stack else {
            return;
        };
        if st.full == full_screen(self) {
            return;
        }
        let (w, h) = st.screen;
        self.stack_resize(w, h);
    }

    /// ほかの表の読み込みを少し進める(BV-16)。
    pub(crate) fn stack_load_step(&mut self, budget: usize) {
        for k in self.other_tables() {
            self.with_table(k, |o| {
                if !o.progress.done {
                    o.load_step(budget);
                }
            });
        }
    }

    /// ほかの表を読み終えたか。
    pub(crate) fn stack_loaded(&self) -> bool {
        self.stack
            .as_ref()
            .is_none_or(|st| st.tables.iter().flatten().all(|o| o.progress.done))
    }

    /// ほかの表の外での変化を見る(BV-9)。ためた変更の外で変更の印は、1つのためた変更に付く。
    pub(crate) fn stack_poll(&mut self) {
        for k in self.other_tables() {
            self.with_table(k, App::poll);
        }
    }

    /// stack.shared-pending: ためた変更が変わっていたら、ほかの表を組み直す(描く前に呼ぶ)。
    pub(crate) fn stack_sync(&mut self) {
        let gen = self.changes.generation();
        for k in self.other_tables() {
            let stale = self
                .stack
                .as_ref()
                .is_some_and(|st| st.seen.get(k) != Some(&gen));
            if stale {
                self.with_table(k, |o| {
                    o.refresh();
                    o.scroll_into_view();
                });
                if let Some(s) = self.stack.as_mut().and_then(|st| st.seen.get_mut(k)) {
                    *s = gen;
                }
            }
        }
    }

    /// stack.focus: 並べた表の画面の点を、選んでいる表の区画の中の点にする。ほかの表の区画なら、先にその表を
    /// 選ぶ(入力の途中なら、決めてから)。見出しの行と区画の外(下の帯の問題の数は NV-30 のとおり)は None。
    /// `select` が真なら、ほかの表の行はその表を選んでから行を選んで None(編集は始めない)。
    pub(crate) fn stack_point(&mut self, x: u16, y: u16, select: bool) -> Option<u16> {
        if self.stack.is_none() || !in_region(self) {
            return Some(y);
        }
        let (regions, focus, screen, top) = {
            let st = self.stack.as_ref()?;
            (st.regions(), st.focus, st.screen, st.top)
        };
        let yy = usize::from(y);
        // notice.focus: 画面の上の通知の欄のクリック(欄の中の位置で、表のときと同じに受ける)。
        if yy < top {
            if matches!(self.mode, Mode::Table | Mode::Notice) {
                self.notice_click(x, y);
            }
            return None;
        }
        let Some(k) = regions.iter().position(|&(y0, h)| yy >= y0 && yy < y0 + h) else {
            // 下の帯の問題の数(NV-30)。
            let footer = usize::from(screen.1).saturating_sub(1 + CHROME);
            if yy == footer && super::bands::problems_x(self, x) {
                self.apply(Action::ProblemsOnly);
            }
            return None;
        };
        let local = yy - regions[k].0;
        if k == focus {
            return (local > 0).then(|| u16::try_from(local).ok()).flatten();
        }
        // 入力の途中なら決めてから移る(決められなければ移らない)。
        if matches!(self.mode, Mode::Edit | Mode::Search | Mode::Filter) {
            self.apply(Action::Commit);
        }
        if self.mode != Mode::Table {
            return None;
        }
        self.stack_focus(k);
        let local = u16::try_from(local).ok()?;
        if local == 0 {
            return None;
        }
        if !select {
            return Some(local);
        }
        if let Some((i, j)) = view::hit(self, x, local) {
            let moved = i != self.row;
            self.row = i;
            self.col = j;
            if moved {
                self.moved();
            }
            self.refresh_if_needed();
        }
        None
    }

    /// 並べた表の、選んでいる表のためた変更のうち、残るほかの表に出ていない行(stack.close-pending)。ほかの表に
    /// 出ている行は、閉じてもその表のためた変更として見えて書けるので、確かめない。
    fn own_pending(&mut self) -> Vec<RowId> {
        let rows = self.changes.rows();
        let mut out = Vec::new();
        for r in rows {
            if self.src.stamp(&r).is_none() {
                continue;
            }
            let elsewhere = self
                .stack
                .as_ref()
                .is_some_and(|st| st.tables.iter().flatten().any(|o| o.rows.contains(&r)));
            if !elsewhere {
                out.push(r);
            }
        }
        out
    }

    /// stack.close: 選んでいる表を並べた表から外す。その表だけのためた変更があれば、保存か捨てるかを確かめる
    /// (stack.close-pending)。表が1つだけなら理由。
    fn stack_close(&mut self) {
        if self.stack.is_none() {
            self.message = Some(Msg::StackCloseOnly.text().into());
            return;
        }
        if self.screen() != Screen::Table {
            self.close_screen();
        }
        let rows = self.own_pending();
        if rows.is_empty() {
            return self.stack_remove_focused();
        }
        if let Some(st) = self.stack.as_mut() {
            st.closing = Some(rows);
        }
        self.set_mode(Mode::Quit);
    }

    /// 閉じる確かめと、その表だけの保存の確認の動作(stack.close-pending)。閉じる確かめの間だけ true。
    pub(crate) fn stack_review_action(&mut self, action: Action) -> bool {
        let Some(rows) = self.stack.as_ref().and_then(|s| s.closing.clone()) else {
            return false;
        };
        match (self.mode, action) {
            (Mode::Quit, Action::QuitSave) => {
                // WB-17: 読んだ値と同じセルは外す。その表の書くものが残っていなければ、そのまま閉じる。
                self.drop_same_pending();
                if rows.iter().any(|r| self.changes.rows().contains(r)) {
                    self.open_review();
                } else {
                    self.stack_remove_focused();
                }
            }
            (Mode::Quit, Action::QuitDiscard) => {
                self.changes.discard_rows(&rows);
                self.stack_remove_focused();
            }
            (_, Action::Back) => {
                if let Some(st) = self.stack.as_mut() {
                    st.closing = None;
                }
                self.review = None;
                self.set_mode(Mode::Table);
            }
            _ => return false,
        }
        true
    }

    /// 閉じる確かめの間の、保存の確認に出す行(その表だけのためた変更)。
    pub(crate) fn stack_closing_rows(&self) -> Option<Vec<RowId>> {
        self.stack.as_ref()?.closing.clone()
    }

    /// 保存のあと(stack.save-all・stack.close-pending): 書けた行を、それを読んでいるほかの表でも読み直し、
    /// 閉じる確かめの間でその表の変更が全部書けたら表を閉じる。
    pub(crate) fn stack_after_save(&mut self, saved: &[RowId]) {
        if self.stack.is_none() {
            return;
        }
        if !saved.is_empty() {
            for k in self.other_tables() {
                let rows = saved.to_vec();
                self.with_table(k, move |o| {
                    let mine: Vec<RowId> = rows
                        .into_iter()
                        .filter(|r| o.src.stamp(r).is_some())
                        .collect();
                    if mine.is_empty() {
                        return;
                    }
                    for r in &mine {
                        let _ = o.src.reload(r);
                    }
                    o.refresh();
                });
            }
        }
        let Some(rows) = self.stack_closing_rows() else {
            return;
        };
        if self.mode == Mode::Table && !rows.iter().any(|r| self.changes.rows().contains(r)) {
            self.stack_remove_focused();
        }
    }

    /// 選んでいる表を外す(下の表を、無ければ上の表を選ぶ)。表が1つになったら並べるのをやめる。
    fn stack_remove_focused(&mut self) {
        self.persist_state();
        let label = table_label(self);
        let Some(mut st) = self.stack.take() else {
            return;
        };
        st.closing = None;
        let old = st.focus;
        let j = if old + 1 < st.tables.len() {
            old + 1
        } else {
            old.saturating_sub(1)
        };
        let Some(mut next) = st.tables.get_mut(j).and_then(Option::take) else {
            self.stack = Some(st);
            return;
        };
        std::mem::swap(&mut self.changes, &mut next.changes);
        next.git_pending.append(&mut self.git_pending);
        next.term_bg = self.term_bg;
        let gen = next.changes.generation();
        let stale = st.seen.get(j) != Some(&gen);
        std::mem::swap(self, &mut next);
        // next は閉じる表。
        drop(next);
        st.tables.remove(old);
        st.seen.remove(old);
        st.focus = if j > old { j - 1 } else { j };
        st.weights = vec![1; st.tables.len()];
        st.page = None;
        let screen = st.screen;
        if st.tables.len() <= 1 {
            self.stacked = false;
            self.stack = None;
        } else {
            st.full = !full_screen(self);
            self.stack = Some(st);
        }
        self.set_mode(Mode::Table);
        if stale {
            self.refresh();
        }
        self.stack_resize(screen.0, screen.1);
        if self.stack.is_none() {
            self.resize_to(screen.0, screen.1);
        }
        self.scroll_into_view();
        self.message = Some(Msg::StackClosed.fill(&[&label]));
    }

    /// stack.add: パレットの「下に表を並べる」。今の範囲の表のうち、まだ並べていないものから選ぶ。表でない画面の
    /// 型からは表の画面に戻ってから。4つ並べていれば、端末が低ければ理由。
    fn start_stack_add(&mut self) {
        // OUT-3: `--pick` は1つの表から選んで終わる起動なので、並べない(並べると選ばずに終わってしまう)。
        if self.pick_out.choosing {
            self.message = Some(Msg::StackPick.text().into());
            return;
        }
        if self.screen() != Screen::Table {
            self.close_screen();
        }
        let n = self.stack_len();
        if n >= MAX_TABLES {
            self.message = Some(Msg::StackMax.fill(&[&MAX_TABLES]));
            return;
        }
        let screen = self.stack.as_ref().map_or(self.size, |s| s.screen);
        let band = super::notices::notice_band(self, usize::from(screen.0.saturating_sub(1))).len();
        let avail = usize::from(screen.1).saturating_sub(1 + CHROME + band);
        if avail < (n + 1) * MIN_H {
            self.message = Some(Msg::StackNoRoom.text().into());
            return;
        }
        let taken = self.stack_names();
        let places: Vec<Place> = self
            .scope_places()
            .into_iter()
            .filter(|p| !taken.contains(&p.name))
            .collect();
        if places.is_empty() {
            self.message = Some(Msg::StackNoTables.text().into());
            return;
        }
        let items = super::places::labels(&places.iter().collect::<Vec<_>>());
        self.open_ask(Ask::StackTable { items, places });
    }

    /// 並べた表の範囲の表としての名前(上から。範囲に無い表は見出しの名前)。
    fn stack_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        match &self.stack {
            None => out.push(table_label(self)),
            Some(st) => {
                for (k, t) in st.tables.iter().enumerate() {
                    let app = if k == st.focus {
                        Some(self)
                    } else {
                        t.as_ref()
                    };
                    if let Some(a) = app {
                        out.push(table_label(a));
                    }
                }
            }
        }
        out
    }

    /// stack.add: 選んだ表を並べるよう頼む(画面を抜けて main が開き、今の表の下に並べる)。
    pub(crate) fn stack_add_picked(&mut self, place: Place) {
        self.close_palette();
        if !workspace::openable(&place.path) {
            self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
            return;
        }
        self.stack_request = Some(StackRequest::Add(place));
        self.quit = true;
    }

    /// stack.page-open: 引き出しで選んだページを開くよう頼む(ためた変更があれば、表を移るときと同じく確かめる)。
    pub(crate) fn open_page_request(&mut self, name: String) {
        if self.stack.as_ref().and_then(|s| s.page.as_deref()) == Some(name.as_str()) {
            return;
        }
        self.stack_request = Some(StackRequest::Page(name));
        self.switch_to = None;
        self.begin_quit();
    }

    /// stack.page-save: 並べた表の、ページの表の並び(上から。表の名前・今のビュー・高さの比)。範囲に無い表が
    /// あれば、その表の見出しの名前で Err。
    fn page_tables(&self) -> Result<Vec<PageTable>, String> {
        let Some(st) = &self.stack else {
            return Ok(Vec::new());
        };
        let ratio = mdgrid::page::ratio(&st.weights);
        let mut out = Vec::new();
        for (k, t) in st.tables.iter().enumerate() {
            let Some(a) = (if k == st.focus {
                Some(self)
            } else {
                t.as_ref()
            }) else {
                continue;
            };
            let Some(table) = scope_name(a) else {
                return Err(table_label(a));
            };
            out.push(PageTable {
                table,
                view: a.current_view_name(),
                height: ratio.get(k).copied().unwrap_or(1),
            });
        }
        Ok(out)
    }

    /// stack.page-save: パレットの「並べ方に名前を付けて残す」。続けて名前を打つ(今のページの名前を入れておく)。
    fn start_page_save(&mut self) {
        if self.stack.is_none() {
            self.message = Some(Msg::PageNoStack.text().into());
            return;
        }
        if self.page_home.is_none() {
            self.message = Some(Msg::PageNotWorkspace.text().into());
            return;
        }
        if let Err(name) = self.page_tables() {
            self.message = Some(Msg::PageTableOutside.fill(&[&name]));
            return;
        }
        let current = self.stack.as_ref().and_then(|s| s.page.clone());
        self.open_ask(Ask::PageName);
        if let (Some(p), Some(name)) = (self.palette.as_mut(), current) {
            p.query = name;
        }
    }

    /// stack.page-save・stack.page-save-marker: 打った名前でページを書き、範囲のページを読み直す。
    pub(crate) fn page_save_to(&mut self, query: &str) {
        let name = query.trim().to_string();
        if !mdgrid::page::valid_name(&name) {
            self.message = Some(Msg::PageNameBad.text().into());
            return;
        }
        let tables = match self.page_tables() {
            Ok(t) => t,
            Err(t) => {
                self.message = Some(Msg::PageTableOutside.fill(&[&t]));
                return;
            }
        };
        let Some(home) = self.page_home.clone() else {
            self.message = Some(Msg::PageNotWorkspace.text().into());
            return;
        };
        self.close_palette();
        let page = Page {
            name: name.clone(),
            tables,
        };
        let file = match &home {
            PageHome::App { .. } => workspace::FILE_NAME.to_string(),
            PageHome::Marker(_) => format!("{}/{}", workspace::MARKER_DIR, workspace::MARKER_FILE),
        };
        match workspace::save_page(&home, &page) {
            Ok(()) => {
                let pages = workspace::load_pages(&home).0;
                for k in self.other_tables() {
                    let p = pages.clone();
                    self.with_table(k, move |o| o.pages = p);
                }
                self.pages = pages;
                if let Some(st) = self.stack.as_mut() {
                    st.page = Some(name.clone());
                }
                self.message = Some(Msg::PageSaved.fill(&[&name, &file]));
            }
            Err(e) => self.message = Some(Msg::PageSaveError.fill(&[&e])),
        }
    }

    /// 範囲のページを読む(stack.page-store)。読めない行の警告を返す。
    pub(crate) fn load_pages(
        &mut self,
        target: &std::path::Path,
        config_dir: Option<&std::path::Path>,
    ) -> Vec<String> {
        let first = target
            .to_string_lossy()
            .lines()
            .next()
            .map_or_else(|| target.to_path_buf(), PathBuf::from);
        self.page_home = self
            .scope
            .as_ref()
            .and_then(|s| workspace::page_home(s, &first, config_dir));
        let (pages, warns) = self
            .page_home
            .as_ref()
            .map(workspace::load_pages)
            .unwrap_or_default();
        self.pages = pages;
        warns
    }

    /// stack.page-resume: 開いているページと、開き直す命令に添える `-w`(workspaces.toml のワークスペースのとき。
    /// 印のワークスペースは開いた表のパスで範囲が決まるので None)。ページを開いていなければ None。
    pub(crate) fn stack_page(&self) -> Option<(String, Option<String>)> {
        let page = self.stack.as_ref()?.page.clone()?;
        let ws = self.scope.as_ref().and_then(|s| {
            matches!(s.source, workspace::Source::App | workspace::Source::Chosen)
                .then(|| s.name.clone())
        });
        Some((page, ws))
    }

    /// 終えるときに、ほかの表の見た目の状態も書く(SR-12)。
    pub(crate) fn stack_persist(&mut self) {
        for k in self.other_tables() {
            self.with_table(k, App::persist_state);
        }
    }

    /// stack.save-all: 保存と差分の確認で使う、ためた変更と読み込み口(並べた表の全部をまとめたもの)。
    pub(crate) fn save_parts(&mut self) -> (&mut Changes, AllSources<'_>) {
        let focus = self.stack.as_ref().map_or(0, |s| s.focus);
        let mut names = vec![(focus, table_label(self))];
        let mut shown: Vec<HashSet<RowId>> = Vec::new();
        if let Some(st) = &self.stack {
            shown.push(self.rows.iter().cloned().collect());
            for (k, t) in st.tables.iter().enumerate() {
                if let Some(o) = t {
                    names.push((k, table_label(o)));
                    shown.push(o.rows.iter().cloned().collect());
                }
            }
        }
        let mut list: Vec<&mut (dyn Source + 'static)> = vec![self.src.as_mut()];
        if let Some(st) = self.stack.as_mut() {
            for o in st.tables.iter_mut().flatten() {
                list.push(o.src.as_mut());
            }
        }
        (&mut self.changes, AllSources { list, names, shown })
    }
}

/// stack.save-all: 並べた表の読み込み口をまとめたもの。行ごとに、その行を出している表(無ければ読んだ表。
/// 選んでいる表が先)へ渡す。保存と差分の確認(`Changes::previews`・`save`・`save_rows`・`drop_same`・
/// `overwrite_seen`)で使う。どの表で書いても同じファイルなので、書き戻しの約束はそれぞれの読み込み口のまま。
pub(crate) struct AllSources<'a> {
    list: Vec<&'a mut (dyn Source + 'static)>,
    /// `list` と同じ並びの (並べた位置, 見出しの表の名前)。
    names: Vec<(usize, String)>,
    /// `list` と同じ並びの、表に出している行(並べていなければ空)。
    shown: Vec<HashSet<RowId>>,
}

impl AllSources<'_> {
    /// 行(か書く単位)の表の `list` の添字: 出している表、無ければ読んだ表。どれにも無ければ 0。
    fn owner(&self, row: &RowId) -> usize {
        if let Some(i) = self.shown.iter().position(|s| s.contains(row)) {
            return i;
        }
        if let Some(i) = self.list.iter().position(|s| s.stamp(row).is_some()) {
            return i;
        }
        if self.list.len() > 1 {
            if let Some(i) = self
                .list
                .iter()
                .position(|s| s.rows().iter().any(|r| s.unit(r) == *row))
            {
                return i;
            }
        }
        0
    }

    /// 行を読んだ表の (並べた位置, 見出しの名前)。
    pub(crate) fn place(&self, row: &RowId) -> (usize, String) {
        self.names.get(self.owner(row)).cloned().unwrap_or_default()
    }

    /// 並べた表か(見出しを付けるか)。
    pub(crate) fn many(&self) -> bool {
        self.list.len() > 1
    }
}

impl Source for AllSources<'_> {
    fn name(&self) -> String {
        self.list[0].name()
    }
    fn load(&mut self, budget: usize) -> mdgrid::vault::Progress {
        self.list[0].load(budget)
    }
    fn cancel(&mut self) {
        self.list[0].cancel();
    }
    fn rows(&self) -> Vec<RowId> {
        let mut out: Vec<RowId> = Vec::new();
        for s in &self.list {
            for r in s.rows() {
                if !out.contains(&r) {
                    out.push(r);
                }
            }
        }
        out
    }
    fn label(&self, row: &RowId) -> String {
        self.list[self.owner(row)].label(row)
    }
    fn mark(&self, row: &RowId) -> Option<String> {
        self.list[self.owner(row)].mark(row)
    }
    fn columns(&self) -> Vec<String> {
        self.list[0].columns()
    }
    fn get(&self, row: &RowId, col: &str) -> Cell {
        self.list[self.owner(row)].get(row, col)
    }
    fn stamp(&self, row: &RowId) -> Option<Stamp> {
        self.list.iter().find_map(|s| s.stamp(row))
    }
    fn reload(&mut self, row: &RowId) -> std::io::Result<()> {
        let i = self.owner(row);
        self.list[i].reload(row)
    }
    fn notes(&self) -> bool {
        self.list[0].notes()
    }
    fn locate(&self, row: &RowId) -> (PathBuf, String) {
        self.list[self.owner(row)].locate(row)
    }
    fn unit(&self, row: &RowId) -> RowId {
        self.list[self.owner(row)].unit(row)
    }
    fn preview_unit(&self, edits: &[(RowId, Vec<Edit>)]) -> Result<(Vec<u8>, Vec<u8>), EditError> {
        let i = edits.first().map_or(0, |(r, _)| self.owner(r));
        self.list[i].preview_unit(edits)
    }
    fn save_unit(
        &mut self,
        base: &Stamp,
        edits: &[(RowId, Vec<Edit>)],
    ) -> Result<Stamp, SaveError> {
        let i = edits.first().map_or(0, |(r, _)| self.owner(r));
        self.list[i].save_unit(base, edits)
    }
    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError> {
        self.list[self.owner(row)].preview(row, edits)
    }
    fn save(&mut self, row: &RowId, base: &Stamp, edits: &[Edit]) -> Result<Stamp, SaveError> {
        let i = self.owner(row);
        self.list[i].save(row, base, edits)
    }
    fn changed(&mut self) -> Vec<RowId> {
        Vec::new()
    }
    fn pause(&mut self, paused: bool) {
        for s in &mut self.list {
            s.pause(paused);
        }
    }
    fn kind(&self, col: &str) -> ColumnKind {
        self.list[0].kind(col)
    }
    fn typed(&self, col: &str) -> bool {
        self.list[0].typed(col)
    }
    fn body(&self, row: &RowId) -> Option<String> {
        self.list[self.owner(row)].body(row)
    }
    fn file(&self, row: &RowId) -> Option<FileInfo> {
        self.list[self.owner(row)].file(row)
    }
    fn candidates(&self, col: &str, max: usize) -> Option<Vec<Value>> {
        self.list[0].candidates(col, max)
    }
    fn list_candidates(&self, col: &str) -> Vec<(String, usize)> {
        self.list[0].list_candidates(col)
    }
}
