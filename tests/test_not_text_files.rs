//! 文字のファイルでないファイル(先頭に NUL・UTF-16 の BOM)を、ノートのほかのどの読む口でも中身として使わない
//! (write-back.refuse-not-text-any-file・cli.read-text-only)。本物の実行ファイルを動かし、出力・終了コード・
//! ファイルのバイトを見る。画面を出さない起動なので、標準出力はパイプで受ける。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 試験ごとの一時フォルダ。Drop で消す。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let p = std::env::temp_dir().join(format!("mdgrid-nt-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        TempDir(p.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// `rel` に `bytes` を書く(途中のフォルダも作る)。
    fn write(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, bytes).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_STATE_HOME", home.join("state"))
        .output()
        .unwrap()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// 名前だけほかの形のファイルの中身: 2行目に NUL のバイトがあり、UTF-8 としては読める。
fn with_nul(first_line: &str) -> Vec<u8> {
    let mut b = first_line.as_bytes().to_vec();
    b.extend_from_slice(b"\nPK\x03\x04\x00\x00report\n");
    b
}

/// ふつうのノートを1つ置いた `notes/`。
fn notes(dir: &TempDir) -> PathBuf {
    dir.write("notes/a.md", b"---\nstatus: todo\n---\n");
    dir.path().join("notes")
}

#[test]
fn test_csv_with_nul_is_not_opened() {
    // [stest -> req~write-back.refuse-not-text-any-file~1] 2行目に NUL のある tasks.csv → 表を出さず、テキストでないという理由1行と終了コード 2。tasks.csv は変わらない。
    let dir = TempDir::new("csv");
    let bytes = with_nul("title,status");
    let csv = dir.write("tasks.csv", &bytes);
    let o = run(dir.path(), &[csv.to_str().unwrap(), "--print"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(
        o.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let e = stderr(&o);
    assert!(e.contains("テキストでない"), "{e}");
    assert_eq!(e.trim_end().lines().count(), 1, "{e}");
    assert_eq!(std::fs::read(&csv).unwrap(), bytes);
}

#[test]
fn test_csv_utf16_names_utf16() {
    // [stest -> req~write-back.refuse-not-text-any-file~1] UTF-16 の BOM で始まる tasks.csv → 開かず、理由は UTF-16。
    let dir = TempDir::new("csv16");
    let mut bytes = vec![0xFF, 0xFE];
    for u in "title,status\nx,todo\n".encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    let csv = dir.write("tasks.csv", &bytes);
    let o = run(dir.path(), &[csv.to_str().unwrap(), "--print"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(stderr(&o).contains("UTF-16"), "{}", stderr(&o));
    assert_eq!(std::fs::read(&csv).unwrap(), bytes);
}

#[test]
fn test_base_with_nul_is_not_opened() {
    // [stest -> req~cli.read-text-only~1] NUL を含む tasks.base → 表を出さず、tasks.base とテキストでないという理由の1行と、終了コード 2。
    let dir = TempDir::new("base");
    notes(&dir);
    let base = dir.write("notes/tasks.base", &with_nul("views:"));
    let o = run(dir.path(), &[base.to_str().unwrap(), "--print"]);
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(o.stdout.is_empty());
    let e = stderr(&o);
    assert!(e.contains("tasks.base"), "{e}");
    assert!(e.contains("テキストでない"), "{e}");
}

#[test]
fn test_config_with_nul_stops() {
    // [stest -> req~cli.read-text-only~1] NUL を含む設定ファイルを --config で渡す → 表を出さず、設定ファイルのパスとテキストでないという理由の1行と、終了コード 2。
    let dir = TempDir::new("config");
    let notes = notes(&dir);
    let cfg = dir.write("my.toml", &with_nul("theme = \"sumi\""));
    let o = run(
        dir.path(),
        &[
            "--config",
            cfg.to_str().unwrap(),
            notes.to_str().unwrap(),
            "--print",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    assert!(o.stdout.is_empty());
    let e = stderr(&o);
    assert!(e.contains("my.toml"), "{e}");
    assert!(e.contains("テキストでない"), "{e}");
}

#[test]
fn test_apply_input_with_nul_writes_nothing() {
    // [stest -> req~cli.read-text-only~1] NUL を含む in.csv を --apply → どのノートも書かず、in.csv とテキストでないという理由の1行と、終了コード 2。
    let dir = TempDir::new("apply");
    let notes = notes(&dir);
    let note = std::fs::read(notes.join("a.md")).unwrap();
    let input = dir.write("in.csv", &with_nul("path,status"));
    let o = run(
        dir.path(),
        &[
            notes.to_str().unwrap(),
            "--apply",
            input.to_str().unwrap(),
            "--yes",
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(e.contains("in.csv"), "{e}");
    assert!(e.contains("テキストでない"), "{e}");
    assert_eq!(std::fs::read(notes.join("a.md")).unwrap(), note);
}

#[test]
fn test_new_note_template_with_nul_makes_nothing() {
    // [stest -> req~cli.read-text-only~1] [new_note] body の雛形が NUL を含む → --new-note でノートを作らず、雛形のパスとテキストでないという理由の1行と、0 以外の終了コード。
    let dir = TempDir::new("newnote");
    let notes = notes(&dir);
    dir.write(
        "notes/.mdgrid/workspace.toml",
        b"open = \"T\"\n\n[new_note]\nbody = \".mdgrid/templates/task.md\"\n\n\
[[table]]\nname = \"T\"\n\n[table.base]\nfilters = { and = [] }\n",
    );
    dir.write("notes/.mdgrid/templates/task.md", &with_nul("# {name}"));
    let o = run(dir.path(), &[notes.to_str().unwrap(), "--new-note", "会議"]);
    assert_ne!(o.status.code(), Some(0), "{}", stderr(&o));
    let e = stderr(&o);
    assert!(e.contains("task.md"), "{e}");
    assert!(e.contains("テキストでない"), "{e}");
    assert!(!notes.join("会議.md").exists());
}
