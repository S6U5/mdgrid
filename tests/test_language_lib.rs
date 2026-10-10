//! language(specs/_changes/2026-10-03-language.md)タスク2の受け入れ: lib の文言(読むだけ・未対応・警告・誤り)が
//! 今の言語で出る(SR-23。specs/screen/spec.md)。
//!
//! 実装を見ずに、記録の「設計」・`mdgrid::i18n` の公開の口・既存の試験の使い方(tests/test_unwritable.rs・
//! tests/test_readonly_permission.rs・tests/test_empty_frontmatter.rs・tests/test_base.rs・tests/test_views.rs・
//! tests/test_print.rs・tests/test_date_format.rs)だけから書いた。
//!
//! 英語は `mdgrid::i18n::scoped(Lang::En)` のもとで確かめる(試験のスレッドだけが英語になる)。試験の環境は
//! .cargo/config.toml で日本語に固定されているので、guard の無い所と `scoped(Lang::Ja)` は今の日本語の文。
//! 実行ファイルは子の環境に LANG を明示し、LC_ALL・LC_MESSAGES は外す。
//!
//! 英語の文の検査: 空でなく、日本語の文字(ひらがな・カタカナ・漢字・「・」「「」「」」・「、」「。」と全角の記号)を
//! 含まない。差し込まれる値(キーの名前・パス・式・ビューの名前)は英字だけで作り、その値が文に出ることも見る。

use mdgrid::base::{self, Base, Shown};
use mdgrid::changes::Changes;
use mdgrid::config::parse as parse_config;
use mdgrid::i18n::{scoped, Lang};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{NewValue, RowId, Source, Value};
use mdgrid::types::{parse_date_input, DateFormat};
use mdgrid::views::load_views;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

// ---- 一時フォルダ(終わりに権限を戻して消す) ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-langlib-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, bytes).unwrap();
        p
    }

    fn mkdir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
            if let Ok(entries) = std::fs::read_dir(&self.0) {
                for e in entries.flatten() {
                    let _ =
                        std::fs::set_permissions(e.path(), std::fs::Permissions::from_mode(0o644));
                }
            }
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 文の検査 ----

/// 日本語の文字か(ひらがな・カタカナ(「・」を含む)・漢字・CJK の記号(「、」「。」「「」「」」)・全角の形)。
fn is_ja_char(c: char) -> bool {
    matches!(c,
        '\u{3000}'..='\u{303F}' // CJK の記号(、。「」など)
        | '\u{3040}'..='\u{309F}' // ひらがな
        | '\u{30A0}'..='\u{30FF}' // カタカナ(・を含む)
        | '\u{3400}'..='\u{4DBF}' // 漢字(拡張 A)
        | '\u{4E00}'..='\u{9FFF}' // 漢字
        | '\u{FF00}'..='\u{FFEF}' // 全角の形
    )
}

fn has_ja(s: &str) -> bool {
    s.chars().any(is_ja_char)
}

/// 英語の文: 空でなく、日本語の文字を含まない。落ちたときの報告の文を返す。
fn english_problem(what: &str, s: &str) -> Option<String> {
    if s.trim().is_empty() {
        return Some(format!("{what}: 文が空"));
    }
    if has_ja(s) {
        let ja: String = s.chars().filter(|c| is_ja_char(*c)).collect();
        return Some(format!(
            "{what}: 英語の文に日本語の文字 {ja:?} がある: {s:?}"
        ));
    }
    None
}

fn assert_english(what: &str, s: &str) {
    if let Some(p) = english_problem(what, s) {
        panic!("[SR-23] {p}");
    }
}

/// 日本語の文: 空でなく、日本語の文字を含む(今の日本語の文のまま)。
fn assert_japanese(what: &str, s: &str) {
    assert!(!s.trim().is_empty(), "[SR-23] {what}: 文が空");
    assert!(
        has_ja(s),
        "[SR-23] {what}: 日本語のときは今の日本語の文のはずが {s:?}"
    );
}

fn report(stage: &str, fails: Vec<String>) {
    assert!(
        fails.is_empty(),
        "[SR-23] 段「{stage}」で落ちたもの:\n  {}",
        fails.join("\n  ")
    );
}

// ---- Source の道具 ----

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load が終わらない");
}

fn row_of(src: &dyn Source, label: &str) -> Option<RowId> {
    src.rows().into_iter().find(|r| src.label(r) == label)
}

fn done() -> NewValue {
    NewValue::Str("done".into())
}

/// root なら 0444 のファイルでも書き込みに開けてしまう(tests/test_readonly_permission.rs と同じ確かめ方)。
#[cfg(unix)]
fn running_as_root() -> bool {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("rootcheck");
    let p = dir.write("probe.md", b"x");
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o444)).unwrap();
    std::fs::OpenOptions::new().write(true).open(&p).is_ok()
}

// ---- 読むだけの形の表 ----

/// 読むだけになる形1つ。
struct Form {
    /// 表の名前(報告に出す)。
    name: &'static str,
    /// 置くノートのファイル名(英字だけ)。
    file: &'static str,
    bytes: Vec<u8>,
    /// 日本語の理由に含まれるべき言葉(今の文。tests/test_unwritable.rs・tests/test_readonly_permission.rs と同じ)。
    /// 空なら日本語の文字を含むことだけを見る。
    ja_keyword: &'static str,
    /// 英語の理由に出るべき差し込みの値(キーの名前)。
    en_value: Option<&'static str>,
    /// ハードリンクの相手のファイル名。
    link: Option<&'static str>,
    /// 0444 にするか(書き込みの権限が無い形)。
    no_permission: bool,
    /// 開くときの add_frontmatter。
    add_frontmatter: bool,
}

fn form(name: &'static str, file: &'static str, bytes: &[u8], ja_keyword: &'static str) -> Form {
    Form {
        name,
        file,
        bytes: bytes.to_vec(),
        ja_keyword,
        en_value: None,
        link: None,
        no_permission: false,
        add_frontmatter: true,
    }
}

/// 表の全部の形。差し込まれる値(重なったキー)は英字の `dupkey`。
fn forms() -> Vec<Form> {
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(b"---\ntitle: t\nstatus: draft\n---\nbody\n");
    let mut not_utf8 = b"---\ntitle: ".to_vec();
    not_utf8.extend_from_slice(&[0xFF, 0xFE]);
    not_utf8.extend_from_slice(b"\nstatus: draft\n---\nbody\n");

    let mut v = vec![
        form("BOM", "bom.md", &bom, "BOM"),
        form(
            "閉じの区切りが無い",
            "unclosed.md",
            b"---\ntitle: t\nstatus: draft\n\nbody\n",
            "閉じていない",
        ),
        Form {
            en_value: Some("dupkey"),
            ..form(
                "同じキーが2回",
                "duplicate.md",
                b"---\ndupkey: 1\ndupkey: 2\n---\nbody\n",
                "同じキー",
            )
        },
        form(
            "YAML として読めない",
            "invalid.md",
            b"---\ntitle: [unclosed\nstatus: draft\n---\nbody\n",
            "読めない",
        ),
        form("UTF-8 でない", "notutf8.md", &not_utf8, "UTF-8"),
        form(
            "改行コードの混ざり",
            "mixed.md",
            b"---\ntitle: t\r\nstatus: draft\n---\nbody\n",
            "改行コード",
        ),
        Form {
            add_frontmatter: false,
            ..form(
                "フロントマターの無いノート(add_frontmatter = false)",
                "nofm.md",
                b"# heading\n\nstatus: draft\n",
                "",
            )
        },
        Form {
            add_frontmatter: false,
            ..form(
                "空のフロントマター(add_frontmatter = false)",
                "emptyfm.md",
                b"---\n---\nbody\n",
                "",
            )
        },
    ];
    #[cfg(unix)]
    {
        v.push(Form {
            link: Some("hardlink.md"),
            ..form(
                "ハードリンク",
                "hard.md",
                b"---\ntitle: t\nstatus: draft\n---\nbody\n",
                "ハードリンク",
            )
        });
        if !running_as_root() {
            v.push(Form {
                no_permission: true,
                ..form(
                    "書き込みの権限が無い",
                    "locked.md",
                    b"---\ntitle: t\nstatus: draft\n---\nbody\n",
                    "書き込めない権限",
                )
            });
        }
    }
    v
}

/// 形のノート(とハードリンクの相手)と書ける ok.md を置き、Markdown で開いて読み切る。
/// 呼ぶ側の言語(scoped)のもとで開く。
fn setup(f: &Form, tag: &str) -> (TempDir, Markdown) {
    let dir = TempDir::new(&format!("{tag}-{}", f.file.trim_end_matches(".md")));
    let note = dir.write(f.file, &f.bytes);
    #[cfg(unix)]
    if let Some(link) = f.link {
        std::fs::hard_link(&note, dir.path().join(link)).unwrap();
    }
    #[cfg(unix)]
    if f.no_permission {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&note, std::fs::Permissions::from_mode(0o444)).unwrap();
    }
    let _ = &note;
    dir.write("ok.md", b"---\nstatus: todo\n---\nok\n");
    let mut md = Markdown::open(&[dir.path().to_path_buf()]).expect("Markdown::open");
    md.set_add_frontmatter(f.add_frontmatter);
    load_all(&mut md);
    (dir, md)
}

// ---- 1. 読むだけの理由(セルの lock) ----

#[test]
fn test_sr_23_lib_read_only_reasons_are_english() {
    // [SR-23] [WB-5] [WB-3] 英語のとき、読むだけの形のどれも、セル(status・title・無いキー)の lock の理由が空でなく、
    // 日本語の文字を含まない。重なったキーの名前はそのまま差し込まれる。形ごとの理由は互いに違う。
    let _g = scoped(Lang::En);
    let mut fails = Vec::new();
    let mut reasons: Vec<(&str, String)> = Vec::new();
    for f in forms() {
        let (_dir, src) = setup(&f, "en-lock");
        let Some(row) = row_of(&src, f.file) else {
            fails.push(format!("{}: 行 {} が無い", f.name, f.file));
            continue;
        };
        for col in ["status", "title", "missingkey"] {
            match src.get(&row, col).lock {
                None => fails.push(format!("{} ({col}): lock が None", f.name)),
                Some(r) => {
                    if let Some(p) = english_problem(&format!("{} ({col})", f.name), &r) {
                        fails.push(p);
                    }
                    if let Some(v) = f.en_value {
                        if !r.contains(v) {
                            fails.push(format!("{} ({col}): 理由 {r:?} に {v:?} が無い", f.name));
                        }
                    }
                    if col == "status" {
                        reasons.push((f.name, r));
                    }
                }
            }
        }
        if let Some(ok) = row_of(&src, "ok.md") {
            if let Some(r) = src.get(&ok, "status").lock {
                fails.push(format!("{}: 隣の ok.md に lock {r:?}", f.name));
            }
        }
    }
    for (i, (a, ra)) in reasons.iter().enumerate() {
        for (b, rb) in &reasons[i + 1..] {
            if ra == rb {
                fails.push(format!("「{a}」と「{b}」の英語の理由が同じ {ra:?}"));
            }
        }
    }
    report("英語の lock の理由", fails);
}

#[test]
fn test_sr_23_lib_read_only_reasons_stay_japanese_under_ja() {
    // [SR-23] [WB-5] 同じノートを scoped(Ja) で開くと、今の日本語の文(形に固有の言葉を含む。権限は「書き込めない権限」)。
    // 英語の文とは違う。
    let mut fails = Vec::new();
    for f in forms() {
        let ja = {
            let _g = scoped(Lang::Ja);
            let (_dir, src) = setup(&f, "ja-lock");
            row_of(&src, f.file).and_then(|row| src.get(&row, "status").lock)
        };
        let en = {
            let _g = scoped(Lang::En);
            let (_dir, src) = setup(&f, "en-lock2");
            row_of(&src, f.file).and_then(|row| src.get(&row, "status").lock)
        };
        let Some(ja) = ja else {
            fails.push(format!("{}: 日本語で lock が None", f.name));
            continue;
        };
        if ja.trim().is_empty() || !has_ja(&ja) {
            fails.push(format!("{}: 日本語の理由が日本語でない {ja:?}", f.name));
        }
        if !ja.contains(f.ja_keyword) {
            fails.push(format!(
                "{}: 日本語の理由 {ja:?} に {:?} が無い",
                f.name, f.ja_keyword
            ));
        }
        if en.as_deref() == Some(ja.as_str()) {
            fails.push(format!("{}: 英語と日本語の理由が同じ {ja:?}", f.name));
        }
    }
    report("日本語の lock の理由", fails);
}

// ---- 2. Changes::set の Skip.reason ----

#[test]
fn test_sr_23_lib_changes_skip_reason_matches_lock_in_english() {
    // [SR-23] [CE-8] 英語のとき、Changes::set は Skip を返し、その理由はセルの lock と同じ英語の文。
    let _g = scoped(Lang::En);
    let mut fails = Vec::new();
    for f in forms() {
        let (_dir, src) = setup(&f, "en-skip");
        let Some(row) = row_of(&src, f.file) else {
            fails.push(format!("{}: 行が無い", f.name));
            continue;
        };
        let lock = src.get(&row, "status").lock;
        let mut ch = Changes::new();
        match ch.set(&src, &row, "status", done()) {
            Ok(()) => fails.push(format!("{}: 期待 Skip、実際 Ok", f.name)),
            Err(s) => {
                if Some(&s.reason) != lock.as_ref() {
                    fails.push(format!(
                        "{}: Skip の理由 {:?} がセルの lock {lock:?} と違う",
                        f.name, s.reason
                    ));
                }
                if let Some(p) = english_problem(&format!("{} (Skip)", f.name), &s.reason) {
                    fails.push(p);
                }
            }
        }
    }
    report("Changes::set の Skip", fails);
}

// ---- 3. 設定の警告 ----

#[test]
fn test_sr_23_lib_config_warnings_are_english() {
    // [SR-23] [CLI-3] 英語のとき、設定の知らない項目と型の違う値の警告は英語で、キーの名前はそのまま出る。
    let cases = [
        ("unknownitem = 1\n", "unknownitem"),
        ("[edit]\nadd_frontmatter = \"no\"\n", "add_frontmatter"),
        ("[edit]\nadd_frontmatter = 0\n", "add_frontmatter"),
        ("[terminal]\ncolor = \"maybe\"\n", "color"),
    ];
    let _g = scoped(Lang::En);
    let mut fails = Vec::new();
    for (text, key) in cases {
        let (_c, w) = parse_config(text).expect("警告にとどまる");
        if w.is_empty() {
            fails.push(format!("{text:?}: 警告が無い"));
        }
        for s in &w {
            if let Some(p) = english_problem(&format!("{text:?}"), s) {
                fails.push(p);
            }
            if !s.contains(key) {
                fails.push(format!("{text:?}: 警告 {s:?} にキー {key:?} が無い"));
            }
        }
    }
    report("設定の警告", fails);
}

#[test]
fn test_sr_23_lib_config_warnings_stay_japanese_under_ja() {
    // [SR-23] [CLI-3] scoped(Ja) では今の日本語の警告。
    let _g = scoped(Lang::Ja);
    for text in ["unknownitem = 1\n", "[edit]\nadd_frontmatter = \"no\"\n"] {
        let (_c, w) = parse_config(text).expect("警告にとどまる");
        assert_eq!(w.len(), 1, "{text:?}: {w:?}");
        assert_japanese(text, &w[0]);
    }
}

// ---- 4. .base の誤りと日付の読み取りの誤り ----

const TODAY: i64 = 20454;
const NOW: i64 = TODAY * 86_400;

/// 保管庫(`.obsidian/` を持つ)に2つのノートと `.base` を書き、開いて読み切る。
fn open_base(dir: &TempDir, base_text: &str) -> (Markdown, Base) {
    std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
    dir.write("a.md", b"---\nstatus: done\npriority: 2\n---\nA\n");
    dir.write("b.md", b"---\nstatus: todo\npriority: 1\n---\nB\n");
    let base_path = dir.write("tasks.base", base_text.as_bytes());
    let mut md = Markdown::open_vault(&base_path).expect("Markdown::open_vault");
    load_all(&mut md);
    let b = base::parse(base_text).unwrap_or_else(|e| panic!("base::parse failed: {e}"));
    (md, b)
}

fn build_err(b: &Base, view: usize, src: &dyn Source) -> String {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    match b.build(view, src, &prop, TODAY, NOW) {
        Ok(g) => panic!("build({view}) は Err のはずが Ok(行 {})", g.rows.len()),
        Err(e) => e,
    }
}

const BASE_TEXT: &str = r#"formulas:
  fixed: 'priority.toFixed(2)'
views:
  - type: table
    name: Linked
    filters:
      and:
        - link("Textbook")
    order:
      - file.name
  - type: cards
    name: Cards
    order:
      - file.name
  - type: table
    name: Formula
    order:
      - file.name
      - formula.fixed
"#;

#[test]
fn test_sr_23_lib_base_errors_are_english() {
    // [SR-23] [BV-7] [SC-8] 英語のとき、未対応の関数を使った filters の理由(関数名はそのまま)、未対応のビューの型の理由、
    // 無いビューの理由、評価できない列の notes とセルの理由が英語。
    let dir = TempDir::new("base-en");
    let _g = scoped(Lang::En);
    let (md, b) = open_base(&dir, BASE_TEXT);
    let src: &dyn Source = &md;
    let mut fails = Vec::new();

    let e = build_err(&b, 0, src);
    fails.extend(english_problem("未対応の関数の filters", &e));
    if !e.contains("link") {
        fails.push(format!("filters の理由 {e:?} に関数名 link が無い"));
    }

    let e = build_err(&b, 1, src);
    fails.extend(english_problem("未対応のビューの型", &e));
    if !e.contains("cards") {
        fails.push(format!("型の理由 {e:?} に cards が無い"));
    }

    let e = build_err(&b, 9, src);
    fails.extend(english_problem("無いビュー", &e));

    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    match b.build(2, src, &prop, TODAY, NOW) {
        Err(e) => fails.push(format!("Formula のビューは開くはずが Err {e:?}")),
        Ok(g) => {
            if g.notes.is_empty() {
                fails.push("評価できない列の notes が無い".into());
            }
            for n in &g.notes {
                fails.extend(english_problem("評価できない列の notes", n));
            }
            for row in &g.rows {
                match b.cell(src, &prop, row, "formula.fixed", TODAY, NOW) {
                    Shown::Unsupported(s) => {
                        fails.extend(english_problem("Shown::Unsupported の理由", &s))
                    }
                    other => fails.push(format!("formula.fixed が Unsupported でない: {other:?}")),
                }
            }
        }
    }

    match base::parse("views: [unclosed\n") {
        Ok(_) => fails.push("壊れた .base が読めてしまう".into()),
        Err(e) => fails.extend(english_problem("壊れた .base の base::parse", &e)),
    }
    report(".base の誤り", fails);
}

#[test]
fn test_sr_23_lib_base_errors_stay_japanese_under_ja() {
    // [SR-23] [BV-7] [SC-8] scoped(Ja) では今の日本語の理由(未対応の型は「未対応」を含む)。
    let dir = TempDir::new("base-ja");
    let _g = scoped(Lang::Ja);
    let (md, b) = open_base(&dir, BASE_TEXT);
    let src: &dyn Source = &md;
    let e = build_err(&b, 0, src);
    assert!(e.contains("link"), "{e}");
    assert_japanese("未対応の関数の filters", &e);
    let e = build_err(&b, 1, src);
    assert!(e.contains("cards") && e.contains("未対応"), "{e}");
}

#[test]
fn test_sr_23_lib_date_errors_are_english() {
    // [SR-23] [CE-22] [CE-5] 英語のとき、読めない日付の形と、読めない・実在しない日付の打ち込みの理由が英語。
    let _g = scoped(Lang::En);
    let mut fails = Vec::new();
    match DateFormat::parse("QQ") {
        Ok(_) => fails.push("形 QQ が読めてしまう".into()),
        Err(e) => {
            fails.extend(english_problem("DateFormat::parse(QQ)", &e));
            if !e.contains("QQ") {
                fails.push(format!("形の理由 {e:?} に QQ が無い"));
            }
        }
    }
    let iso = DateFormat::iso();
    for s in ["2026-02-30", "garbage", "99999-01-01"] {
        match parse_date_input(s, &iso, TODAY) {
            Ok(v) => fails.push(format!("{s:?} が読めてしまう: {v:?}")),
            Err(e) => fails.extend(english_problem(&format!("parse_date_input({s:?})"), &e)),
        }
    }
    report("日付の誤り", fails);
}

#[test]
fn test_sr_23_lib_date_errors_stay_japanese_under_ja() {
    // [SR-23] [CE-22] scoped(Ja) では今の日本語の理由。
    let _g = scoped(Lang::Ja);
    // DateFormat に Debug があるとは仮定しない(tests/test_date_format.rs と同じ)ので match で取り出す。
    match DateFormat::parse("QQ") {
        Ok(_) => panic!("形 QQ が読めてしまう"),
        Err(e) => assert_japanese("DateFormat::parse(QQ)", &e),
    }
    match parse_date_input("garbage", &DateFormat::iso(), TODAY) {
        Ok(v) => panic!("garbage が読めてしまう: {v:?}"),
        Err(e) => assert_japanese("parse_date_input(garbage)", &e),
    }
}

// ---- 5. views.toml の警告 ----

fn toml_str(s: &str) -> String {
    format!("{s:?}")
}

/// views.toml を書いて読む。(警告)
fn views_warnings(t: &TempDir, text: Option<&str>) -> Vec<String> {
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let real = std::fs::canonicalize(&notes).unwrap();
    let body = match text {
        Some(x) => x.to_string(),
        None => format!(
            "[[target]]\npath = {p}\n\n  [[target.view]]\n  name = \"Colored\"\n  order = [\"title\"]\n  hidden = []\n  filters_expr = []\n  colorful = 1\n",
            p = toml_str(real.to_str().unwrap()),
        ),
    };
    std::fs::write(conf.join("views.toml"), body).unwrap();
    load_views(&conf, &notes).1
}

const BROKEN_VIEWS: &str = "[[target]\npath = \"/x\n  [[target.view]]\n  name = \n";

#[test]
fn test_sr_23_lib_views_warnings_are_english() {
    // [SR-23] [BV-20] 英語のとき、views.toml の知らない項目(名前はそのまま)と壊れたファイルの警告が英語。
    let _g = scoped(Lang::En);
    let mut fails = Vec::new();

    let t = TempDir::new("views-unknown-en");
    let w = views_warnings(&t, None);
    if !w.iter().any(|s| s.contains("colorful")) {
        fails.push(format!("知らない項目の名前が警告に無い: {w:?}"));
    }
    for s in &w {
        fails.extend(english_problem("views.toml の知らない項目", s));
    }

    let t = TempDir::new("views-broken-en");
    let w = views_warnings(&t, Some(BROKEN_VIEWS));
    if w.is_empty() {
        fails.push("壊れた views.toml の警告が無い".into());
    }
    for s in &w {
        fails.extend(english_problem("壊れた views.toml", s));
    }
    report("views.toml の警告", fails);
}

#[test]
fn test_sr_23_lib_views_warnings_stay_japanese_under_ja() {
    // [SR-23] [BV-20] scoped(Ja) では今の日本語の警告。
    let _g = scoped(Lang::Ja);
    let t = TempDir::new("views-unknown-ja");
    let w = views_warnings(&t, None);
    let s = w
        .iter()
        .find(|s| s.contains("colorful"))
        .unwrap_or_else(|| panic!("知らない項目の警告が無い: {w:?}"));
    assert_japanese("views.toml の知らない項目", s);
    let t = TempDir::new("views-broken-ja");
    let w = views_warnings(&t, Some(BROKEN_VIEWS));
    assert!(!w.is_empty());
    for s in &w {
        assert_japanese("壊れた views.toml", s);
    }
}

// ---- 6. 実行ファイルの --print の警告 ----

const PRINT_BASE: &str = r#"formulas:
  mystery: 'nosuchfunc(priority)'
views:
  - type: table
    name: Broken
    order:
      - title
      - formula.mystery
"#;

/// notes/ に2つのノートと tasks.base を置き、`--print --view Broken` を子の環境の LANG で動かす。
fn run_print(name: &str, lang: &str) -> Output {
    let t = TempDir::new(name);
    let notes = t.mkdir("notes");
    std::fs::write(
        notes.join("a.md"),
        "---\ntitle: alpha\npriority: 2\n---\nA\n",
    )
    .unwrap();
    std::fs::write(
        notes.join("b.md"),
        "---\ntitle: beta\npriority: 1\n---\nB\n",
    )
    .unwrap();
    let base = notes.join("tasks.base");
    std::fs::write(&base, PRINT_BASE).unwrap();
    let conf = t.mkdir("xdg-config");
    let state = t.mkdir("xdg-state");
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(&base)
        .args(["--print", "--view", "Broken"])
        .current_dir(t.path())
        .env("XDG_CONFIG_HOME", &conf)
        .env("XDG_STATE_HOME", &state)
        .env("LANG", lang)
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .output()
        .expect("mdgrid を起動できる")
}

fn stderr_lines(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn test_sr_23_lib_print_warning_is_english_with_lang_en() {
    // [SR-23] [CLI-5] LANG=en_US.UTF-8 で `--print` → 評価できない列の警告(標準エラーに1行)が英語。終了コード 0。
    let out = run_print("print-en", "en_US.UTF-8");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let w = stderr_lines(&out);
    assert_eq!(w.len(), 1, "標準エラーに1行: {w:?}");
    assert_english("--print の評価できない列の警告", &w[0]);
}

#[test]
fn test_sr_23_lib_print_warning_is_japanese_with_lang_ja() {
    // [SR-23] [CLI-5] LANG=ja_JP.UTF-8 では今と同じ日本語の警告。
    let out = run_print("print-ja", "ja_JP.UTF-8");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let w = stderr_lines(&out);
    assert_eq!(w.len(), 1, "標準エラーに1行: {w:?}");
    assert_japanese("--print の評価できない列の警告", &w[0]);
}
