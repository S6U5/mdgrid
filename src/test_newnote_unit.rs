use super::*;

fn edit(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.to_string(),
        value,
    }
}

#[test]
fn test_ce_25_fixed_values_from_expr() {
    // [CE-25] `&&` の項・逆向きの `==`・note["…"] も拾い、`||`・`!=`・file.* は拾わない。
    let e = expr::parse(r#"status == "todo" && 3 == note["優先"] && tags.contains("a")"#).unwrap();
    assert_eq!(
        e.fixed_values(),
        vec![
            Fixed::Eq("status".to_string(), Val::Str("todo".to_string())),
            Fixed::Eq("優先".to_string(), Val::Num(3.0)),
            Fixed::Contains("tags".to_string(), "a".to_string()),
        ]
    );
    for src in [
        r#"status == "a" || status == "b""#,
        r#"status != "a""#,
        r#"file.name == "a""#,
        r#"!(status == "a")"#,
    ] {
        assert!(expr::parse(src).unwrap().fixed_values().is_empty(), "{src}");
    }
}

#[test]
fn test_ce_25_prefill_merges_list_and_skips_file_columns() {
    // [CE-25] 同じリストの列の値は合わせ、file.* の列は入れない。数の式は数で入れる。
    let kinds: HashMap<String, Kind> = [("tags".to_string(), Kind::List)].into_iter().collect();
    let s = Settings {
        filters: vec![
            crate::settings::Cond {
                col: "tags".to_string(),
                op: Op::Keep(vec![Some("a".to_string())]),
            },
            crate::settings::Cond {
                col: "file.name".to_string(),
                op: Op::Keep(vec![Some("x".to_string())]),
            },
        ],
        ..Settings::default()
    };
    let pre = prefill(
        &s,
        &[
            r#"tags.contains("b")"#.to_string(),
            "level == 2".to_string(),
            "壊れた ((".to_string(),
        ],
        &kinds,
    );
    assert_eq!(
        pre,
        vec![
            edit("tags", NewValue::List(vec!["a".into(), "b".into()])),
            edit("level", NewValue::Int(2)),
        ]
    );
}

#[test]
fn test_ce_26_answer_replaces_set_value() {
    // [CE-26] 聞いた値は設定の値を置き換え、空なら(空のまま進めた)その列を書かない。
    let rule = NewNote {
        set: vec![("kind".to_string(), NewValue::Str("memo".to_string()))],
        ..NewNote::default()
    };
    let out = build(
        &rule,
        &[],
        &[edit("kind", NewValue::Str("idea".to_string()))],
    )
    .unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "---\nkind: idea\n---\n");
    let out = build(&rule, &[], &[edit("kind", NewValue::Str(" ".to_string()))]).unwrap();
    assert_eq!(out, b"");
    assert_eq!(build(&NewNote::default(), &[], &[]).unwrap(), b"");
    assert!(build(
        &NewNote::default(),
        &[edit("a", NewValue::Str("x\ny".into()))],
        &[]
    )
    .is_err());
}

#[test]
fn test_ce_27_set_values_round_trip_through_toml() {
    // [CE-27] set の値は TOML の値と行き来できる(Null は空の並びになる)。
    for v in [
        NewValue::Str("x".into()),
        NewValue::Int(2),
        NewValue::Float(1.5),
        NewValue::Bool(true),
        NewValue::Date("2026-10-05".into()),
        NewValue::List(vec!["a".into()]),
    ] {
        assert_eq!(
            value_from_toml(&value_to_toml(&v)),
            Some(v.clone()),
            "{v:?}"
        );
    }
    assert_eq!(
        value_from_toml(&value_to_toml(&NewValue::Null)),
        Some(NewValue::List(Vec::new()))
    );
    let t: toml::Table = "a = { b = 1 }\nc = [[1]]\nd = 2026-10-05T09:00:00+09:00"
        .parse()
        .unwrap();
    for k in ["a", "c", "d"] {
        assert_eq!(value_from_toml(&t[k]), None, "{k}");
    }
}

#[test]
fn test_ce_25_note_path_trims_and_refuses_folder_name() {
    // [CE-25] 名前の前後の空白は除く。フォルダで終わる名前は断る。
    let root = Path::new("/no/such/root");
    assert_eq!(
        note_path(root, "", "  メモ  ").unwrap(),
        root.join("メモ.md")
    );
    assert!(note_path(root, "", "a/").is_err());
    assert!(note_path(root, "", "./a/./b").is_ok());
    // 部品の前後の空白も除いてつなぐ。
    assert_eq!(
        note_path(root, " 下 ", " a / b ").unwrap(),
        root.join("下").join("a").join("b.md")
    );
}

/// 試験ごとの一時フォルダ(終わったら消す)。
struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!(
            "mdgrid-newnote-unit-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn test_ce_25_spaced_dotdot_does_not_escape() {
    // [CE-25] 空白で囲んだ `..` も外に出る部品として断り、外に何も作らない(フォルダの設定も同じ)。
    let t = Tmp::new("escape");
    let root = t.0.join("vault");
    std::fs::create_dir_all(&root).unwrap();
    for (folder, name) in [
        ("", "a/b/c/ ../ ../ ../ ../escape"),
        ("", " .. /x"),
        (" ..", "x"),
        ("a/ .. / ..", "x"),
    ] {
        assert!(
            note_path(&root, folder, name).is_err(),
            "{folder:?} {name:?}"
        );
    }
    assert_eq!(names_in(&t.0), vec!["vault".to_string()]);
    assert!(names_in(&root).is_empty());
}

#[test]
fn test_ce_25_obsidian_bad_chars_and_dot_parts_are_refused() {
    // [CE-25] Obsidian が許さない文字・`\`・`.` で始まる部品(.obsidian・.git・.trash も)は理由つきで断る。
    let root = Path::new("/no/such/root");
    for name in [
        "a#b",
        "a|b",
        "a^b",
        "a[b",
        "a]b",
        "a:b",
        "a*b",
        "a?b",
        "a\"b",
        "a<b",
        "a>b",
        r"a\b",
        ".obsidian/x",
        ".git/x",
        ".trash/x",
        "下/.隠し",
        ".md",
    ] {
        match note_path(root, "", name) {
            Err(r) => assert!(!r.trim().is_empty(), "{name:?}"),
            Ok(p) => panic!("{name:?} → {p:?}"),
        }
    }
    assert!(note_path(root, ".obsidian", "x").is_err());
    assert!(note_path(root, "a:b", "x").is_err());
    // `.md` の補いは大文字小文字を区別しない。
    assert_eq!(note_path(root, "", "a.MD").unwrap(), root.join("a.md"));
    assert_eq!(note_path(root, "", "a.Md").unwrap(), root.join("a.md"));
}

#[test]
fn test_ce_27_views_date_survives_load_save_load() {
    // [CE-27] [BV-17] views.toml の set の日付は、読む → 保存 → 読む のあとも日付のまま書かれる。
    let t = Tmp::new("views-date");
    let conf = t.0.join("config");
    let notes = t.0.join("notes");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::create_dir_all(&notes).unwrap();
    let key = crate::config::target_key(&notes);
    let text = format!(
        "[[target]]\npath = {:?}\n\n[[target.view]]\nname = \"v\"\n\
         [target.view.new_note]\nfuture = 1\n[target.view.new_note.set]\nd = 2026-10-05\n\
         t = 2026-10-05T09:00:00\n",
        key
    );
    std::fs::write(conf.join("views.toml"), text).unwrap();
    let (views, w) = crate::views::load_views(&conf, &notes);
    assert_eq!(w.len(), 1, "知らない項目 future の警告だけ: {w:?}");
    assert_eq!(
        views[0].new_note.as_ref().unwrap().set[0],
        ("d".to_string(), NewValue::Date("2026-10-05".into()))
    );
    crate::views::save_views(&conf, &notes, &views).unwrap();
    let (back, _) = crate::views::load_views(&conf, &notes);
    assert_eq!(back, views);
    let rule = back[0].new_note.as_ref().unwrap();
    let out = String::from_utf8(build(rule, &[], &[]).unwrap()).unwrap();
    assert!(out.contains("\nd: 2026-10-05\n"), "{out}");
    assert!(out.contains("\nt: 2026-10-05T09:00:00\n"), "{out}");
    // 知らない項目も保存で保つ。
    let saved = std::fs::read_to_string(conf.join("views.toml")).unwrap();
    assert!(saved.contains("future = 1"), "{saved}");
    assert!(!saved.contains("toml_private"), "{saved}");
}

/// 作った中身のフロントマターのキーの値。
fn fm_value(bytes: &[u8], key: &str) -> Option<crate::frontmatter::Value> {
    crate::frontmatter::parse(bytes)
        .unwrap()
        .entries
        .into_iter()
        .find(|e| e.key == key)
        .map(|e| e.value)
}

#[test]
fn test_ce_25_expr_prefill_keeps_row_in_view() {
    // [CE-25] 式から拾った値は書いてある型のまま入れ、作ったノートで式が真になる(ビューに残る)。
    let kinds: HashMap<String, Kind> = [
        ("code".to_string(), Kind::Number),
        ("done".to_string(), Kind::Checkbox),
    ]
    .into_iter()
    .collect();
    for src in [
        r#"code == "3""#,
        "code == 3",
        r#"status == "todo""#,
        "done == true",
        r#"note["優先"] == "高""#,
    ] {
        let pre = prefill(&Settings::default(), &[src.to_string()], &kinds);
        assert_eq!(pre.len(), 1, "{src}");
        let out = build(&NewNote::default(), &pre, &[]).unwrap();
        let file = crate::source::FileInfo {
            name: "n.md".into(),
            basename: "n".into(),
            ext: "md".into(),
            path: "n.md".into(),
            folder: String::new(),
            size: 0,
            mtime: 0,
            ctime: 0,
            tags: Vec::new(),
        };
        let prop = |k: &str| fm_value(&out, k);
        let env = expr::Env {
            prop: &prop,
            file: &file,
            formula: &|_| None,
            today: 0,
            now: 0,
        };
        let e = expr::parse(src).unwrap();
        assert_eq!(
            expr::eval(&e, &env),
            Val::Bool(true),
            "{src}: {}",
            String::from_utf8_lossy(&out)
        );
    }
}

#[test]
fn test_ce_25_settings_prefill_keeps_row_in_view() {
    // [CE-25] 設定の条件(Keep・Cmp ==)から入れた値で、作ったノートが settings::matches で残る。
    let cases = [
        ("status", Kind::Text, Op::Keep(vec![Some("todo".into())])),
        ("tags", Kind::List, Op::Keep(vec![Some("会議".into())])),
        ("done", Kind::Checkbox, Op::Keep(vec![Some("true".into())])),
        ("priority", Kind::Number, Op::Cmp(CmpOp::Eq, " 3 ".into())),
        ("due", Kind::Date, Op::Cmp(CmpOp::Eq, "2026-10-05".into())),
        (
            "at",
            Kind::DateTime,
            Op::Cmp(CmpOp::Eq, "2026-10-05T09:00".into()),
        ),
    ];
    for (col, kind, op) in cases {
        let cond = crate::settings::Cond {
            col: col.to_string(),
            op,
        };
        let s = Settings {
            filters: vec![cond.clone()],
            ..Settings::default()
        };
        let kinds: HashMap<String, Kind> = [(col.to_string(), kind)].into_iter().collect();
        let pre = prefill(&s, &[], &kinds);
        assert_eq!(pre.len(), 1, "{col}");
        let out = build(&NewNote::default(), &pre, &[]).unwrap();
        let v = fm_value(&out, col);
        assert!(
            crate::settings::matches(&cond, v.as_ref(), kind),
            "{col}: {}",
            String::from_utf8_lossy(&out)
        );
    }
}

#[test]
fn test_ce_25_has_tag_goes_into_tags() {
    // [CE-25] `file.hasTag("会議")` は tags の列に入れる(先頭の # は除く)。引数が2つなら決まらない。
    let pre = prefill(
        &Settings::default(),
        &[r##"file.hasTag("#会議")"##.to_string()],
        &HashMap::new(),
    );
    assert_eq!(pre, vec![edit("tags", NewValue::List(vec!["会議".into()]))]);
    let out = build(&NewNote::default(), &pre, &[]).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "---\ntags:\n  - 会議\n---\n"
    );
    let pre = prefill(
        &Settings::default(),
        &[r#"file.hasTag("a", "b")"#.to_string()],
        &HashMap::new(),
    );
    assert!(pre.is_empty());
}

#[test]
fn test_ce_27_bad_ask_and_set_columns_warn_and_drop() {
    // [CE-27] ask・set の file.*・formula.* と ask の重なりは警告して捨てる。
    let (c, w) = crate::config::parse(
        "[new_note]\nask = [\"a\", \"file.name\", \"a\", \"formula.x\"]\n[new_note.set]\n\"file.name\" = \"x\"\nb = 1\n",
    )
    .unwrap();
    assert_eq!(c.new_note.ask, vec!["a".to_string()]);
    assert_eq!(c.new_note.set, vec![("b".to_string(), NewValue::Int(1))]);
    assert_eq!(w.len(), 4, "{w:?}");
}

#[test]
fn test_ce_26_empty_answer_drops_preset_values() {
    // [CE-26] 聞く項目を空のまま進めたら、絞り込みの値も set の値も書かない(ほかの列は残す)。
    let rule = NewNote {
        ask: vec!["status".to_string(), "kind".to_string()],
        set: vec![
            ("kind".to_string(), NewValue::Str("memo".to_string())),
            ("level".to_string(), NewValue::Int(1)),
        ],
        ..NewNote::default()
    };
    let pre = [edit("status", NewValue::Str("todo".to_string()))];
    let answers = [
        edit("status", NewValue::Null),
        edit("kind", NewValue::Str(String::new())),
    ];
    let out = build(&rule, &pre, &answers).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "---\nlevel: 1\n---\n");
    // 空のリストも空の答え。
    let rule = NewNote {
        ask: vec!["tags".to_string()],
        set: vec![("tags".to_string(), NewValue::List(vec!["inbox".into()]))],
        ..NewNote::default()
    };
    let out = build(&rule, &[], &[edit("tags", NewValue::List(Vec::new()))]).unwrap();
    assert_eq!(out, b"");
}
