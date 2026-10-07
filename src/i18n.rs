//! 画面と起動の文言の表と、言語の決め方(SR-23)。形は docs/design.md の `src/i18n.rs`。
//!
//! 文言は `msgs!` の表1か所に英語と日本語の組で置く(片方が欠けた行はコンパイルで落ちる)。
//! 差し込みは `{0}`・`{1}` の番号で、`Msg::fill` が埋める。
//!
//! 言語は、スレッドごとの上書き(`scoped`。試験用)> 大域の既定(`set_default`。main が決める)>
//! 環境(`LC_ALL`・`LC_MESSAGES`・`LANG`)の順で決まる。

use std::cell::Cell;
use std::fmt::Display;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// 出す言語。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ja,
}

/// 設定の `language`(SR-23)。既定は `Auto`(環境に従う)。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Language {
    #[default]
    Auto,
    En,
    Ja,
}

impl Language {
    /// 設定の値(`"auto"`・`"en"`・`"ja"`)を読む。ほかは None。
    pub fn parse(s: &str) -> Option<Language> {
        match s {
            "auto" => Some(Language::Auto),
            "en" => Some(Language::En),
            "ja" => Some(Language::Ja),
            _ => None,
        }
    }
}

/// `auto` で見る環境変数(この順で最初の空でない値)。
pub const LOCALE_VARS: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

/// 設定が En・Ja ならそれ。Auto なら `LOCALE_VARS` の順で最初の空でない値が `ja` で始まれば Ja、
/// ほか(どれも無いときも)は En。`env` は環境変数の名前から値を引く(試験で差し替える)。
pub fn resolve(setting: Language, env: impl Fn(&str) -> Option<String>) -> Lang {
    match setting {
        Language::En => Lang::En,
        Language::Ja => Lang::Ja,
        Language::Auto => LOCALE_VARS
            .iter()
            .filter_map(|name| env(name))
            .find(|v| !v.is_empty())
            .map_or(Lang::En, |v| {
                if v.starts_with("ja") {
                    Lang::Ja
                } else {
                    Lang::En
                }
            }),
    }
}

/// 大域の既定(0 は未設定、1 は En、2 は Ja)。
static DEFAULT: AtomicU8 = AtomicU8::new(0);

/// 未設定のときに使う、環境から決めた言語(最初に引いたときに1回だけ読む)。
static FROM_ENV: OnceLock<Lang> = OnceLock::new();

thread_local! {
    /// このスレッドの上書き(`scoped`)。
    static OVERRIDE: Cell<Option<Lang>> = const { Cell::new(None) };
}

/// 大域の既定の言語を決める(main だけが呼ぶ)。
pub fn set_default(lang: Lang) {
    let v = match lang {
        Lang::En => 1,
        Lang::Ja => 2,
    };
    DEFAULT.store(v, Ordering::Relaxed);
}

/// 今の言語: このスレッドの上書き > 大域の既定 > 環境。
pub fn current() -> Lang {
    if let Some(l) = OVERRIDE.with(Cell::get) {
        return l;
    }
    match DEFAULT.load(Ordering::Relaxed) {
        1 => Lang::En,
        2 => Lang::Ja,
        _ => *FROM_ENV.get_or_init(|| resolve(Language::Auto, |k| std::env::var(k).ok())),
    }
}

/// 落とすまでこのスレッドの言語を変える guard(`scoped` が返す)。落とすと前の言語に戻る。
/// 作った順の逆に落とさないと前の言語に戻らない(試験用)。
#[must_use = "guard を落とすと元の言語に戻る"]
pub struct Scoped {
    prev: Option<Lang>,
}

impl Drop for Scoped {
    fn drop(&mut self) {
        OVERRIDE.with(|c| c.set(self.prev));
    }
}

/// 落とすまでこのスレッドの言語を `lang` にする(試験用。入れ子でよい)。
pub fn scoped(lang: Lang) -> Scoped {
    let prev = OVERRIDE.with(|c| c.replace(Some(lang)));
    Scoped { prev }
}

/// 文言の表を作る。各行は `Key = "english", "日本語";`。
macro_rules! msgs {
    ($( $(#[$meta:meta])* $key:ident = $en:expr, $ja:expr; )*) => {
        /// 画面と起動の文言(SR-23)。英語は `en`、日本語は `ja`、今の言語は `text`。
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Msg {
            $( $(#[$meta])* $key, )*
        }

        impl Msg {
            /// 表の全部の文言。
            pub const ALL: &'static [Msg] = &[ $( Msg::$key, )* ];

            /// 英語の文。
            pub const fn en(self) -> &'static str {
                match self {
                    $( Msg::$key => $en, )*
                }
            }

            /// 日本語の文。
            pub const fn ja(self) -> &'static str {
                match self {
                    $( Msg::$key => $ja, )*
                }
            }
        }
    };
}

impl Msg {
    /// `lang` の文。
    pub const fn get(self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.en(),
            Lang::Ja => self.ja(),
        }
    }

    /// 今の言語(`current`)の文。
    pub fn text(self) -> &'static str {
        self.get(current())
    }

    /// 今の言語の文の `{n}` を `args[n]` で置き換える。ほかの `{` と、値の無い番号はそのまま残す。
    /// 差し込んだ値の中はもう見ない。英語で、数の差し込み(`singular`)の値が `1` なら単数の文を使う。
    pub fn fill(self, args: &[&dyn Display]) -> String {
        let lang = current();
        let src = match (lang, self.singular()) {
            (Lang::En, Some((n, one))) if args.get(n).is_some_and(|v| v.to_string() == "1") => {
                one.en()
            }
            _ => self.get(lang),
        };
        fill_in(src, args)
    }

    /// 英語で数が 1 のときに代わりに使う単数の文言と、その数の差し込みの番号(A-11)。
    /// 日本語は単数と複数を分けないので見ない。
    pub const fn singular(self) -> Option<(usize, Msg)> {
        match self {
            Msg::HelpHead => Some((2, Msg::HelpHeadOne)),
            Msg::GotoPastEnd => Some((1, Msg::GotoPastEndOne)),
            Msg::HeaderRows => Some((0, Msg::HeaderRow)),
            Msg::BarCount => Some((1, Msg::BarCountOne)),
            Msg::HintHeadRow => Some((0, Msg::HintHeadRowOne)),
            Msg::FreezeTooWide => Some((0, Msg::FreezeTooWideOne)),
            Msg::Frozen => Some((0, Msg::FrozenOne)),
            Msg::EditBulkLead => Some((0, Msg::EditBulkLeadOne)),
            Msg::EditOnlyThisRow => Some((0, Msg::EditOnlyThisRowOne)),
            Msg::CopyWhatSelected => Some((0, Msg::CopyWhatSelectedOne)),
            Msg::BulkFoldedOut => Some((0, Msg::BulkFoldedOutOne)),
            Msg::BulkSkipped => Some((0, Msg::BulkSkippedOne)),
            Msg::BulkSet => Some((0, Msg::BulkSetOne)),
            Msg::BulkSame => Some((0, Msg::BulkSameOne)),
            Msg::BulkSetTo => Some((0, Msg::BulkSetToOne)),
            Msg::PickBulkDone => Some((0, Msg::PickBulkDoneOne)),
            Msg::PickBulkSame => Some((0, Msg::PickBulkSameOne)),
            Msg::PickHintBulk => Some((0, Msg::PickHintBulkOne)),
            Msg::PickTitleBulk => Some((1, Msg::PickTitleBulkOne)),
            Msg::HighlightSame => Some((2, Msg::HighlightSameOne)),
            Msg::ReviewSaved => Some((0, Msg::ReviewSavedOne)),
            Msg::ApplyDryRun => Some((0, Msg::ApplyDryRunOne)),
            Msg::ApplyIgnored => Some((0, Msg::ApplyIgnoredOne)),
            Msg::ReviewStopped => Some((0, Msg::ReviewStoppedOne)),
            Msg::ReviewFailed => Some((0, Msg::ReviewFailedOne)),
            Msg::ReviewHead => Some((0, Msg::ReviewHeadOne)),
            _ => None,
        }
    }
}

/// 今の言語の文(`text`)を String に。メッセージの `Some(MSG.into())` をそのまま書ける。
impl From<Msg> for String {
    fn from(m: Msg) -> String {
        m.text().to_string()
    }
}

/// 今の言語の文(`text`)を出す。
impl Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.text())
    }
}

/// `fill` の本体。
fn fill_in(src: &str, args: &[&dyn Display]) -> String {
    let mut out = String::with_capacity(src.len() + 16);
    let mut rest = src;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let arg = (digits > 0 && after.as_bytes().get(digits) == Some(&b'}'))
            .then(|| after[..digits].parse::<usize>().ok())
            .flatten()
            .and_then(|n| args.get(n));
        match arg {
            Some(v) => {
                out.push_str(&v.to_string());
                rest = &after[digits + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

msgs! {
    // ---- main: 使い方(CLI-2) ----
    Usage = "Usage: mdgrid [<.base | folder | .md> ...] [--view <name>] [--readonly] [--no-color] [--config <path>]\n        mdgrid [<.base | folder | .md> ...] --print [--format csv|tsv|json|md] [--with-path] [--view <name>]\n                 [--filter <expression>]... [--sort <column>[:desc]]...\n        mdgrid [<.base | folder | .md> ...] --pick path|<column> [--view <name>]\n        mdgrid [<.base | folder> ...] --apply <path> [--yes]\n        mdgrid --print-config\n        mdgrid --completions <shell>\n        mdgrid --man\n\n  Open a table of the Markdown notes in a folder: one row per note, one column per\n  front matter key. Given a .base, open its table view (notes are looked up under\n  the vault root). Given a .md file, open its folder with that note selected.\n  With no arguments, open the current folder.\n\n  --view <name>           open that view of the .base (the first view if omitted;\n                          --print also looks up mdgrid views)\n  --readonly              open read-only (no editing, no saving)\n  --no-color              draw without colors\n  --config <path>         read this configuration file\n                          (default: $XDG_CONFIG_HOME/mdgrid/config.toml)\n  --print                 print the view's table to standard output without the screen\n                          (notes are not changed)\n  --format <format>       format for --print (csv, tsv, json, md; default csv)\n  --with-path             add a first column \"path\" to --print with each row's note path\n                          (the same path as --pick path)\n  --filter <expression>   keep only the rows where the .base expression is true\n                          (repeatable; with a .base view, combined with its filters)\n  --sort <column>[:desc]  sort --print by this column (repeatable; replaces the sort\n                          of the view)\n  --pick <path|column>    show the screen read-only on the terminal; Enter prints the path\n                          or the column value of the marked rows (or the selected row),\n                          one per line, to standard output and exits\n                          (q or Esc cancels with exit code 1)\n  --apply <path>          read a CSV or JSON in the --print --with-path form (- for stdin)\n                          and change the cells that differ; shows the diff only\n  --yes                   with --apply, write the changes\n  --print-config          print every configuration item as TOML with its default value\n                          and a description (docs/config.md)\n  --completions <shell>   print shell completions (bash, zsh, fish, elvish, powershell)\n  --man                   print the man page (roff)\n  --help                  print this help\n  --version               print the version",
        "使い方: mdgrid [<.base | フォルダ | .md> …] [--view <名前>] [--readonly] [--no-color] [--config <パス>]\n        mdgrid [<.base | フォルダ | .md> …] --print [--format csv|tsv|json|md] [--with-path] [--view <名前>]\n                 [--filter <式>]… [--sort <列>[:desc]]…\n        mdgrid [<.base | フォルダ | .md> …] --pick path|<列の名前> [--view <名前>]\n        mdgrid [<.base | フォルダ> …] --apply <パス> [--yes]\n        mdgrid --print-config\n        mdgrid --completions <シェル>\n        mdgrid --man\n\n  フォルダの中の Markdown のノートを行、フロントマターのキーを列にした表を開く。\n  .base を渡すと、その table ビューで開く(ノートは保管庫の根の下から探す)。\n  .md のファイルを渡すと、そのフォルダを開いてその行を選ぶ。\n  引数が無ければ今のフォルダを開く。\n\n  --view <名前>           .base のそのビューで開く(無ければ先頭のビュー。--print では mdgrid のビューも探す)\n  --readonly              読むだけで開く(編集も保存もしない)\n  --no-color              色なしで出す\n  --config <パス>         この設定ファイルを読む(既定は $XDG_CONFIG_HOME/mdgrid/config.toml)\n  --print                 画面を出さずに、ビューの表を標準出力に出す(ノートは書き換えない)\n  --format <形>           --print の形(csv・tsv・json・md。既定は csv)\n  --with-path             --print の表の先頭に、各行のノートのパスの列 path を足す(--pick path と同じ形)\n  --filter <式>           .base の式が真の行だけを --print に出す(何度でも。.base のビューでは、その絞り込みと両方)\n  --sort <列>[:desc]      --print をこの列で並べる(何度でも。ビューの並べ替えの代わり)\n  --pick <path|列>        画面を端末に読むだけで出し、Enter で印の行(無ければ選んでいる行)のパスか\n                          列の値を1行ずつ標準出力に出して終わる(q・Esc で取りやめ、終了コード 1)\n  --apply <パス>          --print --with-path の形の CSV・JSON(- で標準入力)を読み、違うセルを変える(差分だけ出す)\n  --yes                   --apply で、差分を出すだけでなく書く\n  --print-config          設定の全項目を既定値と説明付きの TOML で出す(docs/config.md)\n  --completions <シェル>  シェルの補完の定義を出す(bash・zsh・fish・elvish・powershell)\n  --man                   man ページ(roff)を出す\n  --help                  この案内を出す\n  --version               版を出す";

    // ---- main: 引数の値の名前(clap の value_name。補完と man は日本語のまま、誤りの文では今の言語) ----
    ValuePath = "path", "パス";
    ValueName = "name", "名前";
    ValueFormat = "format", "形";
    ValuePick = "path|column", "path|列の名前";
    ValueShell = "shell", "シェル";
    ValueExpr = "expression", "式";
    ValueSort = "column[:desc]", "列[:desc]";

    // ---- main: 引数の誤り(CLI-4。clap の誤りを1行に直す) ----
    CliUnknownOption = "unknown option: {0} (see --help)", "知らないオプション: {0}(--help で使い方)";
    /// {0} はオプション、{1} は無いもの(`CliWhat*`)。
    CliMissingValue = "{0} needs {1}", "{0} に{1}が無い";
    CliWhatView = "a view name", "ビューの名前";
    CliWhatConfig = "a config path", "設定のパス";
    CliWhatShell = "a shell name", "シェルの名前";
    CliWhatFormat = "a format name", "形の名前";
    CliWhatPick = "what to print (path or a column name)", "出すもの(path か列の名前)";
    CliWhatValue = "a value", "値";
    CliUnknownValue = "unknown value for {0}: {1} (valid values: {2})", "{0} に知らない値: {1}(使える値: {2})";
    /// 並べた値の区切り。
    ListSep = ", ", "・";
    CliInvalidUtf8 = "arguments are not valid UTF-8", "引数が UTF-8 で読めない";
    CliConflict = "{0} and {1} cannot be used together (see --help)", "{0} と {1} は一緒に使えない(--help で使い方)";
    CliRequired = "{0} is required (see --help)", "{0} が要る(--help で使い方)";
    /// {0} は渡された旗、{1} はそれに要るもの(`--format`・`--with-path` に `--print`)。
    CliRequiredBy = "{0} requires {1} (see --help)", "{0} には {1} が要る(--help で使い方)";
    CliTakesNoValue = "{0} takes no value: {1} (see --help)", "{0} は値を取らない: {1}(--help で使い方)";
    CliBadArgument = "cannot read the argument: {0} (see --help)", "読めない引数: {0}(--help で使い方)";
    CliUnreadable = "cannot read the arguments: {0} (see --help)", "引数が読めない: {0}(--help で使い方)";

    // ---- main: 起動できない理由(CLI-4・SR-10) ----
    CannotWriteOutput = "cannot write the output: {0}", "出せない: {0}";
    ConfigUnreadable = "cannot read the config: {0} ({1})", "設定が読めない: {0}({1})";
    PathUnopenable = "cannot open the path: {0} ({1})", "開けないパス: {0}({1})";
    BaseAlone = ".base must be given alone (not with folders or another .base): {0}", ".base は1つだけで渡す(フォルダや別の .base と並べない): {0}";
    NotAFolder = "not a folder: {0}", "フォルダではない: {0}";
    ViewNeedsBase = "--view {0} can only be used when opening a .base", "--view {0} は .base を開くときだけ使える";
    CannotOpen = "cannot open: {0}", "開けない: {0}";
    BaseUnreadable = "cannot read the .base: {0} ({1})", ".base が読めない: {0}({1})";
    /// {0} はビューの名前、{1} は `ViewsAre` か `NoViews`。
    NoSuchView = "no view \"{0}\" ({1})", "ビュー「{0}」は無い({1})";
    ViewsAre = "views: {0}", "あるビュー: {0}";
    NoViews = "there are no views", "ビューが無い";
    NotATerminal = "standard input and output are not a terminal, so the screen cannot be shown", "標準の入出力が端末ではないので、画面を出せない";
    NoDevTty = "cannot open the terminal (/dev/tty), so the screen cannot be shown ({0})", "端末(/dev/tty)が開けないので、画面を出せない({0})";
    TerminalError = "terminal error: {0}", "端末の誤り: {0}";
    PickNoColumn = "--pick column \"{0}\" is not in the table (columns: {1})", "--pick の列「{0}」は表に無い(ある列: {1})";

    // ---- source/markdown: 読むだけの理由(WB-3・WB-5・CE-8・CE-16・CE-19・BV-12) ----
    LockNoFrontmatterSetting = "no front matter (not added by the setting)", "フロントマターが無い(設定で書かない)";
    LockEmptyFrontmatterSetting = "empty front matter (not filled by the setting)", "空のフロントマター(設定で書かない)";
    LockTypesConflict = "types.json gives this column different types per root (read-only)", "根ごとに types.json の型が違う(読むだけ)";
    LockNoFrontmatter = "no front matter", "フロントマターが無い";
    LockEmptyFrontmatter = "front matter is empty", "フロントマターが空";
    LockBom = "the file starts with a BOM", "BOM つきのファイル";
    // ---- --apply(CLI-17) ----
    /// {0} は JSON の読み取りの誤り。
    ApplyBadJson = "cannot read the JSON: {0}", "JSON を読めない: {0}";
    /// {0} は何番目のオブジェクトか。
    ApplyNoPath = "item {0} has no \"path\" (print with --with-path)", "{0} 番目に \"path\" が無い(--with-path を付けて出す)";
    ApplyBadCsv = "cannot read the CSV: a quoted field is not closed", "CSV を読めない: 引用符で囲んだ欄が閉じていない";
    ApplyNoPathColumn = "the CSV has no \"path\" column (print with --with-path)", "CSV に path の列が無い(--with-path を付けて出す)";
    /// {0} は行、{1} は欄の数、{2} は見出しの欄の数。
    ApplyCsvWidth = "line {0} has {1} fields, the header has {2}", "{0} 行目の欄の数が {1}(見出しは {2})";
    ApplyNotList = "the value must be text, a number, true/false, null or a list of text", "値は文字・数・true/false・null・文字のリストにする";
    /// {0} は行、{1} は path。
    ApplyUnknownPath = "line {0}: no note at {1}", "{0} 行目: {1} にノートが無い";
    /// {0} は行、{1} は path、{2} は列、{3} は理由。
    ApplyBadValue = "line {0}: {1}: {2}: {3}", "{0} 行目: {1}: {2}: {3}";
    /// {0} は行、{1} は path、{2} は列、{3} は読むだけの理由。
    ApplyLocked = "line {0}: {1}: {2} cannot be written ({3})", "{0} 行目: {1}: {2} は書けない({3})";
    /// {0} はノート。
    ApplyChangedOutside = "{0} changed outside mdgrid; not written", "{0} は外で変わったので書かなかった";
    /// {0} は変わるファイルの数。
    ApplyDryRun = "{0} files would change; nothing was written (add --yes to write them)", "{0} 個のファイルが変わる。まだ書いていない(書くには --yes を付ける)";
    /// {0} は数。
    ApplyDryRunOne = "{0} file would change; nothing was written (add --yes to write it)", "{0} 個のファイルが変わる。まだ書いていない(書くには --yes を付ける)";
    /// {0} は捨てた直しの数、{1} は列の名前。
    ApplyIgnored = "{0} edits in columns that cannot be written were ignored ({1})", "書けない列の {0} 個の直しは当てなかった({1})";
    ApplyIgnoredOne = "{0} edit in a column that cannot be written was ignored ({1})", "書けない列の {0} 個の直しは当てなかった({1})";
    /// CLI-16。{0} は列の名前。
    CliUnknownSort = "--sort {0}: no such column in the view or in the notes", "--sort {0}: ビューにもノートにも無い列";
    ApplyNoChanges = "no changes: every value is the same as in the notes", "変わる値が無い(どれもノートと同じ)";
    /// {0} はファイル、{1} は誤り。
    ApplyUnreadable = "cannot read {0}: {1}", "{0} を読めない: {1}";
    ApplyProblems = "nothing was written because of the problems above", "上の理由があるので、何も書かなかった";
    /// {0} は見出し。
    ApplyDuplicateHeader = "the column {0} appears twice in the header", "見出しに列 {0} が2つある";
    /// {0} は見出し。
    ApplyUnknownColumn = "the column {0} is not in the view and no note has that key (check the spelling)", "列 {0} はビューに無く、どのノートにもそのキーが無い(綴りを確かめる)";
    /// {0} は見出し。
    ApplyAmbiguousColumn = "the column {0} could mean two columns (a display name and another column's key)", "列 {0} は2つの列に読める(表示名と、別の列のキー)";
    /// {0} は行、{1} は path。
    ApplyAmbiguousPath = "line {0}: {1} matches different notes under the given folders (write the path from one folder)", "{0} 行目: {1} は渡したフォルダごとに別のノートに当たる(1つのフォルダからのパスで書く)";
    /// {0} は行、{1} は path、{2} は列、{3} は先に書いた行。
    ApplyDuplicateRow = "line {0}: {1}: {2} is already set on line {3}", "{0} 行目: {1}: {2} は {3} 行目でも当てている";
    /// {0} は値。
    ApplyDateForm = "write dates as YYYY-MM-DD (\"{0}\")", "日付は YYYY-MM-DD で書く(「{0}」)";
    ApplyListAmbiguous = "this list has items containing \", \" or edge spaces, so it cannot be edited from CSV (use --format json)", "このリストの要素に「, 」か前後の空白があり、CSV からは直せない(--format json を使う)";
    /// {0} は数。
    ApplyBigNumber = "{0} does not fit in a 64-bit integer (it would be rounded)", "{0} は64ビットの整数に入らない(丸めてしまう)";
    ApplyReadonly = "--apply writes notes, so it cannot be used with --readonly", "--apply はノートを書くので --readonly と一緒に使えない";
    /// CLI-16。{0} は渡した --sort の値。
    CliBadSort = "--sort {0}: write a column, optionally followed by :asc or :desc", "--sort {0}: 列の名前と、後ろに :asc か :desc(無くてよい)を書く";
    /// CLI-16。{0} はビューの名前。
    CliFilterNativeView = "--filter and --sort work with a .base view or a folder, not with the mdgrid view \"{0}\"", "--filter・--sort は .base のビューかフォルダに使う(mdgrid のビュー「{0}」には使えない)";
    LockToml = "the frontmatter is TOML (+++), not YAML", "フロントマターが YAML でなく TOML(+++)";
    LockNotUtf8 = "the file is not UTF-8", "UTF-8 ではない";
    LockMixedNewlines = "mixed line endings", "改行コードが混ざっている";
    /// {0} は重なったキー。
    LockDuplicateKey = "the same key appears twice ({0})", "同じキーが2回ある({0})";
    LockUnclosed = "front matter is not closed", "フロントマターが閉じていない";
    LockHardLink = "the file has hard links", "ハードリンクがある";
    LockInvalidYaml = "unreadable format (read-only)", "読めない形(読むだけ)";
    LockInvalidYamlAt = "unreadable YAML at line {0}: {1} (read-only)", "YAML が {0}行目で読めない: {1}(読むだけ)";
    LockFileAttr = "file attribute (read-only)", "ファイルの属性(読むだけ)";
    LockFormulaColumn = "formula column (read-only)", "式の列(読むだけ)";
    /// WB-5。`writeback::NO_PERMISSION` は日本語。
    NoPermission = "no write permission", "書き込めない権限";
    ListBlankBetween = "list with blank lines between items", "要素の間に空行があるリスト";
    ListCommentBetween = "list with comments between items", "要素の間にコメントがあるリスト";
    ListTrailingComment = "list with comments after items", "要素の行末にコメントがあるリスト";
    ListIndent = "list with items at different indents", "字下げの違う要素があるリスト";
    ListTabAfterDash = "list with a tab after `-`", "`-` のあとがタブのリスト";
    ListMultiLine = "list with an item spanning several lines", "複数行にまたがる要素を含むリスト";
    ListNested = "list with nested items", "入れ子の要素を含むリスト";
    ListNonString = "list with non-string items", "文字列でない要素を含むリスト";
    ValueNested = "nested value", "ネストした値";
    ValueMultiLine = "multi-line value", "複数行の値";
    ValueAnchor = "anchored value", "アンカーの値";
    ValueTagged = "tagged value", "タグつきの値";
    ValueUnwritable = "value in a form that cannot be rewritten", "書き換えられない形の値";
    RowNotLoaded = "row not loaded", "読んでいない行";
    SyncConflictFile = "sync conflict file", "同期の競合ファイル";
    CannotRead = "cannot read: {0}", "読めない: {0}";

    // ---- changes: ためられない理由(CE-8・CE-16・CE-18) ----
    NoBaseAfterReload = "no baseline for the reloaded row", "読み直した行の基準が無い";
    SkipListCell = "cannot write a single value to a list cell (add or remove items instead)", "リストのセルには1つの値を書けない(要素を付け外しする)";
    SkipNotLoaded = "not loaded yet", "まだ読めていない";

    // ---- types: 日付の形と日付の打ち込み(CE-5・CE-22) ----
    /// {0} は形(引用符つき)、{1} は理由。
    DateFormatBad = "cannot read the date format {0}: {1}", "日付の形 {0} が読めない: {1}";
    DateFormatBadChar = "{0} cannot be used (the parts are YYYY, YY, MM, M, DD, D, ddd and non-alphanumeric separators)", "{0} は使えない(部品は YYYY・YY・MM・M・DD・D・ddd と、英数字でない区切りの文字)";
    DateFormatMonthDay = "needs exactly one month (MM or M) and one day (DD or D)", "月(MM か M)と日(DD か D)が1つずつ要る";
    DateFormatYearWeekday = "at most one year and one weekday", "年と曜日は1つまで";
    DateFormatAdjacent = "M and D cannot be placed next to other number parts without a separator", "M・D はほかの数の部品と区切り無しで並べられない";
    DateNotReal = "no such date: {0}", "実在しない日付: {0}";
    /// {0} は打ち込み、{1} は日付、{2} は曜日(訳さない)。
    DateWeekdayMismatch = "the weekday does not match the date: {0} ({1} is {2})", "曜日が日付と合わない: {0}({1} は {2})";
    /// {1} は使える形を `ListSep` で並べたもの。
    DateUnreadable = "not a date: {0} ({1})", "日付として読めない: {0}({1})";
    DateEmptyClears = "empty to clear", "空で消す";
    DateOutOfRange = "outside the date range (0000-01-01 to 9999-12-31): {0}", "日付の範囲(0000-01-01〜9999-12-31)の外: {0}";

    // ---- config: 設定の警告(CLI-3) ----
    ConfigTomlLine = "cannot read the config TOML (line {0}): {1}", "設定の TOML が読めない({0} 行目): {1}";
    ConfigToml = "cannot read the config TOML: {0}", "設定の TOML が読めない: {0}";
    ConfigUnknownItem = "ignored the unknown config item `{0}`", "設定の知らない項目 `{0}` は無視した";
    ConfigBadDateFormat = "ignored the config item `date_format` and used the default YYYY-MM-DD: {0}", "設定の項目 `date_format` を無視して既定の YYYY-MM-DD にした: {0}";
    ConfigBadAsk = "ignored `{0}` in the config item `new_note.ask`: not a writable column, or a duplicate", "設定の項目 `new_note.ask` の `{0}` は書けない列か重なりなので無視した";
    ConfigBadSet = "ignored `{0}` in the config item `new_note.set`: not a writable column", "設定の項目 `new_note.set` の `{0}` は書けない列なので無視した";
    /// {0} は項目、{1} は求める型(`Want*`)。
    ConfigWrongType = "ignored the config item `{0}`: not {1}", "設定の項目 `{0}` は{1}でないので無視した";
    WantIntAtLeast0 = "an integer of 0 or more", "0 以上の整数";
    WantIntAtLeast1 = "an integer of 1 or more", "1 以上の整数";
    WantBool = "true or false", "true か false";
    WantDateFormat = "a date format string", "日付の形の文字列";
    WantWeekStart = "\"sun\" or \"mon\"", "\"sun\" か \"mon\"";
    WantString = "a string", "文字列";
    WantLanguage = "one of \"auto\", \"en\", \"ja\"", "\"auto\"・\"en\"・\"ja\" のどれか";
    WantTheme = "one of \"default\", \"nord\", \"solarized-light\", \"dracula\", \"gruvbox\", \"pink-monster\", \"dozy-pink\"", "\"default\"・\"nord\"・\"solarized-light\"・\"dracula\"・\"gruvbox\"・\"pink-monster\"・\"dozy-pink\" のどれか";
    WantKeysTable = "a [keys.<mode>] table", "[keys.<モード>] の表";
    WantKeyActionTable = "a table of key = action", "キー = 動作 の表";
    WantActionName = "an action name string", "動作の名前の文字列";
    WantNewNoteTable = "a [new_note] table", "[new_note] の表";
    WantColumnNames = "a list of column name strings", "列の名前の文字列の並び";
    WantColumnValueTable = "a table of column = value", "列 = 値 の表";
    WantSetValue = "a string, number, boolean, date, or a list of them", "文字列・数・真偽・日付・その並び";
    WantDisplayTable = "a [display] table", "[display] の表";

    // ---- views: views.toml の警告と誤り(BV-20) ----
    /// {0} はファイル、{1} は行、{2} は TOML の誤り(英語のまま)。
    ViewsTomlLine = "cannot read {0} (line {1}), so mdgrid views are not loaded: {2}", "{0} が読めない({1} 行目)ので mdgrid のビューは読まない: {2}";
    ViewsToml = "cannot read {0}, so mdgrid views are not loaded: {1}", "{0} が読めないので mdgrid のビューは読まない: {1}";
    ViewsNotUtf8 = "{0} is not UTF-8, so mdgrid views are not loaded", "{0} が UTF-8 でないので mdgrid のビューは読まない";
    ViewsUnknownItem = "ignored the unknown item `{1}` in {0}", "{0} の知らない項目 `{1}` は無視した";
    ViewsBadAsk = "ignored `{1}` in `target.view.new_note.ask` of {0}: not a writable column, or a duplicate", "{0} の `target.view.new_note.ask` の `{1}` は書けない列か重なりなので無視した";
    ViewsBadSet = "ignored `{1}` in `target.view.new_note.set` of {0}: not a writable column", "{0} の `target.view.new_note.set` の `{1}` は書けない列なので無視した";
    ViewsTargetNotArray = "ignored `target` in {0}: not an array of [[target]] tables", "{0} の `target` が [[target]] の表の並びでないので無視した";
    /// {1} は1から数えた番号。
    ViewsTargetNotTable = "ignored target #{1} in {0}: not a table", "{0} の {1} 番目の target が表でないので無視した";
    ViewsTargetNoPath = "ignored target #{1} in {0}: no path (string)", "{0} の {1} 番目の target に path(文字列)が無いので無視した";
    ViewsViewNotArray = "ignored the view of target `{1}` in {0}: not an array of [[target.view]] tables", "{0} の target `{1}` の view が [[target.view]] の表の並びでないので無視した";
    ViewsViewNotTable = "ignored view #{2} of target `{1}` in {0}: not a table", "{0} の target `{1}` の {2} 番目のビューが表でないので無視した";
    ViewsViewUnreadable = "ignored view #{2} of target `{1}` in {0}: cannot read it: {3}", "{0} の target `{1}` の {2} 番目のビューが読めないので無視した: {3}";
    ViewsSaveNotUtf8 = "{0} is not UTF-8, so it is not written", "{0} が UTF-8 でないので書かない";
    ViewsSaveBroken = "{0} (not written)", "{0}(書かない)";
    ViewsSaveTargetNotArray = "`target` in {0} is not an array of tables, so it is not written", "{0} の `target` が表の並びでないので書かない";
    ViewsNotATable = "cannot turn the view into a table", "ビューを表にできない";

    // ---- views: .base への書き出しの近似と落としたもの(BV-19・SR-20) ----
    ExportEmptyNote = "how {0} is judged \"empty\": when the value is a map or a nested list, it may look empty to the expression and the rows may differ", "{0} の「空」の判定: 値がマップか入れ子のリストのときは、式では空に見えて行がずれることがある";
    /// {0} は列、{1} は `Approx*`。
    ExportApprox = "the condition on {0} is approximate: {1}", "{0} の条件は近似: {1}";
    ApproxDateValue = "the date or date-time value \"{0}\" is compared as a date or time (values with surrounding spaces, or the same date-time written differently, also match)", "日付・日時の形の値「{0}」は日付・時刻として比べる(前後の空白や別の書き方の同じ日時の値も当たる)";
    ApproxMapValue = "the value \"{…}\" (a map or nested list) cannot be matched by the expression", "値「{…}」(マップ・入れ子のリスト)は式で当てられない";
    ApproxContainsDateTime = "\"{0}\" is searched in date-time values padded to seconds (…T10:00:00)", "「{0}」は日時の値を秒まで補った形(…T10:00:00)の中で探す";
    ApproxContainsJoined = "\"{0}\" is searched in the list items joined as text, so it also matches across item boundaries", "「{0}」はリストの要素をつないだ文字の中で探すので、要素の境目にも当たる";
    ApproxContainsMap = "map and nested-list values (\"{…}\" in settings) cannot be searched the same way by the expression", "マップ・入れ子のリストの値(settings では「{…}」)は式で同じに探せない";
    ApproxCmpList = "list values (values of the wrong type) are not kept by the expression (settings compares each item)", "リストの値(型の合わない値)は式では残らない(settings は要素ごとに比べる)";
    ApproxCmpDateLoose = "values with surrounding spaces, space-separated values and values with Z are also read as dates or date-times (values of the wrong type in settings)", "前後に空白のある・空白区切りの・Z 付きの値も日付・日時として読む(settings では型の合わない値)";
    ApproxCmpUnknownType = "the column type is unknown, so values are compared with \"{0}\" as text (for a number or date column, settings keeps no rows)", "列の型が分からないので「{0}」と文字の順で比べる(数・日付の列なら settings ではどの行も残らない)";
    ApproxCmpListJoined = "list values are compared as their items joined as text, not item by item", "リストの値は要素ごとでなく、要素をつないだ文字で比べる";
    ApproxCmpMapEmpty = "map and nested-list values (\"{…}\" in settings) look empty to the expression and are not kept", "マップ・入れ子のリストの値(settings では「{…}」)は式では空に見えて残らない";
    ApproxCmpDateTimePadded = "date-time values are compared padded to seconds (…T10:00:00)", "日時の値は秒まで補った形(…T10:00:00)で比べる";
    /// {0} は formula の名前を `ListSep` と `formula.` でつないだもの。
    ExportFormulaRefs = "references formula.{0}, but mdgrid views have no formulas, so they were not exported (the references were kept but cannot be evaluated)", "formula.{0} を参照しているが、mdgrid のビューは formulas を持たないので書き出していない(参照は残したが評価できない)";
    ExportHiddenDropped = "dropped the hidden columns ({0}): the column order (order) is empty, so .base cannot express them", "隠す列({0})は、列の並び(order)が空なので .base で表せず落とした";
    ExportHideEmptyDropped = "dropped \"hide empty\" of the group ({0}) (NV-21): .base groupBy cannot express it", "グループ({0})の「空を隠す」(NV-21)は .base の groupBy で表せないので落とした";
    ExportDisplayDropped = "dropped the display toggles (row numbers, zebra stripes, etc. SR-20): .base has none", "表示の切り替え(行番号・一行おきの色など。SR-20)は .base に無いので落とした";
    ImportNoView = ".base has no view {0}", ".base にビュー {0} は無い";
    ImportNoType = "the view has no type, so it was imported as table", "ビューに type が無いので table として取り込んだ";
    ImportTypeAsTable = "imported the type: {0} view as table", "type: {0} のビューを table として取り込んだ";
    ImportLimitDropped = "dropped limit: {0} (row limit): mdgrid views have none", "limit: {0}(行数の上限)は mdgrid のビューに無いので落とした";
    ImportFormulasDropped = "dropped formulas ({0}): mdgrid views have none (columns and expressions using formula.* cannot be evaluated)", "formulas({0})は mdgrid のビューに無いので落とした(formula.* を使う列と式は評価できない)";
    ImportDisplayNameDropped = "dropped displayName of properties ({0}): mdgrid views have none", "properties の displayName({0})は mdgrid のビューに無いので落とした";

    // ---- base: .base の誤りと評価できない理由(BV-7・SC-8) ----
    BaseYaml = "cannot read the .base YAML: {0}", ".base の YAML が読めない: {0}";
    BaseNotMap = "the top of the .base is not a map", ".base の先頭がマップでない";
    /// {0} は深さの上限。
    BaseTooDeep = "the YAML is nested more than {0} levels deep", "YAML の入れ子が {0} 段より深い";
    BaseAliasTooBig = "expanding the aliases (*) in the .base is too large (up to {0} nodes)", ".base の別名(*)を展開すると大きすぎる({0} ノードまで)";
    BaseFilterShape = "cannot read the filters (expected an expression string or a map of and, or, not)", "filters の形が読めない(式の文字列か and・or・not のマップのはず)";
    BaseFilterUnknown = "unknown filters form: {0} (expected and, or, not)", "filters の知らない形: {0}(and・or・not のはず)";
    /// {0} は関数・項目、{1} は式。
    BaseUnsupported = "unsupported function or field {0} (expression: {1})", "未対応の関数・項目 {0}(式: {1})";
    BaseExprSyntax = "cannot read the expression: {0} (expression: {1})", "式が読めない: {0}(式: {1})";
    BaseInExpr = "{0} (expression: {1})", "{0}(式: {1})";
    BaseFilterKept = "the filters expression cannot be evaluated by mdgrid (kept, but the view cannot be opened): {0}", "filters の式は mdgrid で評価できない(残したがビューは開けない): {0}";
    BaseFilterDropped = "dropped an unreadable part of the filters: {0}", "filters の読めない部分を落とした: {0}";
    BaseFormulaCycle = "formula.{0} is circular", "formula.{0} が循環";
    BaseFormulaMissing = "formula.{0} is not in formulas", "formula.{0} は formulas に無い";
    BaseUnsupportedField = "unsupported field {0}", "未対応の項目 {0}";
    BaseBadColumn = "cannot read the column: {0}", "読めない列: {0}";
    BaseNoViewIndex = "no view {0}", "ビュー {0} は無い";
    BaseViewNoType = "view \"{0}\" has no type", "ビュー「{0}」に type が無い";
    /// {0} は型、{1} はビューの名前。
    BaseViewUnsupported = "{0} view \"{1}\" is not supported (mdgrid shows table views only)", "{0} のビュー「{1}」は未対応(mdgrid は table のビューだけ)";
    BaseGlobalFilters = "cannot evaluate the global filters: {0}", "全体の filters を評価できない: {0}";
    BaseViewFilters = "cannot evaluate the filters of view \"{0}\": {1}", "ビュー「{0}」の filters を評価できない: {1}";
    BaseCannotSort = "cannot sort by {0} (treated as empty): {1}", "{0} で並べられない(空として扱う): {1}";
    /// 空のグループの見出し(base・settings)。
    EmptyHeading = "(empty)", "(空)";
    /// {0} は列、{1} は集計の名前、{2} は最上位の summaries の式。
    BaseSummaryFormula = "summary of {0}: the formula summary {1} is not supported ({2})", "{0} の集計: 式の集計 {1} は未対応({2})";
    /// {0} は列、{1} は集計の名前。
    BaseSummaryUnknown = "summary of {0}: unknown summary {1}", "{0} の集計: 知らない集計 {1}";
    BaseSummaryGroups = "summaries per group are not supported (the summary row is for the whole table)", "まとまりごとの集計は未対応(集計の行は表全体)";

    // ---- summary: 組み込みの集計の名前(BV-14。英語は Obsidian の名前) ----
    SummaryAverage = "Average", "平均";
    SummaryMin = "Min", "最小";
    SummaryMax = "Max", "最大";
    SummarySum = "Sum", "合計";
    SummaryRange = "Range", "範囲";
    SummaryMedian = "Median", "中央値";
    SummaryStddev = "Stddev", "標準偏差";
    SummaryEarliest = "Earliest", "最早";
    SummaryLatest = "Latest", "最遅";
    SummaryChecked = "Checked", "チェックあり";
    SummaryUnchecked = "Unchecked", "チェックなし";
    SummaryEmpty = "Empty", "空";
    SummaryFilled = "Filled", "空でない";
    SummaryUnique = "Unique", "種類";
    /// 集計の行の、ノートの列の見出し。
    SummaryRow = "Summary", "集計";

    // ---- expr: 式の読み取りの誤り ----
    ExprBadNumber = "not a number: {0}", "数として読めない: {0}";
    ExprNameAfterNumber = "a name right after a number: {0}{1}", "数の直後に名前がある: {0}{1}";
    ExprUnclosedString = "unclosed string", "文字列が閉じていない";
    ExprBadChar = "unexpected character: {0}", "読めない文字: {0}";
    ExprTooDeep = "the expression is nested too deeply", "式の入れ子が深すぎる";
    ExprMissing = "missing {0}", "{0} が無い";
    ExprCommaOrRParen = "`,` or `)`", "`,` か `)`";
    ExprCommaOrRBrack = "`,` or `]`", "`,` か `]`";
    ExprNoNameAfterDot = "no name after `.`", "`.` の後ろに名前が無い";
    ExprUnexpectedEnd = "the expression ends too early", "式が途中で終わっている";
    ExprUnexpectedToken = "unexpected token here: {0}", "ここに置けない記号: {0}";
    ExprArityAtLeast = "{0} or more", "{0} 以上";
    ExprArityRange = "{0} to {1}", "{0}〜{1}";
    /// {0} は関数、{1} は渡した数、{2} は求める数。
    ExprArity = "wrong number of arguments for {0} ({1} given, expected {2})", "{0} の引数の数が違う({1} 個。{2} 個のはず)";
    ExprTrailing = "extra input after the expression: {0}", "式の後ろに余計なものがある: {0}";

    // ---- newnote: 新しいノートの設定と作れない理由(CE-25・CE-26・CE-27) ----
    NewNoteNotTable = "new_note is not a table", "new_note が表でない";
    NewNoteNotString = "new_note.{0} is not a string", "new_note.{0} が文字列でない";
    NewNoteAskNotArray = "new_note.ask is not an array", "new_note.ask が並びでない";
    NewNoteAskNonString = "new_note.ask has a non-string value", "new_note.ask に文字列でない値がある";
    NewNoteSetNotTable = "new_note.set is not a table", "new_note.set が表でない";
    NewNoteSetBadValue = "`{0}` in new_note.set is not a string, number, boolean, date, or a list of them", "new_note.set の `{0}` は文字列・数・真偽・日付・その並びでない";
    /// {0} は `NewNoteWhat*`。
    NewNoteControl = "{0} has a control character", "{0}に制御文字がある";
    NewNoteAbsolute = "{0} is an absolute path (outside the opened folder), so the note is not created", "{0}が絶対パス(開いたフォルダの外)なので作らない";
    NewNoteBadChar = "{0} has `{1}`, which Obsidian does not allow, so the note is not created", "{0}に Obsidian で使えない文字 `{1}` があるので作らない";
    NewNoteDotDot = "{0} has `..` (outside the opened folder), so the note is not created", "{0}に `..` がある(開いたフォルダの外)ので作らない";
    NewNoteDotPart = "`{1}` in {0} starts with `.` (skipped when reading), so the note is not created", "{0}の `{1}` は `.` で始まる(読み込みで飛ばす)ので作らない";
    NewNoteWhatFolder = "the configured folder", "設定のフォルダ";
    NewNoteWhatName = "the name", "名前";
    NewNoteEmptyName = "the name is empty, so the note is not created", "名前が空なので作らない";
    NewNoteNameIsFolder = "the name ends with a folder, so the note is not created", "名前がフォルダで終わっているので作らない";
    NewNoteExists = "{0} already exists, so the note is not created", "{0} は既にあるので作らない";
    NewNoteEmptyKey = "cannot write a value with an empty column name: {0}", "列の名前が空の値は書けない: {0}";
    NewNoteNewline = "a value has a line break, so the note is not created", "値に改行があるので作らない";
    NewNoteNotEditable = "a value cannot be written, so the note is not created: {0}", "書けない値なので作らない: {0}";
    NewNoteCannotBuild = "cannot build the content of the new note: {0}", "新しいノートの中身を作れない: {0}";

    // ---- print: --print の誤り(CLI-5) ----
    PrintNoViews = ".base has no views", ".base に views が無い";
    PrintNativeView = "mdgrid view \"{0}\": {1}", "mdgrid のビュー「{0}」: {1}";

    // ---- settings: 設定の帯の項目(NV-16) ----
    ChipSort = "sort: {0} {1}", "並べ替え: {0} {1}";
    ChipGroupOff = "group: none", "グループ: なし";
    ChipGroup = "group: {0} {1}", "グループ: {0} {1}";
    ChipHideEmpty = " (hide empty)", "(空を隠す)";
    CondOnly = "only {0}", "{0} だけ";
    CondExcept = "except {0}", "{0} を除く";
    CondContains = "contains \"{0}\"", "「{0}」を含む";
    CondNotContains = "does not contain \"{0}\"", "「{0}」を含まない";
    CondEmpty = "is empty", "空である";
    CondNotEmpty = "is not empty", "空でない";

    // ---- display: 「表示」の節の項目(SR-21) ----
    DisplayRowNumbers = "Row numbers", "行番号";
    DisplayZebra = "Zebra stripes", "一行おきの色";
    DisplayColumnLines = "Column lines", "列の区切り線";
    DisplayGroupGap = "Group gaps", "まとまりの間";
    DisplayTabs = "Tabs", "タブ";
    DisplaySearchBar = "Search bar", "検索の欄";
    DisplayChips = "Settings band", "設定の帯";

    // ---- ui/keymap: モードの表示名(SR-16。下の帯・ヘルプ・警告) ----
    ModeTable = "Table", "表";
    ModeEdit = "Edit", "編集";
    ModeConfirm = "Save review", "保存の確認";
    ModeQuit = "Confirm quit", "終了の確認";
    ModeHelp = "Help", "ヘルプ";
    ModePalette = "Palette", "パレット";
    ModeSearch = "Search", "検索";
    ModeFilter = "Filter", "絞り込み";
    ModeDetail = "Details", "詳細の表示";
    ModeSettings = "View settings", "ビューの設定";
    ModeSettingsText = "Settings input", "設定の値の入力";
    ModeChips = "Settings band", "設定の帯";
    ModeListPick = "List picker", "リストの選択";
    ModeMenu = "Action menu", "操作の一覧";
    ModeFreq = "Value counts", "頻度表";

    // ---- ui/keymap: ヘルプの節(SR-5) ----
    SecMove = "Move", "移動";
    SecEdit = "Edit", "編集";
    SecFile = "Save and quit", "保存と終了";
    SecInput = "Input box", "入力ボックス";
    SecReview = "Save review", "保存の確認";
    SecQuit = "Confirm quit", "終了の確認";
    SecFind = "Help and commands", "ヘルプとコマンド";
    SecOutside = "Editor and copy", "エディタとコピー";
    SecInHelp = "In help", "ヘルプの中";
    SecInPalette = "In the palette", "パレットの中";
    SecShape = "Table layout", "表の形";
    SecSearch = "Search and filter", "検索と絞り込み";
    SecSelect = "Selection and details", "選択と詳細";
    SecInPrompt = "Search and filter input", "検索・絞り込みの入力";
    SecInDetail = "In details", "詳細の表示の中";
    SecInSettings = "In view settings", "ビューの設定の中";
    SecInChips = "In the settings band", "設定の帯の中";
    SecInPick = "In the list picker", "リストの選択の中";
    SecNative = "mdgrid views (palette commands)", "mdgrid のビュー(パレットのコマンド)";
    SecNote = "Create notes (palette commands)", "ノートを作る(パレットのコマンド)";
    SecInMenu = "In the action menu", "操作の一覧の中";
    SecInFreq = "In value counts", "頻度表の中";

    // ---- ui/keymap: キーの表示名(SR-4。下の帯・ヘルプ・パレット) ----
    KeyUp = "Up", "上";
    KeyDown = "Down", "下";
    KeyLeft = "Left", "左";
    KeyRight = "Right", "右";
    KeyFirstColumn = "First column", "列の先頭";
    KeyLastColumn = "Last column", "列の末尾";
    KeyPageUp = "Page up", "1画面上";
    KeyPageDown = "Page down", "1画面下";
    KeyHalfPageUp = "Half page up", "半画面上";
    KeyHalfPageDown = "Half page down", "半画面下";
    KeyFirstRow = "First row", "先頭の行";
    KeyLastRow = "Last row", "末尾の行";
    KeyNextCell = "Next cell", "右のセル";
    KeyPrevCell = "Previous cell", "左のセル";
    KeyEdit = "Edit", "編集";
    KeyClear = "Clear", "空にする";
    /// 表の取り消し(undo)。
    KeyUndo = "Undo", "取り消し";
    KeyRedo = "Redo", "やり直し";
    KeyNewNote = "New note", "新しいノート";
    KeySave = "Save", "保存";
    KeyQuit = "Quit", "終了";
    KeyCancelLoad = "Stop loading", "読み込みの中止";
    KeyHelp = "Help", "ヘルプ";
    KeyCommands = "Commands", "コマンド";
    KeyOpenEditor = "Open in editor", "エディタで開く";
    KeyCopyCell = "Copy cell", "セルをコピー";
    KeyCopyRow = "Copy row", "行をコピー";
    KeyPrevView = "Previous view", "前のビュー";
    KeyNextView = "Next view", "次のビュー";
    KeyNarrower = "Narrow column", "列を狭く";
    KeyWider = "Widen column", "列を広く";
    KeySort = "Sort for now", "一時的な並べ替え";
    KeyHideColumn = "Hide column", "列を隠す";
    KeyShowColumn = "Show hidden column", "隠した列を戻す";
    KeyAddColumn = "Add a column (a new key)", "列を足す(新しいキー)";
    KeyMoveColumnLeft = "Move column left", "列を左へ";
    KeyMoveColumnRight = "Move column right", "列を右へ";
    KeyFreeze = "Freeze columns to here", "ここまでの列を固定";
    KeySearch = "Search", "検索";
    KeySearchNext = "Next match", "次の一致";
    KeySearchPrev = "Previous match", "前の一致";
    KeyQuickFilter = "Quick filter", "簡易の絞り込み";
    KeyHighlightSame = "Highlight same value", "同じ値の行を強調";
    KeyFilterSame = "Only same value", "同じ値の行だけ";
    KeyFrequency = "Value counts", "頻度表";
    KeyFreqRun = "Only this value", "この値の行だけ";
    KeyMark = "Mark row", "行の印";
    KeyVisual = "Select range", "範囲で選ぶ";
    KeyExtendUp = "Extend up", "範囲を上へ";
    KeyExtendDown = "Extend down", "範囲を下へ";
    KeySelectAll = "Select all", "全部を選ぶ";
    KeyEscape = "Clear selection/filter", "選択・絞り込みを解く";
    KeyDetail = "Details", "詳細の表示";
    KeyViewSettings = "View settings", "ビューの設定";
    KeyFocusChips = "Select settings band", "設定の帯を選ぶ";
    KeyCommit = "Confirm", "確定";
    /// 入力の取り消し(cancel)。
    KeyCancel = "Cancel", "取り消し";
    KeyCommitNext = "Confirm, go right", "確定して右へ";
    KeyCommitPrev = "Confirm, go left", "確定して左へ";
    KeyRevert = "Original value", "編集前の値";
    KeyClearInput = "Clear the input", "入力を消す";
    KeyListUpWeek = "Previous item/week", "前の候補・前の週";
    KeyListDownWeek = "Next item/week", "次の候補・次の週";
    KeyLeftDay = "Left/previous day", "左・前の日";
    KeyRightDay = "Right/next day", "右・次の日";
    KeyDatePrevMonth = "Date: previous month", "日付: 前の月";
    KeyDateNextMonth = "Date: next month", "日付: 次の月";
    KeyLeftMonth = "Left/previous month", "左・前の月";
    KeyRightMonth = "Right/next month", "右・次の月";
    KeyListUpYear = "Previous item/year", "前の候補・前の年";
    KeyListDownYear = "Next item/year", "次の候補・次の年";
    KeyDateToday = "Date: today", "日付: 今日";
    KeyDateClear = "Date: clear and confirm", "日付: 空にして確定";
    KeyTimeFocus = "Datetime: switch between day and time", "日時: 日と時刻を切り替え";
    /// 入力の文字のカーソルの先頭・末尾。
    KeyCursorHome = "Start of text", "先頭";
    KeyCursorEnd = "End of text", "末尾";
    KeyDeleteBack = "Delete previous char", "前の字を消す";
    KeyDeleteForward = "Delete next char", "後の字を消す";
    KeyFilterClear = "Clear filter", "解除";
    KeyClose = "Close", "閉じる";
    /// 詳細の表示・ヘルプの先頭・末尾。
    KeyTop = "Top", "先頭";
    KeyBottom = "Bottom", "末尾";
    KeyDecide = "OK", "決める";
    KeyToggle = "Toggle", "切り替え";
    KeyBackCancel = "Back/cancel", "戻る・取り消し";
    KeyNextSection = "Next section", "次の区画";
    KeyPrevSection = "Previous section", "前の区画";
    KeyMoveItemUp = "Move up", "上へ動かす";
    KeyMoveItemDown = "Move down", "下へ動かす";
    KeyDelete = "Delete", "消す";
    KeyBack = "Back", "戻る";
    KeyRemove = "Remove", "外す";
    KeyLeftItem = "Left item", "左の項目";
    KeyRightItem = "Right item", "右の項目";
    KeyBackToTable = "Back to table", "表に戻る";
    KeyPickRun = "Confirm (add while searching)", "確定(検索中は付ける)";
    KeyPickToggle = "Toggle (space while searching)", "付け外し(検索中は空白)";
    KeyPickRevert = "Back to as opened", "開いたときに戻す";
    KeyPrevItem = "Previous item", "前の候補";
    KeyNextItem = "Next item", "次の候補";
    /// 保存の確認で全部を書く。
    KeySaveAll = "Save all", "保存する";
    KeyNextFile = "Next file", "次のファイル";
    KeyPrevFile = "Previous file", "前のファイル";
    KeyOverwrite = "Write this file over the outside change", "このファイルを外の変更の上に書く";
    KeyDiscardRow = "Discard pending changes", "ためた変更を捨てる";
    /// 終了の確認で保存する。
    KeyQuitSave = "Save and quit", "保存する";
    KeyQuitDiscard = "Discard and quit", "捨てて終わる";
    KeyScrollDown = "Scroll down", "下へ流す";
    KeyScrollUp = "Scroll up", "上へ流す";
    KeyRun = "Run", "実行";
    /// 読むだけのときの表の Enter(SR-2)。
    KeyToggleGroup = "Open/close group", "見出しの開閉";
    /// `--pick` のときの表の Enter(OUT-3)。
    KeyPickFinish = "Pick and finish", "選んで終わる";
    /// その場の操作の一覧を開く(SR-24)。
    KeyActionMenu = "Action menu", "操作の一覧";
    CmdExportBase = "Export to .base", ".base に書き出す";
    CmdRenameKey = "Rename the key", "キーの名前を変える";
    CmdDeleteKey = "Delete the key", "キーを消す";
    CmdImportBase = "Import a .base view", ".base のビューを取り込む";

    // ---- ui/keymap: キーの読み取りと割り当て直しの警告(SR-13・SR-16) ----
    KeyErrEmpty = "the key is empty", "キーが空";
    KeyErrModifier = "unknown modifier `{0}`", "知らない修飾キー `{0}`";
    KeyErrSpace = "write the space key as `space`", "空白のキーは `space` と書く";
    KeyErrShiftChar = "for shift+{0}, write the character shift produces", "shift+{0} は、shift で出る文字そのものを書く";
    KeyErrNoKey = "no key in `{0}`", "キーが無い `{0}`";
    KeyErrUnknown = "unknown key `{0}`", "知らないキー `{0}`";
    KeyErrSpaceModifier = "space cannot take a modifier", "space に修飾キーは付けられない";
    KeyErrShiftSame = "shift+{0} cannot be told apart", "shift+{0} は区別できない";
    /// {0} はモード、{1} は動作の名前を `` `、` `` でつないだもの、{2} は戻したキー。
    RebindExitKept = "the settings would leave no key for `{1}` in {0} mode, so they were not applied and the default keys {2} were kept", "設定で{0}のモードの `{1}` のキーが無くなるので、その設定は当てず既定のキー {2} を残した";
    RebindUnknownMode = "`keys.{0}` in the settings is not a known mode, so it was ignored (modes: {1})", "設定の `keys.{0}` は知らないモードなので無視した(モード: {1})";
    RebindBadKey = "`{0}` in the settings is not a readable key, so it was ignored ({1})", "設定の `{0}` は読めないキーなので無視した({1})";
    RebindNoPrefix = "`{0}` in the settings cannot be a prefix key in {1} mode, so it was ignored", "設定の `{0}` は{1}のモードでは前置きのキーにできないので無視した";
    RebindUnknownAction = "the action `{1}` of `{0}` in the settings is unknown, so it was ignored", "設定の `{0}` の動作 `{1}` は知らないので無視した";
    RebindNotInMode = "the action `{1}` of `{0}` in the settings cannot be used in {2} mode, so it was ignored", "設定の `{0}` の動作 `{1}` は{2}のモードでは使えないので無視した";
    /// {0} は設定の場所を `ListSep` で並べたもの、{1} はモード、{2} はキー。
    RebindSameKey = "{0} in the settings give two actions to the same key `{2}` in {1} mode, so none were applied (SR-16)", "設定の {0} は{1}のモードの同じキー `{2}` に2つの動作を当てるので、どれも無視した(SR-16)";
    /// {2} はぶつかるキー、{3} はその動作の名前。
    RebindPrefixClash = "`{0}` in the settings overlaps as a prefix with the key `{2}` ({3}) in {1} mode, so it was ignored (SR-16; remove that first with \"none\")", "設定の `{0}` は{1}のモードのキー `{2}`({3})と前置きが重なるので無視した(SR-16。先に \"none\" で外す)";

    // ---- ui/help: ヘルプとパレット(SR-5・SR-14・NV-7) ----
    HelpNow = " Keys now ({0})", " 今押せるキー({0})";
    /// SR-5: ヘルプの最後の印の節。
    HelpMarks = " Markers", " 印";
    HelpMarksCell = "  In a cell", "  セルの中";
    HelpMarksRow = "  Left of a row", "  行の左";
    MarkNull = "null: the key is there, with no value", "null(キーはあり、値が無い)";
    MarkEmptyStr = "an empty string", "空の文字列";
    MarkBlank = "the note has no such key", "ノートにそのキーが無い";
    MarkBlankSign = "(blank)", "(空欄)";
    MarkMisfit = "the value does not fit the column type", "列の型に合わない値";
    MarkPending = "changed, not saved yet", "直したが、まだ保存していない";
    MarkLock = "read-only (select it to see why)", "読むだけ(選ぶと理由が出る)";
    MarkUnsupported = "a formula mdgrid cannot evaluate", "評価できない式";
    MarkMultiline = "the value has line breaks (first line shown)", "改行を含む値(1行目だけ)";
    MarkRowCurrent = "the row you are on", "今の行";
    MarkRowMarked = "a marked row (Space, v)", "印を付けた行(Space・v)";
    MarkRowExternal = "changed outside mdgrid, or a sync conflict file", "外で変わったノートか、同期の競合ファイル";
    MarkRowHeld = "edited; stays in place until you save or move", "直して元の位置に留めている行(保存か移動まで)";
    /// {0}-{1} は見えている行、{2} は全部の行の数。
    HelpHead = " Help  {0}-{1}/{2} lines  All keys (from the key table)", " ヘルプ  {0}-{1}/{2}行  キーの全部(キーの表から)";
    /// `HelpHead` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    HelpHeadOne = " Help  {0}-{1}/{2} line  All keys (from the key table)", " ヘルプ  {0}-{1}/{2}行  キーの全部(キーの表から)";
    PaletteGoto = "Go to line {0}", "{0} 行目へ移動";
    PaletteEmpty = "  (no matching command)", "  (合うコマンドが無い)";
    PaletteNoMatch = "no matching command: {0}", "合うコマンドが無い: {0}";
    GotoNoRows = "no rows", "行が無い";
    GotoPastEnd = "there is no line {0} ({1} rows). Moved to the last row", "{0} 行目は無い(全 {1} 行)。末尾の行へ移った";
    /// `GotoPastEnd` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    GotoPastEndOne = "there is no line {0} ({1} row). Moved to the last row", "{0} 行目は無い(全 {1} 行)。末尾の行へ移った";
    GotoFolded = "line {0} is in a folded group (moved to its heading)", "{0} 行目は畳んだグループの中(見出しへ移った)";

    // ---- ui/bands: ヘッダー・タブ・検索の欄・設定の帯・下の帯・メッセージ行(SR-1・SR-15・NV-23・NV-16) ----
    HeaderRows = "{0} rows", "{0}行";
    /// `HeaderRows` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    HeaderRow = "{0} row", "{0}行";
    /// {1} は定義と違えば `*`。
    HeaderView = "  view {0}{1}", "  ビュー {0}{1}";
    HeaderLoading = "  loading {0}/{1}", "  読み込み中 {0}/{1}";
    HeaderCancelKey = " ({0} to stop)", "({0} で中止)";
    HeaderCancelled = "  loading stopped", "  読み込みを止めた";
    HeaderFilter = "  filter \"{0}\"", "  絞り込み「{0}」";
    HeaderSame = "  same value {0}={1}", "  同じ値 {0}={1}";
    /// リストの列の要素で絞ったとき(NV-9)。{0} は列、{1} は値。
    HeaderSameHas = "  {0} contains {1}", "  {0} が {1} を含む";
    HeaderHidden = "  hidden columns {0}", "  隠した列 {0}";
    TabDefault = " Default ", " 既定の表 ";
    TabSwitch = "  [ ] to switch", "  [ ] で切り替え";
    BarLead = " Filter: ", " 検索: ";
    BarCount = "  {0}/{1} rows", "  {0}/{1}行";
    /// `BarCount` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BarCountOne = "  {0}/{1} row", "  {0}/{1}行";
    BarKey = "({0} to filter)", "({0} で絞る)";
    BarClick = "(click to filter)", "(クリックで絞る)";
    ChipsLead = " Settings:", " 設定:";
    ChipsRemoveKey = "  {0} to remove", "  {0} で外す";
    ChipsSelectKey = "  {0} to select", "  {0} で選ぶ";
    FooterUnsaved = " Unsaved {0}", " 未保存 {0}";
    FooterSelected = "  selected {0}", "  選択 {0}";
    FooterRow = "row {0}/{1}", "{0}/{1}行";
    FooterHead = "group {0}/{1}", "見出し {0}/{1}";
    FooterCol = " col {0}/{1}", " {0}/{1}列";
    FooterUnsupported = "  ?=unsupported formula", "  ?=未対応の式";
    HintReviewExternal = "changed outside: o writes this file over the outside change, d discards the pending changes", "外で変わったノート: o でこのファイルを外の変更の上に書く、d でためた変更を捨てる";
    HintReview = "Enter writes everything. Esc goes back to the table (pending changes stay)", "Enter で全部を書く。Esc で表に戻る(ためた変更は残る)";
    HintViewError = "this view cannot be opened (reason in the table area). [ ] for other views", "このビューは開けない(理由は表の欄)。[ ] でほかのビューへ";
    /// {0} は理由を ` / ` でつないだもの。
    HintUnsupported = "unsupported: {0}", "未対応: {0}";
    /// BV-2: `.base` の上に `.obsidian/` が無く、0行のとき。{0} は探した根。
    VaultRootGuessed = "no rows: no .obsidian/ folder above the .base, so notes were searched only under {0}", "行が無い: .base の上に .obsidian/ が無いので、{0} の下だけを探した";
    HintHeadRow = "group heading ({0} rows). Enter opens/closes", "見出しの行({0}件)。Enter で開閉";
    /// `HintHeadRow` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    HintHeadRowOne = "group heading ({0} row). Enter opens/closes", "見出しの行({0}件)。Enter で開閉";
    HintExternalNote = "the note changed outside (pending changes are kept)", "外でノートが変わった(ためた変更は残っている)";
    /// {0} は印。
    HintHeld = "{0}: with the edited value this row would not be here. Save or move to re-sort", "{0}: 直した値ではこの位置に来ない行。保存か行の移動で並び直す";
    HintUnsupportedFormula = "unsupported formula (no value shown): {0}", "未対応の式(値を出さない): {0}";
    HintReadOnly = "read-only: {0}", "読むだけ: {0}";
    QuitBand = " {0} unsaved change(s). Choose what to do before quitting", " 未保存の変更が {0} ある。終わる前にどうするか選ぶ";

    // ---- ui/calendar: カレンダー(CE-20・CE-21・CE-23・CE-24) ----
    /// {0} は年、{1} は月(`Month*`)。
    CalTitle = " {1} {0} ", " {0}年{1} ";
    Month1 = "January", "1月";
    Month2 = "February", "2月";
    Month3 = "March", "3月";
    Month4 = "April", "4月";
    Month5 = "May", "5月";
    Month6 = "June", "6月";
    Month7 = "July", "7月";
    Month8 = "August", "8月";
    Month9 = "September", "9月";
    Month10 = "October", "10月";
    Month11 = "November", "11月";
    Month12 = "December", "12月";
    WdSun = "Su", "日";
    WdMon = "Mo", "月";
    WdTue = "Tu", "火";
    WdWed = "We", "水";
    WdThu = "Th", "木";
    WdFri = "Fr", "金";
    WdSat = "Sa", "土";
    CalMonthKeys = "{0} month", "{0} 月";
    CalYearKeys = "{0} year", "{0} 年";
    /// CE-31: 時刻の欄への切り替えのキー。
    CalTimeKey = "{0} time", "{0} 時刻";
    /// CE-30: 時刻の欄の見出し。
    CalTime = "time", "時刻";
    /// CE-30: 時刻の欄は日時の列だけ。
    CalTimeOnly = "the time can be set only in datetime inputs", "時刻は日時の列の入力だけ";
    CalTodayKey = "{0} today", "{0} 今日";
    CalClearKey = "{0} clear", "{0} 空";
    CalDateOnly = "this works only in date and datetime inputs", "この操作は日付・日時の列の入力だけ";

    // ---- ui/new_note: 新しいノート(CE-25・CE-26・CE-27) ----
    NoteButton = "+ New", "+ 新規";
    NoteNoPlace = "the place to create it (the opened folder) is unknown, so the note is not created", "作る場所(開いたフォルダ)が分からないので作らない";
    NoteTooLow = "the terminal is too short for the new note fields, so it did not start (make the terminal taller)", "端末が低くて新しいノートの欄を出せないので始めない(端末を高くする)";
    NoteFirstPlace = "the place is the first field", "作る場所が最初の項目";
    NoteFirstName = "the name is the first field", "名前が最初の項目";
    /// {0} はパス、{1} は理由。
    NoteCannotCreate = "cannot create {0}, so the note is not created: {1}", "{0} を作れないので作らない: {1}";
    NoteUnreadable = "created but cannot read it: {0}", "作ったが読めない: {0}";
    NoteCreated = "created: {0}", "作った: {0}";
    NoteCreatedHidden = "created: {0} (not visible with the current filter or folded groups)", "作った: {0}(今の絞り込みか畳んだまとまりでは見えない)";
    NoteCancelled = "new note cancelled (nothing created)", "新しいノートを取りやめた(作らない)";
    NoteLeadPlace = " New note  place: ", " 新しいノート 作る場所: ";
    NoteLeadName = " New note  name: ", " 新しいノート 名前: ";
    NoteLeadCol = " New note  {0}: ", " 新しいノート {0}: ";
    NoteCreate = "Create", "作る";
    NoteNext = "Next", "次へ";
    NoteToggle = "Toggle", "付け外し";
    NoteStop = "Cancel", "やめる";
    NotePrevField = "Previous field", "前の項目";
    NoteHintPlace = "↑↓ choose the place, Enter to confirm", "↑↓ で作る場所を選ぶ・Enter で決める";
    NoteHintFree = "free input, empty writes nothing (↑↓ back to the list)", "自由入力・空なら書かない(↑↓ でリストに戻る)";
    NoteHintList = "↑↓ choose, Enter confirm, type for free input (*=current value), none = write nothing", "↑↓ で選ぶ・Enter で決める・打つと自由入力(*=今の値)・なし = 書かない";
    NoteHintNumber = "number: a value read as a number, empty writes nothing", "数: 数として読める値・空なら書かない";
    NoteHintCheckbox = "true or false, empty writes nothing", "true か false・空なら書かない";
    /// {0} は使える形。
    NoteHintDate = "date: {0}, +3 / -2 (from today), empty writes nothing", "日付: {0}・+3 / -2(今日から)・空なら書かない";
    NoteHintDateTime = "datetime: YYYY-MM-DDTHH:MM, {0}, +3, empty writes nothing", "日時: YYYY-MM-DDTHH:MM・{0}・+3・空なら書かない";
    /// {0}/{1} は何番目か、{2} はパス。
    NoteGuidePlace = "   choose the place {0}/{1}: {2}", "   作る場所を選ぶ {0}/{1}: {2}";
    /// {0} はフォルダの名前、{1} はパス。
    NoteGuideWhere = "create in {0} ({1})", "{0} に作る({1})";
    /// {0} は名前、{1}/{2} は何番目か。
    NoteGuideField = "field {1}/{2} of {0} (empty writes nothing)", "{0} の項目 {1}/{2}(空なら書かない)";

    // ---- ui/app: 表の操作のメッセージ(WB-1・SR-12) ----
    NothingToSave = "no changes to save", "保存する変更が無い";
    NothingToUndo = "nothing to undo", "取り消す変更が無い";
    NothingToRedo = "nothing to redo", "やり直す変更が無い";
    LoadAlreadyDone = "loading has already finished", "読み込みは終わっている";
    LoadStopped = "stopped loading ({0} read)", "読み込みを止めた({0} 件を読んだ)";

    // ---- ui/columns: 一時的な並べ替え・列を隠す・固定(NV-4) ----
    SortedAsc = "sorted by {0} ascending (the .base is unchanged)", "{0} で昇順に並べた(.base は変えない)";
    SortedDesc = "sorted by {0} descending (the .base is unchanged)", "{0} で降順に並べた(.base は変えない)";
    SortCleared = "sort cleared (.base order)", "並べ替えを解いた(.base の並び)";
    LastColumnHide = "cannot hide the last column", "最後の列は隠せない";
    ColumnHidden = "hid column: {0} (+ to show)", "列を隠した: {0}(+ で戻す)";
    NoHiddenColumns = "no hidden columns", "隠した列が無い";
    ColumnShown = "showed column: {0}", "列を戻した: {0}";
    FreezeCleared = "unfroze the columns", "列の固定を解いた";
    FreezeTooWide = "the left {0} columns do not fit on the screen, so they were not frozen (narrow the columns or pick a column further left)", "左の {0} 列は画面に収まらないので固定しない(列を狭くするか、左の列を選ぶ)";
    /// `FreezeTooWide` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    FreezeTooWideOne = "the left {0} column does not fit on the screen, so it was not frozen (narrow the column or pick a column further left)", "左の {0} 列は画面に収まらないので固定しない(列を狭くするか、左の列を選ぶ)";
    Frozen = "froze the left {0} columns", "左の {0} 列を固定した";
    /// `Frozen` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    FrozenOne = "froze the left {0} column", "左の {0} 列を固定した";

    // ---- ui/detail: 詳細の表示(SR-10) ----
    DetailNoRow = "select a row to show details (not a group heading)", "詳細を出す行を選ぶ(見出しの行では出せない)";
    DetailHead = " Details  {0}  {1}/{2}", " 詳細  {0}  {1}/{2}";
    DetailBody = "Body", "本文";

    // ---- ui/display: 隠した設定の帯の条件(SR-20・NV-22) ----
    ChipRemoved = "removed the band condition: {0}", "帯の条件を外した: {0}";
    /// 2つの知らせをつなぐ。
    TwoMessages = "{0}. {1}", "{0}。{1}";
    HiddenChip = "hidden band condition {0}/{1}: [{2}]  {3} to remove", "隠した帯の条件 {0}/{1}: [{2}]  {3} で外す";

    // ---- ui/entry: セルの入力の読み方と案内(CE-2・CE-5・CE-7・CE-22) ----
    EntryNotNumber = "not a number: {0} (a number, or empty to clear)", "数として読めない: {0}(数か、空で消す)";
    EntryNotBool = "not true or false: {0} (empty to clear)", "true か false でない: {0}(空で消す)";
    /// {1} は打ち込める日付の形。
    EntryNotDateTime = "not a date: {0} (YYYY-MM-DDTHH:MM or {1}, +3, -2, empty to clear)", "日付として読めない: {0}(YYYY-MM-DDTHH:MM か {1}・+3・-2・空で消す)";
    EditBulkLead = "bulk {0} rows: ", "一括 {0}行: ";
    /// `EditBulkLead` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    EditBulkLeadOne = "bulk {0} row: ", "一括 {0}行: ";
    /// CE-10: 印を付けた行の外で編集を始めたとき。
    EditOnlyThisRow = "editing this row only: the {0} marked rows are not included (press Enter on a marked row to edit them all)", "この行だけを直す: 印を付けた {0}行は入れない(印の行の上で Enter なら全部に入れる)";
    /// `EditOnlyThisRow` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    EditOnlyThisRowOne = "editing this row only: the {0} marked row is not included (press Enter on the marked row to edit it)", "この行だけを直す: 印を付けた {0}行は入れない(印の行の上で Enter なら全部に入れる)";
    EditMismatch = "the current value does not fit the column type. ", "今の値は列の型に合わない。";
    EditHintList = "↑↓ choose, Enter confirm, type for free input (*=current value)", "↑↓ で選ぶ・Enter で決める・打つと自由入力(*=今の値)";
    EditHintNumber = "number: a value read as a number, empty to clear", "数: 数として読める値・空で消す";
    EditHintCheckbox = "true or false, empty to clear (↑↓ for the list)", "true か false・空で消す(↑↓ でリスト)";
    EditHintDate = "date: {0}, +3 / -2 (from today), empty to clear", "日付: {0}・+3 / -2(今日から)・空で消す";
    EditHintDateTime = "datetime: YYYY-MM-DDTHH:MM, {0}, +3, empty to clear", "日時: YYYY-MM-DDTHH:MM・{0}・+3・空で消す";
    EditHintBackToList = " (↑↓ back to the list)", "(↑↓ でリストに戻る)";
    /// CE-3: 打った文字で絞った候補を見せているとき。
    EditHintPickNarrowed = " (↑↓ pick a matching value)", "(↑↓ で当たる候補を選ぶ)";

    // ---- ui/external: コピーとエディタ(OUT-1・SR-8) ----
    /// 試験のビルドの偽のクリップボードが返す誤り。
    ClipTestFailure = "test failure", "試験の失敗";
    ClipNoCommand = "{0} not found", "{0} が無い";
    ClipNoOsClipboard = "cannot send to the clipboard of this OS", "この OS のクリップボードには送れない";
    /// 並べた理由の区切り。
    ReasonSep = "; ", "、";
    ClipTimeout = "the OS clipboard does not respond", "OS のクリップボードが応答しない";
    ClipCannotPass = "cannot pass the text: {0}", "渡せない: {0}";
    ClipExitCode = "exit status {0}", "終了コード {0}";
    /// {0} は base64 のバイト数、{1} は上限。
    Osc52TooLong = "too long, so not sent ({0} bytes in base64, limit {1})", "長すぎるので送らない(base64 で {0} バイト、上限 {1})";
    WordsTrailingBackslash = "nothing after the trailing \\", "末尾の \\ の後ろに文字が無い";
    WordsUnclosedSingle = "unclosed '", "閉じていない ' がある";
    WordsUnclosedDouble = "unclosed \"", "閉じていない \" がある";
    CopyWhatSelected = "{0} selected rows", "選んだ {0} 行";
    /// `CopyWhatSelected` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    CopyWhatSelectedOne = "{0} selected row", "選んだ {0} 行";
    CopyWhatCell = "the cell", "セル";
    CopyWhatRow = "the row", "行";
    CopyNoCell = "no cell to copy", "コピーするセルが無い";
    CopyNoRow = "no row to copy", "コピーする行が無い";
    /// {0} は `CopyWhat*`、{1} は誤り。
    CopiedBoth = "copied {0} (OSC 52 and the OS clipboard)", "{0}をコピーした(OSC 52 と OS のクリップボード)";
    CopiedOscOnly = "copied {0} (OSC 52 only; cannot send to the OS clipboard: {1})", "{0}をコピーした(OSC 52 だけ。OS のクリップボードに送れない: {1})";
    CopiedOsOnly = "copied {0} (OS clipboard only; cannot send OSC 52: {1})", "{0}をコピーした(OS のクリップボードだけ。OSC 52 を送れない: {1})";
    CopyFailed = "cannot copy (OSC 52: {0}; OS clipboard: {1})", "コピーできない(OSC 52: {0}。OS のクリップボード: {1})";
    EditorNoRow = "no row to open", "開く行が無い";
    EditorBadSetting = "cannot read the editor setting: {0}", "エディタの設定を読めない: {0}";
    EditorNoTerminal = "cannot restore the terminal, so the editor was not opened: {0}", "端末を戻せないのでエディタを開かない: {0}";
    /// {0} はコマンド、{1} は誤り。
    EditorCannotStart = "cannot start the editor: {0} ({1})", "エディタを起動できない: {0}({1})";
    /// {0} は終了の状態、{1} はノート。
    EditorFailed = "the editor exited with a failure ({0}): {1}", "エディタが失敗で終わった({0}): {1}";
    EditorReturned = "back from the editor: {0}", "エディタから戻った: {0}";
    EditorTermNotRestored = ". Cannot restore the terminal: {0}", "。端末を戻せない: {0}";
    EditorCannotReload = ". Cannot reload: {0}", "。読み直せない: {0}";

    // ---- ui/startup・list・grid・view: 表の画面(WB-15・SR-12・BV-5・BV-7・CE-10) ----
    OpenedReadOnly = "opened read-only (--readonly: no editing, no saving)", "読むだけで開いている(--readonly。編集も保存もしない)";
    StateUnwritable = "cannot write the view state: {0}", "見た目の状態を書けない: {0}";
    /// リストの候補の「なし」(Null)。
    NoneLabel = "(empty)", "なし";
    InputClosedRowGone = "the edited row left the table, so the input was closed ({0}; unconfirmed text was discarded)", "直した行が表から外れたので入力を閉じた({0}。確定していない文字は捨てた)";
    /// ノートの列の見出し。
    NoteColumn = "Note", "ノート";
    /// {0} は選びの印、{1} は開閉の印、{2} は列の名前と `: `、{3} は値、{4} は件数。
    HeadingRow = "{0}{1} {2}{3} ({4})", "{0}{1} {2}{3}({4}件)";
    ViewCannotOpen = "(this view cannot be opened: {0})", "(このビューは開けない: {0})";
    NoNotes = "(no notes)", "(ノートがありません)";
    TableLoading = "(loading…)", "(読み込み中…)";

    // ---- ui/input: セルの編集と一括の設定(CE-1・CE-9・CE-10・CE-11・SR-3) ----
    CellNotDrawn = "the selected cell is not visible on the screen, so it is not edited (widen the terminal or narrow the columns)", "選んだセルが画面に見えていないので直さない(端末を広げるか、列を狭くする)";
    CellNotDrawnEsc = "the selected cell is not visible on the screen, so it is not edited (widen the terminal or narrow the columns) (Esc to cancel)", "選んだセルが画面に見えていないので直さない(端末を広げるか、列を狭くする)(Esc で取り消す)";
    /// 読むだけの理由の前置き(一括で飛ばした理由では外す)。
    ReadOnlyLead = "read-only: ", "読むだけ: ";
    BulkFoldedOut = ". {0} rows are in folded groups, so they were left out", "。{0}行は畳んだまとまりの中なので入れない";
    /// `BulkFoldedOut` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BulkFoldedOutOne = ". {0} row is in a folded group, so it was left out", "。{0}行は畳んだまとまりの中なので入れない";
    /// {1} は理由を ` / ` でつないだもの。
    BulkSkipped = ". Skipped {0} rows ({1})", "。{0}行を飛ばした({1})";
    /// `BulkSkipped` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BulkSkippedOne = ". Skipped {0} row ({1})", "。{0}行を飛ばした({1})";
    NeighborRowGone = "the edited row left the table, so the next cell was not opened ({0})", "直した行が表から外れたので隣のセルは開かない({0})";
    BulkNoEditable = "no editable cell in the selected rows ({0})", "選んだ行に直せるセルが無い({0})";
    EditMultiLine = "a value with line breaks cannot be edited in a one-line input (open it in the editor; a later version)", "改行を含む値は1行の入力では直せない(エディタで開く。後の版)";
    EditNotOneLine = "this value cannot be edited in a one-line input", "この値は1行の入力では直せない";
    NoEditableRight = "no editable cell to the right in this row ({0})", "この行の右に直せるセルが無い({0})";
    NoEditableLeft = "no editable cell to the left in this row ({0})", "この行の左に直せるセルが無い({0})";
    NoCellRight = "no cell to the right in this row", "この行の右にセルが無い";
    NoCellLeft = "no cell to the left in this row", "この行の左にセルが無い";
    CannotCommit = "cannot confirm: {0}", "確定できない: {0}";
    BulkSet = "bulk: set {0} rows", "一括: {0}行に入れた";
    /// `BulkSet` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BulkSetOne = "bulk: set {0} row", "一括: {0}行に入れた";
    BulkSame = " ({0} rows already had the value)", "({0}行は同じ値)";
    /// `BulkSame` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BulkSameOne = " ({0} row already had the value)", "({0}行は同じ値)";
    BulkTypeMismatch = "type mismatch", "型が合わない";
    /// 一括で空にしたときの {1}。
    BulkStateEmpty = "empty", "空";
    BulkSetTo = "bulk: set {0} rows to {1}", "一括: {0}行を{1}にした";
    /// `BulkSetTo` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    BulkSetToOne = "bulk: set {0} row to {1}", "一括: {0}行を{1}にした";
    CannotClear = "cannot clear: {0}", "空にできない: {0}";

    // ---- ui/listpick: リストの選択(CE-16・CE-17・CE-19) ----
    NotAList = "not a list value", "リストでない値";
    PickNotDrawn = "the selected cell is not visible on the screen, so it is not confirmed (widen the terminal, or Esc to cancel)", "選んだセルが画面に見えていないので確定しない(端末を広げるか、Esc で取り消す)";
    PickBulkDone = "bulk: edited {0} rows", "一括: {0}行を直した";
    /// `PickBulkDone` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    PickBulkDoneOne = "bulk: edited {0} row", "一括: {0}行を直した";
    PickBulkSame = " ({0} rows unchanged)", "({0}行は変わらない)";
    /// `PickBulkSame` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    PickBulkSameOne = " ({0} row unchanged)", "({0}行は変わらない)";
    /// {0}・{1} はキー。
    PickKeysSearching = "{0} add (new items are added), {1} confirm", "{0} 付ける(新規は足す)・{1} 確定";
    PickKeys = "{0} toggle, {1} confirm", "{0} 付け外し・{1} 確定";
    /// {1} は `PickKeys*`。
    PickHintBulk = "bulk {0} rows ([x]=all rows [-]=some rows [ ]=none). {1}", "一括 {0}行([x]=全部の行 [-]=一部の行 [ ]=なし)。{1}";
    /// `PickHintBulk` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    PickHintBulkOne = "bulk {0} row ([x]=all rows [-]=some rows [ ]=none). {1}", "一括 {0}行([x]=全部の行 [-]=一部の行 [ ]=なし)。{1}";
    PickHint = "type to narrow ([x]=set). {0}", "打って絞る([x]=付いている)。{0}";
    PickNew = "+ New: {0}", "+ 新規: {0}";
    /// 候補の件数。
    PickCount = "{0}", "{0}件";
    PickSearchLead = "Search: ", "検索: ";
    /// {0} は列、{1} は行の数、{2} は位置。
    PickTitleBulk = "{0} {1} rows {2}", "{0} {1}行 {2}";
    /// `PickTitleBulk` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    PickTitleBulkOne = "{0} {1} row {2}", "{0} {1}行 {2}";
    PickNoMatch = " (no matching item)", " (合う要素が無い)";

    // ---- ui/menu: その場の操作の一覧(SR-24)の節と、キーの無い項目 ----
    MenuSecCell = "Cell", "セル";
    MenuSecColumn = "Column", "列";
    MenuSecRow = "Row", "行";
    MenuSecView = "View and file", "ビューとファイル";
    MenuBulk = "Set on selected rows", "選んだ行にまとめて入れる";
    MenuNoRoom = "the terminal is too small to show the action menu", "端末が狭くて操作の一覧を出せない";

    // ---- ui/freq: 列の値の頻度表(NV-9) ----
    /// {0} は列の名前。
    FreqTitle = "Value counts: {0}", "頻度表: {0}";
    FreqNoRoom = "the terminal is too small to show the value counts", "端末が狭くて頻度表を出せない";
    FreqNoColumn = "select a column to count its values", "値を数える列を選ぶ";
    FreqNoRows = "no rows to count", "数える行が無い";
    FreqLoading = "can't count values while notes are loading", "読み込みの途中は値を数えられない";
    FreqClosed = "the rows changed, so the value counts were closed", "行が変わったので頻度表を閉じた";
    FreqStacked = "counted within the same-value filter; Enter narrows it further", "同じ値の絞り込みの中で数えた。Enter でさらに絞る";

    // ---- ui/native_io: .base への書き出しと取り込み(BV-19・BV-20) ----
    AskExportName = "Export to .base, file name: ", ".base に書き出す ファイルの名前: ";
    AskImportFile = ".base to import: ", "取り込む .base: ";
    AskImportView = "views of {0}: ", "{0} のビュー: ";
    AskExportWhere = "  write a new .base under {0} (existing names are not written)", "  {0} の下に新しい .base を書く(既存の名前には書かない)";
    AskNoMatch = "  (nothing matches)", "  (合うものが無い)";
    ExportNoDir = "the folder to export to is unknown", "書き出す先のフォルダが分からない";
    ImportNoDir = "the folder to look for .base files is unknown", "取り込む .base を探すフォルダが分からない";
    ImportNoFiles = "no .base under {0}", "{0} の下に .base が無い";
    AskCancelled = "cancelled", "やめた";
    /// CE-29。{0} は今の名前。
    AskRenameKey = "Rename {0} to: ", "{0} の新しい名前: ";
    /// CE-29。{0} はキー、{1} はノートの数。
    AskDeleteKey = "Delete {0} from {1} notes? Type y and Enter: ", "{0} を {1} 個のノートから消す? y と打って Enter: ";
    /// CE-29。{0} は古い名前、{1} は新しい名前、{2} はノートの数。
    KeyRenamed = "renaming {0} to {1} in {2} notes (saved with Ctrl+S)", "{0} の名前を {2} 個のノートで {1} に変える(保存で書く)";
    /// CE-29。{0} はキー、{1} はノートの数。
    KeyDeleted = "deleting {0} from {1} notes (saved with Ctrl+S)", "{0} を {1} 個のノートから消す(保存で書く)";
    /// CE-29。{0} は新しい名前。
    KeyOpExists = "the key {0} is already there", "{0} のキーが既にある";
    /// CE-29。{0} はキー。
    KeyOpNoNotes = "no note has the key {0}", "{0} のキーを持つノートが無い";
    KeyOpPending = "the cell has an unsaved edit (save or undo it first)", "そのセルに保存していない直しがある(先に保存か取り消し)";
    KeyOpComputed = "file.* and formula.* columns are not keys in the notes", "file.* と formula.* の列はノートのキーでない";
    KeyOpBadName = "a key name cannot be empty, contain a line break, or start with file. or formula.", "キーの名前は、空・改行を含む・file. か formula. で始まる名前にできない";
    /// CE-28。
    AskNewColumn = "New column (key name): ", "新しい列(キーの名前): ";
    /// OUT-2。
    CmdExportTable = "Export the table to a file", "表をファイルに書き出す";
    AskExportTable = "Export the table, file name (.csv .tsv .json .md): ", "表を書き出す ファイルの名前(.csv .tsv .json .md): ";
    /// OUT-5。{0} はパス。
    AskExportOverwrite = "{0} exists. Overwrite? Type y and Enter: ", "{0} はもうある。上書きする? y と打って Enter: ";
    /// OUT-2。
    ExportTableBadExt = "the file name must end in .csv, .tsv, .json or .md", "ファイルの名前は .csv・.tsv・.json・.md で終える";
    /// OUT-5。{0} は行の数、{1} はパス。
    ExportTableDone = "wrote {0} rows to {1}", "{0} 行を {1} に書いた";
    /// OUT-5。{0} はパス、{1} は理由。
    ExportTableFailed = "could not write {0}: {1}", "{0} に書けない: {1}";
    AddColumnBad = "a column name cannot be empty, contain a line break, or start with file. or formula.", "列の名前は、空・改行を含む・file. か formula. で始まる名前にできない";
    /// {0} は列の名前。
    AddColumnAdded = "added column {0}: only the notes you fill in get the key when you save", "列 {0} を足した: 値を入れたノートにだけ、保存でキーを書く";
    ImportNoMatchFile = "no matching .base", "合う .base が無い";
    ImportUnreadable = "cannot read {0}: {1}", "{0} を読めない: {1}";
    ImportNoViews = "{0} has no views", "{0} に views が無い";
    ImportUnnamed = "(unnamed {0})", "(名前なし {0})";
    ImportNoMatchView = "no matching view", "合うビューが無い";
    /// {0} はファイル、{1} は元のビュー、{2} は新しい名前、{3} は `DroppedNote` か空。
    Imported = "imported view \"{1}\" of {0} as the mdgrid view \"{2}\"{3}", "{0} のビュー「{1}」を mdgrid のビュー「{2}」として取り込んだ{3}";
    ExportNameEmpty = "type a file name (e.g. todo.base)", "ファイルの名前を打つ(例: 未完了.base)";
    ExportNameOnly = "type only the file name (it is written under the opened folder)", "ファイルの名前だけを打つ(書く先は開いたフォルダの下に決まっている)";
    ExportExists = "{0} already exists, so it is not written (existing .base files are not overwritten; use another name)", "{0} はもうあるので書かない(既存の .base は書き換えない。別の名前にする)";
    CannotWriteFile = "cannot write {0}: {1}", "{0} に書けない: {1}";
    /// {0} はビュー、{1} はファイル、{2} は `DroppedNote` か空。
    Exported = "exported view \"{0}\" to {1}{2}", "ビュー「{0}」を {1} に書き出した{2}";

    // ---- ui/native_views: mdgrid のビュー(BV-13・BV-18・BV-20) ----
    /// `.base` なしで開いたときの先頭のタブの名前(日本語は名前の検査とビューの状態にも使う)。
    DefaultTabName = "Default", "既定の表";
    /// {0} は落とした・近似の部分を ` / ` でつないだもの。
    DroppedNote = ". Dropped or approximated: {0}", "。落とした・近似の部分: {0}";
    OnlyOneView = "there is only one view (open a .base, or use \"Save as\" in view settings to switch)", "ビューは1つだけ(.base を開くか、ビューの設定で名前を付けて保存すると切り替えられる)";
    NoViewToSwitch = "no view to switch to", "切り替えるビューが無い";
    SwitchedView = "switched to view \"{0}\"", "ビュー「{0}」に切り替えた";
    ViewsNoDir = "the place for mdgrid views (the config folder) is unknown, so mdgrid views cannot be saved", "mdgrid のビューの置き場(設定のフォルダ)が分からないので、mdgrid のビューを保存できない";
    ViewNameEmpty = "the name is empty (type a name)", "名前が空(名前を打つ)";
    ViewNameFixed = "\"{0}\" is the name of the default table or a .base view (use another name)", "「{0}」は既定の表か .base のビューと同じ名前(別の名前にする)";
    ViewNameTaken = "an mdgrid view \"{0}\" already exists (use another name, or \"Overwrite\" in that view to change it)", "「{0}」という mdgrid のビューはもうある(別の名前にする。変えるならそのビューで「上書き」)";
    /// 空の名前で保存したビューの名前。
    ViewDefaultName = "View", "ビュー";
    StateCannotMove = "cannot move the view state: {0}", "見た目の状態を移せない: {0}";
    /// {0} は既定の表の名前、{1} は押したボタン、{2} は「名前を付けて保存」のボタン。
    ViewButtonFixed = "\"{1}\" cannot be used on {0} or .base views (use \"{2}\" to make an mdgrid view)", "{0}・.base のビューは{1}できない(「{2}」で mdgrid のビューにする)";
    ViewSavedAs = "saved as the mdgrid view \"{0}\"{1}", "mdgrid のビュー「{0}」として保存した{1}";
    ViewGoneOverwrite = "\"{0}\" was deleted elsewhere, so it cannot be overwritten", "「{0}」はほかで消されたので上書きできない";
    ViewOverwritten = "overwrote the mdgrid view \"{0}\"", "mdgrid のビュー「{0}」を上書きした";
    ViewGoneRename = "\"{0}\" was deleted elsewhere, so it cannot be renamed", "「{0}」はほかで消されたので名前を変えられない";
    RenameUnapplied = " (changes in the settings screen were not applied)", "(設定の画面で変えたところは反映していない)";
    /// {2} は `RenameUnapplied` か空。
    ViewRenamed = "renamed the view from \"{0}\" to \"{1}\"{2}", "ビューの名前を「{0}」から「{1}」に変えた{2}";
    ViewDeleted = "deleted the mdgrid view \"{0}\"", "mdgrid のビュー「{0}」を削除した";

    // ---- ui/settings: ビューの設定の画面(NV-13・NV-15・NV-19・NV-22) ----
    BtnApply = "Apply", "反映";
    /// ビューの設定の「取り消し」のボタン。
    BtnCancel = "Cancel", "取り消し";
    BtnReset = "Reset", "既定に戻す";
    BtnSaveAs = "Save as", "名前を付けて保存";
    BtnOverwrite = "Overwrite", "上書き";
    BtnRename = "Rename", "名前の変更";
    BtnDelete = "Delete", "削除";
    KindValues = "Pick from the values", "値の一覧から選ぶ";
    KindContains = "Contains", "含む";
    KindNotContains = "Does not contain", "含まない";
    KindCompare = "Compare (= ≠ < ≤ > ≥)", "比べる(= ≠ < ≤ > ≥)";
    KindEmpty = "Is empty", "空である";
    KindNotEmpty = "Is not empty", "空でない";
    CmpEq = "=  equal", "=  等しい";
    CmpNe = "≠  not equal", "≠  等しくない";
    CmpLt = "<  less than / before", "<  より小さい・より前";
    CmpLe = "≤  at most / until", "≤  以下・その日まで";
    CmpGt = ">  greater than / after", ">  より大きい・より後";
    CmpGe = "≥  at least / from", "≥  以上・その日から";
    PurposeFilter = "filter", "フィルター";
    PurposeSort = "sort", "並べ替え";
    PurposeGroup = "group", "グループ";
    DirAsc = "↑ ascending", "↑ 昇順";
    DirDesc = "↓ descending", "↓ 降順";
    GroupOnlyByColumn = "available when grouping by a column (first choose a column with \"Group by column\")", "列で分けるときに選べる(先に「列で分ける」で列を選ぶ)";
    NotRemovable = "this item cannot be removed", "消せる項目ではない";
    SettingsCancelled = "view settings cancelled (the table is as before)", "ビューの設定を取り消した(表は開く前のまま)";
    SettingsReset = "reset to default (Apply to use it in the table)", "既定に戻した(反映で表に効く)";
    SettingsAppliedReadOnly = "applied the view settings (read-only, so not remembered)", "ビューの設定を反映した(読むだけなので覚えない)";
    SettingsApplied = "applied the view settings", "ビューの設定を反映した";
    ChipsEmpty = "the settings band is empty ({0} opens view settings)", "設定の帯に項目が無い({0} でビューの設定を開く)";

    // ---- ui/settings_view: ビューの設定の画面の描画(NV-18・SR-20) ----
    SetColumns = "Columns (shown and order)", "列(表示と順)";
    SetDisplay = "Display", "表示";
    SetFilters = "Filters (rows matching all)", "フィルター(全部を満たす行だけ)";
    SetAddFilter = "+ Add condition", "+ 条件を足す";
    SetSorts = "Sort (top column first)", "並べ替え(上の列が先)";
    SetAddSort = "+ Add sort", "+ 並べ替えを足す";
    SetGroup = "Group", "グループ";
    SetGroupInheritBase = "as in .base", ".base のまま";
    SetGroupInheritDefault = "default (no groups)", "既定のまま(分けない)";
    /// {0} はラジオの印。
    SetGroupOff = "{0} none", "{0} しない";
    /// {0} はラジオの印、{1} は列か `SetGroupPickColumn`。
    SetGroupBy = "{0} by column: {1}", "{0} 列で分ける: {1}";
    SetGroupPickColumn = "(choose a column)", "(列を選ぶ)";
    /// {0} はチェックの印。
    SetGroupHideEmpty = "{0} hide empty groups", "{0} 空のまとまりを隠す";
    SetGroupOrder = "group order: {0}", "まとまりの並び: {0}";
    /// {0} は目的(`Purpose*`)。
    PickColumnFor = "Choose a column to {0} by", "{0}の列を選ぶ";
    PickKindOf = "Condition type for {0}", "{0} の条件の種類";
    PickCmpOf = "How to compare {0}", "{0} の比べ方";
    PickModeKeep = "mode: keep only rows with checked values", "方式: チェックした値の行だけ残す";
    PickModeHide = "mode: hide rows with checked values", "方式: チェックした値の行を隠す";
    /// {0} はチェックの印、{1} は値、{2} は件数。
    PickValueRow = "{0} {1}  {2}", "{0} {1}  {2}件";
    PickValuesOf = "Values of {0} (count)", "{0} の値(件数)";
    TextContains = "text that {0} contains", "{0} が含む文字";
    TextNotContains = "text that {0} does not contain", "{0} が含まない文字";
    TextSaveAs = "name of the new mdgrid view", "新しい mdgrid のビューの名前";
    TextRename = "new name of the view", "ビューの新しい名前";
    /// {0}・{1}・{2} はキー。
    SetHintValues = "{0} check / switch mode, {1} confirm, {2} back", "{0} でチェック・方式の切り替え、{1} で決める、{2} で戻る";
    SetHintPick = "{0} choose, {1} back", "{0} で選ぶ、{1} で戻る";
    SetHintColumns = "{0} show/hide, {1} / {2} move", "{0} で表示・非表示、{1} / {2} で順を動かす";
    SetHintFilters = "{0} add/edit condition, {1} switch keep/hide, {2} delete", "{0} で条件を足す・直す、{1} で 残す・隠す を切り替え、{2} で消す";
    SetHintSorts = "{0} add / change direction, {1} / {2} order, {3} delete", "{0} で足す・向きを変える、{1} / {2} で順、{3} で消す";
    SetHintGroup = "{0} choose (hide empty and order apply when grouping by a column)", "{0} で選ぶ(空を隠す・並びは列で分けるとき)";
    SetHintDisplay = "{0} show/hide (this view only; Apply to use it in the table)", "{0} で出す・出さないを切り替え(このビューだけ。反映で表に効く)";
    SetHintButtons = "{0} press. The table does not change until Apply", "{0} で押す。反映するまで表は変わらない";
    SetNativeName = "mdgrid view / {0}", "mdgrid のビュー / {0}";
    SetHead = " View settings  {0}  the table does not change until Apply", " ビューの設定  {0}  反映するまで表は変わらない";

    // ---- ui/settings_pick: ビューの設定の選び手と値の入力(NV-14・NV-19) ----
    PickInProgress = "choosing ({0} to go back)", "選んでいる途中({0} で戻る)";
    SortAlready = "{0} is already in the sort", "{0} はもう並べ替えにある";
    NoValueChecked = "no value is checked ({0} to check)", "値にチェックが無い({0} で付ける)";
    ValueEmpty = "the value is empty ({0} to go back)", "値が空({0} で戻る)";
    OperandNumber = "type a number (e.g. 3). \"{0}\" is not a number", "数を打つ(例: 3)。「{0}」は数でない";
    OperandDate = "type a date as YYYY-MM-DD. Cannot read \"{0}\"", "日付は YYYY-MM-DD で打つ。「{0}」は読めない";
    OperandDateTime = "type a datetime as YYYY-MM-DDTHH:MM. Cannot read \"{0}\"", "日時は YYYY-MM-DDTHH:MM で打つ。「{0}」は読めない";

    // ---- ui/nav: 選択・同じ値・検索(NV-1・NV-5・NV-8・OUT-3) ----
    PickNoEditor = "read-only while picking with --pick (no editor; Enter to pick, q to cancel)", "--pick で選んでいる間は読むだけ(エディタでは開かない。Enter で選ぶ・q で取りやめ)";
    VisualHint = "select a range: move, then v to confirm (Esc to clear)", "範囲で選ぶ: 動かして v で決める(Esc で解く)";
    SelectedAll = "selected all rows ({0})", "全部の行を選んだ({0}行)";
    SelectionCleared = "selection cleared", "選択を解いた";
    FilterCleared = "filter cleared", "絞り込みを解いた";
    NoRowToSelect = "no row to select", "選ぶ行が無い";
    SelectValueCell = "select a value cell", "値のセルを選ぶ";
    /// {0} は列、{1} は値、{2} は行の数。
    HighlightSame = "highlighting rows with the same value: {0} = {1} ({2} rows)", "同じ値の行を強調: {0} = {1}({2}行)";
    /// `HighlightSame` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    HighlightSameOne = "highlighting rows with the same value: {0} = {1} ({2} row)", "同じ値の行を強調: {0} = {1}({2}行)";
    SameFilterCleared = "same-value filter cleared ({0})", "同じ値の絞り込みを解いた({0})";
    SameFilterOn = "only rows with the same value: {0} (Esc to clear)", "同じ値の行だけ: {0}(Esc で解く)";
    NotFound = "not found", "見つからない";
    NoSearchTerm = "no search term (/ to search)", "検索の語が無い(/ で検索)";
    NotFoundTerm = "not found: {0}", "見つからない: {0}";
    WrappedToTop = "wrapped from the bottom to the top", "末尾から先頭へ回った";
    WrappedToBottom = "wrapped from the top to the bottom", "先頭から末尾へ回った";

    // ---- ui/review: 保存の確認(WB-9・WB-16) ----
    ReviewReadOnly = "read-only ({0})", "読むだけ({0})";
    ReviewNewline = "a value with line breaks cannot be written", "改行を含む値は書けない";
    ReviewVerify = "stopped by the check of the written result ({0})", "書いた結果の検査で止めた({0})";
    ReviewChanged = "changed outside", "外で変更";
    ReviewIo = "cannot write ({0})", "書けない({0})";
    ReviewDiscarded = "discarded the pending changes: {0}", "ためた変更を捨てた: {0}";
    ReviewSaved = "saved {0} files", "保存した {0}件";
    /// `ReviewSaved` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    ReviewSavedOne = "saved {0} file", "保存した {0}件";
    /// {0} はファイルを `, ` でつないだもの。
    ReviewSavedList = " ({0})", "({0})";
    ReviewStopped = "stopped {0} files changed outside ({1})", "外で変わって止めた {0}件({1})";
    /// `ReviewStopped` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    ReviewStoppedOne = "stopped {0} file changed outside ({1})", "外で変わって止めた {0}件({1})";
    ReviewFailed = "could not write {0} files ({1})", "書けなかった {0}件({1})";
    /// `ReviewFailed` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    ReviewFailedOne = "could not write {0} file ({1})", "書けなかった {0}件({1})";
    /// 文と文の区切り。
    SentenceSep = ". ", "。";
    ReviewNotExternal = "{0} has not changed outside (Enter to save)", "{0} は外で変わっていない(Enter で保存)";
    ReviewNoDiff = "{0} cannot be written (no diff could be made)", "{0} は書けない(差分を作れない)";
    ReviewCannotReload = "cannot reload: {0} ({1})", "読み直せない: {0}({1})";
    ReviewChangedAgain = "{0} changed outside again. Check the diff and press o again", "{0} はまた外で変わった。差分を見直してもう一度 o";
    ReviewExternalHead = "  changed outside: o writes this file over the outside change / d discards the pending changes", "  外で変更: o このファイルを外の変更の上に書く / d ためた変更を捨てる";
    ReviewDiffError = "   cannot write: {0}", "   書けない: {0}";
    ReviewHead = " Save review  {0} files  diff before writing (- current file / + after writing)", " 保存の確認  {0}ファイル  書く前の差分(- 今のファイル / + 書いたあと)";
    /// `ReviewHead` の英語の単数(数が 1 のとき。`Msg::fill` が選ぶ)。
    ReviewHeadOne = " Save review  {0} file  diff before writing (- current file / + after writing)", " 保存の確認  {0}ファイル  書く前の差分(- 今のファイル / + 書いたあと)";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_leaves_other_braces() {
        let a: &dyn Display = &"x{1}";
        let b: &dyn Display = &7;
        assert_eq!(fill_in("{0}-{1}-{2}-{x}-{", &[a, b]), "x{1}-7-{2}-{x}-{");
    }
}
