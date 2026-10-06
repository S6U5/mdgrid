use super::*;

fn hex(b: [u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn sha256_known_vectors() {
    assert_eq!(
        hex(sha256(b"")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        hex(sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        hex(sha256(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        )),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    let million = vec![b'a'; 1_000_000];
    assert_eq!(
        hex(sha256(&million)),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn floats_keep_their_type_in_yaml_1_1_and_1_2() {
    assert_eq!(render_float(1.0), "1.0");
    assert_eq!(render_float(1.5), "1.5");
    assert_eq!(render_float(1e300), "1.0e+300");
    assert_eq!(render_float(1e-7), "1.0e-7");
    assert_eq!(resolve_plain(&render_float(1e300)), Value::Float(1e300));
}

fn write_list(src: &str, items: &[&str]) -> String {
    let e = Edit {
        key: "tags".into(),
        value: NewValue::List(items.iter().map(|s| s.to_string()).collect()),
    };
    let out = apply(src.as_bytes(), &[e]).unwrap_or_else(|e| panic!("{src:?} {items:?}: {e:?}"));
    String::from_utf8(out).unwrap()
}

#[test]
fn block_list_keeps_remaining_lines_byte_for_byte() {
    // [CE-18] [WB-1] 残る要素の行は、`-` のあとの空白・末尾の空白・引用符を含めてそのまま。新しい要素の行だけを作る。
    let src = "---\ntags:\n  -   a\n  - b  \n  - 'c'\nx: 1\n---\n";
    assert_eq!(
        write_list(src, &["a", "b", "c", "d"]),
        "---\ntags:\n  -   a\n  - b  \n  - 'c'\n  - d\nx: 1\n---\n"
    );
    // 消す要素は行ごと消す。
    assert_eq!(
        write_list(src, &["a", "c"]),
        "---\ntags:\n  -   a\n  - 'c'\nx: 1\n---\n"
    );
    // 並べ替えは元の行を並べ替える。
    assert_eq!(
        write_list(src, &["c", "b", "a"]),
        "---\ntags:\n  - 'c'\n  - b  \n  -   a\nx: 1\n---\n"
    );
    // CRLF の行も、そのまま。
    let crlf = "---\r\ntags:\r\n  -  a\r\n---\r\n";
    assert_eq!(
        write_list(crlf, &["b", "a"]),
        "---\r\ntags:\r\n  - b\r\n  -  a\r\n---\r\n"
    );
}

#[test]
fn flow_list_quotes_values_with_quote_chars() {
    // [WB-7] フローの中の `x 'y`・`x "y` は二重引用符で書く(素のままだと読み直しで InvalidYaml)。
    for v in ["x 'y", "x \"y", "it's", "a\"b"] {
        let out = write_list("---\ntags: [a]\n---\n", &["a", v]);
        let fm = parse(out.as_bytes()).unwrap_or_else(|r| panic!("{v:?}: {out:?}: {r:?}"));
        assert_eq!(
            fm.entries[0].value,
            Value::List(vec![Value::Str("a".into()), Value::Str(v.into())]),
            "{out:?}"
        );
        assert!(out.contains(&double_quote(v)), "{out:?}");
    }
    // ブロックの中では素のまま書ける値は素のまま。
    let out = write_list("---\ntags:\n  - a\n---\n", &["a", "it's"]);
    assert_eq!(out, "---\ntags:\n  - a\n  - it's\n---\n");
}

#[test]
fn date_shapes_and_quotes() {
    // [WB-18] 書ける日付は実在する日付と日時だけ。
    for ok in [
        "2026-11-03",
        "2024-02-29",
        "2026-11-03T09:00",
        "2026-11-03T09:00:30",
    ] {
        assert!(is_date(ok), "{ok}");
    }
    for bad in [
        "",
        "2026-11-3",
        "2025-02-29",
        "2026-11-03T25:00",
        "2026-11-03 09:00",
    ] {
        assert!(!is_date(bad), "{bad}");
    }
    let date = |src: &str| {
        let e = Edit {
            key: "due".into(),
            value: NewValue::Date("2026-11-03".into()),
        };
        String::from_utf8(apply(src.as_bytes(), &[e]).unwrap()).unwrap()
    };
    // 空の一重引用符は囲まない。行末のコメントと CRLF は保つ。
    assert_eq!(date("---\ndue: ''\n---\n"), "---\ndue: 2026-11-03\n---\n");
    assert_eq!(
        date("---\r\ndue: 2026-10-05 # c\r\n---\r\n"),
        "---\r\ndue: 2026-11-03 # c\r\n---\r\n"
    );
    // 日付の列の文字の値(元が素の文字)も囲まない。
    assert_eq!(
        date("---\ndue: someday\n---\n"),
        "---\ndue: 2026-11-03\n---\n"
    );
}

fn ed(key: &str, value: NewValue) -> Edit {
    Edit {
        key: key.into(),
        value,
    }
}

#[test]
fn prepend_same_key_twice_keeps_one_line_with_later_value() {
    // [WB-3] 同じキーを2回書けば1行で、あとの値。編集が無ければ元のまま。
    let out = apply(
        b"body\n",
        &[
            ed("status", NewValue::Str("a".into())),
            ed("k", NewValue::Int(1)),
            ed("status", NewValue::Str("b".into())),
        ],
    )
    .unwrap();
    assert_eq!(out, b"---\nstatus: b\nk: 1\n---\nbody\n");
    assert_eq!(apply(b"body\n", &[]).unwrap(), b"body\n");
}

#[test]
fn prepend_quotes_yaml11_bool_key() {
    // [WB-3][WB-7] 足す道でも、YAML 1.1 の真偽値の語のキーは囲んで書き、読み直せる。
    let out = apply(b"body\n", &[ed("n", NewValue::Int(3))]).unwrap();
    assert_eq!(out, b"---\n\"n\": 3\n---\nbody\n");
}

#[test]
fn prepend_refuses_newline_and_bad_date() {
    // [WB-3] 足す道でも、改行を含む値と日付でない Date は書かない。
    let r = apply(b"body\n", &[ed("s", NewValue::Str("a\nb".into()))]);
    assert_eq!(r, Err(EditError::Newline));
    let r = apply(b"body\n", &[ed("due", NewValue::Date("x".into()))]);
    assert!(matches!(r, Err(EditError::NotEditable(_))), "{r:?}");
}

#[test]
fn prepend_misquote_is_caught_by_verify() {
    // [WB-6] 足す道でも、クオートを落とした文字列(`true` → 真偽値)は読み直しの検査で落ちる。
    let r = apply_inner(b"body\n", &[ed("s", NewValue::Str("true".into()))], true);
    assert!(matches!(r, Err(EditError::Verify(_))), "{r:?}");
}

#[test]
fn verify_prepended_checks_body_keys_and_delimiters() {
    // [WB-6] 足したあとの検査: 本文の変化・知らないキー・閉じの行の違い・読めない結果を落とす。
    let edits = [ed("status", NewValue::Str("todo".into()))];
    let ok = b"---\nstatus: todo\n---\nbody\n";
    assert_eq!(verify_prepended(b"body\n", ok, &edits, b"\n"), Ok(()));
    let bad: [&[u8]; 5] = [
        b"---\nstatus: todo\n---\nbodx\n",
        b"---\nstatus: todo\nx: 1\n---\nbody\n",
        b"---\nstatus: done\n---\nbody\n",
        b"---\nstatus: todo\n--- \nbody\n",
        b"---\nstatus: todo\nbody\n",
    ];
    for out in bad {
        let r = verify_prepended(b"body\n", out, &edits, b"\n");
        assert!(
            matches!(r, Err(EditError::Verify(_))),
            "{:?}: {r:?}",
            String::from_utf8_lossy(out)
        );
    }
    // 改行コードが合わない(LF のはずが CRLF)。
    let crlf = b"---\r\nstatus: todo\r\n---\r\nbody\n";
    assert!(verify_prepended(b"body\n", crlf, &edits, b"\n").is_err());
}

#[test]
fn fill_empty_same_key_twice_misquote_and_no_edits() {
    // [WB-3] 空のフロントマターでも、同じキーを2回書けば1行であとの値。編集が無ければ元のまま。
    let out = apply(
        b"---\n---\nbody\n",
        &[
            ed("status", NewValue::Str("a".into())),
            ed("k", NewValue::Int(1)),
            ed("status", NewValue::Str("b".into())),
        ],
    )
    .unwrap();
    assert_eq!(out, b"---\nstatus: b\nk: 1\n---\nbody\n");
    assert_eq!(
        apply(b"---\n---\nbody\n", &[]).unwrap(),
        b"---\n---\nbody\n"
    );
    // [WB-3] 改行を含む値は書かない。
    let r = apply(b"---\n---\n", &[ed("s", NewValue::Str("a\nb".into()))]);
    assert_eq!(r, Err(EditError::Newline));
    // [WB-6] クオートを落とした文字列は読み直しの検査で落ちる。
    let r = apply_inner(
        b"---\n---\nbody\n",
        &[ed("s", NewValue::Str("true".into()))],
        true,
    );
    assert!(matches!(r, Err(EditError::Verify(_))), "{r:?}");
}

#[test]
fn verify_filled_checks_outside_bytes_and_keys() {
    // [WB-6] 空のフロントマターに足したあとの検査: 開きの行・閉じの行・本文の変化、知らないキー、違う値、
    // 読めない結果を落とす。
    let original = b"--- \n---\t\nbody\n";
    let at = 5;
    let edits = [ed("status", NewValue::Str("todo".into()))];
    let ok = b"--- \nstatus: todo\n---\t\nbody\n";
    assert_eq!(verify_filled(original, at, ok, &edits), Ok(()));
    let bad: [&[u8]; 6] = [
        b"---\nstatus: todo\n---\t\nbody\n",
        b"--- \nstatus: todo\n---\nbody\n",
        b"--- \nstatus: todo\n---\t\nbodx\n",
        b"--- \nstatus: todo\nx: 1\n---\t\nbody\n",
        b"--- \nstatus: done\n---\t\nbody\n",
        b"--- \nstatus: todo\nbody\n",
    ];
    for out in bad {
        let r = verify_filled(original, at, out, &edits);
        assert!(
            matches!(r, Err(EditError::Verify(_))),
            "{:?}: {r:?}",
            String::from_utf8_lossy(out)
        );
    }
}
