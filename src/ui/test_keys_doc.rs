//! キーの割り当ての文書(SR-22)の試験。文書 `docs/keys.md`・`docs/keys.ja.md` が
//! キーの表(SR-4。`BINDINGS`)と、割り当て直し(SR-13)に書く動作の名前に食い違わないことを確かめる。
//!
//! 仮定した文書の形(英語・日本語で同じ。見出しの文言と表の頭の行だけが違う):
//!
//! - モードごとに見出し `## \`<モード>\``(`Mode::name()`。設定の `[keys.<モード>]` の名前)。
//!   見出しの段は `##` ちょうど。バッククォートで囲んだ見出しは、どれも知っているモードの名前であること。
//! - その下に3列の表。英語は `| Key | Action | Description |`、日本語は `| キー | 動作 | 説明 |`。
//!   2行目は区切り(`|---|---|---|` など)。
//! - 表の1行が1つの割り当て。キーと動作の名前はバッククォートで囲む(`` | `j` | `down` | Move down | ``)。
//!   同じ動作に複数のキーがあれば、1行に1キーで行を分ける。説明は空にしない。
//! - キーは `BINDINGS` の `key` と同じ表記(`j`・`Down`・`Ctrl+u`・`g g`・`Shift+Tab`)。
//!   `|` のキーは表の中なので `` `\|` `` と書く(GFM の表の決まり)。バッククォートのキーは `` `` ` `` `` と書く。
//! - 末尾に動作の名前の一覧の節。英語は `## Actions`、日本語は `## 動作の名前`。
//!   その節では、バッククォートを含む行ごとに最初の `` `name` `` を1つの動作の名前とする
//!   (`- \`down\`` の箇条書きでも、`` | `down` | ... | `` の表でもよい。表の頭と区切りの行は数えない)。
//!   一覧は `BINDINGS` に出る動作の名前の全部で、多くも少なくもない。パレットだけの動作(`COMMANDS` にだけあるもの)は求めない。
//!   それを書くなら別の `##` の節か、一覧の節の中の `###` の小見出しの下に置く(どちらも数えない)。
//! - 上の見出し以外の `##` の節(使い方の前置きなど)は数えない。
//! - 英語の文書には `](keys.ja.md)` のリンク、日本語の文書には `](keys.md)` のリンクがある。
use super::keymap::{lookup, parse_key, rebind, Action, Binding, Mode, BINDINGS};
use std::collections::{BTreeMap, BTreeSet};

struct Lang {
    file: &'static str,
    header: [&'static str; 3],
    actions_heading: &'static str,
    link_to: &'static str,
}

const EN: Lang = Lang {
    file: "docs/keys.md",
    header: ["Key", "Action", "Description"],
    actions_heading: "Actions",
    link_to: "keys.ja.md",
};

const JA: Lang = Lang {
    file: "docs/keys.ja.md",
    header: ["キー", "動作", "説明"],
    actions_heading: "動作の名前",
    link_to: "keys.md",
};

/// 文書の表の1行。
#[derive(Debug, Clone)]
struct Row {
    mode: String,
    key: String,
    action: String,
    description: String,
    line: usize,
}

struct Doc {
    text: String,
    rows: Vec<Row>,
    /// 動作の名前の一覧の節の名前と行番号。
    actions: Vec<(String, usize)>,
    /// 形の誤り(知らないモードの見出し、列の数の違い、バッククォートの無いセルなど)。
    errors: Vec<String>,
}

fn read_doc(lang: &Lang) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(lang.file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("[SR-22] 文書 {} が読めない: {e}", path.display()))
}

/// 表の行をセルに分ける。`\|` は区切りにせず `|` に戻す。
fn split_cells(line: &str) -> Vec<String> {
    let inner = line.trim();
    let inner = inner.strip_prefix('|').unwrap_or(inner);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    // 末尾の `\|` を削っていたら戻す。
    let inner = if line.trim().ends_with("\\|") {
        format!("{inner}|")
    } else {
        inner.to_string()
    };
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cur.push('|');
            chars.next();
        } else if c == '|' {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

/// 文字列の中の最初のコードスパン(バッククォートの数が同じ区切りで囲んだ所)の中身。
/// 中身の両端に空白が1つずつあり、空白だけでなければ、それを1つずつ除く(CommonMark の決まり)。
fn first_code_span(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && b[i] == b'`' {
            i += 1;
        }
        let n = i - start;
        let open_end = i;
        // 同じ数のバッククォートの閉じを探す。
        let mut j = open_end;
        while j < b.len() {
            if b[j] != b'`' {
                j += 1;
                continue;
            }
            let cs = j;
            while j < b.len() && b[j] == b'`' {
                j += 1;
            }
            if j - cs == n {
                let body = &s[open_end..cs];
                let trimmed = if body.len() >= 2
                    && body.starts_with(' ')
                    && body.ends_with(' ')
                    && !body.trim().is_empty()
                {
                    &body[1..body.len() - 1]
                } else {
                    body
                };
                return Some(trimmed.to_string());
            }
        }
        // 閉じが無ければ、ここは地の文。
        i = open_end;
    }
    None
}

/// セルがコードスパン1つだけ(前後に他の文字が無い)なら、その中身。
fn only_code_span(cell: &str) -> Option<String> {
    let cell = cell.trim();
    if !cell.starts_with('`') || !cell.ends_with('`') {
        return None;
    }
    first_code_span(cell)
}

fn is_separator(cells: &[String]) -> bool {
    cells
        .iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

enum Section {
    None,
    Mode(String),
    Actions,
}

fn parse_doc(lang: &Lang) -> Doc {
    let text = read_doc(lang);
    let mut rows = Vec::new();
    let mut actions = Vec::new();
    let mut errors = Vec::new();
    let mut section = Section::None;
    let mut seen_modes = BTreeSet::new();
    let mut seen_actions_heading = false;
    for (idx, line) in text.lines().enumerate() {
        let no = idx + 1;
        let t = line.trim();
        if let Some(h) = t.strip_prefix("## ") {
            let h = h.trim();
            if h == lang.actions_heading {
                if seen_actions_heading {
                    errors.push(format!("{}:{no}: 動作の名前の節が2つある", lang.file));
                }
                seen_actions_heading = true;
                section = Section::Actions;
            } else if let Some(name) = only_code_span(h) {
                if Mode::by_name(&name).is_none() {
                    errors.push(format!(
                        "{}:{no}: 知らないモードの見出し `{name}`",
                        lang.file
                    ));
                }
                if !seen_modes.insert(name.clone()) {
                    errors.push(format!("{}:{no}: モード `{name}` の節が2つある", lang.file));
                }
                section = Section::Mode(name);
            } else {
                section = Section::None;
            }
            continue;
        }
        if t.starts_with('#') {
            // 動作の名前の一覧の節は、`###` 以下の小見出し(パレットだけのコマンドなど)が来たら終える。
            // ほかの節では `#` の題や `###` 以下の小見出しは節を変えない。
            if matches!(section, Section::Actions) && t.starts_with("###") {
                section = Section::None;
            }
            continue;
        }
        match &section {
            Section::None => {}
            Section::Mode(mode) => {
                if !t.starts_with('|') {
                    continue;
                }
                let cells = split_cells(t);
                if is_separator(&cells) {
                    continue;
                }
                if cells.len() == 3 && cells.iter().zip(lang.header).all(|(c, h)| c == h) {
                    continue;
                }
                if cells.len() != 3 {
                    errors.push(format!(
                        "{}:{no}: 列の数が3でない({} 列): {t}",
                        lang.file,
                        cells.len()
                    ));
                    continue;
                }
                let Some(key) = only_code_span(&cells[0]) else {
                    errors.push(format!(
                        "{}:{no}: キーがバッククォートで囲まれていない: {t}",
                        lang.file
                    ));
                    continue;
                };
                let Some(action) = only_code_span(&cells[1]) else {
                    errors.push(format!(
                        "{}:{no}: 動作の名前がバッククォートで囲まれていない: {t}",
                        lang.file
                    ));
                    continue;
                };
                rows.push(Row {
                    mode: mode.clone(),
                    key,
                    action,
                    description: cells[2].clone(),
                    line: no,
                });
            }
            Section::Actions => {
                if t.starts_with('|') {
                    let cells = split_cells(t);
                    if is_separator(&cells) {
                        continue;
                    }
                    // 表の頭の行(バッククォートの無い行)は下で読み飛ばす。
                }
                if let Some(name) = first_code_span(t) {
                    actions.push((name, no));
                }
            }
        }
    }
    if !seen_actions_heading {
        errors.push(format!(
            "{}: 動作の名前の節(`## {}`)が無い",
            lang.file, lang.actions_heading
        ));
    }
    Doc {
        text,
        rows,
        actions,
        errors,
    }
}

type Triple = (String, String, String);

fn table_triples() -> BTreeSet<Triple> {
    BINDINGS
        .iter()
        .map(|b| {
            (
                b.mode.name().to_string(),
                b.key.to_string(),
                b.action.name().to_string(),
            )
        })
        .collect()
}

/// 割り当て直し(SR-13)に書ける動作の名前 = キーの表に出る動作の名前。
/// パレットだけの動作(`COMMANDS` にだけあるもの)は設定に書けないので一覧に求めない。
fn all_action_names() -> BTreeSet<String> {
    BINDINGS
        .iter()
        .map(|b| b.action.name().to_string())
        .collect()
}

fn fmt_triples<'a>(it: impl Iterator<Item = &'a Triple>) -> String {
    it.map(|(m, k, a)| format!("  [keys.{m}] `{k}` → `{a}`"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn check_bindings(lang: &Lang) {
    let doc = parse_doc(lang);
    assert!(
        doc.errors.is_empty(),
        "[SR-22] {} の形の誤り:\n{}",
        lang.file,
        doc.errors.join("\n")
    );
    let mut in_doc: BTreeMap<Triple, usize> = BTreeMap::new();
    let mut dups = Vec::new();
    for r in &doc.rows {
        let t = (r.mode.clone(), r.key.clone(), r.action.clone());
        if let Some(prev) = in_doc.insert(t, r.line) {
            dups.push(format!(
                "  {}:{} と {} 行目: [keys.{}] `{}` → `{}`",
                lang.file, r.line, prev, r.mode, r.key, r.action
            ));
        }
    }
    let doc_set: BTreeSet<Triple> = in_doc.keys().cloned().collect();
    let table = table_triples();
    let missing: Vec<&Triple> = table.difference(&doc_set).collect();
    let extra: Vec<&Triple> = doc_set.difference(&table).collect();
    let mut msg = String::new();
    if !missing.is_empty() {
        msg += &format!(
            "キーの表にあって {} に無い({}):\n{}\n",
            lang.file,
            missing.len(),
            fmt_triples(missing.into_iter())
        );
    }
    if !extra.is_empty() {
        msg += &format!(
            "{} にあってキーの表に無い({}):\n{}\n",
            lang.file,
            extra.len(),
            fmt_triples(extra.into_iter())
        );
    }
    if !dups.is_empty() {
        msg += &format!("{} で同じ行が2回ある:\n{}\n", lang.file, dups.join("\n"));
    }
    assert!(
        msg.is_empty(),
        "[SR-22] 文書とキーの表(BINDINGS)が食い違う:\n{msg}"
    );
}

fn check_action_names(lang: &Lang) {
    let doc = parse_doc(lang);
    assert!(
        doc.errors.is_empty(),
        "[SR-22] {} の形の誤り:\n{}",
        lang.file,
        doc.errors.join("\n")
    );
    let mut listed = BTreeSet::new();
    let mut dups = Vec::new();
    for (name, line) in &doc.actions {
        if !listed.insert(name.clone()) {
            dups.push(format!("  {}:{line}: `{name}`", lang.file));
        }
    }
    let want = all_action_names();
    let missing: Vec<&String> = want.difference(&listed).collect();
    let extra: Vec<&String> = listed.difference(&want).collect();
    let unknown: Vec<&String> = listed
        .iter()
        .filter(|n| Action::by_name(n).is_none())
        .collect();
    let mut msg = String::new();
    if !missing.is_empty() {
        msg += &format!("一覧に無い動作の名前: {missing:?}\n");
    }
    if !extra.is_empty() {
        msg += &format!("BINDINGS に出ない名前(パレットだけの動作は別の節へ): {extra:?}\n");
    }
    if !unknown.is_empty() {
        msg += &format!("Action::by_name で引けない名前: {unknown:?}\n");
    }
    if !dups.is_empty() {
        msg += &format!("2回ある名前:\n{}\n", dups.join("\n"));
    }
    assert!(
        msg.is_empty(),
        "[SR-22] {} の動作の名前の一覧(`## {}`)が食い違う:\n{msg}",
        lang.file,
        lang.actions_heading
    );
}

fn check_descriptions(lang: &Lang) {
    let doc = parse_doc(lang);
    assert!(
        doc.errors.is_empty(),
        "[SR-22] {} の形の誤り:\n{}",
        lang.file,
        doc.errors.join("\n")
    );
    assert!(!doc.rows.is_empty(), "[SR-22] {} に表の行が無い", lang.file);
    let empty: Vec<String> = doc
        .rows
        .iter()
        .filter(|r| r.description.trim().is_empty())
        .map(|r| {
            format!(
                "  {}:{}: [keys.{}] `{}` → `{}`",
                lang.file, r.line, r.mode, r.key, r.action
            )
        })
        .collect();
    assert!(
        empty.is_empty(),
        "[SR-22] 説明が空の行:\n{}",
        empty.join("\n")
    );
}

#[test]
fn test_sr_22_en_doc_matches_bindings() {
    // [SR-22] 英語の文書の (モード, キー, 動作) が、キーの表と多くも少なくもなく一致する。
    check_bindings(&EN);
}

#[test]
fn test_sr_22_ja_doc_matches_bindings() {
    // [SR-22] 日本語の文書の (モード, キー, 動作) が、キーの表と多くも少なくもなく一致する。
    check_bindings(&JA);
}

#[test]
fn test_sr_22_en_doc_lists_every_action_name() {
    // [SR-22] 英語の文書の動作の名前の一覧が、BINDINGS に出る動作の名前と一致する。
    check_action_names(&EN);
}

#[test]
fn test_sr_22_ja_doc_lists_every_action_name() {
    // [SR-22] 日本語の文書の動作の名前の一覧が、BINDINGS に出る動作の名前と一致する。
    check_action_names(&JA);
}

#[test]
fn test_sr_22_en_doc_descriptions_not_empty() {
    // [SR-22] 英語の文書の各行の説明が空でない。
    check_descriptions(&EN);
}

#[test]
fn test_sr_22_ja_doc_descriptions_not_empty() {
    // [SR-22] 日本語の文書の各行の説明が空でない。
    check_descriptions(&JA);
}

/// 既定の表の `table` のモードで、どの動作にも割り当たっていないキー(設定の表記と表の表記)。
fn free_table_key() -> (String, String) {
    // 設定の表記で書ける、修飾キーつきの英字を順に試す。
    let candidates = (b'a'..=b'z').flat_map(|c| {
        [
            format!("ctrl+alt+{}", c as char),
            format!("alt+{}", c as char),
        ]
    });
    for c in candidates {
        let c = c.as_str();
        let Ok(k) = parse_key(c) else { continue };
        if lookup(BINDINGS, Mode::Table, &k).is_none()
            && !BINDINGS
                .iter()
                .any(|b| b.mode == Mode::Table && b.key.starts_with(&format!("{k} ")))
        {
            return (c.to_string(), k);
        }
    }
    panic!("[SR-22] 試験に使える空きのキーが見つからない");
}

#[test]
fn test_sr_22_doc_action_names_rebind_in_config() {
    // [SR-22] 文書の table の節に書いた動作の名前を、設定の `[keys.table]` に別のキーで書くと、
    // そのキーでその動作が引ける(文書の名前がそのまま設定で使える)。英・日の両方の文書で確かめる。
    let (cfg_key, key) = free_table_key();
    for lang in [&EN, &JA] {
        let doc = parse_doc(lang);
        assert!(
            doc.errors.is_empty(),
            "[SR-22] {} の形の誤り:\n{}",
            lang.file,
            doc.errors.join("\n")
        );
        let names: BTreeSet<String> = doc
            .rows
            .iter()
            .filter(|r| r.mode == Mode::Table.name())
            .map(|r| r.action.clone())
            .collect();
        assert!(
            !names.is_empty(),
            "[SR-22] {} に `## `{}`` の節の行が無い",
            lang.file,
            Mode::Table.name()
        );
        for name in names {
            let mut table: Vec<Binding> = BINDINGS.to_vec();
            let warnings = rebind(
                &mut table,
                &[(
                    Mode::Table.name().to_string(),
                    cfg_key.clone(),
                    name.clone(),
                )],
            );
            assert!(
                warnings.is_empty(),
                "[SR-22] {} の `{name}` を [keys.{}] {cfg_key} に書くと警告: {warnings:?}",
                lang.file,
                Mode::Table.name()
            );
            let got = lookup(&table, Mode::Table, &key);
            assert_eq!(
                got.map(|a| a.name()),
                Some(name.as_str()),
                "[SR-22] {} の `{name}` を [keys.{}] {cfg_key} に書いても、そのキーで引けない",
                lang.file,
                Mode::Table.name()
            );
        }
    }
}

#[test]
fn test_sr_22_docs_link_each_other() {
    // [SR-22] 英語の文書から日本語の文書へ、日本語の文書から英語の文書へのリンクがある。
    for lang in [&EN, &JA] {
        let doc = parse_doc(lang);
        let link = format!("]({})", lang.link_to);
        assert!(
            doc.text.contains(&link),
            "[SR-22] {} に {} へのリンク(`{link}`)が無い",
            lang.file,
            lang.link_to
        );
    }
}

#[test]
fn test_sr_22_parser_reads_assumed_shape() {
    // [SR-22] 試験の読み方の確かめ(文書の形の仮定どおりに読めること)。
    assert_eq!(first_code_span("`j`"), Some("j".into()));
    assert_eq!(first_code_span("`` ` ``"), Some("`".into()));
    assert_eq!(first_code_span("- `down` — move"), Some("down".into()));
    assert_eq!(
        split_cells("| `\\|` | `filter` | Pipe |"),
        vec!["`|`", "`filter`", "Pipe"]
    );
    assert_eq!(
        split_cells("| `g g` | `top` | Go to the first row |"),
        vec!["`g g`", "`top`", "Go to the first row"]
    );
    assert!(is_separator(&split_cells("|---|:--|---:|")));
}
