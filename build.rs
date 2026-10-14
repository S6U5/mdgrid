//! 画面の文言の台帳(`locales/*.toml`)から文言の表を作る(SR-23)。
//!
//! 台帳を src/ledger.rs で読んで確かめ、欠け・余り・空・差し込みの番号の食い違いがあればビルドを落とす。
//! `OUT_DIR/locales.rs` に `Lang`・`Language`・`Msg`(と言語ごとの `const fn`)を書き、src/i18n.rs が `include!` する。
//! 言語はファイルの名前(`en.toml` → `Lang::En`)。英語の台帳が文言の名前と並びの正本で、ほかの言語はそれに合わせる。

#[path = "src/ledger.rs"]
mod ledger;

use std::fmt::Write as _;
use std::path::Path;

/// 言語の符号から値の名前(`en` → `En`、`pt-br` → `PtBr`)。
fn variant(code: &str) -> String {
    code.split(['-', '_'])
        .map(|p| {
            let mut c = p.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn main() {
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=src/ledger.rs");
    let dir = Path::new("locales");
    let mut codes: Vec<String> = std::fs::read_dir(dir)
        .expect("locales/ is readable")
        .filter_map(|e| {
            let p = e.ok()?.path();
            (p.extension()? == "toml").then(|| p.file_stem()?.to_str().map(str::to_string))?
        })
        .collect();
    codes.sort();
    // 英語が正本で先頭(既定の言語)。
    codes.retain(|c| c != "en");
    codes.insert(0, "en".to_string());
    let mut problems = Vec::new();
    let mut ledgers = Vec::new();
    for code in &codes {
        let name = format!("locales/{code}.toml");
        if !code
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_')
            || code.is_empty()
        {
            problems.push(format!(
                "{name}: the file name must be a language code like en or ja"
            ));
            continue;
        }
        println!("cargo:rerun-if-changed={name}");
        let text = std::fs::read_to_string(dir.join(format!("{code}.toml")))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        match ledger::parse(&name, &text) {
            Ok(l) => ledgers.push((code.clone(), name, l)),
            Err(e) => problems.push(e),
        }
    }
    if let Some((_, name, base)) = ledgers.first().filter(|(c, _, _)| c == "en") {
        problems.extend(ledger::check_base(name, base));
        for (_, n, other) in &ledgers[1..] {
            problems.extend(ledger::check(base, n, other));
        }
    } else {
        problems.push("locales/en.toml: the English ledger is required".to_string());
    }
    if !problems.is_empty() {
        panic!(
            "\nthe message ledgers (locales/*.toml) have problems:\n  - {}\n",
            problems.join("\n  - ")
        );
    }
    let base = &ledgers[0].2;
    let langs: Vec<(String, String)> = ledgers
        .iter()
        .map(|(c, _, _)| (c.clone(), variant(c)))
        .collect();
    let mut out = String::new();
    // 言語。
    out.push_str(
        "/// 出す言語(台帳 `locales/*.toml` のファイルの名前から作る。英語が先頭で既定)。\n",
    );
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum Lang {\n");
    for (_, v) in &langs {
        let _ = writeln!(out, "    {v},");
    }
    out.push_str("}\n\nimpl Lang {\n    /// 全部の言語(英語が先頭)。\n    pub const ALL: &'static [Lang] = &[");
    for (_, v) in &langs {
        let _ = write!(out, "Lang::{v}, ");
    }
    out.push_str("];\n\n    /// 言語の符号(台帳のファイルの名前。設定の `language` の値)。\n    pub const fn code(self) -> &'static str {\n        match self {\n");
    for (c, v) in &langs {
        let _ = writeln!(out, "            Lang::{v} => {c:?},");
    }
    out.push_str("        }\n    }\n}\n\n");
    // 設定の言語。
    out.push_str("/// 設定の `language`(SR-23)。既定は `Auto`(環境に従う)。ほかは台帳の言語。\n");
    out.push_str("#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]\npub enum Language {\n    #[default]\n    Auto,\n");
    for (_, v) in &langs {
        let _ = writeln!(out, "    {v},");
    }
    out.push_str("}\n\nimpl Language {\n    /// 全部の値(`Auto` が先頭)。\n    pub const ALL: &'static [Language] = &[Language::Auto, ");
    for (_, v) in &langs {
        let _ = write!(out, "Language::{v}, ");
    }
    out.push_str("];\n\n    /// 決めた言語(`Auto` なら None)。\n    pub const fn lang(self) -> Option<Lang> {\n        match self {\n            Language::Auto => None,\n");
    for (_, v) in &langs {
        let _ = writeln!(out, "            Language::{v} => Some(Lang::{v}),");
    }
    out.push_str("        }\n    }\n}\n\n");
    // 文言。
    out.push_str(
        "/// 画面と起動の文言(SR-23)。文は台帳 `locales/*.toml` にあり、今の言語は `text`。\n",
    );
    out.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\npub enum Msg {\n");
    for e in base {
        for d in &e.doc {
            let _ = writeln!(out, "    #[doc = {:?}]", format!(" {d}"));
        }
        let _ = writeln!(out, "    {},", e.key);
    }
    out.push_str("}\n\nimpl Msg {\n    /// 台帳の全部の文言(英語の台帳の並び)。\n    pub const ALL: &'static [Msg] = &[\n");
    for e in base {
        let _ = writeln!(out, "        Msg::{},", e.key);
    }
    out.push_str("    ];\n");
    for (i, (code, v)) in langs.iter().enumerate() {
        let l = &ledgers[i].2;
        let _ = write!(
            out,
            "\n    /// `{code}` の文(locales/{code}.toml)。\n    pub const fn {f}(self) -> &'static str {{\n        match self {{\n",
            f = code.replace('-', "_")
        );
        for e in base {
            let t = &l.iter().find(|x| x.key == e.key).expect("checked").text;
            let _ = writeln!(out, "            Msg::{} => {t:?},", e.key);
        }
        out.push_str("        }\n    }\n");
        let _ = v;
    }
    out.push_str("\n    /// `lang` の文。\n    pub const fn get(self, lang: Lang) -> &'static str {\n        match lang {\n");
    for (code, v) in &langs {
        let _ = writeln!(
            out,
            "            Lang::{v} => self.{}(),",
            code.replace('-', "_")
        );
    }
    out.push_str("        }\n    }\n}\n");
    let dest = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("locales.rs");
    std::fs::write(dest, out).expect("write locales.rs");
}
