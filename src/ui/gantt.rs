//! ガントの画面(specs/gantt/spec.md の GA-1〜GA-7。`impl App` の続き): 今のビューの行(親子の並び・畳みのまま)を、
//! 左に名前、右に時間軸と横棒で見せる。表の画面とは別の画面の型(SC-9)で、画面の型のメニュー(REL-8)とキー `C` で
//! 切り替える。列の設定は印の `gantt = { start, end, days }`(GA-1)、無ければ日付の列の名前から推定する(GA-7)。
//! 単位とマスの計算は画面に依存しない `mdgrid::gantt`。読むだけの画面で、日付は表で直す(GA-5)。
//!
//! 行は表の `rows`(ビューを通り、親子で並べ替えた行)のうち畳まれていないもの。深さ・番号・進み具合は表と同じ
//! (`App::tree`・`wbs_num`・`wbs_pct`。NV-27・NV-28)。親に日付が無ければ子孫の棒の範囲(GA-3)。

use super::app::App;
use super::keymap::{Action, Mode};
use super::relmap::panel;
use super::width::{fit, sanitize, width, Align};
use mdgrid::gantt::{self, Bar, GanttSpec, Unit};
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;
use mdgrid::types::{parse_date, Kind};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;

/// ガントの画面の状態(開いている間だけ)。
pub(crate) struct Gantt {
    /// 見せる行(表の並びのまま。畳まれた行を除く)。
    pub rows: Vec<RowId>,
    /// 選んでいる行(`rows` の添字)。
    pub sel: usize,
    /// 時間軸の単位。
    pub unit: Unit,
    /// 左端のマスの始まりの日(単位の境)。
    pub origin: i64,
    /// 使っている列。
    pub spec: GanttSpec,
    /// 設定でなく推定した列か。
    pub inferred: bool,
    /// 行ごとの棒(親は自分の日付、無ければ子孫の範囲)。
    pub bars: HashMap<RowId, Bar>,
}

/// 時間軸を動かしたとき、今日を置く位置(マスの何分の1か)。
const TODAY_AT: usize = 3;

/// 体の始まりの画面の行(ヘッダーの上の余白・ヘッダー・帯の上の余白の下)。
fn body_y(app: &App) -> usize {
    super::bands::top_rows(app) + 1 + super::bands::bar_padding(app)
}

/// 体の高さ(下の帯とメッセージ行を除く)。`h` は描ける行の数。
fn body_h(app: &App, h: usize) -> usize {
    h.saturating_sub(body_y(app) + 2)
}

/// 窓の中の幅(枠を除く)。
fn inner_w(w: usize) -> usize {
    w.saturating_sub(2)
}

/// 名前の欄の幅(窓の中の幅から。2/5 を 16〜40 に)。
fn name_w(iw: usize) -> usize {
    (iw * 2 / 5).clamp(16, 40).min(iw.saturating_sub(8))
}

/// 時間軸のマスの数(名前の欄と1つの空白を除いた残り)。
fn axis_cols(iw: usize) -> usize {
    iw.saturating_sub(name_w(iw) + 1).max(1)
}

/// 今の画面の幅から、窓の中の幅(描画 `ui::draw` と同じく右の1列を除く)。
fn inner_now(app: &App) -> usize {
    inner_w(super::popup::screen(app).0)
}

/// 見せる行の数 `vis` の窓で、`pos` 番目が見えるようにするずらし。
fn scroll(pos: usize, vis: usize) -> usize {
    (pos + 1).saturating_sub(vis.max(1))
}

/// 選んだものの見た目(色なしでは反転。SR-15)。
fn sel_style(app: &App) -> Style {
    match super::look::look(app) {
        Some(l) => l.selected(),
        None => Style::default().add_modifier(Modifier::REVERSED),
    }
}

/// 単位の文言。
fn unit_text(u: Unit) -> &'static str {
    match u {
        Unit::Day => Msg::GanttUnitDay.text(),
        Unit::Week => Msg::GanttUnitWeek.text(),
        Unit::Month => Msg::GanttUnitMonth.text(),
    }
}

impl App {
    /// GA-1・GA-7: この表のガントの列(印の設定か、無ければ日付の列の名前からの推定)と、推定したかどうか。
    /// 読み込みの途中は印の設定を列を待たずに返す。
    pub(crate) fn gantt_spec(&self) -> Option<(GanttSpec, bool)> {
        if self.gantt_off {
            return None;
        }
        if let Some((spec, _)) = &self.gantt_spec {
            if !self.progress.done || self.gantt_columns_ok(spec) {
                return Some((spec.clone(), false));
            }
        }
        let cols: Vec<(String, bool)> = self
            .src
            .columns()
            .into_iter()
            .map(|c| {
                let is_date = matches!(self.col_type(&c), Kind::Date | Kind::DateTime);
                (c, is_date)
            })
            .collect();
        gantt::infer(&cols).map(|s| (s, true))
    }

    /// 設定の列が表にあるか(ノートのキーの列か、計算の列・ファイルの列)。
    fn gantt_columns_ok(&self, spec: &GanttSpec) -> bool {
        let cols = self.src.columns();
        spec.columns().iter().all(|c| {
            c.starts_with("formula.") || c.starts_with("file.") || cols.iter().any(|x| x == c)
        })
    }

    /// GA-1: この表でガントを出せるか。
    pub(crate) fn gantt_ready(&self) -> bool {
        self.gantt_spec().is_some()
    }

    /// GA-1: 読み終えたときに、設定の列が表に無ければ CLI-3 の形で警告し、ガントを出さない(`gantt_off`。
    /// 設定を書いた人の意図に反して推定で開かない)。
    pub(crate) fn check_gantt_column(&mut self) {
        let Some((spec, at)) = self.gantt_spec.clone() else {
            return;
        };
        if self.gantt_columns_ok(&spec) {
            return;
        }
        let cols = self.src.columns();
        let is_missing = |c: &str| {
            !(c.starts_with("formula.") || c.starts_with("file.") || cols.iter().any(|x| x == c))
        };
        let (key, missing) = [
            ("start", &spec.start),
            ("end", &spec.end),
            ("days", &spec.days),
        ]
        .into_iter()
        .find_map(|(k, c)| {
            c.as_deref()
                .filter(|c| is_missing(c))
                .map(|c| (k, c.to_string()))
        })
        .unwrap_or(("start", String::new()));
        let w = Msg::SettingWarn.fill(&[
            &format!(
                "{}/{}",
                mdgrid::workspace::MARKER_DIR,
                mdgrid::workspace::MARKER_FILE
            ),
            &format!("{at}.{key}"),
            &Msg::RsnGanttNoColumn.fill(&[&missing]),
        ]);
        self.gantt_spec = None;
        self.gantt_off = true;
        self.message = Some(match self.message.take() {
            Some(m) => format!("{m} / {w}"),
            None => w,
        });
    }

    /// 行の列の値を日付として読む(ためた値を重ねた文字から。日時は日の部分)。
    fn gantt_day(&self, row: &RowId, col: &str) -> Option<i64> {
        let s = self.plain(row, col);
        let s = s.trim();
        parse_date(s).or_else(|| s.get(..10).and_then(parse_date))
    }

    /// 行の自分の日付から棒を作る(子の範囲は見ない)。
    fn own_bar(&self, row: &RowId, spec: &GanttSpec) -> Bar {
        let day = |c: &Option<String>| c.as_deref().and_then(|c| self.gantt_day(row, c));
        let days = spec
            .days
            .as_deref()
            .and_then(|c| self.plain(row, c).trim().parse::<f64>().ok())
            .map(|d| d.round().clamp(-1e15, 1e15) as i64);
        Bar::from_values(day(&spec.start), day(&spec.end), days)
    }

    /// GA-3: 行ごとの棒。親に自分の日付があればそれ、無ければ子孫(畳まれた分も含む)の棒の範囲。
    /// 各行の自分の棒を祖先の全部に畳み込む(行数 × 深さ)。
    fn gantt_bars(&self, spec: &GanttSpec) -> HashMap<RowId, Bar> {
        let own: HashMap<RowId, Bar> = self
            .rows
            .iter()
            .map(|r| (r.clone(), self.own_bar(r, spec)))
            .collect();
        let mut span: HashMap<RowId, Bar> = HashMap::new();
        for r in &self.rows {
            let Some(bar) = own.get(r).filter(|b| b.is_some()) else {
                continue;
            };
            let mut cur = self.tree.get(r).and_then(|n| n.parent.as_ref());
            let mut steps = 0;
            while let Some(p) = cur {
                let e = span.entry(p.clone()).or_default();
                *e = Bar::span(&[*e, *bar]);
                steps += 1;
                if steps > self.tree.len() {
                    break;
                }
                cur = self.tree.get(p).and_then(|n| n.parent.as_ref());
            }
        }
        let mut out = own;
        for r in &self.rows {
            let has_kids = self.tree.get(r).is_some_and(|n| n.has_kids);
            if has_kids && !out.get(r).is_some_and(|b| b.is_some()) {
                out.insert(r.clone(), span.get(r).copied().unwrap_or_default());
            }
        }
        out
    }

    /// 見せる行(表の並びのうち畳まれていないもの)。
    fn gantt_rows(&self) -> Vec<RowId> {
        self.rows
            .iter()
            .filter(|r| !self.under_folded(r))
            .cloned()
            .collect()
    }

    /// GA-2: ガントの画面を開く。設定も推定も無ければ理由、行が無ければ理由。開くときの単位は、日付の範囲が
    /// 幅に収まる最も細かいもの。今の表の行があればそれを選ぶ。
    pub(crate) fn open_gantt(&mut self) {
        let Some((spec, inferred)) = self.gantt_spec() else {
            self.message = Some(Msg::GanttNotSet.text().into());
            return;
        };
        let rows = self.gantt_rows();
        if rows.is_empty() {
            self.message = Some(Msg::GanttNoRows.text().into());
            return;
        }
        let bars = self.gantt_bars(&spec);
        let cols = axis_cols(inner_now(self));
        let bounds = rows
            .iter()
            .filter_map(|r| bars.get(r).and_then(Bar::bounds))
            .fold(None, |acc: Option<(i64, i64)>, (s, e)| {
                Some(acc.map_or((s, e), |(a, b)| (a.min(s), b.max(e))))
            });
        let (unit, origin) = match bounds {
            Some((first, last)) => {
                let unit = gantt::fit_unit(first, last, cols);
                (unit, gantt::floor_unit(first, unit))
            }
            None => {
                self.message = Some(Msg::GanttNoDates.text().into());
                let unit = Unit::Day;
                let today = gantt::floor_unit(self.today, unit);
                let back = i64::try_from(cols / TODAY_AT).unwrap_or(0);
                (unit, gantt::cell_start(-back, today, unit))
            }
        };
        let sel = self
            .cur_row()
            .and_then(|cur| rows.iter().position(|r| *r == cur))
            .unwrap_or(0);
        self.relmap = None;
        self.folder_tree = None;
        self.gantt = Some(Gantt {
            rows,
            sel,
            unit,
            origin,
            spec,
            inferred,
            bars,
        });
        self.set_mode(Mode::Gantt);
    }

    /// 表の画面に戻る。
    pub(crate) fn close_gantt(&mut self) {
        self.gantt = None;
        self.set_mode(Mode::Table);
    }

    /// GA-5: 外で変わった・畳んだときに、今のビューを通った行で組み直す(選んでいた行・単位・位置は保つ)。
    pub(crate) fn rebuild_gantt(&mut self) {
        let Some(g) = self.gantt.as_ref() else {
            return;
        };
        let keep = g.rows.get(g.sel).cloned();
        let (spec, inferred) = match self.gantt_spec() {
            Some(s) => s,
            None => (g.spec.clone(), g.inferred),
        };
        let rows = self.gantt_rows();
        let bars = self.gantt_bars(&spec);
        let sel = keep
            .and_then(|k| rows.iter().position(|r| *r == k))
            .unwrap_or(0)
            .min(rows.len().saturating_sub(1));
        if let Some(g) = self.gantt.as_mut() {
            g.rows = rows;
            g.bars = bars;
            g.sel = sel;
            g.spec = spec;
            g.inferred = inferred;
        }
    }

    /// GA-4: 選んだ行を表の画面で選ぶ。
    fn gantt_show_in_table(&mut self) {
        let Some(row) = self.gantt.as_ref().and_then(|g| g.rows.get(g.sel).cloned()) else {
            return self.close_gantt();
        };
        self.close_gantt();
        let at = self.rows.iter().position(|r| *r == row).and_then(|r| {
            self.slots
                .iter()
                .position(|s| *s == super::grid::Slot::Row(r))
        });
        match at {
            Some(i) => {
                self.row = i;
                self.scroll_into_view();
            }
            None => self.message = Some(Msg::TreeRowHidden.text().into()),
        }
    }

    /// GA-3: 選んだ行が子を持てば、その子孫を畳む・開く(表の畳みと同じ鍵)。
    fn gantt_toggle(&mut self) {
        let Some(row) = self.gantt.as_ref().and_then(|g| g.rows.get(g.sel).cloned()) else {
            return;
        };
        if !self.tree.get(&row).is_some_and(|n| n.has_kids) {
            self.message = Some(Msg::TreeNoKids.into());
            return;
        }
        if !self.tree_folded.remove(&row) {
            self.tree_folded.insert(row.clone());
        }
        // 表の今の行は鍵で保つ(畳まれて見えなくなれば畳んだ親へ)。
        self.build_slots_keep_row(&row);
        self.rebuild_gantt();
    }

    /// GA-4: 時間軸を今日が見える位置へ(今日を左から 1/3 に。戻す数は字でなくマスで数える)。
    fn gantt_today(&mut self) {
        let cols = axis_cols(inner_now(self));
        if let Some(g) = self.gantt.as_mut() {
            let today = gantt::floor_unit(self.today, g.unit);
            let cells = i64::try_from(cols).unwrap_or(0) / gantt::col_chars(g.unit);
            let back = cells / i64::try_from(TODAY_AT).unwrap_or(1);
            g.origin = gantt::cell_start(-back, today, g.unit);
        }
    }

    /// GA-4: ガントの画面の動作。
    pub(crate) fn gantt_action(&mut self, action: Action) {
        if self.gantt.is_none() {
            return self.set_mode(Mode::Table);
        }
        match action {
            Action::Up | Action::Down => {
                if let Some(g) = self.gantt.as_mut() {
                    g.sel = if action == Action::Up {
                        g.sel.saturating_sub(1)
                    } else {
                        (g.sel + 1).min(g.rows.len().saturating_sub(1))
                    };
                }
            }
            Action::Left | Action::Right => {
                if let Some(g) = self.gantt.as_mut() {
                    let k = if action == Action::Left { -1 } else { 1 };
                    // 日付の端(9999-12 など)で `cell_start` が単位の境でない日を返しても、原点は境に保つ。
                    g.origin = gantt::floor_unit(gantt::cell_start(k, g.origin, g.unit), g.unit);
                }
            }
            Action::ZoomIn | Action::ZoomOut => {
                if let Some(g) = self.gantt.as_mut() {
                    let u = if action == Action::ZoomIn {
                        g.unit.finer()
                    } else {
                        g.unit.coarser()
                    };
                    if u != g.unit {
                        g.unit = u;
                        g.origin = gantt::floor_unit(g.origin, u);
                    }
                }
            }
            Action::Today => self.gantt_today(),
            Action::ToggleTree => self.gantt_toggle(),
            Action::Run => self.gantt_show_in_table(),
            Action::Close | Action::GanttView => self.close_gantt(),
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            // パレットから選んだガントに無い動作は、選んでいる行を表で選んでからする(表の別の行に
            // 書かないため)。
            a => {
                self.gantt_show_in_table();
                self.apply(a);
            }
        }
    }

    /// GA-4: ガントの画面のクリック。行のクリックで選び、選んでいる行をもう一度で表のその行へ。
    pub(crate) fn gantt_click(&mut self, _x: u16, y: u16) {
        let Some(g) = self.gantt.as_ref() else {
            return;
        };
        // 描画(`ui::draw` → `render`)と同じ高さから、行の数を出す(枠の2行と見出しの2行を除く)。
        let h = super::popup::screen(self).1 + 2;
        let top = body_y(self) + 1 + 2; // 枠の上の縁と、見出しの2行
        let vis = body_h(self, h).saturating_sub(2 + 2);
        let y = usize::from(y);
        if y < top || y >= top + vis {
            return;
        }
        let first = scroll(g.sel, vis);
        let i = first + (y - top);
        if i >= g.rows.len() {
            return;
        }
        if i == g.sel {
            return self.gantt_show_in_table();
        }
        if let Some(g) = self.gantt.as_mut() {
            g.sel = i;
        }
    }

    /// GA-4: 横のホイールで時間軸を動かす。
    pub(crate) fn gantt_hwheel(&mut self, right: bool) {
        self.gantt_action(if right { Action::Right } else { Action::Left });
    }
}

/// 時間軸の見出しの2行と、下の行で反転して示す今日の字。字はマスを `col_chars` 字ずつ並べたもの(月は3字)。
/// - 日: 上は月の始まるマス(と左端)に「YYYY-MM」、下は月曜のマスにその日の数(1〜2字)。
/// - 週: 上は年の始まる週(と左端)に「YYYY」、下は1日を含む週に月の数「MM」、ほかは「·」。
/// - 月: 上は1月のマス(と左端)に「YYYY」、下はマスの先頭から月の数(右寄せの2字)。
///
/// 上の行の字は、次の切れ目までに収まる形を選ぶ(「YYYY-MM」が収まらなければ「MM」、それも収まらなければ
/// 出さない)ので、1つの字がほかの切れ目を潰さず、年どうしもくっつかない。今日の字は、下の行が空なら `▾`、
/// 数に重なれば数を残してその字を反転で示す(色なしでも見える。SR-15)。
fn axis_header(g: &Gantt, cols: usize, today: i64) -> (String, String, Option<usize>) {
    let n = usize::try_from(gantt::col_chars(g.unit)).unwrap_or(1);
    let blank = if g.unit == Unit::Week { '·' } else { ' ' };
    let mut top = vec![' '; cols];
    let mut low = vec![blank; cols];
    let cell = |k: usize| gantt::cell_start(i64::try_from(k).unwrap_or(0), g.origin, g.unit);
    let put = |row: &mut Vec<char>, at: usize, text: &str| {
        for (i, c) in text.chars().enumerate() {
            if let Some(slot) = row.get_mut(at + i) {
                *slot = c;
            }
        }
    };
    // 上の行の切れ目(その字から新しい年月が始まる): (字, 年, 月。月は 0 なら年だけ)。
    let mut cuts: Vec<(usize, i64, u32)> = Vec::new();
    for k in 0..cols.div_ceil(n) {
        let at = k * n;
        let d = cell(k);
        let (y, m, day) = gantt::civil(d);
        let (ly, lm, _) = gantt::civil(cell(k + 1) - 1);
        match g.unit {
            Unit::Day => {
                // 月曜(1970-01-01 は木曜)。
                if (d + 3).rem_euclid(7) == 0 {
                    put(&mut low, at, &day.to_string());
                }
                if k == 0 || day == 1 {
                    cuts.push((at, y, m));
                }
            }
            Unit::Week => {
                if day == 1 || lm != m {
                    let mm = if day == 1 { m } else { lm };
                    put(&mut low, at, &format!("{mm:02}"));
                }
                if k == 0 || (m == 1 && day == 1) || ly != y {
                    cuts.push((at, ly, 0));
                }
            }
            Unit::Month => {
                put(&mut low, at, &format!("{m:>2}"));
                if k == 0 || m == 1 {
                    cuts.push((at, y, 0));
                }
            }
        }
    }
    for (i, &(at, y, m)) in cuts.iter().enumerate() {
        let gap = cuts.get(i + 1).map_or(cols, |c| c.0) - at;
        let label = match (m, gap) {
            (1.., 7..) => format!("{y:04}-{m:02}"),
            (1.., 2..) => format!("{m:02}"),
            (0, 4..) => format!("{y:04}"),
            _ => String::new(),
        };
        put(&mut top, at, &label);
    }
    let mut mark = None;
    let at = usize::try_from(gantt::col_of(today, g.origin, g.unit)).ok();
    if let Some(slot) = at.and_then(|j| low.get_mut(j)) {
        if *slot == blank {
            *slot = '▾';
        } else {
            mark = at;
        }
    }
    (top.into_iter().collect(), low.into_iter().collect(), mark)
}

/// GA-2: 時間軸の `j` 字目の地の段(`look::axis_band` の level)。1 の位は帯で、日・週の単位では月ごと、月の単位では
/// 年ごとに交互(週は週の最後の日の月)。2 の位は日の単位の土日。縦線の字を使わないので、棒と今日の印 `┊` を切らない。
fn band_of(g: &Gantt, j: usize) -> u8 {
    let (s, e) = gantt::col_days(i64::try_from(j).unwrap_or(0), g.origin, g.unit);
    // 週は週の最後の日の月(見出しの月の数「MM」を1日を含む週に書くのと揃える)。空の範囲(日付の端 9999-12 より
    // 右の旬)は始まりの日が端の外なので、最後の日で決める(10000 年と読んで3字ごとの縞にしない)。
    let (y, m, _) = gantt::civil(if g.unit == Unit::Week || s > e { e } else { s });
    let n = match g.unit {
        Unit::Month => y,
        Unit::Day | Unit::Week => y * 12 + i64::from(m),
    };
    let weekend = g.unit == Unit::Day && (s + 3).rem_euclid(7) >= 5;
    u8::from(n.rem_euclid(2) == 1) | (u8::from(weekend) << 1)
}

/// Span の列をちょうど幅 `w` にする(はみ出す分を切り、足りなければ空白で埋める)。曖昧な字を2桁で数える設定
/// (CV-6)では `█` `┊` `◆` `·` が2桁になり、時間軸の字の数より広くなるので、枠(`relmap::panel` は広すぎる行を
/// 切らない)からはみ出さないように切る。
fn fit_spans(spans: Vec<Span<'static>>, w: usize) -> Vec<Span<'static>> {
    let mut out = Vec::with_capacity(spans.len() + 1);
    let mut used = 0;
    for sp in spans {
        let sw = width(&sp.content);
        if used + sw <= w {
            used += sw;
            out.push(sp);
            continue;
        }
        let t = super::width::take(&sp.content, w - used);
        used += width(&t);
        out.push(Span::styled(t, sp.style));
        break;
    }
    if used < w {
        out.push(Span::raw(" ".repeat(w - used)));
    }
    out
}

/// 時間軸の字 `text` を、字ごとの地の帯(色なしでは無し)を付けた Span に分ける。`mark` の字は反転。
fn axis_spans(app: &App, g: &Gantt, text: &str, mark: Option<usize>) -> Vec<Span<'static>> {
    let colors: Vec<_> = (0..4).map(|l| super::look::axis_band(app, l)).collect();
    let style = |j: usize| {
        // 今日の印の字は反転だけ(帯の地を付けない): 地を持つ反転は `theme::paint` が自分の色とみなして塗らず、
        // classic の見た目では素の反転のまま残るため。モダンな見た目では選びの見た目に置き換わる。
        if mark == Some(j) {
            return Style::default().add_modifier(Modifier::REVERSED);
        }
        match colors[usize::from(band_of(g, j))] {
            Some(c) => Style::default().bg(c),
            None => Style::default(),
        }
    };
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_st: Option<Style> = None;
    for (j, c) in text.chars().enumerate() {
        let st = style(j);
        if run_st.is_some_and(|s| s != st) {
            out.push(Span::styled(
                std::mem::take(&mut run),
                run_st.unwrap_or_default(),
            ));
        }
        run_st = Some(st);
        run.push(c);
    }
    if !run.is_empty() {
        out.push(Span::styled(run, run_st.unwrap_or_default()));
    }
    out
}

/// 1行の棒の文字(幅 `cols` 字)。棒は `█`(進み具合ぶん)と `░`、印は `◆`、今日の空の字は `┊`。
/// 字の日の範囲は `col_days`(月の単位は旬)。塗りは棒の全長(字の数)に対する割合で、棒の始まりの字から
/// 数える(左にはみ出していても同じ)。
fn bar_cells(g: &Gantt, bar: Bar, pct: Option<u8>, cols: usize, today: i64) -> String {
    let mut out = String::with_capacity(cols);
    // 棒の始まりと終わりの字(画面の外も含む)と、塗る字の数。
    let span = bar.range().map(|(bs, be)| {
        let k0 = gantt::col_of(bs, g.origin, g.unit);
        let k1 = gantt::col_of(be, g.origin, g.unit);
        let total = usize::try_from(k1 - k0 + 1).unwrap_or(0);
        let filled = pct.map_or(total, |p| (total * usize::from(p.min(100))).div_ceil(100));
        (k0, k1, i64::try_from(filled).unwrap_or(i64::MAX))
    });
    for k in 0..cols {
        let ki = i64::try_from(k).unwrap_or(0);
        let (s, e) = gantt::col_days(ki, g.origin, g.unit);
        let c = if let Some((k0, _, filled)) = span.filter(|(k0, k1, _)| (*k0..=*k1).contains(&ki))
        {
            if ki - k0 < filled {
                '█'
            } else {
                '░'
            }
        } else if bar.point().is_some_and(|d| (s..=e).contains(&d)) {
            '◆'
        } else if (s..=e).contains(&today) {
            '┊'
        } else {
            ' '
        };
        out.push(c);
    }
    out
}

/// 行の名前の欄: `番号 字下げ 名前`(選んでいる行は `>`)。
fn name_cell(app: &App, g: &Gantt, row: &RowId, w: usize) -> String {
    let sel = g.rows.get(g.sel) == Some(row);
    let depth = app.tree.get(row).map_or(0, |n| n.depth);
    let num = app
        .wbs_num
        .get(row)
        .map(|n| format!("{n} "))
        .unwrap_or_default();
    let head = format!("{}{num}{}", if sel { ">" } else { " " }, "  ".repeat(depth));
    // 表のノートの欄と同じ見せ方(`.md` を除き、フォルダは note_path に従う)。名前を出さない設定なら名前。
    let shown = super::view::label(app, row);
    let name = if shown.trim().is_empty() {
        sanitize(&app.src.label(row))
    } else {
        shown
    };
    fit(
        &format!(
            "{head}{}",
            fit(&name, w.saturating_sub(width(&head)), Align::Left)
        ),
        w,
        Align::Left,
    )
}

/// 窓の題: 「ガント: start → due(日)」(推定なら「推定」を添える。無い列は `-`)。
fn title(g: &Gantt) -> String {
    let start = g.spec.start.clone().unwrap_or_else(|| "-".into());
    let end = g
        .spec
        .end
        .clone()
        .or_else(|| g.spec.days.clone())
        .unwrap_or_else(|| "-".into());
    let unit = if g.inferred {
        format!("{}, {}", unit_text(g.unit), Msg::GanttInferred.text())
    } else {
        unit_text(g.unit).to_string()
    };
    Msg::GanttTitle.fill(&[&sanitize(&start), &sanitize(&end), &unit])
}

/// 体の中身(見出し2行と行)。
fn body_lines(app: &App, g: &Gantt, iw: usize, vis: usize) -> Vec<Line<'static>> {
    let nw = name_w(iw);
    let cols = axis_cols(iw);
    let (top, low, mark) = axis_header(g, cols, app.today);
    // 見出しの2行: 名前の欄の幅の空白のあとに時間軸(地の帯つき。今日が数に重なればその字を反転)。
    let pad = || Span::raw(" ".repeat(nw + 1));
    let head = |text: &str, mark: Option<usize>| {
        let mut spans = vec![pad()];
        spans.extend(axis_spans(app, g, text, mark));
        Line::from(fit_spans(spans, iw))
    };
    let mut lines = vec![head(&top, None), head(&low, mark)];
    let rows_vis = vis.saturating_sub(2);
    let first = scroll(g.sel, rows_vis);
    for (i, row) in g.rows.iter().enumerate().skip(first).take(rows_vis) {
        let bar = g.bars.get(row).copied().unwrap_or_default();
        let pct = app.wbs_pct.get(row).copied();
        let name = name_cell(app, g, row, nw);
        let cells = bar_cells(g, bar, pct, cols, app.today);
        // 選んでいる行は行の全体を選びの見た目に(帯より先)。ほかの行は時間軸に地の帯。
        if i == g.sel {
            let text = format!("{name} {cells}");
            lines.push(Line::from(Span::styled(
                fit(&text, iw, Align::Left),
                sel_style(app),
            )));
        } else {
            let mut spans = vec![Span::raw(format!("{name} "))];
            spans.extend(axis_spans(app, g, &cells, None));
            lines.push(Line::from(fit_spans(spans, iw)));
        }
    }
    lines
}

/// 画面の全部の行(幅 `w`、高さ `h`)。
pub(crate) fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    let Some(g) = app.gantt.as_ref() else {
        return vec![Line::from(" ".repeat(w)); h];
    };
    let mut lines = super::bands::top_lines(app, w);
    lines.push(super::bands::header(app, w));
    for _ in 0..super::bands::bar_padding(app) {
        lines.push(Line::from(" ".repeat(w)));
    }
    let bh = h.saturating_sub(lines.len() + 2);
    let iw = inner_w(w);
    let body = body_lines(app, g, iw, bh.saturating_sub(2));
    lines.extend(panel(app, &title(g), body, w, bh, true));
    let body_end = h.saturating_sub(2);
    while lines.len() < body_end {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body_end);
    // REL-8: 画面の型のメニューの窓(ガントから開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // パレット(ガントから開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
