//! ワークスペースと表の範囲のプロファイル(WS-1・WS-7・SR-44): 読む・画面から書く(手のコメントを残す)。

use super::*;
use crate::profile::ThemeSpec;
use crate::theme::Theme;

fn tmp(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!(
        "mdgrid-wsprof-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn nord() -> Option<ThemeSpec> {
    Some(ThemeSpec::Named(Theme::Nord))
}

#[test]
fn test_ws_1_workspace_and_table_profiles() {
    // [WS-1] ワークスペースと表の項目に表のプロファイルを書ける。アプリ全体の項目は警告して無視。
    let text = "[[workspace]]\nname = \"P\"\n\n[workspace.look]\ntheme = \"nord\"\n\n[[workspace.table]]\nname = \"T\"\npath = \"/t\"\n\n[workspace.table.look]\ntheme = \"dracula\"\n\n[workspace.table.terminal]\ncolor = false\n";
    let (list, w) = parse(text);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].starts_with("workspaces.toml: workspace.table.terminal: "),
        "{w:?}"
    );
    assert_eq!(list[0].profile.look.theme, nord());
    assert_eq!(
        list[0].tables[0].profile.look.theme,
        Some(ThemeSpec::Named(Theme::Dracula))
    );
    // 書き直しても残る。
    let (back, w) = parse(&to_toml(&list));
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(back, list);
}

#[test]
fn test_ws_7_marker_profiles() {
    // [WS-7] 印の最上位にワークスペースのプロファイル、[[table]] の [table.look] に表のもの。範囲の層になる。
    let root = tmp("marker");
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::create_dir_all(root.join("projects")).unwrap();
    std::fs::write(root.join("tasks/a.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::write(root.join("projects/b.md"), "---\nx: 1\n---\n").unwrap();
    std::fs::create_dir_all(root.join(MARKER_DIR)).unwrap();
    std::fs::write(
        root.join(MARKER_DIR).join(MARKER_FILE),
        "name = \"N\"\n[look]\ntheme = \"nord\"\n[[table]]\npath = \"tasks\"\n[table.look]\ntheme = \"dracula\"\n[[table]]\npath = \"projects\"\n",
    )
    .unwrap();
    let scope = resolve(&root.join("tasks"), None, &[], &[])
        .unwrap()
        .unwrap();
    assert!(scope.warnings.is_empty(), "{:?}", scope.warnings);
    assert_eq!(scope.source, Source::Marker);
    let ws = scope.layer().unwrap();
    assert_eq!(ws.profile.look.theme, nord());
    assert!(
        ws.origin.label.contains(".mdgrid/workspace.toml"),
        "{}",
        ws.origin.label
    );
    let t = scope.table_layer(&root.join("tasks")).unwrap();
    assert_eq!(t.profile.look.theme, Some(ThemeSpec::Named(Theme::Dracula)));
    assert!(scope
        .table_layer(&root.join("projects"))
        .unwrap()
        .profile
        .is_empty());
}

#[test]
fn test_sr_44_detected_scope_has_no_workspace_layer() {
    // [SR-44] 検知した範囲にはワークスペースの範囲が無い。
    let root = tmp("detect");
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::write(root.join("tasks/a.md"), "---\nx: 1\n---\n").unwrap();
    let scope = resolve(&root.join("tasks"), None, &[], &[Detect::Vault])
        .unwrap()
        .unwrap();
    assert!(scope.layer().is_none() && scope.table_layer(&root.join("tasks")).is_none());
}

#[test]
fn test_sr_43_save_profile_keeps_comments() {
    // [SR-43] 画面からワークスペースのプロファイルを書くと、その区画の見た目だけを置き換え、手で書いたコメント・
    // ほかのワークスペース・表はそのまま。外すと消える。
    let dir = tmp("save");
    let text = "# my workspaces\n[[workspace]]\nname = \"P\"\n# keep me\nuse = \"old\"\n\n[workspace.look]\ntheme = \"gruvbox\"\n\n[[workspace.table]]\n# table note\nname = \"T\"\npath = \"/t\"\n\n[[workspace]]\nname = \"Q\"\n";
    std::fs::write(dir.join(FILE_NAME), text).unwrap();
    let mut p = Profile::default();
    p.look.theme = nord();
    save_profile(&dir, "P", &p).unwrap();
    let out = std::fs::read_to_string(dir.join(FILE_NAME)).unwrap();
    assert!(
        out.contains("# my workspaces")
            && out.contains("# keep me")
            && out.contains("# table note"),
        "{out}"
    );
    assert!(
        !out.contains("gruvbox") && !out.contains("use = \"old\""),
        "{out}"
    );
    let (list, w) = parse(&out);
    assert!(w.is_empty(), "{w:?}\n{out}");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].profile, p);
    assert_eq!(list[0].tables.len(), 1);
    save_profile(&dir, "P", &Profile::default()).unwrap();
    let (list, _) = load(&dir);
    assert!(list[0].profile.is_empty());
    assert_eq!(list[0].tables.len(), 1);
}

#[test]
fn test_ws_2_add_keeps_table_profile() {
    // [WS-2][SR-44] 同じ表を足し直しても、手で書いた表のプロファイルは残る。
    let dir = tmp("add");
    std::fs::write(
        dir.join(FILE_NAME),
        "[[workspace]]\nname = \"P\"\n\n[[workspace.table]]\nname = \"T\"\npath = \"/t\"\n\n[workspace.table.look]\ntheme = \"nord\"\n",
    )
    .unwrap();
    add(
        &dir,
        "P",
        WsTable {
            name: "T2".into(),
            path: PathBuf::from("/t"),
            ..WsTable::default()
        },
    )
    .unwrap();
    let (list, w) = load(&dir);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(list[0].tables[0].name, "T2");
    assert_eq!(list[0].tables[0].profile.look.theme, nord());
}
