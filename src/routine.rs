//! ルーチンの回を作る(routine.*)の核。印の `[[table]]` の `runs` を読み、ルーチン(表の行)の決まり
//! (`recurrence`)から今日までの一番新しい回の日を決め、回のノートの名前と中身を作り、作った回の記録
//! (見た目の状態の置き場の `routine.json`)を錠で守って読み書きする。表を開いてルーチンの行を集めるのは
//! 画面の側(src/ui/routine.rs)で、ここは表を知らない。形は docs/design.md の「ルーチンの回」。

use crate::i18n::Msg;
use crate::newnote::{self, Ids, Vars};
use crate::profile::{not, warn};
use crate::recurrence::Rule;
use crate::types;
use crate::writeback::{self, Edit, NewValue};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// routine.state: 記録のファイルの名前(見た目の状態の置き場の中)。
pub const STATE_FILE: &str = "routine.json";
/// routine.once: 画面と `--runs`、2つの `--runs` が同時に作らないための錠のファイル。
const LOCK_FILE: &str = "routine.lock";
/// routine.rule: 起点の列(`every` の回はこの日から数える)。
pub const START: &str = "start";
/// routine.run-note: 名前の雛形の既定。
pub const DEFAULT_NAME: &str = "{routine} {date:YYYY-MM-DD}";
/// routine.run-note: ルーチンへのリンクと回の日を書く列。
const ROUTINE_KEY: &str = "routine";
const DATE_KEY: &str = "date";

/// 印の `runs = { folder, rule, name, set, copy, body }`。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Spec {
    /// 回のノートを作るフォルダ(印の根からの相対)。
    pub folder: String,
    /// 決まりの列。
    pub rule: String,
    /// 名前の雛形(空なら `DEFAULT_NAME`)。
    pub name: String,
    /// 回のノートに入れる値(列 = 値)。
    pub set: Vec<(String, NewValue)>,
    /// ルーチンの値を写す列。
    pub copy: Vec<String>,
    /// 本文の雛形のファイル(印の根からの相対。空なら見出し1行)。
    pub body: String,
}

const KEYS: [&str; 6] = ["folder", "rule", "name", "set", "copy", "body"];

/// 印の `runs` を読む(警告の場所は `at`)。書き方が違えば CLI-3 の形で警告して None(表でない・`folder` か
/// `rule` が無い)。知らない鍵と、読めない `set`・`copy` の値は警告して、その鍵か値だけを外す。
pub fn read(v: &toml::Value, file: &str, at: &str, warns: &mut Vec<String>) -> Option<Spec> {
    let Some(t) = v.as_table() else {
        warn(warns, file, at, Msg::RsnRunsNotTable.text().to_string());
        return None;
    };
    for k in t.keys().filter(|k| !KEYS.contains(&k.as_str())) {
        warn(
            warns,
            file,
            &format!("{at}.{k}"),
            Msg::RsnUnknown.text().to_string(),
        );
    }
    let text = |key: &str, warns: &mut Vec<String>| -> String {
        match t.get(key) {
            None => String::new(),
            Some(toml::Value::String(s)) => s.trim().to_string(),
            Some(_) => {
                warn(warns, file, &format!("{at}.{key}"), not(Msg::WantString));
                String::new()
            }
        }
    };
    let folder = text("folder", warns);
    let rule = text("rule", warns);
    let name = text("name", warns);
    let body = text("body", warns);
    let mut set = Vec::new();
    match t.get("set") {
        None => {}
        Some(toml::Value::Table(cols)) => {
            for (col, x) in cols {
                let p = format!("{at}.set.{col}");
                if newnote::not_a_key(col) {
                    warn(warns, file, &p, Msg::RsnBadColumn.fill(&[col]));
                    continue;
                }
                match newnote::value_from_toml(x) {
                    Some(nv) => set.push((col.clone(), nv)),
                    None => warn(warns, file, &p, not(Msg::WantSetValue)),
                }
            }
        }
        Some(_) => warn(
            warns,
            file,
            &format!("{at}.set"),
            not(Msg::WantColumnValueTable),
        ),
    }
    let copy = match t.get("copy") {
        None => Vec::new(),
        Some(v) => v
            .as_array()
            .and_then(|xs| {
                xs.iter()
                    .map(|x| x.as_str().map(|s| s.trim().to_string()))
                    .collect::<Option<Vec<_>>>()
            })
            .unwrap_or_else(|| {
                warn(warns, file, &format!("{at}.copy"), not(Msg::WantStringList));
                Vec::new()
            }),
    };
    if folder.is_empty() {
        warn(
            warns,
            file,
            &format!("{at}.folder"),
            Msg::RsnRunsNeedFolder.text().to_string(),
        );
        return None;
    }
    if rule.is_empty() {
        warn(
            warns,
            file,
            &format!("{at}.rule"),
            Msg::RsnRunsNeedRule.text().to_string(),
        );
        return None;
    }
    Some(Spec {
        folder,
        rule,
        name,
        set,
        copy,
        body,
    })
}

/// ルーチンの表(印の `[[table]]` に `runs` を書いた表)。
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    /// 表の名前(理由に添える)。
    pub name: String,
    /// 開くパス(フォルダか `.base`)。
    pub path: PathBuf,
    /// 印の根(`folder` と `body` の基準)。
    pub root: PathBuf,
    pub spec: Spec,
}

impl Table {
    /// 表の見分け(開くパスの実体。記録の鍵の頭)。
    fn key(&self) -> String {
        std::fs::canonicalize(&self.path)
            .unwrap_or_else(|_| self.path.clone())
            .display()
            .to_string()
    }
}

/// ルーチン1つ(ルーチンの表の1行)。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Routine {
    /// ノートの名前(`.md` なし)。
    pub name: String,
    /// ノートのパス。
    pub path: PathBuf,
    /// 決まりの列の値(無ければ空)。
    pub rule: String,
    /// 起点の列 `start` の値。
    pub start: Option<String>,
    /// `copy` の列の値(値のある列だけ)。
    pub values: Vec<(String, NewValue)>,
}

/// ルーチンの表と、その行(読めなければ理由)。
pub type Loaded = (Table, Result<Vec<Routine>, String>);

/// 作る回1つ。
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    /// 記録の鍵(表とルーチンのノート)。
    pub key: String,
    /// 回の日。
    pub date: i64,
    /// ルーチンの名前。
    pub routine: String,
    /// 見せる名前(印の根からの相対のパス)。
    pub shown: String,
    /// 回のノートのパス(決まらなければ理由)。
    pub path: Result<PathBuf, String>,
    /// 書く中身(作れなければ理由)。
    pub bytes: Result<Vec<u8>, String>,
    /// 同じ名前のノートが既にある。
    pub exists: bool,
}

/// 回を決めた結果。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub runs: Vec<Run>,
    /// 読めない表と決まりの理由(1件1行)。
    pub problems: Vec<String>,
}

/// 回を作った結果。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// 作ったノートのパス。
    pub made: Vec<PathBuf>,
    /// 済んだ回の鍵(作った・同じ名前のノートがあった・もう記録にあった)。
    pub done: Vec<String>,
    /// 作れなかった回と読めない表・決まりの理由(1件1行)。
    pub failures: Vec<String>,
    /// 記録を作り直した旨など。
    pub warnings: Vec<String>,
}

/// 記録のファイル。
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct State {
    version: u32,
    /// 鍵ごとに、作った(か、同じ名前のノートがあった)一番新しい回の日(`YYYY-MM-DD`)。
    #[serde(default)]
    made: BTreeMap<String, String>,
    /// routine.screen-create: 画面でやめた回。鍵ごとに(やめた日, 回の日)。
    #[serde(default)]
    dismissed: BTreeMap<String, (String, String)>,
}

impl State {
    fn made_on_or_after(&self, key: &str, date: i64) -> bool {
        self.made
            .get(key)
            .and_then(|d| types::parse_date(d))
            .is_some_and(|d| d >= date)
    }
}

/// 記録を読む。無ければ空。読めなければ空と、作り直す旨(routine.state)。
fn load(dir: &Path) -> (State, Option<String>) {
    let at = dir.join(STATE_FILE);
    if !at.exists() {
        return (State::default(), None);
    }
    let read = crate::textfile::read_text(&at)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str::<State>(&t).map_err(|e| e.to_string()));
    match read {
        Ok(s) => (s, None),
        Err(e) => (
            State::default(),
            Some(Msg::RoutineStateRebuilt.fill(&[&at.display(), &e])),
        ),
    }
}

fn save(dir: &Path, state: &State) -> std::io::Result<()> {
    let s = State {
        version: 1,
        made: state.made.clone(),
        dismissed: state.dismissed.clone(),
    };
    let text = serde_json::to_string_pretty(&s).map_err(std::io::Error::other)?;
    crate::config::write_atomic(dir, STATE_FILE, format!("{text}\n").as_bytes())
}

/// routine.once: 錠を取る(落とすまで持つ)。
fn lock(dir: &Path) -> std::io::Result<std::fs::File> {
    std::fs::create_dir_all(dir)?;
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(LOCK_FILE))?;
    f.lock()?;
    Ok(f)
}

/// 記録を書けない理由。
fn unwritable(dir: &Path, e: &dyn std::fmt::Display) -> String {
    Msg::RoutineStateUnwritable.fill(&[&dir.join(STATE_FILE).display(), e])
}

/// 記録と比べて、来ている回を決める(書かない)。記録にある回(その日か後の回を作った)は入れない。
fn plan(state: &State, tables: &[Loaded], today: i64, now: i64) -> Plan {
    let mut out = Plan::default();
    for (t, routines) in tables {
        let routines = match routines {
            Ok(r) => r,
            Err(e) => {
                out.problems.push(format!("{}: {e}", t.name));
                continue;
            }
        };
        for r in routines {
            let text = r.rule.trim();
            // routine.rule-unreadable: 決まりを書いていないルーチンは、回を作らないルーチン。
            if text.is_empty() {
                continue;
            }
            let Some(rule) = Rule::parse(text) else {
                out.problems
                    .push(Msg::RoutineRuleUnreadable.fill(&[&r.name, &text]));
                continue;
            };
            let start = r
                .start
                .as_deref()
                .and_then(|s| s.trim().get(..10))
                .and_then(types::parse_date);
            if rule.needs_start() && start.is_none() {
                out.problems
                    .push(Msg::RoutineRuleNoStart.fill(&[&r.name, &text, &START]));
                continue;
            }
            // routine.latest-only: 今日までの一番新しい回だけ。今日より後の回は作らない。
            let Some(date) = rule.latest(start, today) else {
                continue;
            };
            let key = format!("{}\t{}", t.key(), r.path.display());
            // routine.once: 記録にあるその日か後の回は作らない(ノートを人が消していても。tbd~routine.deleted-run)。
            // 記録の無いルーチン(`runs` を初めて書いた)は、今の回を作る(tbd~routine.first-run)。
            if state.made_on_or_after(&key, date) {
                continue;
            }
            out.runs.push(build(t, r, key, date, now));
        }
    }
    out
}

/// `{routine}` をルーチンの名前に、ほかの変数を `expand` で埋める(ルーチンの名前の中の `{…}` は埋めない)。
fn with_routine(s: &str, routine: &str, expand: impl Fn(&str) -> String) -> String {
    s.split("{routine}")
        .map(expand)
        .collect::<Vec<_>>()
        .join(routine)
}

/// 回1つ: 名前・パス・中身を決める(routine.run-note)。
fn build(t: &Table, r: &Routine, key: String, date: i64, now: i64) -> Run {
    let s = &t.spec;
    let ids = Ids::new();
    let template = if s.name.trim().is_empty() {
        DEFAULT_NAME
    } else {
        s.name.as_str()
    };
    let name = with_routine(template, &r.name, |p| {
        newnote::expand_name_ids(p, date, &ids)
    });
    let folder = s.folder.trim().trim_matches('/').to_string();
    let shown = format!("{folder}/{}.md", name.trim());
    let path = target(&t.root, &folder, &name);
    let exists = path
        .as_ref()
        .is_ok_and(|p| std::fs::symlink_metadata(p).is_ok());
    let vars = Vars {
        today: date,
        minutes: now.rem_euclid(86_400) / 60,
        name: name.trim().to_string(),
        folder: folder.clone(),
    };
    let bytes = content(t, r, date, &vars, &ids);
    Run {
        key,
        date,
        routine: r.name.clone(),
        shown,
        path,
        bytes,
        exists,
    }
}

/// 回のノートのパス。名前を決められなくても、同じ名前のノートが既にあればそのパス(作らずに記録する)。
fn target(root: &Path, folder: &str, name: &str) -> Result<PathBuf, String> {
    newnote::note_path(root, folder, name).or_else(|e| {
        let plain = root.join(folder).join(format!("{}.md", name.trim()));
        if std::fs::symlink_metadata(&plain).is_ok() {
            Ok(plain)
        } else {
            Err(e)
        }
    })
}

/// 回のノートの中身: ルーチンへのリンク・回の日・`set`・`copy` の順のフロントマターと、本文の雛形。
fn content(t: &Table, r: &Routine, date: i64, vars: &Vars, ids: &Ids) -> Result<Vec<u8>, String> {
    let s = &t.spec;
    // relations.link-forms: ルーチンへは `[[名前]]` で指す。
    let mut edits = vec![
        Edit {
            key: ROUTINE_KEY.into(),
            value: NewValue::Str(format!("[[{}]]", r.name)),
        },
        Edit {
            key: DATE_KEY.into(),
            value: NewValue::Date(types::format_date(date)),
        },
    ];
    let mut push = |key: &str, value: NewValue| {
        if !edits.iter().any(|e| e.key == key) {
            edits.push(Edit {
                key: key.to_string(),
                value,
            });
        }
    };
    for (k, v) in &s.set {
        let v = match v {
            NewValue::Str(x) => NewValue::Str(fill(x, &r.name, vars, ids)),
            NewValue::Date(x) => NewValue::Date(fill(x, &r.name, vars, ids)),
            NewValue::List(xs) => {
                NewValue::List(xs.iter().map(|x| fill(x, &r.name, vars, ids)).collect())
            }
            other => other.clone(),
        };
        push(k, v);
    }
    for c in &s.copy {
        if let Some((_, v)) = r.values.iter().find(|(k, _)| k == c) {
            push(c, v.clone());
        }
    }
    let mut out = writeback::apply(b"", &edits).map_err(|e| match e {
        writeback::EditError::Newline => Msg::NewNoteNewline.text().to_string(),
        writeback::EditError::NotEditable(m) => Msg::NewNoteNotEditable.fill(&[&m]),
        e => Msg::NewNoteCannotBuild.fill(&[&format!("{e:?}")]),
    })?;
    let body = match newnote::read_body(&t.root, &s.body)? {
        Some(b) => fill(&b, &r.name, vars, ids),
        None => format!("# {}\n", vars.name),
    };
    out.extend_from_slice(body.as_bytes());
    Ok(out)
}

/// 雛形の変数(新しいノートと同じ)と `{routine}` を埋める。
fn fill(s: &str, routine: &str, vars: &Vars, ids: &Ids) -> String {
    with_routine(s, routine, |p| newnote::expand_ids(p, vars, ids))
}

/// 錠を持ったまま、決めた回を作り、記録を進める(`report` に足す)。記録を書き換えたら真。ルーチンのノート
/// (`last_done`・`next_due`)は書き換えない(tbd~routine.next-due)。
fn make_locked(state: &mut State, runs: &[Run], report: &mut Report) -> bool {
    let mut changed = false;
    for run in runs {
        // routine.once: ほかの確かめ(画面と `--runs`)が先に作っていれば作らない。
        if state.made_on_or_after(&run.key, run.date) {
            report.done.push(run.key.clone());
            continue;
        }
        let made = match (&run.path, &run.bytes) {
            (Ok(path), _) if std::fs::symlink_metadata(path).is_ok() => Ok(None),
            (Err(e), _) | (Ok(_), Err(e)) => {
                Err(Msg::RoutineCannotCreate.fill(&[&run.routine, &run.shown, e]))
            }
            (Ok(path), Ok(bytes)) => match newnote::create(path, bytes) {
                Ok(()) => Ok(Some(path.clone())),
                // 同じ名前のノートが、確かめたあとにできた: 上書きしない。
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(None),
                Err(e) => Err(Msg::RoutineCannotCreate.fill(&[&run.routine, &path.display(), &e])),
            },
        };
        match made {
            Ok(path) => {
                report.made.extend(path);
                report.done.push(run.key.clone());
                state
                    .made
                    .insert(run.key.clone(), types::format_date(run.date));
                state.dismissed.remove(&run.key);
                changed = true;
            }
            // routine.fail: 作れなかった回は記録せず、次の確かめでまた作ろうとする。
            Err(e) => report.failures.push(e),
        }
    }
    changed
}

/// routine.screen-offer: 記録と比べて来ている回を決める(書かない)。同じ名前のノートが既にある回と、その日に
/// 画面でやめた回(routine.screen-create)は入れない。
pub fn due(dir: &Path, tables: &[Loaded], today: i64, now: i64) -> Plan {
    let (state, _) = load(dir);
    let mut p = plan(&state, tables, today, now);
    let today_s = types::format_date(today);
    p.runs.retain(|r| {
        !r.exists
            && state
                .dismissed
                .get(&r.key)
                .is_none_or(|(d, date)| *d != today_s || *date != types::format_date(r.date))
    });
    p
}

/// routine.screen-create: 人が自分で選んだ作る操作で並べる回(書かない)。`due` と同じく同じ名前のノートが
/// 既にある回は入れないが、その日に画面でやめた回は入れる(やめて静かにするのは下の帯の知らせだけ)。
pub fn pending(dir: &Path, tables: &[Loaded], today: i64, now: i64) -> Plan {
    let (state, _) = load(dir);
    let mut p = plan(&state, tables, today, now);
    p.runs.retain(|r| !r.exists);
    p
}

/// routine.screen-create: 決めた回を作り、記録する(錠を取り、記録を読み直してから)。
pub fn make(dir: &Path, runs: &[Run]) -> Report {
    let mut report = Report::default();
    let guard = match lock(dir) {
        Ok(g) => g,
        Err(e) => {
            report.failures.push(unwritable(dir, &e));
            return report;
        }
    };
    let (mut state, rebuilt) = load(dir);
    let changed = make_locked(&mut state, runs, &mut report);
    if changed || rebuilt.is_some() {
        match save(dir, &state) {
            Ok(()) => report.warnings.extend(rebuilt),
            Err(e) => report.failures.push(unwritable(dir, &e)),
        }
    }
    drop(guard);
    report
}

/// routine.screen-create: やめた回を、その日のあいだ知らせないよう記録する(前の日にやめた分は忘れる)。
pub fn dismiss(dir: &Path, runs: &[Run], today: i64) -> Result<(), String> {
    let guard = lock(dir).map_err(|e| unwritable(dir, &e))?;
    let (mut state, _) = load(dir);
    let today_s = types::format_date(today);
    state.dismissed.retain(|_, (d, _)| *d == today_s);
    for r in runs {
        state
            .dismissed
            .insert(r.key.clone(), (today_s.clone(), types::format_date(r.date)));
    }
    let r = save(dir, &state).map_err(|e| unwritable(dir, &e));
    drop(guard);
    r
}

/// routine.cli: 錠を取り、来ている回を全部作って記録する。読めない表と決まりの理由も failures に入る。
pub fn run_all(dir: &Path, tables: &[Loaded], today: i64, now: i64) -> Report {
    let mut report = Report::default();
    if tables.is_empty() {
        return report;
    }
    let guard = match lock(dir) {
        Ok(g) => g,
        Err(e) => {
            report.failures.push(unwritable(dir, &e));
            return report;
        }
    };
    let (mut state, rebuilt) = load(dir);
    let p = plan(&state, tables, today, now);
    report.failures.extend(p.problems);
    let changed = make_locked(&mut state, &p.runs, &mut report);
    if changed || rebuilt.is_some() {
        match save(dir, &state) {
            Ok(()) => report.warnings.extend(rebuilt),
            Err(e) => report.failures.push(unwritable(dir, &e)),
        }
    }
    drop(guard);
    report
}
