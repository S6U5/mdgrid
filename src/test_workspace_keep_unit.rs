//! workspaces.toml を書き換えても、手で書いたコメント・空行・`~`・並びを残す(WS-1)。

use super::*;

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-wsk-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::canonicalize(&d).unwrap()
}

const HAND: &str = "# 仕事の表\n[[workspace]]\nname = \"Work\"\n\n[[workspace.table]]\n# いちばん使う\nname = \"Tasks\"\npath = \"tasks\"\n\n# 趣味\n[[workspace]]\nname = \"Hobby\"\n\n[[workspace.table]]\nname = \"Books\"\npath = \"~/notes/books\"\n";

fn table(name: &str, path: PathBuf) -> WsTable {
    WsTable {
        name: name.into(),
        path,
        view: None,
    }
}

#[test]
fn test_ws_1_add_keeps_hand_written_text() {
    // [WS-1] Work に表を足しても、コメント・空行・~ のパス・ほかのワークスペースはそのまま。読み直すと Work に2つ。
    let d = tmp("add");
    std::fs::write(d.join(FILE_NAME), HAND).unwrap();
    add(&d, "Work", table("Projects", d.join("projects"))).unwrap();
    let out = std::fs::read_to_string(d.join(FILE_NAME)).unwrap();
    for keep in [
        "# 仕事の表\n",
        "# いちばん使う\n",
        "# 趣味\n[[workspace]]\nname = \"Hobby\"",
        "path = \"~/notes/books\"",
    ] {
        assert!(out.contains(keep), "{keep:?} が消えた:\n{out}");
    }
    let (list, w) = load(&d);
    assert!(w.is_empty(), "{w:?}");
    let names: Vec<&str> = list[0].tables.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Tasks", "Projects"]);
    assert!(
        out.find("name = \"Projects\"").unwrap() < out.find("# 趣味").unwrap(),
        "Work の区画の中に足す:\n{out}"
    );
    // 新しいワークスペースは末尾に。
    add(&d, "New", table("X", d.join("x"))).unwrap();
    let (list, _) = load(&d);
    assert_eq!(
        list.iter().map(|w| w.name.as_str()).collect::<Vec<_>>(),
        ["Work", "Hobby", "New"]
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_ws_1_replace_and_remove_only_that_part() {
    // [WS-1] 同じ表の置き換えと、表を外す・ワークスペースを消すは、その区画だけ。ほかのコメントは残る。
    let d = tmp("rm");
    std::fs::write(d.join(FILE_NAME), HAND).unwrap();
    add(&d, "Work", table("Todo", d.join("tasks"))).unwrap();
    let out = std::fs::read_to_string(d.join(FILE_NAME)).unwrap();
    assert!(
        out.contains("name = \"Todo\"") && !out.contains("name = \"Tasks\""),
        "{out}"
    );
    assert!(
        out.contains("# 仕事の表") && out.contains("# 趣味"),
        "{out}"
    );
    assert!(remove(&d, "Work", Some(&d.join("tasks"))).unwrap());
    let (list, _) = load(&d);
    assert!(list[0].tables.is_empty());
    assert!(remove(&d, "Hobby", None).unwrap());
    let out = std::fs::read_to_string(d.join(FILE_NAME)).unwrap();
    assert!(!out.contains("Hobby") && !out.contains("Books"), "{out}");
    assert!(out.contains("# 仕事の表"), "{out}");
    assert_eq!(load(&d).0.len(), 1);
    let _ = std::fs::remove_dir_all(&d);
}
