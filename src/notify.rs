//! 新しく当たった行を知らせる(notify.*)の核: 設定の `[[notify]]` を読み、確かめた行と記録を比べて鳴らす行を決め、
//! 出す先(端末の合図・OS の通知・自分のコマンド)へ渡す。表を開いて行を選ぶのは画面(src/ui/notify.rs)と
//! `--notify` の側で、ここは表を知らない。形は docs/design.md の「知らせ(notify)」。

use crate::i18n::Msg;
use crate::notice::Scope;
use crate::profile::{not, warn};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// notify.group: まとめる行の数の既定(これを超えたら1件にまとめる。tbd~notify.group-default)。
pub const GROUP_OVER: usize = 3;
/// notify.while-open: 開いている間に確かめる間隔(tbd~notify.interval)。
pub const INTERVAL: Duration = Duration::from_secs(60);
/// notify.fail: 出す先を打ち切る時間(tbd~notify.timeout)。
pub const TIMEOUT: Duration = Duration::from_secs(10);
/// notify.state: 記録のファイルの名前(見た目の状態の置き場の中)。
pub const STATE_FILE: &str = "notify.json";
/// notify.state: 画面と `--notify` が記録を同時に読み書きしないための錠のファイル。
const LOCK_FILE: &str = "notify.lock";

/// notify.to: 出す先。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum To {
    Terminal,
    Desktop,
    Command,
}

impl To {
    pub const ALL: [To; 3] = [To::Terminal, To::Desktop, To::Command];

    /// 設定に書く名前。
    pub fn name(self) -> &'static str {
        match self {
            To::Terminal => "terminal",
            To::Desktop => "desktop",
            To::Command => "command",
        }
    }
}

/// `[[notify]]` の1項目。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub cond: String,
    pub scope: Scope,
    pub title: Option<String>,
    pub text: Option<String>,
    pub to: Vec<To>,
    pub group_over: usize,
    pub group_text: Option<String>,
    pub command: Vec<String>,
}

/// config.toml の `[[notify]]` を読む(書いた順)。形の問題は警告して、その鍵か項目を外す。
pub fn read(v: &toml::Value, file: &str, w: &mut Vec<String>) -> Vec<Item> {
    let Some(items) = v
        .as_array()
        .filter(|xs| xs.iter().all(toml::Value::is_table))
    else {
        warn(w, file, "notify", not(Msg::WantNotifyArray));
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(i, t)| read_item(t.as_table()?, file, &format!("notify[{}]", i + 1), w))
        .collect()
}

const KEYS: &[&str] = &[
    "cond",
    "scope",
    "title",
    "text",
    "to",
    "group_over",
    "group_text",
    "command",
];

fn read_item(t: &toml::Table, file: &str, path: &str, w: &mut Vec<String>) -> Option<Item> {
    for k in t.keys().filter(|k| !KEYS.contains(&k.as_str())) {
        warn(
            w,
            file,
            &format!("{path}.{k}"),
            Msg::RsnUnknown.text().to_string(),
        );
    }
    let Some(cond) = t.get("cond").and_then(toml::Value::as_str) else {
        warn(w, file, path, Msg::RsnNotifyNeedCond.text().to_string());
        return None;
    };
    let string = |key: &str, w: &mut Vec<String>| {
        t.get(key).and_then(|v| {
            let s = v.as_str().map(str::to_string);
            if s.is_none() {
                warn(w, file, &format!("{path}.{key}"), not(Msg::WantString));
            }
            s
        })
    };
    let title = string("title", w);
    let text = string("text", w);
    let group_text = string("group_text", w);
    // notify.item: 書かなければ範囲の表の全部(tbd~notify.default-scope)。
    let scope = t.get("scope").map_or(Scope::Workspace, |v| {
        crate::notice::read_scope(v).unwrap_or_else(|| {
            warn(w, file, &format!("{path}.scope"), not(Msg::WantNoticeScope));
            Scope::Workspace
        })
    });
    let group_over = match t.get("group_over") {
        None => GROUP_OVER,
        Some(v) => v
            .as_integer()
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or_else(|| {
                warn(
                    w,
                    file,
                    &format!("{path}.group_over"),
                    not(Msg::WantIntAtLeast0),
                );
                GROUP_OVER
            }),
    };
    let strings = |key: &str, w: &mut Vec<String>| -> Option<Vec<String>> {
        let v = t.get(key)?;
        let list = v.as_array().and_then(|xs| {
            xs.iter()
                .map(|x| x.as_str().map(str::to_string))
                .collect::<Option<Vec<_>>>()
        });
        if list.is_none() {
            warn(w, file, &format!("{path}.{key}"), not(Msg::WantStringList));
        }
        list
    };
    let command = strings("command", w).unwrap_or_default();
    // notify.to: 書かなければ OS の通知(tbd~notify.default-to)。知らない値はその値だけを外す。
    let mut to = Vec::new();
    for name in strings("to", w).unwrap_or_else(|| vec![To::Desktop.name().to_string()]) {
        match To::ALL.into_iter().find(|x| x.name() == name) {
            Some(x) if !to.contains(&x) => to.push(x),
            Some(_) => {}
            None => warn(
                w,
                file,
                &format!("{path}.to"),
                Msg::RsnNotifyUnknownTo.fill(&[&name]),
            ),
        }
    }
    if to.contains(&To::Command) && command.is_empty() {
        warn(
            w,
            file,
            &format!("{path}.command"),
            Msg::RsnNotifyNoCommand.text().to_string(),
        );
        to.retain(|x| *x != To::Command);
    }
    Some(Item {
        cond: cond.to_string(),
        scope,
        title,
        text,
        to,
        group_over,
        group_text,
        command,
    })
}

/// `{名前}` を `value(名前)` に置き換える。`value` が None の名前と、閉じない `{` はそのまま残す。
pub fn fill(template: &str, value: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find(['{', '}']) {
            Some(close) if after.as_bytes()[close] == b'}' => {
                let name = &after[..close];
                if let Some(v) = value(name) {
                    out.push_str(&v);
                } else {
                    out.push('{');
                    out.push_str(name);
                    out.push('}');
                }
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// notify.sanitize: 制御文字(改行・ESC・BEL など)を空白に替える。
pub fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// 条件に当たった1行(題と文は、項目の `title`・`text` を行の値で埋めたもの)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    /// 行の見分け(ノートはファイルのパス、CSV・TSV は表のパスと行の値の並び。tbd~notify.csv-row-id)。
    pub id: String,
    /// 行のファイルのパス(`{path}`)。
    pub path: PathBuf,
    pub title: String,
    pub text: String,
}

/// 出す1件の知らせ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub title: String,
    pub text: String,
    pub path: PathBuf,
}

/// 1つの項目の確かめ: 項目、条件の見せる名前、範囲の表ごとの当たった行(読めない表は理由)。
#[derive(Clone, Debug)]
pub struct Check {
    pub item: Item,
    pub label: String,
    /// (表の見分け(開いたパス), 当たった行か読めなかった理由)。
    pub tables: Vec<(String, Result<Vec<Hit>, String>)>,
}

/// 1回の確かめの結果。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// どれかの出す先に届いた知らせ。
    pub rung: Vec<Note>,
    /// 届けられなかった出す先と読めなかった表の理由(1件1行)。
    pub failures: Vec<String>,
    /// 記録を作り直した旨など。
    pub warnings: Vec<String>,
}

/// 出す先へ1件を渡す関数。
pub type Deliver<'a> = dyn FnMut(To, &Item, &Note) -> Result<(), String> + 'a;

/// 記録(`dir` の `STATE_FILE`)を錠で守って読み、新しく当たった行を `deliver` で鳴らし、記録を書く。
/// `checks` が空なら何もしない(ファイルも作らない)。
pub fn run(dir: &Path, checks: &[Check], deliver: &mut Deliver<'_>) -> Report {
    let mut report = Report::default();
    if checks.is_empty() {
        return report;
    }
    // notify.state: 錠を取ってから読み、鳴らして書くまで持つ(画面と `--notify` が同じ行を二度鳴らさない)。
    let lock = std::fs::create_dir_all(dir).and_then(|()| {
        let f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join(LOCK_FILE))?;
        f.lock()?;
        Ok(f)
    });
    let lock = match lock {
        Ok(f) => f,
        Err(e) => {
            let at = dir.join(STATE_FILE);
            report
                .failures
                .push(Msg::NotifyStateUnwritable.fill(&[&at.display(), &e]));
            return report;
        }
    };
    let mut seen = load(dir, &mut report);
    for c in checks {
        ring(c, &mut seen, deliver, &mut report);
    }
    if let Err(e) = save(dir, &seen) {
        let at = dir.join(STATE_FILE);
        report
            .failures
            .push(Msg::NotifyStateUnwritable.fill(&[&at.display(), &e]));
    }
    drop(lock);
    report
}

/// 記録: (条件の名前と表の見分け, 前の確かめで当たっていた行の見分け)。
type Seen = BTreeMap<String, BTreeSet<String>>;

#[derive(serde::Serialize, serde::Deserialize)]
struct StateFile {
    version: u32,
    seen: Seen,
}

/// 記録を読む。無ければ空。読めなければ空にして警告する(notify.state-broken: 空の記録では、どの表も
/// 初めての確かめになって黙って覚え直す)。
fn load(dir: &Path, report: &mut Report) -> Seen {
    let at = dir.join(STATE_FILE);
    if !at.exists() {
        return Seen::new();
    }
    let read = crate::textfile::read_text(&at)
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str::<StateFile>(&t).map_err(|e| e.to_string()));
    match read {
        Ok(f) => f.seen,
        Err(e) => {
            report
                .warnings
                .push(Msg::NotifyStateRebuilt.fill(&[&at.display(), &e]));
            Seen::new()
        }
    }
}

fn save(dir: &Path, seen: &Seen) -> std::io::Result<()> {
    let f = StateFile {
        version: 1,
        seen: seen.clone(),
    };
    let text = serde_json::to_string_pretty(&f).map_err(std::io::Error::other)?;
    crate::config::write_atomic(dir, STATE_FILE, text.as_bytes())
}

/// 1つの項目: 表ごとに記録と比べ、新しく当たった行を(多ければまとめて)鳴らし、記録を進める。
fn ring(c: &Check, seen: &mut Seen, deliver: &mut Deliver<'_>, report: &mut Report) {
    // (表の記録の鍵, 今当たっている行の見分け, 新しく当たった行)。
    let mut tables: Vec<(String, BTreeSet<String>, Vec<&Hit>)> = Vec::new();
    for (table, hits) in &c.tables {
        let key = format!("{}\t{table}", c.item.cond);
        let hits = match hits {
            Ok(h) => h,
            // notify.fail: 読めない表は理由を出し、記録は消さない。
            Err(e) => {
                report.failures.push(e.clone());
                continue;
            }
        };
        let now: BTreeSet<String> = hits.iter().map(|h| h.id.clone()).collect();
        let Some(before) = seen.get(&key) else {
            // notify.first-silent: 初めて確かめる表は、当たっている行を覚えるだけ。
            seen.insert(key, now);
            continue;
        };
        let mut fresh: Vec<&Hit> = hits.iter().filter(|h| !before.contains(&h.id)).collect();
        fresh.sort_by(|a, b| a.id.cmp(&b.id));
        fresh.dedup_by(|a, b| a.id == b.id);
        tables.push((key, now, fresh));
    }
    let total: usize = tables.iter().map(|(_, _, f)| f.len()).sum();
    // 鳴らせた行の見分け(表の記録の鍵ごと)。
    let mut rung: BTreeSet<(usize, String)> = BTreeSet::new();
    if total > c.item.group_over {
        // notify.group: 決めた数を超えたら1件にまとめる。
        let text = match &c.item.group_text {
            Some(t) => t.replace("{count}", &total.to_string()),
            None => Msg::NotifyGroup.fill(&[&total]),
        };
        let note = Note {
            title: clean(&c.label),
            text: clean(&text),
            path: PathBuf::new(),
        };
        if send(&c.item, &note, deliver, report) {
            for (k, (_, _, fresh)) in tables.iter().enumerate() {
                rung.extend(fresh.iter().map(|h| (k, h.id.clone())));
            }
        }
    } else {
        for (k, (_, _, fresh)) in tables.iter().enumerate() {
            for h in fresh {
                let note = Note {
                    title: clean(&h.title),
                    text: clean(&h.text),
                    path: h.path.clone(),
                };
                if send(&c.item, &note, deliver, report) {
                    rung.insert((k, h.id.clone()));
                }
            }
        }
    }
    for (k, (key, now, _)) in tables.into_iter().enumerate() {
        let before = seen.remove(&key).unwrap_or_default();
        // 当たり続けている行と、鳴らせた行を覚える。外れた行は忘れ、また当たれば鳴らす(notify.new-match)。
        // 鳴らせなかった行は覚えず、次の確かめでまた鳴らす(notify.fail)。
        let next: BTreeSet<String> = now
            .into_iter()
            .filter(|id| before.contains(id) || rung.contains(&(k, id.clone())))
            .collect();
        seen.insert(key, next);
    }
}

/// 1件を項目の出す先の全部へ渡す。どれかに届けば真。届かなかった出す先は理由を残す。
fn send(item: &Item, note: &Note, deliver: &mut Deliver<'_>, report: &mut Report) -> bool {
    let mut any = false;
    for to in &item.to {
        match deliver(*to, item, note) {
            Ok(()) => any = true,
            Err(e) => report
                .failures
                .push(Msg::NotifyFailed.fill(&[&to.name(), &e])),
        }
    }
    if any {
        report.rung.push(note.clone());
    }
    any
}

/// notify.terminal: 端末の通知の合図の種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    /// iTerm2。
    Osc9,
    /// Ghostty・WezTerm・foot・Windows Terminal。
    Osc777,
    /// kitty。
    Osc99,
    /// 見分けられない端末。
    Bel,
}

/// 環境変数から端末の合図と、tmux の中か(合図を通り抜けの形で包む)を決める。
pub fn detect_term(env: &dyn Fn(&str) -> Option<String>) -> (Term, bool) {
    let get = |k: &str| env(k).filter(|v| !v.is_empty());
    let has = |k: &str| get(k).is_some();
    let term = get("TERM").unwrap_or_default();
    let program = get("TERM_PROGRAM").unwrap_or_default();
    let tmux = has("TMUX");
    // tmux の中では TERM と TERM_PROGRAM が tmux のものになるので、外の端末が残す環境変数でも見分ける。
    let kind = if term == "xterm-kitty" || has("KITTY_WINDOW_ID") {
        Term::Osc99
    } else if program == "iTerm.app"
        || has("ITERM_SESSION_ID")
        || get("LC_TERMINAL").as_deref() == Some("iTerm2")
    {
        Term::Osc9
    } else if program == "ghostty"
        || program == "WezTerm"
        || term == "xterm-ghostty"
        || term == "wezterm"
        || term.starts_with("foot")
        || has("GHOSTTY_RESOURCES_DIR")
        || has("WEZTERM_PANE")
        || has("WT_SESSION")
    {
        Term::Osc777
    } else {
        Term::Bel
    };
    (kind, tmux)
}

/// 端末へ送るバイト(合図1つ。tmux の中なら通り抜けの DCS で包む)。
pub fn terminal_bytes(term: Term, tmux: bool, title: &str, text: &str) -> Vec<u8> {
    let (title, text) = (clean(title), clean(text));
    let seq = match term {
        Term::Bel => return b"\x07".to_vec(),
        Term::Osc9 => format!("\x1b]9;{title}: {text}\x07"),
        // 題の中の `;` は区切りと読まれるので、`,` にする。
        Term::Osc777 => format!("\x1b]777;notify;{};{text}\x07", title.replace(';', ",")),
        Term::Osc99 => {
            static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let id = ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            format!(
                "\x1b]99;i=mdgrid{id}:d=0;{title}\x1b\\\x1b]99;i=mdgrid{id}:d=1:p=body;{text}\x1b\\"
            )
        }
    };
    if !tmux {
        return seq.into_bytes();
    }
    // tmux の通り抜け(`allow-passthrough on` のとき): DCS `tmux;` の中の ESC は2つに重ねる。
    let mut out = b"\x1bPtmux;".to_vec();
    out.extend(seq.replace('\x1b', "\x1b\x1b").into_bytes());
    out.extend(b"\x1b\\");
    out
}

/// notify.desktop: OS の通知の仕組みの選び分け。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Mac,
    Linux,
    Windows,
    Wsl,
}

/// 作った OS(`target` は "macos"・"windows"・ほか)と環境変数・カーネルの版の名前から OS を決める
/// (WSL は Linux より先に見分ける)。
pub fn detect_os(
    target: &str,
    env: &dyn Fn(&str) -> Option<String>,
    osrelease: Option<&str>,
) -> Os {
    match target {
        "macos" => Os::Mac,
        "windows" => Os::Windows,
        _ if env("WSL_DISTRO_NAME").is_some_and(|v| !v.is_empty())
            || osrelease.is_some_and(|r| r.to_ascii_lowercase().contains("microsoft")) =>
        {
            Os::Wsl
        }
        _ => Os::Linux,
    }
}

/// 今の OS(WSL はカーネルの版の名前でも見分ける)。
pub fn current_os() -> Os {
    let osrelease = if cfg!(target_os = "linux") {
        crate::textfile::read_text("/proc/sys/kernel/osrelease").ok()
    } else {
        None
    };
    detect_os(
        std::env::consts::OS,
        &|k| std::env::var(k).ok(),
        osrelease.as_deref(),
    )
}

/// 本物の出す先(OS と端末は1回だけ見分ける)。
pub struct Outlets {
    pub os: Os,
    pub term: Term,
    pub tmux: bool,
    pub wslenv: Option<String>,
    pub timeout: Duration,
}

impl Outlets {
    /// 今の環境の出す先。
    pub fn current() -> Outlets {
        let env = |k: &str| std::env::var(k).ok();
        let (term, tmux) = detect_term(&env);
        Outlets {
            os: current_os(),
            term,
            tmux,
            wslenv: env("WSLENV"),
            timeout: TIMEOUT,
        }
    }

    /// 1件を1つの出す先へ渡す。端末の合図は `terminal` に渡す(画面は描いたあとに送り、`--notify` は標準出力へ)。
    pub fn send(
        &self,
        to: To,
        item: &Item,
        note: &Note,
        terminal: &mut dyn FnMut(Vec<u8>) -> Result<(), String>,
    ) -> Result<(), String> {
        match to {
            To::Terminal => terminal(terminal_bytes(
                self.term,
                self.tmux,
                &note.title,
                &note.text,
            )),
            To::Desktop => run_spawn(
                &desktop_spawn(self.os, &note.title, &note.text, self.wslenv.as_deref()),
                self.timeout,
            ),
            To::Command => match command_spawn(&item.command, note) {
                Some(sp) => run_spawn(&sp, self.timeout),
                None => Err(Msg::RsnNotifyNoCommand.text().to_string()),
            },
        }
    }
}

/// 動かすプログラム(シェルを通さない)。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spawn {
    pub program: String,
    pub args: Vec<String>,
    /// 標準入力に渡すバイト(無ければ空の入力)。
    pub stdin: Option<Vec<u8>>,
    /// 足す環境変数。
    pub env: Vec<(String, String)>,
}

/// OS の通知に題と文を出すプログラム。`wslenv` は今の `WSLENV`(WSL のときに足して渡す)。
pub fn desktop_spawn(os: Os, title: &str, text: &str, wslenv: Option<&str>) -> Spawn {
    match os {
        // 台本は標準入力から読み(`-`)、題と文は引数で渡す(台本の文字に混ぜない)。
        Os::Mac => Spawn {
            program: "osascript".into(),
            args: vec!["-".into(), title.into(), text.into()],
            stdin: Some(MAC_SCRIPT.as_bytes().to_vec()),
            env: Vec::new(),
        },
        Os::Linux => Spawn {
            program: "notify-send".into(),
            args: vec![
                "--app-name=mdgrid".into(),
                "--".into(),
                title.into(),
                text.into(),
            ],
            stdin: None,
            env: Vec::new(),
        },
        // Windows PowerShell 5.1 の WinRT のトースト。題と文は環境変数で渡す(台本の文字に混ぜない)。WSL からは
        // WSLENV に名前を足すと Windows の側に渡る。
        Os::Windows | Os::Wsl => {
            let mut env = vec![
                ("MDGRID_NOTIFY_TITLE".to_string(), title.to_string()),
                ("MDGRID_NOTIFY_TEXT".to_string(), text.to_string()),
            ];
            if os == Os::Wsl {
                let add = "MDGRID_NOTIFY_TITLE/u:MDGRID_NOTIFY_TEXT/u";
                let v = match wslenv.filter(|v| !v.is_empty()) {
                    Some(v) => format!("{v}:{add}"),
                    None => add.to_string(),
                };
                env.push(("WSLENV".to_string(), v));
            }
            Spawn {
                program: "powershell.exe".into(),
                args: vec![
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-EncodedCommand".into(),
                    encoded_command(WINDOWS_SCRIPT),
                ],
                stdin: None,
                env,
            }
        }
    }
}

const MAC_SCRIPT: &str =
    "on run argv\ndisplay notification (item 2 of argv) with title (item 1 of argv)\nend run\n";

/// 登録の要らない PowerShell の AUMID で出す(mdgrid のアプリの登録をしない)。
const WINDOWS_SCRIPT: &str = "$ErrorActionPreference = 'Stop'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
$x = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
$t = $x.GetElementsByTagName('text')
$t.Item(0).AppendChild($x.CreateTextNode($env:MDGRID_NOTIFY_TITLE)) | Out-Null
$t.Item(1).AppendChild($x.CreateTextNode($env:MDGRID_NOTIFY_TEXT)) | Out-Null
$app = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($app).Show([Windows.UI.Notifications.ToastNotification]::new($x))
";

/// `-EncodedCommand` の形(UTF-16LE の base64)。
fn encoded_command(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64(&bytes)
}

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(T[(n >> shift & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// notify.command: 項目の `command` の引数の `{title}`・`{text}`・`{path}` を埋めたプログラム(空なら None)。
pub fn command_spawn(argv: &[String], note: &Note) -> Option<Spawn> {
    let (program, args) = argv.split_first()?;
    let path = note.path.display().to_string();
    let value = |k: &str| match k {
        "title" => Some(note.title.clone()),
        "text" => Some(note.text.clone()),
        "path" => Some(path.clone()),
        _ => None,
    };
    Some(Spawn {
        program: program.clone(),
        args: args.iter().map(|a| fill(a, &value)).collect(),
        stdin: None,
        env: Vec::new(),
    })
}

/// プログラムを動かし、`timeout` までに終了コード 0 で終われば Ok。出力は捨てる。
pub fn run_spawn(sp: &Spawn, timeout: Duration) -> Result<(), String> {
    let mut cmd = Command::new(&sp.program);
    cmd.args(&sp.args)
        .envs(sp.env.iter().map(|(k, v)| (k, v)))
        .stdin(if sp.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        // 出力は捨てる(画面や `--notify` の出力に混ぜない)。
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: 窓を出さない(定期実行から呼んでも窓が一瞬出ない)。
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| Msg::NotifyNoProgram.fill(&[&sp.program, &e]))?;
    if let (Some(input), Some(mut pipe)) = (&sp.stdin, child.stdin.take()) {
        // 台本は小さいので、書き切れずに待つことは無い。書けなくても終了コードで分かる。
        let _ = pipe.write_all(input);
    }
    let end = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => return Ok(()),
            Ok(Some(st)) => {
                let code = st.code().map_or_else(|| "?".to_string(), |c| c.to_string());
                return Err(Msg::NotifyExit.fill(&[&sp.program, &code]));
            }
            Ok(None) if Instant::now() >= end => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Msg::NotifyTimeout.fill(&[&timeout.as_secs_f64()]));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(Msg::NotifyNoProgram.fill(&[&sp.program, &e])),
        }
    }
}
