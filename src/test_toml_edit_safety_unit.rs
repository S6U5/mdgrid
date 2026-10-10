//! 文字のまま書き換えるときの安全(WS-1・CLI-18): コメント付きの見出し・末尾の別の表・1行の形でも、
//! ほかの項目を壊さない。

use crate::places;
use crate::workspace::{self, WsTable};
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-tes-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::canonicalize(&d).unwrap()
}

fn t(name: &str, path: &str) -> WsTable {
    WsTable {
        name: name.into(),
        path: PathBuf::from(path),
        view: None,
        profile: Default::default(),
    }
}

#[test]
fn test_ws_1_header_with_comment_keeps_neighbour() {
    // [WS-1] `[[workspace]] # second` も見出し。a を消しても b は残る。a に足しても b に入らない。
    let d = tmp("hdr");
    let text = "[[workspace]]\nname = \"a\"\n[[workspace.table]]\npath = \"/p\"\n\n[[workspace]] # second\nname = \"b\"\n[[workspace.table]]\npath = \"/q\"\n";
    std::fs::write(d.join("workspaces.toml"), text).unwrap();
    workspace::add(&d, "a", t("r", "/r")).unwrap();
    let (list, _) = workspace::load(&d);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].tables.len(), 2, "a に足す");
    assert_eq!(list[1].tables.len(), 1, "b はそのまま");
    assert!(workspace::remove(&d, "a", None).unwrap());
    let (list, _) = workspace::load(&d);
    assert_eq!(
        list.iter().map(|w| w.name.as_str()).collect::<Vec<_>>(),
        ["b"]
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_ws_1_trailing_table_and_inline_form_survive() {
    // [WS-1] 末尾の別の表([meta])は消さない。1行の形で書いたワークスペースも壊さない(読み直して違えば書き直す)。
    let d = tmp("tail");
    // パスは一時フォルダの下の絶対パス(Windows でも絶対)。TOML には引用して書く。
    let p = d.join("p");
    let q = |x: &std::path::Path| toml::Value::String(x.to_string_lossy().into_owned()).to_string();
    std::fs::write(
        d.join("workspaces.toml"),
        format!(
            "[[workspace]]\nname = \"a\"\n[[workspace.table]]\npath = {}\n\n[meta]\nnote = \"keep\"\n",
            q(&p)
        ),
    )
    .unwrap();
    assert!(workspace::remove(&d, "a", Some(&p)).unwrap());
    let out = std::fs::read_to_string(d.join("workspaces.toml")).unwrap();
    assert!(out.contains("[meta]") && out.contains("keep"), "{out}");
    std::fs::write(
        d.join("workspaces.toml"),
        format!(
            "workspace = [{{ name = \"a\", table = [{{ path = {} }}] }}]\n",
            q(&p)
        ),
    )
    .unwrap();
    workspace::add(&d, "a", t("q", &d.join("q").to_string_lossy())).unwrap();
    let (list, w) = workspace::load(&d);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(list[0].tables.len(), 2);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_cli_18_place_header_with_comment_keeps_neighbour() {
    // [CLI-18] `[[place]] # work` も見出し。A を置き換えても B は残る。
    let text =
        "[[place]]\nname = \"A\"\npath = \"/a\"\n\n[[place]] # work\nname = \"B\"\npath = \"/b\"\n";
    let out = places::upsert(
        text,
        &places::Place {
            name: "A".into(),
            group: String::new(),
            path: PathBuf::from("/a2"),
            view: None,
        },
    );
    let (ps, _) = places::parse(&out);
    assert_eq!(
        ps.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["A", "B"],
        "{out}"
    );
    assert_eq!(ps[0].path, PathBuf::from("/a2"));
}

#[test]
fn test_ws_7_marker_paths_stay_inside() {
    // [WS-7] 印の表は根の中だけ。`../x` や根の外の絶対パスは警告して使わない。
    let d = tmp("marker");
    std::fs::create_dir_all(d.join("root/tasks")).unwrap();
    std::fs::create_dir_all(d.join("root/.mdgrid")).unwrap();
    std::fs::create_dir_all(d.join("outside")).unwrap();
    std::fs::write(
        d.join("root/.mdgrid/workspace.toml"),
        "[[table]]\npath = \"tasks\"\n[[table]]\npath = \"../outside\"\n",
    )
    .unwrap();
    let (w, warns) = workspace::read_marker(&d.join("root"));
    assert_eq!(w.tables.len(), 1, "{:?}", w.tables);
    assert_eq!(warns.len(), 1, "{warns:?}");
    let _ = std::fs::remove_dir_all(&d);
}
