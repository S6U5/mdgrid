//! お気に入りのワークスペース(workspace.favorite-*): `workspaces.toml` の最上位の `favorites` の読み書き。
//! 期待値は specs/requirements/workspace.md の受け入れの例から取った(印の根は一時フォルダの絶対パスで書く。
//! `~/…` の形はホームを変えた本物の実行ファイルの試験 tests/test_switch_e2e.rs で見る)。

use super::*;
use crate::workspace::{self, WsTable};

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-fav-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::canonicalize(&d).unwrap()
}

/// 設定のフォルダ `cfg` に 趣味(books)を書き、印のワークスペース lab(名前の無い印)を作る。
fn setup(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let d = tmp(name);
    let cfg = d.join("config");
    std::fs::create_dir_all(&cfg).unwrap();
    let books = d.join("books");
    std::fs::create_dir_all(&books).unwrap();
    workspace::add(
        &cfg,
        "趣味",
        WsTable {
            name: "books".into(),
            path: books,
            ..WsTable::default()
        },
    )
    .unwrap();
    let lab = d.join("lab");
    std::fs::create_dir_all(lab.join(MARKER_DIR)).unwrap();
    std::fs::write(lab.join(MARKER_DIR).join(MARKER_FILE), "# 印\n").unwrap();
    (d, cfg, lab)
}

#[test]
fn test_favorites_written_at_top_in_order() {
    // [utest -> req~workspace.favorite-store~2] 「趣味」と印の lab をお気に入りにする → workspaces.toml の最上位に入れた順の favorites、lab の印のバイトは同じ。
    let (_d, cfg, lab) = setup("store");
    let marker = lab.join(MARKER_DIR).join(MARKER_FILE);
    let before = std::fs::read(&marker).unwrap();
    set(&cfg, "趣味", true).unwrap();
    let lab_entry = marker_entry(&lab);
    set(&cfg, &lab_entry, true).unwrap();
    let text = std::fs::read_to_string(cfg.join(FILE_NAME)).unwrap();
    let want = format!(
        "favorites = [\"趣味\", {}]\n",
        toml::Value::String(lab_entry.clone())
    );
    assert!(text.starts_with(&want), "{text}");
    assert_eq!(std::fs::read(&marker).unwrap(), before, "印は書かない");
    // ワークスペースの区画はそのまま読める。
    let (list, warns) = workspace::load(&cfg);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(list.len(), 1);
    assert_eq!(entries(&cfg), vec!["趣味".to_string(), lab_entry]);
}

#[test]
fn test_favorites_resolve_names_and_marker_roots() {
    // [utest -> req~workspace.favorite-store~2] 名前は workspaces.toml のワークスペース、パスは印の根(名前は印の name か、無ければフォルダの名前)。
    let (_d, cfg, lab) = setup("resolve");
    set(&cfg, "趣味", true).unwrap();
    set(&cfg, &marker_entry(&lab), true).unwrap();
    let (apps, _) = workspace::load(&cfg);
    let (favs, warns) = resolve(&cfg, &apps);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(
        favs,
        vec![
            Favorite::App("趣味".into()),
            Favorite::Marker {
                root: lab.clone(),
                name: "lab".into()
            }
        ]
    );
}

#[test]
fn test_favorites_gone_marker_is_left_out_with_warning() {
    // [utest -> req~workspace.favorite-store~2] お気に入りの印の根が無くなった → 一覧から外して「workspaces.toml: favorites[2]: …」の警告。
    let (_d, cfg, lab) = setup("gone");
    set(&cfg, "趣味", true).unwrap();
    set(&cfg, &marker_entry(&lab), true).unwrap();
    std::fs::remove_dir_all(&lab).unwrap();
    let (apps, _) = workspace::load(&cfg);
    let (favs, warns) = resolve(&cfg, &apps);
    assert_eq!(favs, vec![Favorite::App("趣味".into())]);
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(
        warns[0].starts_with("workspaces.toml: favorites[2]: "),
        "{warns:?}"
    );
}

#[test]
fn test_favorites_toggle_off_and_keep_hand_written_text() {
    // [utest -> req~workspace.favorite-mark~2] もう一度で外れる。手で書いたコメントとワークスペースの区画は残し、並びが空なら favorites の行を消す。
    let (_d, cfg, _lab) = setup("off");
    let path = cfg.join(FILE_NAME);
    let hand = format!("# 手で書いた\n{}", std::fs::read_to_string(&path).unwrap());
    std::fs::write(&path, &hand).unwrap();
    set(&cfg, "趣味", true).unwrap();
    assert!(contains(&entries(&cfg), "趣味"));
    set(&cfg, "趣味", false).unwrap();
    assert!(entries(&cfg).is_empty());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains("favorites"), "{text}");
    assert!(text.contains("# 手で書いた\n"), "{text}");
}

#[test]
fn test_favorites_survive_workspace_rewrite() {
    // [utest -> req~workspace.favorite-store~2] ワークスペースの書き換え(表を足す)でも favorites は残る。
    let (d, cfg, _lab) = setup("keep");
    set(&cfg, "趣味", true).unwrap();
    let tasks = d.join("tasks");
    std::fs::create_dir_all(&tasks).unwrap();
    workspace::add(
        &cfg,
        "work",
        WsTable {
            name: "tasks".into(),
            path: tasks,
            ..WsTable::default()
        },
    )
    .unwrap();
    assert_eq!(entries(&cfg), vec!["趣味".to_string()]);
    assert_eq!(workspace::load(&cfg).0.len(), 2);
}

#[test]
fn test_favorites_multiline_array_is_replaced() {
    // 手で複数行に書いた favorites も、1行に置き換える(ほかの行は残す)。
    let d = tmp("multi");
    let text = "favorites = [\n  \"a\",\n  \"b\",\n]\n\n[[workspace]]\nname = \"a\"\n";
    std::fs::write(d.join(FILE_NAME), text).unwrap();
    set(&d, "c", true).unwrap();
    let out = std::fs::read_to_string(d.join(FILE_NAME)).unwrap();
    assert!(
        out.starts_with("favorites = [\"a\", \"b\", \"c\"]\n\n[[workspace]]"),
        "{out}"
    );
}

#[test]
fn test_favorites_broken_toml_is_not_written() {
    // 読めない workspaces.toml は書き換えない(ほかの行を消さないため)。
    let d = tmp("broken");
    std::fs::write(d.join(FILE_NAME), "[[workspace]\n").unwrap();
    assert!(set(&d, "a", true).is_err());
    assert_eq!(
        std::fs::read_to_string(d.join(FILE_NAME)).unwrap(),
        "[[workspace]\n"
    );
}
