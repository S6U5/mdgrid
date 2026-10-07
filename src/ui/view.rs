//! 描画の純関数: `(幅, 高さ, &App) → 行の列`。状態は書き換えない。
//! 画面は上から、ヘッダー・ビューのタブ・検索の欄(NV-23)・設定の帯(NV-16)・表(列の見出しと行)・
//! 下の帯・メッセージ行(SR-1)。
//! 編集のモードでは選んだセルの位置に入力ボックス(CE-1)、保存の確認のモードでは差分の画面(WB-9・WB-16)、
//! 終了の確認のモードでは表の上に確認の帯(WB-11)を出す。
//! セルの文字は width.rs で切り詰めてから ratatui に渡す(SR-9)。ヘッダー・タブ・下の帯・メッセージ行は bands.rs、
//! 詳細の表示(NV-6)は detail.rs。

use super::app::{App, ColorMode};
pub(crate) use super::bands::{
    band, bar_at, bar_rows, chip_at, chip_rows, chips, footer, header, message, prompt_cursor,
    quit_overlay, search_bar, tab_at, tab_rows, tabs,
};
use super::cell::{align_of, shown, val_text};
use super::display::{col_sep, num_span, num_width, zebra};
use super::grid::{Drag, Slot};
use super::input::input_box;
pub use super::input::{cursor, in_input};
use super::keymap::Mode;
use super::review::render_review;
use super::width::{fit, sanitize, set_ambiguous_wide, take, width, Align};
use mdgrid::expr::Val;
use mdgrid::i18n::Msg;
use mdgrid::source::RowId;
use mdgrid::types::Kind;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;

/// 表の上下にある帯の数(ヘッダー・列の見出し・下の帯・メッセージ行。タブは `tab_rows`)。
const CHROME: usize = 4;
/// 表の最初の行の画面の行(ヘッダー・列の見出しの下。タブ・検索の欄・設定の帯を除いて)。
const DATA_Y: usize = 2;
/// ノートの列の見出し(今の言語は `LABEL_HEADER.text()`)。
pub(crate) const LABEL_HEADER: Msg = Msg::NoteColumn;

/// 表の最初の行の画面の行。タブ(SR-20)・検索の欄(NV-23)・設定の帯(NV-16)が出ていれば、その分だけ下がる。
pub(crate) fn data_y(app: &App) -> usize {
    DATA_Y + tab_rows(app) + bar_rows(app) + chip_rows(app)
}

/// 高さ `area_h` の画面の、表の中の行の数。集計の行(BV-14)があれば1行減る。
pub(crate) fn table_height(app: &App, area_h: usize) -> usize {
    area_h
        .saturating_sub(CHROME + tab_rows(app) + bar_rows(app) + chip_rows(app) + summary_rows(app))
}

/// 集計の行の数(BV-14)。集計の無いビューでは 0。
pub(crate) fn summary_rows(app: &App) -> usize {
    usize::from(!app.built.summaries.is_empty())
}

/// 列の集計の文字「集計の名前 値」(BV-14)。日付はセルと同じ形(CE-22)。値が空なら名前だけ。
fn summary_text(app: &App, col: &str) -> Option<String> {
    let (_, s, v) = app.built.summaries.iter().find(|(c, _, _)| c == col)?;
    let value = match v {
        Val::Date(d) => app.date_format.format(*d),
        other => val_text(other),
    };
    let label = s.label();
    Some(if value.is_empty() {
        label.to_string()
    } else {
        format!("{label} {}", sanitize(&value))
    })
}

/// 表の下の集計の行(BV-14): ノートの列に「集計」、集計した列の下に「名前 値」。列の位置と横の送りは表と同じ。
fn summary_row(app: &App, lay: &Layout, cols: &[(usize, usize)]) -> Line<'static> {
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let sep = col_sep(app);
    let mut spans = vec![
        Span::raw(" ".repeat(lay.label_x())),
        Span::styled(fit(Msg::SummaryRow.text(), lay.label_w, Align::Left), bold),
    ];
    for &(j, cw) in cols {
        spans.push(Span::raw(sep));
        let text = summary_text(app, &app.cols[j]).unwrap_or_default();
        let align = match app.kinds[j] {
            Kind::Number => Align::Right,
            _ => Align::Left,
        };
        spans.push(Span::raw(fit(&text, cw, align)));
    }
    pad(spans, lay.width, Style::default())
}

/// 列の幅の決め方の結果。
pub(crate) struct Layout {
    /// 描ける幅(右端の1桁を除いた幅)。
    pub width: usize,
    /// 行番号の欄の幅(数の桁と後ろの空白1つ。出さなければ 0。SR-20)。
    pub num_w: usize,
    pub label_w: usize,
    pub widths: Vec<usize>,
    /// 左に固定した列の数(NV-4)。
    pub frozen: usize,
}

impl Layout {
    /// ノートの列が始まる位置(選択の印1桁 + 行番号の欄)。
    pub(crate) fn label_x(&self) -> usize {
        1 + self.num_w
    }

    /// 値の列が始まる位置(選択の印1桁 + 行番号の欄 + ノートの列 + 区切り1つ)。
    pub(crate) fn data_x(&self) -> usize {
        self.label_x() + self.label_w + 1
    }

    /// 固定した列と、`left` から `col` までの列が全部見えるか(SR-3・NV-4)。
    pub fn fits(&self, left: usize, col: usize) -> bool {
        let frozen = self.frozen.min(self.widths.len());
        if col < frozen {
            return true;
        }
        let fixed: usize = self.widths[..frozen].iter().map(|w| w + 1).sum();
        let from = left.max(frozen);
        if from > col {
            return true;
        }
        let used: usize = self.widths[from..=col].iter().map(|w| w + 1).sum::<usize>() - 1;
        self.data_x() + fixed + used <= self.width
    }
}

/// 1列の幅の上限: 画面の幅の3割(CV-5)。
fn cap(area_w: usize) -> usize {
    (area_w * 3 / 10).max(3)
}

/// SR-29: 表の列に file.name があるか(あれば左のノートの欄に名前と見出しを出さない)。
pub(crate) fn name_column_shown(app: &App) -> bool {
    app.cols.iter().any(|c| c == "file.name")
}

/// ノートの列の幅(見えている行の名前と見出しから。上限は CV-5 と同じ)。
fn label_width(app: &App, visible: &[&RowId], max: usize) -> usize {
    let head = if name_column_shown(app) {
        0
    } else {
        width(LABEL_HEADER.text())
    };
    visible
        .iter()
        .map(|r| width(&label(app, r)))
        .chain([head])
        // BV-14: 集計の行の見出し(集計の無いビューでは数えない。幅は変わらない)。
        .chain((summary_rows(app) > 0).then(|| width(Msg::SummaryRow.text())))
        .max()
        .unwrap_or(1)
        .min(max)
}

/// 手で決める列の幅の上限(値の列に使える幅。SR-3・NV-4)。
pub(crate) fn max_column_width(app: &App) -> usize {
    set_ambiguous_wide(app.ambiguous_wide);
    let area_w = app.size.0.saturating_sub(1) as usize;
    let visible = visible_rows(app);
    let label_w = label_width(app, &visible, cap(area_w));
    area_w.saturating_sub(num_width(app) + label_w + 2)
}

/// 見えている行の値の幅から列の幅を決める(CV-5)。手で決めた幅(SR-3・NV-4)があればそれ。
pub(crate) fn layout(app: &App) -> Layout {
    set_ambiguous_wide(app.ambiguous_wide);
    let area_w = app.size.0.saturating_sub(1) as usize;
    let max = cap(area_w);
    let visible = visible_rows(app);
    let label_w = label_width(app, &visible, max);
    let num_w = num_width(app);
    let room = area_w.saturating_sub(num_w + label_w + 2).max(1);
    let widths = app
        .cols
        .iter()
        .map(|c| {
            if let Some(&w) = app.widths.get(c) {
                return w.clamp(1, room);
            }
            visible
                .iter()
                .map(|r| width(&shown(app, r, c).text))
                .chain([width(&sanitize(&app.head_title(c)))])
                .chain(summary_text(app, c).map(|t| width(&t)))
                .max()
                .unwrap_or(1)
                .clamp(1, max)
        })
        .collect();
    let mut lay = Layout {
        width: area_w,
        num_w,
        label_w,
        widths,
        frozen: 0,
    };
    lay.frozen = effective_frozen(&lay, app.frozen, app.col);
    lay
}

/// 固定した列のうち、選んだ列と一緒に画面に収まる数(NV-4・SR-3)。収まらなければ固定を一時的に減らして、
/// 選んだ列をいつも見せる(見えないセルを選ばない)。
fn effective_frozen(lay: &Layout, want: usize, col: usize) -> usize {
    let n = lay.widths.len();
    let mut f = want.min(n);
    while f > 0 {
        let ok = if col < f {
            let used: usize = lay.widths[..=col].iter().map(|w| w + 1).sum::<usize>() - 1;
            lay.data_x() + used <= lay.width
        } else {
            let fixed: usize = lay.widths[..f].iter().map(|w| w + 1).sum();
            lay.data_x() + fixed + lay.widths.get(col).copied().unwrap_or(0) <= lay.width
        };
        if ok {
            break;
        }
        f -= 1;
    }
    f
}

/// 固定するなら、左から `col` までの列が画面に収まるか(NV-4 の `F`)。
pub(crate) fn freeze_fits(app: &App, col: usize) -> bool {
    let lay = layout(app);
    if col >= lay.widths.len() {
        return false;
    }
    let used: usize = lay.widths[..=col].iter().map(|w| w + 1).sum::<usize>() - 1;
    lay.data_x() + used <= lay.width
}

/// 選んだセルが表の画面に描かれているか(見えないセルを直さない守り。SR-3)。
pub(crate) fn selected_drawn(app: &App) -> bool {
    let h = app.data_height();
    if app.row < app.top || app.row >= app.top + h {
        return false;
    }
    let (_, cols) = visible_layout(app);
    cols.iter().any(|&(j, _)| j == app.col)
}

/// 見えているノートの行(見出しの行は除く)。
fn visible_rows(app: &App) -> Vec<&RowId> {
    app.slots
        .iter()
        .skip(app.top)
        .take(app.data_height())
        .filter_map(|s| match s {
            Slot::Row(r) => app.rows.get(*r),
            Slot::Head(_) | Slot::Gap => None,
        })
        .collect()
}

/// 留めている行(NV-12)の印。
pub(crate) const HELD_MARK: char = '~';

/// 行の表示名。外で変わった行・同期の競合ファイルには `!`、値を直して元の位置に留めている行には `~`
/// を付ける(色に頼らない)。
fn label(app: &App, row: &RowId) -> String {
    let flag = app.changes.external(row) || app.src.mark(row).is_some();
    // SR-29: 共通のフォルダと `.md` を除く。file.name の列があれば名前は出さず、印だけ。
    let mut name = if name_column_shown(app) {
        String::new()
    } else {
        let full = app.src.label(row);
        let s = full
            .strip_prefix(app.built.label_prefix.as_str())
            .unwrap_or(&full);
        sanitize(s.strip_suffix(".md").unwrap_or(s))
    };
    if app.held.contains(row) {
        name = format!("{HELD_MARK}{name}");
    }
    if flag {
        format!("!{name}")
    } else {
        name
    }
}

fn pending_color(mode: ColorMode) -> Option<Color> {
    match mode {
        ColorMode::None => None,
        ColorMode::Indexed => Some(Color::Indexed(214)),
        ColorMode::Rgb => Some(Color::Rgb(255, 175, 0)),
    }
}

/// 1行の幅をちょうど `w` にそろえる(足りない分を空白で埋める)。
/// はみ出す分は切り詰める(SR-9)。
pub(crate) fn pad(spans: Vec<Span<'static>>, w: usize, style: Style) -> Line<'static> {
    let mut out = Vec::new();
    let mut used = 0;
    for s in spans {
        let sw = width(&s.content);
        if used + sw <= w {
            used += sw;
            out.push(s);
        } else {
            let t = take(&s.content, w - used);
            used += width(&t);
            out.push(Span::styled(t, s.style));
            break;
        }
    }
    if used < w {
        out.push(Span::styled(" ".repeat(w - used), style));
    }
    Line::from(out)
}

/// (x, y) が列の見出しの行の、列の右の境界(区切りの空白)なら、その列のドラッグ(SR-3)。
pub fn boundary_at(app: &App, x: u16, y: u16) -> Option<Drag> {
    if y as usize != data_y(app) - 1 {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let x = x as usize;
    let mut cx = lay.data_x();
    for &(j, w) in &cols {
        if x == cx + w {
            return Some(Drag { col: j, x0: cx });
        }
        cx += w + 1;
    }
    None
}

/// 見えている列(固定した列のあと左から、幅に収まる分。最後の列は収まる分だけ切って見せる)。
fn visible_cols(app: &App, lay: &Layout) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut x = lay.data_x();
    let frozen = lay.frozen.min(lay.widths.len());
    let order = (0..frozen).chain(app.left.max(frozen)..lay.widths.len());
    for j in order {
        let cw = lay.widths[j];
        if x >= lay.width {
            break;
        }
        let room = lay.width - x;
        if cw > room {
            if room >= 2 {
                out.push((j, room));
            }
            break;
        }
        out.push((j, cw));
        x += cw + 1;
    }
    out
}

fn column_header(app: &App, lay: &Layout, cols: &[(usize, usize)]) -> Line<'static> {
    let st = Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let sep = col_sep(app);
    let mut spans = vec![
        Span::raw(" ".repeat(lay.label_x())),
        Span::styled(
            fit(
                if name_column_shown(app) {
                    ""
                } else {
                    LABEL_HEADER.text()
                },
                lay.label_w,
                Align::Left,
            ),
            st,
        ),
    ];
    for &(j, cw) in cols {
        spans.push(Span::raw(sep));
        let align = match app.kinds[j] {
            Kind::Number => Align::Right,
            _ => Align::Left,
        };
        let title = sanitize(&app.head_title(&app.cols[j]));
        spans.push(Span::styled(fit(&title, cw, align), st));
    }
    pad(spans, lay.width, Style::default())
}

/// 行ごとに変わらない描画の材料(選択の範囲。NV-5)。
pub(crate) struct RowCtx {
    pub span: Option<(usize, usize)>,
}

fn highlight(mode: ColorMode) -> Option<Color> {
    match mode {
        ColorMode::None => None,
        ColorMode::Indexed => Some(Color::Indexed(24)),
        ColorMode::Rgb => Some(Color::Rgb(0, 95, 135)),
    }
}

/// 選んだ行(印か v の範囲)か(NV-5)。
fn is_selected(app: &App, ctx: &RowCtx, i: usize, row: &RowId) -> bool {
    app.marked.contains(row) || ctx.span.is_some_and(|(a, b)| a <= i && i <= b)
}

fn data_row(
    app: &App,
    lay: &Layout,
    cols: &[(usize, usize)],
    i: usize,
    row: &RowId,
    ctx: &RowCtx,
) -> Line<'static> {
    let sel_row = i == app.row;
    // 左の1桁: 選んだ行(NV-5)は `+`、今の行は `>`。
    let lead = if is_selected(app, ctx, i, row) {
        "+"
    } else if sel_row {
        ">"
    } else {
        " "
    };
    let mut label_st = if sel_row {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    // NV-8 の `*`: 同じ値の行は名前に下線(と色)。NV-1: 名前の一致にも下線。
    let same_row = app
        .same_mark
        .as_ref()
        .is_some_and(|(c, v)| app.plain(row, c) == *v);
    if same_row || app.search_hit(row, None) {
        label_st = label_st.add_modifier(Modifier::UNDERLINED);
        if let Some(c) = super::theme::meaning(app, |p| p.hl_bg).or(highlight(app.color)) {
            label_st = label_st.bg(c);
        }
    }
    let mut spans = vec![Span::raw(lead)];
    // SR-20: 行番号。
    spans.extend(num_span(app, lay, i));
    spans.push(Span::styled(
        fit(&label(app, row), lay.label_w, Align::Left),
        label_st,
    ));
    let edit = if sel_row && app.note.flow.is_none() {
        input_box(app, lay, cols)
    } else {
        None
    };
    let sep = col_sep(app);
    for &(j, cw) in cols {
        spans.push(Span::raw(sep));
        if let Some(b) = edit.as_ref().filter(|_| j == app.col) {
            // 入力ボックス(CE-1)。値より広ければ右の列に重ねて広げる。
            let st = Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
            spans.push(Span::styled(fit(&b.text, b.w, Align::Left), st));
            if b.w != cw {
                break;
            }
            continue;
        }
        let s = shown(app, row, &app.cols[j]);
        let mut st = Style::default();
        if s.null || s.locked.is_some() || s.unsupported.is_some() {
            st = st.add_modifier(Modifier::DIM);
        }
        if s.pending {
            if let Some(c) = super::theme::meaning(app, |p| p.pending).or(pending_color(app.color))
            {
                st = st.fg(c);
            }
        }
        // NV-1 の検索の一致と NV-8 の同じ値は下線(と色)。
        let same = same_row
            && app
                .same_mark
                .as_ref()
                .is_some_and(|(c, _)| *c == app.cols[j]);
        if same
            || app
                .search
                .as_ref()
                .is_some_and(|q| super::nav::matches(q, &s.text))
        {
            st = st.add_modifier(Modifier::UNDERLINED);
            if let Some(c) = super::theme::meaning(app, |p| p.hl_bg).or(highlight(app.color)) {
                st = st.bg(c);
            }
        }
        if sel_row && j == app.col {
            st = st.add_modifier(Modifier::REVERSED);
        }
        spans.push(Span::styled(fit(&s.text, cw, align_of(app.kinds[j])), st));
    }
    // SR-20・SR-15: 一行おきの色。
    zebra(app, i, pad(spans, lay.width, Style::default()))
}

/// groupBy の見出しの行(BV-5・SR-2): `▾ 列: 値(件数)`、畳んだら `▸`。
fn heading_row(app: &App, i: usize, g: usize, w: usize) -> Line<'static> {
    let Some((h, range)) = app.groups.get(g) else {
        return Line::from(" ".repeat(w));
    };
    let open = !app.folded.contains(h);
    let by = app.group_title().map(|t| format!("{}: ", sanitize(&t)));
    let text = Msg::HeadingRow.fill(&[
        &if i == app.row { ">" } else { " " },
        &if open { "▾" } else { "▸" },
        &by.unwrap_or_default(),
        &sanitize(h),
        &range.len(),
    ]);
    let mut st = Style::default().add_modifier(Modifier::BOLD);
    if i == app.row {
        st = st.add_modifier(Modifier::REVERSED);
    }
    Line::from(Span::styled(fit(&text, w, Align::Left), st))
}

/// (x, y) が列の見出し(境界の空白を除く)なら、その列の添字(NV-3 のクリック)。
pub fn header_at(app: &App, x: u16, y: u16) -> Option<usize> {
    if y as usize != data_y(app) - 1 {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let x = x as usize;
    let mut cx = lay.data_x();
    for &(j, w) in &cols {
        if x >= cx && x < cx + w {
            return Some(j);
        }
        cx += w + 1;
    }
    None
}

pub(crate) fn visible_layout(app: &App) -> (Layout, Vec<(usize, usize)>) {
    let lay = layout(app);
    let cols = visible_cols(app, &lay);
    (lay, cols)
}

/// (x, y) にある表のセル(行と列の添字)。ノートの列なら今の列(SR-6)。
pub fn hit(app: &App, x: u16, y: u16) -> Option<(usize, usize)> {
    let (x, y) = (x as usize, y as usize);
    let dy = data_y(app);
    if y < dy || y >= dy + app.data_height() {
        return None;
    }
    let i = app.top + (y - dy);
    // SR-30: 見出しの上の空きは選べない。
    if i >= app.slots.len() || app.slots[i] == Slot::Gap {
        return None;
    }
    let (lay, cols) = visible_layout(app);
    let mut cx = lay.data_x();
    for &(j, w) in &cols {
        if x >= cx && x < cx + w {
            return Some((i, j));
        }
        cx += w + 1;
    }
    (x < lay.data_x()).then_some((i, app.col))
}

/// 画面の全部の行。各行の幅はちょうど `w`。
pub fn render(app: &App, w: usize, h: usize) -> Vec<Line<'static>> {
    set_ambiguous_wide(app.ambiguous_wide);
    if app.mode == Mode::Help {
        return super::help::render_help(app, w, h);
    }
    if let (Mode::Confirm, Some(r)) = (app.mode, &app.review) {
        return render_review(app, r, w, h);
    }
    if app.detail.is_some() && matches!(app.mode, Mode::Detail | Mode::Edit | Mode::ListPick) {
        return super::detail::render_detail(app, w, h);
    }
    if app.draft.is_some() && matches!(app.mode, Mode::Settings | Mode::SettingsText) {
        return super::settings_view::render_settings(app, w, h);
    }
    let lay = layout(app);
    let lay = Layout { width: w, ..lay };
    let cols = visible_cols(app, &lay);
    let mut lines = vec![header(app, w)];
    // BV-13・SR-20: ビューのタブ。隠しても `[` `]` で切り替わる。
    if tab_rows(app) > 0 {
        lines.push(tabs(app, w));
    }
    // NV-23: 検索の欄(タブの下)。設定で出さないこともある。
    if bar_rows(app) > 0 {
        lines.push(search_bar(app, w));
    }
    // NV-16: 効いている設定の帯(検索の欄の下)。無ければ出さない。
    if chip_rows(app) > 0 {
        lines.push(chips(app, w));
    }
    lines.push(column_header(app, &lay, &cols));
    let data_h = table_height(app, h);
    let ctx = RowCtx {
        span: app.anchor_span(app.row),
    };
    for (i, slot) in app.slots.iter().enumerate().skip(app.top).take(data_h) {
        lines.push(match slot {
            Slot::Row(r) => data_row(app, &lay, &cols, i, &app.rows[*r], &ctx),
            Slot::Head(g) => heading_row(app, i, *g, w),
            // SR-30: 見出しの上の空き。
            Slot::Gap => Line::from(" ".repeat(w)),
        });
    }
    if app.slots.is_empty() {
        match &app.view_error {
            // BV-7: 開けないビューの理由は、表の欄に折り返して全部見せる。
            Some(e) => {
                let mut rest = sanitize(&Msg::ViewCannotOpen.fill(&[&e]));
                while !rest.is_empty() && lines.len() < h.saturating_sub(2) {
                    let mut head = take(&rest, w);
                    if head.is_empty() {
                        // 幅に1文字も入らない(狭すぎる)ときも1つずつ進める。
                        head = rest.graphemes(true).next().unwrap_or_default().to_string();
                    }
                    rest.drain(..head.len());
                    lines.push(Line::from(fit(&head, w, Align::Left)));
                }
            }
            None => {
                let note = if app.progress.done {
                    Msg::NoNotes.text()
                } else {
                    Msg::TableLoading.text()
                };
                lines.push(Line::from(fit(note, w, Align::Left)));
            }
        }
    }
    // BV-14: 集計の行は表の下の端(下の帯の上)。集計の無いビューでは出さない。
    let body = h.saturating_sub(2 + summary_rows(app));
    while lines.len() < body {
        lines.push(Line::from(" ".repeat(w)));
    }
    lines.truncate(body);
    if summary_rows(app) > 0 && lines.len() + 2 < h {
        lines.push(summary_row(app, &lay, &cols));
    }
    // CE-25: 新しいノートの欄は表の下の端に(カレンダーとリストの選択の窓はこの上に重なる)。
    super::new_note::overlay(app, &mut lines, w);
    // CE-3: 候補のリストは入力ボックスの下(入らなければ上)に重ねる。
    super::list::overlay(app, &mut lines, w);
    // CE-20: 日付のカレンダーも入力ボックスの下(入らなければ上)に重ねる。
    super::calendar::overlay(app, &mut lines, w);
    // CE-16: リストの選択の窓はセルの下(入らなければ上)に重ねる。
    super::listpick::overlay(app, &mut lines, w);
    // SR-24: その場の操作の一覧の窓もセルの下(入らなければ上)に重ねる。
    super::menu::overlay(app, &mut lines, w);
    // NV-9: 列の値の頻度表の窓も同じ形で重ねる(freq.rs)。
    super::freq::overlay(app, &mut lines, w);
    if app.mode == Mode::Quit {
        quit_overlay(app, &mut lines, w);
    }
    if app.mode == Mode::Palette {
        super::help::palette_overlay(app, &mut lines, w);
    }
    lines.push(footer(app, w));
    lines.push(message(app, w));
    lines.truncate(h);
    lines
}
