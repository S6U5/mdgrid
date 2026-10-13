//! 絞り込みの窓(NV-29): 検索の欄の右の「絞り込み」のボタンで開く小さな窓。今のビューの設定のフィルターの
//! 条件を並べ、足す(列 → 種類 → 値の一覧か値の入力)・外す・直す。変えるとすぐ表に当て、設定のフィルターとして
//! 見た目の状態に残す(set_filters。並べ替えの窓(NV-24)と同じ手軽さ)。
//!
//! 条件の段取りと部品(種類・比べ方の並び、値の件数、比べる値の確かめ)は設定の画面(NV-19)と同じものを使う。
//! 窓は検索の欄の行の下に、右の端にそろえて重ねる(欄が無ければヘッダーの下)。窓の外のクリックと Esc で閉じる。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::settings::{TextKind, CMPS, KINDS};
use super::settings_view::{check, text_prompt};
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::settings::{cond_parts, CmpOp, Cond, Op};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 値の入力の行の打っている位置の印。
const CARET: &str = "▏";

/// 条件の行の欄(NV-29 の表の形): 列・比べ方・値。
const CELLS: usize = 3;
/// 初めに選ぶ欄(値。Enter で値を直す)。
const VALUE_CELL: usize = 2;

/// 開いている絞り込みの窓。
#[derive(Debug, Clone)]
pub(crate) struct FilterWin {
    /// 条件の並びで選んでいる行(条件のあとに「+ 絞り込みを足す」)。
    pub sel: usize,
    /// 条件の行で選んでいる欄(0: 列・1: 比べ方・2: 値)。
    pub cell: usize,
    /// 条件を足す・直す途中の段(None なら条件の並び)。
    pub step: Option<Step>,
}

impl Default for FilterWin {
    fn default() -> Self {
        FilterWin {
            sel: 0,
            cell: VALUE_CELL,
            step: None,
        }
    }
}

/// 条件を足す・直す段。`edit` は直している条件の位置(None なら足す)。
#[derive(Debug, Clone)]
pub(crate) enum Step {
    Column {
        cols: Vec<String>,
        sel: usize,
        edit: Option<usize>,
    },
    Kind {
        col: String,
        sel: usize,
        edit: Option<usize>,
    },
    Cmp {
        col: String,
        sel: usize,
        edit: Option<usize>,
    },
    /// 値の一覧(先頭の行は残す/隠すの切り替え)。
    Values {
        col: String,
        keep: bool,
        checked: Vec<Option<String>>,
        counts: Vec<(Option<String>, usize)>,
        sel: usize,
        edit: Option<usize>,
    },
    /// 値の入力(含む・含まない・比べる)。
    Text {
        col: String,
        kind: TextKind,
        text: String,
        edit: Option<usize>,
    },
}

/// ビューの帯のボタンの文字(前後の空白なし。NV-29)。
pub(crate) fn label(app: &App) -> String {
    button(app).trim().to_string()
}

/// 検索の欄の右の「絞り込み」のボタンの文字(NV-29): 条件が無ければ「絞り込み」、あれば数を添える。
pub(crate) fn button(app: &App) -> String {
    match app.settings.filters.len() {
        0 => Msg::FilterButton.text().to_string(),
        n => Msg::FilterButtonN.fill(&[&n]),
    }
}

/// 窓の題と行の文字。
fn rows(app: &App, w: &FilterWin) -> (String, Vec<String>) {
    let t = |c: &str| sanitize(&app.title(c));
    match &w.step {
        None => {
            let mut out = table_rows(app, w);
            out.push(Msg::FilterAdd.text().to_string());
            (Msg::FilterTitle.text().to_string(), out)
        }
        Some(Step::Column { cols, .. }) => (
            Msg::FilterPickColumn.text().to_string(),
            cols.iter().map(|c| t(c)).collect(),
        ),
        Some(Step::Kind { col, .. }) => (
            format!(" {} ", Msg::PickKindOf.fill(&[&t(col)])),
            KINDS.iter().map(|k| k.to_string()).collect(),
        ),
        Some(Step::Cmp { col, .. }) => (
            format!(" {} ", Msg::PickCmpOf.fill(&[&t(col)])),
            CMPS.iter().map(|(_, m)| m.to_string()).collect(),
        ),
        Some(Step::Values {
            col,
            keep,
            checked,
            counts,
            ..
        }) => {
            let mode = if *keep {
                Msg::PickModeKeep
            } else {
                Msg::PickModeHide
            };
            let mut items = vec![mode.to_string()];
            items.extend(counts.iter().map(|(k, n)| {
                let label = k
                    .as_deref()
                    .map(sanitize)
                    .unwrap_or_else(|| Msg::EmptyHeading.into());
                Msg::PickValueRow.fill(&[&check(checked.contains(k)), &label, n])
            }));
            (format!(" {} ", Msg::PickValuesOf.fill(&[&t(col)])), items)
        }
        Some(Step::Text {
            col, kind, text, ..
        }) => (
            format!(" {} ", text_prompt(app, col, *kind)),
            vec![format!("{}{CARET}", sanitize(text))],
        ),
    }
}

/// 条件の行の欄の文字(列・比べ方・値)。
fn cells(app: &App, c: &Cond) -> [String; CELLS] {
    let (op, value) = cond_parts(c);
    [
        sanitize(&app.title(&c.col)),
        sanitize(&op),
        sanitize(&value),
    ]
}

/// 列と比べ方の欄の幅(行をそろえる)。
fn cell_widths(app: &App) -> (usize, usize) {
    app.settings
        .filters
        .iter()
        .map(|c| cells(app, c))
        .fold((0, 0), |(a, b), [c, o, _]| {
            (a.max(width(&c)), b.max(width(&o)))
        })
}

/// 条件の並びの行(NV-29 の表の形): `列  比べ方  値`。選んでいる行の選んでいる欄は `[ ]` で囲む。
fn table_rows(app: &App, w: &FilterWin) -> Vec<String> {
    let (cw, ow) = cell_widths(app);
    app.settings
        .filters
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let parts = cells(app, c);
            let mark = |k: usize, t: &str, pad: usize| {
                let on = i == w.sel && k == w.cell;
                let t = if on {
                    format!("[{t}]")
                } else {
                    format!(" {t} ")
                };
                fit(&t, pad + 2, Align::Left)
            };
            format!(
                "{}{}{}",
                mark(0, &parts[0], cw),
                mark(1, &parts[1], ow),
                mark(2, &parts[2], width(&parts[2]))
            )
        })
        .collect()
}

/// 条件の行で、窓の中の桁 `dx`(中身の左の端から)が指す欄。
fn cell_at(app: &App, dx: usize) -> usize {
    let (cw, ow) = cell_widths(app);
    // 行の頭の選びの印 `> ` の後ろから。
    let dx = dx.saturating_sub(2);
    if dx < cw + 2 {
        0
    } else if dx < cw + ow + 4 {
        1
    } else {
        2
    }
}

/// 比べ方の種類の行(KINDS の並び)。
fn kind_index(op: &Op) -> usize {
    match op {
        Op::Keep(_) | Op::Drop(_) => 0,
        Op::Contains(_) => 1,
        Op::NotContains(_) => 2,
        Op::Cmp(..) => 3,
        Op::Empty => 4,
        Op::NotEmpty => 5,
    }
}

/// 選んでいる行。
fn selected(w: &FilterWin) -> usize {
    match &w.step {
        None => w.sel,
        Some(Step::Column { sel, .. })
        | Some(Step::Kind { sel, .. })
        | Some(Step::Cmp { sel, .. })
        | Some(Step::Values { sel, .. }) => *sel,
        Some(Step::Text { .. }) => 0,
    }
}

fn selected_mut(w: &mut FilterWin) -> Option<&mut usize> {
    match &mut w.step {
        None => Some(&mut w.sel),
        Some(Step::Column { sel, .. })
        | Some(Step::Kind { sel, .. })
        | Some(Step::Cmp { sel, .. })
        | Some(Step::Values { sel, .. }) => Some(sel),
        Some(Step::Text { .. }) => None,
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    start: usize,
    vis: usize,
}

fn window(app: &App) -> Option<Geom> {
    let w = app.filter_win.as_ref()?;
    let (title, items) = rows(app, w);
    let (sw, limit) = popup::screen(app);
    let inner = items
        .iter()
        .map(|t| width(t) + 5)
        .chain(std::iter::once(width(&title) + 2))
        .max()
        .unwrap_or(10)
        .max(20);
    let iw = (inner + 2).min(sw);
    if iw < 8 {
        return None;
    }
    // NV-23: ビューの帯のボタンのすぐ下に、ボタンの左の端にそろえて置く(右へはみ出すなら左へずらす)。
    // ボタンが描かれていなければ、ヘッダーの下に右の端にそろえて置く。
    let (bx, ay) = super::bands::button_anchor(app, super::bands::ViewButton::Filter)
        .map(|(x, y)| (Some(x), y))
        .unwrap_or((None, super::bands::head_y(app) as usize));
    let popup::Place { top, vis } = popup::place(ay, items.len(), limit)?;
    let sel = selected(w).min(items.len() - 1);
    Some(Geom {
        x: bx.map_or(sw - iw, |x| x.min(sw - iw)),
        top,
        iw,
        start: (sel + 1).saturating_sub(vis),
        vis,
    })
}

impl App {
    /// NV-29: 絞り込みの窓を開く(表のモードから)。
    pub(crate) fn open_filters(&mut self) {
        if self.mode != Mode::Table {
            self.set_mode(Mode::Table);
        }
        self.filter_win = Some(FilterWin::default());
        if window(self).is_none() {
            self.filter_win = None;
            self.message = Some(Msg::FilterNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::FilterWin);
    }

    pub(crate) fn close_filters(&mut self) {
        self.filter_win = None;
        self.set_mode(Mode::Table);
    }

    /// 窓の段を変えてモードを合わせる(値の入力の段だけ文字を受ける)。
    fn put_filter_win(&mut self, w: FilterWin) {
        let text = matches!(w.step, Some(Step::Text { .. }));
        self.filter_win = Some(w);
        // 段が進んで行が増え、低い端末で窓が入らなくなったら、見えないまま動かさずに閉じる。
        if window(self).is_none() {
            self.close_filters();
            self.message = Some(Msg::FilterNoRoom.text().into());
            return;
        }
        self.set_mode(if text {
            Mode::FilterText
        } else {
            Mode::FilterWin
        });
    }

    /// 絞り込みの窓のモード(窓と値の入力)の動作。
    pub(crate) fn filters_action(&mut self, action: Action) {
        let Some(mut w) = self.filter_win.take() else {
            return self.close_filters();
        };
        let len = rows(self, &w).1.len();
        if let Some(s) = selected_mut(&mut w) {
            if let Some(n) = popup::step_sel(*s, len, action, 0) {
                *s = n;
                return self.put_filter_win(w);
            }
        }
        let nf = self.settings.filters.len();
        match (w.step.take(), action) {
            // ---- 条件の並び: ←→ で欄を選び、Enter で選んだ欄を直す(NV-29 の表の形) ----
            (None, Action::Left) => w.cell = w.cell.saturating_sub(1),
            (None, Action::Right) => w.cell = (w.cell + 1).min(CELLS - 1),
            (None, Action::Run | Action::Toggle) if w.sel < nf => {
                let c = self.settings.filters[w.sel].clone();
                let edit = Some(w.sel);
                w.step = Some(match w.cell {
                    0 => {
                        let cols = self.setting_keys();
                        let sel = cols.iter().position(|k| *k == c.col).unwrap_or(0);
                        Step::Column { cols, sel, edit }
                    }
                    1 => Step::Kind {
                        col: c.col,
                        sel: kind_index(&c.op),
                        edit,
                    },
                    _ => self.edit_step(w.sel),
                });
            }
            (None, Action::Run | Action::Toggle) => {
                let cols = self.setting_keys();
                if !cols.is_empty() {
                    w.step = Some(Step::Column {
                        cols,
                        sel: 0,
                        edit: None,
                    });
                }
            }
            (None, Action::RemoveItem) if w.sel < nf => {
                let mut f = self.settings.filters.clone();
                f.remove(w.sel);
                w.sel = w.sel.min(f.len());
                self.set_filters(f);
            }
            (None, Action::Close) => return self.close_filters(),
            (None, _) => {}
            // ---- 段の中: Esc は条件の並びに戻る ----
            (Some(Step::Text { .. }), Action::Cancel) | (Some(_), Action::Close) => {}
            (Some(Step::Column { cols, sel, edit }), Action::Run | Action::Toggle) => {
                w.step = match cols.get(sel) {
                    // 列を変えたら、種類を選び直す(値はその列のもの)。
                    Some(col) => Some(Step::Kind {
                        col: col.clone(),
                        sel: 0,
                        edit,
                    }),
                    None => Some(Step::Column { cols, sel, edit }),
                };
            }
            (Some(Step::Kind { col, sel, edit }), Action::Run | Action::Toggle) => match sel {
                0 => {
                    // 同じ列の値の条件を直すときだけ、今のチェックを引き継ぐ。
                    let (keep, checked) = match edit
                        .and_then(|i| self.settings.filters.get(i))
                        .filter(|c| c.col == col)
                        .map(|c| &c.op)
                    {
                        Some(Op::Keep(k)) => (true, k.clone()),
                        Some(Op::Drop(k)) => (false, k.clone()),
                        _ => (false, Vec::new()),
                    };
                    w.step = Some(self.values_step(col, keep, checked, edit));
                }
                1 => w.step = Some(text_step(col, TextKind::Contains, edit)),
                2 => w.step = Some(text_step(col, TextKind::NotContains, edit)),
                3 => w.step = Some(Step::Cmp { col, sel: 0, edit }),
                4 => return self.put_filter(w, col, Op::Empty, edit),
                _ => return self.put_filter(w, col, Op::NotEmpty, edit),
            },
            (Some(Step::Cmp { col, sel, edit }), Action::Run | Action::Toggle) => {
                let op = CMPS.get(sel).map(|c| c.0).unwrap_or(CmpOp::Eq);
                w.step = Some(text_step(col, TextKind::Cmp(op), edit));
            }
            (
                Some(Step::Values {
                    col,
                    mut keep,
                    mut checked,
                    counts,
                    sel,
                    edit,
                }),
                Action::Toggle,
            ) => {
                if sel == 0 {
                    keep = !keep;
                } else if let Some((k, _)) = counts.get(sel - 1) {
                    match checked.iter().position(|c| c == k) {
                        Some(at) => {
                            checked.remove(at);
                        }
                        None => checked.push(k.clone()),
                    }
                }
                w.step = Some(Step::Values {
                    col,
                    keep,
                    checked,
                    counts,
                    sel,
                    edit,
                });
            }
            (
                Some(Step::Values {
                    col,
                    keep,
                    checked,
                    counts,
                    sel,
                    edit,
                }),
                Action::Run,
            ) => {
                if checked.is_empty() {
                    let sp = self.key_of(Mode::FilterWin, Action::Toggle);
                    self.message = Some(Msg::NoValueChecked.fill(&[&sp]));
                    w.step = Some(Step::Values {
                        col,
                        keep,
                        checked,
                        counts,
                        sel,
                        edit,
                    });
                } else {
                    // チェックした値は一覧の順にそろえる。
                    let keys: Vec<Option<String>> = counts
                        .into_iter()
                        .map(|(k, _)| k)
                        .filter(|k| checked.contains(k))
                        .collect();
                    let op = if keep { Op::Keep(keys) } else { Op::Drop(keys) };
                    return self.put_filter(w, col, op, edit);
                }
            }
            (
                Some(Step::Text {
                    col,
                    kind,
                    mut text,
                    edit,
                }),
                a,
            ) => {
                match a {
                    Action::DeleteBack => {
                        text.pop();
                    }
                    Action::Commit => {
                        let v = text.trim().to_string();
                        let op = match kind {
                            _ if v.is_empty() => {
                                let esc = self.key_of(Mode::FilterText, Action::Cancel);
                                Err(Msg::ValueEmpty.fill(&[&esc]))
                            }
                            TextKind::Contains => Ok(Op::Contains(v)),
                            TextKind::NotContains => Ok(Op::NotContains(v)),
                            TextKind::Cmp(o) => self.check_operand(&col, &v).map(|_| Op::Cmp(o, v)),
                            _ => Err(String::new()),
                        };
                        match op {
                            Ok(op) => return self.put_filter(w, col, op, edit),
                            Err(e) => self.message = Some(e),
                        }
                    }
                    _ => {}
                }
                w.step = Some(Step::Text {
                    col,
                    kind,
                    text,
                    edit,
                });
            }
            (step, _) => w.step = step,
        }
        self.put_filter_win(w);
    }

    /// 足した条件を直す段: 値の一覧はチェックを付けたまま開き、値の条件は値を打ち直す。
    fn edit_step(&self, i: usize) -> Step {
        let c = self.settings.filters[i].clone();
        let edit = Some(i);
        let col = c.col;
        match c.op {
            Op::Keep(k) => self.values_step(col, true, k, edit),
            Op::Drop(k) => self.values_step(col, false, k, edit),
            Op::Contains(text) => Step::Text {
                col,
                kind: TextKind::Contains,
                text,
                edit,
            },
            Op::NotContains(text) => Step::Text {
                col,
                kind: TextKind::NotContains,
                text,
                edit,
            },
            Op::Cmp(o, text) => Step::Text {
                col,
                kind: TextKind::Cmp(o),
                text,
                edit,
            },
            Op::Empty => Step::Kind { col, sel: 4, edit },
            Op::NotEmpty => Step::Kind { col, sel: 5, edit },
        }
    }

    fn values_step(
        &self,
        col: String,
        keep: bool,
        checked: Vec<Option<String>>,
        edit: Option<usize>,
    ) -> Step {
        let counts = self.value_counts_for(&col, &checked);
        let sel = usize::from(!counts.is_empty());
        Step::Values {
            col,
            keep,
            checked,
            counts,
            sel,
            edit,
        }
    }

    /// 条件を足す(`edit` があれば置き換える)。すぐ表に効かせ、条件の並びに戻ってその条件を選ぶ。
    fn put_filter(&mut self, mut w: FilterWin, col: String, op: Op, edit: Option<usize>) {
        let mut f = self.settings.filters.clone();
        let cond = Cond { col, op };
        w.sel = match edit {
            Some(i) if i < f.len() => {
                f[i] = cond;
                i
            }
            _ => {
                f.push(cond);
                f.len() - 1
            }
        };
        w.step = None;
        self.set_filters(f);
        self.put_filter_win(w);
    }

    /// 値の入力に文字を足す。
    pub(crate) fn filter_insert(&mut self, c: char) {
        self.message = None;
        if let Some(FilterWin {
            step: Some(Step::Text { text, .. }),
            ..
        }) = self.filter_win.as_mut()
        {
            text.push(c);
        }
    }

    /// 絞り込みの窓の上のクリック: 行なら選んで Enter と同じ(値の一覧はチェック、条件の行の右の端の「×」は
    /// 外す)。窓の外なら閉じる。縁と値の入力の行では何もしない。
    pub(crate) fn filters_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_filters();
        };
        match popup::hit(x, y, g.x, g.top, g.iw, g.vis) {
            None => self.close_filters(),
            Some(None) => {}
            Some(Some(k)) => {
                let i = g.start + k;
                let Some(w) = self.filter_win.as_mut() else {
                    return;
                };
                let nf = self.settings.filters.len();
                let list = w.step.is_none();
                let values = matches!(w.step, Some(Step::Values { .. }));
                let Some(s) = selected_mut(w) else {
                    return;
                };
                *s = i;
                let on_x = (x as usize) + 3 >= g.x + g.iw && (x as usize) < g.x + g.iw - 1;
                if list && i < nf && !on_x {
                    let cell = cell_at(self, (x as usize).saturating_sub(g.x + 1));
                    if let Some(w) = self.filter_win.as_mut() {
                        w.cell = cell;
                    }
                }
                if list && i < nf && on_x {
                    self.filters_action(Action::RemoveItem);
                } else if values {
                    self.filters_action(Action::Toggle);
                } else {
                    self.filters_action(Action::Run);
                }
            }
        }
    }
}

fn text_step(col: String, kind: TextKind, edit: Option<usize>) -> Step {
    Step::Text {
        col,
        kind,
        text: String::new(),
        edit,
    }
}

/// 値の入力の段の端末のカーソルの位置(SR-17)。
pub(crate) fn text_cursor(app: &App) -> Option<(u16, u16)> {
    let w = app.filter_win.as_ref()?;
    let Some(Step::Text { text, .. }) = &w.step else {
        return None;
    };
    let g = window(app)?;
    let x = (g.x + 1 + 2 + width(&sanitize(text))).min(g.x + g.iw - 2);
    Some((x as u16, (g.top + 1) as u16))
}

/// 窓を重ねる(下の帯より上の行に)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if !matches!(app.mode, Mode::FilterWin | Mode::FilterText) {
        return;
    }
    let (Some(win), Some(g)) = (app.filter_win.as_ref(), window(app)) else {
        return;
    };
    let (title, items) = rows(app, win);
    let sel = selected(win);
    let list = win.step.is_none();
    let text = matches!(win.step, Some(Step::Text { .. }));
    let nf = app.settings.filters.len();
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let mut out: Vec<(String, Style)> = vec![(popup::top_edge(fr, &title, tw), Style::default())];
    for (i, t) in items.iter().enumerate().skip(g.start).take(g.vis) {
        let on = i == sel && !text;
        let st = if on {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let row = format!("{}{t}", if on { "> " } else { "  " });
        // 条件の行は右の端に「×」(押すと外す)。
        let line = if list && i < nf {
            format!("{} ×", fit(&row, tw.saturating_sub(2), Align::Left))
        } else {
            fit(&row, tw, Align::Left)
        };
        out.push((fr.side(&line), st));
    }
    out.push((fr.bottom(tw), Style::default()));
    popup::blit(app, lines, g.x, g.top, g.iw, w, out);
}
