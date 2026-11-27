//! ガントの画面(specs/_history/decidespec/gantt/spec.md の GA-1〜GA-7。`impl App` の続き): 今のビューの行(親子の並び・畳みのまま)を、
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
use mdgrid::tree::SpanRule;
use mdgrid::types::{parse_date, Kind, WeekStart};
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
    /// gantt.range-key: 最後に選んだ期間(`r` の次の期間を決める。開いたときは無し)。
    /// ビューごとには覚えない(tbd~gantt.range-remember)。
    pub range: Option<Range>,
    /// gantt.children-span-mark: 親の期間からはみ出した子と、その親の行(子としてはみ出す日数, 親として子孫が
    /// はみ出す日数。どちらも一番多いもの)。印の children の `gantt` が `mark` のときだけ。
    pub over: HashMap<RowId, (i64, i64)>,
}

/// gantt.children-span-mark: はみ出した子と親の行の名前の欄の右端に付ける、色に頼らない印。
pub(crate) const SPAN_MARK: &str = "!";

/// gantt.range-presets: 選べる期間。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Range {
    Week,
    Month,
    Quarter,
    All,
}

impl Range {
    /// `r` で次に移る期間(今週 → 今月 → 四半期 → 全体 → 今週)。
    fn next(prev: Option<Range>) -> Range {
        match prev {
            None | Some(Range::All) => Range::Week,
            Some(Range::Week) => Range::Month,
            Some(Range::Month) => Range::Quarter,
            Some(Range::Quarter) => Range::All,
        }
    }
}

/// gantt.controls-bar: 操作の帯の項目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctl {
    Earlier,
    Later,
    Unit(Unit),
    Today,
    Range(Range),
}

/// gantt.controls-bar: 操作の帯の項目の字と動作(この順に並べる)。今の単位は `[ ]` で囲む(色に頼らない)。
fn controls(g: &Gantt) -> Vec<(String, Ctl)> {
    let unit = |u: Unit| {
        let t = unit_text(u);
        let t = if u == g.unit {
            format!("[{t}]")
        } else {
            t.to_string()
        };
        (t, Ctl::Unit(u))
    };
    vec![
        ("◀".to_string(), Ctl::Earlier),
        ("▶".to_string(), Ctl::Later),
        unit(Unit::Day),
        unit(Unit::Week),
        unit(Unit::Month),
        (Msg::GanttToday.text().to_string(), Ctl::Today),
        (
            Msg::GanttThisWeek.text().to_string(),
            Ctl::Range(Range::Week),
        ),
        (
            Msg::GanttThisMonth.text().to_string(),
            Ctl::Range(Range::Month),
        ),
        (
            Msg::GanttQuarter.text().to_string(),
            Ctl::Range(Range::Quarter),
        ),
        (Msg::GanttAll.text().to_string(), Ctl::Range(Range::All)),
    ]
}

/// 操作の帯の項目の置き場(窓の左端からの桁・幅・動作)。描画とクリックで同じものを使う。項目の間は2桁あけ、
/// 窓の幅(枠を含む `w`)に入る項目だけを出す。
fn control_spots(g: &Gantt, w: usize) -> Vec<(usize, String, Ctl)> {
    let mut out = Vec::new();
    // 左下の角・横の線1字・空白1字のあとから。
    let mut x = 3;
    for (text, ctl) in controls(g) {
        let tw = width(&text);
        // 右に、空白1字・横の線1字・右下の角を残す。
        if x + tw + 3 > w {
            break;
        }
        out.push((x, text, ctl));
        x += tw + 2;
    }
    out
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
        // 印の設定があれば、列が出そろうまでは設定のまま、出そろって列が無ければ出さない(推定に落ちない)。
        if let Some((spec, _)) = &self.gantt_spec {
            if !self.columns_settled() || self.gantt_columns_ok(spec) {
                return Some((spec.clone(), false));
            }
            return None;
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
        spec.columns()
            .iter()
            .all(|c| c.starts_with("formula.") || c.starts_with("file.") || self.has_column(c))
    }

    /// GA-1: この表でガントを出せるか。
    pub(crate) fn gantt_ready(&self) -> bool {
        self.gantt_spec().is_some()
    }

    /// 設定の列のうち表に無い最初の列(`start`・`end`・`days` の鍵と列の名前)。
    fn gantt_missing(&self, spec: &GanttSpec) -> Option<(&'static str, String)> {
        let is_missing =
            |c: &str| !(c.starts_with("formula.") || c.starts_with("file.") || self.has_column(c));
        [
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
    }

    /// GA-1: 列が出そろったときに、設定の列が表に無ければ CLI-3 の形で1回警告する。設定は外さない(あとで列が
    /// できれば出す。無いあいだは `gantt_spec` が推定に落ちずに出さない)。
    pub(crate) fn check_gantt_column(&mut self) {
        let Some((spec, at)) = self.gantt_spec.clone() else {
            return;
        };
        let Some((key, missing)) = self.gantt_missing(&spec) else {
            return;
        };
        let w = Msg::SettingWarn.fill(&[
            &format!(
                "{}/{}",
                mdgrid::workspace::MARKER_DIR,
                mdgrid::workspace::MARKER_FILE
            ),
            &format!("{at}.{key}"),
            &Msg::RsnGanttNoColumn.fill(&[&missing]),
        ]);
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
            if has_kids && !out.get(r).is_some_and(mdgrid::gantt::Bar::is_some) {
                out.insert(r.clone(), span.get(r).copied().unwrap_or_default());
            }
        }
        out
    }

    /// 印の children の `key` で、表の全部の行の親(自分を指す行は入れない)。印が無いか列が表に無ければ空。
    /// 子が親の期間をはみ出したときの扱い(gantt.children-span-*)は、ビューの並びでなく、この親子でたどる
    /// (絞り込みで外れた親・別のまとまりの親も見る。親を完了にしたときの子孫と同じ)。
    fn children_parents(&self) -> HashMap<RowId, RowId> {
        let Some(key) = self.children_key() else {
            return HashMap::new();
        };
        self.src
            .rows()
            .into_iter()
            .filter_map(|r| {
                let p = self.tree_parent(&r, &key).filter(|p| *p != r)?;
                Some((r, p))
            })
            .collect()
    }

    /// gantt.children-span-mark: 自分の日付を持つ祖先の期間から、自分の棒がはみ出す子と、その祖先(行 → (子として
    /// はみ出す日数, 親として子孫がはみ出す日数)。どちらも一番多いもの)。印の children の `gantt` が `mark`
    /// (書かなければ mark。tbd~gantt.children-span-default の仮決め)のときだけ。
    fn gantt_overruns(&self, spec: &GanttSpec) -> HashMap<RowId, (i64, i64)> {
        let mut out: HashMap<RowId, (i64, i64)> = HashMap::new();
        if !self
            .children_spec
            .as_ref()
            .is_some_and(|(c, _)| c.gantt == SpanRule::Mark)
        {
            return out;
        }
        let parent = self.children_parents();
        // 親子に出てくる行の自分の棒(祖先をたどるたびに読み直さない)。
        let bars: HashMap<&RowId, Bar> = parent
            .iter()
            .flat_map(|(c, p)| [c, p])
            .map(|r| (r, self.own_bar(r, spec)))
            .collect();
        let bar = |r: &RowId| bars.get(r).copied().unwrap_or_default();
        for r in parent.keys() {
            let Some(child) = bar(r).bounds() else {
                continue;
            };
            let mut seen: std::collections::HashSet<&RowId> = std::collections::HashSet::from([r]);
            let mut cur = parent.get(r);
            while let Some(p) = cur {
                if !seen.insert(p) {
                    break;
                }
                if let Some(period) = bar(p).range() {
                    let d = gantt::overrun(period, child);
                    if d > 0 {
                        let e = out.entry(r.clone()).or_default();
                        e.0 = e.0.max(d);
                        let e = out.entry(p.clone()).or_default();
                        e.1 = e.1.max(d);
                    }
                }
                cur = parent.get(p);
            }
        }
        out
    }

    /// gantt.children-span-extend: 子 `row` の棒を `writes`(列 → 新しい文字)のとおりに直すとき、自分の日付を持つ
    /// 祖先の期間からはみ出すなら、その始まりか終わりを子の範囲まで広げる書き込み(行, 列, 文字)。印の children の
    /// `gantt` が `extend` のときだけ。親の棒を動かしても子は動かさない(tbd~gantt.children-move の仮決め)。
    fn gantt_extend(
        &self,
        row: &RowId,
        spec: &GanttSpec,
        writes: &[(String, String)],
    ) -> Vec<(RowId, String, String)> {
        if !self
            .children_spec
            .as_ref()
            .is_some_and(|(c, _)| c.gantt == SpanRule::Extend)
        {
            return Vec::new();
        }
        // 直したあとの子の範囲(書く列は新しい文字、ほかは今の値で、`own_bar` と同じ読み方)。
        let text = |col: &str| -> String {
            writes
                .iter()
                .find(|(c, _)| c == col)
                .map_or_else(|| self.plain(row, col), |(_, t)| t.clone())
        };
        let day = |col: &Option<String>| {
            let s = text(col.as_deref()?);
            let s = s.trim();
            parse_date(s).or_else(|| s.get(..10).and_then(parse_date))
        };
        let days = spec
            .days
            .as_deref()
            .and_then(|c| text(c).trim().parse::<f64>().ok())
            .map(|d| d.round().clamp(-1e15, 1e15) as i64);
        let Some((cs, ce)) = Bar::from_values(day(&spec.start), day(&spec.end), days).bounds()
        else {
            return Vec::new();
        };
        let parent = self.children_parents();
        let mut out = Vec::new();
        let mut seen: std::collections::HashSet<&RowId> = std::collections::HashSet::from([row]);
        let mut cur = parent.get(row);
        while let Some(p) = cur {
            if !seen.insert(p) {
                break;
            }
            cur = parent.get(p);
            // 自分の日付の無い親(棒は子孫の範囲。gantt.tree-wbs-parent-range)には書かない。
            let Some((s, e)) = self.own_bar(p, spec).range() else {
                continue;
            };
            if gantt::overrun((s, e), (cs, ce)) == 0 {
                continue;
            }
            let (ns, ne) = (s.min(cs), e.max(ce));
            let old = |col: &str| self.plain(p, col).trim().to_string();
            if ns < s {
                if let Some(c) = &spec.start {
                    out.push((p.clone(), c.clone(), put_day(ns, &old(c))));
                }
            }
            match (&spec.end, &spec.days) {
                (Some(c), _) if ne > e => out.push((p.clone(), c.clone(), put_day(ne, &old(c)))),
                // `days` の設定: 始まりを前へ広げても終わりを後へ広げても、日数を書き直す。
                (None, Some(c)) if ns < s || ne > e => {
                    out.push((p.clone(), c.clone(), (ne - ns + 1).to_string()));
                }
                _ => {}
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
            // 設定はあって列が表に無いなら、その列を言う(「設定が無い」と言わない)。
            let missing = self
                .gantt_spec
                .clone()
                .and_then(|(s, _)| self.gantt_missing(&s));
            self.message = Some(match missing {
                Some((_, c)) => Msg::RsnGanttNoColumn.fill(&[&c]),
                None => Msg::GanttNotSet.text().into(),
            });
            return;
        };
        let rows = self.gantt_rows();
        if rows.is_empty() {
            self.message = Some(Msg::GanttNoRows.text().into());
            return;
        }
        let bars = self.gantt_bars(&spec);
        let cols = axis_cols(inner_now(self));
        let week = self.week_start;
        let (unit, origin) = if let Some(fit) = fit_all(&rows, &bars, cols, week) {
            fit
        } else {
            self.message = Some(Msg::GanttNoDates.text().into());
            let unit = Unit::Day;
            let today = gantt::floor_unit_in(self.today, unit, week);
            let back = i64::try_from(cols / TODAY_AT).unwrap_or(0);
            (unit, gantt::cell_start(-back, today, unit))
        };
        let sel = self
            .cur_row()
            .and_then(|cur| rows.iter().position(|r| *r == cur))
            .unwrap_or(0);
        let over = self.gantt_overruns(&spec);
        self.leave_screens();
        self.gantt = Some(Gantt {
            rows,
            sel,
            unit,
            origin,
            spec,
            inferred,
            bars,
            range: None,
            over,
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
        let over = self.gantt_overruns(&spec);
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
            g.over = over;
        }
    }

    /// GA-4: 選んだ行を表の画面で選んで表に戻る。行が無いか表で見えない(畳んだまとまり)なら、ガントのまま
    /// 理由を出して false。
    fn gantt_show_in_table(&mut self) -> bool {
        let row = self.gantt.as_ref().and_then(|g| g.rows.get(g.sel).cloned());
        if row.is_none() {
            self.close_gantt();
            return false;
        }
        self.select_in_table(row)
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
        let week = self.week_start;
        if let Some(g) = self.gantt.as_mut() {
            let today = gantt::floor_unit_in(self.today, g.unit, week);
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
                let week = self.week_start;
                if let Some(g) = self.gantt.as_mut() {
                    let k = if action == Action::Left { -1 } else { 1 };
                    // 日付の端(9999-12 など)で `cell_start` が単位の境でない日を返しても、原点は境に保つ。
                    let start = gantt::cell_start(k, g.origin, g.unit);
                    g.origin = gantt::floor_unit_in(start, g.unit, week);
                }
            }
            Action::ZoomIn | Action::ZoomOut => {
                let week = self.week_start;
                if let Some(g) = self.gantt.as_mut() {
                    let u = if action == Action::ZoomIn {
                        g.unit.finer()
                    } else {
                        g.unit.coarser()
                    };
                    if u != g.unit {
                        g.unit = u;
                        g.origin = gantt::floor_unit_in(g.origin, u, week);
                    }
                }
            }
            Action::Today => self.gantt_today(),
            Action::GanttRange => {
                let next = Range::next(self.gantt.as_ref().and_then(|g| g.range));
                self.gantt_set_range(next);
            }
            Action::BarEarlier => self.gantt_edit(true, -1),
            Action::BarLater => self.gantt_edit(true, 1),
            Action::BarShorter => self.gantt_edit(false, -1),
            Action::BarLonger => self.gantt_edit(false, 1),
            Action::Undo | Action::Redo => {
                self.kanban_undo(action == Action::Redo);
                self.rebuild_gantt();
            }
            Action::ToggleTree => self.gantt_toggle(),
            Action::Run => {
                self.gantt_show_in_table();
            }
            Action::Close | Action::GanttView => self.close_gantt(),
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            // パレットから選んだガントに無い動作は、選んでいる行を表で選んでからする(表の別の行に
            // 書かないため)。
            a => {
                let row = self.gantt.as_ref().and_then(|g| g.rows.get(g.sel).cloned());
                self.act_from_screen(row, a, Msg::GanttNoRows);
            }
        }
    }

    /// gantt.range-presets: 期間を選ぶ(時間軸の左端と単位を決める。行は隠さない)。
    fn gantt_set_range(&mut self, range: Range) {
        let cols = axis_cols(inner_now(self));
        let today = self.today;
        let week = self.week_start;
        let Some(g) = self.gantt.as_mut() else {
            return;
        };
        g.range = Some(range);
        // 左端の日から最後の日までが幅に収まる最も細かい単位にし、左端は単位の境に切り下げる。
        let span = match range {
            Range::Week => {
                // gantt.week-start: 今週は設定の週の始まりの曜日から。
                g.unit = Unit::Day;
                g.origin = gantt::floor_unit_in(today, Unit::Week, week);
                return;
            }
            Range::Month => {
                let first = gantt::floor_unit(today, Unit::Month);
                Some((first, mdgrid::types::add_months(first, 1) - 1))
            }
            Range::Quarter => {
                let first = gantt::floor_unit(today, Unit::Month);
                let (_, m, _) = gantt::civil(first);
                let back = i32::try_from((m - 1) % 3).unwrap_or(0);
                let first = mdgrid::types::add_months(first, -back);
                Some((first, mdgrid::types::add_months(first, 3) - 1))
            }
            Range::All => None,
        };
        match span {
            Some((first, last)) => {
                g.unit = gantt::fit_unit_in(first, last, cols, week);
                g.origin = gantt::floor_unit_in(first, g.unit, week);
            }
            None => {
                if let Some((unit, origin)) = fit_all(&g.rows, &g.bars, cols, week) {
                    g.unit = unit;
                    g.origin = origin;
                }
            }
        }
    }

    /// gantt.controls-click: 操作の帯の項目の動作。
    fn gantt_control(&mut self, ctl: Ctl) {
        match ctl {
            Ctl::Earlier => self.gantt_action(Action::Left),
            Ctl::Later => self.gantt_action(Action::Right),
            Ctl::Today => self.gantt_today(),
            Ctl::Range(r) => self.gantt_set_range(r),
            Ctl::Unit(u) => {
                let week = self.week_start;
                if let Some(g) = self.gantt.as_mut() {
                    g.unit = u;
                    g.origin = gantt::floor_unit_in(g.origin, u, week);
                }
            }
        }
    }

    /// gantt.edit-move・gantt.edit-resize: 選んでいる行の棒を1単位動かす(`whole`)か、終わりを1単位伸ばす・縮める。
    /// 値はためる変更として、1手で書く(取り消しも1手)。動かせない棒は理由を出して何も書かない(gantt.edit-refuse)。
    fn gantt_edit(&mut self, whole: bool, dir: i32) {
        let Some(g) = self.gantt.as_ref() else {
            return;
        };
        let Some(row) = g.rows.get(g.sel).cloned() else {
            return;
        };
        let (spec, unit) = (g.spec.clone(), g.unit);
        // WB-15: 読むだけで開いたら書かない。
        if self.readonly {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        let shift = |d: i64| match unit {
            Unit::Day => d + i64::from(dir),
            Unit::Week => d + 7 * i64::from(dir),
            Unit::Month => mdgrid::types::add_months(d, dir),
        };
        // 列の今の値(ためた値を重ねたもの)の日と、日を替えた文字(日時は時刻を残す)。
        let dated = |col: &Option<String>| -> Option<(String, i64, String)> {
            let col = col.as_deref()?;
            let text = self.plain(&row, col);
            let text = text.trim().to_string();
            let day = self.gantt_day(&row, col)?;
            Some((col.to_string(), day, text))
        };
        let put = put_day;
        let (start, end) = (dated(&spec.start), dated(&spec.end));
        let mut writes: Vec<(String, String)> = Vec::new();
        if whole {
            for (col, day, old) in [&start, &end].into_iter().flatten() {
                writes.push((col.clone(), put(shift(*day), old)));
            }
            if writes.is_empty() {
                self.message = Some(Msg::GanttNoOwnDates.text().into());
                return;
            }
        } else if let Some((col, day, old)) = &end {
            let new = shift(*day);
            if start.as_ref().is_some_and(|(_, s, _)| new < *s) {
                self.message = Some(Msg::GanttCannotShrink.text().into());
                return;
            }
            writes.push((col.clone(), put(new, old)));
        } else {
            // `days` の設定: 終わりは「始まり + 日数 - 1」。伸ばした終わりから日数を出して、日数の列を書く。
            let days = spec.days.as_deref().and_then(|c| {
                let n = self.plain(&row, c).trim().parse::<f64>().ok()?.round() as i64;
                (n >= 1).then(|| (c.to_string(), n))
            });
            let (Some((_, s, _)), Some((col, n))) = (&start, days) else {
                self.message = Some(
                    if start.is_none() && end.is_none() {
                        Msg::GanttNoOwnDates
                    } else {
                        Msg::GanttNoEnd
                    }
                    .text()
                    .into(),
                );
                return;
            };
            let new = shift(s + n - 1);
            if new < *s {
                self.message = Some(Msg::GanttCannotShrink.text().into());
                return;
            }
            writes.push((col, (new - s + 1).to_string()));
        }
        // gantt.children-span-extend: 祖先の期間からはみ出すなら、祖先を広げる書き込みも同じ1手に入れる。
        let mut all: Vec<(RowId, String, String)> = writes
            .iter()
            .map(|(c, t)| (row.clone(), c.clone(), t.clone()))
            .collect();
        all.extend(self.gantt_extend(&row, &spec, &writes));
        // 書く前に、どの列も直せるかを確かめる(片方だけ書かない)。
        let mut cells = Vec::new();
        for (r, col, text) in &all {
            // 読むだけのセル(計算の列・ファイルの列など。BV-7・BV-12)は理由を出す。
            if let Err(m) = self.editable(r, col) {
                self.message = Some(m);
                return;
            }
            // CE-29: キーの名前の変更か削除をためた列は、値の書き換えで黙って上書きしない。
            if matches!(
                self.changes.pending(r, col),
                Some(mdgrid::source::NewValue::RenameKey(_) | mdgrid::source::NewValue::DeleteKey)
            ) {
                self.message = Some(Msg::KanbanKeyPending.fill(&[&sanitize(col)]));
                return;
            }
            match super::kanban::value_for(self, col, text) {
                Ok(nv) => cells.push((r.clone(), col.clone(), nv)),
                Err(m) => {
                    self.message = Some(m);
                    return;
                }
            }
        }
        if let Some(s) = self.changes.set_cells(self.src.as_ref(), &cells).pop() {
            self.message = Some(Msg::CannotCommit.fill(&[&s.reason]));
            return;
        }
        // NV-12: 直した行は、保存か移動の操作まで表の元の位置に留める。
        let mut set: Vec<(RowId, Vec<String>)> = Vec::new();
        for (r, c, v) in &all {
            self.stay.insert(r.clone());
            let item = format!("{} = {}", sanitize(c), sanitize(v));
            match set.iter_mut().find(|(x, _)| x == r) {
                Some((_, items)) => items.push(item),
                None => set.push((r.clone(), vec![item])),
            }
        }
        self.regrid = true;
        self.rebuild_gantt();
        let msgs: Vec<String> = set
            .iter()
            .map(|(r, items)| {
                let name = sanitize(&self.src.label(r));
                Msg::GanttBarSet.fill(&[&name, &items.join(", ")])
            })
            .collect();
        self.message = Some(msgs.join(" / "));
    }

    /// GA-4: ガントの画面のクリック。行のクリックで選び、選んでいる行をもう一度で表のその行へ。
    /// 窓の下の縁の操作の帯の項目は、その動作をする(gantt.controls-click)。
    pub(crate) fn gantt_click(&mut self, x: u16, y: u16) {
        let Some(g) = self.gantt.as_ref() else {
            return;
        };
        // 描画(`ui::draw` → `render`)と同じ高さから、行の数を出す(枠の2行と見出しの2行を除く)。
        let h = super::popup::screen(self).1 + 2;
        let bh = body_h(self, h);
        if bh >= 2 && usize::from(y) == body_y(self) + bh - 1 {
            let w = super::popup::screen(self).0;
            let x = usize::from(x);
            let hit = control_spots(g, w)
                .into_iter()
                .find(|(at, text, _)| (*at..*at + width(text)).contains(&x))
                .map(|(_, _, ctl)| ctl);
            if let Some(ctl) = hit {
                self.gantt_control(ctl);
            }
            return;
        }
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
            self.gantt_show_in_table();
            return;
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

/// 日付の列に書く文字: 日を `day` に替える(元の文字が日時なら時刻の部分を残す)。
fn put_day(day: i64, old: &str) -> String {
    match old.get(10..) {
        Some(rest) if parse_date(old).is_none() => {
            format!("{}{rest}", mdgrid::types::format_date(day))
        }
        _ => mdgrid::types::format_date(day),
    }
}

/// gantt.range-presets の「全体」: 行の範囲の最初の日を左端に、最後の日までが幅に収まる最も細かい単位。週は
/// `week` の曜日から(gantt.week-start)。日付を持つ行が無ければ None。
fn fit_all(
    rows: &[RowId],
    bars: &HashMap<RowId, Bar>,
    cols: usize,
    week: WeekStart,
) -> Option<(Unit, i64)> {
    let (first, last) = rows
        .iter()
        .filter_map(|r| bars.get(r).and_then(Bar::bounds))
        .fold(None, |acc: Option<(i64, i64)>, (s, e)| {
            Some(acc.map_or((s, e), |(a, b)| (a.min(s), b.max(e))))
        })?;
    let unit = gantt::fit_unit_in(first, last, cols, week);
    Some((unit, gantt::floor_unit_in(first, unit, week)))
}

/// 時間軸の見出しの2行と、下の行で反転して示す今日の字。字はマスを `col_chars` 字ずつ並べたもの(月は3字)。
/// - 日: 上は月の始まるマス(と左端)に「YYYY-MM」、下は週の始まりの曜日(`week`。gantt.week-start)のマスにその日の
///   数(1〜2字)。
/// - 週: 上は年の始まる週(と左端)に「YYYY」、下は1日を含む週に月の数「MM」、ほかは「·」。
/// - 月: 上は1月のマス(と左端)に「YYYY」、下はマスの先頭から月の数(右寄せの2字)。
///
/// 上の行の字は、次の切れ目の1字手前までに収まる形を選ぶ(「YYYY-MM」が収まらなければ「MM」、それも収まらなければ
/// 出さない)ので、1つの字がほかの切れ目を潰さず、月や年の字どうしもくっつかない。今日の字は、下の行が空なら `▾`、
/// 数に重なれば数を残してその字を反転で示す(色なしでも見える。SR-15)。
fn axis_header(
    g: &Gantt,
    cols: usize,
    today: i64,
    week: WeekStart,
) -> (String, String, Option<usize>) {
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
                // 週の始まりの曜日(週の単位のマスの始まりと同じ日)。
                if gantt::floor_unit_in(d, Unit::Week, week) == d {
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
        // 次の切れ目の前には1字の空きを残す(ちょうど収まる字を置くと「2026-112026-12」「20262027」と
        // くっつく)。最後の切れ目は右の端まで使える。
        let gap = match cuts.get(i + 1) {
            Some(next) => next.0 - at - 1,
            None => cols - at,
        };
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

/// 行の名前の欄: `番号 字下げ 名前`(選んでいる行は `>`)。親の期間からはみ出した子とその親の行は、右端に
/// ` !`(gantt.children-span-mark。色に頼らない)。
fn name_cell(app: &App, g: &Gantt, row: &RowId, w: usize) -> String {
    if g.over.contains_key(row) && w > 2 {
        return format!("{} {SPAN_MARK}", plain_name_cell(app, g, row, w - 2));
    }
    plain_name_cell(app, g, row, w)
}

impl App {
    /// gantt.band-edit-keys: 選んでいる行の棒を動かせるか(`H` `L`)と、終わりを伸ばし縮めできるか(`>` `<`)。
    /// 動かせない棒(gantt.edit-refuse: 読むだけの起動・自分の日付の無い行・読むだけの列)は (false, false)。
    /// 伸ばし縮めは、終わりの日付か(`days` の設定なら)始まりと1以上の日数があるときだけ(`gantt_edit` と同じ)。
    fn gantt_bar_keys(&self) -> (bool, bool) {
        let Some(g) = self.gantt.as_ref() else {
            return (false, false);
        };
        let Some(row) = g.rows.get(g.sel) else {
            return (false, false);
        };
        if self.readonly {
            return (false, false);
        }
        let dated = |c: &Option<String>| {
            c.as_deref()
                .filter(|c| self.gantt_day(row, c).is_some())
                .map(str::to_string)
        };
        let (start, end) = (dated(&g.spec.start), dated(&g.spec.end));
        let ok = |c: &str| self.editable(row, c).is_ok();
        let own: Vec<&String> = start.iter().chain(end.iter()).collect();
        let movable = !own.is_empty() && own.iter().all(|c| ok(c));
        let days = g.spec.days.as_deref().filter(|c| {
            start.is_some()
                && self
                    .plain(row, c)
                    .trim()
                    .parse::<f64>()
                    .is_ok_and(|n| n.round() >= 1.0)
        });
        let resizable = movable
            && match (&end, days) {
                (Some(_), _) => true,
                (None, Some(c)) => ok(c),
                (None, None) => false,
            };
        (movable, resizable)
    }
}

/// gantt.band-edit-keys: ガントの下の帯の「今押せるキー」(表のキーの順位の順)。棒を動かす4つのキーは、1つずつの長い
/// 説明の代わりに、動かせる棒を選んでいるときだけ `H/L 棒を動かす`・`>/< 伸ばす・縮める` の2つにまとめて Enter の
/// 次に置く(狭い端末でも残るように。動かせない棒では出さない)。ガントでなければ None。
pub(crate) fn hints(app: &App) -> Option<Vec<String>> {
    if app.mode != Mode::Gantt {
        return None;
    }
    use super::keymap::{key_for, ranked};
    let bar = [
        Action::BarEarlier,
        Action::BarLater,
        Action::BarLonger,
        Action::BarShorter,
    ];
    let all = ranked(&app.keys, Mode::Gantt);
    let at = all
        .iter()
        .filter(|(a, _)| !bar.contains(a))
        .position(|(a, _)| *a == Action::Run)
        .map_or(0, |p| p + 1);
    let mut out: Vec<String> = all
        .into_iter()
        .filter(|(a, _)| !bar.contains(a))
        .map(|(_, s)| s)
        .collect();
    let pair = |x: Action, y: Action, label: Msg| {
        let keys: Vec<String> = [x, y]
            .into_iter()
            .filter_map(|a| key_for(&app.keys, Mode::Gantt, a))
            .collect();
        (!keys.is_empty()).then(|| format!("{} {}", keys.join("/"), label.text()))
    };
    let (movable, resizable) = app.gantt_bar_keys();
    let mut add = Vec::new();
    if movable {
        add.extend(pair(
            Action::BarEarlier,
            Action::BarLater,
            Msg::GanttBandMove,
        ));
    }
    if resizable {
        add.extend(pair(
            Action::BarLonger,
            Action::BarShorter,
            Msg::GanttBandResize,
        ));
    }
    out.splice(at..at, add);
    Some(out)
}

/// gantt.children-span-mark: 選んでいる行の、はみ出した日数の案内(子として、無ければ親として)。下の帯に出す。
pub(crate) fn span_hint(app: &App) -> Option<String> {
    if app.mode != Mode::Gantt {
        return None;
    }
    let g = app.gantt.as_ref()?;
    let (as_child, as_parent) = g.over.get(g.rows.get(g.sel)?)?;
    Some(if *as_child > 0 {
        Msg::GanttOverParent.fill(&[as_child])
    } else {
        Msg::GanttOverKids.fill(&[as_parent])
    })
}

/// 名前の欄の印の無い形。
fn plain_name_cell(app: &App, g: &Gantt, row: &RowId, w: usize) -> String {
    let sel = g.rows.get(g.sel) == Some(row);
    // navigation.parent-child-away-mark: ガントはまとまりの見出しも薄い行も出さないので、添えた親の分は字下げしない。
    let lift = app.tree_lift.get(row).copied().unwrap_or(0);
    let depth = app
        .tree
        .get(row)
        .map_or(0, |n| n.depth.saturating_sub(lift));
    let num = app
        .wbs_num
        .get(row)
        .map(|n| format!("{n} "))
        .unwrap_or_default();
    let head = format!("{}{num}{}", if sel { ">" } else { " " }, "  ".repeat(depth));
    // 表のノートの欄と同じ見せ方(`.md` を除き、フォルダは note_path に従う)。名前を出さない設定なら名前。
    let shown = super::view::label_in(app, row, lift);
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

/// 窓の題: 「ガント: start → due(日)」(推定なら「推定」を添える。無い列は `-`)。列の名前は列の見出しと同じ
/// displayName(base-view.display-name-everywhere)。
fn title(app: &App, g: &Gantt) -> String {
    let start = g
        .spec
        .start
        .as_deref()
        .map_or_else(|| "-".into(), |c| app.title(c));
    let end = g
        .spec
        .end
        .as_deref()
        .or(g.spec.days.as_deref())
        .map_or_else(|| "-".into(), |c| app.title(c));
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
    let (top, low, mark) = axis_header(g, cols, app.today, app.week_start);
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

/// gantt.controls-bar: 窓の下の縁(`edge`)に、操作の帯の項目を並べた行。項目の間は枠の横の線でつなぐ。
fn controls_line(app: &App, g: &Gantt, w: usize, edge: &Line<'static>) -> Line<'static> {
    let spots = control_spots(g, w);
    if spots.is_empty() || w < 8 {
        return edge.clone();
    }
    let f = super::popup::frame(app);
    let st = edge.spans.first().map(|s| s.style).unwrap_or_default();
    let key_st = match super::look::look(app) {
        Some(l) => l.key(),
        None => Style::default().add_modifier(Modifier::BOLD),
    };
    let h = f.h.to_string();
    let mut spans = vec![Span::styled(format!("{}{h} ", f.bl), st)];
    let mut x = 3;
    for (at, text, _) in spots {
        if at > x {
            spans.push(Span::styled(" ".repeat(at - x), st));
        }
        x = at + width(&text);
        spans.push(Span::styled(text, key_st));
    }
    spans.push(Span::styled(" ", st));
    x += 1;
    let rest = w.saturating_sub(x + 1);
    spans.push(Span::styled(format!("{}{}", h.repeat(rest), f.br), st));
    Line::from(spans)
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
    let mut boxed = panel(app, &title(app, g), body, w, bh, true);
    // gantt.controls-bar: 窓の下の縁に操作の帯を重ねる(行と時間軸の位置は変えない)。
    if bh >= 2 {
        if let Some(last) = boxed.last_mut() {
            *last = controls_line(app, g, w, last);
        }
    }
    lines.extend(boxed);
    let body_end = h.saturating_sub(2);
    while lines.len() < body_end {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body_end);
    // REL-8: 画面の型のメニューの窓(ガントから開いたとき)。
    super::screen_menu::overlay(app, &mut lines, w);
    // workspace.switch-open: お気に入りのワークスペースの切り替えの一覧。
    super::switch::overlay(app, &mut lines, w);
    // パレット(ガントから開いたとき)。
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(super::bands::footer(app, w));
    lines.push(super::bands::message(app, w));
    lines
}
