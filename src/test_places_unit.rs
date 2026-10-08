//! places.toml の読み書き(CLI-18)と一覧の並び(CLI-19)。

use super::*;
use crate::i18n::{scoped, Lang};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mdgrid-places-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn place(name: &str, group: &str, path: &str) -> Place {
    Place {
        name: name.into(),
        group: group.into(),
        path: PathBuf::from(path),
        view: None,
    }
}

#[test]
fn test_cli_18_places_round_trip() {
    // [CLI-18] 登録して読み戻す。同じ名前は置き換え、ほかは後ろに足す。ビューも残る。
    let d = tmp("round");
    save(&d, place("タスク", "仕事", "/n/Tasks")).unwrap();
    let mut b = place("本", "趣味", "/n/Books");
    b.view = Some("読む".into());
    save(&d, b.clone()).unwrap();
    save(&d, place("タスク", "仕事", "/n/Tasks2")).unwrap();
    let (got, warns) = load(&d);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(got, vec![place("タスク", "仕事", "/n/Tasks2"), b]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_cli_18_missing_file_is_empty() {
    let d = tmp("none");
    assert_eq!(load(&d), (Vec::new(), Vec::new()));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_cli_18_bad_rows_warn_and_skip() {
    // [CLI-18] path の無い行・名前の重なり・表でない値は、警告にして飛ばす。ほかは読む。
    let _l = scoped(Lang::Ja);
    let text = r#"
[[place]]
name = "a"
path = "/x"

[[place]]
name = "no path"

[[place]]
name = "a"
path = "/y"

[[place]]
name = "b"
group = "g"
path = "/z"
view = ""
"#;
    let (got, warns) = parse(text);
    assert_eq!(got, vec![place("a", "", "/x"), place("b", "g", "/z")]);
    assert_eq!(warns.len(), 2, "{warns:?}");
    assert!(warns[0].contains("2 番目"), "{warns:?}");
    assert!(warns[1].contains("3 番目"), "{warns:?}");
}

#[test]
fn test_cli_18_broken_toml_warns() {
    let _l = scoped(Lang::Ja);
    let (got, warns) = parse("[[place]\nname=");
    assert!(got.is_empty());
    assert_eq!(warns.len(), 1);
    assert!(warns[0].contains("places.toml が読めない"), "{warns:?}");
}

#[test]
fn test_cli_18_tilde_is_home() {
    // [CLI-18] 先頭の `~` はホームのフォルダ。途中の `~` はそのまま。
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return;
    };
    let (got, _) = parse(
        "[[place]]\nname = \"h\"\npath = \"~/notes\"\n[[place]]\nname = \"m\"\npath = \"a/~/b\"\n",
    );
    assert_eq!(got[0].path, home.join("notes"));
    assert_eq!(got[1].path, PathBuf::from("a/~/b"));
}

#[test]
fn test_cli_19_grouped_order() {
    // [CLI-19] 分類は最初に出た順にまとめ、分類の中は places.toml の順。
    let ps = vec![
        place("a", "仕事", "/a"),
        place("b", "趣味", "/b"),
        place("c", "仕事", "/c"),
        place("d", "", "/d"),
    ];
    assert_eq!(grouped(&ps), vec![0, 2, 1, 3]);
    assert_eq!(groups(&ps), vec!["仕事".to_string(), "趣味".to_string()]);
}
