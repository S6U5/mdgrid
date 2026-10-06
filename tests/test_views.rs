//! mdgrid 独自のビュー(native-views)の核の受け入れテスト(タスク 1)。
//! BV-17・BV-19・BV-20(と BV-3・BV-13・CLI-3)。
//! 仕様: specs/base-view/spec.md、specs/cli/spec.md。
//! 形: docs/design.md の「mdgrid のビュー(BV-17〜BV-20)」の `src/views.rs`
//! (NativeView・load_views・save_views・to_base・from_base、views.toml の形)。
//! 実装を見ずに、公開のインターフェースだけを使う。
//!
//! 材料の保管庫(`.obsidian/` を持つ一時フォルダ)のノート:
//!
//! | note | tags  | status | priority | due        | title          |
//! |------|-------|--------|----------|------------|----------------|
//! | a.md | a     | done   | 2        | 2026-03-01 | Alpha report   |
//! | b.md | a, b  | todo   | 1        | 2026-01-15 | beta report    |
//! | c.md | b     | todo   | 3        | 2026-02-01 | gamma report   |
//! | d.md | a     | doing  | 5        | 2026-02-20 | delta REPORT   |
//! | e.md | a     | todo   | 3        | 2026-02-10 | epsilon report |
//! | f.md | a     | doing  | 4        | 2026-03-10 | zeta report    |
//! | g.md | a     | todo   | 2        | 2026-01-20 | eta            |
//! | h.md | a     | doing  | 6        | 2026-01-05 | theta report   |

use mdgrid::base::{self, Base, Grid};
use mdgrid::settings::{self, CmpOp, Cond, Dir, Group, Op, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source, Value};
use mdgrid::types::Kind;
use mdgrid::views::{from_base, load_views, save_views, to_base, NativeView};
use std::path::{Path, PathBuf};

// ---- 一時フォルダ ----

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-views-{}-{}-{}",
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

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn write(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, bytes).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// フォルダの下の全ファイル(と中身)。
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    let bytes = std::fs::read(&p).unwrap_or_default();
                    out.push((p, bytes));
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn some(v: &[&str]) -> Vec<Option<String>> {
    v.iter().map(|s| Some(s.to_string())).collect()
}

fn cond(col: &str, op: Op) -> Cond {
    Cond {
        col: col.to_string(),
        op,
    }
}

// ---- 材料 ----

const TODAY: i64 = 20454; // 2026-01-01
const NOW: i64 = TODAY * 86_400;

fn note(title: &str, tags: &str, status: &str, priority: i64, due: &str) -> String {
    format!(
        "---\ntags: [{tags}]\nstatus: {status}\npriority: {priority}\ndue: {due}\ntitle: {title}\n---\nbody\n"
    )
}

fn vault(name: &str) -> TempDir {
    let dir = TempDir::new(name);
    std::fs::create_dir_all(dir.path().join(".obsidian")).unwrap();
    dir.write(
        ".obsidian/types.json",
        br#"{"types": {"due": "date", "priority": "number", "status": "text", "title": "text"}}"#,
    );
    let notes = [
        ("a.md", note("Alpha report", "a", "done", 2, "2026-03-01")),
        ("b.md", note("beta report", "a, b", "todo", 1, "2026-01-15")),
        ("c.md", note("gamma report", "b", "todo", 3, "2026-02-01")),
        ("d.md", note("delta REPORT", "a", "doing", 5, "2026-02-20")),
        ("e.md", note("epsilon report", "a", "todo", 3, "2026-02-10")),
        ("f.md", note("zeta report", "a", "doing", 4, "2026-03-10")),
        ("g.md", note("eta", "a", "todo", 2, "2026-01-20")),
        ("h.md", note("theta report", "a", "doing", 6, "2026-01-05")),
    ];
    for (n, text) in notes {
        dir.write(n, text.as_bytes());
    }
    dir
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

/// `.base` の文字列を保管庫に書いて開く。(Markdown, Base)
fn open_base_text(dir: &TempDir, file: &str, text: &str) -> (Markdown, Base) {
    let p = dir.write(file, text.as_bytes());
    let b = base::parse(text).unwrap_or_else(|e| panic!("base::parse failed: {e}\n{text}"));
    let mut md = Markdown::open_vault(&p).expect("Markdown::open_vault");
    load_all(&mut md);
    (md, b)
}

fn build(b: &Base, view: usize, src: &dyn Source) -> Grid {
    let prop = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    match b.build(view, src, &prop, TODAY, NOW) {
        Ok(g) => g,
        Err(e) => panic!("build({view}) failed: {e}"),
    }
}

fn labels(src: &dyn Source, rows: &[RowId]) -> Vec<String> {
    rows.iter().map(|r| src.label(r)).collect()
}

fn kind_of(col: &str) -> Kind {
    match col {
        "tags" => Kind::List,
        "priority" => Kind::Number,
        "due" => Kind::Date,
        _ => Kind::Text,
    }
}

fn has_tag_a(src: &dyn Source, row: &RowId) -> bool {
    match src.get(row, "tags").value {
        Some(Value::List(items)) => items.iter().any(|v| matches!(v, Value::Str(s) if s == "a")),
        Some(Value::Str(s)) => s == "a",
        _ => false,
    }
}

/// mdgrid のビューの行(filters_expr が `file.hasTag("a")` だけのとき)。
/// filters_expr を手で当ててから settings::apply を当てる。
fn native_rows_tag_a(src: &dyn Source, s: &Settings) -> (Vec<String>, Vec<std::ops::Range<usize>>) {
    let rows: Vec<RowId> = src
        .rows()
        .into_iter()
        .filter(|r| has_tag_a(src, r))
        .collect();
    let get = |row: &RowId, col: &str| -> Option<Value> { src.get(row, col).value };
    let (rows, groups) = settings::apply(rows, Vec::new(), s, &get, &kind_of);
    (
        labels(src, &rows),
        groups.into_iter().map(|(_, r)| r).collect(),
    )
}

/// 絞り込み(Drop・Keep・Contains・Cmp)・並べ替え・グループ・filters_expr・order・hidden を持つビュー。
/// `.base` で全部表せる。
fn full_view() -> NativeView {
    NativeView {
        name: "進行中の報告".to_string(),
        order: names(&["file.name", "status", "priority", "due"]),
        hidden: names(&["due"]),
        filters_expr: names(&[r#"file.hasTag("a")"#]),
        settings: Settings {
            filters: vec![
                cond("status", Op::Drop(some(&["done"]))),
                cond("status", Op::Keep(some(&["todo", "doing"]))),
                cond("title", Op::Contains("report".to_string())),
                cond("priority", Op::Cmp(CmpOp::Ge, "3".to_string())),
                cond("due", Op::Cmp(CmpOp::Lt, "2026-03-15".to_string())),
            ],
            sorts: vec![("priority".to_string(), Dir::Desc)],
            group: Group::By {
                col: "status".to_string(),
                dir: Dir::Asc,
                hide_empty: false,
            },
            ..Default::default()
        },
        new_note: None,
    }
}

fn simple_view(name: &str) -> NativeView {
    NativeView {
        name: name.to_string(),
        order: names(&["title", "status"]),
        hidden: Vec::new(),
        filters_expr: names(&[r#"status != "done""#]),
        settings: Settings::default(),
        new_note: None,
    }
}

// ---- BV-17・BV-20: 保存と読み込み ----

#[test]
fn test_bv_17_save_then_load_returns_same_views() {
    // [BV-17] [BV-20] save_views → load_views で、名前・order・hidden・filters_expr・settings が同じビューが戻る。
    let t = TempDir::new("roundtrip");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let mut second = simple_view("未完了");
    second.settings = Settings {
        filters: vec![
            cond("status", Op::Keep(vec![None, Some("todo".to_string())])),
            cond("title", Op::NotContains("memo".to_string())),
            cond("tags", Op::Empty),
            cond("due", Op::NotEmpty),
        ],
        sorts: vec![
            ("due".to_string(), Dir::Asc),
            ("priority".to_string(), Dir::Desc),
        ],
        group: Group::Off,
        ..Default::default()
    };
    let views = vec![full_view(), second, simple_view("既定ふう")];
    save_views(&conf, &notes, &views).expect("save_views");
    let (back, warns) = load_views(&conf, &notes);
    assert!(warns.is_empty(), "警告は無い: {:?}", warns);
    assert_eq!(back, views, "保存したビューが順も含めて戻る");
}

#[test]
fn test_bv_20_views_are_per_target_and_other_targets_are_kept() {
    // [BV-17] [BV-20] 対象(フォルダ)ごとに別のビュー。対象 A を書き直しても対象 B のビューは残る。
    let t = TempDir::new("pertarget");
    let conf = t.mkdir("config");
    let a = t.mkdir("notes-a");
    let b = t.mkdir("notes-b");
    save_views(&conf, &a, &[simple_view("A1"), simple_view("A2")]).expect("save A");
    save_views(&conf, &b, &[full_view()]).expect("save B");

    let (va, _) = load_views(&conf, &a);
    let (vb, _) = load_views(&conf, &b);
    assert_eq!(
        va.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        names(&["A1", "A2"])
    );
    assert_eq!(vb, vec![full_view()]);

    // A を書き直す(1つに減らす)。
    save_views(&conf, &a, &[simple_view("A3")]).expect("save A again");
    let (va, warns) = load_views(&conf, &a);
    assert!(warns.is_empty(), "{:?}", warns);
    assert_eq!(va, vec![simple_view("A3")], "A は全部書き直される");
    let (vb, _) = load_views(&conf, &b);
    assert_eq!(vb, vec![full_view()], "B のビューは残る");

    // ビューの無い対象は空。
    let c = t.mkdir("notes-c");
    let (vc, warns) = load_views(&conf, &c);
    assert!(vc.is_empty());
    assert!(warns.is_empty(), "{:?}", warns);
}

#[test]
fn test_bv_17_does_not_write_into_note_folder() {
    // [BV-17] [BV-20] 定義は設定の置き場の views.toml にだけ書き、ノートのフォルダには書かない。
    let t = TempDir::new("nowrite");
    let conf = t.mkdir("config");
    let v = vault("nowrite-vault");
    let before = snapshot(v.path());
    save_views(&conf, v.path(), &[full_view(), simple_view("x")]).expect("save");
    save_views(&conf, v.path(), &[simple_view("y")]).expect("save again");
    let _ = load_views(&conf, v.path());
    assert_eq!(snapshot(v.path()), before, "ノートのフォルダは変わらない");

    // 置き場には views.toml があり、ほかに残るもの(一時ファイル)は無い。
    let files: Vec<PathBuf> = snapshot(&conf).into_iter().map(|(p, _)| p).collect();
    assert_eq!(files, vec![conf.join("views.toml")], "{:?}", files);
    // 置き場の外(t の下の config 以外)に新しいファイルが無い。
    let outside: Vec<PathBuf> = snapshot(t.path())
        .into_iter()
        .map(|(p, _)| p)
        .filter(|p| !p.starts_with(&conf))
        .collect();
    assert!(outside.is_empty(), "置き場の外にファイル: {:?}", outside);
}

#[cfg(unix)]
#[test]
fn test_bv_20_target_is_keyed_by_real_path() {
    // [BV-20] 対象は実体のパスで引く。シンボリックリンクで渡しても同じビュー。
    let t = TempDir::new("symlink");
    let conf = t.mkdir("config");
    let real = t.mkdir("real-notes");
    let link = t.path().join("link-notes");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    save_views(&conf, &link, &[simple_view("リンクから")]).expect("save via link");
    let (v, _) = load_views(&conf, &real);
    assert_eq!(v, vec![simple_view("リンクから")]);

    save_views(&conf, &real, &[simple_view("実体から")]).expect("save via real");
    let (v, _) = load_views(&conf, &link);
    assert_eq!(v, vec![simple_view("実体から")]);

    // 書いたパスは実体のパス。
    let text = std::fs::read_to_string(conf.join("views.toml")).unwrap();
    let canon = std::fs::canonicalize(&real).unwrap();
    assert!(
        text.contains(canon.to_str().unwrap()),
        "実体のパスで書く: {}",
        text
    );
    assert!(
        !text.contains(link.to_str().unwrap()),
        "リンクのパスで書かない: {}",
        text
    );
}

// ---- BV-20: 手で書いた views.toml・知らない項目・壊れたファイル ----

fn toml_str(s: &str) -> String {
    // TOML の基本の文字列(パスの `\` と `"` を逃がす)。
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[test]
fn test_bv_20_hand_written_views_toml_is_read() {
    // [BV-20] [BV-17] 設計の形で手で書いた views.toml が読める。settings を省いた・空のビューは既定の設定。
    let t = TempDir::new("handwritten");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let other = t.mkdir("other");
    let real = std::fs::canonicalize(&notes).unwrap();
    let real_other = std::fs::canonicalize(&other).unwrap();
    let text = format!(
        r#"[[target]]
path = {p}

  [[target.view]]
  name = "未完了"
  order = ["title", "status", "due"]
  hidden = ["due"]
  filters_expr = ['status != "done"', 'file.hasTag("a")']
  [target.view.settings]

  [[target.view]]
  name = "全部"
  order = []
  hidden = []
  filters_expr = []

[[target]]
path = {o}

  [[target.view]]
  name = "別の対象"
  order = []
  hidden = []
  filters_expr = []
"#,
        p = toml_str(real.to_str().unwrap()),
        o = toml_str(real_other.to_str().unwrap()),
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();

    let (views, warns) = load_views(&conf, &notes);
    assert!(warns.is_empty(), "警告は無い: {:?}", warns);
    assert_eq!(views.len(), 2, "{:?}", views);
    assert_eq!(views[0].name, "未完了");
    assert_eq!(views[0].order, names(&["title", "status", "due"]));
    assert_eq!(views[0].hidden, names(&["due"]));
    assert_eq!(
        views[0].filters_expr,
        names(&[r#"status != "done""#, r#"file.hasTag("a")"#])
    );
    assert!(views[0].settings.is_default());
    assert_eq!(views[1].name, "全部");
    assert!(views[1].order.is_empty());
    assert!(views[1].settings.is_default());

    let (o, _) = load_views(&conf, &other);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].name, "別の対象");
}

#[test]
fn test_bv_20_unknown_key_warns_but_views_load() {
    // [BV-20] [CLI-3] 知らない項目 `color = 1` → 警告の文にその名前が出て、ビューは読める。
    let t = TempDir::new("unknown");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let real = std::fs::canonicalize(&notes).unwrap();
    let text = format!(
        r#"[[target]]
path = {p}

  [[target.view]]
  name = "色つき"
  order = ["title"]
  hidden = []
  filters_expr = []
  color = 1
"#,
        p = toml_str(real.to_str().unwrap()),
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    let (views, warns) = load_views(&conf, &notes);
    assert_eq!(views.len(), 1, "ビューは読める: {:?}", views);
    assert_eq!(views[0].name, "色つき");
    assert_eq!(views[0].order, names(&["title"]));
    assert!(
        warns.iter().any(|w| w.contains("color")),
        "知らない項目の名前が警告に出る: {:?}",
        warns
    );
}

#[test]
fn test_bv_20_broken_toml_warns_and_returns_no_views() {
    // [BV-20] 壊れた views.toml → 警告の文が出て、ビューは空で、パニックしない。
    let t = TempDir::new("broken");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let broken = "[[target]\npath = \"/x\n  [[target.view]]\n  name = \n";
    std::fs::write(conf.join("views.toml"), broken).unwrap();
    let (views, warns) = load_views(&conf, &notes);
    assert!(views.is_empty(), "{:?}", views);
    assert!(!warns.is_empty(), "壊れたファイルは警告");
    assert!(warns.iter().all(|w| !w.trim().is_empty()), "{:?}", warns);
    // 読むだけでは壊れたファイルを書き換えない。
    assert_eq!(
        std::fs::read_to_string(conf.join("views.toml")).unwrap(),
        broken
    );
}

#[test]
fn test_bv_20_missing_file_is_empty_without_warning() {
    // [BV-20] [CLI-3] views.toml が無い(置き場のフォルダも無い)→ 空で警告なし。
    let t = TempDir::new("missing");
    let notes = t.mkdir("notes");
    let conf = t.path().join("no-such-config");
    let (views, warns) = load_views(&conf, &notes);
    assert!(views.is_empty());
    assert!(warns.is_empty(), "{:?}", warns);
    assert!(!conf.exists(), "読むだけでは置き場を作らない");
}

// ---- BV-19: .base への書き出し ----

#[test]
fn test_bv_19_to_base_builds_same_rows_as_native_view() {
    // [BV-19] Drop・Keep・Contains・Cmp・並べ替え・グループ・filters_expr・order・hidden を持つビューを書き出す
    // → base::parse で読める YAML になり、build すると mdgrid のビューと同じ行・同じまとまりが出る。
    let v = vault("export");
    let view = full_view();
    let (yaml, dropped) = to_base(&view);
    assert!(
        dropped.is_empty(),
        "全部表せるので落とした部分は無い: {:?}",
        dropped
    );

    let (md, b) = open_base_text(&v, "exported.base", &yaml);
    let src: &dyn Source = &md;
    assert_eq!(b.views.len(), 1, "{yaml}");
    assert_eq!(b.views[0].kind, "table");
    assert_eq!(b.views[0].name, view.name);

    let g = build(&b, 0, src);
    // filters_expr(tag a)→ a, b, d, e, f, g, h。done を隠す・todo か doing → a が消える。
    // title に report(大小を区別しない。d の REPORT も残る)→ g が消える。
    // priority ≥ 3 → b が消える。due < 2026-03-15 → d, e, f, h は全部残る。
    // priority ↓ で並べ、status ↑ でまとめる: doing(h 6, d 5, f 4)、todo(e 3)。
    assert_eq!(
        labels(src, &g.rows),
        names(&["h.md", "d.md", "f.md", "e.md"]),
        "{yaml}"
    );
    assert_eq!(g.groups.len(), 2, "{:?}", g.groups);
    assert!(g.groups[0].0.contains("doing"), "{:?}", g.groups);
    assert_eq!(g.groups[0].1, 0..3);
    assert!(g.groups[1].0.contains("todo"), "{:?}", g.groups);
    assert_eq!(g.groups[1].1, 3..4);

    // mdgrid のビュー(settings::apply)と同じ行・同じ範囲。
    let (native, native_groups) = native_rows_tag_a(src, &view.settings);
    assert_eq!(labels(src, &g.rows), native);
    assert_eq!(
        g.groups.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>(),
        native_groups
    );

    // 列は order から hidden を除いたもの。
    let cols: Vec<String> = g.columns.iter().map(|c| c.id.clone()).collect();
    assert_eq!(cols, names(&["file.name", "status", "priority"]), "{yaml}");
}

#[test]
fn test_bv_19_to_base_simple_filters_expr_and_desc_group() {
    // [BV-19] filters_expr だけのビューと、まとまりの降順も書き出せて同じ行が出る。
    let v = vault("export-simple");
    let mut view = simple_view("未完了");
    view.order = names(&["file.name", "status"]);
    view.settings.group = Group::By {
        col: "status".to_string(),
        dir: Dir::Desc,
        hide_empty: false,
    };
    view.settings.sorts = vec![("file.name".to_string(), Dir::Asc)];
    let (yaml, dropped) = to_base(&view);
    assert!(dropped.is_empty(), "{:?}", dropped);
    let (md, b) = open_base_text(&v, "simple.base", &yaml);
    let src: &dyn Source = &md;
    let g = build(&b, 0, src);
    // done 以外: b, c, e, g(todo)、d, f, h(doing)。降順 → todo が先。
    assert_eq!(
        labels(src, &g.rows),
        names(&["b.md", "c.md", "e.md", "g.md", "d.md", "f.md", "h.md"]),
        "{yaml}"
    );
    assert_eq!(g.groups.len(), 2, "{:?}", g.groups);
    assert!(g.groups[0].0.contains("todo"), "{:?}", g.groups);
    assert_eq!(g.groups[0].1, 0..4);
    assert!(g.groups[1].0.contains("doing"), "{:?}", g.groups);
}

#[test]
fn test_bv_19_to_base_reports_dropped_parts() {
    // [BV-19] `.base` で表せない部分(グループの「空を隠す」。NV-21)を持つビュー
    // → 書き出しはでき(parse できる)、落とした説明が空でない。
    let mut view = full_view();
    view.settings.group = Group::By {
        col: "status".to_string(),
        dir: Dir::Asc,
        hide_empty: true,
    };
    let (yaml, dropped) = to_base(&view);
    assert!(!dropped.is_empty(), "落とした説明が出る: {yaml}");
    assert!(
        dropped.iter().all(|d| !d.trim().is_empty()),
        "{:?}",
        dropped
    );
    base::parse(&yaml).unwrap_or_else(|e| panic!("落としても parse できる: {e}\n{yaml}"));
}

#[test]
fn test_bv_3_to_base_and_from_base_write_no_files() {
    // [BV-3] [BV-19] 書き出しは文字列を返すだけで、ファイルを書かない。取り込みも書かない。
    let v = vault("nofile");
    let existing = v.write(
        "existing.base",
        b"views:\n  - type: table\n    name: Old\n    order:\n      - file.name\n",
    );
    let conf = TempDir::new("nofile-conf");
    save_views(conf.path(), v.path(), &[full_view()]).expect("save");
    let before_vault = snapshot(v.path());
    let before_conf = snapshot(conf.path());

    let _ = to_base(&full_view());
    let _ = to_base(&simple_view("x"));
    let b = base::parse(&std::fs::read_to_string(&existing).unwrap()).unwrap();
    let _ = from_base(&b, 0);

    assert_eq!(snapshot(v.path()), before_vault, "保管庫は変わらない");
    assert_eq!(snapshot(conf.path()), before_conf, "置き場も変わらない");
}

// ---- BV-19: .base からの取り込み ----

const IMPORT_BASE: &str = r#"filters:
  and:
    - file.hasTag("a")
views:
  - type: table
    name: 取り込む
    filters:
      or:
        - 'status == "doing"'
        - 'priority >= 3'
    order:
      - file.name
      - status
      - priority
    sort:
      - property: priority
        direction: DESC
      - property: file.name
        direction: ASC
    groupBy:
      property: status
      direction: ASC
    limit: 3
  - type: table
    name: 二つ目
    order:
      - file.name
"#;

#[test]
fn test_bv_19_from_base_maps_fields() {
    // [BV-19] 全体の filters とビューの filters は filters_expr に、order・sort・groupBy は対応する欄に入る。
    // NativeView に欄の無い limit は落とした説明に出る。
    let b = base::parse(IMPORT_BASE).expect("parse");
    let (nv, dropped) = from_base(&b, 0);
    assert_eq!(nv.name, "取り込む");
    assert_eq!(nv.order, names(&["file.name", "status", "priority"]));
    assert!(nv.hidden.is_empty(), "{:?}", nv.hidden);
    assert_eq!(
        nv.settings.sorts,
        vec![
            ("priority".to_string(), Dir::Desc),
            ("file.name".to_string(), Dir::Asc)
        ]
    );
    assert_eq!(
        nv.settings.group,
        Group::By {
            col: "status".to_string(),
            dir: Dir::Asc,
            hide_empty: false
        }
    );
    assert!(nv.settings.filters.is_empty(), "式は filters_expr に入る");
    assert!(!nv.filters_expr.is_empty());
    let all = nv.filters_expr.join("\n");
    assert!(all.contains(r#"file.hasTag("a")"#), "全体の filters: {all}");
    assert!(
        all.contains(r#"status == "doing""#),
        "ビューの filters: {all}"
    );
    assert!(all.contains("priority >= 3"), "ビューの filters: {all}");

    assert!(
        dropped.iter().any(|d| d.contains("limit")),
        "limit を落としたと知らせる: {:?}",
        dropped
    );

    // filters の無いビュー: 全体の filters だけが入る。limit も無いので落とした部分は無い。
    let (nv2, dropped2) = from_base(&b, 1);
    assert_eq!(nv2.name, "二つ目");
    assert_eq!(nv2.order, names(&["file.name"]));
    assert!(nv2.filters_expr.join("\n").contains(r#"file.hasTag("a")"#));
    assert!(dropped2.is_empty(), "{:?}", dropped2);
}

#[test]
fn test_bv_19_from_base_then_to_base_keeps_rows() {
    // [BV-19] 取り込んだビューを書き出す → 元のビュー(limit を除く)と同じ行・まとまり。
    // filters の or も filters_expr に落とさずに入っている。
    let v = vault("import");
    let b = base::parse(IMPORT_BASE).expect("parse");
    let (nv, _) = from_base(&b, 0);
    let (yaml, dropped) = to_base(&nv);
    assert!(dropped.is_empty(), "{:?}", dropped);
    let (md, round) = open_base_text(&v, "round.base", &yaml);
    let src: &dyn Source = &md;
    let g = build(&round, 0, src);
    // tag a かつ(doing か priority ≥ 3): d, e, f, h。groupBy status ↑、中は priority ↓ → file.name ↑。
    // doing: h(6), d(5), f(4)。todo: e(3)。
    assert_eq!(
        labels(src, &g.rows),
        names(&["h.md", "d.md", "f.md", "e.md"]),
        "{yaml}"
    );
    assert_eq!(g.groups.len(), 2, "{:?}", g.groups);
    assert_eq!(g.groups[0].1, 0..3);
    assert_eq!(g.groups[1].1, 3..4);
    let cols: Vec<String> = g.columns.iter().map(|c| c.id.clone()).collect();
    assert_eq!(cols, names(&["file.name", "status", "priority"]));
}
