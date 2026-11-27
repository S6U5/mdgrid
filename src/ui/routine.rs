//! ルーチンの回を作る(routine.*。`impl App` の続き)の画面の側。
//!
//! 開いたときと日付が変わったときに、裏の糸でルーチンの表をファイルから読み(`rows`)、記録と比べて来ている回を
//! 決める(`mdgrid::routine::due`。書かない)。来ている回があれば下の帯に数を出し、人が作る操作(帯のクリックか
//! パレットの「ルーチンの回を作る」)をして、作るノートの名前を見て確かめたときだけ作る。作る操作は、その時に表を
//! 読み直して数え直し、その日にやめた回も並べる(`mdgrid::routine::pending`)。`--runs` も同じ表の読み方と
//! 記録を使う(`routine_once`)。読むだけの起動では確かめず、知らせず、作らない。

use super::app::App;
use super::native_io::Ask;
use super::startup::READONLY;
use mdgrid::frontmatter::Value;
use mdgrid::i18n::Msg;
use mdgrid::routine::{Loaded, Plan, Report, Routine, Run, Table};
use mdgrid::source::RowId;
use mdgrid::writeback::NewValue;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

/// ルーチンの回の状態。
#[derive(Default)]
pub(crate) struct RoutineState {
    /// 範囲のルーチンの表(開いたときに印から決める)。
    pub tables: Vec<Table>,
    /// 印の `runs` を読んだときの警告(`--runs` が標準エラーに出す)。
    pub warnings: Vec<String>,
    /// 記録の置き場(見た目の状態の置き場)。None なら確かめない(試験の既定。main が置く)。
    pub dir: Option<PathBuf>,
    /// 最後に確かめ始めた日。
    pub day: Option<i64>,
    rx: Option<Receiver<Plan>>,
    /// 来ている回(下の帯に数を出す回)。
    pub due: Vec<Run>,
    /// 作る操作の確かめに並べた回(その時に数え直したもの。やめた回も入る)。
    asked: Vec<Run>,
    /// 下の帯に出した理由(同じ理由は2回出さない)。
    shown: HashSet<String>,
}

/// フロントマターの値 → 回のノートに写す値。空の値は写さない。日付と日時の形だけの文字は日付のまま書く。
fn copied(v: &Value) -> Option<NewValue> {
    Some(match v {
        Value::Null | Value::Other => return None,
        Value::Bool(b) => NewValue::Bool(*b),
        Value::Int(n) => NewValue::Int(*n),
        Value::Float(f) => NewValue::Float(*f),
        Value::Str(s) if s.trim().is_empty() => return None,
        Value::Str(s) if mdgrid::types::date_or_datetime(s) => NewValue::Date(s.clone()),
        Value::Str(s) => NewValue::Str(s.clone()),
        Value::List(items) if items.is_empty() => return None,
        Value::List(items) => {
            NewValue::List(items.iter().map(mdgrid::print::value_plain).collect())
        }
    })
}

/// ルーチンの表をファイルから読み、ルーチン(行)を集める(画面の裏の糸と `--runs`)。表の行は、`.base` の
/// 最上位の絞り込みを通った行(通知の欄・知らせと同じ)。CSV・TSV の表は、行がノートでないので読まない。
pub(crate) fn rows(t: &Table, today: i64, now: i64) -> Result<Vec<Routine>, String> {
    let target = crate::open_target(std::slice::from_ref(&t.path), None)?;
    let mut src = target.src;
    while !src.load(crate::LOAD_BUDGET).done {}
    if !src.notes() {
        return Err(Msg::RoutineNotNotes.text().to_string());
    }
    let base = target.base.map(|(b, _, _)| b);
    let prop = |r: &RowId, k: &str| src.get(r, k).value;
    let text = |r: &RowId, k: &str| {
        src.get(r, k)
            .value
            .as_ref()
            .map(mdgrid::print::value_plain)
            .filter(|s| !s.trim().is_empty())
    };
    Ok(src
        .rows()
        .iter()
        .filter(|r| {
            base.as_ref()
                .is_none_or(|b| b.keeps(src.as_ref(), &prop, r, today, now))
        })
        // routine.runs-where: `where` に当たるルーチンだけ回を作る(止めたルーチンの決まりの値は残せる)。
        .filter(|r| mdgrid::rules::where_matches(&t.spec.where_, |k| prop(r, k)))
        .map(|r| Routine {
            name: src.file(r).map(|f| f.basename).unwrap_or_default(),
            path: src.locate(r).0,
            rule: text(r, &t.spec.rule).unwrap_or_default(),
            start: text(r, mdgrid::routine::START),
            values: t
                .spec
                .copy
                .iter()
                .filter_map(|c| Some((c.clone(), copied(src.get(r, c).value.as_ref()?)?)))
                .collect(),
        })
        .collect())
}

/// 表を全部読む。
fn load(tables: &[Table], today: i64, now: i64) -> Vec<Loaded> {
    tables
        .iter()
        .map(|t| (t.clone(), rows(t, today, now)))
        .collect()
}

impl App {
    /// routine.opt-in: 範囲の表(と開いた表)のうち、印に `runs` を書いた表をルーチンの表にする。`runs` の書き方が
    /// 違えば警告を返し、その表は外す。
    pub(crate) fn configure_routine(&mut self, target: &Path) -> Vec<String> {
        let mut places: Vec<(String, PathBuf)> = self
            .scope_places()
            .into_iter()
            .map(|p| (p.name, p.path))
            .collect();
        if !target.as_os_str().is_empty() {
            places.push((mdgrid::workspace::stem_of(target), target.to_path_buf()));
        }
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let (mut tables, mut warns) = (Vec::new(), Vec::new());
        for (name, path) in places {
            let real = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if !seen.insert(real) {
                continue;
            }
            let (got, w) = mdgrid::workspace::marker_runs(&path);
            warns.extend(w);
            if let Some((spec, root)) = got {
                tables.push(Table {
                    name,
                    path,
                    root,
                    spec,
                });
            }
        }
        self.routine.tables = tables;
        self.routine.warnings.clone_from(&warns);
        warns
    }

    /// 時計の今日で `routine_poll_at` を呼ぶ(画面を回すたびに)。
    pub(crate) fn routine_poll(&mut self) {
        let (today, now) = mdgrid::clock::today_now(|k| std::env::var(k).ok());
        self.routine_poll_at(today, now);
    }

    /// routine.screen-offer: 糸の結果を拾い、開いたとき(まだ確かめていない)と日付が変わったときに確かめ始める。
    /// 読むだけの起動・ルーチンの表が無い・記録の置き場が無いときは確かめない。
    pub(crate) fn routine_poll_at(&mut self, today: i64, now: i64) {
        if let Some(rx) = &self.routine.rx {
            match rx.try_recv() {
                Ok(plan) => {
                    self.routine.rx = None;
                    self.routine_found(plan);
                }
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => self.routine.rx = None,
            }
        }
        if self.readonly || self.routine.tables.is_empty() || self.routine.day == Some(today) {
            return;
        }
        let Some(dir) = self.routine.dir.clone() else {
            return;
        };
        self.routine.day = Some(today);
        let tables = self.routine.tables.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let loaded = load(&tables, today, now);
            let _ = tx.send(mdgrid::routine::due(&dir, &loaded, today, now));
        });
        self.routine.rx = Some(rx);
    }

    /// 確かめの結果: 来ている回を覚え、読めない表と決まりの理由を下の帯に出す(同じ理由は1回)。
    fn routine_found(&mut self, plan: Plan) {
        self.routine.due = plan.runs;
        let fresh: Vec<String> = plan
            .problems
            .into_iter()
            .filter(|p| self.routine.shown.insert(p.clone()))
            .collect();
        if !fresh.is_empty() {
            self.message = Some(fresh.join(Msg::SentenceSep.text()));
        }
    }

    /// 確かめている途中か。
    #[cfg(test)]
    pub(crate) fn routine_busy(&self) -> bool {
        self.routine.rx.is_some()
    }

    /// 下の帯に出す、来ている回の数(読むだけの起動では 0)。
    /// routine.run-words: 回の知らせと作る操作の言葉に使う名前。範囲に回を作る表が1つならその表の名前、2つ以上
    /// (か無い)なら「繰り返し」。
    pub(crate) fn runs_word(&self) -> String {
        match self.routine.tables.as_slice() {
            [t] => super::width::sanitize(&t.name),
            _ => Msg::RunsGeneric.text().to_string(),
        }
    }

    pub(crate) fn routine_count(&self) -> usize {
        if self.readonly {
            0
        } else {
            self.routine.due.len()
        }
    }

    /// 作る操作で数える今日と今。最後に確かめた日(`routine_poll_at` の日)が時計の今日と違えば、その日の 0 時
    /// (試験は日を決めて確かめる)。
    fn routine_today(&self) -> (i64, i64) {
        let (today, now) = mdgrid::clock::today_now(|k| std::env::var(k).ok());
        match self.routine.day {
            Some(d) if d != today => (d, d * 86_400),
            _ => (today, now),
        }
    }

    /// routine.screen-create: パレットの「ルーチンの回を作る」と、下の帯の知らせのクリック。その時にルーチンの表を
    /// 読み直して来ている回を数え直し(開いたあとに足したルーチンも入る)、作るノートの名前を並べて確かめる
    /// (続けて `y`)。その日にやめた回も並べる(人が自分で選んだため。下の帯の知らせは出し直さない)。
    pub(crate) fn start_routine_runs(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        let Some(dir) = self.routine.dir.clone() else {
            self.message = Some(Msg::RoutineNone.fill(&[&self.runs_word()]));
            return;
        };
        if self.routine.tables.is_empty() {
            self.message = Some(Msg::RoutineNone.fill(&[&self.runs_word()]));
            return;
        }
        let (today, now) = self.routine_today();
        let plan =
            mdgrid::routine::pending(&dir, &load(&self.routine.tables, today, now), today, now);
        let fresh: Vec<String> = plan
            .problems
            .into_iter()
            .filter(|p| self.routine.shown.insert(p.clone()))
            .collect();
        if plan.runs.is_empty() {
            let mut lines = vec![Msg::RoutineNone.fill(&[&self.runs_word()])];
            lines.extend(fresh);
            self.message = Some(lines.join(Msg::SentenceSep.text()));
            return;
        }
        let names = plan.runs.iter().map(|r| r.shown.clone()).collect();
        self.routine.asked = plan.runs;
        self.open_ask(Ask::RoutineRuns(names));
        if !fresh.is_empty() {
            self.message = Some(fresh.join(Msg::SentenceSep.text()));
        }
    }

    /// routine.screen-create・routine.fail: 確かめの答え。y なら並べた回を作って表を読み直し、作れなかった回は
    /// 理由を出して知らせに残す。ほかはやめて、その日のうちは並べた回を下の帯で知らせない。
    pub(crate) fn confirm_routine_runs(&mut self, yes: bool) {
        let Some(dir) = self.routine.dir.clone() else {
            return;
        };
        let runs = std::mem::take(&mut self.routine.asked);
        let due = std::mem::take(&mut self.routine.due);
        let asked = |r: &Run| runs.iter().any(|a| a.key == r.key);
        if !yes {
            let today = self
                .routine
                .day
                .unwrap_or_else(|| mdgrid::clock::today_now(|k| std::env::var(k).ok()).0);
            self.routine.due = due.into_iter().filter(|r| !asked(r)).collect();
            self.message = Some(match mdgrid::routine::dismiss(&dir, &runs, today) {
                Ok(()) => Msg::RoutineDismissed.into(),
                Err(e) => e,
            });
            return;
        }
        let report = mdgrid::routine::make(&dir, &runs);
        let rest: Vec<Run> = due.into_iter().filter(|r| !asked(r)).collect();
        self.routine.due = runs
            .into_iter()
            .chain(rest)
            .filter(|r| !report.done.contains(&r.key))
            .collect();
        self.routine_reload(&report.made);
        let mut lines = vec![Msg::RoutineMade.fill(&[&self.runs_word(), &report.made.len()])];
        lines.extend(report.failures.iter().chain(&report.warnings).cloned());
        self.message = Some(lines.join(Msg::SentenceSep.text()));
    }

    /// 作った回のノートのうち、開いている表(作る場所のフォルダ)の中のものを読み直して行にする。git にも渡す。
    fn routine_reload(&mut self, made: &[PathBuf]) {
        let mut any = false;
        for p in made {
            let real = p.canonicalize().unwrap_or_else(|_| p.clone());
            if !self.note.places.iter().any(|d| real.starts_with(d)) {
                continue;
            }
            if self
                .src
                .reload(&RowId(real.to_string_lossy().into_owned()))
                .is_ok()
            {
                any = true;
            }
        }
        if any {
            self.data_gen += 1;
            self.refresh();
        }
        for p in made {
            self.git_note(p.clone());
        }
        self.git_after_write();
    }

    /// 確かめを Esc で閉じた: やめたのと同じ。
    pub(crate) fn routine_dismiss(&mut self) {
        self.confirm_routine_runs(false);
    }

    /// routine.cli: その場で来ている回を全部、確かめずに作る(`--runs`)。ルーチンの表が無ければ何もせず、記録の
    /// 置き場が無ければ理由を返す。
    pub(crate) fn routine_once(&self, today: i64, now: i64) -> Report {
        if self.routine.tables.is_empty() {
            return Report::default();
        }
        let Some(dir) = self.routine.dir.clone() else {
            return Report {
                failures: vec![Msg::RoutineNoStateDir.text().to_string()],
                ..Report::default()
            };
        };
        let loaded = load(&self.routine.tables, today, now);
        mdgrid::routine::run_all(&dir, &loaded, today, now)
    }
}
