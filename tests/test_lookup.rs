//! つながったノートの値を引く・集める式の列(base-view.linked-from・base-view.rollup-column)を、本物の実行ファイルの
//! `--print` で確かめる。一時のルートフォルダ(根に `.obsidian/`)にノートと `.base` を書く。期待値は要求の受け入れの例から取った。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let nanos = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-lookup-{name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(dir.join(".obsidian")).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, rel: &str, text: &str) -> PathBuf {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `.base` を開いて `--print --format <format>` の標準出力。成功でなければ落とす。
fn print(dir: &TempDir, base_rel: &str, base: &str, format: &str) -> String {
    let bp = dir.write(base_rel, base);
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(&bp)
        .args(["--print", "--format", format])
        .current_dir(dir.path())
        .env("MDGRID_TODAY", "2026-11-25")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        out.status.success(),
        "--print が失敗: {:?}\nstdout: {stdout}\nstderr: {stderr}",
        out.status
    );
    stdout
}

/// 式 `x`(と file.name)を出す、`folder` の中の行のビュー(名前の順)。
fn base(folder: &str, x: &str) -> String {
    format!(
        "filters: 'file.inFolder(\"{folder}\")'\nformulas:\n  x: '{}'\nviews:\n  - type: table\n    name: v\n    order: [file.name, formula.x]\n    sort:\n      - property: file.name\n        direction: ASC\n",
        x.replace('\'', "''")
    )
}

#[test]
fn test_linked_from_print_json_paths() {
    // [stest -> req~base-view.linked-from~1] 問い Q を n1 が question で、q2 が related で、n3 が本文で指す台帳で、
    // 式 file.linkedFrom("question") の列を --print --format json → Q の行の値は ["News/n1"]
    let d = TempDir::new("linked");
    d.write("Questions/Q.md", "---\nstatus: researching\n---\n");
    d.write("News/n1.md", "---\nquestion:\n  - \"[[Q]]\"\n---\n");
    d.write("Questions/q2.md", "---\nrelated:\n  - \"[[Q]]\"\n---\n");
    d.write("n3.md", "本文から [[Q]] を指す\n");
    let out = print(
        &d,
        "Q.base",
        &base("Questions", "file.linkedFrom(\"question\")"),
        "json",
    );
    let row = out
        .lines()
        .find(|l| l.contains("\"Q.md\""))
        .unwrap_or_else(|| panic!("Q の行が無い:\n{out}"));
    assert!(row.contains("[\"News/n1\"]"), "{out}");
    let q2 = out
        .lines()
        .find(|l| l.contains("\"q2.md\""))
        .unwrap_or_else(|| panic!("q2 の行が無い:\n{out}"));
    assert!(q2.contains("[]"), "{out}");
}

#[test]
fn test_rollup_column_last_met_from_done_meetings() {
    // [stest -> req~base-view.rollup-column~1] 安藤 真を attendees に持つ会議が3つ(済みの 2026-11-09T10:00 と 2026-11-24T10:00、
    // 予定の 2026-11-30T10:00)と、会議の無い北川 翔 → 人の表の式 met の列は、安藤 真が 2026-11-24、北川 翔が空
    let d = TempDir::new("rollup");
    d.write("People/安藤 真.md", "---\nrole: dev\n---\n");
    d.write("People/北川 翔.md", "---\nrole: friend\n---\n");
    for (name, when, status) in [
        ("m1", "2026-11-09T10:00", "done"),
        ("m2", "2026-11-24T10:00", "done"),
        ("m3", "2026-11-30T10:00", "planned"),
    ] {
        d.write(
            &format!("Meetings/{name}.md"),
            &format!("---\nwhen: {when}\nstatus: {status}\nattendees:\n  - \"[[安藤 真]]\"\n---\n"),
        );
    }
    let met = "file.linkedFrom(\"attendees\").filter(value.inFolder(\"Meetings\") && value.properties.status == \"done\").map(value.properties.when).max().date()";
    let out = print(&d, "People.base", &base("People", met), "csv");
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"安藤 真.md,2026-11-24"), "{out}");
    assert!(lines.contains(&"北川 翔.md,"), "{out}");
}
