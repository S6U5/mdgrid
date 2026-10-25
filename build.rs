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
    starters(&codes);
}

/// 組み込みのスターター(ST-2)を `OUT_DIR/starters.rs` に書く(文言の台帳と同じ道)。`starters/<名前>/` ごとに、
/// `starter.toml`(`[name]` に台帳の全言語)と `ledger/.mdgrid/workspace.toml` があることを確かめ、`ledger*/`・`samples*/`
/// の中のファイルを `include_str!` で埋め込む(文のファイルだけ)。`<部分>.<符号>/` の符号は台帳の言語に限る。
fn starters(codes: &[String]) {
    println!("cargo:rerun-if-changed=starters");
    let dir = Path::new("starters");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("starters/ is readable")
        .filter_map(|e| {
            let p = e.ok()?.path();
            p.is_dir()
                .then(|| p.file_name()?.to_str().map(str::to_string))?
        })
        .collect();
    names.sort();
    let mut problems = Vec::new();
    let mut out = String::new();
    out.push_str("/// 組み込みのスターターの1つのファイル(ST-2)。`part` はフォルダの名前(`ledger`・`ledger.<言語>`・`samples`・\n/// `samples.<言語>`)、`rel` は台帳の根からの相対パス(`/` 区切り)。\n");
    out.push_str("pub struct BuiltinFile {\n    pub part: &'static str,\n    pub rel: &'static str,\n    pub text: &'static str,\n}\n\n");
    out.push_str("/// 組み込みのスターター(ST-2)。`manifest` は `starter.toml` の文。\n");
    out.push_str("pub struct Builtin {\n    pub name: &'static str,\n    pub manifest: &'static str,\n    pub files: &'static [BuiltinFile],\n}\n\n");
    out.push_str(
        "/// 組み込みのスターターの全部(名前の順)。\npub const BUILTIN: &[Builtin] = &[\n",
    );
    for name in &names {
        let sdir = dir.join(name);
        let label = format!("starters/{name}");
        let manifest = sdir.join("starter.toml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        match std::fs::read_to_string(&manifest) {
            Err(e) => problems.push(format!("{label}/starter.toml: {e}")),
            Ok(t) => match t.parse::<toml::Table>() {
                Err(e) => problems.push(format!("{label}/starter.toml: not valid TOML: {e}")),
                Ok(m) => {
                    let words = m.get("name").and_then(|v| v.as_table());
                    for c in codes {
                        let has = words
                            .and_then(|w| w.get(c.as_str()))
                            .and_then(|v| v.as_str())
                            .is_some_and(|s| !s.trim().is_empty());
                        if !has {
                            problems.push(format!("{label}/starter.toml: [name] needs {c}"));
                        }
                    }
                }
            },
        }
        let marker = sdir.join("ledger/.mdgrid/workspace.toml");
        match std::fs::read_to_string(&marker) {
            Err(_) => problems.push(format!(
                "{label}: ledger/.mdgrid/workspace.toml is required"
            )),
            Ok(t) => {
                if let Err(e) = t.parse::<toml::Table>() {
                    problems.push(format!(
                        "{label}/ledger/.mdgrid/workspace.toml: not valid TOML: {e}"
                    ));
                }
            }
        }
        let _ = writeln!(
            out,
            "    Builtin {{\n        name: {name:?},\n        manifest: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/starters/{name}/starter.toml\")),\n        files: &["
        );
        let mut parts: Vec<std::path::PathBuf> = std::fs::read_dir(&sdir)
            .expect("starter folder is readable")
            .filter_map(|e| {
                let p = e.ok()?.path();
                p.is_dir().then_some(p)
            })
            .collect();
        parts.sort();
        for p in parts {
            let part = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let (kind, lang) = match part.split_once('.') {
                Some((k, l)) => (k, Some(l)),
                None => (part.as_str(), None),
            };
            if kind != "ledger" && kind != "samples" {
                problems.push(format!(
                    "{label}/{part}: only ledger/, ledger.<lang>/, samples/ and samples.<lang>/ are allowed"
                ));
                continue;
            }
            if let Some(l) = lang {
                if !codes.iter().any(|c| c == l) {
                    problems.push(format!("{label}/{part}: {l} is not a language of locales/"));
                    continue;
                }
                // 言語の変種は丸ごと差し替えるので、ledger の変種にも印が要る。
                if kind == "ledger" && !p.join(".mdgrid/workspace.toml").is_file() {
                    problems.push(format!(
                        "{label}/{part}: .mdgrid/workspace.toml is required"
                    ));
                }
            }
            let mut files = Vec::new();
            walk_files(&p, &p, &mut files);
            files.sort();
            for rel in files {
                let rel = rel
                    .iter()
                    .map(|c| c.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                println!("cargo:rerun-if-changed=starters/{name}/{part}/{rel}");
                let abs = format!("/starters/{name}/{part}/{rel}");
                let _ = writeln!(
                    out,
                    "            BuiltinFile {{ part: {part:?}, rel: {rel:?}, text: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {abs:?})) }},"
                );
            }
        }
        out.push_str("        ],\n    },\n");
    }
    out.push_str("];\n");
    if !problems.is_empty() {
        panic!(
            "\nthe starters (starters/*) have problems:\n  - {}\n",
            problems.join("\n  - ")
        );
    }
    let dest = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("starters.rs");
    std::fs::write(dest, out).expect("write starters.rs");
}

/// `dir` の下のファイルを、`base` からの相対パスで集める(フォルダは下へ)。
fn walk_files(base: &Path, dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("starter folder is readable") {
        let p = e.expect("entry").path();
        // リンクはたどらない(ユーザーのスターターを読む側と同じ。リポの外のファイルを埋め込まない)。
        if std::fs::symlink_metadata(&p).is_ok_and(|m| m.file_type().is_symlink()) {
            continue;
        }
        if p.is_dir() {
            walk_files(base, &p, out);
        } else if p.is_file() {
            // 隠しファイル(`.DS_Store`・`.gitkeep`)と編集の控え(`x.md~`)は埋め込まない(台帳に写さない)。
            // フォルダは `.mdgrid/` があるので見ない。
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if name.starts_with('.') || name.ends_with('~') {
                continue;
            }
            out.push(p.strip_prefix(base).expect("under base").to_path_buf());
        }
    }
}
