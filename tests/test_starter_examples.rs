#![cfg(unix)]
//! ST-2: 見本 `examples/bookmarks` は、組み込みのスターター bookmarks(英語・見本つき)から作ったものと一致する。
//! スターターが正本で、見本はその写し。ずれたら見本を作り直す:
//! `mdgrid examples/bookmarks --init-workspace --starter bookmarks --with-samples`(英語で。見本の日付は固定なので
//! いつ作っても同じ)。

mod pty;

use pty::TempDir;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// `root` の下の全部のファイル(根からの相対)とその中身。
fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
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
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                map.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    map
}

#[test]
fn test_st_2_examples_bookmarks_matches_the_builtin_starter() {
    // [ST-2] 組み込みの bookmarks から英語・見本つきで作った木が、リポの examples/bookmarks と(パスも中身も)同じ。
    let t = TempDir::new("st-examples");
    let h = t.path();
    let made = h.join("bookmarks");
    let o = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .args([
            made.to_str().unwrap(),
            "--init-workspace",
            "--starter",
            "bookmarks",
            "--with-samples",
        ])
        .current_dir(h)
        .env("HOME", h)
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("XDG_CONFIG_HOME", h.join("config"))
        .env("XDG_STATE_HOME", h.join("state"))
        .env("MDGRID_TODAY", "2026-10-06")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/bookmarks");
    let a = files(&made);
    let b = files(&repo);
    assert_eq!(
        a.keys().collect::<Vec<_>>(),
        b.keys().collect::<Vec<_>>(),
        "ファイルの木が違う(見本を作り直す)"
    );
    for (k, v) in &a {
        assert!(b[k] == *v, "{k} の中身が違う(見本を作り直す)");
    }
}
