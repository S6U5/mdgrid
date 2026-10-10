//! [SC-17] CSV の `--print --with-path` の path は「ファイル#行の番号」で、`--apply` で同じ行に戻せる。
//! specs/_changes/2026-10-10-csv-source.md。

use std::process::Command;

fn dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!(
        "mdgrid-csvapply-{tag}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn mdgrid(d: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args(args)
        .current_dir(d)
        .env("LANG", "ja_JP.UTF-8")
        .env("XDG_CONFIG_HOME", d.join("config"))
        .env("XDG_STATE_HOME", d.join("state"))
        .output()
        .unwrap()
}

#[test]
fn test_sc_17_with_path_and_apply_roundtrip() {
    let d = dir("rt");
    std::fs::write(d.join("台帳.csv"), "id,qty\r\nA,1\r\nB,2\r\n").unwrap();
    let out = mdgrid(&d, &["--print", "--with-path", "台帳.csv"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().nth(2).unwrap(), "台帳.csv#2,B,2", "{text}");
    std::fs::write(
        d.join("edit.csv"),
        text.replace("台帳.csv#2,B,2", "台帳.csv#2,B,7"),
    )
    .unwrap();
    let out = mdgrid(&d, &["台帳.csv", "--apply", "edit.csv", "--yes"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(d.join("台帳.csv")).unwrap(),
        "id,qty\r\nA,1\r\nB,7\r\n"
    );
    let _ = std::fs::remove_dir_all(&d);
}
