//! 表から新しいノートを作る(new-note)のタスク1(核)の受け入れテスト。CE-25・CE-26・CE-27
//! (関係 WB-2・WB-3・CE-2・BV-17・CLI-12)。画面(「+ 新規」・`a`・パレット・`--readonly`)は次のタスク。
//! 仕様: specs/cell-edit/spec.md。記録: specs/_changes/2026-10-02-new-note.md。
//! 決定: specs/_decisions/2026-10-02-new-note.md・2026-10-02-new-note-details.md。
//!
//! 実装を見ずに、仕様と既存の公開の形(writeback・settings・views・config)だけから書いた。
//!
//! 仮定した公開の形(最小):
//!
//! ```ignore
//! // src/newnote.rs(lib.rs に `pub mod newnote;`)
//! #[derive(Clone, Debug, Default, PartialEq)]
//! pub struct NewNote {
//!     pub folder: String,                 // 既定のフォルダ(開いたフォルダからの相対。空 = 開いたフォルダ)
//!     pub name: String,                   // 名前の雛形(`{date}` = 今日)。空が既定
//!     pub ask: Vec<String>,               // 名前のあとに聞く列の並び
//!     pub set: Vec<(String, NewValue)>,   // [new_note.set] の 列 = 値
//! }
//! /// 作る場所の検査。root/folder/name に `.md` を補ったパス(root.join(...) の形)。
//! /// `..` で外に出る・絶対パス・空・制御文字・既にある → Err(理由)。
//! pub fn note_path(root: &Path, folder: &str, name: &str) -> Result<PathBuf, String>;
//! /// 名前の雛形の `{date}` を今日(1970-01-01 からの日数)の YYYY-MM-DD にする。
//! pub fn expand_name(template: &str, today: i64) -> String;
//! /// 絞り込み(設定の条件と式の文字列)のうち値が1つに決まるものを、前もって入れる編集にする。
//! /// 列の型は kinds(無い列は Text)。
//! pub fn prefill(settings: &Settings, filters_expr: &[String], kinds: &HashMap<String, Kind>) -> Vec<Edit>;
//! /// 新しいノートの中身。絞り込みの値 → 設定の set(絞り込みに同じ列があれば絞り込みが先) → 聞いた値。
//! /// 聞いた値が空(Null・空の Str・空の List)なら書かない。本文は空。
//! pub fn build(rule: &NewNote, prefill: &[Edit], answers: &[Edit]) -> Result<Vec<u8>, String>;
//! /// 下のフォルダを作り、create_new で書く。既にあれば Err で、そのファイルは変えない。
//! pub fn create(path: &Path, bytes: &[u8]) -> std::io::Result<()>;
//! /// ビューに new_note があればそれ、無ければ設定の new_note。
//! pub fn rule_for<'a>(config: &'a Config, view: Option<&'a NativeView>) -> &'a NewNote;
//!
//! // src/config.rs: Config に `pub new_note: NewNote`(既定は NewNote::default())、ITEMS に `new_note`。
//! // src/views.rs: NativeView に `pub new_note: Option<NewNote>`(views.toml の [target.view.new_note])。
//! ```

use mdgrid::config::{parse, Config, ITEMS, KEYS};
use mdgrid::frontmatter::{self, Value};
use mdgrid::newnote::{build, create, expand_name, note_path, prefill, rule_for, NewNote};
use mdgrid::settings::{CmpOp, Cond, Op, Settings};
use mdgrid::types::{parse_date, Kind};
use mdgrid::views::{load_views, save_views, NativeView};
use mdgrid::writeback::{Edit, NewValue};
use std::collections::HashMap;
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
            "mdgrid-newnote-{}-{}-{}",
            name,
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        // macOS の /var → /private/var の違いで比べ損なわないよう、実体のパスにする。
        TempDir(std::fs::canonicalize(&dir).unwrap())
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

/// フォルダの中の全ファイル(相対パス, 中身)を並べたもの。
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                out.push((p.strip_prefix(root).unwrap().to_path_buf(), Vec::new()));
                walk(root, &p, out);
            } else {
                out.push((
                    p.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(&p).unwrap(),
                ));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

// ---- 道具 ----

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn list(v: &[&str]) -> NewValue {
    NewValue::List(v.iter().map(|x| x.to_string()).collect())
}

fn cond(col: &str, op: Op) -> Cond {
    Cond {
        col: col.to_string(),
        op,
    }
}

fn keep(v: &[&str]) -> Op {
    Op::Keep(v.iter().map(|x| Some(x.to_string())).collect())
}

fn settings(filters: Vec<Cond>) -> Settings {
    Settings {
        filters,
        ..Settings::default()
    }
}

fn exprs(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn kinds(v: &[(&str, Kind)]) -> HashMap<String, Kind> {
    v.iter().map(|(k, t)| (k.to_string(), *t)).collect()
}

fn no_kinds() -> HashMap<String, Kind> {
    HashMap::new()
}

fn today() -> i64 {
    parse_date("2026-10-02").unwrap()
}

/// 作った中身を文字列にする(UTF-8 でなければ落とす)。
fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("中身は UTF-8")
}

/// 中身のフロントマターを (キー, 値) の並びで読む。
fn entries(bytes: &[u8]) -> Vec<(String, Value)> {
    let fm = frontmatter::parse(bytes).expect("作った中身のフロントマターが読める");
    fm.entries.into_iter().map(|e| (e.key, e.value)).collect()
}

fn value_of(bytes: &[u8], key: &str) -> Option<Value> {
    entries(bytes)
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

fn sv(v: &str) -> Value {
    Value::Str(v.to_string())
}

/// 設定の文(TOML)を読み、警告が無いことを確かめて Config を返す。
fn parse_ok(toml: &str) -> Config {
    let (c, warns) = parse(toml).expect("設定の TOML が読める");
    assert!(warns.is_empty(), "警告は無い: {:?}", warns);
    c
}

fn toml_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

// ---- CE-25: 作る場所と名前の検査 ----

#[test]
fn test_ce_25_plain_name_is_md_in_open_folder() {
    // [CE-25] 「会議の準備」→ 開いたフォルダの `会議の準備.md`。
    let t = TempDir::new("plain");
    let p = note_path(t.path(), "", "会議の準備").expect("作れる名前");
    assert_eq!(p, t.path().join("会議の準備.md"));
}

#[test]
fn test_ce_25_md_suffix_is_not_doubled() {
    // [CE-25] `a.md` と打っても `a.md.md` にしない。
    let t = TempDir::new("suffix");
    let p = note_path(t.path(), "", "a.md").expect("作れる名前");
    assert_eq!(p, t.path().join("a.md"));
}

#[test]
fn test_ce_25_subfolder_name_goes_into_subfolder() {
    // [CE-25] `プロジェクト/新しい` → `プロジェクト/新しい.md`(フォルダはまだ無くてよい)。
    let t = TempDir::new("sub");
    let p = note_path(t.path(), "", "プロジェクト/新しい").expect("作れる名前");
    assert_eq!(p, t.path().join("プロジェクト").join("新しい.md"));
    assert!(
        !t.path().join("プロジェクト").exists(),
        "検査だけではフォルダを作らない"
    );
}

#[test]
fn test_ce_25_outside_absolute_empty_are_refused() {
    // [CE-25] `../外`・絶対パス・空・制御文字 → 作らず理由(Err)。
    let t = TempDir::new("outside");
    let abs = t.path().join("abs").to_string_lossy().to_string();
    for name in [
        "../外",
        "a/../../外",
        abs.as_str(),
        "/tmp/外",
        "",
        "   ",
        ".md",
        "a\nb",
        "a\tb",
    ] {
        let r = note_path(t.path(), "", name);
        match r {
            Err(reason) => assert!(!reason.trim().is_empty(), "理由が空でない: {:?}", name),
            Ok(p) => panic!("{:?} は作らない(Err)はずが {:?}", name, p),
        }
    }
    assert!(
        note_path(t.path(), "../外", "x").is_err(),
        "設定のフォルダが外でも断る"
    );
    assert!(
        note_path(t.path(), "/tmp", "x").is_err(),
        "設定のフォルダが絶対パスでも断る"
    );
    assert_eq!(snapshot(t.path()), Vec::new(), "何も作らない");
}

#[test]
fn test_ce_25_existing_name_is_refused() {
    // [CE-25] [WB-2] 既にある名前(`.md` を補っても・付けても)→ 作らず理由。ファイルは変わらない。
    let t = TempDir::new("exists");
    let before = "---\nstatus: done\n---\n本文\n".as_bytes().to_vec();
    t.write("会議の準備.md", &before);
    t.write("プロジェクト/新しい.md", b"x");
    for name in ["会議の準備", "会議の準備.md", "プロジェクト/新しい"] {
        let r = note_path(t.path(), "", name);
        assert!(
            matches!(&r, Err(reason) if !reason.trim().is_empty()),
            "{:?} は既にあるので断る: {:?}",
            name,
            r
        );
    }
    assert_eq!(
        std::fs::read(t.path().join("会議の準備.md")).unwrap(),
        before
    );
}

#[test]
fn test_ce_25_create_makes_missing_folder_and_writes_bytes() {
    // [CE-25] 下のフォルダが無ければ作り、中身を書く。
    let t = TempDir::new("create");
    let p = note_path(t.path(), "", "プロジェクト/新しい").unwrap();
    create(&p, b"---\nstatus: todo\n---\n").expect("作れる");
    assert_eq!(
        std::fs::read(t.path().join("プロジェクト").join("新しい.md")).unwrap(),
        b"---\nstatus: todo\n---\n"
    );
}

#[test]
fn test_ce_25_create_never_overwrites_existing_file() {
    // [CE-25] [WB-2] 検査のあとにできたファイルも create_new で守る: Err で、1バイトも変わらない。
    let t = TempDir::new("race");
    let p = note_path(t.path(), "", "a").unwrap();
    let before = b"---\nstatus: done\n---\n# own\n".to_vec();
    t.write("a.md", &before);
    assert!(
        create(&p, b"---\nstatus: todo\n---\n").is_err(),
        "既にあれば作らない"
    );
    assert_eq!(
        std::fs::read(&p).unwrap(),
        before,
        "既にあるファイルは変わらない"
    );
    assert_eq!(
        snapshot(t.path()).len(),
        1,
        "ほかのファイル(一時ファイルなど)も残さない"
    );
}

// ---- CE-25: 絞り込みの値を前もって入れる ----

#[test]
fn test_ce_25_status_todo_from_settings_keep() {
    // [CE-25] [WB-3] 設定の条件「status が todo」(Keep の値1つ)→ `---\nstatus: todo\n---\n`。
    let pre = prefill(
        &settings(vec![cond("status", keep(&["todo"]))]),
        &[],
        &no_kinds(),
    );
    assert_eq!(pre, vec![edit("status", s("todo"))]);
    let out = build(&NewNote::default(), &pre, &[]).expect("作れる");
    assert_eq!(text(out), "---\nstatus: todo\n---\n");
}

#[test]
fn test_ce_25_status_todo_from_settings_cmp_eq() {
    // [CE-25] 設定の条件の比較 `==`(Cmp Eq)も値が1つに決まる。
    let pre = prefill(
        &settings(vec![cond("status", Op::Cmp(CmpOp::Eq, "todo".to_string()))]),
        &[],
        &no_kinds(),
    );
    let out = build(&NewNote::default(), &pre, &[]).expect("作れる");
    assert_eq!(text(out), "---\nstatus: todo\n---\n");
}

#[test]
fn test_ce_25_status_todo_from_expr() {
    // [CE-25] 式の絞り込み `status == "todo"` → `---\nstatus: todo\n---\n`。
    let pre = prefill(
        &Settings::default(),
        &exprs(&[r#"status == "todo""#]),
        &no_kinds(),
    );
    assert_eq!(pre, vec![edit("status", s("todo"))]);
    let out = build(&NewNote::default(), &pre, &[]).expect("作れる");
    assert_eq!(text(out), "---\nstatus: todo\n---\n");
}

#[test]
fn test_ce_25_tag_condition_is_vertical_list() {
    // [CE-25] [WB-3] タグの条件 → `tags:\n  - 会議`(縦の形)。式 `tags.contains("会議")` と、リストの列の Keep の値1つ。
    let k = kinds(&[("tags", Kind::List)]);
    let from_expr = prefill(
        &Settings::default(),
        &exprs(&[r#"tags.contains("会議")"#]),
        &k,
    );
    let from_keep = prefill(&settings(vec![cond("tags", keep(&["会議"]))]), &[], &k);
    for pre in [from_expr, from_keep] {
        assert_eq!(pre, vec![edit("tags", list(&["会議"]))]);
        let out = build(&NewNote::default(), &pre, &[]).expect("作れる");
        assert_eq!(text(out), "---\ntags:\n  - 会議\n---\n");
    }
}

#[test]
fn test_ce_25_typed_cmp_eq_values_are_written_by_kind() {
    // [CE-25] [CE-2] 数の列の `== 3` は数(素)、日付の列の `== 2026-10-05` は日付(囲まない)。
    let k = kinds(&[("priority", Kind::Number), ("due", Kind::Date)]);
    let pre = prefill(
        &settings(vec![
            cond("priority", Op::Cmp(CmpOp::Eq, "3".to_string())),
            cond("due", Op::Cmp(CmpOp::Eq, "2026-10-05".to_string())),
        ]),
        &[],
        &k,
    );
    let out = text(build(&NewNote::default(), &pre, &[]).expect("作れる"));
    assert!(out.contains("\npriority: 3\n"), "数は素で書く: {:?}", out);
    assert!(
        out.contains("\ndue: 2026-10-05\n"),
        "日付は囲まない: {:?}",
        out
    );
}

#[test]
fn test_ce_25_ambiguous_conditions_are_not_prefilled() {
    // [CE-25] 値が1つに決まらない条件(Keep の値2つ・`!=`・`>`・含む・空でない)は入れない。
    let k = kinds(&[("priority", Kind::Number)]);
    let pre = prefill(
        &settings(vec![
            cond("status", keep(&["todo", "doing"])),
            cond("kind", Op::Cmp(CmpOp::Ne, "x".to_string())),
            cond("priority", Op::Cmp(CmpOp::Gt, "2".to_string())),
            cond("title", Op::Contains("会議".to_string())),
            cond("owner", Op::NotEmpty),
        ]),
        &exprs(&[r#"status != "done""#, "priority > 2"]),
        &k,
    );
    assert_eq!(pre, Vec::<Edit>::new(), "何も入れない");
}

#[test]
fn test_ce_25_unique_and_ambiguous_mixed_keeps_only_unique() {
    // [CE-25] 決まる条件と決まらない条件が混ざれば、決まるものだけ入れる。
    let k = kinds(&[("tags", Kind::List), ("priority", Kind::Number)]);
    let pre = prefill(
        &settings(vec![
            cond("status", keep(&["todo"])),
            cond("priority", Op::Cmp(CmpOp::Gt, "2".to_string())),
        ]),
        &exprs(&[r#"tags.contains("会議")"#, r#"kind != "x""#]),
        &k,
    );
    let mut keys: Vec<&str> = pre.iter().map(|e| e.key.as_str()).collect();
    keys.sort();
    assert_eq!(keys, vec!["status", "tags"]);
    let out = build(&NewNote::default(), &pre, &[]).expect("作れる");
    assert_eq!(value_of(&out, "status"), Some(sv("todo")));
    assert_eq!(value_of(&out, "tags"), Some(Value::List(vec![sv("会議")])));
}

#[test]
fn test_ce_25_whole_flow_writes_new_note_only() {
    // [CE-25] [WB-2] 「status が todo」のビューで「会議の準備」→ `会議の準備.md` が `---\nstatus: todo\n---\n` で
    // 作られ、ほかのファイルは変わらない。
    let t = TempDir::new("flow");
    t.write("既存.md", "---\nstatus: todo\n---\n本文\n".as_bytes());
    let before = snapshot(t.path());
    let view = NativeView {
        name: "未着手".to_string(),
        filters_expr: exprs(&[r#"status == "todo""#]),
        ..NativeView::default()
    };
    let config = Config::default();
    let rule = rule_for(&config, Some(&view));
    let pre = prefill(&view.settings, &view.filters_expr, &no_kinds());
    let p = note_path(t.path(), &rule.folder, "会議の準備").unwrap();
    create(&p, &build(rule, &pre, &[]).unwrap()).unwrap();
    assert_eq!(
        std::fs::read_to_string(t.path().join("会議の準備.md")).unwrap(),
        "---\nstatus: todo\n---\n"
    );
    let after = snapshot(t.path());
    let mut expected = before.clone();
    expected.push((
        PathBuf::from("会議の準備.md"),
        b"---\nstatus: todo\n---\n".to_vec(),
    ));
    expected.sort();
    assert_eq!(after, expected, "増えたのは新しいノートだけ");
}

// ---- CE-26: 設定の値・聞く項目 ----

#[test]
fn test_ce_26_set_values_are_written() {
    // [CE-26] [new_note.set] の値(tags に inbox)が縦のリストで入る。
    let rule = NewNote {
        set: vec![("tags".to_string(), list(&["inbox"]))],
        ..NewNote::default()
    };
    let out = build(&rule, &[], &[]).expect("作れる");
    assert_eq!(text(out), "---\ntags:\n  - inbox\n---\n");
}

#[test]
fn test_ce_26_filter_value_wins_over_set_value_same_column() {
    // [CE-26] [CE-27] 同じ列なら絞り込みの値が設定の値より先(作った行がビューに残る)。キーは1回だけ。
    let rule = NewNote {
        set: vec![
            ("status".to_string(), s("inbox")),
            ("kind".to_string(), s("memo")),
        ],
        ..NewNote::default()
    };
    let pre = vec![edit("status", s("todo"))];
    let out = build(&rule, &pre, &[]).expect("作れる");
    let es = entries(&out);
    assert_eq!(
        es.iter().filter(|(k, _)| k == "status").count(),
        1,
        "status は1回だけ: {:?}",
        es
    );
    assert_eq!(value_of(&out, "status"), Some(sv("todo")));
    assert_eq!(value_of(&out, "kind"), Some(sv("memo")));
}

#[test]
fn test_ce_26_asked_values_are_written_by_kind() {
    // [CE-26] [CE-2] 聞いた値は型に合った書き方: 日付は囲まない、リストは縦、数は素。
    let rule = NewNote {
        ask: vec![
            "priority".to_string(),
            "due".to_string(),
            "people".to_string(),
        ],
        set: vec![("tags".to_string(), list(&["inbox"]))],
        ..NewNote::default()
    };
    let answers = vec![
        edit("priority", NewValue::Int(3)),
        edit("due", NewValue::Date("2026-10-05".to_string())),
        edit("people", list(&["佐藤", "鈴木"])),
    ];
    let out = text(build(&rule, &[], &answers).expect("作れる"));
    assert!(
        out.starts_with("---\n") && out.ends_with("---\n"),
        "本文は空: {:?}",
        out
    );
    assert!(out.contains("\npriority: 3\n"), "数は素: {:?}", out);
    assert!(
        out.contains("\ndue: 2026-10-05\n"),
        "日付は囲まない: {:?}",
        out
    );
    assert!(
        out.contains("\npeople:\n  - 佐藤\n  - 鈴木\n"),
        "リストは縦: {:?}",
        out
    );
    assert!(
        out.contains("\ntags:\n  - inbox\n"),
        "設定の値も入る: {:?}",
        out
    );
}

#[test]
fn test_ce_26_empty_answers_are_not_written() {
    // [CE-26] 聞いた値が空なら、その項目は書かない(空の文字列・Null・空のリスト)。
    let rule = NewNote {
        ask: vec![
            "priority".to_string(),
            "note".to_string(),
            "people".to_string(),
        ],
        set: vec![("tags".to_string(), list(&["inbox"]))],
        ..NewNote::default()
    };
    let answers = vec![
        edit("priority", NewValue::Null),
        edit("note", s("")),
        edit("people", NewValue::List(Vec::new())),
    ];
    let out = build(&rule, &[], &answers).expect("作れる");
    assert_eq!(
        text(out),
        "---\ntags:\n  - inbox\n---\n",
        "空の項目の行は無い"
    );
}

#[test]
fn test_ce_26_view_rule_replaces_config_rule() {
    // [CE-26] [BV-17] ビューに new_note があれば設定の代わりにそれ、無ければ設定の new_note。
    let config = parse_ok("[new_note]\nfolder = \"inbox\"\nask = [\"priority\"]\n");
    let view_rule = NewNote {
        folder: "会議".to_string(),
        ask: vec!["due".to_string()],
        ..NewNote::default()
    };
    let with = NativeView {
        name: "会議".to_string(),
        new_note: Some(view_rule.clone()),
        ..NativeView::default()
    };
    let without = NativeView {
        name: "全部".to_string(),
        ..NativeView::default()
    };
    assert_eq!(rule_for(&config, Some(&with)), &view_rule);
    assert_eq!(rule_for(&config, Some(&without)), &config.new_note);
    assert_eq!(rule_for(&config, None), &config.new_note);
    assert_eq!(config.new_note.folder, "inbox");
}

// ---- CE-27: 設定の形・名前の雛形・views.toml ----

#[test]
fn test_ce_27_config_new_note_is_read_without_warnings() {
    // [CE-27] [CLI-3] `[new_note]` の folder・name・ask・[new_note.set] を読み、知らない項目の警告は出ない。
    let c = parse_ok(
        r#"
[new_note]
folder = "inbox"
name = "{date} "
ask = ["priority", "due"]

[new_note.set]
tags = ["inbox"]
kind = "memo"
level = 2
"#,
    );
    let n = &c.new_note;
    assert_eq!(n.folder, "inbox");
    assert_eq!(n.name, "{date} ");
    assert_eq!(
        n.ask,
        vec!["priority".to_string(), "due".to_string()],
        "聞く並びは書いた順"
    );
    let mut set = n.set.clone();
    set.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        set,
        vec![
            ("kind".to_string(), s("memo")),
            ("level".to_string(), NewValue::Int(2)),
            ("tags".to_string(), list(&["inbox"])),
        ]
    );
}

#[test]
fn test_ce_27_default_config_has_empty_rule() {
    // [CE-27] 設定に new_note が無ければ既定(開いたフォルダ・雛形なし・聞かない・入れない)。
    let c = parse_ok("");
    assert_eq!(c.new_note, NewNote::default());
    assert_eq!(Config::default().new_note, NewNote::default());
    assert!(c.new_note.folder.is_empty() && c.new_note.name.is_empty());
    assert!(c.new_note.ask.is_empty() && c.new_note.set.is_empty());
}

#[test]
fn test_ce_27_wrong_types_warn_and_do_not_stop() {
    // [CE-27] [CLI-3] 型の違う値は警告にして、止めない。
    for bad in [
        "new_note = 3\n",
        "[new_note]\nask = \"priority\"\n",
        "[new_note]\nfolder = 1\n",
        "[new_note]\nname = [\"x\"]\n",
        "[new_note]\nset = 1\n",
    ] {
        let (_, warns) = parse(bad).unwrap_or_else(|e| panic!("{:?} で止まらない: {}", bad, e));
        assert!(
            warns.iter().any(|w| w.contains("new_note")
                || w.contains("ask")
                || w.contains("folder")
                || w.contains("name")
                || w.contains("set")),
            "{:?} は警告になる: {:?}",
            bad,
            warns
        );
        assert!(
            !warns.iter().any(|w| w.contains("知らない項目 `new_note`")),
            "new_note は知る項目: {:?}",
            warns
        );
    }
}

#[test]
fn test_ce_27_config_folder_is_default_place() {
    // [CE-25] [CE-27] 設定の folder が既定の場所。名前の下のフォルダはその下。
    let t = TempDir::new("folder");
    let c = parse_ok("[new_note]\nfolder = \"inbox\"\n");
    let rule = rule_for(&c, None);
    assert_eq!(
        note_path(t.path(), &rule.folder, "x").unwrap(),
        t.path().join("inbox").join("x.md")
    );
    assert_eq!(
        note_path(t.path(), &rule.folder, "下/y").unwrap(),
        t.path().join("inbox").join("下").join("y.md")
    );
    let p = note_path(t.path(), &rule.folder, "x").unwrap();
    create(&p, b"---\nstatus: todo\n---\n").unwrap();
    assert!(
        t.path().join("inbox").join("x.md").is_file(),
        "フォルダを作って書く"
    );
}

#[test]
fn test_ce_27_name_template_date_is_today() {
    // [CE-27] `name = "{date} "` → 今日の日付 `2026-10-02 `。
    assert_eq!(expand_name("{date} ", today()), "2026-10-02 ");
    assert_eq!(expand_name("議事録 {date}", today()), "議事録 2026-10-02");
    assert_eq!(expand_name("", today()), "");
    assert_eq!(expand_name("メモ", today()), "メモ");
}

#[test]
fn test_ce_27_views_toml_new_note_is_read() {
    // [CE-27] [BV-17] views.toml の `[target.view.new_note]` を同じ形で読み、警告は出ない。
    let t = TempDir::new("views");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let text = format!(
        r#"[[target]]
path = {p}

  [[target.view]]
  name = "会議"
  order = []
  hidden = []
  filters_expr = ['tags.contains("会議")']
  [target.view.new_note]
  folder = "会議"
  name = "{{date}} "
  ask = ["due"]
  [target.view.new_note.set]
  kind = "meeting"

  [[target.view]]
  name = "全部"
  order = []
  hidden = []
  filters_expr = []
"#,
        p = toml_str(notes.to_str().unwrap()),
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    let (views, warns) = load_views(&conf, &notes);
    assert!(warns.is_empty(), "警告は無い: {:?}", warns);
    assert_eq!(views.len(), 2);
    assert_eq!(
        views[0].new_note,
        Some(NewNote {
            folder: "会議".to_string(),
            name: "{date} ".to_string(),
            ask: vec!["due".to_string()],
            set: vec![("kind".to_string(), s("meeting"))],
        })
    );
    assert_eq!(views[1].new_note, None);
}

#[test]
fn test_ce_27_views_new_note_survives_save_and_load() {
    // [CE-27] [BV-17] ビューを保存して読み直しても new_note は消えない。
    let t = TempDir::new("views-roundtrip");
    let conf = t.mkdir("config");
    let notes = t.mkdir("notes");
    let views = vec![
        NativeView {
            name: "会議".to_string(),
            new_note: Some(NewNote {
                folder: "会議".to_string(),
                name: "{date} ".to_string(),
                ask: vec!["due".to_string(), "priority".to_string()],
                set: vec![("tags".to_string(), list(&["会議"]))],
            }),
            ..NativeView::default()
        },
        NativeView {
            name: "全部".to_string(),
            ..NativeView::default()
        },
    ];
    save_views(&conf, &notes, &views).expect("save_views");
    let (back, warns) = load_views(&conf, &notes);
    assert!(warns.is_empty(), "警告は無い: {:?}", warns);
    assert_eq!(back, views);
}

#[test]
fn test_ce_27_new_note_is_a_documented_config_item() {
    // [CE-27] [CLI-12] 設定の項目の表(文書・--print-config の元)に new_note がある。
    assert!(
        ITEMS.iter().any(|i| i.name == "new_note"),
        "ITEMS に new_note がある"
    );
    assert!(KEYS.contains(&"new_note"), "KEYS に new_note がある");
}
