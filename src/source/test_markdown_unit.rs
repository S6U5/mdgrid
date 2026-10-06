use super::*;

fn lock_of(src: &str, key: &str) -> Option<String> {
    let fm = frontmatter::parse(src.as_bytes()).expect("parse");
    let e = fm.entries.iter().find(|e| e.key == key).expect("entry");
    value_lock(src.as_bytes(), &fm, e)
}

#[test]
fn value_lock_reasons_by_shape() {
    let cases = [
        ("---\na: x\n---\n", None),
        // CE-16: 書ける形のリストは lock なし。CE-8 の形は理由つき。
        ("---\na: [x, y]\n---\n", None),
        ("---\na:\n  - x\n---\n", None),
        ("---\na: [x, [y]]\n---\n", Some("入れ子の要素を含むリスト")),
        (
            "---\na: [x, 1]\n---\n",
            Some("文字列でない要素を含むリスト"),
        ),
        (
            "---\na:\n  - x\n  - k: v\n---\n",
            Some("入れ子の要素を含むリスト"),
        ),
        (
            "---\na:\n  - 2024\n---\n",
            Some("文字列でない要素を含むリスト"),
        ),
        ("---\na:\n  -\n---\n", Some("文字列でない要素を含むリスト")),
        // CE-19: 書けない形の理由は形ごと。
        (
            "---\na:\n  - x # c\n---\n",
            Some("要素の行末にコメントがあるリスト"),
        ),
        (
            "---\na:\n  - x\n\n  - y\n---\n",
            Some("要素の間に空行があるリスト"),
        ),
        (
            "---\na:\n  - x\n  # c\n  - y\n---\n",
            Some("要素の間にコメントがあるリスト"),
        ),
        (
            "---\na:\n  - x\n   - y\n---\n",
            Some("字下げの違う要素があるリスト"),
        ),
        ("---\na:\n  -\tx\n---\n", Some("`-` のあとがタブのリスト")),
        (
            "---\na:\n  - x\n    y\n---\n",
            Some("複数行にまたがる要素を含むリスト"),
        ),
        // 最後の要素の後ろの空行・コメントは範囲の外なので書ける。
        ("---\na:\n  - x\n\n  # c\nb: 1\n---\n", None),
        ("---\na: [x,\n  y]\n---\n", Some("複数行の値")),
        ("---\na: !!str x\n---\n", Some("タグつきの値")),
        ("---\na: plain\n  more\n---\n", Some("複数行の値")),
        ("---\na: \"q\n  r\"\n---\n", Some("複数行の値")),
        ("---\na: &anc x\n---\n", Some("アンカーの値")),
        ("---\na: |\n  x\n---\n", Some("複数行の値")),
        ("---\na:\n  b: 1\n---\n", Some("ネストした値")),
    ];
    for (src, want) in cases {
        assert_eq!(lock_of(src, "a").as_deref(), want, "{src:?}");
    }
}

#[test]
fn column_locks() {
    assert!(column_lock("file.name").is_some());
    assert!(column_lock("formula.x").is_some());
    assert!(column_lock("title").is_none());
}

struct Tmp(PathBuf);
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn tmp(name: &str) -> Tmp {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-md-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    Tmp(d.canonicalize().unwrap())
}
fn loaded(folder: &Path) -> Markdown {
    let mut md = Markdown::open(&[folder.to_path_buf()]).unwrap();
    while !md.load(100).done {}
    md
}
fn labels(md: &Markdown) -> Vec<String> {
    let mut l: Vec<String> = md.rows().iter().map(|r| md.label(r)).collect();
    l.sort();
    l
}

#[test]
fn scalar_in_list_column_is_locked_and_counted() {
    // [CE-16] [CE-19] リストの列で `tags: x`(1つの文字列)のセルは、1つの要素のリストとして開ける(lock なし)。
    // 文字列でない1つの値(`tags: 2024`)は読むだけ。候補には1つの要素として数える。空の値・キーなしは lock なし。
    let t = tmp("scalar-list");
    std::fs::write(t.0.join("a.md"), "---\ntags: [x, y]\n---\n").unwrap();
    std::fs::write(t.0.join("b.md"), "---\ntags: x\n---\n").unwrap();
    std::fs::write(t.0.join("c.md"), "---\ntags: \"\"\n---\n").unwrap();
    std::fs::write(t.0.join("d.md"), "---\ntitle: d\n---\n").unwrap();
    let md = loaded(&t.0);
    let cell = |label: &str| {
        let r = md
            .rows()
            .into_iter()
            .find(|r| md.label(r) == label)
            .unwrap();
        md.get(&r, "tags")
    };
    assert!(cell("b.md").lock.is_none());
    assert!(cell("a.md").lock.is_none());
    assert!(cell("c.md").lock.is_none());
    assert!(cell("d.md").lock.is_none());
    assert_eq!(
        md.list_candidates("tags"),
        vec![("x".to_string(), 2), ("y".to_string(), 1)]
    );
    // リストの列でなければ、1つの値は書ける。
    std::fs::write(t.0.join("e.md"), "---\nstatus: x\n---\n").unwrap();
    std::fs::write(t.0.join("f.md"), "---\ntags: 2024\n---\n").unwrap();
    let md = loaded(&t.0);
    let r = md
        .rows()
        .into_iter()
        .find(|r| md.label(r) == "e.md")
        .unwrap();
    assert!(md.get(&r, "status").lock.is_none());
    let r = md
        .rows()
        .into_iter()
        .find(|r| md.label(r) == "f.md")
        .unwrap();
    assert_eq!(
        md.get(&r, "tags").lock.as_deref(),
        Some("文字列でない要素を含むリスト")
    );
}

#[cfg(unix)]
#[test]
fn symlinked_notes_inside_the_folder_are_rows() {
    let t = tmp("symlink");
    std::fs::create_dir_all(t.0.join(".obsidian")).unwrap();
    std::fs::create_dir_all(t.0.join("f")).unwrap();
    std::fs::create_dir_all(t.0.join("other")).unwrap();
    std::fs::write(t.0.join("other/x.md"), "---\na: 1\n---\n").unwrap();
    std::os::unix::fs::symlink("../other/x.md", t.0.join("f/y.md")).unwrap();
    std::os::unix::fs::symlink("../other", t.0.join("f/linkdir")).unwrap();
    let md = loaded(&t.0.join("f"));
    assert_eq!(md.rows().len(), 1, "one row per real note");
    assert_eq!(md.columns(), vec!["a"]);
    let md = loaded(&t.0.join("other"));
    assert_eq!(labels(&md), vec!["other/x.md"]);
}

#[cfg(unix)]
#[test]
fn hard_link_locks_the_row_and_conflict_is_marked() {
    let t = tmp("hardlink");
    std::fs::write(t.0.join("a.md"), "---\na: 1\n---\n").unwrap();
    std::fs::hard_link(t.0.join("a.md"), t.0.join("b.sync-conflict-1.md")).unwrap();
    std::fs::write(t.0.join("c.md"), "---\na: 1\n---\n").unwrap();
    let md = loaded(&t.0);
    let row = |l: &str| md.rows().into_iter().find(|r| md.label(r) == l).unwrap();
    assert_eq!(
        md.get(&row("a.md"), "a").lock.as_deref(),
        Some("ハードリンクがある")
    );
    assert_eq!(
        md.get(&row("a.md"), "zz").lock.as_deref(),
        Some("ハードリンクがある")
    );
    assert!(md.mark(&row("b.sync-conflict-1.md")).is_some());
    assert!(md.mark(&row("c.md")).is_none());
    assert!(md.get(&row("c.md"), "a").lock.is_none());
    assert!(md.get(&row("c.md"), "file.name").lock.is_some());
}

#[test]
fn save_replaces_without_rereading_and_preview_reads_disk() {
    let t = tmp("save");
    let p = t.0.join("a.md");
    std::fs::write(&p, "---\na: 1\n---\n").unwrap();
    let mut md = loaded(&t.0);
    let row = md.rows()[0].clone();
    let base = md.stamp(&row).unwrap();
    let edits = [Edit {
        key: "a".into(),
        value: writeback::NewValue::Int(2),
    }];
    let next = md.save(&row, &base, &edits).unwrap();
    assert_eq!(
        md.stamp(&row),
        Some(next),
        "stamp follows the written bytes"
    );
    assert_eq!(md.get(&row, "a").value, Some(Value::Int(2)));
    // 外で書き換える → preview の before は今のディスク、読んだ内容は変わらない
    std::fs::write(&p, "---\na: 3\n---\n").unwrap();
    let (before, _) = md.preview(&row, &edits).unwrap();
    assert_eq!(before, b"---\na: 3\n---\n");
    assert_eq!(md.get(&row, "a").value, Some(Value::Int(2)));
}

#[test]
fn setting_false_refuses_save_directly() {
    // [WB-3] add_frontmatter = false: 画面の lock を通らずに save を呼んでも、フロントマターの無いノートと
    // 空のフロントマターのノートは書かない(ファイルは変わらない)。preview は書かないので設定を見ない。
    let t = tmp("addfm-off");
    let files = [("nofm.md", "body\n"), ("empty.md", "---\n---\nbody\n")];
    for (f, b) in files {
        std::fs::write(t.0.join(f), b).unwrap();
    }
    let mut md = Markdown::open(std::slice::from_ref(&t.0)).unwrap();
    md.set_add_frontmatter(false);
    while !md.load(100).done {}
    let edits = [Edit {
        key: "a".into(),
        value: writeback::NewValue::Int(1),
    }];
    for (f, b) in files {
        let row = md.rows().into_iter().find(|r| md.label(r) == f).unwrap();
        let base = md.stamp(&row).unwrap();
        let r = md.save(&row, &base, &edits);
        assert!(
            matches!(r, Err(SaveError::Edit(EditError::NotEditable(_)))),
            "{f}: {r:?}"
        );
        assert_eq!(std::fs::read_to_string(t.0.join(f)).unwrap(), b, "{f}");
    }
}

#[test]
fn setting_false_external_change_to_empty_frontmatter_is_changed() {
    // [WB-4] [WB-16] [WB-3] add_frontmatter = false で普通のノートの a をためる → 外で空のフロントマターに変わり、
    // 読み直しも済む → 保存は設定の失敗ではなく「外の変更」(Changed)。差分も外の変更として出る。
    let t = tmp("addfm-off-external");
    let p = t.0.join("a.md");
    std::fs::write(&p, "---\na: 1\n---\nbody\n").unwrap();
    let mut md = Markdown::open(std::slice::from_ref(&t.0)).unwrap();
    md.set_add_frontmatter(false);
    while !md.load(100).done {}
    let row = md.rows()[0].clone();
    let mut ch = crate::changes::Changes::new();
    ch.set(&md, &row, "a", writeback::NewValue::Int(2))
        .expect("plain note is writable");
    let changed = "---\n---\nbody changed\n";
    std::fs::write(&p, changed).unwrap();
    for _ in 0..50 {
        if !md.changed().is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(md.get(&row, "a").lock.is_some(), "re-read: now locked");
    let previews = ch.previews(&md);
    assert!(
        matches!(previews.as_slice(), [Ok(pv)] if pv.before == changed.as_bytes()),
        "{previews:?}"
    );
    let out = ch.save(&mut md);
    assert!(
        matches!(out.as_slice(), [(_, crate::changes::Outcome::Changed)]),
        "{out:?}"
    );
    assert_eq!(std::fs::read_to_string(&p).unwrap(), changed);
}

#[test]
fn body_tags_skip_code_headings_and_numbers() {
    let body =
        "# Heading\nsee #a/b and x#notag #123 #ok-1.\n`#code` ```\n```\n#fenced\n```\n#last/";
    assert_eq!(body_tags(body), vec!["a/b", "ok-1", "last"]);
}

#[test]
fn body_start_after_unreadable_frontmatter() {
    let text = "---\na: [\n---\n#t\n";
    assert_eq!(&text[body_start(text, None)..], "#t\n");
    assert_eq!(body_start("#t\n", None), 0);
}

#[test]
fn body_tags_code_forms() {
    // 字下げ4つのコードブロック(空行の後)は見ない。段落の続きの字下げは見る。
    assert_eq!(
        body_tags("text\n\n    #code\n\t#tab\n\n#after\n"),
        vec!["after"]
    );
    assert_eq!(body_tags("para\n    #cont\n"), vec!["cont"]);
    // リストの中の字下げはコードではない。
    assert_eq!(body_tags("- item\n\n    #inlist\n"), vec!["inlist"]);
    // 2つのバッククォートのインラインコードの中の ` と #。
    assert_eq!(body_tags("``a ` #in`` #out\n"), vec!["out"]);
    // 閉じないバッククォートは字のまま。
    assert_eq!(body_tags("` #open\n"), vec!["open"]);
    // 1行で閉じる ```a``` はフェンスではない。
    assert_eq!(body_tags("```a``` #x\n#y\n"), vec!["x", "y"]);
    // 記号と長さの対: ```` は ``` や ~~~ では閉じない。
    assert_eq!(body_tags("````\n```\n#a\n~~~\n#b\n`````\n#c\n"), vec!["c"]);
    assert_eq!(body_tags("~~~\n```\n#a\n~~~ \n#b\n"), vec!["b"]);
    // 閉じの後ろに字があれば閉じではない。
    assert_eq!(body_tags("```\n``` x\n#a\n```\n#b\n"), vec!["b"]);
}

#[test]
fn tags_of_read_only_and_bom() {
    // BOM つき(ReadOnly): フロントマターの tags は読まず、YAML の行も本文にしない。
    let bom = "\u{feff}---\ntags: [fm]\nx: a #y\n---\n#body\n";
    assert!(frontmatter::parse(bom.as_bytes()).is_err());
    assert_eq!(tags_of(bom.as_bytes(), None), vec!["body"]);
    // 重複キー(ReadOnly)でも本文の始まりは閉じの後ろ。
    let dup = "---\ntags: [a]\ntags: [b]\n z: #no\n---\n#yes\n";
    assert!(frontmatter::parse(dup.as_bytes()).is_err());
    assert_eq!(tags_of(dup.as_bytes(), None), vec!["yes"]);
    // 閉じていなければ全体が本文。
    assert_eq!(tags_of(b"---\n #a\n", None), vec!["a"]);
    // 読めるフロントマターなら tags と本文の両方。
    let ok = b"---\ntags: p, q\n---\n#r #p\n";
    let fm = frontmatter::parse(ok).unwrap();
    assert_eq!(tags_of(ok, Some(&fm)), vec!["p", "q", "r"]);
}

#[test]
fn open_vault_rows_are_all_notes_under_the_root() {
    // `.base` が保管庫の中のフォルダにあっても、行は根(`.obsidian/` の上)の下の全ノート(BV-2)。
    let t = tmp("open-vault");
    std::fs::create_dir_all(t.0.join(".obsidian")).unwrap();
    std::fs::create_dir_all(t.0.join("views")).unwrap();
    std::fs::create_dir_all(t.0.join("notes")).unwrap();
    std::fs::write(t.0.join("a.md"), "---\na: 1\n---\n").unwrap();
    std::fs::write(t.0.join("notes/b.md"), "---\nb: 1\n---\n").unwrap();
    std::fs::write(t.0.join("views/t.base"), "views: []\n").unwrap();
    let mut md = Markdown::open_vault(&t.0.join("views/t.base")).unwrap();
    while !md.load(100).done {}
    assert_eq!(labels(&md), vec!["a.md", "notes/b.md"]);
    // 根が無ければ `.base` のあるフォルダ。
    let u = tmp("open-vault-plain");
    std::fs::create_dir_all(u.0.join("sub")).unwrap();
    std::fs::write(u.0.join("x.md"), "---\na: 1\n---\n").unwrap();
    std::fs::write(u.0.join("sub/y.md"), "---\na: 1\n---\n").unwrap();
    std::fs::write(u.0.join("sub/t.base"), "views: []\n").unwrap();
    let mut md = Markdown::open_vault(&u.0.join("sub/t.base")).unwrap();
    while !md.load(100).done {}
    assert_eq!(labels(&md), vec!["y.md"]);
    // 無い `.base` は Err。
    assert!(Markdown::open_vault(&u.0.join("missing.base")).is_err());
}
