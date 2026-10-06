use super::*;

#[test]
fn test_sr_8_blank_is_same_as_empty() {
    // [SR-8] 空白だけの値も空とみなし、次の候補へ進む。
    assert_eq!(resolve_editor(None, Some(" "), Some("nvim")), "nvim");
    assert_eq!(resolve_editor(Some("  "), Some("\t"), Some("nvim")), "nvim");
    assert_eq!(resolve_editor(Some(" "), Some(" "), Some(" ")), "vi");
    // 空白でない値は前後の空白も含めてそのまま(単語分けは起動の側)。
    assert_eq!(resolve_editor(Some(" hx "), None, None), " hx ");
    // 設定の editor = "  " は警告なしに None。
    let (c, w) = parse("editor = \"  \"\n").unwrap();
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(c.editor, None);
    assert_eq!(c, Config::default());
}

#[test]
fn test_cli_12_every_item_is_read() {
    // [CLI-12] 項目の表(ITEMS)の名前はどれも parse が読む: どの型にも合わない値で、その項目の型の警告が出る
    // (表にあって読み取りの無い項目は、知らない項目の警告も型の警告も出ないので、ここで落ちる)。
    for name in KEYS {
        let (c, w) = parse(&format!("{name} = [[1]]\n")).unwrap();
        assert_eq!(c, Config::default(), "{name}");
        assert_eq!(w.len(), 1, "{name}: {w:?}");
        assert!(
            w[0].contains(name) && !w[0].contains("知らない項目"),
            "{name}: {w:?}"
        );
    }
}

#[test]
fn test_cli_12_items_have_descriptions() {
    // [CLI-12] 項目の表の各行に型・英語と日本語の説明・例がある。既定で書かない項目は editor と keys だけ。
    for item in ITEMS {
        assert!(
            !item.ty.is_empty()
                && !item.en.is_empty()
                && !item.ja.is_empty()
                && !item.example.is_empty(),
            "{}",
            item.name
        );
    }
    let no_default: Vec<&str> = ITEMS
        .iter()
        .filter(|i| i.default.is_none())
        .map(|i| i.name)
        .collect();
    assert_eq!(no_default, ["editor", "keys"]);
    // 例はどれも警告なしに読める。
    for item in ITEMS {
        let (_, w) = parse(item.example).unwrap();
        assert!(w.is_empty(), "{}: {w:?}", item.name);
    }
}

#[test]
fn test_sr_11_save_state_removes_stale_tmp_files() {
    // [SR-11] 前の保存が途中で止まって残した同じ name の一時ファイルは、次の保存で消える。
    // ほかの name の一時ファイルと状態のファイルには触らない。
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mdgrid-config-unit-{}-{}",
        std::process::id(),
        nanos
    ));
    let dir = root.join("state");
    let notes = root.join("notes");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&notes).unwrap();

    let name = state_file_name(&target_key(&notes), "表");
    let stale1 = dir.join(format!(".{}.tmp.99999", name));
    let stale2 = dir.join(format!(".{}.tmp.12345", name));
    let other = dir.join(format!(".{}.tmp.1", state_file_name("/other", "表")));
    for p in [&stale1, &stale2, &other] {
        std::fs::write(p, "half").unwrap();
    }

    let st = ViewState {
        order: vec!["title".to_string()],
        ..ViewState::default()
    };
    save_state(&dir, &notes, "表", &st).unwrap();

    assert!(!stale1.exists(), "残った一時ファイルが消えていない");
    assert!(!stale2.exists(), "残った一時ファイルが消えていない");
    assert!(other.exists(), "ほかの name の一時ファイルを消した");
    assert_eq!(load_state(&dir, &notes, "表"), st);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut want = vec![
        name,
        other.file_name().unwrap().to_string_lossy().into_owned(),
    ];
    want.sort();
    assert_eq!(names, want);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_ce_22_config_date_items_wrong_types_warn() {
    // [CE-22][CE-21][CLI-3] 文字列でない date_format・week_start、大文字の "Mon" は警告にして既定のまま。
    let (c, w) = parse("date_format = 1\nweek_start = \"Mon\"\n").unwrap();
    assert_eq!(c.date_format, DateFormat::iso());
    assert_eq!(c.week_start, WeekStart::Sun);
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().any(|s| s.contains("date_format")), "{w:?}");
    assert!(w.iter().any(|s| s.contains("week_start")), "{w:?}");
    // 改行を含む形の警告も1行。
    let (_, w) = parse("date_format = \"MM\\nDD\"\n").unwrap();
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(!w[0].contains('\n'), "{w:?}");
}

#[test]
fn test_nv_23_config_search_bar() {
    // [NV-23][CLI-3] 検索の欄は既定で出す。`search_bar = false` で出さない。型の違う値は警告にして既定のまま。
    let (c, w) = parse("").unwrap();
    assert!(c.search_bar);
    assert!(w.is_empty());
    let (c, w) = parse("search_bar = false\n").unwrap();
    assert!(!c.search_bar);
    assert!(w.is_empty(), "知っている項目なので警告しない: {w:?}");
    let (c, w) = parse("search_bar = \"no\"\n").unwrap();
    assert!(c.search_bar);
    assert_eq!(w.len(), 1);
    assert!(w[0].contains("search_bar"), "{w:?}");
    // 綴りの違う項目は知らない項目の警告。
    let (c, w) = parse("searchbar = false\n").unwrap();
    assert!(c.search_bar);
    assert!(
        w[0].contains("知らない項目") && w[0].contains("searchbar"),
        "{w:?}"
    );
}
