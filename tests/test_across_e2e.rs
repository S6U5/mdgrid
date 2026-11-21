//! ワークスペースを横断して探す(specs/requirements/navigation.md の navigation.across-*)を、本物の実行ファイルで
//! 確かめる。画面は疑似端末(80×24)で見る。期待値は要求の受け入れの例から取った: tasks に「見積もり」を含む行が
//! 2つ、projects に1つ → 「見積もり」と打って Enter → 「tasks」の下に2行、「projects」の下に1行。projects の
//! 「見積もりの返事」で Enter → projects の表が開き、検索の欄に「見積もり」、選んだ行が「見積もりの返事」。

use std::path::{Path, PathBuf};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> TempDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("mdgrid-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        TempDir(std::fs::canonicalize(&dir).unwrap_or(dir))
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

/// ワークスペース ws(印あり)の tasks に「見積もり」を含む行が2つと含まない行が1つ、projects に含む行が1つと
/// 含まない行が1つ。設定は `conf`。
fn home(tag: &str, conf: &str) -> TempDir {
    let h = TempDir::new(tag);
    let p = h.path();
    for d in [
        "config/mdgrid",
        "state",
        "ws/tasks",
        "ws/projects",
        "ws/.mdgrid",
    ] {
        std::fs::create_dir_all(p.join(d)).unwrap();
    }
    let w = |rel: &str, text: &str| std::fs::write(p.join(rel), text).unwrap();
    w(
        "ws/tasks/a.md",
        "---\ntitle: 見積もりを出す\nstatus: todo\n---\n",
    );
    w(
        "ws/tasks/b.md",
        "---\ntitle: 請求書\nmemo: 見積もりの数字を写す\n---\n",
    );
    w("ws/tasks/c.md", "---\ntitle: 歯医者\nstatus: done\n---\n");
    w("ws/projects/見積もりの返事.md", "---\nclient: 森\n---\n");
    w("ws/projects/看板.md", "---\nclient: 林\n---\n");
    w("ws/.mdgrid/workspace.toml", "name = \"work\"\n");
    w("config/mdgrid/config.toml", conf);
    h
}

const ON: &str = "[search]\nacross = true\n";

// ---- 画面(疑似端末の道具は Unix だけ。CI の ubuntu と macOS の test で回る) ----

#[cfg(unix)]
mod screen {
    use super::{home, ON};
    use crate::pty::Tui;

    /// 起動の直後の1打は、端末の地の色の問い合わせ(テーマの auto)に吸われることがあるので、`done` まで押し直す。
    fn press_until(t: &mut Tui, key: &str, done: impl Fn(&str) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done(&t.screen()) && std::time::Instant::now() < end {
            t.send(key);
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    /// tasks を開き、パレットの「ワークスペースを横断して探す」で「見積もり」を探して、読み終えるまで待つ。
    fn search(t: &mut Tui) {
        t.wait_for("見積もりを出す");
        press_until(t, ":", |s| {
            s.lines().nth(1).is_some_and(|l| l.contains(':')) || s.contains("\n: ")
        });
        t.send("across_search");
        t.wait_for("ワークスペースを横断");
        t.send("\r");
        t.wait_for("横断して探す:");
        t.send("見積もり\r");
        t.wait_until("結果", |s| {
            s.contains("tasks(2)") && s.contains("projects(1)")
        });
    }

    /// 画面の行(前後の空白を除く)。
    fn lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.trim().to_string()).collect()
    }

    #[test]
    fn test_across_option_hidden_without_setting() {
        // [stest -> req~navigation.across-option~1] 設定に [search] が無い → パレットに出ない。
        let h = home("across-off", "");
        let mut t = Tui::spawn(&["ws/tasks"], h.path(), &[]);
        t.wait_for("見積もりを出す");
        press_until(&mut t, ":", |s| {
            s.lines().nth(1).is_some_and(|l| l.contains(':')) || s.contains("\n: ")
        });
        t.send("across_search");
        t.wait_until("候補", |s| s.contains(": across_search"));
        let s = t.screen();
        assert!(!s.contains("ワークスペースを横断"), "{s}");
        drop(t);
    }

    #[test]
    fn test_across_results_on_screen() {
        // [stest -> req~navigation.across-results~1] 「見積もり」で Enter → 「tasks」の下に2行、「projects」の下に1行(表の並びは範囲の順)。
        let h = home("across-results", ON);
        let mut t = Tui::spawn(&["ws/tasks"], h.path(), &[]);
        search(&mut t);
        t.wait_until("全部の表", |s| {
            s.contains("tasks(2)") && s.contains("projects(1)")
        });
        let s = t.screen();
        let ls = lines(&s);
        let at = |head: &str| ls.iter().position(|l| l.starts_with(head)).unwrap();
        let (tasks, projects) = (at("tasks(2)"), at("projects(1)"));
        let under = |i: usize, n: usize| ls[i + 1..=i + n].to_vec();
        let t2 = under(tasks, 2);
        assert!(
            t2.iter().any(|l| l.contains("title: 見積もりを出す")),
            "{s}"
        );
        assert!(
            t2.iter().any(|l| l.contains("memo: 見積もりの数字を写す")),
            "{s}"
        );
        assert!(under(projects, 1)[0].contains("見積もりの返事"), "{s}");
        assert!(!s.contains("看板") && !s.contains("歯医者"), "{s}");
        drop(t);
    }

    #[test]
    fn test_across_open_on_screen() {
        // [stest -> req~navigation.across-open~1] 「見積もりの返事」で Enter → projects が開き、検索の欄に「見積もり」、その行を選ぶ。
        let h = home("across-open", ON);
        let mut t = Tui::spawn(&["ws/tasks"], h.path(), &[]);
        search(&mut t);
        // 範囲の順で projects が先なら初めから選んでいる。そうでなければ下へ。
        press_until(&mut t, "\x1b[B", |s| s.contains("> 見積もりの返事"));
        t.send("\r");
        t.wait_until("projects を開く", |s| {
            s.contains("projects") && s.contains(">見積もりの返事") && !s.contains("見積もりを出す")
        });
        let s = t.screen();
        assert!(s.contains("検索: 見積もり"), "{s}");
        assert!(s.contains(">見積もりの返事"), "{s}");
        assert!(!s.contains("看板"), "検索で絞っている: {s}");
        drop(t);
    }

    #[test]
    fn test_across_esc_on_screen() {
        // [stest -> req~navigation.across-progress~1] 結果を出したまま Esc → 元の表の画面のまま(検索の語も入らない)。
        let h = home("across-esc", ON);
        let mut t = Tui::spawn(&["ws/tasks"], h.path(), &[]);
        search(&mut t);
        t.send("\x1b");
        t.wait_until("表に戻る", |s| {
            !s.contains("projects(1)") && s.contains("歯医者")
        });
        let s = t.screen();
        assert!(s.contains("3行") && !s.contains("検索: 見積もり"), "{s}");
        drop(t);
    }
}
