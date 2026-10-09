//! 登録した表の一覧の桁(CLI-19): 「分類 › 名前」の欄をそろえ、パスを同じ桁から出す。

use super::places::labels;
use super::width::width;
use mdgrid::places::Place;
use std::path::PathBuf;

fn place(name: &str, group: &str, path: &str, view: Option<&str>) -> Place {
    Place {
        name: name.into(),
        group: group.into(),
        path: PathBuf::from(path),
        view: view.map(str::to_string),
    }
}

#[test]
fn test_cli_19_list_paths_aligned() {
    // [CLI-19] 名前の長さが違っても、パスは同じ桁から。全角の名前も幅で数える。ビューはパスのあと。
    let list = [
        place("タスク", "仕事", "/n/tasks", None),
        place("A very long table name", "", "/n/long", Some("Open")),
        place("b", "趣味", "/n/b", None),
    ];
    let rows = labels(&list.iter().collect::<Vec<_>>());
    let col = |row: &str, path: &str| width(&row[..row.find(path).unwrap()]);
    assert_eq!(col(&rows[0], "/n/tasks"), col(&rows[1], "/n/long"));
    assert_eq!(col(&rows[1], "/n/long"), col(&rows[2], "/n/b"));
    assert!(rows[0].starts_with("仕事 › タスク"), "{}", rows[0]);
    assert!(rows[1].ends_with("/n/long · Open"), "{}", rows[1]);
}
