//! language(specs/_changes/2026-10-03-language.md)タスク1の受け入れ: 文言の表と言語の決め方(SR-23)。
//! 実装を見ずに、記録の「設計」の公開の口だけを仮定して書いた。環境変数は書き換えない
//! (`resolve` には環境を引く関数を渡して差し替える)。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/i18n.rs(lib の `pub mod i18n`)
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Lang { En, Ja }
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Language { Auto, En, Ja }
//! /// 設定が En・Ja ならそれ。Auto なら LC_ALL・LC_MESSAGES・LANG の順で最初の空でない値が
//! /// `ja` で始まれば Ja、ほかは En。`env` は環境変数の名前から値を引く。
//! pub fn resolve(setting: Language, env: impl Fn(&str) -> Option<String>) -> Lang;
//! #[derive(Clone, Copy, Debug, …)] pub enum Msg { … }
//! impl Msg {
//!     pub const ALL: &[Msg];                       // 配列でもよい(`.iter()` を使う)
//!     pub const fn en(self) -> &'static str;
//!     pub const fn ja(self) -> &'static str;
//!     pub fn text(self) -> &'static str;           // 今の言語
//!     pub fn fill(self, args: &[&dyn std::fmt::Display]) -> String; // `{0}`・`{1}` を差し込む
//! }
//! /// 落とすまでこのスレッドの言語を `lang` にする guard。
//! pub fn scoped(lang: Lang) -> impl Drop;
//! ```
//!
//! 試験の環境の言語は `.cargo/config.toml` の [env] で日本語(ja_JP.UTF-8)に固定する予定。

use mdgrid::i18n::{resolve, scoped, Lang, Language, Msg};
use std::collections::BTreeSet;

/// `{数字}` の番号の集合。
fn placeholders(s: &str) -> BTreeSet<u32> {
    let mut set = BTreeSet::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'{' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == b'}' {
                set.insert(s[i + 1..j].parse().unwrap());
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    set
}

/// ひらがな・カタカナ・漢字と、日本語の約物(・「」、。)。
fn is_japanese_char(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{309F}'   // ひらがな
        | '\u{30A0}'..='\u{30FF}' // カタカナ(・ を含む)
        | '\u{31F0}'..='\u{31FF}'
        | '\u{FF66}'..='\u{FF9F}' // 半角カタカナ
        | '\u{3400}'..='\u{4DBF}' // 漢字
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '「' | '」' | '、' | '。')
}

fn has_japanese(s: &str) -> bool {
    s.chars().any(is_japanese_char)
}

/// 名前と値の組から環境を引く関数を作る。
fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |name: &str| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
}

fn auto(pairs: &[(&str, &str)]) -> Lang {
    resolve(Language::Auto, env_of(pairs))
}

// ---- 文言の表 ----

#[test]
fn test_sr_23_table_is_not_empty() {
    // [SR-23] 表に文言がある。
    assert!(!Msg::ALL.is_empty(), "Msg::ALL が空");
}

#[test]
fn test_sr_23_every_msg_has_en_and_ja() {
    // [SR-23] 英語と日本語のどちらかが欠けた文言があれば落ちる。
    for m in Msg::ALL.iter() {
        assert!(!m.en().trim().is_empty(), "{m:?} の英語が空");
        assert!(!m.ja().trim().is_empty(), "{m:?} の日本語が空");
    }
}

#[test]
fn test_sr_23_placeholders_match() {
    // [SR-23] 差し込みの番号 `{n}` の集合が英語と日本語で同じ。
    for m in Msg::ALL.iter() {
        assert_eq!(
            placeholders(m.en()),
            placeholders(m.ja()),
            "{m:?} の差し込みの番号が違う\n en: {}\n ja: {}",
            m.en(),
            m.ja()
        );
    }
}

#[test]
fn test_sr_23_english_has_no_japanese() {
    // [SR-23] 英語の文言に日本語の文字(ひらがな・カタカナ・漢字・「・」「「」「」」)が無い。
    for m in Msg::ALL.iter() {
        let bad: String = m.en().chars().filter(|c| is_japanese_char(*c)).collect();
        assert!(
            bad.is_empty(),
            "{m:?} の英語に日本語の文字 {bad:?} がある: {}",
            m.en()
        );
    }
}

#[test]
fn test_sr_23_fill_replaces_every_placeholder() {
    // [SR-23] fill は `{n}` を n 番目の値で置き換え、番号の差し込みを残さない。
    for lang in [Lang::En, Lang::Ja] {
        let _g = scoped(lang);
        for m in Msg::ALL.iter() {
            let src = match lang {
                Lang::En => m.en(),
                Lang::Ja => m.ja(),
            };
            let nums = placeholders(src);
            let n = nums.iter().max().map(|x| *x as usize + 1).unwrap_or(0);
            let vals: Vec<String> = (0..n).map(|i| format!("<V{i}>")).collect();
            let args: Vec<&dyn std::fmt::Display> =
                vals.iter().map(|v| v as &dyn std::fmt::Display).collect();
            let out = m.fill(&args);
            for i in &nums {
                assert!(
                    out.contains(&format!("<V{i}>")),
                    "{m:?}({lang:?}) の fill に {{{i}}} の値が無い: {out}"
                );
            }
            assert!(
                placeholders(&out).is_empty(),
                "{m:?}({lang:?}) の fill に差し込みが残る: {out}"
            );
            if nums.is_empty() {
                assert_eq!(out, src, "{m:?}({lang:?}) 差し込みの無い文はそのまま");
            }
        }
    }
}

// ---- resolve ----

#[test]
fn test_sr_23_resolve_ja_locale() {
    // [SR-23] ja で始まれば日本語。
    assert_eq!(auto(&[("LANG", "ja_JP.UTF-8")]), Lang::Ja);
    assert_eq!(auto(&[("LANG", "ja")]), Lang::Ja);
    assert_eq!(auto(&[("LC_MESSAGES", "ja_JP.UTF-8")]), Lang::Ja);
    assert_eq!(auto(&[("LC_ALL", "ja_JP.UTF-8")]), Lang::Ja);
}

#[test]
fn test_sr_23_resolve_other_locale_is_english() {
    // [SR-23] それ以外(どれも無いときも)英語。
    assert_eq!(auto(&[("LANG", "en_US.UTF-8")]), Lang::En);
    assert_eq!(auto(&[("LANG", "C")]), Lang::En);
    assert_eq!(auto(&[("LANG", "POSIX")]), Lang::En);
    assert_eq!(auto(&[("LANG", "fr_FR.UTF-8")]), Lang::En);
    assert_eq!(auto(&[]), Lang::En);
    assert_eq!(auto(&[("LANG", "")]), Lang::En);
    assert_eq!(
        auto(&[("LC_ALL", ""), ("LC_MESSAGES", ""), ("LANG", "")]),
        Lang::En
    );
}

#[test]
fn test_sr_23_resolve_order() {
    // [SR-23] LC_ALL が LC_MESSAGES と LANG より先、LC_MESSAGES が LANG より先。
    assert_eq!(
        auto(&[
            ("LC_ALL", "ja_JP.UTF-8"),
            ("LC_MESSAGES", "en_US.UTF-8"),
            ("LANG", "en_US.UTF-8")
        ]),
        Lang::Ja
    );
    assert_eq!(
        auto(&[
            ("LC_ALL", "en_US.UTF-8"),
            ("LC_MESSAGES", "ja_JP.UTF-8"),
            ("LANG", "ja_JP.UTF-8")
        ]),
        Lang::En
    );
    assert_eq!(
        auto(&[("LC_ALL", "C"), ("LANG", "ja_JP.UTF-8")]),
        Lang::En,
        "LC_ALL=C が先"
    );
    assert_eq!(
        auto(&[("LC_MESSAGES", "ja_JP.UTF-8"), ("LANG", "en_US.UTF-8")]),
        Lang::Ja
    );
    assert_eq!(
        auto(&[("LC_MESSAGES", "en_US.UTF-8"), ("LANG", "ja_JP.UTF-8")]),
        Lang::En
    );
}

#[test]
fn test_sr_23_resolve_skips_empty() {
    // [SR-23] 空の値は飛ばして次を見る。
    assert_eq!(auto(&[("LC_ALL", ""), ("LANG", "ja_JP.UTF-8")]), Lang::Ja);
    assert_eq!(
        auto(&[("LC_ALL", ""), ("LC_MESSAGES", ""), ("LANG", "ja_JP.UTF-8")]),
        Lang::Ja
    );
    assert_eq!(
        auto(&[
            ("LC_ALL", ""),
            ("LC_MESSAGES", "ja_JP.UTF-8"),
            ("LANG", "en_US.UTF-8")
        ]),
        Lang::Ja
    );
    assert_eq!(auto(&[("LC_ALL", ""), ("LANG", "en_US.UTF-8")]), Lang::En);
}

#[test]
fn test_sr_23_resolve_setting_wins_over_env() {
    // [SR-23] 設定の en・ja は環境より先。
    let ja_env = [
        ("LC_ALL", "ja_JP.UTF-8"),
        ("LC_MESSAGES", "ja_JP.UTF-8"),
        ("LANG", "ja_JP.UTF-8"),
    ];
    let en_env = [("LC_ALL", "en_US.UTF-8"), ("LANG", "en_US.UTF-8")];
    assert_eq!(resolve(Language::En, env_of(&ja_env)), Lang::En);
    assert_eq!(resolve(Language::Ja, env_of(&en_env)), Lang::Ja);
    assert_eq!(resolve(Language::Ja, env_of(&[])), Lang::Ja);
    assert_eq!(resolve(Language::En, env_of(&[])), Lang::En);
    assert_eq!(resolve(Language::Auto, env_of(&ja_env)), Lang::Ja);
    assert_eq!(resolve(Language::Auto, env_of(&en_env)), Lang::En);
}

// ---- scoped ----

/// 英語と日本語が違う最初の文言。
fn differing_msg() -> Msg {
    *Msg::ALL
        .iter()
        .find(|m| m.en() != m.ja())
        .expect("英語と日本語が違う文言がある")
}

#[test]
fn test_sr_23_scoped_switches_and_restores() {
    // [SR-23] scoped の中はその言語、guard を落とすと外の言語に戻る(入れ子も)。
    let m = differing_msg();
    let _outer = scoped(Lang::Ja);
    assert_eq!(m.text(), m.ja());
    {
        let _g = scoped(Lang::En);
        assert_eq!(m.text(), m.en());
        {
            let _h = scoped(Lang::Ja);
            assert_eq!(m.text(), m.ja());
        }
        assert_eq!(m.text(), m.en(), "内の guard を落とすと En に戻る");
    }
    assert_eq!(m.text(), m.ja(), "guard を落とすと Ja に戻る");
}

#[test]
fn test_sr_23_scoped_en_outside_is_test_env_ja() {
    // [SR-23] 試験の環境は日本語に固定(.cargo/config.toml の [env])。scoped(En) の中は英語、
    // 外と guard を落としたあとは日本語。
    let first = Msg::ALL[0];
    assert_eq!(first.text(), first.ja(), "試験の環境の既定は日本語");
    {
        let _g = scoped(Lang::En);
        assert_eq!(first.text(), first.en());
    }
    assert_eq!(first.text(), first.ja(), "guard を落とすと戻る");
    let g = scoped(Lang::En);
    assert_eq!(first.text(), first.en());
    drop(g);
    assert_eq!(first.text(), first.ja());
}

#[test]
fn test_sr_23_scoped_is_per_thread() {
    // [SR-23] scoped は落とすまでそのスレッドだけ。並ぶ試験の言語を乱さない。
    let m = differing_msg();
    let _g = scoped(Lang::En);
    let other = std::thread::spawn(move || {
        let _h = scoped(Lang::Ja);
        m.text()
    })
    .join()
    .unwrap();
    assert_eq!(other, m.ja());
    assert_eq!(m.text(), m.en());
}

#[test]
fn test_sr_23_has_japanese_helper() {
    // 試験の道具の確かめ。
    assert!(has_japanese("使い方"));
    assert!(has_japanese("a・b"));
    assert!(has_japanese("「x」"));
    assert!(!has_japanese("Usage: mdgrid [--help] — {0}"));
    assert_eq!(
        placeholders("a {0} b {1} {x} {{ {12}"),
        BTreeSet::from([0, 1, 12])
    );
}
