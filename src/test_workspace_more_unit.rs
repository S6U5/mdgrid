//! ワークスペースの核の続き(WS-1・WS-7): 読めないファイルを書き直さない、`~` と相対のパス、印の誤りの警告、名前。

use super::*;

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-wsm-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::canonicalize(&d).unwrap()
}

fn table(path: PathBuf) -> WsTable {
    WsTable {
        name: "T".into(),
        path,
        view: None,
    }
}

#[test]
fn test_ws_1_broken_file_is_not_rewritten() {
    // [WS-1] 読めない workspaces.toml(壊れた TOML・読めない行)には足さず・外さず、ファイルはそのまま。
    let d = tmp("broken");
    for text in [
        "[[workspace]\nname = \"A\"\n",
        "[[workspace]]\n\n[[workspace]]\nname = \"B\"\n",
    ] {
        std::fs::write(d.join(FILE_NAME), text).unwrap();
        let e = add(&d, "New", table(d.clone())).unwrap_err();
        assert!(e.to_string().contains("workspaces.toml"), "{e}");
        assert!(remove(&d, "B", None).is_err());
        assert_eq!(std::fs::read_to_string(d.join(FILE_NAME)).unwrap(), text);
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_ws_1_home_and_relative_paths() {
    // [WS-1] ホームの下は `~/…` で書き直す(HOME の無い環境では確かめない)。相対の path は設定のフォルダから読む。
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        let list = vec![Workspace {
            name: "W".into(),
            tables: vec![table(home.join("notes/tasks"))],
        }];
        assert!(
            to_toml(&list).contains("path = \"~/notes/tasks\""),
            "{}",
            to_toml(&list)
        );
    }
    let d = tmp("rel");
    std::fs::write(
        d.join(FILE_NAME),
        "[[workspace]]\nname = \"W\"\n[[workspace.table]]\npath = \"notes/v1.2\"\n",
    )
    .unwrap();
    let (got, warns) = load(&d);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got[0].tables[0].path, d.join("notes/v1.2"));
    assert_eq!(
        got[0].tables[0].name, "v1.2",
        "フォルダの名前は拡張子のように切らない"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_ws_7_broken_marker_warns() {
    // [WS-7] 印が読めなければ、自動の表で続けつつ、範囲に警告を残す(黙って使わない)。
    let d = tmp("marker");
    std::fs::create_dir_all(d.join("tasks")).unwrap();
    std::fs::write(d.join("tasks/a.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::create_dir_all(d.join(MARKER_DIR)).unwrap();
    std::fs::write(d.join(MARKER_DIR).join(MARKER_FILE), "name = \n").unwrap();
    let s = resolve(&d.join("tasks"), None, &[], &[]).unwrap().unwrap();
    assert_eq!(s.source, Source::Marker);
    assert_eq!(s.warnings.len(), 1, "{:?}", s.warnings);
    let _ = std::fs::remove_dir_all(&d);
}
