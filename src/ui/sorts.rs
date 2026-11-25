//! 並べ替えの窓(NV-24): 検索の欄の右の「並べ替え」のボタンで開く小さな窓。今のビューの並べ替えの決まり
//! (列と向き)を並べ、足す(列を選ぶ)・外す・向きを変える・順を入れ替える。変えるとすぐ表に当て、
//! ビューの設定の並べ替えとして見た目の状態に残す(set_sorts。NV-3 の見出しの並べ替えと同じ保存)。
//!
//! navigation.sort-default-tiebreak・navigation.sort-window-default: 設定の並べ替えの決まりのあとに、既定の並び(`.base` の
//! ビューの `sort`、無ければ表の既定の並べ替え。base-view.table-sort)の決まりを「既定」と出どころを添えて並べる。既定は
//! 並びが同じ行の最後の決め手(表の組み立てが既定で並べ、設定の決まりはその上に安定に並べ直す)なので外せず、向きや順を
//! 変えると、見えている並びのまま設定の決まりになる。mdgrid のビューの並べ替えはビューの設定そのもの(定義に保存して
//! 直せる)なので、既定に数えず外せる。navigation.sort-window-reset: 画面で変えている間は「既定に戻す」を末尾に出す。
//!
//! 窓は検索の欄の行の下に、ボタンの右の端にそろえて重ねる(欄が無ければヘッダーの下)。窓の外のクリックと
//! Esc で閉じる。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::width::{fit, sanitize, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::settings::Dir;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 開いている並べ替えの窓。
#[derive(Debug, Clone, Default)]
pub(crate) struct SortWin {
    /// 選んでいる行(決まりの並びのあとに「+ 並べ替えを足す」、画面で変えていれば「既定に戻す」)。
    pub sel: usize,
    /// 足す列を選んでいる間の、選べる列と選んでいる行。
    pub picking: Option<(Vec<String>, usize)>,
}

/// 既定の並べ替えの出どころ(navigation.sort-window-default)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SortFrom {
    /// 印の `[[table]]` の `sort`(base-view.table-sort)。
    #[default]
    Table,
    /// `.base` のビューの `sort`。
    View,
}

impl SortFrom {
    /// 決まりに添える出どころの名前(「既定・印」・「既定・ビュー」)。
    pub(crate) fn label(self) -> &'static str {
        match self {
            SortFrom::Table => Msg::SortFromTable.text(),
            SortFrom::View => Msg::SortFromView.text(),
        }
    }
}

/// 見えている並べ替えの決まり1つ: 列・向きと、既定から来た決まりならその出どころ。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SortRule {
    pub col: String,
    pub dir: Dir,
    pub from: Option<SortFrom>,
}

/// 今のビューの並べ替えの土台(navigation.sort-default-tiebreak): 画面で変えていないときの設定の並べ替えと、既定の並び。
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SortBase {
    /// 画面で変えていないときの設定の並べ替え(mdgrid のビューは定義の並べ替え、ほかは空)。
    pub unchanged: Vec<(String, Dir)>,
    /// 既定の並び(`.base` のビューの `sort`、無ければ表の既定の並べ替え)。
    pub defaults: Vec<(String, Dir)>,
    pub from: SortFrom,
}

impl SortBase {
    /// 設定の並べ替え `sorts` が、画面で変えていないときと違うか(ボタンの効いている見た目と「既定に戻す」)。
    pub(crate) fn changed(&self, sorts: &[(String, Dir)]) -> bool {
        sorts != self.unchanged.as_slice()
    }

    /// 見えている決まり: 設定の決まり、続けて既定の決まり(設定の決まりの列を除く。最後の決め手)。
    pub(crate) fn rules(&self, sorts: &[(String, Dir)]) -> Vec<SortRule> {
        let own = sorts;
        own.iter()
            .map(|(c, d)| SortRule {
                col: c.clone(),
                dir: *d,
                from: None,
            })
            .chain(
                self.defaults
                    .iter()
                    .filter(|(c, _)| !own.iter().any(|(o, _)| o == c))
                    .map(|(c, d)| SortRule {
                        col: c.clone(),
                        dir: *d,
                        from: Some(self.from),
                    }),
            )
            .collect()
    }

    /// 見えている決まりの全部を、設定の決まりにしたもの。
    fn all(&self, sorts: &[(String, Dir)]) -> Vec<(String, Dir)> {
        self.rules(sorts)
            .into_iter()
            .map(|r| (r.col, r.dir))
            .collect()
    }

    /// `i` 番目の決まりの向きを変えた設定の並べ替え。既定の決まりなら、見えている並びのまま設定の決まりにする。
    pub(crate) fn flip(&self, sorts: &[(String, Dir)], i: usize) -> Vec<(String, Dir)> {
        let mut out = if i < sorts.len() {
            sorts.to_vec()
        } else {
            self.all(sorts)
        };
        if let Some((_, d)) = out.get_mut(i) {
            *d = if *d == Dir::Asc { Dir::Desc } else { Dir::Asc };
        }
        out
    }

    /// `i` 番目の決まりを外した設定の並べ替え。既定の決まりは外せない(None)。
    pub(crate) fn remove(sorts: &[(String, Dir)], i: usize) -> Option<Vec<(String, Dir)>> {
        let mut own = sorts.to_vec();
        (i < own.len()).then(|| {
            own.remove(i);
            own
        })
    }

    /// `i` 番目と `j` 番目の決まりを入れ替えた設定の並べ替え。既定の決まりにかかれば、見えている並びのまま設定の
    /// 決まりにしてから入れ替える。どちらかが無ければ None。
    pub(crate) fn swap(
        &self,
        sorts: &[(String, Dir)],
        i: usize,
        j: usize,
    ) -> Option<Vec<(String, Dir)>> {
        let n = self.rules(sorts).len();
        if i >= n || j >= n {
            return None;
        }
        let mut out = if i < sorts.len() && j < sorts.len() {
            sorts.to_vec()
        } else {
            self.all(sorts)
        };
        out.swap(i, j);
        Some(out)
    }

    /// 列 `col` の昇順を、設定の決まりの後ろ(既定の決まりの前)に足した設定の並べ替え。
    pub(crate) fn add(sorts: &[(String, Dir)], col: String) -> Vec<(String, Dir)> {
        let mut own = sorts.to_vec();
        own.push((col, Dir::Asc));
        own
    }
}

fn to_settings_dir(d: mdgrid::base::Dir) -> Dir {
    match d {
        mdgrid::base::Dir::Asc => Dir::Asc,
        mdgrid::base::Dir::Desc => Dir::Desc,
    }
}

/// ビューの帯のボタンの文字(前後の空白なし。NV-24): 決まりが無ければ「並べ替え」、あれば先頭の決まり(と残りの数)。
pub(crate) fn label(app: &App) -> String {
    button(app).trim().to_string()
}

/// 検索の欄の右の端のボタンの文字(NV-24): 決まりが無ければ「並べ替え」、あれば先頭の決まり(と残りの数)。
/// navigation.sort-button-default: 設定の決まりがあればその決まり、無ければ既定の決まり(既定のままか画面で変えたかは
/// `button_on` で見た目を分ける)。
pub(crate) fn button(app: &App) -> String {
    let list: Vec<(String, Dir)> = if app.settings.sorts.is_empty() {
        app.sort_base().defaults
    } else {
        app.settings.sorts.clone()
    };
    match list.first() {
        None => Msg::SortButton.text().to_string(),
        Some((c, d)) => {
            let arrow = if *d == Dir::Desc { "↓" } else { "↑" };
            let more = list.len() - 1;
            let name = sanitize(&app.title(c));
            if more == 0 {
                format!(" {arrow} {name} ")
            } else {
                format!(" {arrow} {name} +{more} ")
            }
        }
    }
}

/// navigation.sort-button-default: 並べ替えのボタンを効いている見た目にするか(画面で並べ替えを変えているか)。
pub(crate) fn button_on(app: &App) -> bool {
    app.sort_base().changed(&app.settings.sorts)
}

/// 窓の1行の文字(決まり・足す行・既定に戻す・列の候補)。
fn rows(app: &App, w: &SortWin) -> (String, Vec<String>) {
    if let Some((cols, _)) = &w.picking {
        (
            Msg::SortPickTitle.text().to_string(),
            cols.iter().map(|c| sanitize(&app.title(c))).collect(),
        )
    } else {
        let base = app.sort_base();
        let mut out: Vec<String> = base
            .rules(&app.settings.sorts)
            .iter()
            .map(|r| {
                let arrow = if r.dir == Dir::Desc { "↓" } else { "↑" };
                let dir = if r.dir == Dir::Desc {
                    Msg::SortDesc.text()
                } else {
                    Msg::SortAsc.text()
                };
                let from = r
                    .from
                    .map_or_else(String::new, |f| format!("  ({})", f.label()));
                format!("{arrow} {}  {dir}{from}", sanitize(&app.title(&r.col)))
            })
            .collect();
        out.push(Msg::SortAdd.text().to_string());
        if base.changed(&app.settings.sorts) {
            out.push(Msg::SortReset.text().to_string());
        }
        (Msg::SortTitle.text().to_string(), out)
    }
}

/// 下の縁のキー(screen.window-key-hints): 並びでは向き・足すと閉じる、列を選ぶ段では決めると戻る。キーはキーの表から。
/// 窓はボタンのすぐ下に左の端をそろえて置く(NV-23)ので、幅 80 の端末で右へはみ出さないよう、名前は縁の短い名前にし、
/// 外す(各決まりの行の右の端の「×」と下の帯にある)は並べない。
fn hints(app: &App, w: &SortWin) -> Vec<popup::KeyHint> {
    if w.picking.is_some() {
        return popup::key_hints(
            app,
            Mode::Sorts,
            &[
                (Action::Run, Some(Msg::KeyDecide)),
                (Action::Close, Some(Msg::KeyBack)),
            ],
        );
    }
    popup::key_hints(
        app,
        Mode::Sorts,
        &[
            (Action::Run, Some(Msg::HintSortsRun)),
            (Action::Close, None),
        ],
    )
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
    let w = app.sorts_win.as_ref()?;
    let (title, items) = rows(app, w);
    let (sw, limit) = popup::screen(app);
    let inner = items
        .iter()
        .map(|t| width(t) + 5)
        .chain([width(&title) + 2])
        .max()
        .unwrap_or(10);
    // NV-23: ビューの帯のボタンのすぐ下に、ボタンの左の端にそろえて置く(右へはみ出すなら左へずらす)。
    // ボタンが描かれていなければ、ヘッダーの下に右の端にそろえて置く。
    let (bx, ay) = super::bands::button_anchor(app, super::bands::ViewButton::Sort)
        .map_or((None, super::bands::head_y(app) as usize), |(x, y)| {
            (Some(x), y)
        });
    // screen.window-key-hints: 下の縁のキーが入るまで広げる。
    let iw = popup::widen_for_hints(inner + 2, popup::hints_width(&hints(app, w)), sw);
    if iw < 8 {
        return None;
    }
    let popup::Place { top, vis } = popup::place(ay, items.len(), limit)?;
    let sel = match &w.picking {
        Some((_, s)) => *s,
        None => w.sel,
    }
    .min(items.len() - 1);
    Some(Geom {
        x: bx.map_or(sw - iw, |x| x.min(sw - iw)),
        top,
        iw,
        start: (sel + 1).saturating_sub(vis),
        vis,
    })
}

impl App {
    /// navigation.sort-default-tiebreak: 今のビューの並べ替えの土台(画面で変えていないときの設定の並べ替えと既定の並び)。
    /// 既定の並びは `.base` のビューならその `sort`、無ければ(mdgrid のビューとビューの無い表も)表の既定の並べ替え。
    /// mdgrid のビューは、定義の並べ替えが画面で変えていないときの設定の並べ替え。
    pub(crate) fn sort_base(&self) -> SortBase {
        let table: Vec<(String, Dir)> = self
            .table_sort
            .rules
            .iter()
            .map(|(c, d)| (c.clone(), to_settings_dir(*d)))
            .collect();
        if let Some(nv) = self.native_view() {
            return SortBase {
                unchanged: nv.settings.sorts.clone(),
                defaults: table,
                from: SortFrom::Table,
            };
        }
        if let Some(b) = &self.base {
            let (sorts, from_table) = b.base.view_sort(b.view);
            return SortBase {
                unchanged: Vec::new(),
                defaults: sorts
                    .iter()
                    .map(|(c, d)| (c.clone(), to_settings_dir(*d)))
                    .collect(),
                from: if from_table {
                    SortFrom::Table
                } else {
                    SortFrom::View
                },
            };
        }
        SortBase {
            unchanged: Vec::new(),
            defaults: table,
            from: SortFrom::Table,
        }
    }

    /// base-view.table-sort-bad: 列が出そろったときに、印の表の既定の並べ替えの列(ノートのキー)が表に無ければ、CLI-3 の
    /// 形で1回警告する(その決まりは並べるものが無いので、並びは変わらない)。
    pub(crate) fn check_sort_columns(&mut self) {
        let file = format!(
            "{}/{}",
            mdgrid::workspace::MARKER_DIR,
            mdgrid::workspace::MARKER_FILE
        );
        let warns: Vec<String> = self
            .table_sort
            .rules
            .iter()
            .zip(&self.table_sort.at)
            .filter(|((c, _), _)| {
                !c.starts_with("file.") && !c.starts_with("formula.") && !self.has_column(c)
            })
            .map(|((c, _), at)| {
                Msg::SettingWarn.fill(&[
                    &file,
                    &format!("{at}.property"),
                    &Msg::RsnSortNoColumn.fill(&[c]),
                ])
            })
            .collect();
        for w in warns {
            self.message = Some(match self.message.take() {
                Some(m) => format!("{m} / {w}"),
                None => w,
            });
        }
    }

    /// NV-24: 並べ替えの窓を開く(表のモードから)。
    pub(crate) fn open_sorts(&mut self) {
        if self.mode != Mode::Table {
            self.set_mode(Mode::Table);
        }
        self.sorts_win = Some(SortWin::default());
        if window(self).is_none() {
            self.sorts_win = None;
            self.message = Some(Msg::SortNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::Sorts);
    }

    pub(crate) fn close_sorts(&mut self) {
        self.sorts_win = None;
        self.set_mode(Mode::Table);
    }

    /// 足せる列(見えている決まりに無い列。表示している列のあとに隠した列)。
    fn sortable_columns(&self) -> Vec<String> {
        let used: Vec<String> = self
            .sort_base()
            .rules(&self.settings.sorts)
            .into_iter()
            .map(|r| r.col)
            .collect();
        self.cols
            .iter()
            .cloned()
            .chain(self.hidden.iter().map(|h| h.0.clone()))
            .filter(|c| !used.contains(c))
            .collect()
    }

    /// 並べ替えの窓のモードの動作。
    pub(crate) fn sorts_action(&mut self, action: Action) {
        let Some(mut w) = self.sorts_win.take() else {
            return self.close_sorts();
        };
        let base = self.sort_base();
        let now = self.settings.sorts.clone();
        let n = base.rules(&now).len();
        let changed = base.changed(&now);
        // 決まりの行・足す行・(画面で変えていれば)既定に戻す。
        let items = n + 1 + usize::from(changed);
        match (&mut w.picking, action) {
            (Some((cols, s)), Action::Up | Action::Down | Action::Top | Action::Bottom) => {
                *s = popup::step_sel(*s, cols.len(), action, 0).unwrap_or(*s);
            }
            (Some((cols, s)), Action::Run) => {
                if let Some(c) = cols.get(*s).cloned() {
                    let sorts = SortBase::add(&now, c);
                    w.sel = sorts.len() - 1;
                    w.picking = None;
                    self.set_sorts(sorts);
                }
            }
            // 列を選んでいる間の Esc は、決まりの並びに戻る。
            (Some(_), Action::Close) => w.picking = None,
            (Some(_), _) => {}
            (None, Action::Up | Action::Down | Action::Top | Action::Bottom) => {
                w.sel = popup::step_sel(w.sel, items, action, 0).unwrap_or(w.sel);
            }
            (None, Action::Run) => {
                if w.sel < n {
                    self.set_sorts(base.flip(&now, w.sel));
                } else if w.sel == n {
                    let cols = self.sortable_columns();
                    if cols.is_empty() {
                        self.message = Some(Msg::SortNoColumn.text().into());
                    } else {
                        w.picking = Some((cols, 0));
                    }
                } else if changed {
                    // navigation.sort-window-reset: 画面で変えた並べ替えを外して既定の並びに戻す。
                    w.sel = 0;
                    self.set_sorts(base.unchanged.clone());
                    self.message = Some(Msg::SortResetDone.text().into());
                }
            }
            (None, Action::RemoveItem) if w.sel < n => match SortBase::remove(&now, w.sel) {
                Some(sorts) => {
                    self.set_sorts(sorts);
                    let after = self.sort_base().rules(&self.settings.sorts).len();
                    w.sel = w.sel.min(after);
                }
                // navigation.sort-window-default: 既定の決まりは並びが同じ行の最後の決め手なので外せない。
                None => self.message = Some(Msg::SortDefaultFixed.text().into()),
            },
            (None, Action::MoveItemUp) if w.sel > 0 && w.sel < n => {
                if let Some(sorts) = base.swap(&now, w.sel, w.sel - 1) {
                    w.sel -= 1;
                    self.set_sorts(sorts);
                }
            }
            (None, Action::MoveItemDown) if w.sel + 1 < n => {
                if let Some(sorts) = base.swap(&now, w.sel, w.sel + 1) {
                    w.sel += 1;
                    self.set_sorts(sorts);
                }
            }
            (None, Action::Close) => return self.close_sorts(),
            _ => {}
        }
        self.sorts_win = Some(w);
        self.set_mode(Mode::Sorts);
    }

    /// 並べ替えの窓の上のクリック: 行なら選んで Enter と同じ、窓の外なら閉じる。縁では何もしない。
    pub(crate) fn sorts_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_sorts();
        };
        // screen.window-key-hints: 下の縁のキーは、そのキーと同じ。
        if let Some(dx) = popup::on_bottom(x, y, g.x, g.iw, g.top + g.vis + 1) {
            let hit = self
                .sorts_win
                .as_ref()
                .and_then(|w| popup::hint_at(&hints(self, w), g.iw - 2, "", dx));
            if let Some(a) = hit {
                self.sorts_action(a);
            }
            return;
        }
        match popup::hit(x, y, g.x, g.top, g.iw, g.vis) {
            None => self.close_sorts(),
            Some(None) => {}
            Some(Some(k)) => {
                let i = g.start + k;
                // 決まりの行の右の端の「×」なら外す(ほかの所は Enter と同じ)。既定の決まりに「×」は無い。
                let on_x = (x as usize) + 3 >= g.x + g.iw && (x as usize) < g.x + g.iw - 1;
                let own = self.settings.sorts.len();
                let rule = self
                    .sorts_win
                    .as_ref()
                    .is_some_and(|w| w.picking.is_none() && i < own);
                if let Some(w) = self.sorts_win.as_mut() {
                    match &mut w.picking {
                        Some((_, s)) => *s = i,
                        None => w.sel = i,
                    }
                }
                if rule && on_x {
                    self.sorts_action(Action::RemoveItem);
                } else {
                    self.sorts_action(Action::Run);
                }
            }
        }
    }
}

/// 窓を重ねる(下の帯より上の行に)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::Sorts {
        return;
    }
    let (Some(win), Some(g)) = (app.sorts_win.as_ref(), window(app)) else {
        return;
    };
    let (title, items) = rows(app, win);
    let sel = match &win.picking {
        Some((_, s)) => *s,
        None => win.sel,
    };
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    // 「×」を付けるのは設定の決まりの行だけ(既定の決まりは外せない。navigation.sort-window-default)。
    let own = app.settings.sorts.len();
    let mut out: Vec<(String, Style)> = vec![(popup::top_edge(fr, &title, tw), Style::default())];
    for (i, t) in items.iter().enumerate().skip(g.start).take(g.vis) {
        let st = if i == sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let text = format!("{}{t}", if i == sel { "> " } else { "  " });
        // 決まりの行は右の端に「×」(押すと外す。NV-24)。
        let rule = win.picking.is_none() && i < own;
        let line = if rule {
            format!("{} ×", fit(&text, tw.saturating_sub(2), Align::Left))
        } else {
            fit(&text, tw, Align::Left)
        };
        out.push((fr.side(&line), st));
    }
    out.push((
        popup::bottom_hints(fr, tw, "", &hints(app, win)),
        Style::default(),
    ));
    popup::blit(app, lines, g.x, g.top, g.iw, w, out);
}
