//! [BV-9] ノートが 2,000 を超える保管庫では1回の見回りで見るノートを限って順に回すが、
//! 変わった・消えたノートは2回の見回りのうちに分かる(specs/_changes/2026-10-06-poll-round-robin.md)。

use mdgrid::vault::{roots, Vault};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

fn temp(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!("mdgrid-rr-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn bump(p: &Path) {
    let f = std::fs::OpenOptions::new().write(true).open(p).unwrap();
    f.set_modified(SystemTime::now() + Duration::from_secs(10))
        .unwrap();
}

fn big(name: &str, n: usize) -> (PathBuf, Vault) {
    let dir = temp(name);
    for i in 0..n {
        std::fs::write(dir.join(format!("n{i:05}.md")), b"---\nx: 1\n---\n").unwrap();
    }
    let mut v = Vault::open(roots(std::slice::from_ref(&dir)).unwrap());
    for _ in 0..100_000 {
        if v.load(5000).done {
            break;
        }
    }
    assert_eq!(v.notes().len(), n);
    (dir, v)
}

#[test]
fn test_bv_9_round_robin_change_found_within_two_polls() {
    let (dir, mut v) = big("change", 2500);
    v.poll();
    for name in ["n00010.md", "n02490.md"] {
        let p = dir.join(name);
        std::fs::write(&p, b"---\nx: 2\n---\n").unwrap();
        bump(&p);
    }
    let mut changed: Vec<PathBuf> = Vec::new();
    for _ in 0..2 {
        changed.extend(v.poll().changed);
    }
    let names: Vec<String> = changed
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names.contains(&"n00010.md".to_string()), "{names:?}");
    assert!(names.contains(&"n02490.md".to_string()), "{names:?}");
    assert_eq!(names.len(), 2, "ほかは変わらない: {names:?}");
}

#[test]
fn test_bv_9_round_robin_removed_found_within_two_polls() {
    let (dir, mut v) = big("removed", 2500);
    v.poll();
    std::fs::remove_file(dir.join("n02499.md")).unwrap();
    let mut removed = 0;
    for _ in 0..2 {
        removed += v.poll().removed.len();
    }
    assert_eq!(removed, 1);
    assert_eq!(v.notes().len(), 2499);
}
