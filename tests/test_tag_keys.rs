//! [BV-6] タグのキーは大文字小文字を問わず `tags` と `tag`(Obsidian と同じ)。file.hasTag と file.tags で拾う。
//! specs/_changes/2026-10-06-tag-keys.md。本物の実行ファイルの --print で確かめる。

use std::process::Command;

#[test]
fn test_bv_6_tag_keys_any_case() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("mdgrid-tk-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    for (f, t) in [
        ("a.md", "---\ntags: [project/alpha]\n---\n"),
        ("b.md", "---\nTags: [project/beta]\n---\n"),
        ("c.md", "---\ntag: project/gamma\n---\n"),
        ("d.md", "---\nTAG: \"other, project\"\n---\n"),
        ("e.md", "---\ntagline: project\n---\n"),
    ] {
        std::fs::write(root.join(f), t).unwrap();
    }
    std::fs::write(
        root.join("t.base"),
        "filters: 'file.hasTag(\"project\")'\nviews:\n  - type: table\n    name: v\n    order: [file.name]\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(root.join("t.base"))
        .arg("--print")
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env_remove("MDGRID_CONFIG")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    let mut rows: Vec<&str> = text.lines().skip(1).collect();
    rows.sort();
    assert_eq!(rows, ["a.md", "b.md", "c.md", "d.md"], "{text}");
}
