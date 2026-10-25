//! TR-2: 印の `[table.base]`(WS-9。`path` の無い表)にも `tree` が効く。値は作ったもの。

use super::*;

/// 印のある一時フォルダ(根)と、ノートを置く `items`。
struct Root(PathBuf);

impl Root {
    fn new(tag: &str, marker: &str) -> Root {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-tree-base-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join("items")).unwrap();
        std::fs::create_dir_all(dir.join(MARKER_DIR)).unwrap();
        std::fs::write(dir.join(MARKER_DIR).join(MARKER_FILE), marker).unwrap();
        Root(dir)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const BASE_ONLY: &str = "open = \"ブックマーク\"\n\n[[table]]\nname = \"ブックマーク\"\n\
tree = { path = \"folder\", separator = \" > \", order = \"order\" }\n\n[table.base]\n\
filters = 'file.inFolder(\"items\")'\n";

#[test]
fn test_tr_2_base_only_table_has_tree() {
    // [TR-2] `path` の無い `[table.base]` の表を開く(根を開いて open で選ぶ)→ 木の設定が読める。
    let r = Root::new("plain", BASE_ONLY);
    // 印の open が選ぶ表のパス(ファイルは無い。read_marker の道筋のまま)。
    let (t, _) = marker_open(&r.0);
    let opened = t.expect("open の表").path;
    let (spec, warns) = marker_tree(&opened);
    assert!(warns.is_empty(), "{warns:?}");
    let (spec, at) = spec.expect("tree");
    assert_eq!(spec.path, "folder");
    assert_eq!(spec.order.as_deref(), Some("order"));
    assert_eq!(at, "table[1].tree");
}

#[test]
fn test_tr_2_base_only_table_by_other_spelling_of_root() {
    // [TR-2] 開いたパスが実体のパスと違う書き方(`items/..` を挟む・相対の `.` から)でも、同じ表なら木の設定が読める
    // (ファイルの無い `.base` は実体のパスに直せないので、フォルダの側を直して比べる)。
    let r = Root::new("spelling", BASE_ONLY);
    let opened = r.0.join("items").join("..").join("ブックマーク.base");
    let (spec, warns) = marker_tree(&opened);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(spec.expect("tree").0.path, "folder");
    // path で書いた表は今までどおり。
    let r = Root::new(
        "path",
        "[[table]]\nname = \"BM\"\npath = \"items\"\ntree = { path = \"folder\" }\n",
    );
    let (spec, _) = marker_tree(&r.0.join("items").join("..").join("items"));
    assert_eq!(spec.expect("tree").0.separator, " > ");
}
