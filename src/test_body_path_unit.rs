//! [CE-32] 本文の雛形(body)は開いたフォルダの中のファイルだけを読む。specs/_changes/2026-10-10-body-path.md。

use crate::newnote::{body_path, read_body};

fn root(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-body-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("notes/templates")).unwrap();
    std::fs::create_dir_all(d.join("notes/.obsidian/templates")).unwrap();
    std::fs::write(d.join("notes/templates/t.md"), "本文").unwrap();
    std::fs::write(d.join("notes/.obsidian/templates/o.md"), "本文").unwrap();
    std::fs::write(d.join("secret.txt"), "秘密").unwrap();
    d
}

#[test]
fn test_ce_32_body_inside_folder_is_read() {
    let d = root("in");
    let r = d.join("notes");
    assert_eq!(body_path(&r, "").unwrap(), None);
    assert!(body_path(&r, "templates/t.md").unwrap().is_some());
    assert!(body_path(&r, "./templates/t.md").unwrap().is_some());
    assert!(body_path(&r, ".obsidian/templates/o.md").unwrap().is_some());
    assert_eq!(
        read_body(&r, "templates/t.md").unwrap().as_deref(),
        Some("本文")
    );
    // 無いファイルは、確かめられないので読まずに理由。
    let e = body_path(&r, "templates/none.md").unwrap_err();
    assert!(e.contains("読めない"), "{e}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn test_ce_32_body_outside_folder_is_refused() {
    let d = root("out");
    let r = d.join("notes");
    let abs = d.join("secret.txt");
    for bad in [
        "../secret.txt",
        "templates/../../secret.txt",
        abs.to_str().unwrap(),
        "~/secret.txt",
        "\\\\server\\share\\x.md",
    ] {
        let e = body_path(&r, bad).unwrap_err();
        assert!(e.contains("開いたフォルダの中"), "{bad}: {e}");
        assert!(read_body(&r, bad).is_err());
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[cfg(unix)]
#[test]
fn test_ce_32_body_symlink_out_of_folder_is_refused() {
    let d = root("link");
    let r = d.join("notes");
    std::os::unix::fs::symlink(d.join("secret.txt"), r.join("templates/link.md")).unwrap();
    std::os::unix::fs::symlink(&d, r.join("up")).unwrap();
    assert!(body_path(&r, "templates/link.md").is_err());
    assert!(body_path(&r, "up/secret.txt").is_err());
    // 行き先の無いリンクは、あとで行き先を作られても読まないよう、確かめた時点で断る。
    std::os::unix::fs::symlink(d.join("later.txt"), r.join("templates/dang.md")).unwrap();
    assert!(body_path(&r, "templates/dang.md").is_err());
    let _ = std::fs::remove_dir_all(&d);
}
