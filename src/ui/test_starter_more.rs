//! スターターから台帳を作る画面の、作る場所の読み方(ST-5)。セキュリティの回とレビューの指摘から: `~` はホームだが、
//! `~foo` は開いたフォルダからの相対。フォルダを2つ以上開いているときは作らない。値は作ったもの。

use super::keymap::Mode;
use super::starter::resolve_dir;
use super::startup::Startup;
use super::test_relations::workspace;
use super::*;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use std::path::{Path, PathBuf};

/// パスの部品を `/` でつなぐ(区切りは OS に依らず `/`。名前の中の `\` は残る)。
fn slash(p: &Path) -> String {
    p.iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// `root` の下のファイル(根からの相対。揃える。区切りは `/`)。
fn files_under(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(slash(p.strip_prefix(root).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn test_st_5_two_open_folders_refuse_to_create() {
    // [ST-5] フォルダを2つ開いている(`nv.target` は実体のパスを改行で連ねたもの)ときは、作る場所の基準が決められないので
    // 理由を出して始めず、何も作らない(改行入りの名前のフォルダを掘らない)。
    let tmp = workspace("st_two");
    let dirs = [tmp.notes().join("tasks"), tmp.notes().join("projects")];
    let src = Markdown::open(&dirs).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 24);
    let target = PathBuf::from(
        dirs.iter()
            .map(|d| d.canonicalize().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    a.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target,
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    let before = files_under(&tmp.notes());
    a.start_starter_new();
    assert_eq!(a.mode, Mode::Table, "始めない");
    assert!(
        a.message.as_deref().unwrap_or("").contains("複数"),
        "{:?}",
        a.message
    );
    assert_eq!(files_under(&tmp.notes()), before, "何も作らない");
    assert!(a.switch_to.is_none());
}

#[test]
fn test_st_5_folder_is_relative_to_the_open_folder_unless_home_or_absolute() {
    // [ST-5] 相対は開いたフォルダから。`~foo/x` は `~` の展開でなく相対(プロセスの作業フォルダに作らない)。
    // 絶対はそのまま。`~` と `~/x` はホーム(無ければ文字のまま)。
    let base = Path::new("/base/notes");
    assert_eq!(resolve_dir(base, "books"), base.join("books"));
    assert_eq!(resolve_dir(base, "a/b"), base.join("a/b"));
    assert_eq!(resolve_dir(base, "~foo/x"), base.join("~foo/x"));
    // 絶対のパスは OS の一時フォルダから作る(`/abs/x` は Windows では絶対でない)。
    let abs = std::env::temp_dir().join("mdgrid-st-abs").join("x");
    assert!(abs.is_absolute());
    assert_eq!(resolve_dir(base, &abs.to_string_lossy()), abs);
    let home = mdgrid::places::expand_home("~");
    assert_eq!(resolve_dir(base, "~"), home);
    assert_eq!(resolve_dir(base, "~/x"), mdgrid::places::expand_home("~/x"));
    assert!(
        !resolve_dir(base, "~/x").starts_with(base),
        "`~/x` は開いたフォルダの下でない"
    );
}
