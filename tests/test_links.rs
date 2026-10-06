//! タスク 1(links-this)の受け入れ: リンク・被リンク・`.base` を開いたときの this(BV-22。関係: BV-6・BV-7)。
//! 仕様: specs/base-view/spec.md の BV-22、specs/_changes/2026-10-06-links-this.md の「不明点と仮定」。
//! 本物の実行ファイルの `--print --format json` で、一時の保管庫(根に `.obsidian/`)と `.base` を作って確かめる。
//!
//! 仮定(記録のとおり): リンクの値は行き先のノートの根からのパス(拡張子なし)の文字。解けないリンクは書いた文字のまま。
//! 1つのノートの同じ行き先は1つにまとめる。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-links-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(dir.join(".obsidian")).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `.base` を開いて `--print --format json` の標準出力。終わりの状態が成功でなければ落とす。
fn print(dir: &TempDir, base_rel: &str, base: &str) -> String {
    let bp = dir.write(base_rel, base);
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(&bp)
        .args(["--print", "--format", "json"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        out.status.success(),
        "--print が失敗: {:?}\nstdout: {stdout}\nstderr: {stderr}",
        out.status
    );
    stdout
}

/// 列を1つ出すビュー(名前の順)。
fn view(filter: &str, cols: &[&str]) -> String {
    let filter = if filter.is_empty() {
        String::new()
    } else {
        format!("    filters: '{filter}'\n")
    };
    let order: String = cols.iter().map(|c| format!("      - {c}\n")).collect();
    format!(
        "views:\n  - type: table\n    name: v\n{filter}    order:\n{order}    sort:\n      - property: file.path\n        direction: ASC\n"
    )
}

/// json の行(1行に1つのノート)のうち、`"file name": "<name>"` を含む行。
fn row<'a>(out: &'a str, name: &str) -> &'a str {
    let key = format!("\"file name\": \"{name}\"");
    out.lines()
        .find(|l| l.contains(&key))
        .unwrap_or_else(|| panic!("{name} の行が無い:\n{out}"))
}

/// 出た行のノートの名前(順のまま)。
fn names(out: &str) -> Vec<String> {
    out.lines()
        .filter_map(|l| {
            let i = l.find("\"file name\": \"")? + "\"file name\": \"".len();
            let rest = &l[i..];
            Some(rest[..rest.find('"')?].to_string())
        })
        .collect()
}

#[test]
fn test_bv_22_links_wikilink_and_markdown_link() {
    // [BV-22] a.md の本文に [[b]] と [x](sub/c.md) → a の file.links に b と sub/c
    let d = TempDir::new("basic");
    d.write("a.md", "本文 [[b]] と [x](sub/c.md)\n");
    d.write("b.md", "b\n");
    d.write("sub/c.md", "c\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"b\", \"sub/c\"]"),
        "{out}"
    );
    assert!(row(&out, "b.md").contains("\"file links\": []"), "{out}");
}

#[test]
fn test_bv_22_link_forms_alias_heading_embed() {
    // [BV-22] [[b|表示]]・[[b#見出し]]・![[d]] は行き先 b・d(同じ行き先は1つ)
    let d = TempDir::new("forms");
    d.write("a.md", "[[b|表示]] と [[b#見出し]]\n\n![[d]]\n");
    d.write("b.md", "b\n");
    d.write("d.md", "d\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"b\", \"d\"]"),
        "{out}"
    );
}

#[test]
fn test_bv_22_links_in_code_are_not_counted() {
    // [BV-22] コードの区画とインラインのコードの中のリンクは拾わない
    let d = TempDir::new("code");
    d.write(
        "a.md",
        "```\n[[x]]\n```\n\nここは `[[y]]` で、[[b]] だけ\n\n~~~md\n[z](z.md)\n~~~\n",
    );
    d.write("b.md", "b\n");
    d.write("x.md", "x\n");
    d.write("y.md", "y\n");
    d.write("z.md", "z\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"b\"]"),
        "{out}"
    );
}

#[test]
fn test_bv_22_unresolved_link_stays_text() {
    // [BV-22] [[無い]] は文字 "無い" のまま入る
    let d = TempDir::new("unresolved");
    d.write("a.md", "[[無い]]\n");
    d.write("b.md", "b\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"無い\"]"),
        "{out}"
    );
}

#[test]
fn test_bv_22_same_name_resolves_nearest_root() {
    // [BV-22] 同じ名前のノートが複数 → 根に近い・パスの短いもの
    let d = TempDir::new("same-name");
    d.write("a.md", "[[b]]\n");
    d.write("x/deep/b.md", "b1\n");
    d.write("y/b.md", "b2\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"y/b\"]"),
        "{out}"
    );
    // 根からのパスで書けばそのノート。
    let d2 = TempDir::new("same-name-path");
    d2.write("a.md", "[[x/deep/b]]\n");
    d2.write("x/deep/b.md", "b1\n");
    d2.write("y/b.md", "b2\n");
    let out = print(&d2, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"x/deep/b\"]"),
        "{out}"
    );
}

#[test]
fn test_bv_22_backlinks_and_has_link() {
    // [BV-22] file.backlinks はそのノートを指すノートのリスト、file.hasLink(x) は x を指すか(決定 links-backlinks)。
    let d = TempDir::new("backlinks");
    d.write("a.md", "[[b]]\n");
    d.write("c.md", "---\nrel: \"[[b]]\"\n---\n");
    d.write("b.md", "x\n");
    let print = |filter: &str, cols: &[&str]| {
        let bp = d.write("V.base", &view(filter, cols));
        let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
            .arg(&bp)
            .args(["--print", "--with-path"])
            .current_dir(d.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    let out = print("file.hasLink(\"b\")", &["file.name"]);
    let rows: Vec<&str> = out.lines().skip(1).collect();
    assert_eq!(rows.len(), 2, "{out}");
    assert!(
        rows.iter()
            .all(|r| r.contains("a.md") || r.contains("c.md")),
        "{out}"
    );
    let out = print("file.name == \"b.md\"", &["file.name", "file.backlinks"]);
    let row = out.lines().nth(1).unwrap_or_default();
    assert!(
        row.contains("a") && row.contains("c"),
        "b の被リンク: {out}"
    );
}

#[test]
fn test_bv_22_links_contains_name() {
    // [BV-22] file.links.contains("b") は a で真(b を指さないノートは偽)
    let d = TempDir::new("contains");
    d.write("a.md", "[[b]]\n");
    d.write("b.md", "b\n");
    d.write("c.md", "[[d]]\n");
    let out = print(
        &d,
        "V.base",
        &view("file.links.contains(\"b\")", &["file.name"]),
    );
    assert_eq!(names(&out), vec!["a.md"], "{out}");
}

#[test]
fn test_bv_22_frontmatter_links() {
    // [BV-22] フロントマターの値の [[…]] も file.links に入る
    let d = TempDir::new("frontmatter");
    d.write(
        "a.md",
        "---\nrelated: \"[[b]]\"\nup: [\"[[c]]\"]\n---\n本文\n",
    );
    d.write("b.md", "b\n");
    d.write("c.md", "c\n");
    let out = print(&d, "V.base", &view("", &["file.name", "file.links"]));
    assert!(
        row(&out, "a.md").contains("\"file links\": [\"b\", \"c\"]"),
        "{out}"
    );
}

const THIS_BASE: &str = r#"formulas:
  名前: 'this.file.name'
  場所: 'this.file.folder'
views:
  - type: table
    name: here
    filters: 'file.inFolder(this.file.folder)'
    order:
      - file.name
      - formula.名前
      - formula.場所
    sort:
      - property: file.path
        direction: ASC
"#;

#[test]
fn test_bv_22_this_file_name_and_folder() {
    // [BV-22] Projects.base を開いたときの this はその .base: this.file.name・this.file.folder
    let d = TempDir::new("this");
    d.write("P/a.md", "a\n");
    d.write("other.md", "o\n");
    let out = print(&d, "P/Projects.base", THIS_BASE);
    let a = row(&out, "a.md");
    assert!(a.contains("\"名前\": \"Projects.base\""), "{out}");
    assert!(a.contains("\"場所\": \"P\""), "{out}");
}

#[test]
fn test_bv_22_in_folder_of_this() {
    // [BV-22] file.inFolder(this.file.folder) → .base と同じフォルダ(とその下)のノートだけ
    let d = TempDir::new("infolder");
    d.write("P/a.md", "a\n");
    d.write("P/sub/b.md", "b\n");
    d.write("Q/c.md", "c\n");
    d.write("root.md", "r\n");
    let out = print(&d, "P/Projects.base", THIS_BASE);
    assert_eq!(names(&out), vec!["a.md", "b.md"], "{out}");
}

#[test]
fn test_bv_22_links_contains_this_file() {
    // [BV-22] file.links.contains(this.file) → .base を指すノートだけ(名前でもパスでも)
    let d = TempDir::new("contains-this");
    d.write("a.md", "[[Projects.base]] を見る\n");
    d.write("b.md", "[[P/Projects.base|一覧]]\n");
    d.write("c.md", "[[Projects]] は別のもの\n");
    d.write("e.md", "なし\n");
    let base = "views:\n  - type: table\n    name: v\n    filters: 'file.links.contains(this.file)'\n    order:\n      - file.name\n    sort:\n      - property: file.path\n        direction: ASC\n";
    let out = print(&d, "P/Projects.base", base);
    assert_eq!(names(&out), vec!["a.md", "b.md"], "{out}");
}

#[test]
fn test_bv_22_this_without_base_file_is_unsupported() {
    // [BV-22][BV-7] this の無いところ(`.base` のファイルを渡していない)では、this を使う絞り込みは開かず理由に this、
    // this を使う formula の列は未対応
    use mdgrid::base;
    use mdgrid::source::markdown::Markdown;
    use mdgrid::source::{RowId, Source, Value};
    let d = TempDir::new("no-this");
    d.write("a.md", "a\n");
    let mut md = Markdown::open(&[d.path().to_path_buf()]).unwrap();
    while !md.load(1000).done {}
    let src: &dyn Source = &md;
    let prop = |r: &RowId, c: &str| -> Option<Value> { src.get(r, c).value };
    let b = base::parse(
        "formulas:\n  n: 'this.file.name'\nviews:\n  - type: table\n    filters: 'file.inFolder(this.file.folder)'\n  - type: table\n    order: [file.name, formula.n]\n",
    )
    .unwrap();
    let e = b.build(0, src, &prop, 0, 0).unwrap_err();
    assert!(e.contains("this"), "{e}");
    let g = b.build(1, src, &prop, 0, 0).unwrap();
    assert!(g.notes.iter().any(|n| n.contains("this")), "{:?}", g.notes);
    let row = &g.rows[0];
    assert!(
        matches!(
            b.cell(src, &prop, row, "formula.n", 0, 0),
            base::Shown::Unsupported(_)
        ),
        "this の無い formula は未対応"
    );
}
