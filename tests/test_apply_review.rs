//! [CLI-17] --apply のレビューで見つけた形(specs/_changes/2026-10-06-apply.md の照合)。
//! 書き出したものをそのまま戻すと何も書かない(混ざった型・空の文字・`, ` を含む要素・大きな整数・
//! 型の合わない値・入れ子の値)。見出しの読み違い・打ち間違い・重なった行・今日からの日数は理由にする。

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-apr-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join("v")).unwrap();
    root
}

fn note(root: &Path, name: &str, text: &str) {
    let p = root.join("v").join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mdgrid"));
    c.current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
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

/// フォルダの全ファイルの中身(パスの順)。
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(d: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut es: Vec<_> = std::fs::read_dir(d).unwrap().flatten().collect();
        es.sort_by_key(|e| e.path());
        for e in es {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push((p.clone(), std::fs::read(&p).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("v"), &mut out);
    out
}

/// `target` を --print --with-path(形 f)で出し、そのまま --apply --yes で戻す。何も書かないこと。
fn round_trip(root: &Path, target: &str, f: &str) {
    let before = snapshot(root);
    let o = run(
        root,
        &[target, "--print", "--with-path", "--format", f],
        None,
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let text = String::from_utf8(o.stdout).unwrap();
    let o = run(root, &[target, "--apply", "-", "--yes"], Some(&text));
    assert!(
        o.status.success(),
        "{f}: {}\n{text}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        o.stdout.is_empty(),
        "{f}: 書いたファイル: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert_eq!(snapshot(root), before, "{f}: ファイルが変わった");
}

#[test]
fn test_cli_17_round_trip_odd_values_writes_nothing() {
    let root = dir("odd");
    note(
        &root,
        "a.md",
        "---\ntitle: alpha\nws: \" \"\nempty: \"\"\nn: 1\ntags: [x, \"a, b\", \" lead\"]\n---\n",
    );
    note(
        &root,
        "b.md",
        "---\ntitle: true\nws: x\nempty: y\nn: 12345678901234567890\ntags: [y]\n---\n",
    );
    note(
        &root,
        "c.md",
        "---\ntitle: 2026\nn: 1.0\nmap:\n  k: v\ndue: someday\n---\n",
    );
    note(&root, "d.md", "---\ntitle: 1.0\ndue: 2026-10-01\n---\n");
    for f in ["csv", "json"] {
        round_trip(&root, "v", f);
    }
}

#[test]
fn test_cli_17_round_trip_sample_vaults_writes_nothing() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (sample, target) in [
        ("vault", "v"),
        ("demo", "v/Tasks.base"),
        ("vault", "v/タスク.base"),
    ] {
        let root = dir(&format!("sample-{sample}"));
        let src = manifest.join("examples").join(sample);
        let st = Command::new("cp")
            .arg("-R")
            .arg(format!("{}/.", src.display()))
            .arg(root.join("v"))
            .status()
            .unwrap();
        assert!(st.success());
        for f in ["csv", "json"] {
            round_trip(&root, target, f);
        }
    }
}

#[test]
fn test_cli_17_crlf_csv_with_multiline_cell() {
    let root = dir("crlf");
    note(
        &root,
        "a.md",
        "---\nnote: |\n  line one\n  line two\nk: a\n---\n",
    );
    let o = run(&root, &["v", "--print", "--with-path"], None);
    let csv = String::from_utf8(o.stdout).unwrap().replace('\n', "\r\n");
    let before = snapshot(&root);
    let o = run(&root, &["v", "--apply", "-", "--yes"], Some(&csv));
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(snapshot(&root), before);
}

#[test]
fn test_cli_17_header_problems() {
    let root = dir("head");
    note(&root, "a.md", "---\nstatus: todo\nowner: ken\n---\n");
    let before = snapshot(&root);
    // 打ち間違いの見出しはキーを足さず、理由。
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,stauts\nv/a.md,done\n"),
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("stauts"));
    // 同じ見出しが2つ。
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,status,status\nv/a.md,a,b\n"),
    );
    assert_eq!(o.status.code(), Some(2));
    // 表示名が別の列の id と同じ(.base の displayName の入れ替え)。
    std::fs::write(
        root.join("v/t.base"),
        "properties:\n  note.owner:\n    displayName: status\n  note.status:\n    displayName: owner\nviews:\n  - type: table\n    name: v\n    order: [status, owner]\n",
    )
    .unwrap();
    let o = run(
        &root,
        &["v/t.base", "--apply", "-", "--yes"],
        Some("path,status,owner\nv/a.md,todo,ken\n"),
    );
    assert_eq!(
        o.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert_eq!(snapshot(&root)[0], before[0], "a.md は変わらない");
}

#[test]
fn test_cli_17_duplicates_relative_dates_and_lists() {
    let root = dir("dup");
    note(
        &root,
        "a.md",
        "---\nstatus: todo\ndue: 2026-10-01\ntags: [x, \"a, b\"]\n---\n",
    );
    let before = snapshot(&root);
    // 同じノートを2つの書き方で2回。
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,status\nv/a.md,one\n./v/a.md,two\n"),
    );
    assert_eq!(o.status.code(), Some(2));
    // 今日からの日数は読まない。
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,due\nv/a.md,+3\n"),
    );
    assert_eq!(o.status.code(), Some(2));
    // `, ` を含む要素のリストは CSV から直せない(JSON なら直せる)。
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("path,tags\nv/a.md,\"x, a, b, c\"\n"),
    );
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(snapshot(&root), before);
    let o = run(
        &root,
        &["v", "--apply", "-", "--yes"],
        Some("[{\"path\": \"v/a.md\", \"tags\": [\"x\", \"a, b\", \"c\"]}]"),
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let a = std::fs::read_to_string(root.join("v/a.md")).unwrap();
    assert!(a.contains("tags: [x, \"a, b\", c]"), "{a}");
}
