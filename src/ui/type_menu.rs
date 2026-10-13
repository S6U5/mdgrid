//! 列の型の窓(CE-35): 選んだ列の型を選び直す小さな窓。「自動(推定の型)」と各型を並べ、今の型に印を付ける。
//! 選んだ型は開いている表の範囲(views.toml の `[[table]]` の `types`)に書き、すぐ表に効かせる。ノートは書かない。
//!
//! 開き方: 列の見出しの型の印のクリック・パレットと操作の一覧の `column_type`。窓は列の見出しのすぐ下に、
//! 列の左の端にそろえて重ねる(画面の型のメニュー screen_menu.rs と同じ作り)。窓の外のクリックと Esc で閉じる。

use super::app::App;
use super::keymap::{Action, Mode};
use super::popup;
use super::width::{fit, width, Align};
use mdgrid::i18n::Msg;
use mdgrid::profile::{Layer, Origin, Place};
use mdgrid::types::Kind;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

/// 開いている型の窓。
#[derive(Debug, Clone)]
pub(crate) struct TypeMenu {
    /// 型を選び直す列。
    pub col: String,
    /// 選んでいる行(0 は「自動」、ほかは `Kind::ALL` の添字 + 1)。
    pub sel: usize,
}

/// 窓の項目の数(自動と6つの型)。
const ITEMS: usize = 1 + Kind::ALL.len();

/// 型の名前の文言。
pub(crate) fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Text => Msg::TypeText.text(),
        Kind::Number => Msg::TypeNumber.text(),
        Kind::Checkbox => Msg::TypeCheckbox.text(),
        Kind::Date => Msg::TypeDate.text(),
        Kind::DateTime => Msg::TypeDateTime.text(),
        Kind::List => Msg::TypeList.text(),
    }
}

/// 項目 `i` の名前(0 は「自動(推定の型)」)。
fn label(app: &App, col: &str, i: usize) -> String {
    match i {
        0 => Msg::TypeAuto.fill(&[&kind_label(app.guessed_kind(col))]),
        _ => kind_label(Kind::ALL[i - 1]).to_string(),
    }
}

/// 今の型の項目(設定で決めていなければ「自動」)。
fn current(app: &App, col: &str) -> usize {
    match app.prof.resolved.types.get(col) {
        Some(k) => 1 + Kind::ALL.iter().position(|x| x == k).unwrap_or(0),
        None => 0,
    }
}

/// 窓の置き場所と大きさ。
struct Geom {
    x: usize,
    top: usize,
    iw: usize,
    vis: usize,
}

fn window(app: &App) -> Option<Geom> {
    let m = app.type_menu.as_ref()?;
    let (sw, limit) = popup::screen(app);
    let inner = (0..ITEMS)
        .map(|i| 4 + width(&label(app, &m.col, i)))
        .chain(std::iter::once(width(&Msg::TypeMenuTitle.fill(&[&m.col]))))
        .max()
        .unwrap_or(10)
        + 1;
    let iw = (inner + 2).min(sw);
    if iw < 8 {
        return None;
    }
    // 列の見出しのすぐ下に、列の左の端にそろえる(右へはみ出すなら左へずらす)。
    let x = super::view::column_x(app, &m.col).unwrap_or(0);
    let ay = super::view::data_y(app) - 1;
    let popup::Place { top, vis } = popup::place(ay, ITEMS, limit)?;
    Some(Geom {
        x: x.min(sw - iw),
        top,
        iw,
        vis,
    })
}

impl App {
    /// CE-35: 今の列の型の窓を開く。`file.*`・`formula.*` の列では理由を出して開かない。
    pub(crate) fn open_type_menu(&mut self) {
        let Some(col) = self.cols.get(self.col).cloned() else {
            return;
        };
        if col.starts_with("file.") || col.starts_with("formula.") {
            self.message = Some(Msg::TypeComputed.fill(&[&col]));
            return;
        }
        let sel = current(self, &col);
        self.type_menu = Some(TypeMenu { col, sel });
        if window(self).is_none() {
            self.type_menu = None;
            self.message = Some(Msg::ScreenMenuNoRoom.text().into());
            return;
        }
        self.set_mode(Mode::ColumnType);
    }

    fn close_type_menu(&mut self) {
        self.type_menu = None;
        self.set_mode(Mode::Table);
    }

    /// 型の窓のモードの動作。
    pub(crate) fn type_menu_action(&mut self, action: Action) {
        let Some(mut m) = self.type_menu.take() else {
            return self.close_type_menu();
        };
        match action {
            Action::Up | Action::Down | Action::Top | Action::Bottom => {
                m.sel = popup::step_sel(m.sel, ITEMS, action, 0).unwrap_or(m.sel);
            }
            Action::Run => {
                let k = m.sel.checked_sub(1).map(|i| Kind::ALL[i]);
                self.close_type_menu();
                return self.set_column_type(&m.col, k);
            }
            Action::Close => return self.close_type_menu(),
            _ => {}
        }
        self.type_menu = Some(m);
        self.set_mode(Mode::ColumnType);
    }

    /// 窓の上のクリック: 行なら選んで Enter と同じ、窓の外なら閉じる。縁では何もしない。
    pub(crate) fn type_menu_click(&mut self, x: u16, y: u16) {
        self.message = None;
        let Some(g) = window(self) else {
            return self.close_type_menu();
        };
        match popup::hit(x, y, g.x, g.top, g.iw, g.vis) {
            None => self.close_type_menu(),
            Some(None) => {}
            Some(Some(k)) => {
                if let Some(m) = self.type_menu.as_mut() {
                    m.sel = k;
                }
                self.type_menu_action(Action::Run);
            }
        }
    }

    /// CE-35: 列 `col` の型を `k`(None は自動)にする。開いている表の範囲の層に書き、views.toml に保存して、
    /// 層を重ね直して表に効かせる。ノートは書かない。
    pub(crate) fn set_column_type(&mut self, col: &str, k: Option<Kind>) {
        let at = self
            .prof
            .scope
            .iter()
            .position(|l| l.origin.place == Place::TableApp);
        let at = at.unwrap_or_else(|| {
            let first = self
                .nv
                .target
                .to_string_lossy()
                .lines()
                .next()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| self.nv.target.clone());
            let name = mdgrid::workspace::stem_of(&first);
            self.prof.scope.push(Layer {
                origin: Origin::new(
                    Place::TableApp,
                    format!("{} ({name})", mdgrid::views::FILE_NAME),
                ),
                profile: Default::default(),
            });
            self.prof.scope.len() - 1
        });
        // 自動は、表より広い範囲(全体・ワークスペース・手で書いた表)に型があればそれを外す `"auto"`、無ければ書かない。
        let mut wider: Option<Kind> = None;
        let below = 1 + usize::from(!self.prof.ui.profile.is_empty()) + at;
        for l in self.prof.layers().iter().take(below) {
            if let Some(k) = l.profile.types.get(col) {
                wider = *k;
            }
        }
        let wider = wider.is_some();
        let p = &mut self.prof.scope[at].profile;
        match k {
            Some(k) => {
                p.types.insert(col.to_string(), Some(k));
            }
            None if wider => {
                p.types.insert(col.to_string(), None);
            }
            None => {
                p.types.remove(col);
            }
        }
        let p = p.clone();
        let saved = match self.nv.dir.clone() {
            Some(dir) => match mdgrid::views::save_table_profile(&dir, &self.nv.target, &p) {
                Ok(()) => true,
                Err(e) => {
                    self.message =
                        Some(Msg::CannotWriteFile.fill(&[&mdgrid::views::FILE_NAME, &e]));
                    false
                }
            },
            None => false,
        };
        let _ = self.apply_profile();
        self.refresh();
        if self.message.is_none() {
            let now = kind_label(self.col_type(col));
            // 置き場が無ければ書かない(この起動の間だけ効く)と知らせる。
            let msg = if saved {
                Msg::TypeSet
            } else {
                Msg::TypeSetNoSave
            };
            self.message = Some(msg.fill(&[&col, &now]));
        }
    }
}

/// 窓を重ねる(下の帯より上の行に)。
pub(crate) fn overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    if app.mode != Mode::ColumnType {
        return;
    }
    let (Some(m), Some(g)) = (app.type_menu.as_ref(), window(app)) else {
        return;
    };
    let tw = g.iw - 2;
    let fr = popup::frame(app);
    let now = current(app, &m.col);
    let mut out: Vec<(String, Style)> = vec![(
        popup::top_edge(fr, &Msg::TypeMenuTitle.fill(&[&m.col]), tw),
        Style::default(),
    )];
    for i in (0..ITEMS).take(g.vis) {
        let st = if i == m.sel {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let sel = if i == m.sel { "> " } else { "  " };
        let mark = if i == now { "● " } else { "  " };
        let line = fit(
            &format!("{sel}{mark}{}", label(app, &m.col, i)),
            tw,
            Align::Left,
        );
        out.push((fr.side(&line), st));
    }
    out.push((fr.bottom(tw), Style::default()));
    popup::blit(app, lines, g.x, g.top, g.iw, w, out);
}
