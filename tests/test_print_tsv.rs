//! [CLI-5] `--print --format tsv`: 見出しの行つきのタブ区切り。セルの中のタブと改行は空白。
//! specs/_changes/2026-10-07-export-table.md。

use std::process::Command;

#[test]
fn test_cli_5_print_tsv() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mdgrid-tsv-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.md"), "---\nt: \"x\\ty\"\nn: 1\n---\n").unwrap();
    std::fs::write(dir.join("b.md"), "---\nt: \"l1\\nl2\"\nn: 2\n---\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(&dir)
        .args(["--print", "--format", "tsv", "--sort", "n"])
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "t\tn\nx y\t1\nl1 l2\t2\n");
}
