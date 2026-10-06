//! [WB-5][BV-1][BV-6] JSON・フローの形のフロントマターは、読むだけのまま値を見せ、file.tags にも入れる。
//! specs/_changes/2026-10-06-flow-frontmatter.md。本物の実行ファイルで確かめる。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const JSON: &str = "---\n{\"tags\": [\"json\"], \"status\": \"todo\", \"n\": 3}\n---\nbody\n";

fn vault(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-ff-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("v/.obsidian")).unwrap();
    std::fs::write(root.join("v/a.md"), JSON).unwrap();
    std::fs::write(
        root.join("v/b.md"),
        "---\n{tags: [flow], status: doing}\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("v/c.md"), "---\nstatus: done\n---\n").unwrap();
    root
}

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().unwrap();
    if let Some(s) = stdin {
        child.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

#[test]
fn test_wb_5_flow_frontmatter_values_shown() {
    let root = vault("show");
    let o = run(
        &root,
        &["v", "--print", "--with-path", "--format", "json"],
        None,
    );
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(
        s.contains("\"path\": \"v/a.md\", \"tags\": [\"json\"], \"status\": \"todo\""),
        "{s}"
    );
    assert!(s.contains("\"n\": 3"), "{s}");
    assert!(s.contains("\"status\": \"doing\""), "{s}");
    // hasTag にも当たる。
    std::fs::write(
        root.join("v/t.base"),
        "filters: 'file.hasTag(\"json\") || file.hasTag(\"flow\")'\nviews:\n  - type: table\n    name: v\n    order: [file.name]\n",
    )
    .unwrap();
    let o = run(&root, &["v/t.base", "--print"], None);
    let s = String::from_utf8(o.stdout).unwrap();
    assert_eq!(s.lines().count(), 3, "{s}");
}

#[test]
fn test_wb_5_flow_frontmatter_stays_read_only() {
    let root = vault("ro");
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,status\nv/a.md,done\n"),
    );
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(std::fs::read_to_string(root.join("v/a.md")).unwrap(), JSON);
}
