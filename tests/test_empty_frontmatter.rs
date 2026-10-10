//! 空のフロントマターにも書き、フロントマターを足すかを設定で選べる(empty-frontmatter-config)の受け入れテスト。
//! WB-3・WB-5・WB-1・WB-6・CLI-3。
//! 仕様: specs/write-back/spec.md の WB-3・WB-5、specs/cli/spec.md の CLI-3。
//! 記録: specs/_changes/2026-10-02-empty-frontmatter-config.md。
//! 決定: specs/_decisions/2026-10-02-empty-frontmatter-config.md。
//!
//! 実装を見ずに、仕様と公開のインターフェースだけから書いた。
//!
//! 仮定した公開の形(設計「`add_frontmatter` は Config に持ち、Source(Markdown)を開くときに渡す」から):
//! - `mdgrid::config::Config` に `pub add_frontmatter: bool`(既定 true)。設定の TOML の最上位の `add_frontmatter`。
//! - `mdgrid::source::markdown::Markdown` に `pub fn set_add_frontmatter(&mut self, on: bool)`。開いた直後、
//!   読み込み(load)の前に呼ぶ。この仮定は下の `open_with` の1か所だけに置く。

use mdgrid::changes::Changes;
use mdgrid::config::{parse as parse_config, Config};
use mdgrid::frontmatter::{parse, ReadOnly, Value};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::{RowId, Source};
use mdgrid::writeback::{apply, baseline, save, save_with, Edit, Faults, NewValue};
use std::path::{Path, PathBuf};

// ---- 一時フォルダ(tests/test_add_frontmatter.rs と同じ) ----

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

// ---- 道具 ----

/// 空のフロントマターのノート。
const EMPTY_FM: &str = "---\n---\n本文\n";
const EMPTY_FM_CRLF: &str = "---\r\n---\r\n本文\r\n";

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

fn s(v: &str) -> NewValue {
    NewValue::Str(v.to_string())
}

fn todo() -> Vec<Edit> {
    vec![edit("status", s("todo"))]
}

/// status に todo、priority に 2(この順)。
fn two() -> Vec<Edit> {
    vec![
        edit("status", s("todo")),
        edit("priority", NewValue::Int(2)),
    ]
}

fn applied(original: &[u8], edits: &[Edit]) -> String {
    let out = apply(original, edits).unwrap_or_else(|e| {
        panic!(
            "apply failed on {:?}: {e:?}",
            String::from_utf8_lossy(original)
        )
    });
    String::from_utf8(out).expect("result is UTF-8")
}

/// 設定の add_frontmatter を渡して開く(仮定した公開の形はここだけ)。
fn open_with(folder: &Path, add_frontmatter: bool) -> Markdown {
    let mut md = Markdown::open(&[folder.to_path_buf()]).expect("Markdown::open");
    md.set_add_frontmatter(add_frontmatter);
    md
}

/// 既定(何も渡さない)で開く。
fn open_default(folder: &Path) -> Markdown {
    Markdown::open(&[folder.to_path_buf()]).expect("Markdown::open")
}

fn load_all(src: &mut dyn Source) {
    for _ in 0..10_000 {
        if src.load(100).done {
            return;
        }
    }
    panic!("load did not finish");
}

fn row_of(src: &dyn Source, label: &str) -> RowId {
    src.rows()
        .into_iter()
        .find(|r| src.label(r) == label)
        .unwrap_or_else(|| {
            let labels: Vec<String> = src.rows().iter().map(|r| src.label(r)).collect();
            panic!("row {label} not found in {labels:?}")
        })
}

fn lock_of(src: &dyn Source, label: &str, col: &str) -> Option<String> {
    src.get(&row_of(src, label), col).lock
}

// ---- apply: 空のフロントマターに足す ----

#[test]
fn test_wb_3_empty_frontmatter_gets_key_between_delimiters() {
    // [WB-3] `---\n---\n本文\n` の status に todo → `---\nstatus: todo\n---\n本文\n`(バイトで完全一致)。
    // [WB-1] 足すのは区切りの間の1行だけ。
    assert_eq!(parse(EMPTY_FM.as_bytes()), Err(ReadOnly::EmptyFrontmatter));
    let out = apply(EMPTY_FM.as_bytes(), &todo()).expect("[WB-3] empty frontmatter is writable");
    assert_eq!(out, "---\nstatus: todo\n---\n本文\n".as_bytes());
}

#[test]
fn test_wb_3_empty_frontmatter_crlf() {
    // [WB-3] [WB-1] CRLF の空のフロントマター → 足す行も CRLF、ほかのバイトはそのまま。
    let out = applied(EMPTY_FM_CRLF.as_bytes(), &todo());
    assert_eq!(out, "---\r\nstatus: todo\r\n---\r\n本文\r\n");
    assert!(
        !out.replace("\r\n", "").contains('\n'),
        "[WB-1] no bare LF: {out:?}"
    );
}

#[test]
fn test_wb_3_empty_frontmatter_without_body() {
    // [WB-3] 本文の無い `---\n---\n` → `---\nstatus: todo\n---\n`。
    assert_eq!(applied(b"---\n---\n", &todo()), "---\nstatus: todo\n---\n");
    // 閉じの区切りの後ろに改行が無い `---\n---` → 足す行の改行は開きの行に合わせ、閉じの行はそのまま。
    assert_eq!(applied(b"---\n---", &todo()), "---\nstatus: todo\n---");
}

#[test]
fn test_wb_1_empty_frontmatter_keeps_other_bytes() {
    // [WB-1] [WB-3] 区切りの行の末尾の空白や本文の `---` もそのまま。足した1行を除けば元のバイトと同じ。
    let before = "---  \n---\t\n# 見出し\n---\n本文\n";
    let out = applied(before.as_bytes(), &todo());
    assert_eq!(out, "---  \nstatus: todo\n---\t\n# 見出し\n---\n本文\n");
    assert_eq!(out.replacen("status: todo\n", "", 1), before);
}

#[test]
fn test_wb_3_empty_frontmatter_two_keys_in_written_order() {
    // [WB-3] 2つのキー → 区切りの間に書いた順で2行。
    let out = applied(EMPTY_FM.as_bytes(), &two());
    assert_eq!(out, "---\nstatus: todo\npriority: 2\n---\n本文\n");
    let out = applied(
        EMPTY_FM.as_bytes(),
        &[
            edit("priority", NewValue::Int(2)),
            edit("status", s("todo")),
        ],
    );
    assert_eq!(out, "---\npriority: 2\nstatus: todo\n---\n本文\n");
    // CRLF でも2行とも CRLF。
    let out = applied(EMPTY_FM_CRLF.as_bytes(), &two());
    assert_eq!(out, "---\r\nstatus: todo\r\npriority: 2\r\n---\r\n本文\r\n");
}

#[test]
fn test_wb_3_empty_frontmatter_list_date_null() {
    // [WB-3] [CE-19] リスト → 縦の形。CRLF なら要素の行も CRLF。空のリスト → `tags:`。
    let out = applied(
        EMPTY_FM.as_bytes(),
        &[edit("tags", NewValue::List(vec!["a".into(), "b".into()]))],
    );
    assert_eq!(out, "---\ntags:\n  - a\n  - b\n---\n本文\n");
    let out = applied(
        EMPTY_FM_CRLF.as_bytes(),
        &[edit("tags", NewValue::List(vec!["a".into()]))],
    );
    assert_eq!(out, "---\r\ntags:\r\n  - a\r\n---\r\n本文\r\n");
    let out = applied(EMPTY_FM.as_bytes(), &[edit("tags", NewValue::List(vec![]))]);
    assert_eq!(out, "---\ntags:\n---\n本文\n");

    // [WB-3] [WB-18] 日付 → 囲まない。
    let out = applied(
        EMPTY_FM.as_bytes(),
        &[edit("due", NewValue::Date("2026-11-03".into()))],
    );
    assert_eq!(out, "---\ndue: 2026-11-03\n---\n本文\n");

    // [WB-3] [CE-9] Null → `status:`。
    let out = applied(EMPTY_FM.as_bytes(), &[edit("status", NewValue::Null)]);
    assert_eq!(out, "---\nstatus:\n---\n本文\n");

    // [WB-3] [WB-7] 型が変わる文字列は二重引用符。真偽値と数は素のまま。
    let out = applied(EMPTY_FM.as_bytes(), &[edit("status", s("no"))]);
    assert_eq!(out, "---\nstatus: \"no\"\n---\n本文\n");
    let out = applied(
        EMPTY_FM.as_bytes(),
        &[
            edit("done", NewValue::Bool(true)),
            edit("k", NewValue::Int(3)),
        ],
    );
    assert_eq!(out, "---\ndone: true\nk: 3\n---\n本文\n");
}

#[test]
fn test_wb_3_empty_frontmatter_reparses_to_written_values() {
    // [WB-3] [WB-6] 足したあと読み直すと、書いた値が読める(ほかにキーは無い)。本文はそのまま。
    for original in [EMPTY_FM, EMPTY_FM_CRLF] {
        let out = applied(
            original.as_bytes(),
            &[
                edit("status", s("todo")),
                edit("priority", NewValue::Int(2)),
                edit("tags", NewValue::List(vec!["a".into(), "b".into()])),
                edit("memo", NewValue::Null),
            ],
        );
        let fm = parse(out.as_bytes()).expect("reparse");
        let got: Vec<(String, Value)> = fm
            .entries
            .iter()
            .map(|e| (e.key.clone(), e.value.clone()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("status".to_string(), Value::Str("todo".into())),
                ("priority".to_string(), Value::Int(2)),
                (
                    "tags".to_string(),
                    Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
                ),
                ("memo".to_string(), Value::Null),
            ],
            "{original:?}"
        );
        let body = if original.contains('\r') {
            "---\r\n本文\r\n"
        } else {
            "---\n本文\n"
        };
        assert!(out.ends_with(body), "{out:?}");
    }
}

// ---- save(読み直しの検査) ----

#[test]
fn test_wb_6_empty_frontmatter_save_and_corrupt_refused() {
    // [WB-3] [WB-6] save で書ける。読み直しの検査に落ちる失敗を差し込めば、元のファイルは変わらない。
    let dir = TempDir::new("emptyfm-save");
    let p = dir.write("a.md", EMPTY_FM.as_bytes());
    let base = baseline(&p).unwrap();
    save(&p, &base, &todo()).expect("[WB-3] save writes empty frontmatter");
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "---\nstatus: todo\n---\n本文\n"
    );

    let q = dir.write("b.md", EMPTY_FM.as_bytes());
    let base = baseline(&q).unwrap();
    let faults = Faults {
        corrupt: true,
        ..Faults::default()
    };
    let r = save_with(&q, &base, &todo(), &faults);
    assert!(r.is_err(), "[WB-6] corrupted write must be refused");
    assert_eq!(std::fs::read_to_string(&q).unwrap(), EMPTY_FM);
}

// ---- Source: 既定(add_frontmatter = true) ----

#[test]
fn test_wb_3_source_empty_frontmatter_cell_is_writable_by_default() {
    // [WB-3] 既定で開いた source の Markdown で、空のフロントマターのノートのセルは lock なし(値は無し)。
    // file.* の列は lock あり(CE-8)。差分の前後 → 保存 → 読み直すと値が読める。
    let dir = TempDir::new("emptyfm-source");
    let p = dir.write("emptyfm.md", EMPTY_FM.as_bytes());
    dir.write("a.md", b"---\nstatus: done\n---\nbody\n");

    let mut md = open_default(dir.path());
    load_all(&mut md);
    let src: &mut dyn Source = &mut md;

    let row = row_of(src, "emptyfm.md");
    let c = src.get(&row, "status");
    assert_eq!(c.value, None);
    assert!(c.lock.is_none(), "[WB-3] writable, got {:?}", c.lock);
    assert!(
        src.get(&row, "file.name").lock.is_some(),
        "file.* stays locked"
    );

    let (before, after) = src.preview(&row, &two()).expect("preview");
    assert_eq!(before, EMPTY_FM.as_bytes());
    assert_eq!(
        String::from_utf8(after).unwrap(),
        "---\nstatus: todo\npriority: 2\n---\n本文\n"
    );
    let stamp = src.stamp(&row).expect("stamp");
    src.save(&row, &stamp, &two()).expect("save via Source");
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "---\nstatus: todo\npriority: 2\n---\n本文\n"
    );
    assert_eq!(
        src.get(&row, "status").value,
        Some(Value::Str("todo".into()))
    );
    assert_eq!(src.get(&row, "priority").value, Some(Value::Int(2)));
}

#[test]
fn test_wb_3_source_explicit_true_is_same_as_default() {
    // [WB-3] [CLI-3] add_frontmatter = true を渡して開く → フロントマターの無いノートと空のフロントマターのノートの
    // セルは lock なし(既定と同じ)。
    let dir = TempDir::new("emptyfm-true");
    dir.write("emptyfm.md", EMPTY_FM.as_bytes());
    dir.write("nofm.md", "# 見出し\n本文\n".as_bytes());
    let mut md = open_with(dir.path(), true);
    load_all(&mut md);
    for note in ["emptyfm.md", "nofm.md"] {
        assert_eq!(lock_of(&md, note, "status"), None, "{note}");
    }
}

// ---- 設定(CLI-3) ----

#[test]
fn test_cli_3_add_frontmatter_setting() {
    // [CLI-3] [WB-3] 既定は true。`[edit] add_frontmatter = false` を読め、知らない項目の警告は出ない。
    assert!(
        Config::default().resolved().add_frontmatter,
        "default is true"
    );
    let (c, w) = parse_config("").unwrap();
    assert!(c.resolved().add_frontmatter);
    assert!(w.is_empty(), "{w:?}");

    let (c, w) = parse_config("[edit]\nadd_frontmatter = false\n").unwrap();
    assert!(!c.resolved().add_frontmatter, "false is read");
    assert!(w.is_empty(), "[CLI-3] no warning for a known item: {w:?}");

    let (c, w) = parse_config("[edit]\nadd_frontmatter = true\n").unwrap();
    assert!(c.resolved().add_frontmatter);
    assert!(w.is_empty(), "{w:?}");

    // 真偽でない値は警告にとどめ、既定(true)のまま。
    for bad in [
        "[edit]\nadd_frontmatter = \"no\"\n",
        "[edit]\nadd_frontmatter = 0\n",
    ] {
        let (c, w) = parse_config(bad).expect("not a hard error");
        assert!(c.resolved().add_frontmatter, "{bad}: stays default");
        assert_eq!(w.len(), 1, "{bad}: one warning: {w:?}");
        assert!(w[0].contains("add_frontmatter"), "{bad}: {w:?}");
    }

    // ほかの項目と一緒でも読める。
    let (c, w) =
        parse_config("[terminal]\ncolor = false\n[edit]\nadd_frontmatter = false\n").unwrap();
    assert!(!c.resolved().add_frontmatter && !c.terminal.color);
    assert!(w.is_empty(), "{w:?}");
}

// ---- Source: add_frontmatter = false ----

#[test]
fn test_wb_3_setting_false_locks_no_and_empty_frontmatter_with_reasons() {
    // [WB-3] [CLI-3] false で開く → フロントマターの無いノート・空のフロントマターのノート・空のファイルのセルは lock あり、
    // 理由は空でなく、無いと空の2つの理由は区別できる。普通のノートは書ける(キーの無いセルも)。
    let dir = TempDir::new("emptyfm-false");
    dir.write("nofm.md", "# 見出し\n本文\n".as_bytes());
    dir.write("blank.md", b"");
    dir.write("emptyfm.md", EMPTY_FM.as_bytes());
    dir.write("emptyfm-crlf.md", EMPTY_FM_CRLF.as_bytes());
    dir.write("plain.md", b"---\nstatus: done\n---\nbody\n");

    let mut md = open_with(dir.path(), false);
    load_all(&mut md);
    let src: &dyn Source = &md;

    let mut reasons = Vec::new();
    for note in ["nofm.md", "blank.md", "emptyfm.md", "emptyfm-crlf.md"] {
        for col in ["status", "priority"] {
            let r = lock_of(src, note, col).unwrap_or_else(|| {
                panic!("[WB-3] {note} {col}: must be locked when add_frontmatter = false")
            });
            assert!(!r.trim().is_empty(), "{note} {col}: reason is empty");
        }
        reasons.push(lock_of(src, note, "status").unwrap());
    }
    assert_ne!(
        reasons[0], reasons[2],
        "[WB-3] no-frontmatter and empty-frontmatter reasons must differ"
    );
    assert_eq!(
        reasons[2], reasons[3],
        "LF and CRLF empty frontmatter share a reason"
    );

    assert_eq!(
        lock_of(src, "plain.md", "status"),
        None,
        "plain note writable"
    );
    assert_eq!(
        lock_of(src, "plain.md", "priority"),
        None,
        "missing key on plain note writable"
    );
}

#[test]
fn test_wb_3_setting_false_changes_skip_and_nothing_written() {
    // [WB-3] [CE-10] false で開く → Changes::set は Skip(理由はセルの lock と同じ)。set_many でも飛ばし、
    // 保存の流れを通してもファイルは1バイトも変わらない。普通のノートは書かれる。
    let dir = TempDir::new("emptyfm-false-save");
    let files: [(&str, &[u8]); 2] = [
        ("nofm.md", "# 見出し\n本文\n".as_bytes()),
        ("emptyfm.md", EMPTY_FM.as_bytes()),
    ];
    for (f, b) in files {
        dir.write(f, b);
    }
    let plain = dir.write("plain.md", b"---\nstatus: done\n---\nbody\n");

    let mut md = open_with(dir.path(), false);
    load_all(&mut md);

    let mut ch = Changes::new();
    for (f, _) in files {
        let row = row_of(&md, f);
        let lock = md.get(&row, "status").lock;
        match ch.set(&md, &row, "status", s("todo")) {
            Err(skip) => {
                assert_eq!(skip.row, row);
                assert_eq!(Some(&skip.reason), lock.as_ref(), "{f}");
                assert!(!skip.reason.trim().is_empty(), "{f}");
            }
            Ok(()) => panic!("[WB-3] {f}: set must be skipped when add_frontmatter = false"),
        }
    }
    let rows: Vec<RowId> = ["nofm.md", "emptyfm.md", "plain.md"]
        .iter()
        .map(|f| row_of(&md, f))
        .collect();
    let skips = ch.set_many(&md, &rows, "status", s("todo"));
    assert_eq!(skips.len(), 2, "{skips:?}");
    assert_eq!(ch.count(), 1, "only plain.md is pending");

    let _ = ch.save(&mut md);
    for (f, b) in files {
        assert_eq!(
            std::fs::read(dir.path().join(f)).unwrap(),
            b,
            "{f} unchanged"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&plain).unwrap(),
        "---\nstatus: todo\n---\nbody\n"
    );
}

#[test]
fn test_wb_5_other_forms_unaffected_by_setting() {
    // [WB-5] [WB-3] ほかの WB-5 の形(同じキーが2回・BOM・YAML として読めない・閉じない・UTF-8 でない・
    // 改行コードが混ざる・ハードリンク)は、add_frontmatter が true でも false でも読むだけで、理由も同じ。
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend_from_slice(b"---\nstatus: draft\n---\nbody\n");
    let mut not_utf8 = b"---\ntitle: ".to_vec();
    not_utf8.extend_from_slice(&[0xFF, 0xFE]);
    not_utf8.extend_from_slice(b"\nstatus: draft\n---\nbody\n");
    let forms: Vec<(&str, Vec<u8>)> = vec![
        (
            "duplicate.md",
            b"---\nstatus: a\nstatus: b\n---\nbody\n".to_vec(),
        ),
        ("bom.md", bom),
        (
            "invalid.md",
            b"---\ntitle: [unclosed\nstatus: draft\n---\nbody\n".to_vec(),
        ),
        ("unclosed.md", b"---\nstatus: draft\n\nbody\n".to_vec()),
        ("notutf8.md", not_utf8),
        (
            "mixed.md",
            b"---\ntitle: t\r\nstatus: draft\n---\nbody\n".to_vec(),
        ),
        (
            "hard.md",
            b"---\ntitle: t\nstatus: draft\n---\nbody\n".to_vec(),
        ),
    ];

    let mut by_setting: Vec<Vec<(String, Option<String>)>> = Vec::new();
    for on in [true, false] {
        let dir = TempDir::new(&format!("emptyfm-wb5-{on}"));
        for (f, b) in &forms {
            if *f == "hard.md" && !cfg!(unix) {
                continue;
            }
            dir.write(f, b);
        }
        #[cfg(unix)]
        std::fs::hard_link(dir.path().join("hard.md"), dir.path().join("hard-link.md")).unwrap();
        dir.write("plain.md", b"---\nstatus: done\n---\nbody\n");

        let mut md = open_with(dir.path(), on);
        load_all(&mut md);
        let mut got = Vec::new();
        let mut labels: Vec<&str> = forms.iter().map(|(f, _)| *f).collect();
        if cfg!(unix) {
            labels.push("hard-link.md");
        } else {
            labels.retain(|f| *f != "hard.md");
        }
        for f in labels {
            let lock = lock_of(&md, f, "status");
            assert!(
                lock.as_deref().is_some_and(|r| !r.trim().is_empty()),
                "[WB-5] {f} (add_frontmatter = {on}): must be read-only with a reason, got {lock:?}"
            );
            got.push((f.to_string(), lock));
        }
        assert_eq!(
            lock_of(&md, "plain.md", "status"),
            None,
            "plain writable ({on})"
        );

        // apply も設定を知らず、同じく書かない。
        for (f, b) in &forms {
            if *f == "hard.md" {
                continue; // ハードリンクはバイト列では分からない
            }
            assert!(apply(b, &todo()).is_err(), "[WB-5] apply {f}");
        }
        by_setting.push(got);
    }
    assert_eq!(
        by_setting[0], by_setting[1],
        "[WB-5] the setting must not change other forms"
    );
}
