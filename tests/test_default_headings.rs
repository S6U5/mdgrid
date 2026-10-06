//! [BV-24] displayName の無い列の見出しは Obsidian の既定(式の名前・`file name` など)。
//! specs/_changes/2026-10-07-default-headings.md。本物の実行ファイルの `--print` で確かめる。

use std::path::PathBuf;
use std::process::Command;

struct Tmp(PathBuf);

impl Tmp {
    fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "mdgrid-headings-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&d).unwrap();
        Tmp(d)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn header(base: &str) -> String {
    let t = Tmp::new("h");
    std::fs::write(t.0.join("a.md"), "---\nstatus: todo\nprice: 3\n---\n").unwrap();
    std::fs::write(t.0.join("v.base"), base).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(t.0.join("v.base"))
        .args(["--print", "--format", "csv"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn test_bv_24_default_headings() {
    let base = "formulas:\n  価格: price * 2\nviews:\n  - type: table\n    name: t\n    order:\n      - file.name\n      - formula.価格\n      - status\n      - file.ctime\n      - file.mtime\n      - file.ext\n      - file.folder\n";
    assert_eq!(
        header(base),
        "file name,価格,status,created time,modified time,file extension,file folder"
    );
}

#[test]
fn test_bv_24_display_name_wins() {
    let base = "formulas:\n  価格: price * 2\nproperties:\n  file.name:\n    displayName: 名前\n  formula.価格:\n    displayName: 値段\nviews:\n  - type: table\n    name: t\n    order:\n      - file.name\n      - formula.価格\n";
    assert_eq!(header(base), "名前,値段");
}
