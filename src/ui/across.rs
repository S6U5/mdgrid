//! ワークスペースを横断して探す(navigation.across-*。`impl App` の続き)。
//!
//! 設定の `[search] across` を真にしたときだけ、パレットに「ワークスペースを横断して探す」を出す
//! (navigation.across-option)。続けて語を打って Enter で、今の範囲(`App::scope_places`)の表を裏の糸で1つずつ
//! ファイルから開き、表の中の検索(navigation.search。`nav::matches`)と同じ当て方で行を探す
//! (navigation.across-results)。読み終えた表から結果を出し、読んでいる間は「3 / 5 を読みました」、Esc で止めて
//! 表に戻る(navigation.across-progress)。結果の行で Enter を押すと、その表を開き、検索の欄に同じ語を入れて
//! その行を選ぶ(navigation.across-open)。入力と結果は、パレットの続きの入力(`Ask::Across`)の場所に重ねる。

use super::app::App;
use super::native_views::Ask;
use super::nav::matches;
use super::width::{fit, sanitize, width, Align};
use mdgrid::base::{self, Shown};
use mdgrid::expr::Expr;
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::source::RowId;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

/// 当たった行の1つ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Hit {
    /// 行の鍵(`RowId` の文字。ノートは実体のパス、CSV は `パス#番号`)。
    pub row: String,
    /// ノートの名前(表の左の列と同じ)。
    pub note: String,
    /// 当たった列の見出し(ノートの名前だけで当たったら、ノートの列の見出し)。
    pub col: String,
    /// 当たった値(素の文字)。
    pub value: String,
}

/// 表1つの結果(当たった行か、読めなかった理由)。
pub(crate) type Found = (Place, Result<Vec<Hit>, String>);

/// 横断の検索の状態。
#[derive(Default)]
pub(crate) struct AcrossState {
    /// 設定の `[search] across`(navigation.across-option)。
    pub on: bool,
    /// 探した語(Enter で決めた語。None はまだ探していない)。
    pub word: Option<String>,
    /// 読み終えた表の結果(範囲の順)。
    pub found: Vec<Found>,
    /// 探す表の数。
    pub total: usize,
    /// 裏の糸の結果(探した世代, 表の結果)。読み終えたら None。
    pub(crate) rx: Option<Receiver<(usize, Found)>>,
    pub(crate) generation: usize,
    /// 止める合図(Esc・探し直しで立て、裏の糸が表ごとと読み込みの合間に見る)。
    stop: Option<Arc<AtomicBool>>,
}

/// 結果の重ねの1行(当たった行なら、平らにした当たりの添字)。
struct Row {
    line: Line<'static>,
    hit: Option<usize>,
}

/// 結果を一度に見せる行の数の上限。
const ACROSS_ROWS: usize = 16;

impl App {
    /// navigation.across-option: パレットとヘルプにそのコマンドを出すか(横断の検索は設定で入れたときだけ)。
    /// routine.screen-create・routine.readonly: ルーチンの回を作るコマンドは、ルーチンの表があって、読むだけの起動で
    /// ないときだけ。
    pub(crate) fn command_shown(&self, action: super::keymap::Action) -> bool {
        match action {
            super::keymap::Action::AcrossSearch => self.across.on,
            super::keymap::Action::RoutineRuns => !self.readonly && !self.routine.tables.is_empty(),
            _ => true,
        }
    }

    /// パレットの「ワークスペースを横断して探す」: 続けて語を打つ(navigation.across-option)。
    pub(crate) fn start_across(&mut self) {
        if !self.across.on {
            return;
        }
        if self.scope_places().is_empty() {
            self.message = Some(Msg::AcrossNoTables.into());
            return;
        }
        self.across_stop();
        self.across.word = None;
        self.across.found.clear();
        self.across.total = 0;
        self.open_ask(Ask::Across);
    }

    /// navigation.across-results・navigation.across-progress: 語 `word` で、範囲の表を裏の糸で探し始める。
    pub(crate) fn across_begin(&mut self, word: &str) {
        self.across_stop();
        let places = self.scope_places();
        let generation = self.across.generation;
        self.across.word = Some(word.to_string());
        self.across.found.clear();
        self.across.total = places.len();
        if let Some(p) = &mut self.palette {
            p.sel = 0;
        }
        // 表の行は、通知の欄と同じく `.base` の最上位の絞り込みと、いつも隠す条件のあと(notice.count-rows)。
        let hides: Vec<Expr> = self
            .hiding()
            .iter()
            .filter_map(|c| self.cond.all.expr(&c.name).cloned())
            .collect();
        // 見出しの文言は今の言語で(裏の糸は言語の切り替えを見ない)。
        let note_head = super::view::LABEL_HEADER.text().to_string();
        let stop = Arc::new(AtomicBool::new(false));
        self.across.stop = Some(stop.clone());
        let (tx, rx) = mpsc::channel();
        let (today, now) = (self.today, self.now);
        let word = word.to_string();
        std::thread::spawn(move || {
            for place in places {
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                let found = search_table(&place, &word, &note_head, &hides, (today, now), &stop);
                if stop.load(Ordering::Relaxed) || tx.send((generation, (place, found))).is_err() {
                    return;
                }
            }
        });
        self.across.rx = Some(rx);
    }

    /// 裏の糸の結果を拾う(描く前に呼ぶ)。古い世代の結果は捨てる。読み終えたら糸を手放す。
    pub(crate) fn across_poll(&mut self) {
        let Some(rx) = &self.across.rx else {
            return;
        };
        let mut gone = false;
        loop {
            match rx.try_recv() {
                Ok((generation, found)) => {
                    if generation == self.across.generation {
                        self.across.found.push(found);
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    gone = true;
                    break;
                }
            }
        }
        if gone || self.across.found.len() >= self.across.total {
            self.across.rx = None;
            self.across.stop = None;
        }
    }

    /// 読んでいる途中か(画面を短い間隔で描き直す)。
    pub(crate) fn across_busy(&self) -> bool {
        self.across.rx.is_some()
    }

    /// navigation.across-progress: 探すのを止める(Esc・探し直し・パレットを閉じたとき)。届いていない結果は捨てる。
    pub(crate) fn across_stop(&mut self) {
        if let Some(s) = self.across.stop.take() {
            s.store(true, Ordering::Relaxed);
        }
        self.across.rx = None;
        self.across.generation += 1;
    }

    /// 当たった行(読み終えた表の順に平らにしたもの)。
    fn across_hits(&self) -> Vec<(&Place, &Hit)> {
        self.across
            .found
            .iter()
            .filter_map(|(p, r)| r.as_ref().ok().map(|hits| (p, hits)))
            .flat_map(|(p, hits)| hits.iter().map(move |h| (p, h)))
            .collect()
    }

    /// 結果の中で選べる行の数(パレットの ↑↓ の端)。
    pub(crate) fn across_hit_count(&self) -> usize {
        self.across_hits().len()
    }

    /// パレットの Enter: 打った語が探した語と違えば探し、同じなら選んだ行を開く。
    pub(crate) fn across_run(&mut self, query: &str, sel: usize) {
        let word = query.trim();
        if word.is_empty() {
            self.message = Some(Msg::AcrossNoWord.into());
            return;
        }
        if self.across.word.as_deref() != Some(word) {
            self.across_begin(word);
            return;
        }
        self.across_open(sel);
    }

    /// navigation.across-open: 結果の `k` 番目の行の表を開き、検索の欄に語を入れてその行を選ぶ。今の表ならその場で。
    /// ためた変更があれば、引き出しで移るとき(workspace.drawer-unsaved)と同じく終わる手順で確かめる。
    fn across_open(&mut self, k: usize) {
        let Some((place, hit)) = self
            .across_hits()
            .get(k)
            .map(|(p, h)| ((*p).clone(), (*h).clone()))
        else {
            return;
        };
        let word = self.across.word.clone().unwrap_or_default();
        self.close_palette();
        let row = PathBuf::from(&hit.row);
        let here = super::drawer::here(self)
            .is_some_and(|h| mdgrid::workspace::same_path(&place.path, &h));
        if here {
            self.search_for(&word);
            self.select_note(&row);
            return;
        }
        if !mdgrid::workspace::openable(&place.path) {
            self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
            return;
        }
        self.switch_search = Some((place.path.clone(), word));
        self.switch_to = Some(place);
        self.switch_select = Some(row);
        self.switch_cond = None;
        // 前に取りやめた道の範囲の旗を拾わない(範囲は今のまま)。
        self.switch_workspace = None;
        self.switch_leave_workspace = false;
        self.begin_quit();
    }

    /// navigation.across-open: 検索の欄に語を入れて行を絞る(`\` で打って Enter と同じ)。
    pub(crate) fn search_for(&mut self, word: &str) {
        self.filter = (!word.is_empty()).then(|| word.to_string());
        self.relayout();
        self.refresh_if_needed();
    }
}

/// navigation.across-results: 表 `place` をファイルから開き、行(`.base` の最上位の絞り込みと、いつも隠す条件 `hides`
/// のあと)のうち、ノートの名前か開くビューの列の素の値に `word` が当たる行を返す(表の中の検索と同じ当て方)。
/// 列は、`.base` なら開くビュー(登録したビューか先頭のビュー)の列、ほかは表の既定の列(ノートのキー)。
/// 当たった列は左から最初のもの、列に無くノートの名前に当たったら `note_head`。開けない表は理由。
fn search_table(
    place: &Place,
    word: &str,
    note_head: &str,
    hides: &[Expr],
    (today, now): (i64, i64),
    stop: &AtomicBool,
) -> Result<Vec<Hit>, String> {
    let target = crate::open_target(std::slice::from_ref(&place.path), None)?;
    let mut src = target.src;
    while !src.load(crate::LOAD_BUDGET).done {
        if stop.load(Ordering::Relaxed) {
            src.cancel();
            return Ok(Vec::new());
        }
    }
    let src = src.as_ref();
    let base = target.base.map(|(b, _, _)| b);
    let prop = |r: &RowId, k: &str| src.get(r, k).value;
    let columns = match &base {
        Some(b) => {
            let at = place
                .view
                .as_ref()
                .and_then(|v| b.views.iter().position(|x| &x.name == v))
                .unwrap_or(0);
            b.build(at, src, &prop, today, now)
                .map_or_else(|_| base::default_grid(src).columns, |g| g.columns)
        }
        None => base::default_grid(src).columns,
    };
    let hidden = |r: &RowId| {
        hides.iter().any(|e| {
            let v = match &base {
                Some(b) => b.eval_row(src, &prop, r, e, today, now),
                None => base::eval_plain(src, &prop, r, e, today, now),
            };
            mdgrid::expr::passes(&v)
        })
    };
    let text = |r: &RowId, c: &str| match &base {
        Some(b) => match b.cell(src, &prop, r, c, today, now) {
            Shown::Prop(cell) => cell
                .value
                .as_ref()
                .map(mdgrid::print::value_plain)
                .unwrap_or_default(),
            Shown::Computed(v) => super::cell::val_text(&v),
            Shown::Unsupported(_) => String::new(),
        },
        None => src
            .get(r, c)
            .value
            .as_ref()
            .map(mdgrid::print::value_plain)
            .unwrap_or_default(),
    };
    let rows: Vec<RowId> = src
        .rows()
        .into_iter()
        .filter(|r| {
            base.as_ref()
                .is_none_or(|b| b.keeps(src, &prop, r, today, now))
                && !hidden(r)
        })
        .collect();
    // 見せる名前は、表の行に共通のフォルダと `.md` を除いたもの(表の名前は見出しに出るので)。
    let common = super::grid::common_folder(rows.iter().map(|r| src.label(r)));
    let mut out = Vec::new();
    for r in rows {
        // 当てるのは表の中の検索と同じくノートの名前の全部(フォルダと `.md` も)。
        let label = src.label(&r);
        let short = label.strip_prefix(common.as_str()).unwrap_or(&label);
        let name = short.strip_suffix(".md").unwrap_or(short).to_string();
        let found = columns
            .iter()
            .find_map(|c| {
                let v = text(&r, &c.id);
                matches(word, &v).then(|| (c.title.clone(), v))
            })
            .or_else(|| matches(word, &label).then(|| (note_head.to_string(), name.clone())));
        if let Some((col, value)) = found {
            out.push(Hit {
                row: r.0.clone(),
                note: name,
                col,
                value,
            });
        }
    }
    Ok(out)
}

/// 値の見せる部分: 1行にし、当たった所が幅 `room` に入らなければ、その少し前から `…` を付けて見せる。
fn around(value: &str, word: &str, room: usize) -> String {
    let one: String = value
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    // 当て方は nav::matches と同じ(語に大文字が無ければ大文字小文字を区別しない)。
    let hay = if word.chars().any(char::is_uppercase) {
        one.clone()
    } else {
        one.to_lowercase()
    };
    // 小文字にしても字の数が変わらないときだけ、字の位置で切る(変わる字があれば頭から見せる)。
    let at = (hay.chars().count() == one.chars().count())
        .then(|| hay.find(word).map(|b| hay[..b].chars().count()))
        .flatten()
        .unwrap_or(0);
    let upto: String = one.chars().take(at + word.chars().count()).collect();
    if at <= 6 || width(&sanitize(&upto)) <= room {
        return sanitize(&one);
    }
    let skip = at - 6;
    format!("…{}", sanitize(&one.chars().skip(skip).collect::<String>()))
}

/// navigation.across-results・navigation.across-progress: 入力の下に重ねる行(進み・表ごとの当たり・
/// 当たりの無いとき・読めなかった表)。`room` 行に収め、選んだ行が見えるように流す。
pub(crate) fn across_lines(app: &App, w: usize, room: usize) -> Vec<Line<'static>> {
    let st = &app.across;
    let sel = app.palette.as_ref().map_or(0, |p| p.sel);
    let mut rows: Vec<Row> = Vec::new();
    let plain = |t: String| Row {
        line: Line::from(fit(&t, w, Align::Left)),
        hit: None,
    };
    let dim = Style::default().add_modifier(Modifier::DIM);
    let Some(word) = &st.word else {
        let n = app.scope_places().len();
        return vec![Line::from(Span::styled(
            fit(&format!(" {}", Msg::AcrossHint.fill(&[&n])), w, Align::Left),
            dim,
        ))];
    };
    if app.across_busy() {
        rows.push(Row {
            line: Line::from(Span::styled(
                fit(
                    &format!(
                        " {}",
                        Msg::AcrossProgress.fill(&[&st.found.len(), &st.total])
                    ),
                    w,
                    Align::Left,
                ),
                dim,
            )),
            hit: None,
        });
    }
    let note_w = (w / 3).clamp(8, 28);
    let mut k = 0;
    for (place, r) in &st.found {
        let Ok(hits) = r else {
            continue;
        };
        if hits.is_empty() {
            continue;
        }
        let head = Msg::AcrossGroup.fill(&[&sanitize(&place.name), &hits.len()]);
        rows.push(Row {
            line: Line::from(Span::styled(
                fit(&format!(" {head}"), w, Align::Left),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            hit: None,
        });
        for h in hits {
            let mark = if k == sel { ">" } else { " " };
            let head = format!(
                " {mark} {} {}: ",
                fit(&sanitize(&h.note), note_w, Align::Left),
                sanitize(&h.col)
            );
            let room = w.saturating_sub(width(&head));
            let t = format!("{head}{}", around(&h.value, word, room));
            let style = if k == sel {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            rows.push(Row {
                line: Line::from(Span::styled(fit(&t, w, Align::Left), style)),
                hit: Some(k),
            });
            k += 1;
        }
    }
    if !app.across_busy() && k == 0 {
        rows.push(plain(format!(" {}", Msg::AcrossNoHits.text())));
    }
    let gone: Vec<String> = st
        .found
        .iter()
        .filter(|(_, r)| r.is_err())
        .map(|(p, _)| sanitize(&p.name))
        .collect();
    if !gone.is_empty() {
        rows.push(Row {
            line: Line::from(Span::styled(
                fit(
                    &format!(
                        " {}",
                        Msg::AcrossUnreadable.fill(&[&gone.join(Msg::ListSep.text())])
                    ),
                    w,
                    Align::Left,
                ),
                dim,
            )),
            hit: None,
        });
    }
    let room = room.clamp(1, ACROSS_ROWS);
    // 選んだ行が見えるように流す(その行が下の端に来るまで)。
    let at = rows.iter().position(|r| r.hit == Some(sel)).unwrap_or(0);
    let start = (at + 1).saturating_sub(room);
    rows.into_iter()
        .skip(start)
        .take(room)
        .map(|r| r.line)
        .collect()
}
