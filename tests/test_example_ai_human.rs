//! 見本 examples/ai-human(人と AI のタスクを分けて見る)を開く試験。
//! 記録: specs/_changes/2026-10-07-example-ai-human.md のタスク 1。
//!
//! 見本は一時フォルダに写してから開く(リポの examples の中は書かない)。
//! 分け方は cellops と同じ: `actor: human` か、`human_gate` が none 以外なら人。
//! それ以外(`actor` が無いノートも)は AI。

use std::path::{Path, PathBuf};
use std::process::Command;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        // 時計の刻みが粗い(macOS はマイクロ秒)と、並んで動く試験が同じ名前になるので、通し番号も足す。
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mdgrid-test-{}-{}-{}-{}",
            name,
            std::process::id(),
            nanos,
            seq
        ));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dst = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), &dst).unwrap();
        }
    }
}

/// 見本を写し、`.base` の view を `--print --format csv --with-path` で出す。行ごとの列のリストを返す
/// (見出しの行は除く。先頭の列はノートのパス)。
fn print_view(view: &str) -> Vec<Vec<String>> {
    let tmp = TempDir::new("ai-human");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/ai-human");
    copy_dir(&root, tmp.path());
    let out = Command::new(env!("CARGO_BIN_EXE_mdgrid"))
        .arg(tmp.path().join("vault").join("タスク.base"))
        .args(["--print", "--format", "csv", "--with-path", "--view", view])
        .env("XDG_CONFIG_HOME", tmp.path().join("config"))
        .env("XDG_STATE_HOME", tmp.path().join("state"))
        .env("LANG", "ja_JP.UTF-8")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "--print --view {view} が失敗した: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // 見本の値には , と " が無いので、素朴に分ける。
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| l.split(',').map(str::to_string).collect())
        .collect()
}

/// パスの列からノートの名前(`.md` を除いたファイル名)を取る。
fn note_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn names(rows: &[Vec<String>]) -> Vec<String> {
    rows.iter().map(|r| note_name(&r[0])).collect()
}

const HUMAN: [&str; 5] = [
    "サーバー代を払う",
    "業務委託の契約書に署名する",
    "確定申告の書類を出す",
    "区役所で住民票を取る",
    "取引先にお礼のメールを送る",
];

const AI: [&str; 4] = [
    "会議の議事録を要約する",
    "依存ライブラリを更新する",
    "テストの抜けを洗い出す",
    "README の英訳を直す",
];

#[test]
fn test_example_ai_human_groups_people_first() {
    // 終わっていないタスクが、人のまとまり(優先度 → 期限の順)、AI のまとまりの順に並ぶ。
    let rows = print_view("人とAI");
    let got = names(&rows);
    let want: Vec<String> = HUMAN
        .iter()
        .chain(AI.iter())
        .map(|s| s.to_string())
        .collect();
    assert_eq!(got, want);
}

#[test]
fn test_example_ai_human_split_views() {
    let mut human = names(&print_view("人だけ"));
    let mut ai = names(&print_view("AIだけ"));
    human.sort();
    ai.sort();
    let mut want_h: Vec<String> = HUMAN.iter().map(|s| s.to_string()).collect();
    let mut want_a: Vec<String> = AI.iter().map(|s| s.to_string()).collect();
    want_h.sort();
    want_a.sort();
    // human_gate が none 以外(pay・sign・submit・send)か actor: human なら人。
    assert_eq!(human, want_h);
    // actor の無い「README の英訳を直す」は AI に入る。
    assert_eq!(ai, want_a);
}

#[test]
fn test_example_ai_human_done_keeps_the_split() {
    // 終わったタスクにも担当の式が効く(human_gate: send の請求書は人)。
    let rows = print_view("終わった");
    let mut got: Vec<(String, String)> = rows
        .iter()
        .map(|r| (note_name(&r[0]), r[1].clone()))
        .collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            (
                "古いブランチを消す".to_string(),
                "🤖 エージェントのタスク".to_string()
            ),
            (
                "請求書を発行する".to_string(),
                "👤 人間のタスク".to_string()
            ),
        ]
    );
}
