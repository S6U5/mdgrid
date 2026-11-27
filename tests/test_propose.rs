//! [ai.propose-command][ai.proposal-file][ai.write-columns] `--apply <ファイル> --propose <理由>`: AI に許した列は書き、
//! ほかは台帳の `.mdgrid/proposals/` に案として置く。本物の実行ファイルで確かめる。

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const A: &str = "---\nstatus: todo\ndue: 2026-12-01\n---\nbody a\n";

/// 台帳の根 `notes/`(印 `.mdgrid/workspace.toml` に `marker` と表 `t` を書く)と、表のノート `notes/t/a.md`。
fn ledger(name: &str, marker: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mdgrid-propose-{name}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(root.join("notes/.mdgrid")).unwrap();
    std::fs::create_dir_all(root.join("notes/t")).unwrap();
    std::fs::write(
        root.join("notes/.mdgrid/workspace.toml"),
        format!("{marker}\n[[table]]\nname = \"T\"\npath = \"t\"\n"),
    )
    .unwrap();
    std::fs::write(root.join("notes/t/a.md"), A).unwrap();
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .current_dir(root)
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("cfg"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("LANG", "en_US.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("MDGRID_CONFIG")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn note(root: &Path) -> String {
    std::fs::read_to_string(root.join("notes/t/a.md")).unwrap()
}

/// 案のファイル(名前の順)。
fn proposals(root: &Path) -> Vec<PathBuf> {
    let dir = root.join("notes/.mdgrid/proposals");
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|d| d.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    v.retain(|p| p.extension().is_some_and(|e| e == "json"));
    v.sort();
    v
}

fn json(p: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// `--print --with-path` の CSV の status と due を直して `edit.csv` に書く。
fn edited(root: &Path) {
    let o = run(root, &["notes/t", "--print", "--with-path"]);
    assert!(o.status.success(), "{}", err(&o));
    let csv = out(&o)
        .replacen("todo", "done", 1)
        .replacen("2026-12-01", "2026-12-20", 1);
    std::fs::write(root.join("edit.csv"), csv).unwrap();
}

#[test]
fn test_propose_writes_allowed_and_proposes_the_rest() {
    // [stest -> req~ai.propose-command~1, req~ai.write-columns~1] `[ai] write = ["status"]` → status は書き、due は案。
    // 標準出力に書いた数・案にした数・案のファイルの名前。
    let root = ledger("allowed", "[ai]\nwrite = [\"status\"]\n");
    edited(&root);
    let o = run(
        &root,
        &[
            "notes/t",
            "--apply",
            "edit.csv",
            "--propose",
            "締め切りを見直す",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(note(&root), A.replace("todo", "done"));
    let ps = proposals(&root);
    assert_eq!(ps.len(), 1, "{ps:?}");
    let p = json(&ps[0]);
    let set = &p["changes"][0]["set"];
    assert_eq!(set["due"].as_str(), Some("2026-12-20"), "{p}");
    assert!(set.get("status").is_none(), "{p}");
    let s = out(&o);
    let name = ps[0].file_name().unwrap().to_string_lossy().into_owned();
    assert!(s.contains(&name), "{s}");
    assert!(s.contains('1'), "{s}");
}

#[test]
fn test_propose_without_ai_setting_proposes_everything() {
    // [stest -> req~ai.propose-command~1, req~ai.write-columns~1] `[ai]` の無い台帳 → ノートは変わらず、status と due が1つの案。
    let root = ledger("none", "");
    edited(&root);
    let o = run(
        &root,
        &["notes/t", "--apply", "edit.csv", "--propose", "見直す"],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(note(&root), A);
    let ps = proposals(&root);
    assert_eq!(ps.len(), 1, "{ps:?}");
    let set = &json(&ps[0])["changes"][0]["set"];
    assert_eq!(set["status"].as_str(), Some("done"));
    assert_eq!(set["due"].as_str(), Some("2026-12-20"));
}

#[test]
fn test_propose_refuses_stale_rows() {
    // [stest -> req~ai.propose-command~1] 書き出したあとに外で due を直した → 何も書かず、案も置かず、終了コード 2。
    let root = ledger("stale", "[ai]\nwrite = [\"status\"]\n");
    let o = run(&root, &["notes/t", "--print", "--with-path"]);
    let csv = out(&o).replacen("todo", "done", 1);
    let outside = A.replace("2026-12-01", "2026-12-05");
    std::fs::write(root.join("notes/t/a.md"), &outside).unwrap();
    std::fs::write(root.join("edit.csv"), csv).unwrap();
    let o = run(&root, &["notes/t", "--apply", "edit.csv", "--propose", "x"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(note(&root), outside);
    assert!(proposals(&root).is_empty());
}

#[test]
fn test_propose_nothing_changed_writes_nothing() {
    // [stest -> req~ai.propose-command~1] 書き出したまま直していない CSV → 何も書かず、案も置かず、終了コード 0。
    let root = ledger("same", "");
    let o = run(&root, &["notes/t", "--print", "--with-path"]);
    std::fs::write(root.join("edit.csv"), out(&o)).unwrap();
    let o = run(&root, &["notes/t", "--apply", "edit.csv", "--propose", "x"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(note(&root), A);
    assert!(proposals(&root).is_empty());
}

#[test]
fn test_propose_flag_combinations() {
    // [stest -> req~ai.propose-command~1] `--propose` に `--yes`、`--apply` の無い `--propose` → 理由1行と終了コード 2。
    let root = ledger("flags", "");
    edited(&root);
    for args in [
        &["notes/t", "--apply", "edit.csv", "--propose", "x", "--yes"][..],
        &["notes/t", "--propose", "x"][..],
    ] {
        let o = run(&root, args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", err(&o));
        assert_eq!(err(&o).trim().lines().count(), 1, "{args:?}: {}", err(&o));
    }
    assert_eq!(note(&root), A);
    assert!(proposals(&root).is_empty());
}

#[test]
fn test_proposal_files_do_not_overlap_and_have_fields() {
    // [stest -> req~ai.proposal-file~1] 別の直しで2回 `--propose` → 2つの `.json`。どちらも note・created(YYYY-MM-DDTHH:MM)・
    // changes(path は台帳の根からのパス、set、was は案を作ったときの値)を持つ。
    let root = ledger("two", "");
    edited(&root);
    let o = run(
        &root,
        &["notes/t", "--apply", "edit.csv", "--propose", "一つ目"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let csv = "path,status\nnotes/t/a.md,doing\n";
    std::fs::write(root.join("edit2.csv"), csv).unwrap();
    let o = run(
        &root,
        &["notes/t", "--apply", "edit2.csv", "--propose", "二つ目"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let ps = proposals(&root);
    assert_eq!(ps.len(), 2, "{ps:?}");
    let notes: Vec<String> = ps
        .iter()
        .map(|p| json(p)["note"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(notes.contains(&"一つ目".to_string()) && notes.contains(&"二つ目".to_string()));
    for p in &ps {
        let v = json(p);
        let created = v["created"].as_str().unwrap_or_default();
        let b = created.as_bytes();
        assert!(
            b.len() == 16 && b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':',
            "{created}"
        );
        let c = &v["changes"][0];
        assert_eq!(c["path"].as_str(), Some("t/a.md"), "{v}");
        assert_eq!(c["was"]["status"].as_str(), Some("todo"), "{v}");
    }
}

#[test]
fn test_ai_write_columns_resolved_and_unlisted_column_is_proposed() {
    // [stest -> req~ai.write-columns~1] 全体の `[ai] write = ["status"]` と印の表の `[table.ai] write = ["status", "tags"]` → 表を
    // 開いた `--print-config --resolved` の ai.write は表の値で、出どころは表の範囲。書いていない列(priority)の直しは案になる。
    let root = ledger("cols", "");
    std::fs::write(
        root.join("notes/.mdgrid/workspace.toml"),
        "[[table]]\nname = \"T\"\npath = \"t\"\n[table.ai]\nwrite = [\"status\", \"tags\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("cfg/mdgrid")).unwrap();
    std::fs::write(
        root.join("cfg/mdgrid/config.toml"),
        "[ai]\nwrite = [\"status\"]\n",
    )
    .unwrap();
    let o = run(&root, &["notes/t", "--print-config", "--resolved"]);
    assert!(o.status.success(), "{}", err(&o));
    let s = out(&o);
    let lines: Vec<&str> = s.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.replace(' ', "") == "write=[\"status\",\"tags\"]")
        .unwrap_or_else(|| panic!("{s}"));
    // 出どころのコメントは表の範囲(表の名前 T)。
    assert!(
        lines[at - 1].starts_with('#') && lines[at - 1].contains('T'),
        "{s}"
    );
    std::fs::write(
        root.join("notes/t/a.md"),
        "---\nstatus: todo\npriority: 1\n---\n",
    )
    .unwrap();
    std::fs::write(root.join("edit.csv"), "path,priority\nnotes/t/a.md,3\n").unwrap();
    let o = run(&root, &["notes/t", "--apply", "edit.csv", "--propose", "x"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(note(&root), "---\nstatus: todo\npriority: 1\n---\n");
    let ps = proposals(&root);
    assert_eq!(ps.len(), 1);
    assert_eq!(json(&ps[0])["changes"][0]["set"]["priority"], 3);
}
