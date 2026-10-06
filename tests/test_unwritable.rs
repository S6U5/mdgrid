//! 書かないノート(異常系)を端から端まで確かめる受け入れテスト(WB-5。関係: WB-3・WB-6・CE-8・CE-10)。
//! 仕様: specs/write-back/spec.md、specs/cell-edit/spec.md。記録: specs/_changes/2026-10-02-abnormal-tests.md。
//!
//! WB-5 の7つの形(同じキーが2回・BOM・YAML として読めない・閉じない・UTF-8 でない・改行コードが混ざる・
//! ハードリンク)と、対照の書ける3つ(フロントマターの無いノート(WB-3)・空のフロントマターのノート(WB-3。
//! 2026-10-02 empty-frontmatter-config で書ける側へ移した)・普通のノート)を
//! 1つの表にして、段ごと(読み取り・セルの lock と理由・apply・Changes の set と set_many・保存の流れ)に回す。
//! 1つの段の試験は表の全部の形を回してから、落ちた形をまとめて報告する(どの形のどの段かを分かるように)。

use mdgrid::changes::{Changes, Outcome};
use mdgrid::frontmatter::{parse, ReadOnly};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{Edit, EditError, NewValue, RowId, SaveError, Source};
use mdgrid::writeback::{apply, baseline, save};
use std::path::{Path, PathBuf};

// ---- 一時フォルダ(tests/test_changes.rs から写す) ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-{}-{}-{}",
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
        std::fs::write(&p, bytes).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

// ---- 表 ----

/// 1つの形。`expect` が None なら書ける側(対照)。
struct Form {
    /// 表の名前(落ちたときの報告に出す)。
    name: &'static str,
    /// 置くノートのファイル名。
    file: &'static str,
    bytes: Vec<u8>,
    /// frontmatter::parse の期待。ハードリンクはバイト列が普通なので Ok(())、フロントマターの無いノートは
    /// Err(NoFrontmatter)(書ける側)。
    parse: Result<(), ReadOnly>,
    /// 書かない理由(ReadOnly)。None は書ける側。
    expect: Option<ReadOnly>,
    /// セルの lock の理由に含まれるべき、形に固有の言葉。ほかの形の理由には含まれない。
    keyword: &'static str,
    /// ハードリンクを張る相手のファイル名(ハードリンクの形だけ)。
    link: Option<&'static str>,
}

/// UTF-8 でないバイト列と BOM のバイト列は、ここ(Rust の中)で作る。ほかは tests/fixtures の材料。
fn forms() -> Vec<Form> {
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(b"---\ntitle: t\nstatus: draft\n---\nbody\n");
    let mut not_utf8 = b"---\ntitle: ".to_vec();
    not_utf8.extend_from_slice(&[0xFF, 0xFE]);
    not_utf8.extend_from_slice(b"\nstatus: draft\n---\nbody\n");
    assert_eq!(bom, fixture("bom.md"), "BOM の材料は fixtures と同じ");
    assert_eq!(
        not_utf8,
        fixture("not_utf8.md"),
        "UTF-8 でない材料は fixtures と同じ"
    );

    let mut v = vec![
        Form {
            name: "同じキーが2回",
            file: "duplicate.md",
            bytes: fixture("duplicate_key.md"),
            parse: Err(ReadOnly::DuplicateKey("a".into())),
            expect: Some(ReadOnly::DuplicateKey("a".into())),
            keyword: "同じキー",
            link: None,
        },
        Form {
            name: "BOM",
            file: "bom.md",
            bytes: bom,
            parse: Err(ReadOnly::Bom),
            expect: Some(ReadOnly::Bom),
            keyword: "BOM",
            link: None,
        },
        Form {
            name: "YAML として読めない",
            file: "invalid.md",
            bytes: fixture("invalid_yaml.md"),
            parse: Err(ReadOnly::InvalidYaml),
            expect: Some(ReadOnly::InvalidYaml),
            keyword: "読めない",
            link: None,
        },
        Form {
            name: "閉じない",
            file: "unclosed.md",
            bytes: fixture("unclosed.md"),
            parse: Err(ReadOnly::Unclosed),
            expect: Some(ReadOnly::Unclosed),
            keyword: "閉じていない",
            link: None,
        },
        Form {
            name: "UTF-8 でない",
            file: "notutf8.md",
            bytes: not_utf8,
            parse: Err(ReadOnly::NotUtf8),
            expect: Some(ReadOnly::NotUtf8),
            keyword: "UTF-8",
            link: None,
        },
        Form {
            name: "改行コードが混ざる",
            file: "mixed.md",
            bytes: fixture("mixed_newlines.md"),
            parse: Err(ReadOnly::MixedNewlines),
            expect: Some(ReadOnly::MixedNewlines),
            keyword: "改行コード",
            link: None,
        },
    ];
    if cfg!(unix) {
        v.push(Form {
            name: "ハードリンク",
            file: "hard.md",
            bytes: fixture("plain.md"),
            parse: Ok(()),
            expect: Some(ReadOnly::HardLink),
            keyword: "ハードリンク",
            link: Some("hard-link.md"),
        });
    }
    // 対照: 書ける側。
    v.push(Form {
        name: "フロントマターが無い(WB-3)",
        file: "nofm.md",
        bytes: fixture("no_frontmatter.md"),
        parse: Err(ReadOnly::NoFrontmatter),
        expect: None,
        keyword: "",
        link: None,
    });
    // 対照: 空のフロントマター(WB-3。parse は見分けて EmptyFrontmatter を返すが、区切りの間に足して書ける)。
    v.push(Form {
        name: "空のフロントマター(WB-3)",
        file: "empty.md",
        bytes: fixture("empty_frontmatter.md"),
        parse: Err(ReadOnly::EmptyFrontmatter),
        expect: None,
        keyword: "",
        link: None,
    });
    v.push(Form {
        name: "普通のノート",
        file: "plain.md",
        bytes: fixture("plain.md"),
        parse: Ok(()),
        expect: None,
        keyword: "",
        link: None,
    });
    v
}

/// 読むだけの形の、ほかの形の言葉(取り違えの検査に使う)。
fn keywords(forms: &[Form]) -> Vec<(&'static str, &'static str)> {
    forms
        .iter()
        .filter(|f| f.expect.is_some())
        .map(|f| (f.name, f.keyword))
        .collect()
}

/// 書ける相手のノート(set_many と保存の流れで、書ける行が書かれることを確かめる)。
const OK: &[u8] = b"---\nstatus: todo\n---\nok\n";

/// 形のノート(とハードリンクの相手)と ok.md を置いて、Markdown で開いて読み切る。
fn setup(f: &Form, tag: &str) -> (TempDir, Markdown) {
    let dir = TempDir::new(&format!(
        "unwritable-{tag}-{}",
        f.file.trim_end_matches(".md")
    ));
    let note = dir.write(f.file, &f.bytes);
    if let Some(link) = f.link {
        std::fs::hard_link(&note, dir.path().join(link)).unwrap();
    }
    dir.write("ok.md", OK);
    let mut src = Markdown::open(&[dir.path().to_path_buf()]).expect("open");
    let mut guard = 0;
    while !src.load(100).done {
        guard += 1;
        assert!(guard < 10_000, "load が終わらない");
    }
    (dir, src)
}

fn row_of(src: &Markdown, name: &str) -> Option<RowId> {
    src.rows().into_iter().find(|r| src.label(r) == name)
}

fn done() -> NewValue {
    NewValue::Str("done".into())
}

fn edit_done() -> Vec<Edit> {
    vec![Edit {
        key: "status".into(),
        value: done(),
    }]
}

/// 形のファイル(とハードリンクの相手)の今のバイトが、元のバイトと同じか。違えば報告の文を返す。
fn unchanged(dir: &TempDir, f: &Form) -> Result<(), String> {
    for name in std::iter::once(f.file).chain(f.link) {
        let now = std::fs::read(dir.path().join(name)).unwrap();
        if now != f.bytes {
            return Err(format!(
                "{name} のバイトが変わった: 前 {:?} → 後 {:?}",
                String::from_utf8_lossy(&f.bytes),
                String::from_utf8_lossy(&now)
            ));
        }
    }
    Ok(())
}

/// 落ちた形をまとめて報告する。
fn report(stage: &str, fails: Vec<String>) {
    assert!(
        fails.is_empty(),
        "段「{stage}」で落ちた形:\n  {}",
        fails.join("\n  ")
    );
}

// ---- 段: 読み取り ----

#[test]
fn test_wb_5_parse_returns_expected_read_only() {
    // [WB-5] [WB-3] frontmatter::parse が形ごとの ReadOnly を返す。ハードリンクはバイト列が普通なので Ok
    // (ハードリンクは save と Source が見る)。フロントマターの無いノートは NoFrontmatter(書ける側。WB-3)。
    let mut fails = Vec::new();
    for f in forms() {
        let got = parse(&f.bytes).map(|_| ());
        if got != f.parse {
            fails.push(format!("{}: 期待 {:?}、実際 {:?}", f.name, f.parse, got));
        }
    }
    report("parse", fails);
}

// ---- 段: セルの lock と理由 ----

#[test]
fn test_wb_5_cell_lock_has_form_specific_reason() {
    // [WB-5] [CE-8] source の Markdown で、形のノートのセルの lock が Some で理由が空でなく、形に固有の言葉を含み、
    // ほかの形の言葉を含まない(理由を取り違えない)。7つの理由は互いに違う。書ける側(空のフロントマターを含む。WB-3)は lock が None。
    let all = forms();
    let words = keywords(&all);
    let mut fails = Vec::new();
    let mut reasons: Vec<(&str, String)> = Vec::new();
    for f in &all {
        let (_dir, src) = setup(f, "lock");
        let names: Vec<&str> = std::iter::once(f.file).chain(f.link).collect();
        for name in names {
            let Some(row) = row_of(&src, name) else {
                fails.push(format!("{}: 行 {name} が表に無い", f.name));
                continue;
            };
            for col in ["status", "title", "a"] {
                let lock = src.get(&row, col).lock;
                match (&f.expect, &lock) {
                    (None, None) => {}
                    (None, Some(r)) => fails.push(format!(
                        "{} ({name}, {col}): 書ける側のはずが lock {r:?}",
                        f.name
                    )),
                    (Some(_), None) => fails.push(format!(
                        "{} ({name}, {col}): lock が None(書けてしまう)",
                        f.name
                    )),
                    (Some(_), Some(r)) => {
                        if r.trim().is_empty() {
                            fails.push(format!("{} ({name}, {col}): 理由が空", f.name));
                        }
                        if !r.contains(f.keyword) {
                            fails.push(format!(
                                "{} ({name}, {col}): 理由 {r:?} に {:?} が無い",
                                f.name, f.keyword
                            ));
                        }
                        for (other, w) in &words {
                            if *other != f.name && r.contains(w) {
                                fails.push(format!(
                                    "{} ({name}, {col}): 理由 {r:?} がほかの形「{other}」の言葉 {w:?} を含む",
                                    f.name
                                ));
                            }
                        }
                        if col == "status" && name == f.file {
                            reasons.push((f.name, r.clone()));
                        }
                    }
                }
            }
        }
        // 隣の書けるノートは書けるまま(行ごとの lock が漏れない)。
        let ok = row_of(&src, "ok.md").expect("ok.md");
        if let Some(r) = src.get(&ok, "status").lock {
            fails.push(format!("{}: 隣の ok.md に lock {r:?}", f.name));
        }
    }
    for (i, (a, ra)) in reasons.iter().enumerate() {
        for (b, rb) in &reasons[i + 1..] {
            if ra == rb {
                fails.push(format!("「{a}」と「{b}」の理由が同じ {ra:?}"));
            }
        }
    }
    report("lock と理由", fails);
}

// ---- 段: apply と writeback::save ----

#[test]
fn test_wb_5_apply_is_err() {
    // [WB-5] [WB-6] writeback::apply(バイト列に編集を当てる)が形ごとの EditError::ReadOnly を返す。
    // ハードリンクはバイト列だけでは分からないので apply は対象外(ファイルを見る save の段で確かめる)。
    // 書ける側は Ok で、status: done が入る。
    let mut fails = Vec::new();
    for f in forms() {
        let got = apply(&f.bytes, &edit_done());
        match (&f.expect, &got) {
            (Some(ReadOnly::HardLink), _) => {}
            (Some(e), Err(EditError::ReadOnly(r))) if r == e => {}
            (Some(e), _) => fails.push(format!(
                "{}: 期待 Err(ReadOnly({e:?}))、実際 {:?}",
                f.name,
                got.as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
            )),
            (None, Ok(b)) if String::from_utf8_lossy(b).contains("status: done") => {}
            (None, _) => fails.push(format!(
                "{}: 書ける側のはずが {:?}",
                f.name,
                got.as_ref()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
            )),
        }
    }
    report("apply", fails);
}

#[test]
fn test_wb_5_writeback_save_refuses_and_keeps_bytes() {
    // [WB-5] [WB-6] writeback::save(ファイルに書く)が形ごとの ReadOnly で止まり、元のファイル(ハードリンクは
    // 両方)が1バイトも変わらない。書ける側は書ける。
    let mut fails = Vec::new();
    for f in forms() {
        let (dir, _src) = setup(&f, "wbsave");
        let path = dir.path().join(f.file);
        let base = baseline(&path).unwrap();
        let got = save(&path, &base, &edit_done());
        match (&f.expect, &got) {
            (Some(e), Err(SaveError::Edit(EditError::ReadOnly(r)))) if r == e => {}
            (Some(e), _) => fails.push(format!(
                "{}: 期待 Err(Edit(ReadOnly({e:?})))、実際 {got:?}",
                f.name
            )),
            (None, Ok(_)) => {}
            (None, Err(e)) => fails.push(format!("{}: 書ける側のはずが {e:?}", f.name)),
        }
        if f.expect.is_some() {
            if let Err(m) = unchanged(&dir, &f) {
                fails.push(format!("{}: {m}", f.name));
            }
        } else {
            let now = std::fs::read(&path).unwrap();
            if !String::from_utf8_lossy(&now).contains("status: done") {
                fails.push(format!(
                    "{}: 書ける側なのに status: done が書かれない: {:?}",
                    f.name,
                    String::from_utf8_lossy(&now)
                ));
            }
        }
    }
    report("writeback::save", fails);
}

// ---- 段: Changes の set と set_many ----

#[test]
fn test_wb_5_changes_set_is_skipped_with_reason() {
    // [WB-5] [CE-8] Changes::set は Skip(行と、セルの lock と同じ理由)を返し、何もためない。書ける側はためる。
    let mut fails = Vec::new();
    for f in forms() {
        let (_dir, src) = setup(&f, "set");
        for name in std::iter::once(f.file).chain(f.link) {
            let Some(row) = row_of(&src, name) else {
                fails.push(format!("{}: 行 {name} が無い", f.name));
                continue;
            };
            let lock = src.get(&row, "status").lock;
            let mut ch = Changes::new();
            let got = ch.set(&src, &row, "status", done());
            match (&f.expect, got) {
                (Some(_), Err(s)) => {
                    if s.row != row {
                        fails.push(format!("{} ({name}): Skip の行が違う {:?}", f.name, s.row));
                    }
                    if s.reason.trim().is_empty() || Some(&s.reason) != lock.as_ref() {
                        fails.push(format!(
                            "{} ({name}): Skip の理由 {:?} がセルの lock {lock:?} と違う",
                            f.name, s.reason
                        ));
                    }
                    if ch.count() != 0 {
                        fails.push(format!("{} ({name}): ためた数が {}", f.name, ch.count()));
                    }
                }
                (Some(_), Ok(())) => fails.push(format!(
                    "{} ({name}): 期待 Skip、実際 Ok(ためた数 {})",
                    f.name,
                    ch.count()
                )),
                (None, Ok(())) => {
                    if ch.pending(&row, "status") != Some(&done()) {
                        fails.push(format!("{} ({name}): 書ける側なのにたまらない", f.name));
                    }
                }
                (None, Err(s)) => fails.push(format!(
                    "{} ({name}): 書ける側のはずが Skip {:?}",
                    f.name, s.reason
                )),
            }
        }
    }
    report("Changes::set", fails);
}

#[test]
fn test_wb_5_changes_set_many_skips_count_and_reason() {
    // [WB-5] [CE-10] 形のノート(ハードリンクは両方の行)と ok.md を選んで set_many → 形の行は飛ばされ、
    // 飛ばした数と理由(セルの lock と同じ)が返り、ためるのは書ける行だけ。書ける側は飛ばさない(WB-3)。
    let mut fails = Vec::new();
    for f in forms() {
        let (_dir, src) = setup(&f, "many");
        let mine: Vec<RowId> = std::iter::once(f.file)
            .chain(f.link)
            .filter_map(|n| row_of(&src, n))
            .collect();
        let ok = row_of(&src, "ok.md").expect("ok.md");
        let mut rows = mine.clone();
        rows.push(ok.clone());
        let mut ch = Changes::new();
        let skips = ch.set_many(&src, &rows, "status", done());
        let want_skips = if f.expect.is_some() { mine.len() } else { 0 };
        if skips.len() != want_skips {
            fails.push(format!(
                "{}: 飛ばした数 期待 {want_skips}、実際 {} ({skips:?})",
                f.name,
                skips.len()
            ));
        }
        for s in &skips {
            let lock = src.get(&s.row, "status").lock;
            if !mine.contains(&s.row)
                || s.reason.trim().is_empty()
                || Some(&s.reason) != lock.as_ref()
            {
                fails.push(format!(
                    "{}: Skip {:?} の行か理由が違う(lock {lock:?})",
                    f.name, s
                ));
            }
        }
        let want_count = 1 + if f.expect.is_some() { 0 } else { mine.len() };
        if ch.count() != want_count {
            fails.push(format!(
                "{}: ためた数 期待 {want_count}、実際 {}",
                f.name,
                ch.count()
            ));
        }
        if ch.pending(&ok, "status") != Some(&done()) {
            fails.push(format!("{}: 隣の ok.md がたまらない", f.name));
        }
    }
    report("Changes::set_many", fails);
}

// ---- 段: 保存の流れ ----

#[test]
fn test_wb_5_save_flow_keeps_bytes() {
    // [WB-5] [CE-10] [WB-6] set と set_many を試してから Changes::save(保存の流れ)を通しても、形のファイル
    // (ハードリンクは両方)のバイトは1バイトも変わらず、保存の結果に形の行は出ない。隣の ok.md は書かれる。
    // 書ける側(フロントマターの無いノートは先頭にフロントマターを足して。WB-3)は書かれる。
    let mut fails = Vec::new();
    for f in forms() {
        let (dir, mut src) = setup(&f, "flow");
        let mine: Vec<RowId> = std::iter::once(f.file)
            .chain(f.link)
            .filter_map(|n| row_of(&src, n))
            .collect();
        let ok = row_of(&src, "ok.md").expect("ok.md");
        let mut ch = Changes::new();
        for r in &mine {
            let _ = ch.set(&src, r, "status", done());
        }
        let mut rows = mine.clone();
        rows.push(ok.clone());
        let _ = ch.set_many(&src, &rows, "status", done());
        let outcomes = ch.save(&mut src);
        for (r, o) in &outcomes {
            let ours = mine.contains(r);
            match (f.expect.is_some(), ours, o) {
                (true, true, _) => fails.push(format!(
                    "{}: 読むだけの行 {} が保存の結果に出た {o:?}",
                    f.name,
                    src.label(r)
                )),
                (_, _, Outcome::Saved) => {}
                (_, _, o) => fails.push(format!("{}: 行 {} の保存が {o:?}", f.name, src.label(r))),
            }
        }
        if f.expect.is_some() {
            if let Err(m) = unchanged(&dir, &f) {
                fails.push(format!("{}: {m}", f.name));
            }
        } else {
            let now = std::fs::read(dir.path().join(f.file)).unwrap();
            if !String::from_utf8_lossy(&now).contains("status: done") {
                fails.push(format!(
                    "{}: 書ける側なのに書かれない: {:?}",
                    f.name,
                    String::from_utf8_lossy(&now)
                ));
            }
        }
        let ok_now = std::fs::read(dir.path().join("ok.md")).unwrap();
        if ok_now != b"---\nstatus: done\n---\nok\n" {
            fails.push(format!(
                "{}: 隣の ok.md が {:?}",
                f.name,
                String::from_utf8_lossy(&ok_now)
            ));
        }
    }
    report("保存の流れ", fails);
}
