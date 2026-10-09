//! ワークスペースの核(WS-1・WS-5・WS-6・WS-7)。

use super::*;

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d =
            std::env::temp_dir().join(format!("mdgrid-ws-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(std::fs::canonicalize(&d).unwrap())
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn mkdir(&self, rel: &str) {
        std::fs::create_dir_all(self.0.join(rel)).unwrap();
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn notes(t: &Tmp, dir: &str) {
    t.write(&format!("{dir}/a.md"), "---\nx: 1\n---\n");
}

fn names(ts: &[WsTable]) -> Vec<String> {
    ts.iter().map(|t| t.name.clone()).collect()
}

#[test]
fn test_ws_1_read_write_round_trip() {
    // [WS-1] workspaces.toml を読み書きする。表を足す・置き換える・外す・ワークスペースごと消す。
    let t = Tmp::new("rw");
    let cfg = t.0.join("config");
    notes(&t, "tasks");
    notes(&t, "projects");
    add(
        &cfg,
        "Product",
        WsTable {
            name: "Tasks".into(),
            path: t.0.join("tasks"),
            view: None,
        },
    )
    .unwrap();
    add(
        &cfg,
        "Product",
        WsTable {
            name: "Projects".into(),
            path: t.0.join("projects"),
            view: Some("Open".into()),
        },
    )
    .unwrap();
    let (list, warns) = load(&cfg);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(list.len(), 1);
    assert_eq!(names(&list[0].tables), ["Tasks", "Projects"]);
    assert_eq!(list[0].tables[1].view.as_deref(), Some("Open"));
    // 同じ表は置き換える。
    add(
        &cfg,
        "Product",
        WsTable {
            name: "Todo".into(),
            path: t.0.join("tasks"),
            view: None,
        },
    )
    .unwrap();
    assert_eq!(names(&load(&cfg).0[0].tables), ["Todo", "Projects"]);
    assert!(remove(&cfg, "Product", Some(&t.0.join("tasks"))).unwrap());
    assert_eq!(names(&load(&cfg).0[0].tables), ["Projects"]);
    assert!(
        !remove(&cfg, "Nothing", None).unwrap(),
        "無い名前は何も消さない"
    );
    assert!(remove(&cfg, "Product", None).unwrap());
    assert!(load(&cfg).0.is_empty());
}

#[test]
fn test_ws_1_bad_rows_warn() {
    // [WS-1] 読めない行は警告にして飛ばす(名前の無い・重なるワークスペース、path の無い表、壊れた TOML)。
    let text = r#"
[[workspace]]
name = "A"
[[workspace.table]]
path = "~/x"
[[workspace.table]]
name = "no path"

[[workspace]]
name = "A"

[[workspace]]
"#;
    let (list, warns) = parse(text);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].tables.len(), 1);
    assert_eq!(list[0].tables[0].name, "x", "名前が無ければパスの名前");
    assert_eq!(warns.len(), 3, "{warns:?}");
    let (l, w) = parse("[[workspace]\n");
    assert!(l.is_empty() && w.len() == 1);
}

#[test]
fn test_ws_7_marker_auto_and_listed_tables() {
    // [WS-7] 印の表を書かなければ、直下のノートのあるフォルダ(.base は入れない)。書けばその表だけ(相対は根から)。
    let t = Tmp::new("marker");
    notes(&t, "tasks");
    notes(&t, "projects/2026");
    t.mkdir("empty");
    t.write("Tasks.base", "views: []\n");
    t.write(".hidden/a.md", "x");
    let path = init(&t.0).unwrap();
    assert!(path.ends_with(".mdgrid/workspace.toml"));
    assert!(init(&t.0).is_err(), "もうあれば作らない");
    let (w, warns) = read_marker(&t.0);
    assert!(warns.is_empty(), "{warns:?}");
    assert_eq!(w.name, t.0.file_name().unwrap().to_string_lossy());
    assert_eq!(names(&w.tables), ["projects", "tasks"]);
    t.write(
        ".mdgrid/workspace.toml",
        "name = \"Product\"\n[[table]]\nname = \"T\"\npath = \"tasks\"\n",
    );
    let (w, _) = read_marker(&t.0);
    assert_eq!(w.name, "Product");
    assert_eq!(names(&w.tables), ["T"]);
    assert_eq!(w.tables[0].path, t.0.join("tasks"));
}

#[test]
fn test_ws_5_detect_vault_and_git() {
    // [WS-5] 上へたどって最初の保管庫かリポの根。設定で選んだものだけ。
    let t = Tmp::new("detect");
    t.mkdir("repo/.git");
    t.mkdir("repo/vault/.obsidian");
    notes(&t, "repo/vault/tasks");
    let start = t.0.join("repo/vault/tasks");
    assert_eq!(
        detect(&start, &[Detect::Vault, Detect::Git]),
        Some((t.0.join("repo/vault"), Detect::Vault))
    );
    assert_eq!(
        detect(&start, &[Detect::Git]),
        Some((t.0.join("repo"), Detect::Git))
    );
    assert_eq!(detect(&start, &[]), None);
}

#[test]
fn test_ws_6_resolve_order() {
    // [WS-6] -w → 印 → workspaces.toml → 検知 → None(登録した表)。
    let t = Tmp::new("order");
    t.mkdir("vault/.obsidian");
    notes(&t, "vault/tasks");
    notes(&t, "vault/projects");
    notes(&t, "vault/books");
    notes(&t, "elsewhere");
    let tasks = t.0.join("vault/tasks");
    let apps = vec![
        Workspace {
            name: "Work".into(),
            tables: vec![
                WsTable {
                    name: "Tasks".into(),
                    path: tasks.clone(),
                    view: None,
                },
                WsTable {
                    name: "Projects".into(),
                    path: t.0.join("vault/projects"),
                    view: None,
                },
            ],
        },
        Workspace {
            name: "Hobby".into(),
            tables: vec![WsTable {
                name: "Books".into(),
                path: t.0.join("vault/books"),
                view: None,
            }],
        },
    ];
    let modes = [Detect::Vault];
    // -w が先。
    let s = resolve(&tasks, Some("Hobby"), &apps, &modes)
        .unwrap()
        .unwrap();
    assert_eq!((s.name.as_str(), s.source), ("Hobby", Source::Chosen));
    assert!(resolve(&tasks, Some("Nope"), &apps, &modes).is_err());
    // 書いたワークスペースは検知より先。
    let s = resolve(&tasks, None, &apps, &modes).unwrap().unwrap();
    assert_eq!((s.name.as_str(), s.source), ("Work", Source::App));
    // 印は書いたワークスペースより先。
    t.write("vault/.mdgrid/workspace.toml", "name = \"Marked\"\n");
    let s = resolve(&tasks, None, &apps, &modes).unwrap().unwrap();
    assert_eq!((s.name.as_str(), s.source), ("Marked", Source::Marker));
    std::fs::remove_dir_all(t.0.join("vault/.mdgrid")).unwrap();
    // どこにも書いていない表は検知。
    let s = resolve(&t.0.join("vault/books"), None, &[], &modes)
        .unwrap()
        .unwrap();
    assert_eq!(s.source, Source::Detected(Detect::Vault));
    assert_eq!(names(&s.tables), ["books", "projects", "tasks"]);
    // どれにも当たらなければ None。
    assert_eq!(
        resolve(&t.0.join("elsewhere"), None, &[], &modes).unwrap(),
        None
    );
    assert_eq!(
        resolve(&tasks, None, &[], &[]).unwrap(),
        None,
        "検知しない設定"
    );
}
