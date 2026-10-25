//! 台帳のスターターの境界(ST-1・ST-3・ST-4)。セキュリティの回の指摘から: 途中のフォルダのリンク、印の無い言語の変種、
//! ファイルでないもの。`src/starter.rs` の中として `#[path] mod tests_more;` で取り込む。値は作ったもの。

use super::*;
use crate::i18n::Lang;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "[name]\nen = \"Kanban\"\nja = \"カンバン\"\n";
const MARKER: &str = "open = \"Board\"\n\n[[table]]\nname = \"Board\"\npath = \"board\"\n";

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "mdgrid-st-more-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(std::fs::canonicalize(&d).unwrap())
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// ユーザーのスターター `cfg/starters/kanban/`(印・雛形・見本つき)。
fn kanban(cfg: &Path) -> PathBuf {
    let dir = cfg.join("starters/kanban");
    put(&dir.join("starter.toml"), MANIFEST);
    put(&dir.join("ledger/.mdgrid/workspace.toml"), MARKER);
    put(&dir.join("ledger/.mdgrid/templates/card.md"), "# {name}\n");
    put(
        &dir.join("samples/board/First card.md"),
        "---\ntitle: First card\n---\n",
    );
    dir
}

fn now() -> (i64, i64) {
    let today = crate::types::parse_date("2026-10-06").unwrap();
    (today, today * 86_400)
}

#[cfg(unix)]
#[test]
fn test_st_4_create_refuses_when_a_folder_on_the_way_is_a_link() {
    // [ST-4] 既にある根の中で、見本の置き場 board/ がリンク(台帳の外を指す)なら、理由を出してどれも書かない
    // (create_dir_all がリンクをたどって外に書くのを防ぐ)。リンクの先にも何も書かれない。
    let t = Tmp::new("link");
    let s = read_user(&kanban(&t.0.join("config"))).unwrap_or_else(|e| panic!("{e}"));
    let root = t.0.join("ledger");
    let outside = t.0.join("outside");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("board")).unwrap();
    let e = match create(&root, &s, Lang::En, true, now()) {
        Ok(_) => panic!("リンクの下に書いた"),
        Err(e) => e,
    };
    assert!(e.contains("board"), "理由にリンクのパス: {e}");
    assert!(
        std::fs::read_dir(&outside).unwrap().next().is_none(),
        "リンクの先に書かない"
    );
    assert!(!root.join(".mdgrid").exists(), "印も雛形も書かない");
    // 見本を入れなければ、見本の置き場のリンクは関係なく作れる。
    let made = create(&root, &s, Lang::En, false, now()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(made.len(), 2);
}

#[test]
fn test_st_1_ledger_variant_without_marker_is_refused() {
    // [ST-1][ST-3] ledger.ja/ に印が無いスターターは読めない(変種は丸ごと差し替えるので、日本語で作ると印の無い
    // 台帳ができるため)。警告は CLI-3 の形で、無い印のパスを名指す。
    let t = Tmp::new("variant");
    let cfg = t.0.join("config");
    let dir = kanban(&cfg);
    put(
        &dir.join("ledger.ja/.mdgrid/templates/fuda.md"),
        "# {name}\n",
    );
    let e = match read_user(&dir) {
        Ok(_) => panic!("印の無い変種を読んだ"),
        Err(e) => e,
    };
    assert!(
        e.starts_with("starters/kanban: ledger.ja/.mdgrid/workspace.toml:"),
        "{e}"
    );
    let (list, warns) = list(Some(&cfg));
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert!(!list.iter().any(|s| s.name == "kanban"));
    // 変種にも印を置けば読める。
    put(&dir.join("ledger.ja/.mdgrid/workspace.toml"), MARKER);
    assert!(read_user(&dir).is_ok());
}

#[cfg(unix)]
#[test]
fn test_st_3_non_file_entries_in_a_starter_are_skipped() {
    // [ST-3] 設定の置き場のスターターの中の FIFO(ファイルでないもの)は読まない(読むと一覧が止まる)。
    let t = Tmp::new("fifo");
    let cfg = t.0.join("config");
    let dir = kanban(&cfg);
    let fifo = dir.join("ledger/pipe");
    let ok = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        eprintln!("mkfifo が無いので飛ばす");
        return;
    }
    let s = read_user(&dir).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !s.files.iter().any(|f| f.rel == Path::new("pipe")),
        "FIFO は入らない"
    );
    assert_eq!(s.files.len(), 3);
}

#[test]
fn test_st_3_hidden_folders_in_starters_are_not_starters() {
    // [ST-3] `starters/.git` のような隠しフォルダは、スターターとして読まず、警告も出さない。
    let t = Tmp::new("hidden");
    let cfg = t.0.join("config");
    kanban(&cfg);
    put(&cfg.join("starters/.git/HEAD"), "ref: refs/heads/main\n");
    let (list, warns) = list(Some(&cfg));
    assert!(warns.is_empty(), "{warns:?}");
    assert!(list.iter().any(|s| s.name == "kanban"));
    assert!(!list.iter().any(|s| s.name == ".git"));
}

#[cfg(unix)]
#[test]
fn test_st_3_a_linked_starter_folder_is_read() {
    // [ST-3] `starters/<名前>` がシンボリックリンク(dotfiles の置き方)でも、そのスターターは読める。
    // 中のファイルのリンクは、読む側で飛ばすまま。
    let t = Tmp::new("linked");
    let cfg = t.0.join("config");
    let real = t.0.join("elsewhere/kanban");
    put(&real.join("starter.toml"), MANIFEST);
    put(&real.join("ledger/.mdgrid/workspace.toml"), MARKER);
    std::fs::create_dir_all(cfg.join("starters")).unwrap();
    std::os::unix::fs::symlink(&real, cfg.join("starters/kanban")).unwrap();
    let (list, warns) = list(Some(&cfg));
    assert!(warns.is_empty(), "{warns:?}");
    let s = list
        .iter()
        .find(|s| s.name == "kanban")
        .expect("リンクの先を読む");
    assert!(s
        .files
        .iter()
        .any(|f| f.rel == Path::new(".mdgrid/workspace.toml")));
}

#[test]
fn test_st_3_unreadable_file_warning_names_the_file_in_cli_3_shape() {
    // [ST-3] UTF-8 でないファイルのあるスターターは、`starters/<名前>: <中の相対パス>: 理由` の形で警告して飛ばす
    // (道筋が空の「: :」にしない)。
    let t = Tmp::new("notext");
    let cfg = t.0.join("config");
    let dir = kanban(&cfg);
    std::fs::write(dir.join("ledger/bad.md"), [0xff, 0xfe, 0x00, 0x41]).unwrap();
    let e = match read_user(&dir) {
        Ok(_) => panic!("UTF-8 でないファイルを読んだ"),
        Err(e) => e,
    };
    assert!(e.starts_with("starters/kanban: ledger/bad.md: "), "{e}");
    assert!(!e.contains(": : "), "道筋が空でない: {e}");
}
