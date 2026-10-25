//! 木の画面の、画面に依存しない部分(TR-2〜TR-5)。値は作ったもの。

use super::*;
use std::collections::HashSet;

fn e(path: &str, order: &str, name: &str) -> Entry {
    Entry {
        path: split(path, DEFAULT_SEPARATOR),
        order: order.to_string(),
        name: name.to_string(),
    }
}

/// 見せるフォルダを「名前(数)」で、深さの分だけ字下げして並べる。
fn outline(t: &Tree, folded: &HashSet<String>) -> Vec<String> {
    t.shown(folded)
        .into_iter()
        .map(|f| {
            let f = &t.folders[f];
            format!("{}{}({})", "  ".repeat(f.depth), f.name, f.count)
        })
        .collect()
}

#[test]
fn test_tr_2_split_by_separator() {
    // [TR-2] 区切り(省略は ` > `)で切った段が置き場の階層。前後の空白と空の段は捨てる。
    assert_eq!(split("bar > 開発 > Web", " > "), vec!["bar", "開発", "Web"]);
    assert_eq!(split("a/b", "/"), vec!["a", "b"]);
    assert_eq!(split("  ", " > "), Vec::<String>::new());
    assert_eq!(split("a > b", ""), Vec::<String>::new());
}

#[test]
fn test_tr_3_tree_counts_and_none() {
    // [TR-3] 途中の階層もフォルダにし、下の階層までの行の数を添え、空の行は根の「(なし)」(最後)。
    let rows = vec![
        e("bar > 開発", "10", "a"),
        e("bar > 開発", "20", "b"),
        e("bar > 開発 > Web", "", "c"),
        e("other", "", "d"),
        e("", "", "f"),
    ];
    let t = build(&rows, "(なし)");
    assert_eq!(
        outline(&t, &HashSet::new()),
        vec!["bar(3)", "  開発(3)", "    Web(1)", "other(1)", "(なし)(1)"]
    );
    // 開発 を閉じる → Web が隠れ、数は3のまま。
    let dev = t
        .folders
        .iter()
        .find(|f| f.name == "開発")
        .unwrap()
        .key
        .clone();
    let folded: HashSet<String> = [dev].into_iter().collect();
    assert_eq!(
        outline(&t, &folded),
        vec!["bar(3)", "  開発(3)", "other(1)", "(なし)(1)"]
    );
}

#[test]
fn test_tr_3_same_name_under_other_parents() {
    // [TR-3] 同じ名前のフォルダは、親が違えば別のフォルダ。フォルダは名前の順。
    let rows = vec![
        e("zeta > Web", "", "a"),
        e("alpha > Web", "", "b"),
        e("alpha > Docs", "", "c"),
        e("Beta", "", "d"),
    ];
    let t = build(&rows, "(なし)");
    assert_eq!(
        outline(&t, &HashSet::new()),
        vec![
            "alpha(2)",
            "  Docs(1)",
            "  Web(1)",
            "Beta(1)",
            "zeta(1)",
            "  Web(1)"
        ]
    );
    let webs: Vec<&Folder> = t.folders.iter().filter(|f| f.name == "Web").collect();
    assert_eq!(webs.len(), 2);
    assert_ne!(webs[0].key, webs[1].key);
    // 途中の階層だけで行の無いフォルダも作る(a > b > c だけで a と b)。
    let t = build(&[e("a > b > c", "", "x")], "(なし)");
    assert_eq!(
        outline(&t, &HashSet::new()),
        vec!["a(1)", "  b(1)", "    c(1)"]
    );
    assert!(t.folders[t.find(&t.folders[0].key).unwrap()]
        .rows
        .is_empty());
}

#[test]
fn test_tr_4_rows_by_order_then_name() {
    // [TR-4] フォルダの直下の行は order の小さい順。数でない値と空は後ろ、同じ値はノートの名前の順。
    let rows = vec![
        e("開発", "", "empty"),
        e("開発", "20", "twenty"),
        e("開発", "abc", "abc"),
        e("開発", "10", "ten-b"),
        e("開発", "10", "ten-a"),
        e("開発", "2.5", "small"),
    ];
    let t = build(&rows, "(なし)");
    let f = &t.folders[t.roots[0]];
    let names: Vec<&str> = f.rows.iter().map(|&i| rows[i].name.as_str()).collect();
    assert_eq!(
        names,
        vec!["small", "ten-a", "ten-b", "twenty", "abc", "empty"]
    );
    // order が無い(全部空)ならノートの名前の順。
    let rows = vec![e("x", "", "b"), e("x", "", "a"), e("x", "", "C")];
    let t = build(&rows, "(なし)");
    let names: Vec<&str> = t.folders[0]
        .rows
        .iter()
        .map(|&i| rows[i].name.as_str())
        .collect();
    assert_eq!(names, vec!["a", "b", "C"]);
}

#[test]
fn test_tr_5_web_url_only_http() {
    // [TR-5] http・https だけを開いてよい URL とし、ほかの形は開かない。
    assert_eq!(web_url("https://example.com"), Some("https://example.com"));
    assert_eq!(
        web_url("  http://example.com/a?b=1 "),
        Some("http://example.com/a?b=1")
    );
    assert_eq!(web_url("HTTPS://EXAMPLE.COM"), Some("HTTPS://EXAMPLE.COM"));
    for bad in [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "https://",
        "ftp://example.com",
        "https://exa mple.com",
        "https://example.com\n--x",
        "example.com",
        "",
    ] {
        assert_eq!(web_url(bad), None, "{bad:?}");
    }
}

/// 印のある一時フォルダ(根)と、その下の表のフォルダ `bm`。
struct Root(std::path::PathBuf);

impl Root {
    fn new(tag: &str, marker: &str) -> Root {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-tree-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(dir.join("bm")).unwrap();
        std::fs::create_dir_all(dir.join(".mdgrid")).unwrap();
        std::fs::write(dir.join(".mdgrid/workspace.toml"), marker).unwrap();
        Root(dir)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_tr_2_marker_tree_reads_path_order_and_default_separator() {
    // [TR-2] 印の [[table]] の tree = { path, order }。区切りの省略は ` > `。
    let r = Root::new(
        "read",
        "[[table]]\nname = \"BM\"\npath = \"bm\"\ntree = { path = \"folder\", order = \"order\" }\n",
    );
    let (spec, warns) = crate::workspace::marker_tree(&r.0.join("bm"));
    assert!(warns.is_empty(), "{warns:?}");
    let (spec, at) = spec.expect("tree");
    assert_eq!(
        spec,
        TreeSpec {
            path: "folder".into(),
            separator: " > ".into(),
            order: Some("order".into()),
        }
    );
    assert_eq!(at, "table[1].tree");
    // 範囲を決めるときの印の読みでも、tree は知らない項目にならない。
    let (_, warns) = crate::workspace::read_marker(&r.0);
    assert!(warns.is_empty(), "{warns:?}");
    // 別の表(tree の無い表)には木が無い。
    std::fs::create_dir_all(r.0.join("other")).unwrap();
    let (spec, _) = crate::workspace::marker_tree(&r.0.join("other"));
    assert!(spec.is_none());
}

#[test]
fn test_tr_2_empty_separator_warns_and_no_tree() {
    // [TR-2] 区切りが空の文字なら、CLI-3 の形の警告1行で、木を出さない。
    let r = Root::new(
        "sep",
        "[[table]]\npath = \"bm\"\ntree = { path = \"folder\", separator = \"\" }\n",
    );
    let (spec, warns) = crate::workspace::marker_tree(&r.0.join("bm"));
    assert!(spec.is_none());
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(
        warns[0].starts_with(".mdgrid/workspace.toml: table[1].tree.separator: "),
        "{warns:?}"
    );
    // path が無い・表でない も警告して出さない。
    for (bad, at) in [
        ("tree = { order = \"order\" }", "table[1].tree.path: "),
        ("tree = \"folder\"", "table[1].tree: "),
    ] {
        let r = Root::new("bad", &format!("[[table]]\npath = \"bm\"\n{bad}\n"));
        let (spec, warns) = crate::workspace::marker_tree(&r.0.join("bm"));
        assert!(spec.is_none(), "{bad}");
        assert_eq!(warns.len(), 1, "{warns:?}");
        assert!(warns[0].contains(at), "{warns:?}");
    }
}
