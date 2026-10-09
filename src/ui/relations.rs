//! リレーションの画面の側(`impl App` の続き。specs/relations/spec.md): リンクの値を名前で見せる(REL-2・REL-10)、
//! リンクの列の候補(REL-3)。行き先の解き方は核の mdgrid::relations。
//!
//! 探す表は、今の表(読み込んだノート)と登録した表(CLI-18)。行き先の解決の結果は覚えておき、読み込んだ
//! ノートの数か登録が変わったら作り直す。

use super::app::App;
use super::list::{Item, List, NONE_LABEL};
use super::listpick::Cand;
use super::native_io::Ask;
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::relations::{self, Form, Link, Notes, Table};
use mdgrid::source::{NewValue, RowId, Value};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 行き先の無い `[[…]]`・`[…](…)` の印(REL-10。名前の前。評価できない式の `?` は1字だけで薄い)。
pub(crate) const MISSING_MARK: char = '?';

/// リンクを解くための表と、解いた結果。描くたびに全部の行を作り直さないよう、今の表は読み込みが進んだときと
/// 読み直し・保存のとき(世代)だけ、登録した表は登録が変わったときと世代が変わったときだけ作り直す。
#[derive(Default)]
pub(crate) struct LinkCtx {
    /// 今の表を作ったときの (読み込んだ数, 世代)。
    cur_key: Option<(usize, u64)>,
    /// 登録した表を読んだときの (登録のパス, 世代)。
    reg_key: Option<(Vec<PathBuf>, u64)>,
    /// 今の表(読み込んだノートから)。
    current: Option<Notes>,
    /// 登録した表(フォルダを読んで。今の表と同じフォルダは除く)。
    registered: Vec<Notes>,
    /// ワークスペースの表(登録した表と今の表)。
    tables: Vec<Table>,
    cache: HashMap<(PathBuf, String), Option<PathBuf>>,
}

impl LinkCtx {
    /// 探す表(今の表が先)。
    fn notes(&self) -> Vec<&Notes> {
        self.current.iter().chain(self.registered.iter()).collect()
    }
}

/// App に持たせる入れ物(描くときにも作り直せるように RefCell)。
#[derive(Default)]
pub(crate) struct Links {
    ctx: RefCell<LinkCtx>,
    /// 世代: 外の変更・保存・登録の変更で進め、解いた結果を捨てる。
    gen: Cell<u64>,
}

impl Links {
    /// 覚えた表と解いた結果を捨てる(次に使うときに作り直す)。
    pub(crate) fn invalidate(&self) {
        self.gen.set(self.gen.get().wrapping_add(1));
    }
}

impl App {
    /// 作り直しが要るなら作る。
    fn with_links<T>(&self, f: impl FnOnce(&mut LinkCtx) -> T) -> T {
        let gen = self.links.gen.get();
        let cur_key = (self.progress.loaded, gen);
        let places = self.scope_places();
        let reg_paths: Vec<PathBuf> = places.iter().map(|p| p.path.clone()).collect();
        let mut ctx = self.links.ctx.borrow_mut();
        let mut changed = false;
        if ctx.cur_key != Some(cur_key) {
            let rows = self.src.rows();
            let root = rows.first().and_then(|row| {
                let info = self.src.file(row)?;
                let mut root = PathBuf::from(&row.0);
                for _ in Path::new(&info.path).components() {
                    root.pop();
                }
                Some(root)
            });
            ctx.current = root.as_ref().map(|r| {
                let rels = rows
                    .iter()
                    .filter_map(|x| self.src.file(x).map(|f| f.path))
                    .collect();
                Notes::from_rels(r, rels)
            });
            ctx.cur_key = Some(cur_key);
            changed = true;
        }
        let reg_key = (reg_paths, gen);
        if changed || ctx.reg_key.as_ref() != Some(&reg_key) {
            let mut tables: Vec<Table> = places
                .iter()
                .map(|p| Table::new(&p.name, &p.path))
                .collect();
            let cur_root = ctx.current.as_ref().map(|n| n.root.clone());
            if let Some(root) = &cur_root {
                if !tables.iter().any(|t| &t.dir == root) {
                    tables.push(Table::new(&self.src.name(), root));
                }
            }
            if ctx.reg_key.as_ref() != Some(&reg_key) {
                ctx.registered = tables
                    .iter()
                    .filter(|t| Some(&t.dir) != cur_root.as_ref())
                    .map(|t| Notes::scan(&t.dir))
                    .collect();
                ctx.reg_key = Some(reg_key);
            }
            ctx.tables = tables;
            ctx.cache.clear();
        }
        f(&mut ctx)
    }

    /// 値の文字のリンクの行き先(REL-1)。リンクでなければ None、リンクで解けなければ Some(None)。
    pub(crate) fn link_target(&self, row: &RowId, s: &str) -> Option<(Link, Option<PathBuf>)> {
        let link = relations::parse(s)?;
        let note = PathBuf::from(&row.0);
        let target = self.with_links(|ctx| {
            let k = (note.clone(), s.to_string());
            if let Some(v) = ctx.cache.get(&k) {
                return v.clone();
            }
            let v = relations::resolve(&link, &note, &ctx.notes());
            ctx.cache.insert(k, v.clone());
            v
        });
        // ただのパスは、在るノートを指すときだけリンク(REL-1)。
        if target.is_none() && matches!(link.form, Form::Path { .. }) {
            return None;
        }
        Some((link, target))
    }

    /// 文字1つの見せ方(REL-2・REL-10)。リンクでなければ None。
    fn link_word(&self, row: &RowId, s: &str) -> Option<String> {
        let (link, target) = self.link_target(row, s)?;
        let name = relations::display(&link, target.as_deref());
        Some(match target {
            Some(_) => name,
            None => format!("{MISSING_MARK}{name}"),
        })
    }

    /// セルの値のリンクを名前にした文字(REL-2)。リンクを1つも含まなければ None(今までの見せ方)。
    pub(crate) fn link_text(&self, row: &RowId, v: &Value) -> Option<String> {
        match v {
            Value::Str(s) => self.link_word(row, s),
            Value::List(items) => {
                let mut any = false;
                let parts: Vec<String> = items
                    .iter()
                    .map(|x| match x {
                        Value::Str(s) => match self.link_word(row, s) {
                            Some(t) => {
                                any = true;
                                t
                            }
                            None => s.clone(),
                        },
                        other => super::cell::value_plain(other),
                    })
                    .collect();
                any.then(|| format!("[{}]", parts.join(", ")))
            }
            _ => None,
        }
    }

    /// ためた値のリンクを名前にした文字(REL-2)。
    pub(crate) fn link_text_new(&self, row: &RowId, v: &NewValue) -> Option<String> {
        match v {
            NewValue::Str(s) => self.link_text(row, &Value::Str(s.clone())),
            NewValue::List(items) => self.link_text(
                row,
                &Value::List(items.iter().map(|s| Value::Str(s.clone())).collect()),
            ),
            _ => None,
        }
    }

    /// 列がリンクの列なら、書く形と行き先のフォルダ(REL-3)。
    pub(crate) fn link_column(&self, col: &str) -> Option<(Form, Option<PathBuf>)> {
        let values: Vec<(PathBuf, Value)> = self
            .src
            .rows()
            .into_iter()
            .filter_map(|r| {
                let v = self.src.get(&r, col).value?;
                Some((PathBuf::from(&r.0), v))
            })
            .collect();
        self.with_links(|ctx| relations::link_column(&values, &ctx.notes()))
    }

    /// 書く値(REL-3)。`row` のノートから `target` を指す、`form` の形の値。名前が探す表の全部で1つでなければ、
    /// 取り違えないようにパスで書く。
    fn link_value(&self, row: &RowId, form: Form, target: &Path) -> String {
        let note = PathBuf::from(&row.0);
        let unique = self.with_links(|ctx| {
            relations::count_name(&relations::file_stem(target), &ctx.notes()) <= 1
        });
        relations::format(form, target, &note, unique)
    }

    /// リンクの列の1つの値のセルの候補(REL-3): 行き先のフォルダのノート。見せるのは名前、書くのはリンク。
    /// リンクの列でなければ None(今までの候補)。
    pub(crate) fn link_list(&self, row: &RowId, col: &str, cur: Option<&NewValue>) -> Option<List> {
        let (form, dir) = self.link_column(col)?;
        let dir = dir?;
        let cur_target = match cur {
            Some(NewValue::Str(s)) => self.link_target(row, s).and_then(|(_, t)| t),
            _ => None,
        };
        let mut items: Vec<Item> = relations::notes_in(&dir)
            .into_iter()
            .map(|t| Item {
                text: relations::file_stem(&t),
                current: cur_target.as_deref() == Some(t.as_path()),
                value: NewValue::Str(self.link_value(row, form, &t)),
            })
            .collect();
        items.push(Item {
            value: NewValue::Null,
            text: NONE_LABEL.into(),
            current: matches!(cur, None | Some(NewValue::Null)),
        });
        let sel = items.iter().position(|i| i.current).unwrap_or(0);
        Some(List {
            items,
            sel,
            initial_sel: sel,
            moved: false,
            free: false,
            filter: None,
            narrow: true,
        })
    }

    /// リンクのリストの列の候補(REL-3。多対多): 行き先のフォルダのノート。今の要素が同じノートを指すなら、
    /// その要素の文字を使う(付け外しの見分けを今の値と合わせる)。リンクの列でなければ None。
    pub(crate) fn link_cands(
        &self,
        col: &str,
        targets: &[(RowId, Vec<String>)],
    ) -> Option<Vec<(String, String)>> {
        let (form, dir) = self.link_column(col)?;
        let dir = dir?;
        let row = targets.first().map(|(r, _)| r.clone())?;
        let mut existing: Vec<(PathBuf, String)> = Vec::new();
        for (r, items) in targets {
            for e in items {
                if let Some((_, Some(t))) = self.link_target(r, e) {
                    if !existing.iter().any(|(p, _)| p == &t) {
                        existing.push((t, e.clone()));
                    }
                }
            }
        }
        Some(
            relations::notes_in(&dir)
                .into_iter()
                .map(|t| {
                    let name = relations::file_stem(&t);
                    let value = existing
                        .iter()
                        .find(|(p, _)| p == &t)
                        .map(|(_, e)| e.clone())
                        .unwrap_or_else(|| self.link_value(&row, form, &t));
                    (value, name)
                })
                .collect(),
        )
    }
}

/// リンクの候補を、リストの選択の候補にする(見せる名前を label に)。
pub(crate) fn label_cands(cands: &mut [Cand], labels: &[(String, String)]) {
    for c in cands.iter_mut() {
        if let Some((_, l)) = labels.iter().find(|(v, _)| v == &c.name) {
            c.label = Some(l.clone());
        }
    }
}

impl App {
    /// 今のセルのリンクの行き先(解けたもの。重なりは1つ)。
    fn cell_targets(&self) -> Vec<PathBuf> {
        let Some((row, col)) = self.selected() else {
            return Vec::new();
        };
        let v = match self.changes.pending(&row, &col) {
            Some(NewValue::Str(s)) => Value::Str(s.clone()),
            Some(NewValue::List(items)) => {
                Value::List(items.iter().map(|s| Value::Str(s.clone())).collect())
            }
            Some(_) => return Vec::new(),
            None => match self.src.get(&row, &col).value {
                Some(v) => v,
                None => return Vec::new(),
            },
        };
        let mut out: Vec<PathBuf> = Vec::new();
        for s in relations::strings(&v) {
            if let Some((_, Some(t))) = self.link_target(&row, s) {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
        out
    }

    /// 今のセルにリンクがあるか(操作の一覧に「行き先を開く」を出すか)。
    pub(crate) fn cell_has_link(&self) -> bool {
        let Some((row, col)) = self.selected() else {
            return false;
        };
        self.src.get(&row, &col).value.is_some_and(|v| {
            relations::strings(&v)
                .iter()
                .any(|s| self.link_target(&row, s).is_some())
        })
    }

    /// 操作の一覧に「つながった行」を出すか(REL-5): 登録した表があるか(ほかの表からの被リンクは安く
    /// 確かめられないので出す)、読み込んだノートの索引にこの行への被リンクがある。
    pub(crate) fn row_has_backlinks(&self) -> bool {
        let Some(row) = self.cur_row() else {
            return false;
        };
        if !self.scope_places().is_empty() {
            return true;
        }
        let (Some(ix), Some(info)) = (self.src.link_index(), self.src.file(&row)) else {
            return false;
        };
        !ix.backlinks(&info.path).is_empty()
    }

    /// REL-4: 今のセルのリンクの行き先を開く。行き先が2つ以上なら選ぶ一覧を出す。
    pub(crate) fn start_open_link(&mut self) {
        let targets = self.cell_targets();
        match targets.len() {
            0 => self.message = Some(Msg::RelNoTarget.into()),
            1 => {
                let t = targets[0].clone();
                self.open_note(&t);
            }
            _ => {
                let items = targets.iter().map(|t| relations::file_stem(t)).collect();
                self.open_ask(Ask::OpenLink { items, targets });
            }
        }
    }

    /// REL-5: 今の行を指しているノートの一覧を出す(どの表のどの列から)。選ぶとそのノートを開く。
    pub(crate) fn start_linked_rows(&mut self) {
        let Some(row) = self.cur_row() else {
            return;
        };
        let target = PathBuf::from(&row.0);
        let tables = self.with_links(|ctx| ctx.tables.clone());
        let found = relations::backlinks(&target, &tables);
        if found.is_empty() {
            self.message = Some(Msg::RelNoBacklinks.fill(&[&relations::file_stem(&target)]));
            return;
        }
        let items = found
            .iter()
            .map(|b| {
                let table = b.table.clone().unwrap_or_else(|| {
                    b.note
                        .parent()
                        .and_then(|d| d.file_name())
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
                format!("{table} › {} ← {}", b.column, relations::file_stem(&b.note))
            })
            .collect();
        let notes = found.into_iter().map(|b| b.note).collect();
        self.open_ask(Ask::LinkedRows { items, notes });
    }

    /// ノートを開く(REL-4・REL-5): 今の表にあればその行へ移る。無ければ、ノートを含む登録した表(無ければノートの
    /// フォルダ)に、終わる手順(WB-11 の確かめ)を通してから移り、その行を選ぶ。
    pub(crate) fn open_note(&mut self, note: &Path) {
        if !note.is_file() {
            self.message = Some(Msg::RelMissing.fill(&[&note.display()]));
            return;
        }
        self.close_palette();
        if self.select_note(note) {
            return;
        }
        // 今の表のノートなのに選べない(絞り込みで隠れている)なら、開き直さずに理由を出す。
        let id = note.canonicalize().unwrap_or_else(|_| note.to_path_buf());
        if self.src.rows().iter().any(|r| Path::new(&r.0) == id) {
            self.message = Some(Msg::RelHidden.fill(&[&relations::file_stem(note)]));
            return;
        }
        let tables = self.with_links(|ctx| ctx.tables.clone());
        let place = relations::table_of(note, &tables)
            .and_then(|t| {
                self.scope_places()
                    .into_iter()
                    .find(|p| Table::new(&p.name, &p.path).dir == t.dir)
            })
            .unwrap_or_else(|| {
                let dir = note.parent().unwrap_or(Path::new(".")).to_path_buf();
                Place {
                    name: relations::file_stem(&dir),
                    group: String::new(),
                    path: dir,
                    view: None,
                }
            });
        self.switch_to = Some(place);
        self.switch_select = Some(note.to_path_buf());
        self.begin_quit();
    }
}
