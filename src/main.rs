//! mdgrid の実行ファイル: 引数(CLI-1・CLI-2・CLI-4。clap で読み、補完と man も同じ定義から出す CLI-13)、設定の読み込み(CLI-3)、端末の準備と後始末、イベントのループ。

mod apply;
mod diff;
mod edit;
mod ui;
mod ws_cli;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{CommandFactory, Parser, ValueEnum, ValueHint};
use clap_complete::Shell;
use mdgrid::base::{self, Base};
use mdgrid::config::{self, Config};
use mdgrid::i18n::{self, Lang, Language, Msg};
use mdgrid::print;
use mdgrid::source::csv::Csv;
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{Source, Value};
use mdgrid::views;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, MouseButton, MouseEventKind};
use ratatui::crossterm::terminal::{
    self, BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use ratatui::crossterm::{cursor, execute};
use ratatui::Terminal;
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};
use ui::startup::Startup;
use ui::{App, ColorMode};

/// 使い方(`--help`。CLI-2)の日本語。文は文言の表(SR-23)にある。
const USAGE: &str = Msg::Usage.ja();

/// 使い方の英語(SR-23)。
const USAGE_EN: &str = Msg::Usage.en();

/// 今の言語の使い方(SR-23)。
fn usage() -> &'static str {
    match i18n::current() {
        Lang::En => USAGE_EN,
        Lang::Ja => USAGE,
    }
}

/// 1回に読むノートの数(BV-16)。描画のたびにこの分だけ進める。
const LOAD_BUDGET: usize = 200;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    /// CLI-11: 既定の設定を出す。
    PrintConfig,
    /// CLI-21: その表で効く設定を、出どころのコメント付きで出す(開くパス)。
    PrintResolved(Vec<PathBuf>),
    /// CLI-20: 設定ファイルを今の形に写して出す。
    MigrateConfig,
    /// CLI-13: シェルの補完の定義を出す。
    Completions(Shell),
    /// CLI-13: man ページを出す。
    Man,
    /// CLI-17: CSV・JSON の値をノートに当てる(開くパスと、読むファイルと、書くか)。
    Apply(Vec<PathBuf>, PathBuf, bool),
    /// CLI-5: 画面を出さずにビューの表を出す(開くパスと形と、先頭にパスの列を足すか(CLI-14)と、
    /// 絞り込みの式と並べ替え(CLI-16))。
    Print(Vec<PathBuf>, PrintFormat, bool, Vec<String>, Vec<String>),
    /// OUT-3: 画面(`/dev/tty`)で選んだ行のパスか列の値を出す(開くパスと出すもの)。
    Pick(Vec<PathBuf>, Pick),
    /// WS-3・WS-7: ワークスペースの操作(画面を出さない)。
    Workspace(ws_cli::Op),
    Run(Vec<PathBuf>),
}

/// `--pick` で出すもの(OUT-3)。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pick {
    /// ノートのパス(起動の引数のフォルダにノートの相対のパスをつないだもの)。
    Path,
    /// 列の値(列の名前)。
    Column(String),
}

impl Pick {
    fn of(what: String) -> Pick {
        if what == "path" {
            Pick::Path
        } else {
            Pick::Column(what)
        }
    }
}

/// `--print` の形(CLI-5)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum PrintFormat {
    Csv,
    Tsv,
    Json,
    Md,
}

impl PrintFormat {
    fn lib(self) -> print::Format {
        match self {
            PrintFormat::Csv => print::Format::Csv,
            PrintFormat::Tsv => print::Format::Tsv,
            PrintFormat::Json => print::Format::Json,
            PrintFormat::Md => print::Format::Md,
        }
    }
}

/// 引数の定義(CLI-2・CLI-13)。読み取り・補完・man はこの1つから作る。
/// `--help` は clap の英語の案内でなく USAGE を出すので、clap の help と version の旗は切る。
#[derive(Debug, Parser)]
#[command(
    name = "mdgrid",
    bin_name = "mdgrid",
    version,
    about = "Markdown のフロントマターを端末の表で見て直す",
    long_about = "フォルダの中の Markdown のノートを行、フロントマターのキーを列にした表を開く。\n.base を渡すと、その table ビューで開く(ノートは保管庫の根の下から探す)。\n.md のファイルを渡すと、そのフォルダを開いてその行を選ぶ。\n.csv・.tsv のファイルを渡すと、1行目を列の名前にした表として開く。\n引数が無ければ今のフォルダを開く(places.toml に登録した表があれば、その一覧を重ねる)。",
    disable_help_flag = true,
    disable_version_flag = true,
    args_override_self = true
)]
struct Cli {
    /// 開く .base(1つ)かフォルダ(1つ以上)か .md のファイル(そのフォルダを開いてその行を選ぶ)。無ければ今のフォルダ
    #[arg(value_name = Msg::ValuePath.ja(), value_hint = ValueHint::AnyPath)]
    paths: Vec<PathBuf>,
    /// .base のそのビューで開く(無ければ先頭のビュー。--print では mdgrid のビューも探す)
    #[arg(long, value_name = Msg::ValueName.ja(), allow_hyphen_values = true)]
    view: Option<String>,
    /// このワークスペースで開く(パスが無ければその最初の表。WS-3)
    #[arg(short = 'w', long, value_name = Msg::ValueName.ja(), allow_hyphen_values = true)]
    workspace: Option<String>,
    /// 読むだけで開く(編集も保存もしない)
    #[arg(long)]
    readonly: bool,
    /// 色なしで出す
    #[arg(long)]
    no_color: bool,
    /// この設定ファイルを読む(既定は $XDG_CONFIG_HOME/mdgrid/config.toml)
    #[arg(long, value_name = Msg::ValuePath.ja(), allow_hyphen_values = true, value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,
    /// 画面を出さずに、ビューの表を標準出力に出す(ノートは書き換えない)
    #[arg(long)]
    print: bool,
    /// --print の形(既定は csv)
    #[arg(long, value_name = Msg::ValueFormat.ja(), value_enum, requires = "print")]
    format: Option<PrintFormat>,
    /// --print の表の先頭に各行のノートのパスの列(path)を足す
    #[arg(long, requires = "print")]
    with_path: bool,
    /// .base の式が真の行だけを --print に出す(何度でも。.base のビューでは、その絞り込みと両方)
    #[arg(long, value_name = Msg::ValueExpr.ja(), requires = "print", allow_hyphen_values = true)]
    filter: Vec<String>,
    /// --print をこの列で並べる(列[:asc|:desc]。何度でも。ビューの並べ替えの代わり)
    #[arg(long, value_name = Msg::ValueSort.ja(), requires = "print", allow_hyphen_values = true)]
    sort: Vec<String>,
    /// --print の形の CSV・JSON(- で標準入力)の値をノートに当てる(既定は差分だけ。--yes で書く)
    #[arg(long, value_name = Msg::ValuePath.ja(), conflicts_with_all = ["print", "pick"], allow_hyphen_values = true, value_hint = ValueHint::FilePath)]
    apply: Option<PathBuf>,
    /// --apply で、差分を出すだけでなく書く
    #[arg(long, requires = "apply")]
    yes: bool,
    /// 画面(端末)で選んだ行のパスか列の値を標準出力に出して終わる
    #[arg(long, value_name = Msg::ValuePick.ja(), conflicts_with = "print")]
    pick: Option<String>,
    /// ワークスペース(workspaces.toml)の一覧を出す
    #[arg(long, conflicts_with_all = ["print", "pick", "apply"])]
    workspaces: bool,
    /// 開くパスの表を、このワークスペースに足す(無ければ作る)
    #[arg(long, value_name = Msg::ValueName.ja(), allow_hyphen_values = true, conflicts_with_all = ["print", "pick", "apply", "workspaces"])]
    add_to: Option<String>,
    /// --add-to で足す表の名前(無ければフォルダの名前)
    #[arg(long = "as", value_name = Msg::ValueName.ja(), allow_hyphen_values = true, requires = "add_to")]
    as_name: Option<String>,
    /// 開くパスの表を、このワークスペースから外す
    #[arg(long, value_name = Msg::ValueName.ja(), allow_hyphen_values = true, conflicts_with_all = ["print", "pick", "apply", "workspaces", "add_to"])]
    remove_from: Option<String>,
    /// このワークスペースを消す(表のノートには触らない)
    #[arg(long, value_name = Msg::ValueName.ja(), allow_hyphen_values = true, conflicts_with_all = ["print", "pick", "apply", "workspaces", "add_to", "remove_from"])]
    remove_workspace: Option<String>,
    /// 開くパスのフォルダ(無ければ今のフォルダ)に、ワークスペースの印 .mdgrid/workspace.toml を作る
    #[arg(long, conflicts_with_all = ["print", "pick", "apply", "workspaces", "add_to", "remove_from", "remove_workspace"])]
    init_workspace: bool,
    /// 設定の全項目を既定値と説明付きの TOML で出す
    #[arg(long)]
    print_config: bool,
    /// --print-config で、その表で効く設定を出どころのコメント付きで出す
    #[arg(long, requires = "print_config")]
    resolved: bool,
    /// 設定ファイルを今の形に写した TOML を出す(ファイルは書かない)
    #[arg(long, conflicts_with = "print_config")]
    migrate_config: bool,
    /// シェルの補完の定義を出す
    #[arg(long, value_name = Msg::ValueShell.ja())]
    completions: Option<Shell>,
    /// man ページ(roff)を出す
    #[arg(long)]
    man: bool,
    /// 使い方を出す
    #[arg(short = 'h', long)]
    help: bool,
    /// 版を出す
    #[arg(short = 'V', long)]
    version: bool,
}

/// 引数を clap で読む。読めなければ理由を1行で返す(CLI-4)。
fn parse_cli(args: &[OsString]) -> Result<Cli, String> {
    Cli::try_parse_from(std::iter::once(OsString::from("mdgrid")).chain(args.iter().cloned()))
        .map_err(|e| clap_reason(&e, args))
}

/// clap の値の名前(`Cli` の `value_name`。文は文言の表)。
const VALUE_NAMES: [Msg; 7] = [
    Msg::ValuePath,
    Msg::ValueName,
    Msg::ValueFormat,
    Msg::ValuePick,
    Msg::ValueShell,
    Msg::ValueExpr,
    Msg::ValueSort,
];

/// clap が誤りに入れる値の名前(`<パス>`・`[パス]` など、日本語)を、今の言語の名前にする(SR-23)。
fn value_names(s: &str) -> String {
    let mut out = s.to_string();
    if i18n::current() == Lang::Ja {
        return out;
    }
    for m in VALUE_NAMES {
        for (open, close) in [('<', '>'), ('[', ']')] {
            out = out.replace(
                &format!("{open}{}{close}", m.ja()),
                &format!("{open}{}{close}", m.text()),
            );
        }
    }
    out
}

/// clap の誤り(英語の複数行)を、今の言語の理由1行に直す(CLI-4・SR-23)。
/// `args` は渡された引数(`--print` が要る旗のどれが渡されたかを名指すのに使う)。
fn clap_reason(e: &clap::Error, args: &[OsString]) -> String {
    let text = |k| match e.get(k) {
        Some(ContextValue::String(s)) => Some(s.clone()),
        _ => None,
    };
    let arg = text(ContextKind::InvalidArg).unwrap_or_default();
    let sep = Msg::ListSep.text();
    match e.kind() {
        ErrorKind::UnknownArgument => Msg::CliUnknownOption.fill(&[&value_names(&arg)]),
        ErrorKind::InvalidValue => {
            // InvalidArg は `--config <パス>` の形なので、オプションの名前だけにする。
            let opt = arg.split([' ', '=']).next().unwrap_or_default();
            match text(ContextKind::InvalidValue).filter(|v| !v.is_empty()) {
                None => {
                    let what = match opt {
                        "--view" => Msg::CliWhatView,
                        "--config" => Msg::CliWhatConfig,
                        "--completions" => Msg::CliWhatShell,
                        "--format" => Msg::CliWhatFormat,
                        "--pick" => Msg::CliWhatPick,
                        _ => Msg::CliWhatValue,
                    };
                    Msg::CliMissingValue.fill(&[&value_names(opt), &what.text()])
                }
                Some(v) => {
                    let valid = match e.get(ContextKind::ValidValue) {
                        Some(ContextValue::Strings(s)) => s.join(sep),
                        _ => String::new(),
                    };
                    Msg::CliUnknownValue.fill(&[&value_names(opt), &v, &valid])
                }
            }
        }
        ErrorKind::InvalidUtf8 => Msg::CliInvalidUtf8.text().into(),
        // `--pick` と `--print` を一緒に渡したとき。
        ErrorKind::ArgumentConflict => {
            let opt = |s: &str| s.split([' ', '=']).next().unwrap_or_default().to_string();
            let prior = match e.get(ContextKind::PriorArg) {
                Some(ContextValue::String(s)) => opt(s),
                Some(ContextValue::Strings(s)) => {
                    s.iter().map(|x| opt(x)).collect::<Vec<_>>().join(sep)
                }
                _ => String::new(),
            };
            Msg::CliConflict.fill(&[&opt(&arg), &prior])
        }
        // `--format`・`--with-path` だけで `--print` が無いとき。
        ErrorKind::MissingRequiredArgument => {
            let missing = match e.get(ContextKind::InvalidArg) {
                Some(ContextValue::Strings(s)) => s.join(sep),
                _ => String::new(),
            };
            match needs_print(args) {
                Some(by) => Msg::CliRequiredBy.fill(&[&by, &value_names(&missing)]),
                None => Msg::CliRequired.fill(&[&value_names(&missing)]),
            }
        }
        // `--readonly=1` など。clap の英語の文は使わず、どの引数かだけを出す。
        _ if !arg.is_empty() => match text(ContextKind::InvalidValue) {
            Some(v) => Msg::CliTakesNoValue.fill(&[&value_names(&arg), &v]),
            None => Msg::CliBadArgument.fill(&[&value_names(&arg)]),
        },
        _ => {
            let s = e.to_string();
            let line = s.lines().next().unwrap_or_default();
            Msg::CliUnreadable.fill(&[&value_names(line.trim_start_matches("error: "))])
        }
    }
}

/// `--print` が要る旗(`requires = "print"`)のうち、渡された最初のもの。`--` のあとは見ない。
fn needs_print(args: &[OsString]) -> Option<&'static str> {
    for a in args {
        let a = a.to_string_lossy();
        if a == "--" {
            break;
        }
        let name = a.split('=').next().unwrap_or_default();
        if let Some(by) = ["--format", "--with-path", "--filter", "--sort"]
            .into_iter()
            .find(|f| *f == name)
        {
            return Some(by);
        }
    }
    None
}

impl Cli {
    /// 命令を決める。旗が重なったら help・version・print-config・completions・man の順に先のもの。
    fn into_command(self, here: PathBuf) -> Command {
        if self.help {
            Command::Help
        } else if self.version {
            Command::Version
        } else if self.print_config && self.resolved {
            Command::PrintResolved(if self.paths.is_empty() {
                vec![here]
            } else {
                self.paths
            })
        } else if self.print_config {
            Command::PrintConfig
        } else if self.migrate_config {
            Command::MigrateConfig
        } else if let Some(s) = self.completions {
            Command::Completions(s)
        } else if self.man {
            Command::Man
        } else if self.workspaces {
            Command::Workspace(ws_cli::Op::List)
        } else if let Some(name) = self.remove_workspace {
            Command::Workspace(ws_cli::Op::Delete(name))
        } else {
            let paths = if self.paths.is_empty() {
                vec![here]
            } else {
                self.paths
            };
            if let Some(name) = self.add_to {
                Command::Workspace(ws_cli::Op::Add(name, paths, self.as_name))
            } else if let Some(name) = self.remove_from {
                Command::Workspace(ws_cli::Op::Remove(name, paths))
            } else if self.init_workspace {
                Command::Workspace(ws_cli::Op::Init(paths))
            } else if let Some(file) = self.apply {
                Command::Apply(paths, file, self.yes)
            } else if self.print {
                Command::Print(
                    paths,
                    self.format.unwrap_or(PrintFormat::Csv),
                    self.with_path,
                    self.filter,
                    self.sort,
                )
            } else if let Some(what) = self.pick {
                Command::Pick(paths, Pick::of(what))
            } else {
                Command::Run(paths)
            }
        }
    }

    /// CLI-2 のオプションを除いた残りの引数(命令の旗とパス)。`-` で始まるパスからは前に `--` を置く。
    fn rest(&self) -> Vec<OsString> {
        let mut out = Vec::new();
        for (on, flag) in [
            (self.help, "--help"),
            (self.version, "--version"),
            (self.print_config, "--print-config"),
            (self.resolved, "--resolved"),
            (self.migrate_config, "--migrate-config"),
            (self.man, "--man"),
            (self.print, "--print"),
            (self.with_path, "--with-path"),
            (self.workspaces, "--workspaces"),
            (self.init_workspace, "--init-workspace"),
        ] {
            if on {
                out.push(OsString::from(flag));
            }
        }
        if let Some(f) = self.format.and_then(|f| f.to_possible_value()) {
            out.push("--format".into());
            out.push(f.get_name().into());
        }
        if let Some(what) = &self.pick {
            out.push("--pick".into());
            out.push(what.into());
        }
        for f in &self.filter {
            out.push(format!("--filter={f}").into());
        }
        if let Some(f) = &self.apply {
            out.push("--apply".into());
            out.push(f.clone().into_os_string());
        }
        if self.yes {
            out.push("--yes".into());
        }
        for (flag, v) in [
            ("--add-to", &self.add_to),
            ("--as", &self.as_name),
            ("--remove-from", &self.remove_from),
            ("--remove-workspace", &self.remove_workspace),
        ] {
            if let Some(v) = v {
                out.push(format!("{flag}={v}").into());
            }
        }
        for s in &self.sort {
            out.push(format!("--sort={s}").into());
        }
        if let Some(s) = self.completions.and_then(|s| s.to_possible_value()) {
            out.push("--completions".into());
            out.push(s.get_name().into());
        }
        let mut only_paths = false;
        for p in &self.paths {
            let s = p.as_os_str();
            if !only_paths && s.len() > 1 && s.to_string_lossy().starts_with('-') {
                out.push("--".into());
                only_paths = true;
            }
            out.push(s.to_os_string());
        }
        out
    }
}

/// 引数を読む(CLI-1)。読めなければ理由を1行で返す(CLI-4)。パスが無ければ今のフォルダ(`.`)。
fn parse_args(args: &[OsString]) -> Result<Command, String> {
    parse_args_in(args, PathBuf::from("."))
}

/// `parse_args` の本体。パスが無いときに開くフォルダ `here` を差し替えられる(試験では一時フォルダ)。
fn parse_args_in(args: &[OsString], here: PathBuf) -> Result<Command, String> {
    Ok(parse_cli(args)?.into_command(here))
}

/// CLI-2 のオプション。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Options {
    view: Option<String>,
    readonly: bool,
    no_color: bool,
    config: Option<PathBuf>,
    workspace: Option<String>,
}

/// オプションを引数から取り出す(CLI-2)。残りは命令の旗と、`--` の後ろに並べたパス(`parse_args` が読む)。
/// `--view=<名前>` の形も受け、`--` の後ろはパスのまま。
fn take_options(args: &mut Vec<OsString>) -> Result<Options, String> {
    let cli = parse_cli(args)?;
    *args = cli.rest();
    Ok(Options {
        view: cli.view,
        readonly: cli.readonly,
        no_color: cli.no_color,
        config: cli.config,
        workspace: cli.workspace,
    })
}

/// 補完の定義(CLI-13)。引数の定義(`Cli`)から作る。
fn completions(shell: Shell) -> Vec<u8> {
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "mdgrid", &mut buf);
    buf
}

/// man ページ(CLI-13)。引数の定義(`Cli`)から作る。
fn man_page() -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    clap_mangen::Man::new(Cli::command()).render(&mut buf)?;
    Ok(buf)
}

/// 作ったものを標準出力に出す(CLI-13)。
fn emit(out: io::Result<Vec<u8>>) -> ExitCode {
    let mut stdout = io::stdout();
    match out.and_then(|b| stdout.write_all(&b).and_then(|()| stdout.flush())) {
        Ok(()) => ExitCode::SUCCESS,
        // `| head` などで読み手が先に閉じたときは、黙って終わる。
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => fail(&Msg::CannotWriteOutput.fill(&[&e])),
    }
}

/// 設定を読む(CLI-3)。`--config` のパスは無ければ理由1行、既定の置き場は無ければ既定。
/// 壊れた TOML は理由1行(終了コード 2 は呼ぶ側の fail)。知らない項目は警告にして返す。
fn load_config(
    explicit: Option<&Path>,
    default: Option<PathBuf>,
) -> Result<(Config, Vec<String>), String> {
    let path = match explicit {
        Some(p) => p.to_path_buf(),
        None => match default {
            Some(p) if p.exists() => p,
            _ => return Ok((Config::default(), Vec::new())),
        },
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Msg::ConfigUnreadable.fill(&[&path.display(), &e]))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| config::CONFIG_FILE.to_string());
    config::parse_named(&text, &name).map_err(|e| format!("{}: {}", path.display(), e))
}

/// 設定ファイルの文(CLI-20 の `--migrate-config`)。`--config` のパスか既定の置き場。無ければ空。
fn config_text(explicit: Option<&Path>, default: Option<PathBuf>) -> Result<String, String> {
    let path = match explicit {
        Some(p) => p.to_path_buf(),
        None => match default {
            Some(p) if p.exists() => p,
            _ => return Ok(String::new()),
        },
    };
    std::fs::read_to_string(&path).map_err(|e| Msg::ConfigUnreadable.fill(&[&path.display(), &e]))
}

/// 画面を出さない命令(`--apply`・`--print-config --resolved`)のための、開く表で効く設定(SR-44): config.toml・
/// ui.toml・ワークスペース・表・(`--view` があれば)ビューの層を重ねる。警告も返す。
fn resolve_table(
    config: &Config,
    paths: &[PathBuf],
    opts: &Options,
) -> (mdgrid::profile::Resolved, Vec<String>) {
    use mdgrid::profile::{Layer, Origin, Place};
    let mut warnings = Vec::new();
    let mut layers = vec![config.layer()];
    let mut templates = config.templates.clone();
    let dir = config_dir();
    if let Some(d) = &dir {
        let (ui, w) = mdgrid::uifile::load(d);
        warnings.extend(w);
        if !ui.profile.is_empty() {
            layers.push(ui.layer());
        }
        templates.extend(ui.templates);
    }
    let target = state_target(paths);
    let first = paths.first().cloned().unwrap_or_else(|| target.clone());
    let apps = dir
        .as_deref()
        .map(|d| mdgrid::workspace::load(d).0)
        .unwrap_or_default();
    if let Ok(Some(scope)) = mdgrid::workspace::resolve(
        &first,
        opts.workspace.as_deref(),
        &apps,
        &config.workspace_detect,
    ) {
        layers.extend(scope.layer());
        layers.extend(scope.table_layer(&first));
    }
    if let Some(d) = &dir {
        let (p, w) = mdgrid::views::load_table_profile(d, &target);
        warnings.extend(w);
        if !p.is_empty() {
            let name = mdgrid::workspace::stem_of(&first);
            layers.push(Layer {
                origin: Origin::new(
                    Place::TableApp,
                    format!("{} ({name})", mdgrid::views::FILE_NAME),
                ),
                profile: p,
            });
        }
    }
    if let (Some(view), Some(state)) = (&opts.view, config::state_dir()) {
        let mut s = config::load_state(&state, &target, view).settings;
        if s.is_default() {
            if let Some(d) = &dir {
                let (views, _) = mdgrid::views::load_views(d, &target);
                if let Some(nv) = views.into_iter().find(|v| &v.name == view) {
                    s = nv.settings;
                }
            }
        }
        let p = s.profile();
        if !p.is_empty() {
            layers.push(Layer {
                origin: Origin::new(Place::View, format!("view {view}")),
                profile: p,
            });
        }
    }
    let r = mdgrid::profile::resolve(&layers, &templates, &mut warnings);
    (r, warnings)
}

/// 見た目の状態の対象(SR-12): `.base` ならそのパス、フォルダ1つならそのパス、
/// 複数なら実体のパスを改行でつないだもの(並びも含めて同じ組み合わせだけが同じ状態を引く)。
fn state_target(paths: &[PathBuf]) -> PathBuf {
    if let Some(b) = paths.iter().find(|p| is_base(p)) {
        return b.clone();
    }
    if paths.len() == 1 {
        return paths[0].clone();
    }
    let parts: Vec<String> = paths
        .iter()
        .map(|p| {
            std::fs::canonicalize(p)
                .unwrap_or_else(|_| p.clone())
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    PathBuf::from(parts.join("\n"))
}

/// CLI-15: 起動の引数のうち、在る `.md` のファイル(拡張子の大文字小文字を問わない)を、そのファイルの
/// あるフォルダ(親が空なら `.`)に置き換える。置き換えたノートの最初の1つを返す(画面で選ぶ)。
/// 無い `.md` はそのまま残し、`check_paths` が理由1行にする(CLI-4)。`--pick path` の形(OUT-3)は
/// 置き換えたフォルダを起動の引数とみなす。
fn md_to_folder(paths: Vec<PathBuf>) -> (Vec<PathBuf>, Option<PathBuf>) {
    let mut first = None;
    let out = paths
        .into_iter()
        .map(|p| {
            let is_md = p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"));
            if !is_md || !p.is_file() {
                return p;
            }
            let dir = match p.parent() {
                Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
                _ => PathBuf::from("."),
            };
            first.get_or_insert(p);
            dir
        })
        .collect();
    (out, first)
}

fn is_base(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "base")
}

/// SC-15: CSV・TSV のファイル(拡張子の大文字小文字を問わない)。
fn is_csv(p: &Path) -> bool {
    Csv::handles(p)
}

/// 開けるパスか確かめる(CLI-4)。フォルダの並びか、`.base` 1つ(フォルダと並べない)。
fn check_paths(paths: &[PathBuf]) -> Result<(), String> {
    for p in paths {
        let meta =
            std::fs::metadata(p).map_err(|e| Msg::PathUnopenable.fill(&[&p.display(), &e]))?;
        if meta.is_dir() {
            continue;
        }
        if is_base(p) {
            if paths.len() > 1 {
                return Err(Msg::BaseAlone.fill(&[&p.display()]));
            }
            continue;
        }
        if is_csv(p) {
            if paths.len() > 1 {
                return Err(Msg::CsvAlone.fill(&[&p.display()]));
            }
            continue;
        }
        return Err(Msg::NotAFolder.fill(&[&p.display()]));
    }
    Ok(())
}

/// 開く対象: 読み込み口と、`.base` なら (定義, ヘッダーの名前, `--view` で選んだビューの添字)。
struct Target {
    src: Box<dyn Source>,
    base: Option<(Base, String, Option<usize>)>,
    /// BV-2: `.base` の上に `.obsidian/` が無いとき、推した根(`.base` のフォルダ)。
    guessed_root: Option<PathBuf>,
}

/// 対象を開く(CLI-1・CLI-2・CLI-4)。`.base` が読めない・`--view` のビューが無いときは理由1行。
fn open_target(paths: &[PathBuf], view: Option<&str>) -> Result<Target, String> {
    let base_path = paths.iter().find(|p| is_base(p));
    let Some(bp) = base_path else {
        if let Some(v) = view {
            return Err(Msg::ViewNeedsBase.fill(&[&v]));
        }
        // SC-15: CSV・TSV は1つのファイルを表にする。
        let src: Box<dyn Source> = match paths {
            [p] if is_csv(p) => Box::new(Csv::open(p).map_err(|e| Msg::CannotOpen.fill(&[&e]))?),
            _ => Box::new(Markdown::open(paths).map_err(|e| Msg::CannotOpen.fill(&[&e]))?),
        };
        return Ok(Target {
            src,
            base: None,
            guessed_root: None,
        });
    };
    let text =
        std::fs::read_to_string(bp).map_err(|e| Msg::BaseUnreadable.fill(&[&bp.display(), &e]))?;
    let base =
        base::parse(&text).map_err(|e| format!("{}: {}", bp.display(), e.replace('\n', " ")))?;
    let idx = match view {
        None => None,
        Some(v) => Some(base.views.iter().position(|x| x.name == v).ok_or_else(|| {
            let names: Vec<&str> = base.views.iter().map(|x| x.name.as_str()).collect();
            let have = Msg::ViewsAre.fill(&[&names.join(", ")]);
            Msg::NoSuchView.fill(&[&v, &have])
        })?),
    };
    let src = Markdown::open_vault(bp).map_err(|e| Msg::CannotOpen.fill(&[&e]))?;
    let guessed_root = match src.roots() {
        [r] if !r.ancestors().any(|a| a.join(".obsidian").is_dir()) => Some(r.clone()),
        _ => None,
    };
    // BV-22: `.base` を直接開いたときの this はその `.base` のファイル。
    let mut base = base;
    base.set_this(src.this_file(bp));
    let name = bp
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| bp.display().to_string());
    Ok(Target {
        src: Box::new(src),
        base: Some((base, name, idx)),
        guessed_root,
    })
}

/// 画面を出せるか(SR-10): 標準の入出力が端末でなければ理由を返す。
fn check_tty(stdin: bool, stdout: bool) -> Result<(), String> {
    if stdin && stdout {
        Ok(())
    } else {
        Err(Msg::NotATerminal.text().into())
    }
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("mdgrid: {msg}");
    ExitCode::from(2)
}

/// 文言の言語(SR-23)を決める: 設定の `language`(`--config` のファイル、無ければ既定の置き場)が
/// en・ja ならそれ、auto(と、設定が無い・読めない)なら環境変数。誤りと警告は `load_config` が出す。
fn language(explicit: Option<&Path>, default: Option<PathBuf>) -> Lang {
    let path = explicit.map(Path::to_path_buf).or(default);
    let setting = path
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map_or(Language::Auto, |t| config::peek_language(&t));
    i18n::resolve(setting, |k| std::env::var(k).ok())
}

fn main() -> ExitCode {
    // BV-6・CE-21: 今日と今は地域の時刻。時差は糸がまだ1本のうちに決める(time はそうでないと断る)。
    print::set_local_offset(
        time::UtcOffset::current_local_offset()
            .map(|o| o.whole_seconds() as i64)
            .unwrap_or(0),
    );
    // SR-23: 引数の誤りは設定を読む前に出るので、まず環境から決める。
    i18n::set_default(i18n::resolve(Language::Auto, |k| std::env::var(k).ok()));
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let opts = match take_options(&mut args) {
        Ok(o) => o,
        Err(e) => return fail(&e),
    };
    // SR-23: 設定の language で決め直す(--help と起動できない理由もこの言語)。
    i18n::set_default(language(opts.config.as_deref(), config::config_path()));
    let no_args = args.is_empty();
    let (paths, pick) = match parse_args(&args) {
        Ok(Command::Help) => {
            println!("{}", usage());
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("mdgrid {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(Command::PrintConfig) => return emit(Ok(config::default_toml().into_bytes())),
        // CLI-21: その表で効く設定(画面は出さない)。
        Ok(Command::PrintResolved(paths)) => {
            let (config, mut warnings) =
                match load_config(opts.config.as_deref(), config::config_path()) {
                    Ok(c) => c,
                    Err(e) => return fail(&e),
                };
            let (r, w) = resolve_table(&config, &paths, &opts);
            warnings.extend(w);
            for w in &warnings {
                eprintln!("mdgrid: {w}");
            }
            let mut out = config::app_toml(&config);
            out.push_str(&mdgrid::profile::resolved_toml(&r));
            return emit(Ok(out.into_bytes()));
        }
        // CLI-20: 今の形に写した設定(ファイルは書かない)。
        Ok(Command::MigrateConfig) => {
            return match config_text(opts.config.as_deref(), config::config_path())
                .and_then(|t| config::migrate(&t))
            {
                Ok(out) => emit(Ok(out.into_bytes())),
                Err(e) => fail(&e),
            }
        }
        // CLI-13・SR-10: 画面を出さないので、標準出力がパイプでも出す。
        Ok(Command::Completions(shell)) => return emit(Ok(completions(shell))),
        Ok(Command::Man) => return emit(man_page()),
        // WS-3・SR-10: 画面を出さないので、標準出力がパイプでも出す。
        Ok(Command::Workspace(op)) => {
            return match ws_cli::run(op, config_dir().as_deref()) {
                Ok(out) => emit(Ok(out.into_bytes())),
                Err(e) => fail(&e),
            }
        }
        // CLI-5・SR-10: 画面を出さないので、端末かどうかを確かめる前に分ける。
        Ok(Command::Print(p, f, w, filter, sort)) => {
            // CLI-15: 渡した `.md` のノートの行だけを出す(フォルダの表と同じ列)。
            let only: Vec<PathBuf> = p
                .iter()
                .filter(|x| {
                    x.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) && x.is_file()
                })
                .filter_map(|x| x.canonicalize().ok())
                .collect();
            // WS-3: `-w` は `--print` でも名前を確かめ、パスが無ければその最初の表を出す。
            let p = match &opts.workspace {
                Some(name) => match workspace_paths(name, p, no_args) {
                    Ok(p) => p,
                    Err(e) => return fail(&e),
                },
                None => p,
            };
            return print_view(&md_to_folder(p).0, &opts, f.lib(), w, &filter, &sort, &only);
        }
        Ok(Command::Apply(p, file, yes)) => {
            return apply_file(&md_to_folder(p).0, &opts, &file, yes)
        }
        Ok(Command::Pick(p, what)) => match &opts.workspace {
            // WS-3: 無い名前は画面を出さずに理由(パスはそのまま)。
            Some(name) => match workspace_paths(name, p, false) {
                Ok(p) => (p, Some(what)),
                Err(e) => return fail(&e),
            },
            None => (p, Some(what)),
        },
        Ok(Command::Run(p)) => match &opts.workspace {
            // WS-3: 無い名前は画面を出さずに理由。パスが無ければその最初の表。
            Some(name) => match workspace_paths(name, p, no_args) {
                Ok(p) => (p, None),
                Err(e) => return fail(&e),
            },
            None => (p, None),
        },
        Err(e) => return fail(&e),
    };
    // CLI-15: `.md` のファイルはそのフォルダに置き換え、読み終えたらその行を選ぶ。
    let (paths, select) = md_to_folder(paths);
    if let Err(e) = check_paths(&paths) {
        return fail(&e);
    }
    // OUT-3: `--pick` は読むだけ(WB-15)で開く。
    let (mut app, editor) = match open_app(&paths, opts.view.as_deref(), &opts, pick.is_some()) {
        Ok(a) => a,
        Err(e) => return fail(&e),
    };
    app.select_after_load = select;
    // SR-10: `--pick` は画面を端末(`/dev/tty`)に出すので、標準出力は見ない。
    let tty = match pick {
        Some(_) => check_dev_tty(),
        None => check_tty(io::stdin().is_terminal(), io::stdout().is_terminal()),
    };
    if let Err(e) = tty {
        return fail(&e);
    }
    let Some(what) = pick else {
        // CLI-1・CLI-19: 引数なしで起動し、登録した表があれば一覧を重ねて始める。
        if no_args && opts.workspace.is_none() && !app.registered.is_empty() {
            app.start_open_places(true);
        }
        return run_switching(app, editor, &opts);
    };
    // OUT-3: 列の名前は、ノートを読み終えてから表の列で確かめる(無ければ画面を出さずに理由1行)。
    let column = match what {
        Pick::Path => None,
        Pick::Column(name) => {
            while !app.loaded() {
                app.load_step(LOAD_BUDGET);
            }
            match app.find_column(&name) {
                Some(c) => Some(c),
                None => {
                    return fail(&Msg::PickNoColumn.fill(&[&name, &app.column_names().join(", ")]))
                }
            }
        }
    };
    app.start_choosing();
    app.copy_to_tty();
    if let Err(e) = run(&mut app, &editor, Screen::Tty) {
        return fail(&Msg::TerminalError.fill(&[&e]));
    }
    // 画面を戻したあとで、結果を標準出力へ。取りやめは何も出さず終了コード 1。
    let Some(rows) = app.chosen() else {
        return ExitCode::from(1);
    };
    let mut out = String::new();
    for row in rows {
        let line = match &column {
            Some(c) => app.pick_text(row, c),
            None => row_path(&paths, app.src.as_ref(), row),
        };
        out.push_str(&line);
        out.push('\n');
    }
    emit(Ok(out.into_bytes()))
}

/// `-w <名前>`(WS-3): 名前を workspaces.toml で確かめ、パスが無ければその最初の表を開く。
fn workspace_paths(
    name: &str,
    paths: Vec<PathBuf>,
    no_paths: bool,
) -> Result<Vec<PathBuf>, String> {
    let list = config_dir()
        .map(|d| mdgrid::workspace::load(&d).0)
        .unwrap_or_default();
    let w = list
        .iter()
        .find(|w| w.name == name)
        .ok_or_else(|| Msg::WsUnknown.fill(&[&name]))?;
    if !no_paths {
        return Ok(paths);
    }
    let first = w
        .tables
        .first()
        .ok_or_else(|| Msg::WsEmpty.fill(&[&name]))?;
    Ok(vec![first.path.clone()])
}

/// 表を開いて画面の App を作る(CLI-1・CLI-2・CLI-3): 対象・設定・起動の設定。エディタの名前も返す。
fn open_app(
    paths: &[PathBuf],
    view: Option<&str>,
    opts: &Options,
    pick: bool,
) -> Result<(App, String), String> {
    let target = open_target(paths, view)?;
    let (config, mut warnings) = load_config(opts.config.as_deref(), config::config_path())?;
    // SR-43・CLI-3: 画面で選んだ全体の設定(ui.toml。無ければ前の版の look.toml)を config.toml の上に重ねる。
    let ui = match config_dir() {
        Some(dir) => {
            let (ui, warns) = mdgrid::uifile::load(&dir);
            warnings.extend(warns);
            ui
        }
        None => Default::default(),
    };
    // SR-8: 設定の editor > $VISUAL > $EDITOR > vi。
    let editor = config::resolve_editor(
        config.editor.as_deref(),
        std::env::var("VISUAL").ok().as_deref(),
        std::env::var("EDITOR").ok().as_deref(),
    );
    let color = ColorMode::detect(|k| std::env::var(k).ok());
    let mut app = App::new(target.src, color);
    app.prof.ui = ui;
    // SR-36: nerd_font = "auto" は端末の名前で決める(丸い端を自分で描く端末だけ)。
    app.prof.term_program = std::env::var("TERM_PROGRAM").ok();
    // SR-39: 明暗の組のテーマは端末の地の明るさで選ぶ(色を使わない表示では問い合わせない。要るときに1回)。
    if config.terminal.color && !opts.no_color && color != ColorMode::None {
        app.prof.probe = Some(Box::new(|| {
            ui::termbg::light(
                |k| std::env::var(k).ok(),
                std::time::Duration::from_millis(200),
            )
        }));
    }
    app.no_emoji = ui::dumb_terminal(|k| std::env::var(k).ok());
    // WS-6: -w で選んだワークスペースは範囲を決める前に渡す。
    app.workspace_choice = opts.workspace.clone();
    app.start(Startup {
        config,
        warnings,
        readonly: opts.readonly || pick,
        no_color: opts.no_color,
        state_dir: config::state_dir(),
        // BV-17・BV-20: mdgrid のビュー(views.toml)は設定の置き場(config.toml と同じフォルダ)。
        config_dir: config_dir(),
        target: state_target(paths),
        base: target.base,
    });
    app.guessed_root = target.guessed_root;
    Ok((app, editor))
}

/// 画面を出す。一覧で別の表を選んで終わったら(CLI-19)、その表で開き直して続ける。開けなければ
/// 前の表に戻って理由を出す。
fn run_switching(mut app: App, mut editor: String, opts: &Options) -> ExitCode {
    let mut opts = opts.clone();
    loop {
        if let Err(e) = run(&mut app, &editor, Screen::Stdout) {
            return fail(&Msg::TerminalError.fill(&[&e]));
        }
        let Some(place) = app.switch_to.take() else {
            return ExitCode::SUCCESS;
        };
        // REL-4・REL-5: 開き直したあとに選ぶノート。
        let select = app.switch_select.take();
        // CE-25: 関係マップの「+ 新規」で移ったら、開いた表で名前の欄を出す。
        let new_note = std::mem::take(&mut app.switch_new_note);
        // WS-2: ワークスペースを開いて移ったら、以後の範囲はそのワークスペース。
        if let Some(ws) = app.switch_workspace.take() {
            opts.workspace = Some(ws);
        } else if std::mem::take(&mut app.switch_leave_workspace) {
            opts.workspace = None;
        }
        match open_app(std::slice::from_ref(&place.path), None, &opts, false) {
            Ok((next, ed)) => {
                app = next;
                editor = ed;
                app.select_after_load = select;
                app.new_note_after_load = new_note;
                if let Some(v) = &place.view {
                    app.select_view_named(v);
                }
            }
            Err(e) => {
                app.quit = false;
                app.set_mode(ui::keymap::Mode::Table);
                app.message = Some(e);
            }
        }
    }
}

/// `--pick path` の1行(OUT-3): ノートの実体のパスが起動の引数のフォルダ(`.base` ならそのフォルダ)を
/// 正規化した場所の下なら「引数の文字 + 相対」、どれの下でもなければ実体の絶対パス。`.csv`・`.tsv` もそのフォルダ。
pub(crate) fn pick_path(args: &[PathBuf], note: &Path) -> String {
    for arg in args {
        let dir = if is_base(arg) || is_csv(arg) {
            arg.parent().unwrap_or(Path::new(""))
        } else {
            arg.as_path()
        };
        let real = if dir.as_os_str().is_empty() {
            std::fs::canonicalize(".")
        } else {
            std::fs::canonicalize(dir)
        };
        if let Some(rel) = real
            .ok()
            .and_then(|r| note.strip_prefix(r).ok().map(Path::to_path_buf))
        {
            return slash(dir.join(rel).to_string_lossy().into_owned());
        }
    }
    slash(note.to_string_lossy().into_owned())
}

/// 行の `--pick path`・`--with-path` の文字: 行のあるファイルの `pick_path` に、ファイルの中の位置の印
/// (CSV の `#行の番号`。`Source::locate`)を付けたもの。
pub(crate) fn row_path(args: &[PathBuf], src: &dyn Source, row: &mdgrid::source::RowId) -> String {
    let (file, at) = src.locate(row);
    pick_path(args, &file) + &at
}

/// パスの区切りを `/` にする(Windows でも。CLI-14 の `--with-path` を OS をまたいで `--apply` で戻せるように)。
fn slash(p: String) -> String {
    if cfg!(windows) {
        p.replace('\\', "/")
    } else {
        p
    }
}

/// mdgrid のビュー(views.toml)の置き場。config.toml と同じフォルダ(BV-17・BV-20)。
fn config_dir() -> Option<PathBuf> {
    config::config_path().and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

/// `--print`(CLI-5): ノートを全部読んでから、ビューの表を標準出力に出す。画面の見た目の状態は使わず、
/// 何も書かない(WB-15)。警告(設定・views.toml・評価できない列)は標準エラーに1行ずつ。
/// `with_path`(CLI-14)なら、表の先頭に見出し `path` の列と各行の `--pick path` と同じ形のパスを足す。
fn print_view(
    paths: &[PathBuf],
    opts: &Options,
    format: print::Format,
    with_path: bool,
    filter: &[String],
    sort: &[String],
    only: &[PathBuf],
) -> ExitCode {
    // CLI-16: `列[:asc|:desc]` を先に読む(読めなければ理由1行)。
    let mut sorts = Vec::new();
    for s in sort {
        // 後ろの `:asc`・`:desc` だけを向きと読む(列の名前に `:` があってもよい)。
        let (col, dir) = match s.rsplit_once(':') {
            Some((c, "asc")) => (c, base::Dir::Asc),
            Some((c, "desc")) => (c, base::Dir::Desc),
            _ => (s.as_str(), base::Dir::Asc),
        };
        if col.trim().is_empty() {
            return fail(&Msg::CliBadSort.fill(&[s]));
        }
        sorts.push((col.trim().to_string(), dir));
    }
    let narrowing = !filter.is_empty() || !sorts.is_empty();
    if let Err(e) = check_paths(paths) {
        return fail(&e);
    }
    let target = match open_target(paths, None) {
        Ok(t) => t,
        Err(e) => return fail(&e),
    };
    let (_, mut warnings) = match load_config(opts.config.as_deref(), config::config_path()) {
        Ok(c) => c,
        Err(e) => return fail(&e),
    };
    let guessed_root = target.guessed_root;
    let base = target.base.map(|(b, _, _)| b);
    // `--view` は `.base` のビューを先に、無ければ今の対象の mdgrid のビューを名前で探す。
    let mut natives = Vec::new();
    let mut base_view = None;
    let mut native_view = None;
    if let Some(name) = opts.view.as_deref() {
        base_view = base
            .as_ref()
            .and_then(|b| b.views.iter().position(|v| v.name == name));
        if base_view.is_none() {
            if let Some(dir) = config_dir() {
                let (vs, warns) = views::load_views(&dir, &state_target(paths));
                warnings.extend(warns);
                natives = vs;
            }
            native_view = natives.iter().position(|v| v.name == name);
            if native_view.is_none() {
                let names: Vec<&str> = base
                    .iter()
                    .flat_map(|b| b.views.iter().map(|v| v.name.as_str()))
                    .chain(natives.iter().map(|v| v.name.as_str()))
                    .collect();
                let have = if names.is_empty() {
                    Msg::NoViews.text().to_string()
                } else {
                    Msg::ViewsAre.fill(&[&names.join(", ")])
                };
                return fail(&Msg::NoSuchView.fill(&[&name, &have]));
            }
        }
    }
    let mut src = target.src;
    while !src.load(LOAD_BUDGET).done {}
    // CLI-16: .base のビューには絞り込みと並べ替えを足し、フォルダは既定の表の列の .base にして足す。
    let mut base = base;
    if !sorts.is_empty() {
        // 知っている列: ビューの列・ノートのキー・file.*・.base の式。ほかは綴りの誤りとして止める。
        let mut known: std::collections::HashSet<String> = src.columns().into_iter().collect();
        if let Some(b) = &base {
            if let Some(v) = b.views.get(base_view.unwrap_or(0)) {
                known.extend(v.order.iter().cloned());
            }
            known.extend(
                b.formula_names()
                    .into_iter()
                    .map(|n| format!("formula.{n}")),
            );
        }
        for (c, _) in &sorts {
            let bare = c.strip_prefix("note.").unwrap_or(c);
            if !bare.starts_with("file.") && !known.contains(bare) {
                return fail(&Msg::CliUnknownSort.fill(&[c]));
            }
        }
    }
    if narrowing {
        if let Some(k) = native_view {
            return fail(&Msg::CliFilterNativeView.fill(&[&natives[k].name]));
        }
        let b = base.get_or_insert_with(|| base::Base::plain(src.columns()));
        b.narrow(base_view.unwrap_or(0), filter, &sorts);
    }
    let view = match (native_view, &base) {
        (Some(k), _) => print::ViewDef::Native(&natives[k]),
        (None, Some(b)) => print::ViewDef::Base(b, base_view.unwrap_or(0)),
        (None, None) => print::ViewDef::Default,
    };
    let (today, now) = print::today_now(|k| std::env::var(k).ok());
    let mut table = match print::table(src.as_ref(), view, today, now) {
        Ok(t) => t,
        Err(e) => return fail(&e.replace('\n', " ")),
    };
    if !only.is_empty() {
        let keep: Vec<bool> = table
            .row_ids
            .iter()
            .map(|id| only.iter().any(|o| Path::new(&id.0) == o))
            .collect();
        let mut k = keep.iter();
        table.rows.retain(|_| *k.next().unwrap_or(&false));
        let mut k = keep.iter();
        table.row_ids.retain(|_| *k.next().unwrap_or(&false));
    }
    if with_path {
        let column = base::Column {
            id: "path".into(),
            title: "path".into(),
        };
        table.columns.insert(0, column);
        for (cells, id) in table.rows.iter_mut().zip(&table.row_ids) {
            let p = row_path(paths, src.as_ref(), id);
            cells.insert(0, print::PrintCell::Prop(Some(Value::Str(p))));
        }
    }
    for w in warnings.iter().chain(&table.warnings) {
        eprintln!("mdgrid: {w}");
    }
    if let Some(root) = guessed_root.filter(|_| table.rows.is_empty()) {
        eprintln!("mdgrid: {}", Msg::VaultRootGuessed.fill(&[&root.display()]));
    }
    emit(Ok(print::render(&table, format).into_bytes()))
}

/// `--apply`(CLI-17): CSV・JSON を読み、全部を確かめてから、差分を出す(`--yes` なら書く)。
/// 理由が1つでもあれば何も書かずに終了コード 2。書けなかった行(外で変わった等)は終了コード 1。
fn apply_file(paths: &[PathBuf], opts: &Options, file: &Path, yes: bool) -> ExitCode {
    if opts.readonly {
        return fail(Msg::ApplyReadonly.text());
    }
    if let Err(e) = check_paths(paths) {
        return fail(&e);
    }
    let text = if file == Path::new("-") {
        let mut s = String::new();
        match io::Read::read_to_string(&mut io::stdin(), &mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(e),
        }
    } else {
        std::fs::read_to_string(file)
    };
    let text = match text {
        Ok(t) => t,
        Err(e) => return fail(&Msg::ApplyUnreadable.fill(&[&file.display(), &e])),
    };
    let input = match apply::read_input(&text) {
        Ok(r) => r,
        Err(e) => return fail(&e),
    };
    let target = match open_target(paths, opts.view.as_deref()) {
        Ok(t) => t,
        Err(e) => return fail(&e),
    };
    let (config, warnings) = match load_config(opts.config.as_deref(), config::config_path()) {
        Ok(c) => c,
        Err(e) => return fail(&e),
    };
    for w in &warnings {
        eprintln!("mdgrid: {w}");
    }
    let mut src = target.src;
    let (resolved, warns) = resolve_table(&config, paths, opts);
    for w in &warns {
        eprintln!("mdgrid: {w}");
    }
    src.set_add_frontmatter(resolved.add_frontmatter);
    while !src.load(LOAD_BUDGET).done {}
    let (today, now) = print::today_now(|k| std::env::var(k).ok());
    // 見出しの名前(表示名か id)から列の id(開いたビューの列。無ければノートのキー)。
    let view = match &target.base {
        Some((b, _, idx)) => print::ViewDef::Base(b, idx.unwrap_or(0)),
        None => print::ViewDef::Default,
    };
    let table = print::table(src.as_ref(), view, today, now).ok();
    let cols: Vec<(String, String)> = table
        .as_ref()
        .map(|t| {
            t.columns
                .iter()
                .map(|c| (c.id.clone(), c.title.clone()))
                .collect()
        })
        .unwrap_or_default();
    // 書けない列の今の表の文字(--apply が捨てる直しを数える)。
    let mut shown_map: std::collections::HashMap<(String, String), String> =
        std::collections::HashMap::new();
    if let Some(t) = &table {
        for (cells, id) in t.rows.iter().zip(&t.row_ids) {
            for (c, cell) in t.columns.iter().zip(cells) {
                if c.id.starts_with("file.") || c.id.starts_with("formula.") {
                    shown_map.insert((id.0.clone(), c.id.clone()), print::plain(cell));
                }
            }
        }
    }
    // 表に無い file.* の文字の列は、ノートのファイルの属性から(名前の変更の直しも数える)。
    let shown = |r: &mdgrid::source::RowId, c: &str| {
        shown_map
            .get(&(r.0.clone(), c.to_string()))
            .cloned()
            .or_else(|| {
                let f = src.file(r)?;
                Some(match c {
                    "file.name" => f.name,
                    "file.basename" => f.basename,
                    "file.ext" => f.ext,
                    "file.path" => f.path,
                    "file.folder" => f.folder,
                    _ => return None,
                })
            })
    };
    let ids = match apply::header_ids(&input.headers, &cols, &src.columns()) {
        Ok(ids) => ids,
        Err(problems) => {
            for p in &problems {
                eprintln!("mdgrid: {p}");
            }
            return fail(Msg::ApplyProblems.text());
        }
    };
    // path は --with-path と同じ形(起動の引数のフォルダからの相対か絶対。後ろに行の位置の印があれば
    // `Source::locate` の印)。実体のパスと印で行を引く。
    let rows: std::collections::HashMap<String, mdgrid::source::RowId> = src
        .rows()
        .into_iter()
        .map(|r| {
            let (file, at) = src.locate(&r);
            (file.to_string_lossy().into_owned() + &at, r)
        })
        .collect();
    let find = |p: &str| -> apply::Found {
        // 全体をパスとして、と、最後の `#` の後ろを位置の印として(ノートの名前に `#` があっても全体を先に試す)。
        let mut splits = vec![(p, "")];
        if let Some(k) = p.rfind('#') {
            splits.push((&p[..k], &p[k..]));
        }
        let mut hits: Vec<String> = Vec::new();
        for (file, at) in splits {
            let file = Path::new(file);
            let mut tries = vec![file.to_path_buf()];
            for arg in paths {
                let dir = if is_base(arg) || is_csv(arg) {
                    arg.parent().unwrap_or(Path::new("")).to_path_buf()
                } else {
                    arg.clone()
                };
                tries.push(dir.join(file));
            }
            hits.extend(
                tries
                    .into_iter()
                    .filter_map(|t| t.canonicalize().ok())
                    .map(|t| t.to_string_lossy().into_owned() + at)
                    .filter(|t| rows.contains_key(t)),
            );
        }
        hits.sort();
        hits.dedup();
        match hits.len() {
            0 => apply::Found::Missing,
            1 => apply::Found::Row(rows[&hits.remove(0)].clone()),
            _ => apply::Found::Ambiguous,
        }
    };
    // 行の名前(保存の間も引けるよう、先に全部の行で作っておく)。
    let labels: std::collections::HashMap<mdgrid::source::RowId, String> = src
        .rows()
        .into_iter()
        .map(|r| (r.clone(), row_path(paths, src.as_ref(), &r)))
        .collect();
    let label = |r: &mdgrid::source::RowId| {
        labels
            .get(r)
            .cloned()
            .unwrap_or_else(|| pick_path(paths, Path::new(&r.0)))
    };
    let mut plan = apply::plan(
        src.as_ref(),
        &input,
        &find,
        &ids,
        &shown,
        today,
        &resolved.date_format,
    );
    let (diff, more, files) = apply::diff_text(&plan, src.as_ref(), &label);
    plan.problems.extend(more);
    if !plan.problems.is_empty() {
        for p in &plan.problems {
            eprintln!("mdgrid: {p}");
        }
        return fail(Msg::ApplyProblems.text());
    }
    if plan.ignored.0 > 0 {
        eprintln!(
            "mdgrid: {}",
            Msg::ApplyIgnored.fill(&[&plan.ignored.0, &plan.ignored.1.join(", ")])
        );
    }
    if files == 0 {
        eprintln!("mdgrid: {}", Msg::ApplyNoChanges.text());
        return ExitCode::SUCCESS;
    }
    if !yes {
        print!("{diff}");
        eprintln!("mdgrid: {}", Msg::ApplyDryRun.fill(&[&files]));
        return ExitCode::SUCCESS;
    }
    let (saved, failed) = apply::save(&mut plan, src.as_mut(), &label);
    for s in &saved {
        println!("{s}");
    }
    for f in &failed {
        eprintln!("mdgrid: {f}");
    }
    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// 画面の書き先。ふだんは標準出力、`--pick`(OUT-3)は結果を標準出力に出すので端末(`/dev/tty`)。
/// キーの読み取りと raw は crossterm が標準入力(端末でなければ `/dev/tty`)で行うので、書き先だけを分ける。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Stdout,
    Tty,
}

impl Screen {
    /// 書き先を開く。`/dev/tty` は描画のたびの小さな書き込みをまとめる(execute! と ratatui が flush する)。
    fn writer(self) -> io::Result<Box<dyn Write + Send>> {
        Ok(match self {
            Screen::Stdout => Box::new(io::stdout()),
            Screen::Tty => Box::new(io::BufWriter::new(
                std::fs::OpenOptions::new().write(true).open("/dev/tty")?,
            )),
        })
    }
}

/// `--pick` で画面を出せるか(SR-10): 端末(`/dev/tty`)が開けなければ理由を返す。
fn check_dev_tty() -> Result<(), String> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .map(|_| ())
        .map_err(|e| Msg::NoDevTty.fill(&[&e]))
}

/// 端末を戻す(通常の終わりとパニックの両方から呼ぶ)。
fn restore(screen: Screen) {
    let _ = terminal::disable_raw_mode();
    let Ok(mut out) = screen.writer() else {
        return;
    };
    let _ = out.write_all(b"\x1b[?1006l\x1b[?1002l");
    let _ = execute!(
        out,
        EndSynchronizedUpdate,
        LeaveAlternateScreen,
        cursor::Show
    );
}

/// `editor` はノートを開くエディタ(SR-8。`config::resolve_editor` で決めたもの)。`screen` は画面の書き先。
fn run(app: &mut App, editor: &str, screen: Screen) -> io::Result<()> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore(screen);
        hook(info);
    }));
    terminal::enable_raw_mode()?;
    // ここから先は、`?` で抜けても Drop で端末を戻す(パニックのフックと二重に戻しても害は無い)。
    let _guard = Restore(screen);
    enter(screen)?;
    event_loop(app, editor, screen)
}

/// 画面の準備(raw・代替画面・マウス)。起動と $EDITOR から戻ったときに呼ぶ。
fn enter(screen: Screen) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut out = screen.writer()?;
    execute!(out, EnterAlternateScreen, cursor::Hide)?;
    // マウス: ボタンイベントの追跡と SGR(docs/design.md)。
    out.write_all(b"\x1b[?1002h\x1b[?1006h")?;
    out.flush()
}

/// 抜けるときに端末を戻すガード。
struct Restore(Screen);

impl Drop for Restore {
    fn drop(&mut self) {
        restore(self.0);
    }
}

type Term = Terminal<CrosstermBackend<Box<dyn Write + Send>>>;

/// 画面を消して次の描画で全部描き直す。`Terminal::clear` はカーソルの位置を標準出力で問い合わせる
/// (crossterm の `cursor::position`)ので、`/dev/tty` に描くときは問い合わせない `resize` で消す。
fn clear_screen(term: &mut Term, screen: Screen) -> io::Result<()> {
    match screen {
        Screen::Stdout => term.clear(),
        Screen::Tty => {
            let size = term.size()?;
            term.resize(ratatui::layout::Rect::new(0, 0, size.width, size.height))
        }
    }
}

fn event_loop(app: &mut App, editor: &str, screen: Screen) -> io::Result<()> {
    let mut term = Terminal::new(CrosstermBackend::new(screen.writer()?))?;
    clear_screen(&mut term, screen)?;
    let mut last_poll = Instant::now();
    loop {
        if !app.loaded() {
            app.load_step(LOAD_BUDGET);
        } else if last_poll.elapsed() >= app.poll_interval() {
            app.poll();
            last_poll = Instant::now();
        }
        let size = term.size()?;
        app.resize(size.width, size.height);
        execute!(term.backend_mut(), BeginSynchronizedUpdate)?;
        term.draw(|f| ui::draw(f, app))?;
        execute!(term.backend_mut(), EndSynchronizedUpdate)?;

        let wait = if app.loaded() {
            app.poll_interval().saturating_sub(last_poll.elapsed())
        } else {
            Duration::ZERO
        };
        // たまったイベントを全部さばいてから1回だけ描く。
        if event::poll(wait)? {
            handle(app, event::read()?);
            while event::poll(Duration::ZERO)? {
                handle(app, event::read()?);
            }
        }
        if app.wants_editor() {
            // SR-8: 端末を戻してからエディタを起動し、戻ったら画面を作り直す。
            app.open_editor(editor, &mut |leave| {
                if leave {
                    restore(screen);
                    Ok(())
                } else {
                    enter(screen)?;
                    clear_screen(&mut term, screen)
                }
            });
        }
        if app.quit {
            return Ok(());
        }
    }
}

fn handle(app: &mut App, ev: Event) {
    match ev {
        Event::Key(k) => app.key(k),
        // マウスは App の側でキーと同じ意図に直す(CE-1 の入力の外のクリック・SR-6)。
        Event::Mouse(m) => match m.kind {
            MouseEventKind::ScrollDown => app.wheel(true),
            MouseEventKind::ScrollUp => app.wheel(false),
            MouseEventKind::Down(MouseButton::Left) => app.click(m.column, m.row),
            // SR-24: セルの右クリックでそのセルを選び、その場の操作の一覧を開く。
            MouseEventKind::Down(MouseButton::Right) => app.right_click(m.column, m.row),
            // SR-3: 列の見出しの境界のドラッグで幅を変える。
            MouseEventKind::Drag(MouseButton::Left) => app.drag(m.column),
            MouseEventKind::Up(MouseButton::Left) => app.release(),
            _ => {}
        },
        _ => {}
    }
}

#[cfg(test)]
#[path = "test_main.rs"]
mod tests;
