//! [CLI-22] AI エージェント向けのスキル(SKILL.md)とプラグインのマニフェスト。
//! specs/_changes/2026-10-14-agent-skill.md。スキルの `sh` の囲みを本物の実行ファイルで動かし、
//! マニフェストの版が Cargo.toml の版と揃うことを確かめる。

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(rel: &str) -> serde_json::Value {
    let p = repo().join(rel);
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{rel} が読めません({}): {e}", p.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} が JSON として読めません: {e}"))
}

/// 索引の plugins から name が "mdgrid" のものを探す。
fn find_mdgrid<'a>(index: &'a serde_json::Value, rel: &str) -> &'a serde_json::Value {
    index["plugins"]
        .as_array()
        .unwrap_or_else(|| panic!("{rel} に plugins の配列がありません"))
        .iter()
        .find(|p| p["name"] == "mdgrid")
        .unwrap_or_else(|| panic!("{rel} に name が mdgrid のプラグインがありません"))
}

#[test]
fn test_cli_22_plugin_versions_match_crate() {
    let crate_version = env!("CARGO_PKG_VERSION");
    for rel in [
        "plugins/mdgrid/.claude-plugin/plugin.json",
        "plugins/mdgrid/.codex-plugin/plugin.json",
        "plugins/mdgrid/plugin.json",
    ] {
        let v = read_json(rel);
        assert_eq!(
            v["version"].as_str(),
            Some(crate_version),
            "{rel} の version が Cargo.toml の版({crate_version})と違います"
        );
    }

    let rel = ".claude-plugin/marketplace.json";
    let claude = read_json(rel);
    let p = find_mdgrid(&claude, rel);
    assert_eq!(
        p["source"].as_str(),
        Some("./plugins/mdgrid"),
        "{rel} の mdgrid の source が ./plugins/mdgrid ではありません: {}",
        p["source"]
    );

    let rel = ".agents/plugins/marketplace.json";
    let codex = read_json(rel);
    let p = find_mdgrid(&codex, rel);
    assert_eq!(
        p["source"]["source"].as_str(),
        Some("local"),
        "{rel} の mdgrid の source.source が local ではありません: {}",
        p["source"]
    );
    assert_eq!(
        p["source"]["path"].as_str(),
        Some("./plugins/mdgrid"),
        "{rel} の mdgrid の source.path が ./plugins/mdgrid ではありません: {}",
        p["source"]
    );
}

/// 情報の文字列がちょうど `sh` の囲みの中身を、上から順に取り出す。
#[cfg(unix)]
fn sh_blocks(md: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut cur: Option<(String, bool)> = None; // (中身, sh か)
    let mut fence = String::new();
    for line in md.lines() {
        let t = line.trim_start();
        match &mut cur {
            None => {
                let tick = t.starts_with("```");
                let tilde = t.starts_with("~~~");
                if tick || tilde {
                    let ch = if tick { '`' } else { '~' };
                    let n = t.chars().take_while(|&c| c == ch).count();
                    fence = std::iter::repeat_n(ch, n).collect();
                    let info = t[n..].trim();
                    cur = Some((String::new(), info == "sh"));
                }
            }
            Some((body, is_sh)) => {
                if t.starts_with(fence.as_str())
                    && t.trim_end()
                        .chars()
                        .all(|c| c == fence.chars().next().unwrap())
                {
                    if *is_sh {
                        blocks.push(std::mem::take(body));
                    }
                    cur = None;
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    blocks
}

#[cfg(unix)]
fn fixture() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("mdgrid-agent-skill-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(root.join(".obsidian")).unwrap();
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    for (f, t) in [
        (
            "write-report.md",
            "---\nstatus: todo\ndue: 2026-11-03\npriority: 2\n---\n\n報告を書く\n",
        ),
        (
            "fix-login.md",
            "---\nstatus: doing\ndue: 2026-10-20\npriority: 3\n---\n\nログインを直す\n",
        ),
        (
            "order-paper.md",
            "---\nstatus: done\ndue: 2026-10-01\npriority: 1\n---\n\n紙を頼む\n",
        ),
    ] {
        std::fs::write(root.join("tasks").join(f), t).unwrap();
    }
    std::fs::write(
        root.join("tasks.base"),
        "filters:\n  and:\n    - file.inFolder(\"tasks\")\nviews:\n  - type: table\n    name: Open\n    filters:\n      and:\n        - 'status != \"done\"'\n    order:\n      - file.name\n      - status\n      - due\n      - priority\n",
    )
    .unwrap();
    root
}

#[cfg(unix)]
fn command(root: &Path, program: &str) -> std::process::Command {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdgrid"));
    let bin_dir = bin.parent().unwrap().to_path_buf();
    let mut path = std::ffi::OsString::from(bin_dir.as_os_str());
    if let Some(old) = std::env::var_os("PATH") {
        path.push(":");
        path.push(old);
    }
    let mut c = std::process::Command::new(program);
    c.current_dir(root)
        .env("PATH", path)
        .env("XDG_CONFIG_HOME", root.join(".cfg"))
        .env("XDG_STATE_HOME", root.join(".state"))
        .env_remove("MDGRID_CONFIG");
    c
}

#[cfg(unix)]
#[test]
fn test_cli_22_skill_examples_run() {
    let root = fixture();

    // 見本そのものが壊れていないこと(3行のノートが読める)。
    let out = command(&root, "mdgrid")
        .args(["tasks", "--print", "--format", "csv"])
        .output()
        .expect("mdgrid を起動できません");
    assert!(
        out.status.success(),
        "見本の --print が失敗: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let csv = String::from_utf8_lossy(&out.stdout);
    let data_rows = csv.lines().skip(1).filter(|l| !l.trim().is_empty()).count();
    assert_eq!(data_rows, 3, "見本の tasks/ は3行のはず:\n{csv}");
    // 見本の tasks.base のビュー Open は done を除く2行。
    let out = command(&root, "mdgrid")
        .args(["tasks.base", "--view", "Open", "--print", "--format", "csv"])
        .output()
        .expect("mdgrid を起動できません");
    assert!(
        out.status.success(),
        "見本の tasks.base の --print が失敗: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let csv = String::from_utf8_lossy(&out.stdout);
    let data_rows = csv.lines().skip(1).filter(|l| !l.trim().is_empty()).count();
    assert_eq!(
        data_rows, 2,
        "見本の tasks.base の Open は2行のはず:\n{csv}"
    );

    let rel = "plugins/mdgrid/skills/mdgrid/SKILL.md";
    let skill = repo().join(rel);
    let md = std::fs::read_to_string(&skill)
        .unwrap_or_else(|e| panic!("{rel} が読めません({}): {e}", skill.display()));
    let blocks = sh_blocks(&md);
    assert!(!blocks.is_empty(), "{rel} に `sh` の囲みが1つもありません");

    for (i, block) in blocks.iter().enumerate() {
        let out = command(&root, "sh")
            .args(["-e", "-c", block])
            .output()
            .expect("sh を起動できません");
        assert!(
            out.status.success(),
            "{rel} の {} 番目の sh の囲みが失敗(終了 {:?}):\n--- block ---\n{block}--- stdout ---\n{}\n--- stderr ---\n{}",
            i + 1,
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }

    let active = root.join("active.md");
    let text = std::fs::read_to_string(&active)
        .unwrap_or_else(|e| panic!("例を動かしたあとに active.md がありません: {e}"));
    let table: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .collect();
    assert!(
        table.len() >= 2,
        "active.md が Markdown の表ではありません:\n{text}"
    );
    let is_sep = |l: &str| l.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) && l.contains('-');
    assert!(
        is_sep(table[1]),
        "active.md の2行目が表の区切りではありません:\n{text}"
    );
    let rows: Vec<&str> = table[2..].iter().copied().filter(|l| !is_sep(l)).collect();
    assert_eq!(rows.len(), 2, "active.md のデータの行は2つのはず:\n{text}");
    assert_eq!(
        rows.iter().filter(|r| r.contains("todo")).count(),
        1,
        "todo の行が1つのはず:\n{text}"
    );
    assert_eq!(
        rows.iter().filter(|r| r.contains("doing")).count(),
        1,
        "doing の行が1つのはず:\n{text}"
    );
    assert!(
        rows.iter().all(|r| !r.contains("done")),
        "done の行があってはいけません:\n{text}"
    );
}
