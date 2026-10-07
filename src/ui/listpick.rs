//! リストの選択(`impl App` の続き。CE-16・CE-17・CE-19・CE-11・SR-9・SR-17。新しいノートの聞く項目 CE-26 でも)。
//! リストの値のセルで Enter(詳細の表示の Enter からも)で開く。上に検索の入力、下に候補(`[x] 会議  5件`)。
//! 打つたびに候補を大文字小文字を区別しない部分一致で絞り、打った要素が候補に無ければ先頭に「+ 新規: …」を出す。
//! ↑↓ で選ぶ。検索が空なら Space で付け外し・Enter で確定、検索中なら Space は空白・Enter は選んだ候補を
//! 付けるだけ(既に付いていれば何もしない。新規なら足す)で検索を空に戻す。外すのは検索が空のときの Space だけ。
//! tags で `#` を除いた語が空なら検索が空と同じ。Tab はいつでも確定、Esc で取り消し、Ctrl+R で開いたときに戻す。
//! リストのセル(列の型がリストか値がリスト)には、どの道(Tab の移動・一括)でも1行の入力を開かない。
//! 選んだ行(NV-5)を含むときは一括(CE-17): 印は全部の行が持つ `[x]`・一部の行が持つ `[-]`・誰も持たない `[ ]`。
//! 確定は行ごとに「元の要素に、`[x]` にした要素を足し、`[ ]` にした要素を除いた」並び(触らなかった要素は各行のまま)。

use super::app::App;
use super::input::{bulk_tail, read_only_lead};
use super::keymap::{self, Action, Mode};
use super::popup;
use super::view::{data_y, visible_layout};
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::source::{NewValue, RowId, Value};
use mdgrid::types::Kind;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use unicode_segmentation::UnicodeSegmentation;

/// 候補の印。1行なら All か None だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mark {
    /// `[x]`: 全部の行が持つ(付ける)。
    All,
    /// `[-]`: 一部の行だけが持つ(触らない)。
    Some,
    /// `[ ]`: 誰も持たない(外す)。
    None,
}

impl Mark {
    fn text(self) -> &'static str {
        match self {
            Mark::All => "[x]",
            Mark::Some => "[-]",
            Mark::None => "[ ]",
        }
    }
}

/// 候補の1つ。
pub(crate) struct Cand {
    /// 要素(tags は先頭の `#` を除いたもの)。
    pub name: String,
    /// 保管庫の中でこの要素を持つノートの数(CE-19)。
    pub count: usize,
    pub mark: Mark,
    pub initial: Mark,
    /// 「+ 新規」で足した(Ctrl+R で消える)。
    pub added: bool,
}

/// 見せる行。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PickRow {
    /// 「+ 新規: …」(打った要素が候補に無いとき)。
    New(String),
    /// 候補(cands の添字)。
    Cand(usize),
}

/// リストの選択の状態。
pub(crate) struct Pick {
    pub col: String,
    /// 書く行と、開いたときの要素の並び。1行なら1つ。
    pub targets: Vec<(RowId, Vec<String>)>,
    /// 選んだ行への一括(CE-17)。
    pub bulk: bool,
    /// 一括から外した、畳んだまとまりの中の行の数。
    pub folded_out: usize,
    /// 一括で飛ばした行の理由。
    pub skipped: Vec<String>,
    /// tags の列(打った先頭の `#` を除く。CE-19)。
    pub tags: bool,
    pub query: String,
    pub cands: Vec<Cand>,
    /// `[x]` にした順(足す要素を末尾にこの順で並べる)。
    pub order: Vec<String>,
    /// 選んだ行(rows() の添字)。
    pub sel: usize,
}

impl Pick {
    /// 要素の比べる形(tags は先頭の `#` を除く)。
    fn key<'a>(&self, s: &'a str) -> &'a str {
        if self.tags {
            s.strip_prefix('#').unwrap_or(s)
        } else {
            s
        }
    }

    /// 打った文字を要素にしたもの(前後の空白を除き、tags は先頭の `#` を除く)。
    pub(crate) fn typed(&self) -> String {
        self.key(self.query.trim()).trim().to_string()
    }

    /// 検索中か(打った語が空でない。tags で `#` を除くと空なら、検索が空のときと同じに扱う)。
    pub(crate) fn searching(&self) -> bool {
        !self.typed().is_empty()
    }

    /// 同じ要素か。tags の列は大文字小文字を区別しない(`meeting` は既にある `Meeting`)。ほかの列は区別する。
    fn same(&self, a: &str, b: &str) -> bool {
        if self.tags {
            a.to_lowercase() == b.to_lowercase()
        } else {
            a == b
        }
    }

    /// 打った語と同じ候補(`same` で比べる)。tags で大文字小文字違いが複数あれば、この行(一括なら選んだ行の
    /// どれか)が持つもの、無ければ同じ綴りのもの、無ければ件数の多いもの(候補の並びの先)を選ぶ。
    fn find_same(&self, t: &str) -> Option<usize> {
        if t.is_empty() {
            return None;
        }
        let same: Vec<usize> = (0..self.cands.len())
            .filter(|&i| self.same(&self.cands[i].name, t))
            .collect();
        let held = |i: &&usize| {
            let c = &self.cands[**i];
            c.initial != Mark::None || c.mark != Mark::None
        };
        same.iter()
            .find(held)
            .or_else(|| same.iter().find(|&&i| self.cands[i].name == t))
            .or(same.first())
            .copied()
    }

    /// 見せる行: 打った要素が候補に無ければ先頭に新規、続けて部分一致する候補(大文字小文字を区別しない)。
    /// 打った要素と同じ候補があれば、それを先頭に出す。
    pub(crate) fn rows(&self) -> Vec<PickRow> {
        let t = self.typed();
        let q = t.to_lowercase();
        let mut out = Vec::new();
        let exact = self.find_same(&t);
        match exact {
            Some(i) => out.push(PickRow::Cand(i)),
            None if !t.is_empty() => out.push(PickRow::New(t)),
            None => {}
        }
        for (i, c) in self.cands.iter().enumerate() {
            if Some(i) != exact && (q.is_empty() || c.name.to_lowercase().contains(&q)) {
                out.push(PickRow::Cand(i));
            }
        }
        out
    }

    fn touched(&self) -> bool {
        self.cands.iter().any(|c| c.mark != c.initial)
    }

    /// 行の新しい並び: 元の並びから `[ ]` の要素を除き、`[x]` にした要素で行に無いものを付けた順に末尾へ。
    pub(crate) fn result(&self, orig: &[String]) -> Vec<String> {
        let mark = |k: &str| self.cands.iter().find(|c| c.name == k).map(|c| c.mark);
        let mut out: Vec<String> = orig
            .iter()
            .filter(|e| mark(self.key(e)) != Some(Mark::None))
            .cloned()
            .collect();
        for name in &self.order {
            // 行が既に持つ要素(tags は大文字小文字違いも)は足さない。
            if mark(name) == Some(Mark::All) && !out.iter().any(|e| self.same(self.key(e), name)) {
                out.push(name.clone());
            }
        }
        out
    }

    /// Space: 印を回す。一部 → 付ける → 外す(→ 開いたときが一部なら一部へ)。
    fn cycle(&mut self, i: usize) {
        let c = &mut self.cands[i];
        c.mark = match c.mark {
            Mark::Some => Mark::All,
            Mark::All => Mark::None,
            Mark::None if c.initial == Mark::Some => Mark::Some,
            Mark::None => Mark::All,
        };
        let name = c.name.clone();
        let on = c.mark == Mark::All;
        self.order.retain(|n| *n != name);
        if on {
            self.order.push(name);
        }
    }

    /// 検索中の Enter: 候補を付ける(`[x]` にする)だけ。既に付いていれば何もしない(一括の `[-]` は `[x]` に)。
    fn attach(&mut self, i: usize) {
        if self.cands[i].mark != Mark::All {
            self.cands[i].mark = Mark::All;
            let name = self.cands[i].name.clone();
            self.order.retain(|n| *n != name);
            self.order.push(name);
        }
        self.clear_query_at(i);
    }

    /// 新規の要素を足して付ける。同じ要素が候補にあればそれを付ける(2つ足さない)。
    fn add(&mut self, name: String) {
        let i = match self.find_same(&name) {
            Some(i) => i,
            None => {
                self.cands.push(Cand {
                    name: name.clone(),
                    count: 0,
                    mark: Mark::None,
                    initial: Mark::None,
                    added: true,
                });
                self.cands.len() - 1
            }
        };
        self.attach(i);
    }

    /// 検索を空に戻し、候補 `i` を選ぶ(窓は開いたまま)。
    fn clear_query_at(&mut self, i: usize) {
        self.query.clear();
        self.sel = self
            .rows()
            .iter()
            .position(|r| *r == PickRow::Cand(i))
            .unwrap_or(0);
    }
}

fn is_blank(v: &Value) -> bool {
    matches!(v, Value::Null) || matches!(v, Value::Str(s) if s.is_empty())
}

impl App {
    /// リストの選択で開くセルか(CE-16): 今の値がリスト、か列の型がリストで値が空(キーなし・Null・空の文字列)。
    pub(crate) fn wants_pick(&self, row: &RowId, col: &str) -> bool {
        let list_kind = || self.src.kind(col).kind == Kind::List;
        match self.changes.pending(row, col) {
            Some(NewValue::List(_)) => true,
            Some(NewValue::Null) => list_kind(),
            Some(NewValue::Str(s)) if s.is_empty() => list_kind(),
            Some(_) => false,
            None => match self.src.get(row, col).value {
                Some(Value::List(_)) => true,
                None => list_kind(),
                Some(v) if is_blank(&v) => list_kind(),
                // CE-19: リストの列の1つの文字列は、1つの要素のリストとして開く。
                Some(Value::Str(_)) => list_kind(),
                Some(_) => false,
            },
        }
    }

    /// リストのセルか(CE-16): 列の型がリストか、リストの選択で開く値。1行の入力はどの道でも開かない。
    pub(crate) fn is_list_cell(&self, row: &RowId, col: &str) -> bool {
        self.src.kind(col).kind == Kind::List || self.wants_pick(row, col)
    }

    /// 行の今の要素(ためた値があればそれ)。書けなければ理由(CE-8 の形は読むだけ。CE-19)。
    pub(crate) fn list_of(&self, row: &RowId, col: &str) -> Result<Vec<String>, String> {
        const NOT_LIST: Msg = Msg::NotAList;
        if let Some(v) = self.changes.pending(row, col) {
            return match v {
                NewValue::List(items) => Ok(items.clone()),
                NewValue::Null => Ok(Vec::new()),
                NewValue::Str(s) if s.is_empty() => Ok(Vec::new()),
                _ => Err(NOT_LIST.into()),
            };
        }
        let cell = self.src.get(row, col);
        if let Some(r) = &cell.lock {
            return Err(format!("{}{r}", read_only_lead()));
        }
        match cell.value {
            None => Ok(Vec::new()),
            Some(Value::List(items)) => items
                .into_iter()
                .map(|v| match v {
                    Value::Str(s) => Ok(s),
                    _ => Err(format!("{}{}", read_only_lead(), Msg::ListNonString.text())),
                })
                .collect(),
            Some(v) if is_blank(&v) => Ok(Vec::new()),
            // CE-19: リストの列の1つの文字列は1つの要素(確定でフローのリストに書き換わる)。
            Some(Value::Str(s)) if self.src.kind(col).kind == Kind::List => Ok(vec![s]),
            Some(_) => Err(NOT_LIST.into()),
        }
    }

    /// リストの選択を開く(CE-16)。`bulk` は選んだ行と、外した畳んだまとまりの中の行の数(CE-17)。
    pub(crate) fn open_pick(&mut self, row: RowId, col: String, bulk: Option<(Vec<RowId>, usize)>) {
        let (rows, is_bulk, folded_out) = match bulk {
            Some((b, n)) => (b, true, n),
            None => (vec![row], false, 0),
        };
        let mut targets = Vec::new();
        let mut skipped = Vec::new();
        let mut first_err = None;
        for r in rows {
            match self.list_of(&r, &col) {
                Ok(items) => targets.push((r, items)),
                Err(e) => {
                    skipped.push(e.trim_start_matches(read_only_lead()).to_string());
                    first_err.get_or_insert(e);
                }
            }
        }
        if targets.is_empty() {
            let e = first_err.unwrap_or_default();
            self.message = Some(if is_bulk {
                Msg::BulkNoEditable.fill(&[&e])
            } else {
                e
            });
            return;
        }
        let mut pick = self.new_pick(col, targets);
        pick.bulk = is_bulk;
        pick.folded_out = folded_out;
        pick.skipped = skipped;
        self.pick = Some(pick);
        self.set_mode(Mode::ListPick);
    }

    /// リストの選択の状態を作る: 候補は保管庫の要素と、書く行の要素(1行・一括でない)。
    /// 新しいノートの聞く項目(CE-26。new_note.rs)も、行の無い1つの書き先でこれを使う。
    pub(crate) fn new_pick(&self, col: String, targets: Vec<(RowId, Vec<String>)>) -> Pick {
        let tags = col == "tags";
        let mut pick = Pick {
            col: col.clone(),
            targets,
            bulk: false,
            folded_out: 0,
            skipped: Vec::new(),
            tags,
            query: String::new(),
            cands: Vec::new(),
            order: Vec::new(),
            sel: 0,
        };
        let mut names: Vec<(String, usize)> = self.src.list_candidates(&col);
        // ためた値にだけある要素も候補にする(件数は保管庫の中の数なので 0)。
        for (_, items) in &pick.targets {
            for e in items {
                let k = pick.key(e);
                if !k.is_empty() && !names.iter().any(|(n, _)| n == k) {
                    names.push((k.to_string(), 0));
                }
            }
        }
        let n = pick.targets.len();
        for (name, count) in names {
            let has = pick
                .targets
                .iter()
                .filter(|(_, items)| items.iter().any(|e| pick.key(e) == name))
                .count();
            let mark = match has {
                0 => Mark::None,
                h if h == n => Mark::All,
                _ => Mark::Some,
            };
            pick.cands.push(Cand {
                name,
                count,
                mark,
                initial: mark,
                added: false,
            });
        }
        pick
    }

    /// 検索に文字を足す(全角の文字は全角のまま。SR-17)。
    pub(crate) fn list_pick_insert(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            return;
        }
        if let Some(p) = &mut self.pick {
            p.query.push(c);
            p.sel = 0;
        }
    }

    /// リストの選択の動作。
    pub(crate) fn list_pick_action(&mut self, action: Action) {
        let page = self.pick_page();
        let Some(p) = &mut self.pick else {
            return self.close_input();
        };
        let rows = p.rows();
        let last = rows.len().saturating_sub(1);
        let searching = p.searching();
        match action {
            Action::ListUp => p.sel = p.sel.saturating_sub(1),
            Action::ListDown => p.sel = (p.sel + 1).min(last),
            Action::PageUp => p.sel = p.sel.saturating_sub(page),
            Action::PageDown => p.sel = (p.sel + page).min(last),
            Action::DeleteBack => {
                p.query.pop();
                p.sel = 0;
            }
            // 検索中(打った語が空でない。tags の `#` だけは空と同じ)は、Space は空白として検索に入る。
            Action::Toggle if searching => {
                p.query.push(' ');
                p.sel = 0;
            }
            // 外すのは、検索が空のときの Space だけ。
            Action::Toggle => match rows.get(p.sel) {
                Some(PickRow::New(t)) => p.add(t.clone()),
                Some(PickRow::Cand(i)) => p.cycle(*i),
                None => {}
            },
            // 検索中の Enter は、選んだ候補を付けるだけ(既に付いていれば何もしない。新規なら足す)。
            // 検索を空に戻し、窓は開いたまま。
            Action::Run if searching => match rows.get(p.sel) {
                Some(PickRow::New(t)) => p.add(t.clone()),
                Some(PickRow::Cand(i)) => p.attach(*i),
                None => p.query.clear(),
            },
            Action::Run => self.pick_commit(),
            Action::Commit => self.pick_commit(),
            Action::Cancel => {
                self.pick = None;
                self.close_input();
            }
            Action::Revert => {
                p.cands.retain(|c| !c.added);
                for c in &mut p.cands {
                    c.mark = c.initial;
                }
                p.order.clear();
                p.query.clear();
                p.sel = 0;
            }
            _ => {}
        }
    }

    /// 窓の候補の行の数(PageUp / PageDown の幅)。
    fn pick_page(&self) -> usize {
        let w = self.size.0.saturating_sub(1) as usize;
        let h = (self.size.1.saturating_sub(1) as usize).saturating_sub(2);
        geometry(self, w, h).map_or(1, |g| g.vis.max(1))
    }

    /// 確定(CE-16・CE-17)。触らずに閉じたら何も書かない。1行は `Changes::set`、一括は `Changes::set_each` の1手。
    fn pick_commit(&mut self) {
        let Some(p) = self.pick.take() else {
            return;
        };
        // 表から開いたなら、選んだセルが描かれているときだけ確定する(見えないセルに書かない。SR-3)。
        if self.detail.is_none() && !super::view::selected_drawn(self) {
            self.message = Some(Msg::PickNotDrawn.into());
            self.pick = Some(p);
            return;
        }
        if !p.touched() {
            return self.close_input();
        }
        if !p.bulk {
            let (row, orig) = &p.targets[0];
            let new = p.result(orig);
            if new != *orig {
                let v = NewValue::List(new);
                if let Err(s) = self.changes.set(self.src.as_ref(), row, &p.col, v) {
                    self.message = Some(Msg::CannotCommit.fill(&[&s.reason]));
                    self.pick = Some(p);
                    return;
                }
                // NV-12: 直した行は、保存か移動の操作まで元の位置に留める。
                self.stay.insert(row.clone());
                self.regrid = true;
            }
            return self.close_input();
        }
        let mut items = Vec::new();
        for (row, orig) in &p.targets {
            let new = p.result(orig);
            if new != *orig {
                items.push((row.clone(), Some(NewValue::List(new))));
            }
        }
        let same = p.targets.len() - items.len();
        let skips = self.changes.set_each(self.src.as_ref(), &p.col, &items);
        let mut reasons = p.skipped.clone();
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let mut msg = Msg::PickBulkDone.fill(&[&done]);
        if same > 0 {
            msg.push_str(&Msg::PickBulkSame.fill(&[&same]));
        }
        bulk_tail(&mut msg, p.folded_out, reasons);
        self.close_input();
        self.message = Some(msg);
    }
}

/// メッセージ行の案内(リストの選択のとき)。
pub(crate) fn hint(app: &App) -> Option<String> {
    if app.mode != Mode::ListPick {
        return None;
    }
    let p = app.pick.as_ref()?;
    let key = |a: Action| keymap::key_for(&app.keys, Mode::ListPick, a).unwrap_or_default();
    // キーの見せ方は表から(SR-4)。検索中は Enter で付ける(外さない)、Space は空白。
    let keys = if p.searching() {
        Msg::PickKeysSearching.fill(&[&key(Action::Run), &key(Action::Commit)])
    } else {
        Msg::PickKeys.fill(&[&key(Action::Toggle), &key(Action::Run)])
    };
    Some(if p.bulk {
        Msg::PickHintBulk.fill(&[&p.targets.len(), &keys])
    } else {
        Msg::PickHint.fill(&[&keys])
    })
}

// ---- 描画(純関数) ----

/// 窓の置き場所。
struct Geom {
    x: usize,
    /// 窓の一番上の行(上の縁)。
    top: usize,
    /// 窓の幅(縁を含む)。
    iw: usize,
    /// 見せる最初の候補の行と数。
    start: usize,
    vis: usize,
}

/// 縁2行と検索の1行。
const FRAME: usize = 3;
/// 窓の最小の幅。
const MIN_W: usize = 24;

/// 窓を出す元の位置(桁と行)。表なら選んだセル、詳細の表示なら値の欄。見えていなければ None。
fn anchor(app: &App, w: usize) -> Option<(usize, usize)> {
    // CE-26: 新しいノートの聞く項目なら、その入力の欄。
    if app.note.flow.is_some() {
        return super::new_note::note_box(app, w).map(|b| (b.x, b.y));
    }
    if app.detail.is_some() {
        return super::detail::detail_anchor(app, w).map(|(x, y, _)| (x, y));
    }
    if app.row < app.top || app.row >= app.top + app.data_height() {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let mut x = lay.data_x();
    for &(j, cw) in &cols {
        if j == app.col {
            return Some((x, data_y(app) + app.row - app.top));
        }
        x += cw + 1;
    }
    None
}

/// 候補の行の文字(印・要素・件数)。`nw` は要素の欄の幅、`cw` は件数の欄の幅。
fn row_text(p: &Pick, r: &PickRow, nw: usize, cw: usize) -> String {
    match r {
        PickRow::New(t) => Msg::PickNew.fill(&[&sanitize(t)]),
        PickRow::Cand(i) => {
            let c = &p.cands[*i];
            let count = if c.count > 0 {
                Msg::PickCount.fill(&[&c.count])
            } else {
                String::new()
            };
            format!(
                "{} {}  {}",
                c.mark.text(),
                fit(&sanitize(&c.name), nw, Align::Left),
                fit(&count, cw, Align::Right)
            )
        }
    }
}

/// 要素の欄と件数の欄の幅。
fn col_widths(p: &Pick, w: usize) -> (usize, usize) {
    let cw = p
        .cands
        .iter()
        .map(|c| width(&Msg::PickCount.fill(&[&c.count])))
        .max()
        .unwrap_or(0);
    let nw = p
        .cands
        .iter()
        .map(|c| width(&sanitize(&c.name)))
        .max()
        .unwrap_or(1);
    // 窓の中: 縁2・選びの印1・`[x] `4・要素・空白2・件数。収まらなければ要素の欄を縮める。
    let room = w.saturating_sub(2 + 1 + 4 + 2 + cw).max(1);
    (nw.min(room), cw)
}

/// `limit` は下の帯より上の行の数。窓は画面の幅と高さに収める(SR-9)。
fn geometry(app: &App, w: usize, limit: usize) -> Option<Geom> {
    if app.mode != Mode::ListPick {
        return None;
    }
    let p = app.pick.as_ref()?;
    let (ax, ay) = anchor(app, w).unwrap_or((0, 0));
    let rows = p.rows();
    let n = rows.len().max(1);
    if w < 8 {
        return None;
    }
    let popup::Place { top, vis } = popup::place_with(ay, n, limit, FRAME, 0)?;
    let (nw, cw) = col_widths(p, w);
    let inner = rows
        .iter()
        .map(|r| 1 + width(&row_text(p, r, nw, cw)))
        .chain([width(&format!("{}{}", SEARCH.text(), sanitize(&p.query))) + 1])
        .max()
        .unwrap_or(0);
    let iw = (inner + 2).max(MIN_W).min(w);
    let start = (p.sel + 1).saturating_sub(vis);
    Some(Geom {
        x: ax.min(w - iw),
        top,
        iw,
        start,
        vis,
    })
}

/// 検索の欄に見せる文字(カーソルが見えるように左を落とす)。
fn query_view(q: &str, room: usize) -> String {
    let mut s = sanitize(q);
    while width(&s) + 1 > room && !s.is_empty() {
        let first = s.graphemes(true).next().map(str::len).unwrap_or(0);
        s.drain(..first);
    }
    s
}

const SEARCH: Msg = Msg::PickSearchLead;

/// 下の帯より上の行(`lines`)に窓を重ねる。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let Some(g) = geometry(app, w, lines.len()) else {
        return;
    };
    let Some(p) = app.pick.as_ref() else {
        return;
    };
    let tw = g.iw - 2;
    let rows = p.rows();
    let (nw, cw) = col_widths(p, w);
    let mut out: Vec<(String, Style)> = Vec::new();
    // 上の縁: 列の名前(一括なら行の数)と、選んだ候補の位置。
    let pos = if rows.is_empty() {
        "0/0".to_string()
    } else {
        format!("{}/{}", p.sel + 1, rows.len())
    };
    let title = if p.bulk {
        Msg::PickTitleBulk.fill(&[&sanitize(&p.col), &p.targets.len(), &pos])
    } else {
        format!("{} {pos}", sanitize(&p.col))
    };
    out.push((popup::top_edge(&title, tw), Style::default()));
    let qroom = tw.saturating_sub(width(SEARCH.text()));
    let search = format!("{}{}", SEARCH.text(), query_view(&p.query, qroom));
    out.push((
        format!("|{}|", fit(&search, tw, Align::Left)),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ));
    if rows.is_empty() {
        out.push((
            format!("|{}|", fit(Msg::PickNoMatch.text(), tw, Align::Left)),
            Style::default(),
        ));
    }
    for (k, r) in rows.iter().enumerate().skip(g.start).take(g.vis) {
        let sel = k == p.sel;
        let text = format!("{}{}", if sel { ">" } else { " " }, row_text(p, r, nw, cw));
        let st = if sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        out.push((format!("|{}|", fit(&text, tw, Align::Left)), st));
    }
    out.push((format!("+{}+", "-".repeat(tw)), Style::default()));
    popup::blit(lines, g.x, g.top, g.iw, w, out);
}

/// 端末のカーソルの位置(SR-17: 検索の入力の位置。変換の窓がそこに出る)。
pub(crate) fn pick_cursor(app: &App) -> Option<(u16, u16)> {
    let (w, limit) = popup::screen(app);
    let g = geometry(app, w, limit)?;
    let p = app.pick.as_ref()?;
    let tw = g.iw - 2;
    let qroom = tw.saturating_sub(width(SEARCH.text()));
    let x = g.x + 1 + width(SEARCH.text()) + width(&query_view(&p.query, qroom));
    let x = x.min(g.x + g.iw - 2);
    Some((x as u16, (g.top + 1) as u16))
}

#[cfg(test)]
#[path = "test_listpick.rs"]
mod tests;

#[cfg(test)]
#[path = "test_scalar_list_pick.rs"]
mod test_scalar_list_pick;
