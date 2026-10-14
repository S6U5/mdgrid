//! 画面と起動の文言の表と、言語の決め方(SR-23)。形は docs/design.md の `src/i18n.rs`。
//!
//! 文言は言語ごとの台帳 `locales/<言語の符号>.toml` に置く。build.rs が台帳を確かめ(欠け・余り・空・差し込みの
//! 番号の食い違いはビルドで落ちる。src/ledger.rs)、`Lang`・`Language`・`Msg` を作る。ここはそれを `include!` する。
//! 差し込みは `{0}`・`{1}` の番号で、`Msg::fill` が埋める。
//!
//! 言語は、スレッドごとの上書き(`scoped`。試験用)> 大域の既定(`set_default`。main が決める)>
//! 環境(`LC_ALL`・`LC_MESSAGES`・`LANG`)の順で決まる。

use std::cell::Cell;
use std::fmt::Display;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

impl Language {
    /// 設定の値(`"auto"` か台帳の言語の符号 `"en"`・`"ja"` など)を読む。ほかは None。
    pub fn parse(s: &str) -> Option<Language> {
        Language::ALL.iter().copied().find(|l| l.name() == s)
    }

    /// 設定の値の名前。
    pub fn name(self) -> &'static str {
        match self.lang() {
            None => "auto",
            Some(l) => l.code(),
        }
    }
}

/// `auto` で見る環境変数(この順で最初の空でない値)。
pub const LOCALE_VARS: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

/// 設定が言語ならそれ。Auto なら `LOCALE_VARS` の順で最初の空でない値が、英語のほかの台帳の言語の符号
/// (`ja` など)で始まればその言語、ほか(どれも無いときも)は En。`env` は環境変数の名前から値を引く(試験で差し替える)。
pub fn resolve(setting: Language, env: impl Fn(&str) -> Option<String>) -> Lang {
    if let Some(l) = setting.lang() {
        return l;
    }
    LOCALE_VARS
        .iter()
        .filter_map(|name| env(name))
        .find(|v| !v.is_empty())
        .and_then(|v| {
            Lang::ALL
                .iter()
                .copied()
                .filter(|l| *l != Lang::En)
                .find(|l| v.starts_with(l.code()))
        })
        .unwrap_or(Lang::En)
}

/// 大域の既定(0 は未設定、ほかは `Lang::ALL` の添字 + 1)。
static DEFAULT: AtomicU8 = AtomicU8::new(0);

/// 未設定のときに使う、環境から決めた言語(最初に引いたときに1回だけ読む)。
static FROM_ENV: OnceLock<Lang> = OnceLock::new();

thread_local! {
    /// このスレッドの上書き(`scoped`)。
    static OVERRIDE: Cell<Option<Lang>> = const { Cell::new(None) };
}

/// 大域の既定の言語を決める(main だけが呼ぶ)。
pub fn set_default(lang: Lang) {
    let v = Lang::ALL
        .iter()
        .position(|l| *l == lang)
        .map_or(0, |i| i + 1);
    DEFAULT.store(v as u8, Ordering::Relaxed);
}

/// 今の言語: このスレッドの上書き > 大域の既定 > 環境。
pub fn current() -> Lang {
    if let Some(l) = OVERRIDE.with(Cell::get) {
        return l;
    }
    match DEFAULT.load(Ordering::Relaxed) {
        0 => *FROM_ENV.get_or_init(|| resolve(Language::Auto, |k| std::env::var(k).ok())),
        n => Lang::ALL
            .get(usize::from(n) - 1)
            .copied()
            .unwrap_or(Lang::En),
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

impl Msg {
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
