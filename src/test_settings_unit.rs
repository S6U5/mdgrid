use super::*;

#[test]
fn test_nv_19_datetime_cmp_reads_time() {
    // [NV-19] 日時の列は時刻まで比べ、日付だけの値は 00:00。
    let c = Cond {
        col: "at".to_string(),
        op: Op::Cmp(CmpOp::Gt, "2026-10-01T09:00".to_string()),
    };
    let s = |x: &str| Value::Str(x.to_string());
    assert!(matches(&c, Some(&s("2026-10-01T09:30")), Kind::DateTime));
    assert!(matches(&c, Some(&s("2026-10-01T09:00:01")), Kind::DateTime));
    assert!(!matches(&c, Some(&s("2026-10-01T09:00")), Kind::DateTime));
    assert!(!matches(&c, Some(&s("2026-10-01")), Kind::DateTime));
    assert!(!matches(&c, Some(&s("later")), Kind::DateTime));
}

#[test]
fn test_nv_19_bad_operand_keeps_nothing() {
    // [NV-19] 比較の値が列の型で読めなければ、どの行も残らない。
    let c = Cond {
        col: "n".to_string(),
        op: Op::Cmp(CmpOp::Ge, "abc".to_string()),
    };
    assert!(!matches(&c, Some(&Value::Int(3)), Kind::Number));
}

#[test]
fn test_nv_19_value_keys_display() {
    // [NV-19] 表示の文字列: Int は "3"、Float は Rust の表記、Bool は "true"/"false"。
    assert_eq!(
        value_keys(Some(&Value::Int(3))),
        vec![Some("3".to_string())]
    );
    assert_eq!(
        value_keys(Some(&Value::Float(1.5))),
        vec![Some("1.5".to_string())]
    );
    assert_eq!(
        value_keys(Some(&Value::Bool(true))),
        vec![Some("true".to_string())]
    );
}

#[test]
fn test_nv_17_keys_toml_form() {
    // [NV-17] Keep・Drop の値は `{ value = "..." }` と `{ empty = true }` の表の配列で書く。
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct W {
        s: Settings,
    }
    let w = W {
        s: Settings {
            filters: vec![Cond {
                col: "status".to_string(),
                op: Op::Keep(vec![None, Some("todo".to_string())]),
            }],
            ..Settings::default()
        },
    };
    let text = toml::to_string(&w).unwrap();
    assert!(text.contains("empty = true"), "{}", text);
    assert!(text.contains("value = \"todo\""), "{}", text);
    assert_eq!(toml::from_str::<W>(&text).unwrap(), w);
}

#[test]
fn test_nv_15_inherit_drops_emptied_groups() {
    // [NV-15] Inherit で絞ってまとまりが空になったら、その見出しは出さない。
    let rows: Vec<RowId> = ["a", "b", "c"]
        .iter()
        .map(|s| RowId(s.to_string()))
        .collect();
    let groups = vec![("X".to_string(), 0..1), ("Y".to_string(), 1..3)];
    let s = Settings {
        filters: vec![Cond {
            col: "k".to_string(),
            op: Op::Drop(vec![Some("x".to_string())]),
        }],
        ..Settings::default()
    };
    let get = |r: &RowId, _: &str| -> Option<Value> {
        Some(Value::Str(if r.0 == "a" { "x" } else { "y" }.to_string()))
    };
    let (out, g) = apply(rows, groups, &s, &get, &|_| Kind::Text);
    assert_eq!(out.len(), 2);
    assert_eq!(g, vec![("Y".to_string(), 0..2)]);
}

#[test]
fn test_nv_19_ne_on_list_needs_every_element_unequal() {
    // [NV-19] リストの値に ≠ を当てる → 等しい要素が1つでもあれば残らない。
    let c = Cond {
        col: "tags".to_string(),
        op: Op::Cmp(CmpOp::Ne, "会議".to_string()),
    };
    let l = |xs: &[&str]| Value::List(xs.iter().map(|x| Value::Str(x.to_string())).collect());
    assert!(!matches(&c, Some(&l(&["会議", "雑務"])), Kind::List));
    assert!(matches(&c, Some(&l(&["雑務"])), Kind::List));
    assert!(!matches(&c, Some(&l(&[])), Kind::List));
    // = はどれかの要素が等しければ残る。
    let eq = Cond {
        col: "tags".to_string(),
        op: Op::Cmp(CmpOp::Eq, "会議".to_string()),
    };
    assert!(matches(&eq, Some(&l(&["会議", "雑務"])), Kind::List));
}

#[test]
fn test_nv_15_sort_list_value_in_non_list_column_skips_empty_elements() {
    // [NV-15] List でない列のリストの値は、空でない最初の要素で並べる([null, "b"] は "b")。
    let rows: Vec<RowId> = ["x", "y"].iter().map(|s| RowId(s.to_string())).collect();
    let get = |r: &RowId, _: &str| -> Option<Value> {
        Some(if r.0 == "x" {
            Value::Str("c".to_string())
        } else {
            Value::List(vec![Value::Null, Value::Str("b".to_string())])
        })
    };
    let s = Settings {
        sorts: vec![("k".to_string(), Dir::Asc)],
        ..Settings::default()
    };
    let (out, _) = apply(rows, Vec::new(), &s, &get, &|_| Kind::Text);
    assert_eq!(out, vec![RowId("y".to_string()), RowId("x".to_string())]);
}

#[test]
fn test_nv_17_broken_settings_keep_other_state() {
    // [NV-17] 状態のファイルの settings の表が壊れていても、ほかの見た目の状態は読め、設定だけ既定。
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("mdgrid-stunit-{}-{}", std::process::id(), nanos));
    let dir = root.join("state");
    let notes = root.join("notes");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&notes).unwrap();
    let st = crate::config::ViewState {
        order: vec!["title".to_string()],
        hidden: vec!["due".to_string()],
        settings: Settings {
            group: Group::Off,
            ..Settings::default()
        },
        ..Default::default()
    };
    crate::config::save_state(&dir, &notes, "表", &st).unwrap();
    let file = std::fs::read_dir(&dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let text = std::fs::read_to_string(&file).unwrap();
    let mut table: toml::Table = toml::from_str(&text).unwrap();
    let mut bad = toml::Table::new();
    bad.insert("group".to_string(), toml::Value::Integer(42));
    table.insert("settings".to_string(), toml::Value::Table(bad));
    std::fs::write(&file, toml::to_string(&table).unwrap()).unwrap();

    let back = crate::config::load_state(&dir, &notes, "表");
    assert!(back.settings.is_default());
    assert_eq!(back.order, vec!["title".to_string()]);
    assert_eq!(back.hidden, vec!["due".to_string()]);
    let _ = std::fs::remove_dir_all(&root);
}
