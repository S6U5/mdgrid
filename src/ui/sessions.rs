//! 名前を付けたセッションの画面の側(`impl App` の続き。cli.session-*)。ファイルの読み書きは核の
//! `mdgrid::session`、`-s <名前>` の解決は main。
//!
//! - 当てる(`apply_session`): main が開いてビューを選んだあとに、列の並びと隠す列を見た目の状態より先に
//!   (`saved` に入れて、列が表に現れたときに `add_column` が引く)、並びをビューの設定の並べ替えに、画面の
//!   絞り込み(検索・同じ値・式・数字のキーの条件・止めたいつも隠す条件)をそれぞれの置き場に当てる。値は
//!   持たないので、行と値はいつものとおりノートから読む(cli.session-content-fresh-values)。
//! - 保存する(`start_save_session`): パレットのコマンドで名前を聞き、同じ名前があれば置き換えを確かめる。
//!   書くのは設定の置き場の sessions.toml だけで、ノートは書かないので読むだけの起動でもできる。

use super::app::App;
use super::native_views::Ask;
use super::nav::Same;
use mdgrid::i18n::Msg;
use mdgrid::session::{self, Filter, Session};
use std::path::{Path, PathBuf};

/// セッションの画面の状態。
#[derive(Default)]
pub(crate) struct SessionState {
    /// 開いた(か保存した)セッションの名前(保存の名前の欄の初めの文字)。
    pub name: Option<String>,
    /// 読み終えたら、今の表にあるかを確かめるセッションの列(cli.session-content-columns-first)。
    pub check: Option<Vec<String>>,
}

/// 絶対のパス(実体があれば実体。印に中身を書いた `.base` のように無いものは今のフォルダからつなぐ)。
fn absolute(p: &Path) -> PathBuf {
    std::fs::canonicalize(p)
        .unwrap_or_else(|_| std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf()))
}

impl App {
    /// cli.session-save: セッションの表示を当てる(main が表を開いてビューを選んだあと)。`warnings` は
    /// sessions.toml の警告で、当てられなかった絞り込みの理由と合わせてメッセージの行に出す。
    pub fn apply_session(&mut self, s: &Session, mut warnings: Vec<String>) {
        self.session.name = Some(s.name.clone());
        // cli.session-content-columns-first: 列の並びと隠す列は、見た目の状態より先に当てる。今の表に無い列は、
        // 読み終えてから警告する(読み込みの途中では、まだ現れていないだけかもしれない)。
        if !s.columns.is_empty() || !s.hidden.is_empty() {
            let mut order = s.columns.clone();
            order.extend(s.hidden.iter().cloned());
            self.saved.order = order;
            self.saved.hidden.clone_from(&s.hidden);
            self.cols.clear();
            self.hidden.clear();
            self.frozen = 0;
            self.col = 0;
            self.left = 0;
            self.session.check = Some(s.columns.clone());
        }
        // 並びは、見出しからの並べ替えと同じくビューの設定の並べ替えにする(navigation.header-sort)。
        self.settings.sorts.clone_from(&s.sort);
        for f in &s.filters {
            match f {
                Filter::Text(t) => self.filter = Some(t.clone()).filter(|t| !t.is_empty()),
                Filter::Value {
                    key,
                    value,
                    by_keys,
                    list,
                } => self.same.push(Same {
                    col: key.clone(),
                    value: value.clone(),
                    by_keys: *by_keys,
                    list: *list,
                }),
                Filter::Expr(src) => match self.filter_expr(src.trim()) {
                    Ok(e) => self.cond.expr = Some((src.trim().to_string(), e)),
                    Err(reason) => warnings.push(Msg::SessionBadExpr.fill(&[&reason])),
                },
                Filter::Cond(n) => match self.cond.all.get(n) {
                    Some(c) if !c.hide => {
                        if !self.cond.on.contains(n) {
                            self.cond.on.push(n.clone());
                        }
                    }
                    _ => warnings.push(Msg::SessionNoCond.fill(&[n])),
                },
                Filter::Show(n) => match self.cond.all.get(n) {
                    Some(c) if c.hide => {
                        self.cond.shown.insert(n.clone());
                    }
                    _ => warnings.push(Msg::SessionNoCond.fill(&[n])),
                },
            }
        }
        self.stay.clear();
        self.refresh();
        self.row = 0;
        self.top = 0;
        self.add_warnings(warnings);
    }

    /// メッセージの行に警告を足す(前の文があれば後ろに並べる)。
    fn add_warnings(&mut self, warnings: Vec<String>) {
        if warnings.is_empty() {
            return;
        }
        let mut all: Vec<String> = self.message.take().into_iter().collect();
        all.extend(warnings);
        self.message = Some(all.join(" / "));
    }

    /// cli.session-content-columns-first: 読み終えたら、セッションにあって今の表に無い列を警告する(その列だけ飛ばす)。
    pub(crate) fn check_session_columns(&mut self) {
        let Some(want) = self.session.check.take() else {
            return;
        };
        let missing: Vec<String> = want
            .into_iter()
            .filter(|c| !self.cols.contains(c) && !self.hidden.iter().any(|(h, _, _)| h == c))
            .collect();
        if !missing.is_empty() {
            let list = missing.join(Msg::ListSep.text());
            self.add_warnings(vec![Msg::SessionMissingColumns.fill(&[&list])]);
        }
    }

    /// 今の画面の絞り込み(cli.session-content-fields): 検索・同じ値・式・数字のキーの条件・止めたいつも隠す条件。
    fn session_filters(&self) -> Vec<Filter> {
        let mut out = Vec::new();
        if let Some(t) = self.filter.as_ref().filter(|t| !t.is_empty()) {
            out.push(Filter::Text(t.clone()));
        }
        for s in &self.same {
            out.push(Filter::Value {
                key: s.col.clone(),
                value: s.value.clone(),
                by_keys: s.by_keys,
                list: s.list,
            });
        }
        if let Some((src, _)) = &self.cond.expr {
            out.push(Filter::Expr(src.clone()));
        }
        out.extend(self.cond.on.iter().cloned().map(Filter::Cond));
        let mut shown: Vec<&String> = self.cond.shown.iter().collect();
        shown.sort();
        out.extend(shown.into_iter().cloned().map(Filter::Show));
        out
    }

    /// 今の表示のセッション(名前 `name`)。開いている表は、`.base` ならそのパスとフォルダ、ほかは開いたフォルダの並び。
    pub(crate) fn current_session(&self, name: String) -> Session {
        let target = &self.nv.target;
        let (folders, base) = if self.base.is_some() {
            let b = absolute(target);
            let dir: Vec<PathBuf> = b.parent().map(Path::to_path_buf).into_iter().collect();
            (dir, Some(b))
        } else {
            // 見た目の状態の対象と同じ形(フォルダが2つ以上なら実体のパスを改行でつないだもの)。
            let t = target.to_string_lossy();
            (
                t.split('\n').map(|p| absolute(Path::new(p))).collect(),
                None,
            )
        };
        Session {
            name,
            folders,
            base,
            view: self.resume_view(),
            columns: self.cols.clone(),
            hidden: self.hidden.iter().map(|(c, _, _)| c.clone()).collect(),
            sort: self.settings.sorts.clone(),
            filters: self.session_filters(),
            readonly: self.readonly,
        }
    }

    /// cli.session-screen-save: パレットの「セッションを保存」。名前の欄を出す(開いたセッションの名前を入れて始める)。
    pub(crate) fn start_save_session(&mut self) {
        if self.nv.dir.is_none() {
            self.message = Some(Msg::SessionNoConfigDir.into());
            return;
        }
        self.open_ask(Ask::SessionName);
        if let (Some(p), Some(n)) = (&mut self.palette, &self.session.name) {
            p.query.clone_from(n);
        }
    }

    /// 名前の欄で決めた: 同じ名前があれば置き換えを確かめ(cli.session-screen-overwrite)、無ければ書く。
    pub(crate) fn session_named(&mut self, query: &str) {
        let name = match session::check_name(query) {
            Ok(n) => n,
            Err(e) => {
                self.message = Some(e);
                return;
            }
        };
        let Some(dir) = self.nv.dir.clone() else {
            self.close_palette();
            self.message = Some(Msg::SessionNoConfigDir.into());
            return;
        };
        let s = self.current_session(name);
        match session::load(&dir) {
            Err(e) => {
                self.close_palette();
                self.message = Some(Msg::SessionSaveError.fill(&[&e]));
            }
            Ok((list, _)) if list.iter().any(|x| x.name == s.name) => {
                self.open_ask(Ask::SessionOverwrite(Box::new(s)));
            }
            Ok(_) => self.save_session(&s),
        }
    }

    /// sessions.toml に書く(cli.session-store)。
    pub(crate) fn save_session(&mut self, s: &Session) {
        self.close_palette();
        let Some(dir) = self.nv.dir.clone() else {
            self.message = Some(Msg::SessionNoConfigDir.into());
            return;
        };
        match session::save(&dir, s) {
            Ok(()) => {
                self.session.name = Some(s.name.clone());
                self.message = Some(Msg::SessionSaved.fill(&[&s.name]));
            }
            Err(e) => self.message = Some(Msg::SessionSaveError.fill(&[&e])),
        }
    }
}
